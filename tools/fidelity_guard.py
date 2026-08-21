"""fidelity_guard.py -- reject renderers that DRAW A LOOK-ALIKE instead of executing
the game's real generator.

The CM0102 GUI is a deterministic function of decompiled code + shipped assets. A
"faithful" renderer must therefore produce pixels ONLY through:
  - the ported primitives   tools/cm_render/{surface,panel,text}.py
                            (exact ports of graphics_rgb_to_surface_pixel 0x005ce4f0,
                             graphics_draw_panel 0x005cf8e0, graphics_blit_string 0x005ced50)
  - decoded game assets      assets/cm0102/fonts/*.json (real .fnt glyphs), *.RGN/*.mbr images
  - lifted geometry/state    resolved columns, graphics_font_row_height, the view-model

Anything that instead reaches for PIL's ImageDraw/ImageFont, a system TrueType font,
a screenshot-matched literal, or a hand-tuned brightness fudge is a FABRICATION: it
approximates the look instead of computing the pixels. This tool finds those and fails.

A file is judged "faithful-claiming" if its text asserts fidelity (faithful / exact /
pixel-exact / 100% / "port of" / "strictly from" / "no invent|approx") OR carries the
explicit marker `# FIDELITY: faithful`. Such files must be clean. Pure capture/verify
I/O (decoding a real framebuffer grab to PNG) is fine and is NOT a fidelity claim.

Usage:
  python tools/fidelity_guard.py                # scan default render targets, gate
  python tools/fidelity_guard.py --files a.py b.py
  python tools/fidelity_guard.py --all          # scan every .py under tools/ and crates/
Exit code 0 = clean, 1 = fabrication(s) found.
"""
import ast, re, sys, os, glob, argparse

ROOT = "D:/cm0102-rs"

# ---- what counts as a fidelity claim (this file MUST then be clean) ----
CLAIM_RE = re.compile(
    r"\b(faithful|pixel[- ]?exact|exact port|100%\s*accur|strictly from|"
    r"no invent|no approx|no guess|port of\s+(FUN_|0x)|the game'?s (own|real)|"
    r"executes? the (real )?generator|deterministic (function|render))\b",
    re.I)
MARKER_FAITHFUL = re.compile(r"#\s*FIDELITY:\s*faithful", re.I)
MARKER_EXEMPT = re.compile(r"#\s*FIDELITY:\s*(io|capture|verify|exempt)", re.I)

# ---- screenshot-derived / hand-tuned tells (comments) ----
SCREENSHOT_TELL = re.compile(
    r"#.*\b(matched? to (the )?(screenshot|ref|image)|eyeball|"
    r"close enough|looks? (right|like)|hand[- ]?tuned|fudge|by eye|"
    r"approximate(ly|d)?|guess(ed|timate)?|magic number|tweak(ed)? to match)\b",
    re.I)

# ---- fabrication signals in code ----
FONT_PATH_TELL = re.compile(r"""["'][^"']*(?:/Windows/Fonts/|arial\.ttf|arialbd\.ttf|"""
                            r"""\.ttf|\.otf|\.ttc)["']""", re.I)
# note: .fnt is a GAME asset -> allowed; only outline system fonts are forbidden

def read(path):
    return open(path, encoding="utf-8", errors="replace").read()

def analyze(path):
    """Return list of (lineno, code, message) fabrication findings for one file."""
    src = read(path)
    findings = []
    if MARKER_EXEMPT.search(src):
        return findings  # explicitly declared pure-I/O / capture / verify
    claims = bool(CLAIM_RE.search(src)) or bool(MARKER_FAITHFUL.search(src))

    # --- AST pass: forbidden imports + drawing-context construction ---
    forbidden_import_lines = {}   # name -> lineno  (ImageDraw / ImageFont)
    try:
        tree = ast.parse(src, filename=path)
    except SyntaxError as e:
        return [(e.lineno or 0, "", f"unparseable: {e.msg}")]

    draw_ctx_names = set()   # variables bound to ImageDraw.Draw(...)
    for node in ast.walk(tree):
        # from PIL import ImageDraw, ImageFont   /  import PIL.ImageDraw
        if isinstance(node, ast.ImportFrom) and (node.module or "").startswith("PIL"):
            for a in node.names:
                if a.name in ("ImageDraw", "ImageFont"):
                    forbidden_import_lines[a.name] = node.lineno
        if isinstance(node, ast.Import):
            for a in node.names:
                if a.name in ("PIL.ImageDraw", "PIL.ImageFont"):
                    forbidden_import_lines[a.name.split(".")[-1]] = node.lineno
        # x = ImageDraw.Draw(img)   or   ImageFont.truetype(...)
        if isinstance(node, ast.Call):
            fn = node.func
            attr = fn.attr if isinstance(fn, ast.Attribute) else None
            base = (fn.value.id if isinstance(fn, ast.Attribute)
                    and isinstance(fn.value, ast.Name) else None)
            if base == "ImageDraw" and attr == "Draw":
                if isinstance(getattr(node, "parent", None), ast.Assign):
                    pass
            if base == "ImageFont" and attr in ("truetype", "load_default"):
                findings.append((node.lineno, ast.get_source_segment(src, node) or "ImageFont",
                                 "loads an outline/system font instead of the game's .fnt glyphs"))
        # assignment  d = ImageDraw.Draw(...)
        if isinstance(node, ast.Assign) and isinstance(node.value, ast.Call):
            f = node.value.func
            if (isinstance(f, ast.Attribute) and f.attr == "Draw"
                    and isinstance(f.value, ast.Name) and f.value.id == "ImageDraw"):
                for t in node.targets:
                    if isinstance(t, ast.Name):
                        draw_ctx_names.add(t.id)

    DRAW_METHODS = {"text", "rectangle", "line", "polygon", "ellipse", "arc",
                    "chord", "pieslice", "rounded_rectangle", "regular_polygon",
                    "textlength", "multiline_text"}
    for node in ast.walk(tree):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute):
            if (node.func.attr in DRAW_METHODS
                    and isinstance(node.func.value, ast.Name)
                    and node.func.value.id in draw_ctx_names):
                findings.append((node.lineno, f"{node.func.value.id}.{node.func.attr}(...)",
                                 "draws with PIL instead of the ported panel/text primitives"))

    for name, ln in forbidden_import_lines.items():
        findings.append((ln, f"import {name}",
                         f"imports PIL.{name} -- a faithful renderer emits pixels via the "
                         f"ported primitives, never PIL drawing"))

    # --- line regex pass: font paths + screenshot/hand-tuned tells ---
    for i, line in enumerate(src.splitlines(), 1):
        if FONT_PATH_TELL.search(line) and "fnt" not in line.lower():
            findings.append((i, line.strip()[:80], "hardcodes a system/outline font path"))
        if SCREENSHOT_TELL.search(line):
            findings.append((i, line.strip()[:80], "screenshot-matched / hand-tuned literal (not code-derived)"))

    # Only gate on findings when the file CLAIMS fidelity. Non-claiming utility
    # scripts may use PIL freely; they just can't call themselves faithful.
    if not claims:
        return [(-1, "", f"USES fabrication primitives but makes NO fidelity claim "
                          f"({len(findings)} signal(s)) -- allowed, but cannot be trusted as a "
                          f"faithful renderer")] if findings else []
    return findings

def default_targets():
    """Files that are supposed to render the game faithfully."""
    t = []
    t += glob.glob(f"{ROOT}/tools/frida/*render*.py")
    t += glob.glob(f"{ROOT}/tools/frida/replay*.py")
    t += glob.glob(f"{ROOT}/tools/cm_render/*.py")
    t += glob.glob(f"{ROOT}/tools/*render*.py")
    return sorted(set(t))

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--files", nargs="*")
    ap.add_argument("--all", action="store_true")
    a = ap.parse_args()
    if a.files:
        targets = a.files
    elif a.all:
        targets = sorted(set(glob.glob(f"{ROOT}/tools/**/*.py", recursive=True)
                             + glob.glob(f"{ROOT}/crates/**/*.rs", recursive=True)))
        targets = [t for t in targets if t.endswith(".py")]  # AST pass is Python-only
    else:
        targets = default_targets()

    total_violations = 0
    soft = 0
    for path in targets:
        rel = os.path.relpath(path, ROOT)
        findings = analyze(path)
        hard = [f for f in findings if f[0] != -1]
        note = [f for f in findings if f[0] == -1]
        if hard:
            total_violations += len(hard)
            print(f"\n\033[31mFABRICATION\033[0m  {rel}")
            for ln, code, msg in hard:
                print(f"   L{ln}: {msg}")
                if code: print(f"        > {code}")
        elif note:
            soft += 1
            print(f"\n~ note  {rel}: {note[0][2]}")
        else:
            print(f"ok  {rel}")

    print("\n" + "=" * 64)
    if total_violations:
        print(f"REJECTED: {total_violations} fabrication(s) in faithful-claiming renderer(s).")
        print("A faithful renderer must draw via tools/cm_render (ported primitives) and the")
        print("decoded .fnt glyphs -- never PIL ImageDraw/ImageFont or screenshot-matched numbers.")
        return 1
    print(f"CLEAN: no fabrications in faithful renderers." + (f"  ({soft} untrusted utility file(s) noted.)" if soft else ""))
    return 0

if __name__ == "__main__":
    sys.exit(main())
