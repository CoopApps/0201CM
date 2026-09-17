//! English fixture DATES must be real football dates.
//!
//! The production fixture golden (`c11_1_production_fixture_golden.rs`)
//! asserts `(home_name, away_name, round_within_half)` — it does NOT
//! assert dates. A systematic date error therefore passed every gate.
//!
//! One did: the exe's day-of-year is ZERO-based and
//! `CmPackedDate::day_of_year` is one-based, so every English fixture
//! was generated exactly one day early. The whole season landed on
//! Fridays instead of Saturdays, and the Boxing Day round landed on
//! Christmas Day.
//!
//! These tests pin the shape of the calendar rather than individual
//! dates, so they stay meaningful without a per-fixture capture.

use cm_domain::{NewGameOptions, World};
use std::path::PathBuf;

fn weekday(y: u16, m: u8, d: u8) -> &'static str {
    // 2001-01-01 was a Monday.
    let mut days: i64 = 0;
    for yy in 2001..y {
        days += if (yy % 4 == 0 && yy % 100 != 0) || yy % 400 == 0 { 366 } else { 365 };
    }
    let leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let cum = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    days += cum[(m - 1) as usize] as i64
        + if leap && m > 2 { 1 } else { 0 }
        + (d as i64 - 1);
    ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"][(days % 7) as usize]
}

fn boot() -> Option<cm_domain::RuntimeSaveGame> {
    let db = PathBuf::from(
        std::env::var("CM_RUST_DB").unwrap_or_else(|_| "D:/cm0102-rs/rust-db".into()));
    if !db.exists() {
        eprintln!("skipping — rust-db not at {db:?}");
        return None;
    }
    let mut world = World::read_rust_db_dir(&db).expect("read rust-db");
    world.run_start_game_init(Some(&db.join("config/rng_table.bin")));
    let options = NewGameOptions {
        selected_nations: vec!["England".to_string()],
        ..Default::default()
    };
    Some(world.new_game_from_rust_db(&db, &options))
}

/// English league football is played overwhelmingly on Saturdays, with
/// a midweek minority. It is never played predominantly on Fridays.
#[test]
fn english_fixtures_are_mostly_saturdays() {
    let Some(save) = boot() else { return };
    for comp in [7u32, 8, 9, 10, 93] {
        let mut counts: std::collections::BTreeMap<&str, usize> = Default::default();
        for f in save.season.fixtures.iter().filter(|f| f.competition_id == comp) {
            *counts.entry(weekday(f.date.year, f.date.month, f.date.day)).or_insert(0) += 1;
        }
        let total: usize = counts.values().sum();
        assert!(total > 0, "comp {comp} must have fixtures");
        let sat = counts.get("Sat").copied().unwrap_or(0);
        let fri = counts.get("Fri").copied().unwrap_or(0);
        eprintln!("comp {comp}: {counts:?}");
        assert!(
            sat * 2 > total,
            "comp {comp}: Saturdays must be the majority of fixtures, got {sat} of {total} ({counts:?})"
        );
        assert!(
            fri * 4 < total,
            "comp {comp}: Friday should be rare, got {fri} of {total} — \
             an all-Friday season means the day-of-year base is off by one"
        );
    }
}

/// English football does not play on Christmas Day (and has not since
/// 1965), but the Boxing Day round is a fixture of the calendar.
#[test]
fn boxing_day_is_played_and_christmas_day_is_not() {
    let Some(save) = boot() else { return };
    let on = |m: u8, d: u8| {
        save.season
            .fixtures
            .iter()
            .filter(|f| [7u32, 8, 9, 10, 93].contains(&f.competition_id)
                     && f.date.month == m && f.date.day == d)
            .count()
    };
    let christmas = on(12, 25);
    let boxing = on(12, 26);
    eprintln!("25 Dec = {christmas} fixtures, 26 Dec = {boxing} fixtures");
    assert_eq!(
        christmas, 0,
        "no English fixture may fall on Christmas Day — this is the \
         signature of the zero-based day-of-year bug"
    );
    assert!(
        boxing > 0,
        "the Boxing Day round must be played"
    );
}
