//! C11.3 — season-roll scheduler + `CompetitionSeasonRoll` trait.
//!
//! Runtime-evidence-driven port of the exe's per-competition
//! season-roll dispatch mechanism (GDI `sub_005605c0` for English
//! Second Division; sibling functions per each competition class).
//!
//! # The mechanism
//!
//! The exe's daily-tick driver (`FUN_005b6a90` GDI / same VA on
//! DirectDraw) reaches a phase where it iterates a **34-slot
//! scheduler table** at `DAT_00b4bc70` (DirectDraw VA). Each slot
//! carries a trigger day-of-year and a list of registered
//! competition pointers. On each day, `FUN_005bfd90` (the
//! dispatcher) walks the slots — for every slot whose `trigger_day`
//! matches today's day-of-year, it invokes vtable slot **+0x08**
//! (the season-roll method) on each pooled competition.
//!
//! The season-roll method (per-competition-class) mirrors this
//! GDI-verified skeleton for `sub_005605c0`
//! (`crates/cm-domain/../../D:/cm0102-carve/gdi_carve/functions/00004-PE_section_.text/02039_sub_005605c0.asm`):
//!
//! ```asm
//! 005605c0  sub  esp, 0x200        ; local buffers
//! 005605c8  mov  esi, ecx          ; this = ecx (__thiscall)
//!
//! ; Step 1: free schedule buffer at this+0xba
//! 005605d5  mov  eax, [esi+0xba]
//! 005605dd  je   +0x11             ; skip free if null
//! 005605e0  call free
//! 005605e8  mov  [esi+0xba], 0
//!
//! ; Step 2: destroy sub-object at this+0x08
//! ; Step 3: destroy children in array at this+0x0c
//!
//! ; Step 4: THE YEAR ADVANCE
//! 0056061e  mov  eax, [esi]        ; vtable
//! 00560620  inc  word [esi+0x40]   ; year field += 1 (u16)
//! 00560624  mov  ecx, esi
//! 00560626  mov  [esi+0x30], -1    ; reset count
//! 0056062d  call [eax+0x8c]        ; vtable[+0x8c] = reset+regen
//!                                  ; (= sub_00560520 for eng_second)
//! ```
//!
//! # Runtime evidence
//!
//! - `fixtures/gdi_captures/season_2001_02_v3_thiscall_fixed.jsonl`
//!   proved the year field at `Comp+0x40` advances byte-exactly
//!   `2001 → 2002` between the mid-season query and the next
//!   regen call — matching `inc word [esi+0x40]` (delta = +1) not
//!   the sibling classes' `+= 2`.
//!
//! - `fixtures/gdi_captures/season_2001_02_v6_backtrace.jsonl`
//!   stack-backtraced the mid-season regen call chain, landing
//!   frame 3 at `0x00560633` — inside `sub_005605c0` at +0x73,
//!   the return address from the `vtable[+0x8c]` call. That's how
//!   we identified `sub_005605c0` as the eng_second season-roll
//!   method the prior static-analysis pass missed.
//!
//! # What this module DOES port
//!
//! * The trait `CompetitionSeasonRoll` for competitions that
//!   should receive year-turn regeneration on a scheduled day
//! * The `SeasonRollScheduler` — a day-of-year → competition-id
//!   dispatcher that mirrors the exe's 34-slot table shape
//! * The `SlotEntry.last_processed_year` guard preventing a
//!   comp from firing twice within one calendar year turn (the
//!   exe's per-slot year-comparison check gates the announcement
//!   / progress-bar, but is a natural safety net at the dispatch
//!   too)
//!
//! # What this module does NOT port
//!
//! * The specific slot-membership table (which comp lives in
//!   which slot). Populated incrementally per league family as
//!   ports land. English Second Division starts registered on
//!   Jan 1 — matching the observed `2001 → 2002` transition on
//!   its `Comp+0x40` field.
//!
//! * The individual competition classes' `season_roll` bodies.
//!   Each ported competition provides its own trait impl.
//!
//! * The C-runtime helpers (`free()`, `qsort()`) and the child-
//!   destroy pass — Rust ownership handles those.
//!
//! # Relation to the old `hook_year_rollover`
//!
//! The existing `hook_year_rollover` at [lib.rs:20560](../../lib.rs)
//! fires on `date.year != last_year_rollover` — semantically wrong.
//! It matched only one moment per calendar year; the exe fires
//! season-roll work on 34 different scheduled days. This module
//! is the correct dispatcher. Migration is incremental: the old
//! hook stays for season-award firing (which is genuinely
//! per-Rust-calendar-year in the current port) until the awards
//! subsystem's own trigger is decoded.

use std::collections::BTreeMap;

/// One competition class implements this to receive a year-turn
/// fire. The `world` handle lets the implementation read/write
/// its own state and trigger downstream side effects (fixture
/// regen, roster reset, etc.).
///
/// # Semantic contract (from `sub_005605c0`)
///
/// 1. Free/clear any cached schedule state for the previous
///    season.
/// 2. Increment the competition's season-year field by exactly
///    one (`inc word` in the exe, not `+= 2`).
/// 3. Regenerate the schedule for the new season via the frozen
///    fixture engine (equivalent of the vtable `+0x8c` call).
///
/// Return `SeasonRollOutcome::Fired` when the roll actually
/// executed; `Skipped { reason }` when the impl decided not to
/// (e.g. game mode is `V4`, or the competition is inactive).
pub trait CompetitionSeasonRoll {
    fn season_roll(
        &mut self,
        ctx: &mut SeasonRollContext<'_>,
    ) -> SeasonRollOutcome;
}

/// Result of one season-roll fire.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeasonRollOutcome {
    /// The competition class ran its roll — year field advanced
    /// and schedule regen was triggered.
    Fired {
        comp_id: u32,
        old_year: u16,
        new_year: u16,
    },
    /// Skipped for a semantic reason. Not an error.
    Skipped {
        comp_id: u32,
        reason: SkipReason,
    },
}

/// Enumerable reasons a season-roll may skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkipReason {
    /// Game mode is `V4` and this competition is Traditional-only.
    NotTraditionalMode,
    /// Competition marked inactive (exe vtable predicate `+0xc`).
    CompInactive,
    /// This calendar year already processed this comp.
    AlreadyProcessedThisYear,
    /// Other free-form reason.
    Other(String),
}

/// Handle a `CompetitionSeasonRoll` implementation receives. Kept
/// minimal so specific competition ports can opt into which
/// pieces of `World` they touch — avoids a massive god-context.
///
/// Future extensions add fields as new competition types port
/// (e.g. a `stadium_expansion_queue` for competitions that
/// promote clubs whose new tier triggers stadium expansion).
pub struct SeasonRollContext<'a> {
    pub current_year: u16,
    pub current_day_of_year: u16,
    /// Opaque scratch a specific comp implementation can use to
    /// stash cross-comp coordination state (e.g. the English
    /// pyramid orchestrator's per-tier position tables). Sized to
    /// avoid heap churn on the hot per-day tick path.
    pub scratch: &'a mut Vec<u8>,
}

/// Per-slot state in the scheduler.
///
/// Storage rule: match the exe's 34-slot table shape but keep
/// the field types Rust-idiomatic. The `last_processed_year`
/// field mirrors the exe's `slot + 0x35` (u16). Advancing it
/// after a fire prevents a second fire within one calendar year
/// even if the day-of-year re-matches (leap-year edge case,
/// game holidayed backward-then-forward, etc.).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotEntry {
    /// 1-based day of year (matches how the exe stores the
    /// trigger). `1` = Jan 1, `365` = Dec 31 (or 366 in a leap
    /// year — the exe uses raw day-of-year without special
    /// handling).
    pub trigger_day_of_year: u16,
    /// The most recent calendar year this slot processed. `0` on
    /// creation. Mirrors the exe's `slot + 0x35`.
    pub last_processed_year: u16,
    /// Competition ids registered in this slot. Fires in
    /// registration order per day match.
    pub comp_ids: Vec<u32>,
}

impl SlotEntry {
    pub fn new(trigger_day_of_year: u16) -> Self {
        Self {
            trigger_day_of_year,
            last_processed_year: 0,
            comp_ids: Vec::new(),
        }
    }
}

/// Season-roll scheduler. Port of the exe's 34-slot
/// `DAT_00b4bc70` table + the `FUN_005bfd90` dispatcher.
///
/// Storage: a BTreeMap keyed by trigger day-of-year. That's more
/// flexible than the exe's fixed 34-slot array and matches how
/// the exe's dispatcher effectively works — walk slots, day-match,
/// fire per-comp. The `slots` count on the exe is a static-data
/// artefact; nothing about the algorithm depends on 34 exactly.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SeasonRollScheduler {
    slots: BTreeMap<u16, SlotEntry>,
}

impl SeasonRollScheduler {
    pub fn new() -> Self { Self::default() }

    /// Register a competition to fire on `trigger_day_of_year`.
    /// Multiple comps on the same day accumulate into the same
    /// slot's `comp_ids` — mirrors the exe's per-slot pool.
    pub fn register(
        &mut self,
        comp_id: u32,
        trigger_day_of_year: u16,
    ) {
        self.slots
            .entry(trigger_day_of_year)
            .or_insert_with(|| SlotEntry::new(trigger_day_of_year))
            .comp_ids.push(comp_id);
    }

    /// Read-only: which comps fire on the given day-of-year.
    /// Useful for tests + tracing.
    pub fn comps_for_day(&self, day: u16) -> Vec<u32> {
        self.slots.get(&day)
            .map(|s| s.comp_ids.clone())
            .unwrap_or_default()
    }

    /// The full slot list, sorted by day-of-year.
    pub fn slots(&self) -> impl Iterator<Item = (&u16, &SlotEntry)> {
        self.slots.iter()
    }

    /// Total registered comp count across all slots.
    pub fn total_registered(&self) -> usize {
        self.slots.values().map(|s| s.comp_ids.len()).sum()
    }

    /// Fire the day's slot, if any. Returns the list of
    /// `SeasonRollOutcome`s (one per registered comp). Empty
    /// vector when no slot matches today.
    ///
    /// The caller supplies a `resolver` closure that maps a
    /// comp id to the `dyn CompetitionSeasonRoll` for that
    /// comp. This inversion keeps the scheduler generic — it
    /// doesn't know about specific competition types.
    ///
    /// Mirrors the exe's `FUN_005bfd90` inner loop but without
    /// the per-comp active/done vtable predicates (`+0xc` /
    /// `+0x2c`) — those are the impl's own responsibility and
    /// belong inside `season_roll` via `SkipReason::CompInactive`.
    pub fn fire_for_day<F, R>(
        &mut self,
        today_year: u16,
        today_day_of_year: u16,
        mut resolver: F,
        ctx: &mut SeasonRollContext<'_>,
    ) -> Vec<SeasonRollOutcome>
    where
        F: FnMut(u32) -> Option<R>,
        R: FnOnce(&mut SeasonRollContext<'_>) -> SeasonRollOutcome,
    {
        let mut outcomes = Vec::new();
        let Some(slot) = self.slots.get_mut(&today_day_of_year)
            else { return outcomes };
        if slot.last_processed_year == today_year {
            // Guard: already fired this calendar year. Return
            // an AlreadyProcessed skip per registered comp so
            // the trace stays informative.
            for &cid in &slot.comp_ids {
                outcomes.push(SeasonRollOutcome::Skipped {
                    comp_id: cid,
                    reason: SkipReason::AlreadyProcessedThisYear,
                });
            }
            return outcomes;
        }
        for &comp_id in &slot.comp_ids {
            let Some(roll_fn) = resolver(comp_id) else {
                outcomes.push(SeasonRollOutcome::Skipped {
                    comp_id,
                    reason: SkipReason::Other(
                        "no impl registered for comp id".to_string(),
                    ),
                });
                continue;
            };
            outcomes.push(roll_fn(ctx));
        }
        // Mark slot processed for this year (whether any comp
        // actually fired or all skipped — the exe's slot-year
        // update is unconditional).
        slot.last_processed_year = today_year;
        outcomes
    }
}

// ---------------------------------------------------------------------------
// Boot registration: English pyramid on Jan 1
// ---------------------------------------------------------------------------

/// The Rust project's English pyramid comp ids, matching
/// `english_traditional::ENGLISH_ROLLOVER_COMP_IDS`.
pub const ENGLISH_PYRAMID_COMP_IDS: [u32; 5] = [7, 8, 9, 10, 93];

/// Register the 5 English pyramid competitions on Jan 1
/// (`trigger_day_of_year = 1`).
///
/// Runtime justification: the `season_2001_02_v3_thiscall_fixed.jsonl`
/// capture observed the English Second Division's year field
/// advance `2001 → 2002` between the mid-season query at
/// `ms 1,326,057` (year 2001) and the mid-season regen at
/// `ms 2,226,283` (year 2002). The calendar year turn (Dec 31 →
/// Jan 1) is the only game-time boundary between those two events.
///
/// Other English pyramid comps are registered on the same day
/// under the assumption that the 5-tier structure fires as a
/// unit — which the C8 pyramid orchestrator (memory
/// `[[english-pyramid-final-graph]]`) also assumes. If further
/// runtime evidence shows they fire on different days, this
/// bootstrap is the one place to update.
pub fn register_english_pyramid(scheduler: &mut SeasonRollScheduler) {
    for comp_id in ENGLISH_PYRAMID_COMP_IDS {
        scheduler.register(comp_id, 1);
    }
}

// ---------------------------------------------------------------------------
// day_of_year helper
// ---------------------------------------------------------------------------

/// 1-based day of year for the given date. Uses the same
/// leap-year rule as `is_leap_year` in the top-level crate.
pub fn day_of_year(year: u16, month: u8, day: u8) -> u16 {
    let days_before: [u16; 12] =
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let leap = crate::is_leap_year(year);
    let mut d = days_before[(month.saturating_sub(1) as usize).min(11)]
        + day as u16;
    if leap && month > 2 { d += 1; }
    d
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_of_year_matches_reference_dates() {
        assert_eq!(day_of_year(2001, 1, 1), 1);
        assert_eq!(day_of_year(2001, 12, 31), 365);
        assert_eq!(day_of_year(2000, 12, 31), 366); // 2000 was leap
        assert_eq!(day_of_year(2001, 6, 30), 181);  // end of English season
    }

    #[test]
    fn register_accumulates_in_the_same_slot() {
        let mut s = SeasonRollScheduler::new();
        s.register(7, 1);
        s.register(8, 1);
        s.register(9, 1);
        assert_eq!(s.comps_for_day(1), vec![7, 8, 9]);
        assert_eq!(s.total_registered(), 3);
    }

    #[test]
    fn english_pyramid_bootstrap_registers_all_five() {
        let mut s = SeasonRollScheduler::new();
        register_english_pyramid(&mut s);
        let day1 = s.comps_for_day(1);
        for id in ENGLISH_PYRAMID_COMP_IDS {
            assert!(day1.contains(&id), "missing comp {}", id);
        }
        assert_eq!(s.total_registered(), 5);
    }

    #[test]
    fn fire_for_day_with_no_match_returns_empty() {
        let mut s = SeasonRollScheduler::new();
        register_english_pyramid(&mut s);
        let mut scratch = Vec::new();
        let mut ctx = SeasonRollContext {
            current_year: 2001,
            current_day_of_year: 100,
            scratch: &mut scratch,
        };
        let outcomes = s.fire_for_day::<_, fn(&mut SeasonRollContext<'_>) -> SeasonRollOutcome>(
            2001, 100, |_| None, &mut ctx,
        );
        assert!(outcomes.is_empty());
    }

    #[test]
    fn fire_for_day_fires_registered_comps_on_matching_day() {
        // A tiny CompetitionSeasonRoll impl that advances a
        // shared counter. Verifies that fire_for_day dispatches
        // to every registered comp on the matching day.
        use std::cell::RefCell;
        use std::rc::Rc;
        let fires: Rc<RefCell<Vec<u32>>> = Rc::new(RefCell::new(Vec::new()));

        let mut s = SeasonRollScheduler::new();
        register_english_pyramid(&mut s);

        let mut scratch = Vec::new();
        let mut ctx = SeasonRollContext {
            current_year: 2002,
            current_day_of_year: 1,
            scratch: &mut scratch,
        };

        let outcomes = s.fire_for_day(
            2002, 1,
            |comp_id| {
                let fires = fires.clone();
                Some(move |_c: &mut SeasonRollContext<'_>| {
                    fires.borrow_mut().push(comp_id);
                    SeasonRollOutcome::Fired {
                        comp_id,
                        old_year: 2001,
                        new_year: 2002,
                    }
                })
            },
            &mut ctx,
        );
        assert_eq!(outcomes.len(), 5);
        for out in &outcomes {
            assert!(matches!(out, SeasonRollOutcome::Fired { .. }));
        }
        // All 5 comps fired.
        let fired = fires.borrow();
        for id in ENGLISH_PYRAMID_COMP_IDS {
            assert!(fired.contains(&id), "comp {} didn't fire", id);
        }
    }

    #[test]
    fn same_year_second_fire_is_deduplicated() {
        // Once a slot has fired for calendar year N, refiring on
        // the same day within that year should skip everything
        // (guard against holiday-back-then-forward, leap-day
        // edge cases, etc.).
        let mut s = SeasonRollScheduler::new();
        s.register(7, 1);
        let mut scratch = Vec::new();
        let mut ctx = SeasonRollContext {
            current_year: 2002, current_day_of_year: 1,
            scratch: &mut scratch,
        };
        let first = s.fire_for_day(
            2002, 1,
            |_| Some(|_c: &mut SeasonRollContext<'_>| SeasonRollOutcome::Fired {
                comp_id: 7, old_year: 2001, new_year: 2002,
            }),
            &mut ctx,
        );
        assert!(matches!(first[0], SeasonRollOutcome::Fired { .. }));

        // Second fire same year, same day → skipped.
        let second = s.fire_for_day(
            2002, 1,
            |_| Some(|_c: &mut SeasonRollContext<'_>| SeasonRollOutcome::Fired {
                comp_id: 7, old_year: 2001, new_year: 2002,
            }),
            &mut ctx,
        );
        assert!(matches!(
            second[0],
            SeasonRollOutcome::Skipped {
                reason: SkipReason::AlreadyProcessedThisYear, ..
            }
        ));
    }

    #[test]
    fn different_years_both_fire() {
        let mut s = SeasonRollScheduler::new();
        s.register(7, 1);
        let mut scratch = Vec::new();
        let mut ctx = SeasonRollContext {
            current_year: 2002, current_day_of_year: 1,
            scratch: &mut scratch,
        };
        s.fire_for_day(2002, 1,
            |_| Some(|_c: &mut SeasonRollContext<'_>| SeasonRollOutcome::Fired {
                comp_id: 7, old_year: 2001, new_year: 2002,
            }), &mut ctx);
        // Next calendar year, same slot day → fires again.
        let out = s.fire_for_day(2003, 1,
            |_| Some(|_c: &mut SeasonRollContext<'_>| SeasonRollOutcome::Fired {
                comp_id: 7, old_year: 2002, new_year: 2003,
            }), &mut ctx);
        assert!(matches!(out[0], SeasonRollOutcome::Fired { .. }));
    }

    #[test]
    fn resolver_none_skips_gracefully() {
        // A comp registered but no impl provided (e.g. porting
        // work is in progress). Should skip, not panic.
        let mut s = SeasonRollScheduler::new();
        s.register(7, 1);
        let mut scratch = Vec::new();
        let mut ctx = SeasonRollContext {
            current_year: 2002, current_day_of_year: 1,
            scratch: &mut scratch,
        };
        let out = s.fire_for_day::<_, fn(&mut SeasonRollContext<'_>) -> SeasonRollOutcome>(
            2002, 1, |_| None, &mut ctx,
        );
        assert!(matches!(
            out[0], SeasonRollOutcome::Skipped {
                reason: SkipReason::Other(_), ..
            }
        ));
    }
}
