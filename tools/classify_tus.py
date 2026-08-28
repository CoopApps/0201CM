"""Classify every .cpp TU in cm0102.exe by inspecting its decompile.

Signals:
 - Filename tokens: `_prm`, `_first`, `_second`, `_cup`, `_super`, `_awards`,
   `_rules`, `_screens`, etc.
 - `_source` debug string in the ctor (`C:\\dev\\CM3 00-01\\comp\\leagues\\...`
   vs `\\comp\\eurocomp\\`, `\\comp\\intercomp\\`, `\\comp\\awards\\`,
   `\\comp\\rules\\`)
 - Ctor base-class call: `FUN_006674d0` = league/cup base, `FUN_00502320` =
   eurocomp base, `FUN_00490f00` = plain league (older)
 - Vtable pointer set in ctor (`PTR_FUN_00959...`)
 - Function count

Buckets:
  league       — plain domestic league (comp uses simple_league)
  cup          — plain domestic cup
  super        — super cup (league champ + cup winner tie)
  eurocomp     — European club competition (Champions Cup, UEFA Cup, etc.)
  intercomp    — International nation-cup (World Cup, Euros, Copa)
  awards       — country domestic awards (BLOCKED on ratings)
  rules        — country domestic rules (BLOCKED on finance)
  screen       — a screen callback / UI TU
  engine       — engine / infrastructure code (not a competition)
  data         — data model (persistence, records, tables)
  unknown      — needs manual look

Output: reports/tu_classification.md with a big table.
"""
import json
import os
import re
import sys
from collections import defaultdict

REPO = 'D:/cm0102-rs'
CARVE = 'D:/cm0102-carve'
ATTR = f'{CARVE}/ghidra_out/cm0102.exe/file_attribution.json'
DECOMP = f'{CARVE}/ghidra_out/cm0102.exe/decompiled'
LEDGER = f'{REPO}/reports/carve_rename_map.json'

def read_ctor(first_va: str) -> str:
    """Read the ctor decompile source, empty on missing."""
    addr = first_va.replace('0x', '').zfill(8)
    # Ghidra names files without leading zeros for the top-most, but generally lower-hex
    path = f'{DECOMP}/{addr[2:]}.c'  # e.g. 005eb410 -> 5eb410.c? No — files are 8-char.
    if os.path.exists(path):
        return open(path, errors='ignore').read()
    # Try the full 8-char lowercase form
    path = f'{DECOMP}/{addr}.c'
    if os.path.exists(path):
        return open(path, errors='ignore').read()
    return ''


def classify_tu(tu: str, funcs: list, ledgered: set) -> dict:
    signals = {'tu': tu, 'funcs': len(funcs), 'first_va': funcs[0]}

    # Filename tokens
    stem = tu[:-4] if tu.endswith('.cpp') else tu
    tokens = stem.split('_')
    signals['tokens'] = tokens

    # Ctor source
    src = read_ctor(funcs[0])
    signals['ctor_bytes'] = len(src)

    # Debug path in source
    m = re.search(r'C:.dev.CM3 00[_-]01.(?:cm3.code.)?([\w\\]+?)_\w+_009[a-f0-9]+', src)
    if m:
        signals['debug_path'] = m.group(1).replace('\\\\', '\\')
    else:
        # broader — just find the first debug path label
        m2 = re.search(r's_C__dev_CM3_00_01_([\w_]+)_009', src)
        signals['debug_path'] = m2.group(1) if m2 else ''

    # Base ctor called
    base_ctors = {
        '006674d0': 'league_base',
        '00502320': 'eurocomp_base',
        '00490f00': 'plain_base',
        '00533cf0': 'trivial_state_base',
        '008d5460': 'rules_base',
    }
    for va, label in base_ctors.items():
        if f'FUN_00{va}' in src:
            signals['base_ctor'] = label
            break

    # Vtable pointer
    m = re.search(r'PTR_FUN_(00[0-9a-f]+)', src)
    if m:
        signals['vtbl'] = m.group(1)

    # Classification
    bucket = 'unknown'

    # Filename patterns first
    if any(t == 'awards' for t in tokens) or stem.endswith('_awards'):
        bucket = 'awards'
    elif any(t == 'rules' for t in tokens) or stem.endswith('_rules'):
        bucket = 'rules'
    elif 'screens' in tokens or stem.endswith('_screen') or stem.endswith('_screens'):
        bucket = 'screen'
    elif stem.endswith('_super') or 'super' in tokens:
        bucket = 'super'
    elif stem.endswith('_cup') or 'cup' in tokens:
        bucket = 'cup'
    elif any(t in ('prm', 'first', 'second', 'third', 'conf', 'nat', 'reg',
                    'lge', 'liga', 'a_ligue', 'apertura', 'clausura', 'primera',
                    'segunda', 'sa', 'ser', 'div') for t in tokens[1:]):
        bucket = 'league'

    # Content-based refinements from debug path
    dp = signals.get('debug_path', '').lower()
    if 'eurocomp' in dp:
        bucket = 'eurocomp'
    elif 'intercomp' in dp:
        bucket = 'intercomp'
    elif 'comp\\awards' in dp or 'award' in dp:
        if bucket == 'unknown': bucket = 'awards'
    elif 'comp\\rules' in dp or 'rules' in dp:
        if bucket == 'unknown': bucket = 'rules'
    elif 'comp\\leagues' in dp or 'league' in dp:
        if bucket == 'unknown': bucket = 'league'

    # Base ctor overrides
    bc = signals.get('base_ctor')
    if bc == 'rules_base' and bucket in ('unknown', 'league'):
        bucket = 'rules'

    # Engine / data heuristics: filenames without country prefix and no comp shape
    if bucket == 'unknown':
        if any(kw in stem for kw in ('_screens', 'screen_')):
            bucket = 'screen'
        elif stem in ('game', 'game_config', 'cash', 'dispute', 'formation',
                      'friendly', 'fix_man', 'fifa_rankings', 'fog_of_war',
                      'gameplay_mutators', 'guio', 'gui_utils',
                      'hall_of_fame', 'history', 'host_country',
                      'human_manager', 'index', 'honours', 'main', 'startup'):
            bucket = 'engine'
        elif stem.endswith('_history') or stem.endswith('_records'):
            bucket = 'data'
        else:
            bucket = 'engine' if len(funcs) > 1 else 'unknown'

    signals['bucket'] = bucket
    signals['ledgered'] = tu in ledgered
    return signals


def main():
    attr = json.load(open(ATTR))
    ledger = json.load(open(LEDGER))
    ledgered = {k for k in ledger if not k.startswith('_') and k.endswith('.cpp')}

    tus = sorted(k for k in attr.keys() if k.endswith('.cpp'))

    rows = []
    for tu in tus:
        row = classify_tu(tu, attr[tu], ledgered)
        rows.append(row)

    # Sort by bucket then TU
    rows.sort(key=lambda r: (r['bucket'], r['tu']))

    # Group counts
    by_bucket = defaultdict(list)
    for r in rows:
        by_bucket[r['bucket']].append(r)

    # Write report
    out = ['# TU classification — every `.cpp` in cm0102.exe',
           '',
           f'Total .cpp TUs: **{len(tus)}**. Ledgered so far: **{sum(1 for r in rows if r["ledgered"])}**.',
           '',
           '## Bucket summary',
           '',
           '| Bucket | Count | Ledgered | Remaining |',
           '|---|---:|---:|---:|']
    for bucket in sorted(by_bucket):
        rs = by_bucket[bucket]
        total = len(rs)
        done = sum(1 for r in rs if r['ledgered'])
        out.append(f'| {bucket} | {total} | {done} | {total - done} |')
    out.append('')

    # Per-bucket lists
    for bucket in ('league', 'cup', 'super', 'eurocomp', 'intercomp',
                   'awards', 'rules', 'screen', 'engine', 'data', 'unknown'):
        rs = by_bucket.get(bucket, [])
        if not rs: continue
        out.append(f'## `{bucket}` — {len(rs)} TU(s)')
        out.append('')
        out.append('| TU | funcs | first VA | ctor | ledgered |')
        out.append('|---|---:|---|---|---:|')
        for r in rs:
            marker = '✓' if r['ledgered'] else ''
            ctor = r.get('base_ctor', '')
            out.append(f'| `{r["tu"]}` | {r["funcs"]} | {r["first_va"]} | {ctor} | {marker} |')
        out.append('')

    dest = f'{REPO}/reports/tu_classification.md'
    with open(dest, 'w', encoding='utf-8') as f:
        f.write('\n'.join(out))
    print(f'wrote {dest}')
    print(f'{len(tus)} TUs, {len(by_bucket)} buckets')
    for bucket, rs in sorted(by_bucket.items(), key=lambda kv: -len(kv[1])):
        done = sum(1 for r in rs if r['ledgered'])
        print(f'  {bucket}: {len(rs)} total, {done} ledgered')


if __name__ == '__main__':
    main()
