//! Port of `mini_cup.cpp` — compact cup bracket widget.
//!
//! Shown on dashboards, news items and cup preview screens: a compressed
//! bracket view of a domestic knockout showing the current round's ties,
//! the winners so far, and the manager's own tie highlighted. Data comes
//! from an already-generated `domestic_cup::CupState` + the fixture
//! results captured in the season plan.

use serde::{Deserialize, Serialize};

/// One tie in the mini-cup view.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiniCupTie {
    /// Round index (0 = first round).
    pub round: u8,
    pub home_club_id: u32,
    pub home_club_name: String,
    pub away_club_id: u32,
    pub away_club_name: String,
    /// `None` when the tie hasn't been played yet.
    pub result: Option<(u8, u8)>,
    /// True when the tie has been played and the winner emerged — for
    /// bracket-line rendering.
    pub advanced: bool,
    /// True when this is the manager's club — the UI paints it highlighted.
    pub highlighted: bool,
}

/// The compact bracket rendered by the widget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiniCupView {
    pub competition_name: String,
    pub year: u16,
    /// The current round the manager is focused on (0 = first round).
    pub current_round: u8,
    /// All ties in the CURRENT round.
    pub current_round_ties: Vec<MiniCupTie>,
    /// A one-line summary of the previous round for context ("QF: X won 2-1 vs Y",
    /// etc.).
    pub previous_round_summaries: Vec<String>,
}

/// One entry the caller passes in per tie (agnostic of the parent's
/// fixture representation).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TieInput<'a> {
    pub round: u8,
    pub home_club_id: u32,
    pub home_club_name: &'a str,
    pub away_club_id: u32,
    pub away_club_name: &'a str,
    pub result: Option<(u8, u8)>,
}

/// Build a mini-cup view centred on `focus_round`. The current-round list
/// contains every tie at that round; previous rounds are folded into
/// text summaries.
pub fn build(
    competition_name: &str,
    year: u16,
    focus_round: u8,
    ties: &[TieInput],
    focus_club_id: u32,
) -> MiniCupView {
    let mut current = Vec::new();
    let mut prev_summaries: Vec<String> = Vec::new();

    for t in ties {
        let tie = MiniCupTie {
            round: t.round,
            home_club_id: t.home_club_id,
            home_club_name: t.home_club_name.to_string(),
            away_club_id: t.away_club_id,
            away_club_name: t.away_club_name.to_string(),
            result: t.result,
            advanced: t.result.is_some(),
            highlighted: t.home_club_id == focus_club_id || t.away_club_id == focus_club_id,
        };
        if t.round == focus_round {
            current.push(tie);
        } else if t.round < focus_round {
            if let Some((h, a)) = t.result {
                let winner_name = if h >= a { t.home_club_name } else { t.away_club_name };
                let loser_name  = if h >= a { t.away_club_name } else { t.home_club_name };
                prev_summaries.push(format!(
                    "R{}: {} beat {} {}-{}",
                    t.round + 1, winner_name, loser_name, h, a,
                ));
            }
        }
    }

    // Sort current-round ties by home club name so the widget is
    // deterministic across calls.
    current.sort_by(|a, b| a.home_club_name.cmp(&b.home_club_name));

    MiniCupView {
        competition_name: competition_name.to_string(),
        year,
        current_round: focus_round,
        current_round_ties: current,
        previous_round_summaries: prev_summaries,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_round_only_contains_focus_round() {
        let ties = vec![
            TieInput { round: 0, home_club_id: 1, home_club_name: "Aberdeen",
                       away_club_id: 2, away_club_name: "Celtic",
                       result: Some((1, 2)) },
            TieInput { round: 1, home_club_id: 2, home_club_name: "Celtic",
                       away_club_id: 3, away_club_name: "Dundee",
                       result: None },
        ];
        let v = build("Scottish Cup", 2001, 1, &ties, 2);
        assert_eq!(v.current_round_ties.len(), 1);
        assert_eq!(v.current_round_ties[0].round, 1);
    }

    #[test]
    fn previous_round_summaries_come_from_completed_earlier_ties() {
        let ties = vec![
            TieInput { round: 0, home_club_id: 1, home_club_name: "Aberdeen",
                       away_club_id: 2, away_club_name: "Celtic",
                       result: Some((1, 2)) },
            TieInput { round: 1, home_club_id: 2, home_club_name: "Celtic",
                       away_club_id: 3, away_club_name: "Dundee",
                       result: None },
        ];
        let v = build("Scottish Cup", 2001, 1, &ties, 2);
        assert_eq!(v.previous_round_summaries.len(), 1);
        assert!(v.previous_round_summaries[0].contains("Celtic beat Aberdeen"));
    }

    #[test]
    fn manager_tie_gets_highlighted() {
        let ties = vec![
            TieInput { round: 0, home_club_id: 1, home_club_name: "A",
                       away_club_id: 2, away_club_name: "B", result: None },
            TieInput { round: 0, home_club_id: 3, home_club_name: "C",
                       away_club_id: 4, away_club_name: "D", result: None },
        ];
        let v = build("Cup", 2001, 0, &ties, 3);
        assert_eq!(v.current_round_ties.iter().filter(|t| t.highlighted).count(), 1);
    }
}
