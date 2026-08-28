//! Friendlies — partial port of `friendly.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\comp\friendly.cpp`, VA `0x005ac250..0x005b6f0f`,
//! 36 attributed functions). Function decode + names:
//! `reports/carve_rename_map.json` (`friendly.cpp`).
//!
//! The exe's friendlies subsystem arranges pre-season / international-break
//! matches, plays them and records the results — a competition-shaped class
//! with a friendly-specific base ctor (`0x00490f00`, not the club/nation cup
//! bases) and vtable `0x00958e2c`.
//!
//! **Ported piece**: the `FriendlyMatch` value type + a save-side list — a real
//! home for scheduled friendlies to live in.
//!
//! **Deferred**: the arranging AI (~35 helpers) and its integration with the
//! tick's international-break / off-season calendar slots. The Rust tick model
//! has no calendar slot for friendlies yet, so the list stays empty until one
//! exists.

use serde::{Deserialize, Serialize};

use crate::GameDate;

/// One friendly match — the minimal shape a scheduled friendly needs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FriendlyMatch {
    pub date: GameDate,
    pub home_club_id: u32,
    pub home_club_name: String,
    pub away_club_id: u32,
    pub away_club_name: String,
    /// A note explaining why the friendly was arranged (pre-season tour,
    /// international break testimonial, etc.).
    pub occasion: String,
    /// Result once played, `None` until then.
    pub home_score: Option<u8>,
    pub away_score: Option<u8>,
    pub source: String,
}

impl FriendlyMatch {
    /// A newly-arranged friendly (unplayed).
    pub fn arrange(
        date: GameDate,
        home: (u32, String),
        away: (u32, String),
        occasion: impl Into<String>,
    ) -> Self {
        Self {
            date,
            home_club_id: home.0,
            home_club_name: home.1,
            away_club_id: away.0,
            away_club_name: away.1,
            occasion: occasion.into(),
            home_score: None,
            away_score: None,
            source: "friendly.cpp friendly_ctor 0x005ac250 (arrange path)".to_string(),
        }
    }

    pub fn is_played(&self) -> bool {
        self.home_score.is_some() && self.away_score.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arranged_friendly_is_unplayed() {
        let f = FriendlyMatch::arrange(
            GameDate { year: 2001, month: 7, day: 20 },
            (1, "Arsenal".into()),
            (2, "PSG".into()),
            "Pre-season friendly",
        );
        assert!(!f.is_played());
        assert_eq!(f.home_club_name, "Arsenal");
        assert_eq!(f.away_club_name, "PSG");
    }

    #[test]
    fn recorded_result_marks_it_played() {
        let mut f = FriendlyMatch::arrange(
            GameDate { year: 2001, month: 7, day: 20 },
            (1, "Arsenal".into()),
            (2, "PSG".into()),
            "Pre-season friendly",
        );
        f.home_score = Some(2);
        f.away_score = Some(1);
        assert!(f.is_played());
    }
}
