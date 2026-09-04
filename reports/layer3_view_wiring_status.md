# Layer 3 View Wiring Status

Audit of `crates/cm-domain/src/screen_batch{3..27}.rs` — the live 25 batches per
`project-inventory-2026-09` memory (batches 28..35 are dead code and excluded).

For each screen, `to_widget_pool` wiring depends on:
* **easy** — LAB_ draw handler analyzed (`d:/cm0102-carve/analysis/screens/<addr>.json`)
  OR entry FUN_ itself is analyzed. Widget rects available from Ghidra push-arg analysis.
* **medium** — decompile of the entry FUN_ is present but LAB_ draw callback(s) are
  not yet analyzed. Wiring requires walking the LAB_ manually.
* **hard** — decompile file is missing (Ghidra could not find a discrete function
  boundary — e.g. inside a giant carver segment like News' 0x00770170).

## Summary

| Metric | Count |
|--------|------:|
| Batches (3..27) | 25 |
| Total View structs | 91 |
| Cited screen addresses | 57 |
| **easy** (LAB analyzed) | **36** |
| **medium** (LAB walk needed) | **20** |
| **hard** (no decompile) | **1** |

## Per-batch breakdown

### batch3

Views: `LatestScoresView, ManagerHistoryView, FifaRankingsView, UefaCoefficientsView`

Screens: 5 cited; 2 easy, 3 medium, 0 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x004a2190` | medium | no | 004a26d0, 004a2880 |
| `0x004a28c0` | medium | no | — |
| `0x006986a0` | medium | no | 00698e70 |
| `0x00700f20` | easy | no | 00701070, 00701170 |
| `0x00859250` | easy | no | 008596b0, 00876680 |

### batch4

Views: `SendAbuseView`

Screens: 4 cited; 2 easy, 2 medium, 0 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x00698140` | easy | no | 00698160, 00698480 |
| `0x00771810` | easy | no | 007719b0 |
| `0x007dfac0` | medium | no | — |
| `0x007e7790` | medium | no | — |

### batch5

Views: `AwardsView, CompetitionDashboardView`

Screens: 3 cited; 2 easy, 1 medium, 0 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x00415010` | easy | no | 004150e0, 00417390 |
| `0x00493e10` | easy | no | 00494640, 0049bf20, 0049df30 |
| `0x0074bf60` | medium | no | 0074caec |

### batch6

Views: `ContractView, ClubHistoryView, CompetitionSpecificView`

Screens: 3 cited; 3 easy, 0 medium, 0 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x0046ba80` | easy | no | 0046bdf0, 00470aa0 |
| `0x00476df0` | easy | no | 00476ef0 |
| `0x00494250` | easy | no | 00494640, 0049bf20, 0049df30 |

### batch7

Views: `MatchReportView`

Screens: 2 cited; 1 easy, 1 medium, 0 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x00701240` | easy | no | 007013d0, 00701c20, 00701fd0 |
| `0x007116b0` | medium | no | 00711833, 0094c46b |

### batch8

Views: `EntityListView, FindDialogView, WrittenHistoryView, SelectedLeaguesView, HallOfFameView`

Screens: 5 cited; 3 easy, 2 medium, 0 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x0058a550` | easy | no | 0058a740, 0058c930 |
| `0x0058cde0` | easy | no | 0058d000, 0058f420 |
| `0x005dc5e0` | medium | no | 005dcc50, 005dce60 |
| `0x008053d0` | easy | no | 00806640, 00810ce0, 00810e90 |
| `0x0080fac0` | medium | no | 008105f0 |

### batch9

Views: `(no View structs; dispatcher stubs only)`

Screens: 0 cited; 0 easy, 0 medium, 0 hard.

### batch10

Views: `Award417780View, Screen425e70View, Screen470bf0View, Screen472bf0View`

Screens: 5 cited; 2 easy, 3 medium, 0 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x00417780` | easy | no | 00417870 |
| `0x00425e70` | medium | no | — |
| `0x0046ad30` | easy | no | 0046af77, 0046b53d, 0046b56b, 0046b596, 0046b5bb, 0046b981, 00474760, 004751b0, 004757c0, 00944c0b |
| `0x00470bf0` | medium | no | — |
| `0x00472bf0` | medium | no | 00472c4b, 00472d8a, 004746b0 |

### batch11

Views: `ClubSubScreenAView, ClubCompSetupBView, ClubCompSetupCView, ContractOfferView, ContractOfferDirectView`

Screens: 0 cited; 0 easy, 0 medium, 0 hard.

### batch12

Views: `ClubSquadView, RelatedClubListView, ClubRivalsView, FilterDialogView`

Screens: 0 cited; 0 easy, 0 medium, 0 hard.

### batch13

Views: `CompLookupView, TwoKeyLookupView, ContractNegotiationView, ContractOfferView, ContractOfferComparatorView`

Screens: 5 cited; 4 easy, 0 medium, 1 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x004a17f0` | hard | no | — |
| `0x004e2b80` | easy | no | 004e2c70, 004e2f80, 0094612b |
| `0x004e3300` | easy | no | 004e3797, 004e38d0, 004e42b0 |
| `0x004e4580` | easy | no | 004e6680, 004e6b40, 00946148 |
| `0x004e4960` | easy | no | 004e4e04, 004e6680, 004e6b40, 00946168 |

### batch14

Views: `Screen4Eb240View, Screen4Ebf60View, Screen4Ec550View, Screen4Fd1b0View, Screen548170View`

Screens: 5 cited; 4 easy, 1 medium, 0 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x004eb240` | medium | no | 004eba20 |
| `0x004ebf60` | easy | no | 004ebfa0, 004ec240 |
| `0x004ec550` | easy | no | 004ec590, 004fd0d0 |
| `0x004fd1b0` | easy | no | 004fd1f0 |
| `0x00548170` | easy | no | 00548560, 005488f0, 00947086 |

### batch15

Views: `Screen574960View, Screen5792A0View, Screen57B9C0View, Screen57BB30View, Screen5928E0View`

Screens: 0 cited; 0 easy, 0 medium, 0 hard.

### batch16

Views: `HistoryRecordsView, ManageObj51fView, ManageObj588View, ManageObj67eView, MatchAreaScreenView`

Screens: 5 cited; 4 easy, 1 medium, 0 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x005dab70` | easy | no | 005dad10, 005db600, 005db7c0 |
| `0x00696fa0` | medium | no | 006972b0 |
| `0x00697390` | easy | no | 00697440 |
| `0x00697c30` | easy | no | 00697dc0, 0094bf7b |
| `0x006fd6c0` | easy | no | 006fd7b0 |

### batch17

Views: `MatchDetailView, FixtureNavView, TrivialRegistrationView, MediaArticleView, SmallFourSlotView`

Screens: 0 cited; 0 easy, 0 medium, 0 hard.

### batch18

Views: `TwoSelectorListView, LoaderGatedDialogView, DualSelectorScratchView, SeatScreenView, TwoPointerView`

Screens: 0 cited; 0 easy, 0 medium, 0 hard.

### batch19

Views: `PersonBannerView, SearchResultDetailView, SetupBootstrapView, FiveZeroSlotView`

Screens: 5 cited; 4 easy, 1 medium, 0 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x007cdb80` | easy | no | 007cdc70 |
| `0x007e7820` | medium | no | 007e7897 |
| `0x007faec0` | easy | no | 007fb050, 0095005e |
| `0x00803e00` | easy | no | 00804020, 00804340, 00808ae0, 008096e0 |
| `0x00808a70` | easy | no | 00808ae0, 008096e0 |

### batch20

Views: `HumanManagerSetupView, Screen0080BBD0View, Screen0080CC20View, Screen008109C0View, Screen00810CA0View`

Screens: 0 cited; 0 easy, 0 medium, 0 hard.

### batch21

Views: `SimpleScreen00877190View, TacticScreenView`

Screens: 0 cited; 0 easy, 0 medium, 0 hard.

### batch22

Views: `TrainingPanelView, PlayerProfileView`

Screens: 5 cited; 3 easy, 2 medium, 0 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x0088a850` | easy | no | 004544e0, 005a2af0, 005a2bb0, 006d1770, 0088aaa0, 0088aaa4, 0088aaf5, 0088ab18, 0088aba1, 0088acc0, 0088acc4, 0088aead, 0088aeb1, 0088b011, 0088b015, 0088b019, 0088b04f, 0088b06d, 0088b08a, 0088b0a8, 0088b0b0, 0088b19e, 0088b23c, 0088b2a9, 0088b2da, 0088b2f7, 0088b7d1, 0088b7d5, 0088b80b, 0088bb26, 0088bf2c, 0088bf30, 0088bf51, 0088c2d4, 0088c36b, 0088c378, 0088c9fd, 0088cadb, 0088d60f, 0088da1f, 0088da5d, 0088dc32, 0088dcf4, 0088dd07, 0088dd65, 0088def0, 00890e50, 00893500, 0095239e |
| `0x008a20a0` | easy | no | 008a2180, 008a5030, 008a63e0, 0095261b |
| `0x008a6420` | easy | no | 008a6580, 009526ae |
| `0x008d6310` | medium | no | — |
| `0x008d6510` | medium | no | — |

### batch23

Views: `Screen008dd030View, Screen008df0c0View, Screen008e0760View`

Screens: 0 cited; 0 easy, 0 medium, 0 hard.

### batch24

Views: `ContractOfferView, SmallThreeSlotView, WageOfferView, TwoSlot0710View, TwoSlot0B60View`

Screens: 5 cited; 2 easy, 3 medium, 0 hard.

| Addr | Class | Entry analyzed | LABs (analyzed?) |
|------|-------|:-:|------|
| `0x008dfb10` | medium | no | 00953a6b |
| `0x008dfc20` | easy | no | 008e01d0, 00953a8b |
| `0x008dfdf0` | easy | no | 008e9e60, 00953acb |
| `0x008e0710` | medium | no | — |
| `0x008e0b60` | medium | no | 008e0f30 |

### batch25

Views: `Screen008e26a0View, Screen008e26f0View, Screen008e3320View, Screen008e50e0View`

Screens: 0 cited; 0 easy, 0 medium, 0 hard.

### batch26

Views: `Screen008e6cc0View, Screen008e7270View, Screen008e78d0View, Screen008e8590View`

Screens: 0 cited; 0 easy, 0 medium, 0 hard.

### batch27

Views: `GenericEightSlotView, RecallLoanView, GuardedLoaderView`

Screens: 0 cited; 0 easy, 0 medium, 0 hard.

## Wiring plan

PHASE 1 (this commit): batch3 exemplar — 5 screens.
PHASE 2+: work through easy screens first (36 total across all batches),
then medium (20; ~1 hour each to walk the LAB), leaving the 1 hard for
live-capture bridging per News's pattern.
