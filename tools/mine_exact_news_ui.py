import json
from pathlib import Path

import pefile
from capstone import CS_ARCH_X86, CS_MODE_32, Cs
from capstone.x86_const import X86_OP_IMM


EXE = Path("D:/cm0102/cm0102.exe")
CARVE = Path("D:/cm0102-carve")
OUT_DIR = Path("D:/cm0102-rs/reports")

CALL_LABELS = {
    0x00549580: "draw_record",
    0x00549790: "region_record",
    0x005D7070: "gui_tab_list",
    0x006547C0: "string_copy",
    0x0076EF80: "news_unread_counter",
    0x005CF7B0: "font_height",
}

ANCHORS = {
    "main_menu_chrome": {
        "entry": "0x00745540",
        "source": "carve: menubar.cpp::sub_00745540",
        "facts": [
            "creates the left chrome region at x=0, y=0, x2=0x59, y2=599",
            "blits Data/game.mbr through the draw queue after loading game.mbr",
            "draws left-menu entries through FUN_00549580 and selectable regions through FUN_00549790",
        ],
    },
    "news_reader_constructor": {
        "entry": "0x0076ffb0",
        "source": "carve: news.cpp allocation/initializer near exact news reader",
        "facts": [
            "registers news UI callbacks through FUN_007e6430",
            "initializes news state slots 0..8 and row triples up to 0x21",
        ],
    },
    "news_category_tabs": {
        "entry": "0x00770539..0x00770698",
        "source": "static disassembly of cm0102.exe around exact string xrefs",
        "facts": [
            "packs 8 category records: All, Messages, Competitions, Injuries and Bans, Contracts and Media, Transfers, Jobs, Records",
            "sets tab layout state min x=0x6e, max x=0x30c, initial y from caller, bottom y=y+0x5a",
            "hands the category model to gui_utils.cpp::sub_005d7070 for exact region/draw generation",
        ],
    },
    "news_filter_and_next_unread": {
        "entry": "0x00770dde..0x00770f69",
        "source": "static disassembly of cm0102.exe around Filter and Next Unread xrefs",
        "facts": [
            "creates filter region x=0x195, y=base+0x5f, x2=0x28f, y2=base+0x73",
            "draws label text 'Filter :' into that region",
            "uses news.cpp::sub_0076ef80 to decide whether Next Unread is enabled before drawing the button",
        ],
    },
}

EXACT_UI_STRINGS = {
    0x00A5548C: "All<%s - COMMENT - all news categories>",
    0x00A55480: "Messages",
    0x0097B610: "Competitions",
    0x00A5546C: "Injuries and Bans",
    0x00A55458: "Contracts and Media",
    0x0097B168: "Transfers",
    0x00A55450: "Jobs",
    0x0097D35C: "Records",
    0x00A5543C: "Filter :",
    0x00A55430: "Next Unread",
    0x00A46364: "You Have\nNews",
}


def load_strings():
    rows = json.loads((CARVE / "ghidra_out/cm0102.exe/strings.json").read_text(encoding="utf-8"))
    strings_by_addr = {int(row["addr"], 16): row["s"] for row in rows}
    # Ghidra did not export the tiny "Jobs" literal at 0x00a55450, but the PE bytes do.
    strings_by_addr.update(EXACT_UI_STRINGS)
    return rows, strings_by_addr


def disassemble_range(pe, start, end, strings_by_addr):
    base = pe.OPTIONAL_HEADER.ImageBase
    image = pe.get_memory_mapped_image()
    code = image[start - base : end - base]
    md = Cs(CS_ARCH_X86, CS_MODE_32)
    md.detail = True
    instructions = []
    for insn in md.disasm(code, start):
        annotations = []
        for op in insn.operands:
            if op.type != X86_OP_IMM:
                continue
            imm = op.imm & 0xFFFFFFFF
            if imm in strings_by_addr:
                annotations.append({"kind": "string", "addr": f"0x{imm:08x}", "value": strings_by_addr[imm]})
            if insn.mnemonic == "call" and imm in CALL_LABELS:
                annotations.append({"kind": "call", "addr": f"0x{imm:08x}", "value": CALL_LABELS[imm]})
        if annotations:
            instructions.append(
                {
                    "addr": f"0x{insn.address:08x}",
                    "mnemonic": insn.mnemonic,
                    "op_str": insn.op_str,
                    "annotations": annotations,
                }
            )
    return instructions


def main():
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    string_rows, strings_by_addr = load_strings()
    pe = pefile.PE(str(EXE))

    rows_by_addr = {int(row["addr"], 16): row for row in string_rows}
    string_hits = []
    for addr, value in EXACT_UI_STRINGS.items():
        row = rows_by_addr.get(addr)
        string_hits.append(
            {
                "addr": f"0x{addr:08x}",
                "value": value,
                "xrefs": row.get("in", []) if row else ["static byte string at address"],
            }
        )

    disassembly = {
        "news_category_tabs": disassemble_range(pe, 0x00770530, 0x007706A0, strings_by_addr),
        "news_filter_and_next_unread": disassemble_range(pe, 0x00770DDE, 0x00770F70, strings_by_addr),
    }

    report = {
        "status": "exact-replica evidence, not visual approximation",
        "original_exe": str(EXE),
        "anchors": ANCHORS,
        "string_hits": string_hits,
        "annotated_disassembly": disassembly,
        "deprecated": {
            "file": "D:/cm0102-rs/cm0102_exact_squad_slice.html",
            "reason": "prototype slice used inferred layout and is not an exact CM0102 screen replay",
        },
    }

    (OUT_DIR / "cm0102_exact_news_ui_evidence.json").write_text(json.dumps(report, indent=2), encoding="utf-8")

    lines = ["# CM0102 Exact News UI Evidence", ""]
    lines.append("Status: exact-replica evidence mined from `cm0102.exe`, not a visual approximation.")
    lines.append("")
    lines.append("## Anchors")
    lines.append("")
    for name, anchor in ANCHORS.items():
        lines.append(f"- `{name}` `{anchor['entry']}`: {anchor['source']}")
        for fact in anchor["facts"]:
            lines.append(f"- Fact: {fact}")
    lines.append("")
    lines.append("## Exact Strings")
    lines.append("")
    for hit in string_hits:
        lines.append(f"- `{hit['addr']}` `{hit['value']}` via {', '.join(hit['xrefs'])}")
    lines.append("")
    lines.append("## Deprecated Prototype")
    lines.append("")
    lines.append("- `D:/cm0102-rs/cm0102_exact_squad_slice.html` is deprecated as an exact-replica target.")
    lines.append("- Reason: it used inferred layout; next render must replay the original draw/region sequence.")
    lines.append("")
    lines.append("## Outputs")
    lines.append("")
    lines.append("- JSON: `D:/cm0102-rs/reports/cm0102_exact_news_ui_evidence.json`")
    (OUT_DIR / "cm0102_exact_news_ui_evidence.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


if __name__ == "__main__":
    main()
