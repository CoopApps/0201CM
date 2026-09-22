//! Verify the ClubSeasonRecords accumulator against the DECODED exe rules
//! (FUN_00445220 + compares): biggest-win margin & goals-scored tiebreak,
//! biggest-defeat, highest-scoring total (strict), league-only variants,
//! streak transitions (win/draw/loss) and season-max, date-range display.
//!
//! Run: cargo run -q -p cm-domain --bin season_records_probe

use cm_domain::club_season_records::{ClubSeasonRecords, MatchInput};
use cm_domain::GameDate;

fn m(our: u32, their: u32, opp: u32, home: bool, comp: u32, round: &str, league: bool, d: u8, mo: u8, y: u16) -> MatchInput {
    MatchInput {
        our_goals: our,
        their_goals: their,
        opponent_id: opp,
        home,
        competition_id: comp,
        round: round.to_string(),
        is_league: league,
        date: GameDate { year: y, month: mo, day: d },
    }
}

// Test formatters: opponent id -> name, (comp,round) -> display.
fn cname(id: u32) -> String {
    match id { 1 => "Gillingham", 2 => "Hull", 3 => "Leeds", 4 => "Arsenal", 5 => "Everton", 6 => "Chelsea", 7 => "Spurs", _ => "?" }.to_string()
}
fn cdisp(comp: u32, round: &str) -> String {
    let base = match comp { 100 => "Premier Division", 200 => "FA Cup", 300 => "League Cup", _ => "Comp" };
    if round.is_empty() { base.to_string() } else { format!("{base} {round}") }
}

fn main() {
    let mut pass = true;
    let mut check = |name: &str, cond: bool| {
        println!("CHECK {name}: {}", if cond { "PASS" } else { pass = false; "FAIL" });
    };

    let mut rec = ClubSeasonRecords::new(42, 2037);
    // Chronological fixtures. (opp id, home, comp id, round, is_league, d.m.y)
    rec.update_with_match(&m(4, 0, 1, true, 100, "", true, 10, 8, 2037));           // W margin4 Gillingham
    rec.update_with_match(&m(5, 1, 2, false, 200, "3rd Rnd", false, 17, 8, 2037));  // W margin4, cup, 6 goals Hull
    rec.update_with_match(&m(6, 2, 3, true, 100, "", true, 24, 8, 2037));           // W margin4, league, 8 goals Leeds
    rec.update_with_match(&m(0, 3, 4, false, 100, "", true, 31, 8, 2037));          // L margin3 Arsenal
    rec.update_with_match(&m(1, 1, 5, true, 100, "", true, 7, 9, 2037));            // D Everton
    rec.update_with_match(&m(2, 5, 6, false, 100, "", true, 14, 9, 2037));          // L margin3, 7 goals Chelsea
    rec.update_with_match(&m(3, 3, 7, true, 300, "2nd Rnd", false, 21, 9, 2037));   // D, 6 goals Spurs

    // Biggest Win: three wins all margin 4 -> tiebreak MORE GOALS SCORED -> 6-2 Leeds
    let bw = rec.result_row("Biggest Win", &cname, &cdisp).unwrap();
    check("Biggest Win = 6-2 v Leeds (H) Premier Division 24.8.37 (equal margin, most goals)",
        bw.score == "6-2" && bw.opponent == "Leeds" && bw.venue == "H" && bw.competition == "Premier Division" && bw.date == "24.8.37");
    // Biggest LEAGUE Win: cup 5-1 excluded; league wins 4-0 & 6-2, margin4 -> more goals -> 6-2 Leeds
    let blw = rec.result_row("Biggest League Win", &cname, &cdisp).unwrap();
    check("Biggest League Win = 6-2 v Leeds (cup 5-1 excluded, most goals)",
        blw.score == "6-2" && blw.opponent == "Leeds");
    // Biggest Defeat: 0-3 (margin3) then 2-5 (margin3, more conceded) -> replace -> 2-5 v Chelsea
    let bd = rec.result_row("Biggest Defeat", &cname, &cdisp).unwrap();
    check("Biggest Defeat = 2-5 v Chelsea (A) (equal margin, more conceded)",
        bd.score == "2-5" && bd.opponent == "Chelsea" && bd.venue == "A");
    // Highest Scoring (all comps): 8 goals (6-2 Leeds) vs 7 (2-5) -> 6-2 Leeds
    let hs = rec.result_row("Highest Scoring Game", &cname, &cdisp).unwrap();
    check("Highest Scoring = 6-2 v Leeds (8 total)", hs.score == "6-2" && hs.opponent == "Leeds");
    // Highest Scoring LEAGUE: 6-2 Leeds (8) is league -> same
    let hsl = rec.result_row("Highest Scoring League Game", &cname, &cdisp).unwrap();
    check("Highest Scoring League = 6-2 v Leeds", hsl.score == "6-2" && hsl.opponent == "Leeds");
    // Cup round descriptor formatting (Hull 5-1 is the biggest cup... but it's not a record here; check date fmt + round via a direct row):
    check("date format D.M.YY 2-digit year", bw.date == "24.8.37");

    // Sequences.
    // Wins in a row: matches 1,2,3 = 3, then loss. best = 3.
    let won = rec.sequence_row("Most Games Won in Row").unwrap();
    check("Won in Row = 3, 10.8.37 to 24.8.37", won.length == "3" && won.start_date == "10.8.37" && won.end_date == "24.8.37");
    // Without Losing (unbeaten): W W W (3) then L breaks; later D,?,D... let's see: after loss(31.8) unbeaten resets;
    // 7.9 D ->1, 14.9 L ->reset, 21.9 D ->1. best unbeaten = 3.
    let unb = rec.sequence_row("Most Games Without Losing").unwrap();
    check("Without Losing = 3", unb.length == "3");
    // Lost in row: 31.8 L=1, 7.9 D reset, 14.9 L=1 -> best 1 (length-1 => no range)
    let lost = rec.sequence_row("Most Games Lost in Row").unwrap();
    check("Lost in Row = 1 (no date range)", lost.length == "1" && lost.start_date.is_empty() && lost.end_date.is_empty());
    // Without winning: 31.8 L=1, 7.9 D=2, 14.9 L=3, 21.9 D=4 -> best 4
    let winless = rec.sequence_row("Most Games Without Winning").unwrap();
    check("Without Winning = 4, 31.8.37 to 21.9.37", winless.length == "4" && winless.start_date == "31.8.37" && winless.end_date == "21.9.37");

    std::process::exit(if pass { 0 } else { 1 });
}
