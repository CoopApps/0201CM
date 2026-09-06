#!/usr/bin/env python3
"""Snapshot cm0102ed.exe's memory image once it has finished loading index.dat.

Two phases:

  phase 1  — dump every RW committed range in the process to
             editor_dump/range_<baseHex>.bin + editor_dump/index.json

  phase 2  — offline: search the dumps for known signatures ("1.FC Bocholt",
             "Chester City", "Blue 3", the shipped colour.dat byte pattern,
             ...) → each hit pins a pool base + stride to a specific range.

The editor must be running with the DB loaded BEFORE you invoke this script.
Load index.dat first (File → Open → D:\\cm0102\\Data\\index.dat), let it
finish, THEN run.

    py dump_memory.py                     # full dump
    py dump_memory.py --scan "Chester"    # hex/ascii search only, no dump
    py dump_memory.py --list-ranges       # summarise ranges, no dump

Output goes to `tools/editor_frida/editor_dump/` by default.
"""
import argparse, json, sys, time
from pathlib import Path

try:
    import frida
except ImportError:
    sys.exit("Install frida: py -m pip install frida frida-tools")

HERE   = Path(__file__).parent
SCRIPT = (HERE / 'dump_memory.js').read_text(encoding='utf-8')


def attach(pid=None, name='cm0102ed.exe'):
    if pid:
        return frida.attach(pid), pid
    dev = frida.get_local_device()
    pids = [p for p in dev.enumerate_processes() if p.name.lower() == name.lower()]
    if not pids:
        sys.exit(f'no process named {name} is running — launch the editor first')
    if len(pids) > 1:
        print(f'[!] multiple {name}: {[p.pid for p in pids]}, using first')
    return frida.attach(pids[0].pid), pids[0].pid


def load_script(session):
    ready = {'ok': False}
    def on_msg(msg, data):
        if msg['type'] == 'send' and msg.get('payload', {}).get('src') == 'dump_ready':
            ready['ok'] = True
        elif msg['type'] == 'error':
            print('[script error]', msg.get('description'), file=sys.stderr)
            print(msg.get('stack', ''), file=sys.stderr)
    script = session.create_script(SCRIPT)
    script.on('message', on_msg)
    script.load()
    for _ in range(30):
        if ready['ok']: break
        time.sleep(0.05)
    return script


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--pid',  type=int, default=None)
    ap.add_argument('--name', default='cm0102ed.exe')
    ap.add_argument('--out',  default=str(HERE / 'editor_dump'))
    ap.add_argument('--list-ranges', action='store_true',
                    help='List RW ranges without dumping.')
    ap.add_argument('--scan', default=None,
                    help='ASCII pattern to search across all RW ranges — no dump.')
    ap.add_argument('--min-size', type=int, default=64 * 1024,
                    help='Skip ranges smaller than this (default 64 KiB).')
    ap.add_argument('--max-size', type=int, default=64 * 1024 * 1024,
                    help='Skip ranges larger than this (default 64 MiB).')
    args = ap.parse_args()

    session, pid = attach(args.pid, args.name)
    print(f'[+] attached to pid={pid}')
    script = load_script(session)
    print('[+] script loaded')

    api = script.exports_sync

    # Modules — for context in the JSON index.
    modules = api.list_modules()
    print(f'[+] {len(modules)} modules loaded')

    # ---- scan mode: just find the pattern and print hits ----
    if args.scan:
        pattern_hex = args.scan.encode('latin-1').hex(' ').upper()
        print(f'[+] scanning for {args.scan!r} -> hex "{pattern_hex}"')
        hits = api.find_bytes(pattern_hex)
        print(f'[+] {len(hits)} hits (max 200)')
        for h in hits[:50]:
            base = int(h['range_base'], 16)
            addr = int(h['addr'], 16)
            print(f'    range=0x{base:08x}+{h["range_size"]:>10}  '
                  f'addr=0x{addr:08x}  off=+0x{h["offset_in_range"]:x}')
        session.detach()
        return

    # ---- list mode: dump the range table only ----
    ranges = api.list_ranges('rw-')
    print(f'[+] {len(ranges)} RW ranges, '
          f'total {sum(r["size"] for r in ranges):,} bytes')
    if args.list_ranges:
        for r in ranges:
            print(f'    {r["base"]:>12}  {r["size"]:>12,}  {r["protection"]}'
                  + (f'  ({r["file"]["path"]})' if r["file"] else ''))
        session.detach()
        return

    # ---- dump mode: write every reasonable RW range to disk ----
    out_dir = Path(args.out); out_dir.mkdir(parents=True, exist_ok=True)
    index = {'pid': pid, 'modules': modules, 'ranges': []}
    dumped_bytes = 0
    for i, r in enumerate(ranges):
        if r['size'] < args.min_size or r['size'] > args.max_size:
            continue
        try:
            data = api.dump_range(r['base'], r['size'])
        except Exception as e:
            print(f'    [!] skip {r["base"]} +{r["size"]}: {e}')
            continue
        if not data:
            continue
        # `data` is a bytes object (Frida turns ArrayBuffer into bytes).
        fname = f'range_{r["base"]}.bin'
        (out_dir / fname).write_bytes(bytes(data))
        index['ranges'].append({**r, 'file': fname})
        dumped_bytes += r['size']
        if len(index['ranges']) % 25 == 0:
            print(f'    dumped {len(index["ranges"])} ranges, '
                  f'{dumped_bytes:,} bytes so far')

    (out_dir / 'index.json').write_text(json.dumps(index, indent=2))
    print(f'[+] done. {len(index["ranges"])} ranges, '
          f'{dumped_bytes:,} bytes -> {out_dir}')
    session.detach()


if __name__ == '__main__':
    main()
