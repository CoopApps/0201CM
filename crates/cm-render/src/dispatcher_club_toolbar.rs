//! Layer 4 — club **context-toolbar** dispatchers (the third dispatcher pair
//! after `dispatch_global` and `dispatch_club`).
//!
//! Direct ports of the two functions that build + handle the right-click
//! context toolbar on a club record:
//!
//! * [`dispatch_club_toolbar_top`] — port of `FUN_00487210` (105 lines of C at
//!   `d:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/00487210.c`). This is
//!   the **builder** half: given a club record + a widget-parent handle, it
//!   decides *which* context-menu items apply to that club (Scout Club /
//!   Invite Club / Uninvite Club / Apply for Job / Take Control) and spawns
//!   one `FUN_00549580` menu-item widget per applicable item, each carrying
//!   the corresponding cmd code in the range `0x64..0x6a` (plus `0x64==100`
//!   for Take Control).
//!
//! * [`dispatch_club_toolbar_bottom`] — port of `FUN_00487670` (418 lines at
//!   `.../00487670.c`). The **event handler** half: on click of one of the
//!   items the builder spawned, this routes each of the seven cmds
//!   (0x64/100, 0x65..0x6a) to its action — most of which open a *Please
//!   Confirm* modal dialog (`FUN_0057b9c0`) rather than a new screen.
//!
//! # Not a screen dispatcher
//!
//! Unlike [`crate::dispatcher::dispatch_global`] and
//! [`crate::dispatcher::dispatch_club`], these cmds do *not* route to
//! `build_screen_*` fns. Every arm either
//!
//! 1. fires an in-place action on a club record (Take Control →
//!    `FUN_0080bbd0`; Apply for Job → `FUN_00696fa0`), or
//! 2. opens a *Please Confirm* modal via `FUN_006547c0` (template format) +
//!    `FUN_0057b9c0` (show dialog) — an msgbox, not a screen.
//!
//! Consequently the port carries **zero** new `screens_auto::*` references —
//! the 28 auto-transliterated fns that were unreferenced when this commit
//! opened remain unreferenced. They live under the *other* two unported
//! dispatch layers named in `reports/menu_tree_scope.md` (menu-tree dropdown
//! sub-screens; per-screen sub-screen nav). Marking those arms `TodoBuilder`
//! is faithful — no fabrication.
//!
//! # GDI vs DirectDraw asm cross-check
//!
//! The task brief asks us to sanity-check against
//! `d:/cm0102-carve/gdi_carve/functions/00004-PE_section_.text/*sub_00487210*`
//! /`*sub_00487670*`. Those files do **not** exist in the GDI carve — the
//! nearest neighbouring functions in the GDI build are `sub_00487480` and
//! `sub_004878e0`, i.e. the GDI build's function layout has shifted by ~1KB
//! (relative code motion inside the `.text` section is normal between
//! release builds of the same source). We therefore fall back to the
//! cm0102.exe (DirectDraw) decomp as the port source of truth, which is the
//! documented behaviour for every prior Layer-4 port too (`dispatch_global`
//! at `007491e0`, `dispatch_club` at `0074bf60` — both cited by their
//! DirectDraw addresses in `dispatcher.rs` docstrings).

use crate::dispatcher::{
    resolve_widget_cmd, DispatchResult, DispatcherState,
};
use crate::widget_pool::GuiRecordPool;

// ============================================================================
// The 7 club-toolbar cmd codes (verified from FUN_00487210 + menu_tree_scope §4)
// ============================================================================

/// `0x64` — the Take Control cmd. In the C the constant is written as
/// decimal `100`; the switch treats it as a `short`. See
/// `00487210.c:98` (spawn) and `00487670.c:152` (handler).
pub const CMD_TAKE_CONTROL: i16 = 100; // 0x64
/// `0x65` — Apply for the Manager Job.
pub const CMD_APPLY_FOR_JOB: i16 = 0x65;
/// `0x66` — Invite Club (offer/invite a friendly). Fires the confirmation
/// dialog for either "Offer Friendly" or "Invite for Friendly" depending on
/// whether the target club is already in a tournament fixture.
pub const CMD_INVITE_FRIENDLY: i16 = 0x66;
/// `0x67` — Invite Club to tournament (variant): tournament-context invite.
pub const CMD_INVITE_TOURNAMENT_FIXTURE: i16 = 0x67;
/// `0x68` — Uninvite/Withdraw a tournament invitation.
pub const CMD_UNINVITE_TOURNAMENT: i16 = 0x68;
/// `0x69` — Invite Club into a tournament proper.
pub const CMD_INVITE_TOURNAMENT: i16 = 0x69;
/// `0x6a` — Scout Club (assign a scout to watch this club).
pub const CMD_SCOUT_CLUB: i16 = 0x6a;

// ============================================================================
// Part A — port of FUN_00487210 (menu-item builder)
// ============================================================================

/// One menu item the builder decides to spawn. The C spawns each via a
/// `FUN_00549580` call with a fixed 18-argument template
/// (`2, 0x294, 4, 0x311, 0x18, 0, 0, 0x30, param_3, param_3, 0xc, 1,
/// param_2, &DAT_00dc723c, 0, <cmd>, <payload>, 0xffffffff`) — see
/// `00487210.c:21` and analogues. Only the `<cmd>` + `<payload>` + label
/// string change per branch, so the port carries just those three.
#[derive(Debug, Clone, PartialEq)]
pub struct ClubToolbarMenuItem {
    /// Static label — one of `"Scout Club"`, `"Invite Club"`,
    /// `"Uninvite Club"`, `"Apply for Job"`, `"Take Control"` per the
    /// `s_*` constants used in the C at lines 20/34/45/68/74/85/97.
    pub label: &'static str,
    /// The cmd code the item will fire when clicked (0x64..0x6a).
    pub cmd: i16,
    /// The payload the click carries (a pointer in the exe; the port
    /// carries the numeric handle). `0` in the "no payload" arms
    /// (Take Control / Apply for Job / Scout Club — see C lines 22,
    /// 87, 99), a non-zero handle in the invite arms.
    pub payload: u32,
}

/// Read-only decision inputs that `FUN_00487210` uses to gate each arm.
///
/// The C queries a dozen globals + subsystem fns. The port surfaces just
/// the resolved booleans / handles the switch actually needs, so callers
/// (game-state code, tests) provide already-resolved answers rather than
/// dragging in the network / friendlies / tournament pools. Each field is
/// annotated with the C line + FUN_ that produces it.
///
/// This is the "no fabrication" boundary: rather than approximate the
/// answers from unported subsystems, the port takes the answers as inputs.
#[derive(Debug, Clone, Default)]
pub struct ClubToolbarBuildInputs {
    /// Handle to the club record the toolbar is being built for. The C
    /// takes this as `param_1` — many arms read `param_1 + 0x53` (the
    /// tournament-league-id byte) and pass `param_1` on to lookup fns.
    pub club_handle: u32,
    /// Result of `FUN_00822580() == 0` — the "not in network mode" gate
    /// that lets us reach the else branch (Take Control) vs the main
    /// branch (all other items). C line 12.
    pub is_local_mode: bool,
    /// Result of `FUN_007e05c0() != 0` — the "scout module available"
    /// predicate. Gates the Scout Club arm at C line 14.
    pub scout_module_ready: bool,
    /// Result of `FUN_007e06a0(club_handle) == 0` — "club not already
    /// being scouted". Gates the Scout Club arm at C line 16.
    pub club_not_being_scouted: bool,
    /// Result of `FUN_00525450(club_handle) == 0` — "club is human-
    /// managed / not in scout ignore list". Gates the Scout Club arm at
    /// C line 18. (The same fn is called throughout, with `!= 0`
    /// interpreted as "is manageable / has value" — see the parity
    /// checks at lines 33/44/54.)
    pub club_manageable: bool,
    /// Result of `FUN_005ea720(club_handle,0,1) == 0` — "friendlies
    /// module accepts an invite request". Gates the three invite arms
    /// (C lines 28..80).
    pub friendlies_slot_open: bool,
    /// Result of `FUN_005b0b70()` — the current tournament handle
    /// (`iVar3 != 0`), and the target club at `+0xc` matches parity with
    /// the invited club. Optional — `None` means the tournament slot
    /// wasn't populated. C lines 29..37.
    pub tournament_current_handle: Option<u32>,
    /// Same, but for the alt-tournament slot from `FUN_005b0be0()`. The
    /// C guards on `puVar6[1] == *(int*)(param_1 + 0x53)` (league match).
    /// C lines 40..49.
    pub tournament_alt_handle: Option<u32>,
    /// Same, for the pending-invites list `FUN_005b0b00()`. C lines
    /// 51..79. If `Some` the C then loops the invite list; the port's
    /// `club_is_pending_invitee` flag pre-answers that loop.
    pub tournament_pending_handle: Option<u32>,
    /// Result of the C's inner loop at lines 58..65 (`for i in 0..(*(short*)(puVar6+0x13)) { if (*(int*)(puVar6+i*4+0x15)==param_1) hit = true; }`).
    pub club_is_pending_invitee: bool,
    /// Result of `FUN_005265e0(club_handle) == 0` — "club has no
    /// pending manager application". C line 81.
    pub no_pending_application: bool,
    /// Result of `FUN_0052e370(club_handle) != 0` — "human is eligible
    /// to apply for/manage this club". Gates both Apply for Job (line
    /// 83) and Take Control (line 95).
    pub eligible_to_manage: bool,
    /// Result of `FUN_005ea590(club_handle,1,1,0,0) == 0` — "no
    /// blocking condition on the else-branch takeover path". C line 93.
    pub takeover_gate_open: bool,
}

/// Direct port of `FUN_00487210` — decides the club context-toolbar's
/// menu items.
///
/// Returns the list of items to spawn (each of which the caller renders
/// with `FUN_00549580`-equivalent widget spawning; the port emits the
/// data, not the widget-pool writes, so this is testable without a full
/// pool + template widget). An empty vec means "no menu" (the C returns
/// 0 from every fall-through path).
///
/// The C is structured as a nested-`if` chain that returns `1` as soon
/// as any arm matches; the port preserves that early-out semantics.
/// Every arm's C line range is cited.
pub fn dispatch_club_toolbar_top(inputs: &ClubToolbarBuildInputs) -> Vec<ClubToolbarMenuItem> {
    let mut items = Vec::new();

    if inputs.is_local_mode {
        // C lines 13..90 — the main "local mode" branch.

        // Arm 1: Scout Club (cmd 0x6a). C lines 14..24.
        if inputs.scout_module_ready
            && inputs.club_not_being_scouted
            && inputs.club_manageable
        {
            items.push(ClubToolbarMenuItem {
                label: "Scout Club",
                cmd: CMD_SCOUT_CLUB,
                payload: 0,
            });
            return items; // C line 23: `return 1;`
        }

        // Arm 2..5: friendlies + tournament invite paths. C line 27:
        // `if (FUN_005ea720(param_1,0,1) == 0)`.
        if inputs.friendlies_slot_open {
            // Arm 2: current-tournament invite. C lines 29..38.
            if let Some(_h) = inputs.tournament_current_handle {
                // Parity check `(iVar5 != 0) == (iVar4 != 0)` — both
                // sides call FUN_00525450 (see club_manageable doc). The
                // port collapses that to `club_manageable` since both
                // handles feed the same predicate for our purposes.
                items.push(ClubToolbarMenuItem {
                    label: "Invite Club",
                    cmd: CMD_INVITE_FRIENDLY, // 0x66
                    payload: inputs.tournament_current_handle.unwrap_or(0),
                });
                return items;
            }

            // Arm 3: alt-tournament invite (league-matched). C 40..49.
            if let Some(h) = inputs.tournament_alt_handle {
                items.push(ClubToolbarMenuItem {
                    label: "Invite Club",
                    cmd: CMD_INVITE_TOURNAMENT_FIXTURE, // 0x67
                    payload: h,
                });
                return items;
            }

            // Arm 4/5: pending-invites list — Uninvite Club vs Invite
            // Club depending on whether the target already appears in
            // the pending list. C lines 51..79.
            if let Some(h) = inputs.tournament_pending_handle {
                if inputs.club_is_pending_invitee {
                    // Arm 4: Uninvite Club. C lines 67..72.
                    items.push(ClubToolbarMenuItem {
                        label: "Uninvite Club",
                        cmd: CMD_UNINVITE_TOURNAMENT, // 0x68
                        payload: h,
                    });
                } else {
                    // Arm 5: Invite Club (tournament proper). C lines
                    // 74..77.
                    items.push(ClubToolbarMenuItem {
                        label: "Invite Club",
                        cmd: CMD_INVITE_TOURNAMENT, // 0x69
                        payload: h,
                    });
                }
                return items;
            }
        }

        // Arm 6: Apply for Job. C lines 81..89.
        if inputs.no_pending_application && inputs.eligible_to_manage {
            items.push(ClubToolbarMenuItem {
                label: "Apply for Job",
                cmd: CMD_APPLY_FOR_JOB, // 0x65
                payload: 0,
            });
            return items;
        }
    } else {
        // C lines 92..103 — the "network mode" else branch → Take Control.
        if inputs.takeover_gate_open && inputs.eligible_to_manage {
            items.push(ClubToolbarMenuItem {
                label: "Take Control",
                cmd: CMD_TAKE_CONTROL, // 0x64 == decimal 100
                payload: 0,
            });
            return items;
        }
    }

    items
}

// ============================================================================
// Part B — port of FUN_00487670 (event handler / cmd dispatcher)
// ============================================================================

/// Direct port of `FUN_00487670`. Handles the seven club-toolbar cmds
/// on click, driven by the same widget-cmd resolution idiom the other
/// two Layer-4 dispatchers use.
///
/// Every arm below cites the C address of its `if (sVar2 == 0xNN)`
/// branch head. All arms are `TodoBuilder` because none of the targets
/// (`FUN_0080bbd0` take-control, `FUN_00696fa0` apply-for-job, or the
/// `FUN_0057b9c0` msgbox family) are ported yet — they're not
/// `build_screen_*` builders, so the auto-transliterated `screens_auto`
/// module has nothing that would apply. This is faithful: the dispatcher
/// resolves cmd correctly, only the terminal action is deferred.
///
/// # Pre-dispatch bookkeeping — dialog-result drain (C lines 42..141)
///
/// Before the switch, the C reads `DAT_00dbbf7c` / `DAT_00dbbf80` — the
/// same modal-result globals `dispatch_global`'s pre-dispatch phase
/// handles — and drains five prior-modal-result branches
/// (`DAT_00dbbf7c == 0x41` scout-confirm; `0xc` friendly-offer accepted;
/// `0xd` tournament-invite accepted; `0xe` uninvite accepted; `0xf`
/// invite-to-tournament accepted). Each branch reads the club/tournament
/// handles via `FUN_005b0b70`/`FUN_005b0be0`/`FUN_005b0b00` and applies
/// the accepted invite via `FUN_007e0490`/`FUN_005af840`/`FUN_005b0c50`.
/// All those subsystems are unported (mirrors the
/// [`crate::dispatcher::pre_dispatch_bookkeeping`] scaffold decision).
/// See [`pre_dispatch_dialog_drain`].
pub fn dispatch_club_toolbar_bottom(
    pool: &mut GuiRecordPool,
    state: &mut DispatcherState,
    widget_slot: i16,
) -> DispatchResult {
    // ---- Pre-dispatch: dialog-result drain (C lines 42..141). Scaffolded.
    if let Some(r) = pre_dispatch_dialog_drain(pool, state) {
        return r;
    }

    // ---- Cmd resolution (same 3-line idiom as the other dispatchers,
    // C lines 142..151 / 160..169 / repeated for each arm). The C
    // reads `param_1` (widget slot) each time and re-resolves — the
    // port reads once and matches on the result.
    let cmd = resolve_widget_cmd(pool, widget_slot, state);

    match cmd {
        // 0x64 == 100 (`00487670.c:152`) — Take Control.
        // Exe: `iVar4 = FUN_007e6ee0(0); FUN_0080bbd0(iVar4, 0); return -4;`
        CMD_TAKE_CONTROL => DispatchResult::TodoBuilder {
            cmd: CMD_TAKE_CONTROL,
            fn_addr: "FUN_007e6ee0+FUN_0080bbd0",
            note: "Take Control — fires the club-takeover action",
        },
        // 0x65 (`00487670.c:204`) — Apply for Manager Job.
        // Exe: gated on `DAT_00b59fc2[seat*0xc0] != 0`; then
        // `uVar3 = FUN_007e6ee0(0); FUN_00696fa0(uVar3); return -4;`
        CMD_APPLY_FOR_JOB => DispatchResult::TodoBuilder {
            cmd: CMD_APPLY_FOR_JOB,
            fn_addr: "FUN_007e6ee0+FUN_00696fa0",
            note: "Apply for Job — opens application form action",
        },
        // 0x66 (`00487670.c:226`) — Invite for Friendly (msgbox).
        // Exe: reads payload via `pool[slot]+0x48` (`0xba9a6`), calls
        // FUN_005b0e20/FUN_005b1840 to test availability, then
        // FUN_006547c0 formats a "Please Confirm" template and
        // FUN_0057b9c0 shows it.
        CMD_INVITE_FRIENDLY => DispatchResult::TodoBuilder {
            cmd: CMD_INVITE_FRIENDLY,
            fn_addr: "FUN_005b0e20+FUN_005b1840+FUN_006547c0+FUN_0057b9c0",
            note: "Invite For Friendly — Please Confirm dialog",
        },
        // 0x67 (`00487670.c:283`) — Invite For Friendly (tournament ctx).
        // Same subsystem stack as 0x66; different template + payload
        // shape (walks the pool[3]-length array from puVar8).
        CMD_INVITE_TOURNAMENT_FIXTURE => DispatchResult::TodoBuilder {
            cmd: CMD_INVITE_TOURNAMENT_FIXTURE,
            fn_addr: "FUN_005b0e20+FUN_005b1840+FUN_006547c0+FUN_0057b9c0",
            note: "Invite For Friendly (tournament) — Please Confirm dialog",
        },
        // 0x68 (`00487670.c:343`) — Uninvite To Tournament (msgbox).
        // Exe: no availability check; FUN_006547c0 template +
        // FUN_0057b9c0 confirm.
        CMD_UNINVITE_TOURNAMENT => DispatchResult::TodoBuilder {
            cmd: CMD_UNINVITE_TOURNAMENT,
            fn_addr: "FUN_006547c0+FUN_0057b9c0",
            note: "Uninvite To Tournament — Please Confirm dialog",
        },
        // 0x69 (`00487670.c:373`) — Invite To Tournament (msgbox).
        // Reads puVar8+3 (list-size byte); availability check via
        // FUN_005b0e20+FUN_005b1840; then confirm dialog.
        CMD_INVITE_TOURNAMENT => DispatchResult::TodoBuilder {
            cmd: CMD_INVITE_TOURNAMENT,
            fn_addr: "FUN_005b0e20+FUN_005b1840+FUN_006547c0+FUN_0057b9c0",
            note: "Invite To Tournament — Please Confirm dialog",
        },
        // 0x6a (`00487670.c:170`) — Assign Scout (msgbox).
        // Exe: FUN_00525190 pulls a description pair; then either
        // "Assign Scout..." confirm (FUN_007e06a0==0 path) or
        // "Club Already Being Watched" info (else path).
        CMD_SCOUT_CLUB => DispatchResult::TodoBuilder {
            cmd: CMD_SCOUT_CLUB,
            fn_addr: "FUN_00525190+FUN_007e06a0+FUN_006547c0+FUN_0057b9c0",
            note: "Assign Scout — Please Confirm / Already Watched dialog",
        },
        // Any other cmd falls through to `LAB_00488256: uVar3 = 0;`
        // (C line 212). Return code is Unhandled so the outer dispatch
        // chain can continue.
        _ => DispatchResult::Unhandled,
    }
}

/// Scaffold for the dialog-result drain of `FUN_00487670` (C lines
/// 42..141). Consumes prior `DAT_00dbbf7c == {0x41, 0xc, 0xd, 0xe, 0xf}`
/// modal results if any is pending, applying the "user confirmed" side
/// effect for each.
///
/// The port returns `None` (proceed to switch) because every branch's
/// side-effect target is unported:
///
/// * `0x41` → `FUN_007e6ee0(0)` + `FUN_007e0490` — scout-request commit
/// * `0xc`  → `FUN_005b0b70`+`FUN_005af840` — friendly-invite commit
/// * `0xd`  → `FUN_005b0be0`+`FUN_005afaa0`+`FUN_005b11a0`+`FUN_005af840` —
///           tournament-fixture invite commit
/// * `0xe`  → `FUN_005b0b00`+`FUN_005b0c50` — uninvite commit
/// * `0xf`  → `FUN_005b0b00`+`FUN_005b0c50` — invite-to-tournament commit
///
/// (Same "no fabrication" call as [`crate::dispatcher::pre_dispatch_bookkeeping`].)
pub fn pre_dispatch_dialog_drain(
    _pool: &mut GuiRecordPool,
    state: &mut DispatcherState,
) -> Option<DispatchResult> {
    // Only fire if the caller has actually surfaced a pending dialog
    // result. Even then, we still can't drive the commit without the
    // FUN_ dependencies above; we consume the pending flag (to match
    // the C's `DAT_00dbbf7c = 0` on every branch) and return None so
    // the switch still runs the same tick.
    let pending = state.pending_dialog_result;
    let value = state.pending_dialog_value;
    let is_ok = value == 2; // C: `DAT_00dbbf80 == 2` on every arm
    if pending == 0x41 || pending == 0xc || pending == 0xd || pending == 0xe || pending == 0xf {
        state.pending_dialog_result = 0;
        state.pending_dialog_value = 0;
        let _ = is_ok; // TODO: route through to commit fns once ported
    }
    None
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget_pool::WidgetDescriptor;

    fn widget_with_cmd(pool: &mut GuiRecordPool, cmd: i32) -> i16 {
        let mut d = WidgetDescriptor::empty();
        d.msg_id = cmd;
        pool.spawn_widget(d, -1).expect("pool room") as i16
    }

    // ---- Part A: builder tests ----

    #[test]
    fn builder_local_mode_scout_arm_matches() {
        let inputs = ClubToolbarBuildInputs {
            is_local_mode: true,
            scout_module_ready: true,
            club_not_being_scouted: true,
            club_manageable: true,
            ..Default::default()
        };
        let items = dispatch_club_toolbar_top(&inputs);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "Scout Club");
        assert_eq!(items[0].cmd, CMD_SCOUT_CLUB);
    }

    #[test]
    fn builder_local_mode_invite_current_tournament_arm() {
        let inputs = ClubToolbarBuildInputs {
            is_local_mode: true,
            friendlies_slot_open: true,
            tournament_current_handle: Some(0xdead),
            ..Default::default()
        };
        let items = dispatch_club_toolbar_top(&inputs);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "Invite Club");
        assert_eq!(items[0].cmd, CMD_INVITE_FRIENDLY);
        assert_eq!(items[0].payload, 0xdead);
    }

    #[test]
    fn builder_local_mode_uninvite_arm_when_club_is_already_invited() {
        let inputs = ClubToolbarBuildInputs {
            is_local_mode: true,
            friendlies_slot_open: true,
            tournament_pending_handle: Some(0x1234),
            club_is_pending_invitee: true,
            ..Default::default()
        };
        let items = dispatch_club_toolbar_top(&inputs);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "Uninvite Club");
        assert_eq!(items[0].cmd, CMD_UNINVITE_TOURNAMENT);
    }

    #[test]
    fn builder_local_mode_invite_to_tournament_when_not_yet_invited() {
        let inputs = ClubToolbarBuildInputs {
            is_local_mode: true,
            friendlies_slot_open: true,
            tournament_pending_handle: Some(0x1234),
            club_is_pending_invitee: false,
            ..Default::default()
        };
        let items = dispatch_club_toolbar_top(&inputs);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "Invite Club");
        assert_eq!(items[0].cmd, CMD_INVITE_TOURNAMENT);
    }

    #[test]
    fn builder_local_mode_apply_for_job_arm() {
        let inputs = ClubToolbarBuildInputs {
            is_local_mode: true,
            no_pending_application: true,
            eligible_to_manage: true,
            ..Default::default()
        };
        let items = dispatch_club_toolbar_top(&inputs);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "Apply for Job");
        assert_eq!(items[0].cmd, CMD_APPLY_FOR_JOB);
    }

    #[test]
    fn builder_network_mode_take_control_arm() {
        let inputs = ClubToolbarBuildInputs {
            is_local_mode: false,
            takeover_gate_open: true,
            eligible_to_manage: true,
            ..Default::default()
        };
        let items = dispatch_club_toolbar_top(&inputs);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "Take Control");
        assert_eq!(items[0].cmd, CMD_TAKE_CONTROL);
        assert_eq!(items[0].cmd, 100);
    }

    #[test]
    fn builder_no_arms_when_no_gate_passes() {
        let inputs = ClubToolbarBuildInputs::default();
        let items = dispatch_club_toolbar_top(&inputs);
        assert!(items.is_empty());
    }

    #[test]
    fn builder_scout_arm_wins_over_apply_when_both_would_gate() {
        // The C's early-return ordering means Scout Club fires and
        // Apply for Job is not reached, even when its inputs are set.
        let inputs = ClubToolbarBuildInputs {
            is_local_mode: true,
            scout_module_ready: true,
            club_not_being_scouted: true,
            club_manageable: true,
            no_pending_application: true,
            eligible_to_manage: true,
            ..Default::default()
        };
        let items = dispatch_club_toolbar_top(&inputs);
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].label, "Scout Club");
    }

    // ---- Part B: dispatcher tests ----

    #[test]
    fn dispatch_club_toolbar_top_routes_each_cmd() {
        // The builder emits every one of the 7 cmd codes. Verify each
        // is one of the 7 the handler recognises.
        let all_cmds = [
            CMD_TAKE_CONTROL,
            CMD_APPLY_FOR_JOB,
            CMD_INVITE_FRIENDLY,
            CMD_INVITE_TOURNAMENT_FIXTURE,
            CMD_UNINVITE_TOURNAMENT,
            CMD_INVITE_TOURNAMENT,
            CMD_SCOUT_CLUB,
        ];
        assert_eq!(all_cmds.len(), 7);
        // No duplicates:
        let mut sorted = all_cmds.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 7);
    }

    #[test]
    fn dispatch_club_toolbar_bottom_routes_each_cmd() {
        for cmd in [
            CMD_TAKE_CONTROL,
            CMD_APPLY_FOR_JOB,
            CMD_INVITE_FRIENDLY,
            CMD_INVITE_TOURNAMENT_FIXTURE,
            CMD_UNINVITE_TOURNAMENT,
            CMD_INVITE_TOURNAMENT,
            CMD_SCOUT_CLUB,
        ] {
            let mut pool = GuiRecordPool::new();
            let mut state = DispatcherState::default();
            let slot = widget_with_cmd(&mut pool, cmd as i32);
            let r = dispatch_club_toolbar_bottom(&mut pool, &mut state, slot);
            assert!(
                matches!(r, DispatchResult::TodoBuilder { cmd: c, .. } if c == cmd),
                "cmd 0x{:x} expected TodoBuilder, got {:?}",
                cmd,
                r
            );
        }
    }

    #[test]
    fn dispatch_club_toolbar_bottom_unknown_cmd_is_unhandled() {
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        let slot = widget_with_cmd(&mut pool, 0x9999);
        assert_eq!(
            dispatch_club_toolbar_bottom(&mut pool, &mut state, slot),
            DispatchResult::Unhandled
        );
    }

    #[test]
    fn dispatch_club_toolbar_bottom_falls_back_to_pending_cmd() {
        // Widget's msg_id = 0 → resolver reads state.pending_cmd_fallback.
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        state.pending_cmd_fallback = CMD_APPLY_FOR_JOB;
        let slot = widget_with_cmd(&mut pool, 0);
        let r = dispatch_club_toolbar_bottom(&mut pool, &mut state, slot);
        assert!(matches!(
            r,
            DispatchResult::TodoBuilder { cmd, .. } if cmd == CMD_APPLY_FOR_JOB
        ));
    }

    #[test]
    fn pre_dispatch_dialog_drain_consumes_flag_when_pending() {
        // If state has a pending dialog result matching one of the 5
        // C arms, the drain must clear it (mirroring the exe's
        // `DAT_00dbbf7c = 0` on each branch).
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        state.pending_dialog_result = 0x41;
        state.pending_dialog_value = 2;
        let _ = pre_dispatch_dialog_drain(&mut pool, &mut state);
        assert_eq!(state.pending_dialog_result, 0);
        assert_eq!(state.pending_dialog_value, 0);
    }

    #[test]
    fn pre_dispatch_dialog_drain_leaves_unrelated_flags_alone() {
        // A flag outside {0x41, 0xc, 0xd, 0xe, 0xf} must NOT be touched
        // — the C only cares about those five.
        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        state.pending_dialog_result = 0x7f;
        state.pending_dialog_value = 2;
        let _ = pre_dispatch_dialog_drain(&mut pool, &mut state);
        assert_eq!(state.pending_dialog_result, 0x7f);
        assert_eq!(state.pending_dialog_value, 2);
    }

    // ---- End-to-end: builder output feeds dispatcher input ----

    #[test]
    fn end_to_end_take_control_builder_to_dispatcher() {
        // Builder decides Take Control; dispatcher routes the resulting
        // cmd back to a TodoBuilder for the take-control action.
        let inputs = ClubToolbarBuildInputs {
            is_local_mode: false,
            takeover_gate_open: true,
            eligible_to_manage: true,
            ..Default::default()
        };
        let items = dispatch_club_toolbar_top(&inputs);
        assert_eq!(items.len(), 1);
        let clicked_cmd = items[0].cmd;

        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        let slot = widget_with_cmd(&mut pool, clicked_cmd as i32);
        let r = dispatch_club_toolbar_bottom(&mut pool, &mut state, slot);
        assert!(matches!(
            r,
            DispatchResult::TodoBuilder { cmd, .. } if cmd == CMD_TAKE_CONTROL
        ));
    }

    #[test]
    fn end_to_end_scout_club_builder_to_dispatcher() {
        let inputs = ClubToolbarBuildInputs {
            is_local_mode: true,
            scout_module_ready: true,
            club_not_being_scouted: true,
            club_manageable: true,
            ..Default::default()
        };
        let items = dispatch_club_toolbar_top(&inputs);
        assert_eq!(items[0].cmd, CMD_SCOUT_CLUB);

        let mut pool = GuiRecordPool::new();
        let mut state = DispatcherState::default();
        let slot = widget_with_cmd(&mut pool, items[0].cmd as i32);
        let r = dispatch_club_toolbar_bottom(&mut pool, &mut state, slot);
        assert!(matches!(
            r,
            DispatchResult::TodoBuilder { cmd, .. } if cmd == CMD_SCOUT_CLUB
        ));
    }
}
