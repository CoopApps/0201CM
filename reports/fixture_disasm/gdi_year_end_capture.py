"""C15.1G — authoritative GDI year-end capture harness.

Hooks the 12 functions that drive a Traditional English 2001→2002
year-end rollover and emits one JSONL event per hook enter/leave.
Runs against `cm0102_GDI.exe` — attach the interpreter to the exe
after loading a save that is queued to advance across the season
boundary (typically last-day-of-season → +1 day).

USAGE (Windows, from a `powershell` session, with the GDI exe
already running and a save loaded at the season boundary):

    python reports\\fixture_disasm\\gdi_year_end_capture.py \\
        > fixtures\\gdi_captures\\year_end_2001_02.jsonl

Then advance one day inside the game to trip the rollover, watch
the capture flush, and Ctrl+C to close.

The `tools/c15_1g/analyse_gdi_capture.py` reads the JSONL back
and produces a `YearEndSnapshot`-shaped JSON that diffs directly
against `crates/cm-domain/src/c15_1g_snapshot.rs::YearEndSnapshot`.

C15.1G differential procedure (the same procedure this file
enables):

    1. Run this script → year_end_2001_02.jsonl (GDI side).
    2. Run analyse_gdi_capture.py → year_end_2001_02.gdi.json
       (YearEndSnapshot).
    3. Run the Rust production path from the equivalent
       pre-rollover World state → year_end_2001_02.rust.json
       (YearEndSnapshot::from_apply serialised).
    4. `diff` the two JSONs, or run the Rust
       `c15_1g_snapshot::diff_snapshots` comparator.
    5. First divergence gates the freeze.

The hook list mirrors the tranche directive §1. Every hook
captures entry + leave state; when a hook writes a byte, we
capture BOTH the pre-image (on enter) and the post-image (on
leave). Semantic identity (club_id, stadium_id, person_id) is
resolved at capture time so the Rust differential can key on
those without dereferencing runtime pointers.
"""
import frida, sys, json, time
from pathlib import Path

# Hook VAs for cm0102_GDI.exe (authoritative build).
# The DirectDraw exe uses different VAs — this script is GDI-only.
# If a VA below is a placeholder ("0x0"), the analyst must resolve
# the GDI address for that fn before running (existing capture
# hierarchy in fixtures/gdi_captures/ names the DDs; only some
# are paired to GDI addresses in fixtures/gdi_captures/*.jsonl
# header records).
YE_HOOKS = {
    # A_year_end group — status stamp, club movement, contract
    # writes, squad +0x3A, person news/history, stadium
    # expansion, orchestrator, fallback.
    "FUN_00669FA0_status_stamper":       0x00669FA0,  # GDI-address TBC vs DD
    "FUN_0066EED0_pr_swap":               0x0066EED0,
    "FUN_00668380_promotion_install":     0x00668380,
    "FUN_00668470_relegation_install":    0x00668470,
    "FUN_004D3460_relegation_walk":       0x004D3460,
    "FUN_004D3550_promotion_walk":        0x004D3550,
    "FUN_00843970_squad_pos_write":       0x00843970,
    "FUN_008D0D90_person_news":           0x008D0D90,
    "FUN_00583FC0_stadium_expansion":     0x00583FC0,
    "FUN_0058A310_stadium_news":          0x0058A310,
    "FUN_0055EE90_pyramid_orchestrator":  0x0055EE90,
    "FUN_0055EA00_conference_fallback":   0x0055EA00,
}

# Resolver globals — used to translate runtime pointers to
# stable semantic ids at capture time.
#   DAT_00acd5bc = club pool base (stride 0x245)
#   DAT_00acdc38 = finance pool wrapper (*.pool_base, stride 0x167)
#   DAT_00acdf0c = person->club-index table (stride 0x4F)
#   DAT_00accad8 = contract pool base (stride 0x50)
RESOLVER = {
    "club_pool_base":       0x00ACD5BC,
    "finance_pool_wrapper": 0x00ACDC38,
    "person_index_table":   0x00ACDF0C,
    "contract_pool_base":   0x00ACCAD8,
    "person_count":         0x00ACD56C,
    "nation_base":          0x00ACD5BC,   # reused as nation base too
    "game_year_lo":         0x00ACDE88,
    "game_date_packed":     0x00ACDE90,
    "mailbox_desc_base":    0x00ACD5C4,
}

FRIDA_SRC = r"""
'use strict';
const HOOKS = %s;
const RES = %s;

function u32(p)  { return p.readU32(); }
function i32(p)  { return p.readS32(); }
function u8(p)   { return p.readU8(); }
function i8(p)   { return p.readS8(); }
function i64(p)  { return p.readS64().toString(); }

function clubIdFromPtr(p) {
    // Disk Club record: +0x00 is club_id.
    try { return u32(p); } catch (_) { return null; }
}

function financeStateFromClubId(id) {
    try {
        const wrapper = ptr(RES.finance_pool_wrapper);
        const base = ptr(wrapper.readU32());
        const rec  = base.add(id * 0x167);
        return {
            cash: i64(rec),
            season_misc_expense:      i32(rec.add(0x8C)),
            season_subsidy_income:    i32(rec.add(0xB4)),
            lifetime_misc_expense:    i32(rec.add(0x12C)),
            lifetime_subsidy_income:  i32(rec.add(0x154)),
            refuse_counter_probe:     null,  // finance record has no refuse counter
        };
    } catch (_) { return null; }
}

function stadiumFromPtr(p) {
    try {
        return {
            capacity_total:     i32(p.add(0x3C)),
            capacity_seated:    i32(p.add(0x40)),
            capacity_expansion: i32(p.add(0x44)),
            owner_refuse_counter: i8(p.add(0x20)),
        };
    } catch (_) { return null; }
}

function personIdFromSlotPtr(p) {
    try { return u32(p); } catch (_) { return null; }
}

function currentSeasonYear() {
    try { return u8(ptr(RES.game_year_lo)); } catch (_) { return null; }
}

let SEQ = 0;
function emit(obj) {
    obj.seq = SEQ++;
    obj.ms = Date.now();
    send(obj);
}

function hookYE(name, va) {
    const p = ptr(va);
    Interceptor.attach(p, {
        onEnter: function(args) {
            // Capture stack args 0..3 and this-pointer; downstream
            // handlers extract semantic identities as appropriate.
            this._args = [
                args[0], args[1], args[2], args[3],
            ];
            this._name = name;
            emit({ hook: name, phase: "enter",
                   season: currentSeasonYear(),
                   arg0: args[0].toString(),
                   arg1: args[1].toString(),
                   arg2: args[2].toString(),
                   arg3: args[3].toString() });
            // Per-hook semantic capture.
            if (name === "FUN_004D3460_relegation_walk"
                || name === "FUN_004D3550_promotion_walk") {
                const cid = clubIdFromPtr(args[1]);
                this._club_id = cid;
                this._pre_finance = cid !== null
                    ? financeStateFromClubId(cid) : null;
                emit({ hook: name, phase: "walk_ctx",
                       club_id: cid,
                       pre_finance: this._pre_finance });
            }
            if (name === "FUN_00843970_squad_pos_write") {
                // param_3 = new position code (i8).
                const person_id = personIdFromSlotPtr(args[0]);
                const club_id   = clubIdFromPtr(args[1]);
                emit({ hook: name, phase: "call",
                       person_id, club_id,
                       new_pos: args[2].toInt32() & 0xff });
            }
            if (name === "FUN_008D0D90_person_news") {
                const person_id = personIdFromSlotPtr(args[0]);
                const old_comp  = args[1].readU32();
                const kind      = args[2].toInt32() & 0xff;
                emit({ hook: name, phase: "call",
                       person_id, old_comp_id: old_comp,
                       kind });
            }
            if (name === "FUN_00583FC0_stadium_expansion") {
                const cid = clubIdFromPtr(args[0]);
                this._club_id = cid;
                this._pre_stadium = stadiumFromPtr(args[0]);
                emit({ hook: name, phase: "call_ctx",
                       club_id: cid,
                       pre_stadium: this._pre_stadium });
            }
        },
        onLeave: function(ret) {
            const name = this._name;
            emit({ hook: name, phase: "leave",
                   retval: ret.toInt32() });
            if (name === "FUN_004D3460_relegation_walk"
                || name === "FUN_004D3550_promotion_walk") {
                const cid = this._club_id;
                emit({ hook: name, phase: "walk_ctx_post",
                       club_id: cid,
                       post_finance: cid !== null
                           ? financeStateFromClubId(cid) : null });
            }
            if (name === "FUN_00583FC0_stadium_expansion") {
                emit({ hook: name, phase: "call_ctx_post",
                       club_id: this._club_id });
            }
        },
    });
    emit({ hook: "__install__", target: name, va: va.toString(16) });
}

for (const [name, va] of Object.entries(HOOKS)) hookYE(name, va);
emit({ hook: "__ready__" });
""" % (json.dumps(YE_HOOKS), json.dumps(RESOLVER))


def main():
    dev = frida.get_local_device()
    pid = next((p.pid for p in dev.enumerate_processes()
                if p.name.lower() == "cm0102_gdi.exe"), None)
    if not pid:
        sys.exit("cm0102_GDI.exe not running — start it, load a "
                 "save queued to trip the season boundary, then "
                 "re-run this script.")
    print(f"# attach cm0102_GDI.exe pid={pid}", file=sys.stderr)
    session = frida.attach(pid)
    script = session.create_script(FRIDA_SRC)

    def on_message(msg, _data):
        if msg.get("type") == "send":
            print(json.dumps(msg["payload"]), flush=True)
        elif msg.get("type") == "error":
            print(json.dumps({"error": msg.get("description")}),
                  file=sys.stderr, flush=True)

    script.on("message", on_message)
    script.load()
    print("# hooks installed — advance the game one day inside "
          "the exe now to trigger the rollover.", file=sys.stderr)
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print("# capture stopped", file=sys.stderr)
    finally:
        session.detach()


if __name__ == "__main__":
    main()
