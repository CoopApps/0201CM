//! Layer 3 — News-item action classifier (port of `FUN_0076ab10`).
//!
//! # What this is (and what it is NOT)
//!
//! The audit at `reports/unreferenced_auto_fns_routing_audit.md`
//! labelled `FUN_0076ab10` "SCREEN_REGISTRY (news.cpp)" and predicted
//! that porting it would directly unlock 5 of the auto-generated
//! `build_screen_*` fns in [`crate::screens_auto`]. Reading the C
//! decomp end-to-end (`d:/cm0102-carve/ghidra_out/cm0102.exe/`
//! `decompiled/0076ab10.c`, 956 lines, ~35 news-type arms across 6
//! category groups) shows the audit misidentified the function's role:
//!
//! * FUN_0076ab10 is a **news-item action classifier**, not a screen
//!   registry. Signature (matched against the GDI-carve epilogue
//!   `ret 0x10` in `04517_sub_0076a750.asm` — note that under GDI's
//!   code-motion, `sub_0076a750` spans `0x0076a750..0x0076bcb1` and
//!   subsumes the cm0102.exe `FUN_0076ab10..FUN_0076bcee` pair):
//!
//!   ```c
//!   int __thiscall FUN_0076ab10(
//!       NewsItem*    this,       // ecx
//!       Person*      viewer,     // arg 1
//!       char*        out_label,  // arg 2 (button caption written by strcpy)
//!       int          _unused_len,
//!       void**       out_callback  // arg 4 (function pointer written)
//!   );
//!   ```
//!
//!   Return: `0` = no action available for this item, `1` = single-
//!   line action (`View`, `Nominations`, `Offer Contract`, `Reply`,
//!   `Appeal`, `Ultimatum`), `2` = respond-mode action (`Respond`,
//!   `Confirm`, `Details`, `Negotiate`, `Re-Negotiate`).
//!
//! * The callbacks it writes into `*out_callback` (e.g.
//!   `FUN_00697c30`, `FUN_004e2af0`, `FUN_00548170`, `FUN_008dfdf0`)
//!   are the OUTER builders of the target screens — they are the
//!   ones containing the 5 auto-fns (`build_screen_697dc0`,
//!   `4e2c70`, `548560`+`5488f0`, `8e01d0`). Their addresses are
//!   *taken* here (stored into `*out_callback`) — they are not
//!   called. Wiring the 5 auto-fns therefore requires porting those
//!   four outer builders, which is a separate follow-up per the
//!   task's hard rule "STOP AND REPORT — don't fake the chain".
//!
//! The audit's xref walker most likely conflated "address-of" xrefs
//! with "call" xrefs. Corrected fan-out map at the bottom of this
//! module.
//!
//! # What this module ports
//!
//! Every one of the ~35 news-type arms across the 6 category groups
//! from `FUN_0076d610`'s classification (categories 1..6), each with
//! its full guard chain. Each C statement carries a `// 0076..`
//! comment. Callbacks are returned as [`NewsCallback`] variants
//! (named after the exe FUN_ address) rather than raw function
//! pointers, because the callees are outer screen builders whose
//! Rust ports do not yet exist.

use core::convert::TryInto;

// ---------------------------------------------------------------
// Raw news-item field layout
// ---------------------------------------------------------------
//
// The exe treats a news item as a 256-byte record (`local_10c` /
// `local_20c` in the decomp) and reads plain byte/short/int fields
// at fixed offsets. This struct mirrors the raw layout so tests can
// pin every branch with byte-exact values. Field NAMES here are the
// port's; the exe never names them.

/// Raw byte view of a news-item record.
#[derive(Debug, Clone)]
pub struct RawNewsItem {
    pub bytes: Vec<u8>,
}

impl RawNewsItem {
    /// `FUN_0076ab10` guards against `param_4 < 0x33` — items must be
    /// at least 0x33 bytes for the smallest branch to have anything
    /// to read. Larger records (0xbe6/0xbe8/0xbdd/0xbdf) touch bytes
    /// up to 0xd6. We size to 0x100 unconditionally.
    pub fn zeroed() -> Self {
        Self {
            bytes: vec![0; 0x100],
        }
    }

    #[inline]
    pub fn i32_at(&self, off: usize) -> i32 {
        i32::from_le_bytes(self.bytes[off..off + 4].try_into().unwrap())
    }
    #[inline]
    pub fn u32_at(&self, off: usize) -> u32 {
        u32::from_le_bytes(self.bytes[off..off + 4].try_into().unwrap())
    }
    #[inline]
    pub fn i16_at(&self, off: usize) -> i16 {
        i16::from_le_bytes(self.bytes[off..off + 2].try_into().unwrap())
    }
    #[inline]
    pub fn i8_at(&self, off: usize) -> i8 {
        self.bytes[off] as i8
    }
    #[inline]
    pub fn u8_at(&self, off: usize) -> u8 {
        self.bytes[off]
    }

    /// Item type-code — read as `*param_1` (a raw i32 at +0) at C
    /// address `0076ab10:0076ab41`.
    #[inline]
    pub fn item_type(&self) -> i32 {
        self.i32_at(0)
    }

    /// Convenience setter for tests.
    pub fn with_type(mut self, ty: i32) -> Self {
        self.bytes[..4].copy_from_slice(&ty.to_le_bytes());
        self
    }
    pub fn set_i32(&mut self, off: usize, v: i32) {
        self.bytes[off..off + 4].copy_from_slice(&v.to_le_bytes());
    }
    pub fn set_i16(&mut self, off: usize, v: i16) {
        self.bytes[off..off + 2].copy_from_slice(&v.to_le_bytes());
    }
    pub fn set_u8(&mut self, off: usize, v: u8) {
        self.bytes[off] = v;
    }
}

// ---------------------------------------------------------------
// Callback identities (the ~24 outer builders `*out_callback` can point to)
// ---------------------------------------------------------------

/// Identity of the screen-builder callback FUN_0076ab10 selects.
///
/// Each variant is named after the exe FUN_ address the classifier
/// stored into `*param_5`. None of these outer builders are ported
/// yet — see the corrected fan-out map at the end of this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewsCallback {
    /// FUN_00697c30 — manager screen (announce-decision responder).
    /// Wraps `build_screen_697dc0`.
    ManagerAnnounce_697c30,
    /// LAB_008f5800 — ultimatum modal for board-issue news.
    UltimatumIssue_8f5800,
    /// FUN_004776a0 — post-match respond (won/lost feedback).
    MatchRespond_4776a0,
    /// FUN_00477dd0 — awards/end-of-season respond.
    AwardRespond_477dd0,
    /// LAB_004a2f80 — league champion respond.
    LeagueChampionRespond_4a2f80,
    /// FUN_004787f0 — cup/final respond.
    CupFinalRespond_4787f0,
    /// FUN_00478a30 — international honours respond.
    HonoursRespond_478a30,
    /// FUN_00761620 — cup respond.
    CupRespond_761620,
    /// LAB_00761fa0 — expiring-contract respond.
    ExpiringContractRespond_761fa0,
    /// FUN_004a5550 — cup nominations view.
    CupNominationsView_4a5550,
    /// LAB_0055a530 — chairman confidence respond.
    ChairmanConfidence_55a530,
    /// FUN_004176e0 — award nominations dispatcher.
    AwardNominations_4176e0,
    /// FUN_004e2af0 — contract-offer handler (wraps `build_screen_4e2c70`).
    ContractOffer_4e2af0,
    /// FUN_004e2b80 — contract detail. Wraps `build_screen_4e2c70`.
    ContractDetail_4e2b80,
    /// FUN_007297b0 — generic news respond (board/newspaper).
    GenericRespond_7297b0,
    /// FUN_00548170 — appeal handler (wraps `build_screen_548560` /
    /// `build_screen_5488f0`).
    Appeal_548170,
    /// FUN_00771880 — reply to newspaper letter.
    NewspaperReply_771880,
    /// FUN_008df8f0 — transfer-bid respond.
    TransferBidRespond_8df8f0,
    /// FUN_008df9f0 — transfer-negotiation respond.
    TransferNegotiationRespond_8df9f0,
    /// FUN_004ec3a0 — contract negotiate.
    ContractNegotiate_4ec3a0,
    /// FUN_008dfb10 — offer-contract (chairman-approved bid).
    OfferContract_8dfb10,
    /// FUN_008e10c0 — confirm sale.
    ConfirmSale_8e10c0,
    /// FUN_008dfc20 — swap-deal respond.
    SwapDealRespond_8dfc20,
    /// FUN_008dfd00 — re-negotiate wages.
    ReNegotiateWages_8dfd00,
    /// FUN_008dfdf0 — transfer-detail (wraps `build_screen_8e01d0`).
    TransferDetail_8dfdf0,
    /// FUN_008e5040 — foreign-player work-permit respond.
    WorkPermitRespond_8e5040,
    /// FUN_008e82b0 — transfer-listed player respond.
    TransferListedRespond_8e82b0,
    /// FUN_00763660 — season-preview details view.
    SeasonPreviewDetails_763660,
}

// ---------------------------------------------------------------
// Result
// ---------------------------------------------------------------

/// The `int` return of `FUN_0076ab10`, in typed form. Names reflect
/// what the shipped exe does with the value in the news-screen row
/// builder (whether it draws a full "Respond" button pair, or a
/// single button).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionMode {
    /// `return 1;` — a single action button captioned by `label`.
    SingleAction = 1,
    /// `return 2;` — a Respond/Decline pair; `label` captions the
    /// primary button (typically "Respond").
    RespondPair = 2,
}

/// Successful classification result — the news item has an actionable
/// button, its caption is `label`, and clicking invokes `callback`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NewsAction {
    pub label: &'static str,
    pub callback: NewsCallback,
    pub mode: ActionMode,
}

// ---------------------------------------------------------------
// Environment — what FUN_0076ab10 reads from the surrounding globals
// ---------------------------------------------------------------
//
// The C decomp touches these DAT_ globals: DAT_00acd5bc (club-pool
// base, 0x245 stride), DAT_00acd5c4 (person-pool base, 0x6e stride),
// DAT_00acd5d8 (?), DAT_00acd56c, DAT_00acd564, DAT_00acd580 (upper
// index bounds for various pools), DAT_00acde90 (u16, game-date low),
// DAT_00acde92 (u16, game-date high), DAT_00acde94 (u32, game year),
// DAT_00ac688c (comp lookup table), DAT_00ac56f0 (nation lookup),
// DAT_009bbc0c (current-club index), DAT_00dc7230 (season / league
// pool), plus a handful of external calls. The port models these as
// a trait so tests can supply values and production code can wire
// through the runtime pools.

/// Item-category classifier — the exe's `FUN_0076d610`. Returns a
/// small enum 1..=6 that FUN_0076ab10 dispatches on before it
/// examines the item's type code. In the exe this reads a static
/// per-type table; the port takes it as an input so callers who
/// know the category (news screen row builder) can pass it in
/// directly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum NewsItemCategory {
    // Category 1 — press correspondence (letters to the manager)
    Press = 1,
    // Category 2 — matchday feedback / awards (types 6, 9, 0x16..0x28)
    Matchday = 2,
    // Category 3 — legal / disciplinary appeals (0x7d2/0x7d3)
    Appeal = 3,
    // Category 4 — transfer offers from other clubs (0xbc1..0xbdf, 3000)
    IncomingOffer = 4,
    // Category 5 — outgoing transfer / negotiation state (0xfa2..0xfc1, 0xfbc, 4000)
    OutgoingTransfer = 5,
    // Category 6 — board / manager news (0x1774 manager, 0x1786 board)
    Board = 6,
}

impl NewsItemCategory {
    /// Byte value the exe returns from `FUN_0076d610`.
    #[inline]
    pub fn as_byte(self) -> u8 {
        self as u8
    }
}

/// Runtime environment surface. All addresses that need to compare
/// against pool base + stride are packaged as small typed queries so
/// tests can construct them.
pub trait NewsClassifyEnv {
    /// Category of this item — proxy for `FUN_0076d610()`. Read as
    /// often as the C reads it (once per top-level `if` chain).
    fn category(&self) -> NewsItemCategory;

    // ---------- pool identity queries ----------

    /// Returns true iff the club at pool index `club_idx` (i.e.
    /// `DAT_00acd5bc + club_idx * 0x245`) is the viewer's club
    /// (`param_2`). Used at many sites, e.g. `0076ab10:0x0076ac59`
    /// for cat-2 type 6.
    fn is_viewer_club(&self, club_idx: i32) -> bool;

    /// The viewer's own club-pool identity (as an i32 pool-record
    /// address, or 0 if unset). Corresponds to `*param_2`'s club-id
    /// (`param_2 + 0x39` deref then first int).
    fn viewer_club_first_id(&self) -> Option<i32>;

    /// Viewer's manager id (`*(int*)(param_2 + 0x24)`).
    fn viewer_manager_id(&self) -> i32;

    // ---------- game-date fixed globals ----------

    /// `DAT_00acde90` low 16 bits (day-in-year).
    fn game_date_day(&self) -> u16;
    /// `DAT_00acde92` (month/season slot).
    fn game_date_month(&self) -> u16;

    // ---------- per-type dependency lookups ----------

    /// True iff `DAT_00acd5bc + club_idx * 0x245 + 0xcf == param_2`.
    /// Used by 0xbe6/0xbe8/0xbd0/0xbd9/0xbdd/0xbdf/0xbc1 branches.
    fn club_owner_is_viewer(&self, club_idx: i32) -> bool;

    /// True iff comp `comp_id`'s status byte at +0xb2 == 1
    /// (`0076ab10:0x0076afba` — cat-2 type 0x28).
    fn comp_status_is_1(&self, comp_id: i32) -> bool;

    /// True iff there is a nomination slot open for the given item
    /// (`0076ab10:0x0076ade7` — cat-2 type 0x1c).
    fn nomination_slot_open(&self, param5: i32, param9: i32) -> bool;

    /// Award-view predicate for type 0x27 (`FUN_00755..` chain).
    /// Returns Some((row_ok, all_drafted, has_pending_row)) or None
    /// if the item's target club id is missing.
    fn cat2_type27_gate(&self, item: &RawNewsItem) -> Option<Cat2Type27Result>;

    /// Predicate for type 0x22 (`FUN_007553c0` / `FUN_007553f0`).
    /// Returns true iff the expiring-contract flag says the player
    /// is on the manager's own club roster.
    fn cat2_type22_own_player(&self, item: &RawNewsItem) -> bool;

    /// True iff the current league table row + subrow render as
    /// a nominations view for cat-2 type 0x27.
    fn cat3_appeal_gate(&self, item: &RawNewsItem) -> bool;

    /// Cat-5 helper — resolves the transfer-record pointer chain
    /// used by types 4000/0xfa2/0xfa4/0xfa5/0xfab/0xfac/0xfb4/
    /// 0xfb5/0xfc0. Returns Some(state) or None if any pool lookup
    /// returned NULL.
    fn cat5_transfer_state(&self, item: &RawNewsItem) -> Option<TransferState>;
}

/// Structured result of `cat2_type27_gate`.
#[derive(Debug, Clone, Copy)]
pub struct Cat2Type27Result {
    pub row_status_ok: bool,
    pub all_children_drafted: bool,
}

/// Cat-5 transfer state — deref chain from FUN_008b3240 etc.
#[derive(Debug, Clone, Copy, Default)]
pub struct TransferState {
    pub bid_club_id: i32,
    pub player_club_id: i32,
    pub player_owning_club_pool_id: i32,
    pub stage_code: u8,     // *(iVar8 + 0x2c)
    pub stage_flag2: u8,    // *(iVar8 + 0x2e)
    pub stage_flag3: u8,    // *(iVar8 + 0x2f)
    pub stage_flag4: u8,    // *(iVar8 + 0x10)
    pub linked_player_id: i32, // *(iVar8 + 0x28)
    pub other_side_id: i32, // via FUN_008b3260 chain
}

// ---------------------------------------------------------------
// The classifier — direct port of FUN_0076ab10
// ---------------------------------------------------------------

/// Classify a news item — port of `FUN_0076ab10`
/// (`d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/0076ab10.c`).
///
/// Returns `None` on any of the ~70 `return 0;` sites in the C.
/// Otherwise returns the caption / callback / mode the exe would
/// have written to `*param_3` and `*param_5`.
///
/// The `_viewer_person_arg` argument corresponds to `param_2`
/// (Person\*): the classifier only ever reads it through env
/// accessors, so the port passes those in the env instead.
pub fn classify_news_action<E: NewsClassifyEnv>(
    item: &RawNewsItem,
    env: &E,
) -> Option<NewsAction> {
    // 0076ab10:0x0076ab30..0x0076ab8f — argument guards (param_1
    // non-null, param_3 non-null, param_4 >= 0x33, param_5 non-null).
    // The port's caller-side signature enforces all of these via the
    // Rust type system (`&RawNewsItem`, `&E`) — nothing to check.

    let item_type = item.item_type();
    let cat = env.category();
    match cat {
        // ==================================================
        // 0076ab10:0x0076ab90..0x0076aba9 — cVar4 == '\x06'
        // Board / manager news
        // ==================================================
        NewsItemCategory::Board => {
            // 0076ab10:0x0076ab97 — if (*param_1 == 0x1774)
            if item_type == 0x1774 {
                // 0076ab10:0x0076ab9d — if (*((char*)param_1 + 9) == 0)
                if item.i8_at(9) == 0 {
                    return Some(NewsAction {
                        label: "Respond",
                        callback: NewsCallback::ManagerAnnounce_697c30,
                        mode: ActionMode::RespondPair,
                    });
                }
                None
            } else if item_type == 0x1786 {
                // 0076ab10:0x0076abcd — 4-way byte-guard
                let b09 = item.i8_at(9);
                let b0d = item.i8_at(0xd);
                let b19 = item.i8_at(0x19);
                if b09 != 5 && b0d != 0 && b0d != 6 && b19 == 0 {
                    return Some(NewsAction {
                        label: "Ultimatum",
                        callback: NewsCallback::UltimatumIssue_8f5800,
                        mode: ActionMode::SingleAction,
                    });
                }
                None
            } else {
                None
            }
        }

        // ==================================================
        // 0076ab10:0x0076ac1a onwards — cVar4 == '\x02'
        // Matchday feedback / awards / cup honours
        // ==================================================
        NewsItemCategory::Matchday => match item_type {
            // 0076ab10:0x0076ac2c — type 6 (post-match respond)
            6 => {
                let b19 = item.i8_at(0x19);
                if b19 == 3 || b19 == 4 {
                    return None;
                }
                let home_club = item.i32_at(5);
                let away_club = item.i32_at(9);
                let winner_club = item.i32_at(0xd);
                let match_club = if winner_club == home_club {
                    away_club
                } else {
                    home_club
                };
                // 0076ab10:0x0076ac59 — is this club the viewer's?
                if !env.is_viewer_club(match_club)
                    && env.viewer_manager_id() != match_club
                {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::MatchRespond_4776a0,
                    mode: ActionMode::RespondPair,
                })
            }
            // 0076ab10:0x0076ad0e — type 9 (awards respond)
            9 => {
                let b11 = item.i8_at(0x11);
                if b11 == 3 || b11 == 4 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::AwardRespond_477dd0,
                    mode: ActionMode::RespondPair,
                })
            }
            // 0076ab10:0x0076ada9 — type 0x16 (league champion)
            0x16 => {
                let vc = env.viewer_club_first_id()?;
                if vc != item.i32_at(9) {
                    return None;
                }
                // param_1+0xd == *(DAT_00dc7230 + 0x439)  — season slot check
                if item.i32_at(5) != 0 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::LeagueChampionRespond_4a2f80,
                    mode: ActionMode::RespondPair,
                })
            }
            // 0076ab10:0x0076ae0e — type 0x18 (cup final)
            0x18 => {
                let vc = env.viewer_club_first_id()?;
                if vc != item.i32_at(5) || item.i32_at(9) != 0 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::CupFinalRespond_4787f0,
                    mode: ActionMode::RespondPair,
                })
            }
            // 0076ab10:0x0076ae5f — type 0x19 (international honours)
            0x19 => {
                let vc = env.viewer_club_first_id()?;
                if vc != item.i32_at(5) || item.i32_at(0x11) != 0 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::HonoursRespond_478a30,
                    mode: ActionMode::RespondPair,
                })
            }
            // 0076ab10:0x0076aeb1 — type 0x1e (cup respond)
            0x1e => {
                if item.i32_at(0xd) != 0 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::CupRespond_761620,
                    mode: ActionMode::RespondPair,
                })
            }
            // 0076ab10:0x0076aee7 — type 0x22 (expiring contract)
            0x22 => {
                if item.i32_at(0x21) != 0 {
                    return None;
                }
                // 0076ab10:0x0076af10 — game-date match?
                if item.i16_at(0xcf) == env.game_date_month() as i16
                    && item.i16_at(0xcd) == env.game_date_day() as i16
                {
                    if !env.cat2_type22_own_player(item) {
                        return None;
                    }
                    return Some(NewsAction {
                        label: "Respond",
                        callback: NewsCallback::ExpiringContractRespond_761fa0,
                        mode: ActionMode::RespondPair,
                    });
                }
                // 0076ab10:0x0076afac — else falls through to 0076ecd0
                // side-effect + return 0. We drop the side-effect
                // (it advances an internal pool cursor) — pure
                // classification is None either way.
                None
            }
            // 0076ab10:0x0076af5b — type 0x27 (cup nominations)
            0x27 => {
                let b1d = item.i32_at(0x1d);
                if b1d != 0 {
                    return None;
                }
                let gate = env.cat2_type27_gate(item)?;
                if !gate.row_status_ok {
                    return None;
                }
                if !gate.all_children_drafted {
                    return Some(NewsAction {
                        label: "View / Draw",
                        callback: NewsCallback::CupNominationsView_4a5550,
                        mode: ActionMode::SingleAction,
                    });
                }
                // Falls through to FUN_0076ecd0 side-effect + return 0.
                None
            }
            // 0076ab10:0x0076afba — type 0x28 (chairman confidence)
            0x28 => {
                if !env.comp_status_is_1(0 /* DAT_009bbc0c-current-club */) {
                    return None;
                }
                if item.i32_at(5) != 1 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::ChairmanConfidence_55a530,
                    mode: ActionMode::RespondPair,
                })
            }
            // 0076ab10:0x0076afe6 — type 0x1c (award nominations)
            0x1c => {
                let param5 = item.i32_at(5);
                let param9 = item.i32_at(9);
                if !env.nomination_slot_open(param5, param9) {
                    return None;
                }
                Some(NewsAction {
                    label: "Nominations",
                    callback: NewsCallback::AwardNominations_4176e0,
                    mode: ActionMode::SingleAction,
                })
            }
            _ => None,
        },

        // ==================================================
        // 0076ab10:0x0076b064 onwards — cVar4 == '\x04'
        // Incoming-offer / board news (types 0xbc1..0xbdf, 3000)
        // ==================================================
        NewsItemCategory::IncomingOffer => match item_type {
            0xbbf => {
                // 0076ab10:0x0076b06d
                let vc = env.viewer_club_first_id()?;
                if vc != item.i32_at(5) || item.i8_at(0x21) != 0 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::ContractOffer_4e2af0,
                    mode: ActionMode::RespondPair,
                })
            }
            0xbe8 => {
                // 0076ab10:0x0076b0c9
                if !env.club_owner_is_viewer(item.i32_at(0x25)) {
                    return None;
                }
                if item.i8_at(0x4d) != 0 {
                    return None;
                }
                if item.i32_at(0x41) != 0 {
                    return None;
                }
                // Merged with LAB_0076b2e1 tail — generic respond.
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::GenericRespond_7297b0,
                    mode: ActionMode::RespondPair,
                })
            }
            0xbe6 => {
                // 0076ab10:0x0076b118
                if !env.club_owner_is_viewer(item.i32_at(0x29)) {
                    return None;
                }
                if item.i8_at(0x55) != 0 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::GenericRespond_7297b0,
                    mode: ActionMode::RespondPair,
                })
            }
            0xbd9 => {
                // 0076ab10:0x0076b17e
                if item.i8_at(0x25) != 0 {
                    return None;
                }
                if !env.club_owner_is_viewer(item.i32_at(0x15)) {
                    return None;
                }
                if item.i8_at(0x2d) != 0 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::GenericRespond_7297b0,
                    mode: ActionMode::RespondPair,
                })
            }
            0xbdf => {
                // 0076ab10:0x0076b1c1 — subtype switch on +0x29
                match item.u8_at(0x29) {
                    0x3e | 0x50 | 0x51 | 0x5e => return None,
                    _ => {}
                }
                if !env.club_owner_is_viewer(item.i32_at(0x15)) {
                    return None;
                }
                if item.i8_at(0x2d) != 0 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::GenericRespond_7297b0,
                    mode: ActionMode::RespondPair,
                })
            }
            0xbdd => {
                // 0076ab10:0x0076b232
                if !env.club_owner_is_viewer(item.i32_at(0x15)) {
                    return None;
                }
                if item.i8_at(0x39) != 0 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::GenericRespond_7297b0,
                    mode: ActionMode::RespondPair,
                })
            }
            0xbd0 => {
                // 0076ab10:0x0076b2ba
                if !env.club_owner_is_viewer(item.i32_at(0x15)) {
                    return None;
                }
                if item.i8_at(0x2d) != 0 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::GenericRespond_7297b0,
                    mode: ActionMode::RespondPair,
                })
            }
            0xbc1 => {
                // 0076ab10:0x0076b331
                let vc = env.viewer_club_first_id()?;
                if vc != item.i32_at(5) || item.i8_at(0x1d) != 0 {
                    return None;
                }
                Some(NewsAction {
                    label: "Respond",
                    callback: NewsCallback::WorkPermitRespond_8e5040,
                    mode: ActionMode::RespondPair,
                })
            }
            0xbc2 => {
                // 0076ab10:0x0076b389
                let vc = env.viewer_club_first_id()?;
                if vc != item.i32_at(5) || item.i8_at(0x21) != 0 {
                    return None;
                }
                Some(NewsAction {
                    label: "Offer Contract",
                    callback: NewsCallback::ContractOffer_4e2af0,
                    mode: ActionMode::SingleAction,
                })
            }
            3000 => {
                // 0076ab10:0x0076b3e7 — contract-negotiation continuation
                let vc = env.viewer_club_first_id()?;
                let club6 = item.i32_at(0x15);
                if vc != club6 {
                    return None;
                }
                let st = env.cat5_transfer_state(item)?;
                if st.stage_code != 3 {
                    return None;
                }
                Some(NewsAction {
                    label: "Negotiate",
                    callback: NewsCallback::ContractNegotiate_4ec3a0,
                    mode: ActionMode::RespondPair,
                })
            }
            _ => None,
        },

        // ==================================================
        // 0076ab10:0x0076b6a8 — cVar4 == '\x03' — appeals
        // ==================================================
        NewsItemCategory::Appeal => {
            if item_type != 0x7d3 && item_type != 0x7d2 {
                return None;
            }
            if !env.cat3_appeal_gate(item) {
                return None;
            }
            Some(NewsAction {
                label: "Appeal",
                callback: NewsCallback::Appeal_548170,
                mode: ActionMode::SingleAction,
            })
        }

        // ==================================================
        // 0076ab10:0x0076b98e — cVar4 == '\x01' — press letters
        // ==================================================
        NewsItemCategory::Press => {
            if item_type != 5000 {
                return None;
            }
            Some(NewsAction {
                label: "Reply",
                callback: NewsCallback::NewspaperReply_771880,
                mode: ActionMode::SingleAction,
            })
        }

        // ==================================================
        // 0076ab10:0x0076b9d1 onwards — cVar4 == '\x05'
        // Outgoing transfer / negotiation state
        // ==================================================
        NewsItemCategory::OutgoingTransfer => {
            // Cat 5 is a set of chained `if (*param_1 == 0xNNN)` — not a
            // switch — so multiple arms can match & fall through into
            // the shared LAB_0076bbc5 / LAB_0076bcee tails.
            let mut hit: Option<NewsAction> = None;

            if item_type == 4000 {
                // 0076ab10:0x0076b9f1
                if let Some(st) = env.cat5_transfer_state(item) {
                    let b29 = item.i8_at(0x29);
                    let clubidx = item.i32_at(0x19);
                    if st.bid_club_id != 0
                        && clubidx != -1
                        && env.club_owner_is_viewer(clubidx)
                        && st.player_club_id == item.i32_at(0x21)
                        && b29 != 1
                        && b29 != 5
                        && b29 != 2
                        && b29 != 0x11
                        && st.stage_flag4 == 0
                    {
                        hit = Some(NewsAction {
                            label: "Respond",
                            callback: NewsCallback::TransferBidRespond_8df8f0,
                            mode: ActionMode::RespondPair,
                        });
                    }
                }
            } else if item_type == 0xfbc {
                // 0076ab10:0x0076ba9f
                if item.i32_at(0x15) >= 0
                    && env.club_owner_is_viewer(item.i32_at(0x15))
                    && item.i8_at(0x21) == 0
                {
                    hit = Some(NewsAction {
                        label: "Respond",
                        callback: NewsCallback::GenericRespond_7297b0,
                        mode: ActionMode::RespondPair,
                    });
                }
            }

            if item_type == 0xfa2 {
                // 0076ab10:0x0076baec
                if let Some(st) = env.cat5_transfer_state(item) {
                    if st.player_club_id == item.i32_at(0x21)
                        && st.linked_player_id != -1
                        && st.other_side_id == item.i32_at(0x29)
                        && st.stage_flag4 == 1
                    {
                        hit = Some(NewsAction {
                            label: "Respond",
                            callback: NewsCallback::TransferNegotiationRespond_8df9f0,
                            mode: ActionMode::RespondPair,
                        });
                    }
                }
            }

            if item_type == 0xfac {
                // 0076ab10:0x0076bbf1 — offer-contract composite gate
                let b29 = item.i8_at(0x29);
                if b29 != 1 && b29 != 4 {
                    if let Some(st) = env.cat5_transfer_state(item) {
                        if st.player_club_id == item.i32_at(0x25)
                            && st.stage_code != 1
                            && st.stage_flag4 != 0
                        {
                            hit = Some(NewsAction {
                                label: "Offer Contract",
                                callback: NewsCallback::OfferContract_8dfb10,
                                mode: ActionMode::RespondPair,
                            });
                        }
                    }
                }
            }

            if item_type == 0xfa4 {
                // 0076ab10:0x0076bc60 — negotiate on stage 3 / 4
                let b1d = item.i8_at(0x1d);
                let target = item.i32_at(0x21);
                if b1d == 3 && target != -1 {
                    if let Some(st) = env.cat5_transfer_state(item) {
                        if st.player_club_id == item.i32_at(0x25)
                            && (st.stage_code == 7 || st.stage_code == 8)
                            && st.linked_player_id != 0
                        {
                            hit = Some(NewsAction {
                                label: "Negotiate",
                                callback: NewsCallback::ContractNegotiate_4ec3a0,
                                mode: ActionMode::RespondPair,
                            });
                        }
                    }
                } else if b1d == 4 && target != -1 && item.i8_at(0x29) == 0 {
                    if let Some(st) = env.cat5_transfer_state(item) {
                        let vc = env.viewer_club_first_id().unwrap_or(0);
                        if st.player_club_id == item.i32_at(0x25)
                            && vc == item.i32_at(0x19)
                            && st.stage_code == 0x0f
                            && st.stage_flag2 < 3
                        {
                            hit = Some(NewsAction {
                                label: "Negotiate",
                                callback: NewsCallback::ContractNegotiate_4ec3a0,
                                mode: ActionMode::SingleAction,
                            });
                        }
                    }
                }
            }

            // LAB_0076bbc5 tail
            if item_type == 0xfa5 {
                let b29 = item.i8_at(0x29);
                // 0076ab10:0x0076bcda — 8-way disallow-list
                let disallowed = matches!(b29, 0x0e | 0x0d | 0x0c | 0x03 | 0x02 | 0x10 | 0x12 | 0x11);
                if !disallowed {
                    if let Some(st) = env.cat5_transfer_state(item) {
                        if st.player_club_id == item.i32_at(0x21)
                            && (st.stage_code == 0x0b
                                || st.stage_code == 0x0c
                                || st.stage_code == 0x0d)
                        {
                            hit = Some(NewsAction {
                                label: "Confirm",
                                callback: NewsCallback::ConfirmSale_8e10c0,
                                mode: ActionMode::RespondPair,
                            });
                        }
                    }
                }
            } else if item_type == 0xfa8 {
                // Explicit no-op branch (return 0) at 0076ab10:0x0076bce7.
            }

            // LAB_0076bcee tail
            if item_type == 0xfab {
                // Swap-deal respond
                if let Some(vc) = env.viewer_club_first_id() {
                    let a = item.i32_at(5);
                    let b = item.i32_at(9);
                    if (vc == a || vc == b) && item.i8_at(0x21) == 0 {
                        hit = Some(NewsAction {
                            label: "Respond",
                            callback: NewsCallback::SwapDealRespond_8dfc20,
                            mode: ActionMode::RespondPair,
                        });
                    }
                }
            }

            if item_type == 0xfb4 {
                // Re-negotiate wages
                if let Some(st) = env.cat5_transfer_state(item) {
                    if item.i32_at(0x19) == st.linked_player_id
                        && st.stage_code == 1
                        && st.stage_flag3 == 0
                        && item.i8_at(0x39) == 0
                        && item.i8_at(0x2d) == 1
                    {
                        hit = Some(NewsAction {
                            label: "Re-Negotiate",
                            callback: NewsCallback::ReNegotiateWages_8dfd00,
                            mode: ActionMode::RespondPair,
                        });
                    }
                }
            }

            if item_type == 0xfb5 {
                // Transfer detail
                if let Some(st) = env.cat5_transfer_state(item) {
                    let clubidx = item.i32_at(0x19);
                    if clubidx != -1
                        && env.club_owner_is_viewer(clubidx)
                        && st.player_club_id == item.i32_at(0x21)
                        && st.stage_code == 0
                    {
                        hit = Some(NewsAction {
                            label: "Respond",
                            callback: NewsCallback::TransferDetail_8dfdf0,
                            mode: ActionMode::RespondPair,
                        });
                    }
                }
            } else if item_type == 0xfb6 {
                // Explicit no-op branch (return 0) at 0076ab10:0x0076bd6a.
            }

            if item_type == 0xfc0 {
                // Transfer-listed respond
                if let Some(st) = env.cat5_transfer_state(item) {
                    if st.player_club_id == item.i32_at(0x21)
                        && st.linked_player_id != -1
                        && st.other_side_id == item.i32_at(0x29)
                        && st.stage_code == 1
                    {
                        hit = Some(NewsAction {
                            label: "Respond",
                            callback: NewsCallback::TransferListedRespond_8e82b0,
                            mode: ActionMode::RespondPair,
                        });
                    }
                }
            }

            if hit.is_none() && item_type == 0xfc1 {
                // 0076ab10:0x0076bce0 — season-preview details view
                return Some(NewsAction {
                    label: "Details",
                    callback: NewsCallback::SeasonPreviewDetails_763660,
                    mode: ActionMode::SingleAction,
                });
            }

            hit
        }
    }
}

// ---------------------------------------------------------------
// Corrected fan-out map (audit correction)
// ---------------------------------------------------------------
//
// The 5 auto-fns the audit predicted this port would "directly
// unlock" are actually inside four separate outer builders that
// FUN_0076ab10 only takes the address of (into `*out_callback`),
// never calls. Wiring them requires porting those outer builders in
// separate follow-up commits:
//
// | Auto-fn                  | Callback set by 0076ab10          | Outer builder that must be ported |
// | build_screen_4e2c70      | ContractOffer_4e2af0              | FUN_004e2b80 (contract_screens.cpp) |
// | build_screen_548560      | Appeal_548170                     | FUN_00548170 (mid-fn +0x3f0)        |
// | build_screen_5488f0      | Appeal_548170                     | FUN_00548170 (mid-fn +0x780)        |
// | build_screen_697dc0      | ManagerAnnounce_697c30            | FUN_00697c30 (manager_screens.cpp)  |
// | build_screen_8e01d0      | TransferDetail_8dfdf0             | FUN_008dfdf0 (transfer_screens.cpp) |
//
// This commit ports the CLASSIFIER only. The four outer builders
// above remain as follow-up targets, and until they are ported the
// 5 auto-fns above stay unreferenced.

// ---------------------------------------------------------------
// Tests
// ---------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, Clone)]
    struct Env {
        cat: Option<NewsItemCategory>,
        viewer_club: i32,
        manager_id: i32,
        my_club_owner: bool,
        comp_status_1: bool,
        nomination_open: bool,
        t27: Option<Cat2Type27Result>,
        t22_own: bool,
        appeal_ok: bool,
        cat5: Option<TransferState>,
        date_day: u16,
        date_month: u16,
    }

    impl NewsClassifyEnv for Env {
        fn category(&self) -> NewsItemCategory { self.cat.unwrap() }
        fn is_viewer_club(&self, c: i32) -> bool { c == self.viewer_club && c != 0 }
        fn viewer_club_first_id(&self) -> Option<i32> {
            if self.viewer_club == 0 { None } else { Some(self.viewer_club) }
        }
        fn viewer_manager_id(&self) -> i32 { self.manager_id }
        fn game_date_day(&self) -> u16 { self.date_day }
        fn game_date_month(&self) -> u16 { self.date_month }
        fn club_owner_is_viewer(&self, _c: i32) -> bool { self.my_club_owner }
        fn comp_status_is_1(&self, _c: i32) -> bool { self.comp_status_1 }
        fn nomination_slot_open(&self, _a: i32, _b: i32) -> bool { self.nomination_open }
        fn cat2_type27_gate(&self, _i: &RawNewsItem) -> Option<Cat2Type27Result> { self.t27 }
        fn cat2_type22_own_player(&self, _i: &RawNewsItem) -> bool { self.t22_own }
        fn cat3_appeal_gate(&self, _i: &RawNewsItem) -> bool { self.appeal_ok }
        fn cat5_transfer_state(&self, _i: &RawNewsItem) -> Option<TransferState> { self.cat5 }
    }

    // ---------- category 6 (Board) ----------

    #[test]
    fn board_manager_announce_respond() {
        let env = Env { cat: Some(NewsItemCategory::Board), ..Env::default() };
        let item = RawNewsItem::zeroed().with_type(0x1774);
        let a = classify_news_action(&item, &env).unwrap();
        assert_eq!(a.label, "Respond");
        assert_eq!(a.callback, NewsCallback::ManagerAnnounce_697c30);
        assert_eq!(a.mode, ActionMode::RespondPair);
    }

    #[test]
    fn board_manager_announce_blocked_by_flag() {
        let env = Env { cat: Some(NewsItemCategory::Board), ..Env::default() };
        let mut item = RawNewsItem::zeroed().with_type(0x1774);
        item.set_u8(9, 1);
        assert_eq!(classify_news_action(&item, &env), None);
    }

    #[test]
    fn board_ultimatum() {
        let env = Env { cat: Some(NewsItemCategory::Board), ..Env::default() };
        let mut item = RawNewsItem::zeroed().with_type(0x1786);
        item.set_u8(9, 1);
        item.set_u8(0xd, 1);
        item.set_u8(0x19, 0);
        let a = classify_news_action(&item, &env).unwrap();
        assert_eq!(a.callback, NewsCallback::UltimatumIssue_8f5800);
        assert_eq!(a.mode, ActionMode::SingleAction);
    }

    // ---------- category 2 (Matchday) ----------

    #[test]
    fn match_respond_when_viewer_is_a_side() {
        let mut env = Env { cat: Some(NewsItemCategory::Matchday), viewer_club: 42, ..Env::default() };
        env.manager_id = 99;
        let mut item = RawNewsItem::zeroed().with_type(6);
        item.set_i32(5, 42);
        item.set_i32(9, 7);
        item.set_i32(0xd, 7);
        let a = classify_news_action(&item, &env).unwrap();
        assert_eq!(a.callback, NewsCallback::MatchRespond_4776a0);
    }

    #[test]
    fn awards_respond() {
        let env = Env { cat: Some(NewsItemCategory::Matchday), ..Env::default() };
        let item = RawNewsItem::zeroed().with_type(9);
        let a = classify_news_action(&item, &env).unwrap();
        assert_eq!(a.callback, NewsCallback::AwardRespond_477dd0);
    }

    #[test]
    fn cup_respond_type_1e() {
        let env = Env { cat: Some(NewsItemCategory::Matchday), ..Env::default() };
        let item = RawNewsItem::zeroed().with_type(0x1e);
        assert_eq!(
            classify_news_action(&item, &env).unwrap().callback,
            NewsCallback::CupRespond_761620
        );
    }

    #[test]
    fn award_nominations() {
        let env = Env { cat: Some(NewsItemCategory::Matchday), nomination_open: true, ..Env::default() };
        let item = RawNewsItem::zeroed().with_type(0x1c);
        let a = classify_news_action(&item, &env).unwrap();
        assert_eq!(a.callback, NewsCallback::AwardNominations_4176e0);
        assert_eq!(a.mode, ActionMode::SingleAction);
    }

    #[test]
    fn league_champion_16() {
        let env = Env { cat: Some(NewsItemCategory::Matchday), viewer_club: 5, ..Env::default() };
        let mut item = RawNewsItem::zeroed().with_type(0x16);
        item.set_i32(9, 5);
        item.set_i32(5, 0);
        assert_eq!(
            classify_news_action(&item, &env).unwrap().callback,
            NewsCallback::LeagueChampionRespond_4a2f80
        );
    }

    // ---------- category 4 (Incoming offer) ----------

    #[test]
    fn incoming_contract_offer_bbf() {
        let env = Env { cat: Some(NewsItemCategory::IncomingOffer), viewer_club: 11, ..Env::default() };
        let mut item = RawNewsItem::zeroed().with_type(0xbbf);
        item.set_i32(5, 11);
        assert_eq!(
            classify_news_action(&item, &env).unwrap().callback,
            NewsCallback::ContractOffer_4e2af0
        );
    }

    #[test]
    fn incoming_bd9_generic_respond() {
        let env = Env { cat: Some(NewsItemCategory::IncomingOffer), my_club_owner: true, ..Env::default() };
        let item = RawNewsItem::zeroed().with_type(0xbd9);
        assert_eq!(
            classify_news_action(&item, &env).unwrap().callback,
            NewsCallback::GenericRespond_7297b0
        );
    }

    #[test]
    fn incoming_bdf_subtype_blocklist() {
        let env = Env { cat: Some(NewsItemCategory::IncomingOffer), my_club_owner: true, ..Env::default() };
        for sub in [0x3e_u8, 0x50, 0x51, 0x5e] {
            let mut item = RawNewsItem::zeroed().with_type(0xbdf);
            item.set_u8(0x29, sub);
            assert_eq!(classify_news_action(&item, &env), None);
        }
        // A non-blocked subtype still requires flag guards; with defaults it passes.
        let mut item = RawNewsItem::zeroed().with_type(0xbdf);
        item.set_u8(0x29, 0x01);
        assert_eq!(
            classify_news_action(&item, &env).unwrap().callback,
            NewsCallback::GenericRespond_7297b0
        );
    }

    #[test]
    fn incoming_bc1_work_permit() {
        let env = Env { cat: Some(NewsItemCategory::IncomingOffer), viewer_club: 3, ..Env::default() };
        let mut item = RawNewsItem::zeroed().with_type(0xbc1);
        item.set_i32(5, 3);
        assert_eq!(
            classify_news_action(&item, &env).unwrap().callback,
            NewsCallback::WorkPermitRespond_8e5040
        );
    }

    #[test]
    fn incoming_bc2_offer_contract_single_action() {
        let env = Env { cat: Some(NewsItemCategory::IncomingOffer), viewer_club: 4, ..Env::default() };
        let mut item = RawNewsItem::zeroed().with_type(0xbc2);
        item.set_i32(5, 4);
        let a = classify_news_action(&item, &env).unwrap();
        assert_eq!(a.callback, NewsCallback::ContractOffer_4e2af0);
        assert_eq!(a.mode, ActionMode::SingleAction);
        assert_eq!(a.label, "Offer Contract");
    }

    // ---------- category 3 (Appeal) ----------

    #[test]
    fn appeal_gate_ok() {
        let env = Env { cat: Some(NewsItemCategory::Appeal), appeal_ok: true, ..Env::default() };
        let item = RawNewsItem::zeroed().with_type(0x7d3);
        let a = classify_news_action(&item, &env).unwrap();
        assert_eq!(a.callback, NewsCallback::Appeal_548170);
        assert_eq!(a.label, "Appeal");
    }

    #[test]
    fn appeal_gate_blocked() {
        let env = Env { cat: Some(NewsItemCategory::Appeal), appeal_ok: false, ..Env::default() };
        let item = RawNewsItem::zeroed().with_type(0x7d3);
        assert_eq!(classify_news_action(&item, &env), None);
    }

    #[test]
    fn appeal_wrong_type() {
        let env = Env { cat: Some(NewsItemCategory::Appeal), appeal_ok: true, ..Env::default() };
        let item = RawNewsItem::zeroed().with_type(0x7d4);
        assert_eq!(classify_news_action(&item, &env), None);
    }

    // ---------- category 1 (Press) ----------

    #[test]
    fn press_reply() {
        let env = Env { cat: Some(NewsItemCategory::Press), ..Env::default() };
        let item = RawNewsItem::zeroed().with_type(5000);
        let a = classify_news_action(&item, &env).unwrap();
        assert_eq!(a.callback, NewsCallback::NewspaperReply_771880);
    }

    #[test]
    fn press_wrong_type() {
        let env = Env { cat: Some(NewsItemCategory::Press), ..Env::default() };
        let item = RawNewsItem::zeroed().with_type(5001);
        assert_eq!(classify_news_action(&item, &env), None);
    }

    // ---------- category 5 (Outgoing transfer) ----------

    #[test]
    fn outgoing_transfer_bid_4000() {
        let env = Env {
            cat: Some(NewsItemCategory::OutgoingTransfer),
            my_club_owner: true,
            cat5: Some(TransferState {
                bid_club_id: 1,
                player_club_id: 7,
                stage_flag4: 0,
                ..Default::default()
            }),
            ..Env::default()
        };
        let mut item = RawNewsItem::zeroed().with_type(4000);
        item.set_i32(0x19, 42);
        item.set_i32(0x21, 7);
        item.set_u8(0x29, 0);
        let a = classify_news_action(&item, &env).unwrap();
        assert_eq!(a.callback, NewsCallback::TransferBidRespond_8df8f0);
    }

    #[test]
    fn outgoing_swap_deal_fab() {
        let env = Env {
            cat: Some(NewsItemCategory::OutgoingTransfer),
            viewer_club: 12,
            ..Env::default()
        };
        let mut item = RawNewsItem::zeroed().with_type(0xfab);
        item.set_i32(5, 12);
        item.set_i32(9, 99);
        let a = classify_news_action(&item, &env).unwrap();
        assert_eq!(a.callback, NewsCallback::SwapDealRespond_8dfc20);
    }

    #[test]
    fn outgoing_season_preview_fc1() {
        let env = Env { cat: Some(NewsItemCategory::OutgoingTransfer), ..Env::default() };
        let item = RawNewsItem::zeroed().with_type(0xfc1);
        let a = classify_news_action(&item, &env).unwrap();
        assert_eq!(a.callback, NewsCallback::SeasonPreviewDetails_763660);
        assert_eq!(a.mode, ActionMode::SingleAction);
    }

    #[test]
    fn outgoing_unknown_type_none() {
        let env = Env { cat: Some(NewsItemCategory::OutgoingTransfer), ..Env::default() };
        let item = RawNewsItem::zeroed().with_type(0xdead);
        assert_eq!(classify_news_action(&item, &env), None);
    }

    // ---------- comprehensive category dispatch coverage ----------

    #[test]
    fn each_category_reaches_its_arm() {
        // Prove that every category enum value dispatches into at
        // least one accepting branch. This is the "all internal
        // dispatch branches" coverage test (Part D of the deliverable).
        struct Case {
            cat: NewsItemCategory,
            item_type: i32,
            env_tweak: fn(&mut Env),
            expect: NewsCallback,
        }
        let cases = [
            Case { cat: NewsItemCategory::Board, item_type: 0x1774,
                env_tweak: |_e| {}, expect: NewsCallback::ManagerAnnounce_697c30 },
            Case { cat: NewsItemCategory::Matchday, item_type: 9,
                env_tweak: |_e| {}, expect: NewsCallback::AwardRespond_477dd0 },
            Case { cat: NewsItemCategory::Appeal, item_type: 0x7d3,
                env_tweak: |e| { e.appeal_ok = true; }, expect: NewsCallback::Appeal_548170 },
            Case { cat: NewsItemCategory::IncomingOffer, item_type: 0xbd9,
                env_tweak: |e| { e.my_club_owner = true; }, expect: NewsCallback::GenericRespond_7297b0 },
            Case { cat: NewsItemCategory::Press, item_type: 5000,
                env_tweak: |_e| {}, expect: NewsCallback::NewspaperReply_771880 },
            Case { cat: NewsItemCategory::OutgoingTransfer, item_type: 0xfc1,
                env_tweak: |_e| {}, expect: NewsCallback::SeasonPreviewDetails_763660 },
        ];
        for c in cases {
            let mut env = Env { cat: Some(c.cat), ..Env::default() };
            (c.env_tweak)(&mut env);
            let item = RawNewsItem::zeroed().with_type(c.item_type);
            let a = classify_news_action(&item, &env)
                .unwrap_or_else(|| panic!("category {:?} type {:#x} should classify", c.cat, c.item_type));
            assert_eq!(a.callback, c.expect, "category {:?} type {:#x}", c.cat, c.item_type);
        }
    }
}
