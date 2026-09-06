# Editor Frida Harvester

Exposes the entire editor's imported data by hooking three tiers of Windows APIs
in cm0102ed.exe. Output = one JSONL event per hooked call → the diff harness
turns it into `(record_type, id, field, value)` tuples for cross-checking
against our rust-db port.

## Hooks

| Tier          | APIs                                              | What we see                                            |
|---------------|---------------------------------------------------|--------------------------------------------------------|
| File I/O      | `CreateFile{A,W}`, `ReadFile`, `SetFilePointer`   | Which .dat opens; every record-sized read + seek       |
| Controls      | `SendMessage{A,W}` (LB_/CB_/WM_SETTEXT/LVM_/TVM_) | Every string pushed into a list / combo / edit / tree  |
| GDI paint     | `TextOut{A,W}`, `ExtTextOut{A,W}`, `DrawText{A,W}`| Every string the editor actually paints                |

Tier 2 alone gives us **every editor cell value** — that's the primary channel
for club colour names, position codes, nation names, all of it. Tier 1 aligns
those values to disk offsets. Tier 3 is a redundancy check for owner-drawn cells.

## Requirements

```
py -m pip install frida frida-tools
```

Editor must NOT run elevated (Frida can't touch admin processes from a
non-admin shell — this is the same gotcha as with cm0102.exe). Right-click
cm0102ed.exe → Properties → Compatibility → uncheck "Run as administrator".

## Run

**Recommended flow — attach to a running editor with index.dat already loaded:**

```powershell
# 1. Launch the editor manually, open D:\cm0102\Data\index.dat, wait for load.
# 2. In another shell:
py tools\editor_frida\attach.py
# 3. Click around in the editor — each tab paint & list scroll fills the log.
# 4. Ctrl-C to stop.
```

**Alternate — spawn the editor with hooks armed from process start (captures the
initial load):**

```powershell
py tools\editor_frida\attach.py --spawn D:\cm0102\editor\cm0102ed.exe
# then in the editor, File → Open → index.dat
```

Output goes to `tools\editor_frida\editor_events.jsonl` by default; override
with `--out`.

## Sample events

```json
{"t": 1725580000000, "src": "CreateFile", "path": "D:\\cm0102\\Data\\index.dat", "handle": 812}
{"t": 1725580000123, "src": "ReadFile",   "path": "D:\\cm0102\\Data\\index.dat", "handle": 812, "len": 70}
{"t": 1725580005000, "src": "SendMessage","msg": "LB_ADDSTRING", "text": "1.FC Bocholt", "hwnd": "0x1a02c"}
{"t": 1725580005001, "src": "SendMessage","msg": "CB_ADDSTRING", "text": "Blue 3",       "hwnd": "0x1a108"}
{"t": 1725580005050, "src": "TextOutA",   "hdc": "0xd101ab", "x": 349, "y": 245, "text": "White"}
```

## Diff harness

The companion `tools/editor_diff.py` (TODO) walks `data/rust-db/*.json`, formats
each field the way the editor would (name-lookups, colour-name resolves,
position-code build) and compares to the Frida capture. Emits a mismatch report
grouped by `(record_type, field_name)` so type-width bugs (u16 vs u32 truncation)
surface as thousands of mismatches on one field rather than scattered noise.
