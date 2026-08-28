//! Batch 17: 5 more setup functions.
//!
//! Decompiles: `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
//! * `0x006FE590` — Match / fixture setup screen (large slot set, extracts
//!   from a match record at `param_2`; also toggles a human-seen flag on
//!   the linked club and can early-out via `FUN_007E7130(1, 2, 0)`).
//! * `0x0070D350` — "Next/Previous fixture" nav for the human's current
//!   competition — resolves the human, walks their current comp record
//!   to pick home/away flag, writes 3 slots.
//! * `0x0070F550` — Trivial 2-callback setup (no per-slot writes),
//!   only registers `FUN_007E6430(param_1, &LAB_0070F5F0, &LAB_0070F600, 0, 0, 0)`.
//! * `0x007297B0` — News-article/media detail screen with per-event-code
//!   variants (0xBDF, 0xBDD, 0xBE6, 0xBE7, 0xBE8, 0xFBC + default).
//!   Reads slots from an upstream media record via `FUN_0076D7D0(n)`.
//! * `0x00760B60` — 4-slot registration screen.

use serde::{Deserialize, Serialize};

// =====================================================================
// 0x006FE590 — Match / fixture setup
// =====================================================================

/// One `FUN_0076D7D0(n)`-style slot lookup — the caller reads the media
/// record and hands the values in. This is used by the news-article
/// builder below to keep it a pure port.
pub type MediaSlots = [u32; 32];

/// View for the match/fixture-detail screen at `FUN_006FE590`.
///
/// The exe writes 50-ish slots; most are zeroed/`-1`d and only slot 0
/// (match id), slot 0xE (`match.field_63 = *(u32*)(match+99)`) carry
/// runtime data. `bumped_human_seen` mirrors the side-effect on the
/// linked club record (`club.seen[active_human]++`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchDetailView {
    pub match_id: u32,
    /// Value written to slot 0xE = `*(u32*)(match + 0x63)`.
    pub match_field_63: u32,
    /// True if the exe would bump the club's per-human seen counter
    /// (`club.seen[active_human] == 0` before the write).
    pub bumped_human_seen: bool,
    /// True if the exe took the `FUN_007E7130(1, 2, 0)` early return —
    /// i.e. `match.field_57` was 0 and `FUN_0069AC90(match)` populated it.
    pub redispatched_to_alt: bool,
}

/// Direct port of `FUN_006FE590(param_1 = widget, param_2 = match_ptr)`.
///
/// Inputs:
/// * `registration_ok`  — result of `FUN_007E6430(param_1, ..., 1, ..., 0)`.
/// * `match_id`         — `param_2` (0 fires the MsgBox error path).
/// * `match_field_63`   — `*(u32*)(param_2 + 0x63)`.
/// * `human_seen_was_zero` — `club.seen[active_human] == 0`.
/// * `match_field_57_populated_by_alt` — result of the
///   `FUN_0069AC90(param_2)` fallback; true means the exe takes the
///   `FUN_007E7130(1, 2, 0)` early-return branch.
pub fn build_match_detail(
    registration_ok: bool,
    match_id: u32,
    match_field_63: u32,
    human_seen_was_zero: bool,
    match_field_57_populated_by_alt: bool,
) -> Option<MatchDetailView> {
    if match_id == 0 { return None; }        // exe: MsgBox + DAT_00b4d5a8 = 0
    if !registration_ok { return None; }
    Some(MatchDetailView {
        match_id,
        match_field_63,
        bumped_human_seen: human_seen_was_zero,
        redispatched_to_alt: match_field_57_populated_by_alt,
    })
}

// =====================================================================
// 0x0070D350 — Fixture-nav (prev/next competition fixture)
// =====================================================================

/// View for the fixture-nav screen at `FUN_0070D350`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixtureNavView {
    /// Slot 0 — human's current comp id (`*(u32*)(human + 0xE6)`).
    pub current_comp_id: u32,
    /// Slot 1 — pointer to comp fixture struct + 4 (kept as an opaque
    /// id in the port; the exe only re-hands this back to the widget).
    pub fixture_ref: u32,
    /// Slot 2 — home/away boolean:
    /// true iff `comp.home == current_comp_id && !comp.home_played`
    /// OR   `comp.away == current_comp_id && !comp.away_played`.
    pub is_home_leg: bool,
}

/// Direct port of `FUN_0070D350()`.
///
/// * `registration_ok`  — result of `FUN_007E6570(..., 1, 0, 0)`.
/// * `human_ptr_present` — `FUN_0074D010(0)` returned non-zero.
/// * `current_comp_id`   — `*(u32*)(human + 0xE6)`; 0 fires error path.
/// * `fixture_ptr_plus4` — `*(u32*)(human + 0xE2) + 4`; caller passes
///   0 if the fixture struct base was `-4` (the exe's error condition).
/// * `comp_home`/`comp_away` — `*(u32*)(fixture + 0x20/0x24)`.
/// * `comp_home_played`/`comp_away_played` — `*(u8*)(fixture + 0x44/0x45)`.
pub fn build_fixture_nav(
    registration_ok: bool,
    human_ptr_present: bool,
    current_comp_id: u32,
    fixture_ptr_plus4: u32,
    comp_home: u32,
    comp_away: u32,
    comp_home_played: bool,
    comp_away_played: bool,
) -> Option<FixtureNavView> {
    if !registration_ok { return None; }
    if !human_ptr_present { return None; }         // exe: MsgBox 0x1CB7
    if current_comp_id == 0 { return None; }       // exe: MsgBox 0x1CBF
    if fixture_ptr_plus4 == 0 { return None; }     // exe: MsgBox 0x1CC7
    let is_home_leg =
        (current_comp_id == comp_home && !comp_home_played)
        || (current_comp_id == comp_away && !comp_away_played);
    Some(FixtureNavView { current_comp_id, fixture_ref: fixture_ptr_plus4, is_home_leg })
}

// =====================================================================
// 0x0070F550 — trivial 2-callback registration
// =====================================================================

/// View for the trivial screen at `FUN_0070F550`. Registers a builder
/// + tick callback and writes NO slots — the widget id is all we need.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrivialRegistrationView {
    pub widget_id: u32,
}

/// Direct port of `FUN_0070F550(param_1)`. `param_1 == 0` fires the
/// MsgBox error path and returns `None`. This function does not
/// consult a registration boolean because the exe ignores
/// `FUN_007E6430`'s return value here.
pub fn build_trivial_registration(widget_id: u32) -> Option<TrivialRegistrationView> {
    if widget_id == 0 { return None; }
    Some(TrivialRegistrationView { widget_id })
}

// =====================================================================
// 0x007297B0 — News-article / media detail
// =====================================================================

/// Which per-event-code slot pattern the exe uses.
///
/// The switch on `local_2ec[0]` picks how to fill the payload. Codes
/// come from the media record populated by `FUN_0076EB10(param_1, ...)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum MediaEventCode {
    /// 0xBDD — 6-slot pattern, `local_2f4 = slot[5]`, `local_2f0 = 0`.
    Code0BDD = 0x0BDD,
    /// 0xBDF — reshuffled: 0xF/0x10/0x11/0x12 into main uVars, 4 into
    /// `local_2f8`, 9 into `local_2f4`, 1 into `local_2f0`.
    Code0BDF = 0x0BDF,
    /// 0xBE6 — slot 9 → `local_2f8`, `local_2f4 = 0x82`,
    /// `local_2fc = slot[10]`.
    Code0BE6 = 0x0BE6,
    /// 0xBE7 — falls through to the default pattern (goto LAB_00729A39).
    Code0BE7 = 0x0BE7,
    /// 0xBE8 — slot 8 → `local_2f8`, `local_2f4 = 0`, `local_2f0 = 0`.
    Code0BE8 = 0x0BE8,
    /// 0xFBC — slot 4 → `local_2f8`, `local_2f4 = 0x1F`; guard on
    /// slot[0xF]/slot[0x10] against `DAT_00ACD564`.
    Code0FBC = 0x0FBC,
    /// Anything else — same as 0xBE7: default 6-slot pattern with
    /// `local_2f4 = slot[9]`.
    Default = 0xFFFF,
}

/// View for the news-article/media detail screen at `FUN_007297B0`.
///
/// Field names correspond to the exe's local variables. Slots 0..=9
/// are written to the widget in this order:
/// `param_2, local_2f8, local_2f4, 3, u0, u1, u2, u3, local_2f0,
///  local_2fc`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaArticleView {
    pub caller_arg: u32,   // slot 0 = param_2
    pub var_2f8: u32,      // slot 1
    pub var_2f4: u32,      // slot 2
    pub val_u0: u32,       // slot 4
    pub val_u1: u32,       // slot 5
    pub val_u2: u32,       // slot 6
    pub val_u3: u32,       // slot 7
    pub var_2f0: u32,      // slot 8
    pub var_2fc: i32,      // slot 9 (signed — starts at -1)
}

/// Direct port of `FUN_007297B0(param_1, param_2)`.
///
/// * `registration_ok`   — result of `FUN_007E6570(..., 1, ..., 0)`.
/// * `media_record_valid` — result of `FUN_0076EB10(param_1, param_2, buf)`.
/// * `param_2`            — passthrough (goes into slot 0).
/// * `code`               — `buf[0]` — the event code.
/// * `slots`              — 32-entry pool the exe reads via `FUN_0076D7D0(n)`.
/// * `fbc_index_max`      — `DAT_00ACD564` (bound on slot[0x10] for 0xFBC).
pub fn build_media_article(
    param_1: u32,
    registration_ok: bool,
    media_record_valid: bool,
    param_2: u32,
    code: MediaEventCode,
    slots: &MediaSlots,
    fbc_index_max: i32,
) -> Option<MediaArticleView> {
    if param_1 == 0 { return None; }             // MsgBox 0x3B5A
    if !media_record_valid { return None; }      // MsgBox 0x3B61
    if !registration_ok { return None; }

    let mut u0 = 0u32; let mut u1 = 0u32; let mut u2 = 0u32; let mut u3 = 0u32;
    let mut v2f8 = 0u32; let mut v2f4 = 0u32; let mut v2f0 = 0u32;
    let mut v2fc: i32 = -1;

    let default_fill = |u0: &mut u32, u1: &mut u32, u2: &mut u32, u3: &mut u32,
                        v2f8: &mut u32, v2f4: &mut u32| {
        *u0 = slots[0]; *u1 = slots[1]; *u2 = slots[2]; *u3 = slots[3];
        *v2f8 = slots[4]; *v2f4 = slots[9];
    };

    match code {
        MediaEventCode::Code0BE6 => {
            u0 = slots[0]; u1 = slots[1]; u2 = slots[2]; u3 = slots[3];
            v2f8 = slots[9]; v2f4 = 0x82; v2f0 = 0;
            v2fc = slots[10] as i32;
        }
        MediaEventCode::Code0BDD => {
            u0 = slots[0]; u1 = slots[1]; u2 = slots[2]; u3 = slots[3];
            v2f8 = slots[4]; v2f4 = slots[5];
        }
        MediaEventCode::Code0BDF => {
            u0 = slots[0xF]; u1 = slots[0x10]; u2 = slots[0x11]; u3 = slots[0x12];
            v2f8 = slots[4]; v2f4 = slots[9]; v2f0 = slots[1];
        }
        MediaEventCode::Code0BE8 => {
            u0 = slots[0]; u1 = slots[1]; u2 = slots[2]; u3 = slots[3];
            v2f8 = slots[8]; v2f4 = 0; v2f0 = 0;
        }
        MediaEventCode::Code0FBC => {
            u0 = slots[0]; u1 = slots[1]; u2 = slots[2]; u3 = slots[3];
            v2f8 = slots[4]; v2f4 = 0x1F;
            if slots[0xF] != 0 {
                let idx = slots[0x10] as i32;
                if idx >= 0 && idx <= fbc_index_max - 1 { v2fc = idx; }
            }
        }
        MediaEventCode::Code0BE7 | MediaEventCode::Default => {
            default_fill(&mut u0, &mut u1, &mut u2, &mut u3, &mut v2f8, &mut v2f4);
        }
    }

    Some(MediaArticleView {
        caller_arg: param_2,
        var_2f8: v2f8, var_2f4: v2f4,
        val_u0: u0, val_u1: u1, val_u2: u2, val_u3: u3,
        var_2f0: v2f0, var_2fc: v2fc,
    })
}

// =====================================================================
// 0x00760B60 — 4-slot registration
// =====================================================================

/// View for the small registration screen at `FUN_00760B60`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmallFourSlotView {
    pub arg_a: u32,      // slot 0 = param_1
    pub flag: i8,        // slot 1 = param_2 (signed char sign-extended to int)
    pub arg_c: u32,      // slot 2 = param_3
    pub sentinel: i32,   // slot 3 = -1 (0xFFFFFFFF)
}

/// Direct port of `FUN_00760B60(param_1, param_2, param_3)`.
pub fn build_small_four_slot(
    registration_ok: bool,
    a: u32,
    flag: i8,
    c: u32,
) -> Option<SmallFourSlotView> {
    if !registration_ok { return None; }
    Some(SmallFourSlotView { arg_a: a, flag, arg_c: c, sentinel: -1 })
}

// =====================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 006FE590 ----
    #[test]
    fn match_detail_carries_id_and_field_63() {
        let v = build_match_detail(true, 1234, 0xDEAD, false, false).unwrap();
        assert_eq!(v.match_id, 1234);
        assert_eq!(v.match_field_63, 0xDEAD);
        assert!(!v.bumped_human_seen);
        assert!(!v.redispatched_to_alt);
    }
    #[test]
    fn match_detail_zero_id_returns_none() {
        assert!(build_match_detail(true, 0, 0, false, false).is_none());
    }
    #[test]
    fn match_detail_registration_failure_returns_none() {
        assert!(build_match_detail(false, 1, 0, false, false).is_none());
    }
    #[test]
    fn match_detail_bumps_seen_only_when_zero() {
        let v = build_match_detail(true, 1, 0, true, false).unwrap();
        assert!(v.bumped_human_seen);
    }

    // ---- 0070D350 ----
    #[test]
    fn fixture_nav_home_leg_when_current_matches_home_unplayed() {
        let v = build_fixture_nav(true, true, 42, 8, 42, 99, false, false).unwrap();
        assert!(v.is_home_leg);
        assert_eq!(v.current_comp_id, 42);
        assert_eq!(v.fixture_ref, 8);
    }
    #[test]
    fn fixture_nav_not_home_when_home_already_played() {
        let v = build_fixture_nav(true, true, 42, 8, 42, 99, true, false).unwrap();
        assert!(!v.is_home_leg);
    }
    #[test]
    fn fixture_nav_missing_human_returns_none() {
        assert!(build_fixture_nav(true, false, 42, 8, 42, 99, false, false).is_none());
    }
    #[test]
    fn fixture_nav_zero_comp_returns_none() {
        assert!(build_fixture_nav(true, true, 0, 8, 0, 0, false, false).is_none());
    }
    #[test]
    fn fixture_nav_zero_fixture_returns_none() {
        assert!(build_fixture_nav(true, true, 42, 0, 42, 99, false, false).is_none());
    }

    // ---- 0070F550 ----
    #[test]
    fn trivial_registration_carries_widget_id() {
        assert_eq!(build_trivial_registration(7).unwrap().widget_id, 7);
    }
    #[test]
    fn trivial_registration_zero_widget_returns_none() {
        assert!(build_trivial_registration(0).is_none());
    }

    // ---- 007297B0 ----
    #[test]
    fn media_article_default_pattern() {
        let mut slots = [0u32; 32];
        for i in 0..10 { slots[i] = (i as u32) + 100; }
        let v = build_media_article(1, true, true, 0x50,
            MediaEventCode::Default, &slots, 10).unwrap();
        assert_eq!(v.caller_arg, 0x50);
        assert_eq!(v.val_u0, 100); assert_eq!(v.val_u3, 103);
        assert_eq!(v.var_2f8, 104);   // slot 4
        assert_eq!(v.var_2f4, 109);   // slot 9
        assert_eq!(v.var_2fc, -1);
    }
    #[test]
    fn media_article_be6_puts_slot9_into_2f8_and_82_into_2f4() {
        let mut slots = [0u32; 32];
        slots[9] = 900; slots[10] = 25;
        let v = build_media_article(1, true, true, 0,
            MediaEventCode::Code0BE6, &slots, 100).unwrap();
        assert_eq!(v.var_2f8, 900);
        assert_eq!(v.var_2f4, 0x82);
        assert_eq!(v.var_2fc, 25);
    }
    #[test]
    fn media_article_bdf_reshuffles_high_slots() {
        let mut slots = [0u32; 32];
        slots[0xF] = 15; slots[0x10] = 16; slots[0x11] = 17; slots[0x12] = 18;
        slots[1] = 1; slots[4] = 4; slots[9] = 9;
        let v = build_media_article(1, true, true, 0,
            MediaEventCode::Code0BDF, &slots, 100).unwrap();
        assert_eq!(v.val_u0, 15);
        assert_eq!(v.val_u3, 18);
        assert_eq!(v.var_2f8, 4);
        assert_eq!(v.var_2f4, 9);
        assert_eq!(v.var_2f0, 1);
    }
    #[test]
    fn media_article_fbc_bounds_check_rejects_out_of_range() {
        let mut slots = [0u32; 32];
        slots[0xF] = 1; slots[0x10] = 1000;  // out of range vs fbc_index_max=10
        let v = build_media_article(1, true, true, 0,
            MediaEventCode::Code0FBC, &slots, 10).unwrap();
        assert_eq!(v.var_2fc, -1);
        assert_eq!(v.var_2f4, 0x1F);
    }
    #[test]
    fn media_article_zero_param1_returns_none() {
        let slots = [0u32; 32];
        assert!(build_media_article(0, true, true, 0,
            MediaEventCode::Default, &slots, 10).is_none());
    }
    #[test]
    fn media_article_invalid_record_returns_none() {
        let slots = [0u32; 32];
        assert!(build_media_article(1, true, false, 0,
            MediaEventCode::Default, &slots, 10).is_none());
    }

    // ---- 00760B60 ----
    #[test]
    fn small_four_slot_carries_all_args() {
        let v = build_small_four_slot(true, 42, -3, 77).unwrap();
        assert_eq!(v.arg_a, 42);
        assert_eq!(v.flag, -3);
        assert_eq!(v.arg_c, 77);
        assert_eq!(v.sentinel, -1);
    }
    #[test]
    fn small_four_slot_registration_failure_returns_none() {
        assert!(build_small_four_slot(false, 1, 0, 2).is_none());
    }
}

// =====================================================================
// populate_from_world companions
// =====================================================================

pub const TODO_POPULATOR_INFO: &str = "\
b17: MatchDetailView reads handles {b17.match_detail.match_id, match_field_63} + \
flags {b17.match_detail.human_seen_was_zero, b17.match_detail.field_57_alt}; \
FixtureNavView reads flag {b17.fixture_nav.human_ptr_present} + handles \
{b17.fixture_nav.current_comp_id, fixture_ptr_plus4, comp_home, comp_away} + \
flags {b17.fixture_nav.comp_home_played, comp_away_played}; \
TrivialRegistrationView reads handle {b17.trivial.widget_id}; \
MediaArticleView reads handles {b17.media.param_1, param_2, fbc_index_max} + \
flag {b17.media.record_valid} + buffer {b17.media.slots} (interpreted as up to \
32 LE u32s) — event code left as Default until the code decoder lands. \
SmallFourSlotView reads handles {b17.small4.a, b17.small4.c} + byte {b17.small4.flag}.\n\
UNKNOWNS: wave-B typed pools not yet available; the specific MediaEventCode \
is decoded from the media record populated by FUN_0076EB10 — populator defaults \
to `MediaEventCode::Default`.";

use crate::world_facade::WorldFacade;

fn slots_from_facade(world: &WorldFacade, key: &str) -> MediaSlots {
    let raw = world.buffer(key);
    let mut slots: MediaSlots = [0u32; 32];
    for (i, chunk) in raw.chunks(4).enumerate().take(32) {
        let mut b = [0u8; 4];
        for (j, byte) in chunk.iter().enumerate() { b[j] = *byte; }
        slots[i] = u32::from_le_bytes(b);
    }
    slots
}

/// Populator for [`MatchDetailView`].
pub fn populate_match_detail(world: &WorldFacade) -> Option<MatchDetailView> {
    build_match_detail(
        world.registration_ok,
        world.handle("b17.match_detail.match_id"),
        world.handle("b17.match_detail.match_field_63"),
        world.flag("b17.match_detail.human_seen_was_zero"),
        world.flag("b17.match_detail.field_57_alt"),
    )
}

/// Populator for [`FixtureNavView`].
pub fn populate_fixture_nav(world: &WorldFacade) -> Option<FixtureNavView> {
    build_fixture_nav(
        world.registration_ok,
        world.flag("b17.fixture_nav.human_ptr_present"),
        world.handle("b17.fixture_nav.current_comp_id"),
        world.handle("b17.fixture_nav.fixture_ptr_plus4"),
        world.handle("b17.fixture_nav.comp_home"),
        world.handle("b17.fixture_nav.comp_away"),
        world.flag("b17.fixture_nav.comp_home_played"),
        world.flag("b17.fixture_nav.comp_away_played"),
    )
}

/// Populator for [`TrivialRegistrationView`].
pub fn populate_trivial_registration(world: &WorldFacade) -> Option<TrivialRegistrationView> {
    build_trivial_registration(world.handle("b17.trivial.widget_id"))
}

/// Populator for [`MediaArticleView`]. Event code is left at
/// `MediaEventCode::Default` until the media-record decoder lands.
pub fn populate_media_article(world: &WorldFacade) -> Option<MediaArticleView> {
    let slots = slots_from_facade(world, "b17.media.slots");
    build_media_article(
        world.handle("b17.media.param_1"),
        world.registration_ok,
        world.flag("b17.media.record_valid"),
        world.handle("b17.media.param_2"),
        MediaEventCode::Default,
        &slots,
        world.byte("b17.media.fbc_index_max") as i32,
    )
}

/// Populator for [`SmallFourSlotView`].
pub fn populate_small_four_slot(world: &WorldFacade) -> Option<SmallFourSlotView> {
    build_small_four_slot(
        world.registration_ok,
        world.handle("b17.small4.a"),
        world.byte("b17.small4.flag") as i8,
        world.handle("b17.small4.c"),
    )
}

#[cfg(test)]
mod populate_tests {
    use super::*;
    #[test]
    fn populate_match_detail_zero_id_returns_none() {
        assert!(populate_match_detail(&WorldFacade::ready()).is_none());
        let w = WorldFacade::ready().with_handle("b17.match_detail.match_id", 5);
        assert!(populate_match_detail(&w).is_some());
    }
    #[test]
    fn populate_fixture_nav_needs_human_and_comp_and_fixture() {
        assert!(populate_fixture_nav(&WorldFacade::ready()).is_none());
        let w = WorldFacade::ready()
            .with_flag("b17.fixture_nav.human_ptr_present", true)
            .with_handle("b17.fixture_nav.current_comp_id", 3)
            .with_handle("b17.fixture_nav.fixture_ptr_plus4", 4)
            .with_handle("b17.fixture_nav.comp_home", 3);
        let v = populate_fixture_nav(&w).unwrap();
        assert!(v.is_home_leg);
    }
    #[test]
    fn populate_trivial_registration_zero_returns_none() {
        assert!(populate_trivial_registration(&WorldFacade::ready()).is_none());
    }
    #[test]
    fn populate_media_article_needs_param_and_valid() {
        assert!(populate_media_article(&WorldFacade::ready()).is_none());
        let w = WorldFacade::ready()
            .with_handle("b17.media.param_1", 1)
            .with_flag("b17.media.record_valid", true);
        assert!(populate_media_article(&w).is_some());
    }
    #[test]
    fn populate_small_four_slot_ok() {
        assert!(populate_small_four_slot(&WorldFacade::ready()).is_some());
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A pattern)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b17 pool sources: MatchDetail/FixtureNav require a live match + \
    fixture pointer from the day-tick engine (no facade). \
    TrivialRegistration takes a widget id; we use the seat. \
    MediaArticle needs a media record populated by FUN_0076EB10 \
    (event-code driven) — populator uses the Default 6-slot pattern \
    with zeroed slots.";

use crate::world_pools::WorldPools;

pub fn populate_match_detail_from_pools(
    _pools: &WorldPools<'_>,
) -> Option<MatchDetailView> {
    // No live-match pool → exe's zero-id branch returns None.
    build_match_detail(true, 0, 0, false, false)
}

pub fn populate_fixture_nav_from_pools(
    pools: &WorldPools<'_>,
) -> Option<FixtureNavView> {
    // Needs a fixture pointer + comp id; without the fixture pool we
    // can't populate. Mirror the exe's MsgBox path with None.
    let _ = pools;
    build_fixture_nav(true, false, 0, 0, 0, 0, false, false)
}

pub fn populate_trivial_registration_from_pools(
    pools: &WorldPools<'_>,
) -> Option<TrivialRegistrationView> {
    let id = pools.active_human_seat?;
    build_trivial_registration(id)
}

pub fn populate_media_article_from_pools(
    pools: &WorldPools<'_>,
) -> Option<MediaArticleView> {
    // Requires param_1 != 0 and media_record_valid; without a media
    // pool on the facade we return None on empty and provide a "seated"
    // path that yields the Default 6-slot pattern with zeroed slots.
    let param_1 = pools.active_human_seat?;
    let slots: MediaSlots = [0; 32];
    build_media_article(param_1, true, true, 0,
        MediaEventCode::Default, &slots, 0)
}

pub fn populate_small_four_slot_from_pools(
    pools: &WorldPools<'_>,
) -> Option<SmallFourSlotView> {
    let a = pools.active_human_seat.unwrap_or(0);
    build_small_four_slot(true, a, 0, 0)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;

    #[test]
    fn empty_pools_return_expected() {
        let p = WorldPools::empty();
        assert!(populate_match_detail_from_pools(&p).is_none());
        assert!(populate_fixture_nav_from_pools(&p).is_none());
        assert!(populate_trivial_registration_from_pools(&p).is_none());
        assert!(populate_media_article_from_pools(&p).is_none());
        assert!(populate_small_four_slot_from_pools(&p).is_some());
    }

    #[test]
    fn seat_enables_trivial_and_media() {
        let p = WorldPools { active_human_seat: Some(11), ..WorldPools::empty() };
        assert_eq!(populate_trivial_registration_from_pools(&p).unwrap().widget_id, 11);
        assert!(populate_media_article_from_pools(&p).is_some());
    }
}
