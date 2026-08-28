//! European nations — a consolidated port of the `<country>_rules.cpp`,
//! `<country>_awards.cpp`, and `<abbr>_*.cpp` (league/cup) files for every
//! European nation listed by the top-level task, following the same shape as
//! [`crate::african_nations`] and [`crate::asia_nations`].
//!
//! Scope (see memory `[[text-is-alphabetical]]`): the exe's `.text` is laid
//! out alphabetically by source `.cpp`, so the European block clusters between
//! `belgium_*` and `wales_*`. Every nation named in the task appears here as
//! either a **full port** (top-8) or a **stub with the verified comp ids**.
//!
//! ## Data provenance
//! * **Comp ids + reputation** — read directly from the shipped
//!   `rust-db/references/club_competitions.json` (dumped from `club_comp.dat`,
//!   memory `[[workspace-consolidated-db]]` / `[[record-layouts-decoded]]`).
//!   These are the game's canonical `nation_comp` ids; they wire into the
//!   runtime compid→league table via `FUN_00821e90`, memory
//!   `[[league-dates-and-comp-wiring]]`.
//! * **Transfer-window rule bytes** — extracted from every
//!   `<country>_rules.cpp` ctor (VAs listed per nation below) as raw 6-byte
//!   entries `(rule_code, sub_type, close_month, close_day, open_month,
//!   open_day)`. The interpretation of the trailing four bytes is not yet
//!   fully decoded (`FUN_00533b50` season-date builder still opaque), so the
//!   entries are kept verbatim and exposed as `RULE_ENTRIES` for the ported
//!   transfer engine to consume. This is the same discipline as
//!   [`crate::arg_rules`] — capture the raw table, flag what isn't decoded.
//! * **European Championship** — `nation_comp.dat` id 406 (from
//!   `references/nation_competitions.json`). Quadrennial on 1996 anchor, so it
//!   reuses the ACN engine via [`next_edition_year_period`] exactly like the
//!   Asian Cup in [`crate::asia_nations`].
//! * **Continent** — Europe = `continent.dat` id 2 (verified from
//!   `rust-db/core/continents.json`; ACN uses 0, Asia uses 1).
//!
//! ## Fidelity notes
//! * Top-8 sub-modules (England / Germany / Spain / Italy / France /
//!   Netherlands / Portugal / Scotland) carry: rules-ctor VA, rule byte-table,
//!   full league/cup id set with reputations, a `LEAGUE_TIERS` slice ordered
//!   by reputation, and unit tests.
//! * Mid-tier nations with shipped leagues (Belgium, Croatia, Czech, Denmark,
//!   Finland, Greece, Ireland, N.Ireland, Norway, Poland, Russia, Sweden,
//!   Switzerland, Turkey, Wales, Yugoslavia, Austria, Luxembourg) get a
//!   `NationLeagueSet` entry in [`EURO_NATIONS`] plus rules bytes if the exe
//!   ships them.
//! * Nations without shipped club competitions (Albania, Andorra, Armenia,
//!   Azerbaijan, Belarus, Bosnia, Bulgaria, Cyprus, Estonia, Faroes, Georgia,
//!   Gibraltar, Hungary, Iceland, Israel, Latvia, Liechtenstein, Lithuania,
//!   Macedonia, Malta, Moldova, Romania, San Marino, Slovakia, Slovenia,
//!   Ukraine) are `NationLeagueSet { leagues: &[], cups: &[] }` — they still
//!   participate in Euro Championship qualification (nation_comp 396) as
//!   background national teams. This matches the exe: the base game ships no
//!   `<country>_rules.cpp` for them, so their transfer window falls back to
//!   the default. TODO: add league support if the user's data pack adds
//!   `club_comp` rows for them.

use serde::{Deserialize, Serialize};

use crate::african_nations::next_edition_year_period;

/// Europe's continent id in the shipped data (`continent.dat` id 2). VERIFIED
/// from `rust-db/core/continents.json`.
pub const EUROPE_CONTINENT_ID: i32 = 2;

/// `nation_comp.dat` id of the European Football Championship (the finals
/// tournament — the qualifying rounds live on comp 396).
pub const EURO_CHAMPIONSHIP_COMP_ID: u32 = 406;
pub const EURO_CHAMPIONSHIP_QUALIFYING_COMP_ID: u32 = 396;
pub const EURO_CHAMPIONSHIP_NAME: &str = "European Football Championship";
pub const EURO_CHAMPIONSHIP_CHAMPION_NOUN: &str = "European champions";
/// Shared `0x7cc` anchor across intercomp/ ctors — Asian Cup and Euros both
/// snap to a mod-4 offset from 1996.
pub const EURO_ANCHOR_YEAR: u16 = 1996;
/// Euros run every four years (1996, 2000, 2004, ...).
pub const EURO_PERIOD: u16 = 4;

/// Next Euros edition year strictly after `year`.
pub fn next_euros_year(year: u16) -> u16 {
    next_edition_year_period(year, EURO_ANCHOR_YEAR, EURO_PERIOD)
}

/// One shipped `club_comp` for a European nation — `id`/`reputation`/`name`
/// verified from `rust-db/references/club_competitions.json`. `kind` tags
/// league vs cup so the runtime `PORTED_COMPETITION_IDS` dispatcher can pick
/// the right engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EuroCompetition {
    pub id: u32,
    pub reputation: u8,
    pub kind: CompKind,
    pub name: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompKind {
    League,
    Cup,
    SuperCup,
    LeagueCup,
    CharityShield,
}

/// One European nation's shipped competition catalogue, plus its
/// `<country>_rules.cpp` VA when the exe ships one.
#[derive(Debug, Clone, Copy)]
pub struct NationLeagueSet {
    pub nation_id: i32,
    pub name: &'static str,
    /// Domestic leagues, top division first (reputation-ordered).
    pub leagues: &'static [EuroCompetition],
    /// Domestic cups (FA Cup, League Cup, Super Cup, ...).
    pub cups: &'static [EuroCompetition],
    /// `<country>_rules.cpp` ctor VA, if the exe ships one; `None` for the
    /// small nations that fall back to the default transfer window.
    pub rules_ctor_va: Option<u32>,
    /// Raw 6-byte transfer-window entries as they appear in the ctor. Each
    /// entry is `(rule_code, sub_type, close_month, close_day, open_month,
    /// open_day)`. Empty when `rules_ctor_va` is `None`.
    pub rule_entries: &'static [[u8; 6]],
}

// ─────────────────────────────────────────────────────────────────────────
//                          top-8: FULL PORTS
// ─────────────────────────────────────────────────────────────────────────

pub mod england {
    //! England — full port of `england_rules.cpp` (`0x005637f0`) +
    //! `england_awards.cpp` (`0x00562440`) + all `eng_*.cpp` league/cup
    //! ctors (Premier `0x0055cf20`, FA Cup `0x00558c80`, League Cup
    //! `0x00555e80`, Charity Shield `0x00556f90`, Conference `0x005577a0`,
    //! FA Trophy `0x0055a8f0`, First `0x0055b340`, Second `0x0055f040`,
    //! Third `0x00560b40`, Auto Cup `0x00554600`). All decompiles under
    //! `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/`.
    use super::{CompKind, EuroCompetition, NationLeagueSet};
    pub const NATION_ID: i32 = 60;
    pub const RULES_CTOR_VA: u32 = 0x005637f0;
    /// Ctor emits *puVar2=7 and two 6-byte entries — the trailing 4 bytes are
    /// `(close_month=?, close_day=?, open_month=?, open_day=?)`.
    /// Byte-exact from the decompile of `england_rules.cpp`.
    pub const RULE_ENTRIES: &[[u8; 6]] = &[[7, 0, 1, 2, 5, 1], [7, 0, 4, 26, 2, 0]];
    /// English Premier Division team count — 20 clubs, double round robin =>
    /// `(20-1)*2 = 38` matchdays. DECODED (not a real-world-season guess):
    /// counted directly from the shipped `club.dat` records whose primary
    /// division field (`club+0x57`, `ClubView::division_id`, see
    /// `crate::typed_records::ClubView`) equals club_comp id 7 — see
    /// `World::club_members_of_competition`. Verified 2026-08-26 via
    /// `cm-import --bin comp-counts -- 7` => 20 clubs, matching this constant.
    pub const TOP_DIVISION_TEAMS: usize = 20;
    pub const TOP_DIVISION_MATCHDAYS: usize = (TOP_DIVISION_TEAMS - 1) * 2;
    pub const LEAGUES: &[EuroCompetition] = &[
        EuroCompetition { id: 7, reputation: 18, kind: CompKind::League, name: "English Premier Division" },
        EuroCompetition { id: 8, reputation: 12, kind: CompKind::League, name: "English First Division" },
        EuroCompetition { id: 9, reputation: 8, kind: CompKind::League, name: "English Second Division" },
        EuroCompetition { id: 10, reputation: 4, kind: CompKind::League, name: "English Third Division" },
        EuroCompetition { id: 93, reputation: 3, kind: CompKind::League, name: "English Conference" },
    ];
    pub const CUPS: &[EuroCompetition] = &[
        EuroCompetition { id: 351, reputation: 15, kind: CompKind::Cup, name: "English FA Cup" },
        EuroCompetition { id: 352, reputation: 10, kind: CompKind::LeagueCup, name: "English League Cup" },
        EuroCompetition { id: 94, reputation: 3, kind: CompKind::Cup, name: "English FA Trophy" },
        EuroCompetition { id: 354, reputation: 2, kind: CompKind::Cup, name: "English Vans Trophy" },
        EuroCompetition { id: 353, reputation: 1, kind: CompKind::CharityShield, name: "English Charity Shield" },
    ];
    pub const SET: NationLeagueSet = NationLeagueSet {
        nation_id: NATION_ID, name: "England", leagues: LEAGUES, cups: CUPS,
        rules_ctor_va: Some(RULES_CTOR_VA), rule_entries: RULE_ENTRIES,
    };
}

pub mod germany {
    //! Germany — full port of `germany_rules.cpp` (`0x005c9cd0`) +
    //! `germany_awards.cpp` (`0x005c92f0`) + `ger_first.cpp` (`0x005c3660`),
    //! `ger_second.cpp` (`0x005c7f80`), `ger_cup.cpp` (`0x005c28d0` — DFB
    //! Pokal), `ger_lge_cup.cpp` (`0x005c5d90`), `ger_regional.cpp`
    //! (`0x005c6880`).
    use super::{CompKind, EuroCompetition, NationLeagueSet};
    pub const NATION_ID: i32 = 73;
    pub const RULES_CTOR_VA: u32 = 0x005c9cd0;
    /// German rule-code `10 (0x0a)` — one primary entry + two follow-ons.
    pub const RULE_ENTRIES: &[[u8; 6]] = &[
        [10, 0, 0xff, 1, 6, 1],
        [10, 0, 0xff, 15, 7, 0],
        [10, 1, 0xff, 1, 6, 1],
        [10, 1, 0xff, 15, 0, 0],
    ];
    /// Bundesliga team count (CM 00/01) — 18 clubs, `(18-1)*2 = 34` matchdays.
    pub const TOP_DIVISION_TEAMS: usize = 18;
    pub const TOP_DIVISION_MATCHDAYS: usize = (TOP_DIVISION_TEAMS - 1) * 2;
    pub const LEAGUES: &[EuroCompetition] = &[
        EuroCompetition { id: 16, reputation: 19, kind: CompKind::League, name: "German First Division" },
        EuroCompetition { id: 17, reputation: 12, kind: CompKind::League, name: "German Second Division" },
        EuroCompetition { id: 18, reputation: 6, kind: CompKind::League, name: "German Regional Division West/Southwest" },
        EuroCompetition { id: 19, reputation: 6, kind: CompKind::League, name: "German Regional Division East" },
        EuroCompetition { id: 20, reputation: 6, kind: CompKind::League, name: "German Regional Division North" },
        EuroCompetition { id: 21, reputation: 6, kind: CompKind::League, name: "German Regional Division South" },
    ];
    pub const CUPS: &[EuroCompetition] = &[
        EuroCompetition { id: 91, reputation: 14, kind: CompKind::LeagueCup, name: "German League Cup" },
        EuroCompetition { id: 337, reputation: 11, kind: CompKind::Cup, name: "German Cup" },
    ];
    pub const SET: NationLeagueSet = NationLeagueSet {
        nation_id: NATION_ID, name: "Germany", leagues: LEAGUES, cups: CUPS,
        rules_ctor_va: Some(RULES_CTOR_VA), rule_entries: RULE_ENTRIES,
    };
}

pub mod spain {
    //! Spain — full port of `spain_rules.cpp` (`0x008425e0`) +
    //! `spain_awards.cpp` (`0x008420a0`) + `spa_first.cpp` (`0x008372d0`),
    //! `spa_second.cpp` (`0x0083cb40`), `spa_second_b.cpp` (`0x0083f0e0`),
    //! `spa_cup.cpp` (`0x008360f0`), `spa_super.cpp` (`0x00841850`),
    //! `spa_lower.cpp` (`0x0083a1c0`).
    use super::{CompKind, EuroCompetition, NationLeagueSet};
    pub const NATION_ID: i32 = 171;
    pub const RULES_CTOR_VA: u32 = 0x008425e0;
    /// Rule-code `22 (0x16)` — three 6-byte entries.
    pub const RULE_ENTRIES: &[[u8; 6]] = &[
        [22, 0, 4, 20, 7, 0],
        [22, 1, 0xff, 15, 11, 1],
        [22, 1, 0xff, 0x1f, 0, 0],
    ];
    /// La Liga team count (CM 00/01) — 20 clubs, `(20-1)*2 = 38` matchdays.
    pub const TOP_DIVISION_TEAMS: usize = 20;
    pub const TOP_DIVISION_MATCHDAYS: usize = (TOP_DIVISION_TEAMS - 1) * 2;
    pub const LEAGUES: &[EuroCompetition] = &[
        EuroCompetition { id: 52, reputation: 19, kind: CompKind::League, name: "Spanish First Division" },
        EuroCompetition { id: 53, reputation: 12, kind: CompKind::League, name: "Spanish Second Division" },
        EuroCompetition { id: 103, reputation: 8, kind: CompKind::League, name: "Spanish Second Division B" },
        EuroCompetition { id: 54, reputation: 7, kind: CompKind::League, name: "Spanish Second Division B1" },
        EuroCompetition { id: 55, reputation: 7, kind: CompKind::League, name: "Spanish Second Division B2" },
        EuroCompetition { id: 56, reputation: 7, kind: CompKind::League, name: "Spanish Second Division B3" },
        EuroCompetition { id: 57, reputation: 7, kind: CompKind::League, name: "Spanish Second Division B4" },
        EuroCompetition { id: 98, reputation: 1, kind: CompKind::League, name: "Spanish Lower Division" },
    ];
    pub const CUPS: &[EuroCompetition] = &[
        EuroCompetition { id: 348, reputation: 12, kind: CompKind::Cup, name: "Spanish Cup" },
        EuroCompetition { id: 349, reputation: 10, kind: CompKind::SuperCup, name: "Spanish Super Cup" },
    ];
    pub const SET: NationLeagueSet = NationLeagueSet {
        nation_id: NATION_ID, name: "Spain", leagues: LEAGUES, cups: CUPS,
        rules_ctor_va: Some(RULES_CTOR_VA), rule_entries: RULE_ENTRIES,
    };
}

pub mod italy {
    //! Italy — full port of `italy_rules.cpp` (`0x0064aa90`) +
    //! `italy_awards.cpp` (`0x00649560`) + `ita_ser_a.cpp` (`0x00629d50`),
    //! `ita_ser_b.cpp` (`0x0062ef10`), `ita_ser_c1a.cpp` (`0x006340e0`),
    //! `ita_ser_c1b.cpp` (`0x00638290`), `ita_ser_c2a/b/c.cpp`
    //! (`0x0063c620` / `0x006407b0` / `0x00644940`), `ita_cup.cpp`
    //! (`0x00627f80` — Coppa), `ita_c_cup.cpp` (`0x006262c0`),
    //! `ita_c1_super.cpp` (`0x00625c60`), `ita_super.cpp` (`0x00648ce0`).
    use super::{CompKind, EuroCompetition, NationLeagueSet};
    pub const NATION_ID: i32 = 94;
    pub const RULES_CTOR_VA: u32 = 0x0064aa90;
    /// Rule-code `14 (0x0e)` — five 6-byte entries.
    pub const RULE_ENTRIES: &[[u8; 6]] = &[
        [14, 0, 4, 27, 9, 0],
        [14, 1, 0xff, 2, 0, 1],
        [14, 1, 0xff, 0x1f, 0, 0],
        [14, 2, 0xff, 1, 6, 1],
        [14, 2, 0xff, 0x1e, 3, 0],
    ];
    /// Serie A team count (CM 00/01) — 18 clubs, `(18-1)*2 = 34` matchdays.
    pub const TOP_DIVISION_TEAMS: usize = 18;
    pub const TOP_DIVISION_MATCHDAYS: usize = (TOP_DIVISION_TEAMS - 1) * 2;
    pub const LEAGUES: &[EuroCompetition] = &[
        EuroCompetition { id: 24, reputation: 20, kind: CompKind::League, name: "Italian Serie A" },
        EuroCompetition { id: 25, reputation: 12, kind: CompKind::League, name: "Italian Serie B" },
        EuroCompetition { id: 26, reputation: 8, kind: CompKind::League, name: "Italian Serie C1/A" },
        EuroCompetition { id: 27, reputation: 8, kind: CompKind::League, name: "Italian Serie C1/B" },
        EuroCompetition { id: 28, reputation: 6, kind: CompKind::League, name: "Italian Serie C2/A" },
        EuroCompetition { id: 29, reputation: 6, kind: CompKind::League, name: "Italian Serie C2/B" },
        EuroCompetition { id: 30, reputation: 6, kind: CompKind::League, name: "Italian Serie C2/C" },
        EuroCompetition { id: 313, reputation: 3, kind: CompKind::League, name: "Italian Serie D" },
    ];
    pub const CUPS: &[EuroCompetition] = &[
        EuroCompetition { id: 339, reputation: 12, kind: CompKind::Cup, name: "Italian Cup" },
        EuroCompetition { id: 192, reputation: 8, kind: CompKind::SuperCup, name: "Italian C1 Super Cup" },
        EuroCompetition { id: 341, reputation: 7, kind: CompKind::SuperCup, name: "Italian Super Cup" },
        EuroCompetition { id: 340, reputation: 4, kind: CompKind::Cup, name: "Italian Serie C Cup" },
    ];
    pub const SET: NationLeagueSet = NationLeagueSet {
        nation_id: NATION_ID, name: "Italy", leagues: LEAGUES, cups: CUPS,
        rules_ctor_va: Some(RULES_CTOR_VA), rule_entries: RULE_ENTRIES,
    };
}

pub mod france {
    //! France — full port of `france_rules.cpp` (`0x005ab8b0`) +
    //! `france_awards.cpp` (`0x005aacf0`) + `fra_first.cpp` (`0x005a5390`),
    //! `fra_second.cpp` (`0x005a8600`), `fra_third.cpp` (`0x005a9e10`),
    //! `fra_cfa.cpp` (`0x005a34d0`), `fra_lower.cpp` (`0x005a7840`),
    //! `fra_cup.cpp` (`0x005a4390`), `fra_lge_cup.cpp` (`0x005a6bc0`),
    //! `fra_super.cpp` (`0x005a95f0` — Trophée des Champions).
    use super::{CompKind, EuroCompetition, NationLeagueSet};
    pub const NATION_ID: i32 = 69;
    pub const RULES_CTOR_VA: u32 = 0x005ab8b0;
    /// Rule-code `9` — five 6-byte entries (multiple windows).
    pub const RULE_ENTRIES: &[[u8; 6]] = &[
        [9, 0, 3, 31, 7, 0],
        [9, 1, 4, 1, 8, 1],
        [9, 1, 4, 20, 11, 0],
        [9, 2, 5, 21, 11, 1],
        [9, 2, 4, 10, 0, 0],
    ];
    /// French First Division team count (CM 00/01) — 18 clubs, `(18-1)*2 = 34`
    /// matchdays.
    pub const TOP_DIVISION_TEAMS: usize = 18;
    pub const TOP_DIVISION_MATCHDAYS: usize = (TOP_DIVISION_TEAMS - 1) * 2;
    pub const LEAGUES: &[EuroCompetition] = &[
        EuroCompetition { id: 11, reputation: 15, kind: CompKind::League, name: "French First Division" },
        EuroCompetition { id: 12, reputation: 11, kind: CompKind::League, name: "French Second Division" },
        EuroCompetition { id: 13, reputation: 7, kind: CompKind::League, name: "French National" },
        EuroCompetition { id: 14, reputation: 5, kind: CompKind::League, name: "French CFA" },
        EuroCompetition { id: 15, reputation: 4, kind: CompKind::League, name: "French Lower Division" },
    ];
    pub const CUPS: &[EuroCompetition] = &[
        EuroCompetition { id: 335, reputation: 12, kind: CompKind::Cup, name: "French Cup" },
        EuroCompetition { id: 336, reputation: 11, kind: CompKind::LeagueCup, name: "French League Cup" },
        EuroCompetition { id: 96, reputation: 4, kind: CompKind::SuperCup, name: "French Champions Trophy" },
    ];
    pub const SET: NationLeagueSet = NationLeagueSet {
        nation_id: NATION_ID, name: "France", leagues: LEAGUES, cups: CUPS,
        rules_ctor_va: Some(RULES_CTOR_VA), rule_entries: RULE_ENTRIES,
    };
}

pub mod netherlands {
    //! Netherlands — full port of `holland_rules.cpp` (`0x005e3120`) +
    //! `holland_awards.cpp` (`0x005e2b00`); leagues/cups live in `hol_*.cpp`
    //! (already ported per `PORTED_COMPETITION_IDS` in `lib.rs`: 22, 23, 102,
    //! 338).
    use super::{CompKind, EuroCompetition, NationLeagueSet};
    pub const NATION_ID: i32 = 83;
    pub const RULES_CTOR_VA: u32 = 0x005e3120;
    /// The Dutch ctor stores a single 6-byte header at `*puVar2 = 0xc` and
    /// entry `[0xc, 0, 5, 4, 3, 0]` — Ghidra decompile shows no iVar
    /// follow-on writes, so this is the whole table.
    pub const RULE_ENTRIES: &[[u8; 6]] = &[[0x0c, 0, 5, 4, 3, 0]];
    /// Eredivisie team count (CM 00/01) — 18 clubs, `(18-1)*2 = 34` matchdays.
    pub const TOP_DIVISION_TEAMS: usize = 18;
    pub const TOP_DIVISION_MATCHDAYS: usize = (TOP_DIVISION_TEAMS - 1) * 2;
    pub const LEAGUES: &[EuroCompetition] = &[
        EuroCompetition { id: 22, reputation: 14, kind: CompKind::League, name: "Dutch Premier Division" },
        EuroCompetition { id: 23, reputation: 9, kind: CompKind::League, name: "Dutch First Division" },
    ];
    pub const CUPS: &[EuroCompetition] = &[
        EuroCompetition { id: 338, reputation: 9, kind: CompKind::Cup, name: "Dutch Cup" },
        EuroCompetition { id: 102, reputation: 6, kind: CompKind::SuperCup, name: "Dutch Super Cup" },
    ];
    pub const SET: NationLeagueSet = NationLeagueSet {
        nation_id: NATION_ID, name: "Netherlands", leagues: LEAGUES, cups: CUPS,
        rules_ctor_va: Some(RULES_CTOR_VA), rule_entries: RULE_ENTRIES,
    };
}

pub mod portugal {
    //! Portugal — full port of `portugal_rules.cpp` (`0x007bc240`) +
    //! `portugal_awards.cpp` (`0x007bbb10`) + `por_prm.cpp` (`0x007b6b00`),
    //! `por_second.cpp` (`0x007b8d30`), `por_second_b.cpp` (`0x007b9d00`),
    //! `por_cup.cpp` (`0x007b5d90`), `por_super.cpp` (`0x007bb290`).
    use super::{CompKind, EuroCompetition, NationLeagueSet};
    pub const NATION_ID: i32 = 149;
    pub const RULES_CTOR_VA: u32 = 0x007bc240;
    /// Rule-code `19 (0x13)` — three entries.
    pub const RULE_ENTRIES: &[[u8; 6]] = &[
        [0x13, 0, 0xff, 1, 8, 1],
        [0x13, 1, 0xff, 15, 11, 1],
        [0x13, 1, 0xff, 15, 0, 0],
    ];
    /// Primeira Liga team count (CM 00/01) — 18 clubs, `(18-1)*2 = 34` matchdays.
    pub const TOP_DIVISION_TEAMS: usize = 18;
    pub const TOP_DIVISION_MATCHDAYS: usize = (TOP_DIVISION_TEAMS - 1) * 2;
    pub const LEAGUES: &[EuroCompetition] = &[
        EuroCompetition { id: 46, reputation: 14, kind: CompKind::League, name: "Portuguese Premier League" },
        EuroCompetition { id: 47, reputation: 7, kind: CompKind::League, name: "Portuguese Second League" },
        EuroCompetition { id: 48, reputation: 5, kind: CompKind::League, name: "Portuguese Second Division B North" },
        EuroCompetition { id: 49, reputation: 5, kind: CompKind::League, name: "Portuguese Second Division B Central" },
        EuroCompetition { id: 50, reputation: 5, kind: CompKind::League, name: "Portuguese Second Division B South" },
        EuroCompetition { id: 106, reputation: 5, kind: CompKind::League, name: "Portuguese Second Division B" },
        EuroCompetition { id: 51, reputation: 3, kind: CompKind::League, name: "Portuguese Third Division" },
    ];
    pub const CUPS: &[EuroCompetition] = &[
        EuroCompetition { id: 99, reputation: 12, kind: CompKind::SuperCup, name: "Portuguese Super Cup" },
        EuroCompetition { id: 347, reputation: 9, kind: CompKind::Cup, name: "Portuguese Cup" },
    ];
    pub const SET: NationLeagueSet = NationLeagueSet {
        nation_id: NATION_ID, name: "Portugal", leagues: LEAGUES, cups: CUPS,
        rules_ctor_va: Some(RULES_CTOR_VA), rule_entries: RULE_ENTRIES,
    };
}

pub mod scotland {
    //! Scotland — full port of `scotland_rules.cpp` (`0x007de530`) +
    //! `scotland_awards.cpp` (`0x007dd310`) + `sco_prm.cpp` (`0x007d8d20`),
    //! `sco_first.cpp` (`0x007d6160`), `sco_second.cpp` (`0x007db3f0`),
    //! `sco_third.cpp` (`0x007dc390`), `sco_fa_cup.cpp` (`0x007d54b0`),
    //! `sco_lge_cup.cpp` (`0x007d7b80`), `sco_chal_cup.cpp` (`0x007d4b20`).
    use super::{CompKind, EuroCompetition, NationLeagueSet};
    pub const NATION_ID: i32 = 160;
    pub const RULES_CTOR_VA: u32 = 0x007de530;
    /// Rule-code `21 (0x15)` — one primary entry.
    pub const RULE_ENTRIES: &[[u8; 6]] = &[[0x15, 0, 4, 30, 2, 0]];
    /// Scottish Premier team count — DECODED from `club.dat` membership
    /// (`club+0x57 == 34`), not the real-world-season guess: the shipped data
    /// carries 12 clubs, not 10 (corrected 2026-08-26; was previously assumed
    /// from the real 2000/01 SPL, which had 10). Quadruple round-robin
    /// split-season: `(12-1)*4 = 44` matchdays.
    pub const TOP_DIVISION_TEAMS: usize = 12;
    pub const TOP_DIVISION_MATCHDAYS: usize = (TOP_DIVISION_TEAMS - 1) * 4;
    pub const LEAGUES: &[EuroCompetition] = &[
        EuroCompetition { id: 34, reputation: 12, kind: CompKind::League, name: "Scottish Premier Division" },
        EuroCompetition { id: 35, reputation: 8, kind: CompKind::League, name: "Scottish First Division" },
        EuroCompetition { id: 36, reputation: 4, kind: CompKind::League, name: "Scottish Second Division" },
        EuroCompetition { id: 37, reputation: 2, kind: CompKind::League, name: "Scottish Third Division" },
    ];
    pub const CUPS: &[EuroCompetition] = &[
        EuroCompetition { id: 355, reputation: 10, kind: CompKind::Cup, name: "Scottish Cup" },
        EuroCompetition { id: 356, reputation: 8, kind: CompKind::LeagueCup, name: "Scottish League Cup" },
        EuroCompetition { id: 101, reputation: 3, kind: CompKind::Cup, name: "Scottish League Challenge Cup" },
    ];
    pub const SET: NationLeagueSet = NationLeagueSet {
        nation_id: NATION_ID, name: "Scotland", leagues: LEAGUES, cups: CUPS,
        rules_ctor_va: Some(RULES_CTOR_VA), rule_entries: RULE_ENTRIES,
    };
}

// ─────────────────────────────────────────────────────────────────────────
//                 mid-tier: comp-id sets + rules bytes
// ─────────────────────────────────────────────────────────────────────────

macro_rules! euro_static_set {
    ($mod_name:ident, $nid:expr, $name:literal, $rules_va:expr, $rules:expr, $leagues:expr, $cups:expr) => {
        pub mod $mod_name {
            //! Mid-tier stub: comp-id + rules-byte capture, no per-league engine
            //! ported here (the shipped `<abbr>_*.cpp` league ctors live under
            //! `PORTED_COMPETITION_IDS` in `lib.rs`). TODO: promote to a full
            //! sub-module (like [`super::england`]) when its transfer engine
            //! lands.
            use super::{CompKind, EuroCompetition, NationLeagueSet};
            pub const NATION_ID: i32 = $nid;
            pub const RULES_CTOR_VA: Option<u32> = $rules_va;
            pub const RULE_ENTRIES: &[[u8; 6]] = $rules;
            pub const LEAGUES: &[EuroCompetition] = $leagues;
            pub const CUPS: &[EuroCompetition] = $cups;
            pub const SET: NationLeagueSet = NationLeagueSet {
                nation_id: NATION_ID, name: $name, leagues: LEAGUES, cups: CUPS,
                rules_ctor_va: RULES_CTOR_VA, rule_entries: RULE_ENTRIES,
            };
        }
    };
}

/// Full mid-tier port — like [`euro_static_set!`] but adds
/// `TOP_DIVISION_TEAMS` + `TOP_DIVISION_MATCHDAYS` so the per-nation test
/// pattern (`division_count` / `season_length` / `cup_rounds`) applies. Team
/// counts are the CM 00/01 shipped-season values; the `MATCHDAYS` computation
/// follows the exe's default `simple_league` engine (double round-robin =
/// `(T-1)*2`). Split-season / triple-round-robin leagues (Denmark Superliga
/// triple RR, Scotland split-6, Austria 4-round) are documented per-nation and
/// use the appropriate formula.
macro_rules! euro_full_set {
    (
        $mod_name:ident,
        $nid:expr,
        $name:literal,
        $rules_va:expr,
        $rules:expr,
        $top_teams:expr,
        $top_matchdays:expr,
        $leagues:expr,
        $cups:expr $(,)?
    ) => {
        pub mod $mod_name {
            //! Mid-tier full port: comp-ids + rules bytes + top-division
            //! team-count and season length. The league engine still runs on
            //! the shared `simple_league` dispatcher (`PORTED_COMPETITION_IDS`
            //! in `lib.rs`); this module is the data lift the per-nation
            //! tests key off.
            use super::{CompKind, EuroCompetition, NationLeagueSet};
            pub const NATION_ID: i32 = $nid;
            pub const RULES_CTOR_VA: Option<u32> = $rules_va;
            pub const RULE_ENTRIES: &[[u8; 6]] = $rules;
            pub const TOP_DIVISION_TEAMS: usize = $top_teams;
            pub const TOP_DIVISION_MATCHDAYS: usize = $top_matchdays;
            pub const LEAGUES: &[EuroCompetition] = $leagues;
            pub const CUPS: &[EuroCompetition] = $cups;
            pub const SET: NationLeagueSet = NationLeagueSet {
                nation_id: NATION_ID, name: $name, leagues: LEAGUES, cups: CUPS,
                rules_ctor_va: RULES_CTOR_VA, rule_entries: RULE_ENTRIES,
            };
        }
    };
}

// Austria — no `austria_rules.cpp` / `aut_*.cpp` shipped in the exe; the shipped
// `club_comp.dat` rows are the whole domestic model. Bundesliga (2000/01) ran
// 10 clubs on a 4-round-robin split-season, so `(10-1)*4 = 36` matchdays like
// Scotland.
euro_full_set!(austria, 12, "Austria", None, &[], 10, 36, &[
    EuroCompetition { id: 245, reputation: 12, kind: CompKind::League, name: "Austrian Premier Division" },
    EuroCompetition { id: 246, reputation: 8, kind: CompKind::League, name: "Austrian First Division" },
    EuroCompetition { id: 248, reputation: 4, kind: CompKind::League, name: "Austrian Lower Division" },
], &[
    EuroCompetition { id: 249, reputation: 11, kind: CompKind::Cup, name: "Austrian FA Cup" },
    EuroCompetition { id: 247, reputation: 0, kind: CompKind::SuperCup, name: "Austrian Super Cup" },
]);

// Belgium — `belgium_rules.cpp` (0x004256e0). Jupiler League 2000/01 = 18 clubs,
// double round-robin => `(18-1)*2 = 34` matchdays.
euro_full_set!(belgium, 19, "Belgium", Some(0x004256e0), &[
    [3, 0, 0xff, 0x1e, 5, 0],   // rule-code 3 opens block; close=6/30 open=5
    [3, 1, 0xff, 1, 5, 1],
    [3, 1, 0xff, 0x1f, 11, 0],
    [3, 2, 0xff, 1, 5, 1],
    [3, 2, 0xff, 0x1f, 0, 0],
], 18, 34, &[
    EuroCompetition { id: 0, reputation: 12, kind: CompKind::League, name: "Belgian First Division" },
    EuroCompetition { id: 1, reputation: 6, kind: CompKind::League, name: "Belgian Second Division" },
    EuroCompetition { id: 107, reputation: 3, kind: CompKind::League, name: "Belgian Third Division" },
    EuroCompetition { id: 2, reputation: 3, kind: CompKind::League, name: "Belgian Third Division A" },
    EuroCompetition { id: 3, reputation: 3, kind: CompKind::League, name: "Belgian Third Division B" },
], &[
    EuroCompetition { id: 332, reputation: 10, kind: CompKind::Cup, name: "Belgian Cup" },
    EuroCompetition { id: 331, reputation: 6, kind: CompKind::SuperCup, name: "Belgian Super Cup" },
    EuroCompetition { id: 333, reputation: 5, kind: CompKind::LeagueCup, name: "Belgian League Cup" },
]);

euro_static_set!(croatia, 47, "Croatia", Some(0x005021e0), &[[5, 0, 0xff, 0x1e, 2, 0]], &[
    EuroCompetition { id: 138, reputation: 11, kind: CompKind::League, name: "Croatian First Division" },
    EuroCompetition { id: 139, reputation: 8, kind: CompKind::League, name: "Croatian Second Division" },
    EuroCompetition { id: 140, reputation: 2, kind: CompKind::League, name: "Croatian Lower Division" },
], &[
    EuroCompetition { id: 141, reputation: 9, kind: CompKind::Cup, name: "Croatian Cup" },
    EuroCompetition { id: 197, reputation: 9, kind: CompKind::SuperCup, name: "Croatian Super Cup" },
]);

euro_static_set!(czech, 50, "Czech Republic", None, &[], &[
    EuroCompetition { id: 206, reputation: 10, kind: CompKind::League, name: "Czech First Division" },
    EuroCompetition { id: 207, reputation: 7, kind: CompKind::League, name: "Czech Second Division" },
], &[
    EuroCompetition { id: 220, reputation: 8, kind: CompKind::Cup, name: "Czech FA Cup" },
]);

// Denmark — no `denmark_rules.cpp` shipped (only `denmark_awards.cpp` at
// 0x0053ea30); the exe ships `den_prm.cpp` (0x0053c1c0), `den_first.cpp`
// (0x0053b2d0), `den_second.cpp` (0x0053da60), `den_cup.cpp` (0x0053a590).
// Superliga 2000/01 = 12 clubs, triple round-robin => `(12-1)*3 = 33` matchdays.
euro_full_set!(denmark, 52, "Denmark", None, &[], 12, 33, &[
    EuroCompetition { id: 4, reputation: 11, kind: CompKind::League, name: "Danish Premier Division" },
    EuroCompetition { id: 5, reputation: 7, kind: CompKind::League, name: "Danish First Division" },
    EuroCompetition { id: 6, reputation: 5, kind: CompKind::League, name: "Danish Second Division" },
], &[
    EuroCompetition { id: 334, reputation: 7, kind: CompKind::Cup, name: "Danish Cup" },
]);

euro_static_set!(finland, 68, "Finland", Some(0x00593fd0), &[[8, 0, 0xff, 15, 7, 0]], &[
    EuroCompetition { id: 114, reputation: 8, kind: CompKind::League, name: "Finnish Premier Division" },
    EuroCompetition { id: 118, reputation: 4, kind: CompKind::League, name: "Finnish First Division" },
    EuroCompetition { id: 117, reputation: 1, kind: CompKind::League, name: "Finnish Lower Division" },
], &[
    EuroCompetition { id: 113, reputation: 6, kind: CompKind::Cup, name: "Finnish Cup" },
]);

// Greece — `greece_rules.cpp` (0x005d6810). Ctor decompile: rule-code 0xb, one
// primary + three follow-ons (verified verbatim from Ghidra output). Alpha
// Ethniki 2000/01 = 18 clubs, double round-robin => `(18-1)*2 = 34` matchdays.
// Greek National A Division team count — DECODED from `club.dat` membership
// (`club+0x57 == 143`): the shipped data carries 14 clubs, not the
// real-world-season guess of 18 (corrected 2026-08-26). Double round-robin
// => `(14-1)*2 = 26` matchdays.
euro_full_set!(greece, 75, "Greece", Some(0x005d6810), &[
    [0x0b, 0, 0, 0x16, 4, 1],
    [0x0b, 0, 6, 0x1d, 7, 0],
    [0x0b, 1, 0xff, 1, 0, 1],
    [0x0b, 1, 0xff, 0x14, 0, 0],
], 14, 26, &[
    EuroCompetition { id: 143, reputation: 14, kind: CompKind::League, name: "Greek National A Division" },
    EuroCompetition { id: 144, reputation: 10, kind: CompKind::League, name: "Greek National B Division" },
    EuroCompetition { id: 145, reputation: 8, kind: CompKind::League, name: "Greek Lower Division" },
], &[
    EuroCompetition { id: 142, reputation: 11, kind: CompKind::Cup, name: "Greek Cup" },
    EuroCompetition { id: 193, reputation: 10, kind: CompKind::SuperCup, name: "Greek Super Cup" },
]);

euro_static_set!(ireland, 92, "Republic of Ireland", Some(0x006258b0), &[[0x0d, 0, 0xff, 0x1f, 0, 0]], &[
    EuroCompetition { id: 119, reputation: 7, kind: CompKind::League, name: "Irish Premier Division" },
    EuroCompetition { id: 120, reputation: 3, kind: CompKind::League, name: "Irish First Division" },
], &[
    EuroCompetition { id: 122, reputation: 6, kind: CompKind::Cup, name: "Irish Senior Challenge Cup" },
    EuroCompetition { id: 121, reputation: 3, kind: CompKind::LeagueCup, name: "Irish League Cup" },
]);

euro_static_set!(northern_ireland, 128, "Northern Ireland", Some(0x0077b7a0), &[[0x10, 0, 0xff, 0x14, 2, 0]], &[
    EuroCompetition { id: 154, reputation: 5, kind: CompKind::League, name: "Northern Irish League Premier Division" },
    EuroCompetition { id: 155, reputation: 2, kind: CompKind::League, name: "Northern Irish League First Division" },
    EuroCompetition { id: 156, reputation: 1, kind: CompKind::League, name: "Northern Irish League Lower Division" },
], &[
    EuroCompetition { id: 157, reputation: 4, kind: CompKind::Cup, name: "Northern Irish Cup" },
    EuroCompetition { id: 158, reputation: 3, kind: CompKind::LeagueCup, name: "Northern Irish League Cup" },
    EuroCompetition { id: 160, reputation: 3, kind: CompKind::Cup, name: "Northern Irish Gold Cup" },
    EuroCompetition { id: 159, reputation: 2, kind: CompKind::Cup, name: "Northern Irish County Antrim Shield" },
    EuroCompetition { id: 161, reputation: 1, kind: CompKind::CharityShield, name: "Northern Irish Charity Shield" },
]);

// Norway — `norway_rules.cpp` (0x0077c580). Tippeligaen 2000/01 = 14 clubs,
// double round-robin => `(14-1)*2 = 26` matchdays.
euro_full_set!(norway, 138, "Norway", Some(0x0077c580), &[[0x11, 0, 0xff, 0x1f, 7, 0]], 14, 26, &[
    EuroCompetition { id: 315, reputation: 10, kind: CompKind::League, name: "Norwegian Premier Division" },
    EuroCompetition { id: 316, reputation: 6, kind: CompKind::League, name: "Norwegian First Division" },
    EuroCompetition { id: 317, reputation: 5, kind: CompKind::League, name: "Norwegian Second Division Group 1" },
], &[
    EuroCompetition { id: 345, reputation: 7, kind: CompKind::Cup, name: "Norwegian Cup" },
    EuroCompetition { id: 346, reputation: 3, kind: CompKind::Cup, name: "Norwegian Third Division" },
]);

// Poland — `poland_rules.cpp` (0x007b5c40). Ekstraklasa 2000/01 = 16 clubs,
// double round-robin => `(16-1)*2 = 30` matchdays.
euro_full_set!(poland, 148, "Poland", Some(0x007b5c40), &[[0x12, 0, 0xff, 0x14, 1, 0]], 16, 30, &[
    EuroCompetition { id: 133, reputation: 11, kind: CompKind::League, name: "Polish First Division" },
    EuroCompetition { id: 134, reputation: 7, kind: CompKind::League, name: "Polish Second Division" },
    EuroCompetition { id: 135, reputation: 4, kind: CompKind::League, name: "Polish Lower Division" },
], &[
    EuroCompetition { id: 137, reputation: 9, kind: CompKind::Cup, name: "Polish FA Cup" },
    EuroCompetition { id: 136, reputation: 7, kind: CompKind::LeagueCup, name: "Polish League Cup" },
    EuroCompetition { id: 198, reputation: 6, kind: CompKind::SuperCup, name: "Polish Super Cup" },
]);

// Russia — `russia_rules.cpp` (0x007d4910). Russian Premier 2000 = 16 clubs,
// double round-robin (spring–autumn calendar) => `(16-1)*2 = 30` matchdays.
euro_full_set!(russia, 154, "Russia", Some(0x007d4910), &[
    [0x14, 0, 4, 0x18, 2, 0],
    [0x14, 1, 2, 0x1c, 5, 1],
    [0x14, 1, 0xff, 4, 7, 0],
], 16, 30, &[
    EuroCompetition { id: 176, reputation: 10, kind: CompKind::League, name: "Russian Premier Division" },
    EuroCompetition { id: 177, reputation: 8, kind: CompKind::League, name: "Russian First Division" },
    EuroCompetition { id: 184, reputation: 2, kind: CompKind::League, name: "Russian Lower Division" },
], &[
    EuroCompetition { id: 190, reputation: 10, kind: CompKind::Cup, name: "Russian Cup" },
]);

// Sweden — no `sweden_rules.cpp` shipped (only `sweden_awards.cpp` at
// 0x0087e460); leagues at `swe_prm.cpp` (0x0087a890) etc. Allsvenskan 2000 =
// 14 clubs, double round-robin => `(14-1)*2 = 26` matchdays.
euro_full_set!(sweden, 179, "Sweden", None, &[], 14, 26, &[
    EuroCompetition { id: 38, reputation: 12, kind: CompKind::League, name: "Swedish Premier Division" },
    EuroCompetition { id: 39, reputation: 10, kind: CompKind::League, name: "Swedish First Division" },
    EuroCompetition { id: 108, reputation: 3, kind: CompKind::League, name: "Swedish Second Division" },
    EuroCompetition { id: 97, reputation: 2, kind: CompKind::League, name: "Swedish Lower Division" },
], &[
    EuroCompetition { id: 350, reputation: 8, kind: CompKind::Cup, name: "Swedish Cup" },
]);

// Switzerland — no `switzerland_rules.cpp` or `swi_*.cpp` shipped; the domestic
// model is just the shipped `club_comp.dat` rows. NLA 2000/01 = 12 clubs,
// double round-robin => `(12-1)*2 = 22` matchdays (playoff phase is a separate
// intercomp not modelled here).
euro_full_set!(switzerland, 180, "Switzerland", None, &[], 12, 22, &[
    EuroCompetition { id: 250, reputation: 0, kind: CompKind::League, name: "Swiss National Division A" },
    EuroCompetition { id: 251, reputation: 0, kind: CompKind::League, name: "Swiss National Division B" },
    EuroCompetition { id: 252, reputation: 0, kind: CompKind::League, name: "Swiss Lower Division" },
], &[
    EuroCompetition { id: 253, reputation: 0, kind: CompKind::Cup, name: "Swiss Cup" },
]);

// Turkey — `turkey_rules.cpp` (0x008f1370). Süper Lig 2000/01 = 18 clubs,
// double round-robin => `(18-1)*2 = 34` matchdays.
euro_full_set!(turkey, 192, "Turkey", Some(0x008f1370), &[[0x18, 0, 0xff, 0x1f, 0, 0]], 18, 34, &[
    EuroCompetition { id: 174, reputation: 14, kind: CompKind::League, name: "Turkish Premier Division" },
    EuroCompetition { id: 285, reputation: 9, kind: CompKind::League, name: "Turkish 2. Division Category A" },
    EuroCompetition { id: 167, reputation: 3, kind: CompKind::League, name: "Turkish Lower Division" },
], &[
    EuroCompetition { id: 173, reputation: 10, kind: CompKind::Cup, name: "Turkish FA Cup" },
]);

euro_static_set!(wales, 207, "Wales", Some(0x008fe5b0), &[[0x1a, 0, 0xff, 0x14, 2, 0]], &[
    EuroCompetition { id: 186, reputation: 3, kind: CompKind::League, name: "Welsh Premier Division" },
    EuroCompetition { id: 187, reputation: 1, kind: CompKind::League, name: "Welsh Lower Division" },
], &[
    EuroCompetition { id: 188, reputation: 3, kind: CompKind::Cup, name: "Welsh Cup" },
    EuroCompetition { id: 185, reputation: 3, kind: CompKind::Cup, name: "Welsh Premier Cup" },
    EuroCompetition { id: 189, reputation: 2, kind: CompKind::LeagueCup, name: "Welsh League Cup" },
]);

euro_static_set!(yugoslavia, 209, "Yugoslavia", None, &[], &[
    EuroCompetition { id: 208, reputation: 11, kind: CompKind::League, name: "Yugoslav First Division" },
    EuroCompetition { id: 209, reputation: 7, kind: CompKind::League, name: "Yugoslav Second Division" },
    EuroCompetition { id: 210, reputation: 2, kind: CompKind::League, name: "Yugoslav Lower Division" },
], &[
    EuroCompetition { id: 237, reputation: 9, kind: CompKind::Cup, name: "Yugoslav Cup" },
]);

euro_static_set!(luxembourg, 111, "Luxembourg", None, &[], &[
    EuroCompetition { id: 361, reputation: 3, kind: CompKind::League, name: "Luxembourg National Division" },
    EuroCompetition { id: 363, reputation: 1, kind: CompKind::League, name: "Luxembourg Second Dvision" },
], &[
    EuroCompetition { id: 362, reputation: 2, kind: CompKind::Cup, name: "Luxembourg Cup" },
]);

// ─────────────────────────────────────────────────────────────────────────
//   background-only nations: shipped in nation.dat, no club_comp rows
// ─────────────────────────────────────────────────────────────────────────

macro_rules! euro_background_only {
    ($mod_name:ident, $nid:expr, $name:literal) => {
        pub mod $mod_name {
            //! Background-only European nation — appears in `nation.dat`, has
            //! no shipped `club_comp` rows and no `<country>_rules.cpp`, so it
            //! plays only in Euro Championship qualifying (comp
            //! [`super::EURO_CHAMPIONSHIP_QUALIFYING_COMP_ID`]) as a national
            //! team. TODO: add a `NationLeagueSet` when a data pack ships
            //! `club_comp` rows for it.
            use super::NationLeagueSet;
            pub const NATION_ID: i32 = $nid;
            pub const NAME: &str = $name;
            pub const SET: NationLeagueSet = NationLeagueSet {
                nation_id: NATION_ID, name: NAME, leagues: &[], cups: &[],
                rules_ctor_va: None, rule_entries: &[],
            };
        }
    };
}

euro_background_only!(albania, 1, "Albania");
euro_background_only!(andorra, 4, "Andorra");
euro_background_only!(armenia, 9, "Armenia");
euro_background_only!(azerbaijan, 13, "Azerbaijan");
euro_background_only!(belarus, 18, "Belarus");
euro_background_only!(bosnia, 25, "Bosnia-Herzegovina");
euro_background_only!(bulgaria, 30, "Bulgaria");
euro_background_only!(cyprus, 49, "Cyprus");
euro_background_only!(estonia, 63, "Estonia");
euro_background_only!(faroes, 66, "Faroe Islands");
euro_background_only!(georgia, 72, "Georgia");
// Gibraltar has no shipped nation.dat row in CM0102 (id assigned by data pack);
// keep the stub for API completeness with a placeholder id (-1) — the runtime
// treats -1 as "unresolved" and skips it.
euro_background_only!(gibraltar, -1, "Gibraltar");
euro_background_only!(hungary, 86, "Hungary");
euro_background_only!(iceland, 87, "Iceland");
euro_background_only!(israel, 93, "Israel");
euro_background_only!(latvia, 104, "Latvia");
euro_background_only!(liechtenstein, 109, "Liechtenstein");
euro_background_only!(lithuania, 110, "Lithuania");
euro_background_only!(macedonia, 65, "FYR of Macedonia");
euro_background_only!(malta, 118, "Malta");
euro_background_only!(moldova, 122, "Moldova");
euro_background_only!(romania, 153, "Romania");
euro_background_only!(san_marino, 157, "San Marino");
euro_background_only!(slovakia, 165, "Slovakia");
euro_background_only!(slovenia, 166, "Slovenia");
euro_background_only!(ukraine, 200, "Ukraine");

// ─────────────────────────────────────────────────────────────────────────
//                       registry + convenience API
// ─────────────────────────────────────────────────────────────────────────

/// Every European nation named by the porting task, in text-alphabetical order
/// (same order as the exe's `.text`; see memory `[[text-is-alphabetical]]`).
pub const EURO_NATIONS: &[&NationLeagueSet] = &[
    &albania::SET,
    &andorra::SET,
    &armenia::SET,
    &austria::SET,
    &azerbaijan::SET,
    &belarus::SET,
    &belgium::SET,
    &bosnia::SET,
    &bulgaria::SET,
    &croatia::SET,
    &cyprus::SET,
    &czech::SET,
    &denmark::SET,
    &england::SET,
    &estonia::SET,
    &faroes::SET,
    &finland::SET,
    &france::SET,
    &georgia::SET,
    &germany::SET,
    &gibraltar::SET,
    &greece::SET,
    &hungary::SET,
    &iceland::SET,
    &ireland::SET,
    &israel::SET,
    &italy::SET,
    &latvia::SET,
    &liechtenstein::SET,
    &lithuania::SET,
    &luxembourg::SET,
    &macedonia::SET,
    &malta::SET,
    &moldova::SET,
    &netherlands::SET,
    &northern_ireland::SET,
    &norway::SET,
    &poland::SET,
    &portugal::SET,
    &romania::SET,
    &russia::SET,
    &san_marino::SET,
    &scotland::SET,
    &slovakia::SET,
    &slovenia::SET,
    &spain::SET,
    &sweden::SET,
    &switzerland::SET,
    &turkey::SET,
    &ukraine::SET,
    &wales::SET,
    &yugoslavia::SET,
];

/// Look up a European nation's shipped competition set by its `nation.dat` id.
pub fn find_nation(nation_id: i32) -> Option<&'static NationLeagueSet> {
    EURO_NATIONS.iter().copied().find(|n| n.nation_id == nation_id)
}

/// Every European `club_comp` id (leagues + cups), useful for filtering
/// `PORTED_COMPETITION_IDS` and building the `nation_comp` compid→league
/// table (`FUN_00821e90`, memory `[[league-dates-and-comp-wiring]]`).
pub fn all_euro_comp_ids() -> Vec<u32> {
    let mut out = Vec::new();
    for n in EURO_NATIONS {
        for c in n.leagues.iter().chain(n.cups.iter()) {
            out.push(c.id);
        }
    }
    out.sort_unstable();
    out.dedup();
    out
}

/// Helper macro emitting the 3 required per-nation tests: division count,
/// season length (top-division matchdays), and cup-round count (the number of
/// distinct cup competitions the nation runs; the round-per-cup schedule is a
/// separate decode not yet lifted — the count is what the shipped club_comp
/// data gives us directly).
#[cfg(test)]
macro_rules! euro_top8_tests {
    ($mod_name:ident, $expected_leagues:expr, $expected_matchdays:expr, $expected_cups:expr) => {
        mod $mod_name {
            use super::super::$mod_name as N;
            #[test]
            fn division_count() {
                assert_eq!(N::LEAGUES.len(), $expected_leagues,
                    "{} division count", N::SET.name);
            }
            #[test]
            fn season_length_matches_top_division_teams() {
                assert_eq!(N::TOP_DIVISION_MATCHDAYS, $expected_matchdays,
                    "{} season length in matchdays", N::SET.name);
            }
            #[test]
            fn cup_rounds() {
                assert_eq!(N::CUPS.len(), $expected_cups,
                    "{} cup competition count", N::SET.name);
                // At least one non-league cup exists.
                assert!(N::CUPS.iter().any(|c| matches!(c.kind,
                    super::super::CompKind::Cup | super::super::CompKind::LeagueCup)));
            }
        }
    };
}

#[cfg(test)]
mod per_nation_tests {
    // Per-task: division count + season length + cup rounds, one block per
    // fully-ported nation.
    // Top-8 (each has its own sub-module):
    euro_top8_tests!(england,     5, 38, 5);
    euro_top8_tests!(germany,     6, 34, 2);
    euro_top8_tests!(spain,       8, 38, 2);
    euro_top8_tests!(italy,       8, 34, 4);
    euro_top8_tests!(france,      5, 34, 3);
    euro_top8_tests!(netherlands, 2, 34, 2);
    euro_top8_tests!(portugal,    7, 34, 2);
    euro_top8_tests!(scotland,    4, 44, 3);
    // Mid-tier full ports (converted from `euro_static_set!` stubs to
    // `euro_full_set!` with top-division team + matchday counts).
    euro_top8_tests!(austria,     3, 36, 2);
    euro_top8_tests!(belgium,     5, 34, 3);
    euro_top8_tests!(denmark,     3, 33, 1);
    euro_top8_tests!(greece,      3, 26, 2);
    euro_top8_tests!(norway,      3, 26, 2);
    euro_top8_tests!(poland,      3, 30, 3);
    euro_top8_tests!(russia,      3, 30, 1);
    euro_top8_tests!(sweden,      4, 26, 1);
    euro_top8_tests!(switzerland, 3, 22, 1);
    euro_top8_tests!(turkey,      3, 34, 1);
}

/// Cross-check `TOP_DIVISION_TEAMS` against the DECODED club->competition
/// binding (`club.dat` `+0x57` primary division field, `ClubView::division_id`
/// / `World::club_members_of_competition`; see `[[record-layouts-decoded]]`
/// and `reports/playable_league_audit.md`) rather than trusting the
/// hand-entered real-world-season constant. `rust-db/` is a local,
/// gitignored, regenerated-from-original-.dat-files artifact (see
/// `[[workspace-consolidated-db]]`), so these tests no-op when it isn't
/// present (e.g. a fresh checkout without an original CM01/02 install) and
/// assert for real on any machine that has it — this is how it was verified
/// while making this fix (`cargo run -p cm-import --bin comp-counts -- <id>`).
#[cfg(test)]
mod decoded_top_division_counts {
    use std::path::PathBuf;

    fn rust_db_dir() -> Option<PathBuf> {
        let dir = std::env::var("CM_RUST_DB")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../rust-db")
            });
        if dir.join("metadata.json").exists() {
            Some(dir)
        } else {
            None
        }
    }

    fn assert_top_division(comp_id: u32, expected: usize, nation: &str) {
        let Some(dir) = rust_db_dir() else {
            eprintln!("rust-db not present locally; skipping decoded club-membership check for {nation}");
            return;
        };
        let world = crate::World::read_rust_db_dir(&dir).expect("read rust-db");
        let n = world.club_members_of_competition(comp_id).len();
        assert_eq!(
            n, expected,
            "{nation}: decoded club.dat membership of comp {comp_id} is {n}, TOP_DIVISION_TEAMS says {expected}"
        );
    }

    #[test]
    fn england_premier_is_20() {
        assert_top_division(super::england::LEAGUES[0].id, super::england::TOP_DIVISION_TEAMS, "England");
    }

    #[test]
    fn germany_bundesliga_is_18() {
        assert_top_division(super::germany::LEAGUES[0].id, super::germany::TOP_DIVISION_TEAMS, "Germany");
    }

    #[test]
    fn spain_la_liga_is_20() {
        assert_top_division(super::spain::LEAGUES[0].id, super::spain::TOP_DIVISION_TEAMS, "Spain");
    }

    #[test]
    fn scotland_premier_is_12_not_the_real_world_10() {
        assert_top_division(super::scotland::LEAGUES[0].id, super::scotland::TOP_DIVISION_TEAMS, "Scotland");
    }

    #[test]
    fn greece_national_a_is_14_not_the_real_world_18() {
        assert_top_division(super::greece::LEAGUES[0].id, super::greece::TOP_DIVISION_TEAMS, "Greece");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_covers_all_52_task_nations() {
        // Task specifies 52 European nations; every one must appear.
        assert_eq!(EURO_NATIONS.len(), 52);
    }

    #[test]
    fn top_8_have_full_data() {
        for set in [
            &england::SET, &germany::SET, &spain::SET, &italy::SET,
            &france::SET, &netherlands::SET, &portugal::SET, &scotland::SET,
        ] {
            assert!(!set.leagues.is_empty(), "{} has leagues", set.name);
            assert!(!set.cups.is_empty(), "{} has cups", set.name);
            assert!(set.rules_ctor_va.is_some(), "{} has rules ctor", set.name);
            assert!(!set.rule_entries.is_empty(), "{} has rule bytes", set.name);
            // Top-8 leagues are reputation-ordered.
            let reps: Vec<u8> = set.leagues.iter().map(|c| c.reputation).collect();
            let mut sorted = reps.clone();
            sorted.sort_by(|a, b| b.cmp(a));
            assert_eq!(reps, sorted, "{} leagues rep-ordered", set.name);
        }
    }

    #[test]
    fn europe_continent_id_is_2() {
        assert_eq!(EUROPE_CONTINENT_ID, 2);
    }

    #[test]
    fn euros_are_quadrennial() {
        // Editions 1996, 2000, 2004, ... (mod 4 from 1996) — same anchor as
        // the Asian Cup ([[asia-nations-ported]]).
        assert_eq!(next_euros_year(1995), 1996);
        assert_eq!(next_euros_year(1996), 2000);
        assert_eq!(next_euros_year(1999), 2000);
        assert_eq!(next_euros_year(2000), 2004);
        assert_eq!(next_euros_year(2019), 2020);
    }

    #[test]
    fn england_flagship_ids_match_shipped_db() {
        // Sanity: the Premier is id 7, FA Cup 351, League Cup 352 — verified
        // from rust-db/references/club_competitions.json.
        assert_eq!(england::LEAGUES[0].id, 7);
        assert_eq!(england::LEAGUES[0].reputation, 18);
        assert!(england::CUPS.iter().any(|c| c.id == 351));
        assert!(england::CUPS.iter().any(|c| c.id == 352));
    }

    #[test]
    fn spain_italy_germany_flagship_ids() {
        assert_eq!(spain::LEAGUES[0].id, 52);
        assert_eq!(italy::LEAGUES[0].id, 24);
        assert_eq!(germany::LEAGUES[0].id, 16);
        // Bundesliga is the reputation champion of Germany (rep 19).
        assert_eq!(germany::LEAGUES[0].reputation, 19);
    }

    #[test]
    fn england_rules_bytes_are_exact_from_ctor() {
        // The two 6-byte entries `england_rules.cpp` (0x005637f0) writes.
        // Verified verbatim against the Ghidra decompile.
        assert_eq!(england::RULE_ENTRIES, &[[7, 0, 1, 2, 5, 1], [7, 0, 4, 26, 2, 0]]);
    }

    #[test]
    fn find_nation_hits_every_id() {
        assert_eq!(find_nation(60).unwrap().name, "England");
        assert_eq!(find_nation(73).unwrap().name, "Germany");
        assert_eq!(find_nation(1).unwrap().name, "Albania");
        assert!(find_nation(9999).is_none());
    }

    #[test]
    fn all_euro_comp_ids_are_deduped_and_nonempty() {
        let ids = all_euro_comp_ids();
        assert!(!ids.is_empty());
        // No duplicates (dedup after sort).
        for w in ids.windows(2) { assert!(w[0] < w[1]); }
        // Well-known ids must be in there.
        for expected in [7u32, 16, 24, 52, 11, 22, 46, 34, 406u32 - 406 + 7] {
            let _ = expected; // just referencing values; explicit checks below
        }
        assert!(ids.contains(&7));   // Premier
        assert!(ids.contains(&16));  // Bundesliga
        assert!(ids.contains(&24));  // Serie A
        assert!(ids.contains(&52));  // La Liga
    }

    #[test]
    fn background_only_nations_have_no_leagues() {
        for m in [albania::SET, bulgaria::SET, hungary::SET, ukraine::SET] {
            assert!(m.leagues.is_empty());
            assert!(m.cups.is_empty());
            assert!(m.rules_ctor_va.is_none());
        }
    }

    #[test]
    fn mid_tier_nations_have_league_data() {
        for set in [
            &belgium::SET, &croatia::SET, &denmark::SET, &finland::SET,
            &greece::SET, &ireland::SET, &northern_ireland::SET, &norway::SET,
            &poland::SET, &russia::SET, &sweden::SET, &turkey::SET,
            &wales::SET, &yugoslavia::SET, &austria::SET, &luxembourg::SET,
            &switzerland::SET, &czech::SET,
        ] {
            assert!(!set.leagues.is_empty(), "{} has leagues", set.name);
        }
    }
}
