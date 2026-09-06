#!/usr/bin/env python3
"""Dump a fully-decoded pool from the editor memory image.

Once find_tobject.py has located a pool (base + stride + field
layout), this walks every record and emits structured JSON — the same
data the editor would show if you clicked through every record.

Currently implemented:
  colour   — 34 records at 0x09fa8c88, stride 64 (VERIFIED)

Add more pools by editing POOLS below once find_tobject locates them.

Usage:
    py dump_pool.py                    # dump every known pool
    py dump_pool.py --pool colour      # single pool
    py dump_pool.py --out editor_pools.json
"""
import argparse, json, struct
from pathlib import Path

HERE = Path(__file__).parent


def load_ranges(dump_dir):
    idx = json.load((dump_dir / 'index.json').open())
    return [{
        'base': int(r['base'], 16), 'size': r['size'], 'file': r['file'],
        'data': (dump_dir / r['file']).read_bytes(),
    } for r in idx['ranges']]


def read_at(ranges, addr, n):
    for r in ranges:
        off = addr - r['base']
        if 0 <= off < r['size']:
            return r['data'][off:off + n]
    return None


def decode_colour(rec):
    """One 64-byte TColour record. Layout inferred from Chester's editor
    screenshot + colour.dat ground truth:
       +0x00..+0x1f (32B): name (latin-1, null-terminated in 32 buffer)
       +0x36..+0x39 (3B):  RGB triple
       +0x3c..+0x40 (4B):  id (u32, editor-assigned pool index)
    """
    name_end = rec.find(b'\x00')
    name = rec[:name_end].decode('latin-1', 'replace') if name_end > 0 else ''
    r, g, b = rec[0x36], rec[0x37], rec[0x38]
    id_u32 = struct.unpack('<I', rec[0x3c:0x40])[0]
    return {'id': id_u32, 'name': name, 'r': r, 'g': g, 'b': b}


# Pool registry: name -> (base_addr, stride, count, decoder).
POOLS = {
    'colour': (0x09fa8c88, 64, 34, decode_colour),
}


def dump(ranges, name):
    base, stride, count, decoder = POOLS[name]
    out = []
    for i in range(count):
        rec = read_at(ranges, base + i * stride, stride)
        if rec is None: continue
        try: out.append(decoder(rec))
        except Exception as e: out.append({'error': str(e), 'idx': i})
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--dump', default=str(HERE / 'editor_dump'))
    ap.add_argument('--out',  default=str(HERE / 'editor_pools.json'))
    ap.add_argument('--pool', default=None,
                    help=f'one of: {", ".join(POOLS)}')
    args = ap.parse_args()

    ranges = load_ranges(Path(args.dump))
    print(f'[+] {len(ranges)} ranges loaded')

    pools = [args.pool] if args.pool else list(POOLS)
    all_data = {}
    for name in pools:
        if name not in POOLS:
            print(f'  unknown pool {name}'); continue
        entries = dump(ranges, name)
        all_data[name] = entries
        print(f'  {name:12}  {len(entries):>5} records dumped')
        for e in entries[:6]:
            print(f'    {e}')

    Path(args.out).write_text(json.dumps(all_data, indent=2))
    print(f'[+] wrote {args.out}')


if __name__ == '__main__':
    main()
