//! Sidebar message ids — direct port of the dispatch table from
//! `FUN_00745540` (sidebar composer). Every `msg_id` the sidebar emits
//! when a user clicks a row is one of these values.
//!
//! Decompile: `d:/cm0102-carve/decompiled/gui_layout_engine/0x00745540.c`
//! (53 KB — see [`reports/gui_layout_engine_decode.md`](../../../../reports/gui_layout_engine_decode.md) §7).
//!
//! Values are the raw integer the exe stores at `Widget.descriptor.msg_id`;
//! the top-level message dispatcher (`FUN_007491E0` / `FUN_0074BF60`) then
//! routes them to the matching screen builder.

/// The screen or action selected by a sidebar row.
///
/// Numeric values match the exe's msg_ids exactly — used as the
/// `WidgetDescriptor.msg_id` field and by the dispatch dispatchers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum SidebarMsg {
    /// No-op / non-clickable label.
    None = 0,

    // ----- Language toggles (negative sentinels) -----
    /// -2 — switch to language A.
    LangA = -2,
    /// -3 — switch to language B.
    LangB = -3,

    // ----- Manager / staff -----
    Staff              = 0x3E9,   // 1001
    PlayerStaffSearch  = 0x3EB,   // 1003
    ManagerHistory     = 0x3EC,   // 1004
    GoOnHoliday        = 0x3EF,   // 1007
    EndHoliday         = 0x3F0,   // 1008
    Retire             = 0x3F2,   // 1010
    FifaRankings       = 0x3F3,   // 1011
    AddManager         = 0x3FB,   // 1019

    // ----- Game control -----
    ExitGame           = 0x402,   // 1026
    UefaCoefficients   = 0x40C,   // 1036
    SendAbuse          = 0x414,   // 1044
    Transfers          = 0x415,   // 1045
    LatestScores       = 0x418,   // 1048
    ResignFromClub     = 0x41C,   // 1052
    ResignFromNation   = 0x41D,   // 1053
    JobInformation     = 0x41E,   // 1054
    BoardConfidence    = 0x41F,   // 1055
    FaConfidence       = 0x420,   // 1056
    SwitchManager      = 0x421,   // 1057
    ControlTeamToggle  = 0x425,   // 1061
    HolidayCountdown   = 0x42D,   // 1069
    ManagerStats       = 0x42E,   // 1070
    RestartGame        = 0x42F,   // 1071
    ControlNationToggle= 0x433,   // 1075
    ComparePlayers     = 0x434,   // 1076

    // ----- Squad / awards deep-links -----
    Awards             = 0x7D4,   // 2004
    Squad              = 0x7D5,   // 2005
    ReservesOrBSquad   = 0x7D6,   // 2006

    // ----- Match / competition -----
    /// 1000 — continue / play match (primary game-step).
    ContinueGame       = 1000,
    /// 2000 — go to competition (userdata_id = comp*).
    GotoCompetition    = 2000,
}

impl SidebarMsg {
    /// Try to decode an exe-observed msg_id into a known variant.
    /// Unknown ids return `None`.
    pub fn from_exe(msg_id: i32) -> Option<Self> {
        Some(match msg_id {
            0     => Self::None,
            -2    => Self::LangA,
            -3    => Self::LangB,
            0x3E9 => Self::Staff,
            0x3EB => Self::PlayerStaffSearch,
            0x3EC => Self::ManagerHistory,
            0x3EF => Self::GoOnHoliday,
            0x3F0 => Self::EndHoliday,
            0x3F2 => Self::Retire,
            0x3F3 => Self::FifaRankings,
            0x3FB => Self::AddManager,
            0x402 => Self::ExitGame,
            0x40C => Self::UefaCoefficients,
            0x414 => Self::SendAbuse,
            0x415 => Self::Transfers,
            0x418 => Self::LatestScores,
            0x41C => Self::ResignFromClub,
            0x41D => Self::ResignFromNation,
            0x41E => Self::JobInformation,
            0x41F => Self::BoardConfidence,
            0x420 => Self::FaConfidence,
            0x421 => Self::SwitchManager,
            0x425 => Self::ControlTeamToggle,
            0x42D => Self::HolidayCountdown,
            0x42E => Self::ManagerStats,
            0x42F => Self::RestartGame,
            0x433 => Self::ControlNationToggle,
            0x434 => Self::ComparePlayers,
            0x7D4 => Self::Awards,
            0x7D5 => Self::Squad,
            0x7D6 => Self::ReservesOrBSquad,
            1000  => Self::ContinueGame,
            2000  => Self::GotoCompetition,
            _     => return None,
        })
    }

    /// The exe msg_id (i32).
    pub fn as_exe(self) -> i32 { self as i32 }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_variants_round_trip_through_exe_id() {
        for m in [
            SidebarMsg::None,
            SidebarMsg::LangA, SidebarMsg::LangB,
            SidebarMsg::Staff, SidebarMsg::PlayerStaffSearch,
            SidebarMsg::ManagerHistory, SidebarMsg::GoOnHoliday,
            SidebarMsg::EndHoliday, SidebarMsg::Retire,
            SidebarMsg::FifaRankings, SidebarMsg::AddManager,
            SidebarMsg::ExitGame, SidebarMsg::UefaCoefficients,
            SidebarMsg::SendAbuse, SidebarMsg::Transfers,
            SidebarMsg::LatestScores, SidebarMsg::ResignFromClub,
            SidebarMsg::ResignFromNation, SidebarMsg::JobInformation,
            SidebarMsg::BoardConfidence, SidebarMsg::FaConfidence,
            SidebarMsg::SwitchManager, SidebarMsg::ControlTeamToggle,
            SidebarMsg::HolidayCountdown, SidebarMsg::ManagerStats,
            SidebarMsg::RestartGame, SidebarMsg::ControlNationToggle,
            SidebarMsg::ComparePlayers, SidebarMsg::Awards,
            SidebarMsg::Squad, SidebarMsg::ReservesOrBSquad,
            SidebarMsg::ContinueGame, SidebarMsg::GotoCompetition,
        ] {
            assert_eq!(SidebarMsg::from_exe(m.as_exe()), Some(m), "round-trip {m:?}");
        }
    }

    #[test]
    fn unknown_msg_returns_none() {
        assert_eq!(SidebarMsg::from_exe(0x999), None);
        assert_eq!(SidebarMsg::from_exe(0x1234), None);
    }

    #[test]
    fn exe_constants_match_hex_literals_in_decode_report() {
        // Spot-checks against the msg-id table in reports/gui_layout_engine_decode.md §7.
        assert_eq!(SidebarMsg::Transfers as i32, 0x415);
        assert_eq!(SidebarMsg::Squad as i32, 0x7D5);
        assert_eq!(SidebarMsg::ContinueGame as i32, 1000);
        assert_eq!(SidebarMsg::GotoCompetition as i32, 2000);
    }
}
