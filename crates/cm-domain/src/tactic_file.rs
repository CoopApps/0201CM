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
    /// 1..5 — Ultra-Defensive / Defensive / Normal / Attacking / All-Out-Attack.
    pub mentality: u8,
    /// Team-wide flag word #2 — mentality (bits 0..4) plus the other 10
    /// on/off team switches. Bit-level mapping TBD (see report §5); the
    /// XI picker doesn't need it yet, but downstream match-tick code will.
    pub team_flags_2: u32,
    /// Team-wide flag word #1 — preset provenance / reserve-usable flags.
    pub team_flags_1: u32,
    /// 11 per-position slot rows, in role order (GK first, then defenders,
    /// midfielders, attackers).
    pub slots: [TacticSlot; 11],
    /// File version tag (canonical form, obfuscation removed).
    pub version: u32,
    /// True if loaded from a `.pct` (packaged preset), false for `.tct`.
    pub is_packaged: bool,
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
    let mentality = (team_flags_2 & 0x1F) as u8;

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

    Some(Tactic {
        formation_name, author, mentality,
        team_flags_1, team_flags_2, slots,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_tag_bounds() {
        // Two blank slabs of the right size but with a totally wrong
        // version tag → None.
        let bad = vec![0u8; 1476];
        assert!(parse_tactic(&bad, false).is_none());
    }
}
