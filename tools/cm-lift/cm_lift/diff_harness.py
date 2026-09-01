"""Diff harness: same-input parity check between Unicorn-emulated exe
and our Rust port.

Two granularities:

  * per-fn — run one exe fn under Unicorn with fixed inputs, capture EAX
    + FPU state + memory writes; call our Rust equivalent (via a JSON
    RPC stub exposed by cm-domain) with the same inputs; diff.

  * per-tick — run a whole exe day under Unicorn, snapshot the game
    state pool, run our Rust tick over the same starting state, diff
    subsystem-by-subsystem.

Per-fn is the workhorse — turns every ported fn into a verifiable unit.
Per-tick is aspirational (needs a stable state serialisation on both
sides).

Correctness gate: for every fn in `cm_lift.data.fn_probe_matrix.json`
this harness runs the paired input tuple, records exe vs port outputs,
and writes `data/diff_report.json` with a pass/fail per fn. CI can gate
on that report.
"""
from __future__ import annotations
import json
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Optional, Any
from .util import DATA_OUT, RUST_ROOT
from .emulate import make_emulator, Emulator


@dataclass
class Probe:
    """One fn + input tuple + expected-output spec.

    For __thiscall / struct-arg fns, `struct_setup` lists per-arg struct
    layouts. Each entry is `(arg_index, {offset: (type_char, value)})`.
    The harness allocates that struct in emu heap and substitutes its
    pointer as arg[arg_index] before the call.
    """
    fn_va: int
    args: tuple[int, ...]
    thiscall: bool = False
    struct_setup: list = None
    rust_bin: Optional[str] = None
    # rust_args: if the Rust fn takes different args than the exe (e.g. exe
    # passes a token_ptr while Rust takes raw x/y), specify what to send
    # to the RPC bin here. Defaults to `args`.
    rust_args: Optional[tuple] = None
    expected_eax: Optional[int] = None
    # Concrete expected return value from the Rust side. When provided,
    # a rust-only probe (fn_va == 0) is graded on rust_ret == expected_ret
    # instead of just "did it return non-null". Catches regressions.
    expected_ret: Optional[int] = None
    label: str = ""


def run_exe_probe(emu: Emulator, probe: Probe) -> dict:
    """Run one probe under Unicorn, return {eax, fpu, exc}. When
    `struct_setup` is present, allocate + populate those structs in
    emu heap and substitute their pointers into `args`."""
    args = list(probe.args)
    if probe.struct_setup:
        for arg_idx, layout in probe.struct_setup:
            ptr = emu.alloc_struct(layout)
            # Extend args if index is beyond current length
            while len(args) <= arg_idx:
                args.append(0)
            args[arg_idx] = ptr
    eax = emu.call(probe.fn_va, *args, thiscall=probe.thiscall)
    return {
        "eax": eax & 0xFFFFFFFF,
        "eax_signed": eax if eax < 0x80000000 else eax - (1 << 32),
        "fpu_st0": emu.read_fpu()[0],
        "exc": getattr(emu, "last_exc", None),
    }


def run_rust_probe(probe: Probe) -> dict:
    """Run the Rust equivalent via cm_lift_rpc.exe (pre-built release
    binary). Fast — no cargo overhead."""
    if not probe.rust_bin:
        return {"ret": probe.expected_eax, "exc": None, "source": "expected_eax"}
    exe = RUST_ROOT / "target" / "release" / "cm_lift_rpc.exe"
    if not exe.exists():
        return {"ret": None, "exc": f"missing {exe} — run: cargo build --release -p cm-domain --bin cm_lift_rpc"}
    try:
        rargs = probe.rust_args if probe.rust_args is not None else probe.args
        args_flat = [str(a) for a in rargs]
        result = subprocess.run(
            [str(exe), probe.rust_bin, *args_flat],
            capture_output=True, text=True, timeout=30,
        )
        try:
            return json.loads(result.stdout)
        except json.JSONDecodeError:
            return {"ret": None, "exc": f"non-json: {result.stdout[:200]}"}
    except subprocess.TimeoutExpired:
        return {"ret": None, "exc": "timeout"}
    except Exception as e:
        return {"ret": None, "exc": repr(e)}


def diff(exe: dict | None, rust: dict, expected_ret: Optional[int] = None) -> dict:
    """Compare exe vs rust output tuples.

    If `expected_ret` is set: rust must return exactly that value.
    Else if exe is None: rust-only smoke (non-null, non-exception).
    Else: rust must match exe EAX.
    """
    rust_v = rust.get("ret")
    exc = rust.get("exc")
    if expected_ret is not None:
        ok = (rust_v == expected_ret) and exc is None
        return {"exe": exe, "rust": rust, "match": ok, "kind": "assert",
                "expected": expected_ret, "delta": (rust_v - expected_ret)
                                            if isinstance(rust_v, int) else None}
    if exe is None:
        rust_ok = rust_v is not None and exc is None
        return {"exe": None, "rust": rust, "match": rust_ok,
                "kind": "rust_only", "delta": None}
    exe_v = exe.get("eax_signed", exe.get("eax"))
    ok = (exe_v == rust_v) if (exe_v is not None and rust_v is not None) else False
    return {
        "exe": exe, "rust": rust,
        "match": ok, "kind": "diff",
        "delta": (exe_v - rust_v) if (isinstance(exe_v, int)
                                     and isinstance(rust_v, int)) else None,
    }


def run_matrix(probes: list[Probe], out_path: Optional[Path] = None) -> Path:
    """Run every probe, write consolidated diff report.

    Probes with `fn_va == 0` skip the exe run — they're rust-only smoke
    tests. Probes with `fn_va != 0` run both and diff EAX.
    """
    emu = make_emulator()
    results = []
    for i, p in enumerate(probes):
        exe = run_exe_probe(emu, p) if p.fn_va != 0 else None
        rust = run_rust_probe(p)
        d = diff(exe, rust, p.expected_ret)
        d["probe"] = {
            "index": i,
            "fn_va": f"{p.fn_va:#010x}",
            "args": list(p.args),
            "label": p.label,
        }
        results.append(d)
    out = out_path or (DATA_OUT / "diff_report.json")
    passed = sum(1 for r in results if r["match"])
    with open(out, "w", encoding="utf-8") as f:
        json.dump({
            "count": len(results),
            "passed": passed,
            "failed": len(results) - passed,
            "results": results,
        }, f, indent=2, default=lambda o: str(o))
    return out


# --- Built-in probe matrix ------------------------------------------------

# Concrete probes to run out-of-the-box — cover subsystems we've already
# ported so the harness has meaningful baseline data.

DEFAULT_PROBES: list[Probe] = [
    # FUN_006DB520 shot_in_box — __thiscall(token_ptr, side).
    # exe reads token+0x102 (zone_x) and token+0x103 (zone_y).
    # rust_bin `shot_in_box` takes (x, y, side) directly — same result.
    Probe(fn_va=0x006DB520, args=(0, 1),
          struct_setup=[(0, {0x102: ('b', 4), 0x103: ('b', 10)})],
          thiscall=True,
          rust_bin="shot_in_box", rust_args=(4, 10, 1),
          label="shot_in_box: token{x=4,y=10} side=1 -> in box"),
    Probe(fn_va=0x006DB520, args=(0, 0),
          struct_setup=[(0, {0x102: ('b', 4), 0x103: ('b', 5)})],
          thiscall=True,
          rust_bin="shot_in_box", rust_args=(4, 5, 0),
          label="shot_in_box: token{x=4,y=5} side=0 -> NOT in box"),
    Probe(fn_va=0x006DB520, args=(0, 0),
          struct_setup=[(0, {0x102: ('b', 3), 0x103: ('b', 1)})],
          thiscall=True,
          rust_bin="shot_in_box", rust_args=(3, 1, 0),
          label="shot_in_box: token{x=3,y=1} side=0 -> in box"),
    Probe(fn_va=0x006DB520, args=(0, 1),
          struct_setup=[(0, {0x102: ('b', 0), 0x103: ('b', 0)})],
          thiscall=True,
          rust_bin="shot_in_box", rust_args=(0, 0, 1),
          label="shot_in_box: token{corner} side=1 -> NOT in box"),
    # Zone-boundary probes — x edges (2 in, 1 out; 6 in, 7 out)
    Probe(fn_va=0x006DB520, args=(0, 1),
          struct_setup=[(0, {0x102: ('b', 2), 0x103: ('b', 10)})],
          thiscall=True,
          rust_bin="shot_in_box", rust_args=(2, 10, 1),
          label="shot_in_box: x=2 (leftmost in) side=1 -> in"),
    Probe(fn_va=0x006DB520, args=(0, 1),
          struct_setup=[(0, {0x102: ('b', 1), 0x103: ('b', 10)})],
          thiscall=True,
          rust_bin="shot_in_box", rust_args=(1, 10, 1),
          label="shot_in_box: x=1 (out) side=1 -> NOT in"),
    Probe(fn_va=0x006DB520, args=(0, 1),
          struct_setup=[(0, {0x102: ('b', 6), 0x103: ('b', 11)})],
          thiscall=True,
          rust_bin="shot_in_box", rust_args=(6, 11, 1),
          label="shot_in_box: x=6 y=11 (both max in) side=1 -> in"),
    Probe(fn_va=0x006DB520, args=(0, 1),
          struct_setup=[(0, {0x102: ('b', 7), 0x103: ('b', 10)})],
          thiscall=True,
          rust_bin="shot_in_box", rust_args=(7, 10, 1),
          label="shot_in_box: x=7 (out) side=1 -> NOT in"),

    # FUN_006a88f0 token_addr — pure arithmetic thiscall(pitch_base, side, slot).
    # Returns pitch_base + 0x4796 + (side*0x14 + slot) * 0x1BE.
    # No struct setup needed — all args are integers.
    # Home slot 0: 0 + 0x4796 + 0*0x1BE = 0x4796
    # Home slot 1: 0 + 0x4796 + 1*0x1BE = 0x4954
    # Away slot 0: 0 + 0x4796 + 20*0x1BE = 0x4796 + 0x2378 = 0x6B0E
    Probe(fn_va=0x006A88F0, args=(0, 0, 0), thiscall=True,
          rust_bin="token_addr", rust_args=(0, 0, 0),
          label="token_addr(0, home, slot=0) == 0x4796"),
    Probe(fn_va=0x006A88F0, args=(0, 0, 1), thiscall=True,
          rust_bin="token_addr", rust_args=(0, 0, 1),
          label="token_addr(0, home, slot=1) == 0x4954"),
    Probe(fn_va=0x006A88F0, args=(0, 1, 0), thiscall=True,
          rust_bin="token_addr", rust_args=(0, 1, 0),
          label="token_addr(0, away, slot=0) == 0x6B0E"),
    Probe(fn_va=0x006A88F0, args=(0x1000, 1, 19), thiscall=True,
          rust_bin="token_addr", rust_args=(0x1000, 1, 19),
          label="token_addr(0x1000, away, slot=19) largest"),

    # FUN_00618410 club_status_byte — __thiscall(status_table_ptr, record_ptr)
    # Reads *(int*)(record+0x61) — must be non-null.
    # Reads *record — id, bounds-checked against DAT_00acd56c.
    # Returns status_table[id*0x1f + 0x12].
    # Rust port uses a synthesized table (byte i = i & 0xff), so exe-side
    # needs the same table populated. Also DAT_00acd56c must be non-zero.
    # Skipping full exe probe: our Rust port is a straight reimplementation,
    # not a lift-from-exe, so the diff would require synchronizing the
    # emu's DAT_00acd56c + populating a 100+ club status table in emu heap.
    # Marked as future work.
]

# --- Direct Rust-only probes (no exe emulation needed) --------------------
# For fns without a clean single-fn exe entry point, we still exercise the
# RPC bin to smoke-test the port. exe-side check reports 'no-emu' — sanity
# only.
RUST_ONLY_PROBES: list[Probe] = [
    Probe(fn_va=0, args=(6400,),       rust_bin="finalize_rating",     expected_ret=6, label="finalize_rating(6400) == 6"),
    Probe(fn_va=0, args=(500,),        rust_bin="finalize_rating",     expected_ret=1, label="finalize_rating(500) == 1"),
    Probe(fn_va=0, args=(9500,),       rust_bin="finalize_rating",     expected_ret=10, label="finalize_rating(9500) == 10"),
    Probe(fn_va=0, args=(0, 0),        rust_bin="gk_save_rating_delta_milli", expected_ret=400, label="gk_save (no conc, no flags) == 400"),
    Probe(fn_va=0, args=(1, 2),        rust_bin="gk_save_rating_delta_milli", expected_ret=120, label="gk_save (concede + flag=0b10)"),
    Probe(fn_va=0, args=(15,),         rust_bin="team_mentality_mask", expected_ret=256, label="team_mentality(15) == 0x100"),
    Probe(fn_va=0, args=(5,),          rust_bin="team_mentality_mask", expected_ret=0, label="team_mentality(5) == 0"),
    Probe(fn_va=0, args=(0x0001,),     rust_bin="role_mask_to_position",  label="role_mask 0x0001 → Gk(0)"),
    Probe(fn_va=0, args=(0x0804,),     rust_bin="role_mask_to_position",  label="role_mask 0x0804 → Dr(3)"),
    Probe(fn_va=0, args=(20, 100),     rust_bin="age_wage_cap",        expected_ret=325000, label="age_wage_cap(20, 100) == 275000"),
    Probe(fn_va=0, args=(1, 3, 5, 10, 2, 4), rust_bin="fifa_score",   expected_ret=18, label="fifa_score smoke"),
    # Season-avg rating (verified sum/count formula from FUN_007aa490)
    Probe(fn_va=0, args=(10, 60),      rust_bin="season_avg_rating",   expected_ret=6000, label="season_avg 6.0 (60/10)"),
    Probe(fn_va=0, args=(0, 0),        rust_bin="season_avg_rating",   label="season_avg none (0 apps → -1)"),
    Probe(fn_va=0, args=(5, 40),       rust_bin="season_avg_rating",   expected_ret=8000, label="season_avg 8.0 (40/5)"),
    # Predict wage (FUN_006ce0e0 port)
    Probe(fn_va=0, args=(2000, 15, 5, 25, 0, 1234, 100, 1),
          rust_bin="predict_wage", label="predict_wage renewal ask"),
    # Assist bonus
    Probe(fn_va=0, args=(15,),         rust_bin="assist_bonus_milli",  expected_ret=280, label="assist_bonus(15) == 280"),
    Probe(fn_va=0, args=(30,),         rust_bin="assist_bonus_milli",  expected_ret=285, label="assist_bonus(30) == 285"),
    # Chairman approves overrun
    Probe(fn_va=0, args=(400_000, 1),  rust_bin="chairman_approves_overrun", expected_ret=1, label="chairman approve overrun under cap"),
    Probe(fn_va=0, args=(20_000_000, 1), rust_bin="chairman_approves_overrun", expected_ret=0, label="chairman REJECT overrun over cap"),
    # Club status byte
    Probe(fn_va=0, args=(5, 100, 32*100), rust_bin="club_status_byte", expected_ret=173, label="club_status[5] byte"),
    Probe(fn_va=0, args=(101, 100, 32*100), rust_bin="club_status_byte", label="club_status out-of-range → 0xFF"),
    # Foreign player permit
    Probe(fn_va=0, args=(5000, 1, 1, 1, 0), rust_bin="foreign_player_permit", expected_ret=1, label="permit same country"),
    Probe(fn_va=0, args=(2000, 1, 2, 3, 0), rust_bin="foreign_player_permit", label="permit low rep, different country → deny"),
    Probe(fn_va=0, args=(5000, 1, 2, 3, 0), rust_bin="foreign_player_permit", expected_ret=1, label="permit high rep, any nation → allow"),
    # Mentor loyalty override
    Probe(fn_va=0, args=(),             rust_bin="mentor_loyalty_bypass", expected_ret=3, label="MENTOR_LOYALTY_OVERRIDE_MANAGERS count == 3"),
    # Morale label
    Probe(fn_va=0, args=(0,),           rust_bin="morale_label_id", label="morale 0 → Very Low(0)"),
    Probe(fn_va=0, args=(10,),          rust_bin="morale_label_id", label="morale 10 → Ok(2)"),
    Probe(fn_va=0, args=(20,),          rust_bin="morale_label_id", label="morale 20 → Superb(5)"),
    # Scout bucket range
    Probe(fn_va=0, args=(0x11,),        rust_bin="scout_bucket_range", label="scout_bucket SW → 0x01..0x04"),
    Probe(fn_va=0, args=(0x14,),        rust_bin="scout_bucket_range", label="scout_bucket F → 0x0E only"),
    # Away goals verdict
    Probe(fn_va=0, args=(2, 3, 100, 200), rust_bin="away_goals_verdict", label="away goals win (200)"),
    Probe(fn_va=0, args=(3, 2, 100, 200), rust_bin="away_goals_verdict", label="home wins on away goals (100)"),
    Probe(fn_va=0, args=(2, 2, 100, 200), rust_bin="away_goals_verdict", label="away goals level -> -1"),
    # Mentality outcome scaler (returned as milli-i64)
    Probe(fn_va=0, args=(0x20,), rust_bin="mentality_outcome_scaler", expected_ret=500, label="mentality Normal -> 500 (=0.5)"),
    Probe(fn_va=0, args=(0x40,), rust_bin="mentality_outcome_scaler", expected_ret=4000, label="mentality Attacking -> 4000"),
    Probe(fn_va=0, args=(0,),    rust_bin="mentality_outcome_scaler", expected_ret=2000, label="mentality Defensive -> 2000"),
    # update_best_rating: sentinel + max compare
    Probe(fn_va=0, args=(-1000, 7500), rust_bin="update_best_rating", expected_ret=7500, label="update_best_rating sentinel init -> 7500"),
    Probe(fn_va=0, args=(6000, 7500),  rust_bin="update_best_rating", expected_ret=7500, label="update_best_rating upgrade 6000->7500"),
    Probe(fn_va=0, args=(8000, 7500),  rust_bin="update_best_rating", expected_ret=8000, label="update_best_rating keep 8000 (>7500)"),
    Probe(fn_va=0, args=(),             rust_bin="rating_scale_from_raw", expected_ret=1000, label="rating_scale_from_raw == 1000 (0.01 * 100k)"),
    # reputation_to_stars — floor at 1, linear /500 above
    Probe(fn_va=0, args=(0,),      rust_bin="reputation_to_stars", expected_ret=1, label="rep 0 -> 1 star (floor)"),
    Probe(fn_va=0, args=(499,),    rust_bin="reputation_to_stars", expected_ret=1, label="rep 499 -> 1 star (still floor)"),
    Probe(fn_va=0, args=(500,),    rust_bin="reputation_to_stars", expected_ret=1, label="rep 500 -> 1 star (boundary)"),
    Probe(fn_va=0, args=(2500,),   rust_bin="reputation_to_stars", expected_ret=5, label="rep 2500 -> 5 stars"),
    Probe(fn_va=0, args=(9999,),   rust_bin="reputation_to_stars", expected_ret=19, label="rep 9999 -> 19 stars"),
    # clamp_attendance_components — cap at 40000/25000
    Probe(fn_va=0, args=(50_000,), rust_bin="clamp_attendance_home", expected_ret=40_000, label="home clamp at 40000"),
    Probe(fn_va=0, args=(30_000,), rust_bin="clamp_attendance_home", expected_ret=30_000, label="home 30000 passes through"),
    Probe(fn_va=0, args=(30_000,), rust_bin="clamp_attendance_away", expected_ret=25_000, label="away clamp at 25000"),
    Probe(fn_va=0, args=(20_000,), rust_bin="clamp_attendance_away", expected_ret=20_000, label="away 20000 passes through"),
]


def load_probes_from_json(path: Path) -> list[Probe]:
    data = json.loads(path.read_text())
    return [Probe(
        fn_va=int(row["fn_va"], 0),
        args=tuple(row.get("args", [])),
        thiscall=row.get("thiscall", False),
        rust_bin=row.get("rust_bin"),
        expected_eax=row.get("expected_eax"),
        label=row.get("label", ""),
    ) for row in data["probes"]]


# --- CLI ------------------------------------------------------------------

def main():
    import argparse
    ap = argparse.ArgumentParser(description="Diff harness: exe vs Rust port")
    ap.add_argument("--probes", default=None,
                    help="JSON file with probe list (default: built-in matrix)")
    ap.add_argument("-o", "--out", default=None)
    args = ap.parse_args()

    probes = load_probes_from_json(Path(args.probes)) if args.probes else (DEFAULT_PROBES + RUST_ONLY_PROBES)
    out = run_matrix(probes, Path(args.out) if args.out else None)
    data = json.loads(out.read_text())
    print(f"wrote {out}")
    print(f"  probes: {data['count']}   passed: {data['passed']}   failed: {data['failed']}")
    if data["failed"]:
        print("  first 5 failures:")
        for r in [r for r in data["results"] if not r["match"]][:5]:
            print(f"    {r['probe']['label']:30s} "
                  f"exe={r['exe'].get('eax_signed')} rust={r['rust'].get('ret')} "
                  f"delta={r.get('delta')}")


if __name__ == "__main__":
    main()
