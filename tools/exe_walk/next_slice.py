import argparse
import json
from pathlib import Path


DEFAULT_OUT = Path("D:/cm0102-rs/reports/exe_walk")


def load_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def load_state(path):
    if not path.exists():
        return {"version": 1, "reviewed": {}, "notes": {}}
    return load_json(path)


def save_state(path, state):
    path.write_text(json.dumps(state, indent=2), encoding="utf-8")


def render_slice_markdown(slice_row, function_row, note=None):
    lines = [
        f"# EXE Walk Slice `{slice_row['slice_id']}`",
        "",
        f"- Function: `{slice_row['function_addr']}`",
        f"- Function name: `{function_row.get('name') or '(unnamed)'}`",
        f"- Function status: `{function_row.get('status')}`",
        f"- Subsystem: `{function_row.get('subsystem')}`",
        f"- Source file: `{function_row.get('source_file') or '(none)'}`",
        f"- Callers/Callees: `{function_row.get('callers')}` / `{function_row.get('callees')}`",
        f"- Instruction range: `{slice_row['start_addr']}` to `{slice_row['end_addr']}`",
        f"- Slice ordinal: `{slice_row['slice_ordinal']}` of `{function_row.get('slice_count')}`",
        f"- Review status: `{slice_row.get('review_status')}`",
    ]
    if note:
        lines.append(f"- Existing note: {note}")
    lines.extend(["", "## Instructions", "", "| Addr | Bytes | Instruction |", "|---|---|---|"])
    for ins in slice_row["instructions"]:
        instruction = f"{ins['mnemonic']} {ins['op_str']}".strip()
        lines.append(f"| `{ins['addr']}` | `{ins['bytes']}` | `{instruction}` |")
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


def iter_slices(index_path):
    with index_path.open("r", encoding="utf-8") as handle:
        for line in handle:
            if line.strip():
                yield json.loads(line)


def main():
    parser = argparse.ArgumentParser(description="Read or mark a 10-15 instruction EXE walk slice.")
    parser.add_argument("--out", type=Path, default=DEFAULT_OUT)
    parser.add_argument("--function", help="Restrict to a function address such as 0x006dc600.")
    parser.add_argument("--subsystem", help="Restrict to a subsystem such as match_engine.")
    parser.add_argument("--status", choices=["verified", "named_unverified", "unknown"], help="Restrict by function status.")
    parser.add_argument("--slice-id", help="Render a specific slice id.")
    parser.add_argument("--mark-reviewed", help="Mark a slice id as reviewed.")
    parser.add_argument("--note", default="", help="Review note when marking reviewed.")
    parser.add_argument("--limit", type=int, default=1, help="Number of matching unreviewed slices to render.")
    args = parser.parse_args()

    index_path = args.out / "walk_index.jsonl"
    state_path = args.out / "walk_state.json"
    current_path = args.out / "current_slice.md"
    if not index_path.exists():
        raise SystemExit(f"Missing {index_path}; run build_walk_index.py first.")

    functions = load_json(args.out / "function_index.json")
    function_by_addr = {row["addr"].lower(): row for row in functions}
    state = load_state(state_path)

    if args.mark_reviewed:
        state.setdefault("reviewed", {})[args.mark_reviewed] = True
        if args.note:
            state.setdefault("notes", {})[args.mark_reviewed] = args.note
        save_state(state_path, state)
        print(f"marked reviewed: {args.mark_reviewed}")
        return

    matches = []
    reviewed = state.get("reviewed", {})
    for row in iter_slices(index_path):
        function_row = function_by_addr.get(row["function_addr"].lower())
        if not function_row:
            continue
        row["review_status"] = "reviewed" if row["slice_id"] in reviewed else "unreviewed"
        if args.slice_id and row["slice_id"] != args.slice_id:
            continue
        if args.function and row["function_addr"].lower() != args.function.lower():
            continue
        if args.subsystem and function_row.get("subsystem") != args.subsystem:
            continue
        if args.status and function_row.get("status") != args.status:
            continue
        if not args.slice_id and row["review_status"] == "reviewed":
            continue
        matches.append((row, function_row))
        if len(matches) >= args.limit:
            break

    if not matches:
        print("No matching slices.")
        return

    rendered = []
    for row, function_row in matches:
        note = state.get("notes", {}).get(row["slice_id"])
        rendered.append(render_slice_markdown(row, function_row, note))
    text = "\n---\n\n".join(rendered)
    current_path.write_text(text, encoding="utf-8")
    print(text)
    print(f"wrote {current_path}")


if __name__ == "__main__":
    main()
