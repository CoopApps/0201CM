//! International (national-team) match routing + cap/appearance crediting.
//!
//! Directive: the exe plays national-team fixtures through the SAME general
//! match engine as clubs (`match_day_play`→`match_finalize`→`FUN_00753240`),
//! and credits caps/goals/debuts as a branch inside the shared finalize
//! (`FUN_00753240`), gated to national-team fixtures. Caps live at
//! `person+0x22`, intl goals at `+0x23`; a debut (`caps==0`) fires
//! `FUN_00855c00` → person-history code 6 "First international cap against
//! <Nation> aged <age>". Decode: `reports/international_caps_decode.md`.
//!
//! So we route national fixtures through the real `match_engine_exe`
//! (`national_snapshot` builds the XI from the called-up squad — every
//! attribute the engine needs is already on `RatedPlayer`) and port the
//! crediting step here — NOT bolt counters onto the team-strength shortcut.

use crate::match_engine_exe::{EngineTeamPlayer, EngineTeamSnapshot};
use crate::{RuntimeSaveGame, World};

/// Accrued international caps/goals for one person (added to the shipped
/// `international_caps`/`international_goals` starting values).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct IntlCapRow {
    pub person_id: u32,
    pub caps: u16,
    pub goals: u16,
}

impl World {
    /// Build a national-team engine snapshot: the nation's best 20 rated
    /// players (by CA), each mapped to the same `EngineTeamPlayer` the club
    /// path uses. Nationality is resolved per player via the person record.
    pub fn national_snapshot(&self, save: &RuntimeSaveGame, nation_id: i32) -> Option<EngineTeamSnapshot> {
        use crate::typed_records::{PlayerView, NationView};
        let mut squad: Vec<&crate::player_rating::RatedPlayer> = save.player_ratings.players.iter()
            .filter(|p| self.person_nation_id(p.staff_id) == Some(nation_id))
            .collect();
        if squad.len() < 7 { return None; } // not enough for a match
        squad.sort_by(|a, b| b.ca.cmp(&a.ca));
        let players: Vec<EngineTeamPlayer> = squad.iter().take(20).map(|p| EngineTeamPlayer {
            player_id: p.staff_id,
            is_not_injured: save.injuries.is_available(p.staff_id),
            position: p.position_ordinal,
            jumping_heading: p.jumping_heading,
            aggression: p.aggression,
            bravery: p.bravery,
            dirtiness: p.dirtiness,
            current_ability: p.ca.max(1) as u16 * 100,
            age: p.age_est,
            injury_proneness: p.injury_proneness,
            form: 12,
            is_first_choice_gk: p.is_gk,
            speciality_a: 0, speciality_b: 0,
            position_natural: p.position_ordinal, position_learn: 0,
            heading: p.heading,
            important_matches: p.important_matches,
            dribbling: p.dribbling,
            decisions: p.decisions,
            throw_ins: p.throw_ins,
        }).collect();
        let reputation = self.core.nations.iter()
            .map(NationView::new)
            .find(|nv| nv.id() as i32 == nation_id)
            .map(|nv| nv.reputation())
            .unwrap_or(5000);
        let _ = PlayerView::from_split; // keep import used across cfgs
        Some(EngineTeamSnapshot {
            club_id: nation_id.max(0) as u32,
            reputation,
            grudge_score: 0,
            players,
            sum_position_ratings: 0,
            out_of_position_ids: Vec::new(),
            team_settings: crate::tactic_file::TeamSettings::default(),
            slot_movement_tokens: [0; 11],
        })
    }

    /// The nation id a person represents internationally (their nationality),
    /// or `None`. Reads the person record's nation field.
    fn person_nation_id(&self, person_id: u32) -> Option<i32> {
        use crate::typed_records::PlayerView;
        let person = self.staff.type6.iter().find(|p| p.id == person_id)?;
        PlayerView::from_split(person.id, &person.body).nation_id()
    }

    /// Play a national-team fixture through the real match engine and credit
    /// caps/goals/debuts to the appearing XI of both sides (port of the
    /// crediting branch in `FUN_00753240`). `full_cap` gates whether it
    /// counts as a senior cap (friendlies/qualifiers/finals) vs B/U21/Olympic
    /// (no cap). Returns `(home_score, away_score)`. `date` is "D.M.YY" +
    /// year for the debut record; `game_day` for ordering.
    #[allow(clippy::too_many_arguments)]
    pub fn play_and_credit_national_fixture(
        &self,
        save: &mut RuntimeSaveGame,
        home_nation: i32,
        away_nation: i32,
        full_cap: bool,
        seed: u64,
        game_day: u32,
        date: String,
        year: u16,
    ) -> Option<(u8, u8)> {
        let home = self.national_snapshot(save, home_nation)?;
        let away = self.national_snapshot(save, away_nation)?;
        let outcome = crate::match_engine_exe::simulate_one_fixture(&home, &away, seed, None);

        if full_cap {
            // Appearances = the rated XI (per_player_ratings) of each side.
            let home_ids: Vec<u32> = outcome.per_player_ratings.iter().map(|(id, _)| *id)
                .filter(|id| home.players.iter().any(|p| p.player_id == *id)).collect();
            let away_ids: Vec<u32> = outcome.per_player_ratings.iter().map(|(id, _)| *id)
                .filter(|id| away.players.iter().any(|p| p.player_id == *id)).collect();
            let home_goals: Vec<u32> = outcome.home_scorer_ids.clone();
            let away_goals: Vec<u32> = outcome.away_scorer_ids.clone();
            self.credit_caps(save, &home_ids, &home_goals, away_nation, game_day, &date, year);
            self.credit_caps(save, &away_ids, &away_goals, home_nation, game_day, &date, year);
        }
        Some((outcome.home_score, outcome.away_score))
    }

    /// Credit one team's appearances: +1 cap each, + goals; a player whose
    /// TOTAL caps (shipped + accrued) was 0 gets a debut achievement (code 6:
    /// "First international cap against <opponent nation> aged <age>").
    fn credit_caps(
        &self,
        save: &mut RuntimeSaveGame,
        appearances: &[u32],
        scorers: &[u32],
        opponent_nation: i32,
        game_day: u32,
        date: &str,
        year: u16,
    ) {
        use crate::typed_records::NationView;
        let opp_name = self.core.nations.iter().map(NationView::new)
            .find(|nv| nv.id() as i32 == opponent_nation)
            .map(|nv| nv.nationality_name())
            .unwrap_or_default();
        for &pid in appearances {
            let shipped = self.shipped_intl_caps(pid);
            let row_idx = save.intl_caps.iter().position(|r| r.person_id == pid);
            let accrued_before = row_idx.map(|i| save.intl_caps[i].caps).unwrap_or(0);
            let debut = shipped == 0 && accrued_before == 0;
            let goals_here = scorers.iter().filter(|&&s| s == pid).count() as u16;
            match row_idx {
                Some(i) => {
                    save.intl_caps[i].caps = save.intl_caps[i].caps.saturating_add(1);
                    save.intl_caps[i].goals = save.intl_caps[i].goals.saturating_add(goals_here);
                }
                None => save.intl_caps.push(IntlCapRow { person_id: pid, caps: 1, goals: goals_here }),
            }
            if debut {
                let age = self.person_age_for(pid, year);
                let opp = if opp_name.is_empty() { "an international side".to_string() } else { opp_name.clone() };
                save.player_achievements.record(
                    pid, game_day, date.to_string(), 0, String::new(),
                    format!("First international cap against {opp} aged {age}"),
                    crate::player_achievements::AchievementKind::Other,
                );
            }
        }
    }

    /// Shipped starting international caps (the type10/person attribute).
    fn shipped_intl_caps(&self, person_id: u32) -> u16 {
        use crate::typed_records::PlayerView;
        self.staff.type6.iter().find(|p| p.id == person_id)
            .map(|p| PlayerView::from_split(p.id, &p.body).international_caps() as u16)
            .unwrap_or(0)
    }

    /// Player age at `year` (for the debut line).
    fn person_age_for(&self, person_id: u32, year: u16) -> u8 {
        self.staff.type6.iter().find(|p| p.id == person_id)
            .and_then(|p| p.age_at(year, crate::day_of_year(year, 6, 1)))
            .unwrap_or(24)
    }
}

impl RuntimeSaveGame {
    /// The active NATIONAL-team continental comps (not the club ones —
    /// Asian Champions League / Cup Winners are excluded, their players
    /// don't earn caps).
    fn national_comps(&self) -> impl Iterator<Item = &crate::african_nations::AcnState> {
        [
            &self.african_nations,
            &self.asia_cup_of_nations,
            &self.european_championship,
            &self.fifa_confederations_cup,
            &self.concacaf_gold_cup,
        ].into_iter().flatten()
    }

    /// If `(comp_id, home_club, away_club)` is a national-comp fixture,
    /// resolve both sides' nation ids from that comp's team list.
    fn fixture_nations(&self, comp_id: u32, home_club: u32, away_club: u32) -> Option<(i32, i32)> {
        for st in self.national_comps() {
            if st.competition_id != comp_id { continue; }
            let find = |cid: u32| st.groups.iter().flatten().find(|t| t.club_id == cid).map(|t| t.nation_id);
            if let (Some(h), Some(a)) = (find(home_club), find(away_club)) {
                return Some((h, a));
            }
        }
        None
    }

    /// Play every DUE national-team fixture through the real match engine
    /// (real squads → appearances) and credit caps/goals/debuts, marking
    /// them Played so the World-free club commit skips them. Called from
    /// `tick_days_bound` (World-aware) before the club fixture pass.
    pub fn play_due_national_fixtures(&mut self, world: &World, date: &crate::GameDate) {
        let due: Vec<(u32, i32, i32)> = self.season.fixtures.iter()
            .filter(|f| f.status == crate::HeadlessFixtureStatus::Pending && f.date <= *date)
            .filter_map(|f| self.fixture_nations(f.competition_id, f.home_club_id, f.away_club_id)
                .map(|(h, a)| (f.row, h, a)))
            .collect();
        for (row, hn, an) in due {
            let day = self.elapsed_days;
            let dstr = format!("{}.{}.{:02}", self.date.day, self.date.month, self.date.year % 100);
            let yr = self.date.year;
            let seed = (row as u64).wrapping_mul(0x9e3779b97f4a7c15).wrapping_add(day as u64);
            // All modelled continental national comps are senior (full caps).
            if let Some((hs, as_)) = world.play_and_credit_national_fixture(self, hn, an, true, seed, day, dstr, yr) {
                if let Some(f) = self.season.fixtures.iter_mut().find(|f| f.row == row) {
                    f.home_score = Some(hs);
                    f.away_score = Some(as_);
                    f.status = crate::HeadlessFixtureStatus::Played;
                }
            }
        }
    }
}
