// C15.1 year-end runtime capture harness for cm0102_GDI.exe.
//
// Hooks the year-end call chain and dumps every observed
// club / stadium / person / news / RNG-state transition to
// stdout as one JSONL line per event. The Python driver
// (year_end_capture.py) launches Frida with this script attached
// to a running GDI process, ticks the game past a year boundary,
// then post-processes the JSONL into a differential-ready
// snapshot.
//
// GDI VAs — pin these to cm0102_GDI.exe (NOT the DirectDraw
// build). The DirectDraw offsets differ non-uniformly (see memory
// [[gdi-vs-directdraw-builds]]).
//
// Hook targets:
//   0x00669FA0  FUN_00669FA0  position→status stamper
//   0x0066EED0  FUN_0066EED0  P/R swap primitive (inner)
//   0x00668380  FUN_00668380  promotion install
//   0x00668470  FUN_00668470  relegation install
//   0x004D3550  FUN_004D3550  promotion per-person walk
//   0x004D3460  FUN_004D3460  relegation per-person walk
//   0x00583FC0  FUN_00583FC0  stadium expansion transaction
//   0x0055EC00  sub_0055EC00 Conference peer coordinator
//   0x0055EE90  FUN_0055EE90 pyramid orchestrator
//   0x0055EA00  FUN_0055EA00 Conference inactive fallback
//   0x004938D0  FUN_004938D0 news publisher (year-end channel)
//
// RNG state — 4-byte LCG at:
//   0x00AC2610  GDI RNG state (per memory [[perturb-reorders-clubs]])
//
// Every hook logs entry with the club id (`+0x00`), current
// `+0x37`, `+0x57`, `+0x5B` bytes, plus a monotonically
// increasing sequence id.

'use strict';

const MODULE = 'cm0102_GDI.exe';
let SEQ = 0;

function baseVA() {
    const m = Process.getModuleByName(MODULE);
    return m.base;
}

function readClub(clubPtr) {
    if (clubPtr.isNull()) return null;
    try {
        return {
            id:       clubPtr.add(0x00).readU32(),
            status_37: clubPtr.add(0x37).readU8(),
            primary_57: clubPtr.add(0x57).readS32(),
            previous_5b: clubPtr.add(0x5b).readS32(),
            tier_64: clubPtr.add(0x64).readU8(),
            stadium_69: clubPtr.add(0x69).readU8(),
            news_flag_cf: clubPtr.add(0xcf).readU8(),
        };
    } catch (e) { return { err: String(e) }; }
}

function readStadium(stadPtr) {
    if (stadPtr.isNull()) return null;
    try {
        return {
            id:      stadPtr.add(0x00).readU32(),
            refuse_20: stadPtr.add(0x20).readS8(),
            total_3c: stadPtr.add(0x3c).readU32(),
            seated_40: stadPtr.add(0x40).readU32(),
            peak_44: stadPtr.add(0x44).readU32(),
            alt_48: stadPtr.add(0x48).readS32(),
        };
    } catch (e) { return { err: String(e) }; }
}

function rngState() {
    try {
        return Process.getModuleByName(MODULE)
            .base.add(0x00AC2610 - 0x00400000).readU32();
    } catch (e) { return null; }
}

function emit(record) {
    record.seq = SEQ++;
    record.rng = rngState();
    send(JSON.stringify(record));
}

function hookAt(offset, name, argShape) {
    const va = baseVA().add(offset - 0x00400000);
    Interceptor.attach(va, {
        onEnter(args) {
            const rec = { hook: name, phase: 'enter', va: offset };
            if (argShape === 'club_first') {
                rec.club_pre = readClub(args[0]);
            } else if (argShape === 'club_promotion') {
                rec.club_pre = readClub(args[0]);
                rec.new_comp_ptr = args[1].toInt32();
                rec.news_enable = args[2].toInt32();
            } else if (argShape === 'stadium_expansion') {
                rec.club_pre = readClub(args[0]);
                rec.desired_seated = args[1].toInt32();
                rec.desired_total = args[2].toInt32();
                rec.news_ctx = args[3].toInt32();
                rec.affordability_checked = args[4].toInt32();
                // The stadium ptr resolves through Club+0x69, but we
                // grab it here for convenience.
                try {
                    rec.stadium_pre = readStadium(
                        args[0].add(0x69).readPointer());
                } catch (e) {}
            } else if (argShape === 'news') {
                rec.template_id = args[0].toInt32();
                rec.comp_id = args[1].toInt32();
                rec.news_ctx = args[2].toInt32();
                rec.mode = args[3].toInt32();
            }
            this._rec = rec;
            emit(rec);
        },
        onLeave(retval) {
            const post = { hook: name, phase: 'leave', va: offset,
                           retval: retval.toInt32() };
            if (this._rec && this._rec.club_pre &&
                this._rec.club_pre.id !== undefined) {
                // No easy way to re-derive club ptr without saving
                // it — skip post-state for now; the pre-state on the
                // NEXT hook shows what changed.
            }
            emit(post);
        },
    });
}

function main() {
    hookAt(0x00669FA0, 'FUN_00669FA0_status_stamper', 'club_first');
    hookAt(0x0066EED0, 'FUN_0066EED0_pr_swap', 'club_first');
    hookAt(0x00668380, 'FUN_00668380_promotion_install', 'club_promotion');
    hookAt(0x00668470, 'FUN_00668470_relegation_install', 'club_promotion');
    hookAt(0x004D3550, 'FUN_004D3550_promotion_walk', 'club_first');
    hookAt(0x004D3460, 'FUN_004D3460_relegation_walk', 'club_first');
    hookAt(0x00583FC0, 'FUN_00583FC0_stadium_expansion', 'stadium_expansion');
    hookAt(0x0055EC00, 'sub_0055EC00_conf_coordinator', 'club_first');
    hookAt(0x0055EE90, 'FUN_0055EE90_pyramid_orchestrator', 'club_first');
    hookAt(0x0055EA00, 'FUN_0055EA00_conf_fallback', 'club_first');
    hookAt(0x004938D0, 'FUN_004938D0_news_publisher', 'news');
    emit({ hook: '__init__', rng_state_start: rngState() });
}

main();
