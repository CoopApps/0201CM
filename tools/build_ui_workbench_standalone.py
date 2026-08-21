from __future__ import annotations

import json
from pathlib import Path


ROOT = Path("D:/cm0102-rs")
SOURCE_HTML = ROOT / "cm0102_ui_workbench.html"
SOURCE_JSON = ROOT / "reports" / "cm0102_ui_specs.json"
SOURCE_FONTS = ROOT / "assets" / "cm0102" / "fonts" / "font_manifest.json"
OUTPUT_HTML = ROOT / "cm0102_ui_workbench_standalone.html"


def main() -> None:
    html = SOURCE_HTML.read_text(encoding="utf-8")
    spec = json.loads(SOURCE_JSON.read_text(encoding="utf-8"))
    fonts = json.loads(SOURCE_FONTS.read_text(encoding="utf-8"))
    embedded = json.dumps(spec, separators=(",", ":"))
    embedded_fonts = json.dumps(fonts, separators=(",", ":"))
    html = html.replace("const specUrl = 'reports/cm0102_ui_specs.json';", "const specUrl = null;")
    html = html.replace(
        "const fontUrl = 'assets/cm0102/fonts/font_manifest.json';", "const fontUrl = null;"
    )
    html = html.replace("let spec = null;", f"let spec = {embedded};")
    html = html.replace("let fontManifest = null;", f"let fontManifest = {embedded_fonts};")
    html = html.replace(
        "spec = await fetch(specUrl).then(response => response.json());",
        "if (specUrl) spec = await fetch(specUrl).then(response => response.json());",
    )
    html = html.replace(
        "fontManifest = await fetch(fontUrl).then(response => response.json());",
        "if (fontUrl) fontManifest = await fetch(fontUrl).then(response => response.json());",
    )
    OUTPUT_HTML.write_text(html, encoding="utf-8")
    print(OUTPUT_HTML)


if __name__ == "__main__":
    main()
