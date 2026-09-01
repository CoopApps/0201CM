"""Unicorn-based headless runner for cm0102.exe.

Loads the exe into a Unicorn x86 engine, maps every section, stubs the
Win32 imports (returning safe defaults), and exposes primitives to:

  * `call(addr, *args)` — run a single fn with cdecl calling convention
    and return its EAX (or ECX for thiscall). Reads/writes the emulated
    stack directly.
  * `snapshot()` / `restore()` — copy the entire address space + register
    state so a diff pass can rewind between probes.
  * `hook_call(addr, callback)` — every call to `addr` fires `callback(uc, esp)`.
  * `hook_read(va_range, callback)` — every memory read inside range fires
    `callback(uc, addr, size)`.
  * `read_fpu()` / `read_regs()` — dump FPU stack + integer regs.

The stubbed imports return zero / success so we can drive individual
functions without booting the whole game. `bootstrap()` is a lightweight
alternative to running the full startup — it just allocates a heap and
zeros the game state pool.
"""
from __future__ import annotations
import struct
from dataclasses import dataclass, field
from typing import Callable, Optional
from unicorn import Uc, UC_ARCH_X86, UC_MODE_32, UC_HOOK_CODE, UC_HOOK_MEM_READ
from unicorn.x86_const import (
    UC_X86_REG_EAX, UC_X86_REG_EBX, UC_X86_REG_ECX, UC_X86_REG_EDX,
    UC_X86_REG_ESI, UC_X86_REG_EDI, UC_X86_REG_EBP, UC_X86_REG_ESP,
    UC_X86_REG_EIP, UC_X86_REG_EFLAGS,
    UC_X86_REG_ST0, UC_X86_REG_ST1, UC_X86_REG_ST2, UC_X86_REG_ST3,
    UC_X86_REG_ST4, UC_X86_REG_ST5, UC_X86_REG_ST6, UC_X86_REG_ST7,
)
from .util import PeInfo, load_pe

# --- Memory layout constants ---------------------------------------------

STACK_BASE = 0x00E00000
STACK_SIZE = 0x00100000  # 1 MB stack
HEAP_BASE  = 0x10000000
HEAP_SIZE  = 0x08000000  # 128 MB heap for game state
IMPORT_STUB_BASE = 0x20000000
IMPORT_STUB_SIZE = 0x00010000  # 64 KB of "just ret" thunks for imports

# --- Import stubs --------------------------------------------------------

# Small per-import trampolines. Each stub is: mov eax, N ; retn 4*n
# We just make every import return 0 (or a plausible small non-zero).
# Games generally tolerate this for the fns we intercept below.
def _build_import_stub(idx: int, retn_bytes: int) -> bytes:
    # mov eax, 0 ; retn <retn_bytes>
    return b"\xb8\x00\x00\x00\x00" + bytes([0xc2, retn_bytes & 0xff, (retn_bytes >> 8) & 0xff])

# Fns that should return non-zero (else the game aborts):
_NONZERO_IMPORTS = {
    "GetTickCount": 0x1000,
    "timeGetTime": 0x1000,
    "GetVersion": 0x0a280105,      # Win7-ish
    "GetCommandLineA": 0x20000000, # pointer to a valid readable region
    "GetProcessHeap": 0x20000000,
    "HeapCreate": 0x20000000,
    "HeapAlloc": HEAP_BASE,        # give it heap addresses
    "GetModuleHandleA": 0x00400000,
    "GetCurrentProcess": 0xFFFFFFFF,
    "GetCurrentThread": 0xFFFFFFFE,
}


# --- Emulator ------------------------------------------------------------

@dataclass
class Emulator:
    pe: PeInfo
    uc: Uc = field(init=False)
    _import_iat_to_name: dict[int, str] = field(default_factory=dict)
    _call_hooks: dict[int, Callable] = field(default_factory=dict)
    _read_hooks: list = field(default_factory=list)
    _heap_ptr: int = HEAP_BASE
    _step_limit: int = 100_000_000

    def __post_init__(self):
        self.uc = Uc(UC_ARCH_X86, UC_MODE_32)
        self._map_image()
        self._map_stack()
        self._map_heap()
        self._install_import_stubs()
        self._install_code_hook()

    # --- setup ---------------------------------------------------------

    def _map_image(self):
        # Round to 4 KB pages and map contiguously starting from image_base
        # up to the end of the last section.
        max_va = 0
        for name, va, sz, *_ in self.pe.sections:
            end = va + sz
            if end > max_va:
                max_va = end
        span = (max_va - self.pe.image_base + 0xFFF) & ~0xFFF
        self.uc.mem_map(self.pe.image_base, span)
        # Populate each section's bytes.
        for name, va, sz, raw_off, raw_sz in self.pe.sections:
            data = None
            if name == ".text":  data = self.pe.text_bytes
            elif name == ".rdata": data = self.pe.rdata_bytes
            elif name == ".data":  data = self.pe.data_bytes
            if data:
                self.uc.mem_write(va, data)

    def _map_stack(self):
        self.uc.mem_map(STACK_BASE, STACK_SIZE)
        self.uc.reg_write(UC_X86_REG_ESP, STACK_BASE + STACK_SIZE - 0x1000)
        self.uc.reg_write(UC_X86_REG_EBP, STACK_BASE + STACK_SIZE - 0x1000)

    def _map_heap(self):
        self.uc.mem_map(HEAP_BASE, HEAP_SIZE)

    def _install_import_stubs(self):
        """For every import IAT slot, patch a return-stub at IMPORT_STUB_BASE+N
        and write the stub address into the IAT. When the exe indirect-calls
        the IAT slot it lands on our stub instead of the real DLL.
        """
        self.uc.mem_map(IMPORT_STUB_BASE, IMPORT_STUB_SIZE)
        stub_offset = 0
        for dll, entries in self.pe.imports.items():
            for name, iat_va in entries.items():
                if stub_offset + 16 >= IMPORT_STUB_SIZE:
                    break
                stub_va = IMPORT_STUB_BASE + stub_offset
                retval = _NONZERO_IMPORTS.get(name, 0)
                # mov eax, retval ; retn <best-guess argc*4>
                # We don't know argc for each import; use retn 0x40 (16 args)
                # which is safe for most stdcall — extra bytes just clean up
                # stack we don't care about. Better: use RET only and let
                # cdecl callers pop.
                stub = struct.pack("<BI", 0xB8, retval & 0xFFFFFFFF) + b"\xc3"
                self.uc.mem_write(stub_va, stub)
                # Overwrite IAT slot to point at stub
                self.uc.mem_write(iat_va, struct.pack("<I", stub_va))
                self._import_iat_to_name[stub_va] = f"{dll}!{name}"
                stub_offset += 16

    def _install_code_hook(self):
        def _hook(uc, addr, size, user):
            cb = self._call_hooks.get(addr)
            if cb is not None:
                try:
                    cb(self, addr, size)
                except Exception:
                    pass
        self.uc.hook_add(UC_HOOK_CODE, _hook)

    # --- primitives ---------------------------------------------------

    def malloc(self, size: int) -> int:
        addr = self._heap_ptr
        self._heap_ptr = (self._heap_ptr + size + 0xF) & ~0xF
        return addr

    def push(self, val: int):
        esp = self.uc.reg_read(UC_X86_REG_ESP) - 4
        self.uc.reg_write(UC_X86_REG_ESP, esp)
        self.uc.mem_write(esp, struct.pack("<I", val & 0xFFFFFFFF))

    def read_regs(self) -> dict:
        return {
            "eax": self.uc.reg_read(UC_X86_REG_EAX),
            "ebx": self.uc.reg_read(UC_X86_REG_EBX),
            "ecx": self.uc.reg_read(UC_X86_REG_ECX),
            "edx": self.uc.reg_read(UC_X86_REG_EDX),
            "esi": self.uc.reg_read(UC_X86_REG_ESI),
            "edi": self.uc.reg_read(UC_X86_REG_EDI),
            "ebp": self.uc.reg_read(UC_X86_REG_EBP),
            "esp": self.uc.reg_read(UC_X86_REG_ESP),
            "eip": self.uc.reg_read(UC_X86_REG_EIP),
            "eflags": self.uc.reg_read(UC_X86_REG_EFLAGS),
        }

    def read_fpu(self) -> list[float]:
        """Read all 8 FPU register slots as doubles. Slot 0 = ST(0),
        slot 7 = ST(7).
        """
        out = []
        for reg in [UC_X86_REG_ST0, UC_X86_REG_ST1, UC_X86_REG_ST2,
                    UC_X86_REG_ST3, UC_X86_REG_ST4, UC_X86_REG_ST5,
                    UC_X86_REG_ST6, UC_X86_REG_ST7]:
            try:
                raw = self.uc.reg_read(reg)
            except Exception:
                raw = 0
            if isinstance(raw, tuple):
                # some unicorn builds return (mantissa, exponent)
                mant, exp = raw
                out.append(_x87_to_double(mant, exp))
            else:
                out.append(float("nan"))
        return out

    def read_va(self, va: int, n: int) -> bytes:
        try:
            return bytes(self.uc.mem_read(va, n))
        except Exception:
            return b""

    def write_va(self, va: int, data: bytes):
        try:
            self.uc.mem_write(va, data)
        except Exception:
            pass

    # --- call / hook --------------------------------------------------

    def call(self, addr: int, *args: int, thiscall: bool = False,
             max_ticks: int = 10_000_000) -> int:
        """Run fn at `addr` with `args` as cdecl (or thiscall if flagged).
        Returns EAX after `retn`. Sets a sentinel return address so we
        detect return."""
        SENTINEL = 0x30000000
        # Ensure sentinel is mapped — 4KB parked space
        try:
            self.uc.mem_map(SENTINEL & ~0xFFF, 0x1000)
        except Exception:
            pass  # already mapped
        # cdecl: push args right-to-left, push sentinel, jump.
        # thiscall: ECX = args[0], push args[1:] right-to-left.
        if thiscall and args:
            self.uc.reg_write(UC_X86_REG_ECX, args[0] & 0xFFFFFFFF)
            call_args = args[1:]
        else:
            call_args = args
        for v in reversed(call_args):
            self.push(v)
        self.push(SENTINEL)
        try:
            self.uc.emu_start(addr, SENTINEL, count=max_ticks)
        except Exception as e:
            # Return whatever EAX currently holds; report exception via .exc
            self.last_exc = repr(e)
            return self.uc.reg_read(UC_X86_REG_EAX)
        self.last_exc = None
        return self.uc.reg_read(UC_X86_REG_EAX)

    def hook_call(self, addr: int, cb: Callable[["Emulator", int, int], None]):
        """Register a callback to fire when execution reaches `addr`.
        `cb(emu, addr, size)`."""
        self._call_hooks[addr] = cb

    def hook_reads_in(self, lo_va: int, hi_va: int,
                      cb: Callable[["Emulator", int, int], None]):
        def _reader(uc, access, addr, size, value, user):
            if lo_va <= addr < hi_va:
                cb(self, addr, size)
        h = self.uc.hook_add(UC_HOOK_MEM_READ, _reader)
        self._read_hooks.append(h)


def _x87_to_double(mant: int, exp: int) -> float:
    """Convert Unicorn's (mantissa, exponent) x87 register pair to a
    Python float. The 80-bit x87 layout is 64 mantissa bits + 15
    exponent bits + 1 sign bit; Python's float is 64-bit IEEE, so we
    lose a few bits of precision but the value is accurate enough for
    every constant we care about lifting.
    """
    if mant == 0 and exp == 0:
        return 0.0
    sign = (exp >> 15) & 1
    e = exp & 0x7FFF
    if e == 0 and mant == 0:
        return 0.0
    # Reconstruct: (-1)^s * mant / 2^63 * 2^(e - 16383)
    try:
        val = mant / (1 << 63) * (2 ** (e - 16383))
        return -val if sign else val
    except OverflowError:
        return float("inf") * (-1 if sign else 1)


def make_emulator(pe: Optional[PeInfo] = None) -> Emulator:
    return Emulator(pe or load_pe())
