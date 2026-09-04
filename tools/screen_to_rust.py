#!/usr/bin/env python3
"""screen_to_rust.py — Transliterate a Ghidra widget-analysis JSON into a
`to_widget_pool`-style Rust function body.

Input:  one or more `d:/cm0102-carve/analysis/screens/<addr>.json` files
        (from Ghidra `capture_construct` push-arg dumps).

Output: a Rust module fragment containing one `pub fn build_screen_<addr>`
        per input. Each spawn call cites the C address it came from.

The tool consumes the JSON schema documented in analysis/screens/README:
  * strcpy_scratch / sprintf_scratch — prime the "current text" tracker,
    used by the next widget that reads `text_ptr` from `DAT_00DC723C`.
  * area (FUN_00549790) — emit `pool.spawn_area(...)`
  * item (FUN_00549580) — emit `pool.spawn_widget(WidgetDescriptor {...}, area)`
  * sidebar (FUN_00745540) — emit `spawn_sidebar_placeholder(pool, mode)`
  * nav_bar (FUN_005d75b0) — emit `spawn_navbar_stub(pool, root, back, next)`
  * menu_list (FUN_005d6bf0) — emit stub with TODO
  * ret — end of function; ignored.

Field mapping (WidgetDescriptor, per commit 4375e51 `sub_005d76c0` audit):

    arg N (1-based)   JSON pos   WidgetDescriptor field
    ---------------   --------   ----------------------
    1  = type           0        kind (u32 @ +0x0c)
    2  = l              1        grid_x0
    3  = t              2        grid_y0
    4  = r              3        grid_x1
    5  = b              4        grid_y1
    6  = col            5        seq
    7  = row            6        row_index
    8  = flags          7        style_byte (u32 @ +0x38)
    9  = color_a        8        colour_a (u16 @ +0x72)
    10 = color_b        9        colour_b (u16 @ +0x74)
    11 = aux_a         10        text_style (u32 @ +0x3c)  [NB: JSON name misnomer]
    12 = font          11        label_ink (u16 @ +0x76)   [NB: JSON name misnomer]
    13 = aux_b         12        pattern (u16 @ +0x78)
    14 = text_ptr      13        text (strcpy → +0x80)
    15 = aux_c         14        slot_40
    16 = event         15        msg_id (u16 @ +0x08)
    17 = aux_d         16        userdata_id (u32 @ +0x48)
    18 = area_handle   17        parent_area (i16 arg to spawn_widget)

The JSON dumper labelled pos-10 "aux_a" and pos-11 "font" heuristically;
per the commit-4375e51 audit those positions are actually text_style
and label_ink respectively. The hand-port in `screen_wire_batch3.rs`
took the JSON labels at face value and swapped them — the tool's
generated output is CORRECT per the audit; see diff notes in commit
message.

Unresolved args (reg/expr/global_ref/scratch_ref of unknown addr, etc)
emit as `0` with an inline `// TODO(unresolved-arg): <detail>` comment.
"""
from __future__ import annotations

import argparse
import json
import os
import sys
from typing import Any, Optional


def val_or_none(arg: dict) -> Optional[int]:
    """Return literal int value, or None if not a literal."""
    v = arg.get("value", {})
    if "literal" in v:
        return v["literal"]
    return None


def val_desc(arg: dict) -> str:
    """Return a short comment describing an unresolved value."""
    v = arg.get("value", {})
    if "reg" in v:
        return f"reg {v['reg'].get('reg', '?')} unread"
    if "global_ref" in v:
        return f"global {v['global_ref']}"
    if "stack_addr" in v:
        sa = v["stack_addr"]
        return f"stack frame_offset={sa.get('frame_offset')}"
    if "scratch_ref" in v:
        sr = v["scratch_ref"]
        return f"scratch@{sr.get('addr')} = {sr.get('contents','')[:40]!r}"
    if "string" in v:
        return f"str {v['string'].get('text','')[:40]!r}"
    if "expr" in v:
        return f"expr {v['expr']}"
    return f"unknown {v!r}"


def as_i32_literal(arg: dict) -> tuple[str, Optional[str]]:
    """Return (rust_expr, todo_comment_or_none) for a numeric-ish arg."""
    v = arg.get("value", {})
    if "literal" in v:
        n = v["literal"]
        # Negative-int case (Ghidra encodes -1 as 0xffffffff sometimes;
        # trust literal_signed if present).
        if "literal_signed" in v:
            return (str(v["literal_signed"]), None)
        return (str(n), None)
    return ("0", f"TODO(unresolved-arg pos={arg['pos']} {arg['name']}): {val_desc(arg)}")


def as_bool_literal(arg: dict) -> tuple[str, Optional[str]]:
    lit = val_or_none(arg)
    if lit is None:
        return ("false", f"TODO(unresolved-arg pos={arg['pos']} {arg['name']}): {val_desc(arg)}")
    return (("true" if lit != 0 else "false"), None)


def area_handle_i16(arg: dict) -> tuple[str, Optional[str]]:
    """Return the i16 parent-area handle Rust expr for arg pos 17."""
    v = arg.get("value", {})
    if "literal_signed" in v:
        return (f"{v['literal_signed']}i16", None)
    if "literal" in v:
        n = v["literal"]
        if n == 0xFFFFFFFF or n == -1:
            return ("-1i16", None)
        return (f"{n}i16", None)
    return ("-1i16", f"TODO(unresolved-arg pos=17 area_handle): {val_desc(arg)}")


def emit_item(w: dict, scratch_text: Optional[str], out: list[str]) -> None:
    """Emit a spawn_widget for an `item` (FUN_00549580) call."""
    args = w["args"]
    # Build a positional lookup pos -> arg dict.
    by_pos = {a["pos"]: a for a in args}
    todos: list[str] = []

    def slot(pos: int, kind: str = "int") -> str:
        a = by_pos.get(pos)
        if a is None:
            return "0"
        if kind == "int":
            expr, todo = as_i32_literal(a)
        elif kind == "i16_parent":
            expr, todo = area_handle_i16(a)
        else:
            expr, todo = as_i32_literal(a)
        if todo:
            todos.append(todo)
        return expr

    # text: if arg pos 13 is a scratch_ref, use the tracked scratch text.
    text_arg = by_pos.get(13, {})
    v_text = text_arg.get("value", {})
    if "scratch_ref" in v_text:
        text_str = v_text["scratch_ref"].get("contents", "")
        text_expr = f'{json.dumps(text_str)}.into()'
    elif "string" in v_text:
        text_str = v_text["string"].get("text", "")
        text_expr = f'{json.dumps(text_str)}.into()'
    elif "literal" in v_text and scratch_text is not None:
        # literal pointer to dc723c → use tracked scratch
        text_expr = f'{json.dumps(scratch_text)}.into()'
    else:
        text_expr = 'String::new()'
        if v_text:
            todos.append(f"TODO(unresolved-text pos=13): {val_desc(text_arg)}")

    parent_expr = slot(17, "i16_parent")

    out.append(f"        // item @ {w['at_va']} — {w.get('call_target','FUN_00549580')}")
    for t in todos:
        out.append(f"        // {t}")
    out.append("        pool.spawn_widget(WidgetDescriptor {")
    out.append(f"            kind: {slot(0)} as u32,")
    out.append(f"            grid_x0: {slot(1)}, grid_y0: {slot(2)},")
    out.append(f"            grid_x1: {slot(3)}, grid_y1: {slot(4)},")
    out.append(f"            seq: {slot(5)}, row_index: {slot(6)},")
    out.append(f"            style_byte: {slot(7)} as u32,")
    out.append(f"            colour_a: ({slot(8)}i32) as u16,")
    out.append(f"            colour_b: ({slot(9)}i32) as u16,")
    out.append(f"            text_style: {slot(10)} as u32,")
    out.append(f"            label_ink: ({slot(11)}i32) as u16,")
    out.append(f"            pattern: ({slot(12)}i32) as u16,")
    out.append(f"            text: {text_expr},")
    out.append(f"            slot_40: {slot(14)},")
    out.append(f"            msg_id: {slot(15)},")
    out.append(f"            userdata_id: ({slot(16)}i64) as u32,")
    out.append(f"        }}, {parent_expr})?;")


def emit_area(w: dict, area_var: str, out: list[str]) -> None:
    """Emit a spawn_area for an `area` (FUN_00549790) call.

    FUN_00549790 signature (from JSON schema):
        pos 0..3   l,t,r,b
        pos 4      ncols
        pos 5      col_weights_ptr  → we pass empty Vec (extras)
        pos 6      nrows
        pos 7      aux_a            → color_slot
        pos 8      flags            → border_style
        pos 9      aux_b            → bg_color
        pos 10     terminator (-1)  → parent_area
    """
    args = w["args"]
    by_pos = {a["pos"]: a for a in args}
    todos: list[str] = []

    def slot(pos: int) -> str:
        a = by_pos.get(pos)
        if a is None:
            return "0"
        expr, todo = as_i32_literal(a)
        if todo:
            todos.append(todo)
        return expr

    out.append(f"        // area @ {w['at_va']} — FUN_00549790")
    for t in todos:
        out.append(f"        // {t}")
    out.append(f"        let _{area_var} = pool.spawn_area(")
    out.append(f"            {slot(0)} as i16, {slot(1)} as i16, {slot(2)} as i16, {slot(3)} as i16,")
    out.append(f"            {slot(4)} as u8, Vec::new(),                     // ncols + col_weights placeholder")
    out.append(f"            COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0,      // color_slot/grad/border/bg")
    out.append(f"            -1,                                              // parent")
    out.append(f"        )? as i16;")


def emit_sidebar(w: dict, out: list[str]) -> None:
    args = {a["pos"]: a for a in w["args"]}
    mode = val_or_none(args.get(0, {}))
    mode_expr = str(mode) if mode is not None else "4"
    out.append(f"        // sidebar @ {w['at_va']} — FUN_00745540(mode={mode_expr})")
    out.append(f"        spawn_sidebar_placeholder(pool, {mode_expr})?;")


def emit_navbar(w: dict, out: list[str]) -> None:
    args = {a["pos"]: a for a in w["args"]}
    back, back_todo = as_bool_literal(args.get(0, {"pos": 0, "name": "back_flag", "value": {}}))
    nxt, next_todo = as_bool_literal(args.get(1, {"pos": 1, "name": "next_flag", "value": {}}))
    out.append(f"        // nav_bar @ {w['at_va']} — FUN_005d75b0")
    if back_todo:
        out.append(f"        // {back_todo}")
    if next_todo:
        out.append(f"        // {next_todo}")
    out.append(f"        spawn_navbar_stub(pool, _root_area as i16, {back}, {nxt})?;")


def emit_menu_list(w: dict, out: list[str]) -> None:
    out.append(f"        // menu_list @ {w['at_va']} — FUN_005d6bf0 (TODO: dedicated port)")


def transliterate(jsonpath: str) -> str:
    d = json.load(open(jsonpath))
    addr = d.get("callback_va", "0x0").lstrip("0x") or "0"
    out: list[str] = []
    out.append(f"/// Auto-generated from `{os.path.basename(jsonpath)}` by")
    out.append(f"/// `tools/screen_to_rust.py`. DO NOT EDIT BY HAND —")
    out.append(f"/// regenerate to pick up JSON schema fixes.")
    out.append("///")
    out.append(f"/// Source: FUN_{addr} / callback_va {d.get('callback_va')} — {d.get('widget_count', 0)} widgets.")
    out.append(f"pub fn build_screen_{addr}(pool: &mut GuiRecordPool) -> Option<(usize, usize)> {{")
    out.append("    let a0 = pool.areas.len();")
    out.append("    let w0 = pool.widgets.len();")
    out.append("")
    out.append("    // Root area — every screen implicitly has one canvas the")
    out.append("    // substrate widgets attach to (the exe reuses DAT_00B59FC0).")
    out.append("    let _root_area = pool.spawn_area(")
    out.append("        0, 0, 799, 599, 0, Vec::new(),")
    out.append("        COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1,")
    out.append("    )? as i16;")
    out.append("")
    out.append("    {")

    scratch_text: Optional[str] = None
    area_ix = 0

    for w in d.get("widgets", []):
        kind = w.get("kind")
        if kind == "strcpy_scratch":
            args = {a["pos"]: a for a in w["args"]}
            src = args.get(1, {}).get("value", {})
            if "string" in src:
                scratch_text = src["string"].get("text", "")
                out.append(f"        // strcpy_scratch @ {w['at_va']}: scratch := {json.dumps(scratch_text)}")
            else:
                out.append(f"        // strcpy_scratch @ {w['at_va']}: (unresolved src)")
        elif kind == "sprintf_scratch":
            out.append(f"        // sprintf_scratch @ {w['at_va']}: {val_desc(w['args'][0]) if w.get('args') else '(no args)'}")
        elif kind == "area":
            area_ix += 1
            emit_area(w, f"area_{area_ix}", out)
        elif kind == "item":
            emit_item(w, scratch_text, out)
        elif kind == "sidebar":
            emit_sidebar(w, out)
        elif kind == "nav_bar":
            emit_navbar(w, out)
        elif kind == "menu_list":
            emit_menu_list(w, out)
        elif kind == "ret":
            out.append(f"        // ret @ {w['at_va']}")
        else:
            out.append(f"        // UNKNOWN widget kind {kind!r} @ {w.get('at_va','?')}")

    out.append("    }")
    out.append("")
    out.append("    Some((pool.areas.len() - a0, pool.widgets.len() - w0))")
    out.append("}")
    return "\n".join(out) + "\n"


def transliterate_many(paths: list[str]) -> str:
    header = [
        "//! Auto-generated screen builders from `tools/screen_to_rust.py`.",
        "//! DO NOT EDIT BY HAND — see the tool for source of truth.",
        "//!",
        "//! Each `build_screen_<addr>` fn is a direct transliteration of the",
        "//! widget-analysis JSON for that address; args map per the",
        "//! WidgetDescriptor field-map (see tool docstring).",
        "",
        "use crate::widget_pool::{GuiRecordPool, WidgetDescriptor};",
        "",
        "const COLOR_SLOT_PANEL: u8 = 7;",
        "const BORDER_STYLE_PANEL: u32 = 0x30;",
        "const LABEL_INK_STUB: u16 = 29596;",
        "",
        "fn spawn_sidebar_placeholder(pool: &mut GuiRecordPool, _mode: i32) -> Option<()> {",
        "    pool.spawn_area(0, 70, 99, 599, 0, Vec::new(),",
        "                    COLOR_SLOT_PANEL, 0, BORDER_STYLE_PANEL, 0, -1).map(|_| ())",
        "}",
        "",
        "fn spawn_navbar_stub(pool: &mut GuiRecordPool, parent: i16, back: bool, next: bool) -> Option<()> {",
        "    let mk = |pool: &mut GuiRecordPool, x0, x1, text: &str, seq, msg| -> Option<()> {",
        "        pool.spawn_widget(WidgetDescriptor {",
        "            kind: 2, grid_x0: x0, grid_y0: 555, grid_x1: x1, grid_y1: 585,",
        "            seq, row_index: 0, style_byte: 0x30, colour_a: 0, colour_b: 0,",
        "            text_style: 0x0C, label_ink: LABEL_INK_STUB, pattern: 0,",
        "            text: text.into(), slot_40: 0, msg_id: msg, userdata_id: 0,",
        "        }, parent).map(|_| ())",
        "    };",
        "    if back { mk(pool, 338, 500, \"Back\", 0, -2)?; }",
        "    if next { mk(pool, 685, 790, \"Next\", 1, -3)?; }",
        "    Some(())",
        "}",
        "",
    ]
    body = []
    for p in paths:
        body.append(transliterate(p))
        body.append("")
    return "\n".join(header) + "\n" + "\n".join(body)


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("json", nargs="+", help="Analysis JSON path(s)")
    ap.add_argument("-o", "--out", help="Write to this file instead of stdout")
    ap.add_argument("--fragment", action="store_true",
                    help="Emit just the fn bodies without module preamble")
    args = ap.parse_args()

    if args.fragment:
        text = "\n\n".join(transliterate(p) for p in args.json)
    else:
        text = transliterate_many(args.json)

    if args.out:
        with open(args.out, "w", encoding="utf-8") as f:
            f.write(text)
    else:
        sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
