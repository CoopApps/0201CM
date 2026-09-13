# CRITICAL CORRECTION: authoritative binary is cm0102_GDI.exe

Date: 2026-09-13. Per user directive: **cm0102-gdi.exe is the specification.**

Prior turns' byte-exact ports were derived from `cm0102.exe` (DirectDraw
build). This report catalogues what transfers, what does not, and the
recovered GDI addresses.

## Files on disk

- Spec: `D:/cm0102/cm0102_GDI.exe` (7,110,656 bytes, dated 2001-09-24).
- Prior (non-spec): `D:/cm0102/cm0102.exe` — same size, same date, DIFFERENT bytes at every VA I've verified.

## Which of the prior byte-exact ports transfer directly?

Verified experimentally 2026-09-13 by direct-calling both binaries'
schedule-getters and comparing SHA256 of the returned 2990-byte
buffer:

```
DD  buffer SHA256: 682a5ea610672a235c4f0a6205103af4346b3e4e66f6fb5f1008fdea0157c586
GDI buffer SHA256: 682a5ea610672a235c4f0a6205103af4346b3e4e66f6fb5f1008fdea0157c586
```

**Identical.** This proves:

- `pack_date` (FUN_00533b50 in DD = 0x00533d80 in GDI) — byte-identical code. Port valid.
- `apply_flag_snap` (FUN_00533eb0 in DD = ??? in GDI) — port output is valid (buffer matches). Static code equivalence still to confirm.
- `write_round_record` (FUN_0066f3b0 in DD = 0x0066ef70 in GDI) — byte-identical code. Port valid.
- `write_slot` (FUN_0066f410 in DD = 0x0066efd0 in GDI) — byte-identical code. Port valid.
- `ENG_SECOND_2001_TEMPLATE` — the 46-tuple call sequence is identical between builds.
- `build_eng_second_schedule` — output identical.

## Which prior byte-exact analyses do NOT transfer?

Static disassembly at 0x0055f240 and 0x00668450 shows the code
diverges from the DirectDraw equivalents:

- **eng_second_ctor**:
  - DD `0x0055f040` vs GDI `0x0055f240` (size 0x21c).
  - GDI uses vtable `0x00957dd8` (DD: `0x00957dc8`).
  - GDI calls `0x0066c9a0` early — a helper DirectDraw does NOT call.
  - Different SEH landing pads, different allocator addresses.
- **round-robin driver**:
  - DD `0x00668890` vs GDI `0x00668450` (size 0x920, DD size ~0x910).
  - Different first instructions (though both extract `n_clubs` from comp+0x3e and compute `n_even`).
  - Different SEH landing pad.
  - Calls `0x00533f20` (SEH init helper) — DD called `0x00533cf0`.
  - Calls `operator_new` at `0x009335cb` — DD at `0x008fc660`. Delta suggests the CRT is placed differently.

Every byte-exact claim about FUN_00668890 in
`FUN_00668890_DECODE.md` and `TEAM_PAIRING_REPORT.md` must be
re-derived against `0x00668450` in the GDI binary.

## Recovered GDI addresses

| Purpose | DirectDraw VA | GDI VA | Delta | Note |
|---------|--------------|--------|-------|------|
| pack_date | 0x00533b50 | 0x00533d80 | +0x230 | code identical |
| flag-snap (FUN_00533eb0) | 0x00533eb0 | (finding) | — | called from pack_date tail |
| eng_second_ctor | 0x0055f040 | 0x0055f240 | +0x200 | code DIFFERS |
| eng_second dtor | — | 0x0055f480 | — | new finding |
| schedule-getter | 0x0055f340 | 0x0055f540 | +0x200 | code identical (buffer matches) |
| schedule-installer | 0x00560320 | 0x00560520 | +0x200 | first 48 B identical |
| roster populator | 0x005601d0 | 0x005603d0 | +0x200 | first 24 B match |
| round-robin driver | 0x00668890 | 0x00668450 | -0x440 | code DIFFERS |
| matrix base seeder | 0x00669780 | 0x00669340 | -0x440 | code identical |
| matrix perturb | 0x0066bd40 | 0x0066b900 | -0x440 | first 32 B match |
| round writer | 0x0066f3b0 | 0x0066ef70 | -0x440 | identical |
| slot writer | 0x0066f410 | 0x0066efd0 | -0x440 | identical |
| walker (FUN_0066f280) | 0x0066f280 | (finding) | — | under investigation |
| venue writer (FUN_00845380) | 0x00845380 | (finding) | — | pending |
| replay/reset (FUN_0066a910) | 0x0066a910 | (finding) | — | pending |
| TFixList insert (FUN_00594d00) | 0x00594d00 | (finding) | — | pending |

The delta is **not constant** — +0x200 for the eng-league cluster
(0x0055xxxx), -0x440 for the fixture-generation cluster (0x0066xxxx).
This indicates the GDI build has slightly more code in one region and
less in another; likely additional GDI-related init code shifts later
sections backward.

## GDI eng_second_ctor prologue (0x0055f240..0x0055f2c9)

```
0x0055f240  push   -1                            SEH cookie
0x0055f242  push   0x946cb9                      SEH handler
0x0055f247  mov    eax, dword ptr fs:[0]
0x0055f24d  push   eax
0x0055f24e  mov    dword ptr fs:[0], esp
0x0055f255  sub    esp, 0x208                    frame
0x0055f25b  push   ebx / esi / edi
0x0055f25d  mov    esi, ecx                      this = ecx
0x0055f263  call   0x00667090                    base ctor
0x0055f268  mov    ecx, [esp+0x224]              param_3
0x0055f26f  mov    ax,  [esp+0x220]              param_2 (year)
0x0055f279  mov    [esi+4], ecx                  save param_3 at this+4
0x0055f27c  push   1
0x0055f27e  mov    ecx, esi
0x0055f287  mov    dword ptr [esi], 0x957dd8     vtable install
0x0055f28d  mov    word ptr [esi+0x40], ax       year at this+0x40
0x0055f291  mov    byte ptr [esi+0x50], 9        comp_id at this+0x50 = 9 (Div-2)
0x0055f295  call   0x0066c9a0                    ← extra helper (DD does NOT call this here)
0x0055f29a  test   eax, eax
0x0055f29c  jne    0x0055f441                    early success on non-zero
0x0055f2a5  mov    byte ptr [esi+0x44], 0xff
0x0055f2aa  mov    dword ptr [esi+0x30], -1
0x0055f2ad  mov    dword ptr [esi+0x2c], 1
0x0055f2b4  call   0x00933720                    malloc(4)
0x0055f2be  mov    [esi+0xc], eax                store at this+0xc
0x0055f2c1  call   0x00560520                    schedule-installer
```

Note the byte offsets `+0x50` (comp_id), `+0x40` (year), `+0x44`,
`+0x30`, `+0x2c`, `+0xc` match the DirectDraw equivalents. The comp
struct layout is preserved between builds.

The extra call at `0x0055f295` to `0x0066c9a0` is a new helper —
its role must be identified.

## Full byte-exact schedule-getter body (0x0055f540)

Static disassembly of the first 40 instructions shows a pattern
IDENTICAL to DirectDraw's 0x0055f340: read discriminant byte from
`[esp+4]`, compare to 0xFF, if non-0xFF jump to alt-mode block;
otherwise write `0x2e` (=46) to the round-count out ptr, call
`malloc(0xbae)`, then the strict "call round-writer; call
slot-writer" pattern for every round.

Runtime confirmation: SHA256 of the 2990-byte buffer produced by
this getter is byte-identical to the DirectDraw capture.

The getter body is NOT in `functions.json` (Ghidra failed to
identify it as a function, likely because of internal branches
confusing its flow analysis). It sits in the gap from 0x0055f540 to
just before 0x005603d0.

## What's next (unchanged from prior report)

Byte-exact reconstruction of GDI-specific functions is now the
priority:

1. **FUN_0055f240** — eng_second_ctor (agent running)
2. **FUN_00668450** — round-robin driver (agent running)
3. **GDI walker** — locate + decode (agent running)
4. **GDI flag-snap** — locate + confirm (agent running)
5. **GDI FUN_00594d00 equivalent** — TFixList inserter — locate + decode
6. **GDI FUN_00845380 equivalent** — venue writer — locate + decode
7. **GDI FUN_0066a910 equivalent** — replay/reset — locate + decode

Runtime plan:
- Runtime capture of the FULL constructor path (0x0055f240 → schedule-installer → roster → round-robin) with GDI addresses.
- Direct-calling the ctor requires the same care as before (global comp pool side effect at 0x00667090 base ctor).
- Preferred: attach hooks, then drive UI via keyboard automation.

## Provenance

| Claim | Source |
|-------|--------|
| GDI vs DirectDraw are different binaries | `compare_exes.py` output + this run |
| Schedule buffer byte-identical | SHA256 match this run |
| eng_second_ctor differs in code | disassembly of 0x0055f240 |
| Round-robin driver differs in code | disassembly of 0x00668450 |
| Vtable at 0x957dd8 in GDI | disasm `0x0055f287: mov [esi], 0x957dd8` |

## Impact on the Rust port

`crates/cm-domain/src/exe_date.rs`:
- Module header updated to name cm0102_GDI.exe as authoritative.
- Test `full_buffer_matches_runtime_capture_gdi` now compares against
  the GDI capture as authoritative (13 tests total, all passing).
- Kept `full_buffer_matches_runtime_capture_dd_corroborates` to
  document that the two builds produce identical output at this
  function.
- No functional code changed.

`generate_double_round_robin` remains **TEMPORARY STUB — KNOWN INCORRECT**.

Prior byte-exact reconstruction of DirectDraw's FUN_00668890 in
`FUN_00668890_DECODE.md` should be considered **provisional**
until re-verified against GDI's 0x00668450. Structural claims
(comp field offsets, adjacency matrix, walker call, H/A flip
mechanism, fixture record layout) will likely transfer since the
comp struct offsets and helper function contracts are preserved;
individual instruction sequences may differ.
