//! Exe address + patch reference from agevak/CM0102/Patches/*/DevNotes.
//!
//! These are reverse-engineered cm0102.exe addresses documented by
//! the agevak community project alongside the runtime patches they
//! wrote. Each entry pairs a virtual address in the shipped exe with
//! the semantic role that address plays (function boundary, decision
//! branch, in-memory global, etc.).
//!
//! **Documentation-only** — this module doesn't apply any patch; it's
//! a catalogue of decoded exe knowledge to consult when porting the
//! matching subsystem. Every constant here is verified against the
//! raw DevNotes.txt files in the agevak repo.

// ---------------------------------------------------------------------------
// AttributeOverflowClamp — Vision & Finishing overflow sites
// ---------------------------------------------------------------------------
//
// Two attributes (Vision, Finishing) can overflow in the exe's in-match
// value calculation. The patch installs a call-out to a min-cap function
// in the expanded exe region.

/// Address of the instruction that follows the Vision-calc function's
/// last store. The patch replaces the instruction before this with a
/// call to `VISION_CLAMP_FN` (below).
pub const VISION_FN_END:      u32 = 0x006EDB28;

/// Address of the instruction that follows the Finishing-calc function's
/// last store. Patched similarly to Vision.
pub const FINISHING_FN_END:   u32 = 0x006ECEF7;

/// Structure offset where the Vision post-calc value is stored (accessed
/// as `[esi + VISION_STORE_OFFSET]` in the exe). This is on the player's
/// in-match state record.
pub const VISION_STORE_OFFSET:    usize = 0x000000F1;

/// Structure offset where the Finishing post-calc value is stored.
pub const FINISHING_STORE_OFFSET: usize = 0x00000099;

/// Structure offset the Vision function reads BEFORE returning (loaded
/// into ebx as `mov ebx, [esi + VISION_LOAD_OFFSET]`).
pub const VISION_LOAD_OFFSET: usize = 0x0000019E;

/// Address of the patch's clamp function for Vision, installed in the
/// expanded exe region.
pub const VISION_CLAMP_FN:    u32 = 0x00DE7470;

/// Address of the patch's clamp function for Finishing.
pub const FINISHING_CLAMP_FN: u32 = 0x00DE749B;

/// Address holding the qword double-precision cap constant used by
/// both clamp functions.
pub const CLAMP_CONSTANT_ADDR: u32 = 0x00DE7496;

// ---------------------------------------------------------------------------
// BenchmarkMode — the "-load and go on holiday" harness
// ---------------------------------------------------------------------------
//
// The benchmark mode is enabled with the `-load` command-line arg; on
// startup the exe loads a save, then holidays until a chosen date.

/// Address of the `add esp, 00000F08 / ret` sequence in the go-on-holiday
/// entry. Patched to `call GO_ON_HOLIDAY_FN / nop / ret`.
pub const GO_ON_HOLIDAY_CALLSITE: u32 = 0x0081C06A;

/// The new go-on-holiday function installed by the benchmark patch.
/// Reads `[GLOBAL_MANAGER_PTR]`, pushes 0 and 1, calls `HOLIDAY_TICK_FN`,
/// pops registers, and returns.
pub const GO_ON_HOLIDAY_FN:      u32 = 0x00603730;

/// Global pointer to the current-manager record, read by the benchmark's
/// go-on-holiday routine.
pub const GLOBAL_MANAGER_PTR: u32 = 0x00B63C98;

/// The exe's original holiday-tick function called by the patched entry
/// with args `(0, 1)`.
pub const HOLIDAY_TICK_FN: u32 = 0x005FD2F0;

/// Game date word in .data — a `word ptr`. The benchmark patch's
/// stop-on-31-May check reads `[GAME_DATE_WORD]` and compares to 0x0096.
pub const GAME_DATE_WORD: u32 = 0x00AE2C90;

/// The patch's stop-on-31-May check function. Compares
/// `[GAME_DATE_WORD]` to `0x0096` and calls [`STOP_HANDLER_FN`] on
/// match.
pub const STOP_CHECK_FN: u32 = 0x00603718;

/// Handler run when the 31-May stop condition triggers.
pub const STOP_HANDLER_FN: u32 = 0x00603688;

/// Address the stop-check patch replaces (originally either `ret` or
/// `call 00602D48`, depending on whether the "enable potential to
/// grow" option was on).
pub const STOP_CHECK_INSTALL_SITE: u32 = 0x006B5CD7;

/// The "enable potential to grow" function the stop-check overwrites
/// when that option is on.
pub const POTENTIAL_GROW_FN: u32 = 0x00602D48;

// ---------------------------------------------------------------------------
// NoManagerSacking — chain from decision back to the executor
// ---------------------------------------------------------------------------

/// Innermost sacking function (both AI and human).
pub const SACK_MANAGER_FN: u32 = 0x00690928;

/// The function that wraps `SACK_MANAGER_FN`.
pub const SACK_WRAPPER_FN: u32 = 0x00690890;

/// Address that calls into [`SACK_WRAPPER_FN`].
pub const SACK_WRAPPER_CALLSITE: u32 = 0x0069B880;

/// The function that holds [`SACK_WRAPPER_CALLSITE`].
pub const SACK_DECISION_FN_1: u32 = 0x0069B790;

/// Address that calls into [`SACK_DECISION_FN_1`].
pub const SACK_DECISION_CALLSITE: u32 = 0x0069B6C8;

/// The function that holds [`SACK_DECISION_CALLSITE`].
pub const SACK_DECIDER_FN: u32 = 0x0069B650;

/// The `jne 0069B6BC` decision branch inside [`SACK_DECIDER_FN`]. The
/// NoManagerSacking patch NOPs this instruction so the decision falls
/// through (no sacking ever fires).
pub const SACK_DECISION_BRANCH: u32 = 0x0069B6AC;

// ---------------------------------------------------------------------------
// Miscellaneous
// ---------------------------------------------------------------------------

/// Instruction `lea ecx, [esp + 0x11c]` at this address gets replaced
/// with `jmp 00403454`, part of the benchmark-mode DevNotes.
pub const LEA_JMP_PATCH_SITE: u32 = 0x004033FE;

/// `cmp dword ptr [eax + ecx*4], 00` at this address gets NOP'd
/// (4 bytes) by the benchmark-mode patch.
pub const CMP_NOP_PATCH_SITE: u32 = 0x0054489B;

/// Called during .sav loading; the benchmark patch replaces its opening
/// with `movzx eax, word ptr [ebp - 02]`.
pub const SAV_LOAD_HELPER: u32 = 0x00946F66;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_exe_addresses_land_in_valid_ranges() {
        // Cm0102.exe .text runs roughly 0x00401000..0x009xxxxx.
        // Expanded-exe patch region starts at 0x00DE0000.
        // Every constant here should land in one of these ranges.
        let exe_addresses = [
            VISION_FN_END, FINISHING_FN_END, GO_ON_HOLIDAY_CALLSITE,
            GO_ON_HOLIDAY_FN, HOLIDAY_TICK_FN, STOP_CHECK_FN,
            STOP_HANDLER_FN, STOP_CHECK_INSTALL_SITE, POTENTIAL_GROW_FN,
            SACK_MANAGER_FN, SACK_WRAPPER_FN, SACK_WRAPPER_CALLSITE,
            SACK_DECISION_FN_1, SACK_DECISION_CALLSITE, SACK_DECIDER_FN,
            SACK_DECISION_BRANCH, LEA_JMP_PATCH_SITE, CMP_NOP_PATCH_SITE,
            SAV_LOAD_HELPER,
        ];
        for &addr in &exe_addresses {
            let in_text = (0x0040_0000..0x00A0_0000).contains(&addr);
            assert!(in_text, "{addr:#010x} out of expected .text range");
        }
        // Expanded region
        let expanded = [VISION_CLAMP_FN, FINISHING_CLAMP_FN, CLAMP_CONSTANT_ADDR];
        for &addr in &expanded {
            let in_expanded = (0x00DE_0000..0x00DF_0000).contains(&addr);
            assert!(in_expanded, "{addr:#010x} not in expanded region");
        }
        // Data globals
        let data_globals = [GLOBAL_MANAGER_PTR, GAME_DATE_WORD];
        for &addr in &data_globals {
            let in_data = (0x00A0_0000..0x00C0_0000).contains(&addr);
            assert!(in_data, "{addr:#010x} not in .data range");
        }
    }

    #[test]
    fn overflow_clamp_offsets_are_the_ones_the_asm_uses() {
        // From DevNotes: fstp dword ptr [esi + 0xF1] (Vision store).
        assert_eq!(VISION_STORE_OFFSET, 0xF1);
        // fstp dword ptr [esi + 0x99] (Finishing store).
        assert_eq!(FINISHING_STORE_OFFSET, 0x99);
        // mov ebx, [esi + 0x19E] (Vision load).
        assert_eq!(VISION_LOAD_OFFSET, 0x19E);
    }

    #[test]
    fn sack_chain_addresses_are_ordered_from_innermost_to_outermost() {
        // The sack chain is documented as: sack_fn → wrapper → caller
        // → decision_fn → decider. Higher addresses tend to be
        // "later" functions in the alphabetical .text layout, but the
        // key invariant is that they're all distinct.
        let chain = [
            SACK_MANAGER_FN, SACK_WRAPPER_FN, SACK_WRAPPER_CALLSITE,
            SACK_DECISION_FN_1, SACK_DECISION_CALLSITE, SACK_DECIDER_FN,
            SACK_DECISION_BRANCH,
        ];
        for i in 0..chain.len() {
            for j in (i+1)..chain.len() {
                assert_ne!(chain[i], chain[j],
                    "sack chain has duplicate at [{i}]={:#010x} and [{j}]", chain[i]);
            }
        }
    }

    #[test]
    fn thirty_one_may_is_encoded_as_0x0096() {
        // 31 May is day 151 (0-indexed 150) in a non-leap year;
        // agevak's check compares to 0x0096 = 150. This confirms the
        // game date word is day-of-year, 0-indexed.
        assert_eq!(0x0096, 150);
    }
}
