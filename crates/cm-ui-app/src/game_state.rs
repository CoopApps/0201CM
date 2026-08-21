//! Cross-screen game-setup state.
//!
//! The user's choices carry across screens: which leagues to load (Select League(s)),
//! which start season (Select Start Season). This is the Rust equivalent of what the
//! C game accumulates on the setup-callback state globals before firing the world
//! loader (FUN_005121a0) and the runtime-engine setup (FUN_008120d0). Everything is
//! purely in-memory / immutable-after-choose; nothing here writes to disk.

/// One slot in the Select-League(s) picker.
///
/// Mirrors the exe's 72-byte slot layout at `DAT_00b4bc70` (34 slots total). Each
/// slot holds pointers to two competition records (primary + secondary/reserve) plus
/// a `+0x11c` flags byte on the primary record: bit1 = SELECTED, bit2 = "extra"
/// mode (secondary pool active). The draw callback `FUN_008055e0` reads this table;
/// the event handler `FUN_00806640` writes the flags byte on user click.
///
/// **Data source note**: which 34 competitions occupy the picker is decided by the
/// 34 setup handlers `LAB_0081a120..LAB_00821b50` (uncatalogued in Ghidra —
/// force-disassembled). Slot names are lifted from the `create_XXX_comps()`
/// `__FUNCTION__` string baked into each handler; 4 slots that had a copy-pasted
/// string are marked `SlotConfidence::Inferred`. See `real_34_slots()` below.
/// Real competition IDs are not yet extracted (they're built via runtime-populated
/// `DAT_009bbXXX` indices) — that's the reason `primary_comp_id` starts `None`.
#[derive(Debug, Clone)]
pub struct PickerSlot {
    /// Slot index 0..33 — the order the picker draws them (before sort).
    pub index: u8,
    /// Primary competition id (from parsed `club_comp.dat`). None when the primary
    /// pointer in the exe's slot is null — an uncommon degenerate case.
    pub primary_comp_id: Option<u32>,
    /// Primary competition long name — shown as the row label. Preformatted with
    /// the "  %s" left-pad the exe uses (format string `0x975068`).
    pub primary_name: String,
    /// Secondary competition id — present only when the slot has a secondary/reserve
    /// pool. Toggling this is a separate event (0xe) in the exe.
    pub secondary_comp_id: Option<u32>,
    /// bit1 of primary_rec + 0x11c: whether the primary league is included (SELECTED).
    pub selected: bool,
    /// bit2 of the same flags byte: whether the extra/reserve pool is enabled.
    pub extra: bool,
    /// Whether the row shows the "human controlled" indicator column (event 0x38).
    /// Sourced from `FUN_006535f0(rec, 0x7d0)` in the exe.
    pub human_controlled: bool,
    /// Whether the user has clicked the BACKGROUND button on this row. In the exe
    /// this is an independent toggle from `selected`: clicking either button
    /// highlights that button; clicking again un-highlights. The two are
    /// semantically mutually exclusive as game state (a league is either
    /// SELECTED, background-only, or off), but the UI treats each button click
    /// as an independent toggle from the user's perspective.
    pub background_marker: bool,
}

/// The two toggles that live above the picker. Command codes from the exe's event
/// handler `FUN_00806640` are: 0x39/0x3a = Use Real Players Yes/No; 0x3b/0x3c =
/// Attribute Masking Yes/No. The exe's draw callback SKIPS the whole masking group
/// when Real Players is No — replicate that.
#[derive(Debug, Clone, Copy)]
pub struct LoadOptions {
    pub use_real_players: bool,
    pub attribute_masking: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        // The exe defaults both to Yes on entry (Real Players first-run behaviour).
        Self { use_real_players: true, attribute_masking: true }
    }
}

/// State of the Select League(s) screen.
///
/// Owns the 34 picker slots and the two toggle bytes. Every field maps to a real
/// exe global (documented on each field), so this struct is the port equivalent
/// of what the exe carries across renders/clicks.
#[derive(Debug, Clone)]
pub struct SelectLeaguesState {
    /// Port of `DAT_00b4bc70` — the 34 picker slots. Fixed size 34 (exe uses
    /// `for i in 0..0x22` — 0x22 = 34).
    pub slots: Vec<PickerSlot>,
    /// Sort/display order — port of `DAT_00b4bc48` (34-byte order array, sorted
    /// by `FUN_006537a0`). Default: identity 0..34.
    pub order: Vec<u8>,
    pub options: LoadOptions,
    /// `DAT_00dbc3f0`: 0 during picking, non-zero after Next commits (switches
    /// the screen title to "Selected League(s)" and disables toggles). Kept
    /// for parity; we only enter this state when the user hits Next.
    pub post_selection: bool,
}

impl SelectLeaguesState {
    pub fn from_slots(slots: Vec<PickerSlot>) -> Self {
        let order = (0..slots.len() as u8).collect();
        Self { slots, order, options: LoadOptions::default(), post_selection: false }
    }

    /// Count of slots with the primary SELECTED bit set — port of `DAT_00acdf04`.
    pub fn selected_count(&self) -> usize {
        self.slots.iter().filter(|s| s.selected).count()
    }

    /// Toggle the primary SELECTED bit of the slot with this index (event 0xd).
    pub fn toggle_primary(&mut self, index: u8) {
        if let Some(s) = self.slots.iter_mut().find(|s| s.index == index) {
            s.selected = !s.selected;
        }
    }

    /// Toggle the secondary/extra bit (event 0xe).
    pub fn toggle_secondary(&mut self, index: u8) {
        if let Some(s) = self.slots.iter_mut().find(|s| s.index == index) {
            s.extra = !s.extra;
        }
    }

    /// Toggle the human-controlled marker (event 0x38).
    pub fn toggle_human(&mut self, index: u8) {
        if let Some(s) = self.slots.iter_mut().find(|s| s.index == index) {
            s.human_controlled = !s.human_controlled;
        }
    }

    /// Toggle the row's BACKGROUND-cell highlight (col 4 in the picker).
    /// Independent of `selected` per the exe's UI: each button is its own
    /// user-facing toggle. Semantically the two are mutually exclusive as
    /// game state (a league is either selected, background-only, or off)
    /// but the visual toggles are separate as the user described.
    pub fn toggle_background_marker(&mut self, index: u8) {
        if let Some(s) = self.slots.iter_mut().find(|s| s.index == index) {
            s.background_marker = !s.background_marker;
        }
    }

    /// Event 0xb: Select All. Enabled in the exe only when not post-selection.
    pub fn select_all(&mut self) {
        for s in &mut self.slots {
            s.selected = true;
        }
    }

    /// Event 0xa: De-Select All. In the exe: gated behind `selected_count >= 15`;
    /// keep the same gate here.
    pub fn deselect_all(&mut self) {
        if self.selected_count() >= 15 {
            for s in &mut self.slots {
                s.selected = false;
                s.extra = false;
            }
        }
    }
}

/// One row in the Select Start Season list. In the exe (`FUN_00808ae0`) each row is
/// a runtime-populated season record iterated from a list at `DAT_00b5d02c` — three
/// columns per row: `[year] [manager-status] [start-mode-label]`. Start-mode has a
/// three-arm cascade at 0x8090bc/0x8091d1/0x80929c: `NewCareer` (fresh) /
/// `ContinueSave` (loaded save) / `Network` (multiplayer host).
#[derive(Debug, Clone)]
pub struct SeasonRow {
    pub year_label: String,
    pub manager_status: String,
    pub start_mode: StartMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartMode {
    /// The three-arm cascade's first branch (fresh career start).
    NewCareer,
    /// Second branch — resuming a save game the user just loaded.
    ContinueSave,
    /// Third branch — host of a network game.
    Network,
}

impl StartMode {
    pub fn label(self) -> &'static str {
        match self {
            StartMode::NewCareer => "New Career",
            StartMode::ContinueSave => "Continue Save",
            StartMode::Network => "Network Host",
        }
    }
}

/// Select Start Season screen state. In the exe the season list is populated by
/// the Select-Leagues handler before this screen opens. For a fresh install with
/// no save loaded the list collapses to exactly one row: `"2001/02"` with mode
/// `NewCareer` — that's the shipped-CM01/02 behaviour and what we ship by default.
#[derive(Debug, Clone)]
pub struct StartSeasonState {
    pub rows: Vec<SeasonRow>,
    pub selected: usize,
}

impl Default for StartSeasonState {
    fn default() -> Self {
        Self {
            rows: vec![SeasonRow {
                year_label: "2001/02".to_string(),
                manager_status: String::new(),
                start_mode: StartMode::NewCareer,
            }],
            selected: 0,
        }
    }
}

/// Whether a country runs a calendar-year season (one box reads e.g.
/// "Finland 2002") vs a split-year season ("England 01/02"). Ported from
/// the branch at FUN_00807280:0x8073c8 which tests the slot's split-season
/// flag; the country list is verified against the shipped `.dat` calendar
/// (see [[league-dates-and-comp-wiring]]).
pub fn is_calendar_year_season(country: &str) -> bool {
    matches!(
        country,
        "Argentina" | "Brazil" | "Finland" | "Ireland" | "Japan" | "Norway" | "Sweden" | "USA"
    )
}

/// Format one Start-Season box label the way the exe does:
///   split path   (0x8073fe): sprintf("%s %02d/%02d", name, y, y+1)
///   calendar path(0x80741c): sprintf("%s %d", name, full_year)
/// `base_year_2digit` = 1 for the shipped 2001/02 season.
pub fn season_label(country: &str, base_year_2digit: u32) -> String {
    if is_calendar_year_season(country) {
        format!("{country} {}", 2000 + base_year_2digit + 1)
    } else {
        let y = base_year_2digit;
        format!("{country} {y:02}/{:02}", (y + 1) % 100)
    }
}

impl StartSeasonState {
    /// Build the Start-Season list from the leagues the user selected: one
    /// box per selected league, labelled with the country + its season
    /// format. Order follows the picker's slot order. Empty selection is
    /// impossible here (Next is gated on selected_count > 0).
    pub fn from_leagues(leagues: &SelectLeaguesState) -> Self {
        let rows: Vec<SeasonRow> = leagues
            .slots
            .iter()
            .filter(|s| s.selected)
            .map(|s| SeasonRow {
                year_label: season_label(&s.primary_name, 1),
                manager_status: String::new(),
                start_mode: StartMode::NewCareer,
            })
            .collect();
        let rows = if rows.is_empty() {
            StartSeasonState::default().rows
        } else {
            rows
        };
        Self { rows, selected: 0 }
    }

    pub fn selected_label(&self) -> &str {
        &self.rows[self.selected].year_label
    }
}

/// The country picker slots — one row per playable nation.
///
/// **Correction from an earlier wrong list.** The LAB handler trace found 34
/// setup handlers at `LAB_0081a120..LAB_00821b50`. Only ~26 of them populate a
/// picker row at `DAT_00b4bc70`; the other 8 create *background* competitions
/// (World Cup, European Club, etc.) — the exe's picker loop
/// `for i in 0..0x22: if rec == 0: continue` iterates 34 slots but skips those
/// with a null `rec`, so those international slots never render. I had them in
/// the picker before; the game screenshot proves they don't belong here.
///
/// The 26 countries below are the ones with `_comps()` handlers whose called
/// functions push country-prefix `.cpp` filenames (argentine, australian,
/// belgian, bra_*, croatian, danish, english, finnish, french, german, gre_*,
/// dutch, irish (mis-labelled "italian" in `__FUNCTION__`), italian, japanese,
/// northern_irish, norwegian, pol_*, portuguese, russian, scottish, spanish,
/// swedish, tur_*, united_states, wel_*). All HIGH confidence from the file
/// paths embedded in each handler's `__FILE__` pushes.
///
/// `primary_comp_id` remains `None`: the actual competition-record pointer per
/// slot is written by each handler's called constructors, using runtime pool
/// indices that can't be extracted statically. That's a separate trace.
pub fn real_picker_slots() -> Vec<PickerSlot> {
    let entries: [&str; 26] = [
        "Argentina", "Australia", "Belgium", "Brazil", "Croatia", "Denmark",
        "England", "Finland", "France", "Germany", "Greece", "Holland",
        "Ireland", "Italy", "Japan", "Northern Ireland", "Norway", "Poland",
        "Portugal", "Russia", "Scotland", "Spain", "Sweden", "Turkey",
        "USA", "Wales",
    ];
    entries
        .iter()
        .enumerate()
        .map(|(i, &name)| PickerSlot {
            index: i as u8,
            primary_comp_id: None,
            primary_name: name.to_string(),
            secondary_comp_id: None,
            selected: false,
            extra: false,
            human_controlled: false,
            background_marker: false,
        })
        .collect()
}

/// Alias so existing test/call sites (using either name) resolve.
pub fn stub_34_slots() -> Vec<PickerSlot> {
    real_picker_slots()
}
pub fn real_34_slots() -> Vec<PickerSlot> {
    real_picker_slots()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_has_26_country_slots() {
        // The exe iterates 0..0x22=34 in the picker loop but skips slots with a
        // null `rec`. The 8 international-comp handlers don't populate a picker
        // row — so 26 country slots are what the user actually sees.
        assert_eq!(real_picker_slots().len(), 26);
    }

    #[test]
    fn toggle_flips_and_flips_back() {
        let mut s = SelectLeaguesState::from_slots(stub_34_slots());
        let start = s.selected_count();
        s.toggle_primary(3);
        assert_eq!(s.selected_count(), start + 1);
        s.toggle_primary(3);
        assert_eq!(s.selected_count(), start);
    }

    #[test]
    fn deselect_all_is_gated_at_15() {
        let mut s = SelectLeaguesState::from_slots(stub_34_slots());
        // Under 15 selected: deselect_all should be a no-op (exe gate).
        for i in 0..10 {
            s.toggle_primary(i);
        }
        assert_eq!(s.selected_count(), 10);
        s.deselect_all();
        assert_eq!(s.selected_count(), 10, "gate keeps state under threshold");
        // At/above 15: deselect_all clears.
        for i in 10..20 {
            s.toggle_primary(i);
        }
        assert_eq!(s.selected_count(), 20);
        s.deselect_all();
        assert_eq!(s.selected_count(), 0, "above threshold, deselect clears");
    }
}
