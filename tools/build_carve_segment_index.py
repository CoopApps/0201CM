import json
from collections import Counter, defaultdict
from pathlib import Path


CARVE = Path("D:/cm0102-carve")
OUT = Path("D:/cm0102-rs/reports/carve_segment_index")


SEGMENTS = [
    (
        "01_execution_model",
        [
            "main",
            "app",
            "cm3",
            "game",
            "date",
            "time",
            "rng",
            "random",
            "scheduler",
            "menubar",
        ],
        "How the executable runs: startup, main loop, date/day tick, global state, screen dispatch.",
    ),
    (
        "02_data_model",
        [
            "database",
            "db",
            "load",
            "save",
            "club",
            "staff",
            "player",
            "nation",
            "stadium",
            "area",
            "contract",
        ],
        "What records exist, how they are loaded/saved, and what offsets/strides mean.",
    ),
    (
        "03_ui_renderer",
        [
            "gui",
            "screen",
            "menu",
            "font",
            "draw",
            "background",
            "picture",
            "bitmap",
            "cursor",
            "window",
        ],
        "Exact UI primitives, original screen composition, input regions, fonts, assets.",
    ),
    (
        "04_news_inbox",
        ["news", "message", "media", "press", "inbox"],
        "News generation, inbox ownership, templates, routing, unread state.",
    ),
    (
        "05_match_engine",
        ["match", "tactic", "formation", "commentary", "fixture"],
        "Match lifecycle, tactics, events, result writes, commentary/event queues.",
    ),
    (
        "06_transfers_contracts",
        ["transfer", "contract", "wage", "loan", "bid", "bosman", "work_permit"],
        "Offers, wages, AI decisions, registration, contract and transfer mutations.",
    ),
    (
        "07_competitions",
        [
            "league",
            "cup",
            "rules",
            "promotion",
            "relegation",
            "fixture",
            "award",
            "champ",
            "nations",
            "first",
            "second",
            "prem",
            "div",
        ],
        "League/cup state, schedules, standings, tie-breaks, promotion/relegation.",
    ),
    (
        "08_runtime_services",
        ["sound", "network", "file", "disk", "error", "debug", "memory", "thread"],
        "Services around the simulation: files, sound, debugging, allocators, platform calls.",
    ),
]


def load_json(name):
    return json.loads((CARVE / "ghidra_out/cm0102.exe" / name).read_text(encoding="utf-8", errors="replace"))


def classify(source_file):
    stem = source_file.lower()
    scored = []
    for name, tokens, _desc in SEGMENTS:
        score = sum(1 for token in tokens if token in stem)
        if score:
            scored.append((score, name))
    if not scored:
        return "09_unclassified"
    return sorted(scored, reverse=True)[0][1]


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    functions = {row["entry"].lower(): row for row in load_json("functions.json")}
    file_attr = load_json("file_attribution.json")
    strings = load_json("strings.json")
    callgraph = load_json("callgraph.json")

    functions_by_segment = defaultdict(set)
    files_by_segment = defaultdict(list)
    for source_file, addrs in file_attr.items():
        segment = classify(source_file)
        files_by_segment[segment].append(source_file)
        for addr in addrs:
            functions_by_segment[segment].add(addr.lower())

    incoming = Counter(edge["to"].lower() for edge in callgraph)
    outgoing = Counter(edge["from"].lower() for edge in callgraph)
    string_refs = Counter()
    for row in strings:
        for entry in row.get("in", []):
            string_refs[entry.lower()] += 1

    rows = []
    all_segments = [name for name, _tokens, _desc in SEGMENTS] + ["09_unclassified"]
    desc_by_segment = {name: desc for name, _tokens, desc in SEGMENTS}
    desc_by_segment["09_unclassified"] = "Functions/files without enough source filename evidence yet."
    for segment in all_segments:
        funcs = functions_by_segment[segment]
        known_funcs = [functions.get(addr) for addr in funcs if addr in functions]
        total_size = sum(int(row.get("size") or 0) for row in known_funcs if row)
        top = sorted(
            funcs,
            key=lambda addr: (incoming[addr] + outgoing[addr] + string_refs[addr], int(functions.get(addr, {}).get("size") or 0)),
            reverse=True,
        )[:25]
        rows.append(
            {
                "segment": segment,
                "description": desc_by_segment[segment],
                "source_files": sorted(files_by_segment[segment]),
                "source_file_count": len(files_by_segment[segment]),
                "function_count": len(funcs),
                "known_function_count": len(known_funcs),
                "total_function_bytes": total_size,
                "top_functions": [
                    {
                        "entry": addr,
                        "name": functions.get(addr, {}).get("name", "UNKNOWN"),
                        "size": functions.get(addr, {}).get("size"),
                        "fanin": incoming[addr],
                        "fanout": outgoing[addr],
                        "string_refs": string_refs[addr],
                    }
                    for addr in top
                ],
            }
        )

    (OUT / "segment_index.json").write_text(json.dumps(rows, indent=2), encoding="utf-8")

    lines = ["# CM0102 Carve Segment Index", ""]
    lines.append("This is the analysis order for turning the existing Ghidra/carver facts into exact remake evidence.")
    lines.append("")
    for row in rows:
        lines.append(f"## {row['segment']}")
        lines.append("")
        lines.append(row["description"])
        lines.append("")
        lines.append(f"- Source files: `{row['source_file_count']}`")
        lines.append(f"- Attributed functions: `{row['function_count']}`")
        lines.append(f"- Known function rows: `{row['known_function_count']}`")
        lines.append(f"- Total attributed bytes: `{row['total_function_bytes']}`")
        lines.append("- Top functions to ask/decompile first:")
        for fn in row["top_functions"][:10]:
            lines.append(
                f"- `{fn['entry']}` `{fn['name']}` size `{fn['size']}` fanin `{fn['fanin']}` fanout `{fn['fanout']}` strings `{fn['string_refs']}`"
            )
        lines.append("")

    (OUT / "segment_index.md").write_text("\n".join(lines), encoding="utf-8")


if __name__ == "__main__":
    main()
