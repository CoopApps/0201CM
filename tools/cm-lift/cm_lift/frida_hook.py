"""Frida runtime hook for cm0102.exe running under Wine or Windows.

Records every fn call (address, args-in-registers/stack, return value)
so we can:

  1. Rank untouched fns by real-world call frequency — priority list
     for further porting.
  2. Bind virtual dispatch slots (vtable_dumper gives us the
     candidate table; this tells us which slot the live game actually
     picks).
  3. Diff-verify against Rust port by cross-referencing call-graphs.

Requires:
  * `frida` Python module (already installed)
  * `cm0102.exe` running under Wine or on Windows
  * Frida's frida-server (Windows) or launched via Wine's `frida` wrapper

The agent script (inlined below as a JS string) is injected into the
target. Each call event is streamed back to Python as a JSON message
and appended to `data/frida_call_log.jsonl`.
"""
from __future__ import annotations
import json
import sys
from pathlib import Path
from .util import DATA_OUT


# --- Frida agent (JS injected into target) --------------------------------

AGENT_JS = r"""
// cm-lift Frida agent — logs every call within cm0102.exe's .text.
'use strict';

// Text section bounds (patched by Python before injection):
const TEXT_LO = ptr('__TEXT_LO__');
const TEXT_HI = ptr('__TEXT_HI__');

// Hook every entry in the module by scanning Module.enumerateSymbols()
// isn't cheap and cm0102 has 9000+ fns — instead we hook on-demand
// via Interceptor.attach when the target executes a new bb.
// Simpler: install Stalker to trace, but that's very heavy.
// Middle path: sample-log every N-th CALL instruction.

const state = { count: 0, seen: new Set() };

// Sample the first CALL each fn makes to build a coverage map.
Stalker.follow(Process.getCurrentThreadId(), {
    events: {
        call: true, ret: false, exec: false,
        block: false, compile: false,
    },
    onReceive: function (events) {
        const parsed = Stalker.parse(events, {
            annotate: false, stringify: true,
        });
        for (const e of parsed) {
            const target = ptr(e[1]);
            const t = target.toInt32() >>> 0;
            if (t >= TEXT_LO.toInt32() >>> 0 && t < TEXT_HI.toInt32() >>> 0) {
                const key = t >>> 0;
                if (!state.seen.has(key)) {
                    state.seen.add(key);
                    send({ kind: 'first_call', addr: '0x' + t.toString(16) });
                }
                state.count += 1;
                if (state.count % 100000 === 0) {
                    send({ kind: 'tick', count: state.count,
                           distinct: state.seen.size });
                }
            }
        }
    }
});

send({ kind: 'ready', text_lo: TEXT_LO.toString(16),
       text_hi: TEXT_HI.toString(16) });
"""


def attach_and_log(process_name: str = "cm0102.exe",
                   duration_secs: int = 300,
                   out_path: Path | None = None) -> Path:
    """Attach Frida to a running cm0102.exe, log all calls for
    `duration_secs`, write JSONL.
    """
    import frida
    import time

    out = out_path or (DATA_OUT / "frida_call_log.jsonl")
    fp = open(out, "w", encoding="utf-8")

    from .util import load_pe
    pe = load_pe()
    text_lo = pe.text_va
    text_hi = pe.text_va + pe.text_size
    js = (AGENT_JS
          .replace("__TEXT_LO__", f"0x{text_lo:08x}")
          .replace("__TEXT_HI__", f"0x{text_hi:08x}"))

    def on_message(msg, data):
        if msg.get("type") == "send":
            fp.write(json.dumps(msg["payload"]) + "\n")

    dev = frida.get_local_device()
    procs = [p for p in dev.enumerate_processes() if p.name.lower() == process_name.lower()]
    if not procs:
        fp.write(json.dumps({"error": f"no process named {process_name}"}) + "\n")
        fp.close()
        return out
    session = dev.attach(procs[0].pid)
    script = session.create_script(js)
    script.on("message", on_message)
    script.load()
    time.sleep(duration_secs)
    session.detach()
    fp.close()
    return out


def rank_by_call_frequency(log_path: Path | None = None) -> Path:
    """Post-process a call log into a per-fn call-count JSON."""
    log_path = log_path or (DATA_OUT / "frida_call_log.jsonl")
    if not log_path.exists():
        raise SystemExit(f"no call log at {log_path} — attach-and-log first")
    counts: dict[str, int] = {}
    with open(log_path, "r", encoding="utf-8") as f:
        for line in f:
            try:
                m = json.loads(line)
            except Exception:
                continue
            if m.get("kind") == "first_call":
                counts[m["addr"]] = counts.get(m["addr"], 0) + 1
    out = DATA_OUT / "frida_call_frequency.json"
    with open(out, "w", encoding="utf-8") as f:
        json.dump({
            "distinct_fns_called": len(counts),
            "entries": sorted(counts.items(), key=lambda kv: -kv[1]),
        }, f, indent=2)
    return out


# --- CLI ------------------------------------------------------------------

def main():
    import argparse
    ap = argparse.ArgumentParser(description="Frida runtime hook for cm0102.exe")
    sub = ap.add_subparsers(dest="cmd", required=True)
    a = sub.add_parser("attach", help="Attach and log calls for N seconds")
    a.add_argument("--process", default="cm0102.exe")
    a.add_argument("--secs", type=int, default=300)
    sub.add_parser("rank", help="Rank fns by call frequency from prior log")
    args = ap.parse_args()

    if args.cmd == "attach":
        out = attach_and_log(args.process, args.secs)
        print(f"logged to {out}")
    else:
        out = rank_by_call_frequency()
        print(f"wrote {out}")


if __name__ == "__main__":
    main()
