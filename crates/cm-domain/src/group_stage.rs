//! Group-stage engine — port of `sub_league.cpp` / `league_stage.cpp`.
//!
//! Unblocks the format used by `euro_champ_qual.cpp` (Euro qualifying),
//! `intertoto_cup.cpp`, `ita_cup.cpp` group phase, `nir_lge_cup.cpp`
//! group phase, `ire_lge_cup.cpp` group phase, and every continental
//! qualifying tournament.
//!
//! # Model
//!
//! A group stage = N groups × K teams. Within a group every team plays
//! every other twice (double round-robin). Top M teams from each group
//! advance to a knockout that follows.
//!
//! # API
//!
//! * [`GroupStageState::build`] — set up groups from a flat pool.
//! * [`GroupStageState::generate_fixtures`] — emit the fixture list.
//! * [`GroupStageState::standings`] — compute per-group standings from
//!   collected fixture results.
//! * [`GroupStageState::qualifiers`] — top M from each group post-standings.

use serde::{Deserialize, Serialize};

use crate::arg_primera::ArgTeam;

/// One group in a group stage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Group {
    pub label: String,
    pub teams: Vec<ArgTeam>,
}

/// A collected fixture result for a group stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GroupFixtureResult {
    pub group_index: u8,
    pub home_club_id: u32,
    pub away_club_id: u32,
    pub home_goals: u8,
    pub away_goals: u8,
}

/// One row of standings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandingsRow {
    pub club_id: u32,
    pub club_name: String,
    pub played: u16,
    pub wins: u16,
    pub draws: u16,
    pub losses: u16,
    pub goals_for: u16,
    pub goals_against: u16,
    pub points: u16,
}

impl StandingsRow {
    pub fn goal_difference(&self) -> i32 {
        self.goals_for as i32 - self.goals_against as i32
    }
}

/// The group-stage state: N groups and every fixture result collected.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GroupStageState {
    pub competition_id: i32,
    pub competition_name: String,
    pub year: u16,
    pub groups: Vec<Group>,
    pub results: Vec<GroupFixtureResult>,
    /// How many top teams per group qualify for the next round.
    pub qualifiers_per_group: u8,
}

impl GroupStageState {
    /// Set up `num_groups` from the flat pool, distributing teams
    /// snake-draft style (highest-reputation team goes to group 0, next
    /// to group 1, ..., wrap back). Guarantees each group is
    /// approximately balanced.
    // GDI-REG: 00877450 PORTED_BEHAVIOURAL
    pub fn build(
        pool: Vec<ArgTeam>,
        competition_id: i32,
        competition_name: &str,
        year: u16,
        num_groups: u8,
        qualifiers_per_group: u8,
    ) -> Option<Self> {
        if pool.len() < num_groups as usize * 2 || num_groups == 0 {
            return None;
        }
        let mut sorted = pool;
        sorted.sort_by(|a, b| b.reputation.cmp(&a.reputation).then(a.club_id.cmp(&b.club_id)));
        let mut groups: Vec<Group> = (0..num_groups)
            .map(|i| Group { label: format!("Group {}", (b'A' + i) as char), teams: vec![] })
            .collect();
        for (i, team) in sorted.into_iter().enumerate() {
            let g = i % num_groups as usize;
            groups[g].teams.push(team);
        }
        Some(Self {
            competition_id,
            competition_name: competition_name.to_string(),
            year,
            groups,
            results: Vec::new(),
            qualifiers_per_group,
        })
    }

    /// Generate every intra-group fixture. Double round-robin per group.
    /// Returns a list of `(group_index, home_id, away_id)` triples.
    // GDI-REG: 00877770 PORTED_BEHAVIOURAL
    pub fn generate_fixtures(&self) -> Vec<(u8, u32, u32)> {
        let mut out = Vec::new();
        for (gi, group) in self.groups.iter().enumerate() {
            for (hi, home) in group.teams.iter().enumerate() {
                for (ai, away) in group.teams.iter().enumerate() {
                    if hi != ai {
                        out.push((gi as u8, home.club_id, away.club_id));
                    }
                }
            }
        }
        out
    }

    pub fn record_result(&mut self, r: GroupFixtureResult) {
        self.results.push(r);
    }

    /// Standings per group, sorted by (points desc, goal_diff desc, GF desc).
    pub fn standings(&self) -> Vec<Vec<StandingsRow>> {
        let mut by_group: Vec<Vec<StandingsRow>> = self.groups.iter()
            .map(|g| g.teams.iter().map(|t| StandingsRow {
                club_id: t.club_id, club_name: t.name.clone(),
                played: 0, wins: 0, draws: 0, losses: 0,
                goals_for: 0, goals_against: 0, points: 0,
            }).collect())
            .collect();
        for r in &self.results {
            let group_rows = &mut by_group[r.group_index as usize];
            for row in group_rows.iter_mut() {
                if row.club_id == r.home_club_id {
                    row.played += 1;
                    row.goals_for += r.home_goals as u16;
                    row.goals_against += r.away_goals as u16;
                    if r.home_goals > r.away_goals { row.wins += 1; row.points += 3; }
                    else if r.home_goals == r.away_goals { row.draws += 1; row.points += 1; }
                    else { row.losses += 1; }
                } else if row.club_id == r.away_club_id {
                    row.played += 1;
                    row.goals_for += r.away_goals as u16;
                    row.goals_against += r.home_goals as u16;
                    if r.away_goals > r.home_goals { row.wins += 1; row.points += 3; }
                    else if r.away_goals == r.home_goals { row.draws += 1; row.points += 1; }
                    else { row.losses += 1; }
                }
            }
        }
        for group_rows in &mut by_group {
            group_rows.sort_by(|a, b| b.points.cmp(&a.points)
                .then(b.goal_difference().cmp(&a.goal_difference()))
                .then(b.goals_for.cmp(&a.goals_for)));
        }
        by_group
    }

    /// The top-N clubs per group who advance to the knockout stage.
    pub fn qualifiers(&self) -> Vec<u32> {
        let s = self.standings();
        let mut out = Vec::new();
        for group in s {
            for row in group.iter().take(self.qualifiers_per_group as usize) {
                out.push(row.club_id);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn teams(n: usize) -> Vec<ArgTeam> {
        (0..n).map(|i| ArgTeam {
            club_id: 100 + i as u32,
            name: format!("Club {}", i),
            reputation: (10 + i as u16 * 2),
        }).collect()
    }

    #[test]
    fn build_distributes_teams_across_groups() {
        let s = GroupStageState::build(teams(16), 100, "Test", 2001, 4, 2).unwrap();
        assert_eq!(s.groups.len(), 4);
        for g in &s.groups { assert_eq!(g.teams.len(), 4); }
    }

    #[test]
    fn generate_fixtures_produces_double_round_robin_per_group() {
        let s = GroupStageState::build(teams(16), 100, "Test", 2001, 4, 2).unwrap();
        // 4 teams per group → 4*3 = 12 fixtures per group (double round-robin) × 4 groups = 48
        assert_eq!(s.generate_fixtures().len(), 48);
    }

    #[test]
    fn standings_reflect_recorded_results() {
        let mut s = GroupStageState::build(teams(4), 100, "Test", 2001, 1, 2).unwrap();
        // Club 100 beats Club 101 2-0, Club 100 draws Club 102 1-1
        let ids: Vec<u32> = s.groups[0].teams.iter().map(|t| t.club_id).collect();
        s.record_result(GroupFixtureResult {
            group_index: 0, home_club_id: ids[0], away_club_id: ids[1],
            home_goals: 2, away_goals: 0,
        });
        s.record_result(GroupFixtureResult {
            group_index: 0, home_club_id: ids[0], away_club_id: ids[2],
            home_goals: 1, away_goals: 1,
        });
        let st = s.standings();
        assert_eq!(st[0][0].club_id, ids[0]);
        assert_eq!(st[0][0].points, 4);
        assert_eq!(st[0][0].played, 2);
    }

    #[test]
    fn qualifiers_takes_top_n_from_each_group() {
        let mut s = GroupStageState::build(teams(8), 100, "Test", 2001, 2, 1).unwrap();
        // Fake results — record enough for standings to have order
        let g0 = s.groups[0].teams.iter().map(|t| t.club_id).collect::<Vec<_>>();
        for i in 1..g0.len() {
            s.record_result(GroupFixtureResult {
                group_index: 0, home_club_id: g0[0], away_club_id: g0[i],
                home_goals: 3, away_goals: 0,
            });
        }
        let q = s.qualifiers();
        assert_eq!(q.len(), 2);   // 2 groups × 1 qualifier each
        assert!(q.contains(&g0[0]));
    }

    #[test]
    fn empty_pool_returns_none() {
        assert!(GroupStageState::build(vec![], 100, "T", 2001, 4, 2).is_none());
    }
}
