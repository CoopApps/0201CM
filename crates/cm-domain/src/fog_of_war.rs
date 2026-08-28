//! Fog of war — port of `fog_of_war.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\fog_of_war.cpp`, VA `0x00599d60..0x0059cc3f`,
//! 4 attributed functions). Function decode + names: `reports/carve_rename_map.json`
//! (`fog_of_war.cpp`).
//!
//! The exe uses "fog of war" for two things: (a) scout knowledge — what a
//! manager knows about hidden player attributes, and (b) a **save-integrity
//! guard** — the first function loads a global (`DAT_00dbc3e8`), sign-extends
//! it via `cdq/xor/sub` (the classic branchless absolute-value trick), and
//! compares it against a magic literal `0x3e036`.
//!
//! Only the magic guard is self-contained and portable now (the scout system
//! isn't built). This module ports the guard as a save-integrity check the
//! save loader can call — a real, tested piece of the fog_of_war subsystem
//! even while the scouting side stays deferred.

use serde::{Deserialize, Serialize};

/// The exe's fog-of-war integrity magic (`fogwar_validate_magic` at
/// `0x00599d60` compares `|value|` against this literal).
pub const FOG_OF_WAR_MAGIC: i32 = 0x3e036;

/// A stored fog-of-war integrity value. The exe holds this in
/// `DAT_00dbc3e8`; here it travels with the save.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct FogOfWarIntegrity {
    /// The raw stored value (may be positive or negative in the exe).
    pub value: i32,
}

impl Default for FogOfWarIntegrity {
    fn default() -> Self {
        Self { value: FOG_OF_WAR_MAGIC }
    }
}

impl FogOfWarIntegrity {
    /// Port of the ctor's branchless absolute-value check: given `value`,
    /// compute `eax=value; cdq; xor eax,edx; sub eax,edx; cmp eax, 0x3e036`.
    /// Returns whether the stored value passes the magic guard.
    pub fn is_valid(self) -> bool {
        self.value.unsigned_abs() as i32 == FOG_OF_WAR_MAGIC
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_magic() {
        assert!(FogOfWarIntegrity::default().is_valid());
    }

    #[test]
    fn positive_magic_is_valid() {
        assert!(FogOfWarIntegrity { value: FOG_OF_WAR_MAGIC }.is_valid());
    }

    #[test]
    fn negative_magic_is_valid_via_absolute_value() {
        // The exe's cdq/xor/sub is a branchless abs; -0x3e036 also validates.
        assert!(FogOfWarIntegrity { value: -FOG_OF_WAR_MAGIC }.is_valid());
    }

    #[test]
    fn wrong_value_is_invalid() {
        assert!(!FogOfWarIntegrity { value: 0 }.is_valid());
        assert!(!FogOfWarIntegrity { value: FOG_OF_WAR_MAGIC + 1 }.is_valid());
        assert!(!FogOfWarIntegrity { value: -(FOG_OF_WAR_MAGIC - 1) }.is_valid());
    }
}
