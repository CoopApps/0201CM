#!/usr/bin/env python3
"""Byte-diff each pool's records: shipped Data/*.dat  vs  editor memory.

For every entry in pool_map.json we know:
  - the shipped .dat file (what the editor loaded from)
  - the range file it lives in (dumped memory)
  - a pool_base_probable within that range
  - the record stride

If the editor kept the raw file contiguously, both should match byte-for-
byte at stride intervals. Any mismatch is either:
  a) editor writes ephemeral state into pool records (dirty flags, etc.)
  b) memory has drifted since the file loaded (unlikely for a fresh open)
  c) OUR port is reading the field with the wrong type/offset — the class
     of bug we chase.

The report groups mismatches by field-offset within each record so a
u16-vs-u32 error surfaces as thousands of hits at one specific offset
rather than scattered noise.

Usage:
    py editor_diff.py                     # full report to stdout + diff.json
    py editor_diff.py --dat colour.dat    # single .dat
    py editor_diff.py --max-recs 100      # sample first 100 records per .dat
"""
import argparse, json, sys
from collections import Counter, defaultdict
from pathlib import Path

HERE = Path(__file__).parent


def load_map(pool_map_path):
    return json.loads(Path(pool_map_path).read_text())


def load_range(dump_dir, range_file):
    return (Path(dump_dir) / range_file).read_bytes()


def diff_dat(data_dir, dump_dir, entry, max_recs=None):
    """Return (record_diffs, offset_histogram) for one .dat."""
    if not entry.get('pool_base') and not entry.get('pool_base_probable'):
        return None
    stride = entry['stride']
    count = entry.get('record_count', 0)
    if max_recs:
        count = min(count, max_recs)
    file_bytes = (Path(data_dir) / entry['dat']).read_bytes()
    range_bytes = load_range(dump_dir, entry['range_file'])
    pool_off = entry['offset_in_range']

    record_diffs = []                  # per-record byte-mismatch count
    offset_hits = Counter()            # field-offset frequency across recs
    perfect = 0
    mismatched = 0
    total_byte_diffs = 0

    for i in range(count):
        file_start = i * stride
        pool_start = pool_off + i * stride
        f = file_bytes[file_start:file_start + stride]
        p = range_bytes[pool_start:pool_start + stride]
        if len(f) != stride or len(p) != stride:
            break                       # ran off the end of either buffer
        if f == p:
            perfect += 1
            continue
        mismatched += 1
        diffs = [k for k in range(stride) if f[k] != p[k]]
        for k in diffs:
            offset_hits[k] += 1
        total_byte_diffs += len(diffs)
        if len(record_diffs) < 10:
            record_diffs.append({'record': i, 'byte_diffs': len(diffs),
                                 'sample_offsets': diffs[:8]})

    return {
        'dat':      entry['dat'],
        'stride':   stride,
        'compared': perfect + mismatched,
        'perfect':  perfect,
        'mismatched': mismatched,
        'total_byte_diffs': total_byte_diffs,
        'top_offsets':  offset_hits.most_common(20),
        'sample_records': record_diffs,
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--data',      default='D:/cm0102/Data')
    ap.add_argument('--dump',      default=str(HERE / 'editor_dump'))
    ap.add_argument('--pool-map',  default=str(HERE / 'pool_map.json'))
    ap.add_argument('--dat',       default=None)
    ap.add_argument('--max-recs',  type=int, default=None,
                    help='sample the first N records per .dat instead of all')
    ap.add_argument('--out',       default=str(HERE / 'diff_report.json'))
    args = ap.parse_args()

    pmap = load_map(args.pool_map)
    dats = [args.dat] if args.dat else sorted(pmap.keys())

    report = {}
    for name in dats:
        entry = pmap.get(name)
        if not entry: continue
        if not (entry.get('pool_base') or entry.get('pool_base_probable')):
            report[name] = {'skipped': 'no pool base'}
            continue
        r = diff_dat(args.data, args.dump, entry, args.max_recs)
        if r is None: continue
        report[name] = r
        pct = 100.0 * r['perfect'] / max(1, r['compared'])
        summary = f'{name:26}  {r["compared"]:>6} records  {pct:6.2f}% byte-perfect'
        if r['mismatched']:
            top = r['top_offsets'][:3]
            summary += '  top-mismatch offsets: ' + \
                       ', '.join(f'+0x{o:x}({c})' for o,c in top)
        print(summary)

    Path(args.out).write_text(json.dumps(report, indent=2, default=list))
    print(f'\nwrote {args.out}')


if __name__ == '__main__':
    main()
