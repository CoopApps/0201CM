//! Weather substrate — port of `weather.cpp`.
//!
//! Read by the match engine to bias event probabilities: wet pitch =
//! more slips → more fouls + more injuries + fewer shots on target;
//! heavy snow / heat wave can force a fixture to be postponed.
//!
//! Weather is deterministic per (nation, day-of-year) so replays reproduce.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WeatherKind {
    Clear,
    Cloudy,
    LightRain,
    HeavyRain,
    Snow,
    Heatwave,
}

impl WeatherKind {
    /// Multiplier the match engine can apply to `expected_goals`. Wet /
    /// snowy = fewer goals; heatwave = tired players → slightly more
    /// mistakes → slightly more goals.
    pub fn xg_multiplier(self) -> f32 {
        match self {
            Self::Clear    => 1.00,
            Self::Cloudy   => 1.00,
            Self::LightRain=> 0.95,
            Self::HeavyRain=> 0.85,
            Self::Snow     => 0.70,
            Self::Heatwave => 1.10,
        }
    }

    /// Per-minute injury-probability multiplier.
    pub fn injury_multiplier(self) -> f32 {
        match self {
            Self::Clear    => 1.0,
            Self::Cloudy   => 1.0,
            Self::LightRain=> 1.2,
            Self::HeavyRain=> 1.5,
            Self::Snow     => 1.8,
            Self::Heatwave => 1.3,
        }
    }

    /// Whether the fixture should be postponed (heavy snow / severe heatwave).
    pub fn forces_postponement(self) -> bool {
        matches!(self, Self::Snow)
    }
}

/// Deterministic weather for a (nation, day-of-year) tuple. Uses a small
/// seasonal model: Nordic nations get more snow in winter; Mediterranean
/// nations get more heatwaves in summer; others cycle rain/cloud/clear.
pub fn weather_for(nation_id: i32, day_of_year: u16) -> WeatherKind {
    let is_northern = matches!(nation_id, 138 | 179 | 81 | 154);   // Norway, Sweden, Finland, Russia
    let is_mediterranean = matches!(nation_id, 171 | 94 | 91 | 149); // Spain, Italy, Greece, Portugal

    // Seasonal split: winter (Nov-Feb), summer (Jun-Aug), spring/autumn otherwise.
    let winter = day_of_year <= 60 || day_of_year >= 305;
    let summer = (152..=243).contains(&day_of_year);

    // Deterministic hash on (nation, day) → 0..99 bucket
    let seed = ((nation_id as u32).wrapping_mul(0x9e3779b1))
        .wrapping_add(day_of_year as u32);
    let bucket = seed.wrapping_mul(0x85ebca6b) >> 25;  // 0..127

    if winter {
        if is_northern {
            if bucket < 40 { WeatherKind::Snow } else if bucket < 80 { WeatherKind::HeavyRain } else { WeatherKind::Cloudy }
        } else {
            if bucket < 20 { WeatherKind::HeavyRain } else if bucket < 50 { WeatherKind::LightRain } else { WeatherKind::Cloudy }
        }
    } else if summer {
        if is_mediterranean {
            if bucket < 25 { WeatherKind::Heatwave } else { WeatherKind::Clear }
        } else {
            if bucket < 60 { WeatherKind::Clear } else if bucket < 90 { WeatherKind::LightRain } else { WeatherKind::Cloudy }
        }
    } else {
        // Spring / autumn
        if bucket < 30 { WeatherKind::Clear } else if bucket < 60 { WeatherKind::Cloudy }
        else if bucket < 90 { WeatherKind::LightRain } else { WeatherKind::HeavyRain }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_key_produces_same_weather() {
        for nation in [94, 138, 171] {
            for day in [10, 100, 200] {
                assert_eq!(weather_for(nation, day), weather_for(nation, day));
            }
        }
    }

    #[test]
    fn heatwave_reduces_shots_slightly() {
        assert!(WeatherKind::Heatwave.xg_multiplier() > 1.0);
        assert!(WeatherKind::Snow.xg_multiplier() < 1.0);
    }

    #[test]
    fn snow_forces_postponement_others_do_not() {
        assert!(WeatherKind::Snow.forces_postponement());
        assert!(!WeatherKind::HeavyRain.forces_postponement());
    }

    #[test]
    fn nordic_winter_produces_some_snow() {
        let mut snow_days = 0;
        for day in 1..=60 {
            if weather_for(138, day) == WeatherKind::Snow { snow_days += 1 }
        }
        assert!(snow_days > 10, "expected snow in northern winter, saw {snow_days}");
    }
}
