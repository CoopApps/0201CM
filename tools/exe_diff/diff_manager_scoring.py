"""Differential harness: cm0102.exe vs the Rust cm-scoring port.

Reuses the existing Unicorn emulator (tools/cm-lift/cm_lift/emulate.py) as the PE
loader / memory mapper / call-ABI. This module is the target-specific layer:
fixture builders + corpus + comparison for the manager scoring primitives.

Trust anchors (the harness is not trusted until these pass across broad inputs):
  FUN_0052a330 manager_club_repfit   exe == Rust
  FUN_0052a410 closeness_class       exe == Rust

Rust side runs via the standalone `scoring_probe` bin (cm-scoring builds without
cm-domain's LLVM-OOM burden).
"""
import os, sys, struct, subprocess, random
from unicorn.x86_const import UC_X86_REG_EAX, UC_X86_REG_ESP, UC_X86_REG_EIP

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
sys.path.insert(0, os.path.join(ROOT, "tools", "cm-lift"))
from cm_lift.util import load_pe
from cm_lift.emulate import Emulator

EXE = r"D:\cm0102\cm0102.exe"
PROBE = os.path.join(ROOT, "target", "debug", "scoring_probe.exe")

FUN_REPFIT   = 0x0052a330
FUN_CLOSE    = 0x0052a410
FUN_KNOWN    = 0x005274d0   # "ref known in nation" sub-call (force its return)

def force_return(emu, eax_val):
    """Make the current function (entered at the hooked VA) return eax_val
    immediately (cdecl: caller cleans args, so we only pop the return addr)."""
    uc = emu.uc
    esp = uc.reg_read(UC_X86_REG_ESP)
    ret = struct.unpack("<I", uc.mem_read(esp, 4))[0]
    uc.reg_write(UC_X86_REG_ESP, esp + 4)
    uc.reg_write(UC_X86_REG_EAX, eax_val & 0xFFFFFFFF)
    uc.reg_write(UC_X86_REG_EIP, ret)

def run_probe(lines):
    p = subprocess.run([PROBE], input="\n".join(lines) + "\n",
                       capture_output=True, text=True)
    return p.stdout.split("\n")

def block_hex(shorts):
    """16-byte block; shorts is dict off->i16."""
    b = bytearray(0x10)
    for off, v in shorts.items():
        b[off:off+2] = struct.pack("<h", v)
    return b

# ── A. repfit selection (FUN_0052a330), forcing the closeness code ───────────
def test_repfit(emu, n_per=60, seed=1):
    rng = random.Random(seed)
    emu._call_hooks.clear()
    cases = []      # (block_bytes, mode0, code)
    exe_res = []
    forced = {"c": 0}
    emu.hook_call(FUN_CLOSE, lambda e, a, s: force_return(e, forced["c"]))
    for code in (0, 1, 2, 3, 4):
        for mode in (0, 1):
            for _ in range(n_per):
                sh = {o: rng.randint(-2000, 12000) for o in (8, 9, 0xa, 0xb, 0xc, 0xd)}
                blk = block_hex(sh)
                standing = emu.malloc(0x20); emu.uc.mem_write(standing, bytes(blk))
                # person: +0x61 and +0x69 both -> standing block
                person = emu.malloc(0x80)
                emu.write_field(person, 0x61, "I", standing)
                emu.write_field(person, 0x69, "I", standing)
                forced["c"] = code
                r = emu.call(FUN_REPFIT, person, person, mode)
                # result is a signed short in AX
                r = struct.unpack("<h", struct.pack("<H", r & 0xFFFF))[0]
                exe_res.append(r)
                cases.append((bytes(blk), 1 if mode == 0 else 0, code))
    lines = [f"repfit {c[0].hex()} {c[1]} {c[2]}" for c in cases]
    rust = run_probe(lines)
    mismatches = []
    for i, (c, er) in enumerate(zip(cases, exe_res)):
        rr = int(rust[i])
        if rr != er:
            mismatches.append((c, er, rr))
    return len(cases), mismatches

# ── B. closeness classifier (FUN_0052a410) ──────────────────────────────────
def test_closeness(emu):
    """Build person/ref records realizing each tree branch; force FUN_005274d0
    ('ref known in nation') by its 2nd arg. Compare exe code vs probe."""
    NAT_A, NAT_B = 0x11110000, 0x22220000  # nation ids (values compared for equality)
    cases = []   # (inputs_dict, expected_via_probe_line)
    exe_codes = []
    # known-flag state, keyed by the nation-id arg passed to 005274d0
    emu._call_hooks.clear()
    known = {"person_nat": 1, "club_nat": 1, "person_nat_id": NAT_A, "club_nat_id": 0}
    def known_hook(e, a, s):
        arg2 = struct.unpack("<I", e.uc.mem_read(e.uc.reg_read(UC_X86_REG_ESP) + 8, 4))[0]
        if arg2 == known["club_nat_id"]:
            force_return(e, known["club_nat"])
        else:
            force_return(e, known["person_nat"])
    emu.hook_call(FUN_KNOWN, known_hook)

    def build(has_club, same_club, same_club_nation, same_person_nation,
              known_person_nat, known_club_nat):
        cnat_a = 0x33330000
        cnat_b = cnat_a if same_club_nation else 0x44440000
        club_a = emu.malloc(0x300); emu.write_field(club_a, 0x53, "I", cnat_a)
        emu.write_field(club_a, 0x80, "h", 5000)
        if same_club:
            club_b = club_a
        else:
            club_b = emu.malloc(0x300); emu.write_field(club_b, 0x53, "I", cnat_b)
            emu.write_field(club_b, 0x80, "h", 5000)
        person = emu.malloc(0x80); ref = emu.malloc(0x80)
        if has_club:
            emu.write_field(person, 0x39, "I", club_a)
        emu.write_field(ref, 0x39, "I", club_b)
        emu.write_field(person, 0x1a, "I", NAT_A)
        emu.write_field(ref, 0x1a, "I", NAT_A if same_person_nation else NAT_B)
        known["person_nat"] = known_person_nat
        known["club_nat"] = known_club_nat
        known["person_nat_id"] = NAT_A
        known["club_nat_id"] = cnat_a
        return person, ref, club_a

    # enumerate the branch combinations that select each return value
    combos = []
    combos.append(("ref_null", None))
    for hc in (0, 1):
        for sc in (0, 1):
            for scn in (0, 1):
                for spn in (0, 1):
                    for kpn in (0, 1):
                        for kcn in (0, 1):
                            combos.append(("x", (hc, sc, scn, spn, kpn, kcn)))
    probe_lines = []
    for kind, c in combos:
        if kind == "ref_null":
            person = emu.malloc(0x80)
            code = emu.call(FUN_CLOSE, person, 0) & 0xFF
            exe_codes.append(code)
            # inputs: ref_null=1
            probe_lines.append("closeness 1 0 0 0 0 0 0 0")
            continue
        hc, sc, scn, spn, kpn, kcn = c
        person, ref, _ = build(hc, sc, scn, spn, kpn, kcn)
        code = emu.call(FUN_CLOSE, person, ref) & 0xFF
        exe_codes.append(code)
        # ClosenessInputs bits: refnull person_has_club same_club same_club_nation
        #   same_person_nation ref_known_person_nat ref_known_club_nat regional
        bits = [0, hc, (sc if hc else 0), (scn if hc else 0), spn, kpn, kcn, 0]
        probe_lines.append("closeness " + " ".join(str(b) for b in bits))
    rust = run_probe(probe_lines)
    mism = []
    for i, (kind, c) in enumerate(combos):
        rr = int(rust[i]); er = exe_codes[i]
        if rr != er:
            mism.append((c, er, rr))
    return len(combos), mism

def main():
    if not os.path.exists(PROBE):
        sys.exit(f"build the probe first: cargo build -p cm-scoring --bin scoring_probe\nmissing {PROBE}")
    emu = Emulator(pe=load_pe(EXE))
    print(f"loaded {EXE}")
    total, mm = test_repfit(emu)
    print(f"\n=== FUN_0052a330 manager_club_repfit ===")
    print(f"cases: {total}   mismatches: {len(mm)}")
    for c, er, rr in mm[:20]:
        print(f"  MISMATCH block={c[0].hex()} mode0={c[1]} c={c[2]}  exe={er} rust={rr}")
    print("REPFIT:", "PASS" if not mm else "FAIL")

    ctotal, cmm = test_closeness(emu)
    print(f"\n=== FUN_0052a410 closeness_class ===")
    print(f"cases: {ctotal}   mismatches: {len(cmm)}")
    for c, er, rr in cmm[:20]:
        print(f"  MISMATCH inputs={c}  exe={er} rust={rr}")
    print("CLOSENESS:", "PASS" if not cmm else "FAIL")

if __name__ == "__main__":
    main()
