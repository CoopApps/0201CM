#!/usr/bin/env python3
"""Flag UI fixture objects whose confidence overstates their evidence."""
from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any


STRICT_CODE_DERIVED = {"code_derived"}
APPROX_TOKENS = ("scaffold", "pending", "dynamic", "inferred", "illustrative")


def concrete_bounds(bounds: dict[str, Any]) -> bool:
    return all(isinstance(bounds.get(key), int) for key in ["left", "top", "right", "bottom"])


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("fixture", type=Path)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()

    fixture = json.loads(args.fixture.read_text(encoding="utf-8"))
    findings = []
    for obj in fixture.get("objects", []):
        confidence = obj.get("confidence", "")
        bounds = obj.get("bounds") or {}
        evidence = obj.get("evidence_callsites") or []
        if confidence in STRICT_CODE_DERIVED and not evidence and obj.get("kind") != "background":
            findings.append({"id": obj.get("id"), "severity": "warn", "issue": "code_derived_without_callsite_evidence"})
        if confidence in STRICT_CODE_DERIVED and not concrete_bounds(bounds):
            findings.append({"id": obj.get("id"), "severity": "warn", "issue": "code_derived_with_non_concrete_bounds"})
        if any(token in confidence for token in APPROX_TOKENS) and concrete_bounds(bounds):
            findings.append({"id": obj.get("id"), "severity": "info", "issue": "concrete_bounds_are_not_strict_code_derived", "confidence": confidence})

    payload = {
        "schema": "cm0102.ui.fixture_provenance_audit.v1",
        "fixture": str(args.fixture).replace("\\", "/"),
        "screen": fixture.get("screen"),
        "object_count": len(fixture.get("objects", [])),
        "finding_count": len(findings),
        "findings": findings,
    }
    out = args.out or args.fixture.with_name(args.fixture.stem + "_provenance_audit.json")
    out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"wrote {out} ({len(findings)} findings)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
