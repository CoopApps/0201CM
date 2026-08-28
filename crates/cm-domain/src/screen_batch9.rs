//! Batch 9: Club-context-menu builder (FUN_00487210, 1107 B).
//!
//! Called from the club toolbar when the user right-clicks a club to
//! show the context menu. Emits at most ONE entry from the following
//! priority-ordered set (`ContextEntry`), then wires it up via
//! `FUN_00549580(2, 0x294, 4, 0x311, 0x18, 0, 0, 0x30, param_3, param_3,
//! 0xC, 1, param_2, label, 0, ACTION_CODE, PAYLOAD, -1)`.
//!
//! The decision tree branches on whether a human is currently seated
//! (FUN_00822580 == 0 = no-active-human path, != 0 = active-human path):
//!
//! * No active human:
//!   * Scout Club (0x6A) if the club is scout-eligible + not already scouted
//!   * Invite Club (0x66) — matches prev-club "invite" invitation
//!   * Invite Club (0x67) — matches nation-invite
//!   * Uninvite Club (0x68) or Invite Club (0x69) — via clubs-shortlist
//!   * Apply for Job (0x65) — if unmanageable + manageable-by-user
//! * Active human:
//!   * Take Control (0x64) — if club is not-current-user + manageable
//!
//! Same pattern as the exe: only the FIRST matching branch fires.

use serde::{Deserialize, Serialize};

/// Action codes emitted as the 16th arg to `FUN_00549580` (context-menu
/// wire-up) — matches exe hex literals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u16)]
pub enum ContextAction {
    /// 0x64 (100) — take control of this club as active human.
    TakeControl = 0x64,
    /// 0x65 — apply for the manager job at this club.
    ApplyForJob = 0x65,
    /// 0x66 — accept a prev-club "invite" pending invitation.
    InviteClubPrev = 0x66,
    /// 0x67 — accept a nation-invite pending invitation.
    InviteClubNation = 0x67,
    /// 0x68 — remove this club from the "clubs to invite" shortlist.
    UninviteClub = 0x68,
    /// 0x69 — add this club to the "clubs to invite" shortlist.
    InviteClubShortlist = 0x69,
    /// 0x6A — mark this club to be scouted.
    ScoutClub = 0x6A,
}

impl ContextAction {
    /// String label the exe pulls via `FUN_006547C0` for each action.
    pub fn label(self) -> &'static str {
        match self {
            Self::TakeControl        => "Take Control",
            Self::ApplyForJob        => "Apply for Job",
            Self::InviteClubPrev
            | Self::InviteClubNation
            | Self::InviteClubShortlist => "Invite Club",
            Self::UninviteClub       => "Uninvite Club",
            Self::ScoutClub          => "Scout Club",
        }
    }
}

/// Result of running the club context-menu builder — either "no menu"
/// (all predicates failed) or one selected [`ContextAction`] with its
/// payload pointer (opaque, matches exe's 17th arg).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClubContextEntry {
    pub action: ContextAction,
    /// Opaque payload — the exe passes the invite/shortlist pointer
    /// here so the action handler can dereference it.
    pub payload: u32,
}

/// Predicates for one context-menu decision. Every field is derived
/// from the exe by name; a `bool` matches the exe's `iVar != 0`
/// pattern, a `u32` matches an opaque record pointer.
#[derive(Debug, Clone, Copy, Default)]
pub struct ClubContextInputs {
    /// Whether an active human is currently seated
    /// (`FUN_00822580() != 0`).
    pub active_human_seated: bool,

    // --- Active-human path (FUN_005ea590) ---
    /// True when the club is manageable by the active human
    /// (`FUN_005ea590 == 0` + `FUN_0052e370 != 0`).
    pub takeable_by_active_human: bool,

    // --- No-active-human path ---
    /// True when the club is scout-eligible
    /// (`FUN_007e05c0() != 0` + `FUN_007e06a0(club) == 0` +
    /// `FUN_00525450(club) == 0`).
    pub scout_eligible: bool,
    /// Prev-club invite record (`FUN_005b0b70()`); nonzero when a
    /// pending "invite from previous club" applies.
    pub invite_prev_record: u32,
    /// True when the prev-club invite matches the target club's
    /// division-status (both league or both non-league).
    pub invite_prev_matches: bool,
    /// Nation-invite record (`FUN_005b0be0()`); nonzero when a pending
    /// nation invitation targets this specific club.
    pub invite_nation_record: u32,
    /// True when the nation invite matches the target club's
    /// division-status.
    pub invite_nation_matches: bool,
    /// Clubs-shortlist record (`FUN_005b0b00()`); nonzero when any
    /// shortlist entry exists.
    pub shortlist_record: u32,
    /// True when the shortlist and target club match on division.
    pub shortlist_matches: bool,
    /// True when the target club is ALREADY on the shortlist (Uninvite
    /// path). When false and `shortlist_matches`, InviteClubShortlist
    /// is chosen instead.
    pub club_on_shortlist: bool,
    /// True when the club is manageable + not currently managed
    /// (`FUN_005265e0 == 0` + `FUN_0052e370 != 0`) — the "Apply for
    /// Job" gate.
    pub apply_eligible: bool,
}

/// Direct port of `FUN_00487210(club, param_2, param_3)`. Returns
/// `None` when no menu entry applies. Priority ordering matches the
/// exe's top-to-bottom `if` cascade.
pub fn build_club_context_entry(inp: ClubContextInputs) -> Option<ClubContextEntry> {
    if inp.active_human_seated {
        // Active-human path: only Take Control candidate.
        if inp.takeable_by_active_human {
            return Some(ClubContextEntry { action: ContextAction::TakeControl, payload: 0 });
        }
        return None;
    }
    // No-active-human path — 5 candidates in strict order.
    if inp.scout_eligible {
        return Some(ClubContextEntry { action: ContextAction::ScoutClub, payload: 0 });
    }
    if inp.invite_prev_record != 0 && inp.invite_prev_matches {
        return Some(ClubContextEntry {
            action: ContextAction::InviteClubPrev,
            payload: inp.invite_prev_record,
        });
    }
    if inp.invite_nation_record != 0 && inp.invite_nation_matches {
        return Some(ClubContextEntry {
            action: ContextAction::InviteClubNation,
            payload: inp.invite_nation_record,
        });
    }
    if inp.shortlist_record != 0 && inp.shortlist_matches {
        let action = if inp.club_on_shortlist {
            ContextAction::UninviteClub
        } else {
            ContextAction::InviteClubShortlist
        };
        return Some(ClubContextEntry { action, payload: inp.shortlist_record });
    }
    if inp.apply_eligible {
        return Some(ClubContextEntry { action: ContextAction::ApplyForJob, payload: 0 });
    }
    None
}

// =====================================================================
// World-populator companions
// =====================================================================

use crate::world_pools::WorldPools;

/// Populate a Club-Context-menu entry for the given target club.
///
/// The exe's real predicates need decoded club/nation/scout tables
/// (FUN_007e05c0 scout eligibility, FUN_005b0b70 prev-club invite,
/// FUN_005b0be0 nation invite, FUN_005b0b00 shortlist,
/// FUN_005265e0/FUN_005ea590 manageability). The facade doesn't carry
/// any of those tables yet, so this populator only makes the ONE
/// world-observable decision it can: `active_human_seated ==
/// active_human_seat.is_some()`. All other predicates default to false
/// — which selects the exe's "no menu applies" branch until those
/// tables land on the facade.
pub fn populate_club_context(
    pools: &WorldPools<'_>,
    _target_club_id: u32,
) -> Option<ClubContextEntry> {
    let inp = ClubContextInputs {
        active_human_seated: pools.active_human_seat.is_some(),
        ..Default::default()
    };
    build_club_context_entry(inp)
}

#[cfg(test)]
#[allow(dead_code)]
const TODO_POPULATOR_INFO: &str = "\
    Every ClubContextInputs predicate beyond `active_human_seated` needs \
    a decoded pool that isn't on the facade yet: scout-registry \
    (FUN_007e05c0/00525450), invite-tables (FUN_005b0b70/be0/b00), and \
    manageability gates (FUN_005265e0, FUN_005ea590, FUN_0052e370). \
    Until those land, the populator can only emit None or (with the \
    right eligibility on the facade) a TakeControl entry.";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_human_only_take_control_offered() {
        let inp = ClubContextInputs {
            active_human_seated: true,
            takeable_by_active_human: true,
            // all no-active-human predicates true → still ignored
            scout_eligible: true,
            invite_prev_record: 42, invite_prev_matches: true,
            apply_eligible: true,
            ..Default::default()
        };
        let e = build_club_context_entry(inp).unwrap();
        assert_eq!(e.action, ContextAction::TakeControl);
    }

    #[test]
    fn active_human_untakeable_yields_nothing() {
        let inp = ClubContextInputs { active_human_seated: true, ..Default::default() };
        assert!(build_club_context_entry(inp).is_none());
    }

    #[test]
    fn no_human_scout_priority_1() {
        let inp = ClubContextInputs {
            scout_eligible: true,
            invite_prev_record: 42, invite_prev_matches: true,
            apply_eligible: true,
            ..Default::default()
        };
        let e = build_club_context_entry(inp).unwrap();
        assert_eq!(e.action, ContextAction::ScoutClub);
    }

    #[test]
    fn no_human_prev_invite_priority_2() {
        let inp = ClubContextInputs {
            invite_prev_record: 99, invite_prev_matches: true,
            invite_nation_record: 100, invite_nation_matches: true,
            apply_eligible: true,
            ..Default::default()
        };
        let e = build_club_context_entry(inp).unwrap();
        assert_eq!(e.action, ContextAction::InviteClubPrev);
        assert_eq!(e.payload, 99);
    }

    #[test]
    fn no_human_prev_invite_ignored_when_not_matching() {
        let inp = ClubContextInputs {
            invite_prev_record: 99, invite_prev_matches: false,   // rec != 0 but mismatch
            invite_nation_record: 100, invite_nation_matches: true,
            ..Default::default()
        };
        let e = build_club_context_entry(inp).unwrap();
        assert_eq!(e.action, ContextAction::InviteClubNation);
    }

    #[test]
    fn shortlist_toggle_uninvite_vs_invite() {
        let base = ClubContextInputs {
            shortlist_record: 55, shortlist_matches: true,
            ..Default::default()
        };
        let on = ClubContextInputs { club_on_shortlist: true, ..base };
        assert_eq!(build_club_context_entry(on).unwrap().action, ContextAction::UninviteClub);
        let off = ClubContextInputs { club_on_shortlist: false, ..base };
        assert_eq!(build_club_context_entry(off).unwrap().action,
                   ContextAction::InviteClubShortlist);
    }

    #[test]
    fn apply_for_job_is_lowest_priority() {
        let e = build_club_context_entry(ClubContextInputs {
            apply_eligible: true, ..Default::default()
        }).unwrap();
        assert_eq!(e.action, ContextAction::ApplyForJob);
    }

    #[test]
    fn no_predicates_yields_none() {
        assert!(build_club_context_entry(ClubContextInputs::default()).is_none());
    }

    #[test]
    fn action_codes_match_exe_hex() {
        assert_eq!(ContextAction::TakeControl as u16,       0x64);
        assert_eq!(ContextAction::ApplyForJob as u16,       0x65);
        assert_eq!(ContextAction::InviteClubPrev as u16,    0x66);
        assert_eq!(ContextAction::InviteClubNation as u16,  0x67);
        assert_eq!(ContextAction::UninviteClub as u16,      0x68);
        assert_eq!(ContextAction::InviteClubShortlist as u16, 0x69);
        assert_eq!(ContextAction::ScoutClub as u16,         0x6A);
    }

    #[test]
    fn labels_match_exe_strings() {
        assert_eq!(ContextAction::TakeControl.label(),  "Take Control");
        assert_eq!(ContextAction::ScoutClub.label(),    "Scout Club");
        assert_eq!(ContextAction::UninviteClub.label(), "Uninvite Club");
        assert_eq!(ContextAction::InviteClubPrev.label(),      "Invite Club");
        assert_eq!(ContextAction::InviteClubNation.label(),    "Invite Club");
        assert_eq!(ContextAction::InviteClubShortlist.label(), "Invite Club");
        assert_eq!(ContextAction::ApplyForJob.label(),  "Apply for Job");
    }

    // --- Populators ---
    use crate::world_pools::WorldPools;

    #[test]
    fn populate_context_no_seat_yields_none() {
        // No seat + no other predicates → exe's no-menu branch.
        assert!(populate_club_context(&WorldPools::empty(), 42).is_none());
    }

    #[test]
    fn populate_context_seated_but_untakeable_yields_none() {
        // Seat present but takeable_by_active_human predicate isn't on
        // the facade — the active-human path returns None per the exe.
        let pools = WorldPools { active_human_seat: Some(7), ..WorldPools::empty() };
        assert!(populate_club_context(&pools, 42).is_none());
    }
}

// =====================================================================
// world-pools populator companions (WorldPools-based, wave-A `_from_pools`)
// =====================================================================

#[allow(dead_code)]
const TODO_POPULATOR_INFO_POOLS: &str = "\
    b9 pool sources: every ClubContextInputs predicate beyond \
    `active_human_seated` needs a decoded pool that isn't on the pools \
    facade yet: scout-registry (FUN_007e05c0/00525450), invite-tables \
    (FUN_005b0b70/be0/b00), and manageability gates (FUN_005265e0, \
    FUN_005ea590, FUN_0052e370). Until those land, the populator can \
    only emit None or (with the right eligibility) a TakeControl entry.";

pub fn populate_club_context_from_pools(
    pools: &WorldPools<'_>,
    target_club_id: u32,
) -> Option<ClubContextEntry> {
    populate_club_context(pools, target_club_id)
}

#[cfg(test)]
mod pools_populate_tests {
    use super::*;

    #[test]
    fn empty_pools_yield_none() {
        assert!(populate_club_context_from_pools(&WorldPools::empty(), 42).is_none());
    }

    #[test]
    fn seated_but_untakeable_still_none() {
        let pools = WorldPools { active_human_seat: Some(7), ..WorldPools::empty() };
        assert!(populate_club_context_from_pools(&pools, 42).is_none());
    }
}
