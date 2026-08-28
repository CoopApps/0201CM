//! Honours / awards — the port of the `award/` subsystem, starting with
//! `argentina_awards.cpp` (`argawards_create_champion_honours`, `0x0040a440`).
//!
//! When a competition is decided, the exe registers a **championship honour**
//! for the winning club via `honour_award_create` (`FUN_0089bc20`), tagged with
//! an honour-type id (e.g. the Argentine Primera champion is honour `0x7d0`,
//! the Second Division champion `0x3e8`). This module holds the honour record
//! the ported competition classes emit and the save stores.

use serde::{Deserialize, Serialize};

/// Argentine Primera champion honour type (the exe's `0x7d0`).
pub const ARG_PRIMERA_CHAMPION_HONOUR: u32 = 0x7d0;
/// Argentine Second Division champion honour type (the exe's `0x3e8`).
pub const ARG_SECOND_CHAMPION_HONOUR: u32 = 0x3e8;

/// A single honour awarded to a club — one row of a club's honours list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Honour {
    /// Season year the honour was won.
    pub year: u16,
    /// Honour-type id (the exe's honour catalogue id, e.g. `0x7d0`).
    pub honour_id: u32,
    /// Human-readable competition/title (e.g. "Argentine Apertura").
    pub competition: String,
    pub club_id: u32,
    pub club_name: String,
    /// Provenance: the exe function this honour was ported from.
    pub source: String,
}

impl Honour {
    /// A championship honour for `club`, as `argawards_create_champion_honours`
    /// (`0x0040a440`) registers it.
    pub fn champion(
        year: u16,
        honour_id: u32,
        competition: impl Into<String>,
        club_id: u32,
        club_name: impl Into<String>,
    ) -> Self {
        Self {
            year,
            honour_id,
            competition: competition.into(),
            club_id,
            club_name: club_name.into(),
            source: "argentina_awards.cpp argawards_create_champion_honours 0x0040a440 / honour_award_create 0x0089bc20".to_string(),
        }
    }
}
