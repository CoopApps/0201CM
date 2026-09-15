#!/usr/bin/env python3
"""Verify every VA in season_capture_manifest.json against the GDI
function-carve index. A hook whose VA does NOT correspond to an
exact GDI function start is a mid-function jump — Interceptor.attach
there installs a trampoline that corrupts surrounding instructions
and crashes the exe as soon as normal execution walks through them.

Exit 0 iff every ENABLED hook has an exact-match GDI function.

Usage:
    D:/Python312/python.exe tools/gdi_capture/verify_manifest_vas.py
"""

from __future__ import annotations
import json
import re
import sys
from pathlib import Path

CARVE_DIR = Path("D:/cm0102-carve/gdi_carve/functions/00004-PE_section_.text")
MANIFEST = Path(__file__).parent / "season_capture_manifest.json"


def load_gdi_function_starts() -> set[int]:
    """Every function start in the GDI .text section, as an int set."""
    starts: set[int] = set()
    pat = re.compile(r"sub_([0-9a-fA-F]{6,8})\.asm$")
    for entry in CARVE_DIR.iterdir():
        m = pat.search(entry.name)
        if m:
            starts.add(int(m.group(1), 16))
    return starts


def main() -> int:
    if not CARVE_DIR.exists():
        print(f"missing carve dir: {CARVE_DIR}", file=sys.stderr)
        return 2
    if not MANIFEST.exists():
        print(f"missing manifest: {MANIFEST}", file=sys.stderr)
        return 2

    starts = load_gdi_function_starts()
    starts_sorted = sorted(starts)
    print(f"loaded {len(starts)} GDI function starts")

    manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
    bad_enabled = []
    for gname, gdef in (manifest.get("groups") or {}).items():
        hooks = gdef.get("hooks") if isinstance(gdef, dict) else None
        if not hooks:
            continue
        for h in hooks:
            va_hex = h.get("va")
            if not va_hex:
                continue
            va = int(va_hex, 16)
            exact = va in starts
            # Find nearest smaller.
            nearest = max((s for s in starts_sorted if s <= va), default=None)
            delta = va - nearest if nearest else None
            status = "OK  " if exact else "MISS"
            enabled = "ENABLED " if h.get("enabled") else "disabled"
            near_str = (f"nearest 0x{nearest:08x} (+{delta} bytes)"
                        if nearest else "no nearest")
            print(f"  {status} {enabled} {gname:20s} {h['name']:40s} "
                  f"{va_hex}  {near_str}")
            if not exact and h.get("enabled"):
                bad_enabled.append((gname, h["name"], va_hex, near_str))

    if bad_enabled:
        print(f"\nFAIL: {len(bad_enabled)} enabled hook(s) point at "
              "non-function addresses (mid-function → crash risk).")
        for entry in bad_enabled:
            print(f"  {entry}")
        return 1
    print("\nOK: every enabled hook lands on a GDI function start.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
