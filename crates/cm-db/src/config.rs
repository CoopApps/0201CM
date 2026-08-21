//! Config-file data: weather and tactic templates.
//!
//! These are NOT part of the `.dat` record database (they're absent from
//! `index.dat`); the game reads them as standalone files from `Data/`. The
//! importer parses them into the native database under `rust-db/config/` so
//! the Rust game never needs the original install for them either.
//!
//! - **Weather** (`weather.cfg`, text) is FULLY parsed into typed records.
//! - **Tactics** (`*.pct`, binary) are imported as typed templates: the
//!   filename-derived name, the author string, size, and the raw bytes are
//!   preserved. The per-position tactical fields inside the blob are not yet
//!   decoded (same honesty as the raw staff bodies elsewhere in rust-db) —
//!   the templates round-trip byte-exact and are ready for deeper decode.

use serde::{Deserialize, Serialize};

// ---------------- Weather ----------------

/// One season's weather curve within a configuration. Each threshold array is
/// cumulative probability to 100 (a roll < W[0] = Calm, < W[1] = Breezy, ...).
/// `-1` marks a category that never occurs in this season.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherSeason {
    pub name: String, // SPRING / SUMMER / AUTUMN / WINTER
    pub start_day: i32,
    /// Wind: Calm, Breezy, Gusty, Strong, Gale (5 cumulative thresholds).
    pub wind: [i32; 5],
    /// Precipitation: Dry, Wet, Drizzle, Shower, Down Pour.
    pub precipitation: [i32; 5],
    /// Warmth: Freezing, Cold, Mild, Fine, Warm, Hot, Very Hot (7).
    pub warmth: [i32; 7],
}

/// A named weather configuration (e.g. "English Coastal") with four seasons.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherConfig {
    pub id: u32,
    pub name: String,
    pub seasons: Vec<WeatherSeason>,
}

/// Parse the whole `weather.cfg` text into typed configurations.
pub fn parse_weather_cfg(text: &str) -> Vec<WeatherConfig> {
    let mut configs = Vec::new();
    let mut current: Option<WeatherConfig> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Season line: "SPRING: 45, W75, ..."
        if let Some(colon) = line.find(':') {
            let label = &line[..colon];
            if matches!(label, "SPRING" | "SUMMER" | "AUTUMN" | "WINTER") {
                if let Some(season) = parse_weather_season(label, &line[colon + 1..]) {
                    if let Some(cfg) = current.as_mut() {
                        cfg.seasons.push(season);
                    }
                }
                continue;
            }
        }
        // Otherwise a config header "<id> <name>". Starts a new config; flush
        // the previous one.
        let mut parts = line.splitn(2, char::is_whitespace);
        if let (Some(id_str), Some(name)) = (parts.next(), parts.next()) {
            if let Ok(id) = id_str.parse::<u32>() {
                if let Some(done) = current.take() {
                    configs.push(done);
                }
                current = Some(WeatherConfig {
                    id,
                    name: name.trim().to_string(),
                    seasons: Vec::new(),
                });
                continue;
            }
        }
        // A bare count line ("43") or anything else — ignore.
    }
    if let Some(done) = current.take() {
        configs.push(done);
    }
    configs
}

fn parse_weather_season(name: &str, rest: &str) -> Option<WeatherSeason> {
    // Values: start_day, then W*5, P*5, H*7. Prefix letters are stripped.
    let nums: Vec<i32> = rest
        .split(',')
        .map(|t| t.trim().trim_start_matches(['W', 'P', 'H']).trim())
        .filter(|t| !t.is_empty())
        .filter_map(|t| t.parse::<i32>().ok())
        .collect();
    if nums.len() < 18 {
        return None;
    }
    let mut wind = [0i32; 5];
    let mut precipitation = [0i32; 5];
    let mut warmth = [0i32; 7];
    wind.copy_from_slice(&nums[1..6]);
    precipitation.copy_from_slice(&nums[6..11]);
    warmth.copy_from_slice(&nums[11..18]);
    Some(WeatherSeason {
        name: name.to_string(),
        start_day: nums[0],
        wind,
        precipitation,
        warmth,
    })
}

// ---------------- Tactics ----------------

/// A formation/tactic template imported from a `.pct` file. The raw bytes are
/// preserved byte-exact; the per-position tactical fields are not yet decoded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TacticTemplate {
    /// Filename stem, e.g. "442_default".
    pub name: String,
    /// Author string embedded in the file (e.g. "Marc Vaughan"), if present.
    pub author: Option<String>,
    /// Raw file size (bytes) — one of the known variants (1428/1432/1476).
    pub size: usize,
    /// The complete raw file contents.
    pub raw: Vec<u8>,
}

impl TacticTemplate {
    /// Build a template from a `.pct` file's name stem and bytes.
    pub fn from_bytes(name: &str, bytes: &[u8]) -> Self {
        let author = extract_author(bytes);
        TacticTemplate {
            name: name.to_string(),
            author,
            size: bytes.len(),
            raw: bytes.to_vec(),
        }
    }
}

/// Pull the first embedded printable-ASCII run of length >= 5 that looks like a
/// name (contains a space) — the `.pct` files carry the designer's name.
fn extract_author(bytes: &[u8]) -> Option<String> {
    let mut run = Vec::new();
    for &b in bytes {
        if (0x20..0x7f).contains(&b) {
            run.push(b);
        } else {
            if run.len() >= 5 && run.contains(&b' ') {
                return Some(String::from_utf8_lossy(&run).into_owned());
            }
            run.clear();
        }
    }
    None
}

// ---------------- Bundle + IO ----------------

/// Both config groups, as written under `rust-db/config/`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigData {
    pub weather: Vec<WeatherConfig>,
    pub tactics: Vec<TacticTemplate>,
}

impl ConfigData {
    /// Import from an original install's `Data/` directory: parse
    /// `weather.cfg` and every `*.pct` tactic file.
    pub fn import_from_data_dir(data_dir: &std::path::Path) -> std::io::Result<Self> {
        let weather = match std::fs::read(data_dir.join("weather.cfg")) {
            // weather.cfg is Latin-1, not UTF-8 — decode byte-for-byte.
            Ok(bytes) => {
                let text: String = bytes.iter().map(|&b| b as char).collect();
                parse_weather_cfg(&text)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e),
        };
        let mut tactics = Vec::new();
        if let Ok(entries) = std::fs::read_dir(data_dir) {
            let mut paths: Vec<_> = entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.extension()
                        .and_then(|x| x.to_str())
                        .map(|x| x.eq_ignore_ascii_case("pct"))
                        .unwrap_or(false)
                })
                .collect();
            paths.sort();
            for p in paths {
                let bytes = std::fs::read(&p)?;
                let stem = p
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                tactics.push(TacticTemplate::from_bytes(&stem, &bytes));
            }
        }
        Ok(Self { weather, tactics })
    }

    /// Write to `<dir>/config/` as `weather.json` + `tactics.json`.
    pub fn write_dir(&self, dir: &std::path::Path) -> std::io::Result<()> {
        let cdir = dir.join("config");
        std::fs::create_dir_all(&cdir)?;
        let w = serde_json::to_vec_pretty(&self.weather)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(cdir.join("weather.json"), w)?;
        let t = serde_json::to_vec_pretty(&self.tactics)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(cdir.join("tactics.json"), t)?;
        Ok(())
    }

    /// Load the RNG ring-buffer table from `<dir>/config/rng_table.bin` as
    /// i32 entries — the game's precomputed random table (FUN_008fc4f0). Empty
    /// if the file is absent. Feed this to `cm_rng::MatchRng` for bit-exact
    /// game randomness.
    pub fn read_rng_table(dir: &std::path::Path) -> std::io::Result<Vec<i32>> {
        match std::fs::read(dir.join("config").join("rng_table.bin")) {
            Ok(bytes) => Ok(bytes
                .chunks_exact(4)
                .map(|c| i32::from_le_bytes([c[0], c[1], c[2], c[3]]))
                .collect()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    /// Read back from `<dir>/config/`. Missing files yield empty vecs.
    pub fn read_dir(dir: &std::path::Path) -> std::io::Result<Self> {
        let cdir = dir.join("config");
        let weather = match std::fs::read(cdir.join("weather.json")) {
            Ok(b) => serde_json::from_slice(&b)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e),
        };
        let tactics = match std::fs::read(cdir.join("tactics.json")) {
            Ok(b) => serde_json::from_slice(&b)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e),
        };
        Ok(Self { weather, tactics })
    }
}
