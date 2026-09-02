//! Utility functions translated from agevak/CM0102/CM0102Core/Model/
//! `ContractExtensions.cs` and `PlayerExtensions.cs`.
//!
//! **The bit-decode formulas** for transfer status and squad status
//! are the primary value here — they turn the raw byte at
//! `TContract +0x4E` / `+0x4F` into the enum labels the UI shows.

use crate::agevak_enums::{AgevakSquadStatus, AgevakTransferStatus};

// ---------------------------------------------------------------------------
// TransferStatus bit decode  (TContract +0x4E)
// ---------------------------------------------------------------------------

/// Decode the transfer-status byte per agevak's `ContractExtensions.GetTransferStatus`.
///
/// Bit map (LSB tested first — the first matching bit wins):
/// - `& 1`  → ListedByClub
/// - `& 8`  → ListedByRequest
/// - `& 2`  → ListedForLoan
/// - `== 4` (implicit — no other bit set) → Unknown
/// - anything else → Unknown
pub fn decode_transfer_status(status: u8) -> AgevakTransferStatus {
    if (status & 1) != 0 { AgevakTransferStatus::ListedByClub }
    else if (status & 8) != 0 { AgevakTransferStatus::ListedByRequest }
    else if (status & 2) != 0 { AgevakTransferStatus::ListedForLoan }
    else { AgevakTransferStatus::Unknown }
}

/// UI-label variant of the transfer-status decode.
pub fn transfer_status_ui(status: u8) -> String {
    if (status & 1) != 0 { "Listed by club".to_string() }
    else if (status & 8) != 0 { "Listed by request".to_string() }
    else if (status & 2) != 0 { "List for loan".to_string() }
    else if status == 4 { "Unknown".to_string() }
    else { format!("{status}UnkownTransferStatus") }
}

// ---------------------------------------------------------------------------
// SquadStatus bit decode  (TContract +0x4F)
// ---------------------------------------------------------------------------

/// Decode the squad-status byte per agevak's `ContractExtensions.GetSquadStatus`.
///
/// The scheme is a **progressively-narrowing high-bits mask**:
/// - `(& 240) == 0` (bits 4..7 all clear) → Uncertain
/// - `(& 224) == 0`                        → Indispensable
/// - `(& 208) == 0`                        → FirstTeam
/// - `(& 192) == 0`                        → SquadRotation
/// - `(& 176) == 0`                        → Backup
/// - `(& 160) == 0`                        → HotProspect
/// - `(& 144) == 0`                        → DecentYoung
/// - `(& 128) == 0`                        → NotNeeded
/// - `(& 112) == 0`                        → OnTrial
/// - else                                  → Uncertain (fallback)
///
/// Reading the masks in binary shows the scheme is single-bit-per-status
/// (240=11110000, 224=11100000, 208=11010000, ...), so the byte's
/// upper-nibble bits are a state encoding.
pub fn decode_squad_status(status: u8) -> AgevakSquadStatus {
    if (status & 240) == 0 { AgevakSquadStatus::Uncertain }
    else if (status & 224) == 0 { AgevakSquadStatus::Indispensable }
    else if (status & 208) == 0 { AgevakSquadStatus::FirstTeam }
    else if (status & 192) == 0 { AgevakSquadStatus::SquadRotation }
    else if (status & 176) == 0 { AgevakSquadStatus::Backup }
    else if (status & 160) == 0 { AgevakSquadStatus::HotProspect }
    else if (status & 144) == 0 { AgevakSquadStatus::DecentYoung }
    else if (status & 128) == 0 { AgevakSquadStatus::NotNeeded }
    else if (status & 112) == 0 { AgevakSquadStatus::OnTrial }
    else                        { AgevakSquadStatus::Uncertain }
}

/// UI-label variant of the squad-status decode.
pub fn squad_status_ui(status: u8) -> String {
    if (status & 240) == 0 { "Uncertain".into() }
    else if (status & 224) == 0 { "Indispensable".into() }
    else if (status & 208) == 0 { "First team".into() }
    else if (status & 192) == 0 { "Squad rotation".into() }
    else if (status & 176) == 0 { "Backup".into() }
    else if (status & 160) == 0 { "Hot prospect".into() }
    else if (status & 144) == 0 { "Decent young".into() }
    else if (status & 128) == 0 { "Not needed".into() }
    else if (status & 112) == 0 { "On trial".into() }
    else                        { format!("{status}UnkownSquadStatus") }
}

// ---------------------------------------------------------------------------
// Position UI string  (PlayerExtensions.GetPositionUIString)
// ---------------------------------------------------------------------------

/// Format a player's positional aptitudes as the classic "GK/SW/D/DM
/// R" UI string (agevak's exact format). Threshold for each label is
/// aptitude ≥ 15.
///
/// The oddity: if the player has no natural outfield position but
/// wing-back ≥ 15, the label is `"D/AM"` (a stand-in D-to-AM
/// versatility marker).
pub fn position_ui_string(
    goalkeeper: i8, sweeper: i8, defender: i8,
    defensive_mid: i8, midfielder: i8, attacking_mid: i8,
    attacker: i8, wing_back: i8,
    right_side: i8, left_side: i8, central: i8,
) -> String {
    let mut pos = String::new();
    if goalkeeper >= 15    { pos.push_str("/GK"); }
    if sweeper >= 15       { pos.push_str("/SW"); }
    if defender >= 15      { pos.push_str("/D"); }
    if defensive_mid >= 15 { pos.push_str("/DM"); }
    if midfielder >= 15    { pos.push_str("/M"); }
    if attacking_mid >= 15 { pos.push_str("/AM"); }
    if attacker >= 15      { pos.push_str("/F"); }
    if pos.is_empty() && wing_back >= 15 { pos.push_str("D/AM"); }

    let mut side = String::new();
    if right_side >= 15 { side.push('R'); }
    if left_side  >= 15 { side.push('L'); }
    if central    >= 15 { side.push('C'); }

    // Trim leading '/' from pos, matching agevak's Trim(' ', '/')
    let pos = pos.trim_matches(|c: char| c == ' ' || c == '/');
    let side = side.trim_matches(|c: char| c == ' ' || c == '/');
    format!("{pos} {side}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transfer_status_bit_priority() {
        // Bit 0 wins over bit 3 (both set → ListedByClub)
        assert_eq!(decode_transfer_status(0b1001), AgevakTransferStatus::ListedByClub);
        // Bit 3 wins over bit 1 (0b1010 → bit 3 = 8 → ListedByRequest)
        assert_eq!(decode_transfer_status(0b1010), AgevakTransferStatus::ListedByRequest);
        // Only bit 1 → ListedForLoan
        assert_eq!(decode_transfer_status(0b0010), AgevakTransferStatus::ListedForLoan);
        // Only bit 2 → Unknown
        assert_eq!(decode_transfer_status(0b0100), AgevakTransferStatus::Unknown);
        // Zero → Unknown
        assert_eq!(decode_transfer_status(0), AgevakTransferStatus::Unknown);
    }

    #[test]
    fn squad_status_zero_is_uncertain() {
        assert_eq!(decode_squad_status(0), AgevakSquadStatus::Uncertain);
    }

    #[test]
    fn squad_status_progressive_scan() {
        // 0b0001_0000 = 16: (& 240)=16 → not Uncertain; (& 224)=0 → Indispensable
        assert_eq!(decode_squad_status(16), AgevakSquadStatus::Indispensable);
        // 0b0010_0000 = 32: (& 240)=32; (& 224)=32; (& 208)=0 → FirstTeam
        assert_eq!(decode_squad_status(32), AgevakSquadStatus::FirstTeam);
        // 0b0011_0000 = 48: (& 240)=48; (& 224)=32; (& 208)=16; (& 192)=0 → SquadRotation
        assert_eq!(decode_squad_status(48), AgevakSquadStatus::SquadRotation);
        // 0b1000_0000 = 128: all narrower masks fail → OnTrial (& 112=0)
        assert_eq!(decode_squad_status(128), AgevakSquadStatus::OnTrial);
        // 0xFF: every mask retains bits → Uncertain fallback
        assert_eq!(decode_squad_status(0xFF), AgevakSquadStatus::Uncertain);
    }

    #[test]
    fn position_ui_string_striker_right() {
        // Right-footed striker
        let s = position_ui_string(0, 0, 0, 0, 0, 0, 18, 0, 17, 0, 0);
        assert_eq!(s, "F R");
    }

    #[test]
    fn position_ui_string_versatile_defender_lc() {
        // Defender + defensive-mid, plays left + central
        let s = position_ui_string(0, 0, 16, 15, 0, 0, 0, 0, 0, 17, 18);
        assert_eq!(s, "D/DM LC");
    }

    #[test]
    fn position_ui_string_pure_wingback_falls_to_d_am() {
        // No natural position but wing-back ≥ 15 → "D/AM"
        let s = position_ui_string(0, 0, 0, 0, 0, 0, 0, 18, 15, 0, 0);
        assert_eq!(s, "D/AM R");
    }

    #[test]
    fn position_ui_string_gk_only() {
        let s = position_ui_string(20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
        assert_eq!(s, "GK ");
    }
}
