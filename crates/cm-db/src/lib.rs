//! cm-db — the game's single entry point to the native Rust database.
//!
//! The canonical data lives in `rust-db/` (JSON, one file per table), produced
//! once from an original CM01/02 install by `cm-import`. At runtime the game
//! opens the database through this crate and never touches `.dat` files.
//!
//! This crate is a deliberate facade over `cm-domain`'s `World`:
//! - `Database::open(dir)` = `World::read_rust_db_dir` + an integrity check
//!   that every table is present with its expected record count.
//! - Typed field access reuses `cm_domain::typed_records` (offsets decoded
//!   from the exe's loader functions — see that module's provenance notes).
//!
//! Expected table counts are those of the shipped 3.9.60 database. A modified
//! database (edited players, added leagues) will differ — that's allowed; the
//! check distinguishes "table missing/empty" (hard error) from "count differs
//! from shipping data" (reported, not fatal).

use std::io;
use std::path::{Path, PathBuf};

pub mod config;

pub use cm_domain::typed_records::{ClubView, NationView, PlayerView};
pub use cm_domain::World;
pub use config::{ConfigData, TacticTemplate, WeatherConfig, WeatherSeason};

/// Shipping-data record counts (CM01/02 v3.9.60, from `Data/index.dat`,
/// cross-checked by exact file-size division — every table divides exactly).
pub const SHIPPED_COUNTS: &[(&str, usize)] = &[
    ("clubs", 10_580),
    ("nat_clubs", 426),
    ("colours", 34),
    ("continents", 6),
    ("nations", 213),
    ("staff_type6", 132_722),
    ("staff_type8", 0),
    ("staff_type9", 23_785),
    ("staff_type10", 109_940),
    ("cities", 5_418),
    ("officials", 3_124),
    ("stadiums", 7_099),
    ("staff_competitions", 542),
    ("club_competitions", 390),
    ("nation_competitions", 19),
    ("first_names", 34_363),
    ("second_names", 82_338),
    ("common_names", 8_493),
    ("staff_history", 270_654),
    ("staff_comp_history", 1_998),
    ("club_comp_history", 7_194),
    ("nation_comp_history", 143),
];

/// One table's presence/count as found on open.
#[derive(Debug, Clone)]
pub struct TableStatus {
    pub name: &'static str,
    pub found: usize,
    pub shipped: usize,
}

impl TableStatus {
    pub fn matches_shipping(&self) -> bool {
        self.found == self.shipped
    }
}

/// An opened native database.
pub struct Database {
    pub world: World,
    pub dir: PathBuf,
    pub tables: Vec<TableStatus>,
    /// Config-file data (weather + tactic templates) from `rust-db/config/`.
    /// Empty if the config directory hasn't been imported yet.
    pub config: ConfigData,
}

#[derive(Debug)]
pub enum OpenError {
    Io(io::Error),
    /// A table the game cannot run without is missing or empty
    /// (empty is fine only where the shipping DB is also empty).
    MissingTable(&'static str),
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenError::Io(e) => write!(f, "database io error: {e}"),
            OpenError::MissingTable(t) => write!(f, "required table `{t}` missing or empty"),
        }
    }
}

impl std::error::Error for OpenError {}

impl From<io::Error> for OpenError {
    fn from(e: io::Error) -> Self {
        OpenError::Io(e)
    }
}

impl Database {
    /// Open the native database at `dir` (a `rust-db/` directory).
    pub fn open(dir: &Path) -> Result<Self, OpenError> {
        let world = World::read_rust_db_dir(dir)?;
        let tables = table_statuses(&world);
        for t in &tables {
            if t.found == 0 && t.shipped != 0 {
                return Err(OpenError::MissingTable(t.name));
            }
        }
        let config = ConfigData::read_dir(dir).unwrap_or(ConfigData {
            weather: Vec::new(),
            tactics: Vec::new(),
        });
        Ok(Self {
            world,
            dir: dir.to_path_buf(),
            tables,
            config,
        })
    }

    /// True when every table carries exactly the shipping-data record count.
    pub fn is_pristine_shipping_data(&self) -> bool {
        self.tables.iter().all(|t| t.matches_shipping())
    }
}

fn count_for(world: &World, name: &str) -> usize {
    match name {
        "clubs" => world.core.clubs.len(),
        "nat_clubs" => world.core.nat_clubs.len(),
        "colours" => world.core.colours.len(),
        "continents" => world.core.continents.len(),
        "nations" => world.core.nations.len(),
        "staff_type6" => world.staff.type6.len(),
        "staff_type8" => world.staff.type8.len(),
        "staff_type9" => world.staff.type9.len(),
        "staff_type10" => world.staff.type10.len(),
        "cities" => world.references.cities.len(),
        "officials" => world.references.officials.len(),
        "stadiums" => world.references.stadiums.len(),
        "staff_competitions" => world.references.staff_competitions.len(),
        "club_competitions" => world.references.club_competitions.len(),
        "nation_competitions" => world.references.nation_competitions.len(),
        "first_names" => world.references.first_names.len(),
        "second_names" => world.references.second_names.len(),
        "common_names" => world.references.common_names.len(),
        "staff_history" => world.references.staff_history.len(),
        "staff_comp_history" => world.references.staff_comp_history.len(),
        "club_comp_history" => world.references.club_comp_history.len(),
        "nation_comp_history" => world.references.nation_comp_history.len(),
        _ => 0,
    }
}

fn table_statuses(world: &World) -> Vec<TableStatus> {
    SHIPPED_COUNTS
        .iter()
        .map(|&(name, shipped)| TableStatus {
            name,
            found: count_for(world, name),
            shipped,
        })
        .collect()
}
