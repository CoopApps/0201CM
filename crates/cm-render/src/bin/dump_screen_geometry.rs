//! Dump the ported code's widget geometry + slot list for a named screen,
//! matching the shape produced by `tools/capture_all_screens.py`
//! (reports/screen_captures/<name>.json).
//!
//! Usage:
//!   cargo run -p cm-render --bin dump_screen_geometry -- <screen>
//!   cargo run -p cm-render --bin dump_screen_geometry -- --list
//!
//! For every registered screen slug, this binary:
//!   1. instantiates a default View from cm-domain,
//!   2. calls `to_widget_pool()` (via `RenderableView`),
//!   3. classifies the returned widgets into areas / objects / tabs by KIND_,
//!   4. materialises the slot-index → value map the exe setup fn would push
//!      (per-View slot map coded below from the `/// Slot N:` doc annotations),
//!   5. writes reports/screen_captures/<slug>.render.json.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use cm_render::view_render::RenderableView;
use cm_render::widget_pool::{
    Widget, KIND_BUTTON, KIND_HEADER, KIND_LABEL, KIND_ROOT_HOLDER,
};

// ---------------------------------------------------------------------------
// Output structs

#[derive(Default)]
struct GeomOut {
    areas: Vec<AreaOut>,
    objects: Vec<ObjOut>,
    tabs: Vec<TabOut>,
    slots: Vec<SlotOut>,
}

struct AreaOut {
    id: u32,
    l: i32, t: i32, r: i32, b: i32,
    flags: u32,
    // cntA/wA/cntB/wB are the exe's spawn_area args 4-7 (nchildren_hint,
    // extra-buffer-ptr, color_slot, gradient_ptr per widget_pool::spawn_area's
    // doc comment). Every area captured so far (reports/screen_captures/
    // news.json) uses the exe's default for a simple, non-tabular,
    // non-gradient area: nchildren_hint=1, no extra buffer (0), color_slot=1,
    // no gradient (0). Hardcoded rather than threaded through
    // WidgetDescriptor until a captured screen shows a differing area.
    cnt_a: i64, w_a: i64, cnt_b: i64, w_b: i64,
}
struct ObjOut {
    id: u32,
    ty: i64,
    l: i32, t: i32, r: i32, b: i32,
    rflags: u32,
    font: i64,
    text: String,
}
struct TabOut {
    id: u32,
    l: i32, t: i32, r: i32, b: i32,
    label: String,
    highlighted: bool,
}
struct SlotOut { idx: u32, val: i64, kind: u32 }

// ---------------------------------------------------------------------------
// Widget → geom classification

fn classify_widgets(widgets: &[Widget]) -> GeomOut {
    let mut out = GeomOut::default();
    for (i, w) in widgets.iter().enumerate() {
        let id = (i + 1) as u32;
        match w.descriptor.kind {
            KIND_HEADER | KIND_LABEL => out.objects.push(ObjOut {
                id, ty: w.descriptor.kind as i64,
                l: w.left, t: w.top, r: w.right, b: w.bottom,
                rflags: w.flags, font: w.descriptor.text_style as i64,
                text: w.descriptor.text.clone(),
            }),
            KIND_BUTTON => out.tabs.push(TabOut {
                id,
                l: w.left, t: w.top, r: w.right, b: w.bottom,
                label: w.descriptor.text.clone(),
                highlighted: (w.descriptor.kind & 0x0800) != 0,
            }),
            KIND_ROOT_HOLDER => out.areas.push(AreaOut {
                id,
                l: w.left, t: w.top, r: w.right, b: w.bottom,
                flags: w.flags,
                cnt_a: 1, w_a: 0, cnt_b: 1, w_b: 0,
            }),
            _ => out.objects.push(ObjOut {
                id, ty: w.descriptor.kind as i64,
                l: w.left, t: w.top, r: w.right, b: w.bottom,
                rflags: w.flags, font: w.descriptor.text_style as i64,
                text: w.descriptor.text.clone(),
            }),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Per-screen builders. Each returns (widgets_from_view, ordered_slots).
//
// Slot maps are transcribed from the `/// Slot N:` doc annotations on each
// View struct in cm-domain. See reports/screen_captures/<slug>.json for the
// exe ground-truth values.

type Builder = fn() -> (Vec<Widget>, Vec<SlotOut>);

fn slot(i: u32, v: i64) -> SlotOut { SlotOut { idx: i, val: v, kind: 0 } }

// ---- screen_batch3 --------------------------------------------------------

fn build_fifa_rankings() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch3::FifaRankingsView;
    let v = FifaRankingsView::default();
    let s = vec![
        slot(0, v.selected_nation as i64),
        slot(1, v.continent_filter as i64),
        slot(2, v.sort_by_points as i64),
        slot(3, v.show_averages as i64),
    ];
    (v.to_widget_pool(), s)
}

fn build_uefa_coefficients() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch3::UefaCoefficientsView;
    let v = UefaCoefficientsView::default();
    let s = vec![
        slot(0, v.selected_nation as i64),
        slot(1, v.season_offset as i64),
    ];
    (v.to_widget_pool(), s)
}

fn build_latest_scores() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch3::LatestScoresView;
    let v = LatestScoresView::default();
    // No captured slot writes (empty in exe capture).
    (v.to_widget_pool(), Vec::new())
}

fn build_manager_history() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch3::ManagerHistoryView;
    let v = ManagerHistoryView::default();
    // Slot map (from the /// Slot annotations):
    //   0: person_id (pointer)
    //   1: history mode byte (param_2 — 5 = full career)
    //   2: sub mode
    //   ... many zeros written by the setup fn.
    let s = vec![
        slot(0, v.person_id as i64),
        slot(1, v.history_mode as i64),
        slot(2, v.sub_mode as i64),
        slot(3, v.extra_filter as i64),
    ];
    (v.to_widget_pool(), s)
}

// ---- screen_batch4 --------------------------------------------------------

fn build_send_abuse() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch4::SendAbuseView;
    let v = SendAbuseView::default();
    let s = vec![
        slot(0, v.recipient_id.map(|x| x as i64).unwrap_or(0)),
        slot(1, 0), // 101-byte message body pointer
        slot(2, v.note as i64),
    ];
    (v.to_widget_pool(), s)
}

// ---- screen_batch5 --------------------------------------------------------

fn build_awards() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch5::AwardsView;
    let v = AwardsView::default();
    let s = vec![
        slot(0, v.award_table as i64),
        slot(1, v.view_mode as i64),
    ];
    (v.to_widget_pool(), s)
}

fn build_competition_dashboard() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch5::CompetitionDashboardView;
    let v = CompetitionDashboardView::default();
    // Slot 0: view_mode; Slot 1: view_sub_mode; Slot 0x1E (30): primary_tab (0x11=17).
    let s = vec![
        slot(0, v.view_mode as i64),
        slot(1, v.view_sub_mode as i64),
        slot(30, v.primary_tab as i64),
    ];
    (v.to_widget_pool(), s)
}

// ---- screen_batch6 --------------------------------------------------------

fn build_club_history() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch6::ClubHistoryView;
    let v = ClubHistoryView::default();
    let s = vec![
        slot(0, v.club_id as i64),
        slot(1, v.view_mode as i64),
        slot(2, v.sort_order as i64),
    ];
    (v.to_widget_pool(), s)
}

fn build_player_contract() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch6::ContractView;
    let v = ContractView {
        player_id: 0, is_managed: false, player_flag_write: None,
    };
    let s = vec![
        slot(0, v.player_id as i64),
        slot(1, v.is_managed as i64),
    ];
    (v.to_widget_pool(), s)
}

// ---- screen_batch7 --------------------------------------------------------

fn build_match_report() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch7::MatchReportView;
    let v = MatchReportView::default();
    let s = vec![
        slot(0, v.fixture_handle as i64),
        slot(1, v.detail_level as i64),
        slot(2, v.default_section as i64),
    ];
    (v.to_widget_pool(), s)
}

// ---- screen_batch8 --------------------------------------------------------

fn build_entity_list() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch8::EntityListView;
    let v = EntityListView::default();
    // Exe writes focus at 1, 4, 7, 16; mode at 0.
    let s = vec![
        slot(0, v.mode as i64),
        slot(1, v.focus as i64),
        slot(4, v.focus as i64),
        slot(7, v.focus as i64),
        slot(16, v.focus as i64),
    ];
    (v.to_widget_pool(), s)
}

fn build_find_dialog() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch8::FindDialogView;
    let v = FindDialogView::default();
    // 4 buffer ptrs + mode
    let s = vec![
        slot(0, 0), slot(1, 0), slot(2, 0), slot(3, 0),
        slot(4, v.mode as i64),
    ];
    (v.to_widget_pool(), s)
}

fn build_hall_of_fame() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch8::HallOfFameView;
    let v = HallOfFameView::default();
    let s = vec![
        slot(0, v.selected_tab as i64),
        slot(1, v.scroll_offset as i64),
        slot(2, v.filter as i64),
    ];
    (v.to_widget_pool(), s)
}

fn build_written_history() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch8::WrittenHistoryView;
    let v = WrittenHistoryView::default();
    let mut s = vec![slot(0, v.category as i64)];
    for (i, r) in v.reserved.iter().enumerate() {
        s.push(slot(1 + i as u32, *r as i64));
    }
    (v.to_widget_pool(), s)
}

fn build_selected_leagues() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_batch8::SelectedLeaguesView;
    let v = SelectedLeaguesView::default();
    let s = vec![slot(0, v.cursor as i64)];
    (v.to_widget_pool(), s)
}

// ---- registry -------------------------------------------------------------

fn registry() -> BTreeMap<&'static str, Builder> {
    let mut m: BTreeMap<&'static str, Builder> = BTreeMap::new();
    // Exe-capture slugs.
    m.insert("screen_batch3_fifa_rankings_cmd_0x3f3_98_b",     build_fifa_rankings);
    m.insert("screen_batch3_uefa_coefficients_cmd_0x40c_63_b", build_uefa_coefficients);
    m.insert("screen_batch3_latest_scores_cmd_0x418_325_b_pe", build_latest_scores);
    m.insert("screen_batch3_manager_history_cmd_0x3ec_929_b",  build_manager_history);
    m.insert("screen_batch4_send_abuse_cmd_0x414_109_b",       build_send_abuse);
    m.insert("screen_batch5_awards_cmd_0x7d4_196_b_2_slots",   build_awards);
    m.insert("screen_batch5_competition_dashboard_cmd_2000_1", build_competition_dashboard);
    m.insert("screen_batch6_club_history_screen_cmd_0x7e6_86", build_club_history);
    m.insert("screen_batch6_player_contract_screen_cmd_0x7d1", build_player_contract);
    m.insert("screen_batch7_match_report_screen_setup_cmd_0x", build_match_report);
    m.insert("screen_batch8_entity_list_cmd_0x3f4_6_7_etc_48", build_entity_list);
    m.insert("screen_batch8_find_dialog_cmd_0x3fa_544_b_4_se", build_find_dialog);
    m.insert("screen_batch8_hall_of_fame_cmd_0x42a_79_b_3_sl", build_hall_of_fame);
    m.insert("screen_batch8_written_history_cmd_0x429_117_b",  build_written_history);
    m.insert("screen_batch8_selected_leagues_setup_cmd_0x431", build_selected_leagues);
    // Short/friendly aliases.
    m.insert("fifa_rankings",        build_fifa_rankings);
    m.insert("uefa_coefficients",    build_uefa_coefficients);
    m.insert("latest_scores",        build_latest_scores);
    m.insert("manager_history",      build_manager_history);
    m.insert("hall_of_fame",         build_hall_of_fame);
    m.insert("match_report",         build_match_report);
    m.insert("competition_dashboard", build_competition_dashboard);
    m.insert("club_history",         build_club_history);
    m.insert("news",                 build_news);
    // "dashboard" (the club dashboard, a different screen from News/home)
    // has no exe capture yet -- ClubDashboardView is its real port, kept
    // as its own entry rather than a News stand-in.
    m.insert("dashboard",            build_club_dashboard_alias);
    m
}

fn build_news() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::{NewsCategory, NewsItem, NewsView};
    // Real state from a live Frida capture of the running exe (see
    // DIRECTDRAW_CAPTURE_HANDOVER.md §11) -- the earlier Unicorn-emulation
    // ground truth (news.json) only ever saw the header, since it forced
    // "0 unread news" and skipped the whole list-building loop.
    fn item(date_label: &str, headline: &str, body: &str) -> NewsItem {
        NewsItem {
            date: cm_domain::GameDate { year: 2001, month: 10, day: 10 },
            date_label: date_label.to_string(), headline: headline.to_string(),
            body: body.to_string(), category: NewsCategory::Message, unread: false,
        }
    }
    let v = NewsView {
        title: "Christoph Olewicz News".to_string(),
        items: vec![
            item("Wed 10th Oct EVE", "Board reaction to Rhayader game",
                "The Haverfordwest County board of directors are pleased with the 1-0 \
                 Welsh League Cup win against Rhayader Town."),
            item("Wed 10th Oct EVE", "Haverfordwest win in Welsh League Cup Quarter Final", ""),
            item("Mon 8th Oct EVE", "Brazil out for about 2 weeks", ""),
            item("Sun 7th Oct EVE", "Sweden ready for 2002", ""),
            item("Sun 7th Oct EVE", "Burrows resumes full training", ""),
        ],
        selected_tab: 0, tab_count: 8,
        selected_item: 0, nav_back_enabled: true, nav_next_enabled: false,
    };
    (v.to_widget_pool(), Vec::new())
}

fn build_club_dashboard_alias() -> (Vec<Widget>, Vec<SlotOut>) {
    use cm_domain::screen_club_dashboard::ClubDashboardView;
    let v = ClubDashboardView::default();
    // Slot layout for the club dashboard is decoded but not comprehensively
    // annotated; emit tab_index as slot 0 as a first cut.
    let s = vec![slot(0, v.tab_index as i64)];
    (v.to_widget_pool(), s)
}

// ---------------------------------------------------------------------------
// JSON emission

fn to_json(name: &str, g: &GeomOut) -> String {
    let mut s = String::new();
    s.push_str("{\n");
    s.push_str(&format!("  \"screen\": \"{name}\",\n"));

    s.push_str("  \"areas\": [\n");
    for (i, a) in g.areas.iter().enumerate() {
        s.push_str(&format!(
            "    {{ \"id\": {}, \"L\": {}, \"T\": {}, \"R\": {}, \"B\": {}, \"cntA\": {}, \"wA\": {}, \"cntB\": {}, \"wB\": {}, \"flags\": {} }}{}\n",
            a.id, a.l, a.t, a.r, a.b, a.cnt_a, a.w_a, a.cnt_b, a.w_b, a.flags,
            if i + 1 == g.areas.len() { "" } else { "," }));
    }
    s.push_str("  ],\n  \"objects\": [\n");
    for (i, o) in g.objects.iter().enumerate() {
        s.push_str(&format!(
            "    {{ \"id\": {}, \"type\": {}, \"L\": {}, \"T\": {}, \"R\": {}, \"B\": {}, \"rflags\": {}, \"font\": {}, \"text\": {:?} }}{}\n",
            o.id, o.ty, o.l, o.t, o.r, o.b, o.rflags, o.font, o.text,
            if i + 1 == g.objects.len() { "" } else { "," }));
    }
    s.push_str("  ],\n  \"tabs\": [\n");
    for (i, t) in g.tabs.iter().enumerate() {
        s.push_str(&format!(
            "    {{ \"id\": {}, \"L\": {}, \"T\": {}, \"R\": {}, \"B\": {}, \"label\": {:?}, \"highlighted\": {} }}{}\n",
            t.id, t.l, t.t, t.r, t.b, t.label, t.highlighted,
            if i + 1 == g.tabs.len() { "" } else { "," }));
    }
    s.push_str("  ],\n  \"nav\": [],\n  \"slots\": [\n");
    for (i, sl) in g.slots.iter().enumerate() {
        s.push_str(&format!(
            "    {{ \"idx\": {}, \"val\": {}, \"kind\": {} }}{}\n",
            sl.idx, sl.val, sl.kind,
            if i + 1 == g.slots.len() { "" } else { "," }));
    }
    s.push_str("  ]\n}\n");
    s
}

fn out_path(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); p.pop();
    p.push("reports"); p.push("screen_captures");
    let _ = fs::create_dir_all(&p);
    p.push(format!("{name}.render.json"));
    p
}

fn main() {
    let mut args = std::env::args().skip(1);
    let Some(first) = args.next() else {
        eprintln!("usage: dump_screen_geometry <screen> | --gen <screen> | --list");
        std::process::exit(2);
    };
    let reg = registry();
    if first == "--list" {
        for k in reg.keys() { println!("  {k}"); }
        println!("  -- generated (use --gen) --");
        for k in cm_render::gen_screens::all_slugs() { println!("  {k}"); }
        return;
    }
    // --gen <slug>: emit the GENERATED skeleton (tools/gen_screen_rs.py
    // output) verbatim, full capture fidelity, bypassing the hand-written
    // View builders. This is the round-trip half of the codegen validation:
    //   capture.json -> gen_screens/<slug>.rs -> <slug>.render.json -> diff.
    if first == "--gen" {
        let name = args.next().unwrap_or_default();
        let Some(f) = cm_render::gen_screens::lookup(&name) else {
            eprintln!("no generated screen {name:?} -- run tools/gen_screen_rs.py");
            std::process::exit(2);
        };
        let json = f().to_capture_json();
        let p = out_path(&name);
        fs::write(&p, &json).expect("write render.json");
        eprintln!("wrote {} (generated)", p.display());
        return;
    }
    // Accept --screen NAME too.
    let name = if first == "--screen" {
        args.next().unwrap_or_default()
    } else {
        first
    };
    let Some(builder) = reg.get(name.as_str()) else {
        eprintln!("unknown screen {name:?} -- pass --list");
        std::process::exit(2);
    };
    let (widgets, slots) = builder();
    let mut geom = classify_widgets(&widgets);
    geom.slots = slots;
    let json = to_json(&name, &geom);
    let p = out_path(&name);
    fs::write(&p, &json).expect("write render.json");
    eprintln!("wrote {}", p.display());
}
