//! Port of `mini_league.cpp` — compact league standings widget.
//!
//! The exe's mini-league renders a 3-4 row compressed table (position,
//! club, played, points) shown on dashboards, news items and cup preview
//! screens. Data comes from an already-ranked league table; this module
//! defines the row model + a `build` helper that slices the parent
//! `simple_league::ranked_table` into a mini view around a target club.

use serde::{Deserialize, Serialize};

/// One row of the fully-ranked parent standings, as supplied by the caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StandingsRow {
    pub club_id: u32,
    pub club_name: String,
    pub played: u16,
    pub points: u16,
    pub goal_difference: i16,
}

/// One row of the mini-league window shown by the widget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiniLeagueRow {
    /// 1-indexed position in the parent league.
    pub position: u16,
    pub club_id: u32,
    pub club_name: String,
    pub played: u16,
    pub points: u16,
    pub goal_difference: i16,
    /// True when this row is the manager's own club — the UI paints it
    /// in the highlight colour.
    pub highlighted: bool,
}

/// The compact table the widget renders.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MiniLeagueView {
    pub competition_name: String,
    pub year: u16,
    pub rows: Vec<MiniLeagueRow>,
}

/// How many rows the mini-league widget shows. The exe's compact layout
/// carries four (verified against dashboard geometry).
pub const MINI_ROWS: usize = 4;

/// Build a mini-league view centred on `focus_club_id` from a
/// pre-ranked full standings list. Shows one row above and two below the
/// focus club. If the focus is near the top or bottom, the window slides
/// to keep four rows visible. If the parent has fewer than four clubs,
/// the view returns all of them.
pub fn build(
    competition_name: &str,
    year: u16,
    ranked: &[StandingsRow],
    focus_club_id: u32,
) -> MiniLeagueView {
    let n = ranked.len();
    if n == 0 {
        return MiniLeagueView {
            competition_name: competition_name.to_string(), year, rows: vec![],
        };
    }
    let focus_ix = ranked.iter()
        .position(|r| r.club_id == focus_club_id)
        .unwrap_or(0);

    let mut lo = focus_ix.saturating_sub(1);
    let mut hi = (lo + MINI_ROWS).min(n);
    if hi - lo < MINI_ROWS && lo > 0 {
        lo = hi.saturating_sub(MINI_ROWS);
    }

    let rows = ranked[lo..hi].iter().enumerate().map(|(off, r)| MiniLeagueRow {
        position: (lo + off + 1) as u16,
        club_id: r.club_id,
        club_name: r.club_name.clone(),
        played: r.played,
        points: r.points,
        goal_difference: r.goal_difference,
        highlighted: r.club_id == focus_club_id,
    }).collect();

    MiniLeagueView { competition_name: competition_name.to_string(), year, rows }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk(n: usize) -> Vec<StandingsRow> {
        (0..n).map(|i| StandingsRow {
            club_id: 100 + i as u32,
            club_name: format!("Club {}", i),
            played: 10, points: (30 - i as u16), goal_difference: 5 - i as i16,
        }).collect()
    }

    #[test]
    fn window_centres_around_focus() {
        let rows = mk(20);
        let v = build("Test", 2001, &rows, 105);
        assert_eq!(v.rows.len(), 4);
        assert!(v.rows.iter().any(|r| r.club_id == 105 && r.highlighted));
    }

    #[test]
    fn window_slides_up_at_bottom() {
        let rows = mk(20);
        let v = build("Test", 2001, &rows, 119);
        assert_eq!(v.rows.len(), 4);
        assert!(v.rows.iter().any(|r| r.club_id == 119));
    }

    #[test]
    fn only_focus_row_is_highlighted() {
        let rows = mk(8);
        let v = build("Test", 2001, &rows, 103);
        assert_eq!(v.rows.iter().filter(|r| r.highlighted).count(), 1);
    }

    #[test]
    fn returns_all_when_parent_smaller_than_window() {
        let rows = mk(2);
        let v = build("Test", 2001, &rows, 100);
        assert_eq!(v.rows.len(), 2);
    }

    #[test]
    fn empty_parent_yields_empty_view() {
        let v = build("Test", 2001, &[], 100);
        assert!(v.rows.is_empty());
    }
}
