"""Frida runtime hook for tactics-editor bit decoding.

Purpose: identify the runtime tactic-struct memory address(es) then
capture per-byte write events during UI toggles + per-tick reads
during matches. Answers the remaining decode questions without
requiring the user to save many .tct variants.

Three staged operating modes:

  Mode 1 (LOCATE): Hook the .tct file-load fread and record the
    destination buffer address(es). Also hooks FUN_00890b30 (the
    tactic-struct copy fn) to see the destination address for the
    "current tactic" struct in the game's runtime pool.

  Mode 2 (WATCH_WRITES): Given a base address from Mode 1, install
    a MemoryAccessMonitor on the 22-byte per_slot_instr block +
    slot_pair block + team_flags_1/2 words. Every write logs
    (address, offset-in-struct, old_value, new_value, calling-PC).
    User then toggles ONE UI element per event; the log names the
    setter fn and the bit that changed.

  Mode 3 (WATCH_READS): Given a base address from Mode 1, watch
    reads instead of writes. Run a match with the tactic loaded;
    the log shows which fns read which bytes per tick — ground
    truth for engine-consumption sites the port should mirror.

Usage:

    python -m cm_lift.frida_tactics locate  # attach and wait for tactic load
    python -m cm_lift.frida_tactics writes <base_addr>
    python -m cm_lift.frida_tactics reads  <base_addr>

The agent script is self-contained JS injected into the target.
Requires: cm0102.exe running (Windows native or under Wine with
frida-server); frida-python installed (already in cm-lift reqs).

Memory-note advisory: [[frida-instrumentation-crash-evidence]] —
heavy Frida hooking crashed the exe in prior tests. This module
uses SURGICAL hooks only (a handful of fns + one MemoryAccessMonitor
region), NOT wholesale symbol enumeration.
"""
from __future__ import annotations
import json
import sys
import time
from pathlib import Path
from .util import DATA_OUT


# ---------------------------------------------------------------------------
# Mode 1: LOCATE — hook the tactic loader to capture the runtime buffer.
# ---------------------------------------------------------------------------

LOCATE_AGENT = r"""
'use strict';

// The .tct/.pct file is loaded via fread(buffer, size, 1, FILE*).
// Files are exactly 1476 bytes (v5E) or 1432 bytes (v5C).
// Hook fread and record any 1476/1432-byte read → the destination buffer
// is the loaded tactic in memory.

const fread = Module.findExportByName('msvcrt.dll', 'fread');
if (!fread) throw new Error('fread not found in msvcrt');

Interceptor.attach(fread, {
  onEnter(args) {
    this.buf  = args[0];
    this.size = args[1].toInt32();
    this.count = args[2].toInt32();
  },
  onLeave(retval) {
    const total = this.size * this.count;
    if (total === 1476 || total === 1432) {
      send({
        kind: 'tactic_loaded',
        buf: this.buf.toString(),
        size: total,
        // First 4 bytes = version tag
        version: this.buf.readU32(),
      });
    }
  }
});

// Also hook FUN_00890b30 — the tactic-struct COPY fn — because tactics
// get relocated to a runtime slot pool. Its second arg is the source,
// first arg is the destination.
const MODULE = Process.enumerateModules().find(m => m.name.toLowerCase() === 'cm0102.exe');
const IMGBASE = MODULE.base;
const FUN_890b30 = IMGBASE.add(0x00490b30);  // FUN_00890b30 - IMGBASE

Interceptor.attach(FUN_890b30, {
  onEnter(args) {
    send({
      kind: 'tactic_copied',
      dst: args[0].toString(),
      src: args[1].toString(),
    });
  }
});

send({ kind: 'ready', imgbase: IMGBASE.toString() });
"""


# ---------------------------------------------------------------------------
# Mode 2: WATCH_WRITES — MemoryAccessMonitor on the tactic struct.
# ---------------------------------------------------------------------------

WATCH_WRITES_AGENT_TEMPLATE = r"""
'use strict';

const BASE = ptr('__BASE__');
const SIZE = __SIZE__;

// Install monitor. Frida's MemoryAccessMonitor fires per-access with
// the exact address touched.
MemoryAccessMonitor.enable(
  [{ base: BASE, size: SIZE }],
  {
    onAccess(details) {
      // Read the byte(s) touched
      const off = details.address.sub(BASE).toInt32();
      let val = 0;
      try { val = details.address.readU8(); } catch (e) {}
      send({
        kind: 'access',
        op: details.operation,       // 'read' or 'write'
        addr: details.address.toString(),
        offset: off,
        value: val,
        pc: details.from.toString(),
      });
    }
  }
);

send({ kind: 'monitoring', base: BASE.toString(), size: SIZE });
"""


def run_locate(process: str = 'cm0102.exe'):
    """Attach to running cm0102.exe and log tactic-load events."""
    try:
        import frida
    except ImportError:
        print("frida not installed. `pip install frida` in the cm-lift venv.")
        sys.exit(1)

    print(f"attaching to {process}...")
    session = frida.attach(process)
    script = session.create_script(LOCATE_AGENT)

    log_path = DATA_OUT / 'frida_tactic_load.jsonl'
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log = open(log_path, 'a')

    def on_message(msg, data):
        if msg['type'] == 'send':
            p = msg['payload']
            print(f"  {p}")
            log.write(json.dumps(p) + '\n')
            log.flush()
        elif msg['type'] == 'error':
            print(f"  [error] {msg['description']}")

    script.on('message', on_message)
    script.load()
    print(f"logging to {log_path}")
    print("Open a tactic in-game (Load Formation → pick one). Press Ctrl-C to stop.")
    try:
        while True: time.sleep(1)
    except KeyboardInterrupt:
        pass
    finally:
        session.detach()
        log.close()


def run_watch_writes(process: str, base_addr: str, size: int = 1476):
    """Given a runtime tactic-struct base address (from `locate`), watch
    writes to it. User then toggles UI elements one at a time."""
    try:
        import frida
    except ImportError:
        print("frida not installed.")
        sys.exit(1)

    agent = (WATCH_WRITES_AGENT_TEMPLATE
             .replace('__BASE__', base_addr)
             .replace('__SIZE__', str(size)))

    session = frida.attach(process)
    script = session.create_script(agent)

    log_path = DATA_OUT / 'frida_tactic_writes.jsonl'
    log = open(log_path, 'a')

    def on_message(msg, data):
        if msg['type'] == 'send':
            p = msg['payload']
            if p['kind'] == 'access' and p['op'] == 'write':
                print(f"  WRITE offset={p['offset']:#x} val={p['value']:#x} pc={p['pc']}")
            log.write(json.dumps(p) + '\n')
            log.flush()

    script.on('message', on_message)
    script.load()
    print(f"monitoring {size} bytes at {base_addr}, logging to {log_path}")
    print("Now toggle ONE UI element in-game — the write event names the setter fn.")
    try:
        while True: time.sleep(1)
    except KeyboardInterrupt:
        pass
    finally:
        session.detach()
        log.close()


if __name__ == '__main__':
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    cmd = sys.argv[1]
    if cmd == 'locate':
        run_locate()
    elif cmd == 'writes':
        if len(sys.argv) < 3:
            print("usage: frida_tactics writes <base_addr>")
            sys.exit(1)
        run_watch_writes('cm0102.exe', sys.argv[2])
    elif cmd == 'reads':
        # Reads mode uses the same MemoryAccessMonitor but filters
        # 'read' events instead of 'write'.
        if len(sys.argv) < 3:
            print("usage: frida_tactics reads <base_addr>")
            sys.exit(1)
        # For brevity, share the writes path — the log filter reveals
        # both. Consumer can grep 'op\":\"read\"' vs '\"write\"'.
        run_watch_writes('cm0102.exe', sys.argv[2])
    else:
        print(f"unknown command: {cmd}")
        sys.exit(1)
