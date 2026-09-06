#!/usr/bin/env python3
"""Attach to cm0102ed.exe, load hook_editor.js, stream events to a JSONL file.

Usage:
  py attach.py                       # attach to running editor, log to editor_events.jsonl
  py attach.py --pid 10744           # explicit pid
  py attach.py --out capture.jsonl   # custom log path
  py attach.py --spawn path\to\cm0102ed.exe   # spawn + hook from load

Load index.dat in the editor BEFORE running the script — the file-load traffic
is what we're capturing. If you want to see the load itself, use --spawn and
open index.dat after attach.
"""
import argparse, json, sys, time, os
from pathlib import Path

try:
    import frida
except ImportError:
    sys.exit("Install frida: py -m pip install frida frida-tools")

HERE = Path(__file__).parent
SCRIPT = (HERE / 'hook_editor.js').read_text(encoding='utf-8')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--pid', type=int, default=None)
    ap.add_argument('--name', default='cm0102ed.exe')
    ap.add_argument('--out', default=str(HERE / 'editor_events.jsonl'))
    ap.add_argument('--spawn', default=None,
                    help='Path to cm0102ed.exe — spawns instead of attaching')
    ap.add_argument('--quiet', action='store_true',
                    help='Only write JSONL, do not echo to stdout')
    args = ap.parse_args()

    if args.spawn:
        pid = frida.spawn([args.spawn])
        session = frida.attach(pid)
        print(f'[+] spawned pid={pid}, hooking then resuming')
    elif args.pid:
        session = frida.attach(args.pid)
        print(f'[+] attached to pid={args.pid}')
    else:
        # Find by name
        dev = frida.get_local_device()
        pids = [p for p in dev.enumerate_processes() if p.name.lower() == args.name.lower()]
        if not pids:
            sys.exit(f'no process named {args.name} running')
        if len(pids) > 1:
            print(f'[!] multiple {args.name}: {[p.pid for p in pids]}, using first')
        pid = pids[0].pid
        session = frida.attach(pid)
        print(f'[+] attached to pid={pid}')

    out_path = Path(args.out)
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out = out_path.open('w', encoding='utf-8')
    print(f'[+] logging to {out_path}')

    counts = {'total': 0}
    def on_message(msg, data):
        if msg['type'] == 'send':
            payload = msg['payload']
            out.write(json.dumps(payload, ensure_ascii=False) + '\n')
            out.flush()
            counts['total'] += 1
            if counts['total'] % 1000 == 0:
                print(f'    events: {counts["total"]}')
            if not args.quiet and counts['total'] <= 20:
                print('   ', payload)
        elif msg['type'] == 'error':
            print('[error]', msg.get('description'), file=sys.stderr)
            print(msg.get('stack', ''), file=sys.stderr)

    script = session.create_script(SCRIPT)
    script.on('message', on_message)
    script.load()
    print('[+] script loaded, hooks armed')

    if args.spawn:
        frida.resume(pid)
        print('[+] resumed spawned process')

    print('[+] streaming events. Press Ctrl-C to stop.')
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        print(f'\n[+] stopping. total events: {counts["total"]}')
    finally:
        out.close()
        session.detach()


if __name__ == '__main__':
    main()
