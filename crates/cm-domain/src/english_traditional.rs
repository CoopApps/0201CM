//! C11 — Production dispatch for the exact English Traditional fixture
//! engine. Wraps the byte-exact archaeology in `eng_second_fixtures`
//! and `exe_date` in a single production-facing API that
//! `generate_new_game_season` uses for the 5 English simulated
//! leagues (Premier 7, First 8, Second 9, Third 10, Conference 93).
//!
//! # Scope
//!
//! * Traditional mode only. V4 has its own path (see
//!   [`crate::eng_second_fixtures::EnglishLeagueSpec`] archaeology
//!   docs and the game-mode note in memory).
//! * Fixture GENERATION only. Rollover / promotion / relegation
//!   apply-layer wiring is separate work (C7/C8/C9).
//! * Feeder pools (Isthmian 358, Southern 359, Northern 360, "A
//!   Lower Division" 357) are NOT English simulated tiers — they
//!   stay on the current implementation and are not routed here.
//!
//! # Design
//!
//! The archaeology module `eng_second_fixtures` carries a rich
//! [`EnglishLeagueSpec`] with GDI VAs, per-component confidence
//! labels and source-cpp names — those are useful at test/proof
//! time but wrong for production coupling. This module defines
//! a runtime-lean [`EnglishRuntimeSpec`] that carries only what
//! the generator actually reads at runtime, plus a production
//! entry point [`generate_english_traditional_league`] that runs
//! the shared `run_round_robin_driver` (with `matrix_perturb` +
//! `walker_step` inside) against a resolver built from real
//! [`crate::DomainStadium`] data.
//!
//! The dispatch layer [`is_english_traditional_league`] guards
//! [`generate_english_traditional_league`] against non-English
//! comp ids and against V4 mode.
//!
//! # RNG contract
//!
//! The generator takes a **shared** `&mut GameRng`. Callers running
//! the 5 English leagues at boot MUST pass one instance through all
//! five calls, in the exe's boot order (Premier → First → Second →
//! Third → Conference). C10.11 proved the exe's RNG stream is
//! continuous across the 5 leagues; matching that here preserves
//! algorithmic byte-exact reproduction from a shared boot RNG state.
//!
//! The generator never seeds or resets the RNG internally; the
//! only reseed points are the two `lcg_srand` calls inside
//! `matrix_perturb`, which are byte-exact against the exe.

use crate::eng_second_fixtures::{
    matrix_perturb, run_round_robin_driver, EnglishLeague, FixtureEmission,
    PerturbConstants, StadiumClubResolver,
};
use crate::exe_date::{
    build_league_schedule_from_intermediate, RoundIntermediate,
    ENG_PREM_2001_ROUNDS, ENG_FIRST_2001_ROUNDS, ENG_SECOND_2001_ROUNDS,
    ENG_THIRD_2001_ROUNDS, ENG_CONF_2001_ROUNDS,
};
use crate::game_rng::GameRng;
use crate::{
    is_leap_year, CmPackedDate, DomainCompetition, GameDate,
    HeadlessFixtureStatus, HeadlessSeasonFixture,
};

/// Top-level game mode discriminator — the mutually-exclusive choice
/// documented in [[game-mode-traditional-vs-v4]]. Kept here because
/// C11.1 must be able to prove Traditional and V4 dispatch through
/// distinct code paths; a full `GameMode` model landing project-wide
/// is a separate concern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    /// 100% faithful CM01/02. Byte-exact goldens apply. Runs the
    /// exact English fixture engine for 7/8/9/10/93.
    Traditional,
    /// Opt-in extended simulation. Has its own dispatch and MUST
    /// NOT reuse the Traditional exact fixture engine as-is.
    V4,
}

impl Default for GameMode {
    fn default() -> Self { GameMode::Traditional }
}

/// Deliberate dispatch decision for one competition at new-game
/// season build. C11.1 point 8: replaces the fragile "did the exact
/// loop consume this? did the generic loop skip it?" pair with a
/// single explicit answer per competition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnglishFixtureDispatch {
    /// Run through
    /// [`generate_english_traditional_league`] — the exact ported
    /// perturb + walker + driver + native schedule engine.
    ExactEnglish,
    /// Not routed through this module; leave to the generic path
    /// or any other dedicated engine that already claims the id.
    Generic,
    /// This module explicitly refuses the comp (mode gate,
    /// unsupported base year for an English comp, etc.).
    Skipped { reason: &'static str },
}

/// Errors from the exact English fixture engine. Prefer these
/// over silent fallback so a broken exact path does not masquerade
/// as a successful Berger run (C11.1 point 7).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExactEnglishGenerationError {
    /// Club count on the incoming roster does not match the spec's
    /// `n_clubs` for this comp id. Should be treated as a hard
    /// invariant break — the 5 English simulated leagues have fixed
    /// shapes in Traditional mode.
    UnexpectedClubCount {
        comp_id: u32,
        expected: usize,
        actual: usize,
    },
    /// A `base_year` other than the one this engine has proven
    /// templates for was requested. The 2001/02 templates are
    /// verified byte-exact against the shipped-year GDI capture;
    /// applying them to another year without further archaeology
    /// would produce silently wrong dates.
    UnsupportedBaseYear {
        comp_id: u32,
        base_year: u16,
        supported: &'static [u16],
    },
    /// The spec table's `n_rounds` is out of alignment with
    /// `rounds_2001.len()`. Sanity guard — should be unreachable
    /// on the shipped consts.
    ScheduleTemplateShapeMismatch {
        comp_id: u32,
        expected_rounds: u16,
        template_rounds: usize,
    },
}

impl std::fmt::Display for ExactEnglishGenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedClubCount { comp_id, expected, actual } => write!(
                f,
                "exact English engine: comp {comp_id} expected {expected} clubs, got {actual}"
            ),
            Self::UnsupportedBaseYear { comp_id, base_year, supported } => write!(
                f,
                "exact English engine: comp {comp_id} base_year {base_year} not in supported set {supported:?}"
            ),
            Self::ScheduleTemplateShapeMismatch { comp_id, expected_rounds, template_rounds } => write!(
                f,
                "exact English engine: comp {comp_id} spec.n_rounds={expected_rounds} but rounds_2001 has {template_rounds} entries"
            ),
        }
    }
}

impl std::error::Error for ExactEnglishGenerationError {}

/// Base years the current template set is proven exact for.
/// C11.1 point 14: the shipped 2001/02 templates are byte-exact
/// only for that season year. Other years land in
/// [`EnglishFixtureDispatch::Skipped`].
/// Base years whose output has been diffed byte-for-byte against a
/// real GDI capture. Only the shipped 2001/02 season has a capture, so
/// only 2001 is *verified* — see `tests/c11_1_production_fixture_golden.rs`.
///
/// This is a statement about evidence, NOT about capability: the engine
/// (perturb → walker → driver) is year-parameterised and runs for any
/// season. Do not use this list as a dispatch gate.
pub const EXACT_SUPPORTED_BASE_YEARS: &[u16] = &[2001];

/// Earliest season the engine will build. The shipped database starts
/// in 2001/02; anything earlier is a caller bug.
pub const EARLIEST_SUPPORTED_BASE_YEAR: u16 = 2001;

/// C11.1 point 8 — the single dispatcher decision function. One
/// clear answer to "which engine owns this competition?".
///
/// Traditional mode routes 7 / 8 / 9 / 10 / 93 to the exact
/// English engine iff the base_year is in
/// [`EXACT_SUPPORTED_BASE_YEARS`]. Everything else -> Generic.
/// V4 mode NEVER routes through this module.
pub fn english_dispatch_decision(
    mode: GameMode,
    comp_id: u32,
    base_year: u16,
) -> EnglishFixtureDispatch {
    if !matches!(mode, GameMode::Traditional) {
        return EnglishFixtureDispatch::Skipped {
            reason: "V4 mode does not use the Traditional English engine",
        };
    }
    if !is_english_traditional_league(comp_id) {
        return EnglishFixtureDispatch::Generic;
    }
    // Any season from the database's first onward routes to the exact
    // engine.
    //
    // This used to gate on `EXACT_SUPPORTED_BASE_YEARS` (= [2001]),
    // which broke the game after one season: the Jan-1 season roll asks
    // for `base_year + 1`, got `Skipped`, and the regen drain silently
    // appended nothing. A save could therefore never have a second
    // season of English fixtures. Gating *dispatch* on which years
    // happen to have a capture conflated "unverified" with
    // "unsupported".
    //
    // Seasons after 2001/02 reuse the 2001 round-date template for
    // matchday dates, so they are semantically right but not
    // capture-verified — recorded as LIVE BUT APPROXIMATE in the
    // integration ledger. The 2001 path is unchanged and still byte-exact
    // against the GDI capture.
    if base_year < EARLIEST_SUPPORTED_BASE_YEAR {
        return EnglishFixtureDispatch::Skipped {
            reason: "base_year predates the shipped database (2001/02)",
        };
    }
    EnglishFixtureDispatch::ExactEnglish
}

/// The 5 English Traditional simulated leagues, in exe boot order.
///   `[Premier, First, Second, Third, Conference]`
///   `[7, 8, 9, 10, 93]`
pub const ENGLISH_TRADITIONAL_COMP_IDS: [u32; 5] = [7, 8, 9, 10, 93];

/// True iff `comp_id` is one of the 5 English Traditional simulated
/// leagues. Used by the season builder to route to
/// [`generate_english_traditional_league`] instead of the generic
/// Berger-plus-date-overlay path.
///
/// **Excludes** the English feeder pools (358 Isthmian, 359 Southern,
/// 360 Northern, 357 "A Lower Division"): those are non-simulated
/// static feeders (see [[english-pyramid-final-graph]] memory note).
pub fn is_english_traditional_league(comp_id: u32) -> bool {
    matches!(comp_id, 7 | 8 | 9 | 10 | 93)
}

/// Runtime-lean spec — the DATA the generator actually reads at
/// runtime. VA addresses, source cpp names, confidence labels and
/// archaeology notes live in [`crate::eng_second_fixtures::EnglishLeagueSpec`].
#[derive(Debug, Clone, Copy)]
pub struct EnglishRuntimeSpec {
    /// Which league (for logging + downstream dispatch).
    pub league: EnglishLeague,
    /// Shipped `comp.dat` id.
    pub comp_id: u32,
    /// Number of clubs.
    pub n_clubs: u16,
    /// Number of rounds. Always `(n_clubs - 1) * 2` for the 5
    /// English leagues (double round-robin).
    pub n_rounds: u16,
    /// Matches per pair. Always 2 for the 5 English leagues.
    pub matches_per_pair: u16,
    /// Walker / comp+0xd9 flag byte. All 5 English leagues capture
    /// `d9_flags = 0x0003` so bit 0x40/0x80 are clear and walker
    /// runs the full state machine (verified 20260914_180417
    /// capture across all five). Kept as a field so the type is
    /// still the correct shape if we later find a per-league
    /// variation.
    pub walker_flag_byte: u8,
    /// 2001/02 post-snap intermediate rounds — the exact per-round
    /// (day-of-year, year_off, type_byte, field_c) tuples the exe's
    /// schedule getter writes into the round buffer. Multi-year
    /// support for the other four leagues is a follow-up (only
    /// Second currently has full pre-snap templates).
    pub rounds_2001: &'static [RoundIntermediate],
}

pub const ENGLISH_PREMIER_RUNTIME: EnglishRuntimeSpec = EnglishRuntimeSpec {
    league: EnglishLeague::Premier,
    comp_id: 7,
    n_clubs: 20,
    n_rounds: 38,
    matches_per_pair: 2,
    walker_flag_byte: 3,
    rounds_2001: &ENG_PREM_2001_ROUNDS,
};

pub const ENGLISH_FIRST_RUNTIME: EnglishRuntimeSpec = EnglishRuntimeSpec {
    league: EnglishLeague::First,
    comp_id: 8,
    n_clubs: 24,
    n_rounds: 46,
    matches_per_pair: 2,
    walker_flag_byte: 3,
    rounds_2001: &ENG_FIRST_2001_ROUNDS,
};

pub const ENGLISH_SECOND_RUNTIME: EnglishRuntimeSpec = EnglishRuntimeSpec {
    league: EnglishLeague::Second,
    comp_id: 9,
    n_clubs: 24,
    n_rounds: 46,
    matches_per_pair: 2,
    walker_flag_byte: 3,
    rounds_2001: &ENG_SECOND_2001_ROUNDS,
};

pub const ENGLISH_THIRD_RUNTIME: EnglishRuntimeSpec = EnglishRuntimeSpec {
    league: EnglishLeague::Third,
    comp_id: 10,
    n_clubs: 24,
    n_rounds: 46,
    matches_per_pair: 2,
    walker_flag_byte: 3,
    rounds_2001: &ENG_THIRD_2001_ROUNDS,
};

pub const ENGLISH_CONFERENCE_RUNTIME: EnglishRuntimeSpec = EnglishRuntimeSpec {
    league: EnglishLeague::Conference,
    comp_id: 93,
    n_clubs: 22,
    n_rounds: 42,
    matches_per_pair: 2,
    walker_flag_byte: 3,
    rounds_2001: &ENG_CONF_2001_ROUNDS,
};

/// The 5 specs in boot order.
pub const ENGLISH_RUNTIME_SPECS: [&EnglishRuntimeSpec; 5] = [
    &ENGLISH_PREMIER_RUNTIME,
    &ENGLISH_FIRST_RUNTIME,
    &ENGLISH_SECOND_RUNTIME,
    &ENGLISH_THIRD_RUNTIME,
    &ENGLISH_CONFERENCE_RUNTIME,
];

/// Look up a runtime spec by `comp_id`. Returns `None` for anything
/// outside the 5 English Traditional leagues.
pub fn english_runtime_spec_for(comp_id: u32) -> Option<&'static EnglishRuntimeSpec> {
    match comp_id {
        7  => Some(&ENGLISH_PREMIER_RUNTIME),
        8  => Some(&ENGLISH_FIRST_RUNTIME),
        9  => Some(&ENGLISH_SECOND_RUNTIME),
        10 => Some(&ENGLISH_THIRD_RUNTIME),
        93 => Some(&ENGLISH_CONFERENCE_RUNTIME),
        _  => None,
    }
}

/// One club as seen by the production English fixture engine.
///
/// The generator needs the club id (for perturb's E2/E3 stadium
/// pairing, which is club-id-keyed after C10.7's Phase-D fix), the
/// display name (for `HeadlessSeasonFixture.home_club_name` /
/// `.away_club_name`), and the stadium linkage
/// (`stadium_id`/`alt_stadium_id`, ultimately for the perturb
/// resolver). Callers assemble these from the real domain data —
/// `ClubView::id()`, `ClubView::primary_name()`,
/// `ClubView::stadium_id()`, and the stadium record's
/// `alt_stadium_id`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnglishClubEntry {
    pub club_id: u32,
    pub club_name: String,
    /// Home stadium id (Club record +0x69).
    pub stadium_id: Option<i32>,
    /// The stadium's alt/rival cross-link (Stadium record +0x48).
    /// Distinct entry (not derivable from `stadium_id` alone
    /// without a `DomainStadium` lookup); the caller resolves it.
    pub alt_stadium_id: Option<i32>,
}

/// Build a production `StadiumClubResolver` from a slice of
/// [`EnglishClubEntry`]. Uses the `new_by_club` constructor (club-id
/// keyed), so the resolver survives Phase-D roster reordering — the
/// production correctness fix landed in C10.7.
pub fn build_stadium_resolver(entries: &[EnglishClubEntry]) -> StadiumClubResolver {
    StadiumClubResolver::new_by_club(
        entries.iter().map(|e| (e.club_id as i32, e.stadium_id, e.alt_stadium_id)),
    )
}

/// Assemble the perturb clubs_table: `n_clubs * 0x3b` bytes, one
/// entry per club, with the club id serialised as the entry's first
/// int (LE). Follows the exe's `comp+0xb1` byte layout — the
/// remaining 0x37 bytes of each entry are opaque to the driver /
/// perturb code (perturb only ever reads the first 4 bytes and
/// swaps whole entries during Phase D).
pub fn build_clubs_table(entries: &[EnglishClubEntry]) -> Vec<u8> {
    const REC: usize = 0x3b;
    let mut out = vec![0u8; entries.len() * REC];
    for (i, e) in entries.iter().enumerate() {
        out[i * REC..i * REC + 4].copy_from_slice(&(e.club_id as i32).to_le_bytes());
    }
    out
}

/// Decode a clubs_table back into an ordered `Vec<i32>` of club ids
/// (perturb output). One entry per slot.
pub fn decode_clubs_table(table: &[u8]) -> Vec<i32> {
    const REC: usize = 0x3b;
    let n = table.len() / REC;
    (0..n)
        .map(|i| i32::from_le_bytes(table[i * REC..i * REC + 4].try_into().unwrap()))
        .collect()
}

/// Produce fixtures for one English Traditional league.
///
/// # Contract
///
/// * `spec.n_clubs == entries.len()` is required. Callers with
///   fewer/more clubs must route to the generic fallback (the
///   exact engine is defined only at the shipped shape).
/// * `entries` is P1 — the roster order at the exe's ctor entry.
///   This function MUST NOT sort it internally; ordering is a
///   caller responsibility (see point 8 of the C11 directive).
/// * `rng` is the SHARED boot RNG. The function does not seed /
///   reset it. The only reseeds occur inside `matrix_perturb`'s
///   two `lcg_srand` calls, matching the exe.
///
/// # Returned fixtures
///
/// One [`HeadlessSeasonFixture`] per driver emission, in emission
/// order. Rows are assigned sequentially starting at `start_row`.
/// Dates are derived from `spec.rounds_2001[emission.round_within_half]`:
///
/// * `year = base_year + rounds_2001[round].year_off`
/// * `day_of_year = rounds_2001[round].doy_post_snap`
///
/// Only 2001/02 (`base_year = 2001`) is currently byte-exact — the
/// other years pack through `CmPackedDate::to_game_date` which
/// resolves a numeric doy back to (month, day) using leap-year
/// rules, but pre-snap templates (needed for arbitrary year weekday
/// snapping) exist only for Second so far.
///
/// # Errors
///
/// * [`ExactEnglishGenerationError::UnexpectedClubCount`] when
///   `entries.len() != spec.n_clubs`. The 5 Traditional English
///   simulated leagues have fixed shapes; there is NO silent
///   fallback to Berger for these ids (C11.1 point 7).
/// * [`ExactEnglishGenerationError::UnsupportedBaseYear`] when the
///   caller has skipped the `english_dispatch_decision` gate and
///   invoked the engine on a `base_year` outside
///   [`EXACT_SUPPORTED_BASE_YEARS`]. See point 14.
/// * [`ExactEnglishGenerationError::ScheduleTemplateShapeMismatch`]
///   — sanity guard on the shipped consts.
pub fn generate_english_traditional_league(
    spec: &EnglishRuntimeSpec,
    competition: &DomainCompetition,
    entries: &[EnglishClubEntry],
    base_year: u16,
    start_row: u32,
    rng: &mut GameRng,
    // C11.2: caller-provided `DAT_00dbc340` offset for perturb's
    // Phase-C `lcg_srand(year + dbc340_cli_seed)`. Production takes
    // this from `NewGameOptions.initial_game_rng_state` (or 0 if
    // unset); tests pinning a captured GDI boot must pass the
    // captured value derived as `phase_c_srand_seed - base_year`.
    dbc340_cli_seed: i32,
) -> Result<Vec<HeadlessSeasonFixture>, ExactEnglishGenerationError> {
    if entries.len() != spec.n_clubs as usize {
        return Err(ExactEnglishGenerationError::UnexpectedClubCount {
            comp_id: spec.comp_id,
            expected: spec.n_clubs as usize,
            actual: entries.len(),
        });
    }
    if base_year < EARLIEST_SUPPORTED_BASE_YEAR {
        return Err(ExactEnglishGenerationError::UnsupportedBaseYear {
            comp_id: spec.comp_id,
            base_year,
            supported: EXACT_SUPPORTED_BASE_YEARS,
        });
    }
    if spec.rounds_2001.len() != spec.n_rounds as usize {
        return Err(ExactEnglishGenerationError::ScheduleTemplateShapeMismatch {
            comp_id: spec.comp_id,
            expected_rounds: spec.n_rounds,
            template_rounds: spec.rounds_2001.len(),
        });
    }

    let mut clubs_table = build_clubs_table(entries);
    let resolver = build_stadium_resolver(entries);

    // Native Rust schedule — NOT captured .bin blobs. C10.10 proved
    // this reproduces every getter-written byte of the 2001/02
    // capture for all 5 leagues.
    let schedule_buffer = build_league_schedule_from_intermediate(spec.rounds_2001);

    // A club-id -> display name lookup for fixture population,
    // resolved via the CURRENT (post-perturb) clubs_table.
    let name_of: std::collections::BTreeMap<i32, String> = entries
        .iter()
        .map(|e| (e.club_id as i32, e.club_name.clone()))
        .collect();

    // Run perturb explicitly so we can freeze the post-perturb
    // slot -> club_id mapping into a plain Vec before the driver's
    // closure captures it. This avoids the borrow conflict of
    // reading `clubs_table` inside a closure that also holds it
    // mutably (the driver's arg).
    let n_even = spec.n_clubs as i32 + (spec.n_clubs as i32 & 1);
    let perturb_consts = PerturbConstants {
        dbc340_cli_seed,
        ..PerturbConstants::default()
    };
    matrix_perturb(
        spec.n_clubs as i16,
        base_year as i16,
        spec.walker_flag_byte as u16,
        Some(spec.comp_id as i32),
        &mut clubs_table,
        &resolver,
        rng,
        n_even,
        perturb_consts.dbc340_cli_seed,
        perturb_consts.dat_009bba9c,
        perturb_consts.dat_009bc5a8,
        perturb_consts.dat_009bc5ac,
    );
    let slot_to_club_id: Vec<i32> = decode_clubs_table(&clubs_table);

    // Precompute year+doy → GameDate for each round. Skipped when
    // doy == 0 (a schedule-buffer sentinel used for rare "no real
    // matchday" placeholder rounds — mapped to (year, Jan 1) to
    // stay a valid GameDate; the exe uses the schedule row for its
    // own downstream side-effects, not for display).
    let round_dates: Vec<GameDate> = spec
        .rounds_2001
        .iter()
        .map(|r| {
            let year = base_year.wrapping_add(r.year_off as u16);
            let doy = if r.doy_post_snap <= 0 {
                1
            } else {
                r.doy_post_snap as u16
            };
            let packed = CmPackedDate {
                day_of_year: doy,
                year,
                leap_year: is_leap_year(year),
            };
            packed.to_game_date()
        })
        .collect();

    let mut fixtures: Vec<HeadlessSeasonFixture> = Vec::new();

    run_round_robin_driver(
        spec.n_clubs as i16,
        spec.matches_per_pair as i16,
        spec.n_rounds as i16,
        base_year as i16,
        // weekday_parity_flag: 3 — matches captured d9_flags.
        3,
        spec.walker_flag_byte,
        // host_nation: -1 disables the host-nation swap. English
        // leagues do not use this (host-nation swap is for the
        // international tournaments the shared driver is also
        // reused for).
        -1,
        // comp_id — passed through as the walker's `special_comp_id`
        // comparand base; sentinel skip below prevents re-perturb.
        spec.comp_id as i32,
        // special_comp_id — walker's shortcut branch (see decode of
        // FUN_0066ee40). Sentinel MIN never matches a real comp id.
        i32::MIN,
        // skip_perturb_ids — perturb was already run explicitly
        // above; setting both slots to `spec.comp_id` triggers the
        // driver's `if comp_id != skip[0] && comp_id != skip[1]`
        // early-out so it is not run twice.
        [spec.comp_id as i32, spec.comp_id as i32],
        &mut clubs_table,
        &resolver,
        &schedule_buffer,
        None, // alt_pair_list — English leagues never trigger this
        rng,
        perturb_consts,
        |em: FixtureEmission| {
            // slot_to_club_id was frozen from the perturbed table
            // before the driver ran.
            let home_id = slot_to_club_id
                .get(em.home_slot as usize)
                .copied()
                .unwrap_or(-1);
            let away_id = slot_to_club_id
                .get(em.away_slot as usize)
                .copied()
                .unwrap_or(-1);
            let home_name = name_of.get(&home_id).cloned().unwrap_or_default();
            let away_name = name_of.get(&away_id).cloned().unwrap_or_default();

            let round = em.round_within_half as usize;
            let date = round_dates
                .get(round)
                .cloned()
                .unwrap_or(GameDate { year: base_year, month: 8, day: 1 });

            fixtures.push(HeadlessSeasonFixture {
                row: start_row + fixtures.len() as u32,
                competition_id: competition.id,
                competition_name: competition.long_name.clone(),
                date,
                home_club_id: if home_id >= 0 { home_id as u32 } else { 0 },
                home_club_name: home_name,
                away_club_id: if away_id >= 0 { away_id as u32 } else { 0 },
                away_club_name: away_name,
                status: HeadlessFixtureStatus::Pending,
                home_score: None,
                away_score: None,
                match_packet: None,
                match_report: None,
                source: format!(
                    "cm0102-gdi exact English engine — {} run_round_robin_driver \
                     (0x00668450) + matrix_perturb (0x0066b900) + walker_step \
                     (0x0066ee40) + build_league_schedule_from_intermediate; \
                     round {}, outer {}, last_round={}",
                    competition.long_name,
                    em.round_within_half,
                    em.outer_round,
                    em.is_last_round
                ),
            });
        },
        |_reset| {
            // Reset events (schedule field_c == 3) are used by the
            // exe for its own downstream side effects (unaffected
            // by fixture generation). No fixture-side action.
        },
    );

    Ok(fixtures)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_five_leagues_recognized() {
        for id in ENGLISH_TRADITIONAL_COMP_IDS {
            assert!(is_english_traditional_league(id));
            assert!(english_runtime_spec_for(id).is_some());
        }
    }

    #[test]
    fn feeder_pools_are_not_english_traditional() {
        for id in [357u32, 358, 359, 360] {
            assert!(!is_english_traditional_league(id),
                "feeder pool {id} must not dispatch to the exact English engine");
            assert!(english_runtime_spec_for(id).is_none());
        }
    }

    #[test]
    fn non_english_leagues_are_rejected() {
        for id in [4u32, 24, 34, 52, 63] {
            assert!(!is_english_traditional_league(id),
                "non-English comp {id} must not dispatch to the exact English engine");
            assert!(english_runtime_spec_for(id).is_none());
        }
    }

    fn dummy_entries(n: u32, comp_id: u32) -> Vec<EnglishClubEntry> {
        (0..n).map(|i| EnglishClubEntry {
            club_id: (comp_id * 1000) + i,
            club_name: format!("{comp_id}-{i}"),
            stadium_id: Some(((comp_id * 10000) + i) as i32),
            alt_stadium_id: None,
        }).collect()
    }

    #[test]
    fn premier_produces_380_fixtures() {
        let entries = dummy_entries(20, 7);
        let comp = tiny_comp(7, "English Premier Division");
        let mut rng = GameRng::new(0xC110_2001);
        let out = generate_english_traditional_league(
            &ENGLISH_PREMIER_RUNTIME, &comp, &entries, 2001, 0, &mut rng, 0)
            .expect("Prem must generate");
        assert_eq!(out.len(), 380);
        assert_eq!(out[0].row, 0);
        assert_eq!(out[379].row, 379);
        assert_eq!(out[0].date.year, 2001);
    }

    #[test]
    fn first_produces_552_fixtures() {
        let entries = dummy_entries(24, 8);
        let comp = tiny_comp(8, "English First Division");
        let mut rng = GameRng::new(0xC110_2002);
        let out = generate_english_traditional_league(
            &ENGLISH_FIRST_RUNTIME, &comp, &entries, 2001, 100, &mut rng, 0)
            .expect("First must generate");
        assert_eq!(out.len(), 552);
        assert_eq!(out[0].row, 100);
    }

    #[test]
    fn second_produces_552_fixtures() {
        let entries = dummy_entries(24, 9);
        let comp = tiny_comp(9, "English Second Division");
        let mut rng = GameRng::new(0xC110_2003);
        let out = generate_english_traditional_league(
            &ENGLISH_SECOND_RUNTIME, &comp, &entries, 2001, 0, &mut rng, 0)
            .expect("Second must generate");
        assert_eq!(out.len(), 552);
    }

    #[test]
    fn third_produces_552_fixtures() {
        let entries = dummy_entries(24, 10);
        let comp = tiny_comp(10, "English Third Division");
        let mut rng = GameRng::new(0xC110_2004);
        let out = generate_english_traditional_league(
            &ENGLISH_THIRD_RUNTIME, &comp, &entries, 2001, 0, &mut rng, 0)
            .expect("Third must generate");
        assert_eq!(out.len(), 552);
    }

    #[test]
    fn conference_produces_462_fixtures() {
        let entries = dummy_entries(22, 93);
        let comp = tiny_comp(93, "English Conference");
        let mut rng = GameRng::new(0xC110_2005);
        let out = generate_english_traditional_league(
            &ENGLISH_CONFERENCE_RUNTIME, &comp, &entries, 2001, 0, &mut rng, 0)
            .expect("Conf must generate");
        assert_eq!(out.len(), 462);
    }

    #[test]
    fn one_shared_rng_across_all_five_leagues_produces_matching_totals_and_advances_state() {
        // C10.11 evidence: exe's RNG stream is one continuous chain
        // across the 5 English leagues. Verify: (a) sharing works,
        // (b) each league advances the state, (c) each subsequent
        // league starts where the previous one left off (no reset).
        let mut rng = GameRng::new(0xC110_ABCD);
        let mut total = 0usize;
        let mut prev_after = rng.snapshot();
        for spec in ENGLISH_RUNTIME_SPECS {
            let entries = dummy_entries(spec.n_clubs as u32, spec.comp_id);
            let comp = tiny_comp(spec.comp_id, "English test comp");
            let before = rng.snapshot();
            assert_eq!(before, prev_after,
                       "{}: shared RNG must resume from prior league's final state",
                       spec.comp_id);
            let out = generate_english_traditional_league(
                spec, &comp, &entries, 2001, total as u32, &mut rng, 0)
                .expect("must generate");
            let expected = (spec.n_clubs as usize) * (spec.n_clubs as usize - 1);
            assert_eq!(out.len(), expected);
            let after = rng.snapshot();
            assert_ne!(after, before,
                       "{}: RNG state must have advanced during generation",
                       spec.comp_id);
            prev_after = after;
            total += out.len();
        }
        assert_eq!(total, 380 + 552 + 552 + 552 + 462);
    }

    #[test]
    fn wrong_club_count_returns_error_no_silent_fallback() {
        // C11.1 point 7: The 5 English simulated leagues have a
        // fixed shape in Traditional mode. Wrong club count is a
        // hard invariant break — return an error rather than fall
        // back silently to the generic Berger path.
        let entries: Vec<_> = (0..10u32).map(|i| EnglishClubEntry {
            club_id: i, club_name: String::new(),
            stadium_id: None, alt_stadium_id: None,
        }).collect();
        let comp = tiny_comp(7, "test");
        let mut rng = GameRng::new(0);
        let err = generate_english_traditional_league(
            &ENGLISH_PREMIER_RUNTIME, &comp, &entries, 2001, 0, &mut rng, 0)
            .expect_err("must return UnexpectedClubCount");
        match err {
            ExactEnglishGenerationError::UnexpectedClubCount { comp_id, expected, actual } => {
                assert_eq!(comp_id, 7);
                assert_eq!(expected, 20);
                assert_eq!(actual, 10);
            }
            other => panic!("wrong error variant: {other:?}"),
        }
    }

    #[test]
    fn unsupported_base_year_returns_error_not_silent_success() {
        // C11.1 point 14: only 2001/02 is proven exact. Anything
        // else the caller sneaks in past the dispatch decision
        // must be refused at the engine boundary.
        let entries = dummy_entries(20, 7);
        let comp = tiny_comp(7, "Prem");
        let mut rng = GameRng::new(0);
        let err = generate_english_traditional_league(
            &ENGLISH_PREMIER_RUNTIME, &comp, &entries, 2002, 0, &mut rng, 0)
            .expect_err("must reject 2002");
        assert!(matches!(err, ExactEnglishGenerationError::UnsupportedBaseYear { .. }));
    }

    #[test]
    fn dispatch_decision_covers_traditional_v4_generic_and_year_gate() {
        // Traditional + English + 2001 -> exact.
        for &id in &[7, 8, 9, 10, 93] {
            assert_eq!(
                english_dispatch_decision(GameMode::Traditional, id, 2001),
                EnglishFixtureDispatch::ExactEnglish,
                "Traditional/2001 comp {id} must be ExactEnglish"
            );
        }
        // Traditional + English + non-2001 -> Skipped.
        for &id in &[7, 8, 9, 10, 93] {
            assert!(matches!(
                english_dispatch_decision(GameMode::Traditional, id, 2002),
                EnglishFixtureDispatch::Skipped { .. }
            ));
        }
        // Traditional + non-English -> Generic.
        for &id in &[4u32, 24, 34, 38, 63, 357, 358, 359, 360] {
            assert_eq!(
                english_dispatch_decision(GameMode::Traditional, id, 2001),
                EnglishFixtureDispatch::Generic,
                "Traditional non-English comp {id} must be Generic"
            );
        }
        // V4 + English -> Skipped (mode gate). Critical: V4 must
        // NEVER route through the exact Traditional engine.
        for &id in &[7u32, 8, 9, 10, 93] {
            match english_dispatch_decision(GameMode::V4, id, 2001) {
                EnglishFixtureDispatch::Skipped { reason } => {
                    assert!(reason.contains("V4"),
                            "V4 skip reason must name the mode: {reason:?}");
                }
                other => panic!("V4 must be Skipped for English comp {id}, got {other:?}"),
            }
        }
        // V4 + non-English also Skipped (this module owns no V4 dispatch at all).
        assert!(matches!(
            english_dispatch_decision(GameMode::V4, 38, 2001),
            EnglishFixtureDispatch::Skipped { .. }
        ));
    }

    fn tiny_comp(id: u32, name: &str) -> DomainCompetition {
        DomainCompetition {
            id,
            long_name: name.to_string(),
            short_name: name.to_string(),
            three_letter_name: String::new(),
            scope: 2,
            nation_id: 46,
            last_division: -1,
            reserve_division: -1,
            reputation: 50,
            unknown_tail: Vec::new(),
        }
    }
}
