# cm-lift

Toolchain that pushes the CM01/02 → Rust port from 26% real coverage to 100%
fidelity by removing the four current tooling ceilings:

1. **FP-lift** — Ghidra drops FPU operands (~840 elided `_DAT_*` constants,
   ~40 whole fns).
2. **Vtable dispatch** — ~30 slots blocked (penalty shootout, cup deciders).
3. **Correctness** — no diff harness against ground truth.
4. **Volume** — ~2,200 remaining fns; ~1,000 auto-emittable.

## Components

| Module | Role | Runs on |
|---|---|---|
| `emulate.py`      | Unicorn-based headless runner for `cm0102.exe` | any |
| `fp_sniper.py`    | FPU-stack constant extractor at every `__ftol` | emulator |
| `vtable_dumper.py`| Static `.rdata` C++ vtable parser | pefile |
| `codegen.py`      | Reads Ghidra `.c` files, emits Rust for mechanical patterns | none |
| `diff_harness.py` | Side-by-side runner: emulated exe vs Rust port | emulator + Rust bin |
| `frida_hook.py`   | Runtime call log via Frida (Wine/Windows) | Frida agent |

## CLIs

```
cm-lift emulate         # boot cm0102.exe under Unicorn, print state
cm-lift snipe-fp        # dump all _DAT_* FP constants → JSON
cm-lift dump-vtables    # dump C++ vtables → JSON
cm-lift codegen         # scan decompile, emit Rust stubs → Rust files
cm-lift diff            # run same seed against exe + port, log divergences
cm-lift frida-log       # attach Frida to Wine-hosted exe, log call graph
```

## Prerequisites

- Python 3.12 at `D:/Python312/python.exe`
- unicorn, pefile, capstone, frida (all installed)
- `cm0102.exe` at `D:/cm0102/cm0102.exe`
- Ghidra decompile at `D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`
- Rust port at `D:/cm0102-rs/`

## Directory layout

```
cm-lift/
├── cm_lift/          — Python package
├── bin/              — CLI wrappers
├── tests/            — smoke tests
├── data/             — extracted artifacts (JSON, tables)
└── README.md
```
