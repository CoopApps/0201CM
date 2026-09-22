//! Verify the Positions / league-scope season-row builder (`season_rows_for_club`)
//! against synthetic archived league tables: correct final position (standard
//! points/GD/GF order), season labels, newest-first ordering, all-divisions
//! coverage. No full RuntimeSaveGame needed.
//!
//! Run: cargo run -q -p cm-domain --bin history_seasons_probe

use cm_domain::club_history::{season_rows_for_club, season_top_scorers_for_club};
use cm_domain::player_profile::AccruedSeason;
use cm_domain::{ArchivedLeagueTable, HeadlessSeasonStanding};

fn standing(club: u32, comp: u32, p: u32, w: u32, d: u32, l: u32, gf: u32, ga: u32, pts: u32) -> HeadlessSeasonStanding {
    HeadlessSeasonStanding {
        club_id: club,
        competition_id: comp,
        club_name: format!("club{club}"),
        played: p,
        won: w,
        drawn: d,
        lost: l,
        goals_for: gf,
        goals_against: ga,
        goal_difference: gf as i32 - ga as i32,
        points: pts,
    }
}

fn main() {
    let club = 5339u32; // our club
    let prem = 7u32;
    let first = 12u32;

    // 2021/22 in Premier (comp 7): our club 2nd (72 pts) behind 88, ahead of 70.
    let t2021 = ArchivedLeagueTable {
        competition_id: prem,
        season_year: 2021,
        rows: vec![
            standing(999, prem, 38, 28, 4, 6, 90, 30, 88),
            standing(club, prem, 38, 22, 6, 10, 70, 45, 72),
            standing(888, prem, 38, 21, 7, 10, 65, 44, 70),
        ],
    };
    // 2022/23 in First (comp 12): our club 1st (93 pts) — champions.
    let t2022 = ArchivedLeagueTable {
        competition_id: first,
        season_year: 2022,
        rows: vec![
            standing(club, first, 46, 28, 9, 9, 92, 40, 93),
            standing(777, first, 46, 27, 8, 11, 80, 50, 89),
        ],
    };
    // A table our club is NOT in — must be skipped.
    let t_other = ArchivedLeagueTable {
        competition_id: prem,
        season_year: 2020,
        rows: vec![standing(111, prem, 38, 30, 5, 3, 95, 25, 95)],
    };

    let name = |cid: u32| match cid {
        7 => "Premier Division".to_string(),
        12 => "First Division".to_string(),
        _ => format!("comp{cid}"),
    };

    let (positions, leagues) =
        season_rows_for_club(&[t2021, t2022, t_other], club, name);

    println!("POSITIONS ({}):", positions.len());
    for p in &positions {
        println!("  {} - {} in {}", p.season, p.position, p.division);
    }
    println!("LEAGUE SEASONS ({}):", leagues.len());
    for l in &leagues {
        println!(
            "  {} {} {} P{} W{} D{} L{} F{} A{} Pts{}",
            l.season, l.position, l.league, l.played, l.won, l.drawn, l.lost, l.goals_for, l.goals_against, l.points
        );
    }

    // Checks.
    let mut pass = true;
    let mut check = |name: &str, cond: bool| {
        println!("CHECK {name}: {}", if cond { "PASS" } else { pass = false; "FAIL" });
    };
    check("2 rows (skip table without our club)", positions.len() == 2 && leagues.len() == 2);
    check("newest-first (2022/3 before 2021/2)", positions[0].season == "2022/3" && positions[1].season == "2021/2");
    check("2022/3 = 1st in First Division", positions[0].position == "1st" && positions[0].division == "First Division");
    check("2021/2 = 2nd in Premier Division", positions[1].position == "2nd" && positions[1].division == "Premier Division");
    check("league row P/W/D/L/F/A/Pts for 2021/2", {
        let l = &leagues[1];
        l.played == 38 && l.won == 22 && l.drawn == 6 && l.lost == 10 && l.goals_for == 70 && l.goals_against == 45 && l.points == 72
    });

    // --- Players "Top Goalscorer" from accrued player-seasons ---
    let seasons = vec![
        AccruedSeason { person_id: 10, year: 2021, club_id: club as i32, apps: 30, goals: 18 },
        AccruedSeason { person_id: 11, year: 2021, club_id: club as i32, apps: 28, goals: 24 }, // top 2021
        AccruedSeason { person_id: 12, year: 2021, club_id: 999, apps: 30, goals: 40 },          // other club, ignore
        AccruedSeason { person_id: 11, year: 2022, club_id: club as i32, apps: 20, goals: 9 },
        AccruedSeason { person_id: 13, year: 2022, club_id: club as i32, apps: 34, goals: 15 },  // top 2022
        AccruedSeason { person_id: 14, year: 2023, club_id: club as i32, apps: 10, goals: 0 },   // no goals -> season omitted
    ];
    let pname = |pid: u32| format!("Player{pid}");
    let scorers = season_top_scorers_for_club(&seasons, club, pname);
    println!("\nTOP SCORERS ({}):", scorers.len());
    for r in &scorers {
        println!("  {} {} - {}", r.season, r.value, r.player);
    }
    check("2 scorer rows (0-goal season omitted)", scorers.len() == 2);
    check("newest-first 2022/3 then 2021/2", scorers[0].season == "2022/3" && scorers[1].season == "2021/2");
    check("2022/3 top = Player13 - 15", scorers[0].player == "Player13" && scorers[0].value == "15");
    check("2021/2 top = Player11 - 24 (club-scoped)", scorers[1].player == "Player11" && scorers[1].value == "24");

    std::process::exit(if pass { 0 } else { 1 });
}
