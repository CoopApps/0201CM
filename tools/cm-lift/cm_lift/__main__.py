"""cm-lift CLI dispatcher.

Usage:
  python -m cm_lift <command> [args...]

Commands mirror the module names:
  emulate           — bring up the Unicorn emulator and print state
  snipe-fp          — dump _DAT_* constants (scan-refs subcommand)
  dump-vtables      — extract C++ vtables from .rdata
  codegen           — auto-emit Rust stubs from decompile
  diff              — run diff harness
  frida-log         — attach Frida to running exe
  status            — print per-tool status summary
"""
import sys
from . import util

USAGE = """cm-lift <command> [args...]

Commands:
  emulate           bring up Unicorn and print image state
  snipe-fp          dump _DAT_* constants (scan-refs subcommand)
  dump-vtables      extract C++ vtables from .rdata
  codegen           auto-emit Rust stubs from decompile
  diff              run diff harness
  frida-log         attach Frida to running exe
  status            print per-tool status summary
"""


def cmd_emulate():
    from .emulate import make_emulator
    emu = make_emulator()
    r = emu.read_regs()
    print(f"emulator up. image_base={emu.pe.image_base:#010x}")
    for k, v in r.items():
        print(f"  {k}: {v:#010x}")


def cmd_snipe_fp(argv):
    from . import fp_sniper
    sys.argv = ["fp_sniper"] + argv
    fp_sniper.main()


def cmd_dump_vtables(argv):
    from . import vtable_dumper
    sys.argv = ["vtable_dumper"] + argv
    vtable_dumper.main()


def cmd_codegen(argv):
    from . import codegen
    sys.argv = ["codegen"] + argv
    codegen.main()


def cmd_diff(argv):
    from . import diff_harness
    sys.argv = ["diff_harness"] + argv
    diff_harness.main()


def cmd_frida(argv):
    from . import frida_hook
    sys.argv = ["frida_hook"] + argv
    frida_hook.main()


def cmd_status():
    import json
    from pathlib import Path
    print(f"cm-lift status")
    print(f"  exe:        {util.CM_EXE}  ({'ok' if util.CM_EXE.exists() else 'MISSING'})")
    print(f"  decompile:  {util.DECOMPILE}  "
          f"({sum(1 for _ in util.DECOMPILE.glob('*.c'))} .c files)")
    print(f"  rust:       {util.RUST_ROOT}")
    print(f"  data out:   {util.DATA_OUT}")
    for f, label in [
        ("vtables.json", "vtables"),
        ("dat_constants.json", "_DAT constants"),
        ("static_constants.json", "static consts"),
        ("diff_report.json", "diff report"),
        ("frida_call_log.jsonl", "frida log"),
    ]:
        p = util.DATA_OUT / f
        if p.exists():
            try:
                d = json.loads(p.read_text()) if f.endswith(".json") else None
                n = d.get("count") if isinstance(d, dict) else "-"
                print(f"  {label:20s} {f:26s} {n}")
            except Exception:
                print(f"  {label:20s} {f:26s} (unparseable)")
        else:
            print(f"  {label:20s} {f:26s} not run yet")


def main():
    argv = sys.argv[1:]
    if not argv or argv[0] in ("-h", "--help"):
        print(USAGE)
        return
    cmd = argv[0]
    rest = argv[1:]
    if cmd == "emulate":       cmd_emulate()
    elif cmd == "snipe-fp":    cmd_snipe_fp(rest)
    elif cmd == "dump-vtables":cmd_dump_vtables(rest)
    elif cmd == "codegen":     cmd_codegen(rest)
    elif cmd == "diff":        cmd_diff(rest)
    elif cmd == "frida-log":   cmd_frida(rest)
    elif cmd == "status":      cmd_status()
    else:
        print(f"unknown command: {cmd}")
        print(USAGE)
        sys.exit(1)


if __name__ == "__main__":
    main()
