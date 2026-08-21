#!/usr/bin/env python3
"""Summarize lifted CM0102 competition UI screen-family constructor evidence."""
from __future__ import annotations

import json
from collections import Counter, defaultdict
from pathlib import Path


ROOT = Path("D:/cm0102-rs/reports/carve_segment_index")
DECODED = ROOT / "fixtures/competition_family_decoded_callsites.json"
OUT_JSON = ROOT / "fixtures/competition_ui_family_map.json"
OUT_MD = ROOT / "competition_ui_family_map.md"

KNOWN_ROLES = {
    "0x00497520": "competition cup draw screen builder (string-derived: Automatic Draw / Draw All Teams / Draw Next Team)",
    "0x00498f50": "competition fixtures unavailable/fixtures navigation builder (string-derived: No fixtures have been scheduled / << %s / %s >>)",
    "0x00499880": "competition schedule/game list fallback builder (string-derived: competition not scheduled / Game %d)",
    "0x00499dd0": "competition summary/history facts builder (string-derived: Host nation / Previous winner)",
    "0x0049a0d0": "competition seed/pool display builder (string-derived: Pool 1/2/3, European Cup, Intertoto, Seeded)",
    "0x0049a5b0": "competition coefficient/stage table builder (string-derived: Average/Played/Total/Stage)",
    "0x0049ae10": "competition subview builder (pending role lift; no direct literal refs in strings export)",
    "0x0049b3a0": "competition player-stat filter builder (string-derived: Filter / Attackers / Midfielders / Defenders / Goalkeepers)",
}


def annotation(value: dict) -> str | None:
    if "font" in value:
        return f"font:{value['font']['name']}"
    if "color" in value:
        color = value["color"]
        packed = color.get("hex") or color.get("rgb565") or color.get("rgb555")
        return f"color:{color.get('name')}{' ' + packed if packed else ''}"
    return None


def main() -> int:
    data = json.loads(DECODED.read_text(encoding="utf-8"))
    by_func: dict[str, list[dict]] = defaultdict(list)
    for call in data["callsites"]:
        by_func[call["function"]].append(call)

    functions = []
    for func, calls in sorted(by_func.items()):
        target_counts = Counter(call["target_name"] for call in calls)
        fonts = Counter()
        colors = Counter()
        flags = Counter()
        bounds = []
        for call in calls:
            decoded = call.get("decoded_args", {})
            for key, value in decoded.items():
                note = annotation(value)
                if note and note.startswith("font:"):
                    fonts[note.removeprefix("font:")] += 1
                if note and note.startswith("color:"):
                    colors[note.removeprefix("color:")] += 1
                if key in {"render_flags", "text_box_flags"}:
                    raw = value.get("raw")
                    if raw:
                        flags[f"{key}={raw}"] += 1
            if call.get("decoded_bounds"):
                bounds.append(call["decoded_bounds"])
        functions.append(
            {
                "function": func,
                "role": KNOWN_ROLES.get(func, "pending role lift"),
                "role_confidence": "string_derived" if func in KNOWN_ROLES and "pending role lift" not in KNOWN_ROLES[func] else "pending_role_lift",
                "constructor_calls": len(calls),
                "targets": dict(target_counts),
                "fonts": dict(fonts),
                "colors": dict(colors),
                "flags": dict(flags),
                "first_call": calls[0]["call_addr"] if calls else None,
                "last_call": calls[-1]["call_addr"] if calls else None,
                "sample_bounds": bounds[:5],
            }
        )

    payload = {
        "schema": "cm0102.ui.family_map.v1",
        "source": str(DECODED).replace("\\", "/"),
        "functions": functions,
        "totals": {
            "functions": len(functions),
            "constructor_calls": sum(row["constructor_calls"] for row in functions),
        },
    }
    OUT_JSON.write_text(json.dumps(payload, indent=2), encoding="utf-8")

    lines = [
        "# CM0102 Competition UI Family Map",
        "",
        "This is the first broad competition-screen UI map lifted from `cm0102.exe` constructor evidence. It covers eight screen-family builders reached from the competition parent shell dispatch.",
        "",
        "| Function | Current role | Constructor calls | Targets | Fonts | Colors | Flags |",
        "|---|---|---:|---|---|---|---|",
    ]
    for row in functions:
        targets = ", ".join(f"{k}:{v}" for k, v in sorted(row["targets"].items())) or "-"
        fonts = ", ".join(f"{k}:{v}" for k, v in sorted(row["fonts"].items())) or "-"
        colors = ", ".join(f"{k}:{v}" for k, v in sorted(row["colors"].items())) or "-"
        flags = ", ".join(f"{k}:{v}" for k, v in sorted(row["flags"].items())) or "-"
        lines.append(
            f"| `{row['function']}` | {row['role']} | {row['constructor_calls']} | {targets} | {fonts} | {colors} | {flags} |"
        )
    lines.extend(
        [
            "",
            "## Next Exact UI Targets",
            "",
            "- `0x00497520` is the largest sibling builder and should be lifted next because it has the most competition-view UI surface.",
            "- `0x0049b3a0` and `0x00498f50` are also high-value because they have dense GUI constructor use and likely correspond to visible competition subviews.",
            "- `0x00499dd0` and `0x0049a0d0` are smaller, clean vertical targets for proving the renderer handles repeated list/table variants.",
            "",
            "No roles in this file are promoted to VERIFIED until the string refs/control-flow inside each builder are read and linked to a visible screen name.",
        ]
    )
    OUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"wrote {OUT_JSON}")
    print(f"wrote {OUT_MD}")
    print(f"{payload['totals']['functions']} functions, {payload['totals']['constructor_calls']} constructor calls")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
