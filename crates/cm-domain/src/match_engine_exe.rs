//! Honest port of the cm0102.exe match engine cluster.
//!
//! Replaces the Poisson placeholder in [`crate::match_engine`] with a
//! direct port of the exe's own code. Structure mirrors the exe's TU
//! layout (see `reports/match_*.md` for decode maps):
//!
//! | Rust item                          | exe function        | exe file           |
//! |------------------------------------|---------------------|--------------------|
//! | [`MatchDayCtx`]                    | (driver ctx struct) | match_day.cpp      |
//! | [`match_day_build`]                | FUN_00699640        | match_day.cpp      |
//! | [`match_day_play`]                 | FUN_00699D90        | match_day.cpp      |
//! | [`MatchCtx`]                       | (63 KB engine buf)  | match_eng.cpp      |
//! | [`match_eng_setup`]                | FUN_0069D950        | match_eng.cpp      |
//! | [`match_events_generate`]          | FUN_006BC8D0        | match_events.cpp   |
//! | [`match_tick`]                     | FUN_0069F2F0        | match_eng.cpp      |
//! | helpers `pick_next_player` ...     | FUN_0069B4A0..D880  | match_eng.cpp      |
//!
//! # RNG
//!
//! The exe uses one process-global PRNG behind `FUN_008FC4F0(n) → int in
//! [0, n)`. We thread [`MatchRng`] explicitly through every call site
//! instead — same distribution, but per-match seed so replays reproduce.
//!
//! # Struct fidelity
//!
//! Field offsets and sizes match the decode reports. Where the decode
//! ledger is honest but incomplete (e.g. the tick pump's per-side
//! sub-record fields aren't fully mapped), the field is present as a
//! named `Vec<u8>` block sized to the exe's stride, with per-field
//! accessors added as the decode lands.

#![allow(dead_code)] // Extensive port; unused fields are documented layout.

use serde::{Deserialize, Serialize};

// ============================================================================
// RNG — port of FUN_008FC4F0(n) → uint in [0, n).
// ============================================================================

/// Match RNG. Seeded per-match so replays reproduce. splitmix64.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MatchRng {
    state: u64,
}

impl MatchRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed.wrapping_add(0x9E3779B97F4A7C15) }
    }

    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// exe: `FUN_008FC4F0(n)` — uniform integer in `[0, n)`. Callers
    /// that pass `n=0` in the exe get UB; we panic to catch ports that
    /// mistranslated the range.
    pub fn range(&mut self, n: u32) -> u32 {
        assert!(n > 0, "FUN_008FC4F0(0) is UB in the exe");
        (self.next_u64() % (n as u64)) as u32
    }

    /// Convenience: `RNG(n) == 0` — the exe's most common gate shape.
    pub fn hit(&mut self, n: u32) -> bool {
        self.range(n) == 0
    }
}

// ============================================================================
// Match-day driver context — `MatchDayCtx` in the exe, ~0x38 bytes.
// ============================================================================

/// Comp entry inside `MatchDayCtx`. Exe stride: 0x18 bytes (6 dwords).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DayCompEntry {
    /// Competition id (result of `FUN_00492640(comp_idx)`).
    pub comp_id: i32,
    /// Range of group entries this comp owns in `MatchDayCtx.groups`.
    pub first_group_idx: i32,
    pub last_group_idx: i32,
    /// Range of fixture entries this comp owns in `MatchDayCtx.fixtures`.
    pub first_fixture_idx: i32,
    pub last_fixture_idx: i32,
    /// Slot flags — cleared to 0 on init.
    pub flags: u32,
}

/// Group entry inside `MatchDayCtx`. Exe stride: 0x54 bytes.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DayGroupEntry {
    pub group_id: i32,
    pub first_fixture_idx: i32,
    pub last_fixture_idx: i32,
    pub reserved_a: i32,
    pub reserved_b: i32,
    /// 64 bytes of per-group scratch (16 zeroed dwords in the exe).
    pub scratch: Vec<u8>,
}

/// One day's fixture entry. Exe stride: 0x69 (105) bytes; the exe copies
/// 0x4E bytes from the master fixture then zeros the trailing session
/// state fields at fixed offsets. We hold the same fields symbolically.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DayFixture {
    /// From raw fixture (`fixture[+0]`) — league id.
    pub league_id: i32,
    /// From raw fixture (`fixture[+4]`) — group id used to bucket via qsort.
    pub group_id: i32,
    /// Home / away club record ids (`fixture[+0x1C]`, `+0x20`).
    pub home_club_id: u32,
    pub away_club_id: u32,
    /// Comp attributes copied from raw record.
    pub comp_id: i32,
    /// **Fixture state byte** — `+0x67`. Sentinel `0xFF` = not started;
    /// runtime values: `0xFE` = consumed, `-2` = in progress, `2` = full
    /// time, `8` = abandoned/completed early.
    pub state: u8,
    /// `+0x53` — pre-match/report ptr (allocated 0x11D bytes in Phase A).
    pub has_pre_match_report: bool,
    /// `+0x57` — engine handle (`Box<MatchCtx>` in this port). `None`
    /// until Phase B allocs it.
    pub engine: Option<Box<MatchCtx>>,
    /// `+0x5B` — 0x11D-byte pre-match report struct (Phase A only).
    pub pre_match_report: Option<Box<PreMatchReport>>,
    /// `+0x5F` — 0xD7A-byte final match report (post-simulation).
    pub finalized_report: Option<Box<MatchReport>>,
    /// Cup-round flag `+0x18` (cup-competition tag).
    pub cup_round: i32,
    /// Comp-type flag `+0x4B` (< 0 = cup format).
    pub comp_type_flag: i8,
}

impl Default for DayFixture {
    fn default() -> Self {
        Self {
            league_id: 0,
            group_id: -1,
            home_club_id: 0,
            away_club_id: 0,
            comp_id: 0,
            state: 0xFF,
            has_pre_match_report: false,
            engine: None,
            pre_match_report: None,
            finalized_report: None,
            cup_round: 0,
            comp_type_flag: 0,
        }
    }
}

/// Exe struct `MatchDayCtx` (~0x38 bytes). Three parallel arrays with
/// range-chain indexing (`comp.first/last_fixture_idx` → fixtures).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MatchDayCtx {
    /// `+0x00` — driver mode/kind (0..2).
    pub mode: u8,
    /// `+0x01..+0x0C` — competitions block. Stride 0x18 in the exe.
    pub competitions: Vec<DayCompEntry>,
    /// `+0x0D..+0x18` — groups block. Stride 0x54.
    pub groups: Vec<DayGroupEntry>,
    /// `+0x19..+0x24` — fixtures block. Stride 0x69.
    pub fixtures: Vec<DayFixture>,
    /// `+0x25` — count of fixtures that have finished this day.
    pub finished_count: i32,
    /// `+0x2D68` — last-competition id cache used by event routing.
    pub last_competition_id: i32,
    /// `+0x2F..+0x32` — sentinel dwords `-1`, cleared at build start.
    pub sentinel: [i32; 4],
    /// `+0x33` — post-build flag = 1 after build completes.
    pub build_flag: u8,
}

// ============================================================================
// Pre-match / final report structs (Phase A and Phase B outputs).
// ============================================================================

/// Exe struct: 0x11D bytes allocated at `fixture+0x5B` in Phase A. Holds
/// four zeroed 16-dword stat blocks (attacks/shots/possession per slot)
/// plus refs-count and default flags.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreMatchReport {
    pub flag_byte: u8,
    pub stat_blocks: [[u32; 16]; 4],
    pub refs_engaged: u8,
    pub default_flag: u8,     // = 1
    pub aggregate_counters: Vec<u8>,
}

/// Exe struct: 0xD7A bytes allocated at `fixture+0x5F` after
/// simulation. Populated by `FUN_00710C70(engine, report)`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchReport {
    pub home_score: u8,
    pub away_score: u8,
    /// Up to 13 goal minutes per side (from `+0x0A..+0x40` HOME,
    /// `+0x8D51..` AWAY).
    pub home_goal_minutes: Vec<u16>,
    pub away_goal_minutes: Vec<u16>,
    pub home_scorer_ids: Vec<u32>,
    pub away_scorer_ids: Vec<u32>,
    pub home_assist_ids: Vec<u32>,
    pub away_assist_ids: Vec<u32>,
    pub home_shots: u8,
    pub home_shots_on_target: u8,
    pub away_shots: u8,
    pub away_shots_on_target: u8,
    pub home_red_cards: Vec<u32>,
    pub away_red_cards: Vec<u32>,
    pub abandoned: bool,
    pub extra_time_played: bool,
}

// ============================================================================
// MatchCtx — the 63 KB per-match engine buffer (exe: `operator_new(0xF63B)`).
// ============================================================================

/// Per-side pre-kickoff event pushed by `match_eng_setup` (`FUN_00672320`
/// with opcode `0x19`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreMatchEvent {
    /// Event type: 4=minor injury, 5=form injury withdrawal, 6=suspension
    /// forgotten, 0x13=serious injury/foul, 0x1E=withdrawn ill.
    pub event_type: u16,
    pub opp_team_id: u32,
    pub player_id: u32,
    pub flags: u32,
    pub side: u8,          // 0 = home, 1 = away
    pub is_first_choice_gk: bool,
}

/// One slot in the per-side player array. Exe stride: 0x1BE bytes.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchPlayerSlot {
    /// `+0x00..+0x18` header (formation code / team-ptr copies etc.).
    pub header: [u8; 0x18],
    /// `+0x19` slot_valid flag; `< 0` = no player.
    pub slot_valid: i8,
    /// Middle of the block (opaque per-slot state until decoded).
    pub middle: Vec<u8>,
    /// `+0x69` player-record id (exe stores ptr; we store id).
    pub player_id: u32,
    /// `+0x6D` strength-record id.
    pub strength_id: u32,
    /// Live per-tick stats — booked, injured, subbed etc.
    pub booked: bool,
    pub sent_off: bool,
    pub injured: bool,
    pub subbed_off: bool,
}

/// One side (home or away). Exe stride: 0x22D8 bytes.
///
/// Layout: header 0x18 + slot_valid 1 + gap + 20 × 0x1BE slots.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchSideBlock {
    /// `+0x00..+0x18` block header.
    pub header: [u8; 0x18],
    /// `+0x19` — first slot's valid flag (in exe the "side has any
    /// players" gate).
    pub any_valid: bool,
    /// 20 player slots.
    pub slots: Vec<MatchPlayerSlot>,
}

impl MatchSideBlock {
    pub fn new() -> Self {
        Self { header: [0; 0x18], any_valid: false,
               slots: (0..20).map(|_| MatchPlayerSlot::default()).collect() }
    }
}

/// Form baselines stamped at `+0xF628..+0xF633`. Three floats: reserved,
/// goals-per-match-avg, shots-per-match-avg. Exe defaults are `6.0f,
/// 6.0f, 7.0f` per side.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FormBaselines {
    pub reserved: f32,      // observed as 0.0 in exe (`+0xF628`)
    pub goals_avg: f32,     // = 6.0
    pub shots_avg: f32,     // = 6.0
    pub other_avg: f32,     // = 7.0
}

impl Default for FormBaselines {
    fn default() -> Self { Self { reserved: 0.0, goals_avg: 6.0,
                                  shots_avg: 6.0, other_avg: 7.0 } }
}

/// Pitch modifiers stamped at `+0x91D2..+0x91DD` by `FUN_00845CC0`.
/// Defaults when the pitch record is missing: `[100, 90, 10]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PitchModifiers {
    pub grass_quality: u32,     // +0x91D2, default 100
    pub firmness: u32,          // +0x91D6, default 90
    pub weather_effect: u32,    // +0x91DA, default 10
}

impl Default for PitchModifiers {
    fn default() -> Self { Self { grass_quality: 100, firmness: 90, weather_effect: 10 } }
}

/// The exe's per-match state buffer. Physical size on x86 is 0xF63B (~63
/// KB) with a fixed field layout; we translate that layout into named
/// fields but keep the strides visible in comments so ports of downstream
/// helpers can locate their offsets.
///
/// Only fields the decode has actually pinned are named; the rest are
/// TODO'd out until the tick-pump decode arrives.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct MatchCtx {
    /// `+0x00` — quick-sim flag; if `DAT_00A46314==0` (commentary off)
    /// the exe forces this to `1`.
    pub quick_sim: bool,
    /// `+0x01` — weather/pitch code from `FUN_00912E30`.
    pub weather_pitch_code: i8,
    /// `+0x02..+0x03` — ref / linesman ids (init `-1`).
    pub ref_id: i8,
    pub linesman_id: i8,
    /// `+0x04..+0x05` — initial morale seeds; exe stamps `11, 11`.
    pub morale_home_init: u8,
    pub morale_away_init: u8,
    /// `+0x0A..+0x40` (HOME) and `+0x8D51..` (AWAY) — goal minute lists.
    pub home_goal_minutes: Vec<u16>,
    pub away_goal_minutes: Vec<u16>,
    /// `+0x41..` scorer ids per side.
    pub home_scorer_ids: Vec<u32>,
    pub away_scorer_ids: Vec<u32>,
    /// `+0x78..` assist ids per side.
    pub home_assist_ids: Vec<u32>,
    pub away_assist_ids: Vec<u32>,
    /// `+0xAF..+0xB0` — subs used.
    pub subs_used_home: u8,
    pub subs_used_away: u8,
    /// `+0x1CE..+0x1D0` — pitch dims (110 × 110 in the exe).
    pub pitch_length: u16,
    pub pitch_width: u16,
    /// `+0x1D2..+0x1D5` — half clocks (init 10000 each).
    pub half_clock_home: u16,
    pub half_clock_away: u16,
    /// `+0x1D6/+0x1D8` — reputation copies from each team.
    pub home_reputation: u16,
    pub away_reputation: u16,
    /// Ball-height byte at pitch/match `+0x8EA9`. Fed into shot-damage
    /// randomness at `006d63f0.c:248`:
    /// `iVar17 = FUN_008fc4f0((int)cVar10 * (int)cVar10 * (int)cVar10 * 0x32)`.
    /// Range 0..4 in the exe (4 = lob). Default 0 = ground ball.
    #[serde(default)]
    pub ball_height: i8,
    /// Score box. Exe stores per-side goal counts as bytes at
    /// `state+0xF5BC` (HOME) and `state+0xF5F2` (AWAY) — verified by
    /// FUN_006A4020 which copies them out as
    /// `fixture[+0x49] = state[+0xF5BC]; fixture[+0x4A] = state[+0xF5F2]`
    /// (see decompiled/gameplay_lifts_match/0x006a4020.c lines 223-224).
    /// The old comment claiming these live at `+0x475F/+0x4760` was
    /// wrong — those bytes are a scorer-slot sentinel (init to 0xFF,
    /// reset to 0xFFFF) and never hold the actual score. This field's
    /// increment is driven by the goal-event branch in
    /// [`match_events_generate`] (etype == 2), which mirrors the exe's
    /// path: shot-outcome resolver → 0x2153 goal event → +0xF5BC++.
    pub score_home: u8,
    pub score_away: u8,
    /// `+0x4782..+0x4789` — shots + shots-on-target per side.
    pub shots_home: u8,
    pub shots_away: u8,
    pub shots_on_home: u8,
    pub shots_on_away: u8,
    /// `+0x4796` — HOME per-side block.
    pub home_block: MatchSideBlock,
    /// `+0x6A6E` — AWAY per-side block.
    pub away_block: MatchSideBlock,
    /// `+0x478E` — match-event queue head; exe alloc 14 bytes.
    pub event_queue: Vec<PreMatchEvent>,
    /// `+0x8ED4` — next-event countdown seed (exe = `0x01EF` = 495).
    pub next_event_countdown: i16,
    /// `+0x8ED8` — secondary countdown seed (exe = `0x00B4` = 180).
    pub secondary_countdown: i16,
    /// `+0x8EAE`/`+0x8EB6` — kick-off side chosen by RNG.
    pub kickoff_side: u8,
    /// `+0x8EBA` — abort flag (set by `FUN_006CEE80` if match aborted).
    pub abort: bool,
    /// `+0x8EBB` — `param_4` manual-formation override.
    pub manual_formation: u8,
    /// `+0xF628..+0xF633` — form baselines per side.
    pub form_baselines_home: FormBaselines,
    pub form_baselines_away: FormBaselines,
    /// `+0x91D2..+0x91DD` — pitch modifiers.
    pub pitch_modifiers: PitchModifiers,
    /// `+0xF5CD` — HOME first-choice GK slot index.
    pub home_gk_slot: i8,
    /// `+0xF603` — AWAY first-choice GK slot index.
    pub away_gk_slot: i8,
    /// `+0xF638` — extra-time length in minutes (30 or 15), or `-1`.
    pub extra_time_len: i8,
    /// `+0xF639/+0xF63A` — abandoned/replay.
    pub abandoned: bool,
    pub replay_flag: bool,

    // ----- Runtime fields the tick loop uses -----
    /// Current minute (0..90; 91..105 = ET; 121+ = pens). Exe: `M+0x8ED0`.
    pub minute: u16,
    /// Setup completed?
    pub setup_done: bool,
    /// Home / away red-card counts (`+0x477A..+0x477D`).
    pub reds_home: u8,
    pub reds_away: u8,
    /// Half-phase counter — exe `M+0x8EB3`. 6→0. 6..=4 = 1st half,
    /// 3..=1 = 2nd half, 0 = ended.
    pub phase: u8,
    /// In-play state — exe `M+0x8EB2`. 0=dead, 8=pre-kick, else live.
    pub in_play_state: u8,
    /// Persistent event log — exe `M+0x720` ring capped at 700.
    pub event_log: Vec<EventSlot>,
    /// Which side is in possession — exe `M+0x8EAE`.
    pub possession_side: u8,
}

impl MatchCtx {
    /// Fresh 63 KB match state, zeroed to exe defaults. Matches the
    /// field-clear block at the start of `FUN_0069D950` (lines 63-297).
    pub fn new() -> Box<Self> {
        Box::new(Self {
            ref_id: -1,
            linesman_id: -1,
            morale_home_init: 11,
            morale_away_init: 11,
            pitch_length: 110,
            pitch_width: 110,
            half_clock_home: 10000,
            half_clock_away: 10000,
            next_event_countdown: 0x01EF,
            secondary_countdown: 0x00B4,
            home_block: MatchSideBlock::new(),
            away_block: MatchSideBlock::new(),
            form_baselines_home: FormBaselines::default(),
            form_baselines_away: FormBaselines::default(),
            pitch_modifiers: PitchModifiers::default(),
            extra_time_len: -1,
            ..Default::default()
        })
    }
}

// ============================================================================
// Input structs required to drive the port.
// ============================================================================

/// Simplified team snapshot handed to the engine — the exe pulls this
/// from ~a dozen structs; we bundle only the fields the decoded logic
/// actually reads so the port stays honest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineTeamSnapshot {
    pub club_id: u32,
    pub reputation: u16,       // from `team+0x80`
    pub grudge_score: u8,      // computed by FUN_006BA1E0 (derby / grudge)
    pub players: Vec<EngineTeamPlayer>,
    /// Sum of position ratings for the 11 selected XI (`FUN_006c8930` per
    /// player). Populated by `snapshot_team_for_engine` when it picks the XI;
    /// consumed as the team-strength driver via `tactics::team_score` — the
    /// real formula-derived strength, replacing avg-CA. (kill #T)
    #[serde(default)]
    pub sum_position_ratings: i32,
    /// Player ids in the picked XI whose per-slot `position_rating` came out
    /// negative — i.e. their aptitude for the role they were forced into was
    /// below the 10-neutral point on `ATTR_CURVE`. Feeds the tactics gap #7
    /// per-player morale penalty (`MOOD_OUT_OF_POSITION`).
    #[serde(default)]
    pub out_of_position_ids: Vec<u32>,
    /// Team-wide tactic settings for THIS team in this fixture — the decoded
    /// [`crate::tactic_file::TeamSettings`] from the club's assigned tactic.
    /// Populated by `snapshot_team_for_engine` if the club has a tactic set
    /// in `club_tactics`; empty (all `Unset`) if not.
    ///
    /// Tactics gap #6 wire: the pre-match snapshot now carries the
    /// mentality / passing / marking / tackling / counter / offside /
    /// pressing settings into the engine. The per-tick decision code
    /// inside the token model can read these directly; the current
    /// shot-gate + possession engine still reads only the `sum_position_
    /// ratings` delta, but each new engine sub-step now has a place to
    /// look for these values instead of hardcoding neutrality.
    #[serde(default = "default_team_settings")]
    pub team_settings: crate::tactic_file::TeamSettings,
}

fn default_team_settings() -> crate::tactic_file::TeamSettings {
    crate::tactic_file::TeamSettings {
        passing:        crate::tactic_file::Passing::Unset,
        mentality:      crate::tactic_file::Mentality::Unset,
        counter_attack: false,
        offside_trap:   false,
        pressing:       crate::tactic_file::Pressing::Unset,
        marking:        crate::tactic_file::Marking::Unset,
        tackling:       crate::tactic_file::Tackling::Unset,
    }
}

/// Per-player snapshot the exe reads inside the pre-match injury pass
/// and the tick loop. Fields correspond 1:1 to the decoded offsets.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineTeamPlayer {
    pub player_id: u32,
    pub is_not_injured: bool,     // player +0x22
    pub position: u8,              // player +0x3D (12 = GK)
    pub jumping_heading: i8,       // player +0x57
    pub aggression: i8,            // player +0x5A
    pub bravery: i8,               // player +0x5B
    pub dirtiness: i8,             // player +0x5D
    pub current_ability: u16,      // attributes[+0x0B]
    pub age: u8,                   // attributes[+0x25]
    pub injury_proneness: u8,      // attributes[+0x3C]
    pub form: i8,                  // player[+0x18]
    pub is_first_choice_gk: bool,
    // Position specialities (attributes +0x0F/+0x11/+0x14/+0x15).
    pub speciality_a: u8,
    pub speciality_b: u8,
    pub position_natural: u8,
    pub position_learn: u8,
    /// Type10 attribute source for the token f32 fields decoded in
    /// reports/token_float_field_sources_decode.md — each populated at
    /// kickoff by `FUN_006d1a20`. Type10 offsets:
    /// heading +0x26, important_matches +0x27, dribbling +0x2E,
    /// decisions +0x31, throw_ins +0x40.
    #[serde(default)] pub heading: i8,
    #[serde(default)] pub important_matches: i8,
    #[serde(default)] pub dribbling: i8,
    #[serde(default)] pub decisions: i8,
    #[serde(default)] pub throw_ins: i8,
}

/// Grudge / recent-actions bitmask returned by `FUN_004D5A20(player, opp_team)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GrudgeMask {
    pub bits: u32,   // player-vs-team relation record field +0x45
}

impl GrudgeMask {
    /// Bit `0x40` — "recent aggressive foul" flag consulted at 934-943.
    pub fn has_recent_foul(self) -> bool { self.bits & 0x40 != 0 }
    /// Bit `0x2000000` — "serious grudge" flag.
    pub fn has_serious_grudge(self) -> bool { self.bits & 0x02000000 != 0 }
    /// Mask `0x2F7A0B` — red-card-worthy foul bits.
    pub fn red_card_worthy(self) -> u32 { self.bits & 0x002F7A0B }
}

// ============================================================================
// match_eng_setup — port of FUN_0069D950 §7 (pre-match injury pass).
//
// The bulk of FUN_0069D950 is struct-zeroing already covered by
// MatchCtx::new(). The gameplay-relevant part is the pre-kickoff injury
// / foul / illness / suspension roll, which is fully decoded and portable.
// ============================================================================

/// CA thresholds from the pre-match injury pass (exe magic numbers).
const CA_THRESHOLD_A: u16 = 0x0CB2; // 3250
const CA_THRESHOLD_B: u16 = 0x0EA6; // 3750
const CA_THRESHOLD_C: u16 = 0x109A; // 4250
const CA_THRESHOLD_D: u16 = 0x186A; // 6250
const CA_THRESHOLD_E: u16 = 0x1D4C; // 7500

/// Pre-match event types (into `PreMatchEvent.event_type`).
pub const EVT_MINOR_INJURY:  u16 = 4;
pub const EVT_FORM_INJURY:   u16 = 5;
pub const EVT_SUSPENSION:    u16 = 6;
pub const EVT_SERIOUS_FOUL:  u16 = 0x13;
pub const EVT_WITHDRAWN_ILL: u16 = 0x1E;

/// Run the pre-kickoff pass — exe FUN_0069D950 §7, lines 782-1122.
///
/// For each of the 40 slots (2 sides × 20 slots) we roll:
///   1. Grudge injury (recent-foul bit or serious-grudge bit set)
///   2. Foul / yellow card (aggregate aggro score vs RNG(30))
///   3. Pre-match illness (age ≥ 16, CA ≥ 3750, bravery < 7, RNG gates)
///   4. Injury on low form
///   5. Injury above threshold
///   6. Suspension forgotten
pub fn run_pre_match_pass(
    ctx: &mut MatchCtx,
    home: &EngineTeamSnapshot,
    away: &EngineTeamSnapshot,
    rng: &mut MatchRng,
    grudge: impl Fn(u32, u32) -> GrudgeMask,
    league_avg_goals: Option<f32>,
) {
    // League-average goals per match. 6.8 was the original speculative value
    // and produced absurd scorelines (6+ goals per match) — real Premier
    // League is 2.6-2.8. See simulate_season observations pre-tuning where
    // Everton had 216 GF in 36 games (6.0/game) via this fallback path.
    let avg = league_avg_goals.unwrap_or(2.8);

    for (side_id, (team, opp)) in [(0u8, (home, away)), (1u8, (away, home))].iter() {
        let side = *side_id;
        for (slot_idx, p) in team.players.iter().enumerate().take(20) {
            if !p.is_not_injured { continue; }

            let mask = grudge(p.player_id, opp.club_id);

            // --- (1) Grudge injury -----------------------------------
            if mask.has_recent_foul() || mask.has_serious_grudge() {
                ctx.event_queue.push(PreMatchEvent {
                    event_type: EVT_SERIOUS_FOUL,
                    opp_team_id: opp.club_id,
                    player_id: p.player_id,
                    flags: 0,
                    side,
                    is_first_choice_gk: p.is_first_choice_gk,
                });
                continue;
            }

            // --- (2) Foul / yellow ---------------------------------
            let foul_type = classify_foul_type(p, opp);
            let aggro = aggro_score(p, foul_type, mask, opp);
            if aggro > 5 && rng.range(30) < aggro as u32 {
                ctx.event_queue.push(PreMatchEvent {
                    event_type: EVT_SERIOUS_FOUL,
                    opp_team_id: opp.club_id,
                    player_id: p.player_id,
                    flags: 0x40,           // yellow-card marker
                    side,
                    is_first_choice_gk: p.is_first_choice_gk,
                });
                continue;
            }

            // --- (3) Illness before kick-off --------------------------
            if p.age >= 16 && p.current_ability >= CA_THRESHOLD_B
                && p.bravery < 7 && rng.range(15) as i8 > p.bravery
                && rng.range(p.current_ability.max(1) as u32)
                    < rng.range((opp.reputation.max(1) as u32 & 0xFF).max(1))
            {
                ctx.event_queue.push(PreMatchEvent {
                    event_type: EVT_WITHDRAWN_ILL,
                    opp_team_id: opp.club_id,
                    player_id: p.player_id,
                    flags: mask.bits,
                    side,
                    is_first_choice_gk: p.is_first_choice_gk,
                });
                continue;
            }

            // --- (4) Injury, low form --------------------------------
            let adj: i16 = match p.injury_proneness {
                _ if p.injury_proneness >= 15 => -2,
                _ if p.injury_proneness >= 10 => -1,
                _ => 1,
            };
            let form_val = (p.form as i16 + adj) as f32;
            if (p.form as i16 + adj) < 23 && p.current_ability >= CA_THRESHOLD_A
                && rng.hit(deriv_from_form(p.form, p.injury_proneness))
                && form_val < avg
            {
                ctx.event_queue.push(PreMatchEvent {
                    event_type: EVT_FORM_INJURY,
                    opp_team_id: opp.club_id,
                    player_id: p.player_id,
                    flags: mask.bits,
                    side,
                    is_first_choice_gk: p.is_first_choice_gk,
                });
                continue;
            }

            // --- (5) Injury above threshold (symmetric) -------------
            if p.current_ability >= CA_THRESHOLD_E
                && rng.hit(deriv_from_form(p.form, p.injury_proneness).saturating_mul(2))
                && (p.form as i16 + adj) as f32 > avg
            {
                ctx.event_queue.push(PreMatchEvent {
                    event_type: EVT_MINOR_INJURY,
                    opp_team_id: opp.club_id,
                    player_id: p.player_id,
                    flags: mask.bits,
                    side,
                    is_first_choice_gk: p.is_first_choice_gk,
                });
                continue;
            }

            // --- (6) Suspension forgotten ----------------------------
            let _ = slot_idx;
            if p.dirtiness > 0 && rng.hit(p.dirtiness.max(1) as u32)
                && p.injury_proneness > 15
            {
                ctx.event_queue.push(PreMatchEvent {
                    event_type: EVT_SUSPENSION,
                    opp_team_id: opp.club_id,
                    player_id: p.player_id,
                    flags: 0,
                    side,
                    is_first_choice_gk: p.is_first_choice_gk,
                });
            }
        }
    }
}

/// exe: computes foul type from position matchups. Ported as a small
/// integer classifier — decoded taxonomy: 6=violent, 2=late, 1=tactical,
/// 0=professional, -1=neutral. Uses only positions + dirtiness for now
/// (the exe pulls more context we don't fully own yet).
fn classify_foul_type(p: &EngineTeamPlayer, opp: &EngineTeamSnapshot) -> i8 {
    let _ = opp;
    if p.dirtiness >= 15                    { 6 }        // violent
    else if p.aggression >= 15              { 2 }        // late
    else if p.position <= 4                 { 1 }        // tactical (defender)
    else if p.position >= 10                { 0 }        // professional (attacker)
    else                                    { -1 }
}

/// Composite "aggro score" from the exe's line 819-929 branch.
fn aggro_score(p: &EngineTeamPlayer, foul_type: i8, mask: GrudgeMask, opp: &EngineTeamSnapshot) -> i8 {
    let ca_pts   = (p.current_ability / 1000) as i16;
    let head_pts = p.jumping_heading as i16;
    let agg_pts  = p.aggression as i16;
    let bra_pts  = p.bravery as i16;
    let dir_pts  = p.dirtiness as i16;
    let foul_pts = foul_type as i16 * 3;
    let grudge_pts = if mask.has_serious_grudge() { 4 } else { 0 };
    let rep_pts  = if opp.reputation > 1500 { 2 } else { 0 };
    let raw = ca_pts / 2 + head_pts / 3 + agg_pts + dir_pts - bra_pts
            + foul_pts + grudge_pts + rep_pts;
    raw.clamp(0, 30) as i8
}

fn deriv_from_form(form: i8, proneness: u8) -> u32 {
    // Rough proxy for the exe's `RNG(derived)` denominator: better form
    // and lower proneness → larger denom → fewer injuries fire.
    let base = 40i32 + (form.max(0) as i32) * 5 - (proneness as i32) * 2;
    base.max(4) as u32
}

// ============================================================================
// match_day_build — port of FUN_00699640.
// ============================================================================

/// Raw fixture row as the exe's iterator (`FUN_00672400`) returns them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawFixture {
    pub league_id: i32,       // +0
    pub group_id: i32,         // +4
    pub home_club_id: u32,     // +0x1C
    pub away_club_id: u32,     // +0x20
    pub comp_id: i32,          // +0x28
    /// `+0x43` — inclusion flag. Exe accepts `>= 0x80 && != 0xFE`.
    pub include_flag: u8,
    pub cup_round: i32,        // +0x18
    pub comp_type_flag: i8,    // +0x4B
}

/// Per-competition day-schedule header from `FUN_00598D20(day, mode, idx)`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DaySchedule {
    pub count: i16,
    pub raw_fixtures: Vec<RawFixture>,
}

/// Port of `match_day_build` (FUN_00699640). Assembles today's playable
/// fixtures into `MatchDayCtx`, grouping by competition and group.
///
/// The exe iterates `DAT_00B4C600` competitions and pulls each schedule
/// via `FUN_00598D20`; we accept the schedules as an argument so callers
/// (the tick driver) can supply them from the world state.
pub fn match_day_build(
    ctx: &mut MatchDayCtx,
    mode: u8,
    schedules: Vec<(i32, DaySchedule)>,
) {
    *ctx = MatchDayCtx::default();
    ctx.mode = mode;
    ctx.sentinel = [-1, -1, -1, -1];
    ctx.build_flag = 1;

    for (comp_id, schedule) in schedules.into_iter() {
        if schedule.count <= 0 || schedule.raw_fixtures.is_empty() { continue; }

        let comp_idx = ctx.competitions.len();
        let first_fx_idx_before = ctx.fixtures.len() as i32;

        for raw in schedule.raw_fixtures.into_iter() {
            // Exe filter: only fixtures the master-list marks includable.
            if raw.include_flag < 0x80 || raw.include_flag == 0xFE { continue; }

            ctx.fixtures.push(DayFixture {
                league_id: raw.league_id,
                group_id: raw.group_id,
                home_club_id: raw.home_club_id,
                away_club_id: raw.away_club_id,
                comp_id: raw.comp_id,
                state: 0xFF,      // exe: +0x67 sentinel
                cup_round: raw.cup_round,
                comp_type_flag: raw.comp_type_flag,
                ..Default::default()
            });
        }

        let last_fx_idx = ctx.fixtures.len() as i32 - 1;
        if last_fx_idx < first_fx_idx_before { continue; }

        ctx.competitions.push(DayCompEntry {
            comp_id,
            first_group_idx: -1,
            last_group_idx: -1,
            first_fixture_idx: first_fx_idx_before,
            last_fixture_idx: last_fx_idx,
            flags: 0,
        });

        // Exe: qsort fixtures by group_id then build groups block.
        let range = &mut ctx.fixtures[first_fx_idx_before as usize ..= last_fx_idx as usize];
        range.sort_by(|a, b| a.group_id.cmp(&b.group_id));

        let group_start = ctx.groups.len() as i32;
        let mut cursor_group_id: Option<i32> = None;
        let mut cur_first: i32 = -1;
        for (i, fx) in ctx.fixtures[first_fx_idx_before as usize ..= last_fx_idx as usize].iter().enumerate() {
            let abs_idx = first_fx_idx_before + i as i32;
            let gid = fx.group_id;
            if Some(gid) != cursor_group_id {
                if let Some(_) = cursor_group_id {
                    // Close previous group.
                    let last_g = ctx.groups.last_mut().unwrap();
                    last_g.last_fixture_idx = abs_idx - 1;
                }
                ctx.groups.push(DayGroupEntry {
                    group_id: gid, first_fixture_idx: abs_idx,
                    last_fixture_idx: abs_idx, reserved_a: 0, reserved_b: 0,
                    scratch: vec![0; 64],
                });
                cursor_group_id = Some(gid);
                cur_first = abs_idx;
                let _ = cur_first;
            }
        }
        // Close final group of this comp.
        if let Some(last_g) = ctx.groups.last_mut() {
            last_g.last_fixture_idx = last_fx_idx;
        }
        ctx.competitions[comp_idx].first_group_idx = group_start;
        ctx.competitions[comp_idx].last_group_idx = ctx.groups.len() as i32 - 1;
    }
}

// ============================================================================
// match_events_generate — port of FUN_006BC8D0.
//
// The exe's event emitter maintains two ring buffers on the per-side
// match_state buffer: an "active" ring of 12 slots and a persistent
// "log" ring of up to 700 slots. Emits are reentrant (a goal event
// triggers a celebration event, a foul triggers a card event, etc.).
// Goal events (event_type byte==2) update the engine score box.
// ============================================================================

/// One entry in the event rings. Exe stride: 14 (0x0E) bytes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventSlot {
    pub event_code: u16,     // +0 — top bit is "special" flag
    pub scorer_sub_idx: u16, // +2 — index into DAT_00B4E300
    pub event_type: u8,      // +4 — 0=none, 1=on-target, 2=goal
    pub minute_hi: u8,       // +5
    pub side: u8,            // +6 — 0=home, 1=away
    pub player_id: u8,       // +7
    pub extra: u8,           // +8
    pub payload: u32,        // +9
    pub trailer: u8,         // +D — 0xFF sentinel
}

/// exe: event-type byte lookup at `DAT_00B4E2F8[code*0x12]`. Byte 0 tells
/// the emitter whether the event is on-target(1), goal(2), or
/// neither(0). We honour the decoded code ranges from
/// `reports/match_events.cpp` decode.
fn event_type_byte(code: u16) -> u8 {
    match code {
        // Goal event — the "goal announced" broadcast code. Ported from
        // the shot-resolver decode; falls into FUN_006BC8D0's default
        // switch branch which increments the score.
        0x2153 | 0x2020 | 0x2021 | 0x2022 => 2,       // goal
        0x1FF7 | 0x1FF8 | 0x1FF9 => 1,                // blocked/wide/saved = on-target
        c if (0x2022..=0x20CF).contains(&c) => 1,     // shot on target range
        c if c >= 0x1F40 && c < 0x21E5 => 0,          // neutral event
        _ => 0,
    }
}

/// Port of `FUN_006BC8D0` — pushes an event onto both rings. Recursive
/// for follow-ups (goal → celebration, foul → card).
pub fn match_events_generate(
    ctx: &mut MatchCtx,
    event_code: u16,
    minute_hi: u8,
    side: u8,
    player_id: u8,
    extra: u8,
    payload: u32,
    // The scorer's REAL staff id (0 when not applicable). The exe's own event
    // record keeps only the u8 lineup slot (`player_id`); we additionally carry
    // the resolved staff id so goals attribute to the actual player, not a slot
    // index — the fix for kill #B's goal accumulation.
    real_scorer_id: u32,
) {
    if event_code < 0x1F40 { return; }   // exe rejects below range
    let etype = event_type_byte(event_code);
    let slot = EventSlot {
        event_code: event_code & 0x7FFF,
        scorer_sub_idx: 0,
        event_type: etype,
        minute_hi,
        side,
        player_id,
        extra,
        payload,
        trailer: 0xFF,
    };
    // Score update — exe writes to +0x475F/+0x4760 inside this function's
    // goal branch. Ported directly.
    if etype == 2 {
        // The per-tick shot-decision gate (`shot_attempt_gate`, ported
        // from FUN_006F99C0 — see match_action_path_decode.md) now
        // restricts shot attempts to the actual ball carrier and folds in
        // the exe's threshold/quality/marking checks, so the shot-record
        // pipeline no longer over-fires. No artificial goal cap needed.
        if side == 0 {
            ctx.score_home = ctx.score_home.saturating_add(1);
        } else {
            ctx.score_away = ctx.score_away.saturating_add(1);
        }
        // Also record goal on the appropriate scorer list.
        let minute = (minute_hi as u16) * 10 + ctx.minute % 10;
        if side == 0 {
            ctx.home_goal_minutes.push(minute);
            ctx.home_scorer_ids.push(real_scorer_id);
        } else {
            ctx.away_goal_minutes.push(minute);
            ctx.away_scorer_ids.push(real_scorer_id);
        }
        // NOTE: token.goals (+0x0c) counter increment happens in the
        // token-model tick body when it reaches this event; here in the
        // match-report-only path we don't have a token pool to mutate.
        // The MotM selector's fallback to scorer-list counts covers the
        // token-model-not-firing case (see select_motm).
    }
    if etype == 1 {
        // Shots-on-target counter.
        if side == 0 { ctx.shots_on_home = ctx.shots_on_home.saturating_add(1); }
        else         { ctx.shots_on_away = ctx.shots_on_away.saturating_add(1); }
    }
    // Persistent log — bounded at 700 in the exe.
    if ctx.event_log.len() < 700 { ctx.event_log.push(slot); }

    // Reentrant follow-ups (from decode taxonomy):
    // - 0x2153 → probes 0x219F first (anim lookup), else 0x21A0
    // - 0x21BF / 0x21C0 = terminals, no further chain
    // - 0x21E3 = abandonment
    match event_code {
        0x2153 => {
            // (Exe checks FUN_006A88F0/006FCE70 anim lookup; we always fire the celebration.)
            match_events_generate(ctx, 0x219F, minute_hi, side, player_id, 0, 0, real_scorer_id);
        }
        0x1F46 | 0x1F47 => {
            match_events_generate(ctx, 0x21C0, minute_hi, side, player_id, 0, 0, real_scorer_id);
        }
        _ => {}
    }
}

// ============================================================================
// match_tick — port of FUN_0069F2F0.
//
// Runs the state machine for up to `budget` ticks; returns:
//   1 = budget exhausted, keep pumping
//   2 = match ended (full-time / abandoned)
//   4 = half-time transition (emit HT/FT event, caller may pause)
//   8 = hard-abort (extra-time cap exceeded)
// ============================================================================

/// Result of one call to [`match_tick`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickResult {
    Continue = 1,
    Ended = 2,
    HalfBreak = 4,
    Aborted = 8,
}

/// The exe pumps ~11 ticks per real-time frame; a "minute" completes when
/// the sub-tick counter crosses zero. Set to 90 to sim one full half in
/// one call.
pub const TICKS_PER_FRAME: i32 = 11;

// ---------------------------------------------------------------------------
// Shot outcome — direct port of FUN_006CFEF0 (633 bytes of decompiled C).
//
// Signature (recovered from Ghidra):
//   FUN_006CFEF0(shooter_stat, gk_stat, *outcome_state, param4,
//                shot_difficulty /*param5, 0..20*/, tackled_flag /*param6*/,
//                *xg_float /*param7*/, is_key_shooter /*param8*/)
//
// The `outcome_state` byte is in/out: caller passes 1 (regular shot) or 7
// (set-piece continuation). Function transforms it to:
//   1 → (unchanged) GOAL
//   2 → SAVED
//   3 → BLOCKED
//   4 → OFF-TARGET / WIDE
//   5 → hard miss / lost ball
//   7 → set-piece continuation
// ---------------------------------------------------------------------------

/// Shot outcome codes matching the exe's `*param_3` return values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShotOutcome {
    Goal = 1,       // outcome unchanged from 1 = goal
    Saved = 2,      // FUN_006B6C10 → event 0x1FF9
    Blocked = 3,    // FUN_006B6C10 → event 0x1FF7
    Wide = 4,       // FUN_006B6C10 → event 0x1FF8
    HardMiss = 5,   // fell-through fatigue penalty branch
    SetPiece = 7,   // set-piece continuation
}

/// State the exe reads/writes on the shooter token (+0x14 shot count,
/// +0x16 key-shooter flag, +0x24 pending-shot cursor, +0x25/+0x26
/// blocker/keeper ids, +0x35 fatigue).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShooterMutable {
    pub shot_count: u8,             // +0x14
    pub is_key_shooter: bool,       // +0x16
    pub pending_shot_cursor: u8,    // +0x24
    pub blocker_id: u8,             // +0x25
    pub keeper_id: u8,              // +0x26
    pub fatigue: i16,               // +0x35
    pub on_pitch: bool,             // +0x19 >= 0
    pub side: u8,                   // +0x27
    /// Zone x (0..8). Exe: +0x102. Fed to FUN_006DB520 in-box test.
    pub zone_x: i8,
    /// Zone y (0..11). Exe: +0x103. Fed to FUN_006DB520 in-box test.
    pub zone_y: i8,
    /// Stamina short (+0x29). Fed into shot-damage formula.
    /// VERIFIED 006d63f0.c:250.
    pub stamina_short: i16,
    /// Pass-bias short (+0x19C), clamped [-200, +200]. Written by the
    /// __ftol() expression in `006d63f0.c:208-221` and `:878-887`.
    pub pass_bias: i16,
    /// Per-match rating milli-accumulator (+0x35). VERIFIED per Kill
    /// #B-slice2 decode; mirrors MatchToken.rating_milli. Deltas here
    /// track only VERIFIED-exact matches to the exe's rating event
    /// table (`reports/per_match_ratings_decode.md`); the port's
    /// `fatigue` writes are a parallel mislabelled accumulator kept
    /// for compile safety while the physics reads are still wired to
    /// it.
    pub rating_milli: i16,
}

/// Port of `FUN_006DB520(token, side)` — the "is this token inside the
/// shooting box" predicate. VERIFIED from decompiled/006db520.c:2-19:
///   * side == 1 → box iff zone_x in 2..=6 AND zone_y in 10..=11
///   * side == 0 → box iff zone_x in 2..=6 AND zone_y in 0..=1
///
/// Returns `true` when the token is in the shooting box (the exe returns 1).
/// Used as a gate in the tackled-branch shot-outcome dispatch: the exe only
/// writes block/save outcomes when this is true.
pub fn shot_in_box(zone_x: i8, zone_y: i8, target_goal_side: u8) -> bool {
    if !(2..=6).contains(&zone_x) { return false; }
    if target_goal_side == 1 { (10..=11).contains(&zone_y) }
    else                     { (0..=1).contains(&zone_y) }
}

/// Per-side shot-attempt counter stored on the engine at
/// `state+0xF5C9 + side*0x36`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SideShotCounters {
    pub attempts: u8,
}

/// Port of `FUN_006CFEF0` (0x006cfef0.c).
///
/// Returns `(outcome, xg_float)`. Mutates the shooter's fatigue, shot
/// counter, and pending-shot cursor exactly like the exe.
pub fn shot_outcome_resolver(
    shooter: &mut ShooterMutable,
    gk_shot_count: &mut u8,     // param_2 +0x15
    shot_counters: &mut SideShotCounters,
    engine_phase: u8,            // state +0x8EB3
    initial_state: u8,           // *param_3
    param_4: u8,                 // written to +0x25/+0x26
    shot_difficulty: u8,         // param_5, 0..20
    tackled_flag: u8,            // param_6
    is_key_shooter: bool,        // param_8
    rng: &mut MatchRng,
) -> (ShotOutcome, f32) {
    // Default xG = 1.5 (0x3FC00000).
    let mut xg = 1.5f32;
    let mut outcome = initial_state;

    // Line 14-17: off-pitch check.
    if !shooter.on_pitch {
        return (map_outcome(outcome), 10.0);   // 0x41200000
    }

    // Line 18-22: set-piece continuation.
    if initial_state == 7 {
        xg = 1.25;                            // 0x3FA00000
        shot_counters.attempts = shot_counters.attempts.saturating_add(1);
    } else if initial_state == 1 {
        // Line 27-30: gate on engine phase — must be live.
        if engine_phase != 0 {
            return (map_outcome(outcome), xg);
        }
        shot_counters.attempts = shot_counters.attempts.saturating_add(1);

        if tackled_flag == 0 {
            // ---- Not-tackled branch (lines 34-70) ----
            let r1 = rng.range(0x0F) as i32;
            if r1 + 7 < shot_difficulty as i32 {
                // OFF-TARGET / WIDE branch (line 35-47).
                shooter.fatigue = shooter.fatigue.saturating_sub(1000);
                // VERIFIED rating delta from FUN_006CFEF0:36 —
                // 'miss (sitter)' — see rating_delta::SITTER_MISS.
                shooter.rating_milli = shooter.rating_milli
                    .saturating_add(rating_delta::SITTER_MISS);
                outcome = 4;                  // WIDE
                xg = 5.0;                     // 0x40A00000
                if shooter.pending_shot_cursor == 0 {
                    shooter.blocker_id = param_4;
                } else {
                    shooter.keeper_id = param_4;
                }
                if is_key_shooter {
                    // The exe writes +0x16 = 1; we mirror that.
                    shooter.is_key_shooter = true;
                }
            } else {
                // Line 50-69: try SAVE/BLOCK.
                let r2 = rng.range(6) as u8;
                if r2 < shot_difficulty {
                    // The __ftol() in the exe converts a computed float to
                    // short and adds to fatigue. We approximate with a
                    // small penalty proportional to shot difficulty.
                    let fatigue_delta = -((shot_difficulty / 2) as i16 + 3);
                    shooter.fatigue = shooter.fatigue.saturating_add(fatigue_delta);

                    if shooter.pending_shot_cursor == 0 {
                        shooter.pending_shot_cursor = 1;
                        outcome = 2;          // SAVED
                        xg = 2.0;             // 0x40000000
                        shooter.blocker_id = param_4;
                    } else {
                        shooter.pending_shot_cursor = shooter.pending_shot_cursor
                                                          .saturating_add(1);
                        outcome = 3;          // BLOCKED
                        xg = 3.5;             // 0x40600000
                        shooter.keeper_id = param_4;
                    }
                } else {
                    // Line 68: outcome stays 1 = GOAL. Fatigue penalty -0x19 (25).
                    shooter.fatigue = shooter.fatigue.saturating_sub(0x19);
                    // VERIFIED — small penalty on penalty/weak attempt AND
                    // big goal bonus fires here (only path that stays outcome=1).
                    // FUN_006CFEF0:68 (weak-attempt) + FUN_006D63F0:934 (goal +100).
                    shooter.rating_milli = shooter.rating_milli
                        .saturating_add(rating_delta::PENALTY_WEAK_ATTEMPT)
                        .saturating_add(rating_delta::GOAL_SCORED);
                }
            }
        } else {
            // ---- Tackled branch (lines 72-105) ----
            let r = rng.range(3000);
            if r < 0x178 {                   // r < 376
                if r < 0x33 {                // r < 51 → possible skip via FUN_006DB520
                    // The exe gates block/save writes on FUN_006DB520
                    // (VERIFIED at decompiled/006db520.c:2-19 — port at
                    // `shot_in_box()`). Only take the writes when the
                    // shooter is inside the opponent's shooting box; the
                    // exe's target_goal_side = 1 - shooter.side.
                    let target = 1u8.wrapping_sub(shooter.side);
                    if !shot_in_box(shooter.zone_x, shooter.zone_y, target) {
                        // No clean touch — skip the outcome writes; the
                        // rest of the resolver still bumps counters below.
                        outcome = 0;
                        // Fall out of the shot-write sub-block without
                        // writing block/save/pending-cursor changes.
                        // Bump counter and return early per exe LAB.
                        shooter.shot_count = shooter.shot_count.saturating_add(1);
                        *gk_shot_count = gk_shot_count.saturating_add(1);
                        return (map_outcome(outcome), xg);
                    }
                }
                let fatigue_delta = -((shot_difficulty / 2) as i16 + 3);
                shooter.fatigue = shooter.fatigue.saturating_add(fatigue_delta);
                if shooter.pending_shot_cursor == 0 {
                    shooter.pending_shot_cursor = 1;
                    outcome = 2;              // SAVED
                    xg = 2.0;
                    shooter.blocker_id = param_4;
                } else {
                    shooter.pending_shot_cursor = shooter.pending_shot_cursor
                                                      .saturating_add(1);
                    outcome = 3;              // BLOCKED
                    xg = 3.5;
                    shooter.keeper_id = param_4;
                }
            } else {
                // Hard-miss branch (line 96-104).
                shooter.fatigue = shooter.fatigue.saturating_sub(0x2EE);
                // VERIFIED rating delta from FUN_006CFEF0:96 —
                // 'miss (bad chance)' — see rating_delta::BAD_CHANCE_TAKEN.
                shooter.rating_milli = shooter.rating_milli
                    .saturating_add(rating_delta::BAD_CHANCE_TAKEN);
                outcome = 5;
                xg = 5.0;
                if shooter.pending_shot_cursor == 0 {
                    shooter.blocker_id = 9;
                } else {
                    shooter.keeper_id = 9;
                }
            }
        }
    }

    // LAB_006D014E: bump counters.
    shooter.shot_count = shooter.shot_count.saturating_add(1);
    *gk_shot_count = gk_shot_count.saturating_add(1);

    (map_outcome(outcome), xg)
}

fn map_outcome(byte: u8) -> ShotOutcome {
    match byte {
        1 => ShotOutcome::Goal,
        2 => ShotOutcome::Saved,
        3 => ShotOutcome::Blocked,
        4 => ShotOutcome::Wide,
        5 => ShotOutcome::HardMiss,
        7 => ShotOutcome::SetPiece,
        _ => ShotOutcome::Wide,
    }
}

/// Port of FUN_0069F2F0. Advances the match state machine.
///
/// Model of the exe's internal state:
/// * `phase` (M+0x8EB3) starts at 6 and counts down: 6,5,4,3 = 1st half;
///   2,1 = 2nd half; 0 = ended.
/// * `minute` (M+0x8ED0) is incremented once per exhausted sub-tick.
/// * `sub_tick` (M+0x8ED2) is a small counter that gates minute advance.
/// * The `FUN_006A3240` arbitrator decides half-boundaries at minute 45/90.
///
/// This port produces goals via `match_events_generate` on subtype 0x11
/// action tokens (the exe's shot subtype) with an RNG roll biased by team
/// CA.
pub fn match_tick(
    ctx: &mut MatchCtx,
    rng: &mut MatchRng,
    home_ca_avg: u16,
    away_ca_avg: u16,
    mut budget: i32,
) -> TickResult {
    // Exe M[0] = abort/teardown-requested flag.
    if ctx.abort { return TickResult::Ended; }
    if !ctx.setup_done {
        ctx.setup_done = true;
        // exe: phase M+0x8EB3 starts at 6.
        ctx.phase = 6;
        ctx.in_play_state = 1;
    }

    loop {
        if budget <= 0 { return TickResult::Continue; }

        // Phase-transition worker (exe: switch on M[0x8EB3]-1 → FUN_006A4020).
        if ctx.phase > 0 && ctx.phase <= 6 {
            match ctx.phase {
                // Cases 3,4,5 → half-time worker.
                4..=6 if ctx.minute >= 45 => {
                    // Reached HT. Fire HT event and drop into second half.
                    match_events_generate(ctx, 0x2002, (ctx.minute / 10) as u8, 0, 0, 0, 0, 0);
                    ctx.phase = 3;
                    // Snap minute back to segment boundary (exe: sVar6/0x1E*0x1E).
                    ctx.minute = (ctx.minute / 30) * 30 + 45;
                    return TickResult::HalfBreak;
                }
                // Cases 0,1,2 → FT worker.
                1..=3 if ctx.minute >= 90 => {
                    // Extra-time check.
                    if ctx.extra_time_len > 0
                        && ctx.minute < (90 + ctx.extra_time_len as u16)
                    {
                        // Enter ET.
                    } else {
                        match_events_generate(ctx, 0x2003, (ctx.minute / 10) as u8, 0, 0, 0, 0, 0);
                        ctx.phase = 0;
                        return TickResult::Ended;
                    }
                }
                _ => {}
            }
        }

        budget -= 1;

        // Extra-time cap (exe: M[0xF638] * 11).
        if ctx.extra_time_len >= 0
            && ctx.minute > (ctx.extra_time_len as u16).saturating_mul(11)
        {
            ctx.reds_home = 0;  // exe sets M[0xF5BD]=-3 — force-abort marker.
            ctx.reds_away = 0;
            match_events_generate(ctx, 0x217B, (ctx.minute / 10) as u8, 0, 0, 0, 0, 0);
            return TickResult::Aborted;
        }

        // Advance one minute.
        ctx.minute += 1;

        // Live body — port of the exe's per-minute shot pipeline.
        //
        // In the exe this is: FUN_006F99C0 (shot-choice gate) fires with
        // probability driven by (2.5 - fatigue*0.0002) * teammates *
        // shootAttr; if it fires, FUN_006B4510(side, 5, filter) picks the
        // shooter; FUN_006B6C10 then calls FUN_006CFEF0 for the
        // save/goal/miss dice. Here we condense the choice gate to a
        // team-CA-driven roll (until per-token modelling lands) but we
        // use the REAL FUN_006CFEF0 outcome dice.
        for (side, ca) in [(0u8, home_ca_avg), (1u8, away_ca_avg)] {
            // Safety valve for this *condensed* engine only. This loop is a
            // simplified team-CA stand-in for the real per-token pipeline
            // (used as a fallback when `simulate_one_fixture_token_model`
            // returns zero shots — undersized squads, missing rosters,
            // etc.), and its `rng.range(0x28) < shoot_attr` gate is far
            // looser than the real `shot_attempt_gate` (port of
            // FUN_006F99C0) the token model now applies — the token model
            // no longer needs a goal cap, but this condensed path still
            // does. Stop offering a side new shot attempts once it's
            // banked a plausible number of goals.
            if (side == 0 && ctx.score_home >= 6) || (side == 1 && ctx.score_away >= 6) {
                continue;
            }
            // Shot-choice gate (port of FUN_006F99C0's `roll1 < shotThreshold`).
            // exe uses rand(0x32) or rand(0x1E) depending on tactical bit;
            // we split the difference at rand(0x28) = 40.
            let shoot_attr = (ca.min(200) / 10).max(1) as u32;
            if rng.range(0x28) < shoot_attr {
                // Shot fires.
                if side == 0 { ctx.shots_home = ctx.shots_home.saturating_add(1); }
                else         { ctx.shots_away = ctx.shots_away.saturating_add(1); }

                // Build shooter + counters + phase context for the real
                // dice roll.
                let opp_ca = if side == 0 { away_ca_avg } else { home_ca_avg };
                // Shot difficulty (param_5). The exe writes GOAL only when
                // `rand(6) >= diff`, so diff must be in [0..5] for goals
                // to fire — the exe's shot-record +0xB9 is populated from
                // FUN_006B4510's finishing/technique blend divided down.
                // Model: diff = 2 + (opp_ca - my_ca) / 40 clamped to [0..5].
                let ca_delta = opp_ca as i32 - ca as i32;
                let shot_difficulty = (2 + ca_delta / 40).clamp(0, 5) as u8;

                // Condensed engine has no per-token zone geometry; source
                // zone_x/y that MAKE shot_in_box true for the shooter's
                // target-goal so the FUN_006DB520 rare-skip branch fires
                // like the exe. Target-goal side = 1 - shooter.side.
                let (fx, fy) = if side == 0 { (4i8, 10i8) } else { (4i8, 1i8) };
                let mut shooter = ShooterMutable {
                    on_pitch: true,
                    side,
                    fatigue: 0,
                    shot_count: 0,
                    is_key_shooter: false,
                    pending_shot_cursor: 0,
                    blocker_id: 0,
                    keeper_id: 0,
                    zone_x: fx,
                    zone_y: fy,
                    stamina_short: 10_000,
                    pass_bias: 0,
                    rating_milli: 6400,   // FUN_006d08b0:84 init (6.4)
                };
                let mut gk_shot_count = 0u8;
                let mut counters = SideShotCounters::default();

                let (outcome, _xg) = shot_outcome_resolver(
                    &mut shooter,
                    &mut gk_shot_count,
                    &mut counters,
                    0,          // engine_phase (live)
                    1,          // initial_state = 1 (shot)
                    (rng.range(10) + 1) as u8, // param_4
                    shot_difficulty,
                    0,          // tackled_flag
                    rng.hit(8), // is_key_shooter (rare)
                    rng,
                );

                let (evt_code, is_on_target) = match outcome {
                    ShotOutcome::Goal    => (0x2153, true),   // GOAL
                    ShotOutcome::Saved   => (0x1FF9, true),   // saved
                    ShotOutcome::Blocked => (0x1FF7, true),   // blocked
                    ShotOutcome::Wide    => (0x1FF8, false),  // wide
                    ShotOutcome::HardMiss=> (0x1F74, false),  // hard miss
                    ShotOutcome::SetPiece=> (0x2004, false),
                };
                let _ = is_on_target; // shots-on tracked inside emitter
                // This condensed fallback models teams by average CA and has no
                // per-token shooter, so it cannot attribute the goal to a real
                // player — credit 0 (accumulation skips unattributed goals). The
                // token model (primary path) carries real scorer ids. (kill #B)
                match_events_generate(
                    ctx, evt_code, (ctx.minute / 10) as u8, side,
                    (rng.range(10) + 1) as u8, 0, 0,
                    0,
                );
            }
        }

        // Card / injury opportunity — exe rolls per token per minute.
        if rng.hit(80) {
            let side = if rng.hit(2) { 0 } else { 1 };
            // Rare event — emit a foul commentary code.
            match_events_generate(
                ctx, 0x1F74, (ctx.minute / 10) as u8, side,
                (rng.range(10) + 1) as u8, 0, 0,
                0,   // foul commentary — no scorer
            );
        }
    }
}

// ============================================================================
// Simulate one fixture end-to-end — the exe's outer loop condensed.
//
// This is what match_day_play does per fixture:
//   1. Alloc MatchCtx (`operator_new(0xF63B)` + FUN_0069D880)
//   2. FUN_0069D950 setup (pre-match injury pass)
//   3. Loop FUN_0069F2F0(-1) until state ∈ {2, 8}
//   4. FUN_00710C70 extract results
//   5. Writeback FUN_006BA380
// ============================================================================

/// The final output of simulating one fixture — matches the exe's
/// `MatchReport` shape (populated by FUN_00710C70).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExeMatchResult {
    pub home_score: u8,
    pub away_score: u8,
    pub home_goal_minutes: Vec<u16>,
    pub away_goal_minutes: Vec<u16>,
    pub home_scorer_ids: Vec<u32>,
    pub away_scorer_ids: Vec<u32>,
    pub home_shots: u8,
    pub away_shots: u8,
    pub home_shots_on: u8,
    pub away_shots_on: u8,
    pub event_log: Vec<EventSlot>,
    pub abandoned: bool,
    pub pre_match_events: Vec<PreMatchEvent>,
    /// Man-of-the-match player_id. Selected via [`select_motm`] — verified
    /// port of `FUN_006b69e0`. `None` when no tokens qualify (e.g. abandoned
    /// match). Written to the exe's match record at `+0x477a` on the real
    /// engine; here we return it up through the fixture result.
    #[serde(default)]
    pub motm_player_id: Option<u32>,
    /// Every XI player's finalized display rating (1..=10) paired with
    /// their staff_id. Fed by [`finalize_rating`] on the pitch-token pool
    /// at match end. Consumed downstream by
    /// [`crate::player_rating::PlayerRatingBook::record_match_rating`] to
    /// accumulate the season sum/count per the verified formula in
    /// `reports/rating_accumulator_writer.md`.
    #[serde(default)]
    pub per_player_ratings: Vec<(u32, i8)>,
    /// Injury events emitted this match: (player_id, severity_days).
    /// Rolled per XI player from `EngineTeamPlayer.injury_proneness` (real
    /// staff attribute already threaded through the snapshot). Consumed by
    /// the fixture-commit block to feed `InjuryBook::add_injury`. Was
    /// silent-zero before this wire (advance_day recovered nothing because
    /// no code was generating injuries).
    #[serde(default)]
    pub injury_events: Vec<(u32, u16)>,
}

/// Simulate one fixture. This condenses the exe's match_day_play inner
/// body: setup → tick pump → extract → return.
pub fn simulate_one_fixture(
    home: &EngineTeamSnapshot,
    away: &EngineTeamSnapshot,
    seed: u64,
    league_avg_goals: Option<f32>,
) -> ExeMatchResult {
    let mut ctx = *MatchCtx::new();
    ctx.home_reputation = home.reputation;
    ctx.away_reputation = away.reputation;
    let mut rng = MatchRng::new(seed);

    // Pre-match pass (FUN_0069D950 §7).
    run_pre_match_pass(&mut ctx, home, away, &mut rng, |_, _| GrudgeMask::default(),
                       league_avg_goals);

    let home_ca = avg_ca(home);
    let away_ca = avg_ca(away);

    // Tick pump loop (exe: match_day_play calls FUN_0069F2F0(-1) until 2/8).
    let mut safety = 200;  // matches never take more than ~200 pump-loops.
    loop {
        safety -= 1;
        if safety == 0 { break; }
        match match_tick(&mut ctx, &mut rng, home_ca, away_ca, TICKS_PER_FRAME) {
            TickResult::Continue | TickResult::HalfBreak => continue,
            TickResult::Ended => break,
            TickResult::Aborted => { ctx.abandoned = true; break; }
        }
    }

    ExeMatchResult {
        home_score: ctx.score_home,
        away_score: ctx.score_away,
        home_goal_minutes: ctx.home_goal_minutes.clone(),
        away_goal_minutes: ctx.away_goal_minutes.clone(),
        home_scorer_ids: ctx.home_scorer_ids.clone(),
        away_scorer_ids: ctx.away_scorer_ids.clone(),
        home_shots: ctx.shots_home,
        away_shots: ctx.shots_away,
        home_shots_on: ctx.shots_on_home,
        away_shots_on: ctx.shots_on_away,
        event_log: ctx.event_log.clone(),
        abandoned: ctx.abandoned,
        pre_match_events: ctx.event_queue.clone(),
        // Condensed engine has no per-token pool — fall back to top-scorer
        // as a MotM proxy (real MotM picker needs the token engine).
        motm_player_id: ctx.home_scorer_ids.iter()
            .chain(ctx.away_scorer_ids.iter())
            .next().copied(),
        // Condensed engine has no per-player token pool → no per-player
        // ratings to emit. Season accumulator only feeds from the
        // token-model engine (foreground fixtures).
        per_player_ratings: Vec::new(),
        // Roll injury events from injury_proneness — same generator both
        // engines use.
        injury_events: roll_injuries(home, away, &mut rng),
    }
}

fn avg_ca(team: &EngineTeamSnapshot) -> u16 {
    if team.players.is_empty() { return 100; }
    let sum: u32 = team.players.iter().map(|p| p.current_ability as u32).sum();
    (sum / team.players.len() as u32) as u16
}

// ============================================================================
// Token engine — port of the per-token action pipeline that lives inside
// the tick body.
//
// See `reports/match_action_path_decode.md` for the decode. Chain per tick:
//
//   for side in 0..2:  FUN_006B3A90(side)                    — refresh GK
//   for each active token T at M[0x4796+side*0x22D8]:
//       if T.subtype == 0xFFFF: FUN_006F99C0(T)              — decide action
//       FUN_006F5DE0(T)                                       — physics tick
//   for each token T with T.shot_queue_count > 0:
//       FUN_006B6C10(T)                                       — resolve shots
//                                                               via FUN_006CFEF0
//
// Fields on `MatchToken` mirror the exe's 0x1BE-byte slot layout at the
// offsets the decode has pinned. Fields not yet decoded live as opaque
// bytes we don't touch.
// ============================================================================

/// Per-player token stride in the exe: 0x1BE (446) bytes.
pub const TOKEN_STRIDE: usize = 0x1BE;
/// Tokens per side (20 = 11 starters + 5 subs + 4 unused).
pub const TOKENS_PER_SIDE: usize = 20;

/// One queued shot record — the exe stores up to 8 of these at
/// `token+0x1A4 + i*0xB` (11 bytes each).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueuedShot {
    /// Token index (0..20) of the shooter within the same side.
    pub shooter_slot: u8,        // +0xB1
    /// Token index of the opposing goalkeeper.
    pub keeper_slot: u8,         // +0xB5
    /// Shot accuracy / difficulty parameter (0..20).
    pub accuracy: u8,            // +0xB9
    /// Shot type code (0x25..0x32 = various open-play; 0x60/0x61 = set-piece).
    pub shot_type: u8,           // +0xBA
    /// Defender's side.
    pub defender_side: u8,       // +0xBB
}

/// Zone bits from `FUN_006A91D0` — per-formation-slot attribute word
/// looked up in `state+0x8EBC + side*4 → pool[slot*2]`.
///
/// Decoded meanings (from `FUN_006FCE10` + `FUN_006F99C0`):
///
/// * `0x004` = clear angle from close range
/// * `0x008` = open shooting lane
/// * `0x040` = pace bonus / counter-attack path
/// * `0x080` = supporter density
/// * `0x200` = "in attacking third with shot on"
/// * `0x800` = same-team stripe (used in pass scoring)
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneBits {
    pub bits: u16,
}

impl ZoneBits {
    pub fn clear_angle(self) -> bool     { self.bits & 0x004 != 0 }
    pub fn open_shot(self) -> bool       { self.bits & 0x008 != 0 }
    pub fn pace_bonus(self) -> bool      { self.bits & 0x040 != 0 }
    pub fn supporter_dense(self) -> bool { self.bits & 0x080 != 0 }
    pub fn in_shot_third(self) -> bool   { self.bits & 0x200 != 0 }
    /// exe's shot-subtype gate: 0x10 = pass, 0x12 = open-play shot, 0x14 = clear shot.
    pub fn shot_subtype(self) -> u8 {
        if self.in_shot_third() && self.clear_angle() { 0x14 }
        else if self.in_shot_third() && self.open_shot() { 0x12 }
        else { 0x10 }
    }
}

/// One player's token — the exe's 0x1BE-byte slot. Field offsets in
/// comments reference the exe layout. Fields we haven't decoded yet stay
/// out; we only model what the ported logic reads or writes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchToken {
    /// Side (0 home, 1 away). Exe: `+0x27`.
    pub side: u8,
    /// Position slot on-pitch (-1 = off, 0..10 = active). Exe: `+0x19`.
    pub position_slot: i8,
    /// Player CA / role rating byte. Exe: `+0x18`.
    pub role_ca: u8,
    /// Current zone x on pitch (0..8). Exe: `+0x102`.
    pub zone_x: i8,
    /// Current zone y on pitch (0..11). Exe: `+0x103`.
    pub zone_y: i8,
    /// Action subtype (0xFFFF = undecided; 0x68 dribble, 0x69 pass, 0x6A shoot,
    /// 0x76 hold, 0x100 GK-distribute, 0x105 receive-pass). Exe: `+0x198`.
    pub subtype: u16,
    /// Cumulative fatigue (grows +7 per commit, drops via completed actions).
    /// Exe: `+0x35`.
    pub fatigue: i16,
    /// Shooting/Finishing attribute (0..20). Exe: `+0x107`.
    pub shooting: u8,
    /// Technique component. Exe: `+0x109`.
    pub technique: u8,
    /// Dribbling / Tackling. Exe: `+0x10A`.
    pub dribbling: u8,
    /// Passing / short-ball strength. Exe: `+0x104`.
    pub passing: u8,
    /// Just-tackled cool-down. Exe: `+0x10D`.
    pub tackled_cooldown: u8,
    /// Teammate count usable this tick. Exe: `+0x101`.
    pub teammate_count: u8,
    /// Number of queued shots this tick. Exe: `+0x1A3`.
    pub shot_queue_count: u8,
    /// Queued shot records (max 8). Exe: `+0x1A4..` with stride 0xB.
    pub shot_queue: [QueuedShot; 8],
    /// Chosen pass target slot (this-side index). Exe: `+0x1AA`.
    pub pass_target_slot: Option<u8>,
    /// Kinetic-x velocity accumulator. Exe: `+0x4D`.
    pub kinetic_x: f32,
    /// Touched-this-tick flag. Exe: `+0x2B`.
    pub touched: bool,
    /// Player id (from `player_record` at `+0x6D`).
    pub player_id: u32,
    /// Club id (from `+0x69`).
    pub club_id: u32,
    /// True if this token is the side's first-choice GK.
    pub is_gk: bool,
    /// Zone bits computed by `FUN_006A91D0`. Cached per tick.
    pub zone_bits: ZoneBits,

    // ----- Shot-resolver mutable state (mirrors ShooterMutable fields) -----
    /// Shot count (+0x14).
    pub shot_count: u8,
    /// Key-shooter flag (+0x16).
    pub is_key_shooter: bool,
    /// Pending-shot cursor (+0x24).
    pub pending_shot_cursor: u8,
    /// Blocker id (+0x25).
    pub blocker_id: u8,
    /// Keeper id (+0x26).
    pub keeper_id: u8,

    // ----- Per-token exe fields the target picker reads -----
    /// Tactical zone bias (0/1). Exe: `+0x2C`. Set from formation slot.
    pub zone_bias: u8,
    /// Positional weight (scaled × 0.125). Exe: `+0x2D`. Set from role CA.
    pub positional_weight: u8,
    /// Aggression byte from player attributes. Exe: `+0xE1`.
    pub aggression: u8,
    /// Formation slot index this token occupies (0..10, 10 = GK). Used to
    /// look up per-slot zone attributes from the pool.
    pub formation_slot: u8,
    /// Stamina short (+0x29). The exe reads this as
    /// `stamina/10 - 30` in shot-damage RNG (VERIFIED 006d63f0.c:250).
    pub stamina_short: i16,
    /// Pass-bias short (+0x19C). Written via __ftol() FP expression
    /// (`006d63f0.c:208-221`, `:878-887`), clamped `[-200, +200]`. This is
    /// NOT the fatigue field the earlier port assumed — that assumption
    /// was mis-routed to +0x35 (which is actually the per-match rating
    /// milli-accumulator per Kill #B-slice2 decode).
    pub pass_bias: i16,
    /// Pass-target physique float (+0xAD). Read via `rand(30) < float(+0xAD)`
    /// double-threshold in pass-target scoring. VERIFIED 006a1940.c:173-183.
    pub pass_marker_float: f32,
    /// Carrier-marker physique float (+0xB1). Same threshold shape as
    /// +0xAD but in the carrier-marker loop. VERIFIED 006a1940.c:401-408.
    pub carrier_marker_float: f32,
    /// Per-match rating milli-accumulator (+0x35). Starts at 6400 (= 6.4
    /// display) via FUN_006d08b0:84. Grows/shrinks by the event delta
    /// table decoded in reports/per_match_ratings_decode.md
    /// (goals +75/+100, assists +9, saves +350/+600, concede -500, misses
    /// -375/-750/-1000, per-touch +7 etc.). Finalized to display byte at
    /// +0x1B via `FUN_006b3de0`: `display = clamp(round((acc+500)/1000), 1, 10)`.
    /// Kill #B-slice2 partial port. The earlier port confusingly called this
    /// `fatigue` — those writes were rating writes all along.
    pub rating_milli: i16,
    /// Finalized display rating (+0x1B), byte 1..10. Written at half-time /
    /// full-time / extra-time by [`finalize_ratings`] via the same formula.
    pub rating_final: i8,
    /// Key-pass counter (+0x03). Weight 50 in MotM composite (FUN_006b69e0:25-27).
    /// Incremented on the passer when a pass is completed to a live outfielder
    /// (receiver subtype not 0x34/0x37) that either advanced the ball or was a
    /// long ball. Cite: 006e7a60:235, 006e7a60:1292, 006f63f0:1593. VERIFIED.
    pub key_passes: u8,
    /// Successful attacking-action counter (+0x06). Weight 25 in MotM composite.
    /// Incremented when a duel/dribble is won AND actor is in the opposition
    /// penalty box (FUN_006db520) or positionally ahead of the last defender
    /// (FUN_006b2f70). Same event that awards +300 rating_milli. VERIFIED.
    /// Cite: 006d63f0:2269, 006dc600:1478, 006e0740:921, 006e7a60:759.
    pub take_ons_won: u8,
    /// Goals scored (+0x0c). Weight 250 (dominant term). VERIFIED.
    /// Cite: 006e7a60:1221, 006f0320:492. Bumped on scorer at goal-event time.
    pub goals: u8,
    /// Creative involvement (+0x10). Weight 125. VERIFIED per FUN_006b69e0:25-27.
    /// Exe uniformly bumps this for: assister on scored goal (006e7a60:1188),
    /// second-phase involvement (006e7a60:1247), passer on any completed pass
    /// (006e7a60:1286). Not a clean "assists" — a broader creative counter.
    pub assists_composite: u8,
}

impl Default for MatchToken {
    fn default() -> Self {
        Self {
            side: 0, position_slot: -1, role_ca: 10,
            zone_x: 4, zone_y: 5, subtype: 0xFFFF, fatigue: 0,
            shooting: 10, technique: 10, dribbling: 10, passing: 10,
            tackled_cooldown: 0, teammate_count: 10, shot_queue_count: 0,
            shot_queue: [QueuedShot::default(); 8], pass_target_slot: None,
            kinetic_x: 0.0, touched: false, player_id: 0, club_id: 0,
            is_gk: false, zone_bits: ZoneBits::default(),
            shot_count: 0, is_key_shooter: false, pending_shot_cursor: 0,
            blocker_id: 0, keeper_id: 0,
            zone_bias: 0, positional_weight: 10, aggression: 8, formation_slot: 0,
            // Item #3, #5, #8, #2 field additions.
            stamina_short: 10_000,     // full stamina; drops during play
            pass_bias: 0,              // clamped [-200, 200]
            pass_marker_float: 0.0,
            carrier_marker_float: 0.0,
            rating_milli: 6400,        // 6.4 display — FUN_006d08b0:84 init
            rating_final: 6,           // 6.4 rounds down to 6 initially
            key_passes: 0, take_ons_won: 0, goals: 0, assists_composite: 0,
        }
    }
}

/// Port of `FUN_006b3de0`:64-73 — finalize the per-match rating from the
/// milli-accumulator at `token+0x35` to the display byte at `token+0x1B`.
///
/// Formula (VERIFIED byte-for-byte from decompile):
///   `display = clamp( round((acc + 500) / 1000), 1, 10 )`
///
/// Called at each period boundary (half-time / full-time / extra-time) —
/// the exe walks 20 slots × 2 sides at each of the three period cases.
pub fn finalize_rating(rating_milli: i16) -> i8 {
    let acc = rating_milli as i32;
    let raw = (acc + 500) / 1000;   // integer round-toward-zero after +500 bias
    raw.clamp(1, 10) as i8
}

/// Per-match rating deltas — the full verified table from
/// `reports/per_match_rating_events_port_plan.md`. All values in
/// "milli-rating"; MatchToken.rating_milli starts at 6400 = 6.4 display.
/// Every constant is cited to a single cm0102.exe decompile site.
///
/// Prior partial port applied 4 sites inline (see individual usages in
/// this file). These constants unify all 25 verified per-event sites so
/// the event-dispatch code can apply them uniformly.
pub mod rating_delta {
    // ---- Positive events ----------------------------------------------
    /// Pass or shot commit. FUN_006F99C0:107,126,353,362,388,396 (6 sites).
    pub const PASS_OR_SHOT_COMMIT:      i16 =    7;
    /// Interior short pass / GK distribute / carrier micro-move.
    /// FUN_006FA740:426,537; FUN_006D63F0:1797.
    pub const INTERIOR_SHORT_PASS:      i16 =    4;
    /// Clean receive unopposed. FUN_006D63F0:901.
    pub const CLEAN_RECEIVE_UNOPPOSED:  i16 =    9;
    /// Shot-on-target animation fired. FUN_006F63F0:1421,1567.
    pub const SHOT_ON_TARGET_ANIM:      i16 =   25;
    /// Cross to different-side receiver completed. FUN_006D63F0:1803.
    pub const CROSS_TO_OTHER_SIDE:      i16 =   28;
    /// Notable long-range effort (sVar25 == 0xf branch). FUN_006D63F0:909.
    pub const NOTABLE_LONG_RANGE:       i16 =   75;
    /// Goal scored — big flat delta on the scorer. FUN_006D63F0:934.
    pub const GOAL_SCORED:              i16 =  100;

    // ---- Negative events ----------------------------------------------
    /// Ball lost to opponent challenge. FUN_006F5DE0:123.
    pub const CHALLENGE_LOST:           i16 =  -15;
    /// Weak hold in mid. FUN_006D63F0:2072.
    pub const WEAK_HOLD:                i16 =  -20;
    /// Receiver miscontrol. FUN_006D63F0:787.
    pub const RECEIVER_MISCONTROL:      i16 =  -25;
    /// Penalty / weak attempt. FUN_006CFEF0:68.
    pub const PENALTY_WEAK_ATTEMPT:     i16 =  -25;
    /// Unreachable target in current zone. FUN_006D63F0:803.
    pub const UNREACHABLE_IN_ZONE:      i16 =  -59;
    /// Unreachable in final third. FUN_006D63F0:800.
    pub const UNREACHABLE_FINAL_THIRD:  i16 =  -70;
    /// Own-half bad hold in wrong cell. FUN_006D63F0:2069.
    pub const OWN_HALF_BAD_HOLD:        i16 = -100;
    /// Panic clearance (+0x198 = 0x1f42 animation). FUN_006D63F0:816.
    pub const PANIC_CLEARANCE:          i16 = -150;
    /// Giveaway at LAB_006d7cdc. FUN_006D63F0:2079.
    pub const GIVEAWAY:                 i16 = -250;
    /// Block victim on cross intercept (3 sites).
    /// FUN_006D63F0:572; FUN_006A0550:107,357.
    pub const BLOCK_VICTIM:             i16 = -500;
    /// Bad chance taken (poor decision + outcome 5). FUN_006CFEF0:96.
    pub const BAD_CHANCE_TAKEN:         i16 = -750;
    /// Sitter miss (shot rolled +7 < param_5). FUN_006CFEF0:36.
    pub const SITTER_MISS:              i16 =-1000;
}

/// Roll per-XI injury events based on each player's injury_proneness
/// (0..20 scaled attribute already carried on EngineTeamPlayer). Rolled
/// once per match per side; not per-tick or per-tackle. Returns
/// `(player_id, days_remaining)` pairs the fixture-commit block feeds to
/// `InjuryBook::add_injury`.
///
/// Injury-proneness reads as the exe's per-player byte scaled 0..20; the
/// per-match injury probability is `injury_proneness / 400.0` (so a very
/// injury-prone player at 20/20 sees ~5% chance per match, roughly
/// matching the real Premier League per-match injury rate of ~4-6% for
/// glass-jaw players). Non-XI slots skipped.
///
/// Not a decode of a specific exe function — the exe's injury generator
/// lives in an FP-heavy path (Ghidra couldn't lift the odds table) so
/// this is a plausible-envelope generator using verified attribute data.
pub fn roll_injuries(home: &EngineTeamSnapshot, away: &EngineTeamSnapshot,
                     rng: &mut MatchRng) -> Vec<(u32, u16)> {
    let mut out = Vec::new();
    for team in [home, away] {
        for (i, p) in team.players.iter().enumerate().take(11) {
            let _ = i;
            if p.player_id == 0 || p.injury_proneness <= 0 { continue; }
            // Per-match roll: rand(400) < injury_proneness (0..20) →
            // gives injury_proneness/400 = up to 5% per match.
            let roll = rng.range(400) as i16;
            if roll >= p.injury_proneness as i16 { continue; }
            // Severity split (approximate distribution — 60% knock, 25% minor,
            // 12% moderate, 3% major). Career-ending left to a separate rare
            // path not yet ported.
            let severity_days: u16 = match rng.range(100) {
                0..=59 => crate::injury::InjurySeverity::Knock.recovery_days(),
                60..=84 => crate::injury::InjurySeverity::Minor.recovery_days(),
                85..=96 => crate::injury::InjurySeverity::Moderate.recovery_days(),
                _ => crate::injury::InjurySeverity::Major.recovery_days(),
            };
            out.push((p.player_id, severity_days));
        }
    }
    out
}

/// Shot tier bucket — VERIFIED port of `FUN_006f0320:79-115` and
/// `FUN_006f1a50:83-119` (both branches identical). Given the RNG-driven
/// shot-quality delta and a difficulty flag, returns a 1..=13 tier code:
///
///   1     — goal-worthy shot (top tier)
///   2, 4-5 — early miss / poor chance
///   7-8    — set-piece / header
///   8-11   — misses at various bands (bucket dispatch in home only)
///   12-13  — close chance (late clock-kill bracket)
///
/// See reports/away_shot_and_618410_decode.md.
pub fn shot_tier_bucket(rng_delta: i16, flag: i8, rng: &mut MatchRng) -> i8 {
    if rng_delta >= 0xf1 { return 1; }
    if rng_delta < 10 {
        return if flag == 0 { 2 } else { 4 - (rng.range(4) != 0) as i8 };
    }
    let d = 0xf0 - rng_delta;
    if d < 0x33 {
        let bonus = (flag != 0) as i8;
        if d < 0xb {
            if d < 7 { bonus + 12 } else { bonus + 10 }
        } else { bonus + 8 }
    } else if flag == 0 { 5 } else { 7 - (rng.range(4) != 0) as i8 }
}

/// Goal-scored rating + fatigue bump on the scorer. VERIFIED port
/// (both FUN_006f0320:466-468 AND FUN_006f1a50:434-436 identical):
///   rating_milli += 140 (0x8c)
///   fatigue      += rand(3) + 1
///
/// This is the SCORER's bump; separate from the +100 GOAL_SCORED delta
/// in `rating_delta` (that constant is the FUN_006d63f0:934 fire, this
/// is FUN_006f0320:466 in the goal-branch tail).
pub fn apply_scorer_goal_bump(rating_milli: &mut i16, fatigue: &mut i16,
                              rng: &mut MatchRng) {
    *rating_milli = rating_milli.wrapping_add(0x8c);
    *fatigue      = fatigue.wrapping_add(rng.range(3) as i16 + 1);
}

/// Man-of-the-Match selector — VERIFIED port of `FUN_006b69e0`
/// (`006b69e0.c:19-40`). Walks both teams' tokens, computes a composite
/// score, and returns the winning player_id. Called at match end
/// (before the finalize_rating pass in the exe's sequence).
///
/// Composite formula from `006b69e0.c:25-27`:
///   `score = ( ((b[+0x10] + b[+0x0c]*2) * 5) + b[+3]*2 + b[+6] ) * 25
///          + short[+0x35] // rating_milli`
///
/// Expanding: `goals * 250 + assists * 125 + b_03 * 50 + b_06 * 25 + rating_milli`.
///
/// Our MatchToken doesn't currently carry the four per-token in-match
/// counters at exe offsets +0x03/+0x06/+0x0c/+0x10 as named fields
/// (semantic OPEN GAP per `reports/motm_selector_hunt.md` — likely goals
/// at +0x0c, assists at +0x10; +0x03/+0x06 unknown). Until those are
/// promoted onto MatchToken, this port uses `rating_milli` alone plus
/// scorer-list goal counts as a proxy for +0x0c (which the exe treats
/// as the dominant term at weight 250).
///
/// Tie-break matches exe order: first-scanned wins on strict-greater
/// (team 0 preferred over team 1; low-slot preferred).
pub fn select_motm(engine: &TokenEngine,
                    home_scorer_ids: &[u32],
                    away_scorer_ids: &[u32]) -> Option<u32> {
    // Fold scorer-list goals back into token.goals in case per-tick
    // wiring didn't count them (safety belt — token.goals is the primary).
    let mut goal_count = std::collections::HashMap::<u32, u32>::new();
    for id in home_scorer_ids.iter().chain(away_scorer_ids.iter()) {
        *goal_count.entry(*id).or_insert(0) += 1;
    }
    let mut best_score: i64 = i64::MIN;
    let mut best_pid: Option<u32> = None;
    for team in 0..2 {
        for tok in engine.tokens[team].iter() {
            // Skip slot-valid predicate (`006b69e0.c:24` — both being false skips).
            // We approximate with player_id != 0 (real ports of the +0x19/+0x20
            // slot-valid bytes would replace this).
            if tok.player_id == 0 { continue; }
            // Prefer token.goals (populated per-tick); fall back to
            // scorer-list count so the composite is never blind to goals
            // even if the per-tick wiring missed a scorer.
            let goals = (tok.goals as i64).max(
                *goal_count.get(&tok.player_id).unwrap_or(&0) as i64
            );
            // VERIFIED composite per FUN_006b69e0:25-27. All four token
            // bytes now real (see reports/motm_composite_token_bytes.md).
            let composite = goals * 250
                          + (tok.assists_composite as i64) * 125
                          + (tok.key_passes as i64)   * 50
                          + (tok.take_ons_won as i64) * 25
                          + tok.rating_milli as i64;
            // Strict-greater, first-wins on tie (exe scan order: team0, low-slot).
            if best_pid.is_none() || composite > best_score {
                best_score = composite;
                best_pid = Some(tok.player_id);
            }
        }
    }
    best_pid
}

/// Ball-carrier assist bonus on cross/pass frame.
/// EXE: `*(short*)(carrier+0x35) += (short)((stamina*0x55555556)>>32) + 0x113 - sign`,
/// which is `stamina/3 + 275` for `stamina >= 0`. FUN_006F63F0:1581-1583.
#[inline]
pub fn assist_bonus_milli(stamina: i16) -> i16 {
    (stamina / 3).saturating_add(0x113)
}

/// Mentality-driven shot-outcome value scaler — VERIFIED port of
/// FUN_006AE160:107-129 (was OPEN GAP in
/// `reports/match_engine_tactic_reads_decode.md` — three `_DAT_*` FP
/// constants now lifted via cm-lift).
///
/// Applies ONLY when the attacker's role bit 0x200 is set. Reads
/// `pitch+0x9766+side*0x18E3` bits 0x20 (Normal) / 0x40 (Attacking).
/// Returns the multiplier fed into the shot-outcome value calculation:
///
///   bit 0x20 set (Normal)      → 0.5
///   bit 0x40 set, 0x20 clear (Attacking) → 4.0
///   neither (Defensive)        → 2.0
///
/// Attacking mentality yields 8× the outcome value of Normal — the
/// "counter-attack shot is worth more" bias explicitly encoded.
#[inline]
pub fn mentality_outcome_scaler(team_tactic_word: u32) -> f32 {
    if team_tactic_word & 0x20 != 0 {
        crate::exe_constants::DAT_00956F10_F32 // 0.5 — Normal
    } else if team_tactic_word & 0x40 != 0 {
        crate::exe_constants::DAT_0095AEF0_F32 // 4.0 — Attacking
    } else {
        crate::exe_constants::DAT_009569A8_F32 // 2.0 — Defensive
    }
}

/// GK "made-save" rating micro-boost — VERIFIED port of FUN_006D63F0
/// lines 2245/2249. Exact for keepers (non-GK slots take an FP-unlifted
/// branch at :2256 that Ghidra dropped, so this fn covers ONLY the GK
/// path; that's the correct scope for a "save bonus").
///
/// See `reports/shot_damage_and_custom_formation_decode.md` §1. Corrects
/// the earlier hallucinated `save_bonus_milli(shot_power_milli, gk_bias)`
/// signature — the exe reads neither parameter.
///
/// `conceded` — set at 006d63f0.c:2131 when a low-quality shot penetrated
/// the keeper's stat gate; `outcome_flags` — result of `FUN_006a91d0(token) & 0b110`.
#[inline]
pub fn gk_save_rating_delta_milli(conceded: bool, outcome_flags: u8) -> i16 {
    let base: i16 = if conceded { 120 } else { 200 };
    if (outcome_flags & 0b110) == 0 { base * 2 } else { base }
}

/// Pitch dimensions matching the exe's grid: 12 rows × 9 columns.
pub const PITCH_ROWS: usize = 12;
pub const PITCH_COLS: usize = 9;

/// One pitch-grid cell — the exe's 0x5A-byte record at
/// `pitch + 0x215E + (y + x*12)*0x5A`. Fields: 4 pointer slots at
/// `+0..+0x2C` (2 per side), byte at `+0x58` = defender count.
///
/// In our port we hold per-side occupant slot indices instead of raw
/// pointers. The `+0x58` byte is derived as `occupants[opp_side].len()`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PitchCell {
    pub home_occupants: Vec<u8>,     // 0..3 typically
    pub away_occupants: Vec<u8>,
}

impl PitchCell {
    pub fn count_for_side(&self, side: u8) -> u8 {
        if side == 0 { self.home_occupants.len() as u8 }
        else         { self.away_occupants.len() as u8 }
    }
    pub fn count_opposing(&self, side: u8) -> u8 {
        if side == 0 { self.away_occupants.len() as u8 }
        else         { self.home_occupants.len() as u8 }
    }
}

/// The full pitch grid — 12 × 9 cells. Rebuilt each tick from token
/// positions, ported from the exe's `pitch + 0x215E` block that its
/// physics engine populates continuously.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PitchGrid {
    /// Row-major: `cells[y][x]`.
    pub cells: Vec<Vec<PitchCell>>,
}

impl Default for PitchGrid {
    fn default() -> Self {
        Self { cells: (0..PITCH_ROWS)
            .map(|_| (0..PITCH_COLS).map(|_| PitchCell::default()).collect())
            .collect() }
    }
}

impl PitchGrid {
    /// Rebuild the entire grid from token positions. O(40) — 20 per side.
    /// Port of the physics-engine's per-tick cell allocator.
    pub fn rebuild(engine: &TokenEngine) -> Self {
        let mut grid = Self::default();
        for side_idx in 0..2 {
            for t in &engine.tokens[side_idx] {
                if t.position_slot < 0 { continue; }
                let x = t.zone_x;
                let y = t.zone_y;
                if x < 0 || x as usize >= PITCH_COLS { continue; }
                if y < 0 || y as usize >= PITCH_ROWS { continue; }
                let cell = &mut grid.cells[y as usize][x as usize];
                if side_idx == 0 { cell.home_occupants.push(t.position_slot as u8); }
                else             { cell.away_occupants.push(t.position_slot as u8); }
            }
        }
        grid
    }

    /// Defender count at cell for `side` — exe's `pitch+0x215E+...+0x58`
    /// read. Returns opposing occupant count within one cell of (x, y).
    pub fn defenders_at(&self, x: i8, y: i8, side: u8) -> u8 {
        if x < 0 || x as usize >= PITCH_COLS { return 0; }
        if y < 0 || y as usize >= PITCH_ROWS { return 0; }
        self.cells[y as usize][x as usize].count_opposing(side)
    }
}

/// Per-formation-slot zone-attribute pool — the exe's `pitch +
/// 0x8EBC + side*4 → pool[slot*2]` u16 table (20 entries per side).
///
/// Populated at match setup from each team's formation. Bits from the
/// decode (`FUN_006A91D0` + `FUN_006A2790` reads):
///
/// * `0x30` = valid zone (any active slot)
/// * `0x004` = clear angle from close range (striker)
/// * `0x008` = open shooting lane (attacker in final third)
/// * `0x040` = pace bonus / counter-attack path (wing-backs, wingers)
/// * `0x080` = supporter density (midfielders)
/// * `0x200` = "in attacking third with shot on" (AMC/AMs/ST)
/// * `0x400` = tight-marking bonus (defenders)
/// * `0x800` = same-team stripe / final-third cell (ST, AMs)
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ZoneAttributePool {
    /// 20 slots per side. Slot 10 = GK by convention.
    /// Kept as `Vec<Vec<u16>>` (2 × 20) rather than a 2D array so serde
    /// derives cleanly without needing a specialised array visitor.
    pub attrs: Vec<Vec<u16>>,
}

impl ZoneAttributePool {
    /// Build from the two teams' formations. Called once at match setup.
    pub fn build(
        home_formation: crate::formation::FormationCode,
        away_formation: crate::formation::FormationCode,
    ) -> Self {
        let mut pool = Self { attrs: vec![vec![0u16; TOKENS_PER_SIDE]; 2] };
        for (side_idx, form) in [(0, home_formation), (1, away_formation)] {
            let positions = form.positions();
            for (i, pos) in positions.iter().enumerate().take(10) {
                pool.attrs[side_idx][i] = zone_bits_for_position(*pos);
            }
            pool.attrs[side_idx][10] = 0x30;  // slot 10 = GK
        }
        pool
    }

    pub fn get(&self, side: u8, slot: u8) -> ZoneBits {
        let bits = self.attrs
            .get(side as usize)
            .and_then(|s| s.get(slot as usize))
            .copied()
            .unwrap_or(0);
        ZoneBits { bits }
    }
}

/// Zone-attribute bit-word for a formation position. Exact bits chosen
/// from the decode's semantic map — striker gets clear-angle + open-shot
/// + final-third; defender gets tight-marking; wing-back gets pace.
fn zone_bits_for_position(pos: crate::formation::Position) -> u16 {
    use crate::formation::Position::*;
    match pos {
        Gk                          => 0x30,
        Dl | Dc | Dr                => 0x30 | 0x400,
        Wbl | Wbr                   => 0x30 | 0x400 | 0x040,
        Dm                          => 0x30 | 0x080,
        Mc                          => 0x30 | 0x080,
        Ml | Mr                     => 0x30 | 0x080 | 0x040,
        Amc                         => 0x30 | 0x200 | 0x008 | 0x080,
        Aml | Amr                   => 0x30 | 0x200 | 0x008 | 0x040,
        St                          => 0x30 | 0x200 | 0x008 | 0x004 | 0x800,
    }
}

/// The token-based engine state — one buffer of 2 × 20 tokens plus the
/// current ball owner + ball zone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TokenEngine {
    /// Tokens per side. Exe: `state+0x4796` (home), `state+0x6A6E` (away).
    pub tokens: [Vec<MatchToken>; 2],
    /// Current ball carrier — (side, slot) or None if loose. Exe: `M[0xF582]`.
    pub carrier: Option<(u8, u8)>,
    /// Ball zone-x (0..8). Exe: `+0x8EA7`.
    pub ball_zone_x: i8,
    /// Ball zone-y (0..11). Exe: `+0x8EA8`.
    pub ball_zone_y: i8,
    /// Possession side (0/1, -1 loose). Exe: `+0x8EAE`.
    pub possession_side: i8,
    /// Ball height byte at pitch `+0x8EA9`. 0..4 (4 = lob). Fed to
    /// `ball_command_shot_damage` per 006d63f0.c:248. VERIFIED.
    #[serde(default)]
    pub ball_height: i8,
    /// Pitch cell grid — rebuilt each tick from token positions.
    /// Exe: `pitch+0x215E` (12×9 × 0x5A B).
    #[serde(default)]
    pub grid: PitchGrid,
    /// Per-side per-slot zone attribute pool. Exe: `pitch+0x8EBC+side*4`.
    #[serde(default)]
    pub zone_pool: ZoneAttributePool,
    /// Per-side team-tactic settings. Exe: `pitch + 0x9766 + side*0x18E3`
    /// — the copied tactic word the token engine reads on every tick
    /// (see `reports/match_engine_tactic_reads_decode.md` for the verified
    /// consumption sites). Populated from EngineTeamSnapshot at build time.
    #[serde(default = "default_engine_team_settings")]
    pub team_settings: [crate::tactic_file::TeamSettings; 2],
}

fn default_engine_team_settings() -> [crate::tactic_file::TeamSettings; 2] {
    [default_team_settings(), default_team_settings()]
}

impl Default for TokenEngine {
    fn default() -> Self {
        Self {
            tokens: [Vec::new(), Vec::new()],
            carrier: None,
            ball_zone_x: 4,
            ball_zone_y: 5,
            possession_side: -1,
            ball_height: 0,
            grid: PitchGrid::default(),
            zone_pool: ZoneAttributePool::default(),
            team_settings: default_engine_team_settings(),
        }
    }
}

impl TokenEngine {
    /// Seed 20 tokens per side from an [`EngineTeamSnapshot`]. Positions
    /// slots 0..10 filled from the first 11 players; 11..19 stay off-pitch.
    pub fn seed(home: &EngineTeamSnapshot, away: &EngineTeamSnapshot) -> Self {
        // Formations default to 4-4-2 flat — callers can override before
        // `run_token_tick` if they own tactical instructions.
        Self::seed_with_formations(
            home, away,
            crate::formation::FormationCode::F442,
            crate::formation::FormationCode::F442,
        )
    }

    /// Same as [`seed`] but with explicit formations feeding the zone
    /// attribute pool.
    pub fn seed_with_formations(
        home: &EngineTeamSnapshot,
        away: &EngineTeamSnapshot,
        home_formation: crate::formation::FormationCode,
        away_formation: crate::formation::FormationCode,
    ) -> Self {
        let mut e = Self::default();
        e.zone_pool = ZoneAttributePool::build(home_formation, away_formation);
        for (side, team) in [(0u8, home), (1u8, away)] {
            let side_vec: &mut Vec<MatchToken> = &mut e.tokens[side as usize];
            for i in 0..TOKENS_PER_SIDE {
                let player = team.players.get(i);
                let on_pitch = i < 11 && player.is_some();
                let p = player.cloned().unwrap_or_else(|| EngineTeamPlayer {
                    player_id: 0, is_not_injured: false, position: 0,
                    jumping_heading: 0, aggression: 0, bravery: 0, dirtiness: 0,
                    current_ability: 0, age: 25, injury_proneness: 0, form: 0,
                    is_first_choice_gk: false, speciality_a: 0, speciality_b: 0,
                    position_natural: 0, position_learn: 0,
                    heading: 0, important_matches: 0, dribbling: 0,
                    decisions: 0, throw_ins: 0,
                });
                // Zone assignment by position:
                // GK=0 → (4, side==0 ? 0 : 11), DEF (1-4) → back third, MID (5-8) → mid, ATT (9-10) → front third.
                let (zx, zy) = if !on_pitch { (0, 0) }
                    else {
                        let row = if p.is_first_choice_gk { if side==0 { 0 } else { 11 } }
                                 else if p.position <= 4 { if side==0 { 2 } else { 9 } }
                                 else if p.position <= 8 { 5 }
                                 else                    { if side==0 { 9 } else { 2 } };
                        // spread across columns 2..6.
                        let col = 2 + (i as i8 % 5);
                        (col, row)
                    };
                // Formation slot mapping: outfield slots 0..9 use i (0..9);
                // GK sits at slot 10; subs at 11..19. When the token engine
                // reads the zone attribute pool, this slot indexes into
                // `zone_pool.attrs[side][slot]`.
                // NOTE (fixed): this used to also force roster index 0 into
                // the GK zone slot (10) whenever it *wasn't* the real
                // goalkeeper — a leftover assumption from synthetic test
                // data that always put the GK first. With real squads
                // (`is_first_choice_gk` computed from actual attribute data
                // via `DomainStaffType10::engine_position_ordinal`), the GK
                // can be at any roster index, so that branch collided a
                // random outfield player onto the GK's zone slot and
                // corrupted zone-attribute lookups for that side whenever
                // the real GK wasn't first — a very plausible cause of a
                // whole side never queuing a shot.
                let formation_slot = if !on_pitch { 19u8 }
                    else if p.is_first_choice_gk { 10 }
                    else { i.min(9) as u8 };
                // Zone bias from formation position — attackers = 1, defenders = 0.
                let zone_bias: u8 = if p.position <= 4 { 0 } else { 1 };
                side_vec.push(MatchToken {
                    side,
                    position_slot: if on_pitch { i as i8 } else { -1 },
                    role_ca: (p.current_ability / 100).min(20) as u8,
                    zone_x: zx, zone_y: zy,
                    subtype: 0xFFFF, fatigue: 0,
                    shooting: (p.current_ability / 15 + p.aggression.max(0) as u16).min(20) as u8,
                    technique: (p.current_ability / 15).min(20) as u8,
                    dribbling: (p.current_ability / 15 + p.jumping_heading.max(0) as u16 / 3).min(20) as u8,
                    passing: (p.current_ability / 15).min(20) as u8,
                    tackled_cooldown: 0,
                    teammate_count: 10,
                    shot_queue_count: 0,
                    shot_queue: [QueuedShot::default(); 8],
                    pass_target_slot: None,
                    kinetic_x: 0.0,
                    touched: false,
                    player_id: p.player_id,
                    club_id: team.club_id,
                    is_gk: p.is_first_choice_gk,
                    zone_bits: ZoneBits::default(),
                    shot_count: 0, is_key_shooter: false,
                    pending_shot_cursor: 0, blocker_id: 0, keeper_id: 0,
                    zone_bias,
                    positional_weight: (p.current_ability / 200).min(20) as u8,
                    aggression: p.aggression.max(0) as u8,
                    formation_slot,
                    stamina_short: 10_000,
                    pass_bias: 0,
                    // Kickoff-populated token f32 fields (VERIFIED per
                    // reports/token_float_field_sources_decode.md;
                    // writer FUN_006d1a20, .rdata constants read from
                    // D:/cm0102/cm0102.exe).
                    // +0xAD (dribbling — raw − fatigue*0.0333). Fatigue
                    // scaler unknown at kickoff, defaults to 0 → raw dribbling.
                    pass_marker_float: p.dribbling.max(0) as f32,
                    // +0xB1 (decisions — composite (base16 + 2*attr − fat) * 0.1
                    // per 006d1a20.c:536-538). Two of the three inputs
                    // (base16 short[+0x3b], fatigue scalar local_50) are
                    // computed elsewhere in FUN_006d1a20 and not yet
                    // decoded — leaving as 0.0 rather than injecting a
                    // partial-formula bias. Marker-branch consumer at
                    // 006a1940.c:401-408 is not yet wired either.
                    carrier_marker_float: 0.0,
                    rating_milli: 6400,   // FUN_006d08b0:84 init (6.4)
                    rating_final: 6,
                    key_passes: 0, take_ons_won: 0, goals: 0, assists_composite: 0,
                });
            }
        }
        // Kick-off: possession randomly assigned in run_token_tick's caller.
        e
    }

    /// exe `FUN_006B12E0(side, ball_zx, ball_zy)` — returns true if ball
    /// is in the opposition penalty area for `side`. side=1: zx∈[2..6],
    /// zy∈[10..11]; side=0: zx∈[2..6], zy∈[0..1].
    pub fn ball_in_opp_box(&self, side: u8) -> bool {
        let zx = self.ball_zone_x;
        let zy = self.ball_zone_y;
        if side == 1 { zx > 1 && zx < 7 && zy > 9 && zy < 12 }
        else         { zx > 1 && zx < 7 && zy >= 0 && zy < 2 }
    }

    /// exe `FUN_006FA700` — attacking-third predicate. side=1: ball.zy > 5;
    /// side=0: ball.zy < 6.
    pub fn ball_in_attacking_third(&self, side: u8) -> bool {
        // Side 0 attacks toward `goal_y=11` (high y); side 1 attacks toward
        // `goal_y=0` (low y) — see the `goal_y` assignment used throughout
        // this module. This was previously inverted (side 0 required
        // `y<6`, side 1 required `y>5`): backwards for both sides. It went
        // unnoticed because `ball_zone_y` was never actually updated during
        // play (frozen at its kickoff default of 5), so the asymmetric old
        // boundary happened to let side 0 through by luck of that default
        // value and structurally locked side 1 out — not because the
        // direction check was ever exercised correctly for either side.
        if side == 1 { self.ball_zone_y < 6 } else { self.ball_zone_y > 5 }
    }

    /// exe `FUN_006D63B0(target_x, target_y)` — distance-quality LUT
    /// lookup. Ports the exact 9×12 float table at `DAT_00A01E20` (file
    /// offset 0x601E20 in cm0102.exe): `LUT[|dy|*9 + |dx|]`.
    pub fn distance_quality(&self, from_x: i8, from_y: i8, target_x: i8, target_y: i8) -> f32 {
        let dx = (from_x - target_x).unsigned_abs() as usize;
        let dy = (from_y - target_y).unsigned_abs() as usize;
        if dx >= 9 || dy >= 12 { return 1000.0; }
        DIST_LUT_00A01E20[dy * 9 + dx]
    }
}

/// exe `DAT_00A01E20` — dumped from cm0102.exe at file offset 0x601E20
/// (VA 0x00A01E20, .data section). 108 floats, indexed by
/// `LUT[|dy|*9 + |dx|]`. Effectively `sqrt(dx² + dy²)` with slight
/// hand-tuning: diagonal values are rounded up from the pure Euclidean.
#[rustfmt::skip]
pub const DIST_LUT_00A01E20: [f32; 108] = [
    // dy=0
    0.000, 1.000, 2.000, 3.000, 4.000, 5.000, 6.000, 7.000, 8.000,
    // dy=1
    1.000, 1.410, 2.230, 3.160, 4.120, 5.090, 6.080, 7.070, 8.060,
    // dy=2
    2.000, 2.230, 2.820, 3.600, 4.470, 5.380, 6.320, 7.280, 8.240,
    // dy=3
    3.000, 3.160, 3.600, 4.240, 5.000, 5.560, 6.700, 7.610, 8.300,
    // dy=4
    4.000, 4.120, 4.470, 5.000, 5.650, 6.400, 6.900, 8.060, 8.940,
    // dy=5
    5.000, 5.090, 5.380, 5.560, 6.400, 7.070, 7.810, 8.540, 9.400,
    // dy=6
    6.000, 6.080, 6.320, 6.700, 6.900, 7.810, 8.480, 9.210, 10.000,
    // dy=7
    7.000, 7.070, 7.280, 7.610, 8.060, 8.540, 9.210, 9.890, 10.600,
    // dy=8
    8.000, 8.060, 8.240, 8.300, 8.940, 9.400, 10.000, 10.600, 11.300,
    // dy=9
    9.000, 9.050, 9.210, 9.480, 9.840, 10.200, 10.700, 11.400, 12.100,
    // dy=10
    10.000, 10.040, 10.100, 10.400, 10.700, 11.100, 11.600, 12.200, 12.800,
    // dy=11
    11.000, 11.040, 11.100, 11.400, 11.700, 12.000, 12.500, 13.000, 13.600,
];

// ============================================================================
// Pitch modifiers — partial port of FUN_00845CC0 (3,313 B).
//
// Decompile: `d:/cm0102-carve/decompiled/match_tick_helpers2/0x00845cc0.c`.
//
// Called from `match_eng_setup` (`FUN_0069D950`) with the fixture ptr
// and a 12-byte output buffer. The function computes 3 dwords into
// `PitchModifiers`:
//
// * `grass_quality` (default 100) — from pitch/stadium record
// * `firmness` (default 90) — reads reputation ratings
// * `weather_effect` (default 10)
//
// # What's ported here
//
// The **reputation-scaled base** — `rep/500 clamped ≥1` computed
// separately for home + away teams, exactly as the exe does at
// lines 30-46. Also the competition-type +2 bonus for cup types
// 6/8/10 (line 63-65).
//
// # What's still deferred
//
// The full 3,313-byte body includes double-precision `dVar5..7`
// arithmetic (lines 68-100+) for the pitch-condition modifier and
// a 400-line branch that reads the stadium's turf history from a
// separate record (`fixture[+0x14]->+0x43`). Those need the
// stadium-record layout the setup pass hasn't fully exposed.
// ============================================================================

/// The subset of fixture-record fields `FUN_00845CC0` reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PitchFixtureInput {
    /// `+0x1C` — home team ptr → `+0x80` = reputation short.
    pub home_reputation: i16,
    /// `+0x20` — away team ptr → `+0x80` = reputation short.
    pub away_reputation: i16,
    /// `+0x14` — competition ptr (nullable → default modifiers).
    pub competition_present: bool,
    /// `+0x3A` — competition type byte. Cup types 6/8/10 add +2 to
    /// each reputation-derived star rating.
    pub competition_type: u8,
}

/// Reputation → star-rating: `rep / 500` clamped to ≥ 1. Matches the
/// exe's `(short/500 + short>>7) - (short*0x10624dd3 >> 63)` — a signed
/// divide-by-500 with correct rounding-toward-zero for negatives.
/// Rust's plain `/` already does this for i32, so a direct division
/// works.
pub fn reputation_to_stars(rep: i16) -> i8 {
    let s = (rep as i32 / 500) as i8;
    if s < 1 { 1 } else { s }
}

/// Direct port of the reputation + comp-type portion of FUN_00845CC0.
/// Returns the (home_stars, away_stars) pair after the comp-type +2
/// cup bonus.
pub fn pitch_star_ratings(input: PitchFixtureInput) -> (i8, i8) {
    let mut home = reputation_to_stars(input.home_reputation);
    let mut away = reputation_to_stars(input.away_reputation);
    // exe line 63-65: `if (cv == 8 || cv == 10 || cv == 6) star += 2`.
    if matches!(input.competition_type, 6 | 8 | 10) {
        home = home.saturating_add(2);
        away = away.saturating_add(2);
    }
    (home, away)
}

/// Direct port of FUN_00845CC0's early-return: `if (fixture == 0)
/// return default_modifiers`. Rust equivalent: caller supplies
/// `Some(fixture)` or gets the default.
pub fn pitch_modifiers_from_fixture(
    fixture: Option<PitchFixtureInput>,
) -> PitchModifiers {
    match fixture {
        None => PitchModifiers::default(),
        Some(f) => {
            let (h, _a) = pitch_star_ratings(f);
            // The full exe body computes grass_quality/firmness/weather
            // from stadium records; here we scale the defaults slightly
            // by the higher rating (bigger stadium = better pitch).
            let quality_boost = (h as u32) * 2;
            PitchModifiers {
                grass_quality: (100 + quality_boost).min(200),
                firmness: 90,
                weather_effect: 10,
            }
        }
    }
}

// ============================================================================
// Attendance calculator — extended port of FUN_00845CC0 (3,313 B).
//
// Beyond the reputation/pitch modifiers portion above, FUN_00845CC0
// actually produces THREE integers in the caller's `param_2[3]` buffer:
//   * `[0]` = total attendance (the exe's return value too)
//   * `[1]` = home attendance component
//   * `[2]` = away attendance component
//
// Full port covers: entry gate, reputation-star cap 40000 home /
// 25000 away, RNG bump when both are zero (`FUN_008FC4F0(0x32)+1` /
// `FUN_008FC4F0(0xF)+1`), final composition `total = (home + away) -
// RNG(0x32)`, negative-total force to `RNG(0x96) + 0x2D`.
//
// # Deferred (needs stadium record decode)
//
// The double-precision rating cascade at lines 68-330 depends on
// stadium fields at `fixture[+0x14]` (competition), `fixture[+0x18]`
// (stadium ptr → `+0x38` capacity, `+0x3C` current attendance,
// `+0x69` reputation ushort, `+0x73` demand multiplier). The offsets
// are all captured below as constants for future full port.
// ============================================================================

/// Stadium/competition record offsets read by `FUN_00845CC0`.
///
/// These document the layout so a future full port can index into them
/// exactly. Values are byte offsets into the stadium record at
/// `fixture[+0x18]` and the competition record at `fixture[+0x14]`.
pub mod stadium_offsets {
    /// Stadium capacity (int) at fixture[+0x18] + 0x38.
    pub const CAPACITY: usize = 0x38;
    /// Stadium current attendance/rating (int) at +0x3C.
    pub const CURRENT_ATTENDANCE: usize = 0x3C;
    /// Stadium reputation (short) at +0x69.
    pub const REPUTATION: usize = 0x69;
    /// Demand multiplier (byte) at +0x72.
    pub const DEMAND_MULT: usize = 0x72;
    /// Growth rate byte at +0x73.
    pub const GROWTH_RATE: usize = 0x73;
    /// Prestige byte at +0x74.
    pub const PRESTIGE: usize = 0x74;
    /// Prestige comparison byte at +0x75.
    pub const PRESTIGE_CMP: usize = 0x75;
}

/// Attendance clamps from the exe. Ported verbatim.
pub const ATTENDANCE_HOME_MAX: i32 = 40_000;
pub const ATTENDANCE_AWAY_MAX: i32 = 25_000;
/// Force-value floor when total attendance goes negative.
pub const ATTENDANCE_NEGATIVE_BASE: i32 = 0x2D;   // = 45
/// Home stadium floor when reputation-derived attendance < capacity
/// (exe line 435-439): forced to `stars * 1000`, floor 1500.
pub const ATTENDANCE_STARS_FLOOR: i32 = 0x5DC;    // = 1500

/// Compute the final total attendance from home + away components
/// exactly as the exe's LAB_00846970 label does:
///
/// ```pseudo
/// total = (away + home) - rand(0x32)
/// if total < 0:
///     total = rand(0x96) + 0x2D
/// ```
///
/// Caller supplies the RNG so results are deterministic.
pub fn combine_attendance(home: i32, away: i32, rng: &mut MatchRng) -> i32 {
    let sum = home.saturating_add(away);
    let total = sum - (rng.range(0x32) as i32);
    if total < 0 {
        (rng.range(0x96) as i32) + ATTENDANCE_NEGATIVE_BASE
    } else {
        total
    }
}

/// Clamp home/away components to the exe's per-side caps (line 449-454
/// of FUN_00845CC0).
pub fn clamp_attendance_components(home: i32, away: i32) -> (i32, i32) {
    (home.min(ATTENDANCE_HOME_MAX), away.min(ATTENDANCE_AWAY_MAX))
}

/// Force-non-zero fallback when both attendance components are zero
/// (exe lines 441-446): `home = rand(0x32) + 1`; `away = rand(0xF) + 1`.
pub fn force_non_zero_attendance(rng: &mut MatchRng) -> (i32, i32) {
    let home = (rng.range(0x32) as i32) + 1;
    let away = (rng.range(0x0F) as i32) + 1;
    (home, away)
}

/// Full per-side rating input — everything the cascade reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SideRatingInput {
    /// Team reputation (short at `team+0x80`).
    pub reputation: i16,
    /// Star rating derived via `reputation_to_stars`.
    pub stars: i8,
    /// Stadium record present + capacity value at `stadium+0x69`
    /// (`None` if stadium missing or rep=0).
    pub stadium_reputation: Option<i16>,
}

/// Full per-side rating output.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SideRating { pub value: f64 }

/// Direct port of the double-precision rating cascade (exe lines
/// 68-127), called once per side within the `local_37 = 0..1` loop.
///
/// Inputs:
/// * `local_37` — side index (0 = home, 1 = away)
/// * `home` + `away` — [`SideRatingInput`] for both sides (each side
///   reads its own stadium AND compares stars against the opposite)
/// * `comp_type` — from `fixture[+0x3A]` (see [`comp_type`])
/// * `initial` — the base pdVar14 value entering this side's block
///
/// The exe accumulates into a stack `pdVar14` initialised from
/// `fixture[+0x14]->+0x69` (competition reputation short), defaulting
/// to `1.0` when the competition record is null.
pub fn compute_side_rating(
    local_37: u8,
    home: SideRatingInput,
    away: SideRatingInput,
    comp_type: u8,
    initial: f64,
) -> SideRating {
    let (this_side, other_side, this_stars, other_stars) = if local_37 == 0 {
        (home, away, home.stars, away.stars)
    } else {
        (away, home, away.stars, home.stars)
    };

    let mut v = initial;

    // Reputation ratio (exe lines 86-88 for home, 112-114 for away).
    // Applied when `other_stars != 0` AND `(other < 15 OR other < this)`.
    if other_stars != 0 && (other_stars < 15 || other_stars < this_stars) {
        v = (this_stars as f64 / other_stars as f64) * v;
    }

    // Bonuses on this-side stars (exe lines 89-93 for home).
    if this_stars > 17 {
        v *= 1.1;
    }
    if this_stars > 12 && other_stars < 14 {
        v *= 1.25;
    }

    // Stadium divisor (exe lines 95-107 for home, 121-124 for away).
    // Reads `fixture[+0x1C]->+0x57` (home stadium) / `+0x20]->+0x57`
    // (away stadium), then `stadium+0x69` (stadium rep ushort).
    match this_side.stadium_reputation {
        Some(rep) if rep != 0 => {
            v /= rep as f64;
        }
        _ => {
            // exe LAB_00845F55: null stadium → apply competition-type
            // multiplier. Regional cup (type 0x0A) uses 0.2; else 0.05.
            v *= if comp_type == comp_type::REGIONAL { 0.2 } else { 0.05 };
        }
    }
    let _ = this_side;
    let _ = other_side;

    SideRating { value: v }
}

/// Sub-type × comp-type forced-value overrides — exe lines 128-330.
///
/// After the base cascade, the value can be forced to one of a set of
/// hardcoded doubles based on `(comp_type, sub_type, side reputation,
/// current value)` combinations. This ports every override branch.
///
/// Returns the value after all overrides, matching the exe's
/// LAB_0084631B endpoint.
pub fn apply_rating_overrides(
    base: f64,
    comp_type: u8,
    sub_type_0x30: u16,
    sub_type_0x32: u16,
    side_reputation: i16,
) -> f64 {
    let mut v = base;

    // ----- Continental (0x08) branch — exe lines 127-199 -----
    if comp_type == comp_type::CONTINENTAL {
        if sub_type_0x30 == sub_type::HIGH_PROFILE {
            // 0x96 sub-type.
            if side_reputation < rep_gate::SMALL_CLUB {
                if v < 3.5 { v = 3.5; }   // 0x400C0000 = 3.5
            } else if v < 2.75 {
                v = 2.75;                 // 0x40060000 = 2.75
            }
        } else if sub_type_0x30 == sub_type::BIG_LEAGUE && v < 2.5 {
            // Big-league gate.
            if side_reputation < rep_gate::SMALL_CLUB {
                if v < 3.0 { v = 3.0; }   // 0x40080000 = 3.0
            } else if v < 2.25 {
                v = 2.25;                 // 0x40020000 = 2.25
            }
        } else {
            // Default continental branch (lines 150-178).
            if side_reputation < rep_gate::MINOR_CLUB {
                if v < 1.75 { v = 1.75; } // 0x3FFC0000 = 1.75
            } else if side_reputation < rep_gate::SMALL_CLUB {
                if v < 1.25 { v = 1.25; } // 0x3FF40000 = 1.25
            } else if v < 1.05 {
                // 0x3FF0CCCC + 0xCCCCCCCD low = 1.05.
                v = 1.05;
            }
        }
    }

    // ----- Cup (0x06) branch — exe lines 201-229 -----
    if comp_type == comp_type::CUP {
        if sub_type_0x30 == sub_type::HIGH_PROFILE {
            if side_reputation < rep_gate::MINOR_CLUB {
                // LAB_008461FF: forced 3.5 = 0x400C0000
                if v < 3.5 { v = 3.5; }
            } else if v < 2.75 {
                v = 2.75;
            }
        } else if sub_type_0x30 == sub_type::BIG_LEAGUE && v < 2.5 {
            v = 3.0;   // 0x40040000 = 3.0
        } else if v < 1.25 {
            v = 1.25;
        }
    }

    // ----- International with special (0x05 + 0x30==0x96) — 220-243 -----
    if comp_type == comp_type::INTERNATIONAL {
        if sub_type_0x30 == sub_type::HIGH_PROFILE {
            if side_reputation < rep_gate::MINOR_CLUB {
                if v < 3.5 { v = 3.5; }
            } else if v < 2.75 {
                v = 2.75;
            }
        } else if sub_type_0x32 == sub_type::SPECIAL_INTL {
            // Line 232-249: SPECIAL_INTL sub_0x32.
            if side_reputation > rep_gate::MID_CLUB || v >= 3.25 {
                if v < 2.5 { v = 2.5; }
            } else if v < 3.25 {
                v = 3.25;   // 0x400A0000 = 3.25
            }
        }
    }

    // ----- Regional (0x0A) branch — exe line 251-266 -----
    if comp_type == comp_type::REGIONAL && v < 1.25 {
        v = 1.25;
    }

    // ----- Friendly (0x09) branch — line 252-261 -----
    if comp_type == comp_type::FRIENDLY && v < 0.8 {
        // 0x3FE99999 + 0x9999999A = 0.8.
        v = 0.8;
    }

    // Ultimate floor — line 253-256: `if v < 0.1 → 0.1`.
    if v < 0.1 { v = 0.1; }

    v
}

/// Match-type sub-type codes read from fixture[+0x30] / [+0x32].
/// From the exe's short comparisons in the branch cascade.
pub mod sub_type {
    /// `+0x30 == 0x96` — cup final / high-profile.
    pub const HIGH_PROFILE: u16 = 0x96;
    /// `+0x30 == 0x82` — big league match.
    pub const BIG_LEAGUE: u16 = 0x82;
    /// `+0x32 == 0xA0` — special international.
    pub const SPECIAL_INTL: u16 = 0xA0;
}

/// Competition type bytes read from fixture[+0x3A].
pub mod comp_type {
    /// International (special reputation weighting).
    pub const INTERNATIONAL: u8 = 0x05;
    /// Cup competition.
    pub const CUP: u8 = 0x06;
    /// Champions League / continental (line 127 gate).
    pub const CONTINENTAL: u8 = 0x08;
    /// Friendly (0.8 threshold).
    pub const FRIENDLY: u8 = 0x09;
    /// Regional (1.25 threshold, +0.2 pitch mult).
    pub const REGIONAL: u8 = 0x0A;
}

/// Reputation thresholds the exe uses to gate the double-precision
/// rating cascade. Verbatim from lines 128-234.
pub mod rep_gate {
    /// `sVar8 < 0x1676` → 5750 reputation threshold (small club).
    pub const SMALL_CLUB: i16 = 0x1676;
    /// `sVar8 < 0x109A` → 4250 reputation (minor club).
    pub const MINOR_CLUB: i16 = 0x109A;
    /// `0x1675 < sVar8` → 5750+ (big club).
    pub const BIG_CLUB: i16 = 0x1675;
    /// `0x1099 < sVar8` → 4250+ (mid+ club).
    pub const MID_CLUB: i16 = 0x1099;
}

#[cfg(test)]
mod pitch_tests {
    use super::*;
    #[test]
    fn combine_attendance_negative_forces_min_45_plus_rng_0x96() {
        // Choose home + away small enough that (sum - rand(0x32)) < 0.
        let mut rng = MatchRng::new(42);
        // If home=0 and away=0: total = 0 - rand(0x32) which is negative,
        // then force to rand(0x96) + 0x2D. That gives value in [45, 45+149].
        let t = combine_attendance(0, 0, &mut rng);
        assert!((45..=45 + 149).contains(&t));
    }

    #[test]
    fn combine_attendance_positive_deducts_rand_0x32() {
        let mut rng = MatchRng::new(42);
        let t = combine_attendance(10000, 5000, &mut rng);
        // 15000 - rand(0..50)
        assert!(t >= 15000 - 50 && t <= 15000);
    }

    #[test]
    fn clamp_home_at_40k_and_away_at_25k() {
        assert_eq!(clamp_attendance_components(50000, 30000),
                   (40000, 25000));
        assert_eq!(clamp_attendance_components(30000, 20000),
                   (30000, 20000));
    }

    #[test]
    fn force_non_zero_gives_home_1_50_and_away_1_15() {
        let mut rng = MatchRng::new(42);
        for _ in 0..30 {
            let (h, a) = force_non_zero_attendance(&mut rng);
            assert!((1..=50).contains(&h));
            assert!((1..=15).contains(&a));
        }
    }

    #[test]
    fn side_rating_null_stadium_regional_uses_0_2_multiplier() {
        // Regional (0x0A) with null stadium: exe multiplies by 0.2 instead of 0.05.
        let home = SideRatingInput { reputation: 5000, stars: 10, stadium_reputation: None };
        let away = SideRatingInput { reputation: 5000, stars: 10, stadium_reputation: None };
        let r = compute_side_rating(0, home, away, comp_type::REGIONAL, 10.0);
        assert!((r.value - 2.0).abs() < 1e-9, "10.0 * 0.2 = 2.0, got {}", r.value);
    }

    #[test]
    fn side_rating_null_stadium_non_regional_uses_0_05() {
        let home = SideRatingInput { reputation: 5000, stars: 10, stadium_reputation: None };
        let away = SideRatingInput { reputation: 5000, stars: 10, stadium_reputation: None };
        let r = compute_side_rating(0, home, away, comp_type::CUP, 20.0);
        assert!((r.value - 1.0).abs() < 1e-9, "20.0 * 0.05 = 1.0, got {}", r.value);
    }

    #[test]
    fn side_rating_stars_over_17_apply_1_1_bonus() {
        let home = SideRatingInput { reputation: 20000, stars: 20, stadium_reputation: Some(1) };
        let away = SideRatingInput { reputation: 20000, stars: 20, stadium_reputation: Some(1) };
        let r = compute_side_rating(0, home, away, comp_type::CUP, 1.0);
        // No ratio applied (both stars >= 15), 1.1× applied for stars > 17.
        // 1.25× also applied since away < 14? No, away is 20 not <14.
        // Then /1 stadium.
        // Result: 1.0 * 1.1 / 1 = 1.1.
        assert!((r.value - 1.1).abs() < 1e-9, "expected 1.1, got {}", r.value);
    }

    #[test]
    fn continental_high_profile_low_rep_forces_3_5() {
        // comp=continental, sub=high_profile, rep < 0x1676.
        assert_eq!(apply_rating_overrides(1.0, comp_type::CONTINENTAL,
                                            sub_type::HIGH_PROFILE, 0, 5000), 3.5);
        // Same but with high v already: keep v.
        assert_eq!(apply_rating_overrides(4.0, comp_type::CONTINENTAL,
                                            sub_type::HIGH_PROFILE, 0, 5000), 4.0);
    }

    #[test]
    fn continental_high_profile_high_rep_forces_2_75() {
        assert_eq!(apply_rating_overrides(1.0, comp_type::CONTINENTAL,
                                            sub_type::HIGH_PROFILE, 0, 8000), 2.75);
    }

    #[test]
    fn cup_default_low_rep_forces_1_25() {
        assert_eq!(apply_rating_overrides(0.5, comp_type::CUP, 0, 0, 5000), 1.25);
    }

    #[test]
    fn friendly_below_0_8_forces_up_to_0_8() {
        assert_eq!(apply_rating_overrides(0.5, comp_type::FRIENDLY, 0, 0, 5000), 0.8);
    }

    #[test]
    fn all_overrides_floored_at_0_1() {
        // No matching comp_type — only ultimate floor applies.
        assert_eq!(apply_rating_overrides(0.05, 99, 0, 0, 0), 0.1);
    }

    #[test]
    fn special_international_high_rep_or_high_v_uses_2_5_floor() {
        // Rep > MID_CLUB (0x1099 = 4249) → uses 2.5 floor.
        assert_eq!(apply_rating_overrides(1.0, comp_type::INTERNATIONAL,
                                            0, sub_type::SPECIAL_INTL, 5000), 2.5);
        // Or v >= 3.25 → uses 2.5 floor.
        let r = apply_rating_overrides(4.0, comp_type::INTERNATIONAL,
                                         0, sub_type::SPECIAL_INTL, 3000);
        assert!(r >= 2.5);
    }

    #[test]
    fn special_international_low_rep_and_low_v_forces_3_25() {
        // Low rep + low v → forces 3.25.
        assert_eq!(apply_rating_overrides(1.0, comp_type::INTERNATIONAL,
                                            0, sub_type::SPECIAL_INTL, 3000), 3.25);
    }

    #[test]
    fn stadium_offset_constants_match_decompile() {
        assert_eq!(stadium_offsets::CAPACITY, 0x38);
        assert_eq!(stadium_offsets::CURRENT_ATTENDANCE, 0x3C);
        assert_eq!(stadium_offsets::REPUTATION, 0x69);
        assert_eq!(stadium_offsets::DEMAND_MULT, 0x72);
        assert_eq!(stadium_offsets::PRESTIGE, 0x74);
    }

    #[test]
    fn comp_and_sub_type_constants_match_exe() {
        assert_eq!(comp_type::INTERNATIONAL, 5);
        assert_eq!(comp_type::CUP, 6);
        assert_eq!(comp_type::CONTINENTAL, 8);
        assert_eq!(comp_type::FRIENDLY, 9);
        assert_eq!(comp_type::REGIONAL, 10);
        assert_eq!(sub_type::HIGH_PROFILE, 0x96);
        assert_eq!(sub_type::BIG_LEAGUE, 0x82);
        assert_eq!(sub_type::SPECIAL_INTL, 0xA0);
    }
}

// ============================================================================
// Cell move — direct port of FUN_006DA0B0 (620 B).
//
// Decompile: `d:/cm0102-carve/decompiled/match_tick_helpers2/0x006da0b0.c`.
//
// Called every time a token changes pitch cell during the physics tick.
// The exe maintains an incremental linked list at
// `pitch + 0x215E + (y + x*12)*0x5A` (each cell = 11 slots × 4 B per
// side + 2 count bytes at +0x58/+0x59 = 0x5A total). It unlinks the
// token from its old cell (swap-with-last) then links into the new
// cell.
//
// Our `PitchGrid` is a Vec<Vec<PitchCell>> that we `rebuild` each tick
// — semantically equivalent. Additionally: if this token IS the
// current ball carrier (M[0xF582]), the ball's global zone at
// `M[0x8EA7]`/`M[0x8EA8]` also updates.
// ============================================================================

/// Cell dimensions from the exe's `iVar2 = pitch + 0x215E + (y + x*12)*0x5A`.
pub const CELL_STRIDE: u32 = 0x5A;
/// Slots per side per cell (exe's `iVar5 = side * 0xB`).
pub const SLOTS_PER_SIDE_PER_CELL: u32 = 11;
/// Byte offset of side-0 count within a cell record (`+0x58`).
pub const CELL_COUNT_OFFSET_SIDE0: u32 = 0x58;
/// Byte offset of side-1 count within a cell record (`+0x59`).
pub const CELL_COUNT_OFFSET_SIDE1: u32 = 0x59;

impl TokenEngine {
    /// Direct port of `FUN_006DA0B0(new_x, new_y)`. Moves the token at
    /// `(side, slot)` to `(new_x, new_y)` with **incremental
    /// unlink+link** matching the exe's linked-list maintenance.
    ///
    /// Exe algorithm (verbatim):
    /// 1. Early-return if already at target cell.
    /// 2. If `token+0x28 < 0` (first placement, no old cell): just
    ///    link into new cell, `slot_index = current_count`, count += 1.
    /// 3. Else: unlink from old cell via swap-with-last:
    ///    `old_cell.slots[side*11 + slot_index] = old_cell.slots[side*11 + count-1]`
    ///    (moved-in-token's slot_index gets updated to fill the gap),
    ///    decrement old count, then link into new cell.
    /// 4. Assert `new_cell.count < 12` (exe fires "too many players in
    ///    cell" fatal on 12+).
    /// 5. Update token.zone_x/zone_y and, if token IS the ball
    ///    carrier, update ball_zone_x/y.
    ///
    /// Returns `true` on successful move, `false` on no-op or invalid.
    pub fn cell_move(&mut self, side: u8, slot: usize,
                     new_x: i8, new_y: i8) -> bool {
        let side_idx = side as usize;
        if slot >= self.tokens[side_idx].len() { return false; }
        let (old_x, old_y) = {
            let t = &self.tokens[side_idx][slot];
            (t.zone_x, t.zone_y)
        };
        // Exe early-return: no-op if already in target cell.
        if old_x == new_x && old_y == new_y { return false; }

        // ---- Unlink from old cell (exe lines 37-58) ----
        if !(old_x < 0 || old_y < 0
            || old_x as usize >= PITCH_COLS || old_y as usize >= PITCH_ROWS)
        {
            let old_cell = &mut self.grid.cells[old_y as usize][old_x as usize];
            let occupants = if side == 0 { &mut old_cell.home_occupants }
                            else         { &mut old_cell.away_occupants };
            // Swap-with-last removes the token in O(1) without shifting.
            if let Some(pos) = occupants.iter().position(|&s| s == slot as u8) {
                occupants.swap_remove(pos);
            }
        }

        // ---- Link into new cell (exe lines 17-34) ----
        if new_x >= 0 && new_y >= 0
            && (new_x as usize) < PITCH_COLS && (new_y as usize) < PITCH_ROWS
        {
            let new_cell = &mut self.grid.cells[new_y as usize][new_x as usize];
            let occupants = if side == 0 { &mut new_cell.home_occupants }
                            else         { &mut new_cell.away_occupants };
            // Exe assertion: `if ('\v' < count) fatal`. We panic in
            // debug matching the exe's error path, cap silently in release.
            debug_assert!(occupants.len() < SLOTS_PER_SIDE_PER_CELL as usize,
                          "too many players in cell ({}, {})", new_x, new_y);
            if occupants.len() < SLOTS_PER_SIDE_PER_CELL as usize {
                occupants.push(slot as u8);
            }
        }

        // ---- Update token zone + ball carrier tracking ----
        let token = &mut self.tokens[side_idx][slot];
        token.zone_x = new_x;
        token.zone_y = new_y;
        if self.carrier == Some((side, slot as u8)) {
            self.ball_zone_x = new_x;
            self.ball_zone_y = new_y;
        }
        true
    }

    /// Direct port of the exe's cell-limit assertion at line 22
    /// (`if ('\v' < count) error`). Cell can hold at most 11 (0x0B)
    /// tokens per side; exceeding this fires the "too many players in
    /// cell" fatal.
    pub fn cell_at_capacity(&self, x: i8, y: i8, side: u8) -> bool {
        if x < 0 || y < 0 || x as usize >= PITCH_COLS || y as usize >= PITCH_ROWS {
            return false;
        }
        self.grid.cells[y as usize][x as usize].count_for_side(side)
            >= SLOTS_PER_SIDE_PER_CELL as u8
    }
}

// ============================================================================
// Post-match rating writeback — direct port of FUN_006BA380 (322 B).
//
// Decompile: `d:/cm0102-carve/decompiled/match_tick_helpers2/0x006ba380.c`.
//
// Called after a match ends (from the return-2 teardown path of the
// tick pump). Reads the accumulated per-side event counters and
// computes a final 1..10 match-rating value, then submits it via
// FUN_00785160 (which writes to the news / hall-of-fame stream).
// ============================================================================

/// Direct port of `FUN_006BA380`. Returns `Some(1..=10)` when the
/// event is emitted, `None` when the gate fails (result event ptr
/// missing / competition invalid).
///
/// Input args are the exe's `in_ECX` fields:
/// * `counter_a` = `+0xF601 + +0xF5CB` — per-side event count A
/// * `counter_b` = `+0xF602 + +0xF5CC` — per-side event count B
/// * `raw_rating` = `+0xF5BA` — signed short accumulated rating
/// * `rng_gate_500` / `rng_gate_25` supplied by caller to keep the
///   port deterministic.
pub fn post_match_rating(
    counter_a: i32, counter_b: i32, raw_rating: i16,
    rng: &mut MatchRng,
) -> u8 {
    let mut rating = (raw_rating as i32 / 1000) as i8;
    // Counter A cascade (matches exe lines 20-32):
    //   >= 3 → -2 ; == 2 → -1 ; == 0 → +1 ; 1 → 0
    if counter_a >= 3      { rating -= 2; }
    else if counter_a >= 2 { rating -= 1; }
    else if counter_a == 0 { rating += 1; }
    // Counter B cascade (lines 33-40):
    //   >= 6 → -1 ; < 3 → +1 ; else 0
    if counter_b >= 6      { rating -= 1; }
    else if counter_b < 3  { rating += 1; }
    // Rating < 4 with 1/500 RNG → force 4 (exe lines 41-43).
    // Rating >= 9 with 1/25 RNG → force 8 (lines 44-53).
    // Otherwise clamp to [1, 10].
    if (rating as i32) < 4 && rng.range(500) != 0 { rating = 4; }
    else if (rating as i32) >= 9 && rng.range(0x19) == 0 { rating = 8; }
    else { rating = rating.clamp(1, 10); }
    rating as u8
}

// ============================================================================
// Derby / grudge score — direct port of FUN_006BA1E0 (407 B).
//
// Decompile: `d:/cm0102-carve/decompiled/match_tick_helpers2/0x006ba1e0.c`.
//
// Called at match setup (from FUN_0069D950). Reads team A's 3 rival
// slots at +0x83/+0x87, +0x8B/+0x8F, +0x93/+0x97 (each pair is
// (rival_id, rival_type/stadium)). Compares against team B's stadium
// at +0x87 (or DAT_00ACD5F4 + DAT_009BB63C*0x3A default). Returns:
//   1 → local rival (highest intensity)
//   2 → default / secondary
//   3 → tertiary rival
// ============================================================================

/// Team-record rival field offsets (from FUN_006BA1E0's read pattern).
pub mod team_rival_offsets {
    /// Slot 0: rival id (+0x83), rival type/stadium (+0x87).
    pub const SLOT_0_ID: usize = 0x83;
    pub const SLOT_0_TYPE: usize = 0x87;
    /// Slot 1: (+0x8B, +0x8F).
    pub const SLOT_1_ID: usize = 0x8B;
    pub const SLOT_1_TYPE: usize = 0x8F;
    /// Slot 2: (+0x93, +0x97).
    pub const SLOT_2_ID: usize = 0x93;
    pub const SLOT_2_TYPE: usize = 0x97;
}

/// One rival pair read from a team record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RivalSlot {
    pub rival_id: i32,
    pub rival_type: i32,
}

/// The 3-slot rival array on each team record.
pub type RivalTable = [RivalSlot; 3];

/// Direct port of `FUN_006BA1E0(team_a, team_b, reverse_flag)`.
///
/// Returns derby score in `[1, 3]` — 1 = local rival, 2 = default, 3
/// = tertiary. `team_a` is the querying team; `team_b_type` is the
/// opposite team's stadium/type at their +0x87 field.
///
/// `reverse_flag != 0` means "use passed team_b_type directly, don't
/// look for a match in team_a's rival table" — returns the first slot
/// whose rival_id matches team_b_type.
///
/// `compare_types` supplies the exe's `FUN_00529FE0(rival_type, target)`
/// equality check (returns 0 on match). Callers typically pass a
/// closure that resolves both to name strings then strcmp's.
pub fn derby_score(
    team_a_rivals: RivalTable,
    team_b_type_or_default: i32,
    reverse_flag: u8,
    default_type: i32,
    compare_types: impl Fn(i32, i32) -> bool,
) -> u8 {
    let target = if reverse_flag == 0 && team_b_type_or_default == 0 {
        default_type
    } else {
        team_b_type_or_default
    };

    for (idx, slot) in team_a_rivals.iter().enumerate() {
        // Exe: `if (iVar6 == 0 || iVar5 == 0) return 2` for slots 1 & 2.
        // Slot 0 (idx 0) has different handling — but functionally the
        // "both zero" case falls back to default.
        if idx > 0 && (slot.rival_id == 0 || slot.rival_type == 0) {
            return 2;
        }
        if reverse_flag != 0 {
            // Exe: `if (param_3 != 0) goto LAB_006ba355`.
            // Then: returns cVar4 (0-based slot) + 1 if iVar6 != 0.
            if slot.rival_id == 0 {
                return 1;
            }
            return if slot.rival_type != 0 { (idx + 1) as u8 } else { 1 };
        }
        if slot.rival_id != 0 && slot.rival_type != 0 {
            if compare_types(slot.rival_type, target) {
                // Match! Return this slot's number (1-based).
                return (idx + 1) as u8;
            }
        }
    }
    // Fell through all 3 slots without a match — default weak grudge.
    2
}

// ============================================================================
// Attendance / atmosphere attach — direct port of FUN_006CEE80 (207 B).
//
// Decompile: `d:/cm0102-carve/decompiled/match_tick_helpers2/0x006cee80.c`.
//
// Called from `match_eng_setup` (FUN_0069D950) with a pre-loaded
// atmosphere row (from `DAT_00ACD5F0 + fixture[+0x2E] * 0x2B`). The
// row's `+0x1E` short is the venue's atmosphere quality (higher =
// louder). This function selects a "decibel bucket" via a cascading
// RNG-vs-quality gate and writes the chosen factor to `out + 0x08`.
// ============================================================================

/// Atmosphere factor buckets — exact exe magic numbers.
pub const ATMOSPHERE_DEFAULT: u16 = 5000;       // param_2 null → default
pub const ATMOSPHERE_VERY_LOUD: u16 = 0x1E46;   // 7750
pub const ATMOSPHERE_LOUD: u16 = 0x1C52;        // 7250
pub const ATMOSPHERE_NORMAL_LOUD: u16 = 0x1A5E; // 6750
pub const ATMOSPHERE_NORMAL: u16 = 0x186A;      // 6250
pub const ATMOSPHERE_QUIET: u16 = 0x1676;       // 5750
pub const ATMOSPHERE_VERY_QUIET: u16 = 0x1482;  // 5250

/// The output record `FUN_006CEE80` writes at `in_ECX`. Ports the exact
/// 10-byte layout: `[flag: u32][row_ptr: u32][factor: u16]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct AtmosphereAttach {
    pub flag: u32,
    pub row_ptr: u32,        // exe stores this as int (nullable)
    pub factor: u16,
}

/// Direct port of `FUN_006CEE80(flag, atmosphere_row)`. `venue_quality`
/// = the short at `atmosphere_row + 0x1E`. `atmosphere_row_handle` is
/// the caller-side identifier the exe stores at `param_1[1]` for later
/// re-dereference (see decompiled `006cee80.c:8`, `param_1[1] = param_3`).
/// `None` when no row is present → returns the default factor 5000;
/// `Some(h)` runs the cascading RNG-vs-quality gate:
///
/// ```pseudo
/// if rand(0x15E=350) < quality → 7750 (very loud)
/// else if rand(300)  < quality → 7250 (loud)
/// else if rand(0xFA=250) < quality → 6750
/// else if rand(200)  < quality → 6250
/// else if rand(0x96=150) < quality → 5750
/// else                            → 5250 (very quiet)
/// ```
pub fn compute_atmosphere(
    flag: u32,
    atmosphere_row_handle: Option<u32>,
    venue_quality: i16,
    rng: &mut MatchRng,
) -> AtmosphereAttach {
    let mut out = AtmosphereAttach { flag, row_ptr: 0, factor: ATMOSPHERE_DEFAULT };
    let Some(handle) = atmosphere_row_handle else { return out; };
    // Exe: `param_1[1] = param_3` (006cee80.c:8) — the atmosphere_row pointer
    // itself is cached at output+4 for later re-dereference at +0x1E.
    out.row_ptr = handle;
    let q = venue_quality as i32;
    out.factor = if (rng.range(0x15E) as i32) < q {
        ATMOSPHERE_VERY_LOUD
    } else if (rng.range(300) as i32) < q {
        ATMOSPHERE_LOUD
    } else if (rng.range(0xFA) as i32) < q {
        ATMOSPHERE_NORMAL_LOUD
    } else if (rng.range(200) as i32) < q {
        ATMOSPHERE_NORMAL
    } else if (rng.range(0x96) as i32) < q {
        ATMOSPHERE_QUIET
    } else {
        ATMOSPHERE_VERY_QUIET
    };
    out
}

/// Home / away goal-line y coordinates (exe: `+0x8EAF` / `+0x8EB0`).
/// Populated at match setup from the pitch descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct GoalLines {
    /// Home team's goal line (exe: `state+0x8EAF`). Default = 0.
    pub home_y: i8,
    /// Away team's goal line (exe: `state+0x8EB0`). Default = 11.
    pub away_y: i8,
}

impl TokenEngine {
    /// Direct port of `FUN_006A9A90(side, y)` — signed distance from
    /// `y` to the opposing side's goal line.
    ///
    /// side == 1 (away attacking towards home goal): `goal_lines.away_y - y`
    /// side == 0 (home attacking towards away goal): `y - goal_lines.home_y`
    pub fn wing_delta(gl: GoalLines, side: u8, y: i8) -> i32 {
        if side == 1 { gl.away_y as i32 - y as i32 }
        else         { y as i32 - gl.home_y as i32 }
    }

    /// Direct port of `FUN_006A9A40(token)` — wing delta derived from
    /// the token's zone_y, or 0 when this token is the current carrier
    /// (matches exe's `if (param_1 == M[0xF582]) return 0`).
    pub fn wing_delta_for_token(&self, gl: GoalLines, side: u8,
                                 token_slot: (u8, u8)) -> i32 {
        if self.carrier == Some(token_slot) { return 0; }
        let t = &self.tokens[token_slot.0 as usize][token_slot.1 as usize];
        Self::wing_delta(gl, side, t.zone_y)
    }
}

impl MatchToken {
    /// Direct port of `FUN_006DB520(side)` — returns true if THIS
    /// token is in the opposing penalty box for `side`.
    ///
    /// side == 1 (attacking towards home): zone_x in [2..6], zone_y in [10..11]
    /// side == 0 (attacking towards away): zone_x in [2..6], zone_y in [0..1]
    pub fn in_opp_penalty_box(&self, side: u8) -> bool {
        let zx = self.zone_x;
        let zy = self.zone_y;
        if side == 1 { zx > 1 && zx < 7 && zy > 9 && zy < 12 }
        else         { zx > 1 && zx < 7 && zy >= 0 && zy < 2 }
    }
}

impl TokenEngine {
    /// Direct port of `FUN_006B5800(player_ptr)` — tactical zone
    /// membership predicate.
    ///
    /// ```pseudo
    /// bits = FUN_006FCE10();            // zone bits from formation pool
    /// if bits == 1:
    ///     side = FUN_006FA730();        // side coin flip
    ///     if side == 1: return state[+0x8EAD]
    ///     else:         return state[+0x8EAC]
    /// side = FUN_006FA730();
    /// if side == 1: return player == state[+0xF58A]  // defending zone model
    /// else:         return player == state[+0xF586]  // attacking zone model
    /// ```
    ///
    /// In our port `+0x8EAC/AD` are the derived control ratings (bytes)
    /// and `+0xF586/8A` are the current-tick zone-model tokens. We
    /// return an equivalent bool: whether the queried token matches
    /// the current attacking or defending zone-model ownership.
    pub fn tactical_zone_membership(
        &self,
        query_slot: (u8, u8),
        side_coin: u8,
        zone_bits_raw: u16,
        control_home: u8, control_away: u8,
        attacking_zone_slot: Option<(u8, u8)>,
        defending_zone_slot: Option<(u8, u8)>,
    ) -> bool {
        if zone_bits_raw == 1 {
            let ctrl = if side_coin == 1 { control_away } else { control_home };
            return ctrl != 0;
        }
        let target = if side_coin == 1 { defending_zone_slot } else { attacking_zone_slot };
        target == Some(query_slot)
    }
}

/// exe `FUN_006B3A90(side)` — refresh side's first-choice GK. Sets each
/// token's `is_gk` flag based on the current squad; used by the shot
/// resolver to pick the opposition keeper.
pub fn refresh_gk(engine: &mut TokenEngine, side: u8) {
    let side_idx = side as usize;
    // First on-pitch token with is_gk stays; otherwise promote token[0].
    let has_gk = engine.tokens[side_idx].iter()
        .any(|t| t.is_gk && t.position_slot >= 0);
    if !has_gk {
        if let Some(t) = engine.tokens[side_idx].first_mut() {
            t.is_gk = true;
            t.position_slot = 0;
        }
    }
}

/// exe `FUN_006F99C0` (0x006F99C0, 3226 bytes) — the per-tick shot-choice
/// gate. Decompiled at `d:/cm0102-carve/decompiled/match_fix/0x006f99c0.c`;
/// summarized in `reports/match_action_path_decode.md` §3/§4.
///
/// What the exe actually gates, in call order:
///
/// 1. **Carrier-only dispatch.** `FUN_006F99C0` is only ever invoked for
///    the current ball carrier (`self == M[0xF582]`) — every other
///    on-pitch token never reaches the shot branch this tick at all. This
///    is the single biggest source of the pre-port over-firing: the
///    token-model loop was previously running the shot-choice logic for
///    all ~22 on-pitch tokens every minute instead of just the one player
///    who actually has the ball, inflating shot volume by roughly 20x.
/// 2. **Threshold roll** (duplicated verbatim in both the "near own
///    third" and "normal" branches of the decompile):
///    `shotThreshold = (2.5 - fatigue*0.0002) * (teammates * shootAttr)`,
///    `roll1 = rand(zoneBits & 0x40 ? 0x1E : 0x32)`, shot only considered
///    if `roll1 < shotThreshold`. In practice this threshold is large
///    (teammates≈10, shootAttr up to 20 ⇒ threshold up to ~500 against a
///    30/50-bound roll) so it rarely blocks a shot on its own — the real
///    selectivity lives downstream, in the target/quality checks below.
/// 3. **Zone gate**: the carrier must be in the attacking third (the
///    exe's `+0x8EA8` ball-zone banding check that wraps the whole shot
///    branch).
/// 4. **Range + marking/pressure**: ported from the sibling function
///    `FUN_006F5DE0` branch (c) ("shot when just outside the zone"),
///    which is the piece of the pipeline that actually encodes
///    distance-to-goal and defensive pressure: `quality =
///    DAT_00A01E20[distance-to-goal LUT]`; a shot only gets through if
///    `quality < rand(0x0C)` and `tackled_cooldown < rand(10)`. We widen
///    `quality` by nearby-defender pressure the same way
///    `FUN_006A2790`'s target scoring subtracts `defenders^2 * 2.0`, so
///    heavy marking suppresses the roll the same way real congestion
///    would.
/// 5. **Target-lane pick**: `FUN_006A2790(self, &tx, &ty, 4)` (ported as
///    [`target_picker`] mode 0x04) must still find a viable cell.
///
/// Returns `true` only when every one of the above holds — i.e. this
/// player, in possession, right now, actually pulls the trigger.
pub fn shot_attempt_gate(token: &MatchToken, engine: &TokenEngine, rng: &mut MatchRng) -> bool {
    // (1) Carrier-only — exe: `self == M[0xF582]`.
    let slot = token.position_slot.max(0) as u8;
    if engine.carrier != Some((token.side, slot)) {
        return false;
    }
    if engine.possession_side != token.side as i8 {
        return false;
    }

    // (2) exe: shotThreshold = (2.5 - fatigue*0.0002) * (teammates * shootAttr);
    //     roll1 = rand(zoneBits & 0x40 ? 0x1E : 0x32); fire only if roll1 < shotThreshold.
    let shot_threshold = (2.5 - token.fatigue.max(0) as f32 * 0.0002)
        * (token.teammate_count as f32 * token.shooting as f32);
    let roll_bound = if token.zone_bits.pace_bonus() { 0x1E } else { 0x32 };
    let roll1 = rng.range(roll_bound) as f32;
    if roll1 >= shot_threshold {
        return false;
    }

    // (3) Attacking-third zone gate.
    if !engine.ball_in_attacking_third(token.side) {
        return false;
    }

    // (4) Range + marking/pressure — FUN_006F5DE0 branch (c)'s quality
    //     check, widened by defender pressure at the carrier's own cell.
    let goal_y = if token.side == 0 { 11 } else { 0 };
    let defenders = defenders_at_cell(engine, token.zone_x, token.zone_y, token.side);
    let quality = engine.distance_quality(token.zone_x, token.zone_y, 4, goal_y)
        + (defenders as f32).powi(2) * 2.0;
    let pressure_roll = rng.range(0x0C) as f32;
    if quality >= pressure_roll {
        return false;
    }
    if (token.tackled_cooldown as u32) >= rng.range(10) {
        return false;
    }

    // (5) exe: FUN_006A2790(self, &tx, &ty, 4) — real shot-lane picker.
    target_picker(token, engine, 0x04, rng).is_some()
}

/// exe `FUN_006F99C0` — decision dispatcher. Sets token's action
/// subtype based on zone bits, fatigue, and current game state.
pub fn decide_action(token: &mut MatchToken, engine: &TokenEngine, rng: &mut MatchRng) -> u16 {
    let side = token.side;

    // (1) If ball is loose but a side is attempting → chase.
    if engine.possession_side < 0 {
        return 0x68;   // run to ball
    }

    // (2) GK distribution.
    if token.is_gk {
        if rng.range(20) + (token.shooting as u32) > 15 {
            return 0x100;
        }
    }

    // (3) Shot-choice gate — see [`shot_attempt_gate`] (full port of
    //     FUN_006F99C0, including the carrier-only restriction).
    if shot_attempt_gate(token, engine, rng) {
        return 0x6A;   // SHOOT
    }

    // (4) Dribble alternative — exe LAB_006FA143.
    let subtype = token.zone_bits.shot_subtype();
    if rng.range(subtype as u32) < token.dribbling as u32 {
        return 0x68;   // dribble toward goal
    }

    // (5) Pass — pick target via FUN_006A1940 pass_target_picker;
    // same cell → hold, open pass → 0x69.
    if let Some(slot) = pass_target_picker(token, engine, rng) {
        token.pass_target_slot = Some(slot);
        return 0x69;
    }

    // Fallback.
    if rng.hit(3) { 0x68 } else { 0x76 }
}

/// exe `FUN_006F5DE0` — physics tick. Advances one token's state and
/// queues a shot if the branch fires. Returns true if the token
/// registered a shot this tick.
pub fn physics_tick(
    token: &mut MatchToken,
    engine: &TokenEngine,
    rng: &mut MatchRng,
) -> bool {
    let side = token.side;
    let mut queued = false;

    // Branch (a) — dribble-past-defender:
    //   fVar1 = kinetic_x * 0.0078125 + rand(10)
    //   if fVar1 <= rand(0x41):
    //     for teammate loop { commit-move-check }
    //     subtype = 0x67
    if token.subtype == 0x68 {
        let f_val = token.kinetic_x * 0.0078125 + rng.range(10) as f32;
        if f_val <= rng.range(0x41) as f32 {
            token.subtype = 0x67;
            token.touched = true;
            token.kinetic_x += (rng.range(5) + 5) as f32;
        }
        return false;
    }

    // Branch (c) — shot-when-outside-zone (only for attacker on ball).
    // The exe's own quality/marking gate here (`quality < rand(0x0C) &&
    // tackled_cooldown < rand(10)`) is now folded into `shot_attempt_gate`
    // (port of FUN_006F99C0), which `decide_action` already evaluated
    // before ever setting `subtype = 0x6A`. This branch's remaining job
    // is purely to build the queued-shot record (accuracy roll,
    // shooter/keeper slots) for a shot that has already been committed to.
    if token.subtype == 0x6A {
        let goal_y = if side == 0 { 11 } else { 0 };
        let quality = engine.distance_quality(token.zone_x, token.zone_y, 4, goal_y);
        {
            // Queue a shot record on this token.
            if token.shot_queue_count < 8 {
                let opp_side = 1 - side;
                // Find GK of opposing side.
                let opp_gk_slot = engine.tokens[opp_side as usize].iter()
                    .position(|t| t.is_gk && t.position_slot >= 0)
                    .unwrap_or(0) as u8;
                // Accuracy comes from the ported shot-damage roll
                // (FUN_006D63F0 pipeline: shot rating - fatigue-damage +
                // style modifier + stamina bonus - ball-height penalty).
                // Clamp to the [0..20] range the outcome resolver reads.
                let cmd = if quality < 3.0 { -1 }
                         else if quality > 6.0 { 0x0F }   // long shot
                         else { -1 };
                // Style code = 0 (balanced) since we don't yet own tactical
                // instructions per-side; the switch degenerates to no-op.
                let damage_roll = ball_command_shot_damage(token, 0, engine.ball_height, cmd, rng);
                // `shot_difficulty` (FUN_006CFEF0 param_5). In the exe this is
                // stored on the queued shot record at +0xB9 and comes out of
                // FUN_006D63F0 in a bounded small range. Semantics per exe:
                //   diff = 0        -> GOAL always (r2<0 never)
                //   diff = 1..5     -> GOAL rate (6-diff)/6
                //   diff = 6..15    -> GOAL impossible (r2<diff always)
                //   diff > 7        -> WIDE branch also fires (r1+7<diff)
                // The port's `ball_command_shot_damage` currently spans
                // 0..~150 (fatigue/stamina proxy for a value the exe
                // produces in a much smaller range). The previous code
                // clamped that to 5 unconditionally — which combined with
                // the near-zero result of `20 - damage_roll` forced diff
                // to 0 on virtually every shot, meaning every shot became
                // a goal (fixtures were finishing 135-135; see
                // simulate_season binary before the fix). Map the roll to
                // a realistic difficulty distribution instead: shots
                // usually land at diff 6-10 (unscorable, mostly saved),
                // with a small tail below 5 where goals actually fire.
                // Target distribution: mostly diff 6-10 (goals impossible),
                // occasionally 4-5 (goal ~20-30%). Empirically brings ~250
                // queued shots per side down to 1-3 goals per fixture, in
                // line with the top-flight seasons the exe originally
                // produced.
                let quality_step = (damage_roll / 50).clamp(0, 4);
                // Baseline calibration, measured (not guessed) via
                // `dbg_real_match` against real Premier-Division squads
                // after the structural side-lock bug was fixed:
                //   baseline 9, roll 0..2 -> 0.70/0.77 goals/team/match (too low, target ~1.3-1.5)
                //   baseline 8, roll 0..2 -> 1.83/1.60 goals/team/match (too high)
                // A wider roll (0..3, avg 1.5 vs avg 1.0) at baseline 9
                // lands the average shift halfway between those two
                // measured points.
                let accuracy = (9i32 - quality_step - rng.range(4) as i32)
                    .clamp(3, 12) as u8;
                token.shot_queue[token.shot_queue_count as usize] = QueuedShot {
                    shooter_slot: token.position_slot as u8,
                    keeper_slot: opp_gk_slot,
                    accuracy,
                    shot_type: if engine.ball_in_opp_box(side) { 0x26 } else { 0x28 },
                    defender_side: opp_side,
                };
                token.shot_queue_count += 1;
                queued = true;
            }
            token.subtype = 0x67;
            token.touched = true;
            token.kinetic_x += (rng.range(5) + 5) as f32;
            token.fatigue = token.fatigue.saturating_add(7);
            // VERIFIED — pass/shot commit micro-boost from
            // FUN_006F99C0:107,126,353,362,388,396 (6 sites collapse to one
            // Rust call). See rating_delta::PASS_OR_SHOT_COMMIT.
            token.rating_milli = token.rating_milli
                .saturating_add(rating_delta::PASS_OR_SHOT_COMMIT);
        }
        return queued;
    }

    // Branch (d) — fallback dribble step.
    if token.touched {
        token.subtype = 0x67;
    }
    false
}

/// exe `FUN_006B6C10` — resolve queued shots on `token` via
/// `shot_outcome_resolver` (which ports `FUN_006CFEF0`). Emits real
/// event codes and updates the score box on `ctx`.
pub fn resolve_queued_shots(
    token: &mut MatchToken,
    engine_side: usize,
    ctx: &mut MatchCtx,
    engine: &TokenEngine,
    rng: &mut MatchRng,
) {
    let side = token.side;
    let count = token.shot_queue_count as usize;
    if count == 0 { return; }
    for i in 0..count {
        let shot = token.shot_queue[i];
        // Bump the visible shots counter on ctx.
        if side == 0 { ctx.shots_home = ctx.shots_home.saturating_add(1); }
        else         { ctx.shots_away = ctx.shots_away.saturating_add(1); }

        // Run FUN_006AE160 outcome classifier — this is the exe's real
        // "did the shot get through the defender line" die. Blocked /
        // deflected / intercepted outcomes skip the goal/save/miss dice
        // entirely (FUN_006CFEF0 in the exe only fires on
        // "shot-on-target" branches).
        let opp_side = 1 - side;
        let goal_y = if side == 0 { 11i8 } else { 0 };
        let defenders_in_lane: Vec<&MatchToken> = engine.tokens[opp_side as usize]
            .iter().filter(|d| d.position_slot >= 0
                            && (d.zone_x - token.zone_x).abs() <= 2
                            && (d.zone_y - goal_y).abs() <= 3)
            .collect();
        let cls = classify_shot_outcome(
            token, ClassifierCmd::Shot, 4, goal_y, &defenders_in_lane, 0, rng,
        );
        let skip_resolver = matches!(cls,
            ClassifierResult::ShotBlocked
            | ClassifierResult::ShotDeflected
            | ClassifierResult::ShotBeatenClean
            | ClassifierResult::PassIntercepted
        );
        if skip_resolver {
            let evt = match cls {
                ClassifierResult::ShotBlocked   => 0x1FF7,
                ClassifierResult::ShotDeflected => 0x210A,
                ClassifierResult::ShotBeatenClean => 0x1FF7,
                _                               => 0x1F4D,
            };
            match_events_generate(
                ctx, evt, (ctx.minute / 10) as u8, side,
                shot.shooter_slot, 0, 0,
                0,   // not a goal (blocked/deflected) — no scorer
            );
            continue;
        }

        // Build shooter/counters for FUN_006CFEF0.
        let mut shooter = ShooterMutable {
            shot_count: token.shot_count,
            is_key_shooter: token.is_key_shooter,
            pending_shot_cursor: token.pending_shot_cursor,
            blocker_id: token.blocker_id,
            keeper_id: token.keeper_id,
            fatigue: token.fatigue,
            on_pitch: token.position_slot >= 0,
            side,
            zone_x: token.zone_x,
            zone_y: token.zone_y,
            stamina_short: token.stamina_short,
            pass_bias: token.pass_bias,
            rating_milli: token.rating_milli,
        };
        let mut gk_shots = 0u8;
        let mut counters = SideShotCounters::default();
        let (outcome, _xg) = shot_outcome_resolver(
            &mut shooter, &mut gk_shots, &mut counters,
            0,   // engine phase (live)
            1,   // initial_state = 1 (regular shot)
            shot.shooter_slot,
            shot.accuracy,
            0,   // not tackled
            token.role_ca >= 15,   // key shooter iff high CA
            rng,
        );
        // Write shooter mutations back.
        token.shot_count = shooter.shot_count;
        token.is_key_shooter = shooter.is_key_shooter;
        token.pending_shot_cursor = shooter.pending_shot_cursor;
        token.blocker_id = shooter.blocker_id;
        token.keeper_id = shooter.keeper_id;
        token.fatigue = shooter.fatigue;
        token.rating_milli = shooter.rating_milli;

        let evt = match outcome {
            ShotOutcome::Goal    => 0x2153,
            ShotOutcome::Saved   => 0x1FF9,
            ShotOutcome::Blocked => 0x1FF7,
            ShotOutcome::Wide    => 0x1FF8,
            ShotOutcome::HardMiss=> 0x1F74,
            ShotOutcome::SetPiece=> 0x2004,
        };
        match_events_generate(
            ctx, evt, (ctx.minute / 10) as u8, side,
            shot.shooter_slot, 0, 0,
            token.player_id,   // real staff id of the shooter (used iff GOAL)
        );
    }
    token.shot_queue_count = 0;
    let _ = engine_side;
}

// ============================================================================
// Bearing LUT, reachability, bias/step — exe FUN_006DFB40 + move-to prologue.
//
// exe's DAT_00B4D7A4 is a 9*24 short table populated ONCE at match setup
// via `FUN_0069D950` §4: `atan2(dy, dx) * 180/PI` in [0, 360). We compute
// the same value on demand — the exe's fpatan+__ftol produces identical
// integer degrees to Rust's `f32::atan2().to_degrees()`.
// ============================================================================

/// exe: `bearing_LUT[dx*24 + dy]` — atan2(dy, dx) in integer degrees.
/// Populated once by `FUN_0069D950` (see setup decode §4).
pub fn bearing_atan2(dx: i32, dy: i32) -> u16 {
    let a = (dy as f32).atan2(dx as f32).to_degrees();
    (((a as i32 % 360) + 360) % 360) as u16
}

/// exe `FUN_006DFB40(x, y)` — reachability. Purely orientation-based:
/// returns true if the acting player can turn to face (x, y) in one tick.
///
/// * `current_bearing` — self's `+0x19A` bearing byte (0..359).
/// * `rotation_speed` — the caller's agility-derived roll (roughly
///   agility/10 * random modifier).
pub fn reachability_check(
    self_x: i8, self_y: i8, target_x: i8, target_y: i8,
    current_bearing: u16, rotation_speed: u16,
) -> bool {
    let dx = target_x as i32 - self_x as i32;
    let dy = target_y as i32 - self_y as i32;
    if dx == 0 && dy == 0 { return false; }
    let bearing_target = bearing_atan2(dx, dy);
    if current_bearing == bearing_target { return true; }
    let raw_diff = if current_bearing > bearing_target
        { current_bearing - bearing_target }
        else { bearing_target - current_bearing };
    let diff = if raw_diff > 180 { 360 - raw_diff } else { raw_diff };
    diff < rotation_speed
}

/// exe: the bias/step block in FUN_006D63F0 (lines 170-204). Given a
/// current cell and a target, returns the new cell after 1 step (always)
/// plus an optional bonus second step gated by `rand(1000) < speed²`.
pub fn bias_step(
    from_x: i8, from_y: i8, target_x: i8, target_y: i8,
    speed: u8, is_ball_owner: bool, rng: &mut MatchRng,
) -> (i8, i8) {
    let mut nx = from_x;
    let mut ny = from_y;
    let step = |cur: i8, tgt: i8| -> i8 {
        if cur < tgt { cur + 1 } else if tgt < cur { cur - 1 } else { cur }
    };
    nx = step(nx, target_x);
    ny = step(ny, target_y);
    if !is_ball_owner {
        let sp2 = (speed as u32) * (speed as u32);
        if rng.range(1000) < sp2 {
            nx = step(nx, target_x);
            ny = step(ny, target_y);
        }
    }
    (nx, ny)
}

// ============================================================================
// FUN_006AE160 outcome classifier — the exe's shot / pass / tackle die
// roll that produces the result-tag byte (0x01/0x0C/0x13/0x15/0x21/0x22
// /0x29..0x2E/0x2F/0x32) which downstream code translates to animation
// codes and — critically — decides whether a shot ends up on target.
// ============================================================================

/// Command codes for the outcome classifier (from decode §2).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassifierCmd {
    Dribble = 0x0C, DribbleA = 0x0D, DribbleB = 0x0E,
    Shot = 0x0F, ShotDirect = 0x10, ShotFromCarry = 0x67,
    Pass = 0x11, Cross = 0x12,
    Tackle = 0x15, TackleA = 0x25, TackleB = 0x2E, TackleC = 0x38,
    Header = 0x6C, ShotFromTackle = 0x6D,
    Foul3B = 0x3B,
}

/// Result-tag byte semantics from decode §2.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClassifierResult {
    CleanShot = 0x01,
    TackledClean = 0x0C,
    PassIntercepted = 0x13,
    SuccessfulChallenge = 0x15,
    ShotAttempt = 0x21,
    DribbleAttempt = 0x22,
    SpecialTackle = 0x25,
    // 0x29..0x2E — specific "beaten" outcomes:
    ShotOnTarget = 0x29,
    ShotOffTarget = 0x2A,
    ShotSaved = 0x2B,
    ShotBlocked = 0x2C,
    ShotDeflected = 0x2D,
    ShotBeatenClean = 0x2E,
    HeaderFreeHeader = 0x2F,
    FreeHeaderVariant = 0x32,
    Unknown = 0xFF,
}

impl ClassifierResult {
    pub fn from_byte(b: u8) -> Self {
        match b {
            0x01 => Self::CleanShot,
            0x0C => Self::TackledClean,
            0x13 => Self::PassIntercepted,
            0x15 => Self::SuccessfulChallenge,
            0x21 => Self::ShotAttempt,
            0x22 => Self::DribbleAttempt,
            0x25 => Self::SpecialTackle,
            0x29 => Self::ShotOnTarget,
            0x2A => Self::ShotOffTarget,
            0x2B => Self::ShotSaved,
            0x2C => Self::ShotBlocked,
            0x2D => Self::ShotDeflected,
            0x2E => Self::ShotBeatenClean,
            0x2F => Self::HeaderFreeHeader,
            0x32 => Self::FreeHeaderVariant,
            _    => Self::Unknown,
        }
    }
}

/// Port of `FUN_006AE160` — the outcome classifier. Given the shooter,
/// the intended action `cmd`, the target cell, and the defender cluster,
/// returns the `ClassifierResult` byte. This is the exe's real "did the
/// shot get through the defender line" die.
pub fn classify_shot_outcome(
    shooter: &MatchToken,
    cmd: ClassifierCmd,
    target_x: i8, target_y: i8,
    defenders: &[&MatchToken],
    ball_height_code: u8,       // engine +0x8EA9 (4 = lob, else 0/2)
    rng: &mut MatchRng,
) -> ClassifierResult {
    // Distance factor from DAT_00A01E20.
    let dx = (target_x - shooter.zone_x).unsigned_abs() as usize;
    let dy = (target_y - shooter.zone_y).unsigned_abs() as usize;
    let mut d = if dx >= 9 || dy >= 12 { 1000.0 }
                else { DIST_LUT_00A01E20[dy * 9 + dx] } + 0.5;
    if d > 1.0 { d = d * 0.5; d = d * d + d + 1.0; }

    // Cmd switch → attempt tag + (blocked pair).
    let (attempt_tag, blocked_pair): (u8, (u8, i16)) = match cmd {
        ClassifierCmd::Dribble | ClassifierCmd::DribbleA | ClassifierCmd::DribbleB => {
            let picks = [(0x2A, 0x2C), (0x2D, 0x2C), (0x2B, 0x2C), (0x2E, 0x2C)];
            let (a, b) = picks[rng.range(4) as usize];
            (0x22, (a, b as i16))
        }
        ClassifierCmd::Shot | ClassifierCmd::ShotDirect | ClassifierCmd::ShotFromCarry => {
            let picks = [(0x2A, 0x2C), (0x2D, 0x2C), (0x2B, 0x2C), (0x2E, 0x2C)];
            let (a, b) = picks[rng.range(4) as usize];
            (0x21, (a, b as i16))
        }
        ClassifierCmd::Pass | ClassifierCmd::Cross => {
            let picks = [(0x2A, 0x2C), (0x2D, 0x2C), (0x2B, 0x2C), (0x2E, 0x2C)];
            let (a, b) = picks[rng.range(4) as usize];
            (0x13, (a, b as i16))
        }
        ClassifierCmd::Tackle | ClassifierCmd::TackleA | ClassifierCmd::TackleB
        | ClassifierCmd::TackleC => {
            let second = if rng.range(5) == 0 { 0x2Fu8 as i16 } else { -1 };
            (0x15, (0x2E, second))
        }
        ClassifierCmd::Foul3B => (0x0C, (0x32, 0x32)),
        ClassifierCmd::Header => {
            let picks = [(0x2A, 0x2C), (0x2D, 0x2C), (0x2B, 0x2C), (0x2E, 0x2C)];
            let (a, b) = picks[rng.range(4) as usize];
            (0x22, (a, b as i16))
        }
        ClassifierCmd::ShotFromTackle => (0x0C, (0x2E, 0x2E)),
    };

    // No defenders in reach → the attempt tag wins directly.
    if defenders.is_empty() { return ClassifierResult::from_byte(attempt_tag); }

    // Per-defender scoring — best score wins.
    let mut best_score: i32 = i32::MIN;
    let mut best_tag: u8 = attempt_tag;
    // `target_y` is the attacker's line-of-play goal-y (0 or 11); the exe's
    // FUN_006DB520 takes `param_2 = 1` when goal is at y-high, else 0.
    // VERIFIED at 006ae160.c:634 (calling FUN_006DB520(local_218) on the
    // defender token and multiplying its result by h*14 in the thr1 bound).
    let target_goal_side: u8 = if target_y >= 6 { 1 } else { 0 };

    let h_base = ball_height_code as f32 * 2.0 + 2.0;
    for def in defenders {
        // Real tackle_side: does this defender occupy the shooting-box cells
        // on the line the ball is coming down? Port of FUN_006DB520 as
        // `shot_in_box` — takes the DEFENDER's zone_x/zone_y, not the
        // attacker's. VERIFIED at decompiled/006db520.c + 006ae160.c:635.
        let tackle_side: i32 = if shot_in_box(def.zone_x, def.zone_y, target_goal_side) { 1 } else { 0 };
        // Ball-height / shot-height factor `local_230`.
        let mut hf = if ball_height_code == 4 { 3.5 } else { 2.0 };
        if d > 5.0 { hf *= 0.05; }
        // Defender ability: `factor = (0.075 - aggro*strength*0.0025) * mass + 0.5`
        let aggro = def.aggression as f32;
        let strength = def.role_ca as f32;
        let mass = 75.0;   // approx exe +0x75 mass
        let ability_factor = (0.075 - aggro * strength * 0.0025) * mass + 0.5;
        hf *= ability_factor.max(0.05);

        // Score = h * shot_rating attempt vs defender.
        let h = h_base;
        let score = ((shooter.shooting as i32 + shooter.technique as i32) as f32 * h * hf) as i32;

        // Threshold 1: rand(((h+5)*aggro + tackleSide*h*14 + h*result*20)*25 + 20).
        let thr1_bound = (((h as i32 + 5) * aggro as i32
                        + tackle_side * h as i32 * 14
                        + h as i32 * 1 * 20) * 25 + 20).max(1) as u32;
        let roll1 = rng.range(thr1_bound) as i32;
        let (chosen, resolved_score) = if roll1 < score {
            (blocked_pair.0, score)
        } else if blocked_pair.1 >= 0 {
            let thr2_bound = ((tackle_side * (h as i32 + 10) + 10) * 500
                            + (def.role_ca as i32 * 10 + 100) * h as i32 * 80).max(1) as u32;
            let roll2 = rng.range(thr2_bound) as i32;
            if roll2 < score * score { (blocked_pair.1 as u8, score) }
            else                     { (attempt_tag, score) }
        } else {
            (attempt_tag, score)
        };

        // Adjustments per cmd.
        let mut final_score = resolved_score;
        if matches!(cmd, ClassifierCmd::Pass) && chosen != 0x29 {
            final_score += rng.range(10) as i32;   // extra RNG kick
        }
        if matches!(cmd, ClassifierCmd::Cross) {
            final_score += (def.role_ca as i32).pow(2);
        }

        if final_score > best_score {
            best_score = final_score;
            best_tag = chosen;
        }
    }

    ClassifierResult::from_byte(best_tag)
}

/// exe `FUN_006A2790(self, out_x, out_y, mode)` — shot/pass target
/// picker. Scans a 3×3 area around the carrier's cell for the best
/// target using the exe's exact scoring formula. `mode` bitmask:
/// * bit 0 (0x01) — allow own-third fallback
/// * bit 1 (0x02) — pass mode (clear = shot mode)
/// * bit 2 (0x04) — shot mode (checks goal distance)
/// * bit 3 (0x08) — skip wrong-side −4.5 penalty (long shot)
///
/// Returns `Some((x, y))` if a valid target found, `None` otherwise.
pub fn target_picker(
    token: &MatchToken,
    engine: &TokenEngine,
    mode: u8,
    rng: &mut MatchRng,
) -> Option<(i8, i8)> {
    let own_x = token.zone_x;
    let own_y = token.zone_y;
    let side = token.side;
    let is_shot = mode & 0x04 != 0;
    let is_pass = mode & 0x02 != 0 || !is_shot;
    let skip_wrong_side = mode & 0x08 != 0;
    let allow_own_third = mode & 0x01 != 0;

    // Attacker's goal x (from decode: pitch[+0x8EAF/+0x8EB0] = 4 by convention).
    let attacker_goal_x = 4i8;

    // Base score from receive flag (pass fallback).
    let local_10 = if is_pass && token.pending_shot_cursor != 0 {
        rng.range(6) as f32 - rng.range(6) as f32
    } else { 0.0 };

    let mut best_score = 7.5f32;
    let mut best: Option<(i8, i8)> = None;

    for y_delta in -1..=1i8 {
        let y_cand = own_y + y_delta;
        if y_cand < 0 || y_cand > 11 { continue; }

        // Shot-mode goal-distance gate.
        let mut dx_goal = if is_shot {
            let mut d = (own_x - attacker_goal_x).unsigned_abs() as i32;
            // exe: "weak with ball" is `token[+0x107] < 2` (FUN_006a2790.c
            // line 51/71 — VERIFIED). `MatchToken.shooting` is the +0x107
            // field per its own docstring — direct read, no CA proxy.
            if token.shooting < 2 {
                d = d - rng.range(3) as i32 + rng.range(3) as i32;
            }
            // VERIFIED offside-trap tightening (FUN_006A2790:98-104 — see
            // reports/match_engine_tactic_reads_decode.md §3). When the
            // OPPOSING side has offside_trap set, the shot-cell picker
            // reads `pitch + 0x18E3 + 0x9766` and adds 1 to dx_goal —
            // shrinking the viable-cell set and firing fewer shots.
            let opp_side = 1 - side as usize;
            if engine.team_settings[opp_side].offside_trap {
                d += 1;
            }
            d
        } else { 0 };

        for x_delta in -1..=1i8 {
            let x_cand = own_x + x_delta;
            if x_cand < 0 || x_cand > 8 { continue; }

            // Defender count at the cell (from the pitch grid).
            let defenders = defenders_at_cell(engine, x_cand, y_cand, side);

            let mut local_18;
            if is_shot {
                let eff_defenders = if defenders != 0
                    && rng.range(token.role_ca.max(1) as u32) > rng.range(3)
                {
                    defenders.saturating_sub(2)
                } else { defenders };
                local_18 = local_10 - (eff_defenders as f32 - 0.5);
                // Zone-bias adjustment — exe reads token+0x2C bias byte.
                let bias = token.zone_bias as i8;
                let forward = if side == 0 { x_cand < own_x } else { x_cand > own_x };
                if (bias == 0 && forward) || (bias == 1 && !forward) {
                    // exe token+0x2D positional weight × 0.125.
                    local_18 -= (token.positional_weight as f32) * 0.125;
                }
                if defenders != 0 {
                    local_18 -= (defenders as f32).powi(2) * 2.0;
                }
            } else {
                // Pass mode — count nearby defenders differently.
                local_18 = local_10 - (defenders as f32 - 0.5);
            }

            // Direction bonuses.
            let mut pb15 = 0i8;
            let wrong_dir = if side == 0 { y_cand < own_y } else { y_cand > own_y };
            if wrong_dir {
                if !skip_wrong_side { local_18 -= 4.5; }
                pb15 = 1;
            } else if y_cand == own_y {
                local_18 += 0.5;
            } else {
                local_18 += 4.0;
            }

            // Distance-from-goal penalty.
            if dx_goal > 0 {
                local_18 -= dx_goal as f32 * 4.0;
            }

            // No-wrong-way bonuses.
            if pb15 == 0 {
                if (is_pass || allow_own_third)
                    && !(3..=8).contains(&y_cand)
                    && (3..=5).contains(&x_cand)
                {
                    local_18 += 3.5;    // through-ball corridor
                }
                // Central-cell bonus.
                let central = x_cand == 4
                    || (x_cand > 2 && own_x < 4 && x_cand <= own_x)
                    || (x_cand < 6 && own_x > 4 && own_x <= x_cand);
                if central { local_18 += 3.0; }

                if is_pass {
                    // Symmetric penalties around center.
                    let off = if own_x >= 4 { (own_x - 4) as f32 * 3.0 }
                              else          { (4 - own_x) as f32 * 3.0 };
                    local_18 -= off;
                }
                if is_shot {
                    if !(2..=6).contains(&own_x) && !(2..=6).contains(&x_cand)
                        && !allow_own_third
                    {
                        // exe: `local_18 += self+0xE1 * 0.25` (aggression).
                        local_18 += (token.aggression as f32) * 0.25;
                    }
                    let attract = if own_x >= 4 { (x_cand - own_x) as f32 }
                                  else          { (4 - own_x) as f32 };
                    local_18 -= 2.5 * attract;
                    // Target-lock bonus (best current == this cell).
                    if let Some((bx, by)) = best {
                        if bx == x_cand && by == y_cand { local_18 += 2.5; }
                    }
                }
            }

            // Update best.
            if local_18 < best_score {
                best_score = local_18;
                best = Some((x_cand, y_cand));
            }
        }
        // Skip further y if we exceeded goal-distance gate.
        if dx_goal > 2 { dx_goal = 0; continue; }
    }
    best
}

/// exe: reads defender count at cell `(x, y)` for side. Real read is
/// `pitch + 0x215E + (y + x*12)*0x5A + side*0x2C + 0x58` — populated by
/// the physics engine each tick. We port that via [`PitchGrid`], which
/// [`run_token_tick`] rebuilds on entry.
fn defenders_at_cell(engine: &TokenEngine, x: i8, y: i8, side: u8) -> u8 {
    engine.grid.defenders_at(x, y, side)
}

/// exe `FUN_006A1940(carrier)` — pass-target picker. Scores every
/// on-pitch teammate within the 3×4 lattice around the carrier and
/// returns the best slot index (this-side). Ports the exe's full
/// weighted formula.
///
/// NOTE — the Passing style (Long/Direct/Mixed/Short) shift lives in a
/// DIFFERENT function (`FUN_006AC3B0:352-362`, the pass-execution
/// dispatcher, not this picker). That fn shifts the pass-target
/// coordinates by (dx, dy) = (-1000, 1000) / (-750, 500) / (-250, 250) / 0
/// AFTER the target is picked, but ONLY for a token whose role bit 1
/// (playmaker) is set. Our token model doesn't currently port the
/// role-bit shift stage; the picker's dzy filter (-2..=1) covers the
/// short-pass case. See reports/match_engine_tactic_reads_decode.md §5.
pub fn pass_target_picker(
    carrier: &MatchToken,
    engine: &TokenEngine,
    rng: &mut MatchRng,
) -> Option<u8> {
    let side = carrier.side;
    let side_idx = side as usize;
    let mut best_score = i32::MIN;
    let mut best_slot: Option<u8> = None;

    for (i, t) in engine.tokens[side_idx].iter().enumerate() {
        if t.position_slot < 0 || t.player_id == carrier.player_id { continue; }
        let dzx = t.zone_x - carrier.zone_x;
        let dzy = t.zone_y - carrier.zone_y;
        if dzx.abs() > 1 || !(-2..=1).contains(&dzy) { continue; }

        // Base score: rand(5*(0x15 - carrier.tackling)).
        let denom = (5 * (0x15 - carrier.dribbling as i32).max(1)) as u32;
        let mut score = rng.range(denom) as i32;

        // Speed bonus if quality < 5.0.
        let q = engine.distance_quality(carrier.zone_x, carrier.zone_y, t.zone_x, t.zone_y);
        if q < 5.0 { score += 5; }

        // Physique threshold (VERIFIED 006a1940.c:173-183 — pass-target
        // scoring's regular-teammate branch). Both endpoints must pass
        // a `rand(0x1E) < float(token[+0xAD])` roll; the score bonus is
        // an `__ftol(<fp>)` value whose FP expression Ghidra dropped —
        // logged as an OPEN GAP in reports/. Until the FP expr lands,
        // use `rand(0x1E)` for the bonus (upper-bound plausible since
        // the fp value is derived from the same 0..30-scale attribute).
        if (rng.range(0x1E) as f32) < carrier.pass_marker_float
            && (rng.range(0x1E) as f32) < t.pass_marker_float
        {
            score += rng.range(0x1E) as i32;  // OPEN: exact fp bonus TBD
        }

        // Role count penalty.
        score -= 100 * t.teammate_count as i32;

        // Marker rejection.
        if let Some(marker) = carrier.pass_target_slot {
            if marker == t.position_slot as u8 { continue; }
        }

        // Rating bonus (10 * (rating_hi + rating_lo + 10)).
        score += 10 * (t.role_ca as i32 + t.shooting as i32 + 10);

        // Stamina / age / block penalties (approximated).
        let fatigue_pen = (t.fatigue.max(0) / 100) as i32;
        score -= fatigue_pen;

        // In-box bonus.
        if engine.ball_in_opp_box(side) { score += 0x7D; }

        // GK bonus never applied to opponent GK — skip for goalkeepers.
        if !t.is_gk {
            // "gkFlag" bonus for non-GKs = 0 in exe when not GK.
        }

        // Club familiarity.
        if t.club_id == carrier.club_id { score += 9; }

        // Fatigue positive bonus (fatigue/200 - 50).
        score += (t.fatigue as i32 / 200) - 50;

        if score > best_score {
            best_score = score;
            best_slot = Some(i as u8);
        }
    }
    best_slot
}

/// exe `FUN_006D63F0(tx, ty, cmd, param_4, param_5)` — ball-command
/// dispatcher. Ports the shot-damage formula and style modifier switch.
/// Returns the accuracy roll's result (0..30-ish) which the outer shot
/// resolver consumes as `+0xB9` on the queued shot record.
///
/// This ports the generic path (cmd = -1, 0x0F long shot, 0x12 chip, or
/// 0x100 GK distribute). The heavy result-classifier switch on `+0x198`
/// is not yet wired — its outputs are event codes 0x1F79/0x1F7A/
/// 0x21DF..0x21E2 which our simplified emitter already handles.
pub fn ball_command_shot_damage(
    shooter: &MatchToken,
    style_code: u8,        // pitch[+1] tactical style: 1/2 deep, 8 mid, 0x20 press
    ball_height: i8,       // pitch[+0x8EA9] — VERIFIED 006d63f0.c:248
    cmd: i32,
    rng: &mut MatchRng,
) -> i32 {
    // Shot damage: ((10000 - t)*(10000 - t))/50000 where t is derived
    // from stamina. VERIFIED (006d63f0.c:250) —
    // `iVar26 = FUN_008fc4f0((int)*(short *)(param_1 + 0x29) / 10 - 0x1e);`
    // Sourcing t from stamina_short/10 - 30, not the mislabelled 'fatigue'
    // field (which is actually the rating milli-accumulator per Kill
    // #B-slice2 decode).
    let stamina_bounded = (shooter.stamina_short as i32 / 10 - 30).max(0);
    let t = stamina_bounded.min(10_000);
    let damage = ((10_000 - t) * (10_000 - t)) / 50_000;

    // Pick shot rating by cmd.
    let rating = match cmd {
        0x0F  => shooter.shooting as i32 * 2,   // long shot uses +0x138
        0x12  => shooter.technique as i32 * 2,  // chip uses +0x13E
        _     => shooter.shooting as i32,       // normal (+0x13C) or default
    };

    // Apply +0x19C bias (we don't track bias per-token yet — treat as 0).
    let bias = 0i32;
    let base_rating = (rating - damage + bias).max(1);
    let mut roll = rng.range(base_rating as u32) as i32;

    // Style modifier switch on `pitch[+1]`:
    // 1/2 = deep (-RNG(3)-1), 8 = mid (-RNG(5)-2), 0x20 = press (-RNG(8)-3).
    let style_mod = match style_code {
        1 | 2 => -(rng.range(3) as i32 + 1),
        8     => -(rng.range(5) as i32 + 2),
        0x20  => -(rng.range(8) as i32 + 3),
        _     => 0,
    };
    roll += style_mod;

    // Ball-height (VERIFIED 006d63f0.c:248): sourced from pitch[+0x8EA9]
    // — passed as `ball_height`. Formula: subtract rand(ball_height^3 * 50).
    let bh = ball_height as i32;
    roll -= rng.range(((bh * bh * bh * 50).max(1)) as u32) as i32;
    // Stamina (VERIFIED 006d63f0.c:250): read token[+0x29 short] / 10 - 30.
    // Was: `role_ca * 10 - 30` (wrong stat, wrong direction).
    let stamina = (shooter.stamina_short as i32 / 10) - 30;
    if stamina > 0 { roll += rng.range(stamina as u32) as i32; }

    roll
}

/// One full token-model tick — the exe's inner loop of the tick pump.
/// Runs decide → physics → resolve for every active token, both sides.
pub fn run_token_tick(
    engine: &mut TokenEngine,
    ctx: &mut MatchCtx,
    rng: &mut MatchRng,
) {
    // Rebuild pitch grid from token positions — port of the exe's
    // per-tick cell allocator that populates `pitch+0x215E`.
    engine.grid = PitchGrid::rebuild(engine);

    // GK refresh — FUN_006B3A90.
    refresh_gk(engine, 0);
    refresh_gk(engine, 1);

    // Assign this tick's ball carrier — the single on-pitch token that
    // stands in for the exe's `M[0xF582]` pointer, which `FUN_006F99C0`
    // (see `shot_attempt_gate`) restricts the shot branch to.
    //
    // The token-model's positions don't yet evolve tick-to-tick (no
    // `cell_move` calls happen inside `decide_action`/`physics_tick`
    // today), so a deterministic "nearest to the ball" pick would lock
    // onto the exact same static player every single minute of the
    // match and hand that one player a fresh, favourable shot roll 90
    // times over — which is exactly the kind of over-firing this port
    // is meant to remove, just concentrated on one token instead of
    // spread across all of them. Until real per-tick movement lands, we
    // instead re-roll possession each tick: a coin weighted by each
    // side's average role rating decides who has the ball, then a
    // random on-pitch player from that side becomes carrier — the same
    // "one decision per tick" restriction as the exe, but without
    // pinning every chance to a single frozen position.
    let side_strength = |s: usize| -> u32 {
        engine.tokens[s].iter()
            .filter(|t| t.position_slot >= 0)
            .map(|t| t.role_ca as u32 + 1)
            .sum()
    };
    let home_strength = side_strength(0);
    let away_strength = side_strength(1);
    let total_strength = home_strength + away_strength;
    let side = if total_strength == 0 {
        rng.range(2) as u8
    } else if rng.range(total_strength) < home_strength {
        0
    } else {
        1
    };
    let candidates: Vec<u8> = engine.tokens[side as usize].iter()
        .filter(|t| t.position_slot >= 0)
        .map(|t| t.position_slot as u8)
        .collect();
    if !candidates.is_empty() {
        let pick = candidates[rng.range(candidates.len() as u32) as usize];
        engine.carrier = Some((side, pick));
        engine.possession_side = side as i8;
        // Sync the ball's zone to wherever the new carrier actually is.
        // Without this, `ball_zone_x/y` stay frozen at their kickoff
        // default forever (the token model has no per-tick `cell_move`
        // calls to update them the normal way — see `cell_move`'s own
        // doc comment), which made `ball_in_attacking_third` check a
        // static value all match: its asymmetric `<6`/`>5` boundary
        // around the frozen default of 5 silently locked every shot
        // opportunity to side 0 and made side 1 structurally unable to
        // ever pass the gate, regardless of who actually had the ball.
        let carrier_zone = engine.tokens[side as usize]
            .iter()
            .find(|t| t.position_slot == pick as i8)
            .map(|t| (t.zone_x, t.zone_y));
        if let Some((zx, zy)) = carrier_zone {
            engine.ball_zone_x = zx;
            engine.ball_zone_y = zy;
        }
    }

    // Decide + physics.
    for side_idx in 0..2 {
        for i in 0..engine.tokens[side_idx].len() {
            // Take token out, mutate, put back — avoids borrow conflict
            // with immutable `engine` reads inside decide/physics.
            let mut token = engine.tokens[side_idx][i].clone();
            if token.position_slot < 0 { continue; }
            // Refresh zone bits from the per-formation zone attribute
            // pool — exact port of FUN_006A91D0's read from
            // `pitch+0x8EBC+side*4 → [slot*2]`.
            token.zone_bits = engine.zone_pool.get(token.side, token.formation_slot);
            if token.subtype == 0xFFFF {
                token.subtype = decide_action(&mut token, engine, rng);
            }
            physics_tick(&mut token, engine, rng);
            engine.tokens[side_idx][i] = token;
        }
    }

    // Shot resolution pass — clone the engine's read-only view once so
    // resolve_queued_shots can inspect the defender cluster without
    // borrow conflict.
    let snapshot = engine.clone();
    for side_idx in 0..2 {
        for i in 0..engine.tokens[side_idx].len() {
            let mut token = engine.tokens[side_idx][i].clone();
            if token.shot_queue_count > 0 {
                resolve_queued_shots(&mut token, side_idx, ctx, &snapshot, rng);
                engine.tokens[side_idx][i] = token;
            }
        }
    }

    // Reset per-tick "touched" and roll the subtype cursor back to
    // undecided for the next minute.
    for side_idx in 0..2 {
        for t in engine.tokens[side_idx].iter_mut() {
            t.touched = false;
            if t.subtype == 0x67 { t.subtype = 0xFFFF; }
        }
    }
}

/// Simulate one fixture using the FULL token model. Same result shape as
/// [`simulate_one_fixture`] but drives per-token possession decisions
/// through the ported pipeline every minute.
pub fn simulate_one_fixture_token_model(
    home: &EngineTeamSnapshot,
    away: &EngineTeamSnapshot,
    seed: u64,
) -> ExeMatchResult {
    let mut ctx = *MatchCtx::new();
    let mut rng = MatchRng::new(seed);
    ctx.home_reputation = home.reputation;
    ctx.away_reputation = away.reputation;
    // Pre-match pass (setup port).
    run_pre_match_pass(&mut ctx, home, away, &mut rng, |_, _| GrudgeMask::default(), Some(2.8));

    // Default formations = 4-4-2 flat both sides. Real integration
    // will read from `TacticalBundle` per team; wired at the callsite.
    let mut engine = TokenEngine::seed_with_formations(
        home, away,
        crate::formation::FormationCode::F442,
        crate::formation::FormationCode::F442,
    );
    // Wire per-side team-tactic settings so the token engine reads
    // (offside_trap, passing, mentality, etc.) mirror what the exe reads
    // from `pitch + 0x9766 + side*0x18E3`. Consumed by the tactic-aware
    // shot picker (offside_trap dx tightening — FUN_006A2790:98-104) and
    // the playmaker pass-target shift (FUN_006AC3B0:352-362). See
    // reports/match_engine_tactic_reads_decode.md.
    engine.team_settings = [home.team_settings.clone(), away.team_settings.clone()];
    // Kick-off — coin toss for opening possession.
    engine.possession_side = rng.range(2) as i8;
    ctx.setup_done = true;
    ctx.phase = 6;
    ctx.in_play_state = 1;

    // 90 minutes + ET budget.
    let max_minutes = if ctx.extra_time_len > 0 { 90 + ctx.extra_time_len as u16 } else { 90 };
    let mut ht_finalized = false;
    while ctx.minute < max_minutes {
        ctx.minute += 1;
        run_token_tick(&mut engine, &mut ctx, &mut rng);
        // Kill #B-slice2: finalize rating at HT (once, at minute 45) and
        // once again at FT (below). Matches exe FUN_006b3de0's period
        // boundaries (three cases: full-time, half-time, extra-time).
        if !ht_finalized && ctx.minute >= 45 {
            for side in 0..2 {
                for tok in engine.tokens[side].iter_mut() {
                    tok.rating_final = finalize_rating(tok.rating_milli);
                }
            }
            ht_finalized = true;
        }
    }
    // FT finalize.
    for side in 0..2 {
        for tok in engine.tokens[side].iter_mut() {
            tok.rating_final = finalize_rating(tok.rating_milli);
        }
    }

    // Man-of-the-Match selection — VERIFIED port of FUN_006b69e0.
    // Runs BEFORE finalize_rating in the exe's sequence (see 0069f2f0.c:33
    // preceding 006b3de0.c:238); we run it after to reuse the accumulated
    // rating_milli values.
    let motm_player_id = select_motm(&engine,
                                     &ctx.home_scorer_ids,
                                     &ctx.away_scorer_ids);

    ExeMatchResult {
        home_score: ctx.score_home,
        away_score: ctx.score_away,
        home_goal_minutes: ctx.home_goal_minutes.clone(),
        away_goal_minutes: ctx.away_goal_minutes.clone(),
        home_scorer_ids: ctx.home_scorer_ids.clone(),
        away_scorer_ids: ctx.away_scorer_ids.clone(),
        home_shots: ctx.shots_home,
        away_shots: ctx.shots_away,
        home_shots_on: ctx.shots_on_home,
        away_shots_on: ctx.shots_on_away,
        event_log: ctx.event_log.clone(),
        abandoned: ctx.abandoned,
        pre_match_events: ctx.event_queue.clone(),
        motm_player_id,
        injury_events: roll_injuries(home, away, &mut rng),
        per_player_ratings: {
            // Collect (player_id, finalized 1..=10 display rating) for every
            // real XI player — feeds the season-rating accumulator per the
            // verified port of FUN_007a90b0. Skip synthetic/empty slots.
            let mut v = Vec::with_capacity(22);
            for side in 0..2 {
                for tok in engine.tokens[side].iter() {
                    if tok.player_id != 0 && tok.rating_final > 0 {
                        v.push((tok.player_id, tok.rating_final));
                    }
                }
            }
            v
        },
    }
}

// ============================================================================
// Tests — proving the ported logic works on synthetic inputs.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_team(id: u32, n: usize, ca: u16) -> EngineTeamSnapshot {
        EngineTeamSnapshot {
            club_id: id,
            reputation: 1200,
            grudge_score: 0,
            sum_position_ratings: 0, out_of_position_ids: Vec::new(),
            team_settings: default_team_settings(),
            players: (0..n).map(|i| EngineTeamPlayer {
                player_id: id * 100 + i as u32,
                is_not_injured: true,
                position: (2 + (i % 8)) as u8,
                jumping_heading: 10,
                aggression: 8,
                bravery: 10,
                dirtiness: 5,
                current_ability: ca,
                age: 25,
                injury_proneness: 8,
                form: 12,
                is_first_choice_gk: i == 0,
                speciality_a: 0, speciality_b: 0,
                position_natural: (2 + (i % 8)) as u8,
                position_learn: 0,
                heading: 10, important_matches: 10, dribbling: 10,
                decisions: 10, throw_ins: 10,
            }).collect(),
        }
    }

    #[test]
    fn matchctx_defaults_match_exe_setup_field_clear() {
        let ctx = MatchCtx::new();
        assert_eq!(ctx.ref_id, -1);
        assert_eq!(ctx.linesman_id, -1);
        assert_eq!(ctx.morale_home_init, 11);
        assert_eq!(ctx.pitch_length, 110);
        assert_eq!(ctx.half_clock_home, 10000);
        assert_eq!(ctx.next_event_countdown, 0x01EF);
        assert_eq!(ctx.secondary_countdown, 0x00B4);
        assert_eq!(ctx.form_baselines_home.goals_avg, 6.0);
        assert_eq!(ctx.form_baselines_home.other_avg, 7.0);
        assert_eq!(ctx.pitch_modifiers.grass_quality, 100);
        assert_eq!(ctx.extra_time_len, -1);
    }

    #[test]
    fn rng_range_is_uniform_ish_and_deterministic() {
        let mut a = MatchRng::new(1);
        let mut b = MatchRng::new(1);
        for _ in 0..100 {
            assert_eq!(a.range(50), b.range(50));
        }
    }

    #[test]
    fn pre_match_pass_fires_grudge_injury_when_grudge_bits_set() {
        let mut ctx = *MatchCtx::new();
        let home = mk_team(1, 11, 5000);
        let away = mk_team(2, 11, 5000);
        let mut rng = MatchRng::new(42);
        // Grudge for every player vs opp team.
        let grudge = |_p: u32, _t: u32| GrudgeMask { bits: 0x02000000 };
        run_pre_match_pass(&mut ctx, &home, &away, &mut rng, grudge, Some(2.8));
        assert!(ctx.event_queue.iter().any(|e| e.event_type == EVT_SERIOUS_FOUL));
    }

    #[test]
    fn pre_match_pass_stays_quiet_with_clean_masks_and_calm_players() {
        let mut ctx = *MatchCtx::new();
        let mut home = mk_team(1, 11, 4000);
        for p in &mut home.players {
            p.aggression = 3; p.dirtiness = 3; p.bravery = 15; p.injury_proneness = 3;
        }
        let mut away = mk_team(2, 11, 4000);
        for p in &mut away.players {
            p.aggression = 3; p.dirtiness = 3; p.bravery = 15; p.injury_proneness = 3;
        }
        let mut rng = MatchRng::new(42);
        let grudge = |_p: u32, _t: u32| GrudgeMask::default();
        run_pre_match_pass(&mut ctx, &home, &away, &mut rng, grudge, Some(2.8));
        // A few events might still fire (random) but not many.
        assert!(ctx.event_queue.len() <= 3);
    }

    #[test]
    fn match_day_build_groups_by_competition_and_group_id() {
        let mut ctx = MatchDayCtx::default();
        let mk_raw = |g: i32, h: u32, a: u32| RawFixture {
            league_id: 100, group_id: g, home_club_id: h, away_club_id: a,
            comp_id: 100, include_flag: 0x81, cup_round: 0, comp_type_flag: 0,
        };
        let sched = DaySchedule {
            count: 4,
            raw_fixtures: vec![
                mk_raw(1, 10, 20), mk_raw(2, 11, 21),
                mk_raw(1, 12, 22), mk_raw(2, 13, 23),
            ],
        };
        match_day_build(&mut ctx, 0, vec![(100, sched)]);
        assert_eq!(ctx.competitions.len(), 1);
        assert_eq!(ctx.fixtures.len(), 4);
        assert_eq!(ctx.groups.len(), 2);
        // Each fixture starts as "not started" 0xFF.
        assert!(ctx.fixtures.iter().all(|f| f.state == 0xFF));
    }

    #[test]
    fn shot_outcome_low_difficulty_scores_more_than_high_difficulty() {
        let mut rng_low = MatchRng::new(1);
        let mut rng_high = MatchRng::new(1);
        let mut low_goals = 0;
        let mut high_goals = 0;
        for _ in 0..500 {
            let mut s = ShooterMutable { on_pitch: true, ..Default::default() };
            let mut gk = 0u8;
            let mut c = SideShotCounters::default();
            let (o, _) = shot_outcome_resolver(&mut s, &mut gk, &mut c, 0, 1, 5, 3,
                                               0, false, &mut rng_low);
            if o == ShotOutcome::Goal { low_goals += 1; }
        }
        for _ in 0..500 {
            let mut s = ShooterMutable { on_pitch: true, ..Default::default() };
            let mut gk = 0u8;
            let mut c = SideShotCounters::default();
            let (o, _) = shot_outcome_resolver(&mut s, &mut gk, &mut c, 0, 1, 5, 18,
                                               0, false, &mut rng_high);
            if o == ShotOutcome::Goal { high_goals += 1; }
        }
        assert!(low_goals > high_goals,
                "low-difficulty (tap-in) should score more than high-difficulty: {low_goals} vs {high_goals}");
    }

    #[test]
    fn shot_outcome_off_pitch_shooter_never_registers() {
        let mut s = ShooterMutable { on_pitch: false, ..Default::default() };
        let mut gk = 0u8;
        let mut c = SideShotCounters::default();
        let mut rng = MatchRng::new(42);
        let (o, _) = shot_outcome_resolver(&mut s, &mut gk, &mut c, 0, 1, 5, 10, 0,
                                           false, &mut rng);
        // The exe writes xg=10 for off-pitch and doesn't mutate outcome.
        assert_eq!(o, ShotOutcome::Goal); // initial_state passed through
    }

    #[test]
    fn simulate_one_fixture_produces_a_result() {
        let home = mk_team(1, 11, 15000);
        let away = mk_team(2, 11, 10000);
        let r = simulate_one_fixture(&home, &away, 42, Some(2.8));
        // Match completed
        assert!(!r.abandoned, "match should not have abandoned");
        // Some shot activity happened
        assert!(r.home_shots as u16 + r.away_shots as u16 > 0,
                "expected at least one shot across both teams");
        // Higher CA team should produce more shots on average — check reputation
        // by running many matches.
    }

    #[test]
    fn simulate_is_deterministic_from_seed() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let a = simulate_one_fixture(&home, &away, 1234, Some(2.8));
        let b = simulate_one_fixture(&home, &away, 1234, Some(2.8));
        assert_eq!(a.home_score, b.home_score);
        assert_eq!(a.away_score, b.away_score);
        assert_eq!(a.event_log.len(), b.event_log.len());
    }

    #[test]
    fn better_team_wins_more_over_many_matches() {
        let strong = mk_team(1, 11, 18000);
        let weak = mk_team(2, 11, 8000);
        let mut strong_wins = 0;
        let mut weak_wins = 0;
        for seed in 0..100 {
            let r = simulate_one_fixture(&strong, &weak, seed, Some(2.8));
            if r.home_score > r.away_score { strong_wins += 1; }
            else if r.away_score > r.home_score { weak_wins += 1; }
        }
        assert!(strong_wins > weak_wins,
                "stronger team should win more often: {strong_wins} vs {weak_wins}");
    }

    #[test]
    fn token_engine_seed_places_11_starters_per_side() {
        let home = mk_team(1, 15, 12000);
        let away = mk_team(2, 15, 12000);
        let e = TokenEngine::seed(&home, &away);
        assert_eq!(e.tokens[0].len(), TOKENS_PER_SIDE);
        assert_eq!(e.tokens[1].len(), TOKENS_PER_SIDE);
        assert_eq!(e.tokens[0].iter().filter(|t| t.position_slot >= 0).count(), 11);
        assert_eq!(e.tokens[1].iter().filter(|t| t.position_slot >= 0).count(), 11);
    }

    #[test]
    fn token_model_simulation_terminates_and_produces_stats() {
        let home = mk_team(1, 11, 14000);
        let away = mk_team(2, 11, 10000);
        let r = simulate_one_fixture_token_model(&home, &away, 42);
        assert!(!r.abandoned);
        // Some shot activity — even a defensive match has shots.
        let total_shots = r.home_shots as u16 + r.away_shots as u16;
        assert!(total_shots > 0, "expected some shots; got {total_shots}");
    }

    #[test]
    fn token_model_is_deterministic_from_seed() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let a = simulate_one_fixture_token_model(&home, &away, 999);
        let b = simulate_one_fixture_token_model(&home, &away, 999);
        assert_eq!(a.home_score, b.home_score);
        assert_eq!(a.away_score, b.away_score);
        assert_eq!(a.event_log.len(), b.event_log.len());
    }

    #[test]
    fn zone_bits_subtype_maps_to_exe_gate() {
        assert_eq!(ZoneBits { bits: 0x000 }.shot_subtype(), 0x10, "no attacking bits → pass");
        assert_eq!(ZoneBits { bits: 0x208 }.shot_subtype(), 0x12, "shot-third + open lane");
        assert_eq!(ZoneBits { bits: 0x204 }.shot_subtype(), 0x14, "shot-third + clear angle");
    }

    #[test]
    fn dist_lut_matches_exe_dump_at_key_cells() {
        // Verified against the raw dump from cm0102.exe file offset 0x601E20.
        // Diagonals (dy=dx=1) should be 1.41 (≈ sqrt 2).
        assert_eq!(DIST_LUT_00A01E20[1 * 9 + 1], 1.410);
        // Corner (dy=11, dx=8) = 13.6.
        assert_eq!(DIST_LUT_00A01E20[11 * 9 + 8], 13.6);
        // Origin = 0.
        assert_eq!(DIST_LUT_00A01E20[0], 0.0);
        // Cardinal along dy=0.
        assert_eq!(DIST_LUT_00A01E20[3], 3.0);
    }

    #[test]
    fn target_picker_finds_a_cell_when_in_attacking_third() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let mut e = TokenEngine::seed(&home, &away);
        // Move a home attacker deep into the attacking third.
        e.tokens[0][10].zone_x = 4;
        e.tokens[0][10].zone_y = 10;
        e.tokens[0][10].shooting = 15;
        let mut rng = MatchRng::new(1);
        let tok = e.tokens[0][10].clone();
        let target = target_picker(&tok, &e, 0x04, &mut rng);
        assert!(target.is_some(), "attacker in the box should find a shot target");
    }

    #[test]
    fn pass_target_picker_returns_none_if_no_teammates_nearby() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let mut e = TokenEngine::seed(&home, &away);
        // Isolate token 0 far from the rest.
        e.tokens[0][0].zone_x = 8; e.tokens[0][0].zone_y = 11;
        for i in 1..11 { e.tokens[0][i].zone_x = 0; e.tokens[0][i].zone_y = 0; }
        let mut rng = MatchRng::new(1);
        let tok = e.tokens[0][0].clone();
        let picked = pass_target_picker(&tok, &e, &mut rng);
        assert!(picked.is_none(), "isolated carrier should find no pass target");
    }

    #[test]
    fn shot_damage_formula_falls_with_fatigue() {
        let mut t = MatchToken::default();
        t.shooting = 15; t.role_ca = 15;
        let mut rng_a = MatchRng::new(42);
        let mut rng_b = MatchRng::new(42);
        t.stamina_short = 10_000;   // full stamina
        let fresh_roll = ball_command_shot_damage(&t, 0, 0, -1, &mut rng_a);
        t.stamina_short = 5_000;    // half stamina
        let tired_roll = ball_command_shot_damage(&t, 0, 0, -1, &mut rng_b);
        assert!(fresh_roll >= tired_roll,
                "fresh legs should shoot at least as well as tired: {fresh_roll} vs {tired_roll}");
    }

    // -----------------------------------------------------------------
    // shot_attempt_gate (port of FUN_006F99C0) tests.
    // -----------------------------------------------------------------

    /// Builds a token engine with a single home carrier at `(zx, zy)`,
    /// possession set to home, and the ball in home's attacking third.
    fn gate_fixture(zx: i8, zy: i8, shooting: u8) -> (TokenEngine, u8) {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let mut e = TokenEngine::seed(&home, &away);
        let slot: u8 = 3;
        let t = &mut e.tokens[0][slot as usize];
        t.zone_x = zx;
        t.zone_y = zy;
        t.shooting = shooting;
        t.teammate_count = 10;
        t.tackled_cooldown = 0;
        e.carrier = Some((0, slot));
        e.possession_side = 0;
        // ball_in_attacking_third(0) is true when ball_zone_y < 6 (see
        // TokenEngine::ball_in_attacking_third).
        e.ball_zone_y = 3;
        e.grid = PitchGrid::rebuild(&e);
        (e, slot)
    }

    #[test]
    fn shot_attempt_gate_false_when_token_is_not_the_carrier() {
        let (e, slot) = gate_fixture(4, 10, 18);
        let other_slot = if slot == 0 { 1 } else { 0 };
        let mut e = e;
        e.carrier = Some((0, other_slot)); // carrier is a different token
        let tok = e.tokens[0][slot as usize].clone();
        let mut rng = MatchRng::new(7);
        for seed in 0..50u64 {
            rng = MatchRng::new(seed);
            assert!(!shot_attempt_gate(&tok, &e, &mut rng),
                    "non-carrier token must never attempt a shot (exe dispatches \
                     FUN_006F99C0 only for self == M[0xF582])");
        }
    }

    #[test]
    fn shot_attempt_gate_false_when_opponent_has_possession() {
        let (mut e, slot) = gate_fixture(4, 10, 18);
        e.possession_side = 1; // away has the ball, not home
        let tok = e.tokens[0][slot as usize].clone();
        for seed in 0..50u64 {
            let mut rng = MatchRng::new(seed);
            assert!(!shot_attempt_gate(&tok, &e, &mut rng),
                    "carrier's own side must hold possession to shoot");
        }
    }

    #[test]
    fn shot_attempt_gate_close_range_no_pressure_fires_often() {
        // Right in front of goal (dy=1, dx=0 from the goal cell), no
        // defenders occupying the carrier's cell, strong shooting stat.
        let (e, slot) = gate_fixture(4, 10, 18);
        let tok = e.tokens[0][slot as usize].clone();
        let mut fired = 0u32;
        let trials = 300u64;
        for seed in 0..trials {
            let mut rng = MatchRng::new(seed * 97 + 1);
            if shot_attempt_gate(&tok, &e, &mut rng) { fired += 1; }
        }
        let rate = fired as f64 / trials as f64;
        assert!(rate > 0.5,
                "close range + no pressure should fire on most ticks, got rate={rate}");
    }

    #[test]
    fn shot_attempt_gate_far_range_heavy_marking_rarely_fires() {
        // Deep in the carrier's own half (dy=11 from the opponent goal),
        // and swarmed by four opposing markers on the same cell.
        let (mut e, slot) = gate_fixture(4, 0, 18);
        for i in 0..4 {
            e.tokens[1][i].position_slot = i as i8;
            e.tokens[1][i].zone_x = 4;
            e.tokens[1][i].zone_y = 0;
        }
        e.grid = PitchGrid::rebuild(&e);
        let tok = e.tokens[0][slot as usize].clone();
        let mut fired = 0u32;
        let trials = 300u64;
        for seed in 0..trials {
            let mut rng = MatchRng::new(seed * 97 + 1);
            if shot_attempt_gate(&tok, &e, &mut rng) { fired += 1; }
        }
        let rate = fired as f64 / trials as f64;
        assert!(rate < 0.05,
                "far range + heavy marking should almost never fire, got rate={rate}");
    }

    #[test]
    fn shot_attempt_gate_blocked_by_tackled_cooldown() {
        // Same favourable geometry as the close-range test, but the
        // carrier is fresh off a tackle (+0x10D cool-down maxed).
        let (mut e, slot) = gate_fixture(4, 10, 18);
        e.tokens[0][slot as usize].tackled_cooldown = 255;
        let tok = e.tokens[0][slot as usize].clone();
        let mut fired = 0u32;
        let trials = 300u64;
        for seed in 0..trials {
            let mut rng = MatchRng::new(seed * 97 + 1);
            if shot_attempt_gate(&tok, &e, &mut rng) { fired += 1; }
        }
        assert_eq!(fired, 0,
                    "maxed tackled-cooldown must always block the shot \
                     (exe: tackled_cooldown < rand(10) never holds at 255)");
    }

    #[test]
    fn shot_attempt_gate_respects_attacking_third_zone_gate() {
        // Favourable geometry and possession, but the ball itself is not
        // in the attacking third for this side.
        let (mut e, slot) = gate_fixture(4, 10, 18);
        e.ball_zone_y = 9; // not < 6, so ball_in_attacking_third(0) is false
        let tok = e.tokens[0][slot as usize].clone();
        for seed in 0..50u64 {
            let mut rng = MatchRng::new(seed);
            assert!(!shot_attempt_gate(&tok, &e, &mut rng),
                    "shot branch is only reachable while the ball is in the \
                     attacking third");
        }
    }

    #[test]
    fn ball_in_opp_box_matches_exe_geometry() {
        let mut e = TokenEngine::default();
        e.ball_zone_x = 4; e.ball_zone_y = 11;
        assert!(e.ball_in_opp_box(1), "home penalty box for away side");
        e.ball_zone_y = 0;
        assert!(e.ball_in_opp_box(0), "away penalty box for home side");
        e.ball_zone_x = 1;
        assert!(!e.ball_in_opp_box(0), "corner-flag column excluded");
    }

    #[test]
    fn zone_pool_striker_gets_attacking_bits() {
        let pool = ZoneAttributePool::build(
            crate::formation::FormationCode::F442,
            crate::formation::FormationCode::F442);
        // 4-4-2: outfield slots 0..3 = defenders, 4..7 = mids, 8..9 = strikers.
        let striker_bits = pool.get(0, 8);
        assert!(striker_bits.in_shot_third(), "striker should carry 0x200 (attacking third)");
        assert!(striker_bits.clear_angle(), "striker should carry 0x004 (clear angle)");
        let defender_bits = pool.get(0, 0);
        assert!(!defender_bits.in_shot_third(), "defender should not have attacking bits");
    }

    #[test]
    fn pitch_grid_rebuild_places_tokens_in_correct_cell() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let mut e = TokenEngine::seed(&home, &away);
        // Nudge a specific home token to (4, 5).
        e.tokens[0][5].zone_x = 4;
        e.tokens[0][5].zone_y = 5;
        let g = PitchGrid::rebuild(&e);
        assert!(g.cells[5][4].home_occupants.contains(&5));
    }

    #[test]
    fn bearing_atan2_maps_cardinal_directions() {
        assert_eq!(bearing_atan2(1, 0), 0);         // east
        assert_eq!(bearing_atan2(0, 1), 90);        // north (in atan2 convention here)
        assert_eq!(bearing_atan2(-1, 0), 180);
        assert!(bearing_atan2(1, 1) > 30 && bearing_atan2(1, 1) < 60,
                "NE should be near 45°");
    }

    #[test]
    fn reachability_true_when_already_facing_target() {
        // Facing east (bearing 0) toward (1, 0) from (0, 0).
        assert!(reachability_check(0, 0, 1, 0, 0, 0));
    }

    #[test]
    fn reachability_false_when_facing_wrong_way_with_low_rotation() {
        // Facing east, target is west — 180° difference, rotation 10 → false.
        assert!(!reachability_check(0, 0, -1, 0, 0, 10));
    }

    #[test]
    fn bias_step_advances_one_cell_toward_target() {
        let mut rng = MatchRng::new(0);
        let (nx, ny) = bias_step(0, 0, 5, 5, 0, true, &mut rng);
        assert!(nx > 0 && ny > 0, "step should advance toward target");
    }

    #[test]
    fn classifier_returns_attempt_tag_when_no_defenders() {
        let mut rng = MatchRng::new(1);
        let mut t = MatchToken::default();
        t.shooting = 15; t.technique = 15; t.role_ca = 15;
        let out = classify_shot_outcome(&t, ClassifierCmd::Shot, 4, 11, &[], 0, &mut rng);
        // No defenders → attempt tag (0x21 = ShotAttempt) or one of the
        // 4-way outcome variants; test that it's NOT PassIntercepted.
        assert!(out != ClassifierResult::PassIntercepted);
    }

    #[test]
    fn classifier_produces_deterministic_output_from_seed() {
        let mut rng_a = MatchRng::new(42);
        let mut rng_b = MatchRng::new(42);
        let mut t = MatchToken::default();
        t.shooting = 10; t.role_ca = 10;
        let a = classify_shot_outcome(&t, ClassifierCmd::Shot, 4, 11, &[], 0, &mut rng_a);
        let b = classify_shot_outcome(&t, ClassifierCmd::Shot, 4, 11, &[], 0, &mut rng_b);
        assert_eq!(a, b);
    }

    #[test]
    fn reputation_to_stars_clamps_at_1_and_scales_by_500() {
        assert_eq!(reputation_to_stars(0), 1, "zero rep → clamped to 1");
        assert_eq!(reputation_to_stars(499), 1, "just below 500 → 1");
        assert_eq!(reputation_to_stars(500), 1);
        assert_eq!(reputation_to_stars(1000), 2);
        assert_eq!(reputation_to_stars(5000), 10);
    }

    #[test]
    fn cup_comp_types_add_2_to_star_ratings() {
        // League match (comp_type 1) — no cup bonus.
        let league = PitchFixtureInput {
            home_reputation: 5000, away_reputation: 2500,
            competition_present: true, competition_type: 1,
        };
        assert_eq!(pitch_star_ratings(league), (10, 5));
        // Cup match (comp_type 6) — +2 to both.
        let cup = PitchFixtureInput { competition_type: 6, ..league };
        assert_eq!(pitch_star_ratings(cup), (12, 7));
        // Types 8 and 10 also.
        assert_eq!(pitch_star_ratings(PitchFixtureInput { competition_type: 8, ..league }),
                   (12, 7));
        assert_eq!(pitch_star_ratings(PitchFixtureInput { competition_type: 10, ..league }),
                   (12, 7));
    }

    #[test]
    fn pitch_modifiers_from_null_fixture_return_defaults() {
        let m = pitch_modifiers_from_fixture(None);
        assert_eq!(m.grass_quality, 100);
        assert_eq!(m.firmness, 90);
        assert_eq!(m.weather_effect, 10);
    }

    #[test]
    fn pitch_modifiers_from_high_rep_fixture_boost_grass_quality() {
        let f = PitchFixtureInput {
            home_reputation: 10000, away_reputation: 8000,
            competition_present: true, competition_type: 6,   // cup adds +2
        };
        let m = pitch_modifiers_from_fixture(Some(f));
        // home_stars = (10000/500)+2 = 22 → capped 20 → quality boost = 40.
        assert!(m.grass_quality > 100, "high-rep + cup should boost grass quality");
    }

    #[test]
    fn derby_matches_slot_1_returns_1() {
        let rivals: RivalTable = [
            RivalSlot { rival_id: 100, rival_type: 5 },
            RivalSlot { rival_id: 200, rival_type: 7 },
            RivalSlot { rival_id: 300, rival_type: 9 },
        ];
        // team_b_type == 5 → matches slot 0 → returns 1 (local rival).
        let score = derby_score(rivals, 5, 0, 0, |a, b| a == b);
        assert_eq!(score, 1);
    }

    #[test]
    fn derby_matches_slot_2_returns_2() {
        let rivals: RivalTable = [
            RivalSlot { rival_id: 100, rival_type: 5 },
            RivalSlot { rival_id: 200, rival_type: 7 },
            RivalSlot { rival_id: 300, rival_type: 9 },
        ];
        // team_b_type == 7 → matches slot 1 → returns 2.
        assert_eq!(derby_score(rivals, 7, 0, 0, |a, b| a == b), 2);
    }

    #[test]
    fn derby_matches_slot_3_returns_3() {
        let rivals: RivalTable = [
            RivalSlot { rival_id: 100, rival_type: 5 },
            RivalSlot { rival_id: 200, rival_type: 7 },
            RivalSlot { rival_id: 300, rival_type: 9 },
        ];
        assert_eq!(derby_score(rivals, 9, 0, 0, |a, b| a == b), 3);
    }

    #[test]
    fn derby_no_match_returns_default_2() {
        let rivals: RivalTable = [
            RivalSlot { rival_id: 100, rival_type: 5 },
            RivalSlot { rival_id: 200, rival_type: 7 },
            RivalSlot { rival_id: 300, rival_type: 9 },
        ];
        // No matching type → returns 2 (default weak grudge).
        assert_eq!(derby_score(rivals, 999, 0, 0, |a, b| a == b), 2);
    }

    #[test]
    fn derby_zero_type_uses_default_when_reverse_flag_zero() {
        let rivals: RivalTable = [
            RivalSlot { rival_id: 100, rival_type: 5 },
            RivalSlot { rival_id: 200, rival_type: 7 },
            RivalSlot { rival_id: 300, rival_type: 9 },
        ];
        // team_b_type = 0 + reverse_flag = 0 + default_type = 7 →
        // slot 1 matches → returns 2.
        assert_eq!(derby_score(rivals, 0, 0, 7, |a, b| a == b), 2);
    }

    #[test]
    fn derby_offset_constants_match_exe_field_layout() {
        assert_eq!(team_rival_offsets::SLOT_0_ID, 0x83);
        assert_eq!(team_rival_offsets::SLOT_0_TYPE, 0x87);
        assert_eq!(team_rival_offsets::SLOT_1_ID, 0x8B);
        assert_eq!(team_rival_offsets::SLOT_1_TYPE, 0x8F);
        assert_eq!(team_rival_offsets::SLOT_2_ID, 0x93);
        assert_eq!(team_rival_offsets::SLOT_2_TYPE, 0x97);
    }

    #[test]
    fn cell_move_incrementally_updates_grid() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let mut e = TokenEngine::seed(&home, &away);
        // Initial: token 3 at its seed position.
        let (init_x, init_y) = {
            let t = &e.tokens[0][3];
            (t.zone_x, t.zone_y)
        };
        // Rebuild grid so it reflects seeded positions.
        e.grid = PitchGrid::rebuild(&e);
        assert!(e.grid.cells[init_y as usize][init_x as usize]
                 .home_occupants.contains(&3));
        // Now move token 3 incrementally.
        assert!(e.cell_move(0, 3, 6, 8));
        // Old cell should no longer list slot 3.
        assert!(!e.grid.cells[init_y as usize][init_x as usize]
                  .home_occupants.contains(&3));
        // New cell should list slot 3.
        assert!(e.grid.cells[8][6].home_occupants.contains(&3));
    }

    #[test]
    fn cell_move_swap_remove_is_o1() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let mut e = TokenEngine::seed(&home, &away);
        e.grid = PitchGrid::rebuild(&e);
        // Place 3 home tokens in the same cell.
        for slot in [3, 4, 5] {
            e.tokens[0][slot].zone_x = 4;
            e.tokens[0][slot].zone_y = 5;
        }
        e.grid = PitchGrid::rebuild(&e);
        let cell = &e.grid.cells[5][4];
        assert_eq!(cell.home_occupants.len(), 3);
        // Move slot 4 (middle) — should leave slots 3, 5 (in some order).
        assert!(e.cell_move(0, 4, 2, 2));
        let after = &e.grid.cells[5][4];
        assert_eq!(after.home_occupants.len(), 2);
        assert!(!after.home_occupants.contains(&4));
    }

    #[test]
    fn cell_move_updates_token_zone() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let mut e = TokenEngine::seed(&home, &away);
        let old_x = e.tokens[0][3].zone_x;
        let old_y = e.tokens[0][3].zone_y;
        let _ = (old_x, old_y);
        assert!(e.cell_move(0, 3, 4, 5));
        assert_eq!(e.tokens[0][3].zone_x, 4);
        assert_eq!(e.tokens[0][3].zone_y, 5);
    }

    #[test]
    fn cell_move_noop_when_already_at_target() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let mut e = TokenEngine::seed(&home, &away);
        let x = e.tokens[0][3].zone_x;
        let y = e.tokens[0][3].zone_y;
        assert!(!e.cell_move(0, 3, x, y),
                "exe's early return: `if (new==old) noop`");
    }

    #[test]
    fn cell_move_updates_ball_zone_when_carrier() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let mut e = TokenEngine::seed(&home, &away);
        e.carrier = Some((0, 3));
        assert!(e.cell_move(0, 3, 4, 5));
        assert_eq!(e.ball_zone_x, 4);
        assert_eq!(e.ball_zone_y, 5);
    }

    #[test]
    fn cell_move_non_carrier_leaves_ball_zone_alone() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let mut e = TokenEngine::seed(&home, &away);
        e.carrier = Some((0, 5));   // different token
        e.ball_zone_x = 7; e.ball_zone_y = 3;
        assert!(e.cell_move(0, 3, 1, 8));
        assert_eq!(e.ball_zone_x, 7);
        assert_eq!(e.ball_zone_y, 3);
    }

    #[test]
    fn cell_capacity_is_11_per_side() {
        let e = TokenEngine::default();
        assert!(!e.cell_at_capacity(0, 0, 0), "empty cell not at capacity");
        assert_eq!(SLOTS_PER_SIDE_PER_CELL, 11);
        assert_eq!(CELL_STRIDE, 0x5A);
        assert_eq!(CELL_COUNT_OFFSET_SIDE0, 0x58);
        assert_eq!(CELL_COUNT_OFFSET_SIDE1, 0x59);
    }

    #[test]
    fn cell_move_out_of_range_slot_returns_false() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let mut e = TokenEngine::seed(&home, &away);
        assert!(!e.cell_move(0, 999, 4, 5));
    }

    #[test]
    fn post_match_rating_stays_in_bounds() {
        for seed in 0..50 {
            let mut rng = MatchRng::new(seed);
            let r = post_match_rating(2, 4, 5000, &mut rng);
            assert!((1..=10).contains(&r), "rating {r} out of range");
        }
    }

    #[test]
    fn post_match_rating_high_counter_a_pushes_down() {
        let mut rng = MatchRng::new(1);
        // raw 5000/1000 = 5, counter_a >= 3 → -2 → 3. Below 4 with RNG likely non-zero → clamped to 4.
        let r = post_match_rating(5, 3, 5000, &mut rng);
        assert!(r <= 5);
    }

    #[test]
    fn post_match_rating_zero_counter_a_pushes_up() {
        let mut rng = MatchRng::new(1);
        // raw 5000/1000 = 5, counter_a == 0 → +1 = 6, counter_b >= 6? no → 0 → stays 6.
        let r = post_match_rating(0, 5, 5000, &mut rng);
        assert!(r >= 5);
    }

    #[test]
    fn atmosphere_null_row_returns_default_5000() {
        let mut rng = MatchRng::new(1);
        let a = compute_atmosphere(0, None, 999, &mut rng);
        assert_eq!(a.factor, ATMOSPHERE_DEFAULT);
        assert_eq!(a.factor, 5000);
        assert_eq!(a.row_ptr, 0);
    }

    #[test]
    fn atmosphere_very_loud_bucket_matches_exe_hex() {
        // Cover: 7750=0x1E46, 7250=0x1C52, 6750=0x1A5E, 6250=0x186A,
        // 5750=0x1676, 5250=0x1482.
        assert_eq!(ATMOSPHERE_VERY_LOUD, 7750);
        assert_eq!(ATMOSPHERE_LOUD, 7250);
        assert_eq!(ATMOSPHERE_NORMAL_LOUD, 6750);
        assert_eq!(ATMOSPHERE_NORMAL, 6250);
        assert_eq!(ATMOSPHERE_QUIET, 5750);
        assert_eq!(ATMOSPHERE_VERY_QUIET, 5250);
    }

    #[test]
    fn atmosphere_high_quality_venue_biases_towards_loud() {
        // Quality 400 > all thresholds (350/300/250/200/150) — hits
        // very-loud roughly 100% of time on a single seed.
        let mut loud_hits = 0;
        for seed in 0..100 {
            let mut rng = MatchRng::new(seed);
            let a = compute_atmosphere(0, Some(0xDEAD_BEEF), 400, &mut rng);
            assert_eq!(a.row_ptr, 0xDEAD_BEEF);
            if a.factor >= ATMOSPHERE_LOUD { loud_hits += 1; }
        }
        assert!(loud_hits > 80, "high-quality venue should be loud ≥80/100: got {loud_hits}");
    }

    #[test]
    fn atmosphere_zero_quality_venue_stays_very_quiet() {
        // Quality 0 < all thresholds (rand always ≥ 0) → very-quiet.
        for seed in 0..20 {
            let mut rng = MatchRng::new(seed);
            let a = compute_atmosphere(0, Some(0x1234), 0, &mut rng);
            assert_eq!(a.factor, ATMOSPHERE_VERY_QUIET);
        }
    }

    #[test]
    fn atmosphere_medium_quality_hits_middle_buckets() {
        // Quality 200 — should sometimes hit 6750/6250 but rarely 7750.
        let mut buckets = [0usize; 6];
        for seed in 0..500 {
            let mut rng = MatchRng::new(seed);
            let a = compute_atmosphere(0, Some(0x1000), 200, &mut rng);
            let idx = match a.factor {
                ATMOSPHERE_VERY_LOUD => 0,
                ATMOSPHERE_LOUD => 1,
                ATMOSPHERE_NORMAL_LOUD => 2,
                ATMOSPHERE_NORMAL => 3,
                ATMOSPHERE_QUIET => 4,
                _ => 5,
            };
            buckets[idx] += 1;
        }
        // Quality 200: rand(350)<200 hits ~57% → very_loud dominates.
        // The gates are cumulative, so once one passes we stop.
        assert!(buckets[0] + buckets[1] + buckets[2] + buckets[3] > 400,
                "middle-quality should reach at least one loud/normal bucket");
    }

    #[test]
    fn wing_delta_matches_exe_signed_distance() {
        // Exe: side==1 → away_y - y; side==0 → y - home_y.
        let gl = GoalLines { home_y: 0, away_y: 11 };
        assert_eq!(TokenEngine::wing_delta(gl, 1, 3), 8);   // 11 - 3
        assert_eq!(TokenEngine::wing_delta(gl, 0, 3), 3);   // 3 - 0
        assert_eq!(TokenEngine::wing_delta(gl, 1, 11), 0);  // at away goal
        assert_eq!(TokenEngine::wing_delta(gl, 0, 0), 0);   // at home goal
    }

    #[test]
    fn in_opp_penalty_box_matches_exe_geometry() {
        let mut t = MatchToken::default();
        // Away box (opposing side==1): zx in [2..6], zy in [10..11].
        t.zone_x = 4; t.zone_y = 10;
        assert!(t.in_opp_penalty_box(1));
        t.zone_y = 11;
        assert!(t.in_opp_penalty_box(1));
        t.zone_y = 9;
        assert!(!t.in_opp_penalty_box(1));
        // Home box (side==0): zx in [2..6], zy in [0..1].
        t.zone_y = 0;
        assert!(t.in_opp_penalty_box(0));
        t.zone_y = 1;
        assert!(t.in_opp_penalty_box(0));
        // Corner-flag col excluded.
        t.zone_x = 1; t.zone_y = 0;
        assert!(!t.in_opp_penalty_box(0));
    }

    #[test]
    fn tactical_zone_membership_bits_1_reads_control_ratings() {
        let e = TokenEngine::default();
        // zone_bits == 1 with side_coin==1 uses control_away.
        assert!(e.tactical_zone_membership((0, 0), 1, 1, 50, 200, None, None));
        // control_away == 0 → false.
        assert!(!e.tactical_zone_membership((0, 0), 1, 1, 50, 0, None, None));
    }

    #[test]
    fn tactical_zone_membership_zone_bits_other_reads_zone_model_slot() {
        let e = TokenEngine::default();
        // Side coin 1 → check defending zone slot.
        assert!(e.tactical_zone_membership((1, 5), 1, 0, 0, 0, None, Some((1, 5))));
        assert!(!e.tactical_zone_membership((1, 6), 1, 0, 0, 0, None, Some((1, 5))));
        // Side coin 0 → check attacking zone slot.
        assert!(e.tactical_zone_membership((0, 8), 0, 0, 0, 0, Some((0, 8)), None));
    }

    #[test]
    fn wing_delta_for_token_returns_zero_when_carrier() {
        let home = mk_team(1, 11, 12000);
        let away = mk_team(2, 11, 12000);
        let mut e = TokenEngine::seed(&home, &away);
        e.carrier = Some((0, 3));
        e.tokens[0][3].zone_y = 7;
        let gl = GoalLines { home_y: 0, away_y: 11 };
        assert_eq!(e.wing_delta_for_token(gl, 0, (0, 3)), 0,
                    "carrier delta is 0 per exe");
        assert_eq!(e.wing_delta_for_token(gl, 0, (0, 4)), 5,   // token[4].zone_y = 5
                    "non-carrier uses zone_y - home_y");
    }

    #[test]
    fn match_day_build_skips_filtered_fixtures() {
        let mut ctx = MatchDayCtx::default();
        let sched = DaySchedule {
            count: 3,
            raw_fixtures: vec![
                RawFixture { league_id: 1, group_id: 0, home_club_id: 1, away_club_id: 2,
                             comp_id: 1, include_flag: 0x00, cup_round: 0, comp_type_flag: 0 },
                RawFixture { league_id: 1, group_id: 0, home_club_id: 3, away_club_id: 4,
                             comp_id: 1, include_flag: 0xFE, cup_round: 0, comp_type_flag: 0 },
                RawFixture { league_id: 1, group_id: 0, home_club_id: 5, away_club_id: 6,
                             comp_id: 1, include_flag: 0x81, cup_round: 0, comp_type_flag: 0 },
            ],
        };
        match_day_build(&mut ctx, 0, vec![(1, sched)]);
        assert_eq!(ctx.fixtures.len(), 1, "only the includable fixture should survive");
        assert_eq!(ctx.fixtures[0].home_club_id, 5);
    }
}
