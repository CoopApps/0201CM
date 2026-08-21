#!/usr/bin/env python3
"""Decode raw x86 UI constructor callsites into named CM0102 object arguments."""
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

GUI_ARG_NAMES = [
    "object_flags_or_type",      # stored at guio +0x0c
    "left",                      # guio +0x10
    "top",                       # guio +0x14
    "right",                     # guio +0x18
    "bottom",                    # guio +0x1c
    "field_20",
    "field_24",
    "render_flags",              # guio +0x38
    "color_primary",             # guio +0x72
    "color_secondary",           # guio +0x74
    "text_box_flags",            # guio +0x3c
    "font_slot",                 # guio +0x76
    "text_mode",                 # guio +0x78
    "text_ptr",                  # copied to guio +0x80
    "field_40",
    "field_08",
    "field_48",
    "parent_area_index",         # guio +0x7a
]

AREA_ARG_NAMES = [
    "left",
    "top",
    "right",
    "bottom",
    "scroll_count_a",
    "scroll_bytes_a",
    "scroll_count_b",
    "scroll_bytes_b",
    "area_flags",
    "field_20c",
    "parent_guio_index_or_child",
]

SELECTOR_TAB_ARG_NAMES = [
    "context_or_competition",
    "left",
    "top",
    "right",
    "bottom",
    "label_template",
    "value_or_stage_index",
    "selected_or_mode",
    "slot_index",
    "row_payload_base",
    "area_row_count",
    "label_buffer_or_source",
    "vertical_span_or_offset",
    "field_38",
    "field_3c",
    "field_40",
    "parent_area_index",
]

CHILD_GUI_ZERO_BOUNDS_WRAPPER_ARG_NAMES = [
    "parent_area_index",         # forwarded to GUI arg18
    "field_20",                  # forwarded to GUI arg6
    "field_24",                  # forwarded to GUI arg7
    "render_flags",              # forwarded to GUI arg8
    "color_primary",             # forwarded to GUI arg9
    "text_box_flags",            # forwarded to GUI arg11
    "font_slot",                 # forwarded to GUI arg12
    "text_mode",                 # forwarded to GUI arg13
    "text_ptr",                  # forwarded to GUI arg14
]


def parse_int(value: str):
    if not isinstance(value, str):
        return None
    if value.startswith("-0x"):
        return -int(value[3:], 16)
    if value.startswith("0x"):
        return int(value[2:], 16)
    try:
        return int(value)
    except ValueError:
        return None


REGS = {"EAX", "EBX", "ECX", "EDX", "ESI", "EDI", "EBP", "ESP", "AX", "BX", "CX", "DX", "AL", "BL", "CL", "DL"}


def normalize_reg(reg: str) -> str:
    reg = reg.upper()
    if reg in {"AX", "AL"}:
        return "EAX"
    if reg in {"BX", "BL"}:
        return "EBX"
    if reg in {"CX", "CL"}:
        return "ECX"
    if reg in {"DX", "DL"}:
        return "EDX"
    return reg


def load_constants(path: Path | None) -> dict:
    if not path or not path.exists():
        return {"colors_by_global": {}, "fonts_by_slot": {}}
    data = json.loads(path.read_text(encoding="utf-8"))
    colors = {}
    for color in data.get("colors", []):
        colors[color["global"]] = color
        colors[color["global"].replace("DAT_", "")] = color
    fonts = {font["slot"]: font for font in data.get("fonts", [])}
    return {"colors_by_global": colors, "fonts_by_slot": fonts}


def infer_constant(raw: str, resolved: str | None, constants: dict, field_name: str) -> dict:
    text = " ".join(x for x in [raw, resolved] if x)
    out = {}
    for global_name, color in constants["colors_by_global"].items():
        if global_name in text:
            out["color"] = {
                "global": color["global"],
                "name": color["name"],
                "hex": color.get("hex"),
                "rgb565": color.get("rgb565"),
                "rgb555": color.get("rgb555"),
            }
            break
    if field_name == "font_slot":
        value = parse_int(raw)
        if value in constants["fonts_by_slot"]:
            font = constants["fonts_by_slot"][value]
            out["font"] = {"slot": font["slot"], "name": font["name"]}
    return out


def annotate_arg(value: str, string: str | None, resolved: str | None = None, constants: dict | None = None, field_name: str = "") -> dict:
    parsed = parse_int(value)
    out = {"raw": value, "value": parsed}
    if resolved and resolved != value:
        out["resolved"] = resolved
    if string:
        out["string"] = string
    if constants:
        out.update(infer_constant(value, resolved, constants, field_name))
    return out


def split_operands(text: str) -> list[str]:
    parts = []
    depth = 0
    start = 0
    for i, ch in enumerate(text):
        if ch == "[":
            depth += 1
        elif ch == "]":
            depth -= 1
        elif ch == "," and depth == 0:
            parts.append(text[start:i].strip())
            start = i + 1
    parts.append(text[start:].strip())
    return parts


def simplify_expr(expr: str) -> str:
    expr = expr.replace(" + -0x", " - 0x")
    expr = expr.replace("+ -0x", "- 0x")
    return expr


def expr_for_operand(operand: str, regs: dict[str, str], stack: dict[str, str]) -> str:
    operand = operand.strip()
    reg = normalize_reg(operand)
    if reg in REGS:
        return regs.get(reg, operand)
    if operand.startswith("dword ptr ") or operand.startswith("word ptr ") or operand.startswith("byte ptr "):
        operand = operand.split(" ptr ", 1)[1]
    if operand.startswith("[") and operand.endswith("]"):
        key = operand[1:-1].strip()
        key = key.replace("ESP + ", "stack+").replace("ESP", "stack")
        return stack.get(key, f"mem[{key}]")
    return operand


def parse_lea(operand: str, regs: dict[str, str]) -> str:
    inner = operand.strip()
    if inner.startswith("[") and inner.endswith("]"):
        inner = inner[1:-1].strip()
    expr = inner
    for reg in sorted(REGS, key=len, reverse=True):
        expr = re.sub(rf"\b{reg}\b", f"({regs.get(reg, reg)})", expr)
    expr = expr.replace("(ESP)", "stack")
    return simplify_expr(expr)


def add_expr(left: str, right: str, op: str = "+") -> str:
    if op == "+":
        return simplify_expr(f"{left} + {right}")
    return simplify_expr(f"{left} - {right}")


def resolve_instruction_pushes(instruction_dir: Path | None) -> dict[tuple[str, str], str]:
    if not instruction_dir:
        return {}
    resolved: dict[tuple[str, str], str] = {}
    for path in sorted(instruction_dir.glob("*.instructions.json")):
        function = path.name.replace(".instructions.json", "")
        instructions = json.loads(path.read_text(encoding="utf-8"))
        regs = {reg: reg for reg in ["EAX", "EBX", "ECX", "EDX", "ESI", "EDI", "EBP", "ESP"]}
        stack: dict[str, str] = {}
        for ins in instructions:
            mnemonic = ins["mnemonic"].upper()
            operands = split_operands(ins.get("operands", ""))
            if mnemonic == "PUSH" and operands:
                resolved[(function, ins["addr"])] = expr_for_operand(operands[0], regs, stack)
                continue
            if mnemonic in {"MOV", "MOVSX", "MOVZX"} and len(operands) >= 2:
                dest, src = operands[0], operands[1]
                dest_reg = normalize_reg(dest)
                value = expr_for_operand(src, regs, stack)
                if dest_reg in regs:
                    regs[dest_reg] = value
                elif dest.startswith("dword ptr [") or dest.startswith("word ptr [") or dest.startswith("byte ptr ["):
                    key = dest.split("[", 1)[1].rsplit("]", 1)[0].strip()
                    key = key.replace("ESP + ", "stack+").replace("ESP", "stack")
                    stack[key] = value
                continue
            if mnemonic == "LEA" and len(operands) >= 2:
                dest_reg = normalize_reg(operands[0])
                if dest_reg in regs:
                    regs[dest_reg] = parse_lea(operands[1], regs)
                continue
            if mnemonic in {"ADD", "SUB"} and len(operands) >= 2:
                dest_reg = normalize_reg(operands[0])
                if dest_reg in regs and dest_reg != "ESP":
                    regs[dest_reg] = add_expr(regs.get(dest_reg, dest_reg), expr_for_operand(operands[1], regs, stack), "+" if mnemonic == "ADD" else "-")
                continue
            if mnemonic == "SHL" and len(operands) >= 2:
                dest_reg = normalize_reg(operands[0])
                if dest_reg in regs:
                    regs[dest_reg] = f"({regs.get(dest_reg, dest_reg)}) << {operands[1]}"
                continue
            if mnemonic.startswith("CALL"):
                target = ins.get("targets", [""])[0].lower() if ins.get("targets") else ""
                # Keep callee-saved registers. Volatile return/arg registers are only reliable if overwritten later.
                regs["EAX"] = f"ret[{target or ins.get('operands', 'call')}]"
                regs["ECX"] = "ECX"
                regs["EDX"] = "EDX"
    return resolved


def decode_callsite(call: dict, resolved_pushes: dict[tuple[str, str], str], constants: dict) -> dict:
    args = list(reversed(call["pushes"]))
    if call["target_name"] in {"display_create_guio_object", "display_create_guio_object_passthrough_wrapper"} and len(args) >= 18:
        names = GUI_ARG_NAMES
    elif call["target_name"] == "display_create_area_or_emit_area_packet" and len(args) >= 11:
        names = AREA_ARG_NAMES
    elif call["target_name"] == "comp_screen_create_selector_tab" and len(args) >= 17:
        names = SELECTOR_TAB_ARG_NAMES
    elif call["target_name"] == "display_create_child_guio_zero_bounds_wrapper" and len(args) >= 9:
        names = CHILD_GUI_ZERO_BOUNDS_WRAPPER_ARG_NAMES
    else:
        names = [f"arg_{i + 1}" for i in range(len(args))]

    decoded = {}
    for name, arg in zip(names, args):
        resolved = resolved_pushes.get((call["function"], arg["addr"]))
        decoded[name] = annotate_arg(arg["value"], arg.get("string"), resolved, constants, name)

    bounds = None
    if all(name in decoded for name in ["left", "top", "right", "bottom"]):
        bounds = {name: decoded[name] for name in ["left", "top", "right", "bottom"]}
    elif call["target_name"] == "display_create_child_guio_zero_bounds_wrapper":
        bounds = {
            "left": annotate_arg("0x0", None, constants=constants, field_name="left"),
            "top": annotate_arg("0x0", None, constants=constants, field_name="top"),
            "right": annotate_arg("0x0", None, constants=constants, field_name="right"),
            "bottom": annotate_arg("0x0", None, constants=constants, field_name="bottom"),
        }
        decoded = {
            "object_flags_or_type": annotate_arg("0x1", None, constants=constants, field_name="object_flags_or_type"),
            "left": bounds["left"],
            "top": bounds["top"],
            "right": bounds["right"],
            "bottom": bounds["bottom"],
            **decoded,
            "color_secondary": annotate_arg("0x0", None, constants=constants, field_name="color_secondary"),
            "field_40": annotate_arg("0x0", None, constants=constants, field_name="field_40"),
            "field_08": annotate_arg("0x0", None, constants=constants, field_name="field_08"),
            "field_48": annotate_arg("0x0", None, constants=constants, field_name="field_48"),
        }

    return {
        **call,
        "argument_order": "decoded from reversed x86 pushes",
        "decoded_args": decoded,
        "decoded_bounds": bounds,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("callsites", type=Path)
    parser.add_argument("--instruction-dir", type=Path)
    parser.add_argument("--constants", type=Path, default=Path("D:/cm0102-rs/reports/carve_segment_index/fixtures/ui_constants.json"))
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args()

    data = json.loads(args.callsites.read_text(encoding="utf-8"))
    resolved_pushes = resolve_instruction_pushes(args.instruction_dir)
    constants = load_constants(args.constants)
    decoded = [decode_callsite(call, resolved_pushes, constants) for call in data["callsites"]]
    payload = {
        "schema": "cm0102.ui.decoded_callsites.v1",
        "source": str(args.callsites).replace("\\", "/"),
        "notes": [
            "Arguments are named from guio_init_object_record 0x005d7bd0 and area_init_record 0x00402b00.",
            "Register expressions remain raw until local dataflow resolves them.",
            "x86 cdecl/stdcall callsites push right-to-left, so raw pushes are reversed before naming.",
            "When --instruction-dir is supplied, straight-line register expressions are resolved conservatively from the exported instruction stream.",
        ],
        "callsites": decoded,
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_text(json.dumps(payload, indent=2), encoding="utf-8")
    print(f"wrote {args.out} ({len(decoded)} decoded callsites)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
