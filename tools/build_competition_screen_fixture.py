#!/usr/bin/env python3
"""Build the first CM0102 screen fixture from lifted competition UI evidence.

The output is not a guessed mockup. Every executable-derived object carries the
callsite(s) that prove the geometry/role currently known from the lift report.
Unresolved fields stay null rather than being invented.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path


def load_calls(path: Path) -> dict[str, dict]:
    data = json.loads(path.read_text(encoding="utf-8"))
    return {row["call_addr"]: row for row in data["callsites"]}


def load_call_list(path: Path) -> list[dict]:
    if not path.exists():
        return []
    data = json.loads(path.read_text(encoding="utf-8"))
    return data.get("callsites", [])


def rect(left: int, top: int, right: int, bottom: int) -> dict[str, int]:
    return {"left": left, "top": top, "right": right, "bottom": bottom}


def obj(kind: str, object_id: str, bounds: dict, text: str | None, evidence: list[str], **extra) -> dict:
    row = {
        "id": object_id,
        "kind": kind,
        "bounds": bounds,
        "text": text,
        "evidence_callsites": evidence,
        "confidence": "code_derived",
    }
    row.update(extra)
    return row


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--callsites", type=Path, default=Path("D:/cm0102-rs/reports/carve_segment_index/fixtures/competition_league_table_callsites.json"))
    parser.add_argument("--decoded-callsites", type=Path, default=Path("D:/cm0102-rs/reports/carve_segment_index/fixtures/competition_league_table_decoded_callsites.json"))
    parser.add_argument("--shell-decoded-callsites", type=Path, default=Path("D:/cm0102-rs/reports/carve_segment_index/fixtures/competition_shell_decoded_callsites.json"))
    parser.add_argument("--out", type=Path, default=Path("D:/cm0102-rs/reports/carve_segment_index/fixtures/competition_league_table_screen.json"))
    args = parser.parse_args()

    calls = load_calls(args.decoded_callsites if args.decoded_callsites.exists() else args.callsites)
    shell_calls = load_call_list(args.shell_decoded_callsites)
    fixture = {
        "schema": "cm0102.screen.v1",
        "screen": "competition.league_table",
        "native_size": {"width": 800, "height": 600},
        "source_report": "D:/cm0102-rs/reports/carve_segment_index/competition_league_table_screen_lift.md",
        "raw_callsites": str(args.callsites).replace("\\", "/"),
        "decoded_callsites": str(args.decoded_callsites).replace("\\", "/"),
        "shell_decoded_callsites": str(args.shell_decoded_callsites).replace("\\", "/"),
        "area_layout_model": "D:/cm0102-rs/reports/carve_segment_index/fixtures/ui_area_layout_model.json",
        "notes": [
            "Fixed 800x600 is code-derived from area/object clamping.",
            "Central content x-range 0x6e..0x30c is code-derived from 0x00495ad0 callsites.",
            "Competition selector lanes 0x28f..0x30c and 0x210..0x28d are code-derived from 0x004a3770 callsites.",
            "Top-right Print button is code-derived from the competition shell range at 0x00494c38.",
            "Top/bottom tab strings and child object styling are shell-range-derived; scaffold rectangles are illustrative only and hidden by default in the viewer.",
            "Main title/sidebar/Back/Next positions still need their exact parent-screen builder lifted; they are marked scaffold, not code_derived.",
        ],
        "strings": {
            "table": {"addr": "0x009888e0", "text": "Table"},
            "results": {"addr": "0x0097d33c", "text": "Results"},
            "fixtures": {"addr": "0x0097b15c", "text": "Fixtures"},
            "schedule": {"addr": "0x009888a0", "text": "Schedule"},
            "team_stats": {"addr": "0x0098886c", "text": "Team Stats"},
            "player_stats": {"addr": "0x0098885c", "text": "Player Stats"},
            "referee_stats": {"addr": "0x0098884c", "text": "Referee Stats"},
            "awards": {"addr": "0x00988844", "text": "Awards"},
            "history": {"addr": "0x00988810", "text": "History"},
            "print": {"addr": "0x0097b184", "text": "Print"},
            "league_table": {"addr": "0x00988a18", "text": "League Table"},
        },
        "objects": [
            obj("background", "background.default_pic", rect(0, 0, 799, 599), None, [], asset="assets/cm0102/Data/default_pic.png", confidence="asset_extracted_scaffold"),
            obj("side_nav_scaffold", "shell.left_nav", rect(0, 0, 87, 599), None, [], confidence="scaffold_pending_shell_builder_lift"),
            obj("title_scaffold", "shell.competition_title", rect(98, 8, 792, 54), "English Premier Division", [], confidence="scaffold_pending_shell_builder_lift"),
            obj("button", "shell.print_button", rect(710, 15, 785, 35), "Print", ["0x00494bc9", "0x00494c38"], font_slot=1, render_flags="0x30", text_mode_global="DAT_00acdf74"),
            obj("area", "shell.competition_tab_area", rect(0, 0, 0, 0), None, ["0x00494c6e"], flags={"area": "0x130"}, confidence="code_derived_pending_parent_area_layout"),
            obj("tab_scaffold", "top.tab.table", rect(98, 65, 272, 93), "Table", ["0x00494844"], selected=True, confidence="string_verified_shell_range_pending_child_area_layout"),
            obj("tab_scaffold", "top.tab.results", rect(272, 65, 446, 93), "Results", ["0x00494892"], confidence="string_verified_shell_range_pending_child_area_layout"),
            obj("tab_scaffold", "top.tab.fixtures", rect(446, 65, 620, 93), "Fixtures", ["0x004948d3"], confidence="string_verified_shell_range_pending_child_area_layout"),
            obj("tab_scaffold", "top.tab.schedule", rect(620, 65, 792, 93), "Schedule", ["0x004948ff"], confidence="string_verified_shell_range_pending_child_area_layout"),
            obj("text", "table.title", rect(110, None, 780, None), "League Table", ["0x00495d0c", "0x004962a8"], confidence="code_derived_dynamic_y"),
            obj("area", "table.main_area", rect(110, None, 780, None), None, ["0x004963c9"], confidence="code_derived_dynamic_y", flags={"area": "0x2", "columns": 11, "rows": "dynamic"}),
            obj("row", "table.header_wide", rect(110, None, 780, None), None, ["0x00495d0c", "0x004962a8"], confidence="code_derived_dynamic_y", dynamic_y="input y / competition branch, bottom=y+0x23"),
            obj("area", "table.selector_area", rect(110, None, 235, None), None, ["0x00495ec6", "0x00495efb"], confidence="code_derived_dynamic_y", dynamic_y="selector y, bottom=y+0x14", flags={"area": "0x130"}),
            obj("selector_lane", "selector.right_lane", rect(655, None, 780, None), None, ["0x004a3930", "0x004a396f", "0x004a3a53", "0x004a3a8e", "0x004a3b21", "0x004a3b89", "0x004a3bbe", "0x004a3c08", "0x004a3d04"], confidence="code_derived_dynamic_y", dynamic_y="param_y..param_y+0x14"),
            obj("selector_lane", "selector.middle_lane", rect(528, None, 653, None), None, ["0x004a39c0", "0x004a3c79", "0x004a3cb1"], confidence="code_derived_dynamic_y", dynamic_y="param_y..param_y+0x14"),
            obj("column_series", "table.header_columns", rect(None, None, None, None), None, ["0x00496427", "0x00496488", "0x00496501", "0x00496562", "0x004965be", "0x0049661a", "0x00496676"], confidence="code_derived_pending_area_layout", slots=[4, 5, 6, 7, 8, 9, 10]),
            obj("row_series", "table.club_rows", rect(None, None, None, None), None, ["0x004968aa", "0x00496970", "0x00496a08", "0x00496a71", "0x00496b04", "0x00496b5e", "0x00496bb8", "0x00496c12", "0x00496c6c", "0x00496cc6", "0x00496d26"], confidence="code_derived_pending_area_layout", slots=list(range(0, 11)), row_count_field="competition+0x3e"),
            obj("marker", "table.cutoff_marker", rect(112, None, None, None), None, ["0x004967c8"], confidence="code_derived_dynamic_y_and_width", dynamic_width="right = 0x30a - dynamic", fields=["competition+0xbe", "competition+0xbf", "competition+0xc0", "competition+0xc1", "competition+0xd9&1"]),
            obj("area", "fixture_progress.area", rect(110, None, 780, None), None, ["0x00496ee5"], confidence="code_derived_dynamic_y", dynamic_y="lower fixture/progress section if bVar1"),
            obj("row_series", "fixture_progress.rows", rect(None, None, None, None), None, ["0x00496f9d", "0x0049702d", "0x00497220", "0x004972b2", "0x004973e8", "0x00497431", "0x004974a1"], confidence="code_derived_pending_area_layout", constants=["0x7d2", "0x7d5", "0x00988924"]),
            obj("tab_scaffold", "bottom.tab.team_stats", rect(98, 508, 264, 536), "Team Stats", ["0x00494a39"], confidence="string_verified_shell_range_pending_child_area_layout"),
            obj("tab_scaffold", "bottom.tab.player_stats", rect(264, 508, 430, 536), "Player Stats", ["0x00494a6e"], confidence="string_verified_shell_range_pending_child_area_layout"),
            obj("tab_scaffold", "bottom.tab.referee_stats", rect(430, 508, 596, 536), "Referee Stats", ["0x00494ad8"], confidence="string_verified_shell_range_pending_child_area_layout"),
            obj("tab_scaffold", "bottom.tab.awards", rect(596, 508, 694, 536), "Awards", ["0x00494b22"], confidence="string_verified_shell_range_pending_child_area_layout"),
            obj("tab_scaffold", "bottom.tab.history", rect(694, 508, 792, 536), "History", ["0x00494b80"], confidence="string_verified_shell_range_pending_child_area_layout"),
            obj("button_scaffold", "bottom.back", rect(98, 552, 400, 586), "Back", [], confidence="scaffold_pending_shell_builder_lift"),
            obj("button_scaffold", "bottom.next", rect(400, 552, 792, 586), "Next", [], confidence="scaffold_pending_shell_builder_lift"),
        ],
        "callsites": shell_calls + list(calls.values()),
    }

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(fixture, indent=2), encoding="utf-8")
    print(f"wrote {args.out} ({len(fixture['objects'])} screen objects, {len(fixture['callsites'])} callsites)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
