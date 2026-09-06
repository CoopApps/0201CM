#!/usr/bin/env python3
"""Find every editor pool base + record stride from a memory dump.

Approach: for each shipped Data/*.dat file, take the first record's raw
bytes as a signature and search every dumped RW range for it. When we
find a hit, verify by checking that (hit + record_size) starts the
second record's bytes. That pins pool_base + stride + record_count.

Result: a JSON manifest that the diff harness can consume:

    {
      "clubs":   {"pool_base": "0x08320000", "offset_in_range": 0,
                  "record_size": 581, "record_count": 10580,
                  "range_size": 1048576, "range_file": "range_0x8320000.bin"},
      "nations": ...,
      ...
    }

Which .dat files ship (30-ish):
    club.dat, nation.dat, staff.dat (3 sections), player.dat, comp.dat,
    club_comp.dat, colour.dat, continent.dat, city.dat, stadium.dat, ...

Usage:
    py find_pools.py                        # match every shipped .dat
    py find_pools.py --dat club.dat         # single file
    py find_pools.py --data D:\\cm0102\\Data # override Data dir
"""
import argparse, json, sys
from pathlib import Path

HERE = Path(__file__).parent
DUMP_DIR = HERE / 'editor_dump'


def load_ranges(dump_dir):
    """Return a list of (base_int, size, bytes) for each dumped range."""
    idx = json.load((dump_dir / 'index.json').open())
    ranges = []
    for r in idx['ranges']:
        data = (dump_dir / r['file']).read_bytes()
        ranges.append({
            'base': int(r['base'], 16),
            'size': r['size'],
            'file': r['file'],
            'data': data,
        })
    return ranges


def find_all(ranges, needle):
    """Yield (range, offset_in_range) for every occurrence of needle."""
    hits = []
    for r in ranges:
        d = r['data']
        i = 0
        while True:
            j = d.find(needle, i)
            if j == -1: break
            hits.append((r, j))
            i = j + 1
    return hits


# Known .dat record sizes from project memory / prior decode. Anything
# not in here gets stride-auto-probed by header+divisibility below.
KNOWN_SIZES = {
    'club.dat':                 581,   # verified port
    'nat_club.dat':             581,   # same record shape as club.dat
    'nation.dat':               290,
    'colour.dat':                58,   # 0x3a — VERIFIED
    'continent.dat':            198,   # 0xc6 — VERIFIED
    'club_comp.dat':            107,   # per index.dat load run
    'nation_comp.dat':          107,   # same shape as club_comp
    'staff_comp.dat':           107,   # same shape as club_comp
    'city.dat':                  76,
    # stadium.dat 553,722 leaves 26 leftover with stride 44 (12,584 recs);
    # keep 44 and rely on the leftover accounting to accept it.
    'stadium.dat':               44,
    'first_names.dat':           70,
    'second_names.dat':          76,
    'common_names.dat':          76,
    'officials.dat':             44,
    'staff_comp_history.dat':    58,
    'club_comp_history.dat':     44,
    'nation_comp_history.dat':   44,
    'staff_history.dat':         58,
    # staff.dat + index.dat are multi-section — file_size is NOT
    # count*stride, so we skip auto-probing and mark them below.
}

# Files with an on-disk header (record 0 does NOT start at file offset 0).
# For these we can't just take bytes[:32] as the first-record signature.
# staff.dat is 3 sections (157/68/70-byte) with a 12-byte prefix;
# index.dat is a directory of .dat pointers.
MULTI_SECTION = {'staff.dat', 'index.dat'}


def auto_probe_stride(raw):
    """Try to infer a plausible per-record stride from file size alone.

    Assumption: file_size == record_count * stride, with stride in a
    reasonable range and count > 1. Returns the SMALLEST plausible
    stride that (a) divides the file size and (b) leaves the FIRST
    32 bytes of record 1 (offset=stride) distinct from those of
    record 0. That distinctness check keeps us from matching absurdly
    small strides (e.g. stride=1 for an all-zero file)."""
    n = len(raw)
    best = None
    # Common CM record widths — try these first, then any divisor.
    candidates = [32, 40, 44, 50, 58, 68, 70, 76, 90, 107, 148, 157,
                  198, 200, 220, 290, 400, 500, 581, 640]
    seen = set()
    for s in candidates:
        if s in seen: continue
        seen.add(s)
        if s < 16 or s > 1024: continue
        if n % s != 0: continue
        count = n // s
        if count < 4 or count > 500_000: continue
        # Distinctness — record 0 vs record 1 vs record 2 must differ.
        r0 = raw[0:32]
        r1 = raw[s:s + 32]
        r2 = raw[2 * s:2 * s + 32]
        if r0 == r1 or r1 == r2: continue
        best = s
        break
    return best


def probe_stride(ranges, r_hit, off_hit, first_rec_bytes):
    """Given a match location, find the second occurrence in the same
    range and return that as the stride."""
    d = r_hit['data']
    # Look for the SAME first-record bytes further into the range —
    # doesn't work (records differ). Instead: find the shortest gap
    # after off_hit where a plausible new record starts. Cheap
    # heuristic — walk forward until we find a match to the third-
    # earliest full-shipped-file record signature. But we don't have
    # that here; return None and let the caller supply KNOWN_SIZES.
    return None


def analyze_dat(data_dir, dat_name, ranges, known_size=None):
    """Locate one .dat file's editor pool. Returns a dict or None."""
    path = data_dir / dat_name
    if not path.exists():
        return None
    raw = path.read_bytes()
    if len(raw) < 128:
        return None
    # Multi-section files (staff.dat, index.dat) don't have a
    # (count * stride) shape. Search for a longer signature deeper in
    # the file — bytes at file_size/2 should be unique enough that a
    # match locates the raw copy in memory. Skip stride verification.
    if dat_name in MULTI_SECTION:
        mid = len(raw) // 2 & ~0x1f
        sig_hd = raw[:32]
        sig_md = raw[mid:mid + 32]
        hits_hd = find_all(ranges, sig_hd)
        hits_md = find_all(ranges, sig_md)
        info = {'dat': dat_name, 'file_size': len(raw),
                'multi_section': True,
                'header_hits': len(hits_hd),
                'mid_hits': len(hits_md)}
        # Prefer a header hit whose paired mid signature is at exactly
        # the same offset within the same range — that's the raw copy.
        for r, off in hits_hd:
            expected = off + mid
            for r2, off2 in hits_md:
                if r2 is r and off2 == expected:
                    info.update({
                        'pool_base':       f'0x{r["base"] + off:08x}',
                        'range_base':      f'0x{r["base"]:08x}',
                        'range_size':      r['size'],
                        'range_file':      r['file'],
                        'offset_in_range': off,
                        'pool_size_bytes': len(raw),
                        'verified':        True,
                    })
                    return info
        return info

    stride = known_size or KNOWN_SIZES.get(dat_name)
    if not stride:
        stride = auto_probe_stride(raw)
    if not stride:
        return {'dat': dat_name, 'error': 'no plausible stride',
                'file_size': len(raw)}
    # Allow up to `stride` bytes of leading header + trailing padding —
    # some .dat files (stadium.dat, city.dat) carry a small header block
    # before record 0. Record `count = floor(file_size / stride)`; the
    # `leftover` figure tells the diff harness by how many bytes.
    count = len(raw) // stride
    leftover = len(raw) - count * stride
    if count < 4:
        return {'dat': dat_name, 'error': 'stride implies < 4 records',
                'file_size': len(raw), 'stride': stride}

    # Try signatures at multiple offsets — the very first bytes may be a
    # header (zeroed or file-name), or the first record may be all zeros
    # ("no player", "no colour"). Pick the DEEPEST signature that still
    # matches somewhere; that's most likely to be the raw file copy.
    def _try(off):
        sig = raw[off:off + 32]
        if sig == b'\x00' * 32:
            return None, []
        return sig, find_all(ranges, sig)

    probes = []
    for probe_off in (0, stride, stride * 2, stride * 10, stride * 100,
                      max(64, len(raw) // 2 & ~0x1f)):
        if probe_off + 32 > len(raw):
            continue
        sig, hits = _try(probe_off)
        if sig and hits:
            probes.append((probe_off, sig, hits))

    if not probes:
        return {'dat': dat_name, 'stride': stride, 'record_count': count,
                'file_size': len(raw), 'leftover': leftover, 'found': False}

    # Consistency check across all successful probes — if signatures at
    # offsets A and B land at addresses that differ by exactly (B - A),
    # that's the raw file copy. Score every (range, base_off) candidate
    # by how many probes it satisfies.
    scores = {}   # (range_idx, base_off) -> matched-probe count
    range_by_idx = {id(r): r for r, _ in probes[0][2]}
    for probe_off, sig, hits in probes:
        for r, off in hits:
            base_off = off - probe_off  # candidate file base
            if base_off < 0: continue
            key = (id(r), base_off)
            scores[key] = scores.get(key, 0) + 1
            range_by_idx[id(r)] = r

    # Best candidate = highest matched-probe count.
    best_key = max(scores, key=scores.get) if scores else None
    best_score = scores[best_key] if best_key else 0

    result = {
        'dat': dat_name,
        'stride': stride,
        'record_count': count,
        'file_size': len(raw),
        'leftover': leftover,
        'probes_tried': len(probes),
        'probes_matched': best_score,
    }
    if best_score >= 2:
        r = range_by_idx[best_key[0]]
        base_off = best_key[1]
        result.update({
            'pool_base':       f'0x{r["base"] + base_off:08x}',
            'range_base':      f'0x{r["base"]:08x}',
            'range_size':      r['size'],
            'range_file':      r['file'],
            'offset_in_range': base_off,
            'pool_size_bytes': stride * count,
            'verified':        True,
        })
    else:
        # Single-probe match — record it but don't call it verified.
        _, _, hits = probes[-1]
        r, off = hits[0]
        result.update({
            'pool_base_probable': f'0x{r["base"] + off - probes[-1][0]:08x}',
            'range_base':         f'0x{r["base"]:08x}',
            'range_size':         r['size'],
            'range_file':         r['file'],
            'offset_in_range':    off - probes[-1][0],
            'note': 'only 1 probe matched — could be a partial copy or '
                    'a different in-memory layout',
        })
    return result


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--data', default='D:/cm0102/Data')
    ap.add_argument('--dump', default=str(DUMP_DIR))
    ap.add_argument('--dat',  default=None,
                    help='Analyse only this dat (default: all shipped .dat)')
    ap.add_argument('--out',  default=str(HERE / 'pool_map.json'))
    args = ap.parse_args()

    data_dir = Path(args.data)
    dump_dir = Path(args.dump)
    if not (dump_dir / 'index.json').exists():
        sys.exit(f'no dump at {dump_dir} — run dump_memory.py first')

    print(f'[+] loading dump from {dump_dir}...')
    ranges = load_ranges(dump_dir)
    print(f'[+] {len(ranges)} ranges, '
          f'{sum(r["size"] for r in ranges):,} bytes')

    if args.dat:
        dats = [args.dat]
    else:
        dats = sorted(p.name for p in data_dir.glob('*.dat'))
    print(f'[+] {len(dats)} .dat files to check')

    pool_map = {}
    for name in dats:
        result = analyze_dat(data_dir, name, ranges)
        if not result:
            continue
        pool_map[name] = result
        found = result.get('pool_base') or result.get('pool_base_probable') or '(none)'
        stride = result.get('stride', '?')
        count = result.get('record_count', '?')
        probes = result.get('probes_matched', 0)
        left = result.get('leftover', '')
        left_s = f' left={left}' if left else ''
        v = '[v]' if result.get('verified') else '   '
        print(f'  {name:24}  stride={stride!s:>4}  count={count!s:>6}  '
              f'probes={probes}  {v} base={found}{left_s}')

    Path(args.out).write_text(json.dumps(pool_map, indent=2))
    print(f'[+] wrote {args.out}')


if __name__ == '__main__':
    main()
