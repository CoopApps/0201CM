//! `.tct` / `.pct` tactic-file reader — canonical `Tactic` model that
//! `snapshot_team_for_engine` can consume in place of the hardcoded
//! `FLAT_442_ROLES`.
//!
//! Full format decode: [`../../reports/tactic_file_decode.md`].
//! Format primer:
//!   * Both `.tct` and `.pct` use the same body; `.pct` only obfuscates the
//!     version-tag u32 at file offset 0 by ADDING `0x075BCD15`.
//!   * Six known versions `0x0098EC59..0x0098EC5E`; only v5C/v5D/v5E appear
//!     in shipped `Data/*.pct` and user `tactics/*.tct` files.
//!   * v5E body size = 1472 B; v5C/v5D body size = 1428 B; a `0xF2FFFFF2`
//!     4-byte tail sentinel may follow (loader ignores it — some saves
//!     write it, some don't).
//!
//! Loader:
//!   * `load_tactic(path)` — reads a `.tct` or `.pct` file end-to-end and
//!     returns a [`Tactic`] with 11 per-slot [`TacticSlot`] rows plus the
//!     global team-wide settings.
//!   * Handles both extensions (auto-detects `.pct` from suffix).
//!   * Does NOT yet expand v5C/v5D (44-B slot-pair block) to v5E (88-B) —
//!     the eleven role-mask + aux + depth + slot-flag fields all live at
//!     the same file offsets and are what the XI picker actually needs.

use serde::{Deserialize, Serialize};

/// A canonical, decode-once tactic. Carries only what the sim currently
/// consumes; leave the widget-only fields (per-slot instructions blob,
/// area-A defaults) unparsed for now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tactic {
    pub formation_name: String,
    pub author: String,
    /// Team-wide mentality. 3 states, NOT 5 (earlier assumption was wrong —
    /// see `reports/team_flags_2_decode.md`). Bits 4..6 of `team_flags_2`,
    /// one-hot. Use [`team_settings`] for a full typed decode of every
    /// team-wide switch.
    pub mentality: Mentality,
    /// Team-wide flag word #2 — mentality (bits 0..4) plus the other 10
    /// on/off team switches. Bit-level mapping TBD (see report §5); the
    /// XI picker doesn't need it yet, but downstream match-tick code will.
    pub team_flags_2: u32,
    /// Team-wide flag word #1 — preset provenance / reserve-usable flags.
    pub team_flags_1: u32,
    /// 11 per-position slot rows, in role order (GK first, then defenders,
    /// midfielders, attackers).
    pub slots: [TacticSlot; 11],
    /// Per-slot 96-byte positional-play grid — the 2×3×4 waypoints each
    /// slot's player targets during possession vs defence, decoded from
    /// file offset 0x0139 + slot*96. Populated at parse time so callers
    /// don't need to keep the raw bytes around.
    ///
    /// Tactics gap #5 wire — the match engine can index into this to
    /// bias per-tick token movement (`token.target = grid[side][row][col]`)
    /// instead of using role-mask fallbacks.
    #[serde(default = "default_slot_instructions_array")]
    pub slot_instructions: [SlotInstructions; 11],
    /// File version tag (canonical form, obfuscation removed).
    pub version: u32,
    /// True if loaded from a `.pct` (packaged preset), false for `.tct`.
    pub is_packaged: bool,
}

fn default_slot_instructions_array() -> [SlotInstructions; 11] {
    [SlotInstructions::default(); 11]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TacticSlot {
    /// One-hot role mask over the 12 roles GK/SW/D/DM/M/AM/ST/WB/RS/LS/C/FR.
    /// This is the value `tactics::position_rating` takes as `position_mask`.
    pub role_mask: u16,
    /// Aux role byte (added at v5C, `0` for legacy).
    pub aux_role: u8,
    /// Depth / formation-slot short (positional coordinate).
    pub depth: u16,
    /// First u32 of the per-slot pair — movement token (opaque).
    pub movement_token: u32,
    /// Second u32 of the per-slot pair — version flag (constant `10` on
    /// legacy files, real value on v5E).
    pub movement_flag: u32,
    /// Per-slot flag byte at file `+0x5B5`. `0x11` = normal.
    pub flag: u8,
}

/// Team-level tempo bitmask — VERIFIED port of FUN_006c34c0:63-83.
/// Applied via FUN_0059f3e0(mask) at the top of the per-team tactic
/// decision pass. Reads manager's +0x13 attribute.
///
///   attr < 9  → 0x02 (slow)
///   attr < 15 → 0x01 (normal)
///   attr < 19 → 0x04 (quick)
///   attr < 21 → 0x08 (very quick)
///   else      → None (unset, uses default)
///
/// See reports/contract_tactic_comp_giants.md §FUN_006c34c0.
pub fn team_tempo_mask(mgr_attr_0x13: i8) -> Option<u32> {
    match mgr_attr_0x13 {
        i8::MIN..=8 => Some(0x02),
        9..=14      => Some(0x01),
        15..=18     => Some(0x04),
        19..=20     => Some(0x08),
        _           => None,
    }
}

/// Team-level mentality bitmask — VERIFIED port of FUN_006c34c0:84-91.
/// Reads manager's +0x1f attribute:
///   attr < 6  → 0     (unset)
///   attr < 15 → 0x80  (defensive)
///   else      → 0x100 (attacking)
pub fn team_mentality_mask(mgr_attr_0x1f: i8) -> u32 {
    if mgr_attr_0x1f < 6 { 0 }
    else if mgr_attr_0x1f < 15 { 0x80 }
    else { 0x100 }
}

impl Tactic {
    /// Boot-time default 4-4-2 preset — plants a viable Tactic on every
    /// club so `snapshot_team_for_engine` reads a real per-club Tactic
    /// rather than the `FLAT_442_ROLES` fallback. Role masks come from
    /// [`crate::tactics::FLAT_442_ROLES`], the same bit set the fallback
    /// used. Team settings default to Unset (engine treats as "none set,
    /// use exe defaults"). Overwritten per-club by [`RuntimeSaveGame::assign_tactic`]
    /// when the human opens the tactics screen.
    pub fn flat_442() -> Self {
        let roles = crate::tactics::FLAT_442_ROLES;
        let mut slots = [TacticSlot::default(); 11];
        for (i, mask) in roles.iter().enumerate() {
            slots[i].role_mask = *mask;
            slots[i].flag = 0x11;
            slots[i].movement_flag = 10; // legacy version constant
        }
        Tactic {
            formation_name: "4-4-2 (Flat)".to_string(),
            author: "cm0102-rs default".to_string(),
            mentality: Mentality::Unset,
            team_flags_2: 0,
            team_flags_1: 0,
            slots,
            slot_instructions: [SlotInstructions::default(); 11],
            version: 0x0098EC5C, // v5C — legacy safe (any of 5C..5E works)
            is_packaged: false,
        }
    }
}

/// Load a tactic file. Auto-detects `.pct` (packaged/preset) vs `.tct`
/// (user-authored) from the extension.
pub fn load_tactic(path: &std::path::Path) -> std::io::Result<Tactic> {
    let bytes = std::fs::read(path)?;
    let is_packaged = path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.eq_ignore_ascii_case("pct"))
        .unwrap_or(false);
    parse_tactic(&bytes, is_packaged)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData,
            format!("not a recognised tactic file: {}", path.display())))
}

/// Parse tactic bytes. Returns `None` if the version tag is not one of the
/// six known versions.
pub fn parse_tactic(bytes: &[u8], is_packaged: bool) -> Option<Tactic> {
    if bytes.len() < 0x5C0 { return None; }
    let version = {
        let raw = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
        if is_packaged { raw.wrapping_sub(0x075BCD15) } else { raw }
    };
    if !(0x0098EC59..=0x0098EC5E).contains(&version) {
        return None;
    }

    // Formation name — bit-inverted ASCII c-string at +0x04, cap 50 bytes.
    let formation_name: String = bytes[0x04..0x36].iter()
        .map(|&b| if b == 0xff { 0xff } else { !b })
        .take_while(|&b| b != 0)
        .map(|b| b as char)
        .collect();

    // Author — ASCII inside area A at file +0x39, cap 64.
    let author = {
        let raw = &bytes[0x39..0x39 + 0x40];
        let n = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
        String::from_utf8_lossy(&raw[..n]).to_string()
    };

    let team_flags_1 = u32::from_le_bytes(bytes[0x00FE..0x0102].try_into().ok()?);
    let team_flags_2 = u32::from_le_bytes(bytes[0x0559..0x055D].try_into().ok()?);
    let mentality = match team_flags_2 & 0x0000_0070 {
        0x10 => Mentality::Normal,
        0x20 => Mentality::Defensive,
        0x40 => Mentality::Attacking,
        _    => Mentality::Unset,
    };

    let mut slots = [TacticSlot::default(); 11];
    for i in 0..11 {
        slots[i].role_mask = u16::from_le_bytes(
            bytes[0x0102 + i*2..0x0102 + i*2 + 2].try_into().ok()?);
        // v5C+ has the aux-role byte block; v5B and earlier do not — file is
        // too small to hold it and this branch is dead in shipped data.
        slots[i].aux_role = if version >= 0x0098EC5C {
            bytes[0x0118 + i]
        } else { 0 };
        slots[i].depth = u16::from_le_bytes(
            bytes[0x0123 + i*2..0x0123 + i*2 + 2].try_into().ok()?);
        // Slot pair — v5E has 8 bytes per slot; v5C/v5D have 4 (u32) and the
        // loader synthesises the second u32 as constant `10`.
        let pair_off = 0x055D + i * (if version >= 0x0098EC5E { 8 } else { 4 });
        slots[i].movement_token = u32::from_le_bytes(
            bytes[pair_off..pair_off + 4].try_into().ok()?);
        slots[i].movement_flag = if version >= 0x0098EC5E {
            u32::from_le_bytes(bytes[pair_off + 4..pair_off + 8].try_into().ok()?)
        } else { 10 };
        // Slot flag byte lives after the slot pair block. For v5E its file
        // offset is 0x05B5; for v5C/v5D the file offset is 0x0589 (pair block
        // is 44 B not 88 B). Compute from the pair-block size.
        let flag_base = if version >= 0x0098EC5E { 0x05B5 } else { 0x0589 };
        slots[i].flag = bytes[flag_base + i];
    }

    // Positional-play waypoint grid — populate all 11 slots from the
    // 96-byte per-slot blocks at file offset 0x0139 + slot*96.
    let mut slot_instructions = [SlotInstructions::default(); 11];
    for i in 0..11 {
        slot_instructions[i] = slot_instructions_from_bytes(bytes, i);
    }

    Some(Tactic {
        formation_name, author, mentality,
        team_flags_1, team_flags_2, slots, slot_instructions,
        version, is_packaged,
    })
}

/// Load every `.pct` shipped in `dir` (typically `D:/cm0102/Data`) into a
/// `name → Tactic` map. The `name` is the file stem (`342_default`,
/// `352_attacking_default`, `442_default`, …).
pub fn load_shipped_presets(dir: &std::path::Path)
    -> std::io::Result<std::collections::BTreeMap<String, Tactic>>
{
    let mut out = std::collections::BTreeMap::new();
    for ent in std::fs::read_dir(dir)? {
        let ent = ent?;
        let path = ent.path();
        let is_pct = path.extension().and_then(|s| s.to_str())
            .map(|s| s.eq_ignore_ascii_case("pct")).unwrap_or(false);
        if !is_pct { continue; }
        if let Ok(t) = load_tactic(&path) {
            let stem = path.file_stem().and_then(|s| s.to_str())
                .unwrap_or("").to_string();
            out.insert(stem, t);
        }
    }
    Ok(out)
}

/// Convenience: the tactic's 11 role masks as an array, in the shape
/// `tactics::position_rating` and the XI picker expect.
pub fn role_masks(t: &Tactic) -> [u16; 11] {
    let mut r = [0u16; 11];
    for i in 0..11 { r[i] = t.slots[i].role_mask; }
    r
}

// ---------------------------------------------------------------------------
// Per-slot positional-play waypoints (24 pitch coordinates per slot).
//
// The 96-byte block at struct `+0x164` (file `0x0139 + slot*96`) is NOT
// a bank of the named individual-instruction sliders — those live in
// `slot_pair.movement_token` (nibble-packed) and `slot_flag` (mutually-
// exclusive bit-set). This block holds 24 (x, y) pitch waypoints per
// slot which the tactics screen renders as movement arrows on the pitch.
//
// Decoded from setter `FUN_0059d870` (writes at `+0x164 + slot*96 + …`,
// with the value validated as `x ∈ 0..=26, y ∈ 0..=35` on line 73).
// Verified against `442_default.pct`: slot 0 (GK) all x ≈ 13 (goal-mouth
// centre), y ∈ 0..6 (six-yard box); slot 10 (ST) x ∈ 2..14, y ∈ 6..27
// (attacking third).
// ---------------------------------------------------------------------------

/// One (x, y) pitch coordinate — the game's internal 0..=26 × 0..=35 grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PitchXY {
    pub x: u16,
    pub y: u16,
}

/// Per-slot 24-waypoint positional-play grid.
///
/// Shape: `[side ∈ 0..2][row ∈ 0..3][col ∈ 0..4]` → one [`PitchXY`].
/// `side` = home / away mentality mirror.
/// `row` = lateral corridor (left / centre / right).
/// `col` = depth waypoint (front → back).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SlotInstructions {
    pub grid: [[[PitchXY; 4]; 3]; 2],
}

impl SlotInstructions {
    /// Read one anchor point.
    pub fn point(&self, side: usize, row: usize, col: usize) -> PitchXY {
        self.grid[side][row][col]
    }
    /// Iterate every (side, row, col, xy) tuple in canonical order.
    pub fn iter_points(&self) -> impl Iterator<Item = (usize, usize, usize, PitchXY)> + '_ {
        (0..2).flat_map(move |side| {
            (0..3).flat_map(move |row| {
                (0..4).map(move |col| (side, row, col, self.grid[side][row][col]))
            })
        })
    }
}

/// Decode one slot's 96-byte positional-play block from raw `.tct`/`.pct`
/// body bytes (offset 0 = version tag). Slot index must be `0..=10`.
pub fn slot_instructions_from_bytes(body: &[u8], slot: usize) -> SlotInstructions {
    assert!(slot < 11, "slot index must be 0..=10");
    let base = 0x0139 + slot * 96;
    let mut out = SlotInstructions::default();
    for side in 0..2 {
        for row in 0..3 {
            for col in 0..4 {
                let dword = (side * 3 + row) * 4 + col; // 0..24
                let off = base + dword * 4;
                let x = u16::from_le_bytes(body[off..off + 2].try_into().unwrap());
                let y = u16::from_le_bytes(body[off + 2..off + 4].try_into().unwrap());
                out.grid[side][row][col] = PitchXY { x, y };
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Nibble-packed individual-instruction sliders (in `slot_pair.movement_token`).
//
// The 8 named sliders per position (mentality-override, closing-down,
// marking-tightness, distribution, forward-runs, hold-up-ball, passing-focus,
// long-shots, plus flags for playmaker / target-man / free-role via
// `slot_flag`) live in the low 32 bits of `movement_token`, packed 4 bits
// per slider. Precise nibble→slider naming still TBD (agent flagged this as
// a follow-up decode) — accessor returns the 8 raw nibble values so the
// downstream code has SOMETHING to bind against once the mapping lands.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Team-wide settings (from tactic.team_flags_2).
//
// The team_flags_2 u32 at struct +0x584 / file 0x0559 is one-hot-packed
// across seven groups (bits 0..17). Setter is FUN_0059F3E0 — verified via
// grep as the only writer to +0x584. Marking group VERIFIED by loader
// force-set of Zonal (bit 0x2000) when the group's bits are all zero.
//
// Field-by-field: reports/team_flags_2_decode.md §5.
// ---------------------------------------------------------------------------

/// Team-wide passing style. Group A, bits 0..3.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Passing { Short, Mixed, Direct, Long, Unset }

/// Team-wide mentality. Group B, bits 4..6. Three states — NOT the five
/// (Ultra-Def..All-Out-Att) that an earlier port assumed; the tactics
/// screen has only three labels for the team-wide setting. The five-state
/// slider IS a real thing but lives on `slot_pair` per-position, not here.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mentality { Defensive, Normal, Attacking, Unset }

impl Mentality {
    /// Convert to the `team_tactic_word` u32 that
    /// [`crate::match_engine_exe::mentality_outcome_scaler`] reads.
    ///
    /// The exe uses bit `0x20` for Normal, bit `0x40` for Attacking, and
    /// zero (default = 2.0 = Defensive) otherwise. Unset defaults to the
    /// Defensive path so a club without a chosen tactic still gets a
    /// deterministic scaler value (matches the exe's fallback: no bit
    /// set → 2.0×).
    #[inline]
    pub fn to_scaler_word(self) -> u32 {
        match self {
            Mentality::Normal    => 0x20,
            Mentality::Attacking => 0x40,
            Mentality::Defensive => 0,
            Mentality::Unset     => 0,
        }
    }
}

/// Team-wide pressing intensity. Group E, bits 11..12.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Pressing { Normal, High, Unset }

/// Team-wide marking scheme. Group F, bits 13..14. VERIFIED via loader
/// force-set (Zonal is the force-default when group bits are all zero).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Marking { Zonal, ManToMan, Unset }

/// Team-wide tackling firmness. Group G, bits 15..17. Normal is loader-forced
/// as the default on legacy (v5C) files whose group bits are all zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tackling { Easy, Hard, Normal, Unset }

/// Decoded team-wide settings from `Tactic.team_flags_2`. See the bit table
/// in `reports/team_flags_2_decode.md`. `Marking` is VERIFIED; the rest are
/// INFERRED from group cardinality and shipped-preset value distribution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TeamSettings {
    #[serde(default = "default_passing")]
    pub passing: Passing,
    #[serde(default = "default_mentality")]
    pub mentality: Mentality,
    #[serde(default)]
    pub counter_attack: bool,
    #[serde(default)]
    pub offside_trap: bool,
    #[serde(default = "default_pressing")]
    pub pressing: Pressing,
    #[serde(default = "default_marking")]
    pub marking: Marking,
    #[serde(default = "default_tackling")]
    pub tackling: Tackling,
}

// Default constructors for TeamSettings serde defaults (also used to
// derive a Default impl that matches how tactic-less clubs are treated
// in the engine — everything Unset).
fn default_passing()   -> Passing   { Passing::Unset }
fn default_mentality() -> Mentality { Mentality::Unset }
fn default_pressing()  -> Pressing  { Pressing::Unset }
fn default_marking()   -> Marking   { Marking::Unset }
fn default_tackling()  -> Tackling  { Tackling::Unset }

impl Default for TeamSettings {
    fn default() -> Self {
        TeamSettings {
            passing: default_passing(),
            mentality: default_mentality(),
            counter_attack: false,
            offside_trap: false,
            pressing: default_pressing(),
            marking: default_marking(),
            tackling: default_tackling(),
        }
    }
}

/// Read the seven team switches out of a tactic's `team_flags_2` word.
pub fn team_settings(t: &Tactic) -> TeamSettings {
    let w = t.team_flags_2;
    TeamSettings {
        passing: match w & 0x0000_000F {
            0x1 => Passing::Short,
            0x2 => Passing::Mixed,
            0x4 => Passing::Direct,
            0x8 => Passing::Long,
            _   => Passing::Unset,
        },
        mentality: match w & 0x0000_0070 {
            0x10 => Mentality::Normal,
            0x20 => Mentality::Defensive,
            0x40 => Mentality::Attacking,
            _    => Mentality::Unset,
        },
        counter_attack: (w & 0x0000_0180) == 0x100,
        offside_trap:   (w & 0x0000_0600) == 0x400,
        pressing: match w & 0x0000_1800 {
            0x0800 => Pressing::Normal,
            0x1000 => Pressing::High,
            _      => Pressing::Unset,
        },
        marking: match w & 0x0000_6000 {
            0x2000 => Marking::Zonal,
            0x4000 => Marking::ManToMan,
            _      => Marking::Unset,
        },
        tackling: match w & 0x0003_8000 {
            0x08000 => Tackling::Easy,
            0x10000 => Tackling::Hard,
            0x20000 => Tackling::Normal,
            _       => Tackling::Unset,
        },
    }
}

/// Extract the 8 nibbles from a slot's `movement_token` (LSB first).
/// Names are placeholder ordinals until the setter cluster is decoded.
pub fn slot_slider_nibbles(t: &Tactic, slot: usize) -> [u8; 8] {
    assert!(slot < 11);
    let tok = t.slots[slot].movement_token;
    let mut out = [0u8; 8];
    for i in 0..8 { out[i] = ((tok >> (i * 4)) & 0xF) as u8; }
    out
}

/// The 8 per-slot slider names — VERIFIED via exe .rdata tactics-editor
/// label cluster at 0x006779ed (in order):
///
///   Marker | Marking | Cross Ball | Try Through Balls | Long Shots |
///   Hold Up Ball | Run With Ball | Forward Runs | Free Role |
///   Set Pieces (D) | Set Pieces (A) | Pass To
///
/// Not all 12 UI labels are 8-nibble sliders — some are boolean flags
/// (Free Role, Cross Ball, Try Through Balls, Run With Ball, Set Pieces)
/// that live on [`TacticSlot::flag`], not in the 8 nibbles.
///
/// The 8 nibbles most likely encode the sliders that have MULTI-STATE
/// values: mentality, closing-down, marking, distribution, forward-runs
/// (frequency), hold-up-ball, long-shots, pass-focus. This mirrors the
/// standard CM01/02 per-position instruction set.
///
/// **ORDERING DISCLAIMER**: The exact nibble→slider mapping requires
/// exe callsite evidence (e.g. `movement_token & 0xF` reads on a
/// specific runtime tactic pointer). No decoded setter fn is available;
/// this enum captures the SET of sliders but not their bit positions.
/// Callers that need to READ a specific slider by name must use
/// [`slot_slider_nibbles`] and interpret positions carefully — until
/// the setter cluster is decoded, treat the ordering as **unstable**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotSlider {
    /// Player's individual mentality override (Ultra-Defensive to All-Out
    /// Attack — the 5-state per-position slider mentioned at tactic_file
    /// line 343).
    Mentality,
    /// Closing-down intensity (Rarely / Sometimes / Often / Always).
    ClosingDown,
    /// Marking tightness (Loose / Normal / Tight).
    Marking,
    /// Distribution (Short / Mixed / Long).
    Distribution,
    /// Forward-runs frequency (Rarely / Mixed / Often).
    ForwardRuns,
    /// Hold-up ball behavior.
    HoldUpBall,
    /// Long shots preference.
    LongShots,
    /// Pass-to specific player slot.
    PassTo,
}

/// Boolean flags in [`TacticSlot::flag`] — VERIFIED from exe .rdata
/// tactics-editor labels at 0x006779ed. These are TRUE/FALSE toggles,
/// unlike the multi-state sliders in [`SlotSlider`].
///
/// The bit assignments within `slot.flag` are envelope — no decoded
/// setter fn found. Callers reading a specific flag should mask
/// carefully.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotFlag {
    Playmaker,
    TargetMan,
    FreeRole,
    CrossBall,
    TryThroughBalls,
    RunWithBall,
    SetPiecesDef,
    SetPiecesAtt,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_442_carries_empty_slot_instructions() {
        let t = Tactic::flat_442();
        // 11 slots, each with default (0,0) waypoints — the default
        // constructor doesn't seed positional data.
        assert_eq!(t.slot_instructions.len(), 11);
        for slot in 0..11 {
            for side in 0..2 {
                for row in 0..3 {
                    for col in 0..4 {
                        let p = t.slot_instructions[slot].point(side, row, col);
                        assert_eq!(p.x, 0);
                        assert_eq!(p.y, 0);
                    }
                }
            }
        }
    }

    #[test]
    fn slot_slider_and_flag_enums_cover_verified_ui_labels() {
        // Not a behavioral test — just confirms the two enums exist and
        // cover the 8+8 labels found at exe .rdata 0x006779ed.
        let sliders = [
            SlotSlider::Mentality, SlotSlider::ClosingDown,
            SlotSlider::Marking, SlotSlider::Distribution,
            SlotSlider::ForwardRuns, SlotSlider::HoldUpBall,
            SlotSlider::LongShots, SlotSlider::PassTo,
        ];
        assert_eq!(sliders.len(), 8);
        let flags = [
            SlotFlag::Playmaker, SlotFlag::TargetMan, SlotFlag::FreeRole,
            SlotFlag::CrossBall, SlotFlag::TryThroughBalls,
            SlotFlag::RunWithBall, SlotFlag::SetPiecesDef, SlotFlag::SetPiecesAtt,
        ];
        assert_eq!(flags.len(), 8);
    }

    #[test]
    fn version_tag_bounds() {
        // Two blank slabs of the right size but with a totally wrong
        // version tag → None.
        let bad = vec![0u8; 1476];
        assert!(parse_tactic(&bad, false).is_none());
    }
}
