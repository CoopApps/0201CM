#!/usr/bin/env python3
"""Decode the schedule_getter enter records emitted by the
C11.3 Phase-A enhanced season_capture harness.

For every `sub_0055F540_schedule_getter` enter record, prints:
  wall-clock ms, arg0 (raw + signed), competition identity, and
  the return address of the immediate caller.

The competition identity is decoded from the 128-byte hex dump
in `comp_record.hex128` (arg0 dereferenced when arg0 is a
plausible heap pointer). Fields extracted use the pool-record
conventions from memory [[record-layouts-decoded]]:

  +0x00 u32   id
  +0x04 str   name (Latin-1, null-terminated within its slot)
  +0x53 str   three-letter name (blank on cups / feeder divs)
  +0x5d i32   nation id
  +0x69 u16   reputation

Usage:
    D:/Python312/python.exe tools/gdi_capture/decode_schedule_getter_calls.py \\
        fixtures/gdi_captures/season_2001_02_boot_plus_2seasons.jsonl
"""
from __future__ import annotations
import argparse
import json
import struct
import sys
from pathlib import Path


def parse_args() -> argparse.Namespace:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("jsonl", type=Path)
    return ap.parse_args()


def decode_latin1_cstr(b: bytes) -> str:
    end = b.find(b"\x00")
    if end == -1: end = len(b)
    try:
        return b[:end].decode("latin-1", errors="replace")
    except Exception:
        return "?"


def decode_comp_record(hex128: str | None) -> dict:
    if not hex128 or len(hex128) < 256:
        return {}
    try:
        raw = bytes.fromhex(hex128)
    except ValueError:
        return {"decode_err": "not hex"}
    if len(raw) < 128:
        return {"decode_err": f"short: {len(raw)} bytes"}
    out: dict = {}
    out["id"]              = struct.unpack_from("<I", raw, 0x00)[0]
    out["name"]            = decode_latin1_cstr(raw[0x04:0x53])
    out["three_letter"]    = decode_latin1_cstr(raw[0x53:0x56])
    if len(raw) >= 0x61:
        out["nation_id"]   = struct.unpack_from("<i", raw, 0x5d)[0]
    if len(raw) >= 0x6b:
        out["reputation"]  = struct.unpack_from("<H", raw, 0x69)[0]
    return out


def main() -> int:
    args = parse_args()
    if not args.jsonl.exists():
        print(f"missing: {args.jsonl}", file=sys.stderr); return 2

    n_getter = 0
    for line in args.jsonl.open("r", encoding="utf-8"):
        line = line.strip()
        if not line: continue
        try: rec = json.loads(line)
        except json.JSONDecodeError: continue
        hook = rec.get("hook", "")
        # C11.3: decode both getter and driver — both take a
        # Comp* first arg and both were enriched in Phase A.
        if not (hook == "sub_0055F540_schedule_getter"
                or hook == "FUN_00668450_shared_driver"):
            continue
        if rec.get("phase") != "enter": continue
        n_getter += 1

        ms = rec.get("ms")
        # v1 harness (pre A.2): arg0_raw + comp_record.
        # v2 harness (post A.2 __thiscall fix):
        #   stack_arg0_raw + this_ptr + this_record.
        mode_raw = rec.get("stack_arg0_raw") or rec.get("arg0_raw") or rec.get("arg0")
        mode_int = (rec.get("stack_arg0_int")
                    if "stack_arg0_int" in rec else rec.get("arg0_int"))
        this_ptr = rec.get("this_ptr")
        ret_addr = rec.get("return_addr")
        rng      = rec.get("rng")
        this_rec = rec.get("this_record") or rec.get("comp_record") or {}
        decoded  = decode_comp_record(this_rec.get("hex128"))
        sentinel = this_rec.get("sentinel")

        short_hook = hook.split("_", 1)[1] if "_" in hook else hook
        print(f"call #{n_getter}  {short_hook}  ms={ms}  rng={rng}")
        print(f"    mode (stack arg0) = {mode_raw}  (signed={mode_int})")
        print(f"    this  (ecx)       = {this_ptr}")
        print(f"    ret_addr          = {ret_addr}")
        if sentinel is not None:
            print(f"    this is a sentinel value ({sentinel}); no record to decode")
        elif this_rec.get("err"):
            print(f"    this deref error: {this_rec['err']}")
        elif decoded:
            print(f"    this-> id={decoded.get('id')}  name={decoded.get('name')!r}  "
                  f"3l={decoded.get('three_letter')!r}  "
                  f"nation={decoded.get('nation_id')}  "
                  f"rep={decoded.get('reputation')}")
        else:
            print("    (no comp record data)")
        print()

    if n_getter == 0:
        print("No schedule_getter records found. The enhanced harness "
              "may not have been used to produce this capture. Re-run "
              "season_capture.py against the current manifest.",
              file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
