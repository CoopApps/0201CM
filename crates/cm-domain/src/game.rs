//! Game object markers — a small, honest port of `game.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\game.cpp`, VA `0x005b6f10..0x005c153f`,
//! 3 attributed functions of ~42KB). Function decode + names:
//! `reports/carve_rename_map.json` (`game.cpp`).
//!
//! The exe's `game.cpp` is the **central game object / dispatcher**. Its init
//! (`0x005b6f10`) sets a global "live game" marker `DAT_00dbbcf2` to the
//! game-object vtable address `0x005c0460`, then hands off to the game-tick
//! driver at `0x00672770`. The main dispatcher (`0x005b85b0`) is 30467 bytes
//! (9271 instrs) — a huge per-turn state machine that our Rust
//! `RuntimeSaveGame::tick_cm_phase` is the functional equivalent of.
//!
//! Rather than replicate the giant state machine, this module records the
//! decoded constants: the vtable address (a save-format marker) and the
//! tick-driver address (already ported as `tick_cm_phase`).

/// Address of the exe's game-object vtable (written to `DAT_00dbbcf2` on init).
/// A live-game marker in the exe; ports as a documented constant.
pub const GAME_OBJECT_VTABLE_ADDR: u32 = 0x005c_0460;

/// Address of the game-tick driver (`FUN_00672770`) called from `game_init`.
/// The Rust port is [`crate::RuntimeSaveGame::tick_cm_phase`] (which is the
/// port of `FUN_005b6a90`, the deeper tick driver — see memory `game-tick`).
pub const GAME_TICK_DRIVER_ADDR: u32 = 0x0067_2770;

/// Address of the "live game" marker global (`DAT_00dbbcf2`) — the exe writes
/// the vtable to this address on init to mark that a game is running.
pub const LIVE_GAME_MARKER_ADDR: u32 = 0x00db_bcf2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_have_expected_values() {
        // These are the exe addresses; they cross-reference the ledger.
        assert_eq!(GAME_OBJECT_VTABLE_ADDR, 0x005c_0460);
        assert_eq!(GAME_TICK_DRIVER_ADDR, 0x0067_2770);
        assert_eq!(LIVE_GAME_MARKER_ADDR, 0x00db_bcf2);
    }
}
