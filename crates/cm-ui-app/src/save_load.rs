//! Save / load for the playable app (`cm-ui-app`).
//!
//! A save is a DIRECTORY (a "slot") holding two JSON files:
//!   * `save.json`  — the `RuntimeSaveGame` (all mutable game state);
//!   * `world.json` — the mutated `World` (the loaded `.dat` database PLUS the
//!     in-place cross-season mutations: promotions/relegations via
//!     `run_english_year_end`, `run_start_game_init` output, etc.).
//!
//! We persist the World rather than re-reading `rust-db` on load because the
//! `SaveWorldOverlay` that would otherwise capture those in-place mutations is
//! currently a stub — re-reading the fresh database would silently lose every
//! cross-season World change (see reports/match_engine_observational_gap.md).
//! This is NOT original CM binary-save compatibility; it is a modern
//! round-trippable Rust format (the domain's own serde JSON), chosen for
//! correctness first.

use std::path::{Path, PathBuf};

use cm_domain::{RuntimeSaveGame, World};

/// The default single save slot (env `CM_SAVE_DIR` overrides). Multi-slot UI is
/// a later refinement; one working slot satisfies Save → quit → Load → continue.
pub fn default_slot_dir() -> PathBuf {
    std::env::var("CM_SAVE_DIR")
        .unwrap_or_else(|_| "D:/cm0102-rs/saves/slot1".to_string())
        .into()
}

fn save_json_path(dir: &Path) -> PathBuf {
    dir.join("save.json")
}
fn world_json_path(dir: &Path) -> PathBuf {
    dir.join("world.json")
}

/// Is there a loadable save in `dir`?
pub fn has_save(dir: &Path) -> bool {
    save_json_path(dir).is_file() && world_json_path(dir).is_file()
}

/// Persist `save` + the mutated `world` into slot `dir` (created if needed).
/// Writes to sibling temp files then renames, so an interrupted write never
/// leaves a half-written slot that would fail to load.
// GDI-REG: 005176c0 REPLACED_BY_RUST
// GDI-REG: 00818060 REPLACED_BY_RUST
pub fn save_game(dir: &Path, save: &RuntimeSaveGame, world: &World) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let save_tmp = dir.join("save.json.tmp");
    let world_tmp = dir.join("world.json.tmp");
    // World first (the big one) — if it fails we haven't clobbered a good save.
    // STREAMING writes via serde_json::to_writer: a full World JSON is ~1 GB and
    // building it in memory (serde_json::to_vec) OOMs on a low-RAM host, so we
    // serialize straight into a buffered file writer.
    {
        let writer = std::io::BufWriter::new(std::fs::File::create(&world_tmp)?);
        serde_json::to_writer(writer, world)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    }
    {
        let writer = std::io::BufWriter::new(std::fs::File::create(&save_tmp)?);
        serde_json::to_writer(writer, save)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    }
    std::fs::rename(&world_tmp, world_json_path(dir))?;
    std::fs::rename(&save_tmp, save_json_path(dir))?;
    Ok(())
}

/// Load `save` + `world` from slot `dir`. Rebuilds nothing derived here — the
/// caller re-establishes `world_init_done` (the saved World is already
/// initialised) and re-registers the season-roll scheduler if a legacy save
/// arrived with an empty one.
// GDI-REG: 00814870 REPLACED_BY_RUST
pub fn load_game(dir: &Path) -> std::io::Result<(RuntimeSaveGame, World)> {
    // STREAMING reads: avoid slurping the ~1 GB World JSON into a String first.
    let world: World = {
        let reader = std::io::BufReader::new(std::fs::File::open(world_json_path(dir))?);
        serde_json::from_reader(reader)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
    };
    let mut save: RuntimeSaveGame = {
        let reader = std::io::BufReader::new(std::fs::File::open(save_json_path(dir))?);
        serde_json::from_reader(reader)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?
    };
    // Legacy-save safety: a save written before `season_roll_scheduler` existed
    // (or before the English pyramid was registered) loads with an empty
    // scheduler under `#[serde(default)]`, which would silently lose future
    // Jan-1 fixture regeneration. Fresh saves serialize a full scheduler, so
    // this only fires for such legacy saves. `register_english_pyramid` is
    // World-free and idempotent on an empty scheduler.
    if save.season_roll_scheduler.total_registered() == 0 {
        cm_domain::season_roll_scheduler::register_english_pyramid(
            &mut save.season_roll_scheduler,
        );
    }
    Ok((save, world))
}
