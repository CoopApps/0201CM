//! Player development — a faithful port of CM0102's training system
//! (`FUN_0089de50` weekly tick + `FUN_0089f350` attribute application +
//! `FUN_008a0940` coach quality). Replaces the CA-level heuristic in
//! `training.rs` with the real per-attribute, coach-driven, schedule-driven
//! model shown on the game's Training screens.
//!
//! # The real model (decompiled)
//!
//! For each of the five training categories — Fitness, Tactics, Shooting,
//! Skills, Goalkeeping — the weekly tick computes an **effectiveness**:
//! ```text
//!   eff = clamp( intensity*2 + 60 + (coachAvg - 100)/5 , 0, 200 )   (0089de50:240)
//! ```
//! `intensity` is the schedule dial (None/Light/Medium/Intensive = 0..3);
//! `coachAvg` is the club's averaged coach rating in that category (base 100).
//! Effectiveness then drives a per-week random grow/decline walk of a signed
//! accumulator, bounded by two hidden per-player mentals (growth-headroom at
//! player+0x58, decline-floor at player+0x5b, each /3) with diminishing
//! returns (`0089de50:282-304`). The accumulated delta is applied to the
//! category's attributes (`0089f350`).
//!
//! # Category → attribute map (alphabetical type10 indices, verified off the
//! `FUN_0089f350` offset writes, offset−0x1b):
//!   Fitness    = Acceleration, Agility, Jumping, Natural Fitness, Pace,
//!                Reflexes, Stamina, Strength   (all physical)
//!   Tactics    = Decisions, Marking, Off The Ball, Positioning, Teamwork
//!   Shooting   = Finishing, Long Shots, Penalties
//!   Skills     = Corners, Crossing, Dribbling, Set Pieces, Heading, Passing,
//!                Tackling, Technique, Throw Ins
//!   Goalkeeping= Handling, One On Ones
//!
//! The exe stores some of these as ±125 signed deltas at 6× resolution around a
//! base byte, others as absolute 1..20; in *display* terms (base + stored/6)
//! both change by ~`d` per tick, so we apply `+d` to our uniform 1..20 model.
//!
//! # Documented approximations (flagged, not hidden)
//! * The two hidden mentals (player-object +0x58/+0x5b) aren't in our shipped
//!   record set; we use a neutral constant [`NEUTRAL_MENTAL`]. Per-player
//!   development spread therefore comes from coach/intensity/category/GK-gate,
//!   not the mental — a refinement once those bytes are located.
//! * The runtime coach-rating snapshot (0..200-ish) is computed from a coach's
//!   raw 0..20 stats by a formula we haven't decoded; we map a category stat
//!   `s` to `50 + s*5` (10→100 neutral, 20→150, floor 50) and average across
//!   the club's coaches. Effectiveness only *modulates* on coaches, so the
//!   intensity-driven behaviour is faithful regardless.

use serde::{Deserialize, Serialize};

/// The five training categories, in the exe's order (0089de50 `local_424`).
pub const CAT_FITNESS: usize = 0;
pub const CAT_TACTICS: usize = 1;
pub const CAT_SHOOTING: usize = 2;
pub const CAT_SKILLS: usize = 3;
pub const CAT_GOALKEEPING: usize = 4;

/// Category → the DFM-order 42-attribute indices it trains (from `FUN_0089f350`
/// offset writes; index = type10 offset − 0x1b, remapped through the
/// authoritative DFM order verified by the editor decode agent).
///
/// Names (DFM order): Acceleration(0), Aggression(1), Agility(2),
/// Anticipation(3), Balance(4), Bravery(5), Consistency(6), Corners(7),
/// Crossing(8), Free Kicks(9), Handling(10), Heading(11), Important Matches(12),
/// Injury Proneness(13), Jumping(14), Leadership(15), Left Foot(16),
/// Long Shots(17), Dirtiness(18), Dribbling(19), Finishing(20), Flair(21),
/// Decisions(22), Movement(23), Natural Fitness(24), One On Ones(25),
/// Marking(26), Pace(27), Passing(28), Penalties(29), Positioning(30),
/// Reflexes(31), Right Foot(32), Stamina(33), Strength(34), Tackling(35),
/// Teamwork(36), Throw Ins(37), Versatility(38), Vision(39), Work Rate(40),
/// Technique(41).
pub const CATEGORY_ATTRS: [&[usize]; 5] = [
    // Fitness (physical): Acceleration, Agility, Balance, Jumping, Natural Fitness,
    // Pace, Stamina, Strength
    &[0, 2, 4, 14, 24, 27, 33, 34],
    // Tactics: Anticipation, Decisions, Marking, Movement, Positioning, Teamwork
    &[3, 22, 26, 23, 30, 36],
    // Shooting: Finishing, Long Shots, Penalties
    &[20, 17, 29],
    // Skills: Corners, Crossing, Dribbling, Free Kicks, Heading, Passing,
    // Tackling, Technique, Throw Ins
    &[7, 8, 19, 9, 11, 28, 35, 41, 37],
    // Goalkeeping: Handling, One On Ones, Reflexes
    &[10, 25, 31],
];

/// Neutral value for the two hidden development mentals (player+0x58/+0x5b),
/// which our shipped records don't carry. /3 ⇒ 5 in the walk's headroom/floor.
pub const NEUTRAL_MENTAL: i8 = 15;

/// Coach-stat indices within `DomainStaffType9::coaching_attributes()` (the
/// editor's non-player order) for each training category.
///   Shooting→attacking(0), Fitness→coaching(2), Goalkeeping→coaching_gk(3),
///   Skills→coaching_technique(4), Tactics→tactics(19).
const COACH_STAT_FOR_CAT: [usize; 5] = [2, 19, 0, 4, 3];

/// One player's live, trainable state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlayerDev {
    pub staff_id: u32,
    pub club_id: Option<i32>,
    /// The 42 attributes (alphabetical), mutated weekly by training.
    pub attributes: Vec<u8>,
    /// type10 +0x0f goalkeeping-aptitude byte — gates the GK throttle.
    pub gk_aptitude: i8,
    /// Growth-headroom mental — the exe's player+0x58. Not in our shipped
    /// records, so PROXIED from PA-CA headroom (talented youngsters with room
    /// to grow get a higher cap). Flagged approximation for the exact byte.
    pub growth_mental: i8,
    /// Decline-floor mental — the exe's player+0x5b. PROXIED from age (players
    /// past their late-20s peak decline). Flagged approximation.
    pub decline_mental: i8,
    /// Per-category progress memory (the exe's stored nibble hysteresis).
    pub progress: [i8; 5],
    /// Intensity dials per category (0=None..3=Intensive).
    pub schedule: [u8; 5],
}

/// Per-club averaged coach rating (base 100) for each of the five categories.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct PlayerDevelopmentBook {
    pub players: Vec<PlayerDev>,
    /// club_id → [coachAvg per category] (base 100; 100 = no/neutral coaching).
    pub club_coaching: std::collections::BTreeMap<i32, [i32; 5]>,
    #[serde(skip)]
    id_index: std::collections::HashMap<u32, usize>,
}

/// Training-intensity dial magnitudes (`FUN_008a15c0`): None/Light/Medium/
/// Intensive. These feed `intensity*2` in the effectiveness formula, so their
/// scale is what puts a Medium schedule (2·25+60 = 110) into the grow zone
/// (≥101) and leaves None (60) declining.
pub const DIAL_NONE: u8 = 0;
pub const DIAL_LIGHT: u8 = 10;
pub const DIAL_MEDIUM: u8 = 25; // 0x19
pub const DIAL_INTENSIVE: u8 = 50; // 0x32

/// The default schedules the exe's preset builder produces: outfielders get the
/// "General" preset (Medium on all four outfield categories, GK off); keepers
/// get the "Gk" preset (Gk Intensive, others Light). Per-player schedule
/// customisation from the UI/save is a follow-up.
fn default_schedule(is_gk: bool) -> [u8; 5] {
    if is_gk {
        // Gk preset (case 6): Fit/Tac/Ski Light, Shooting off, Gk Intensive.
        [DIAL_LIGHT, DIAL_LIGHT, DIAL_NONE, DIAL_LIGHT, DIAL_INTENSIVE]
    } else {
        // General preset (case 1): Medium on Fit/Tac/Sho/Ski, Gk off.
        [DIAL_MEDIUM, DIAL_MEDIUM, DIAL_MEDIUM, DIAL_MEDIUM, DIAL_NONE]
    }
}

impl PlayerDevelopmentBook {
    /// Build the development book from the generated init states (kill #6
    /// attributes), the person→club links (type6), the GK-aptitude byte
    /// (type10 +0x0f), and the club coaching pools (type9 coach stats).
    pub fn build(
        init: &[crate::PlayerInitState],
        staff: &crate::StaffBook,
    ) -> Self {
        use std::collections::{BTreeMap, HashMap};
        // person id → current club id.
        let club_of: HashMap<u32, i32> = staff.type6.iter()
            .filter_map(|p| p.current_club_id().map(|c| (p.id, c as i32)))
            .collect();
        // type10 by id (for the GK-aptitude byte +0x0f).
        let t10: HashMap<u32, &crate::DomainStaffType10> =
            staff.type10.iter().map(|t| (t.id, t)).collect();

        // Club coaching: average each qualifying coach's category rating.
        // A coach = a person (type6) with a type9 record and a club.
        let t9: HashMap<u32, &crate::DomainStaffType9> =
            staff.type9.iter().map(|t| (t.id, t)).collect();
        let mut sums: BTreeMap<i32, ([i64; 5], [i64; 5])> = BTreeMap::new(); // (sum, count) per cat
        for p in &staff.type6 {
            let (Some(&club), Some(coach)) = (club_of.get(&p.id).map(|c| c), t9.get(&p.id)) else {
                continue;
            };
            let ca = coach.coaching_attributes();
            let entry = sums.entry(club).or_insert(([0; 5], [0; 5]));
            for cat in 0..5 {
                let stat = ca[COACH_STAT_FOR_CAT[cat]] as i64;
                if stat > 0 {
                    entry.0[cat] += 50 + stat * 5; // → base-100 scale
                    entry.1[cat] += 1;
                }
            }
        }
        let club_coaching: BTreeMap<i32, [i32; 5]> = sums.into_iter().map(|(club, (s, c))| {
            let mut avg = [100i32; 5];
            for cat in 0..5 {
                if c[cat] > 0 {
                    avg[cat] = (s[cat] / c[cat]) as i32;
                }
            }
            (club, avg)
        }).collect();

        let mut players = Vec::with_capacity(init.len());
        for st in init {
            if st.attributes.len() < 42 {
                continue;
            }
            let gk_aptitude = t10.get(&st.player_id)
                .map(|t| t.full_attributes()[0] as i8)
                .unwrap_or(0);
            let is_gk = gk_aptitude >= 0x13;
            // Mental proxies (flagged): growth cap from PA headroom, decline
            // onset from age. See field docs.
            let headroom = (st.potential_ability - st.current_ability).max(0) as i32;
            let growth_mental = (headroom / 2 + 3).clamp(3, 45) as i8;
            let age = st.age.unwrap_or(24) as i32;
            let decline_mental = if age < 30 {
                45
            } else {
                (45 - (age - 30) * 4).clamp(3, 45)
            } as i8;
            players.push(PlayerDev {
                staff_id: st.player_id,
                club_id: club_of.get(&st.player_id).copied(),
                attributes: st.attributes.clone(),
                gk_aptitude,
                growth_mental,
                decline_mental,
                progress: [0; 5],
                schedule: default_schedule(is_gk),
            });
        }
        let id_index = players.iter().enumerate().map(|(i, p)| (p.staff_id, i)).collect();
        Self { players, club_coaching, id_index }
    }

    /// Effectiveness for one category — `FUN_0089de50:240`.
    fn effectiveness(intensity: u8, coach_avg: i32) -> i32 {
        (intensity as i32 * 2 + 60 + (coach_avg - 100) / 5).clamp(0, 200)
    }

    /// The per-week grow/decline random walk for one category — a direct port
    /// of `FUN_0089de50:282-304`. Returns the signed accumulator after `weeks`.
    fn category_walk(
        eff: i32,
        growth3: i32,
        decline3: i32,
        gk_aptitude: i8,
        is_gk_cat: bool,
        weeks: u32,
        prior: i8,
        rng: &mut impl crate::game_rng::PoolRand,
    ) -> i8 {
        let mut acc: i32 = 0;
        for _ in 0..weeks {
            let combined = prior as i32 + acc; // cVar12
            // GK-aptitude gate (0089de50:282/307): keepers throttle non-GK work.
            if gk_aptitude >= 0x13 {
                let n: u32 = if is_gk_cat { 2 } else { 4 };
                if rng.range(n) != 0 {
                    continue;
                }
            }
            let dim: u32 = (combined.abs() * 2 + 0xe) as u32; // diminishing-returns window
            // decline-eligible? (eff<101, or the rand miss, or hit headroom)
            let decline_eligible = eff < 0x65
                || rng.range((0xd2 - eff) as u32) as i32 > 0x13
                || growth3 <= combined;
            if decline_eligible {
                if eff < 0x6e
                    && (rng.range((eff + 0x3c) as u32) as i32) < 0x14
                    && combined > -7
                    && (decline3 - 6) < combined
                    && (rng.range(dim) as i32) < 10
                {
                    acc -= 1; // DECLINE
                }
            } else if (rng.range(dim) as i32) < 10 {
                acc += 1; // GROW
            }
        }
        acc.clamp(-125, 125) as i8
    }

    fn ensure_index(&mut self) {
        if self.id_index.is_empty() && !self.players.is_empty() {
            self.id_index = self.players.iter().enumerate()
                .map(|(i, p)| (p.staff_id, i)).collect();
        }
    }

    /// One training pass over every player, advancing `weeks` weeks (the exe's
    /// lazy catch-up, capped at 30). Mutates attributes in place.
    pub fn weekly_tick(&mut self, weeks: u32, rng: &mut impl crate::game_rng::PoolRand) {
        let weeks = weeks.min(0x1e).max(1);
        for dev in &mut self.players {
            let coach = dev.club_id
                .and_then(|c| self.club_coaching.get(&c).copied())
                .unwrap_or([100; 5]);
            let growth3 = dev.growth_mental as i32 / 3;
            let decline3 = dev.decline_mental as i32 / 3;
            for cat in 0..5 {
                let eff = Self::effectiveness(dev.schedule[cat], coach[cat]);
                let acc = Self::category_walk(
                    eff, growth3, decline3, dev.gk_aptitude,
                    cat == CAT_GOALKEEPING, weeks, dev.progress[cat], rng,
                );
                // Half-step blend with the stored nibble (0089f350:34/118).
                let prior = dev.progress[cat] as i32;
                let d = (acc as i32 + prior) / 2 - prior / 2;
                if d != 0 {
                    for &ai in CATEGORY_ATTRS[cat] {
                        if let Some(v) = dev.attributes.get_mut(ai) {
                            *v = (*v as i32 + d).clamp(1, 20) as u8;
                        }
                    }
                }
                dev.progress[cat] = acc;
            }
        }
    }

    /// Total absolute attribute change since a baseline snapshot — for
    /// verification/inspection.
    pub fn attribute_of(&mut self, staff_id: u32, idx: usize) -> Option<u8> {
        self.ensure_index();
        self.id_index.get(&staff_id).and_then(|&i| self.players[i].attributes.get(idx).copied())
    }
}
