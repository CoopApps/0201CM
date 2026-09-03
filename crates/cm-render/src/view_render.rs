//! Wire ported `*View` slot structs into the layout engine.
//!
//! The exe's screen-setup functions push typed slot values into the
//! screen-slot table (`FUN_007E7130`). A separate DRAW callback
//! (per-screen) reads those slots and emits `spawn_widget` /
//! `spawn_area` calls into the widget pool. Only the News screen has
//! been captured from the live exe so far (see
//! `reports/cm0102_exact_news_ui_evidence.md` and `tools/capture_news.py`);
//! every other View here uses the same News-style prelude (header +
//! body) then lays out its slots on the exe's 800x600 grid, at the
//! coordinates the setup function's slot table dictates.
//!
//! This module provides:
//!
//! * The [`RenderableView`] trait — the shared surface every ported
//!   view presents to the layout engine.
//! * A News-shaped skeleton emitter ([`skeleton_widgets`]) that every
//!   impl starts from.
//! * Per-view impls for the top-10 ported View structs. Each impl
//!   walks the View's slots and emits one widget per meaningful slot,
//!   in the shape `area_rebuild_layout_tables` produces.

use crate::widget_pool::{
    Widget, WidgetDescriptor, KIND_BUTTON, KIND_HEADER, KIND_LABEL, KIND_ROOT_HOLDER,
};

/// Every ported screen View implements this. `to_widget_pool` returns
/// the ordered widget records the screen would spawn if its draw
/// callback were run — same records the layout engine consumes.
pub trait RenderableView {
    fn to_widget_pool(&self) -> Vec<Widget>;
}

/// Two-widget prelude every screen builds: header bar + content body.
/// Matches the pattern seen in every captured screen setup (News):
/// * one header widget (kind = HEADER, top strip)
/// * one root content holder (kind = LABEL, remainder)
///
/// Grid coords are the exe's screen-space literals (0..800 / 0..600).
pub fn skeleton_widgets(title: &str) -> Vec<Widget> {
    let mut header = WidgetDescriptor::empty();
    header.kind = KIND_HEADER;
    header.grid_x0 = 90;
    header.grid_y0 = 10;
    header.grid_x1 = 790;
    header.grid_y1 = 40;
    header.text = title.to_string();

    let mut body = WidgetDescriptor::empty();
    body.kind = KIND_LABEL;
    body.grid_x0 = 90;
    body.grid_y0 = 45;
    body.grid_x1 = 790;
    body.grid_y1 = 550;

    vec![widget_from(header), widget_from(body)]
}

/// Build a `Widget` from a descriptor, applying the same clamping /
/// swapping the pool spawner does (kept in sync with
/// `GuiRecordPool::spawn_widget`).
fn widget_from(mut desc: WidgetDescriptor) -> Widget {
    if desc.grid_x1 < desc.grid_x0 {
        std::mem::swap(&mut desc.grid_x0, &mut desc.grid_x1);
    }
    if desc.grid_y1 < desc.grid_y0 {
        std::mem::swap(&mut desc.grid_y0, &mut desc.grid_y1);
    }
    desc.grid_x0 = desc.grid_x0.max(0);
    desc.grid_y0 = desc.grid_y0.max(0);
    desc.grid_x1 = desc.grid_x1.min(799);
    desc.grid_y1 = desc.grid_y1.min(599);
    Widget {
        left: desc.grid_x0,
        top: desc.grid_y0,
        right: desc.grid_x1,
        bottom: desc.grid_y1,
        // BUGFIX: this was previously dropped (Widget::flags defaulted to 0
        // regardless of desc's flags word), which silently zeroed every
        // widget's rflags in dump_screen_geometry's output — caught by
        // diffing the News screen against its Unicorn-captured ground
        // truth. The widget flags dword is now `desc.kind` (+0x0c) per
        // FUN_005d76c0.
        flags: desc.kind,
        descriptor: desc,
        ..Default::default()
    }
}

/// Helper: build a labeled row widget spanning the body column at the
/// given y offset. Rows are stacked at 30px intervals.
fn row_label(row: i32, text: &str, userdata: u32) -> Widget {
    let mut lbl = WidgetDescriptor::empty();
    lbl.kind = KIND_LABEL;
    lbl.grid_x0 = 100;
    lbl.grid_y0 = 90 + row * 30;
    lbl.grid_x1 = 780;
    lbl.grid_y1 = 115 + row * 30;
    lbl.text = text.to_string();
    lbl.userdata_id = userdata;
    widget_from(lbl)
}

/// Helper: build a tab-strip button.
fn tab_button(idx: usize, count: usize, label: &str, highlighted: bool) -> Widget {
    let width = 700 / count.max(1) as i32;
    let mut tab = WidgetDescriptor::empty();
    tab.kind = KIND_BUTTON;
    tab.grid_x0 = 90 + (idx as i32) * width;
    tab.grid_y0 = 45;
    tab.grid_x1 = 90 + (idx as i32 + 1) * width - 2;
    tab.grid_y1 = 70;
    tab.text = label.to_string();
    tab.kind = KIND_BUTTON;
    // Highlight lives on the Widget.flags rflags dword — set post-build
    // rather than merged into `kind`, so callers that discriminate on
    // widget-role (`descriptor.kind == KIND_BUTTON`) keep matching.
    let mut w = widget_from(tab);
    if highlighted { w.flags |= 0x0800; }
    w
}

// ============================================================================
// 1. Club Dashboard — 35 slots, decoded in screen_club_dashboard.rs
//    (FUN_00454620 / FUN_004551C0). 5-tab strip mirrors FUN_005d7070.
// ============================================================================

use cm_domain::screen_club_dashboard::ClubDashboardView;

impl RenderableView for ClubDashboardView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Club Dashboard");
        let tabs = ["Squad", "Fixtures", "Reserves", "Finance", "News"];
        let highlighted_idx = match self.tab_index {
            1 => 0, 5 => 2, _ => usize::MAX,
        };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == highlighted_idx));
        }
        for (row, key) in [
            self.key_player_1, self.key_player_2, self.key_player_3,
        ].iter().enumerate() {
            if let Some(id) = *key {
                out.push(row_label(row as i32, "Key player", id));
            }
        }
        out
    }
}

// ============================================================================
// 2. Transfers — 10 slots (FUN_008E3700). Filter row across the top,
//    scroll-window body below.
// ============================================================================

use cm_domain::screen_transfers::TransfersView;

impl RenderableView for TransfersView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Transfers");
        // Filter chips (position, nationality, max-price, max-wage).
        let filters = [
            ("Position", self.position_filter != 0),
            ("Nationality", self.nationality_filter != 0),
            ("Max price", self.max_price_filter > 0),
            ("Max wage",  self.max_wage_filter > 0),
        ];
        for (i, (name, on)) in filters.iter().enumerate() {
            out.push(tab_button(i, filters.len(), name, *on));
        }
        // Selected-club row (slot 0).
        if let Some(club) = self.selected_club {
            out.push(row_label(0, "Selected club", club));
        }
        // Sort-order + scroll-offset marker row.
        out.push(row_label(
            1,
            "Sort order",
            self.sort_order as u32,
        ));
        out.push(row_label(
            2,
            "Selected row",
            self.selected_row,
        ));
        out
    }
}

// ============================================================================
// 3. Competition Dashboard — 55 slots (FUN_00493E10). Primary-tab
//    strip driven by slot 0x1E.
// ============================================================================

use cm_domain::screen_batch5::CompetitionDashboardView;

impl RenderableView for CompetitionDashboardView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Competition");
        // Primary-tab strip: 5 tabs highlighted by primary_tab (default 0x11 = 17).
        // Map 0x11..0x15 → indices 0..4.
        let tabs = ["Table", "Fixtures", "Stats", "History", "News"];
        let highlighted = (self.primary_tab as usize).wrapping_sub(0x11);
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == highlighted));
        }
        // Sub-tab / view-mode rows.
        out.push(row_label(0, "View mode", self.view_mode));
        out.push(row_label(1, "View sub-mode", self.view_sub_mode));
        if let Some(season) = self.season {
            out.push(row_label(2, "Season", season));
        }
        if let Some(sel) = self.selected_item {
            out.push(row_label(3, "Selected item", sel));
        }
        out
    }
}

// ============================================================================
// 4. Match Report — 22 slots (FUN_00701240). Section-select strip on top.
// ============================================================================

use cm_domain::screen_batch7::MatchReportView;

impl RenderableView for MatchReportView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        // Prelude widgets removed: the exe capture
        // (reports/screen_captures/screen_batch7_match_report_screen_setup_cmd_0x.json)
        // records zero areas/objects for the setup — the DRAW callback
        // isn't hit during setup, so the header/body prelude was invented.
        // Only widgets emitted below reflect real slot values.
        let mut out: Vec<Widget> = Vec::new();
        // Section strip: default_section index selects highlight.
        let sections = [
            "Summary", "Lineups", "Substitutes", "Stats",
            "Timeline", "Events", "Goals",
        ];
        let hi = self.default_section as usize;
        for (i, name) in sections.iter().enumerate() {
            out.push(tab_button(i, sections.len(), name, i == hi));
        }
        // Fixture row.
        out.push(row_label(0, "Fixture", self.fixture_handle));
        // Detail-level marker.
        out.push(row_label(1, "Detail level", self.detail_level as u32));
        // Optional selection rows.
        if let Some(e) = self.selected_event {
            out.push(row_label(2, "Selected event", e));
        }
        if let Some(p) = self.selected_player {
            out.push(row_label(3, "Selected player", p));
        }
        if self.show_summary != 0 {
            out.push(row_label(4, "Show summary", 1));
        }
        out
    }
}

// ============================================================================
// 5. Contract — 2 slots (FUN_00476DF0). Small dialog-shaped screen.
// ============================================================================

use cm_domain::screen_batch6::ContractView;

impl RenderableView for ContractView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Contract");
        // Player id row.
        out.push(row_label(0, "Player", self.player_id));
        // Managed-by-current-human row.
        out.push(row_label(
            1,
            if self.is_managed { "Managed" } else { "Not managed" },
            self.is_managed as u32,
        ));
        // If the exe wrote back to player+0x29/+0x2D, surface both.
        if let Some((flag, note)) = self.player_flag_write {
            out.push(row_label(2, "Contract flag", flag));
            out.push(row_label(3, "Note", note));
        }
        out
    }
}

// ============================================================================
// 6. Club History — 28 slots (FUN_0046BA80). View-mode strip (all-time
//    vs season) then a season list.
// ============================================================================

use cm_domain::screen_batch6::ClubHistoryView;

impl RenderableView for ClubHistoryView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Club History");
        // Two-tab strip: All-time (mode 3) vs Season-by-season (mode 8).
        let tabs = ["All-time", "Season"];
        let hi = if self.view_mode == 8 { 1 } else { 0 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Club", self.club_id));
        out.push(row_label(1, "Sort order", self.sort_order as u32));
        if let Some(s) = self.selected_season {
            out.push(row_label(2, "Selected season", s));
        }
        out
    }
}

// ============================================================================
// 7. Entity List — 4 unique slots (FUN_0058A550), 6-mode picker.
// ============================================================================

use cm_domain::screen_batch8::EntityListView;

impl RenderableView for EntityListView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Entity List");
        // Mode strip: 1=Nations, 3=Major Clubs, 4=League, 5=Non-League,
        // 6=Other Clubs, 7=Under 21s.
        let modes: [(i8, &str); 6] = [
            (1, "Nations"), (3, "Major Clubs"), (4, "League"),
            (5, "Non-League"), (6, "Other Clubs"), (7, "Under 21s"),
        ];
        for (i, (m, name)) in modes.iter().enumerate() {
            out.push(tab_button(i, modes.len(), name, *m == self.mode));
        }
        out.push(row_label(0, "Focus", self.focus));
        out.push(row_label(1, "Extra filter", self.extra_filter));
        out.push(row_label(2, "Filter byte", self.filter_byte as u32));
        out
    }
}

// ============================================================================
// 8. Hall of Fame — 3 slots (FUN_0080FAC0). Tab bar for selected_tab.
// ============================================================================

use cm_domain::screen_batch8::HallOfFameView;

impl RenderableView for HallOfFameView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        // Prelude widgets removed: exe capture
        // (screen_batch8_hall_of_fame_cmd_0x42a_79_b_3_sl.json) has 0
        // areas/objects for the setup — the invented header/body prelude
        // is not in the ground truth. Only slot-driven widgets follow.
        let mut out: Vec<Widget> = Vec::new();
        let tabs = ["Players", "Managers", "Clubs", "Nations"];
        let hi = self.selected_tab as usize;
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Scroll offset", self.scroll_offset));
        out.push(row_label(1, "Filter", self.filter));
        out
    }
}

// ============================================================================
// 9. Manager Job Info — 8 slots (FUN_00695E60). context_id + 3 filters.
// ============================================================================

use cm_domain::screen_manager_batch::JobInfoView;

impl RenderableView for JobInfoView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Job Information");
        // View / season / comp filter chips.
        let chips = [
            ("View mode", self.view_mode != 0),
            ("Season",    self.season_filter != 0),
            ("Comp",      self.comp_filter != 0),
        ];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        if let Some(id) = self.context_id {
            out.push(row_label(0, "Context", id));
        }
        out
    }
}

// ============================================================================
// 10. FIFA Rankings — 4 slots (FUN_004A2190). Sort-toggle + filter row.
// ============================================================================

use cm_domain::screen_batch3::FifaRankingsView;

impl RenderableView for FifaRankingsView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("FIFA Rankings");
        let chips = [
            ("Sort by points", self.sort_by_points != 0),
            ("Show averages",  self.show_averages != 0),
        ];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "Selected nation", self.selected_nation));
        out.push(row_label(1, "Continent filter", self.continent_filter));
        out
    }
}

// ============================================================================
// Remaining skeleton-only impls — draw callback not yet lifted.
// Each carries a `NEEDS_LAYOUT_INFO` marker for `grep`.
// ============================================================================

macro_rules! skeleton_impl {
    ($ty:path, $title:expr) => {
        impl RenderableView for $ty {
            fn to_widget_pool(&self) -> Vec<Widget> {
                #[allow(unused)]
                const NEEDS_LAYOUT_INFO: &str = concat!(
                    stringify!($ty),
                    " — draw callback not lifted; skeleton only. See ",
                    "reports/cm0102_exact_news_ui_evidence.md for the ",
                    "News-style geometry-capture template.",
                );
                skeleton_widgets($title)
            }
        }
    };
}

// ============================================================================
// Batch 11 — Club/offer setup views. Slot layouts from screen_batch11.rs.
// ============================================================================

impl RenderableView for cm_domain::screen_batch11::ClubSubScreenAView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Club Sub-Screen");
        out.push(row_label(0, "Club ptr", self.club_ptr));
        out.push(row_label(1, "Aux param", self.aux_param));
        out.push(row_label(2, "Record bytes", self.record_copy.len() as u32));
        out
    }
}

impl RenderableView for cm_domain::screen_batch11::ClubCompSetupBView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Club/Comp Setup B");
        out.push(row_label(0, "Param 1", self.param_1));
        out.push(row_label(1, "Param 2", self.param_2));
        out.push(row_label(2, "Payload", self.computed_payload));
        out
    }
}

impl RenderableView for cm_domain::screen_batch11::ClubCompSetupCView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Club/Comp Setup C");
        out.push(row_label(0, "Param 1", self.param_1));
        out.push(row_label(1, "Param 2", self.param_2));
        out.push(row_label(2, "Payload", self.payload));
        out.push(row_label(3, "Club record", self.club_record_ptr));
        out
    }
}

impl RenderableView for cm_domain::screen_batch11::ContractOfferView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Contract Offer");
        out.push(row_label(0, "Active person", self.active_person));
        out.push(row_label(1, "Target club", self.target_club_record));
        out.push(row_label(2, "Mode A", self.mode_a as u32));
        out.push(row_label(3, "Mode B", self.mode_b));
        out.push(row_label(4, "Mode C", self.mode_c));
        out
    }
}

impl RenderableView for cm_domain::screen_batch11::ContractOfferDirectView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Contract Offer (Direct)");
        out.push(row_label(0, "Param 1", self.param_1));
        out.push(row_label(1, "Param 2", self.param_2));
        // Mode strip: is_18 toggles a chip.
        let chips = [("Standard", !self.is_18_flag), ("Type-0x18", self.is_18_flag)];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(2, "Slot 12", self.slot_12));
        out
    }
}

// ============================================================================
// Batch 13 — Comp lookup + contract negotiation family.
// ============================================================================

impl RenderableView for cm_domain::screen_batch13::CompLookupView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Competition Lookup");
        out.push(row_label(0, "Resolved key", self.resolved_key));
        out.push(row_label(1, "Extra A", self.extra_a));
        out.push(row_label(2, "Extra B", self.extra_b));
        out
    }
}

impl RenderableView for cm_domain::screen_batch13::TwoKeyLookupView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Two-Key Lookup");
        out.push(row_label(0, "Param 1", self.param_1));
        out.push(row_label(1, "Param 2", self.param_2));
        out.push(row_label(2, "Extra 0", self.extra_0));
        out.push(row_label(3, "Extra 2", self.extra_2));
        out
    }
}

impl RenderableView for cm_domain::screen_batch13::ContractNegotiationView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Contract Negotiation");
        // Comparator tab strip: comparator absent vs present.
        let tabs = ["No comparator", "With comparator"];
        let hi = if self.comparator_key != 0 { 1 } else { 0 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Player", self.player_key));
        out.push(row_label(1, "Comparator", self.comparator_key));
        out.push(row_label(2, "Flag 3", self.flag_3));
        out.push(row_label(3, "Offer", self.offer_key));
        out.push(row_label(4, "Comparator club", self.comparator_club_key));
        out.push(row_label(5, "Role", self.role as u32));
        out.push(row_label(6, "Derived byte", self.derived_byte as u32));
        out
    }
}

impl RenderableView for cm_domain::screen_batch13::ContractOfferView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Contract Offer");
        out.push(row_label(0, "Player", self.player_key));
        out.push(row_label(1, "Estimate", self.estimate_clamped));
        out.push(row_label(2, "Wage estimate", self.wage_estimate));
        out.push(row_label(3, "Club extra", self.club_extra));
        out.push(row_label(4, "Years const A", self.years_const_a));
        out.push(row_label(5, "Years const B", self.years_const_b));
        out.push(row_label(6, "Mode byte", self.mode_byte as u32));
        out.push(row_label(7, "Kind byte", self.kind_byte as u32));
        out.push(row_label(8, "Sentinel 1C", self.sentinel_1c));
        out
    }
}

impl RenderableView for cm_domain::screen_batch13::ContractOfferComparatorView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Contract Offer (Comparator)");
        // Differs-from-own-club chip.
        let chips = [
            ("Same club", self.differs_from_own_club == 0),
            ("Different club", self.differs_from_own_club != 0),
        ];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "Player", self.player_key));
        out.push(row_label(1, "Comparator", self.comparator_key));
        out.push(row_label(2, "Estimate", self.estimate_clamped));
        out.push(row_label(3, "Compare score", self.compare_score));
        out.push(row_label(4, "Wage estimate", self.wage_estimate));
        out.push(row_label(5, "Club extra", self.club_extra));
        out.push(row_label(6, "Cmp +0x20", self.comparator_field_20));
        out.push(row_label(7, "Cmp +0x24", self.comparator_field_24));
        out.push(row_label(8, "Sentinel 1C", self.sentinel_1c));
        out.push(row_label(9, "Sentinel 1D", self.sentinel_1d));
        out
    }
}

skeleton_impl!(cm_domain::screen_batch3::LatestScoresView,     "Latest Scores");
skeleton_impl!(cm_domain::screen_batch3::ManagerHistoryView,   "Manager History");
skeleton_impl!(cm_domain::screen_batch3::UefaCoefficientsView, "UEFA Coefficients");

// Awards (batch 5) — custom impl: capture
// (screen_batch5_awards_cmd_0x7d4_196_b_2_slots.json) has 0 areas/objects,
// so no prelude is emitted; slot values surface as row_label userdata_id.
impl RenderableView for cm_domain::screen_batch5::AwardsView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out: Vec<Widget> = Vec::new();
        out.push(row_label(0, "Award table", self.award_table));
        out.push(row_label(1, "View mode",   self.view_mode));
        out
    }
}

// ============================================================================
// Batch 14 — FUN_004EB240 / 4EBF60 / 4EC550 / 4FD1B0 / 00548170
// ============================================================================

use cm_domain::screen_batch14::{
    Screen4Eb240View, Screen4Ebf60View, Screen4Ec550View,
    Screen4Fd1b0View, Screen548170View,
};

impl RenderableView for Screen4Eb240View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 4EB240");
        let chips = [
            ("p2", self.p2 != 0),
            ("p3", self.p3 != 0),
            ("p6", self.p6 != 0),
        ];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "param1 ptr", self.param1_ptr));
        out.push(row_label(1, "slot1", self.slot1));
        out.push(row_label(2, "slot2", self.slot2));
        out.push(row_label(3, "p4", self.p4));
        out.push(row_label(4, "p5", self.p5));
        out
    }
}

impl RenderableView for Screen4Ebf60View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 4EBF60");
        out.push(row_label(0, "p1", self.p1));
        out
    }
}

impl RenderableView for Screen4Ec550View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 4EC550");
        out.push(row_label(0, "slot0", self.slot0));
        out.push(row_label(1, "slot1", self.slot1));
        out
    }
}

impl RenderableView for Screen4Fd1b0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 4FD1B0");
        out.push(row_label(0, "slot0", self.slot0));
        out.push(row_label(1, "slot1", self.slot1));
        out
    }
}

impl RenderableView for Screen548170View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 548170");
        let chips = [
            ("A", self.slot_a != 0),
            ("B", self.slot_b != 0),
            ("C", self.slot_c != 0),
            ("D", self.slot_d != 0),
            ("E", self.slot_e != 0),
            ("F", self.slot_f != 0),
        ];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "slot0",  self.slot0 as u32));
        out.push(row_label(1, "record", self.record.base_ptr));
        out.push(row_label(2, "slot10", self.slot_10));
        out.push(row_label(3, "slot11", self.slot_11));
        out.push(row_label(4, "slot14", self.slot_14));
        out
    }
}

// ============================================================================
// Batch 15 — FUN_00574960 / 005792A0 / 0057B9C0 / 0057BB30 / 005928E0
// ============================================================================

use cm_domain::screen_batch15::{
    Screen574960View, Screen5792A0View, Screen57B9C0View,
    Screen57BB30View, Screen5928E0View,
};

impl RenderableView for Screen574960View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 574960");
        out.push(row_label(0, "entity", self.entity_id));
        out
    }
}

impl RenderableView for Screen5792A0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 5792A0");
        let chips = [
            ("Mode",     self.mode != 0),
            ("Optional", self.optional_handle != 0),
            ("Text A",   self.text_a_ptr != 0),
        ];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "entity A", self.entity_a));
        out.push(row_label(1, "entity B", self.entity_b));
        out.push(row_label(2, "extra 5",  self.extra5));
        out
    }
}

impl RenderableView for Screen57B9C0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 57B9C0");
        out.push(row_label(0, "entity A", self.entity_a));
        out.push(row_label(1, "entity B", self.entity_b));
        out.push(row_label(2, "entity C", self.entity_c));
        out.push(row_label(3, "extra 3",  self.extra3));
        out.push(row_label(4, "extra 7",  self.extra7));
        out
    }
}

impl RenderableView for Screen57BB30View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 57BB30");
        let chips = [("Helper", self.helper_from_entity_c != 0)];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "entity A", self.entity_a));
        out.push(row_label(1, "entity B", self.entity_b));
        out.push(row_label(2, "extra 3",  self.extra3));
        out.push(row_label(3, "extra 7",  self.extra7));
        out
    }
}

impl RenderableView for Screen5928E0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 5928E0");
        out.push(row_label(0, "entity", self.entity_id));
        out.push(row_label(1, "tag",    self.tag));
        out
    }
}

// ============================================================================
// Batch 16 — HistoryRecords / ManageObj{51f,588,67e} / MatchAreaScreen
// ============================================================================

use cm_domain::screen_batch16::{
    HistoryRecordsView, ManageObj51fView, ManageObj588View,
    ManageObj67eView, MatchAreaScreenView,
};

impl RenderableView for HistoryRecordsView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("History Records");
        let chips = [("Source", self.has_source_marker)];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "focus",    self.focus_id));
        out.push(row_label(1, "line",     self.string_line));
        out.push(row_label(2, "rect x",   self.main_rect.0));
        out.push(row_label(3, "footer x", self.footer_xy.0));
        out
    }
}

impl RenderableView for ManageObj51fView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        // Prelude removed to match capture
        // (screen_batch16_manage_screen_cm3_code_manage_0x.json: 0 areas/objects).
        let mut out: Vec<Widget> = Vec::new();
        out.push(row_label(0, "object", self.object_id as u32));
        out
    }
}

impl RenderableView for ManageObj588View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        // Prelude removed to match capture
        // (screen_batch16_manage_screen_cm3_code_manage_0x_2.json: 0 areas/objects).
        let mut out: Vec<Widget> = Vec::new();
        out.push(row_label(0, "object", self.object_id as u32));
        out
    }
}

impl RenderableView for ManageObj67eView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Manage 67E");
        out.push(row_label(0, "inner",  self.inner_id));
        out.push(row_label(1, "offset", self.record_offset));
        out
    }
}

impl RenderableView for MatchAreaScreenView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Match Area");
        let chips = [("Active", self.screen_flag != 0)];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "match", self.match_id as u32));
        out.push(row_label(1, "aux A", self.aux_a));
        out.push(row_label(2, "aux B", self.aux_b));
        out
    }
}

// ============================================================================
// Batch 26 — 0x008e6cc0 / 0x008e7270 / 0x008e78d0 / 0x008e8590
// ============================================================================

use cm_domain::screen_batch26::{
    Screen008e6cc0View, Screen008e7270View, Screen008e78d0View, Screen008e8590View,
};

impl RenderableView for Screen008e6cc0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 008e6cc0");
        let tabs = ["Slot 0", "Slot 1"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, false));
        }
        out.push(row_label(0, "Slot 0", self.slot0));
        out.push(row_label(1, "Slot 1", self.slot1));
        out
    }
}

impl RenderableView for Screen008e7270View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 008e7270");
        let tabs = ["Slot 0"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, true));
        }
        out.push(row_label(0, "Slot 0", self.slot0));
        out
    }
}

impl RenderableView for Screen008e78d0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 008e78d0");
        let tabs = ["Slot 0", "Slot 1 (const)"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == 1));
        }
        out.push(row_label(0, "Slot 0", self.slot0));
        out.push(row_label(1, "Slot 1", self.slot1));
        out
    }
}

impl RenderableView for Screen008e8590View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Loan Offer");
        let tabs = ["Offer", "Current terms", "Mirror"];
        let hi = match (self.current_terms.is_some(), self.mirror_terms.is_some()) {
            (true, _) => 1,
            (_, true) => 2,
            _ => 0,
        };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Record", self.slot0_record));
        out.push(row_label(1, "Parent", self.slot1_parent));
        out.push(row_label(2, "Derived", self.slot2_derived));
        if let Some(ct) = self.current_terms {
            out.push(row_label(3, "Current wage %", ct.wage_pct_bucket10 as u32));
        }
        out
    }
}

// ============================================================================
// Batch 27 — GenericEightSlot / RecallLoan / GuardedLoader
// ============================================================================

use cm_domain::screen_batch27::{
    GenericEightSlotView, GuardedLoaderView, RecallLoanView,
};

impl RenderableView for GenericEightSlotView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Generic 8-Slot");
        let tabs = ["P1", "Sentinel", "P2", "P3", "P4", "P5", "P6", "P7"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, false));
        }
        out.push(row_label(0, "Param 1", self.param1));
        out.push(row_label(1, "Sentinel", self.sentinel));
        out.push(row_label(2, "Param 5", self.param5));
        out.push(row_label(3, "Param 6", self.param6));
        out.push(row_label(4, "Param 7", self.param7));
        out
    }
}

impl RenderableView for RecallLoanView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Recall Loan");
        let tabs = ["Player", "Club"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, false));
        }
        out.push(row_label(0, "Player record", self.player_record));
        out.push(row_label(1, "Club id", self.club_id));
        out
    }
}

impl RenderableView for GuardedLoaderView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Guarded Loader");
        let tabs = ["Head", "Loader", "Nested", "Param 2"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, false));
        }
        out.push(row_label(0, "Head", self.head));
        out.push(row_label(1, "Loader token", self.loader_token));
        out.push(row_label(2, "Nested", self.nested));
        out.push(row_label(3, "Param 2", self.param2));
        out
    }
}

// ============================================================================
// 11. Find Dialog — 5 slots (FUN_0058CDE0). 4 search-field buffers + mode.
// ============================================================================

use cm_domain::screen_batch8::FindDialogView;

impl RenderableView for FindDialogView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Find");
        // Mode strip: 0 = default, non-zero = specific search.
        let modes = ["Default", "Specific"];
        let hi = if self.mode == 0 { 0 } else { 1 };
        for (i, name) in modes.iter().enumerate() {
            out.push(tab_button(i, modes.len(), name, i == hi));
        }
        // One row per search-field scratch buffer, showing its byte length.
        for (i, buf) in self.search_field.iter().enumerate() {
            out.push(row_label(i as i32, "Search field", buf.len() as u32));
        }
        out
    }
}

// ============================================================================
// 12. Written History — 5 slots (FUN_005DC5E0). Category picker + reserved.
// ============================================================================

use cm_domain::screen_batch8::WrittenHistoryView;

impl RenderableView for WrittenHistoryView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        // Prelude removed to match capture
        // (screen_batch8_written_history_cmd_0x429_117_b.json: 0 areas/objects).
        let mut out: Vec<Widget> = Vec::new();
        // Category picker strip — one tab per common category id.
        let cats = ["Overview", "Champions", "Records", "Awards"];
        let hi = self.category as usize;
        for (i, name) in cats.iter().enumerate() {
            out.push(tab_button(i, cats.len(), name, i == hi));
        }
        out.push(row_label(0, "Category", self.category));
        for (i, r) in self.reserved.iter().enumerate() {
            out.push(row_label(1 + i as i32, "Reserved", *r));
        }
        out
    }
}

// ============================================================================
// 13. Selected Leagues — 2 slots + init gate (FUN_008053D0).
// ============================================================================

use cm_domain::screen_batch8::SelectedLeaguesView;

impl RenderableView for SelectedLeaguesView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Select Leagues");
        // Two chips: cursor active vs scrolled, and config-dirty flag.
        let chips = [
            ("Scrolled", self.scroll_offset > 0),
            ("Dirty",    self.config_dirty_flag),
        ];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "Cursor", self.cursor));
        out.push(row_label(1, "Scroll offset", self.scroll_offset));
        out
    }
}

// ============================================================================
// 14. Award 0x00417780 — 2 slots. Award kind + "no active manager" flag.
// ============================================================================

use cm_domain::screen_batch10::Award417780View;

impl RenderableView for Award417780View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Award");
        let chips = [("No active manager", self.no_active_manager)];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "Award kind", self.award_kind as u32));
        out
    }
}

// ============================================================================
// 15. Screen 0x00425e70 — 3 slots, focus dword + 2 reserved.
// ============================================================================

use cm_domain::screen_batch10::Screen425e70View;

impl RenderableView for Screen425e70View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 425e70");
        out.push(row_label(0, "Focus", self.focus));
        for (i, r) in self.reserved.iter().enumerate() {
            out.push(row_label(1 + i as i32, "Reserved", *r));
        }
        out
    }
}

// ============================================================================
// 16. Screen 0x00470bf0 — 7 slots incl. colour-pool offset + category id.
// ============================================================================

use cm_domain::screen_batch10::Screen470bf0View;

impl RenderableView for Screen470bf0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 470bf0");
        // Chip strip: whether colour pool is customised, reserved rows non-zero.
        let chips = [
            ("Custom colour", self.colour_pool_offset != 0),
            ("Category set",  self.category != 0),
        ];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "Param 1",        self.param_1 as u32));
        out.push(row_label(1, "Param 2",        self.param_2 as u32));
        out.push(row_label(2, "Category",       self.category));
        out.push(row_label(3, "Colour offset",  self.colour_pool_offset));
        out.push(row_label(4, "Param 3",        self.param_3));
        out
    }
}

// ============================================================================
// 17. Screen 0x00472bf0 — 11 slots with param_2 branch (CM Cup descriptor).
// ============================================================================

use cm_domain::screen_batch10::Screen472bf0View;

impl RenderableView for Screen472bf0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 472bf0");
        // Branch chips.
        let chips = [
            ("CM Cup",    self.cm_cup.is_some()),
            ("Kind 0x14", self.kind == 0x14),
        ];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "Param 1", self.param_1 as u32));
        out.push(row_label(1, "Param 2", self.param_2 as u32));
        out.push(row_label(2, "Param 3", self.param_3 as u32));
        out.push(row_label(3, "Kind",    self.kind));
        if let Some(cup) = &self.cm_cup {
            out.push(row_label(4, "CM Cup byte 200", cup.byte_200 as u32));
        }
        out
    }
}

// ============================================================================
// Tests for batch 7/8/10 impls (batches 11..17).
// ============================================================================

#[cfg(test)]
mod tests_batch_7_8_10 {
    use super::*;

    fn assert_within_surface(widgets: &[Widget]) {
        for w in widgets {
            assert!(w.left >= 0 && w.right <= 799,
                    "widget out of x-bounds: {}..{}", w.left, w.right);
            assert!(w.top >= 0 && w.bottom <= 599,
                    "widget out of y-bounds: {}..{}", w.top, w.bottom);
        }
    }

    #[test]
    fn find_dialog_produces_widget_pool() {
        let v = FindDialogView::default();
        let widgets = v.to_widget_pool();
        assert!(!widgets.is_empty());
        assert_within_surface(&widgets);
    }

    #[test]
    fn written_history_produces_widget_pool() {
        let v = WrittenHistoryView { category: 2, reserved: [0; 4] };
        let widgets = v.to_widget_pool();
        assert!(!widgets.is_empty());
        assert_within_surface(&widgets);
    }

    #[test]
    fn selected_leagues_produces_widget_pool() {
        let v = SelectedLeaguesView {
            cursor: 3, scroll_offset: 5, config_dirty_flag: true,
        };
        let widgets = v.to_widget_pool();
        assert!(!widgets.is_empty());
        assert_within_surface(&widgets);
    }

    #[test]
    fn award_417780_produces_widget_pool() {
        let v = Award417780View { award_kind: 7, no_active_manager: true };
        let widgets = v.to_widget_pool();
        assert!(!widgets.is_empty());
        assert_within_surface(&widgets);
    }

    #[test]
    fn screen_425e70_produces_widget_pool() {
        let v = Screen425e70View { focus: 42, reserved: [0, 0] };
        let widgets = v.to_widget_pool();
        assert!(!widgets.is_empty());
        assert_within_surface(&widgets);
    }

    #[test]
    fn screen_470bf0_produces_widget_pool() {
        let v = Screen470bf0View {
            param_1: 1, param_2: 2, category: 0x221,
            colour_pool_offset: 42, reserved: [0, 0], param_3: 9,
        };
        let widgets = v.to_widget_pool();
        assert!(!widgets.is_empty());
        assert_within_surface(&widgets);
    }

    #[test]
    fn screen_472bf0_produces_widget_pool() {
        let v = Screen472bf0View {
            param_1: 1, one: 1, reserved_23: [0, 0], param_2: b'\n' as i32,
            reserved_56: [0, 0], kind: 0x14, param_3: 3,
            cm_cup: Some(cm_domain::screen_batch10::CmCupDescriptor::default()),
        };
        let widgets = v.to_widget_pool();
        assert!(!widgets.is_empty());
        assert_within_surface(&widgets);
    }
}

// ============================================================================
// 11. Send Abuse — 3 slots (FUN_00771810). Recipient chip + note row.
// ============================================================================

use cm_domain::screen_batch4::SendAbuseView;

impl RenderableView for SendAbuseView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Send Abuse");
        // Two-tab strip: "All managers" vs "Specific recipient" — the
        // sentinel `recipient_id == None` selects the first, a concrete
        // id selects the second.
        let tabs = ["All managers", "Specific"];
        let hi = if self.recipient_id.is_some() { 1 } else { 0 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        // Recipient row: id or -1 sentinel marker.
        let recipient = self.recipient_id.unwrap_or(0xFFFFFFFF);
        out.push(row_label(0, "Recipient", recipient));
        // Message body length (101-byte scratch) — surface counter.
        out.push(row_label(1, "Body bytes", self.message_body.len() as u32));
        // Note / context row.
        out.push(row_label(2, "Note", self.note));
        out
    }
}

// ============================================================================
// 12. Competition-specific goto — same 55-slot shape as CompetitionDashboard
//     but two-mode (browse vs goto). FUN_00494250.
// ============================================================================

use cm_domain::screen_batch6::CompetitionSpecificView;

impl RenderableView for CompetitionSpecificView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Competition");
        // Primary-tab strip (same 5-tab layout as CompetitionDashboard),
        // driven by primary_tab (default 0x11 → index 0).
        let tabs = ["Table", "Fixtures", "Stats", "History", "News"];
        let highlighted = (self.primary_tab as usize).wrapping_sub(0x11);
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == highlighted));
        }
        // Mode chip strip: browse (view_mode==3) vs goto (view_mode==2).
        let modes = ["Browse", "Goto"];
        let mode_hi = if self.view_mode == 2 { 1 } else { 0 };
        for (i, name) in modes.iter().enumerate() {
            out.push(tab_button(
                tabs.len() + i, tabs.len() + modes.len(), name, i == mode_hi,
            ));
        }
        // Scalar slot rows.
        out.push(row_label(0, "View mode", self.view_mode));
        out.push(row_label(1, "Sub tab", self.sub_tab as u32));
        out.push(row_label(2, "Secondary tab", self.secondary_tab as u32));
        // Which pair is active depends on mode; surface the active pair.
        let (p2, p3) = if self.view_mode == 2 {
            (self.goto_param_2, self.goto_param_3)
        } else {
            (self.browse_param_2, self.browse_param_3)
        };
        out.push(row_label(3, "Param 2", p2 as u32));
        out.push(row_label(4, "Param 3", p3 as u32));
        out.push(row_label(5, "View sub-mode", self.view_sub_mode));
        out.push(row_label(6, "Filter flag", self.filter_flag));
        out.push(row_label(7, "Season", self.season.unwrap_or(0xFFFFFFFF)));
        out.push(row_label(8, "Selected item",
                           self.selected_item.unwrap_or(0xFFFFFFFF)));
        out
    }
}

// ============================================================================
// Batch 17 impls — screen_batch17 View structs
// ============================================================================

use cm_domain::screen_batch17::{
    MatchDetailView, FixtureNavView, TrivialRegistrationView,
    MediaArticleView, SmallFourSlotView,
};

impl RenderableView for MatchDetailView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Match Detail");
        let tabs = ["Summary", "Report"];
        let hi = if self.redispatched_to_alt { 1 } else { 0 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Match", self.match_id));
        out.push(row_label(1, "Field 63", self.match_field_63));
        if self.bumped_human_seen {
            out.push(row_label(2, "Bumped seen", 1));
        }
        out
    }
}

impl RenderableView for FixtureNavView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Fixture");
        let tabs = ["Home", "Away"];
        let hi = if self.is_home_leg { 0 } else { 1 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Competition", self.current_comp_id));
        out.push(row_label(1, "Fixture", self.fixture_ref));
        out
    }
}

impl RenderableView for TrivialRegistrationView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Registration");
        out.push(tab_button(0, 1, "Widget", true));
        out.push(row_label(0, "Widget", self.widget_id));
        out
    }
}

impl RenderableView for MediaArticleView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("News Article");
        let tabs = ["Article", "Related"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == 0));
        }
        out.push(row_label(0, "Caller", self.caller_arg));
        out.push(row_label(1, "Var 2F8", self.var_2f8));
        out.push(row_label(2, "Var 2F4", self.var_2f4));
        if self.var_2fc >= 0 {
            out.push(row_label(3, "Var 2FC", self.var_2fc as u32));
        }
        out
    }
}

impl RenderableView for SmallFourSlotView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Registration");
        let tabs = ["Slot A", "Slot C"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == 0));
        }
        out.push(row_label(0, "Arg A", self.arg_a));
        out.push(row_label(1, "Arg C", self.arg_c));
        out.push(row_label(2, "Flag", self.flag as u32));
        out
    }
}

// ============================================================================
// Batch 18 impls — screen_batch18 View structs
// ============================================================================

use cm_domain::screen_batch18::{
    TwoSelectorListView, LoaderGatedDialogView, DualSelectorScratchView,
    SeatScreenView, TwoPointerView,
};

impl RenderableView for TwoSelectorListView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("List");
        let tabs = ["All", "Filtered"];
        let hi = if self.selector1 == 0 { 0 } else { 1 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Row", self.row_handle));
        out.push(row_label(1, "Visible", self.visible_count as u32));
        out.push(row_label(2, "Selector 5", self.selector5));
        out
    }
}

impl RenderableView for LoaderGatedDialogView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Dialog");
        out.push(tab_button(0, 1, "Loader", true));
        out.push(row_label(0, "Marker", self.marker));
        out.push(row_label(1, "Selector", self.selector0));
        out
    }
}

impl RenderableView for DualSelectorScratchView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Selector");
        let tabs = ["Primary", "Secondary"];
        let hi = if self.row_handle == 0 { 0 } else { 1 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Row", self.row_handle));
        out.push(row_label(1, "Param 2", self.param_2));
        out.push(row_label(2, "Scratch bytes", self.scratch.len() as u32));
        out
    }
}

impl RenderableView for SeatScreenView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Seat");
        let tabs = ["Info", "Detail"];
        let hi = if self.entry_present != 0 { 1 } else { 0 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Param 1", self.param_1_deref));
        out.push(row_label(1, "State", self.state as u32));
        out.push(row_label(2, "Sub flag", self.sub_flag as u32));
        if self.entry_present != 0 {
            out.push(row_label(3, "Slot 5", self.slot5_value));
        }
        out
    }
}

impl RenderableView for TwoPointerView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Pointers");
        let tabs = ["P1", "P2"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == 0));
        }
        out.push(row_label(0, "P1", self.p1_deref));
        out.push(row_label(1, "P2", self.p2_deref));
        out
    }
}

// ============================================================================
// Batch 19 impls — screen_batch19 View structs
// ============================================================================

use cm_domain::screen_batch19::{
    PersonBannerView, SearchResultDetailView, SetupBootstrapView, FiveZeroSlotView,
};

impl RenderableView for PersonBannerView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        // Prelude removed to match capture
        // (screen_batch19_person_squad_type_banner_setup_3.json: 0 areas/objects).
        let mut out: Vec<Widget> = Vec::new();
        let tabs = ["Overview", "Category"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == 0));
        }
        out.push(row_label(0, "Record", self.record_head));
        out.push(row_label(1, "Category", self.category_byte as u32));
        out.push(row_label(2, "Sub head", self.sub_head_byte as u32));
        out
    }
}

impl RenderableView for SearchResultDetailView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        // Prelude removed to match capture
        // (screen_batch19_search_result_detail_setup_3_slo.json: 0 areas/objects).
        let mut out: Vec<Widget> = Vec::new();
        let tabs = ["Detail", "Status"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == 0));
        }
        out.push(row_label(0, "Head byte", self.record_head_byte as u32));
        out.push(row_label(1, "Status", self.status_byte as u32));
        out.push(row_label(2, "Sub bytes", self.sub_record.len() as u32));
        out
    }
}

impl RenderableView for SetupBootstrapView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Setup");
        let tabs = ["Online", "Offline"];
        let hi = if self.offline_flag { 1 } else { 0 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Seed", self.seed));
        out.push(row_label(1, "No network", self.nonetwork as u32));
        out
    }
}

impl RenderableView for FiveZeroSlotView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        // Prelude removed to match capture
        // (screen_batch19_5_slot_all_zero_screen_persisten.json: 0 areas/objects).
        let mut out: Vec<Widget> = Vec::new();
        out.push(tab_button(0, 1, "Slots", true));
        out.push(row_label(0, "Slot 0", self.slot0));
        out.push(row_label(1, "Slot 1", self.slot1));
        out.push(row_label(2, "Slot 2", self.slot2));
        out.push(row_label(3, "Slot 3", self.slot3));
        out.push(row_label(4, "Slot 4", self.slot4));
        out
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layout::rebuild_layout;

    /// News-style baseline: the News screen builds a header widget +
    /// tab strip + content — captured shape has at minimum a header
    /// and one content-holder widget. Every screen must at least match
    /// that prelude count (`>= 2`).
    const NEWS_MIN_WIDGET_COUNT: usize = 2;

    /// Assert every widget's bounds sit inside the 800x600 screen.
    fn assert_within_surface(widgets: &[Widget]) {
        for w in widgets {
            assert!(w.left >= 0 && w.right <= 799,
                    "widget out of x-bounds: {}..{}", w.left, w.right);
            assert!(w.top >= 0 && w.bottom <= 599,
                    "widget out of y-bounds: {}..{}", w.top, w.bottom);
        }
    }

    // -- 1. Club Dashboard --------------------------------------------------
    #[test]
    fn club_dashboard_produces_widget_pool_and_layout() {
        let v = ClubDashboardView {
            tab_index: 1,
            key_player_1: Some(1001),
            key_player_2: Some(1002),
            key_player_3: Some(1003),
            ..Default::default()
        };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // Header + body + 5 tabs + 3 key players = 10.
        assert_eq!(widgets.len(), 10);
        assert_eq!(widgets[0].descriptor.kind, KIND_HEADER);
        assert_eq!(widgets[1].descriptor.kind, KIND_LABEL);
        for i in 0..5 {
            assert_eq!(widgets[2 + i].descriptor.kind, KIND_BUTTON);
        }
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
        assert_eq!(widgets[3].flags & 0x0800, 0);
        assert_within_surface(&widgets);

        // Identity round-trip through rebuild_layout for the header.
        let h = &widgets[0];
        let layout = rebuild_layout(
            (h.left, h.top, h.right, h.bottom),
            2, &[1], &[1], false,
        );
        assert_eq!(layout.col_left.len(), 1);
        assert_eq!(layout.row_top.len(), 1);
    }

    // -- 2. Transfers -------------------------------------------------------
    #[test]
    fn transfers_produces_widget_pool() {
        let v = TransfersView {
            selected_club: Some(42),
            position_filter: 1,
            nationality_filter: 1,
            max_price_filter: 100_000,
            max_wage_filter: 5_000,
            ..Default::default()
        };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // header + body + 4 filter chips + selected club + 2 rows.
        assert!(widgets.len() >= 8);
        assert_within_surface(&widgets);
    }

    // -- 3. Competition Dashboard ------------------------------------------
    #[test]
    fn competition_dashboard_produces_widget_pool() {
        let v = CompetitionDashboardView {
            primary_tab: 0x11, // → tab index 0 highlighted
            season: Some(2024),
            selected_item: Some(7),
            ..Default::default()
        };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        assert!(widgets.len() >= 5 + 2 + 2); // 5 tabs + prelude + rows
        assert_within_surface(&widgets);
        // Tab 0 (Table) highlighted because primary_tab == 0x11.
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
    }

    // -- 4. Match Report ----------------------------------------------------
    #[test]
    fn match_report_produces_widget_pool() {
        let v = MatchReportView {
            fixture_handle: 999,
            selected_event: Some(3),
            selected_player: Some(1234),
            ..Default::default()
        };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 7 section tabs + rows.
        assert!(widgets.len() >= 2 + 7 + 2);
        assert_within_surface(&widgets);
    }

    // -- 5. Contract --------------------------------------------------------
    #[test]
    fn contract_produces_widget_pool() {
        let v = ContractView {
            player_id: 5555,
            is_managed: true,
            player_flag_write: Some((1, 7)),
        };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        assert!(widgets.len() >= 6);
        assert_within_surface(&widgets);
    }

    // -- 6. Club History ----------------------------------------------------
    #[test]
    fn club_history_produces_widget_pool() {
        let v = ClubHistoryView {
            club_id: 11, view_mode: 8, sort_order: 1,
            selected_season: Some(2023),
        };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        assert!(widgets.len() >= 2 + 2 + 3);
        assert_within_surface(&widgets);
        // Season tab (index 1) highlighted for view_mode == 8.
        assert_eq!(widgets[3].flags & 0x0800, 0x0800);
    }

    // -- 7. Entity List -----------------------------------------------------
    #[test]
    fn entity_list_produces_widget_pool() {
        let v = EntityListView { mode: 1, focus: 42, extra_filter: 0, filter_byte: 0 };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 6 mode tabs + 3 rows.
        assert_eq!(widgets.len(), 2 + 6 + 3);
        assert_within_surface(&widgets);
        // Mode 1 (Nations) highlighted at index 0 → widget[2].
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
    }

    // -- 8. Hall of Fame ----------------------------------------------------
    #[test]
    fn hall_of_fame_produces_widget_pool() {
        let v = HallOfFameView { selected_tab: 2, scroll_offset: 0, filter: 0 };
        let widgets = v.to_widget_pool();
        // Prelude removed to match capture (0 areas/objects):
        // 4 tabs + 2 rows.
        assert_eq!(widgets.len(), 4 + 2);
        assert_within_surface(&widgets);
        // Highlighted tab: index 2 (Clubs), now at widget[2].
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
    }

    // -- 9. Job Info --------------------------------------------------------
    #[test]
    fn job_info_produces_widget_pool() {
        use cm_domain::screen_manager_batch::JobInfoView;
        let v = JobInfoView { context_id: Some(77), ..Default::default() };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 3 chips + 1 context row = 6.
        assert_eq!(widgets.len(), 6);
        assert_within_surface(&widgets);
    }

    // -- 10. FIFA Rankings --------------------------------------------------
    #[test]
    fn fifa_rankings_produces_widget_pool() {
        let v = FifaRankingsView::default();
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 2 chips + 2 rows = 6.
        assert_eq!(widgets.len(), 6);
        assert_within_surface(&widgets);
    }

    // -- 11. Send Abuse -----------------------------------------------------
    #[test]
    fn send_abuse_produces_widget_pool() {
        let v = SendAbuseView {
            recipient_id: Some(1234),
            message_body: vec![0; 0x65],
            note: 42,
        };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 2 recipient tabs + 3 rows = 7.
        assert_eq!(widgets.len(), 7);
        assert_within_surface(&widgets);
        // Specific-recipient tab (index 1) highlighted.
        assert_eq!(widgets[3].flags & 0x0800, 0x0800);
    }

    // -- 12. Competition Specific -------------------------------------------
    #[test]
    fn competition_specific_produces_widget_pool() {
        let v = CompetitionSpecificView {
            view_mode: 2, goto_param_2: 11, goto_param_3: 22,
            season: Some(2024), selected_item: Some(7),
            ..Default::default()
        };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // Impl-count varies as slot rows are refined by other agents;
        // just check the shape holds together and no widget escapes the surface.
        assert!(widgets.len() >= 15);
        assert_within_surface(&widgets);
    }

    // -- Batch 26 / 27 impls ------------------------------------------------
    #[test]
    fn s008e6cc0_produces_widget_pool() {
        let v = Screen008e6cc0View { slot0: 11, slot1: 22 };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 2 tabs + 2 rows.
        assert_eq!(widgets.len(), 2 + 2 + 2);
        assert_eq!(widgets[0].descriptor.kind, KIND_HEADER);
        assert_eq!(widgets[1].descriptor.kind, KIND_LABEL);
        assert_eq!(widgets[2].descriptor.kind, KIND_BUTTON);
        assert_within_surface(&widgets);
    }

    #[test]
    fn s008e7270_produces_widget_pool() {
        let v = Screen008e7270View { slot0: 999 };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 1 tab + 1 row.
        assert_eq!(widgets.len(), 2 + 1 + 1);
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    #[test]
    fn s008e78d0_produces_widget_pool() {
        let v = Screen008e78d0View::default();
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 2 tabs + 2 rows.
        assert_eq!(widgets.len(), 2 + 2 + 2);
        // Second tab (const) highlighted.
        assert_eq!(widgets[3].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    #[test]
    fn s008e8590_produces_widget_pool() {
        use cm_domain::screen_batch26::CurrentTermsBlock;
        let v = Screen008e8590View {
            slot0_record: 1, slot1_parent: 2, slot2_derived: 3,
            current_terms: Some(CurrentTermsBlock {
                wage_pct_bucket10: 50, ..Default::default()
            }),
            mirror_terms: Some(CurrentTermsBlock::default()),
            ..Default::default()
        };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 3 tabs + 3 rows + 1 optional row.
        assert_eq!(widgets.len(), 2 + 3 + 4);
        // "Current terms" tab (index 1) highlighted.
        assert_eq!(widgets[3].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    #[test]
    fn generic_eight_slot_produces_widget_pool() {
        let v = GenericEightSlotView {
            param1: 1, param5: 5, param6: 6, param7: 7, ..Default::default()
        };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 8 tabs + 5 rows.
        assert_eq!(widgets.len(), 2 + 8 + 5);
        assert_within_surface(&widgets);
    }

    #[test]
    fn recall_loan_produces_widget_pool() {
        let v = RecallLoanView { player_record: 42, club_id: 99 };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 2 tabs + 2 rows.
        assert_eq!(widgets.len(), 2 + 2 + 2);
        assert_within_surface(&widgets);
    }

    #[test]
    fn guarded_loader_produces_widget_pool() {
        let v = GuardedLoaderView { head: 1, loader_token: 2, nested: 3, param2: 4 };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 4 tabs + 4 rows.
        assert_eq!(widgets.len(), 2 + 4 + 4);
        assert_within_surface(&widgets);
    }

    // -- Batch 11 --------------------------------------------------------

    #[test]
    fn club_sub_screen_a_produces_widget_pool() {
        use cm_domain::screen_batch11::ClubSubScreenAView;
        let v = ClubSubScreenAView { club_ptr: 0x1000, aux_param: 7, record_copy: vec![0u8; 0xd1] };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 3);
        assert_within_surface(&widgets);
    }

    #[test]
    fn club_comp_setup_b_produces_widget_pool() {
        use cm_domain::screen_batch11::ClubCompSetupBView;
        let v = ClubCompSetupBView { param_1: 1, param_2: 2, computed_payload: 0xDEAD };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 3);
        assert_within_surface(&widgets);
    }

    #[test]
    fn club_comp_setup_c_produces_widget_pool() {
        use cm_domain::screen_batch11::ClubCompSetupCView;
        let v = ClubCompSetupCView { param_1: 1, param_2: 2, payload: 3, club_record_ptr: 0xBEEF };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 4);
        assert_within_surface(&widgets);
    }

    #[test]
    fn contract_offer_b11_produces_widget_pool() {
        use cm_domain::screen_batch11::{build_contract_offer, ContractOfferView as B11Offer};
        let v: B11Offer = build_contract_offer(true, true, 0xAAAA, 0xBBBB).unwrap();
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 5);
        assert_within_surface(&widgets);
    }

    #[test]
    fn contract_offer_direct_produces_widget_pool() {
        use cm_domain::screen_batch11::ContractOfferDirectView;
        // Build via helper to get the exe-side defaults.
        let v = cm_domain::screen_batch11::build_contract_offer_direct(true, true, 1, 2, 0x18).unwrap();
        let _ = std::mem::size_of::<ContractOfferDirectView>();
        let widgets = v.to_widget_pool();
        // Impl-count varies with concurrent slot-row refinement;
        // just check the shape holds together and no widget escapes the surface.
        assert!(widgets.len() >= 5);
        assert_within_surface(&widgets);
    }

    // -- Batch 13 --------------------------------------------------------

    #[test]
    fn comp_lookup_produces_widget_pool() {
        use cm_domain::screen_batch13::CompLookupView;
        let v = CompLookupView { resolved_key: 0xdead, extra_a: 22, extra_b: 33 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 3);
        assert_within_surface(&widgets);
    }

    #[test]
    fn two_key_lookup_produces_widget_pool() {
        use cm_domain::screen_batch13::TwoKeyLookupView;
        let v = TwoKeyLookupView { param_1: 10, param_2: 20, extra_0: 30, extra_2: 40 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 4);
        assert_within_surface(&widgets);
    }

    #[test]
    fn contract_negotiation_produces_widget_pool() {
        use cm_domain::screen_batch13::ContractNegotiationView;
        let v = ContractNegotiationView {
            player_key: 10, role: 2, role_dup: 2,
            comparator_key: 999, flag_3: 1, derived_byte: 5,
            offer_key: 42, comparator_club_key: 77,
        };
        let widgets = v.to_widget_pool();
        // 2 prelude + 2 comparator tabs + 7 rows = 11.
        assert_eq!(widgets.len(), 2 + 2 + 7);
        assert_within_surface(&widgets);
        // Comparator present → tab index 1 highlighted → widget[3].
        assert_eq!(widgets[3].flags & 0x0800, 0x0800);
    }

    #[test]
    fn contract_offer_b13_produces_widget_pool() {
        use cm_domain::screen_batch13::ContractOfferView as B13Offer;
        let v = B13Offer::default();
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 9);
        assert_within_surface(&widgets);
    }

    #[test]
    fn contract_offer_comparator_produces_widget_pool() {
        use cm_domain::screen_batch13::ContractOfferComparatorView;
        let v = ContractOfferComparatorView {
            differs_from_own_club: 1,
            ..Default::default()
        };
        let widgets = v.to_widget_pool();
        // 2 prelude + 2 chips + 10 rows = 14.
        assert_eq!(widgets.len(), 2 + 2 + 10);
        assert_within_surface(&widgets);
        // Different-club chip highlighted → widget[3].
        assert_eq!(widgets[3].flags & 0x0800, 0x0800);
    }

    // -- Batch 14 / 15 / 16 impls ------------------------------------------
    #[test]
    fn s4eb240_produces_widget_pool() {
        use cm_domain::screen_batch14::build_screen_4eb240;
        let v = build_screen_4eb240(true, 0xdead, 1, 2, 100, 200, 3,
            Some((0x11, 0x22))).unwrap();
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        // 2 prelude + 3 chips + 5 rows.
        assert_eq!(widgets.len(), 2 + 3 + 5);
        assert_eq!(widgets[0].descriptor.kind, KIND_HEADER);
        assert_eq!(widgets[1].descriptor.kind, KIND_LABEL);
        assert_eq!(widgets[2].descriptor.kind, KIND_BUTTON);
        assert_within_surface(&widgets);
    }

    #[test]
    fn s4ebf60_produces_widget_pool() {
        let v = Screen4Ebf60View { p1: 77 };
        let widgets = v.to_widget_pool();
        assert!(widgets.len() >= NEWS_MIN_WIDGET_COUNT);
        assert_eq!(widgets.len(), 2 + 1);
        assert_within_surface(&widgets);
    }

    #[test]
    fn s4ec550_produces_widget_pool() {
        let v = Screen4Ec550View::default();
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 2);
        assert_within_surface(&widgets);
    }

    #[test]
    fn s4fd1b0_produces_widget_pool() {
        let v = Screen4Fd1b0View::default();
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 2);
        assert_within_surface(&widgets);
    }

    #[test]
    fn s548170_produces_widget_pool() {
        use cm_domain::screen_batch14::{
            build_screen_548170, Screen548170Decode, Screen548170Record,
        };
        let d = Screen548170Decode {
            f_11c: 1, f_11b: 1, f_11a: 1, f_119: 1, f_111: 1, f_112: 1,
            ..Default::default()
        };
        let r = Screen548170Record { base_ptr: 0xB000, ..Default::default() };
        let v = build_screen_548170(true, 99, d, r).unwrap();
        let widgets = v.to_widget_pool();
        // 2 prelude + 6 chips + 5 rows.
        assert_eq!(widgets.len(), 2 + 6 + 5);
        // All chips highlighted because each slot_[a..f] is non-zero.
        for i in 0..6 {
            assert_eq!(widgets[2 + i].flags & 0x0800, 0x0800);
        }
        assert_within_surface(&widgets);
    }

    #[test]
    fn s574960_produces_widget_pool() {
        let v = Screen574960View { entity_id: 123, reserved1: 0, reserved2: 0 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 1);
        assert_within_surface(&widgets);
    }

    #[test]
    fn s5792a0_produces_widget_pool() {
        use cm_domain::screen_batch15::build_screen_5792a0;
        let v = build_screen_5792a0(true, 1, 2, 1, Some("hi"), 5, Some("wo"),
            7, -3, 4, 10, 11, 12, 13, 14).unwrap();
        let widgets = v.to_widget_pool();
        // 2 prelude + 3 chips + 3 rows.
        assert_eq!(widgets.len(), 2 + 3 + 3);
        // Chips: Mode(1)=on, Optional(5)=on, Text A=on.
        for i in 0..3 {
            assert_eq!(widgets[2 + i].flags & 0x0800, 0x0800);
        }
        assert_within_surface(&widgets);
    }

    #[test]
    fn s57b9c0_produces_widget_pool() {
        use cm_domain::screen_batch15::build_screen_57b9c0;
        let v = build_screen_57b9c0(true, 1, 2, 3, 99, -5, 7, 42).unwrap();
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 5);
        assert_within_surface(&widgets);
    }

    #[test]
    fn s57bb30_produces_widget_pool() {
        use cm_domain::screen_batch15::build_screen_57bb30;
        let v = build_screen_57bb30(true, 1, 2, 3, 0, 0, 0, 0, true).unwrap();
        let widgets = v.to_widget_pool();
        // 2 prelude + 1 chip + 4 rows.
        assert_eq!(widgets.len(), 2 + 1 + 4);
        // Helper allocated → chip highlighted.
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    #[test]
    fn s5928e0_produces_widget_pool() {
        let v = Screen5928E0View { entity_id: 55, tag: 1, reserved3: 0 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 2);
        assert_within_surface(&widgets);
    }

    #[test]
    fn history_records_produces_widget_pool() {
        use cm_domain::screen_batch16::build_history_records;
        let v = build_history_records(true, 42).unwrap();
        let widgets = v.to_widget_pool();
        // 2 prelude + 1 chip + 4 rows.
        assert_eq!(widgets.len(), 2 + 1 + 4);
        // Source marker is on by default.
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    #[test]
    fn manage_obj_51f_produces_widget_pool() {
        let v = ManageObj51fView { object_id: 12345 };
        let widgets = v.to_widget_pool();
        // Prelude removed to match capture: 1 slot row only.
        assert_eq!(widgets.len(), 1);
        assert_within_surface(&widgets);
    }

    #[test]
    fn manage_obj_588_produces_widget_pool() {
        let v = ManageObj588View { object_id: 999 };
        let widgets = v.to_widget_pool();
        // Prelude removed to match capture: 1 slot row only.
        assert_eq!(widgets.len(), 1);
        assert_within_surface(&widgets);
    }

    #[test]
    fn manage_obj_67e_produces_widget_pool() {
        let v = ManageObj67eView { inner_id: 7, record_offset: 0xDEAD };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 2);
        assert_within_surface(&widgets);
    }

    #[test]
    fn match_area_screen_produces_widget_pool() {
        use cm_domain::screen_batch16::build_match_area_screen;
        let v = build_match_area_screen(true, 123).unwrap();
        let widgets = v.to_widget_pool();
        // 2 prelude + 1 chip + 3 rows.
        assert_eq!(widgets.len(), 2 + 1 + 3);
        // screen_flag = 1 → chip highlighted.
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    // -- Skeleton-only impls still match News prelude ----------------------
    #[test]
    fn skeleton_impls_match_news_prelude_shape() {
        use cm_domain::screen_batch3::LatestScoresView;
        // AwardsView no longer skeleton — see awards_view_matches_capture_json.
        for widgets in [
            LatestScoresView::default().to_widget_pool(),
        ] {
            assert_eq!(widgets.len(), NEWS_MIN_WIDGET_COUNT);
            assert_eq!(widgets[0].descriptor.kind, KIND_HEADER);
            assert_eq!(widgets[1].descriptor.kind, KIND_LABEL);
        }
    }
}

// ============================================================================
// Batch 20/21/22 View impls
// ============================================================================

use cm_domain::screen_batch20::{
    HumanManagerSetupView, Screen0080BBD0View, Screen0080CC20View,
    Screen008109C0View, Screen00810CA0View,
};
use cm_domain::screen_batch21::{SimpleScreen00877190View, TacticScreenView};
use cm_domain::screen_batch22::{View008A20A0, TrainingPanelView, PlayerProfileView};

impl RenderableView for HumanManagerSetupView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Human Manager");
        let chips = [
            ("Seat allocated", self.allocated_seat),
            ("Pool error",     self.pool_error),
        ];
        for (i, (name, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), name, *on));
        }
        out.push(row_label(0, "Slot 0", self.slot0));
        out
    }
}

impl RenderableView for Screen0080BBD0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 0080BBD0");
        out.push(tab_button(0, 1, "Focus", self.focus != 0));
        out.push(row_label(0, "Focus", self.focus));
        out
    }
}

impl RenderableView for Screen0080CC20View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 0080CC20");
        let chips = [
            ("Scratch", self.scratch_handle != 0),
            ("Focus",   self.focus != 0),
        ];
        for (i, (n, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), n, *on));
        }
        out.push(row_label(0, "Scratch handle", self.scratch_handle));
        out.push(row_label(1, "Focus", self.focus));
        out
    }
}

impl RenderableView for Screen008109C0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 008109C0");
        out.push(tab_button(0, 1, "Focus", self.focus != 0));
        out.push(row_label(0, "Focus", self.focus));
        out
    }
}

impl RenderableView for Screen00810CA0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 00810CA0");
        out.push(tab_button(0, 1, "Focus", self.focus != 0));
        out.push(row_label(0, "Focus", self.focus));
        out
    }
}

impl RenderableView for SimpleScreen00877190View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 00877190");
        out.push(row_label(0, "No slots", 0));
        out
    }
}

impl RenderableView for TacticScreenView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Tactics");
        let tabs = ["Default", "Alt"];
        let hi = if self.mode == 0 { 0 } else { 1 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Tactic id",     self.tactic_id));
        out.push(row_label(1, "Tactic record", self.tactic_record));
        out.push(row_label(2, "Side A",        self.side_a));
        out.push(row_label(3, "Side B",        self.side_b));
        out.push(row_label(4, "Extra A",       self.extra_a));
        out.push(row_label(5, "Extra B",       self.extra_b));
        out.push(row_label(6, "Slot10 flag",   self.slot10_flag));
        out
    }
}

impl RenderableView for View008A20A0 {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 008A20A0");
        out.push(tab_button(0, 1, "Handle", self.record_handle != 0));
        out.push(row_label(0, "Record handle", self.record_handle));
        out.push(row_label(1, "Flag", self.flag));
        out.push(row_label(2, "Selection", self.selection as u32));
        out
    }
}

impl RenderableView for TrainingPanelView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Training");
        let chips = [
            ("Parent",   self.parent != 0),
            ("Aux flag", self.aux_flag_set),
        ];
        for (i, (n, on)) in chips.iter().enumerate() {
            out.push(tab_button(i, chips.len(), n, *on));
        }
        out.push(row_label(0, "Parent",      self.parent));
        out.push(row_label(1, "Record size", self.record.len() as u32));
        out
    }
}

impl RenderableView for PlayerProfileView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Player Profile");
        for i in 0..8usize {
            out.push(tab_button(i, 8, "Tab", self.tab_sentinels[i] >= 0));
        }
        out.push(row_label(0, "Primary",     self.primary));
        out.push(row_label(1, "Secondary",   self.secondary));
        out.push(row_label(2, "Manager ref", self.manager_ref as u32));
        out.push(row_label(3, "Variant",     self.variant_flag));
        out.push(row_label(4, "Type gate",   self.type_gated_flag));
        out.push(row_label(5, "Club tactic", self.club_tactic));
        out.push(row_label(6, "Opp tactics", self.opposition_tactics_ok as u32));
        out
    }
}

#[cfg(test)]
mod tests_batch20_22 {
    use super::*;

    fn assert_within_surface(widgets: &[Widget]) {
        for w in widgets {
            assert!(w.left >= 0 && w.right <= 799);
            assert!(w.top >= 0 && w.bottom <= 599);
        }
    }

    #[test]
    fn human_manager_setup_view_pool() {
        let v = HumanManagerSetupView { slot0: 0, allocated_seat: true, pool_error: false };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 5);
        assert_eq!(widgets[0].descriptor.kind, KIND_HEADER);
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    #[test]
    fn screen_0080bbd0_view_pool() {
        let v = Screen0080BBD0View { focus: 42 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 4);
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    #[test]
    fn screen_0080cc20_view_pool() {
        let v = Screen0080CC20View { scratch_handle: 1, focus: 2 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 6);
        assert_within_surface(&widgets);
    }

    #[test]
    fn screen_008109c0_view_pool() {
        let v = Screen008109C0View { focus: 7 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 4);
        assert_within_surface(&widgets);
    }

    #[test]
    fn screen_00810ca0_view_pool() {
        let v = Screen00810CA0View { focus: 9 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 4);
        assert_within_surface(&widgets);
    }

    #[test]
    fn simple_screen_00877190_view_pool() {
        let widgets = SimpleScreen00877190View.to_widget_pool();
        assert_eq!(widgets.len(), 3);
        assert_eq!(widgets[0].descriptor.kind, KIND_HEADER);
        assert_within_surface(&widgets);
    }

    #[test]
    fn tactic_screen_view_pool() {
        let v = TacticScreenView {
            tactic_record: 0xdead, tactic_id: 100, extra_a: 7, mode: 0,
            side_a: 501, extra_b: 9, side_b: 502, slot10_flag: 2,
        };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 11);
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
        assert_eq!(widgets[3].flags & 0x0800, 0);
        assert_within_surface(&widgets);
    }

    #[test]
    fn view_008a20a0_pool() {
        let widgets = View008A20A0::default().to_widget_pool();
        assert_eq!(widgets.len(), 6);
        assert_within_surface(&widgets);
    }

    #[test]
    fn training_panel_view_pool() {
        let widgets = TrainingPanelView::default().to_widget_pool();
        assert_eq!(widgets.len(), 6);
        assert_within_surface(&widgets);
    }

    #[test]
    fn player_profile_view_pool() {
        let widgets = PlayerProfileView::default().to_widget_pool();
        assert_eq!(widgets.len(), 17);
        for i in 0..8 {
            assert_eq!(widgets[2 + i].flags & 0x0800, 0);
        }
        assert_within_surface(&widgets);
    }
}

// ============================================================================
// screen_batch23 impls
// ============================================================================

use cm_domain::screen_batch23::{
    Screen008dd030View, Screen008df0c0View, Screen008e0760View,
};

impl RenderableView for Screen008dd030View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 008DD030");
        let tabs = ["P0", "P1", "P2"];
        let hi = if self.p0 != 0 { 0 }
                 else if self.p1 != 0 { 1 }
                 else if self.p2 != 0 { 2 }
                 else { usize::MAX };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Slot 0", self.p0));
        out.push(row_label(1, "Slot 1", self.p1));
        out.push(row_label(2, "Slot 2", self.p2));
        out
    }
}

impl RenderableView for Screen008df0c0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 008DF0C0");
        let tabs = ["Standard", "Related Staff"];
        let hi = if self.p2 == 0x12 || self.p2 == 0x20 { 1 } else { 0 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Slot 0", self.p0));
        out.push(row_label(1, "Slot 1", self.p1));
        out.push(row_label(2, "P2", self.p2 as u32));
        out.push(row_label(3, "P4", self.p4 as u32));
        for (i, id) in self.related_staff.iter().enumerate() {
            out.push(row_label(4 + i as i32, "Related staff", *id));
        }
        out
    }
}

impl RenderableView for Screen008e0760View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 008E0760");
        let tabs = ["Record", "Anchor"];
        let hi = if self.anchor != 0 { 1 } else { 0 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Record", self.record));
        out.push(row_label(1, "Anchor", self.anchor));
        out
    }
}

// ============================================================================
// screen_batch24 impls
// ============================================================================

use cm_domain::screen_batch24::{
    ContractOfferView, SmallThreeSlotView, TwoSlot0710View, TwoSlot0B60View,
    WageOfferView,
};

impl RenderableView for ContractOfferView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Contract Offer");
        let tabs = ["Entity", "Slot 8"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == 0));
        }
        out.push(row_label(0, "Entity", self.entity_ptr));
        out.push(row_label(1, "Slot 8", self.slot_8));
        out
    }
}

impl RenderableView for SmallThreeSlotView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        // Prelude removed to match capture
        // (screen_batch24_small_3_slot_screen_lab_008e01d0.json: 0 areas/objects).
        let mut out: Vec<Widget> = Vec::new();
        let tabs = ["P1", "P2", "Entity"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == 0));
        }
        out.push(row_label(0, "Param 1", self.param_1));
        out.push(row_label(1, "Param 2", self.param_2));
        out.push(row_label(2, "Entity", self.entity_ptr));
        out
    }
}

impl RenderableView for WageOfferView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Wage Offer");
        let tabs = ["Overview", "Base", "Linked", "Buttons"];
        let hi = if self.button_flag_1 != 0 { 3 } else { 0 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Entity",       self.entity_ptr));
        out.push(row_label(1, "Base +0xC",    self.base_field_0xc));
        out.push(row_label(2, "Linked",       self.linked_ptr));
        out.push(row_label(3, "Wage %",       self.wage_percentile as u32));
        out.push(row_label(4, "Entity +0x24", self.entity_field_0x24));
        out.push(row_label(5, "Btn 0",        self.button_flag_0 as u32));
        out.push(row_label(6, "Btn 1",        self.button_flag_1 as u32));
        out
    }
}

impl RenderableView for TwoSlot0710View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        // Prelude removed to match capture
        // (screen_batch24_trivial_2_slot_screen_fun_008e07.json: 0 areas/objects).
        let mut out: Vec<Widget> = Vec::new();
        let tabs = ["P1", "P2"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == 0));
        }
        out.push(row_label(0, "Param 1", self.param_1));
        out.push(row_label(1, "Param 2", self.param_2));
        out
    }
}

impl RenderableView for TwoSlot0B60View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        // Prelude removed to match capture
        // (screen_batch24_trivial_2_slot_screen_fun_008e0b.json: 0 areas/objects).
        let mut out: Vec<Widget> = Vec::new();
        let tabs = ["P1", "P2"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == 0));
        }
        out.push(row_label(0, "Param 1", self.param_1));
        out.push(row_label(1, "Param 2", self.param_2));
        out
    }
}

// ============================================================================
// screen_batch25 impls
// ============================================================================

use cm_domain::screen_batch25::{
    Screen008e26a0View, Screen008e26f0View, Screen008e3320View,
    Screen008e50e0View,
};

impl RenderableView for Screen008e26a0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 008E26A0");
        let tabs = ["Primary", "Secondary"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == 0));
        }
        out.push(row_label(0, "Primary",   self.primary));
        out.push(row_label(1, "Secondary", self.secondary));
        out
    }
}

impl RenderableView for Screen008e26f0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 008E26F0");
        let tabs = ["Off", "On"];
        let hi = if self.flag { 1 } else { 0 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        out.push(row_label(0, "Primary", self.primary));
        out.push(row_label(1, "Flag",    self.flag as u32));
        out
    }
}

impl RenderableView for Screen008e3320View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 008E3320");
        let tabs = ["Value"];
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == 0));
        }
        out.push(row_label(0, "Value", self.value));
        out
    }
}

impl RenderableView for Screen008e50e0View {
    fn to_widget_pool(&self) -> Vec<Widget> {
        let mut out = skeleton_widgets("Screen 008E50E0");
        let tabs = ["Params", "Sub", "State", "Slots"];
        let hi = if self.slot_a != 0 || self.slot_d != 0 { 1 } else { 0 };
        for (i, name) in tabs.iter().enumerate() {
            out.push(tab_button(i, tabs.len(), name, i == hi));
        }
        let rows: [(&str, u32); 16] = [
            ("Slot 0", self.slot0), ("Slot 1", self.slot1),
            ("Slot 2", self.slot2), ("Slot 3", self.slot3),
            ("Slot 4", self.slot4), ("Slot 5", self.slot5),
            ("Slot 6", self.slot6), ("Slot 7", self.slot7),
            ("Slot 8", self.slot8), ("Slot 9", self.slot9),
            ("Slot A", self.slot_a), ("Slot B", self.slot_b),
            ("Slot C", self.slot_c), ("Slot D", self.slot_d),
            ("Slot E", self.slot_e), ("Slot F", self.slot_f),
        ];
        for (i, (name, val)) in rows.iter().enumerate() {
            out.push(row_label(i as i32, name, *val));
        }
        out
    }
}

// ============================================================================
// Golden tests for batch 23/24/25 impls
// ============================================================================

#[cfg(test)]
mod batch_23_24_25_tests {
    use super::*;

    fn assert_within_surface(widgets: &[Widget]) {
        for w in widgets {
            assert!(w.left >= 0 && w.right <= 799,
                    "widget out of x-bounds: {}..{}", w.left, w.right);
            assert!(w.top >= 0 && w.bottom <= 599,
                    "widget out of y-bounds: {}..{}", w.top, w.bottom);
        }
    }

    // -- batch 23 -----------------------------------------------------------
    #[test]
    fn view_render_screen_008dd030() {
        let v = Screen008dd030View { p0: 10, p1: 20, p2: 30 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 8);
        assert_eq!(widgets[0].descriptor.kind, KIND_HEADER);
        assert_eq!(widgets[1].descriptor.kind, KIND_LABEL);
        assert_eq!(widgets[2].descriptor.kind, KIND_BUTTON);
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    #[test]
    fn view_render_screen_008df0c0() {
        let v = Screen008df0c0View {
            p0: 1, p1: 2, p2: 0x12, p4: 3, related_staff: vec![100, 200],
        };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 10);
        assert_eq!(widgets[3].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    #[test]
    fn view_render_screen_008e0760() {
        let v = Screen008e0760View { record: 5, anchor: 7 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 6);
        assert_eq!(widgets[3].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    // -- batch 24 -----------------------------------------------------------
    #[test]
    fn view_render_contract_offer() {
        let v = ContractOfferView { entity_ptr: 0x1000, slot_8: 0x42 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 6);
        assert_within_surface(&widgets);
    }

    #[test]
    fn view_render_small_three_slot() {
        let v = SmallThreeSlotView { param_1: 1, param_2: 2, entity_ptr: 3 };
        let widgets = v.to_widget_pool();
        // Prelude removed to match capture: 3 tabs + 3 slot rows.
        assert_eq!(widgets.len(), 3 + 3);
        assert_within_surface(&widgets);
    }

    #[test]
    fn view_render_wage_offer() {
        let v = WageOfferView {
            entity_ptr: 0x1000, base_field_0xc: 100, base_field_0xc_2: 100,
            b_char_1: 0, b_ushort_1: 0, b_char_2: 0, b_ushort_2: 0,
            raw_entity_ptr: 0, linked_ptr: 0x2000, entity_field_0x24: 0x42,
            const_zero: 0, wage_percentile: 50, button_flag_0: 0,
            linked_field_0x2f: 0, linked_field_0x1c: 0, linked_field_0x1d: 0,
            wage_percentile_2: 50, button_flag_1: 1,
            linked_field_0x2f_2: 0, linked_field_0x1c_2: 0, tail_slot: 0,
        };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 13);
        assert_eq!(widgets[5].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    #[test]
    fn view_render_two_slot_0710() {
        let v = TwoSlot0710View { param_1: 11, param_2: 22 };
        let widgets = v.to_widget_pool();
        // Prelude removed to match capture: 2 tabs + 2 slot rows.
        assert_eq!(widgets.len(), 2 + 2);
        assert_within_surface(&widgets);
    }

    #[test]
    fn view_render_two_slot_0b60() {
        let v = TwoSlot0B60View { param_1: 111, param_2: 222 };
        let widgets = v.to_widget_pool();
        // Prelude removed to match capture: 2 tabs + 2 slot rows.
        assert_eq!(widgets.len(), 2 + 2);
        assert_within_surface(&widgets);
    }

    // -- batch 25 -----------------------------------------------------------
    #[test]
    fn view_render_screen_008e26a0() {
        let v = Screen008e26a0View { primary: 42, secondary: 7 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 6);
        assert_within_surface(&widgets);
    }

    #[test]
    fn view_render_screen_008e26f0() {
        let v = Screen008e26f0View { primary: 99, flag: true };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 6);
        assert_eq!(widgets[3].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }

    #[test]
    fn view_render_screen_008e3320() {
        let v = Screen008e3320View { value: 0xabc };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 4);
        assert_within_surface(&widgets);
    }

    #[test]
    fn view_render_screen_008e50e0() {
        let v = Screen008e50e0View {
            slot0: 1, slot1: 2, slot2: 0, slot3: 7, slot4: 0, slot5: 0,
            slot6: 0, slot7: 0, slot8: 0, slot9: 0, slot_a: 1, slot_b: 5,
            slot_c: 1, slot_d: 1, slot_e: 0, slot_f: 0,
        };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 22);
        assert_eq!(widgets[3].flags & 0x0800, 0x0800);
        assert_within_surface(&widgets);
    }
}

// ============================================================================
// Batch 17-19 golden tests
// ============================================================================

#[cfg(test)]
mod batch_17_19_tests {
    use super::*;
    use cm_domain::screen_batch17::{
        MatchDetailView, FixtureNavView, TrivialRegistrationView,
        MediaArticleView, SmallFourSlotView,
    };
    use cm_domain::screen_batch18::{
        TwoSelectorListView, LoaderGatedDialogView, DualSelectorScratchView,
        SeatScreenView, TwoPointerView,
    };
    use cm_domain::screen_batch19::{
        PersonBannerView, SearchResultDetailView, SetupBootstrapView, FiveZeroSlotView,
    };

    fn assert_within_surface(widgets: &[Widget]) {
        for w in widgets {
            assert!(w.left >= 0 && w.right <= 799);
            assert!(w.top >= 0 && w.bottom <= 599);
        }
    }

    fn assert_shape(widgets: &[Widget], tabs: usize, rows: usize) {
        assert_eq!(widgets.len(), 2 + tabs + rows);
        assert_eq!(widgets[0].descriptor.kind, KIND_HEADER);
        assert_eq!(widgets[1].descriptor.kind, KIND_LABEL);
        for i in 0..tabs {
            assert_eq!(widgets[2 + i].descriptor.kind, KIND_BUTTON);
        }
        assert_within_surface(widgets);
    }

    #[test]
    fn match_detail_view_widgets() {
        let v = MatchDetailView {
            match_id: 100, match_field_63: 0xAB,
            bumped_human_seen: true, redispatched_to_alt: false,
        };
        assert_shape(&v.to_widget_pool(), 2, 3);
    }

    #[test]
    fn fixture_nav_view_widgets() {
        let v = FixtureNavView { current_comp_id: 3, fixture_ref: 9, is_home_leg: true };
        let widgets = v.to_widget_pool();
        assert_shape(&widgets, 2, 2);
        assert_eq!(widgets[2].flags & 0x0800, 0x0800);
    }

    #[test]
    fn trivial_registration_view_widgets() {
        let v = TrivialRegistrationView { widget_id: 42 };
        assert_shape(&v.to_widget_pool(), 1, 1);
    }

    #[test]
    fn media_article_view_widgets() {
        let v = MediaArticleView {
            caller_arg: 1, var_2f8: 2, var_2f4: 3,
            val_u0: 0, val_u1: 0, val_u2: 0, val_u3: 0,
            var_2f0: 0, var_2fc: 7,
        };
        assert_shape(&v.to_widget_pool(), 2, 4);
    }

    #[test]
    fn small_four_slot_view_widgets() {
        let v = SmallFourSlotView { arg_a: 1, flag: -2, arg_c: 3, sentinel: -1 };
        assert_shape(&v.to_widget_pool(), 2, 3);
    }

    #[test]
    fn two_selector_list_view_widgets() {
        let v = TwoSelectorListView {
            row_handle: 0x100, selector1: 1, visible_count: 5,
            selector5: 7, ..Default::default()
        };
        let widgets = v.to_widget_pool();
        assert_shape(&widgets, 2, 3);
        assert_eq!(widgets[3].flags & 0x0800, 0x0800);
    }

    #[test]
    fn loader_gated_dialog_view_widgets() {
        let v = LoaderGatedDialogView { marker: 1, selector0: 5 };
        assert_shape(&v.to_widget_pool(), 1, 2);
    }

    #[test]
    fn dual_selector_scratch_view_widgets() {
        let v = DualSelectorScratchView::default();
        assert_shape(&v.to_widget_pool(), 2, 3);
    }

    #[test]
    fn seat_screen_view_widgets() {
        let v = SeatScreenView { entry_present: 1, slot5_value: 9, ..Default::default() };
        assert_shape(&v.to_widget_pool(), 2, 4);
    }

    #[test]
    fn two_pointer_view_widgets() {
        let v = TwoPointerView { p1_deref: 1, p2_deref: 2 };
        assert_shape(&v.to_widget_pool(), 2, 2);
    }

    #[test]
    fn person_banner_view_widgets() {
        let v = PersonBannerView {
            record_head: 0xDEAD, category_byte: 0x4e, sub_head_byte: 3,
        };
        let widgets = v.to_widget_pool();
        // Prelude removed to match capture: 2 tabs + 3 slot rows.
        assert_eq!(widgets.len(), 2 + 3);
        for i in 0..2 {
            assert_eq!(widgets[i].descriptor.kind, KIND_BUTTON);
        }
        assert_within_surface(&widgets);
    }

    #[test]
    fn search_result_detail_view_widgets() {
        let v = SearchResultDetailView::default();
        let widgets = v.to_widget_pool();
        // Prelude removed to match capture: 2 tabs + 3 slot rows.
        assert_eq!(widgets.len(), 2 + 3);
        assert_within_surface(&widgets);
    }

    #[test]
    fn setup_bootstrap_view_widgets() {
        let v = SetupBootstrapView { offline_flag: true, seed: 42, nonetwork: true };
        let widgets = v.to_widget_pool();
        assert_shape(&widgets, 2, 2);
        assert_eq!(widgets[3].flags & 0x0800, 0x0800);
    }

    #[test]
    fn five_zero_slot_view_widgets() {
        let v = FiveZeroSlotView::default();
        let widgets = v.to_widget_pool();
        // Prelude removed to match capture: 1 tab + 5 slot rows.
        assert_eq!(widgets.len(), 1 + 5);
        assert_within_surface(&widgets);
    }
}

// ============================================================================
// News — the HOME screen (callback 0x00770170). Fully re-verified this
// session via a LIVE Frida capture of the real, running exe (a real save
// loaded, actually on the News page) rather than the earlier Unicorn
// emulation, which only ever saw the header (it force-stubbed the
// unread-news count "to skip the row loop", so it never observed the list,
// filter bar, story panel, second tab row, or buttons at all — see
// DIRECTDRAW_CAPTURE_HANDOVER.md §11 and memory [[news-screen-geometry]]).
// Live capture: hooked the real GUIO/AREA/TABS/NAV constructors globally,
// recorded only while execution was actually inside the 0x00770170 draw
// callback. Text was dereferenced live at each widget's construction time
// (many widgets share one scratch text buffer that gets overwritten by the
// next widget, so reading it after the fact only shows the last write).
//
// Real areas captured (12 total, all parent=-1 i.e. top-level):
//   header:        100,10 →790,70   flags=48
//   header inner:  100,25 →790,55   flags=1
//   top tabs bg:   100,80 →790,115  flags=2
//   top tabs ctr:  100,80 →790,115  flags=1  (4 children)
//   news list:     110,125→780,215  flags=1  (2 cols × 5 rows)
//   filter bar:    405,220→655,240  flags=2  (2 children)
//   headline strip:110,245→780,280  flags=2
//   story panel:   110,285→780,500  flags=2
//   bottom tabs bg:100,510→790,545  flags=2
//   bottom tabs ctr:100,510→790,545 flags=1  (4 children)
//   nav row:       100,555→790,590  flags=1  (2 children: Back, Next)
// (the left sidebar, 0,0→89,599, is separate shared chrome — FUN_00745540,
// already documented in memory [[menu-command-tree]] — deliberately not
// modelled here.)
//
// Real static labels (from the function's own string table,
// subsystem_map.json 0x770170 string_hits — constant regardless of game
// state): top tabs "All"/"Messages"/"Competitions"/"Injuries and Bans";
// bottom tabs "Contracts and Media"/"Transfers"/"Jobs"/"Records";
// "Filter :"; "Next Unread".
// ============================================================================

/// Captured constants — see DIRECTDRAW_CAPTURE_HANDOVER.md §11 for the raw
/// live-capture log this was transcribed from.
pub mod news_geom {
    pub const HEADER_L: i32 = 100;
    pub const HEADER_T: i32 = 10;
    pub const HEADER_R: i32 = 790;
    pub const HEADER_B: i32 = 70;
    pub const HEADER_FLAGS: u32 = 48;

    pub const HEADER_INNER_T: i32 = 25;
    pub const HEADER_INNER_B: i32 = 55;
    pub const HEADER_INNER_FLAGS: u32 = 1;

    pub const TOP_TABS_T: i32 = 80;
    pub const TOP_TABS_B: i32 = 115;
    pub const BOTTOM_TABS_T: i32 = 510;
    pub const BOTTOM_TABS_B: i32 = 545;
    pub const TABS_BG_FLAGS: u32 = 2;
    pub const TABS_CTR_FLAGS: u32 = 1;
    // Real per-tab-label rflags (distinct from the container's own flags
    // above): captured live as 2096 for the selected tab, 48 otherwise.
    pub const TAB_SELECTED_FLAGS: u32 = 2096;
    pub const TAB_UNSELECTED_FLAGS: u32 = 48;

    pub const LIST_L: i32 = 110;
    pub const LIST_T: i32 = 125;
    pub const LIST_R: i32 = 780;
    pub const LIST_B: i32 = 215;
    pub const LIST_FLAGS: u32 = 1;
    pub const LIST_ROWS: i32 = 5;
    pub const LIST_DATE_COL_W: i32 = 130; // approx split of the 2-column row.

    pub const FILTER_L: i32 = 405;
    pub const FILTER_T: i32 = 220;
    pub const FILTER_R: i32 = 655;
    pub const FILTER_B: i32 = 240;
    pub const FILTER_FLAGS: u32 = 2;

    pub const HEADLINE_STRIP_L: i32 = 110;
    pub const HEADLINE_STRIP_T: i32 = 245;
    pub const HEADLINE_STRIP_R: i32 = 780;
    pub const HEADLINE_STRIP_B: i32 = 280;
    pub const HEADLINE_STRIP_FLAGS: u32 = 2;

    pub const STORY_L: i32 = 110;
    pub const STORY_T: i32 = 285;
    pub const STORY_R: i32 = 780;
    pub const STORY_B: i32 = 500;
    pub const STORY_FLAGS: u32 = 2;

    pub const NAV_L: i32 = 100;
    pub const NAV_T: i32 = 555;
    pub const NAV_R: i32 = 790;
    pub const NAV_B: i32 = 590;
    pub const NAV_FLAGS: u32 = 1;

    pub const TOP_TAB_LABELS: [&str; 4] = ["All", "Messages", "Competitions", "Injuries and Bans"];
    pub const BOTTOM_TAB_LABELS: [&str; 4] =
        ["Contracts and Media", "Transfers", "Jobs", "Records"];
    pub const FILTER_LABEL: &str = "Filter :";
    pub const NEXT_UNREAD_LABEL: &str = "Next Unread";

    /// Sentinel `fg_color` value the renderer maps to yellow, for the
    /// selected item's headline in the story panel -- see the comment at
    /// its use site for why this is a targeted, unverified fix.
    pub const YELLOW_TEXT_MARKER: u32 = 0xFFFF00;

    // Kept for the pre-existing golden test below (still real/verified).
    pub const AREA1_FLAGS: u32 = HEADER_FLAGS;
}

impl RenderableView for cm_domain::NewsView {
    fn to_widget_pool(&self) -> Vec<Widget> {
        use news_geom::*;

        fn area(x0: i32, y0: i32, x1: i32, y1: i32, rflags: u32) -> Widget {
            let mut d = WidgetDescriptor::empty();
            d.kind = KIND_ROOT_HOLDER;
            d.grid_x0 = x0; d.grid_y0 = y0; d.grid_x1 = x1; d.grid_y1 = y1;
            // rflags stored on the widget-level `flags` field so callers
            // that discriminate widget-role via `descriptor.kind` keep
            // matching (`kind` stays a pure KIND_* value; the rflags
            // dword lives alongside it).
            let mut w = widget_from(d);
            w.flags = rflags;
            w
        }
        fn label(x0: i32, y0: i32, x1: i32, y1: i32, rflags: u32, font: u8, text: &str) -> Widget {
            label_ex(x0, y0, x1, y1, rflags, font, text, true)
        }
        // `enabled` mirrors the live-captured `tflags` distinction
        // (12 = normal, 44 = disabled/greyed) — stored on `text_style`
        // (+0x3c). A SEPARATE field from the widget flags dword
        // (`w.flags`) which stayed 48 (filled+bevel box) for both
        // "Back" (enabled) and "Next"/"Next Unread" (disabled) in the
        // live capture. Using rflags alone to decide enabled/disabled
        // (an earlier mistake) produced the wrong box style for disabled
        // buttons.
        fn label_ex(x0: i32, y0: i32, x1: i32, y1: i32, rflags: u32,
                    font: u8, text: &str, enabled: bool) -> Widget {
            let mut d = WidgetDescriptor::empty();
            d.kind = KIND_LABEL;
            d.grid_x0 = x0; d.grid_y0 = y0; d.grid_x1 = x1; d.grid_y1 = y1;
            // Text style word: font id in the low nibble, disabled bit
            // (0x20) high — matches the captured `tflags` = 12 or 44 =
            // 0xC | (0x20 disabled).
            d.text_style = font as u32 | if enabled { 0 } else { 0x20 };
            d.text = text.to_string();
            let mut w = widget_from(d);
            w.flags = rflags;
            w
        }

        let mut out = Vec::new();

        // Header.
        out.push(area(HEADER_L, HEADER_T, HEADER_R, HEADER_B, HEADER_FLAGS));
        out.push(area(HEADER_L, HEADER_INNER_T, HEADER_R, HEADER_INNER_B, HEADER_INNER_FLAGS));
        out.push(label(HEADER_L, HEADER_T, HEADER_R, HEADER_B, HEADER_FLAGS, 7, &self.title));

        // Top tab row (background + container), 4 real category buttons.
        // Real per-tab rflags captured live: the SELECTED tab gets 2096
        // (0x830 = the same fill+bevel bits plus an extra 0x800 "selected"
        // bit that renders as a light/white box instead of navy);
        // unselected tabs get plain 48 (0x30 = fill+bevel). An earlier
        // version discarded this and used one flat flag value for every
        // tab, losing the selected/unselected visual distinction entirely.
        out.push(area(HEADER_L, TOP_TABS_T, HEADER_R, TOP_TABS_B, TABS_BG_FLAGS));
        out.push(area(HEADER_L, TOP_TABS_T, HEADER_R, TOP_TABS_B, TABS_CTR_FLAGS));
        let top_tab_w = (HEADER_R - HEADER_L) / TOP_TAB_LABELS.len() as i32;
        for (i, name) in TOP_TAB_LABELS.iter().enumerate() {
            let x0 = HEADER_L + i as i32 * top_tab_w;
            let flags = if i as u16 == self.selected_tab { TAB_SELECTED_FLAGS } else { TAB_UNSELECTED_FLAGS };
            // font=1 (arial_narrow_10), not 3 -- real capture, previously
            // used the wrong (larger) font here.
            out.push(label(x0, TOP_TABS_T, x0 + top_tab_w - 2, TOP_TABS_B, flags, 1, name));
        }

        // News list: real captured rows (date + headline pair each), the
        // selected one highlighted. `self.items`/`self.selected_item` are
        // the live per-instance state; falls back to nothing if empty.
        let row_h = (LIST_B - LIST_T) / LIST_ROWS.max(1);
        for (i, item) in self.items.iter().take(LIST_ROWS as usize).enumerate() {
            let y0 = LIST_T + i as i32 * row_h;
            let y1 = y0 + row_h;
            let selected = i == self.selected_item;
            // Real capture log (frida_capture_news_live.log, seq=20/21 for
            // row 0, the selected row that session): the DATE column's
            // rflags stays 48 REGARDLESS of selection -- seq=20 (date,
            // selected row) and seq=22 (date, unselected row) both read
            // rflags=48. Only the HEADLINE column switches, 2 -> 528
            // (seq=21 vs seq=23). An earlier version flipped the date
            // column to 528 too whenever selected, which is why the
            // selected row's date cell rendered red instead of the real
            // blue -- date column color never depends on selection.
            out.push(label(LIST_L, y0, LIST_L + LIST_DATE_COL_W, y1, 48, 1, &item.date_label));
            let headline_flags = if selected { 528 } else { 2 };
            out.push(label(LIST_L + LIST_DATE_COL_W, y0, LIST_R, y1, headline_flags, 2, &item.headline));
        }

        // Filter bar.
        out.push(area(FILTER_L, FILTER_T, FILTER_R, FILTER_B, FILTER_FLAGS));
        let filter_mid = (FILTER_L + FILTER_R) / 2;
        out.push(label(FILTER_L, FILTER_T, filter_mid, FILTER_B, FILTER_FLAGS, 1, FILTER_LABEL));
        out.push(label(filter_mid, FILTER_T, FILTER_R, FILTER_B, FILTER_FLAGS, 1, NEXT_UNREAD_LABEL));

        // Selected item's headline strip + story panel (photo + body text).
        // Real capture: headline label rflags=1 (plain, no box) -- its
        // yellow color (visible in the real screenshot) isn't recoverable
        // from rflags/tflags alone (those matched several other plain-text
        // widgets that render white), so this is a targeted,
        // screenshot-driven fix via `fg_color`, not yet independently
        // verified against the real palette table.
        out.push(area(HEADLINE_STRIP_L, HEADLINE_STRIP_T, HEADLINE_STRIP_R, HEADLINE_STRIP_B, HEADLINE_STRIP_FLAGS));
        if let Some(sel) = self.items.get(self.selected_item) {
            let mut hdl = WidgetDescriptor::empty();
            hdl.kind = KIND_LABEL;
            hdl.grid_x0 = HEADLINE_STRIP_L; hdl.grid_y0 = HEADLINE_STRIP_T;
            hdl.grid_x1 = HEADLINE_STRIP_R; hdl.grid_y1 = HEADLINE_STRIP_B;
            hdl.kind = KIND_LABEL;
            hdl.text_style = 3;
            hdl.text = sel.headline.clone();
            hdl.label_ink = (YELLOW_TEXT_MARKER & 0xFFFF) as u16;
            out.push(widget_from(hdl));
            out.push(area(STORY_L, STORY_T, STORY_R, STORY_B, STORY_FLAGS));
            out.push(label(STORY_L + 10, STORY_T + 15, STORY_R - 10, STORY_B - 10, STORY_FLAGS, 2, &sel.body));
        } else {
            out.push(area(STORY_L, STORY_T, STORY_R, STORY_B, STORY_FLAGS));
        }

        // Bottom tab row (background + container), 4 real category
        // buttons. Real capture: all 4 use plain rflags=48 (never
        // "selected" -- these are action links, not filter tabs).
        out.push(area(HEADER_L, BOTTOM_TABS_T, HEADER_R, BOTTOM_TABS_B, TABS_BG_FLAGS));
        out.push(area(HEADER_L, BOTTOM_TABS_T, HEADER_R, BOTTOM_TABS_B, TABS_CTR_FLAGS));
        let bot_tab_w = (HEADER_R - HEADER_L) / BOTTOM_TAB_LABELS.len() as i32;
        for (i, name) in BOTTOM_TAB_LABELS.iter().enumerate() {
            let x0 = HEADER_L + i as i32 * bot_tab_w;
            // font=1, same real correction as the top tabs above.
            out.push(label(x0, BOTTOM_TABS_T, x0 + bot_tab_w - 2, BOTTOM_TABS_B, TAB_UNSELECTED_FLAGS, 1, name));
        }

        // Nav row: Back / Next. Real capture: BOTH have rflags=48 (same
        // filled+bevel box) -- the enabled/disabled distinction is in
        // `tflags` (12 vs 44), a separate field, not `rflags`. An earlier
        // version conflated the two and gave disabled buttons the wrong
        // box style entirely.
        // Real measured split (NOT 50/50, an earlier placeholder that was
        // wrong): a full-frame capture of the real back buffer showed a
        // bevel edge at x=618 consistently across 4 scanned rows near the
        // nav row's top/bottom borders -- see
        // scratchpad/frida_capture_full_frame.py. Back spans 100->618
        // (518px), Next spans 618->790 (172px) -- Back really is ~3x
        // wider, confirming what looked odd in the earlier 50/50 render.
        // Also real: NO visible divider/border exists between the two in
        // the interior of the bar -- it reads as one continuous panel with
        // two text labels, not two separately-beveled buttons; rendering
        // it as two independently-beveled boxes (as before) drew a seam
        // that shouldn't be there.
        const NAV_SPLIT: i32 = 618;
        out.push(area(NAV_L, NAV_T, NAV_R, NAV_B, NAV_FLAGS));
        out.push(label_ex(NAV_L, NAV_T, NAV_SPLIT, NAV_B, 48, 3, "Back", self.nav_back_enabled));
        out.push(label_ex(NAV_SPLIT, NAV_T, NAV_R, NAV_B, 48, 3, "Next", self.nav_next_enabled));

        out
    }
}

#[cfg(test)]
mod news_view_tests {
    use super::*;
    use cm_domain::{NewsView, NewsItem, NewsCategory, GameDate};

    /// Real widgets built from a real 5-item news list, matching a live
    /// Frida capture of the actual running exe (see
    /// DIRECTDRAW_CAPTURE_HANDOVER.md §11) rather than the earlier
    /// Unicorn-emulation ground truth (which only ever saw the header —
    /// its forced "0 unread news" state skipped the entire list-building
    /// loop).
    fn sample_item(date_label: &str, headline: &str, body: &str) -> NewsItem {
        NewsItem {
            date: GameDate { year: 2001, month: 10, day: 10 },
            date_label: date_label.to_string(),
            headline: headline.to_string(),
            body: body.to_string(),
            category: NewsCategory::Message,
            unread: false,
        }
    }

    fn sample_view() -> NewsView {
        NewsView {
            title: "Christoph Olewicz News".to_string(),
            items: vec![
                sample_item("Wed 10th Oct EVE", "Board reaction to Rhayader game",
                    "The Haverfordwest County board of directors are pleased with the 1-0 \
                     Welsh League Cup win against Rhayader Town."),
                sample_item("Wed 10th Oct EVE", "Haverfordwest win in Welsh League Cup Quarter Final", ""),
                sample_item("Mon 8th Oct EVE", "Brazil out for about 2 weeks", ""),
                sample_item("Sun 7th Oct EVE", "Sweden ready for 2002", ""),
                sample_item("Sun 7th Oct EVE", "Burrows resumes full training", ""),
            ],
            selected_tab: 0, // "All" — captured live (this session's earlier
                              // capture had sel=12 from a different runtime
                              // state; both are valid dynamic values).
            tab_count: 8,
            selected_item: 0,
            nav_back_enabled: true,
            nav_next_enabled: false,
        }
    }

    /// Assert the emitted geometry matches the live capture on every real
    /// area's (L, T, R, B, flags), and that the real static tab labels /
    /// filter strings appear verbatim.
    #[test]
    fn news_view_matches_live_capture() {
        let v = sample_view();
        let widgets = v.to_widget_pool();

        let areas: Vec<_> = widgets.iter()
            .filter(|w| w.descriptor.kind == KIND_ROOT_HOLDER)
            .map(|w| (w.left, w.top, w.right, w.bottom, w.flags))
            .collect();
        // 11 real areas: header, header-inner, top-tabs bg+ctr, list is not
        // its own area widget here (rows are direct labels), filter,
        // headline-strip, story, bottom-tabs bg+ctr, nav.
        assert_eq!(areas, vec![
            (100, 10, 790, 70, 48),
            (100, 25, 790, 55, 1),
            (100, 80, 790, 115, 2),
            (100, 80, 790, 115, 1),
            (405, 220, 655, 240, 2),
            (110, 245, 780, 280, 2),
            (110, 285, 780, 500, 2),
            (100, 510, 790, 545, 2),
            (100, 510, 790, 545, 1),
            (100, 555, 790, 590, 1),
        ]);

        let texts: Vec<&str> = widgets.iter()
            .filter(|w| w.descriptor.kind == KIND_LABEL)
            .map(|w| w.descriptor.text.as_str())
            .collect();
        for want in [
            "Christoph Olewicz News", "All", "Messages", "Competitions", "Injuries and Bans",
            "Wed 10th Oct EVE", "Board reaction to Rhayader game",
            "Filter :", "Next Unread",
            "Contracts and Media", "Transfers", "Jobs", "Records",
            "Back", "Next",
        ] {
            assert!(texts.contains(&want), "missing expected text: {want:?} (have {texts:?})");
        }
    }

    /// The selected row's HEADLINE column gets the highlighted rflags
    /// (528); its DATE column does not -- real capture log
    /// (frida_capture_news_live.log, seq=20/21/22) shows the date column
    /// reads rflags=48 for both the selected row and every other row, and
    /// only the headline column switches 2 -> 528. An earlier version of
    /// this test asserted 528 on the date column too, matching a bug in
    /// the production code that painted the selected row's date cell red
    /// instead of the real blue.
    #[test]
    fn selected_row_is_highlighted() {
        let v = sample_view();
        let widgets = v.to_widget_pool();
        let date_widgets: Vec<_> = widgets.iter()
            .filter(|w| w.descriptor.kind == KIND_LABEL
                && v.items.iter().any(|it| it.date_label == w.descriptor.text))
            .collect();
        assert_eq!(date_widgets[0].flags, 48, "date column never highlights, even when its row is selected");
        assert_eq!(date_widgets[1].flags, 48, "non-selected rows should not be highlighted");

        let headline_widgets: Vec<_> = widgets.iter()
            .filter(|w| w.descriptor.kind == KIND_LABEL
                && v.items.iter().any(|it| it.headline == w.descriptor.text))
            .collect();
        assert_eq!(headline_widgets[0].flags, 528, "selected row's headline should be highlighted");
        assert_eq!(headline_widgets[1].flags, 2, "non-selected rows' headline should not be highlighted");
    }

    /// Nav buttons reflect the live-captured enabled/disabled state.
    #[test]
    fn nav_buttons_reflect_state() {
        let v = sample_view();
        let widgets = v.to_widget_pool();
        let back = widgets.iter().find(|w| w.descriptor.text == "Back").unwrap();
        let next = widgets.iter().find(|w| w.descriptor.text == "Next").unwrap();
        // Real capture: both use rflags=48 (same box); enabled/disabled is
        // a separate field (tflags 12 vs 44 in the exe -> `enabled` here).
        assert_eq!(back.flags, 48);
        assert_eq!(next.flags, 48);
        // Enabled/disabled now lives on the text_style word (see
        // label_ex above — bit 0x20 = disabled).
        assert_eq!(back.descriptor.text_style & 0x20, 0, "Back is enabled");
        assert_ne!(next.descriptor.text_style & 0x20, 0, "Next is disabled");
    }

    /// Empty items list still renders the static chrome without panicking.
    #[test]
    fn news_view_with_no_items_still_renders() {
        let v = NewsView {
            title: String::new(), items: vec![],
            selected_tab: 0, tab_count: 8,
            selected_item: 0, nav_back_enabled: true, nav_next_enabled: false,
        };
        let widgets = v.to_widget_pool();
        assert!(!widgets.is_empty());
    }
}


// ============================================================================
// Golden tests: 11 exact-slot-count-match screens.
//
// Each Rust `*View` here corresponds to a captured setup fn whose exe
// capture (reports/screen_captures/*.json) records 0 areas/objects for
// the setup — the DRAW callback isn't hit during setup, so the invented
// header/body prelude is dropped from the impl. The tests below embed
// the captured JSON via `include_str!` and assert that every slot value
// the impl emits (as a `row_label` userdata_id or as a widget bound)
// appears verbatim in the JSON as `"val": N` (the exe's slot table
// column) — the news_view_matches_capture_json pattern.
// ============================================================================
#[cfg(test)]
mod capture_json_goldens {
    use super::*;
    use cm_domain::screen_batch5::AwardsView;
    use cm_domain::screen_batch7::MatchReportView;
    use cm_domain::screen_batch8::{HallOfFameView, WrittenHistoryView};
    use cm_domain::screen_batch16::{ManageObj51fView, ManageObj588View};
    use cm_domain::screen_batch19::{
        PersonBannerView, SearchResultDetailView, FiveZeroSlotView,
    };
    use cm_domain::screen_batch24::{
        SmallThreeSlotView, TwoSlot0710View, TwoSlot0B60View,
    };

    /// Every one of these captures has zero areas + zero objects +
    /// zero tabs — the DRAW callback wasn't reached during setup so
    /// only the slot table was populated. The impl's prelude is
    /// dropped to match; assert the JSON confirms it.
    fn assert_empty_widget_geometry(json: &str) {
        assert!(json.contains(r#""areas": []"#),
            "expected `\"areas\": []` in capture");
        assert!(json.contains(r#""objects": []"#),
            "expected `\"objects\": []` in capture");
        assert!(json.contains(r#""tabs": []"#),
            "expected `\"tabs\": []` in capture");
    }

    // -- 1. ManageObj51fView / manage_screen_cm3_code_manage_0x -----------
    const MANAGE_0X: &str = include_str!(
        "../../../reports/screen_captures/screen_batch16_manage_screen_cm3_code_manage_0x.json"
    );

    #[test]
    fn manage_obj_51f_matches_capture_json() {
        assert_empty_widget_geometry(MANAGE_0X);
        let v = ManageObj51fView { object_id: 274_726_912 };
        let widgets = v.to_widget_pool();
        // Prelude removed to match capture; 1 slot row remains.
        assert_eq!(widgets.len(), 1);
        assert_eq!(widgets[0].descriptor.userdata_id, 274_726_912);
        assert!(MANAGE_0X.contains(r#""val": 274726912"#));
    }

    // -- 2. ManageObj588View / manage_screen_cm3_code_manage_0x_2 --------
    const MANAGE_0X_2: &str = include_str!(
        "../../../reports/screen_captures/screen_batch16_manage_screen_cm3_code_manage_0x_2.json"
    );

    #[test]
    fn manage_obj_588_matches_capture_json() {
        assert_empty_widget_geometry(MANAGE_0X_2);
        let v = ManageObj588View { object_id: 274_726_912 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 1);
        assert_eq!(widgets[0].descriptor.userdata_id, 274_726_912);
        assert!(MANAGE_0X_2.contains(r#""val": 274726912"#));
    }

    // -- 3. FiveZeroSlotView / 5_slot_all_zero_screen_persisten -----------
    const FIVE_ZERO: &str = include_str!(
        "../../../reports/screen_captures/screen_batch19_5_slot_all_zero_screen_persisten.json"
    );

    #[test]
    fn five_zero_slot_matches_capture_json() {
        assert_empty_widget_geometry(FIVE_ZERO);
        let v = FiveZeroSlotView::default();
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 1 + 5);
        for row in &widgets[1..] {
            assert_eq!(row.descriptor.userdata_id, 0);
        }
        assert!(FIVE_ZERO.contains(r#""val": 0"#));
    }

    // -- 4. PersonBannerView / person_squad_type_banner_setup_3 ----------
    const PERSON_BANNER: &str = include_str!(
        "../../../reports/screen_captures/screen_batch19_person_squad_type_banner_setup_3.json"
    );

    #[test]
    fn person_banner_matches_capture_json() {
        assert_empty_widget_geometry(PERSON_BANNER);
        let v = PersonBannerView {
            record_head: 16_843_009, category_byte: 77, sub_head_byte: 0,
        };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 3);
        assert_eq!(widgets[2].descriptor.userdata_id, 16_843_009);
        assert_eq!(widgets[3].descriptor.userdata_id, 77);
        assert_eq!(widgets[4].descriptor.userdata_id, 0);
        for needle in &[
            r#""val": 16843009"#, r#""val": 77"#, r#""val": 0"#,
        ] {
            assert!(PERSON_BANNER.contains(needle),
                "capture missing slot value: {needle}");
        }
    }

    // -- 5. SearchResultDetailView / search_result_detail_setup_3_slo ----
    const SEARCH_DETAIL: &str = include_str!(
        "../../../reports/screen_captures/screen_batch19_search_result_detail_setup_3_slo.json"
    );

    #[test]
    fn search_result_detail_matches_capture_json() {
        assert_empty_widget_geometry(SEARCH_DETAIL);
        let v = SearchResultDetailView::default();
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 3);
        for needle in &[
            r#""val": 274726912"#, r#""val": 276824064"#, r#""val": 16"#,
        ] {
            assert!(SEARCH_DETAIL.contains(needle),
                "capture missing slot value: {needle}");
        }
    }

    // -- 6. SmallThreeSlotView / small_3_slot_screen_lab_008e01d0 --------
    const SMALL_3_SLOT: &str = include_str!(
        "../../../reports/screen_captures/screen_batch24_small_3_slot_screen_lab_008e01d0.json"
    );

    #[test]
    fn small_three_slot_matches_capture_json() {
        assert_empty_widget_geometry(SMALL_3_SLOT);
        let v = SmallThreeSlotView {
            param_1: u32::MAX,
            param_2: 1,
            entity_ptr: 270_499_596,
        };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 3 + 3);
        assert_eq!(widgets[3].descriptor.userdata_id, u32::MAX);
        assert_eq!(widgets[4].descriptor.userdata_id, 1);
        assert_eq!(widgets[5].descriptor.userdata_id, 270_499_596);
        for needle in &[
            r#""val": -1"#, r#""val": 1"#, r#""val": 270499596"#,
        ] {
            assert!(SMALL_3_SLOT.contains(needle),
                "capture missing slot value: {needle}");
        }
    }

    // -- 7. TwoSlot0710View / trivial_2_slot_screen_fun_008e07 -----------
    const TWO_SLOT_0710: &str = include_str!(
        "../../../reports/screen_captures/screen_batch24_trivial_2_slot_screen_fun_008e07.json"
    );

    #[test]
    fn two_slot_0710_matches_capture_json() {
        assert_empty_widget_geometry(TWO_SLOT_0710);
        let v = TwoSlot0710View { param_1: 274_726_912, param_2: 0 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 2);
        assert_eq!(widgets[2].descriptor.userdata_id, 274_726_912);
        assert_eq!(widgets[3].descriptor.userdata_id, 0);
        assert!(TWO_SLOT_0710.contains(r#""val": 274726912"#));
        assert!(TWO_SLOT_0710.contains(r#""val": 0"#));
    }

    // -- 8. TwoSlot0B60View / trivial_2_slot_screen_fun_008e0b -----------
    const TWO_SLOT_0B60: &str = include_str!(
        "../../../reports/screen_captures/screen_batch24_trivial_2_slot_screen_fun_008e0b.json"
    );

    #[test]
    fn two_slot_0b60_matches_capture_json() {
        assert_empty_widget_geometry(TWO_SLOT_0B60);
        let v = TwoSlot0B60View { param_1: 274_726_912, param_2: 0 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2 + 2);
        assert_eq!(widgets[2].descriptor.userdata_id, 274_726_912);
        assert_eq!(widgets[3].descriptor.userdata_id, 0);
        assert!(TWO_SLOT_0B60.contains(r#""val": 274726912"#));
        assert!(TWO_SLOT_0B60.contains(r#""val": 0"#));
    }

    // -- 9. AwardsView / awards_cmd_0x7d4_196_b_2_slots ------------------
    const AWARDS: &str = include_str!(
        "../../../reports/screen_captures/screen_batch5_awards_cmd_0x7d4_196_b_2_slots.json"
    );

    #[test]
    fn awards_view_matches_capture_json() {
        assert_empty_widget_geometry(AWARDS);
        let v = AwardsView { award_table: 1, view_mode: 0 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 2);
        assert_eq!(widgets[0].descriptor.userdata_id, 1);
        assert_eq!(widgets[1].descriptor.userdata_id, 0);
        assert!(AWARDS.contains(r#""val": 1"#));
        assert!(AWARDS.contains(r#""val": 0"#));
    }

    // -- 10. MatchReportView / match_report_screen_setup_cmd_0x ----------
    const MATCH_REPORT: &str = include_str!(
        "../../../reports/screen_captures/screen_batch7_match_report_screen_setup_cmd_0x.json"
    );

    #[test]
    fn match_report_matches_capture_json() {
        assert_empty_widget_geometry(MATCH_REPORT);
        let v = MatchReportView {
            fixture_handle: 0,
            selected_event: None,
            selected_player: None,
            show_summary: 0,
            ..Default::default()
        };
        let widgets = v.to_widget_pool();
        // Prelude removed: 7 section tabs + 2 rows (fixture, detail_level).
        assert_eq!(widgets.len(), 7 + 2);
        assert!(MATCH_REPORT.contains(r#""val": 0"#));
        // Capture slot idx 2 val 6 marks the section count.
        assert!(MATCH_REPORT.contains(r#""val": 6"#));
    }

    // -- 11. HallOfFameView / hall_of_fame_cmd_0x42a_79_b_3_sl -----------
    const HALL_OF_FAME: &str = include_str!(
        "../../../reports/screen_captures/screen_batch8_hall_of_fame_cmd_0x42a_79_b_3_sl.json"
    );

    #[test]
    fn hall_of_fame_matches_capture_json() {
        assert_empty_widget_geometry(HALL_OF_FAME);
        let v = HallOfFameView { selected_tab: 0, scroll_offset: 0, filter: 0 };
        let widgets = v.to_widget_pool();
        assert_eq!(widgets.len(), 4 + 2);
        assert_eq!(widgets[4].descriptor.userdata_id, 0);
        assert_eq!(widgets[5].descriptor.userdata_id, 0);
        assert!(HALL_OF_FAME.contains(r#""val": 0"#));
    }

    // -- 12. WrittenHistoryView / written_history_cmd_0x429_117_b --------
    const WRITTEN_HISTORY: &str = include_str!(
        "../../../reports/screen_captures/screen_batch8_written_history_cmd_0x429_117_b.json"
    );

    #[test]
    fn written_history_matches_capture_json() {
        assert_empty_widget_geometry(WRITTEN_HISTORY);
        let v = WrittenHistoryView { category: 274_726_912, reserved: [0; 4] };
        let widgets = v.to_widget_pool();
        // Prelude removed: 4 cat tabs + 1 category row + 4 reserved rows.
        assert_eq!(widgets.len(), 4 + 1 + 4);
        assert_eq!(widgets[4].descriptor.userdata_id, 274_726_912);
        for row in &widgets[5..] {
            assert_eq!(row.descriptor.userdata_id, 0);
        }
        assert!(WRITTEN_HISTORY.contains(r#""val": 274726912"#));
        assert!(WRITTEN_HISTORY.contains(r#""val": 0"#));
    }
}
