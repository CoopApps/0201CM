import argparse
import json
from collections import Counter
from pathlib import Path

import pefile
from capstone import Cs, CS_ARCH_X86, CS_MODE_32
from capstone.x86_const import X86_OP_IMM, X86_OP_MEM, X86_OP_REG


DEFAULT_EXE = Path("D:/cm0102/cm0102.exe")
DEFAULT_CARVE = Path("D:/cm0102-carve")
DEFAULT_OUT = Path("D:/cm0102-rs/reports/exe_walk")


def norm_addr(value):
    if isinstance(value, int):
        return f"0x{value:08x}"
    text = str(value).strip().lower()
    if text.startswith("0x"):
        return f"0x{int(text, 16):08x}"
    return f"0x{int(text):08x}"


def load_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def source_spans(subsystems):
    spans = []
    for source_file, info in subsystems.items():
        span = info.get("span") or []
        if len(span) != 2:
            continue
        try:
            spans.append((int(span[0], 16), int(span[1], 16), source_file))
        except ValueError:
            continue
    return sorted(spans)


def source_for_addr(addr, spans):
    lo = 0
    hi = len(spans) - 1
    while lo <= hi:
        mid = (lo + hi) // 2
        start, end, source_file = spans[mid]
        if start <= addr <= end:
            return source_file
        if addr < start:
            hi = mid - 1
        else:
            lo = mid + 1
    return ""


def classify_subsystem(text):
    value = text.lower()
    rules = [
        ("match_engine", ["match", "fixture", "event_queue", "commentary", "goal", "penalty", "match_eng", "match_pl", "match_events"]),
        ("transfers_contracts", ["transfer", "contract", "wage", "bid", "scout", "loan", "shortlist"]),
        ("competitions", ["league", "cup", "competition", "promotion", "relegation", "rules", "nations", "division", "prem"]),
        ("news_inbox", ["news", "inbox", "message"]),
        ("ui_renderer", ["screen", "draw", "button", "font", "bitmap", "sprite", "window", "menu", "dialog", "directdraw", "gdi", "scrman"]),
        ("data_model", ["db_", "record", ".dat", "load", "save", "staff", "club", "nation", "player", "person"]),
        ("runtime_services", ["random", "rng", "date", "malloc", "heap", "free", "path", "qsort", "string", "sort", "file"]),
    ]
    scored = []
    for name, terms in rules:
        score = sum(1 for term in terms if term in value)
        if score:
            scored.append((score, name))
    if not scored:
        return "unclassified"
    return sorted(scored, reverse=True)[0][1]


def decode_function(exe_bytes, pe, image_base, function, decoder):
    entry = int(function["entry"], 16)
    size = int(function.get("size") or 0)
    offset = pe.get_offset_from_rva(entry - image_base)
    code = exe_bytes[offset : offset + size]
    instructions = []
    for ins in decoder.disasm(code, entry):
        operands = []
        for op in ins.operands:
            if op.type == X86_OP_IMM:
                operands.append({"type": "imm", "value": op.imm})
            elif op.type == X86_OP_MEM:
                operands.append(
                    {
                        "type": "mem",
                        "base": ins.reg_name(op.mem.base),
                        "index": ins.reg_name(op.mem.index),
                        "scale": op.mem.scale,
                        "disp": op.mem.disp,
                    }
                )
            elif op.type == X86_OP_REG:
                operands.append({"type": "reg", "reg": ins.reg_name(op.reg)})
        instructions.append(
            {
                "addr": norm_addr(ins.address),
                "size": ins.size,
                "bytes": ins.bytes.hex(),
                "mnemonic": ins.mnemonic,
                "op_str": ins.op_str,
                "operands": operands,
            }
        )
    return instructions


def load_existing_state(path):
    if not path.exists():
        return {"version": 1, "reviewed": {}, "notes": {}}
    return load_json(path)


def render_slice_markdown(slice_row, function_row):
    lines = [
        f"# EXE Walk Slice `{slice_row['slice_id']}`",
        "",
        f"- Function: `{slice_row['function_addr']}`",
        f"- Function name: `{function_row.get('name') or '(unnamed)'}`",
        f"- Status: `{function_row.get('status')}`",
        f"- Subsystem: `{function_row.get('subsystem')}`",
        f"- Source file: `{function_row.get('source_file') or '(none)'}`",
        f"- Instruction range: `{slice_row['start_addr']}` to `{slice_row['end_addr']}`",
        f"- Slice ordinal: `{slice_row['slice_ordinal']}` of `{function_row.get('slice_count')}`",
        "",
        "## Instructions",
        "",
        "| Addr | Bytes | Instruction |",
        "|---|---|---|",
    ]
    for ins in slice_row["instructions"]:
        inst = f"{ins['mnemonic']} {ins['op_str']}".strip()
        lines.append(f"| `{ins['addr']}` | `{ins['bytes']}` | `{inst}` |")
    lines.extend(
        [
            "",
            "## Review Template",
            "",
            "- Meaning:",
            "- Inputs/read offsets:",
            "- Writes/side effects:",
            "- Calls/branch purpose:",
            "- Evidence/provenance:",
            "- Follow-up:",
        ]
    )
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description="Build a 10-15 instruction canonical EXE walk index.")
    parser.add_argument("--exe", type=Path, default=DEFAULT_EXE)
    parser.add_argument("--carve", type=Path, default=DEFAULT_CARVE)
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    parser.add_argument("--slice-size", type=int, default=15)
    args = parser.parse_args()

    args.out.mkdir(parents=True, exist_ok=True)

    functions = load_json(args.carve / "analysis" / "functions.json")
    callgraph = load_json(args.carve / "analysis" / "callgraph.json")
    subsystems = load_json(args.carve / "analysis" / "subsystems.json")
    findings = load_json(args.carve / "findings.json")
    spans = source_spans(subsystems)
    verified = {key.lower(): value for key, value in findings.get("verified", {}).items()}

    callers = {norm_addr(key): len(value.get("callers", [])) for key, value in callgraph.items()}
    exe_bytes = args.exe.read_bytes()
    pe = pefile.PE(str(args.exe), fast_load=True)
    image_base = pe.OPTIONAL_HEADER.ImageBase
    decoder = Cs(CS_ARCH_X86, CS_MODE_32)
    decoder.detail = True

    state_path = args.out / "walk_state.json"
    state = load_existing_state(state_path)

    slice_path = args.out / "walk_index.jsonl"
    function_path = args.out / "function_index.json"
    current_path = args.out / "current_slice.md"

    function_rows = []
    slice_count = 0
    instruction_count = 0
    status_counts = Counter()
    subsystem_counts = Counter()

    with slice_path.open("w", encoding="utf-8", newline="\n") as out_file:
        for function in functions:
            entry = norm_addr(function["entry"])
            entry_int = int(entry, 16)
            source_file = function.get("source_file") or source_for_addr(entry_int, spans)
            proposed = function.get("proposed") or ""
            verified_name = verified.get(entry, {}).get("name")
            name = verified_name or proposed
            if verified_name:
                status = "verified"
            elif proposed or (function.get("confidence") or "").upper() in {"INFERRED", "PROPOSED"}:
                status = "named_unverified"
            else:
                status = "unknown"
            subsystem = classify_subsystem(" ".join([entry, name, source_file, function.get("evidence", "")]))
            instructions = decode_function(exe_bytes, pe, image_base, function, decoder)
            slices_for_function = []
            for ordinal, start in enumerate(range(0, len(instructions), args.slice_size)):
                chunk = instructions[start : start + args.slice_size]
                if not chunk:
                    continue
                slice_id = f"{entry}:{ordinal:04d}"
                row = {
                    "slice_id": slice_id,
                    "function_addr": entry,
                    "slice_ordinal": ordinal,
                    "start_addr": chunk[0]["addr"],
                    "end_addr": chunk[-1]["addr"],
                    "instruction_count": len(chunk),
                    "review_status": "reviewed" if slice_id in state.get("reviewed", {}) else "unreviewed",
                    "instructions": chunk,
                }
                out_file.write(json.dumps(row, separators=(",", ":")) + "\n")
                slices_for_function.append(slice_id)
                slice_count += 1
                instruction_count += len(chunk)
            function_row = {
                "addr": entry,
                "end": norm_addr(function["end"]),
                "size": int(function.get("size") or 0),
                "instruction_count": len(instructions),
                "slice_count": len(slices_for_function),
                "status": status,
                "name": name,
                "source_file": source_file,
                "subsystem": subsystem,
                "callers": callers.get(entry, 0),
                "callees": len(function.get("callees", [])),
                "slices": slices_for_function,
            }
            function_rows.append(function_row)
            status_counts[status] += 1
            subsystem_counts[subsystem] += 1

    function_path.write_text(json.dumps(function_rows, indent=2), encoding="utf-8")
    state_path.write_text(json.dumps(state, indent=2), encoding="utf-8")

    first_slice = None
    function_by_addr = {row["addr"]: row for row in function_rows}
    with slice_path.open("r", encoding="utf-8") as in_file:
        for line in in_file:
            row = json.loads(line)
            if row["review_status"] == "unreviewed":
                first_slice = row
                break
    if first_slice:
        current_path.write_text(render_slice_markdown(first_slice, function_by_addr[first_slice["function_addr"]]), encoding="utf-8")

    summary = [
        "# CM0102 EXE Walk",
        "",
        "Canonical complete-map index built from x86 disassembly of `D:/cm0102/cm0102.exe`.",
        "",
        f"- Slice size: `{args.slice_size}` instructions",
        f"- Functions: `{len(function_rows)}`",
        f"- Instructions decoded: `{instruction_count}`",
        f"- Slices: `{slice_count}`",
        f"- State file: `D:/cm0102-rs/reports/exe_walk/walk_state.json`",
        f"- Current slice: `D:/cm0102-rs/reports/exe_walk/current_slice.md`",
        "",
        "## Function Status",
        "",
        "| Status | Functions |",
        "|---|---:|",
    ]
    for status, count in status_counts.most_common():
        summary.append(f"| `{status}` | {count} |")
    summary.extend(["", "## Subsystems", "", "| Subsystem | Functions |", "|---|---:|"])
    for subsystem, count in subsystem_counts.most_common():
        summary.append(f"| `{subsystem}` | {count} |")
    summary.extend(
        [
            "",
            "## Commands",
            "",
            "```powershell",
            "D:/python312/python.exe D:/cm0102-rs/tools/exe_walk/next_slice.py",
            "D:/python312/python.exe D:/cm0102-rs/tools/exe_walk/next_slice.py --subsystem match_engine",
            "D:/python312/python.exe D:/cm0102-rs/tools/exe_walk/next_slice.py --function 0x006dc600",
            "D:/python312/python.exe D:/cm0102-rs/tools/exe_walk/next_slice.py --mark-reviewed '<slice_id>' --note 'verified meaning here'",
            "```",
        ]
    )
    (args.out / "README.md").write_text("\n".join(summary) + "\n", encoding="utf-8")

    print(json.dumps({"functions": len(function_rows), "instructions": instruction_count, "slices": slice_count, "out": str(args.out)}, indent=2))


if __name__ == "__main__":
    main()
