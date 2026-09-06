#!/usr/bin/env python3
"""Second-generation pool locator — enforces multi-record verification.

Approach: for each .dat, sample N record signatures at known offsets in
the shipped file, search each in memory, and only accept a candidate
pool_base for which >= 3 signatures land at the correct relative offsets
in one range.

The v1 script accepted any single 32-byte match, which for small
records catches random collisions (e.g. shipped-file-header bytes
appearing in a text buffer). Diffing against those bogus bases returned
0% byte-perfect, which is what surfaced the issue.

Enforced verification means: if we report pool_base X, then
memory[X + rec_i * stride : X + rec_i * stride + sig_len] MUST equal
disk[rec_i * stride : rec_i * stride + sig_len] for at least 3 distinct
rec_i values.
"""
import argparse, json, sys
from pathlib import Path

HERE = Path(__file__).parent


def load_ranges(dump_dir):
    idx = json.load((dump_dir / 'index.json').open())
    return [{
        'base': int(r['base'], 16),
        'file': r['file'],
        'size': r['size'],
        'data': (dump_dir / r['file']).read_bytes(),
    } for r in idx['ranges']]


def find_all(ranges, needle):
    """Yield (range, offset_in_range) for every occurrence of needle."""
    for r in ranges:
        d = r['data']; i = 0
        while True:
            j = d.find(needle, i)
            if j == -1: break
            yield (r, j)
            i = j + 1


# Same strides as v1, plus staff.dat noted as multi-section.
KNOWN_SIZES = {
    'club.dat':                 581,
    'nat_club.dat':             581,
    'nation.dat':               290,
    'colour.dat':                58,
    'continent.dat':            198,
    'club_comp.dat':            107,
    'nation_comp.dat':          107,
    'staff_comp.dat':           107,
    'city.dat':                  76,
    'stadium.dat':               44,
    'first_names.dat':           70,
    'second_names.dat':          76,
    'common_names.dat':          76,
    'officials.dat':             44,
    'staff_comp_history.dat':    58,
    'club_comp_history.dat':     44,
    'nation_comp_history.dat':   44,
    'staff_history.dat':         58,
}


def find_pool(raw, ranges, stride, sig_len=64, min_verified=3,
              max_probes=20):
    """Return (range, base_offset) that satisfies >= min_verified probes,
    or None."""
    n = len(raw)
    max_recs = n // stride
    if max_recs < min_verified: return None

    # Spread probes across the file — skips leading null slot, avoids
    # trailing padding. If the .dat file has 4251 records we pick
    # records 1, 100, 500, 1000, 2000, 3000, 4000 etc.
    step = max(1, max_recs // (max_probes + 1))
    probes = []
    for k in range(1, max_probes + 1):
        rec = k * step
        if rec >= max_recs: break
        off = rec * stride
        if off + sig_len > n: break
        sig = raw[off:off + sig_len]
        if sig == b'\x00' * sig_len:
            continue                    # all-null record, useless as sig
        probes.append((off, sig))
    if len(probes) < min_verified: return None

    # For each probe hit, compute implied pool_base = hit_off - probe_off.
    # Accumulate votes per (range, pool_base) pair.
    votes = {}
    for probe_off, sig in probes:
        for r, hit_off in find_all(ranges, sig):
            base_off = hit_off - probe_off
            if base_off < 0: continue
            key = (id(r), base_off)
            votes[key] = votes.get(key, {'range': r, 'base_off': base_off,
                                        'matched_probes': set()})
            votes[key]['matched_probes'].add(probe_off)

    # Best candidate = most matched probes.
    best = None
    for v in votes.values():
        if best is None or len(v['matched_probes']) > len(best['matched_probes']):
            best = v
    if not best or len(best['matched_probes']) < min_verified:
        return None
    return best


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--data', default='D:/cm0102/Data')
    ap.add_argument('--dump', default=str(HERE / 'editor_dump'))
    ap.add_argument('--out',  default=str(HERE / 'pool_map.json'))
    ap.add_argument('--sig-len', type=int, default=64,
                    help='Bytes per probe signature (default 64).')
    ap.add_argument('--min-verified', type=int, default=3,
                    help='Minimum probes that must line up (default 3).')
    args = ap.parse_args()

    dump_dir = Path(args.dump)
    ranges = load_ranges(dump_dir)
    print(f'[+] {len(ranges)} ranges, '
          f'{sum(r["size"] for r in ranges):,} bytes')

    data_dir = Path(args.data)
    pool_map = {}
    for dat_path in sorted(data_dir.glob('*.dat')):
        name = dat_path.name
        raw = dat_path.read_bytes()
        stride = KNOWN_SIZES.get(name)
        if not stride:
            print(f'  {name:26}  (no known stride — skipped)')
            pool_map[name] = {'skipped': 'no known stride',
                              'file_size': len(raw)}
            continue
        result = find_pool(raw, ranges, stride,
                           sig_len=args.sig_len,
                           min_verified=args.min_verified)
        if not result:
            print(f'  {name:26}  stride={stride:>3}  count={len(raw)//stride:>6}  '
                  f'-- NOT FOUND (needs >= {args.min_verified} verified probes)')
            pool_map[name] = {
                'dat': name, 'stride': stride,
                'record_count': len(raw) // stride,
                'file_size': len(raw), 'verified': False,
                'note': f'no candidate satisfied >= {args.min_verified} probes',
            }
            continue
        r = result['range']
        base_off = result['base_off']
        matched = len(result['matched_probes'])
        pool_base = r['base'] + base_off
        print(f'  {name:26}  stride={stride:>3}  count={len(raw)//stride:>6}  '
              f'verified={matched:>2}  base=0x{pool_base:08x}')
        pool_map[name] = {
            'dat': name, 'stride': stride,
            'record_count': len(raw) // stride,
            'file_size': len(raw),
            'pool_base':       f'0x{pool_base:08x}',
            'range_base':      f'0x{r["base"]:08x}',
            'range_size':      r['size'],
            'range_file':      r['file'],
            'offset_in_range': base_off,
            'pool_size_bytes': stride * (len(raw) // stride),
            'verified':        True,
            'verified_probes': matched,
        }

    Path(args.out).write_text(json.dumps(pool_map, indent=2))
    print(f'\nwrote {args.out}')


if __name__ == '__main__':
    main()
