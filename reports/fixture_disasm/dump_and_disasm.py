"""Disassemble cm0102.exe over the un-decompiled block 0x0081a118..0x00821e90.

Goal: locate every CALL to FUN_00670350 (0x00670350), then walk back over the
preceding PUSH instructions to recover the arguments — most importantly the
schedule-template pointer (param_7) that lives in .rdata and gets memcpy'd
into `comp+0xba`.

We use capstone in 32-bit x86 mode. Output goes to two files:
  - `raw_gap.bin`     : the raw 30KB byte range
  - `disasm.txt`      : full disassembly of the gap
  - `call_sites.txt`  : every CALL 0x00670350 with the 20 prior insns
"""
import struct
from pathlib import Path
from capstone import Cs, CS_ARCH_X86, CS_MODE_32

EXE = Path(r"D:\cm0102\cm0102.exe")
OUT = Path(r"D:\cm0102-rs\reports\fixture_disasm")
OUT.mkdir(parents=True, exist_ok=True)

GAP_START_VA = 0x0081a118
GAP_END_VA   = 0x00821e90
TARGET_VA    = 0x00670350   # FUN_00670350


def parse_pe(b):
    e_lfanew = struct.unpack_from("<I", b, 0x3c)[0]
    assert b[e_lfanew:e_lfanew+4] == b"PE\0\0"
    coff = e_lfanew + 4
    n_sec = struct.unpack_from("<H", b, coff + 2)[0]
    opt_size = struct.unpack_from("<H", b, coff + 16)[0]
    opt_off = coff + 20
    image_base = struct.unpack_from("<I", b, opt_off + 28)[0]
    sec_off = opt_off + opt_size
    secs = []
    for i in range(n_sec):
        base = sec_off + i * 40
        name = b[base:base+8].rstrip(b"\0").decode("latin1", "replace")
        vsz = struct.unpack_from("<I", b, base+8)[0]
        va = struct.unpack_from("<I", b, base+12)[0]
        rsz = struct.unpack_from("<I", b, base+16)[0]
        ro = struct.unpack_from("<I", b, base+20)[0]
        secs.append(dict(name=name, va=va, vsz=vsz, ro=ro, rsz=rsz))
    return image_base, secs


def va_to_file(va, image_base, sections):
    rva = va - image_base
    for s in sections:
        if s["va"] <= rva < s["va"] + max(s["vsz"], s["rsz"]):
            return s["ro"] + (rva - s["va"]), s["name"]
    return None, None


def main():
    exe_bytes = EXE.read_bytes()
    image_base, sections = parse_pe(exe_bytes)
    print(f"image_base = {image_base:#010x}")
    for s in sections:
        print(f"  {s['name']:8s} va {s['va']:#010x} vsz {s['vsz']:#x} "
              f"ro {s['ro']:#010x} rsz {s['rsz']:#x}")

    off_start, sec_start = va_to_file(GAP_START_VA, image_base, sections)
    off_end,   sec_end   = va_to_file(GAP_END_VA,   image_base, sections)
    print(f"\ngap start VA {GAP_START_VA:#010x} -> file {off_start:#010x} in {sec_start}")
    print(f"gap end   VA {GAP_END_VA:#010x}   -> file {off_end:#010x}   in {sec_end}")

    blob = exe_bytes[off_start:off_end]
    (OUT / "raw_gap.bin").write_bytes(blob)
    print(f"wrote raw_gap.bin ({len(blob)} bytes)")

    md = Cs(CS_ARCH_X86, CS_MODE_32)
    md.detail = True

    lines = []
    call_sites = []
    for insn in md.disasm(blob, GAP_START_VA):
        lines.append(f"{insn.address:#010x}  {insn.bytes.hex():<20}  "
                     f"{insn.mnemonic} {insn.op_str}")
        if insn.mnemonic == "call" and insn.op_str.startswith("0x"):
            try:
                tgt = int(insn.op_str, 16)
            except ValueError:
                tgt = None
            if tgt == TARGET_VA:
                call_sites.append(insn.address)

    (OUT / "disasm.txt").write_text("\n".join(lines), encoding="utf-8")
    print(f"wrote disasm.txt ({len(lines)} insns)")

    # For each call to FUN_00670350, dump the preceding ~40 instructions.
    dump = []
    dump.append(f"# {len(call_sites)} calls to FUN_00670350 in gap")
    dump.append("")
    for site in call_sites:
        # find index of site in `lines`; simple linear search
        idx = None
        for i, ln in enumerate(lines):
            if ln.startswith(f"{site:#010x}"):
                idx = i
                break
        if idx is None:
            continue
        lo = max(0, idx - 40)
        dump.append(f"## CALL FUN_00670350 at {site:#010x}")
        dump.append("")
        for j in range(lo, idx + 1):
            dump.append(lines[j])
        dump.append("")
    (OUT / "call_sites.txt").write_text("\n".join(dump), encoding="utf-8")
    print(f"wrote call_sites.txt ({len(call_sites)} sites)")


if __name__ == "__main__":
    main()
