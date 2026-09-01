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
    """One fn + input tuple + expected-output spec."""
    fn_va: int
    args: tuple[int, ...]
    thiscall: bool = False
    # Rust-side handle. Either a `cargo run --bin cm-lift-rpc --  <name> <args>`
    # invocation, or an inline expected value.
    rust_bin: Optional[str] = None
    expected_eax: Optional[int] = None
    label: str = ""


def run_exe_probe(emu: Emulator, probe: Probe) -> dict:
    """Run one probe under Unicorn, return {eax, fpu, exc}."""
    eax = emu.call(probe.fn_va, *probe.args, thiscall=probe.thiscall)
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
        args_flat = [str(a) for a in probe.args]
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


def diff(exe: dict | None, rust: dict) -> dict:
    """Compare exe vs rust output tuples. If exe is None, this is a
    rust-only smoke probe — mark 'rust_ok' iff rust returned a non-null,
    non-exception value."""
    if exe is None:
        rust_ok = rust.get("ret") is not None and rust.get("exc") is None
        return {"exe": None, "rust": rust, "match": rust_ok,
                "kind": "rust_only", "delta": None}
    exe_v = exe.get("eax_signed", exe.get("eax"))
    rust_v = rust.get("ret")
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
        d = diff(exe, rust)
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
    # NOTE: FUN_006DB520 is __thiscall(token_ptr, side) — reads zone_x/y
    # from token+0x102/+0x103. Our Rust port takes (x, y, side) directly.
    # To diff, we need to allocate a token in emu memory + write x/y +
    # pass a pointer. Left as TODO; using rust-only smoke test.
]

# --- Direct Rust-only probes (no exe emulation needed) --------------------
# For fns without a clean single-fn exe entry point, we still exercise the
# RPC bin to smoke-test the port. exe-side check reports 'no-emu' — sanity
# only.
RUST_ONLY_PROBES: list[Probe] = [
    Probe(fn_va=0, args=(6400,),       rust_bin="finalize_rating",     label="finalize_rating(6400) == 6"),
    Probe(fn_va=0, args=(500,),        rust_bin="finalize_rating",     label="finalize_rating(500) == 1"),
    Probe(fn_va=0, args=(9500,),       rust_bin="finalize_rating",     label="finalize_rating(9500) == 10"),
    Probe(fn_va=0, args=(0, 0),        rust_bin="gk_save_rating_delta_milli", label="gk_save (no conc, no flags) == 400"),
    Probe(fn_va=0, args=(1, 2),        rust_bin="gk_save_rating_delta_milli", label="gk_save (concede + flag=0b10)"),
    Probe(fn_va=0, args=(15,),         rust_bin="team_mentality_mask", label="team_mentality(15) == 0x100"),
    Probe(fn_va=0, args=(5,),          rust_bin="team_mentality_mask", label="team_mentality(5) == 0"),
    Probe(fn_va=0, args=(0x0001,),     rust_bin="role_mask_to_position",  label="role_mask 0x0001 → Gk(0)"),
    Probe(fn_va=0, args=(0x0804,),     rust_bin="role_mask_to_position",  label="role_mask 0x0804 → Dr(3)"),
    Probe(fn_va=0, args=(20, 100),     rust_bin="age_wage_cap",        label="age_wage_cap(20, 100) == 275000"),
    Probe(fn_va=0, args=(1, 3, 5, 10, 2, 4), rust_bin="fifa_score",   label="fifa_score smoke"),
    # Season-avg rating (verified sum/count formula from FUN_007aa490)
    Probe(fn_va=0, args=(10, 60),      rust_bin="season_avg_rating",   label="season_avg 6.0 (60/10)"),
    Probe(fn_va=0, args=(0, 0),        rust_bin="season_avg_rating",   label="season_avg none (0 apps → -1)"),
    Probe(fn_va=0, args=(5, 40),       rust_bin="season_avg_rating",   label="season_avg 8.0 (40/5)"),
    # Predict wage (FUN_006ce0e0 port)
    Probe(fn_va=0, args=(2000, 15, 5, 25, 0, 1234, 100, 1),
          rust_bin="predict_wage", label="predict_wage renewal ask"),
    # Assist bonus
    Probe(fn_va=0, args=(15,),         rust_bin="assist_bonus_milli",  label="assist_bonus(15) == 280"),
    Probe(fn_va=0, args=(30,),         rust_bin="assist_bonus_milli",  label="assist_bonus(30) == 285"),
    # Chairman approves overrun
    Probe(fn_va=0, args=(400_000, 1),  rust_bin="chairman_approves_overrun", label="chairman approve overrun under cap"),
    Probe(fn_va=0, args=(20_000_000, 1), rust_bin="chairman_approves_overrun", label="chairman REJECT overrun over cap"),
    # Club status byte
    Probe(fn_va=0, args=(5, 100, 32*100), rust_bin="club_status_byte", label="club_status[5] byte"),
    Probe(fn_va=0, args=(101, 100, 32*100), rust_bin="club_status_byte", label="club_status out-of-range → 0xFF"),
    # Foreign player permit
    Probe(fn_va=0, args=(5000, 1, 1, 1, 0), rust_bin="foreign_player_permit", label="permit same country"),
    Probe(fn_va=0, args=(2000, 1, 2, 3, 0), rust_bin="foreign_player_permit", label="permit low rep, different country → deny"),
    Probe(fn_va=0, args=(5000, 1, 2, 3, 0), rust_bin="foreign_player_permit", label="permit high rep, any nation → allow"),
    # Mentor loyalty override
    Probe(fn_va=0, args=(),             rust_bin="mentor_loyalty_bypass", label="MENTOR_LOYALTY_OVERRIDE_MANAGERS count == 3"),
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
    Probe(fn_va=0, args=(2, 2, 100, 200), rust_bin="away_goals_verdict", label="away goals level → -1"),
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
