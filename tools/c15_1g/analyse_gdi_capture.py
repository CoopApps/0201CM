"""C15.1G — reduce a raw Frida capture JSONL into a
`YearEndSnapshot`-shaped JSON, ready to diff against the Rust
production path's snapshot.

The Rust wire format is `crates/cm-domain/src/c15_1g_snapshot.rs`
`YearEndSnapshot`. This analyser emits the same schema so
`serde_json` will deserialise it into that struct and
`diff_snapshots` can compare directly.

USAGE:

    python tools/c15_1g/analyse_gdi_capture.py \\
        fixtures/gdi_captures/year_end_2001_02.jsonl \\
        > fixtures/gdi_captures/year_end_2001_02.gdi.json

Emits a single-line JSON document with these keys:

    { year, events, post_rollover_club_status, finance,
      person_mailboxes, stadiums }

`events` is an ordered list of `{event, ...}` records — the
tagged form the Rust enum's serde derive produces.
"""
import argparse, json, sys
from collections import defaultdict
from pathlib import Path


def analyse(path):
    events = []
    finance = {}                # club_id -> {cash, s_expense, l_expense, s_sub, l_sub}
    finance_pre = {}            # walk-ctx pre snapshots
    person_mailboxes = defaultdict(list)
    stadiums = {}
    post_rollover_status = {}
    year = 0

    for line in Path(path).read_text().splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        try:
            rec = json.loads(line)
        except json.JSONDecodeError:
            continue
        if isinstance(rec.get("season"), int):
            year = int(rec["season"]) or year
        hook = rec.get("hook", "")
        phase = rec.get("phase", "")

        # C15.1B / C15.1E — relegation walk captures pre/post
        # finance for the walked club; contract 1→2 writes and
        # news appends fire between the two ctx events (implicit,
        # captured via the FUN_008D0D90 hook).
        if hook == "FUN_004D3460_relegation_walk" and phase == "walk_ctx":
            cid = rec.get("club_id")
            pf = rec.get("pre_finance")
            if cid is not None and pf is not None:
                finance_pre[cid] = pf
        if hook == "FUN_004D3460_relegation_walk" and phase == "walk_ctx_post":
            cid = rec.get("club_id")
            pf = rec.get("post_finance")
            if cid is not None and pf is not None:
                finance[cid] = pf
                # Emit a FinanceWrite event if the state changed.
                pre = finance_pre.get(cid)
                if pre and pre != pf:
                    events.append({
                        "event": "FinanceWrite",
                        "club_id": cid,
                        "new_cash": int(pf["cash"]),
                        "new_season_misc_expense": pf["season_misc_expense"],
                        "new_lifetime_misc_expense": pf["lifetime_misc_expense"],
                        "new_season_subsidy_income": pf["season_subsidy_income"],
                        "new_lifetime_subsidy_income": pf["lifetime_subsidy_income"],
                    })

        # C15.1E — person history append.
        if hook == "FUN_008D0D90_person_news" and phase == "call":
            pid = rec.get("person_id")
            k   = rec.get("kind", 0) & 0xff
            oc  = rec.get("old_comp_id", 0)
            entry = {
                "category": 0x0FBF,
                "kind": k,
                "old_comp_id": oc,
                "staff_id": 0,     # not observable at this hook depth
                "year": year,
                "news_id": len(person_mailboxes.get(pid, [])),
            }
            if pid is not None:
                person_mailboxes[pid].append(entry)
                events.append({
                    "event": "PersonHistoryAppend",
                    "person_id": pid,
                    "category": 0x0FBF,
                    "kind": k,
                    "old_comp_id": oc,
                    "staff_id": 0,
                    "news_id": entry["news_id"],
                })

        # C15.1C — squad +0x3A write.
        if hook == "FUN_00843970_squad_pos_write" and phase == "call":
            pid = rec.get("person_id")
            new_pos = rec.get("new_pos", 0)
            # Sign-extend the byte to i8.
            if new_pos >= 0x80: new_pos -= 0x100
            events.append({
                "event": "SquadPositionWrite",
                "person_id": pid,
                "record_slot": "Primary",
                "old_value": 0,
                "new_value": new_pos,
            })

        # C15.1A/D — stadium expansion pre/post.
        if hook == "FUN_00583FC0_stadium_expansion" and phase == "call_ctx":
            cid = rec.get("club_id")
            ps = rec.get("pre_stadium")
            if cid is not None and ps is not None:
                stadiums[cid] = ps  # pre-image; overwritten by post if seen

        # C15.1B — contract clause writes are implicit in the
        # walk (no dedicated hook here; add FUN_00843970-style
        # ContractClauseWrite hook if the differential needs
        # byte-level ordering, since the exe writes +0x1F BEFORE
        # calling FUN_008D0D90 — cf. reports/c15_1b_staff_archaeology.md).

    return {
        "year": year,
        "events": events,
        "post_rollover_club_status": post_rollover_status,
        "finance": finance,
        "person_mailboxes": {
            str(k): v for k, v in person_mailboxes.items()
        },
        "stadiums": stadiums,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("jsonl", help="raw Frida capture path")
    args = ap.parse_args()
    snap = analyse(args.jsonl)
    json.dump(snap, sys.stdout, indent=None, sort_keys=True)
    sys.stdout.write("\n")


if __name__ == "__main__":
    main()
