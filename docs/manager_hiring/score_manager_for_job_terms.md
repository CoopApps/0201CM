# FUN_00682420 score_manager_for_job — term map (authoritative, from disassembly)

Built from `cm0102.exe` disassembly (the executable is the authority), verified
through the differential harness (tools/exe_diff). The emulator now runs the full
function to a real RET (Emulator.setup_seh). Constants: docs/manager_hiring/rating_constants.md.

## Verified skeleton (pre-switch) — see cm_scoring
- `if *(person+0x69)==0 -> return -10000` (no standing record).
- resolve ref (param_3): 0 / +0x69==0 -> club+0xbf assistant -> else 0.
- base rating `local_94 = FUN_0052a330(person, ref_or_synth, 1)` — **BYTE_EXACT verified** (manager_club_repfit).
- standing bonus (status 5/0xc & club_rep>0x109a & standing+0xc>0xcb2): `+= standing+4*25; += standing+6*5`.
- incumbency (person+0x39==club): `+= ambition²` (active) or `+= 100`.
- affinity base re-weights (00531370/b0 -> max(1.1L,L+2000); 005313f0/420 -> 0.25L).

## CORRECTION to the resolved-constants model
The per-jobtype terms are NOT `L*(1 ± a·w)` as the constant-recovery pass assumed.
Disassembly of case 5/0xc (0x682b04+) shows they are attribute-**similarity** products:
```
term:  L = round( L * ( 1.0 - |R1[off] - R2[off]| / denom ) )     ; 1.0 = qword 0x00955890
```
(`movsx edx,[edi+off]; movsx eax,[ebp+off]; sub; fild; fabs; fdiv denom; fsubr 1.0;
 fmulp; call 0x009346d0`). This is why a zeroed standing block did NOT leave the
score at base (the second operand R2 was nonzero) — the earlier "skeleton = base"
hypothesis was disproven by the harness (base 3000 -> 1159).

## case 5/0xc term sequence (offsets/denoms read; R1/R2 identity being confirmed)
| VA | shape | off | denom const | notes |
|---|---|---|---|---|
| 0x682902 | `L *= 0.9 or 0.95` | — | 0x9569d8=0.9 / 0x9569d0=0.95 | age-gated (cVar6<0x26 branch) |
| 0x682927 | `L = round((R[+0x1b]*0.05 - 0.95) * L)` | +0x1b | 0.05@0x955888,0.95@0x9569d0 | age<0x26 & standing+0xc<0x109a |
| 0x68295f | additive `edi += round((min(Δ,0x32))*0.5*prefbyte*0.1)` | s+4,s+6,+0x1d | 0.5@0x956950,0.1@0x955880 | rep-gated |
| 0x682b04 | similarity | +0x1a | [esp+0x24] | R1=edi R2=ebp (person vs club/pref) |
| 0x682b31 | similarity | +0x1c | [esp+0x24] | |
| 0x682b5e | similarity | +0x1f | ×200@0x958638 then / | |
| 0x682b93 | similarity | +0x22 | [esp+0x1c] | |
| 0x682bc4.. | similarity run (edi=person+0x69 standing) | +0x11,… | — | continues to ~0x682d90 (10 conversions total) |
| 0x683823 | tail term | — | — | age/rep block |
| 0x6839c9 | loyalty re-weight (FUN_007aeb90) | — | — | if club+0x53!=0 |
| 0x683d2d | final __ftol -> return | — | — | return site 459 |

STATUS: structure decoded; R1/R2 record identities + each denom's provenance being
resolved from the surrounding int ops, then transcribed + harness-verified in
chunks. **00682420 = PORTED_PARTIAL** until every reachable branch is differential-clean.

## case 5/0xC — PROVEN register/record identities + sub-block decode (from disasm)
Registers (prologue 0x68243e-0x682458): **esi = person** (param_1), **ebx = club**
(param_2), **ebp = ref** (param_3, later reused). Accumulator L lives in [esp+0x14]
as an int, re-loaded/rounded (0x009346d0) after each float term.

Case 5/0xC executes three sub-blocks in order, then the shared tail:

### (a) pre-block  0x682902–0x6829bd  (base → L)
- 0x682902: age-gated fixed re-weight `L *= 0.9` (0x9569d8) or `0.95` (0x9569d0).
- 0x682918/0x682927: if age<0x26 & person.standing+0xc<0x109a: `L = round((standing+0x1b *0.05 - 0.95) * L)`.
- 0x68295f: rep-gated ADDITIVE `L += round(min(standing6-standing4,0x32) * 0.5(0x956950) via 0x935080 * prefbyte(ref.standing+0x1d or 0xa) * 0.1(0x955880))`.
- 0x6829bd: `L` committed. **This block transforms base even at zero attributes** (reads standing+0xc, rep, age) — the 3000→1159 seen in the harness.

### (b) similarity block  0x6829d0–0x682bc0  (ONLY if param_3/ref != 0)
R1 = person.standing (esi+0x69), R2 = ref.standing (edx=param_3, +0x69).
`L = round( L * (1.0(0x955890) - |R1[off]-R2[off]| / denom) )`, offsets seen:
+0xe (denom via 0x955878=0.025), +0x16, then +0x1a/+0x1c/+0x1f(×200 0x958638)/+0x22
(denoms in [esp+0x24]/[esp+0x1c]). ~6–7 terms. SKIPPED when ref==0.

### (c) attr block  0x682bc4–0x682caf  (always; edi = person.standing)
`L = round( L * (1.0 + person.standing[off] * w) )`:
| off | weight VA | value |
|---|---|---|
| +0x11 | 0x95af30 | 1/1200 |
| +0x10 | 0x95af30 | 1/1200 |
| +0x17 | 0x957038 | 1/500 |
| +0x18 | 0x957038 | 1/500 |
| +0x19 | 0x957038 | 1/500 |
| +0x1b | 0x957038 | 1/500 |
| +0x21 | rep-gated: club+0x80>0x1c52 → 0x95af28(1/125); >0x1676 → 0x956d60(1/250); else 0x95… | conditional |

### shared tail  0x683823–0x683d2d (see §return tail): age/rep term, loyalty
(FUN_007aeb90, only if club+0x53!=0), final `__ftol` → return (site 459).

VERIFICATION STATUS: identities + shapes + offsets/weights PROVEN from disassembly
and cross-checked against the resolved constants. Rust transcription of (a)+(b)+(c)+
tail and per-term differential freeze via the harness is the active next step;
00682420 remains PORTED_PARTIAL. The earlier single-shape assumption is superseded
by this three-sub-block decode.
