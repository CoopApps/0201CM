"""One-command drift gate for the generated GUI screens.

For every geometry-bearing capture in reports/screen_captures/:
  1) regenerate its Rust module (tools/gen_screen_rs.py)
  2) build dump_screen_geometry once
  3) emit the generated skeleton's geometry (--gen <slug>)
  4) diff against the exe capture (tools/diff_screens.py)

Prints a PASS/FAIL table; exit 0 iff every screen matches. Run this after
any change to the generator, gen_screen_types.rs, or a capture. CI-able.

Usage:
    python tools/verify_screens.py            # all geometry-bearing captures
    python tools/verify_screens.py <slug>...  # just these
"""
import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[1]
CAPTURES = REPO / "reports" / "screen_captures"
PY = sys.executable


def run(args, **kw):
    return subprocess.run(args, cwd=REPO, **kw).returncode


def geometry_slugs():
    out = []
    for p in sorted(CAPTURES.glob("*.json")):
        if p.name.endswith(".render.json"):
            continue
        d = json.loads(p.read_text())
        if isinstance(d, dict) and (
            d.get("areas") or d.get("objects") or d.get("tabs") or d.get("nav")
        ):
            out.append(p.stem)
    return out


def main():
    slugs = sys.argv[1:] or geometry_slugs()
    if not slugs:
        sys.exit("no geometry-bearing captures found")

    if run([PY, "tools/gen_screen_rs.py", "--all"]) != 0:
        sys.exit("generator failed")
    if run(["cargo", "build", "-q", "-p", "cm-render",
            "--bin", "dump_screen_geometry"]) != 0:
        sys.exit("cargo build failed")

    results = []
    for slug in slugs:
        rc = run(["cargo", "run", "-q", "-p", "cm-render",
                  "--bin", "dump_screen_geometry", "--", "--gen", slug],
                 capture_output=True)
        if rc != 0:
            results.append((slug, "NO-GEN (module missing?)"))
            continue
        rc = run([PY, "tools/diff_screens.py", slug], capture_output=True)
        results.append((slug, "PASS" if rc == 0 else "FAIL"))

    width = max(len(s) for s, _ in results)
    fails = 0
    print()
    for slug, status in results:
        if status != "PASS":
            fails += 1
        print(f"  {slug:<{width}}  {status}")
    print(f"\n{len(results) - fails}/{len(results)} screens match the exe capture.")
    if fails:
        print("Rerun `python tools/diff_screens.py <slug>` for field-level detail.")
    sys.exit(1 if fails else 0)


if __name__ == "__main__":
    main()
