//! African Cup of Nations — a faithful port of `african_nations.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\comp\intercomp\african_nations.cpp`, VA
//! `0x00401000..0x00402ac5`, vtable `0x00955270`). Full decode:
//! `reports/african_nations_analysis.md`; function names:
//! `reports/carve_rename_map.json`.
//!
//! The exe treats the ACN as a **background continental competition**: it
//! constructs on a biennial (even-year) schedule anchored to 1996, draws 16
//! African national teams via the game RNG, seeds them into four groups of
//! four, plays a group stage + knockout, and reports results to the global
//! news/results manager. This module reproduces that as a real competition in
//! the headless engine: its fixtures play through the ordinary phase-2 batch
//! (`RuntimeSaveGame::execute_due_fixture_batch`) and emit `pending_events`
//! news exactly like league fixtures.
//!
//! ## Fidelity notes
//! * **Biennial year** ([`next_edition_year`]) — exact port of the ctor
//!   (`afrcup_ctor` `0x00401000`): `year+1`, snapped up to the next year with an
//!   even offset from 1996.
//! * **Continent filter** — exact: nation `+0x71 == 0` (Africa), the field
//!   `afrcup_draw_qualifiers` (`0x00402300`) reads. Verified 14/14.
//! * **≥16 gate** — exact: `afrcup_draw_qualifiers` fails (returns 0) when
//!   fewer than 16 African teams exist.
//! * **Pot codes / group seeding** — exact codes from `afrcup_seed_groups`
//!   (`0x00402810`): `[1,2,3,4, 5,5,5,5, 9,9,9,9, 0xd,0xd,0xd,0xd]`.
//! * **Draw** — the *mechanism* is the genuine game RNG (`cm_rng::MatchRng`,
//!   the port of `FUN_008fc4f0` the exe's draw calls). What is NOT yet lifted:
//!   the pre-qualified-seed injection and the sort comparator `FUN_004b3970`/
//!   `0x4b6bd0`, so which 16 teams qualify is derived here by reputation
//!   (strongest first) with an RNG tie-break rather than replicating the exe's
//!   exact seed list. Flagged, not invented.
//! * **Knockout pairing** — standard ACN cross-group bracket (A1–B2, …). Drawn
//!   knockout ties are decided by reputation as a placeholder for the penalty
//!   path (not yet decoded); flagged in the fixture `source`.
//! * **Dates** — ACN runs in January of the edition year; the exact matchday
//!   dates come from `FUN_00533ad0` (undecoded), so a documented January
//!   cadence stands in.

use serde::{Deserialize, Serialize};

use crate::typed_records::{ClubView, NationView};
use crate::{
    CmPackedDate, DomainOpaqueRecord, GameDate, HeadlessFixtureStatus, HeadlessSeasonFixture,
};

/// Africa's continent id in the shipped data (`continent.dat` id 0). VERIFIED.
pub const AFRICA_CONTINENT_ID: i32 = 0;
/// `nation_comp.dat` id of "African Cup of Nations".
pub const ACN_COMPETITION_ID: u32 = 408;
pub const ACN_COMPETITION_NAME: &str = "African Cup of Nations";
/// `0x7cc` in `afrcup_ctor` — the biennial anchor year.
pub const ACN_ANCHOR_YEAR: u16 = 1996;
pub const ACN_TEAM_COUNT: usize = 16;
pub const ACN_GROUP_COUNT: usize = 4;
pub const ACN_GROUP_SIZE: usize = 4;

/// Per-team pot/seed codes, exactly as `afrcup_seed_groups` (`0x00402810`)
/// writes them: the four group top-seeds (1..4), then pots 2/3/4 (5/9/13).
pub const ACN_POT_CODES: [u8; ACN_TEAM_COUNT] =
    [1, 2, 3, 4, 5, 5, 5, 5, 9, 9, 9, 9, 0xd, 0xd, 0xd, 0xd];

/// The next ACN edition year on or after `year+1`.
///
/// Exact port of `afrcup_ctor` (`0x00401000`): the ctor stores `param+1` then,
/// while `(y - 0x7cc) & 1 != 0`, increments `y`. Since 1996 is even this snaps
/// the start year up to the next even year — the tournament's biennial cadence.
pub fn next_edition_year(year: u16) -> u16 {
    let mut y = year.saturating_add(1);
    while ((y as i32 - ACN_ANCHOR_YEAR as i32) & 1) != 0 {
        y = y.saturating_add(1);
    }
    y
}

/// A national team drawn into the tournament.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcnTeam {
    pub club_id: u32,
    pub nation_id: i32,
    pub name: String,
    pub reputation: u16,
    /// Pot/seed code from [`ACN_POT_CODES`].
    pub pot: u8,
}

/// Which phase of the tournament the stored edition has reached. Mirrors the
/// exe's stage index (`african_nations` object `+0x30`: -1..3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcnStage {
    Groups,
    QuarterFinals,
    SemiFinals,
    Final,
    Complete,
}

/// The persisted state of the current ACN edition — enough for the save-side
/// knockout advancement to run without the base database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcnState {
    pub year: u16,
    pub competition_id: u32,
    pub competition_name: String,
    pub start_date: GameDate,
    /// Four groups of four, strength-ordered within each pot.
    pub groups: Vec<Vec<AcnTeam>>,
    pub stage: AcnStage,
    /// The champion phrasing, e.g. "African champions" / "Asian club champions".
    /// This is what makes the group+knockout engine reusable across the shared
    /// continental-cup framework (comp/eurocomp/): the news lines read from
    /// `competition_name` + `champion_noun` rather than hardcoding the ACN.
    #[serde(default = "default_champion_noun")]
    pub champion_noun: String,
    pub provenance: String,
}

fn default_champion_noun() -> String {
    "champions".to_string()
}

impl AcnState {
    fn team(&self, club_id: u32) -> Option<&AcnTeam> {
        self.groups
            .iter()
            .flatten()
            .find(|t| t.club_id == club_id)
    }
}

/// The pure algorithmic core: gather → draw → seed → group fixtures. Kept free
/// of `World`/`RuntimeSaveGame` so it is unit-testable in isolation.
pub struct AcnDraw;

impl AcnDraw {
    /// Sort the candidate pool strongest-first with a deterministic RNG
    /// tie-break, then take the top [`ACN_TEAM_COUNT`]. Returns `None` when
    /// fewer than 16 candidates exist — the exact failure of
    /// `afrcup_draw_qualifiers` (`0x00402300`), which returns 0 below 16.
    pub fn draw(mut pool: Vec<AcnTeam>, rng: &mut cm_rng::MatchRng) -> Option<Vec<AcnTeam>> {
        if pool.len() < ACN_TEAM_COUNT {
            return None;
        }
        // Reputation desc, RNG-jittered so equal-reputation teams draw randomly
        // (the exe randomises among near-equals after its comparator sort).
        let mut keyed: Vec<(u16, i32, AcnTeam)> = pool
            .drain(..)
            .map(|t| (t.reputation, rng.random(1_000), t))
            .collect();
        keyed.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
        Some(keyed.into_iter().take(ACN_TEAM_COUNT).map(|(_, _, t)| t).collect())
    }

    /// Assign pot codes and distribute the 16 strength-ordered teams into four
    /// groups — one team per pot per group. Port of `afrcup_seed_groups`
    /// (`0x00402810`): team `i` gets code `ACN_POT_CODES[i]` and lands in group
    /// `i % 4` (pot = `i / 4`).
    pub fn seed_into_groups(mut sixteen: Vec<AcnTeam>) -> Vec<Vec<AcnTeam>> {
        let mut groups: Vec<Vec<AcnTeam>> = vec![Vec::with_capacity(ACN_GROUP_SIZE); ACN_GROUP_COUNT];
        for (i, mut team) in sixteen.drain(..).take(ACN_TEAM_COUNT).enumerate() {
            team.pot = ACN_POT_CODES[i];
            groups[i % ACN_GROUP_COUNT].push(team);
        }
        groups
    }
}

/// The four-team single round-robin matchday pairs (indices into a group).
/// Each team plays the other three across three matchdays.
const GROUP_SCHEDULE: [[(usize, usize); 2]; 3] =
    [[(0, 1), (2, 3)], [(0, 2), (3, 1)], [(0, 3), (1, 2)]];

/// Generate the group-stage fixtures for a seeded edition. Two matches per
/// matchday, matchdays four days apart starting from `start_date`.
pub fn generate_group_stage(
    state: &AcnState,
    mut next_row: u32,
) -> Vec<HeadlessSeasonFixture> {
    let mut fixtures = Vec::new();
    let base = CmPackedDate::from_game_date(state.start_date.clone());
    for (gi, group) in state.groups.iter().enumerate() {
        if group.len() < ACN_GROUP_SIZE {
            continue;
        }
        for (md, pairs) in GROUP_SCHEDULE.iter().enumerate() {
            let date = base.add_days((md as i16) * 4).to_game_date();
            for &(h, a) in pairs.iter() {
                let home = &group[h];
                let away = &group[a];
                fixtures.push(HeadlessSeasonFixture {
                    row: next_row,
                    competition_id: state.competition_id,
                    competition_name: state.competition_name.clone(),
                    date: date.clone(),
                    home_club_id: home.club_id,
                    home_club_name: home.name.clone(),
                    away_club_id: away.club_id,
                    away_club_name: away.name.clone(),
                    status: HeadlessFixtureStatus::Pending,
                    home_score: None,
                    away_score: None,
                    match_packet: None,
                    match_report: None,
                    source: format!(
                        "ACN {} group {} matchday {} (afrcup_build_stage_template 0x00401310 / afrcup_seed_groups 0x00402810)",
                        state.year,
                        (b'A' + gi as u8) as char,
                        md + 1
                    ),
                });
                next_row += 1;
            }
        }
    }
    fixtures
}

// ---- Knockout progression (save-side, pure) ----

/// Round tags embedded in a fixture's `source` so the save-side advancer can
/// tell group / quarter / semi / final fixtures apart without a round field.
pub const TAG_QF: &str = "ACN-QF";
pub const TAG_SF: &str = "ACN-SF";
pub const TAG_FINAL: &str = "ACN-FINAL";

fn acn_fixtures<'a>(
    state: &AcnState,
    fixtures: &'a [HeadlessSeasonFixture],
) -> impl Iterator<Item = &'a HeadlessSeasonFixture> {
    let cid = state.competition_id;
    fixtures.iter().filter(move |f| f.competition_id == cid)
}

/// Fixtures of one knockout round, in creation (row) order.
fn round_fixtures<'a>(
    state: &AcnState,
    fixtures: &'a [HeadlessSeasonFixture],
    tag: &str,
) -> Vec<&'a HeadlessSeasonFixture> {
    let mut v: Vec<&HeadlessSeasonFixture> =
        acn_fixtures(state, fixtures).filter(|f| f.source.contains(tag)).collect();
    v.sort_by_key(|f| f.row);
    v
}

fn all_played(fixtures: &[&HeadlessSeasonFixture]) -> bool {
    !fixtures.is_empty() && fixtures.iter().all(|f| f.status == HeadlessFixtureStatus::Played)
}

/// Group tables: for each group, the club ids ranked best-first from the played
/// group fixtures. Points 3/1/0; ties broken by goal difference, goals for,
/// reputation, then id — a documented ordering (the exe's exact tiebreak chain
/// is a separate decode).
pub fn group_rankings(
    state: &AcnState,
    fixtures: &[HeadlessSeasonFixture],
) -> Vec<Vec<u32>> {
    let mut out = Vec::with_capacity(state.groups.len());
    for group in &state.groups {
        let ids: std::collections::BTreeSet<u32> = group.iter().map(|t| t.club_id).collect();
        // (points, gd, gf) per team.
        let mut tab: std::collections::BTreeMap<u32, (i32, i32, i32)> =
            group.iter().map(|t| (t.club_id, (0, 0, 0))).collect();
        for f in acn_fixtures(state, fixtures) {
            if f.status != HeadlessFixtureStatus::Played {
                continue;
            }
            if !ids.contains(&f.home_club_id) || !ids.contains(&f.away_club_id) {
                continue;
            }
            let (Some(hs), Some(as_)) = (f.home_score, f.away_score) else { continue };
            let (hs, as_) = (hs as i32, as_ as i32);
            let (hp, ap) = match hs.cmp(&as_) {
                std::cmp::Ordering::Greater => (3, 0),
                std::cmp::Ordering::Less => (0, 3),
                std::cmp::Ordering::Equal => (1, 1),
            };
            if let Some(e) = tab.get_mut(&f.home_club_id) {
                e.0 += hp;
                e.1 += hs - as_;
                e.2 += hs;
            }
            if let Some(e) = tab.get_mut(&f.away_club_id) {
                e.0 += ap;
                e.1 += as_ - hs;
                e.2 += as_;
            }
        }
        let mut ranked: Vec<u32> = group.iter().map(|t| t.club_id).collect();
        ranked.sort_by(|a, b| {
            let ta = tab[a];
            let tb = tab[b];
            let ra = state.team(*a).map(|t| t.reputation).unwrap_or(0);
            let rb = state.team(*b).map(|t| t.reputation).unwrap_or(0);
            tb.0.cmp(&ta.0)
                .then(tb.1.cmp(&ta.1))
                .then(tb.2.cmp(&ta.2))
                .then(rb.cmp(&ra))
                .then(a.cmp(b))
        });
        out.push(ranked);
    }
    out
}

/// Winner of a played knockout tie. Higher score wins; a draw is decided by
/// reputation then home advantage — a placeholder for the penalty path
/// (`african_nations.cpp` reports results to the manager but the shoot-out
/// resolution is not yet decoded).
fn ko_winner(state: &AcnState, f: &HeadlessSeasonFixture) -> Option<(u32, String)> {
    let (hs, as_) = (f.home_score?, f.away_score?);
    let home_wins = match hs.cmp(&as_) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => {
            let hr = state.team(f.home_club_id).map(|t| t.reputation).unwrap_or(0);
            let ar = state.team(f.away_club_id).map(|t| t.reputation).unwrap_or(0);
            hr >= ar
        }
    };
    Some(if home_wins {
        (f.home_club_id, f.home_club_name.clone())
    } else {
        (f.away_club_id, f.away_club_name.clone())
    })
}

fn ko_fixture(
    state: &AcnState,
    row: u32,
    date: GameDate,
    tag: &str,
    label: &str,
    home: (u32, String),
    away: (u32, String),
) -> HeadlessSeasonFixture {
    HeadlessSeasonFixture {
        row,
        competition_id: state.competition_id,
        competition_name: state.competition_name.clone(),
        date,
        home_club_id: home.0,
        home_club_name: home.1,
        away_club_id: away.0,
        away_club_name: away.1,
        status: HeadlessFixtureStatus::Pending,
        home_score: None,
        away_score: None,
        match_packet: None,
        match_report: None,
        source: format!(
            "{} {} {} (afrcup_advance_stage 0x00401e50 / afrcup_build_final 0x004020d0; drawn ties decided by reputation pending penalty decode)",
            tag, state.year, label
        ),
    }
}

/// The champion of a completed edition: the winner of its final. `None` until
/// the final is played. Used by dependent competitions (e.g. the Asian Super
/// Cup, which ties two cups' champions).
pub fn champion_of(
    state: &AcnState,
    fixtures: &[HeadlessSeasonFixture],
) -> Option<(u32, String)> {
    let fin = round_fixtures(state, fixtures, TAG_FINAL);
    let f = fin.first()?;
    if f.status != HeadlessFixtureStatus::Played {
        return None;
    }
    ko_winner(state, f)
}

/// The result of one advancement check.
#[derive(Debug, Default)]
pub struct AcnAdvance {
    pub new_fixtures: Vec<HeadlessSeasonFixture>,
    /// `(news_kind, message)` — message already `"<headline> - <summary>"`.
    pub news: Vec<(String, String)>,
    pub new_stage: Option<AcnStage>,
}

/// Advance the tournament by at most one round if the current round is complete
/// and the next round has not yet been created. Pure: returns what to append;
/// the caller mutates the save. Idempotent — a no-op until the current round's
/// fixtures are all played.
pub fn advance(state: &AcnState, fixtures: &[HeadlessSeasonFixture], next_row: u32) -> AcnAdvance {
    let mut out = AcnAdvance::default();
    let base = CmPackedDate::from_game_date(state.start_date.clone());
    match state.stage {
        AcnStage::Groups => {
            // Group stage = all ACN comp fixtures that are not knockout-tagged.
            let group_fx: Vec<&HeadlessSeasonFixture> = acn_fixtures(state, fixtures)
                .filter(|f| {
                    !f.source.contains(TAG_QF)
                        && !f.source.contains(TAG_SF)
                        && !f.source.contains(TAG_FINAL)
                })
                .collect();
            if !all_played(&group_fx) {
                return out;
            }
            let r = group_rankings(state, fixtures);
            if r.len() < 4 || r.iter().any(|g| g.len() < 2) {
                return out;
            }
            let name = |id: u32| state.team(id).map(|t| t.name.clone()).unwrap_or_default();
            // Standard ACN cross-group quarter-finals.
            let pairs = [
                (r[0][0], r[1][1]),
                (r[1][0], r[0][1]),
                (r[2][0], r[3][1]),
                (r[3][0], r[2][1]),
            ];
            let date = base.add_days(14).to_game_date();
            for (i, (h, a)) in pairs.iter().enumerate() {
                out.new_fixtures.push(ko_fixture(
                    state,
                    next_row + i as u32,
                    date.clone(),
                    TAG_QF,
                    "quarter-final",
                    (*h, name(*h)),
                    (*a, name(*a)),
                ));
            }
            out.new_stage = Some(AcnStage::QuarterFinals);
            out.news.push((
                "competition".into(),
                format!(
                    "{} - the {} group stage is over; the quarter-final line-up is set",
                    state.competition_name, state.year
                ),
            ));
        }
        AcnStage::QuarterFinals => {
            let qf = round_fixtures(state, fixtures, TAG_QF);
            if !all_played(&qf) || qf.len() < 4 {
                return out;
            }
            let w: Vec<(u32, String)> = qf.iter().filter_map(|f| ko_winner(state, f)).collect();
            if w.len() < 4 {
                return out;
            }
            let date = base.add_days(18).to_game_date();
            // Bracket: (QF1,QF3) and (QF2,QF4).
            let semis = [(w[0].clone(), w[2].clone()), (w[1].clone(), w[3].clone())];
            for (i, (h, a)) in semis.iter().enumerate() {
                out.new_fixtures.push(ko_fixture(
                    state,
                    next_row + i as u32,
                    date.clone(),
                    TAG_SF,
                    "semi-final",
                    h.clone(),
                    a.clone(),
                ));
            }
            out.new_stage = Some(AcnStage::SemiFinals);
            out.news.push((
                "competition".into(),
                format!("{} - the {} semi-finalists are decided", state.competition_name, state.year),
            ));
        }
        AcnStage::SemiFinals => {
            let sf = round_fixtures(state, fixtures, TAG_SF);
            if !all_played(&sf) || sf.len() < 2 {
                return out;
            }
            let w: Vec<(u32, String)> = sf.iter().filter_map(|f| ko_winner(state, f)).collect();
            if w.len() < 2 {
                return out;
            }
            let date = base.add_days(22).to_game_date();
            out.new_fixtures.push(ko_fixture(
                state,
                next_row,
                date,
                TAG_FINAL,
                "final",
                w[0].clone(),
                w[1].clone(),
            ));
            out.new_stage = Some(AcnStage::Final);
            out.news.push((
                "competition".into(),
                format!("{} - {} vs {} will contest the {} final", state.competition_name, w[0].1, w[1].1, state.year),
            ));
        }
        AcnStage::Final => {
            let fin = round_fixtures(state, fixtures, TAG_FINAL);
            if !all_played(&fin) {
                return out;
            }
            if let Some((_, champ)) = fin.first().and_then(|f| ko_winner(state, f)) {
                out.news.push((
                    "competition".into(),
                    format!("{} - crowned {} in {}", champ, state.champion_noun, state.year),
                ));
            }
            out.new_stage = Some(AcnStage::Complete);
        }
        AcnStage::Complete => {}
    }
    out
}

// ---- Gathering African national teams from the base world ----

/// The set of nation ids whose continent is Africa (`+0x71 == 0`).
pub fn african_nation_ids(nations: &[DomainOpaqueRecord]) -> Vec<i32> {
    nations
        .iter()
        .filter(|rec| NationView::new(rec).continent_id() == AFRICA_CONTINENT_ID)
        .map(|rec| NationView::new(rec).id() as i32)
        .collect()
}

/// Build the candidate pool: every national team (`nat_clubs`) whose nation is
/// on `continent_id`, as [`AcnTeam`]s (pot unset — assigned during seeding).
/// Shared across the `intercomp/` national-team cups (African/Asian Nations).
pub fn continental_national_teams(
    nat_clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
    continent_id: i32,
) -> Vec<AcnTeam> {
    let members: std::collections::BTreeSet<i32> = nations
        .iter()
        .filter(|rec| NationView::new(rec).continent_id() == continent_id)
        .map(|rec| NationView::new(rec).id() as i32)
        .collect();
    nat_clubs
        .iter()
        .filter_map(|rec| {
            let club = ClubView::new(rec);
            let nation_id = club.nation_id()?;
            if !members.contains(&nation_id) {
                return None;
            }
            Some(AcnTeam {
                club_id: club.id(),
                nation_id,
                name: club.primary_name(),
                reputation: club.reputation(),
                pot: 0,
            })
        })
        .collect()
}

/// The African national-team pool (continent 0).
pub fn african_national_teams(
    nat_clubs: &[DomainOpaqueRecord],
    nations: &[DomainOpaqueRecord],
) -> Vec<AcnTeam> {
    continental_national_teams(nat_clubs, nations, AFRICA_CONTINENT_ID)
}

/// The next edition year on or after `year+1` on a fixed period anchored to
/// `anchor`. Port of the intercomp ctor's year-snap: the ACN uses period 2
/// (`& 0x80000001`), the Asian Cup period 4 (`& 0x80000003`).
pub fn next_edition_year_period(year: u16, anchor: u16, period: u16) -> u16 {
    let period = period.max(1) as i32;
    let mut y = year.saturating_add(1) as i32;
    while (y - anchor as i32).rem_euclid(period) != 0 {
        y += 1;
    }
    y as u16
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biennial_year_snaps_to_even() {
        // 1996 anchor: editions in even years. next after a year lands on the
        // next even year strictly after `year`.
        assert_eq!(next_edition_year(2000), 2002);
        assert_eq!(next_edition_year(2001), 2002);
        assert_eq!(next_edition_year(2002), 2004);
        assert_eq!(next_edition_year(1995), 1996);
        assert_eq!(next_edition_year(1996), 1998);
    }

    fn team(id: u32, rep: u16) -> AcnTeam {
        AcnTeam { club_id: id, nation_id: id as i32, name: format!("Nat{id}"), reputation: rep, pot: 0 }
    }

    #[test]
    fn draw_requires_sixteen() {
        let mut rng = cm_rng::MatchRng::new((0..64).collect(), cm_rng::CrtRand::new(1));
        let pool: Vec<AcnTeam> = (0..15).map(|i| team(i, 10)).collect();
        assert!(AcnDraw::draw(pool, &mut rng).is_none(), "15 < 16 must fail like the exe");
        let pool: Vec<AcnTeam> = (0..20).map(|i| team(i, i as u16)).collect();
        let mut rng = cm_rng::MatchRng::new((0..64).collect(), cm_rng::CrtRand::new(1));
        let drawn = AcnDraw::draw(pool, &mut rng).expect("20 >= 16");
        assert_eq!(drawn.len(), 16);
        // strongest-first: the top reputations (19..=4) survive the cut.
        assert!(drawn.iter().all(|t| t.reputation >= 4));
    }

    #[test]
    fn seeding_makes_four_groups_of_four_with_pot_codes() {
        let sixteen: Vec<AcnTeam> = (0..16).map(|i| team(i, (16 - i) as u16)).collect();
        let groups = AcnDraw::seed_into_groups(sixteen);
        assert_eq!(groups.len(), 4);
        assert!(groups.iter().all(|g| g.len() == 4));
        // Each group holds one pot-1 top seed (codes 1..4, one per group) plus
        // one team from each of pots 2/3/4 (codes 5/9/13) — exactly the
        // afrcup_seed_groups distribution.
        for (gi, g) in groups.iter().enumerate() {
            let mut pots: Vec<u8> = g.iter().map(|t| t.pot).collect();
            pots.sort_unstable();
            assert_eq!(pots, vec![gi as u8 + 1, 5, 9, 0xd]);
        }
    }

    #[test]
    fn advance_draws_quarter_finals_when_groups_complete() {
        let sixteen: Vec<AcnTeam> = (0..16).map(|i| team(i, (16 - i) as u16)).collect();
        let groups = AcnDraw::seed_into_groups(sixteen);
        let state = AcnState {
            year: 2002,
            competition_id: ACN_COMPETITION_ID,
            competition_name: ACN_COMPETITION_NAME.into(),
            start_date: GameDate { year: 2002, month: 1, day: 10 },
            groups,
            stage: AcnStage::Groups,
            champion_noun: "African champions".to_string(),
            provenance: "test".into(),
        };
        let mut fx = generate_group_stage(&state, 0);
        // Play every group match: home wins 1-0 (deterministic).
        for f in &mut fx {
            f.status = HeadlessFixtureStatus::Played;
            f.home_score = Some(1);
            f.away_score = Some(0);
        }
        let next_row = fx.iter().map(|f| f.row + 1).max().unwrap_or(0);
        let adv = advance(&state, &fx, next_row);
        assert_eq!(adv.new_stage, Some(AcnStage::QuarterFinals));
        assert_eq!(adv.new_fixtures.len(), 4, "4 quarter-finals drawn");
        assert!(adv.new_fixtures.iter().all(|f| f.source.contains(TAG_QF)));
        assert_eq!(adv.news.len(), 1);
        // Every QF participant is a real drawn team.
        let ids: std::collections::BTreeSet<u32> =
            state.groups.iter().flatten().map(|t| t.club_id).collect();
        for f in &adv.new_fixtures {
            assert!(ids.contains(&f.home_club_id) && ids.contains(&f.away_club_id));
        }
    }

    #[test]
    fn group_stage_has_six_matches_per_group() {
        let sixteen: Vec<AcnTeam> = (0..16).map(|i| team(i, (16 - i) as u16)).collect();
        let groups = AcnDraw::seed_into_groups(sixteen);
        let state = AcnState {
            year: 2002,
            competition_id: ACN_COMPETITION_ID,
            competition_name: ACN_COMPETITION_NAME.into(),
            start_date: GameDate { year: 2002, month: 1, day: 10 },
            groups,
            stage: AcnStage::Groups,
            champion_noun: "African champions".to_string(),
            provenance: "test".into(),
        };
        let fx = generate_group_stage(&state, 0);
        // 4 groups x 6 matches.
        assert_eq!(fx.len(), 24);
        assert!(fx.iter().all(|f| f.competition_id == ACN_COMPETITION_ID));
    }
}
