//! A reusable single-elimination **domestic cup** engine — the shape of
//! `bel_fa_cup.cpp` (VA `0x0041dba0`, vtable `0x009558a0`, club base
//! `0x00502320`) and every other national cup. Clubs are seeded into a
//! power-of-two bracket (weaker entrants get byes into later rounds); each round
//! is played through the ordinary phase-2 batch and the winners advance until a
//! single champion remains.
//!
//! Reused across nations by supplying the participant pool + competition id.

use serde::{Deserialize, Serialize};

use crate::arg_primera::ArgTeam;
use crate::honours::Honour;
use crate::{
    CmPackedDate, GameDate, HeadlessFixtureStatus, HeadlessSeasonFixture,
};

/// Runtime competition-id base for ported cups (`+ real_comp_id`).
pub const CUP_RUNTIME_BASE: u32 = 0x7800;

/// Decoded per-round **leg-A** dates for cups whose round-helper
/// tables have been recovered from the exe. Index 0 = round 1
/// (the round AFTER the first-round date passed to `build`),
/// so `advance()` reads `[next_round - 1]`.
///
/// Source: `reports/cup_round_schedules.md` (FA Cup 351, League
/// Cup 352, French Cup 335, German Cup 337, Dutch Cup 338,
/// Italian Cup 339) and `reports/cup_round_schedules_batch2.md`
/// (French League Cup 336, Belgian Cup 332, FA Trophy 94, Vans
/// Trophy 354). Months are 1-indexed here (the reports note the
/// exe stores them 0-indexed).
///
/// `year` is the season's base year (2001 for the shipped DB);
/// dates after the New Year use `year + 1`. Cups not in this
/// table return an empty vec and `advance()` falls back to
/// `start_date + 14 days * round`.
///
/// Two-phase cups (Italy 339, Holland 338) list their FULL-branch
/// rounds — the exe's short qualifying branch (3 rounds, Aug) is
/// what `build`'s first-round date covers; the full table then
/// follows. Vans 354 lists the 5-round main draw only (the final
/// is a separate `param_2 == 1` instantiation in the exe).
pub fn decoded_round_dates(real_comp_id: i32, year: u16) -> Vec<GameDate> {
    let y = year;
    let n = year + 1;
    let d = |yr: u16, m: u8, dd: u8| GameDate { year: yr, month: m, day: dd };
    match real_comp_id {
        // English FA Cup — FUN_00558f60, 9 rounds single-leg.
        351 => vec![d(y,10,17), d(y,11,19), d(y,12,10), d(n,1,7),
                    d(n,1,28), d(n,2,18), d(n,3,11), d(n,4,9)],
        // English League Cup — FUN_00556150, 7 rounds.
        352 => vec![d(y,8,23), d(y,9,28), d(y,10,29), d(y,11,12),
                    d(y,12,3), d(n,2,18)],
        // English FA Trophy — comp 94, 5 rounds.
        94  => vec![d(y,11,26), d(n,1,17), d(n,2,7), d(n,4,11)],
        // English LDV Vans Trophy — comp 354, main draw 5 rounds.
        354 => vec![d(y,12,11), d(n,1,15), d(n,2,5), d(n,2,19)],
        // French Cup — FUN_005a4650, 9 rounds.
        335 => vec![d(y,11,12), d(y,12,3), d(n,1,6), d(n,1,27),
                    d(n,2,20), d(n,3,5), d(n,3,20), d(n,4,13)],
        // French League Cup — comp 336, 7 rounds.
        336 => vec![d(y,10,15), d(y,11,26), d(n,1,14), d(n,2,3),
                    d(n,3,4), d(n,4,2)],
        // German Cup (DFB-Pokal) — FUN_005c2b90, 6 rounds.
        337 => vec![d(y,8,27), d(y,11,12), d(y,11,30), d(y,12,21),
                    d(n,2,8)],
        // Dutch Cup (KNVB Beker) — FUN_005dd170, full branch 6 rounds.
        338 => vec![d(y,8,26), d(y,9,30), d(y,11,10), d(y,11,30),
                    d(n,1,13), d(n,4,21)],
        // Italian Cup (Coppa Italia) — FUN_006282d0, full branch 5 rounds.
        339 => vec![d(y,8,30), d(y,10,25), d(y,11,29), d(n,1,10),
                    d(n,2,7)],
        // Belgian Cup (Beker van België) — comp 332, 8 rounds.
        332 => vec![d(y,8,13), d(y,8,20), d(y,9,15), d(y,11,6),
                    d(y,11,30), d(n,1,25), d(n,4,12)],
        _ => Vec::new(),
    }
}

// ============================================================================
// Progressive-draw schedule model (commit 1 — data only, no behaviour change).
//
// The English domestic cups are decoded from the exe (reports/
// cup_draw_structure_decode.md): rounds are drawn on their own DRAW date,
// only that round's fixtures are materialised, and clubs enter at staggered
// rounds via contiguous entry-pool windows. This module expresses that as
// DATA so no `if comp_id == ...` behaviour is baked into the engine.
// ============================================================================

/// How a cup materialises its fixtures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CupMode {
    /// Whole bracket seeded and generated at boot (the original engine).
    /// Used by undecoded cups (most foreign) — APPROXIMATE / UNDECODED.
    LegacyPreGenerated,
    /// Faithful: each round is drawn on its decoded draw date, staggered
    /// entry via pool windows, replays/two-legs per the decoded flags. Used
    /// only by cups with decoded executable evidence (the English cups).
    ProgressiveDecoded,
}

impl Default for CupMode {
    fn default() -> Self { CupMode::LegacyPreGenerated }
}

/// One round of a decoded cup: the exe's 0x68-byte round record, as data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CupRoundSpec {
    pub round: u32,
    /// When the draw is performed (the round's fixtures materialise now).
    pub draw_date: GameDate,
    /// When this round's ties are played (leg 1 for a two-legged round).
    pub match_date: GameDate,
    /// New clubs entering the pool at THIS round (exe round-record `+0x1c`).
    pub incoming: u32,
    /// Ties this round (exe `+0x1a`). Participants should be `2*capacity`
    /// (survivors + incoming); any shortfall becomes byes.
    pub capacity: u32,
    /// Two physical legs, venues swapped (exe `+0x21 == 2`).
    pub two_leg: bool,
    /// Drawn ties are replayed (exe `+0x20 == 1`); else settled on the day.
    pub replay: bool,
    /// Final played at a neutral venue.
    pub neutral_final: bool,
}

/// A decoded cup's full lifecycle spec.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CupSchedule {
    /// Total entrants across all rounds (Σ incoming) — the exe pool size.
    pub pool_size: usize,
    pub rounds: Vec<CupRoundSpec>,
}

/// The decoded English-cup schedule for `real_comp_id`, or `None` for an
/// undecoded (legacy) cup. `euro_qual_count` = number of English clubs holding
/// a European qualification (`Club+0x1db != -1`), clamped 0..=12 by the caller;
/// it parameterises the League Cup entry windows (exe `b2 = comp+0xb2`).
///
/// FA Cup uses the shipped `b3=0, b4=false` branch → a fixed 152-team pool with
/// Premier clubs in the round-3 (3rd Round Proper) entry slice.
/// Source: reports/cup_draw_structure_decode.md.
pub fn decoded_cup_schedule(
    real_comp_id: i32,
    year: u16,
    euro_qual_count: u32,
) -> Option<CupSchedule> {
    let y = year;
    let n = year + 1;
    let d = |yr: u16, m: u8, dd: u8| GameDate { year: yr, month: m, day: dd };
    let r = |round: u32, draw: GameDate, play: GameDate, incoming: u32, capacity: u32,
             two_leg: bool, replay: bool, neutral_final: bool| CupRoundSpec {
        round, draw_date: draw, match_date: play, incoming, capacity, two_leg, replay, neutral_final,
    };
    match real_comp_id {
        // ---- English FA Cup (351) — FUN_00558f60, b3=0 ⇒ 152-team pool. ----
        // Single-leg throughout; replays rounds 0-6; neutral final.
        351 => Some(CupSchedule {
            pool_size: 152,
            rounds: vec![
                r(0, d(y,10, 9), d(y,10,16), 56, 28, false, true,  false),
                r(1, d(y,10,17), d(y,11,18), 52, 40, false, true,  false),
                r(2, d(y,11,19), d(y,12,11),  0, 20, false, true,  false),
                r(3, d(y,12,10), d(n, 1, 6), 44, 32, false, true,  false), // Premier enter (3rd Rd Proper)
                r(4, d(n, 1, 7), d(n, 1,27),  0, 16, false, true,  false),
                r(5, d(n, 1,28), d(n, 2,17),  0,  8, false, true,  false),
                r(6, d(n, 2,18), d(n, 3,10),  0,  4, false, true,  false),
                r(7, d(n, 3,11), d(n, 4, 8),  0,  2, false, false, false), // SF (single-leg in CM)
                r(8, d(n, 4, 9), d(n, 5,12),  0,  1, false, false, true ), // Final (neutral)
            ],
        }),
        // ---- English League Cup (352) — FUN_00556150, b2=euro_qual_count. ----
        // Single-leg except the two-legged SF (round 5); no replays.
        352 => {
            let b2 = euro_qual_count.min(12);
            Some(CupSchedule {
                pool_size: 92,
                rounds: vec![
                    r(0, d(y, 7,23), d(y, 8,22), 56 + 2*b2, 28 + b2, false, false, false),
                    r(1, d(y, 8,23), d(y, 9,20), (12 - b2)*3, 32 - b2, false, false, false),
                    r(2, d(y, 9,28), d(y,10,28), b2,          16,      false, false, false), // Euro-quals enter
                    r(3, d(y,10,29), d(y,11,11), 0,            8,      false, false, false),
                    r(4, d(y,11,12), d(y,12, 2), 0,            4,      false, false, false),
                    r(5, d(y,12, 3), d(n, 1,27), 0,            2,      true,  false, false), // SF two-legged
                    r(6, d(n, 2,18), d(n, 4, 1), 0,            1,      false, false, true ), // Final (neutral)
                ],
            })
        }
        // ---- English FA Trophy (94) — FUN_0055abb0, 5 rounds single-leg. ----
        // Non-league; no decoded qualifying staggering → all enter round 0.
        // pool_size 0 = "all supplied clubs enter round 0" (engine fills it).
        94 => Some(CupSchedule {
            pool_size: 0,
            rounds: vec![
                r(0, d(y,11, 8), d(y,11,25), 0, 0, false, false, false),
                r(1, d(y,11,26), d(n, 1,16), 0, 0, false, false, false),
                r(2, d(n, 1,17), d(n, 2, 6), 0, 0, false, false, false),
                r(3, d(n, 2, 7), d(n, 4,10), 0, 0, false, false, false),
                r(4, d(n, 4,11), d(n, 5,15), 0, 1, false, false, true ),
            ],
        }),
        // ---- English Charity Shield (353) — single neutral match. Modelled by
        // the super_cup engine (champion vs FA Cup winner), listed here for the
        // decoded draw/match dates: draw 5 Jul / match 13 Aug. ----
        353 => Some(CupSchedule {
            pool_size: 2,
            rounds: vec![ r(0, d(y, 7, 5), d(y, 8,13), 2, 1, false, false, true) ],
        }),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CupState {
    pub year: u16,
    pub real_comp_id: i32,
    pub runtime_comp_id: u32,
    pub name: String,
    /// The bracket teams, strongest first (used for seeding + bye placement).
    pub teams: Vec<ArgTeam>,
    /// The current round number (1 = first round created).
    pub round: u32,
    pub start_date: GameDate,
    /// Optional per-round leg-A dates decoded from the exe's cup
    /// round-helper tables (see `reports/cup_round_schedules.md`).
    ///
    /// Index 0 = the round AFTER `start_date` (exe table "R1";
    /// this engine's `round == 2`). `advance()` reads
    /// `round_dates[next_round - 2]` — see the numbering note
    /// there — and falls back to `start_date + round * 14` when
    /// the index is out of range. `#[serde(default)]` keeps
    /// older saves loading unchanged.
    #[serde(default)]
    pub round_dates: Vec<GameDate>,
    pub champion_honour: u32,
    pub complete: bool,
    pub provenance: String,
}

/// Largest power of two that is `<= n` (the bracket size).
fn bracket_size(n: usize) -> usize {
    let mut p = 1;
    while p * 2 <= n {
        p *= 2;
    }
    p
}

impl CupState {
    /// Build from a candidate pool; keeps the strongest `2^k` clubs so the
    /// bracket is clean. `None` if fewer than two clubs.
    pub fn build(
        mut pool: Vec<ArgTeam>,
        real_comp_id: i32,
        name: &str,
        year: u16,
        start_date: GameDate,
        champion_honour: u32,
        source_va: &str,
    ) -> Option<Self> {
        if pool.len() < 2 {
            return None;
        }
        pool.sort_by(|a, b| b.reputation.cmp(&a.reputation).then(a.club_id.cmp(&b.club_id)));
        pool.truncate(bracket_size(pool.len()));
        let count = pool.len();
        Some(Self {
            year,
            real_comp_id,
            runtime_comp_id: CUP_RUNTIME_BASE + real_comp_id as u32,
            name: name.to_string(),
            teams: pool,
            round: 0,
            start_date,
            // Every cup with a decoded round table gets it here so
            // no construction site has to remember to attach it.
            round_dates: decoded_round_dates(real_comp_id, year),
            champion_honour,
            complete: false,
            provenance: format!("{name} {year}: {count}-team single-elimination cup; ported from {source_va}."),
        })
    }

    /// Attach the decoded per-round leg-A dates (index 0 = round 1).
    /// Passes-through unchanged; the effect is on the `advance()`
    /// scheduling step.
    pub fn with_round_dates(mut self, dates: Vec<GameDate>) -> Self {
        self.round_dates = dates;
        self
    }

    fn tag(&self, round: u32) -> String {
        format!("CUP{}-R{}", self.runtime_comp_id, round)
    }
}

fn fixture(
    state: &CupState,
    row: u32,
    round: u32,
    date: GameDate,
    home: &ArgTeam,
    away: &ArgTeam,
) -> HeadlessSeasonFixture {
    HeadlessSeasonFixture {
        row,
        competition_id: state.runtime_comp_id,
        competition_name: state.name.clone(),
        date,
        home_club_id: home.club_id,
        home_club_name: home.name.clone(),
        away_club_id: away.club_id,
        away_club_name: away.name.clone(),
        status: HeadlessFixtureStatus::Pending,
        home_score: None,
        away_score: None,
        match_packet: None,
        match_report: None,
        source: format!("{} {} round (tag {})", state.name, state.year, state.tag(round)),
    }
}

/// Generate the first round (seeded 1-v-n, 2-v-(n-1), … so top seeds meet
/// weakest first). Mutates `state.round` to 1.
pub fn generate_first_round(state: &mut CupState, mut next_row: u32) -> Vec<HeadlessSeasonFixture> {
    state.round = 1;
    let n = state.teams.len();
    let date = state.start_date.clone();
    let mut out = Vec::new();
    for i in 0..n / 2 {
        let home = state.teams[i].clone();
        let away = state.teams[n - 1 - i].clone();
        out.push(fixture(state, next_row, 1, date.clone(), &home, &away));
        next_row += 1;
    }
    out
}

/// The winner of a played tie: higher score, draw → higher reputation (a
/// documented stand-in for the cup's replay/extra-time path).
fn winner(f: &HeadlessSeasonFixture, rep: &dyn Fn(u32) -> u16) -> Option<ArgTeam> {
    let (hs, as_) = (f.home_score?, f.away_score?);
    let home = match hs.cmp(&as_) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => rep(f.home_club_id) >= rep(f.away_club_id),
    };
    Some(if home {
        ArgTeam { club_id: f.home_club_id, name: f.home_club_name.clone(), reputation: rep(f.home_club_id) }
    } else {
        ArgTeam { club_id: f.away_club_id, name: f.away_club_name.clone(), reputation: rep(f.away_club_id) }
    })
}

/// The result of a cup advancement step.
#[derive(Debug, Default)]
pub struct CupAdvance {
    pub new_fixtures: Vec<HeadlessSeasonFixture>,
    pub news: Vec<(String, String)>,
    pub honours: Vec<Honour>,
    pub completed: bool,
    /// The new round number, if the cup advanced.
    pub new_round: Option<u32>,
}

/// Advance the cup one round: when the current round's ties are all played,
/// pair the winners into the next round; when one winner remains, crown the
/// champion. Idempotent until the round completes.
pub fn advance(state: &CupState, fixtures: &[HeadlessSeasonFixture], next_row: u32) -> CupAdvance {
    let mut out = CupAdvance::default();
    if state.complete || state.round == 0 {
        return out;
    }
    let rep = |id: u32| state.teams.iter().find(|t| t.club_id == id).map(|t| t.reputation).unwrap_or(0);
    let tag = state.tag(state.round);
    let mut cur: Vec<&HeadlessSeasonFixture> =
        fixtures.iter().filter(|f| f.source.contains(&tag)).collect();
    cur.sort_by_key(|f| f.row);
    if cur.is_empty() || cur.iter().any(|f| f.status != HeadlessFixtureStatus::Played) {
        return out;
    }
    let winners: Vec<ArgTeam> = cur.iter().filter_map(|f| winner(f, &rep)).collect();
    if winners.len() == 1 {
        let champ = &winners[0];
        out.news.push((
            "competition".into(),
            format!("{} - win the {} {}", champ.name, state.name, state.year),
        ));
        out.honours.push(Honour::champion(
            state.year,
            state.champion_honour,
            state.name.clone(),
            champ.club_id,
            champ.name.clone(),
        ));
        out.completed = true;
        return out;
    }
    // Pair winners into the next round.
    let next_round = state.round + 1;
    // Prefer the decoded per-round leg-A date when available;
    // otherwise fall back to +14 days per round.
    //
    // Numbering: this engine labels the first round `round = 1`
    // (generate_first_round) whereas the exe tables label it R0.
    // So engine round N is table R(N-1), and `round_dates[0]` is
    // the round AFTER the start round → index = next_round - 2.
    // next_round >= 2 here (round==0 already returned above).
    let date = next_round.checked_sub(2)
        .and_then(|i| state.round_dates.get(i as usize))
        .cloned()
        .unwrap_or_else(|| {
            let base = CmPackedDate::from_game_date(state.start_date.clone());
            base.add_days((state.round as i16) * 14).to_game_date()
        });
    let mut row = next_row;
    for pair in winners.chunks(2) {
        if let [h, a] = pair {
            out.new_fixtures.push(fixture(state, row, next_round, date.clone(), h, a));
            row += 1;
        }
    }
    // Retag the new fixtures for the next round.
    for f in &mut out.new_fixtures {
        f.source = format!("{} {} round (tag {})", state.name, state.year, state.tag(next_round));
    }
    out.new_round = Some(next_round);
    out.news.push((
        "competition".into(),
        format!("{} - round {} of the {} is set", state.name, next_round, state.year),
    ));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pool(n: u32) -> Vec<ArgTeam> {
        (0..n).map(|i| ArgTeam { club_id: i, name: format!("C{i}"), reputation: (200 - i) as u16 }).collect()
    }

    /// Locks the decoded per-round table shape for every cup that
    /// has one, so an edit to `decoded_round_dates` can't silently
    /// drop or reorder a round. Counts + first/last dates come from
    /// reports/cup_round_schedules.md and _batch2.md.
    #[test]
    fn decoded_round_tables_match_reports() {
        // (comp, rounds after R0, first leg-A, last leg-A)
        let expect: &[(i32, usize, (u16, u8, u8), (u16, u8, u8))] = &[
            (351, 8, (2001, 10, 17), (2002, 4, 9)),   // FA Cup
            (352, 6, (2001, 8, 23),  (2002, 2, 18)),  // League Cup
            (94,  4, (2001, 11, 26), (2002, 4, 11)),  // FA Trophy
            (354, 4, (2001, 12, 11), (2002, 2, 19)),  // Vans (main draw)
            (335, 8, (2001, 11, 12), (2002, 4, 13)),  // French Cup
            (336, 6, (2001, 10, 15), (2002, 4, 2)),   // French League Cup
            (337, 5, (2001, 8, 27),  (2002, 2, 8)),   // DFB-Pokal
            (338, 6, (2001, 8, 26),  (2002, 4, 21)),  // KNVB Beker (full)
            (339, 5, (2001, 8, 30),  (2002, 2, 7)),   // Coppa Italia (full)
            (332, 7, (2001, 8, 13),  (2002, 4, 12)),  // Beker van België
        ];
        for &(comp, n, (fy, fm, fd), (ly, lm, ld)) in expect {
            let v = decoded_round_dates(comp, 2001);
            assert_eq!(v.len(), n, "comp {comp} round count");
            assert_eq!((v[0].year, v[0].month, v[0].day), (fy, fm, fd), "comp {comp} first");
            let l = v.last().unwrap();
            assert_eq!((l.year, l.month, l.day), (ly, lm, ld), "comp {comp} last");
            // Strictly increasing across the season boundary.
            for w in v.windows(2) {
                let a = (w[0].year, w[0].month, w[0].day);
                let b = (w[1].year, w[1].month, w[1].day);
                assert!(a < b, "comp {comp}: {a:?} !< {b:?}");
            }
        }
        // Un-decoded cups fall back to empty (→ +14-day path in advance()).
        assert!(decoded_round_dates(999, 2001).is_empty());
        // Year threading: base year shifts every date by the same delta.
        let a = decoded_round_dates(351, 2001);
        let b = decoded_round_dates(351, 2005);
        assert!(a.iter().zip(&b).all(|(x, y)| y.year == x.year + 4 && y.month == x.month && y.day == x.day));
    }

    /// `build()` attaches the table automatically and `advance()`
    /// schedules round 2 on the decoded leg-A date, not `start + 14`.
    #[test]
    fn advance_uses_decoded_round_date_over_plus_fourteen() {
        // FA Cup: R0 = 9 Oct 2001; decoded R1 = 17 Oct 2001; the
        // +14 fallback would have said 23 Oct. Distinguishable.
        let mut st = CupState::build(pool(8), 351, "English FA Cup", 2001,
            GameDate { year: 2001, month: 10, day: 9 }, 0x7d0, "test").unwrap();
        assert_eq!(st.round_dates.len(), 8, "build attached the FA Cup table");
        let mut fx = generate_first_round(&mut st, 0);
        for f in fx.iter_mut() {
            f.status = HeadlessFixtureStatus::Played;
            let (h, a) = (f.home_club_id, f.away_club_id);
            let (hs, as_) = if h < a { (2, 0) } else { (0, 2) };
            f.home_score = Some(hs);
            f.away_score = Some(as_);
        }
        let adv = advance(&st, &fx, fx.len() as u32);
        assert_eq!(adv.new_round, Some(2));
        assert!(!adv.new_fixtures.is_empty());
        for f in &adv.new_fixtures {
            assert_eq!((f.date.year, f.date.month, f.date.day), (2001, 10, 17),
                "round 2 must land on decoded 17 Oct, not +14 = 23 Oct");
        }
        // A cup with no table keeps the +14 fallback.
        let mut nt = CupState::build(pool(8), 999, "Nowhere Cup", 2001,
            GameDate { year: 2001, month: 9, day: 1 }, 0x7d0, "test").unwrap();
        assert!(nt.round_dates.is_empty());
        let mut fx2 = generate_first_round(&mut nt, 0);
        for f in fx2.iter_mut() {
            f.status = HeadlessFixtureStatus::Played;
            f.home_score = Some(1); f.away_score = Some(0);
        }
        let adv2 = advance(&nt, &fx2, fx2.len() as u32);
        for f in &adv2.new_fixtures {
            assert_eq!((f.date.month, f.date.day), (9, 15), "fallback = 1 Sep + 14");
        }
    }

    #[test]
    fn bracket_trims_to_power_of_two() {
        let st = CupState::build(pool(18), 332, "Belgian Cup", 2001, GameDate { year: 2001, month: 9, day: 1 }, 0x7d0, "test").unwrap();
        assert_eq!(st.teams.len(), 16);
    }

    #[test]
    fn cup_runs_to_a_single_champion() {
        let mut st = CupState::build(pool(8), 332, "Belgian Cup", 2001, GameDate { year: 2001, month: 9, day: 1 }, 0x7d0, "test").unwrap();
        let mut fx = generate_first_round(&mut st, 0);
        let play = |fx: &mut Vec<HeadlessSeasonFixture>| {
            for f in fx.iter_mut().filter(|f| f.status == HeadlessFixtureStatus::Pending) {
                f.status = HeadlessFixtureStatus::Played;
                // Lower club id (stronger seed) always wins.
                let (h, a) = (f.home_club_id, f.away_club_id);
                let (hs, as_) = if h < a { (2, 0) } else { (0, 2) };
                f.home_score = Some(hs);
                f.away_score = Some(as_);
            }
        };
        // 8 -> 4 -> 2 -> 1: three advance steps.
        let mut guard = 0;
        loop {
            play(&mut fx);
            let next = fx.iter().map(|f| f.row + 1).max().unwrap_or(0);
            let adv = advance(&st, &fx, next);
            fx.extend(adv.new_fixtures);
            if let Some(r) = adv.new_round {
                st.round = r;
            }
            if adv.completed {
                assert_eq!(adv.honours.len(), 1);
                assert!(adv.news[0].1.contains("C0"), "top seed wins the cup");
                break;
            }
            guard += 1;
            assert!(guard < 6, "cup did not converge");
        }
    }
}
