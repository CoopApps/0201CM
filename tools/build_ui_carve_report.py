from __future__ import annotations

import json
from collections import Counter, defaultdict
from pathlib import Path


CARVE_ROOT = Path("D:/cm0102-carve")
OUT = Path("D:/cm0102-rs/reports/ui_replica_workbench.json")


UI_TERMS = [
    "directdraw",
    "gdi",
    "blt",
    "bitmap",
    "font",
    "arial",
    ".rgn",
    ".fnt",
    "button",
    "menu",
    "screen",
    "window",
    "draw",
    "textout",
    "manager",
    "squad",
    "fixtures",
    "tactics",
    "inbox",
    "news",
]


def read_json(path: Path):
    return json.loads(path.read_text(encoding="utf-8", errors="replace"))


def lower_values(value):
    if isinstance(value, str):
        yield value.lower()
    elif isinstance(value, dict):
        for item in value.values():
            yield from lower_values(item)
    elif isinstance(value, list):
        for item in value:
            yield from lower_values(item)


def contains_any_ui_term(value) -> list[str]:
    text = "\n".join(lower_values(value))
    return sorted({term for term in UI_TERMS if term in text})


def load_optional_json(path: Path):
    if path.exists():
        return read_json(path)
    return []


def row_address(row) -> str:
    for key in ["entry", "address", "addr", "from", "to", "function"]:
        value = row.get(key) if isinstance(row, dict) else None
        if isinstance(value, str) and value.startswith("0x"):
            return value.lower()
        if isinstance(value, int):
            return f"0x{value:08x}"
    return ""


def main() -> None:
    ghidra = CARVE_ROOT / "ghidra_out" / "cm0102.exe"
    strings = load_optional_json(ghidra / "strings.json")
    functions = load_optional_json(ghidra / "functions.json")
    xrefs = load_optional_json(ghidra / "xrefs.json")
    data_symbols = load_optional_json(ghidra / "data_symbols.json")
    claims = load_optional_json(CARVE_ROOT / "claims.json")
    findings = load_optional_json(CARVE_ROOT / "findings.json")

    string_hits = []
    for row in strings if isinstance(strings, list) else strings.get("strings", []):
        terms = contains_any_ui_term(row)
        if terms:
            string_hits.append({"terms": terms, "row": row})

    function_hits = []
    for row in functions if isinstance(functions, list) else functions.get("functions", []):
        terms = contains_any_ui_term(row)
        if terms:
            function_hits.append({"address": row_address(row), "terms": terms, "row": row})

    symbol_hits = []
    for row in data_symbols if isinstance(data_symbols, list) else data_symbols.get("symbols", []):
        terms = contains_any_ui_term(row)
        if terms:
            symbol_hits.append({"address": row_address(row), "terms": terms, "row": row})

    known_hits = []
    for source_name, source in [("claims", claims), ("findings", findings)]:
        rows = source if isinstance(source, list) else source.get("claims", source.get("findings", []))
        for row in rows:
            terms = contains_any_ui_term(row)
            if terms:
                known_hits.append(
                    {
                        "source": source_name,
                        "address": row_address(row),
                        "terms": terms,
                        "row": row,
                    }
                )

    candidate_counter = Counter()
    function_string_hits = defaultdict(list)
    for hit in string_hits:
        row = hit["row"]
        for source in row.get("in", []):
            if isinstance(source, str) and source.startswith("0x"):
                source = source.lower()
                candidate_counter[source] += 1 + len(hit["terms"])
                function_string_hits[source].append(
                    {
                        "string": row.get("s", ""),
                        "string_addr": row.get("addr", ""),
                        "terms": hit["terms"],
                    }
                )
    for hit in function_hits:
        if hit["address"]:
            candidate_counter[hit["address"]] += 3
    for hit in known_hits:
        if hit["address"]:
            candidate_counter[hit["address"]] += 5

    asset_manifest_path = Path("D:/cm0102-rs/assets/cm0102/asset_manifest.json")
    asset_manifest = read_json(asset_manifest_path) if asset_manifest_path.exists() else {}
    image_assets = [
        asset
        for asset in asset_manifest.get("assets", [])
        if asset.get("kind") == "image"
    ]

    report = {
        "format": "cm0102-rs-ui-replica-workbench",
        "version": 1,
        "source": {
            "carve_root": str(CARVE_ROOT),
            "ghidra_facts": str(ghidra),
            "asset_manifest": str(asset_manifest_path),
            "method": "Fact-mined UI/render candidates; exact UI semantics still require targeted decompile and screenshot/runtime validation.",
        },
        "summary": {
            "string_hits": len(string_hits),
            "function_hits": len(function_hits),
            "symbol_hits": len(symbol_hits),
            "known_claim_or_finding_hits": len(known_hits),
            "candidate_functions": len(candidate_counter),
            "exported_image_assets": len(image_assets),
        },
        "top_candidate_functions": [
            {
                "address": address,
                "score": score,
                "strings": function_string_hits.get(address, [])[:20],
            }
            for address, score in candidate_counter.most_common(80)
        ],
        "screen_candidate_clusters": {
            "startup_rgn_loader": {
                "address": "0x005b6f10",
                "evidence": [
                    hit
                    for hit in function_string_hits.get("0x005b6f10", [])
                    if ".rgn" in hit["string"].lower()
                ],
            },
            "font_loader": {
                "address": "0x005ce750",
                "evidence": function_string_hits.get("0x005ce750", []),
            },
            "squad_screen": {
                "address": "0x00457200",
                "evidence": function_string_hits.get("0x00457200", []),
            },
            "fixtures_screen": {
                "addresses": ["0x00498f50", "0x004559a9", "0x00455b28"],
                "evidence": (
                    function_string_hits.get("0x00498f50", [])
                    + function_string_hits.get("0x004559a9", [])
                    + function_string_hits.get("0x00455b28", [])
                ),
            },
            "news_screen": {
                "addresses": ["0x0045dbc0", "0x00460023", "0x004601c2"],
                "evidence": (
                    function_string_hits.get("0x0045dbc0", [])
                    + function_string_hits.get("0x00460023", [])
                    + function_string_hits.get("0x004601c2", [])
                ),
            },
            "tactics_screen": {
                "addresses": ["0x0088a850", "0x00455ba0", "0x0046948e"],
                "evidence": (
                    function_string_hits.get("0x0088a850", [])
                    + function_string_hits.get("0x00455ba0", [])
                    + function_string_hits.get("0x0046948e", [])
                ),
            },
        },
        "string_hits": string_hits[:300],
        "function_hits": function_hits[:120],
        "symbol_hits": symbol_hits[:120],
        "known_claim_or_finding_hits": known_hits[:120],
        "ui_asset_groups": {
            "data_images": [
                asset for asset in image_assets if asset.get("relative_source", "").startswith("Data/")
            ],
            "screen_backgrounds": [
                asset for asset in image_assets if asset.get("relative_source", "").startswith("Pictures/")
            ],
        },
        "next_steps": [
            "Run targeted Ghidra decompile for top render/input candidates.",
            "Find calls that load Data/*.fnt and Pictures/*.rgn to bind screens to assets.",
            "Capture or reconstruct key 800x600 screen rectangles: manager home, squad, fixtures, table, inbox.",
            "Build cm0102_ui.html using exported PNGs and the Rust manager APIs.",
        ],
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report["summary"], indent=2))
    print(OUT)


if __name__ == "__main__":
    main()
