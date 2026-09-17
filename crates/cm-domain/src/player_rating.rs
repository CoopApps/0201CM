//! Player-rating substrate — CA-driven per-season performance ratings.
//!
//! This is what the game's per-country `*_awards.cpp` files consume to
//! decide the annual Player of the Season / Top Scorer / Young Player /
//! Team of the Season awards. Without it, none of the awards TUs can be
//! ported — every one of them just reads back a "who's the best" answer
//! this module produces.
//!
//! # Deliberate scope
//!
//! We do NOT reproduce the exe's full performance-rating algorithm. That
//! would require the match engine (per-fixture stats), the form-tracking
//! subsystem, and the injury/morale/fitness modifiers — none of which are
//! ported yet.
//!
//! What we DO produce is a **CA-driven proxy** that's monotone with
//! current ability + a small per-player deterministic wobble seeded from
//! the player's staff id. Concretely:
//!
//! * `season_rating(staff_id) = CA * 0.8 + wobble(staff_id) * 0.2`
//! * `top_scorer_score(staff_id) = CA * 0.6 + wobble(staff_id) * 0.4`
//!
//! The wobble uses a splitmix-style hash of the staff id so it's stable
//! across ticks and deterministic across saves. Real match performance
//! stats will replace the wobble term once the match engine lands — the
//! caller-facing API (`player_of_the_season`, `top_scorer`,
//! `team_of_the_season`) stays the same.
//!
//! # Substrate role
//!
//! Unblocks ALL `*_awards.cpp` TUs:
//! * `holland_awards.cpp`, `ireland_awards.cpp`, `italy_awards.cpp`,
//!   `japan_awards.cpp`, `international_awards.cpp`
//! * `european_awards.cpp` (partially — champion honours already recorded)
//! * `argentina_awards.cpp`, `australia_awards.cpp` (already registered
//!   at low fidelity)
//! * `award_manager.cpp` — the shared engine that consumes what this
//!   substrate exposes
//! * `hall_of_fame.cpp` slot-selection (which player fills each 27x100
//!   slot every season)

use serde::{Deserialize, Serialize};

use crate::{ClubView, DomainOpaqueRecord, StaffBook};

/// Current year assumed for the initial rating snapshot. The book is
/// rebuilt from world data at new-game creation with the real start year.
const DEFAULT_RATING_YEAR: u16 = 2001;
const DEFAULT_RATING_DAY: u16 = 213;  // ~1 Aug — start-of-season

/// A player's per-season score in a given competition. Higher = better.
/// Range roughly 0..250 given CA is 1..200 and wobble is 0..50.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct SeasonScore {
    pub staff_id: u32,
    pub score: f32,
    /// Current ability at the time of scoring — makes debugging easy.
    pub ca: i16,
}

/// Per-competition season/career average rating — VERIFIED port of
/// `FUN_007aa490:73-77` case 0x11. The exe's real rating-average formula
/// is a straight arithmetic mean: `avg = rating_sum / appearances`, where
/// the sum is an i16 at sub-block `+0x0e` and the count is a u8 at `+0x00`.
/// Both are per-competition (fetched via `FUN_007abc60(person, comp_id)`).
///
/// Returns `None` when the player hasn't appeared in that competition yet
/// (dividing 0/0 in the exe just falls through — L73 `if (*param_1 != 0)`).
///
/// Composite career variants (categories 'd'/'e'/'f') in `007aa170.c`
/// blend blocks 2..5 with weights 6.5f; the primitive is still this
/// `sum/count`. See `reports/season_avg_writer_hunt.md`.
#[inline]
pub fn season_avg_rating(appearances: u8, rating_sum: i16) -> Option<f32> {
    if appearances == 0 { None } else { Some(rating_sum as f32 / appearances as f32) }
}

/// Deterministic per-id wobble in the range [0.0, 50.0). Seeded by the
/// staff id so a player's score is stable across ticks + saves.
fn wobble(staff_id: u32) -> f32 {
    // splitmix64-style hash on the id then take the low 8 bits as a 0..255
    // value and scale.
    let mut z = (staff_id as u64).wrapping_add(0x9e3779b97f4a7c15);
    z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
    z ^= z >> 31;
    ((z & 0xff) as f32) * (50.0 / 255.0)
}

/// One rated player's snapshot: CA + PA + club-affiliation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RatedPlayer {
    pub staff_id: u32,
    pub club_id: Option<i32>,
    pub division_id: Option<i32>,
    pub ca: i16,
    pub pa: i16,
    /// Estimated goals this season (proxy — CA + wobble).
    pub goals_est: u16,
    /// Approximate age. Placeholder until DOB is wired through.
    pub age_est: u8,
    /// Coarse engine-ordinal position (`DomainStaffType10::engine_position_ordinal`) —
    /// byte-exact from the real position-eligibility formula (`FUN_005a2030`),
    /// reduced to what `EngineTeamPlayer::position` actually consumes.
    pub position_ordinal: u8,
    /// Whether this player's real attributes clear the goalkeeper threshold.
    pub is_gk: bool,
    /// The real match-engine attributes (1..20) this player carries — sourced
    /// from the shipped-or-generated type10 attribute block (alphabetical), so
    /// `snapshot_team_for_engine` no longer hardcodes them (kill #1a). Mapping:
    /// aggression→attr 1, bravery→5, dirtiness→10, injury_proneness→18,
    /// jumping_heading→Jumping (19).
    #[serde(default)]
    pub aggression: i8,
    #[serde(default)]
    pub bravery: i8,
    #[serde(default)]
    pub dirtiness: i8,
    #[serde(default)]
    pub injury_proneness: u8,
    #[serde(default)]
    pub jumping_heading: i8,
    /// REAL accumulated season stats, fed from actual simulated match events
    /// (`ExeMatchResult::home_scorer_ids`/`away_scorer_ids`/assist ids) — the
    /// replacement for the CA-proxy `goals_est` in award selection (kill #B).
    /// Reset to 0 at each year rollover (after awards fire for the closing
    /// season).
    #[serde(default)]
    pub season_goals: u16,
    #[serde(default)]
    pub season_assists: u16,
    /// Real transfer value + weekly wage from the ported `FUN_0084d5d0`
    /// quality² formula (kill #2) — replaces the `CA²×100` / `CA×250`
    /// heuristics. Computed at `build` from CA/PA + the three reputation shorts.
    #[serde(default)]
    pub market_value: i64,
    #[serde(default)]
    pub weekly_wage: u32,
    /// The 12 position-aptitude bytes (type10 +0x0f..+0x1a) — carried on the
    /// RatedPlayer so `snapshot_team_for_engine` can compute real position
    /// ratings for the tactics core (kill #T `FUN_006c8930` position rating +
    /// `FUN_006c5c40` team score).
    #[serde(default)]
    pub position_aptitudes: [u8; 12],
    /// Extra type10 attributes needed by the match engine's token f32
    /// fields (populated at kickoff — see reports/token_float_field_sources_decode.md):
    /// heading +0x26, important_matches +0x27, dribbling +0x2E,
    /// decisions +0x31, throw_ins +0x40.
    #[serde(default)] pub heading: i8,
    #[serde(default)] pub important_matches: i8,
    #[serde(default)] pub dribbling: i8,
    #[serde(default)] pub decisions: i8,
    #[serde(default)] pub throw_ins: i8,
}

/// The book of player ratings — one entry per staff record with a valid
/// type10 outfield attribute record.
///
/// Built once per season-rollover and read by every awards engine call.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayerRatingBook {
    pub players: Vec<RatedPlayer>,
    /// Real club reputation (`ClubView::reputation`, the loader's ×500 value)
    /// keyed by club id — so `snapshot_team_for_engine` uses the actual club
    /// reputation instead of `avg_ca*20` (kill #1b).
    #[serde(default)]
    pub club_reputation: std::collections::BTreeMap<i32, u16>,
    /// staff_id → index into `players`, for O(1) stat updates. Not serialized
    /// (rebuilt on demand); the players vector is stable after `build`. `pub`
    /// only so cross-module `..Default::default()` literals compile — treat as
    /// internal; use `record_goal`/`record_assist` rather than touching it.
    #[serde(skip)]
    pub id_index: std::collections::HashMap<u32, usize>,
    /// Per-player season rating accumulator — VERIFIED port of
    /// `FUN_007a90b0`:132 (count++) + :154-155 (sum += display_rating).
    /// Value: `(sum_of_finalized_ratings, appearances_count)`.
    /// Consumed by [`PlayerRatingBook::season_avg_rating_of`], which uses
    /// the verified `sum / count` formula from FUN_007aa490:73-77.
    /// Populated by [`PlayerRatingBook::record_match_rating`] per XI player
    /// per match. Kills the `CA*0.8 + wobble*0.2` heuristic in
    /// [`PlayerRatingBook::season_rating`] once fed.
    #[serde(default)]
    pub season_rating_stats: std::collections::BTreeMap<u32, (u16, u8)>,
}

/// DFM-order indices into the 42-attribute block (`ATTRIBUTE_NAMES`) for the
/// five attributes the match engine consumes per player. CORRECTED from the
/// prior alphabetical-guess values — every one of these was wrong before, the
/// engine's per-player attribute reads were silently targeting the wrong bytes.
/// (Editor decode agent verified against DFM tabsheet_staff_pl2.)
const ATTR_AGGRESSION: usize        = crate::DomainStaffType10::ATTR_IDX_AGGRESSION;    // 1  (was 1, ✓)
const ATTR_BRAVERY: usize           = crate::DomainStaffType10::ATTR_IDX_BRAVERY;       // 5  (was 5, ✓)
const ATTR_DIRTINESS: usize         = crate::DomainStaffType10::ATTR_IDX_DIRTINESS;     // 18 (was 10, WRONG)
const ATTR_INJURY_PRONENESS: usize  = crate::DomainStaffType10::ATTR_IDX_INJURY_PRONE;  // 13 (was 18, WRONG)
const ATTR_JUMPING: usize           = crate::DomainStaffType10::ATTR_IDX_JUMPING;       // 14 (was 19, WRONG)
// Attributes consumed by the match-engine token f32 fields. Type10 offsets
// map to attr-array indices via `(offset - 0x1B)`. VERIFIED against the
// existing ATTR_IDX_DECISIONS (= 22 = 0x31 - 0x1B).
const ATTR_HEADING: usize           = 11; // type10 +0x26
const ATTR_IMPORTANT_MATCHES: usize = 12; // type10 +0x27
const ATTR_DRIBBLING: usize         = 19; // type10 +0x2E
const ATTR_DECISIONS: usize         = crate::DomainStaffType10::ATTR_IDX_DECISIONS; // 22 = 0x31
const ATTR_THROW_INS: usize         = 37; // type10 +0x40

impl PlayerRatingBook {
    /// Build a fresh rating book from the world's staff pool + clubs + the
    /// generated init states (kill #6 — CA/PA/attributes generated for the ~28k
    /// records the base ships as CA=0). The state is the source of truth for
    /// CA/PA/attributes so **every** player — including the previously-dropped
    /// CA=0 ones — is included with real data. This is what fills club squads
    /// so `snapshot_team_for_engine` stops falling back to the reputation
    /// scorer (kill #1).
    ///
    /// Players are represented by paired staff records: type6 (person —
    /// name, DOB, current club) + type10 (outfield attributes), joined by their
    /// shared `id` (memory [[player-init-decode]]: "person N ↔ type10[N]").
    pub fn build(
        clubs: &[DomainOpaqueRecord],
        staff: &StaffBook,
        init: &[crate::PlayerInitState],
    ) -> Self {
        // Generated per-player state (CA/PA/attributes) keyed by id.
        let mut state_by_id =
            std::collections::HashMap::with_capacity(init.len());
        for s in init {
            state_by_id.insert(s.player_id, s);
        }
        // Index type10 by staff id (for position eligibility from the raw bytes).
        let mut t10_by_id = std::collections::HashMap::with_capacity(staff.type10.len());
        for t in &staff.type10 {
            t10_by_id.insert(t.id, t);
        }
        // Index club division + reputation by club id.
        let mut club_div = std::collections::HashMap::with_capacity(clubs.len());
        let mut club_reputation = std::collections::BTreeMap::new();
        for rec in clubs {
            let cv = ClubView::new(rec);
            club_div.insert(cv.id() as i32, cv.division_id());
            club_reputation.insert(cv.id() as i32, cv.reputation());
        }

        let attr = |s: &crate::PlayerInitState, idx: usize| -> u8 {
            s.attributes.get(idx).copied().unwrap_or(0)
        };

        let mut rated = Vec::with_capacity(staff.type6.len());
        for p in &staff.type6 {
            // CRITICAL FIX: join type6 → type10 via `player_data_id` (the
            // foreign key stored on the type6 record), NOT by matching
            // type6.id == type10.id. The two ID spaces overlap coincidentally
            // but they are independent — matching by id gave every player a
            // RANDOM other player's attributes. Verified against the SWFC
            // squad: Kevin Pressman type6.id=57342 → player_data_id=47735 →
            // type10 with GK aptitude=20 (correct), whereas type10.id=57342
            // was some midfielder's record.
            // The `PlayerView::player_data_id()` accessor handles the v1/v2
            // disk-format offset difference (v1 at +0x91, v2 at +0x61).
            let pv = crate::typed_records::PlayerView::from_split(p.id, &p.body);
            let Some(pdi) = pv.player_data_id() else { continue };
            let Some(t10) = t10_by_id.get(&(pdi as u32)) else { continue };
            let sid = p.id;

            // Filter out non-player staff by `club_job` (record +0x3d).
            // 5..10 are coach/scout/physio roles.
            let club_job = pv.club_job();
            if (5..=10).contains(&club_job) { continue; }

            // Prefer the generated init state (keyed by TYPE-10 id, since the
            // init states are seeded per attribute record). Fall back to raw.
            let ca = state_by_id.get(&(pdi as u32)).map(|s| s.current_ability)
                .unwrap_or_else(|| t10.current_ability());
            if ca <= 0 { continue }
            let pa = state_by_id.get(&(pdi as u32)).map(|s| s.potential_ability)
                .unwrap_or_else(|| t10.resolved_potential_ability());
            let club_id = p.current_club_id().map(|c| c as i32);
            let div_id = club_id.and_then(|c| club_div.get(&c).copied()).flatten();
            let goals = (ca as f32 * 0.06 + wobble(sid) * 0.2) as u16;
            let age = p.age_at(DEFAULT_RATING_YEAR, DEFAULT_RATING_DAY).unwrap_or(25);
            let (position_ordinal, is_gk) = t10.engine_position_ordinal();
            // Real valuation + wage (kill #2, FUN_0084d5d0 quality²). Use the
            // init state's reputations (generated/proper for CA=0 players).
            let [rep9, rep_b, rep_d] = state_by_id.get(&(pdi as u32))
                .map(|s| s.reputation)
                .unwrap_or_else(|| {
                    let r = crate::valuation::reputations_of(t10);
                    [r.0, r.1, r.2]
                });
            let market_value = crate::valuation::market_value(ca, pa, rep9, rep_b, rep_d);
            let weekly_wage = crate::valuation::weekly_wage(ca, pa, rep9, rep_b, rep_d);
            // Real engine attributes from the shipped/generated block.
            let (aggression, bravery, dirtiness, injury_proneness, jumping_heading) =
                match state_by_id.get(&(pdi as u32)) {
                    Some(s) => (
                        attr(s, ATTR_AGGRESSION) as i8,
                        attr(s, ATTR_BRAVERY) as i8,
                        attr(s, ATTR_DIRTINESS) as i8,
                        attr(s, ATTR_INJURY_PRONENESS),
                        attr(s, ATTR_JUMPING) as i8,
                    ),
                    None => (8, 10, 5, 8, 10),
                };
            // Extras needed by the match engine's token f32 fields (item #5
            // pass-target physique + item #8 pass-bias FP delta sources).
            let (heading, important_matches, dribbling, decisions, throw_ins) =
                match state_by_id.get(&(pdi as u32)) {
                    Some(s) => (
                        attr(s, ATTR_HEADING) as i8,
                        attr(s, ATTR_IMPORTANT_MATCHES) as i8,
                        attr(s, ATTR_DRIBBLING) as i8,
                        attr(s, ATTR_DECISIONS) as i8,
                        attr(s, ATTR_THROW_INS) as i8,
                    ),
                    None => (10, 10, 10, 10, 10),
                };
            rated.push(RatedPlayer {
                staff_id: sid,
                club_id,
                division_id: div_id,
                ca,
                pa,
                goals_est: goals,
                age_est: age,
                position_ordinal,
                is_gk,
                aggression,
                bravery,
                dirtiness,
                injury_proneness,
                jumping_heading,
                season_goals: 0,
                season_assists: 0,
                market_value,
                weekly_wage,
                // Via full_attributes(): the legacy `unknown_bytes_15_26`
                // array is absent (zero) on post-migration rust-db.
                position_aptitudes: {
                    let fa = t10.full_attributes();
                    let mut a = [0u8; 12];
                    a.copy_from_slice(&fa[0..12]);
                    a
                },
                heading, important_matches, dribbling, decisions, throw_ins,
            });
        }
        let id_index = rated.iter().enumerate().map(|(i, p)| (p.staff_id, i)).collect();
        Self { players: rated, club_reputation, id_index,
               season_rating_stats: std::collections::BTreeMap::new() }
    }

    /// Record that `staff_id` now plays for `club_id`. O(1) via the id
    /// index. Returns `false` if the staff id isn't in the book.
    ///
    /// This is the propagation hook for the byte-exact boot regen
    /// (`crate::player_regen::regen_fill_club_squad`, the port of
    /// `FUN_0078E970`): that routine writes the assignment into a
    /// private copy of the type6 pool (the World is the read-only
    /// master), and the caller mirrors each pick here so the engine —
    /// which reads `club_id` off this book — sees the real player.
    ///
    /// (A previous CA-ranked "runtime version" of the regen used to
    /// live here; it duplicated the exe routine approximately and was
    /// removed so there is exactly one implementation.)
    pub fn assign_club(&mut self, staff_id: u32, club_id: i32) -> bool {
        self.ensure_index();
        match self.id_index.get(&staff_id) {
            Some(&i) => { self.players[i].club_id = Some(club_id); true }
            None => false,
        }
    }

    /// Rebuild the staff_id → index map if it's empty (e.g. after deserialize).
    fn ensure_index(&mut self) {
        if self.id_index.is_empty() && !self.players.is_empty() {
            self.id_index = self.players.iter().enumerate()
                .map(|(i, p)| (p.staff_id, i)).collect();
        }
    }

    /// Record a goal from a simulated match into the scorer's season tally —
    /// the real feed that replaces `goals_est` for the top-scorer award (kill
    /// #B). O(1) via the id index.
    pub fn record_goal(&mut self, scorer_id: u32) {
        self.ensure_index();
        if let Some(&i) = self.id_index.get(&scorer_id) {
            self.players[i].season_goals = self.players[i].season_goals.saturating_add(1);
        }
    }
    pub fn record_assist(&mut self, assist_id: u32) {
        self.ensure_index();
        if let Some(&i) = self.id_index.get(&assist_id) {
            self.players[i].season_assists = self.players[i].season_assists.saturating_add(1);
        }
    }

    /// Reset every player's accumulated season stats — called at the year
    /// rollover AFTER the closing season's awards have fired.
    pub fn reset_season_stats(&mut self) {
        for p in self.players.iter_mut() {
            p.season_goals = 0;
            p.season_assists = 0;
        }
        // Clear season rating accumulator so awards + display refresh.
        self.season_rating_stats.clear();
    }

    /// The score every award category uses as its base metric.
    ///
    /// VERIFIED path — when the player has appeared in matches this season,
    /// returns the real `sum / count` from the accumulator (VERIFIED port of
    /// FUN_007aa490:73-77 case 0x11). Falls back to the CA×0.8 + wobble×0.2
    /// heuristic only when no appearances have been recorded (pre-season /
    /// haven't played yet). The heuristic path used to fire for everyone;
    /// with the accumulator wired from [`record_match_rating`] it now only
    /// fires for players who literally haven't played.
    pub fn season_rating(&self, p: &RatedPlayer) -> f32 {
        if let Some((sum, count)) = self.season_rating_stats.get(&p.staff_id) {
            if let Some(avg) = season_avg_rating(*count, *sum as i16) {
                return avg;
            }
        }
        // Fallback: no appearances yet.
        p.ca as f32 * 0.8 + wobble(p.staff_id) * 0.2
    }

    /// Fold one match's finalized display rating (1..=10) into the season
    /// accumulator for `staff_id`. VERIFIED port of the write sequence in
    /// FUN_007a90b0:132 (count += 1) + :154-155 (sum += rating). Call once
    /// per XI player per match, AFTER `finalize_rating` produces the
    /// display byte.
    ///
    /// Bucket selection (FUN_007a90b0:39-99) — the exe stores per-competition
    /// buckets, but we currently keep a single per-season aggregate. Fine for
    /// awards + season-average display; if per-competition breakdowns land as
    /// a follow-up, promote this to a `BTreeMap<u32, BTreeMap<u16 comp, ..>>`.
    pub fn record_match_rating(&mut self, staff_id: u32, display_rating: i8) {
        if display_rating <= 0 { return; }
        let entry = self.season_rating_stats.entry(staff_id).or_insert((0, 0));
        entry.0 = entry.0.saturating_add(display_rating as u16);
        entry.1 = entry.1.saturating_add(1);
    }

    /// Direct read of the season-average rating for a player. Returns
    /// `None` when no appearances recorded.
    pub fn season_avg_rating_of(&self, staff_id: u32) -> Option<f32> {
        self.season_rating_stats.get(&staff_id)
            .and_then(|(sum, count)| season_avg_rating(*count, *sum as i16))
    }

    /// Top-scorer specifically weights the wobble higher — a striker with
    /// a lower CA can outscore a higher-CA midfielder.
    pub fn top_scorer_score(&self, p: &RatedPlayer) -> f32 {
        p.ca as f32 * 0.6 + wobble(p.staff_id) * 0.4
    }

    /// The players eligible for a competition (those whose club is in the
    /// competition's division).
    fn eligible_for(&self, division_id: i32) -> impl Iterator<Item = &RatedPlayer> {
        self.players.iter().filter(move |p| p.division_id == Some(division_id))
    }

    /// Player of the Season for a competition. Returns None if the
    /// competition has no eligible players (e.g. a cup that draws from
    /// multiple divisions — the caller should use `player_of_the_season_across`
    /// for that case).
    pub fn player_of_the_season(&self, division_id: i32) -> Option<&RatedPlayer> {
        self.eligible_for(division_id)
            .max_by(|a, b| self.season_rating(a).partial_cmp(&self.season_rating(b))
                .unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Top Scorer for a competition. Ranks by REAL accumulated season goals
    /// (kill #B — fed from actual simulated match events), with the old CA-proxy
    /// `goals_est` only as a tiebreak / pre-simulation fallback.
    pub fn top_scorer(&self, division_id: i32) -> Option<&RatedPlayer> {
        self.eligible_for(division_id)
            .max_by(|a, b| a.season_goals.cmp(&b.season_goals)
                .then(a.goals_est.cmp(&b.goals_est)))
    }

    /// Young Player of the Season — eligible == age < 21.
    pub fn young_player_of_the_season(&self, division_id: i32) -> Option<&RatedPlayer> {
        self.eligible_for(division_id)
            .filter(|p| p.age_est < 21)
            .max_by(|a, b| self.season_rating(a).partial_cmp(&self.season_rating(b))
                .unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Player of the Season across a set of divisions (for cross-division
    /// awards like "Italian Serie B Team of the Year" or "European Footballer
    /// of the Year" that draw from multiple leagues).
    pub fn player_of_the_season_across(&self, division_ids: &[i32]) -> Option<&RatedPlayer> {
        self.players.iter()
            .filter(|p| p.division_id.map_or(false, |d| division_ids.contains(&d)))
            .max_by(|a, b| self.season_rating(a).partial_cmp(&self.season_rating(b))
                .unwrap_or(std::cmp::Ordering::Equal))
    }

    /// Best XI for a division — top 11 players by season rating.
    pub fn team_of_the_season(&self, division_id: i32) -> Vec<&RatedPlayer> {
        let mut v: Vec<&RatedPlayer> = self.eligible_for(division_id).collect();
        v.sort_by(|a, b| self.season_rating(b).partial_cmp(&self.season_rating(a))
            .unwrap_or(std::cmp::Ordering::Equal));
        v.into_iter().take(11).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wobble_is_deterministic() {
        assert_eq!(wobble(42), wobble(42));
        assert!(wobble(1) != wobble(2));
        for id in [0u32, 1, 100, 10_000, u32::MAX] {
            let w = wobble(id);
            assert!((0.0..50.0).contains(&w), "wobble({}) = {}", id, w);
        }
    }

    #[test]
    fn empty_book_yields_no_awards() {
        let b = PlayerRatingBook { players: vec![], ..Default::default() };
        assert!(b.player_of_the_season(7).is_none());
        assert!(b.top_scorer(7).is_none());
        assert!(b.young_player_of_the_season(7).is_none());
        assert!(b.team_of_the_season(7).is_empty());
    }

    #[test]
    fn poty_picks_highest_rated_in_division() {
        let a = RatedPlayer { staff_id: 1, club_id: Some(10), division_id: Some(7),
                              ca: 150, pa: 160, goals_est: 10, age_est: 25, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 0, weekly_wage: 0, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] };
        let b = RatedPlayer { staff_id: 2, club_id: Some(11), division_id: Some(7),
                              ca: 180, pa: 190, goals_est: 15, age_est: 26, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 0, weekly_wage: 0, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] };
        let c = RatedPlayer { staff_id: 3, club_id: Some(20), division_id: Some(8),
                              ca: 200, pa: 200, goals_est: 30, age_est: 27, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 0, weekly_wage: 0, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] };  // other division
        let book = PlayerRatingBook { players: vec![a.clone(), b.clone(), c.clone()], ..Default::default() };
        assert_eq!(book.player_of_the_season(7).unwrap().staff_id, 2);
        assert_eq!(book.player_of_the_season(8).unwrap().staff_id, 3);
    }

    #[test]
    fn cross_division_award_picks_best_across_set() {
        let a = RatedPlayer { staff_id: 1, club_id: Some(10), division_id: Some(24),
                              ca: 170, pa: 180, goals_est: 12, age_est: 25, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 0, weekly_wage: 0, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] };
        let b = RatedPlayer { staff_id: 2, club_id: Some(11), division_id: Some(25),
                              ca: 195, pa: 195, goals_est: 25, age_est: 28, position_ordinal: 0, is_gk: false, aggression: 0, bravery: 0, dirtiness: 0, injury_proneness: 0, jumping_heading: 0, season_goals: 0, season_assists: 0, market_value: 0, weekly_wage: 0, heading: 0, important_matches: 0, dribbling: 0, decisions: 0, throw_ins: 0, position_aptitudes: [0;12] };
        let book = PlayerRatingBook { players: vec![a, b], ..Default::default() };
        assert_eq!(book.player_of_the_season_across(&[24, 25]).unwrap().staff_id, 2);
    }
}
