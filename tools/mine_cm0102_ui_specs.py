from __future__ import annotations

import argparse
import json
import re
from collections import Counter
from pathlib import Path


DEFAULT_CARVE_ROOT = Path("D:/cm0102-carve")
DEFAULT_OUTPUT = Path("D:/cm0102-rs/reports/cm0102_ui_specs.json")

DISPLAY_CALLS = {
    "FUN_00549580": "display_queue_record",
    "FUN_00549790": "display_region_or_list",
    "FUN_00415b10": "display_wrapper_type1",
    "FUN_00415b70": "display_wrapper_type2",
    "FUN_00415bd0": "display_wrapper_type2_region",
    "FUN_00415c30": "display_wrapper_text_global",
    "FUN_005cd840": "border_or_line_frame",
    "FUN_005ccba0": "fill_or_clear_region",
    "FUN_005ce750": "font_loader",
}
TEXT_BUFFER_CALLS = {
    "FUN_006547c0": "format_or_copy_text_to_ui_buffer",
}

SCREEN_NAMES = {
    "0x457200": "squad_screen",
    "0x00457200": "squad_screen",
    "0x0045dbc0": "news_screen",
    "0x00498f50": "fixtures_screen",
    "0x005b6f10": "startup_asset_loader",
    "0x005ce750": "font_loader",
    "0x0088a850": "tactics_screen",
    "0x549580": "display_queue_record",
    "0x005cc310": "window_create_and_screen_init",
    "0x005cd840": "border_or_line_frame",
    "0x00415c90": "shared_screen_list_builder",
}

COLOUR_SYMBOLS = {
    "DAT_00acdf6e": "colour_or_pen_acdf6e",
    "DAT_00acdf70": "colour_or_window_metric_acdf70",
    "DAT_00acdf74": "colour_or_pen_acdf74",
    "DAT_00acdf82": "colour_or_pen_acdf82",
    "DAT_00acdf9a": "green_or_selected_colour_token",
    "DAT_00ad6bc4": "text_colour_token_ad6bc4",
    "DAT_00ad6bc8": "window_metric_or_colour_ad6bc8",
    "DAT_00ad6bd8": "text_colour_token_ad6bd8",
    "DAT_00ad6bda": "black_or_shadow_colour_token",
    "DAT_00ad6bdc": "white_or_highlight_colour_token",
    "DAT_00ad6bf4": "panel_colour_token_ad6bf4",
}

ADDRESS_RE = re.compile(r"//\s*(0x[0-9a-fA-F]+)\s+")
STRING_RE = re.compile(r'"([^"\\]*(?:\\.[^"\\]*)*)"')
SYMBOL_RE = re.compile(r"\bDAT_[0-9a-fA-F]{8}\b")
GHIDRA_STRING_SYMBOL_RE = re.compile(r"\bs_[A-Za-z0-9_<>]+_([0-9a-fA-F]{8})\b")


def strip_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    return text


def split_args(args: str) -> list[str]:
    parts: list[str] = []
    start = 0
    depth = 0
    in_string = False
    escaped = False
    for index, char in enumerate(args):
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char in "([{":
            depth += 1
        elif char in ")]}":
            depth -= 1
        elif char == "," and depth == 0:
            parts.append(args[start:index].strip())
            start = index + 1
    tail = args[start:].strip()
    if tail:
        parts.append(tail)
    return parts


def find_calls(text: str, function_name: str) -> list[tuple[int, str]]:
    calls: list[tuple[int, str]] = []
    pattern = function_name + "("
    index = 0
    while True:
        start = text.find(pattern, index)
        if start == -1:
            break
        args_start = start + len(pattern)
        depth = 1
        cursor = args_start
        in_string = False
        escaped = False
        while cursor < len(text):
            char = text[cursor]
            if in_string:
                if escaped:
                    escaped = False
                elif char == "\\":
                    escaped = True
                elif char == '"':
                    in_string = False
            elif char == '"':
                in_string = True
            elif char == "(":
                depth += 1
            elif char == ")":
                depth -= 1
                if depth == 0:
                    line = text.count("\n", 0, start) + 1
                    calls.append((line, text[args_start:cursor]))
                    break
            cursor += 1
        index = cursor + 1
    return calls


def numeric_constant(value: str) -> int | None:
    value = value.strip()
    if re.fullmatch(r"0x[0-9a-fA-F]+", value):
        return int(value, 16)
    if re.fullmatch(r"-?\d+", value):
        return int(value)
    return None


def classify_arg(value: str) -> dict:
    numeric = numeric_constant(value)
    symbols = sorted(set(SYMBOL_RE.findall(value)))
    return {
        "expr": value,
        "numeric": numeric,
        "symbols": symbols,
        "symbol_meanings": [COLOUR_SYMBOLS.get(symbol, "unknown_global") for symbol in symbols],
    }


def call_record(path: Path, root: Path, screen: str, function_name: str, line: int, args: str) -> dict:
    parsed_args = split_args(args)
    return {
        "screen": screen,
        "file": str(path),
        "relative_file": path.relative_to(root).as_posix(),
        "line": line,
        "function": function_name,
        "semantic": DISPLAY_CALLS[function_name],
        "arg_count": len(parsed_args),
        "args": [classify_arg(arg) for arg in parsed_args],
        "raw": f"{function_name}({args})",
    }


def string_literals(text: str) -> list[str]:
    values = []
    for value in STRING_RE.findall(text):
        try:
            decoded = bytes(value, "utf-8").decode("unicode_escape")
        except Exception:
            decoded = value
        if decoded and any(char.isalpha() for char in decoded):
            values.append(decoded)
    return values


def load_ghidra_strings(root: Path) -> dict[str, str]:
    strings_path = root / "ghidra_out" / "cm0102.exe" / "strings.json"
    if not strings_path.exists():
        return {}
    data = json.loads(strings_path.read_text(encoding="utf-8", errors="replace"))
    rows = data if isinstance(data, list) else data.get("strings", [])
    by_addr = {}
    for row in rows:
        addr = str(row.get("addr", "")).lower()
        value = row.get("s")
        if addr.startswith("0x") and isinstance(value, str):
            by_addr[addr] = value
    return by_addr


def string_symbols(text: str, string_table: dict[str, str]) -> list[dict]:
    seen = set()
    records = []
    for match in GHIDRA_STRING_SYMBOL_RE.finditer(text):
        addr = f"0x{match.group(1).lower()}"
        if addr in seen:
            continue
        seen.add(addr)
        records.append(
            {
                "symbol": match.group(0),
                "addr": addr,
                "text": string_table.get(addr, ""),
            }
        )
    return records


def extract_function_address(text: str, path: Path) -> str:
    first_line = text.splitlines()[0] if text else ""
    match = ADDRESS_RE.search(first_line)
    if match:
        return match.group(1).lower()
    return path.stem.lower()


def mine_decompiled(root: Path) -> dict:
    decompiled_dirs = [
        root / "decompiled" / "ui_replica",
        root / "decompiled" / "ui_display_primitives",
    ]
    calls = []
    labels_by_screen: dict[str, list[str]] = {}
    string_symbols_by_screen: dict[str, list[dict]] = {}
    symbols = Counter()
    files_seen = []
    string_table = load_ghidra_strings(root)

    for directory in decompiled_dirs:
        if not directory.exists():
            continue
        for path in sorted(directory.glob("*.c")):
            text = strip_comments(path.read_text(encoding="utf-8", errors="replace"))
            address = extract_function_address(text, path)
            screen = SCREEN_NAMES.get(address, address)
            files_seen.append({"screen": screen, "address": address, "file": str(path)})

            resolved_symbols = string_symbols(text, string_table)
            resolved_labels = [record["text"] for record in resolved_symbols if record["text"]]
            labels = sorted(set(string_literals(text) + resolved_labels))
            if labels:
                labels_by_screen[screen] = labels
            if resolved_symbols:
                string_symbols_by_screen[screen] = resolved_symbols

            for symbol in SYMBOL_RE.findall(text):
                symbols[symbol] += 1

            for function_name in DISPLAY_CALLS:
                for line, args in find_calls(text, function_name):
                    calls.append(call_record(path, root, screen, function_name, line, args))
            for function_name in TEXT_BUFFER_CALLS:
                for line, args in find_calls(text, function_name):
                    parsed_args = split_args(args)
                    call = {
                        "screen": screen,
                        "file": str(path),
                        "relative_file": path.relative_to(root).as_posix(),
                        "line": line,
                        "function": function_name,
                        "semantic": TEXT_BUFFER_CALLS[function_name],
                        "arg_count": len(parsed_args),
                        "args": [classify_arg(arg) for arg in parsed_args],
                        "resolved_strings": [
                            {
                                "symbol": match.group(0),
                                "addr": f"0x{match.group(1).lower()}",
                                "text": string_table.get(f"0x{match.group(1).lower()}", ""),
                            }
                            for match in GHIDRA_STRING_SYMBOL_RE.finditer(args)
                        ],
                        "raw": f"{function_name}({args})",
                    }
                    calls.append(call)

    primitive_counts = Counter(call["semantic"] for call in calls)
    screen_counts = Counter(call["screen"] for call in calls)
    symbol_records = [
        {
            "symbol": symbol,
            "count": count,
            "meaning": COLOUR_SYMBOLS.get(symbol, "unknown_global"),
        }
        for symbol, count in symbols.most_common()
    ]
    return {
        "format": "cm0102-rs-ui-specs",
        "version": 1,
        "source": {
            "carve_root": str(root),
            "method": (
                "Static extraction from Ghidra decompile. These records describe original "
                "display queue/list/region calls and labels; exact runtime values for "
                "variables still require further code lifting or execution of equivalent paths."
            ),
        },
        "summary": {
            "files": len(files_seen),
            "display_calls": len(calls),
            "screens": len(screen_counts),
            "labels": sum(len(labels) for labels in labels_by_screen.values()),
            "global_symbols": len(symbol_records),
            "primitive_counts": dict(sorted(primitive_counts.items())),
            "screen_call_counts": dict(sorted(screen_counts.items())),
        },
        "files": files_seen,
        "display_primitives": {
            name: semantic for name, semantic in sorted(DISPLAY_CALLS.items())
        },
        "known_global_symbols": symbol_records,
        "labels_by_screen": labels_by_screen,
        "string_symbols_by_screen": string_symbols_by_screen,
        "calls": calls,
    }


def write_markdown(report: dict, output: Path) -> None:
    md = output.with_suffix(".md")
    lines = [
        "# CM0102 UI Static Spec",
        "",
        "This is the current static lift of the original UI draw pipeline.",
        "",
        "## Summary",
        "",
    ]
    for key, value in report["summary"].items():
        if isinstance(value, dict):
            continue
        lines.append(f"- {key}: {value}")
    lines.extend(["", "## Primitive Counts", ""])
    for key, value in report["summary"]["primitive_counts"].items():
        lines.append(f"- {key}: {value}")
    lines.extend(["", "## Screen Call Counts", ""])
    for key, value in report["summary"]["screen_call_counts"].items():
        lines.append(f"- {key}: {value}")
    lines.extend(["", "## Lift Status", ""])
    lines.append("- Done: display queue/list/region call extraction with line provenance.")
    lines.append("- Done: screen label/string extraction from decompiled UI functions.")
    lines.append("- Done: global colour/font/window token inventory.")
    lines.append("- Remaining: resolve variable expressions into final runtime coordinates.")
    lines.append("- Remaining: lift input/hitbox dispatch and button-state semantics.")
    lines.append("- Remaining: bind screen specs to Rust/Godot rendering components.")
    md.write_text("\n".join(lines) + "\n", encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser(description="Mine CM0102 UI draw specs from carve decompile.")
    parser.add_argument("--carve-root", default=str(DEFAULT_CARVE_ROOT))
    parser.add_argument("--output", default=str(DEFAULT_OUTPUT))
    args = parser.parse_args()

    report = mine_decompiled(Path(args.carve_root))
    output = Path(args.output)
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(report, indent=2), encoding="utf-8")
    write_markdown(report, output)
    print(json.dumps(report["summary"], indent=2))
    print(output)
    print(output.with_suffix(".md"))


if __name__ == "__main__":
    main()
