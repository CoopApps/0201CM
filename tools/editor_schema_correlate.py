"""Offline correlator — read reports/editor_capture/events.jsonl and derive a
per-record field-offset schema by searching each ReadFile buffer for the
displayed values that followed it.

Algorithm:
  For each 'text' event (a value shown in a control):
    - the LAST 'read' event tells us the current record's raw bytes
    - if the displayed text parses as an integer, search the buffer for that
      value as u8/u16/u32/i8/i16/i32 little-endian
    - record all offsets where it matches
    - if the text is a short ASCII string, search for it verbatim
  After processing all events, offsets that appear consistently for a given
  control-class-of-widget cluster to a stable field-offset mapping.

Output: reports/editor_schema_offsets.json
  {file_stem: [{offset, width, endian, control_class, text_sample, hits}]}
"""
import json, re, struct, sys
from pathlib import Path
from collections import Counter, defaultdict

REPO = Path(__file__).resolve().parents[1]
LOG = REPO / "reports/editor_capture/events.jsonl"
OUT = REPO / "reports/editor_schema_offsets.json"

INT_RE = re.compile(r"^-?\d+$")

def widths_and_endians(v):
    """Yield (width_bytes, signed, endian) candidates that COULD represent v."""
    for signed in (False, True):
        for width in (1, 2, 4):
            lo, hi = (0, 2**(8*width)-1) if not signed else (-(2**(8*width-1)), 2**(8*width-1)-1)
            if lo <= v <= hi:
                yield (width, signed, 'le')

def search_int(buf: bytes, v: int):
    """Return list of (offset, width, signed) where v matches in buf, LE."""
    hits = []
    for width, signed, _ in widths_and_endians(v):
        fmt = {(1,False):'B',(1,True):'b',(2,False):'<H',(2,True):'<h',
               (4,False):'<I',(4,True):'<i'}[(width, signed)]
        try:
            raw = struct.pack(fmt, v)
        except struct.error:
            continue
        i = 0
        while True:
            j = buf.find(raw, i)
            if j < 0: break
            hits.append((j, width, signed))
            i = j + 1
    return hits

def search_str(buf: bytes, s: str):
    if len(s) < 2 or len(s) > 40: return []
    hits = []
    try:
        needle = s.encode('latin-1')
    except UnicodeEncodeError:
        return hits
    i = 0
    while True:
        j = buf.find(needle, i)
        if j < 0: break
        hits.append((j, len(needle), 'str'))
        i = j + 1
    return hits

def main():
    if not LOG.exists():
        sys.exit(f"no capture at {LOG} — run editor_field_capture.py first")

    # State: last read buffer per file (approximation — matches display flow
    # of "load one record → show it in the form → user pages next").
    last_buf_for = {}
    file_events = defaultdict(list)  # file → list of (offset, width, text, cls, hwnd)
    text_since_last_read = 0

    hits_per_file_offset = defaultdict(Counter)  # (file, offset, width, signed) → count of matches
    values_at_offset = defaultdict(list)         # same key → list of sample texts

    with open(LOG, encoding='utf-8') as fh:
        for line in fh:
            try:
                p = json.loads(line)
            except json.JSONDecodeError:
                continue
            t = p.get('t')
            if t == 'read':
                fname = Path(p['file']).name.lower()
                # Only .dat files
                if not fname.endswith('.dat'): continue
                try:
                    buf = bytes.fromhex(p['hex'])
                except ValueError:
                    continue
                last_buf_for[fname] = (buf, p['off'])
                text_since_last_read = 0
            elif t == 'text':
                # Attribute to the most recently seen buffer of ANY file — the
                # correlator is imperfect. A more sophisticated version would
                # correlate by which .dat is "active" per record-type panel.
                if not last_buf_for: continue
                s = p.get('s', '').strip()
                if not s: continue
                # Skip menu/label/window titles (long strings, spaces, etc)
                if len(s) > 40: continue
                for fname, (buf, off) in last_buf_for.items():
                    if INT_RE.match(s):
                        v = int(s)
                        for (o, w, signed) in search_int(buf, v):
                            key = (fname, o, w, signed)
                            hits_per_file_offset[key][s] += 1
                            if len(values_at_offset[key]) < 5:
                                values_at_offset[key].append(s)
                    for (o, w, _) in search_str(buf, s):
                        key = (fname, o, w, 'str')
                        hits_per_file_offset[key][s] += 1
                        if len(values_at_offset[key]) < 5:
                            values_at_offset[key].append(s)
                text_since_last_read += 1

    # Rank: an offset that hit MULTIPLE distinct values (a real field) beats
    # one that only hit one value (probably coincidence).
    out = defaultdict(list)
    for key, counter in hits_per_file_offset.items():
        fname, off, width, signed = key
        distinct = len(counter)
        total = sum(counter.values())
        if distinct < 2 and total < 3: continue
        out[fname].append({
            'offset': f'0x{off:03x}', 'width': width, 'signed': signed if isinstance(signed, bool) else None,
            'kind': 'str' if signed == 'str' else 'int',
            'distinct_values': distinct, 'total_hits': total,
            'sample_values': values_at_offset[key][:5],
        })
    # Sort each file's entries by offset.
    for fname in out:
        out[fname].sort(key=lambda e: int(e['offset'], 16))

    OUT.write_text(json.dumps(out, indent=2), encoding='utf-8')
    print(f'[correlate] wrote {OUT}')
    for fname, entries in sorted(out.items()):
        print(f'  {fname:<20} {len(entries)} candidate field offsets')

if __name__ == '__main__':
    main()
