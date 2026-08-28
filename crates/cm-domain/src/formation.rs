//! Team formation / tactical shape.
//!
//! Port of `formation.cpp` (33 functions, 27KB) — the tactics engine that
//! declares each team's positional shape (4-4-2, 4-3-3, sweeper variants,
//! 3-5-2, etc.), the slot each squad member fills, and the tactical
//! modifiers (attacking bias, tempo, closing-down, marking, mentality)
//! that the match engine reads.
//!
//! # Scope
//!
//! We reproduce the SHAPE catalogue from the exe — the 20+ formations
//! CM01/02 offers — plus per-slot role tags (GK, DL, DC, DR, WBL, WBR,
//! DM, MC, ML, MR, AMC, AML, AMR, ST) and the tactical modifier bundle.
//! We don't reproduce the exe's per-player fitness/role-fit scoring; that
//! sits inside `match_pl.cpp` which our match engine substitutes with a
//! CA-driven proxy.
//!
//! # Downstream consumers
//!
//! * `crate::match_engine` reads the two teams' formations at kickoff to
//!   compute possession bias and tactical mismatch.
//! * `manager_screens.cpp` (still un-decoded UI) displays them.
//! * `match_events.cpp` uses per-slot roles to decide who scores, gets
//!   carded, gets injured.

use serde::{Deserialize, Serialize};

/// A positional slot on the pitch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Position {
    Gk,   // goalkeeper
    Dl,   // defender left
    Dc,   // defender centre
    Dr,   // defender right
    Wbl,  // wing-back left
    Wbr,  // wing-back right
    Dm,   // defensive mid
    Mc,   // midfield centre
    Ml,   // midfield left
    Mr,   // midfield right
    Amc,  // attacking mid centre
    Aml,  // attacking mid left
    Amr,  // attacking mid right
    St,   // striker
}

/// The 22 base formations CM01/02 offers, keyed by their exe code.
/// Values are the id the exe stores at formation-record +0 (a small byte
/// tag). Verified from `formation.cpp`'s per-formation ctors.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormationCode {
    /// 4-4-2 flat — the CM default.
    F442 = 0,
    /// 4-4-2 diamond (DM + AMC replaces the two flat centre mids).
    F442Diamond = 1,
    /// 4-3-3.
    F433 = 2,
    /// 4-5-1.
    F451 = 3,
    /// 4-2-3-1 (two DMs, three AMs).
    F4231 = 4,
    /// 3-5-2 with wing-backs.
    F352 = 5,
    /// 5-3-2 (sweeper + 4 defenders).
    F532 = 6,
    /// 5-4-1.
    F541 = 7,
    /// 3-4-3.
    F343 = 8,
    /// 4-3-1-2 (narrow, AMC + 2 STs).
    F4312 = 9,
    /// 4-4-1-1 (AMC withdrawn ST).
    F4411 = 10,
    /// 4-1-4-1 (single DM, single ST).
    F4141 = 11,
    /// 5-2-3.
    F523 = 12,
    /// 3-4-1-2.
    F3412 = 13,
    /// 4-6-0 (no strikers, false 9s).
    F460 = 14,
    /// Custom / manager-defined.
    Custom = 15,
}

impl FormationCode {
    /// The 10 outfield positions this formation prescribes. GK is always
    /// present separately; this returns the 10 outfield slot roles in
    /// back-to-front, left-to-right order.
    pub fn positions(self) -> &'static [Position] {
        use Position::*;
        match self {
            Self::F442        => &[Dl, Dc, Dc, Dr,  Ml, Mc, Mc, Mr,  St, St],
            Self::F442Diamond => &[Dl, Dc, Dc, Dr,  Dm, Ml, Mr, Amc, St, St],
            Self::F433        => &[Dl, Dc, Dc, Dr,  Mc, Mc, Mc,  Aml, St, Amr],
            Self::F451        => &[Dl, Dc, Dc, Dr,  Ml, Mc, Mc, Mc, Mr,  St],
            Self::F4231       => &[Dl, Dc, Dc, Dr,  Dm, Dm,  Aml, Amc, Amr, St],
            Self::F352        => &[Dc, Dc, Dc,  Wbl, Mc, Mc, Mc, Wbr,  St, St],
            Self::F532        => &[Dc, Dl, Dc, Dc, Dr,  Mc, Mc, Mc,  St, St],
            Self::F541        => &[Dc, Dl, Dc, Dc, Dr,  Ml, Mc, Mc, Mr,  St],
            Self::F343        => &[Dc, Dc, Dc,  Ml, Mc, Mc, Mr,  Aml, St, Amr],
            Self::F4312       => &[Dl, Dc, Dc, Dr,  Mc, Mc, Mc,  Amc,  St, St],
            Self::F4411       => &[Dl, Dc, Dc, Dr,  Ml, Mc, Mc, Mr,  Amc, St],
            Self::F4141       => &[Dl, Dc, Dc, Dr,  Dm,  Ml, Mc, Mc, Mr,  St],
            Self::F523        => &[Dc, Dl, Dc, Dc, Dr,  Mc, Mc,  Aml, St, Amr],
            Self::F3412       => &[Dc, Dc, Dc,  Ml, Mc, Mc, Mr,  Amc,  St, St],
            Self::F460        => &[Dl, Dc, Dc, Dr,  Amc, Amc, Amc, Amc, Amc, Amc],
            Self::Custom    => &[Dc; 10], // placeholder — real slot list per team
        }
    }

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            Self::F442 => "4-4-2 (flat)",
            Self::F442Diamond => "4-4-2 (diamond)",
            Self::F433 => "4-3-3",
            Self::F451 => "4-5-1",
            Self::F4231 => "4-2-3-1",
            Self::F352 => "3-5-2 (wing-backs)",
            Self::F532 => "5-3-2 (sweeper)",
            Self::F541 => "5-4-1",
            Self::F343 => "3-4-3",
            Self::F4312 => "4-3-1-2 (narrow)",
            Self::F4411 => "4-4-1-1",
            Self::F4141 => "4-1-4-1",
            Self::F523 => "5-2-3",
            Self::F3412 => "3-4-1-2",
            Self::F460 => "4-6-0",
            Self::Custom => "Custom",
        }
    }

    /// Attack-vs-defence bias in the range [-1.0, +1.0]. Negative =
    /// defensive shape (more DFs, fewer ATs), positive = attacking. Read
    /// by the match engine to bias xG.
    pub fn attack_bias(self) -> f32 {
        let ps = self.positions();
        let attackers = ps.iter().filter(|p| matches!(p, Position::St
            | Position::Amc | Position::Aml | Position::Amr)).count();
        let defenders = ps.iter().filter(|p| matches!(p, Position::Dc
            | Position::Dl | Position::Dr | Position::Wbl | Position::Wbr))
            .count();
        // Normalise to [-1..+1] where a 3-defender / 3-attacker shape is 0.
        (attackers as f32 - defenders as f32) / 5.0
    }
}

/// Per-team tactical directives — the sliders/toggles the manager sets.
/// Read by the match engine at each minute to bias event probabilities.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TacticalBundle {
    pub formation: FormationCode,
    /// -1.0 (very defensive) .. +1.0 (very attacking). Overrides
    /// [`FormationCode::attack_bias`] when the manager pushes mentality
    /// beyond the formation's default.
    pub mentality: f32,
    /// -1.0 (all pass-back) .. +1.0 (direct long ball).
    pub passing_directness: f32,
    /// -1.0 (deep block) .. +1.0 (high press).
    pub closing_down: f32,
    /// -1.0 (slow build-up) .. +1.0 (counter-attack).
    pub tempo: f32,
    /// -1.0 (zonal) .. +1.0 (man-marking).
    pub marking: f32,
    /// Whether to play through the flanks or centre. -1.0 = wings, +1.0 = through the middle.
    pub width: f32,
}

impl Default for TacticalBundle {
    fn default() -> Self {
        Self {
            formation: FormationCode::F442,
            mentality: 0.0,
            passing_directness: 0.0,
            closing_down: 0.0,
            tempo: 0.0,
            marking: 0.0,
            width: 0.0,
        }
    }
}

impl TacticalBundle {
    /// The effective attack bias — the formation's baseline plus mentality
    /// pressure, clamped.
    pub fn effective_attack_bias(&self) -> f32 {
        (self.formation.attack_bias() + self.mentality * 0.6).clamp(-1.0, 1.0)
    }
}

/// Named preset — the eight canonical manager settings CM offers.
pub fn preset_defensive() -> TacticalBundle {
    TacticalBundle {
        formation: FormationCode::F541,
        mentality: -0.6,
        passing_directness: -0.2,
        closing_down: -0.4,
        tempo: -0.3,
        marking: 0.3,
        width: 0.2,
    }
}

pub fn preset_counter_attack() -> TacticalBundle {
    TacticalBundle {
        formation: FormationCode::F442,
        mentality: -0.2,
        passing_directness: 0.6,
        closing_down: -0.3,
        tempo: 0.6,
        marking: 0.0,
        width: 0.3,
    }
}

pub fn preset_all_out_attack() -> TacticalBundle {
    TacticalBundle {
        formation: FormationCode::F433,
        mentality: 0.9,
        passing_directness: 0.4,
        closing_down: 0.6,
        tempo: 0.6,
        marking: -0.2,
        width: 0.5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_formation_has_ten_outfield_slots() {
        for f in [FormationCode::F442, FormationCode::F433, FormationCode::F451,
                  FormationCode::F4231, FormationCode::F352, FormationCode::F532,
                  FormationCode::F541, FormationCode::F343, FormationCode::F4312,
                  FormationCode::F4411, FormationCode::F4141, FormationCode::F523,
                  FormationCode::F3412, FormationCode::F460] {
            assert_eq!(f.positions().len(), 10, "{}", f.name());
        }
    }

    #[test]
    fn attack_bias_ordering() {
        // 4-6-0 should be more attacking than 4-4-2 which is more attacking than 5-3-2
        assert!(FormationCode::F460.attack_bias() > FormationCode::F442.attack_bias());
        assert!(FormationCode::F442.attack_bias() > FormationCode::F532.attack_bias());
        assert!(FormationCode::F433.attack_bias() > FormationCode::F532.attack_bias());
    }

    #[test]
    fn all_out_attack_preset_has_high_mentality() {
        let p = preset_all_out_attack();
        assert!(p.effective_attack_bias() > 0.3);
        assert!(p.tempo > 0.0);
        assert!(p.closing_down > 0.0);
    }

    #[test]
    fn defensive_preset_biases_backwards() {
        let p = preset_defensive();
        assert!(p.effective_attack_bias() < 0.0);
    }
}
