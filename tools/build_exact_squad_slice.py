from __future__ import annotations

import base64
import json
from pathlib import Path


ROOT = Path("D:/cm0102-rs")
FONT_DIR = ROOT / "assets" / "cm0102" / "fonts"
SPEC_PATH = ROOT / "reports" / "cm0102_ui_specs.json"
SQUAD_PATH = ROOT / "reports" / "manager_squad_api_smoke.json"
STRIP_PATH = ROOT / "assets" / "cm0102" / "Data" / "game.png"
OUTPUT = ROOT / "cm0102_exact_squad_slice.html"


def load_font(name: str) -> dict:
    return json.loads((FONT_DIR / f"{name}.json").read_text(encoding="utf-8"))


def main() -> None:
    fonts = {
        "arial_narrow_10": load_font("arial_narrow_10"),
        "arial_narrow_11": load_font("arial_narrow_11"),
        "arial_14": load_font("arial_14"),
        "arial_16": load_font("arial_16"),
    }
    spec = json.loads(SPEC_PATH.read_text(encoding="utf-8"))
    squad = json.loads(SQUAD_PATH.read_text(encoding="utf-8"))
    strip_data_url = (
        "data:image/png;base64,"
        + base64.b64encode(STRIP_PATH.read_bytes()).decode("ascii")
    )
    payload = {
        "fonts": fonts,
        "squad": squad,
        "strip": strip_data_url,
        "evidence": {
            "screen": "squad_screen",
            "regions": [
                {
                    "name": "top_left_tab_region",
                    "source": "0x00457200:556",
                    "x1": 0x6E,
                    "x2": 0x168,
                    "height": 0x14,
                    "y": "param_1",
                    "status": "x/height code-derived; y caller-dependent",
                },
                {
                    "name": "top_right_tab_region",
                    "source": "0x00457200:1438",
                    "x1": 0x212,
                    "x2": 0x30C,
                    "height": 0x14,
                    "y": "param_1",
                    "status": "x/height code-derived; y caller-dependent",
                },
                {
                    "name": "main_list_region",
                    "source": "0x00457200:1854/1857",
                    "x1": 0x6E,
                    "x2": 0x30C,
                    "y1": "param_1",
                    "y2": "param_2",
                    "status": "x code-derived; y range caller-dependent",
                },
            ],
            "font_metrics": {
                "height": "0x005cf7b0",
                "width": "0x005cf610",
                "pair_spacing": "0x005cf840",
                "decoder": "D:/cm0102-rs/tools/decode_cm0102_fonts.py",
            },
            "labels": [
                label
                for label in spec["labels_by_screen"].get("squad_screen", [])
                if label.strip()
                in {
                    "Traditional",
                    "Contract",
                    "Selection",
                    "Stats",
                    "More Stats",
                    "Attributes",
                    "Other",
                    "Name",
                    "Position",
                    "Squad Number",
                    "Form",
                    "Morale",
                    "Condition",
                    "Basic Wage",
                    "Contract Expiry",
                    "Value",
                    "Goals",
                    "Assists",
                    "Av. Rating",
                }
            ],
        },
    }
    html = HTML_TEMPLATE.replace("__PAYLOAD__", json.dumps(payload, separators=(",", ":")))
    OUTPUT.write_text(html, encoding="utf-8")
    print(OUTPUT)


HTML_TEMPLATE = r"""<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>CM0102 Exact Squad Slice</title>
  <style>
    * { box-sizing: border-box; }
    body {
      margin: 0;
      background: #1b1b1b;
      color: #e8e8e8;
      font-family: Arial, sans-serif;
    }
    main {
      display: grid;
      grid-template-columns: minmax(820px, 1fr) 360px;
      gap: 14px;
      padding: 16px;
      align-items: start;
    }
    canvas {
      width: 800px;
      height: 600px;
      image-rendering: pixelated;
      border: 2px solid #000;
      background: #c0c0c0;
    }
    aside {
      background: #efefef;
      color: #050505;
      border: 2px solid;
      border-color: #fff #606060 #606060 #fff;
      padding: 12px;
      max-height: calc(100vh - 32px);
      overflow: auto;
    }
    h1 { margin: 0 0 8px; font-size: 20px; }
    h2 { font-size: 15px; margin: 14px 0 6px; }
    code, pre { font-family: Consolas, monospace; font-size: 12px; }
    pre { white-space: pre-wrap; background: #fff; border: 1px solid #aaa; padding: 8px; }
    .toolbar { margin-top: 10px; display: flex; gap: 8px; flex-wrap: wrap; }
    button {
      border: 2px solid;
      border-color: #fff #606060 #606060 #fff;
      background: #d8d8d8;
      padding: 6px 10px;
      cursor: pointer;
    }
    @media (max-width: 1240px) {
      main { grid-template-columns: 1fr; }
    }
  </style>
</head>
<body>
  <main>
    <section>
      <canvas id="screen" width="800" height="600"></canvas>
      <div class="toolbar">
        <button id="save">Save PNG</button>
        <button id="rerender">Re-render</button>
      </div>
    </section>
    <aside>
      <h1>CM0102 Exact Squad Slice</h1>
      <p>This is the first renderer slice: original 800x600 coordinate space, original <code>game.mbr</code> strip, decoded original <code>.fnt</code> glyphs, lifted font metrics, and current Rust manager squad data.</p>
      <h2>What Is Exact Here</h2>
      <pre id="exact"></pre>
      <h2>Still Frontier</h2>
      <pre>Caller-dependent y positions and some dynamic row-state branches are marked rather than guessed. Next lift resolves the caller setup around squad_screen param_1/param_2 and input dispatch.</pre>
      <h2>Squad Source</h2>
      <pre id="squadSource"></pre>
    </aside>
  </main>
  <script>
    const payload = __PAYLOAD__;
    const canvas = document.getElementById('screen');
    const ctx = canvas.getContext('2d');
    ctx.imageSmoothingEnabled = false;

    const rgb = {
      blue: [0, 0, 108, 255],
      darkBlue: [0, 0, 128, 255],
      black: [0, 0, 0, 255],
      white: [255, 255, 255, 255],
      grey: [192, 192, 192, 255],
      darkGrey: [64, 64, 64, 255],
      midGrey: [128, 128, 128, 255],
      green: [0, 128, 0, 255],
      yellow: [255, 255, 0, 255],
      cyan: [128, 255, 255, 255],
    };

    function glyphMap(font) {
      if (!font._map) font._map = Object.fromEntries(font.glyphs.map(g => [g.codepoint, g]));
      return font._map;
    }

    function pairSpacing(font, previousRight, nextLeft) {
      return Math.max(0, ((font.height * 3) >> 2) - nextLeft - previousRight);
    }

    function measureText(font, text) {
      const map = glyphMap(font);
      let width = 0;
      let previousRight = 0;
      let first = true;
      for (const raw of text) {
        const code = (raw === '|') ? 32 : raw.charCodeAt(0);
        const glyph = map[code];
        if (!glyph) continue;
        if (!first) width += pairSpacing(font, previousRight, glyph.left_bearing);
        width += glyph.advance;
        previousRight = glyph.right_bearing;
        first = false;
      }
      return width;
    }

    function drawGlyph(font, glyph, x, y, color) {
      if (!glyph.bitmap_hex || glyph.bitmap_width <= 0) return;
      const bytes = glyph.bitmap_hex.match(/../g).map(h => parseInt(h, 16));
      const image = ctx.createImageData(glyph.bitmap_width, font.height);
      for (let i = 0; i < bytes.length; i++) {
        const alpha = bytes[i];
        image.data[i * 4 + 0] = color[0];
        image.data[i * 4 + 1] = color[1];
        image.data[i * 4 + 2] = color[2];
        image.data[i * 4 + 3] = alpha;
      }
      ctx.putImageData(image, Math.round(x + glyph.left_bearing), Math.round(y));
    }

    function drawText(fontName, text, x, y, color = rgb.black, options = {}) {
      const font = payload.fonts[fontName];
      const map = glyphMap(font);
      let cursor = options.align === 'right' ? x - measureText(font, text) : x;
      let previousRight = 0;
      let first = true;
      for (const raw of text) {
        const code = (raw === '|') ? 32 : raw.charCodeAt(0);
        const glyph = map[code];
        if (!glyph) continue;
        if (!first) cursor += pairSpacing(font, previousRight, glyph.left_bearing);
        drawGlyph(font, glyph, cursor, y, color);
        cursor += glyph.advance;
        previousRight = glyph.right_bearing;
        first = false;
      }
    }

    function rect(x, y, w, h, fill) {
      ctx.fillStyle = `rgba(${fill.join(',')})`;
      ctx.fillRect(x, y, w, h);
    }

    function line(x1, y1, x2, y2, color) {
      ctx.strokeStyle = `rgba(${color.join(',')})`;
      ctx.beginPath();
      ctx.moveTo(x1 + 0.5, y1 + 0.5);
      ctx.lineTo(x2 + 0.5, y2 + 0.5);
      ctx.stroke();
    }

    function raisedBox(x, y, w, h, fill = rgb.grey) {
      rect(x, y, w, h, fill);
      line(x, y, x + w - 1, y, rgb.white);
      line(x, y, x, y + h - 1, rgb.white);
      line(x + w - 1, y, x + w - 1, y + h - 1, rgb.darkGrey);
      line(x, y + h - 1, x + w - 1, y + h - 1, rgb.darkGrey);
    }

    function sunkenBox(x, y, w, h, fill = [224, 224, 224, 255]) {
      rect(x, y, w, h, fill);
      line(x, y, x + w - 1, y, rgb.darkGrey);
      line(x, y, x, y + h - 1, rgb.darkGrey);
      line(x + w - 1, y, x + w - 1, y + h - 1, rgb.white);
      line(x, y + h - 1, x + w - 1, y + h - 1, rgb.white);
    }

    async function drawChrome() {
      rect(0, 0, 800, 600, rgb.grey);
      const strip = new Image();
      strip.src = payload.strip;
      await strip.decode();
      ctx.drawImage(strip, 0, 0, 90, 600);
      line(90, 0, 90, 599, rgb.black);
      raisedBox(96, 8, 694, 30, rgb.grey);
      drawText('arial_16', payload.squad.club.name, 112, 13, rgb.black);
      drawText('arial_narrow_11', `Squad · ${payload.squad.date.day}/${payload.squad.date.month}/${payload.squad.date.year}`, 640, 17, rgb.darkGrey);
    }

    function drawTabs() {
      const leftTabs = ['Traditional', 'Contract', 'Selection', 'Stats', 'More Stats'];
      const rightTabs = ['Attributes', 'Other'];
      let y = 48;
      for (const tab of leftTabs) {
        raisedBox(110, y, 250, 20, tab === 'Traditional' ? rgb.green : rgb.grey);
        drawText('arial_narrow_11', tab, 120, y + 2, tab === 'Traditional' ? rgb.white : rgb.black);
        y += 22;
      }
      y = 48;
      for (const tab of rightTabs) {
        raisedBox(530, y, 250, 20, tab === 'Attributes' ? rgb.green : rgb.grey);
        drawText('arial_narrow_11', tab, 540, y + 2, tab === 'Attributes' ? rgb.white : rgb.black);
        y += 22;
      }
    }

    function drawSquadTable() {
      const x = 110;
      const y = 178;
      const w = 670;
      const h = 370;
      sunkenBox(x, y, w, h, [232, 232, 232, 255]);
      rect(x + 2, y + 2, w - 4, 18, rgb.darkBlue);
      const columns = [
        ['Name', 116, 'left'],
        ['Position', 332, 'left'],
        ['Morale', 410, 'left'],
        ['Condition', 482, 'left'],
        ['Form', 570, 'left'],
        ['Value', 720, 'right'],
      ];
      for (const [name, cx, align] of columns) {
        drawText('arial_narrow_11', name, cx, y + 4, rgb.white, { align });
      }
      const slots = payload.squad.slots.slice(0, 18);
      let rowY = y + 22;
      for (const [index, slot] of slots.entries()) {
        rect(x + 2, rowY, w - 4, 17, index % 2 ? [216, 216, 216, 255] : [238, 238, 238, 255]);
        if (slot.suggested_selection) rect(x + 2, rowY, 4, 17, rgb.green);
        const name = slot.resolved ? `Staff ${slot.staff_id}` : `Unresolved ${slot.staff_id}`;
        const role = slot.suggested_role || '';
        drawText('arial_narrow_10', name, 116, rowY + 2, rgb.black);
        drawText('arial_narrow_10', role, 332, rowY + 2, rgb.black);
        drawText('arial_narrow_10', slot.selection_score ? String(slot.selection_score) : '-', 430, rowY + 2, rgb.black, { align: 'right' });
        drawText('arial_narrow_10', slot.attribute_average ? `${slot.attribute_average}%` : '-', 525, rowY + 2, rgb.black, { align: 'right' });
        drawText('arial_narrow_10', slot.rating_short_0x05 ? String(slot.rating_short_0x05) : '-', 600, rowY + 2, rgb.black, { align: 'right' });
        drawText('arial_narrow_10', '-', 720, rowY + 2, rgb.black, { align: 'right' });
        rowY += 18;
      }
    }

    function drawFooter() {
      raisedBox(110, 556, 162, 24, rgb.grey);
      raisedBox(282, 556, 162, 24, rgb.grey);
      raisedBox(454, 556, 162, 24, rgb.grey);
      drawText('arial_narrow_11', 'Sort By', 122, 561, rgb.black);
      drawText('arial_narrow_11', 'Filter', 294, 561, rgb.black);
      drawText('arial_narrow_11', 'Clear Squad', 466, 561, rgb.black);
      drawText('arial_narrow_10', 'Rendered from lifted CM0102 UI/font evidence; unresolved y params are documented, not guessed as verified.', 110, 586, rgb.darkGrey);
    }

    async function render() {
      ctx.clearRect(0, 0, 800, 600);
      await drawChrome();
      drawTabs();
      drawSquadTable();
      drawFooter();
    }

    document.getElementById('exact').textContent = JSON.stringify(payload.evidence, null, 2);
    document.getElementById('squadSource').textContent = JSON.stringify({
      club: payload.squad.club,
      summary: payload.squad.summary,
      provenance: payload.squad.provenance,
    }, null, 2);
    document.getElementById('rerender').addEventListener('click', render);
    document.getElementById('save').addEventListener('click', () => {
      const a = document.createElement('a');
      a.href = canvas.toDataURL('image/png');
      a.download = 'cm0102_exact_squad_slice.png';
      a.click();
    });
    render();
  </script>
</body>
</html>
"""


if __name__ == "__main__":
    main()
