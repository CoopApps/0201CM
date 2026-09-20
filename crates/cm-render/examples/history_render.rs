//! THROWAWAY (2026-09-20): render the Player Profile → History subtab with
//! Martin Keown's captured season list, scrolled to the bottom, to diff
//! against `fixtures/player_history_screen/keown_scrolled.json`.
//! Run: cargo run -p cm-render --example history_render -- out.raw

use cm_render::font::Fonts;
use cm_render::packed::PackedSurface;
use cm_render::screen_player_profile_faithful::{
    render_player_profile, HistoryRow, PlayerProfileState,
};
use std::io::Write;

fn main() {
    let out = std::env::args().nth(1).unwrap_or_else(|| "history_ours.raw".into());
    let mut fonts = Fonts::new("D:/cm0102/Data");
    let mut surface = PackedSurface::rgb555(800, 600);

    // Keown's season list, newest first (year DESC, apps ASC within year) —
    // exactly what `player_history_view_for` produces for person 55141.
    let rows: [(&str, &str, bool, &str, &str); 22] = [
        ("2001/2", "Arsenal", false, "-", "-"),
        ("2000/1", "Arsenal", false, "28", "0"),
        ("1999/0", "Arsenal", false, "27", "1"),
        ("1998/9", "Arsenal", false, "34", "1"),
        ("1997/8", "Arsenal", false, "18", "0"),
        ("1996/7", "Arsenal", false, "33", "1"),
        ("1995/6", "Arsenal", false, "34", "0"),
        ("1994/5", "Arsenal", false, "31", "1"),
        ("1993/4", "Arsenal", false, "33", "0"),
        ("1992/3", "Everton", false, "13", "0"),
        ("1992/3", "Arsenal", false, "16", "0"),
        ("1991/2", "Everton", false, "39", "0"),
        ("1990/1", "Everton", false, "24", "0"),
        ("1989/0", "Everton", false, "20", "0"),
        ("1988/9", "Aston Villa", false, "34", "0"),
        ("1987/8", "Aston Villa", false, "42", "3"),
        ("1986/7", "Aston Villa", false, "36", "0"),
        ("1985/6", "Brighton", true, "7", "1"),
        ("1985/6", "Arsenal", false, "22", "0"),
        ("1984/5", "Arsenal", false, "-", "-"),
        ("1984/5", "Brighton", true, "16", "0"),
        ("1983/4", "Arsenal", false, "-", "-"),
    ];
    let hist: Vec<HistoryRow> = rows.iter()
        .map(|(s, c, l, a, g)| HistoryRow { season: s, club: c, is_loan: *l, apps: a, goals: g })
        .collect();

    // Bottom breakdown: selected = current 2001/2 season, all blank.
    let breakdown: [(&str, [&str; 9]); 6] = [
        ("Non Competitive", ["-", "-", "-", "-", "-", "-", "-", "-", "----"]),
        ("League",          ["-", "-", "-", "-", "-", "-", "-", "-", "----"]),
        ("Cup",             ["-", "-", "-", "-", "-", "-", "-", "-", "----"]),
        ("Continental",     ["-", "-", "-", "-", "-", "-", "-", "-", "----"]),
        ("International",   ["-", "-", "-", "-", "-", "-", "-", "-", "----"]),
        ("Senior Club",     ["-", "-", "-", "-", "-", "-", "-", "-", "----"]),
    ];

    let ach_mode = std::env::args().nth(2).as_deref() == Some("ach");
    let ach_rows: Vec<cm_render::screen_player_profile_faithful::AchievementRow> = if ach_mode {
        vec![
            ("1.4.35", "Stalybridge", "English League Cup Champions"),
            ("25.5.33", "Stalybridge", "English Premier Division Champions"),
            ("5.4.31", "Stalybridge", "Named in 2030/31 English Premier Division Select"),
            ("22.5.30", "Stalybridge", "European Champions Cup Champions"),
            ("12.8.29", "Stalybridge", "English Premier Division Player of the Season"),
        ].into_iter().map(|(d, c, t)| cm_render::screen_player_profile_faithful::AchievementRow {
            date: d, club: c, text: t,
        }).collect()
    } else { Vec::new() };

    let mode = std::env::args().nth(2);
    let inj_rows: Vec<cm_render::screen_player_profile_faithful::InjuryRow> =
        if mode.as_deref() == Some("inj") {
            vec![
                ("14.2.37", "twisted knee", "Match", "3 weeks"),
                ("20.12.36", "bruised thigh", "Match", "10 days"),
                ("22.8.35", "thigh strain", "Training", "11 days"),
                ("11.4.35", "chest injury", "Training", "2 weeks"),
            ].into_iter().map(|(d,n,k,p)| cm_render::screen_player_profile_faithful::InjuryRow {
                date:d, injury:n, kind:k, period:p }).collect()
        } else { Vec::new() };
    let ban_rows: Vec<cm_render::screen_player_profile_faithful::BanRow> =
        if mode.as_deref() == Some("ban") {
            vec![
                ("26.12.35", "3 match English ban", "Red card"),
                ("8.11.34", "1 match European ban", "Red card"),
                ("31.1.32", "1 match English ban", "5 yellow cards"),
            ].into_iter().map(|(d,b,r)| cm_render::screen_player_profile_faithful::BanRow {
                date:d, ban:b, reason:r }).collect()
        } else { Vec::new() };

    let empty: [String; 4] = std::array::from_fn(|_| String::new());
    let picker: Vec<String> = [
        "Blackwell, C", "Dawson, G", "Dolby, W", "Donaghy, T", "Dunne, D",
        "Edwards, M", "Evrard, K", "Gibson, L", "Hall, C", "Hamilton, J",
        "Hutt, P", "Jones, R", "Lawrence, A", "Mitten, D", "Morris, C",
    ].iter().map(|s| s.to_string()).collect();
    let st = PlayerProfileState {
        title: "5. Martin Keown (Arsenal)",
        born_line: "",
        attributes: &[],
        status: &empty,
        position: "Defender (Centre)",
        career: &breakdown,
        active_subtab: 4,
        injuries: &empty_arr(),
        contract: &empty_arr7(),
        transfer: &empty_arr7(),
        is_goalkeeper: false,
        kit_bg: 0x7c00, // red-ish Arsenal kit for the banner
        photo_seed: 12345,
        has_manager: true,
        history_rows: &hist,
        history_total: ("507", "8"),
        history_selected_label: "  2001/2 Arsenal",
        history_selected_idx: 0,
        history_scroll: 0,
        history_page: 0,
        history_bot_page: 0,
        history_view: match mode.as_deref() {
            Some("ach") => 0, Some("inj") => 2, Some("ban") => 3, _ => 1,
        },
        history_filter: 0,
        open_menu: match std::env::args().nth(2).as_deref() {
            Some("view") => cm_render::screen_player_profile_faithful::ProfileMenu::View,
            Some("picker") => cm_render::screen_player_profile_faithful::ProfileMenu::Picker,
            Some("action") => cm_render::screen_player_profile_faithful::ProfileMenu::Compare,
            _ => cm_render::screen_player_profile_faithful::ProfileMenu::None,
        },
        picker_items: &picker,
        achievements: &ach_rows,
        injuries_list: &inj_rows,
        bans_list: &ban_rows,
    };
    render_player_profile(&mut surface, &mut fonts, &st);

    let mut f = std::fs::File::create(&out).unwrap();
    for &p in &surface.buf {
        f.write_all(&p.to_le_bytes()).unwrap();
    }
    eprintln!("wrote {out}");
}

fn empty_arr() -> [String; 5] { std::array::from_fn(|_| String::new()) }
fn empty_arr7() -> [String; 7] { std::array::from_fn(|_| String::new()) }
