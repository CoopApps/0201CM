//! CM0102 match event / commentary config (`Data/events_*.cfg`).
//!
//! Provenance (carve: D:/cm0102-carve/MATCH_ENGINE.md):
//! The match engine rolls action codes `8000 + EventID`; `EventID` (0..676) is
//! defined here. Format (from the file's own header, VERIFIED):
//! ```text
//! = EventID, Priority, Silent, FollowOn, Flashing, reportCount, soundCentre, waitOnSound, NoRollback
//! > DisplayProbability, DelayMs, SoundFile
//! G <in-game display text>
//! R <match-report text>
//! ```
//! This is the whole match commentary vocabulary — fully data-driven and moddable.

#![forbid(unsafe_code)]

use std::collections::BTreeMap;

/// The action code the match engine uses for a config `EventID`.
///
/// Cite: `FUN_006fce70` gates `param_2` to `8000..=8676` = `8000 + (0..=676)`.
pub const ACTION_BASE: u32 = 8000;

/// One match event.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Event {
    pub id: u32,
    pub priority: i32,
    /// `>` line: display probability (first field).
    pub display_probability: Option<i32>,
    /// `G` line: in-game commentary text.
    pub in_game: String,
    /// `R` line: match-report text (first one).
    pub report: String,
}

impl Event {
    /// The match-engine action code for this event (`8000 + id`).
    pub fn action_code(&self) -> u32 {
        ACTION_BASE + self.id
    }
}

/// A parsed `events_*.cfg`, keyed by EventID.
#[derive(Debug, Default)]
pub struct EventConfig {
    pub events: BTreeMap<u32, Event>,
}

impl EventConfig {
    /// Parse the config text. Comments (`#`) and blank lines are ignored.
    pub fn parse(text: &str) -> Self {
        let mut events = BTreeMap::new();
        let mut cur: Option<u32> = None;
        for line in text.lines() {
            let line = line.trim_end_matches(['\r', '\n']);
            match line.as_bytes().first() {
                Some(b'=') => {
                    let fields: Vec<&str> = line[1..].split(',').map(str::trim).collect();
                    if let Ok(id) = fields.first().unwrap_or(&"").parse::<u32>() {
                        let priority = fields.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                        cur = Some(id);
                        events.insert(
                            id,
                            Event {
                                id,
                                priority,
                                ..Default::default()
                            },
                        );
                    } else {
                        cur = None;
                    }
                }
                Some(b'>') => {
                    if let Some(e) = cur.and_then(|c| events.get_mut(&c)) {
                        e.display_probability = line[1..]
                            .split(',')
                            .next()
                            .and_then(|s| s.trim().parse().ok());
                    }
                }
                Some(b'G') => {
                    if let Some(e) = cur.and_then(|c| events.get_mut(&c)) {
                        e.in_game = line[1..].trim().to_string();
                    }
                }
                Some(b'R') => {
                    if let Some(e) = cur.and_then(|c| events.get_mut(&c)) {
                        if e.report.is_empty() {
                            e.report = line[1..].trim().to_string();
                        }
                    }
                }
                _ => {}
            }
        }
        EventConfig { events }
    }

    /// Look up an event by the match engine's action code (`8000 + id`).
    pub fn by_action_code(&self, code: u32) -> Option<&Event> {
        code.checked_sub(ACTION_BASE)
            .and_then(|id| self.events.get(&id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
# Match event config file
= 606, 5, 0, 0, 0, 1, 0, 0, 0
> 50, 1000, foo.wav
G appears short on confidence
R The player looked nervous today.
= 440, 9, 0, 0, 0, 1, 0, 0, 0
G sends the keeper the wrong way and scores !!!
";

    #[test]
    fn parses_events_and_action_codes() {
        let cfg = EventConfig::parse(SAMPLE);
        assert_eq!(cfg.events.len(), 2);
        let goal = &cfg.events[&440];
        assert_eq!(
            goal.in_game,
            "sends the keeper the wrong way and scores !!!"
        );
        assert_eq!(goal.action_code(), 8440); // 8000 + 440
        assert_eq!(cfg.by_action_code(8606).unwrap().id, 606);
        assert_eq!(cfg.events[&606].display_probability, Some(50));
    }
}
