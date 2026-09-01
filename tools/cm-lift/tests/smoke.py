"""Smoke test — verifies every cm-lift module imports + basic op works."""
import sys, os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from cm_lift import util
from cm_lift.util import load_pe, touched_fns

def test_pe_load():
    pe = load_pe()
    assert pe.image_base == 0x00400000, f"unexpected base {pe.image_base:#x}"
    assert pe.text_size > 5_000_000, f"text too small: {pe.text_size}"
    assert pe.rdata_size > 100_000, f"rdata too small: {pe.rdata_size}"
    print(f"  pe_load  OK  base={pe.image_base:#x} text={pe.text_size} rdata={pe.rdata_size}")

def test_touched():
    tk = touched_fns()
    assert len(tk) > 500, f"only {len(tk)} touched — expected >500"
    print(f"  touched  OK  {len(tk)} touched fn addresses")

def test_vtables():
    from cm_lift.vtable_dumper import scan_vtables
    tables = scan_vtables(load_pe())
    assert len(tables) > 20, f"only {len(tables)} vtables"
    print(f"  vtables  OK  {len(tables)} tables found")

def test_dat_scan():
    from cm_lift.fp_sniper import lookup_constant
    v = lookup_constant(0x00955890, "f64")
    assert v is not None and abs(v - 1.0) < 1e-9, f"expected 1.0, got {v}"
    print(f"  dat_scan OK  _DAT_00955890 = {v}")

def test_emulate():
    from cm_lift.emulate import make_emulator
    emu = make_emulator()
    r = emu.read_regs()
    assert r["esp"] > 0x00E00000, f"stack not set: {r['esp']:#x}"
    print(f"  emulate  OK  esp={r['esp']:#x}")

def test_codegen():
    from cm_lift import codegen
    ems = codegen.scan(limit=10, only_untouched=True)
    print(f"  codegen  OK  {len(ems)} emissions in first 10")

if __name__ == "__main__":
    print("cm-lift smoke tests:")
    for name, fn in list(globals().items()):
        if name.startswith("test_") and callable(fn):
            try:
                fn()
            except Exception as e:
                print(f"  {name} FAIL: {e}")
                sys.exit(1)
    print("all pass")
