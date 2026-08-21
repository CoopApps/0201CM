//! cm-gui: render the CM0102 daily/news screen ("Moby Player News").
//! Ported primitives (surface/font/panel); layout + palette matched to the
//! reference screenshot; real decoded arial glyph fonts; a team photo behind
//! the news detail. Standings/news are the VIEW-MODEL the backend fills.

mod surface;
mod font;
mod panel;

use font::{draw_text_box, Font, F_LEFT, F_RIGHT};
use panel::{border, fill_rect};
use surface::{pack565, Surface, W};

const ASSETS: &str = "D:/cm0102-rs/assets/cm0102/fonts";

// palette matched to the reference screenshot (the CM 01/02 navy skin)
const SIDE_BG: (u8, u8, u8) = (10, 14, 46);
const SIDE_TX: (u8, u8, u8) = (150, 170, 216);
const SIDE_HI: (u8, u8, u8) = (176, 196, 236);
const BAN_TOP: (u8, u8, u8) = (46, 66, 176);
const BAN_BOT: (u8, u8, u8) = (26, 40, 116);
const WHITE: (u8, u8, u8) = (245, 246, 255);
const TAB: (u8, u8, u8) = (78, 60, 128);
const TABA: (u8, u8, u8) = (52, 38, 96);
const GOLD: (u8, u8, u8) = (208, 172, 58);
const ROW: (u8, u8, u8) = (146, 146, 148);
const ROWA: (u8, u8, u8) = (126, 126, 128);
const RED: (u8, u8, u8) = (150, 46, 46);
const DARK: (u8, u8, u8) = (24, 24, 26);
const YELLOW: (u8, u8, u8) = (246, 224, 42);
const BTN: (u8, u8, u8) = (72, 56, 120);
const GREY: (u8, u8, u8) = (150, 150, 162);

fn vgrad(s: &mut Surface, x0: i32, y0: i32, x1: i32, y1: i32, c0: (u8, u8, u8), c1: (u8, u8, u8)) {
    let h = (y1 - y0).max(1);
    for y in y0..y1 {
        let t = (y - y0) as i32;
        let lerp = |a: u8, b: u8| (a as i32 + (b as i32 - a as i32) * t / h) as u8;
        let p = pack565(lerp(c0.0, c1.0), lerp(c0.1, c1.1), lerp(c0.2, c1.2));
        for x in x0..x1 {
            s.set(x, y, p);
        }
    }
}

fn photo(s: &mut Surface, x0: i32, y0: i32, path: &str, pw: i32, ph: i32) {
    if let Ok(raw) = std::fs::read(path) {
        for y in 0..ph {
            for x in 0..pw {
                let i = ((y * pw + x) * 3) as usize;
                if i + 2 < raw.len() {
                    s.set(x0 + x, y0 + y, pack565(raw[i], raw[i + 1], raw[i + 2]));
                }
            }
        }
    }
}

fn bevel_btn(s: &mut Surface, x0: i32, y0: i32, x1: i32, y1: i32, c: (u8, u8, u8)) {
    fill_rect(s, x0, y0, x1, y1, c);
    let lt = (c.0.saturating_add(40), c.1.saturating_add(40), c.2.saturating_add(40));
    let dk = (c.0 / 2, c.1 / 2, c.2 / 2);
    border(s, x0, y0, x1, y1, dk);
    for x in x0..=x1 {
        s.set(x, y0, pack565(lt.0, lt.1, lt.2));
    }
}

fn main() {
    let big = Font::load(&format!("{ASSETS}/arial_18.json"));
    let mid = Font::load(&format!("{ASSETS}/arial_14.json"));
    let sm = Font::load(&format!("{ASSETS}/arial_narrow_11.json"));

    let mut s = Surface::new();
    s.fill(SIDE_BG.0, SIDE_BG.1, SIDE_BG.2);

    // ---- left sidebar ----
    fill_rect(&mut s, 0, 0, 95, 599, SIDE_BG);
    border(&mut s, 5, 6, 90, 44, (60, 70, 130));
    draw_text_box(&mut s, &sm, 6, 10, 90, "Sunday", 0, SIDE_TX);
    draw_text_box(&mut s, &sm, 6, 25, 90, "15.7.01 AM", 0, SIDE_TX);
    draw_text_box(&mut s, &mid, 8, 54, 90, "<<<   >>>", 0, SIDE_TX);
    for (y, t, c) in [
        (78, "Continue", SIDE_HI), (90, "Game", SIDE_HI),
        (120, "Moby", SIDE_HI), (132, "Player", SIDE_HI),
        (162, "Competitions", SIDE_TX), (192, "Nations", SIDE_TX), (204, "& Clubs", SIDE_TX),
        (234, "Find", SIDE_TX), (264, "Game", SIDE_TX), (276, "Options", SIDE_TX),
    ] {
        draw_text_box(&mut s, &sm, 4, y, 92, t, 0, c);
    }

    // ---- title banner ----
    vgrad(&mut s, 96, 0, 800, 62, BAN_TOP, BAN_BOT);
    draw_text_box(&mut s, &big, 96, 20, 800, "Moby Player News", 0, WHITE);

    // ---- top tabs ----
    let tabs = [("All", true), ("Messages", false), ("Competitions", false), ("Injuries and Bans", false)];
    let mut tx = 100;
    let tw = 172;
    for (lab, active) in tabs {
        let x1 = tx + tw;
        fill_rect(&mut s, tx, 78, x1, 108, if active { TABA } else { TAB });
        if active {
            border(&mut s, tx, 78, x1, 108, GOLD);
            border(&mut s, tx + 1, 79, x1 - 1, 107, GOLD);
        }
        draw_text_box(&mut s, &sm, tx, 87, x1, lab, 0, WHITE);
        tx = x1 + 3;
    }

    // ---- news list ----
    let news = [
        ("Sun 15th Jul AM", "Derby transfer bid for Bentley", false),
        ("Sun 15th Jul AM", "Middlesbrough transfer bid for Bentley", false),
        ("Sun 15th Jul AM", "Southampton transfer bid for Bentley", false),
        ("Sun 15th Jul AM", "Sunderland transfer bid for Bentley", false),
        ("Sun 15th Jul AM", "Everton transfer bid for Bentley", true),
    ];
    let ly = 128;
    for (i, (date, head, sel)) in news.iter().enumerate() {
        let y = ly + i as i32 * 17;
        let fill = if *sel { RED } else if i % 2 == 0 { ROW } else { ROWA };
        fill_rect(&mut s, 100, y, 772, y + 16, fill);
        draw_text_box(&mut s, &sm, 106, y + 1, 200, date, F_LEFT, DARK);
        draw_text_box(&mut s, &sm, 210, y + 1, 770, head, F_LEFT, if *sel { WHITE } else { DARK });
    }
    // scrollbar
    bevel_btn(&mut s, 774, 128, 794, 213, TAB);
    // filter + next unread
    draw_text_box(&mut s, &sm, 330, 220, 430, "Filter :", F_LEFT, WHITE);
    bevel_btn(&mut s, 656, 218, 794, 238, TAB);
    draw_text_box(&mut s, &sm, 656, 223, 794, "Next Unread", 0, WHITE);

    // ---- detail area over the team photo ----
    photo(&mut s, 100, 244, "D:/cm0102-rs/crates/cm-gui/news_bg.raw", 700, 320);
    draw_text_box(&mut s, &mid, 100, 250, 800, "Everton transfer bid for Bentley", 0, YELLOW);
    draw_text_box(&mut s, &mid, 118, 296, 720, "Everton have offered Chesterfield's Dave Bentley a job as scout.", F_LEFT, WHITE);
    draw_text_box(&mut s, &mid, 118, 336, 720, "Chesterfield will be due \u{a3}26,000 in compensation if the deal goes", F_LEFT, WHITE);
    draw_text_box(&mut s, &mid, 118, 356, 720, "ahead", F_LEFT, WHITE);

    // ---- bottom tabs + Back/Next ----
    let btabs = ["Contracts and Media", "Transfers", "Jobs", "Records"];
    let mut bx = 100;
    for lab in btabs {
        let x1 = bx + 172;
        bevel_btn(&mut s, bx, 516, x1, 544, BTN);
        draw_text_box(&mut s, &sm, bx, 524, x1, lab, 0, WHITE);
        bx = x1 + 3;
    }
    bevel_btn(&mut s, 100, 554, 446, 588, GREY);
    draw_text_box(&mut s, &big, 100, 562, 446, "Back", 0, DARK);
    bevel_btn(&mut s, 450, 554, 795, 588, GREY);
    draw_text_box(&mut s, &big, 450, 562, 795, "Next", 0, DARK);

    let out = "D:/cm0102-rs/reports/carve_segment_index/renders/cm_gui_news.ppm";
    std::fs::write(out, s.to_ppm()).expect("write ppm");
    println!("cm-gui rendered news screen -> {out} ({}x{})", W, surface::H);
}
