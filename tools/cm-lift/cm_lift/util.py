"""Shared paths, PE parsing helpers, and .rdata resolution.

Every other module reads exe/decompile/port paths from here so the CLIs
don't have to pass them around.
"""
from __future__ import annotations
import os
import re
from pathlib import Path
from dataclasses import dataclass
from typing import Optional, Iterator

# --- Well-known paths (override via env vars) -----------------------------

CM_EXE       = Path(os.environ.get("CM_LIFT_EXE",
    "D:/cm0102/cm0102.exe"))
DECOMPILE    = Path(os.environ.get("CM_LIFT_DECOMPILE",
    "D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled"))
RUST_ROOT    = Path(os.environ.get("CM_LIFT_RUST",
    "D:/cm0102-rs"))
DATA_OUT     = Path(os.environ.get("CM_LIFT_DATA",
    "D:/cm0102-rs/tools/cm-lift/data"))

DATA_OUT.mkdir(parents=True, exist_ok=True)


# --- PE structure helpers -------------------------------------------------

@dataclass
class PeInfo:
    """Cached PE structure — image base, section list, .rdata / .text
    bounds, imports table.
    """
    path: Path
    image_base: int
    entry_point_va: int
    sections: list          # list[(name, va, size, raw_offset, raw_size)]
    rdata_va: int
    rdata_size: int
    rdata_bytes: bytes
    text_va: int
    text_size: int
    text_bytes: bytes
    data_va: int
    data_size: int
    data_bytes: bytes
    imports: dict           # dll -> {name: iat_va}

    def section_containing(self, va: int) -> Optional[str]:
        for name, sva, sz, *_ in self.sections:
            if sva <= va < sva + sz:
                return name
        return None

    def read_va(self, va: int, n: int) -> Optional[bytes]:
        """Read `n` bytes from virtual address `va`. Returns None if
        the VA doesn't map into any file-backed section."""
        for name, sva, sz, raw_off, raw_sz in self.sections:
            if sva <= va < sva + sz:
                off = va - sva
                if off + n > raw_sz:
                    n = raw_sz - off
                if n <= 0:
                    return b""
                if name == ".rdata":
                    return self.rdata_bytes[off:off+n]
                if name == ".text":
                    return self.text_bytes[off:off+n]
                if name == ".data":
                    return self.data_bytes[off:off+n]
        return None


def load_pe(path: Path = CM_EXE) -> PeInfo:
    """Parse the CM01/02 exe via pefile, cache the .text/.rdata/.data
    section bytes for random access. Runs in <1s.
    """
    import pefile
    pe = pefile.PE(str(path), fast_load=True)
    pe.parse_data_directories(directories=[
        pefile.DIRECTORY_ENTRY['IMAGE_DIRECTORY_ENTRY_IMPORT'],
    ])
    base = pe.OPTIONAL_HEADER.ImageBase
    entry = pe.OPTIONAL_HEADER.AddressOfEntryPoint + base

    sections = []
    text_va = text_sz = rdata_va = rdata_sz = data_va = data_sz = 0
    text_bytes = rdata_bytes = data_bytes = b""
    for s in pe.sections:
        name = s.Name.decode("ascii", errors="ignore").rstrip("\x00")
        va = s.VirtualAddress + base
        sz = s.Misc_VirtualSize
        raw_off = s.PointerToRawData
        raw_sz = s.SizeOfRawData
        sections.append((name, va, sz, raw_off, raw_sz))
        raw = s.get_data()
        if name == ".text":  text_va, text_sz, text_bytes = va, sz, raw
        if name == ".rdata": rdata_va, rdata_sz, rdata_bytes = va, sz, raw
        if name == ".data":  data_va, data_sz, data_bytes = va, sz, raw

    # Imports — dll -> {name: iat_va}
    imports: dict = {}
    for entry_dll in getattr(pe, "DIRECTORY_ENTRY_IMPORT", []):
        dll = entry_dll.dll.decode("ascii", errors="ignore").lower()
        by_name = {}
        for imp in entry_dll.imports:
            if imp.name:
                by_name[imp.name.decode("ascii", errors="ignore")] = imp.address
        imports[dll] = by_name

    return PeInfo(
        path=path, image_base=base, entry_point_va=entry,
        sections=sections,
        rdata_va=rdata_va, rdata_size=rdata_sz, rdata_bytes=rdata_bytes,
        text_va=text_va,   text_size=text_sz,   text_bytes=text_bytes,
        data_va=data_va,   data_size=data_sz,   data_bytes=data_bytes,
        imports=imports,
    )


# --- Decompile navigation -------------------------------------------------

def decompile_file(addr: int | str) -> Optional[Path]:
    """Return the .c file for a fn address (int or hex str). Ghidra names
    each fn's decompile as `<addr_hex_lower>.c` in the decompiled dir.
    """
    if isinstance(addr, int):
        name = f"{addr:08x}"[-6:]  # e.g. 006b3de0
    else:
        name = addr.lower().replace("0x", "").lstrip("0")
        if len(name) < 6:
            name = name.rjust(6, "0")
    for candidate in (DECOMPILE / f"{name}.c", DECOMPILE / f"{name.upper()}.c"):
        if candidate.exists():
            return candidate
    return None


def iter_decompiled_fns() -> Iterator[tuple[int, Path]]:
    """Yield (address_int, path) for every decompiled fn."""
    for p in DECOMPILE.glob("*.c"):
        try:
            addr = int(p.stem, 16)
            yield addr, p
        except ValueError:
            continue


def touched_fns() -> set[str]:
    """Return the set of FUN_XXXXXX addresses (lowercase hex, no prefix)
    referenced anywhere in the Rust source or in reports/."""
    seen = set()
    pat = re.compile(r"FUN_([0-9a-fA-F]{6,8})")
    for root in [RUST_ROOT / "crates", RUST_ROOT / "reports"]:
        if not root.exists():
            continue
        for f in root.rglob("*.rs" if root.name == "crates" else "*.md"):
            try:
                for m in pat.finditer(f.read_text(encoding="utf-8", errors="ignore")):
                    seen.add(m.group(1).lower())
            except Exception:
                pass
    return seen


# --- Convenient re-exports for CLIs --------------------------------------

__all__ = [
    "CM_EXE", "DECOMPILE", "RUST_ROOT", "DATA_OUT",
    "PeInfo", "load_pe",
    "decompile_file", "iter_decompiled_fns", "touched_fns",
]
