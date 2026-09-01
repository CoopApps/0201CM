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
    """Run the Rust equivalent via subprocess. Expects a small
    `cm-lift-rpc` binary that accepts (fn_name, args...) → JSON on stdout.
    Returns {ret, exc}."""
    if not probe.rust_bin:
        return {"ret": probe.expected_eax, "exc": None, "source": "expected_eax"}
    try:
        args_flat = [str(a) for a in probe.args]
        result = subprocess.run(
            ["cargo", "run", "--release", "--quiet", "--bin", "cm-lift-rpc",
             "--", probe.rust_bin, *args_flat],
            cwd=RUST_ROOT, capture_output=True, text=True, timeout=30,
        )
        try:
            return json.loads(result.stdout)
        except json.JSONDecodeError:
            return {"ret": None, "exc": f"non-json: {result.stdout[:200]}"}
    except subprocess.TimeoutExpired:
        return {"ret": None, "exc": "timeout"}
    except Exception as e:
        return {"ret": None, "exc": repr(e)}


def diff(exe: dict, rust: dict) -> dict:
    """Compare exe vs rust output tuples."""
    exe_v = exe.get("eax_signed", exe.get("eax"))
    rust_v = rust.get("ret")
    ok = (exe_v == rust_v) if (exe_v is not None and rust_v is not None) else False
    return {
        "exe": exe, "rust": rust,
        "match": ok,
        "delta": (exe_v - rust_v) if (isinstance(exe_v, int)
                                     and isinstance(rust_v, int)) else None,
    }


def run_matrix(probes: list[Probe], out_path: Optional[Path] = None) -> Path:
    """Run every probe, write consolidated diff report."""
    emu = make_emulator()
    results = []
    for i, p in enumerate(probes):
        exe = run_exe_probe(emu, p)
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
    # FUN_006A2790:98-104 — box gate. shot_in_box(x, y, side).
    Probe(fn_va=0x006DB520, args=(2, 10, 1), label="shot_in_box side1"),
    Probe(fn_va=0x006DB520, args=(4, 5, 0),  label="shot_in_box mid"),
    Probe(fn_va=0x006DB520, args=(3, 1, 0),  label="shot_in_box side0"),
    # FUN_006b3de0:64-73 finalize_rating (via wrapper if needed)
    # Args: rating_milli only lives on tokens, so this probe needs pre-setup;
    # left as placeholder for a full-fn probe.
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

    probes = load_probes_from_json(Path(args.probes)) if args.probes else DEFAULT_PROBES
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
