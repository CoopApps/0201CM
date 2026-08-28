//! Typed borrow-only facade over the world pools used by the
//! `populate_from_world` companions in `screen_batch{3..9}`.
//!
//! Complements the string-keyed [`crate::world_facade::WorldFacade`]
//! (which the wave-A batches 18..27 use for opaque handle lookups) by
//! exposing the actual typed pools — clubs, players, staff,
//! competitions, nations — plus the runtime active-human seat pointer
//! (`DAT_00b5d016`). This lets the wave-A batches 3..9 populate their
//! views from real record fields (e.g. `players[N].current_club_id()`
//! at body+0x35) instead of from named opaque handles.
//!
//! # Field mapping
//! * `clubs`      — `world.core.clubs`
//! * `nations`    — `world.core.nations`
//! * `staff`      — `world.staff.type6` (person records; `current_club_id`
//!                  at body+0x35 is the "which club is this manager at?"
//!                  back-pointer read by many screen setups)
//! * `players`    — `world.staff.type10` (player-attribute records)
//! * `comps`      — `world.references.club_competitions` (club-level
//!                  competitions — `three_letter_name` non-empty ==
//!                  manageable league, per [`crate::World::is_manageable_league`])
//! * `active_human_seat` — the exe's `DAT_00b5d016`; when the save
//!                  overlay is available this equals
//!                  `save.world.active_human as u32`. Kept optional
//!                  because pre-new-game screens run before any seat
//!                  exists.

use crate::{DomainCompetition, DomainOpaqueRecord, DomainStaffType10, DomainStaffType6, World};

/// Borrow-only bundle of world pools + runtime seat pointer.
#[derive(Debug, Clone, Copy)]
pub struct WorldPools<'a> {
    /// Base-club pool (`core.clubs`).
    pub clubs: &'a [DomainOpaqueRecord],
    /// Nation pool (`core.nations`).
    pub nations: &'a [DomainOpaqueRecord],
    /// Staff / person pool (`staff.type6`).
    pub staff: &'a [DomainStaffType6],
    /// Player-attribute pool (`staff.type10`).
    pub players: &'a [DomainStaffType10],
    /// Club-competition pool (`references.club_competitions`).
    pub comps: &'a [DomainCompetition],
    /// Active human seat index (`DAT_00b5d016`); `None` when no human
    /// is seated (fresh new-game screen, sim mode).
    pub active_human_seat: Option<u32>,
}

impl<'a> WorldPools<'a> {
    /// Borrow-only bundle from a `World` + the runtime seat pointer.
    pub fn new(world: &'a World, active_human_seat: Option<u32>) -> Self {
        Self {
            clubs: &world.core.clubs,
            nations: &world.core.nations,
            staff: &world.staff.type6,
            players: &world.staff.type10,
            comps: &world.references.club_competitions,
            active_human_seat,
        }
    }

    /// An empty facade — the pre-new-game default used by many
    /// batch-`populate` unit tests.
    pub fn empty() -> Self {
        Self {
            clubs: &[],
            nations: &[],
            staff: &[],
            players: &[],
            comps: &[],
            active_human_seat: None,
        }
    }

    /// The active human's current club id — resolved via the staff
    /// pool's `current_club_id()` accessor (record body+0x35). `None`
    /// when no human is seated or the seat person isn't in the staff
    /// pool.
    pub fn active_human_club_id(&self) -> Option<u32> {
        let sid = self.active_human_seat?;
        self.staff
            .iter()
            .find(|s| s.id == sid)
            .and_then(|s| s.current_club_id())
    }

    /// A reasonable "focus competition" for the active human — the
    /// first competition whose `three_letter_name` is populated
    /// (manageable league marker), or the first competition of any
    /// kind as a fallback. `None` when the comp pool is empty.
    ///
    /// The exe's precise resolution reads club+0x27 (nation) → nation's
    /// primary league; that needs decoded club records. Documented in
    /// each caller's `TODO_POPULATOR_INFO`.
    pub fn active_human_focus_competition(&self) -> Option<u32> {
        self.comps
            .iter()
            .find(|c| !c.three_letter_name.is_empty())
            .or_else(|| self.comps.first())
            .map(|c| c.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_pools_yield_no_ids() {
        let p = WorldPools::empty();
        assert!(p.active_human_seat.is_none());
        assert!(p.active_human_club_id().is_none());
        assert!(p.active_human_focus_competition().is_none());
    }

    #[test]
    fn focus_competition_prefers_manageable_league() {
        let comps = vec![
            DomainCompetition {
                id: 10,
                long_name: "Cup".into(),
                short_name: "Cup".into(),
                three_letter_name: String::new(),
                scope: 2,
                nation_id: 1,
                last_division: -1,
                reserve_division: -1,
                reputation: 0,
                unknown_tail: vec![],
            },
            DomainCompetition {
                id: 20,
                long_name: "Premier".into(),
                short_name: "Prem".into(),
                three_letter_name: "PRM".into(),
                scope: 2,
                nation_id: 1,
                last_division: -1,
                reserve_division: -1,
                reputation: 100,
                unknown_tail: vec![],
            },
        ];
        let p = WorldPools {
            clubs: &[], nations: &[], staff: &[], players: &[],
            comps: &comps, active_human_seat: None,
        };
        assert_eq!(p.active_human_focus_competition(), Some(20));
    }
}
