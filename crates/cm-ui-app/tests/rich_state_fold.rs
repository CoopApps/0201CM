//! Category B rich-state fold coverage.
//!
//! Each of the seven rich-state builders in
//! `cm_render::screen_rich_state` must accept a live cm-domain View,
//! spawn widgets into a fresh pool, and the resulting pool must feed
//! `packed_widget::render_widget` without panicking. This test drives
//! the same code path `render_new::try_render_rich_state` uses in
//! `App::render`, minus the softbuffer bridge.

use cm_render::packed::PackedSurface;
use cm_render::packed_glyph::PixelFont;
use cm_render::packed_widget::{render_widget, WidgetGlobals};
use cm_render::pool_to_render::to_render_widget;
use cm_render::screen_rich_state;
use cm_render::widget_pool::GuiRecordPool;

fn dummy_date() -> cm_domain::GameDate {
    cm_domain::GameDate { day: 10, month: 8, year: 2001 }
}

fn render_pool_smoke(pool: &GuiRecordPool) {
    let font = PixelFont::empty(11);
    let mut packed = PackedSurface::rgb555(800, 600);
    let n = pool.widgets.len();
    for i in 0..n {
        let mut rw = to_render_widget(&pool.widgets[i]);
        rw.frame_idx = -1;
        render_widget(&mut packed, &mut rw, Some(pool), &font, WidgetGlobals::default(), true);
    }
    // Non-zero widget count is the sanity we care about — the render
    // pass is a "does not panic" check.
    assert!(n > 0, "rich-state builder must spawn widgets");
}

#[test]
fn news_view_populates_and_renders() {
    let view = cm_domain::NewsView {
        title: "Test News".into(),
        items: vec![cm_domain::NewsItem {
            date: dummy_date(),
            date_label: "Wed 10 Aug".into(),
            headline: "Test headline".into(),
            body: "Test body".into(),
            category: cm_domain::NewsCategory::Message,
            unread: true,
        }],
        selected_tab: 12,
        tab_count: 8,
        selected_item: 0,
        nav_back_enabled: true,
        nav_next_enabled: false,
    };
    let mut pool = GuiRecordPool::new();
    screen_rich_state::build_news_from_view(&mut pool, &view).unwrap();
    render_pool_smoke(&pool);
}

#[test]
fn dashboard_club_view_populates_and_renders() {
    let club = cm_domain::ClubDashboard {
        manager_name: "Mgr".into(),
        club_id: 1, club_name: "FC".into(),
        division_name: "Div".into(),
        position: 1, division_size: 20,
        date: dummy_date(), next_fixture: None,
        squad: vec![],
    };
    let mut pool = GuiRecordPool::new();
    screen_rich_state::build_dashboard_from_view(
        &mut pool, &cm_domain::DashboardView::Club(club), 0,
    ).unwrap();
    render_pool_smoke(&pool);
}

#[test]
fn league_table_view_populates_and_renders() {
    let view = cm_domain::LeagueTableView {
        competition_id: 1, competition_name: "Test".into(),
        rows: vec![],
    };
    let mut pool = GuiRecordPool::new();
    screen_rich_state::build_league_table_from_view(&mut pool, &view, 0).unwrap();
    render_pool_smoke(&pool);
}

#[test]
fn player_profile_view_populates_and_renders() {
    let view = cm_domain::PlayerProfile {
        player_id: 1, name: "Test".into(),
        age: Some(25), club_name: None,
        born_line: "Born".into(), nation_name: None,
        wage: 0, value: 0, international_caps: 0, international_goals: 0,
        positions: vec![], attributes: vec![],
    };
    let mut pool = GuiRecordPool::new();
    screen_rich_state::build_player_profile_from_view(&mut pool, &view).unwrap();
    render_pool_smoke(&pool);
}

#[test]
fn club_fixtures_view_populates_and_renders() {
    let view = cm_domain::ClubFixturesView {
        club_id: 1, club_name: "FC".into(), rows: vec![],
    };
    let mut pool = GuiRecordPool::new();
    screen_rich_state::build_club_fixtures_from_view(&mut pool, &view, 0).unwrap();
    render_pool_smoke(&pool);
}

#[test]
fn selected_leagues_view_populates_and_renders() {
    let mut pool = GuiRecordPool::new();
    screen_rich_state::build_selected_leagues_from_view(&mut pool, &[], None).unwrap();
    render_pool_smoke(&pool);
}

#[test]
fn widget_pool_debug_populates_and_renders() {
    let mut pool = GuiRecordPool::new();
    screen_rich_state::build_widget_pool_debug_from_view(&mut pool, "smoke", &[]).unwrap();
    render_pool_smoke(&pool);
}
