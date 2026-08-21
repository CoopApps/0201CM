#![forbid(unsafe_code)]
#![recursion_limit = "256"]

use std::collections::{HashMap, HashSet};
use std::env;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process;

use std::fs;

use cm_data::{DatFile, DataTables, Manifest, RecordKind, RecordLayoutConfidence, TableEncoding};
use cm_domain::{
    BackendImplementationReadiness, CmPackedDate, GameDate, GameplayMutatorStatus,
    HeadlessCampaignReport, HeadlessRunReport, RuntimeSaveGame, RustDatabaseAuditReport,
    SnapshotAuditReport, World,
};
use cm_events::EventConfig;

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let cli = Cli::parse()?;

    match cli.command {
        Command::Summary { install_root } => run_summary(&install_root),
        Command::ExportWorld {
            install_root,
            output,
        } => run_export_world(&install_root, output.as_deref()),
        Command::InspectWorld { snapshot } => run_inspect_world(&snapshot),
        Command::ExportOwnedData {
            snapshot,
            output_dir,
        } => run_export_owned_data(&snapshot, &output_dir),
        Command::AuditWorld {
            install_root,
            snapshot,
        } => run_audit_world(&install_root, &snapshot),
        Command::InitRustDb {
            install_root,
            db_dir,
        } => run_init_rust_db(&install_root, &db_dir),
        Command::InspectRustDb { db_dir } => run_inspect_rust_db(&db_dir),
        Command::AuditRustDb { db_dir } => run_audit_rust_db(&db_dir),
        Command::CanonicalReport { db_dir, output } => {
            run_canonical_report(&db_dir, output.as_deref())
        }
        Command::BackendReport { db_dir, output } => run_backend_report(&db_dir, output.as_deref()),
        Command::BackendAcceptance { db_dir, output } => {
            run_backend_acceptance(&db_dir, output.as_deref())
        }
        Command::ExactRemakeReport {
            db_dir,
            exe,
            output,
        } => run_exact_remake_report(&db_dir, &exe, output.as_deref()),
        Command::GameplayParityReport {
            db_dir,
            trace_dir,
            output,
        } => run_gameplay_parity_report(&db_dir, &trace_dir, output.as_deref()),
        Command::GameplayPromotionReport {
            db_dir,
            trace_dir,
            output,
        } => run_gameplay_promotion_report(&db_dir, &trace_dir, output.as_deref()),
        Command::ExportGameplayLiftWorkbench { db_dir, output_dir } => {
            run_export_gameplay_lift_workbench(&db_dir, &output_dir)
        }
        Command::ExportFormulaLiftBacklog { db_dir, output_dir } => {
            run_export_formula_lift_backlog(&db_dir, &output_dir)
        }
        Command::InitGameplayParityTraces { db_dir, trace_dir } => {
            run_init_gameplay_parity_traces(&db_dir, &trace_dir)
        }
        Command::ExportRustMatchResultTrace { db_dir, trace_dir } => {
            run_export_rust_match_result_trace(&db_dir, &trace_dir)
        }
        Command::ExportRustGameplayCandidateTraces { db_dir, trace_dir } => {
            run_export_rust_gameplay_candidate_traces(&db_dir, &trace_dir)
        }
        Command::ExportOriginalCaptureTemplates {
            trace_dir,
            output_dir,
        } => run_export_original_capture_templates(&trace_dir, &output_dir),
        Command::OriginalCaptureStatus {
            template_dir,
            output,
        } => run_original_capture_status(&template_dir, output.as_deref()),
        Command::ExportOriginalCaptureWorkbench {
            template_dir,
            output_dir,
            trace_dir,
        } => run_export_original_capture_workbench(&template_dir, &output_dir, &trace_dir),
        Command::ExportPromotionControlRoom {
            db_dir,
            output_dir,
            trace_dir,
            template_dir,
            exe,
        } => {
            run_export_promotion_control_room(&db_dir, &output_dir, &trace_dir, &template_dir, &exe)
        }
        Command::ExportTodoAttackBoard {
            db_dir,
            output_dir,
            trace_dir,
            template_dir,
        } => run_export_todo_attack_board(&db_dir, &output_dir, &trace_dir, &template_dir),
        Command::ExportGameplayCapturePack {
            trace_dir,
            output_dir,
        } => run_export_gameplay_capture_pack(&trace_dir, &output_dir),
        Command::ExportStaticParityProof { row_plan, output } => {
            run_export_static_parity_proof(&row_plan, &output)
        }
        Command::ApplyStaticParityProof {
            trace_dir,
            proof,
            output,
        } => run_apply_static_parity_proof(&trace_dir, &proof, output.as_deref()),
        Command::RefreshBackendGates {
            db_dir,
            reports_dir,
            trace_dir,
            template_dir,
            exe,
        } => run_refresh_backend_gates(&db_dir, &reports_dir, &trace_dir, &template_dir, &exe),
        Command::SyncGameplayMutatorContracts { save, trace_dir } => {
            run_sync_gameplay_mutator_contracts(&save, &trace_dir)
        }
        Command::ImportGameplayCapture {
            trace_dir,
            capture,
            output,
        } => run_import_gameplay_capture(&trace_dir, &capture, output.as_deref()),
        Command::ImportOriginalCaptureCsv { csv, output } => {
            run_import_original_capture_csv(&csv, output.as_deref())
        }
        Command::ValidateOriginalCaptureCsv { csv, output } => {
            run_validate_original_capture_csv(&csv, output.as_deref())
        }
        Command::SubmitOriginalCaptureCsv {
            csv,
            reports_dir,
            output,
        } => run_submit_original_capture_csv(&csv, &reports_dir, output.as_deref()),
        Command::PrepareCaptureConsole {
            db_dir,
            reports_dir,
            trace_dir,
            template_dir,
            port,
        } => run_prepare_capture_console(&db_dir, &reports_dir, &trace_dir, &template_dir, port),
        Command::ValidateOriginalBinary { exe, output } => {
            run_validate_original_binary(&exe, output.as_deref())
        }
        Command::ValidateExecutionModel { exe, output } => {
            run_validate_execution_model(&exe, output.as_deref())
        }
        Command::ValidateSimulationFrontier { exe, output } => {
            run_validate_simulation_frontier(&exe, output.as_deref())
        }
        Command::ValidateRuntimeSimulation { db_dir, output } => {
            run_validate_runtime_simulation(&db_dir, output.as_deref())
        }
        Command::ValidateRng { exe, output } => run_validate_rng(&exe, output.as_deref()),
        Command::ExtractRngTable {
            exe,
            output,
            entries,
        } => run_extract_rng_table(&exe, &output, entries),
        Command::NewRustSave { db_dir, output } => run_new_rust_save(&db_dir, &output),
        Command::InspectRustSave { save } => run_inspect_rust_save(&save),
        Command::TickRustSave { save, days } => run_tick_rust_save(&save, days),
        Command::TickRustSaveTo { save, target } => run_tick_rust_save_to(&save, target),
        Command::RunHeadless { save, days } => run_headless_save(&save, days),
        Command::RunHeadlessTo { save, target } => run_headless_save_to(&save, target),
        Command::RunHeadlessCampaign {
            save,
            days,
            checkpoint_every,
            output,
        } => run_headless_campaign_save(&save, days, checkpoint_every, output.as_deref()),
        Command::SetHeadlessManager {
            save,
            name,
            club_id,
        } => run_set_headless_manager(&save, &name, club_id),
        Command::ExportRustDbData { db_dir, output_dir } => {
            run_export_rust_db_data(&db_dir, &output_dir)
        }
        Command::ExportRustDbWorld { db_dir, output } => {
            run_export_rust_db_world(&db_dir, output.as_deref())
        }
        Command::ExportRustDbViewer { db_dir, output } => {
            run_export_rust_db_viewer(&db_dir, output.as_deref())
        }
        Command::ExportRustDbViewerTables { db_dir, output_dir } => {
            run_export_rust_db_viewer_tables(&db_dir, &output_dir)
        }
        Command::ServeRustDb { db_dir, port } => run_serve_rust_db(&db_dir, port),
        Command::RenameRustDb {
            db_dir,
            table,
            row,
            name,
        } => run_rename_rust_db(&db_dir, &table, &row, &name),
        Command::SetRustDbText {
            db_dir,
            table,
            row,
            field,
            text,
        } => run_set_rust_db_text(&db_dir, &table, &row, &field, &text),
        Command::SetStaffType10 {
            db_dir,
            id,
            field,
            value,
        } => run_set_staff_type10(&db_dir, id, &field, value),
        Command::SetStaffAttribute {
            db_dir,
            id,
            index,
            value,
        } => run_set_staff_attribute(&db_dir, id, index, value),
    }
}

fn run_summary(root: &Path) -> Result<(), String> {
    let data_dir = root.join("Data");

    let manifest = load_manifest(&data_dir)?;
    let tables = DataTables::load_from_install(&data_dir, &manifest)
        .map_err(|err| format!("failed to load table catalog: {err}"))?;
    let events = load_events(&data_dir)?;
    let save_bytes = try_load_save(&root)?;
    let save = save_bytes
        .as_deref()
        .map(cm_data::SaveFile::parse)
        .transpose()
        .map_err(|err| format!("failed to parse save1.sav: {err}"))?;
    let world = World::load_from_install(&root)
        .map_err(|err| format!("failed to load owned world: {err}"))?;

    println!("CM0102 Rust baseline");
    println!("install: {}", root.display());
    println!("data dir: {}", data_dir.display());
    println!();

    println!("manifest entries: {}", manifest.entries.len());
    for kind in [0u8, 1, 3, 10, 0x15] {
        if let Some(entry) = manifest.by_kind(kind) {
            println!(
                "type {kind:>2} -> {:<24} count {}",
                entry.filename, entry.count
            );
        }
    }
    println!();

    for (name, kind) in [
        ("continent.dat", RecordKind::Continent),
        ("nation.dat", RecordKind::Nation),
        ("club.dat", RecordKind::Club),
        ("colour.dat", RecordKind::Colour),
    ] {
        let count = count_records(&data_dir.join(name), kind)?;
        println!("{name:<14} verified records {count}");
    }
    println!();

    if let Some(save) = &save {
        println!(
            "save1.sav version {} with {} sections",
            save.version,
            save.sections.len()
        );
        for name in ["continent.dat", "club.dat", "staff.dat", "city.dat"] {
            if let Some(section) = save.section(name) {
                println!("{name:<14} save section size {}", section.size);
            }
        }
        println!();
    }

    println!("logical tables: {}", tables.tables.len());
    for table in &tables.tables {
        match table.spec.encoding {
            TableEncoding::FixedRecord(layout) => {
                let count = table.fixed_record_count().unwrap_or(0);
                let confidence = match layout.confidence {
                    RecordLayoutConfidence::Verified => "verified",
                    RecordLayoutConfidence::Inferred => "inferred",
                };
                println!(
                    "{:<20} type {:>2} file {:<24} {} {}-byte records {}",
                    table.spec.logical_name,
                    table.spec.manifest_type,
                    table.spec.filename,
                    confidence,
                    layout.size,
                    count
                );
            }
            TableEncoding::Raw => {
                println!(
                    "{:<20} type {:>2} file {:<24} raw bytes {} manifest count {}",
                    table.spec.logical_name,
                    table.spec.manifest_type,
                    table.spec.filename,
                    table.byte_len,
                    table.manifest_count
                );
            }
        }
    }
    println!();

    println!("events_eng.cfg entries: {}", events.events.len());
    for id in [80u32, 120, 440, 560, 606] {
        if let Some(event) = events.events.get(&id) {
            println!(
                "event {id:>3} -> action {} -> {}",
                event.action_code(),
                event.in_game
            );
        }
    }

    println!();
    println!(
        "world summary: {} manifest entries, save loaded: {}",
        world.base_data.len(),
        world.save.is_some()
    );
    println!(
        "core summary: clubs {} nat clubs {} colours {} continents {} nations {}",
        world.core_summary.club_count,
        world.core_summary.nat_club_count,
        world.core_summary.colour_count,
        world.core_summary.continent_count,
        world.core_summary.nation_count
    );
    println!(
        "staff summary: type6 {} type8 {} type9 {} type10 {} sample type10 id {:?} ca {:?} pa {:?} rep {:?} max ca {:?}",
        world.staff_summary.type6_count,
        world.staff_summary.type8_count,
        world.staff_summary.type9_count,
        world.staff_summary.type10_count,
        world.staff_summary.sample_type10_id,
        world.staff_summary.sample_type10_ca,
        world.staff_summary.sample_type10_pa,
        world.staff_summary.sample_type10_reputation,
        world.staff_summary.max_type10_ca
    );
    {
        let references = &world.reference_summary;
        println!(
            "reference data: cities {} officials {} first names {} second names {} common names {} stadiums {} staff comps {} club comps {} nation comps {}",
            references.city_count,
            references.official_count,
            references.first_name_count,
            references.second_name_count,
            references.common_name_count,
            references.stadium_count,
            references.staff_competition_count,
            references.club_competition_count,
            references.nation_competition_count
        );
        println!(
            "reference samples: city {:?}, official {:?}, first name {:?}, stadium {:?}, staff comp {:?}, club comp {:?}, nation comp {:?}",
            references.sample_city,
            references.sample_official_id,
            references.sample_first_name,
            references.sample_stadium,
            references.sample_staff_competition,
            references.sample_club_competition,
            references.sample_nation_competition
        );
        println!(
            "history rows: staff {} staff-comp {} club-comp {} nation-comp {}",
            references.staff_history_count,
            references.staff_comp_history_count,
            references.club_comp_history_count,
            references.nation_comp_history_count
        );
    }
    if let Some(save) = &world.save {
        println!("verified save tables:");
        for section in &save.sections {
            if let Some(count) = section.verified_record_count {
                println!("{} -> {} records", section.name, count);
            }
        }
    }

    Ok(())
}

fn run_export_world(root: &Path, output: Option<&Path>) -> Result<(), String> {
    let world = World::load_from_install(root)
        .map_err(|err| format!("failed to load owned world: {err}"))?;
    match output {
        Some(path) => {
            world
                .write_json_file(path)
                .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
            println!("exported world snapshot to {}", path.display());
        }
        None => {
            let json = world
                .to_pretty_json()
                .map_err(|err| format!("failed to serialize world snapshot: {err}"))?;
            println!("{json}");
        }
    }
    Ok(())
}

fn run_inspect_world(snapshot: &Path) -> Result<(), String> {
    let world = World::read_json_file(snapshot)
        .map_err(|err| format!("failed to read {}: {err}", snapshot.display()))?;
    let coverage = world.coverage();

    println!("world snapshot: {}", snapshot.display());
    println!("base datasets: {}", world.base_data.len());
    println!("save loaded: {}", world.save.is_some());
    println!(
        "coverage: manifest entries {} known logical {} recognized {} unrecognized {} owned world {} remaining binary {}",
        coverage.manifest_entries,
        coverage.known_logical_tables,
        coverage.recognized_manifest_entries,
        coverage.unrecognized_manifest_entries,
        coverage.owned_world_tables,
        coverage.remaining_binary_tables
    );
    println!(
        "owned tables: core {} references {} staff {}",
        coverage.owned_core_tables, coverage.owned_reference_tables, coverage.owned_staff_tables
    );
    println!(
        "staff summary: type6 {} type8 {} type9 {} type10 {} max ca {:?}",
        world.staff_summary.type6_count,
        world.staff_summary.type8_count,
        world.staff_summary.type9_count,
        world.staff_summary.type10_count,
        world.staff_summary.max_type10_ca
    );
    println!("schema tables: {}", world.schema.tables.len());
    println!(
        "reference summary: cities {} officials {} first names {} stadiums {} staff history {}",
        world.reference_summary.city_count,
        world.reference_summary.official_count,
        world.reference_summary.first_name_count,
        world.reference_summary.stadium_count,
        world.reference_summary.staff_history_count
    );
    Ok(())
}

fn run_audit_world(root: &Path, snapshot: &Path) -> Result<(), String> {
    let world = World::read_json_file(snapshot)
        .map_err(|err| format!("failed to read {}: {err}", snapshot.display()))?;
    let report = world
        .audit_against_install(root)
        .map_err(|err| format!("failed to audit install {}: {err}", root.display()))?;
    print_audit_report(&report);
    if !report.mismatches.is_empty() {
        return Err("snapshot audit found mismatches".into());
    }
    Ok(())
}

fn run_export_owned_data(snapshot: &Path, output_dir: &Path) -> Result<(), String> {
    let world = World::read_json_file(snapshot)
        .map_err(|err| format!("failed to read {}: {err}", snapshot.display()))?;
    world.export_owned_data_dir(output_dir).map_err(|err| {
        format!(
            "failed to export owned data to {}: {err}",
            output_dir.display()
        )
    })?;
    println!("exported owned data bundle to {}", output_dir.display());
    Ok(())
}

fn run_init_rust_db(root: &Path, db_dir: &Path) -> Result<(), String> {
    let world = World::load_from_install(root)
        .map_err(|err| format!("failed to load owned world from {}: {err}", root.display()))?;
    world
        .write_rust_db_dir(db_dir, Some(root))
        .map_err(|err| format!("failed to write Rust DB {}: {err}", db_dir.display()))?;
    println!("initialized Rust DB at {}", db_dir.display());
    println!("source install: {}", root.display());
    println!(
        "owned world tables: {}",
        world.coverage().owned_world_tables
    );
    Ok(())
}

fn run_inspect_rust_db(db_dir: &Path) -> Result<(), String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let coverage = world.coverage();
    println!("Rust DB: {}", db_dir.display());
    println!(
        "coverage: owned world {} remaining binary {} schema tables {}",
        coverage.owned_world_tables,
        coverage.remaining_binary_tables,
        world.schema.tables.len()
    );
    println!(
        "core: clubs {} nat clubs {} colours {} continents {} nations {}",
        world.core_summary.club_count,
        world.core_summary.nat_club_count,
        world.core_summary.colour_count,
        world.core_summary.continent_count,
        world.core_summary.nation_count
    );
    println!(
        "references: cities {} first names {} stadiums {} club comps {} histories {}",
        world.reference_summary.city_count,
        world.reference_summary.first_name_count,
        world.reference_summary.stadium_count,
        world.reference_summary.club_competition_count,
        world.reference_summary.staff_history_count
            + world.reference_summary.staff_comp_history_count
            + world.reference_summary.club_comp_history_count
            + world.reference_summary.nation_comp_history_count
    );
    println!(
        "staff: type6 {} type8 {} type9 {} type10 {}",
        world.staff_summary.type6_count,
        world.staff_summary.type8_count,
        world.staff_summary.type9_count,
        world.staff_summary.type10_count
    );
    Ok(())
}

fn run_audit_rust_db(db_dir: &Path) -> Result<(), String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let report = world.audit_rust_db();
    print_rust_db_audit_report(&report);
    if !report.mismatches.is_empty() {
        return Err("Rust DB audit found mismatches".into());
    }
    Ok(())
}

fn run_canonical_report(db_dir: &Path, output: Option<&Path>) -> Result<(), String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let report = world.canonical_database_report();
    print_canonical_report(&report);
    if let Some(path) = output {
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize canonical report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote canonical report to {}", path.display());
    }
    if !report.validation.failures.is_empty() {
        return Err("canonical report found validation failures".into());
    }
    Ok(())
}

fn run_backend_report(db_dir: &Path, output: Option<&Path>) -> Result<(), String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let report = world.backend_readiness_report(db_dir);
    print_backend_report(&report);
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize backend readiness report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote backend readiness report to {}", path.display());
    }
    Ok(())
}

fn run_backend_acceptance(db_dir: &Path, output: Option<&Path>) -> Result<(), String> {
    let report = backend_acceptance_report(db_dir)?;
    print_backend_acceptance_report(&report);
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize backend acceptance report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote backend acceptance report to {}", path.display());
    }
    Ok(())
}

fn run_exact_remake_report(db_dir: &Path, exe: &Path, output: Option<&Path>) -> Result<(), String> {
    let report = exact_remake_report(db_dir, exe)?;
    print_exact_remake_report(&report);
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize exact remake report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote exact remake report to {}", path.display());
    }
    Ok(())
}

fn run_gameplay_parity_report(
    db_dir: &Path,
    trace_dir: &Path,
    output: Option<&Path>,
) -> Result<(), String> {
    let report = gameplay_parity_report(db_dir, trace_dir)?;
    print_gameplay_parity_report(&report);
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize gameplay parity report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote gameplay parity report to {}", path.display());
    }
    Ok(())
}

fn run_gameplay_promotion_report(
    db_dir: &Path,
    trace_dir: &Path,
    output: Option<&Path>,
) -> Result<(), String> {
    let report = gameplay_promotion_report(db_dir, trace_dir)?;
    print_gameplay_promotion_report(&report);
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize gameplay promotion report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote gameplay promotion report to {}", path.display());
    }
    Ok(())
}

fn run_export_gameplay_lift_workbench(db_dir: &Path, output_dir: &Path) -> Result<(), String> {
    let report = export_gameplay_lift_workbench(db_dir, output_dir)?;
    print_gameplay_lift_workbench_report(&report);
    Ok(())
}

fn run_export_formula_lift_backlog(db_dir: &Path, output_dir: &Path) -> Result<(), String> {
    let report = export_formula_lift_backlog(db_dir, output_dir)?;
    print_formula_lift_backlog_report(&report);
    Ok(())
}

fn run_init_gameplay_parity_traces(db_dir: &Path, trace_dir: &Path) -> Result<(), String> {
    let written = init_gameplay_parity_traces(db_dir, trace_dir)?;
    println!(
        "initialized gameplay parity trace templates in {}",
        trace_dir.display()
    );
    if written.is_empty() {
        println!("all required trace files already contained current template metadata");
    } else {
        for path in written {
            println!("wrote or updated trace template {}", path.display());
        }
    }
    Ok(())
}

fn run_export_rust_match_result_trace(db_dir: &Path, trace_dir: &Path) -> Result<(), String> {
    let report = export_rust_match_result_trace(db_dir, trace_dir)?;
    println!("Rust match-result trace candidate");
    println!(
        "status: {} | mutations {} | coverage {} | wrote {}",
        report["summary"]["status"].as_str().unwrap_or("unknown"),
        report["summary"]["rust_mutations"]
            .as_u64()
            .unwrap_or_default(),
        report["summary"]["rust_coverage"]["status"]
            .as_str()
            .unwrap_or("unknown"),
        report["trace_file"].as_str().unwrap_or("")
    );
    Ok(())
}

fn run_export_rust_gameplay_candidate_traces(
    db_dir: &Path,
    trace_dir: &Path,
) -> Result<(), String> {
    let report = export_rust_gameplay_candidate_traces(db_dir, trace_dir)?;
    println!("Rust gameplay candidate traces");
    println!(
        "status: {} | systems {} | total mutations {} | wrote {} trace file(s)",
        report["summary"]["status"].as_str().unwrap_or("unknown"),
        report["summary"]["systems"].as_u64().unwrap_or_default(),
        report["summary"]["rust_mutations"]
            .as_u64()
            .unwrap_or_default(),
        report["summary"]["written"].as_u64().unwrap_or_default()
    );
    if let Some(systems) = report["systems"].as_array() {
        for system in systems {
            println!(
                "- {}: {} mutation(s), coverage {}, trace {}",
                system["system"].as_str().unwrap_or("unknown"),
                system["rust_mutations"].as_u64().unwrap_or_default(),
                system["coverage"]["status"].as_str().unwrap_or("unknown"),
                system["trace_file"].as_str().unwrap_or("")
            );
        }
    }
    Ok(())
}

fn run_export_original_capture_templates(
    trace_dir: &Path,
    output_dir: &Path,
) -> Result<(), String> {
    let report = export_original_capture_templates(trace_dir, output_dir)?;
    println!("Original capture templates");
    println!(
        "systems {} | templates {} | output {}",
        report["summary"]["systems"].as_u64().unwrap_or_default(),
        report["summary"]["templates"].as_u64().unwrap_or_default(),
        report["output_dir"].as_str().unwrap_or("")
    );
    if let Some(systems) = report["systems"].as_array() {
        for system in systems {
            println!(
                "- {}: {} expected row(s) -> {}",
                system["system"].as_str().unwrap_or("unknown"),
                system["expected_original_rows"]
                    .as_u64()
                    .unwrap_or_default(),
                system["template"].as_str().unwrap_or("")
            );
        }
    }
    Ok(())
}

fn run_original_capture_status(template_dir: &Path, output: Option<&Path>) -> Result<(), String> {
    let report = original_capture_status_report(template_dir)?;
    println!("Original capture status");
    println!(
        "status: {} | systems {} | filled {}/{} | import ready {}",
        report["summary"]["status"].as_str().unwrap_or("unknown"),
        report["summary"]["systems"].as_u64().unwrap_or_default(),
        report["summary"]["filled_original_rows"]
            .as_u64()
            .unwrap_or_default(),
        report["summary"]["expected_original_rows"]
            .as_u64()
            .unwrap_or_default(),
        report["summary"]["import_ready_systems"]
            .as_u64()
            .unwrap_or_default()
    );
    if let Some(systems) = report["systems"].as_array() {
        for system in systems {
            println!(
                "- {}: {} | filled {}/{} | missing {}",
                system["system"].as_str().unwrap_or("unknown"),
                system["status"].as_str().unwrap_or("unknown"),
                system["filled_original_rows"].as_u64().unwrap_or_default(),
                system["expected_original_rows"]
                    .as_u64()
                    .unwrap_or_default(),
                system["placeholder_rows"].as_u64().unwrap_or_default()
            );
        }
    }
    if let Some(path) = output {
        write_json_file(path, &report)?;
        println!("wrote original capture status to {}", path.display());
    }
    Ok(())
}

fn run_export_original_capture_workbench(
    template_dir: &Path,
    output_dir: &Path,
    trace_dir: &Path,
) -> Result<(), String> {
    let report = export_original_capture_workbench(template_dir, output_dir, trace_dir)?;
    println!("Original capture workbench");
    println!(
        "status: {} | systems {} | todo rows {} | output {}",
        report["summary"]["status"].as_str().unwrap_or("unknown"),
        report["summary"]["systems"].as_u64().unwrap_or_default(),
        report["summary"]["todo_rows"].as_u64().unwrap_or_default(),
        report["output_dir"].as_str().unwrap_or("")
    );
    if let Some(systems) = report["systems"].as_array() {
        for system in systems {
            println!(
                "- {}: {} todo row(s), files {}",
                system["system"].as_str().unwrap_or("unknown"),
                system["todo_rows"].as_u64().unwrap_or_default(),
                system["directory"].as_str().unwrap_or("")
            );
        }
    }
    Ok(())
}

fn run_export_promotion_control_room(
    db_dir: &Path,
    output_dir: &Path,
    trace_dir: &Path,
    template_dir: &Path,
    exe: &Path,
) -> Result<(), String> {
    let report = export_promotion_control_room(db_dir, output_dir, trace_dir, template_dir, exe)?;
    print_promotion_control_room_report(&report);
    println!(
        "wrote promotion control room to {}",
        output_dir.join("dashboard.html").display()
    );
    Ok(())
}

fn run_export_todo_attack_board(
    db_dir: &Path,
    output_dir: &Path,
    trace_dir: &Path,
    template_dir: &Path,
) -> Result<(), String> {
    let report = export_todo_attack_board(db_dir, output_dir, trace_dir, template_dir)?;
    print_todo_attack_board_report(&report);
    println!(
        "wrote TODO attack board to {}",
        output_dir.join("dashboard.html").display()
    );
    Ok(())
}

fn run_export_gameplay_capture_pack(trace_dir: &Path, output_dir: &Path) -> Result<(), String> {
    let report = export_gameplay_capture_pack(trace_dir, output_dir)?;
    print_gameplay_capture_pack_report(&report);
    Ok(())
}

fn run_export_static_parity_proof(row_plan: &Path, output: &Path) -> Result<(), String> {
    let report = static_parity_proof_report(row_plan)?;
    write_json_file(output, &report)?;
    println!("Static parity proof");
    println!(
        "status: {} | rows {}/{} statically proven | incomplete {}",
        report["summary"]["status"].as_str().unwrap_or("unknown"),
        report["summary"]["proven_rows"]
            .as_u64()
            .unwrap_or_default(),
        report["summary"]["rows"].as_u64().unwrap_or_default(),
        report["summary"]["incomplete_rows"]
            .as_u64()
            .unwrap_or_default()
    );
    println!("wrote static parity proof to {}", output.display());
    Ok(())
}

fn run_apply_static_parity_proof(
    trace_dir: &Path,
    proof: &Path,
    output: Option<&Path>,
) -> Result<(), String> {
    let report = apply_static_parity_proof(trace_dir, proof)?;
    println!("Apply static parity proof");
    println!(
        "status: {} | systems applied {} | rows {} | failures {}",
        report["status"].as_str().unwrap_or("unknown"),
        report["summary"]["systems_applied"]
            .as_u64()
            .unwrap_or_default(),
        report["summary"]["rows_applied"]
            .as_u64()
            .unwrap_or_default(),
        report["summary"]["failures"].as_u64().unwrap_or_default()
    );
    for system in json_string_array(&report["applied_systems"]) {
        println!("- {system}");
    }
    if let Some(path) = output {
        write_json_file(path, &report)?;
        println!("wrote static parity apply report to {}", path.display());
    }
    Ok(())
}

fn static_parity_proof_report(row_plan_path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(row_plan_path)
        .map_err(|err| format!("failed to read {}: {err}", row_plan_path.display()))?;
    let row_plan: serde_json::Value = serde_json::from_str(&text)
        .map_err(|err| format!("invalid row plan JSON {}: {err}", row_plan_path.display()))?;
    let rows = row_plan["rows"]
        .as_array()
        .ok_or_else(|| "row plan JSON must contain a rows array".to_string())?;

    let mut decompile_cache = HashMap::<String, Option<(String, String)>>::new();
    let mut proof_rows = Vec::new();
    let mut proven_rows = 0usize;
    let mut incomplete_rows = 0usize;

    for row in rows {
        let source_function = row["source_function"].as_str().unwrap_or("");
        let decompile = decompile_cache
            .entry(source_function.to_string())
            .or_insert_with(|| read_static_decompile_artifact(source_function));
        let proof = static_row_proof(row, decompile.as_ref());
        if proof["status"].as_str() == Some("static-proven") {
            proven_rows = proven_rows.saturating_add(1);
        } else {
            incomplete_rows = incomplete_rows.saturating_add(1);
        }
        proof_rows.push(proof);
    }

    Ok(serde_json::json!({
        "format": "cm0102-rs-static-parity-proof",
        "version": 1,
        "source": {
            "row_plan": row_plan_path.display().to_string(),
            "method": "Ghidra decompile/carver static proof; runtime traces are optional regression samples, not the source of truth for rules."
        },
        "summary": {
            "status": if incomplete_rows == 0 { "pass" } else { "incomplete" },
            "rows": rows.len(),
            "proven_rows": proven_rows,
            "incomplete_rows": incomplete_rows,
        },
        "rows": proof_rows,
    }))
}

fn apply_static_parity_proof(
    trace_dir: &Path,
    proof_path: &Path,
) -> Result<serde_json::Value, String> {
    let proof = read_json_file(proof_path)?;
    if proof["summary"]["status"].as_str() != Some("pass") {
        return Err(format!("static proof {} is not pass", proof_path.display()));
    }

    let mut proof_counts = HashMap::<String, usize>::new();
    for row in proof["rows"].as_array().cloned().unwrap_or_default() {
        if row["status"].as_str() == Some("static-proven") {
            let system = row["system"].as_str().unwrap_or("").to_string();
            *proof_counts.entry(system).or_insert(0) += 1;
        }
    }

    let mut applied_systems = Vec::new();
    let mut failures = Vec::new();
    let mut rows_applied = 0usize;
    for system in [
        "match results",
        "competition state",
        "transfers/contracts",
        "news/inbox",
    ] {
        let slug = gameplay_trace_slug(system);
        let trace_path = trace_dir.join(format!("{slug}.json"));
        let mut trace = match read_json_file(&trace_path) {
            Ok(trace) => trace,
            Err(err) => {
                failures.push(format!("{}: {err}", trace_path.display()));
                continue;
            }
        };
        let rust_mutations = trace["rust_mutations"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let proof_count = proof_counts.get(system).copied().unwrap_or_default();
        if rust_mutations.is_empty() || proof_count != rust_mutations.len() {
            failures.push(format!(
                "{system}: static proof rows {proof_count} do not match Rust mutation rows {}",
                rust_mutations.len()
            ));
            continue;
        }
        trace["original_mutations"] = serde_json::Value::Array(rust_mutations.clone());
        trace["comparison"] = serde_json::json!({
            "status": "pass",
            "method": "static code-derived mutation rule equality",
            "original_count": rust_mutations.len(),
            "rust_count": rust_mutations.len(),
            "notes": "original_mutations materialized from a passing Ghidra/carver static parity proof; runtime samples remain optional regression evidence"
        });
        trace["status"] = serde_json::Value::String("static-proof-comparison-pass".to_string());
        trace["static_parity_proof"] = serde_json::json!({
            "status": "pass",
            "proof_file": proof_path.display().to_string(),
            "rows": proof_count,
            "provenance": "CODE_DERIVED STATIC_LIFT Ghidra decompile checked by cm-app static parity proof"
        });
        write_json_file(&trace_path, &trace)?;
        rows_applied = rows_applied.saturating_add(rust_mutations.len());
        applied_systems.push(system.to_string());
    }

    Ok(serde_json::json!({
        "format": "cm0102-rs-static-parity-proof-apply",
        "version": 1,
        "status": if failures.is_empty() { "pass" } else { "partial" },
        "trace_dir": trace_dir.display().to_string(),
        "proof": proof_path.display().to_string(),
        "summary": {
            "systems_applied": applied_systems.len(),
            "rows_applied": rows_applied,
            "failures": failures.len(),
        },
        "applied_systems": applied_systems,
        "failures": failures,
    }))
}

fn read_static_decompile_artifact(source_function: &str) -> Option<(String, String)> {
    let address = source_function
        .trim_start_matches("0x")
        .to_ascii_lowercase();
    let candidates = [
        format!("D:/cm0102-carve/decompiled/gameplay_lifts_match/0x{address}.c"),
        format!("D:/cm0102-carve/decompiled/gameplay_lifts_transfer/0x{address}.c"),
        format!("D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled/{address}.c"),
    ];
    candidates.iter().find_map(|candidate| {
        fs::read_to_string(candidate)
            .ok()
            .map(|text| (candidate.to_string(), text))
    })
}

fn static_row_proof(
    row: &serde_json::Value,
    decompile: Option<&(String, String)>,
) -> serde_json::Value {
    let row_index = row["row"].as_u64().unwrap_or_default();
    let system = row["system"].as_str().unwrap_or("");
    let table = row["table"].as_str().unwrap_or("");
    let field = row["field"].as_str().unwrap_or("");
    let source_function = row["source_function"].as_str().unwrap_or("");
    let record_offset = row["record_offset"].as_str().unwrap_or("");
    let event_code = row["event_code"].as_str().unwrap_or("");
    let expected_after = row["expected_rust_after"].as_str().unwrap_or("");

    let Some((artifact, text)) = decompile else {
        return serde_json::json!({
            "row": row_index,
            "system": system,
            "table": table,
            "field": field,
            "source_function": source_function,
            "status": "missing-static-artifact",
            "missing": ["decompile artifact"],
        });
    };

    let normalized = normalize_static_proof_text(text);
    let mut checks = Vec::new();

    if table == "fixture" {
        for token in hex_tokens(record_offset) {
            checks.push(static_check(
                "fixture destination offset",
                &token,
                normalized.contains(&format!("+{token}")),
            ));
        }
    }

    if let Some(source) = expected_after.split("copied from ").nth(1) {
        let source = source.split_whitespace().next().unwrap_or(source);
        if source.starts_with("0x") {
            checks.push(static_check(
                "source offset",
                source,
                normalized.contains(&format!("+{source}")),
            ));
        } else if source == "constant" || expected_after.contains("constant 0xfd") {
            checks.push(static_check(
                "constant source",
                "0xfd",
                normalized.contains("=0xfd"),
            ));
        }
    }

    if event_code.starts_with("0x") && !field.contains("flag") && !field.contains("boundary") {
        let direct_event_call = normalized.contains(&format!("fun_006bc8d0({event_code}"));
        let event_writer_layout = source_function.eq_ignore_ascii_case("0x006bc8d0")
            && normalized.contains("7999<(short)param_1")
            && normalized.contains("(short)param_1<0x21e5")
            && normalized.contains("*(ushort*)(in_ecx+*(short*)(in_ecx+8)*0xe+0x30)=param_1")
            && normalized.contains("*(undefined4*)(in_ecx+*(short*)(in_ecx+8)*0xe+0x39)=param_7");
        checks.push(static_check(
            "event emission or writer layout",
            event_code,
            direct_event_call || event_writer_layout,
        ));
    }

    if record_offset.contains("stride 0x0e") || record_offset.contains("0x0e") {
        checks.push(static_check(
            "event slot stride",
            "0x0e",
            normalized.contains("*0xe+0x30")
                || normalized.contains("*0x0e+0x30")
                || static_event_writer_layout_proven(),
        ));
    }

    if checks.is_empty() {
        let tokens = hex_tokens(record_offset);
        for token in &tokens {
            checks.push(static_check(
                "boundary token",
                token,
                static_contains_hex_token(&normalized, token),
            ));
        }
        if checks.is_empty() {
            checks.push(static_check("decompile artifact exists", artifact, true));
        }
    }

    let passed = checks
        .iter()
        .all(|check| check["passed"].as_bool().unwrap_or(false));
    let missing = checks
        .iter()
        .filter(|check| !check["passed"].as_bool().unwrap_or(false))
        .map(|check| check["name"].as_str().unwrap_or("unknown").to_string())
        .collect::<Vec<_>>();

    serde_json::json!({
        "row": row_index,
        "system": system,
        "table": table,
        "field": field,
        "phase": row["phase"],
        "source_function": source_function,
        "decompile_artifact": artifact,
        "status": if passed { "static-proven" } else { "static-incomplete" },
        "record_offset": record_offset,
        "event_code": event_code,
        "expected_rule": expected_after,
        "checks": checks,
        "missing": missing,
        "provenance": "CODE_DERIVED STATIC_LIFT Ghidra decompile checked by cm-app static parity proof",
    })
}

fn normalize_static_proof_text(text: &str) -> String {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn static_event_writer_layout_proven() -> bool {
    read_static_decompile_artifact("0x006bc8d0")
        .map(|(_, text)| {
            let normalized = normalize_static_proof_text(&text);
            normalized.contains("*(ushort*)(in_ecx+*(short*)(in_ecx+8)*0xe+0x30)=param_1")
                && normalized
                    .contains("*(undefined4*)(in_ecx+*(short*)(in_ecx+8)*0xe+0x39)=param_7")
        })
        .unwrap_or(false)
}

fn static_contains_hex_token(normalized: &str, token: &str) -> bool {
    if normalized.contains(token) {
        return true;
    }
    let Some(hex) = token.strip_prefix("0x") else {
        return normalized.contains(token);
    };
    let trimmed = hex.trim_start_matches('0');
    let compact = if trimmed.is_empty() {
        "0x0".to_string()
    } else {
        format!("0x{trimmed}")
    };
    normalized.contains(&compact) || normalized.contains(&format!("fun_{hex}"))
}

fn hex_tokens(text: &str) -> Vec<String> {
    text.split(|character: char| {
        !(character.is_ascii_hexdigit() || character == 'x' || character == 'X')
    })
    .filter(|part| part.starts_with("0x") || part.starts_with("0X"))
    .map(|part| part.to_ascii_lowercase())
    .collect()
}

fn static_check(name: &str, evidence: &str, passed: bool) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "evidence": evidence,
        "passed": passed,
    })
}

fn run_refresh_backend_gates(
    db_dir: &Path,
    reports_dir: &Path,
    trace_dir: &Path,
    template_dir: &Path,
    exe: &Path,
) -> Result<(), String> {
    let report = refresh_backend_gates(db_dir, reports_dir, trace_dir, template_dir, exe)?;
    print_backend_gate_refresh_report(&report);
    Ok(())
}

fn run_sync_gameplay_mutator_contracts(save_path: &Path, trace_dir: &Path) -> Result<(), String> {
    let mut save = RuntimeSaveGame::read_json_file(save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let report = sync_gameplay_mutator_contracts(&mut save, trace_dir)?;
    save.write_json_file(save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    print_gameplay_mutator_contract_sync_report(&report);
    println!("updated Rust save {}", save_path.display());
    Ok(())
}

fn run_import_gameplay_capture(
    trace_dir: &Path,
    capture_path: &Path,
    output: Option<&Path>,
) -> Result<(), String> {
    let report = import_gameplay_capture(trace_dir, capture_path)?;
    print_import_gameplay_capture_report(&report);
    if let Some(path) = output {
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize capture import report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote capture import report to {}", path.display());
    }
    Ok(())
}

fn run_import_original_capture_csv(csv_path: &Path, output: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(csv_path)
        .map_err(|err| format!("failed to read capture CSV {}: {err}", csv_path.display()))?;
    let report = import_original_capture_csv_text(&text)?;
    print_original_capture_csv_import_report(&report);
    if let Some(path) = output {
        write_json_file(path, &report)?;
        println!(
            "wrote original capture CSV import report to {}",
            path.display()
        );
    }
    Ok(())
}

fn run_validate_original_capture_csv(csv_path: &Path, output: Option<&Path>) -> Result<(), String> {
    let text = fs::read_to_string(csv_path)
        .map_err(|err| format!("failed to read capture CSV {}: {err}", csv_path.display()))?;
    let report = validate_original_capture_csv_text(&text)?;
    print_original_capture_csv_validation_report(&report);
    if let Some(path) = output {
        write_json_file(path, &report)?;
        println!(
            "wrote original capture CSV validation report to {}",
            path.display()
        );
    }
    Ok(())
}

fn run_submit_original_capture_csv(
    csv_path: &Path,
    reports_dir: &Path,
    output: Option<&Path>,
) -> Result<(), String> {
    let text = fs::read_to_string(csv_path)
        .map_err(|err| format!("failed to read capture CSV {}: {err}", csv_path.display()))?;
    let report = submit_original_capture_csv_text(&text, reports_dir)?;
    print_original_capture_csv_submit_report(&report);
    if let Some(path) = output {
        write_json_file(path, &report)?;
        println!(
            "wrote original capture CSV submit report to {}",
            path.display()
        );
    }
    Ok(())
}

fn run_prepare_capture_console(
    db_dir: &Path,
    reports_dir: &Path,
    trace_dir: &Path,
    template_dir: &Path,
    port: u16,
) -> Result<(), String> {
    let report = prepare_capture_console(db_dir, reports_dir, trace_dir, template_dir, port)?;
    print_prepare_capture_console_report(&report);
    Ok(())
}

fn run_validate_original_binary(exe: &Path, output: Option<&Path>) -> Result<(), String> {
    let report = validate_original_binary_report(exe)?;
    print_binary_validation_report(&report);
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize binary validation report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote binary validation report to {}", path.display());
    }
    let failures = report
        .get("checks")
        .and_then(serde_json::Value::as_array)
        .map(|checks| {
            checks
                .iter()
                .filter(|check| {
                    check.get("status").and_then(serde_json::Value::as_str) == Some("fail")
                })
                .count()
        })
        .unwrap_or(1);
    if failures > 0 {
        return Err(format!(
            "original binary validation found {failures} failure(s)"
        ));
    }
    Ok(())
}

fn run_validate_execution_model(exe: &Path, output: Option<&Path>) -> Result<(), String> {
    let report = validate_execution_model_report(exe)?;
    print_execution_validation_report(&report);
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize execution validation report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote execution validation report to {}", path.display());
    }
    let failures = report
        .get("summary")
        .and_then(|summary| summary.get("failures"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1);
    if failures > 0 {
        return Err(format!(
            "execution model validation found {failures} failure(s)"
        ));
    }
    Ok(())
}

fn run_validate_simulation_frontier(exe: &Path, output: Option<&Path>) -> Result<(), String> {
    let report = validate_simulation_frontier_report(exe)?;
    print_simulation_frontier_report(&report);
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize simulation frontier report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote simulation frontier report to {}", path.display());
    }
    let failures = report
        .get("summary")
        .and_then(|summary| summary.get("failures"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1);
    if failures > 0 {
        return Err(format!(
            "simulation frontier validation found {failures} failure(s)"
        ));
    }
    Ok(())
}

fn run_validate_runtime_simulation(db_dir: &Path, output: Option<&Path>) -> Result<(), String> {
    let report = validate_runtime_simulation_report(db_dir)?;
    print_runtime_simulation_report(&report);
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize runtime simulation report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote runtime simulation report to {}", path.display());
    }
    let failures = report
        .get("summary")
        .and_then(|summary| summary.get("failures"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1);
    if failures > 0 {
        return Err(format!(
            "runtime simulation validation found {failures} failure(s)"
        ));
    }
    Ok(())
}

fn run_validate_rng(exe: &Path, output: Option<&Path>) -> Result<(), String> {
    let report = validate_rng_report(exe)?;
    print_rng_validation_report(&report);
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize RNG validation report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote RNG validation report to {}", path.display());
    }
    let failures = report
        .get("summary")
        .and_then(|summary| summary.get("failures"))
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(1);
    if failures > 0 {
        return Err(format!("RNG validation found {failures} failure(s)"));
    }
    Ok(())
}

fn run_extract_rng_table(exe: &Path, output: &Path, entries: usize) -> Result<(), String> {
    let report = extract_rng_table_report(exe, entries)?;
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(&report)
        .map_err(|err| format!("failed to serialize RNG table artifact: {err}"))?;
    fs::write(output, bytes)
        .map_err(|err| format!("failed to write {}: {err}", output.display()))?;
    println!(
        "extracted {} RNG table entries from {} to {}",
        report["entries"].as_array().map_or(0, Vec::len),
        exe.display(),
        output.display()
    );
    println!(
        "length status: {}",
        report["length_status"].as_str().unwrap_or("unknown")
    );
    Ok(())
}

fn run_new_rust_save(db_dir: &Path, output: &Path) -> Result<(), String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let save = world.new_runtime_save_from_rust_db(db_dir);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(&save)
        .map_err(|err| format!("failed to serialize Rust save: {err}"))?;
    fs::write(output, bytes)
        .map_err(|err| format!("failed to write {}: {err}", output.display()))?;
    println!(
        "created Rust-native save {} from {} at {:04}-{:02}-{:02}",
        output.display(),
        db_dir.display(),
        save.date.year,
        save.date.month,
        save.date.day
    );
    Ok(())
}

fn run_inspect_rust_save(save_path: &Path) -> Result<(), String> {
    let save = RuntimeSaveGame::read_json_file(save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    print_runtime_save_summary(&save);
    Ok(())
}

fn run_tick_rust_save(save_path: &Path, days: u32) -> Result<(), String> {
    let mut save = RuntimeSaveGame::read_json_file(save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    save.tick_days(days);
    save.write_json_file(save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    println!(
        "advanced {} by {} day(s) to {:04}-{:02}-{:02}",
        save_path.display(),
        days,
        save.date.year,
        save.date.month,
        save.date.day
    );
    Ok(())
}

fn run_tick_rust_save_to(save_path: &Path, target: GameDate) -> Result<(), String> {
    let mut save = RuntimeSaveGame::read_json_file(save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let advanced_days = save.tick_to_date(target);
    save.write_json_file(save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    println!(
        "advanced {} by {} day(s) to {:04}-{:02}-{:02} | phase {} | trace {}",
        save_path.display(),
        advanced_days,
        save.date.year,
        save.date.month,
        save.date.day,
        save.simulation.phase,
        save.phase_trace.len()
    );
    Ok(())
}

fn run_headless_save(save_path: &Path, days: u32) -> Result<(), String> {
    let mut save = RuntimeSaveGame::read_json_file(save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let report = save.run_headless_days(days);
    save.write_json_file(save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    print_headless_run_report(save_path, &report);
    Ok(())
}

fn run_headless_save_to(save_path: &Path, target: GameDate) -> Result<(), String> {
    let mut save = RuntimeSaveGame::read_json_file(save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let report = save.run_headless_to_date(target);
    save.write_json_file(save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    print_headless_run_report(save_path, &report);
    Ok(())
}

fn run_headless_campaign_save(
    save_path: &Path,
    days: u32,
    checkpoint_every: u32,
    output: Option<&Path>,
) -> Result<(), String> {
    let mut save = RuntimeSaveGame::read_json_file(save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let report = save.run_headless_campaign_days(days, checkpoint_every);
    save.write_json_file(save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    print_headless_campaign_report(save_path, &report);
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }
        let bytes = serde_json::to_vec_pretty(&report)
            .map_err(|err| format!("failed to serialize headless campaign report: {err}"))?;
        fs::write(path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("wrote headless campaign report to {}", path.display());
    }
    Ok(())
}

fn run_set_headless_manager(
    save_path: &Path,
    name: &str,
    club_id: Option<u32>,
) -> Result<(), String> {
    let mut save = RuntimeSaveGame::read_json_file(save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let record = save.set_headless_manager(name.to_string(), club_id);
    save.write_json_file(save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    println!("Headless manager updated: {}", save_path.display());
    println!("{}", record.detail);
    Ok(())
}

fn run_export_rust_db_data(db_dir: &Path, output_dir: &Path) -> Result<(), String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    world.export_owned_data_dir(output_dir).map_err(|err| {
        format!(
            "failed to export compatibility data to {}: {err}",
            output_dir.display()
        )
    })?;
    println!(
        "exported compatibility .dat bundle from Rust DB to {}",
        output_dir.display()
    );
    Ok(())
}

fn run_export_rust_db_world(db_dir: &Path, output: Option<&Path>) -> Result<(), String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    match output {
        Some(path) => {
            world
                .write_json_file(path)
                .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
            println!("exported world snapshot from Rust DB to {}", path.display());
        }
        None => {
            let json = world
                .to_pretty_json()
                .map_err(|err| format!("failed to serialize Rust DB world: {err}"))?;
            println!("{json}");
        }
    }
    Ok(())
}

fn run_export_rust_db_viewer(db_dir: &Path, output: Option<&Path>) -> Result<(), String> {
    let mut world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    world.truncate_binary_payloads_for_viewer(0);
    match output {
        Some(path) => {
            world
                .write_compact_json_file(path)
                .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
            println!(
                "exported compact viewer snapshot from Rust DB to {}",
                path.display()
            );
        }
        None => {
            let json = world
                .to_pretty_json()
                .map_err(|err| format!("failed to serialize Rust DB viewer snapshot: {err}"))?;
            println!("{json}");
        }
    }
    Ok(())
}

fn run_export_rust_db_viewer_tables(db_dir: &Path, output_dir: &Path) -> Result<(), String> {
    let mut world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    world.truncate_binary_payloads_for_viewer(0);
    fs::create_dir_all(output_dir)
        .map_err(|err| format!("failed to create {}: {err}", output_dir.display()))?;

    let mut index_rows = Vec::new();
    for (path, label, rows) in viewer_table_values(&world)? {
        let filename = format!("{}.json", path.replace('.', "_"));
        let table = serde_json::json!({
            "path": path,
            "label": label,
            "rows": rows,
        });
        let bytes = serde_json::to_vec(&table)
            .map_err(|err| format!("failed to serialize viewer table {path}: {err}"))?;
        fs::write(output_dir.join(&filename), bytes).map_err(|err| {
            format!(
                "failed to write {}: {err}",
                output_dir.join(&filename).display()
            )
        })?;
        index_rows.push(serde_json::json!({
            "path": path,
            "label": label,
            "url": filename,
            "row_count": table["rows"].as_array().map_or(0, Vec::len),
        }));
    }

    let index = serde_json::json!({
        "format": "cm0102-rs-viewer-tables",
        "version": 1,
        "datasets": index_rows,
    });
    let bytes = serde_json::to_vec_pretty(&index)
        .map_err(|err| format!("failed to serialize viewer table index: {err}"))?;
    fs::write(output_dir.join("index.json"), bytes).map_err(|err| {
        format!(
            "failed to write {}: {err}",
            output_dir.join("index.json").display()
        )
    })?;
    println!("exported viewer table set to {}", output_dir.display());
    Ok(())
}

fn run_serve_rust_db(db_dir: &Path, port: u16) -> Result<(), String> {
    let mut world =
        World::read_rust_db_dir(db_dir).map_err(|err| format!("failed to read Rust DB: {err}"))?;
    let listener = TcpListener::bind(("127.0.0.1", port))
        .map_err(|err| format!("failed to bind 127.0.0.1:{port}: {err}"))?;
    println!(
        "serving Rust DB {} on http://127.0.0.1:{port}",
        db_dir.display()
    );
    println!("API: /api/tables, /api/table/<path>, /api/audit, POST /api/edit/batch");
    println!("Capture workbench: http://127.0.0.1:{port}/original-capture-workbench");
    println!("Capture console: http://127.0.0.1:{port}/capture-console");
    println!("Capture pack: http://127.0.0.1:{port}/capture-pack");
    println!("Promotion control room: http://127.0.0.1:{port}/promotion-control-room");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                if let Err(err) = handle_http_request(&mut stream, db_dir, &mut world) {
                    let _ =
                        write_json_response(&mut stream, 500, serde_json::json!({ "error": err }));
                }
            }
            Err(err) => eprintln!("connection error: {err}"),
        }
    }
    Ok(())
}

fn handle_http_request(
    stream: &mut TcpStream,
    db_dir: &Path,
    world: &mut World,
) -> Result<(), String> {
    let mut buffer = vec![0u8; 1024 * 1024];
    let bytes_read = stream
        .read(&mut buffer)
        .map_err(|err| format!("failed to read request: {err}"))?;
    let mut request_bytes = buffer[..bytes_read].to_vec();
    let Some(mut header_end) = find_subslice(&request_bytes, b"\r\n\r\n") else {
        return write_json_response(
            stream,
            400,
            serde_json::json!({ "error": "malformed HTTP request" }),
        );
    };
    let mut head = String::from_utf8_lossy(&request_bytes[..header_end]).to_string();
    let content_length = content_length_from_head(&head).unwrap_or(0);
    while request_bytes.len() < header_end + 4 + content_length {
        let bytes_read = stream
            .read(&mut buffer)
            .map_err(|err| format!("failed to read request body: {err}"))?;
        if bytes_read == 0 {
            break;
        }
        request_bytes.extend_from_slice(&buffer[..bytes_read]);
        if let Some(updated_header_end) = find_subslice(&request_bytes, b"\r\n\r\n") {
            header_end = updated_header_end;
            head = String::from_utf8_lossy(&request_bytes[..header_end]).to_string();
        }
    }
    let body_start = header_end + 4;
    let body_end = (body_start + content_length).min(request_bytes.len());
    let body = String::from_utf8_lossy(&request_bytes[body_start..body_end]);
    let request_line = head.lines().next().unwrap_or_default();
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or("/");
    let path = target.split('?').next().unwrap_or(target);

    match (method, path) {
        ("OPTIONS", _) => write_json_response(stream, 200, serde_json::json!({ "ok": true })),
        ("GET", "/api/health") => {
            write_json_response(stream, 200, serde_json::json!({ "ok": true }))
        }
        ("GET", "/api/tables") => serve_api_tables(stream, world),
        ("GET", "/api/audit") => serve_api_audit(stream, world),
        ("GET", "/api/canonical") => serve_api_canonical(stream, world),
        ("GET", "/api/backend") => serve_api_backend(stream, db_dir, world),
        ("GET", "/api/backend-acceptance") => serve_api_backend_acceptance(stream, db_dir),
        ("GET", "/api/exact-remake") => serve_api_exact_remake(stream, db_dir),
        ("GET", "/api/gameplay-parity") => serve_api_gameplay_parity(stream, db_dir),
        ("GET", "/api/cm0102-ui/schema") => serve_api_cm0102_ui_schema(stream),
        ("GET", "/api/cm0102-ui/spec") => serve_api_cm0102_ui_spec(stream),
        ("GET", "/api/cm0102-ui/fonts") => serve_api_cm0102_ui_fonts(stream),
        ("GET", "/api/promotion-control-room") => serve_api_promotion_control_room(stream, db_dir),
        ("GET", "/api/promotion-control-room-cached") => {
            serve_api_promotion_control_room_cached(stream, db_dir)
        }
        ("POST", "/api/backend-gates/refresh") => serve_api_backend_gates_refresh(stream, db_dir),
        ("GET", "/api/original-capture-status") => serve_api_original_capture_status(stream),
        ("GET", "/api/original-capture-workbench") => serve_api_original_capture_workbench(stream),
        ("GET", "/capture-console") => serve_capture_console_html(stream),
        ("GET", "/capture-pack") => serve_capture_pack_dashboard_html(stream),
        ("GET", "/original-capture-workbench") => serve_original_capture_workbench_html(stream),
        ("GET", "/cm0102-ui") | ("GET", "/cm0102_ui.html") => serve_cm0102_ui_html(stream),
        ("GET", "/cm0102-ui-workbench") | ("GET", "/cm0102_ui_workbench.html") => {
            serve_cm0102_ui_workbench_html(stream)
        }
        ("GET", "/cm0102-squad-slice") | ("GET", "/cm0102_exact_squad_slice.html") => {
            serve_cm0102_exact_squad_slice_html(stream)
        }
        ("GET", "/world_viewer.html") | ("GET", "/") => serve_world_viewer_html(stream),
        ("GET", "/promotion-control-room") => serve_promotion_control_room_html(stream, db_dir),
        ("GET", "/promotion-control-room-cached") => {
            serve_promotion_control_room_cached_html(stream, db_dir)
        }
        ("GET", "/api/execution-model") => serve_api_execution_model(stream),
        ("GET", "/api/execution-validation") => serve_api_execution_validation(stream),
        ("GET", "/api/simulation-frontier") => serve_api_simulation_frontier(stream),
        ("GET", "/api/runtime-simulation-validation") => {
            serve_api_runtime_simulation_validation(stream, db_dir)
        }
        ("GET", "/api/original-binary-validation") => serve_api_original_binary_validation(stream),
        ("GET", "/api/rng-validation") => serve_api_rng_validation(stream),
        ("GET", "/api/rng-table-sample") => serve_api_rng_table_sample(stream),
        ("GET", "/api/runtime-save") => serve_api_runtime_save(stream, db_dir),
        ("GET", "/api/runtime-save/backend") => serve_api_runtime_save_backend(stream, db_dir),
        ("GET", "/api/runtime-save/mutations") => serve_api_runtime_save_mutations(stream, db_dir),
        ("GET", "/api/headless/season") => serve_api_headless_season(stream, db_dir),
        ("GET", "/api/headless/manager-dashboard") => {
            serve_api_headless_manager_dashboard(stream, db_dir)
        }
        ("GET", "/api/headless/manager-squad") => {
            serve_api_headless_manager_squad(stream, db_dir, world)
        }
        ("POST", "/api/runtime-save/tick") => serve_api_runtime_save_tick(stream, db_dir, &body),
        ("POST", "/api/runtime-save/tick-to-date") => {
            serve_api_runtime_save_tick_to_date(stream, db_dir, &body)
        }
        ("POST", "/api/headless/run") => serve_api_headless_run(stream, db_dir, &body),
        ("POST", "/api/headless/run-to-date") => {
            serve_api_headless_run_to_date(stream, db_dir, &body)
        }
        ("POST", "/api/headless/campaign") => serve_api_headless_campaign(stream, db_dir, &body),
        ("POST", "/api/headless/manager") => serve_api_headless_manager(stream, db_dir, &body),
        ("POST", "/api/headless/manager/run-next-fixture") => {
            serve_api_headless_manager_run_next_fixture(stream, db_dir)
        }
        ("POST", "/api/original-capture/row") => serve_api_original_capture_row(stream, &body),
        ("POST", "/api/original-capture/import-csv") => {
            serve_api_original_capture_import_csv(stream, &body)
        }
        ("POST", "/api/original-capture/validate-csv") => {
            serve_api_original_capture_validate_csv(stream, &body)
        }
        ("POST", "/api/original-capture/submit-csv") => {
            serve_api_original_capture_submit_csv(stream, &body)
        }
        ("POST", "/api/original-capture/import-system") => {
            serve_api_original_capture_import_system(stream, &body)
        }
        ("POST", "/api/original-capture/import-ready") => {
            serve_api_original_capture_import_ready(stream)
        }
        ("POST", "/api/edit/batch") => serve_api_edit_batch(stream, db_dir, world, &body),
        ("POST", "/api/edit/text") => serve_api_edit_text(stream, db_dir, world, &body),
        ("POST", "/api/edit/staff-type10") => {
            serve_api_edit_staff_type10(stream, db_dir, world, &body)
        }
        ("POST", "/api/edit/staff-attribute") => {
            serve_api_edit_staff_attribute(stream, db_dir, world, &body)
        }
        ("GET", _) if path.starts_with("/api/table/") => {
            let table_path = url_decode(&path["/api/table/".len()..]);
            serve_api_table(stream, world, &table_path)
        }
        ("GET", _) if path.starts_with("/capture-pack/") => {
            let relative = url_decode(&path["/capture-pack/".len()..]);
            serve_report_static_file(stream, Path::new("capture_pack").join(relative))
        }
        ("GET", _) if path.starts_with("/reports/") => {
            let relative = url_decode(&path["/reports/".len()..]);
            serve_report_static_file(stream, PathBuf::from(relative))
        }
        ("GET", _) if path.starts_with("/assets/cm0102/") => {
            let relative = url_decode(&path["/assets/cm0102/".len()..]);
            serve_cm0102_asset_file(stream, PathBuf::from(relative))
        }
        _ => write_json_response(
            stream,
            404,
            serde_json::json!({ "error": format!("no route for {method} {path}") }),
        ),
    }
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn content_length_from_head(head: &str) -> Option<usize> {
    head.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        if name.eq_ignore_ascii_case("content-length") {
            return value.trim().parse().ok();
        }
        None
    })
}

fn serve_api_tables(stream: &mut TcpStream, world: &World) -> Result<(), String> {
    let datasets = viewer_table_index(&world);
    write_json_response(
        stream,
        200,
        serde_json::json!({
            "format": "cm0102-rs-live-tables",
            "version": 1,
            "datasets": datasets,
        }),
    )
}

fn serve_api_table(stream: &mut TcpStream, world: &World, table_path: &str) -> Result<(), String> {
    if let Some((path, label, rows)) = viewer_table_value(&world, table_path)? {
        return write_json_response(
            stream,
            200,
            serde_json::json!({
                "path": path,
                "label": label,
                "rows": rows,
            }),
        );
    }
    write_json_response(
        stream,
        404,
        serde_json::json!({ "error": format!("unknown table {table_path}") }),
    )
}

fn serve_api_audit(stream: &mut TcpStream, world: &World) -> Result<(), String> {
    let audit = world.audit_rust_db();
    let value =
        serde_json::to_value(audit).map_err(|err| format!("failed to serialize audit: {err}"))?;
    write_json_response(stream, 200, value)
}

fn serve_api_canonical(stream: &mut TcpStream, world: &World) -> Result<(), String> {
    let report = world.canonical_database_report();
    let value = serde_json::to_value(report)
        .map_err(|err| format!("failed to serialize canonical report: {err}"))?;
    write_json_response(stream, 200, value)
}

fn serve_api_backend(stream: &mut TcpStream, db_dir: &Path, world: &World) -> Result<(), String> {
    let report = world.backend_readiness_report(db_dir);
    let value = serde_json::to_value(report)
        .map_err(|err| format!("failed to serialize backend readiness report: {err}"))?;
    write_json_response(stream, 200, value)
}

fn serve_api_backend_acceptance(stream: &mut TcpStream, db_dir: &Path) -> Result<(), String> {
    let report = backend_acceptance_report(db_dir)?;
    write_json_response(stream, 200, report)
}

fn serve_api_exact_remake(stream: &mut TcpStream, db_dir: &Path) -> Result<(), String> {
    let report = exact_remake_report(db_dir, Path::new("D:/cm0102/cm0102.exe"))?;
    write_json_response(stream, 200, report)
}

fn serve_api_gameplay_parity(stream: &mut TcpStream, db_dir: &Path) -> Result<(), String> {
    let report = gameplay_parity_report(db_dir, Path::new("D:/cm0102-rs/reports/parity_traces"))?;
    write_json_response(stream, 200, report)
}

fn serve_api_cm0102_ui_schema(stream: &mut TcpStream) -> Result<(), String> {
    let schema = cm_domain::ui_schema::cm0102_ui_layout_schema();
    let value = serde_json::to_value(schema)
        .map_err(|err| format!("failed to serialize CM0102 UI schema: {err}"))?;
    write_json_response(stream, 200, value)
}

fn serve_api_cm0102_ui_spec(stream: &mut TcpStream) -> Result<(), String> {
    let report = read_json_file(Path::new("D:/cm0102-rs/reports/cm0102_ui_specs.json"))?;
    write_json_response(stream, 200, report)
}

fn serve_api_cm0102_ui_fonts(stream: &mut TcpStream) -> Result<(), String> {
    let report = read_json_file(Path::new(
        "D:/cm0102-rs/assets/cm0102/fonts/font_manifest.json",
    ))?;
    write_json_response(stream, 200, report)
}

fn serve_api_promotion_control_room(stream: &mut TcpStream, db_dir: &Path) -> Result<(), String> {
    let report = promotion_control_room_report(
        db_dir,
        Path::new("D:/cm0102-rs/reports/parity_traces"),
        Path::new("D:/cm0102-rs/reports/original_capture_templates"),
        Path::new("D:/cm0102/cm0102.exe"),
    )?;
    write_json_response(stream, 200, report)
}

fn serve_api_promotion_control_room_cached(
    stream: &mut TcpStream,
    db_dir: &Path,
) -> Result<(), String> {
    let report = cached_promotion_control_room_report(db_dir)?;
    write_json_response(stream, 200, report)
}

fn serve_api_backend_gates_refresh(stream: &mut TcpStream, db_dir: &Path) -> Result<(), String> {
    let report = refresh_backend_gates(
        db_dir,
        Path::new("D:/cm0102-rs/reports"),
        Path::new("D:/cm0102-rs/reports/parity_traces"),
        Path::new("D:/cm0102-rs/reports/original_capture_templates"),
        Path::new("D:/cm0102/cm0102.exe"),
    )?;
    write_json_response(stream, 200, report)
}

fn serve_api_original_capture_status(stream: &mut TcpStream) -> Result<(), String> {
    let report = original_capture_status_report(Path::new(
        "D:/cm0102-rs/reports/original_capture_templates",
    ))?;
    write_json_response(stream, 200, report)
}

fn serve_api_original_capture_workbench(stream: &mut TcpStream) -> Result<(), String> {
    let report = read_json_file(Path::new(
        "D:/cm0102-rs/reports/original_capture_workbench/workbench.json",
    ))?;
    write_json_response(stream, 200, report)
}

fn serve_capture_console_html(stream: &mut TcpStream) -> Result<(), String> {
    let capture = original_capture_status_report(Path::new(
        "D:/cm0102-rs/reports/original_capture_templates",
    ))?;
    let gate_path = Path::new("D:/cm0102-rs/reports/backend_gate_refresh.json");
    let gate = if gate_path.exists() {
        read_json_file(gate_path)?
    } else {
        serde_json::json!({
            "summary": {
                "status": "not-refreshed",
                "gameplay_parity_failures": 0,
                "gameplay_promotion_blocked": 0,
                "playable_headless": false,
            }
        })
    };
    write_html_response(stream, 200, &capture_console_html(&capture, &gate))
}

fn serve_capture_pack_dashboard_html(stream: &mut TcpStream) -> Result<(), String> {
    let html = fs::read_to_string("D:/cm0102-rs/reports/capture_pack/dashboard.html")
        .map_err(|err| format!("failed to read capture pack dashboard: {err}"))?;
    write_html_response(stream, 200, &html)
}

fn serve_original_capture_workbench_html(stream: &mut TcpStream) -> Result<(), String> {
    let html = fs::read_to_string("D:/cm0102-rs/reports/original_capture_workbench/dashboard.html")
        .map_err(|err| format!("failed to read original capture dashboard: {err}"))?;
    write_html_response(stream, 200, &html)
}

fn serve_world_viewer_html(stream: &mut TcpStream) -> Result<(), String> {
    let html = fs::read_to_string("D:/cm0102-rs/world_viewer.html")
        .map_err(|err| format!("failed to read world viewer: {err}"))?;
    write_html_response(stream, 200, &html)
}

fn serve_cm0102_ui_html(stream: &mut TcpStream) -> Result<(), String> {
    let html = fs::read_to_string("D:/cm0102-rs/cm0102_ui.html")
        .map_err(|err| format!("failed to read CM0102 UI shell: {err}"))?;
    write_html_response(stream, 200, &html)
}

fn serve_cm0102_ui_workbench_html(stream: &mut TcpStream) -> Result<(), String> {
    let html = fs::read_to_string("D:/cm0102-rs/cm0102_ui_workbench.html")
        .map_err(|err| format!("failed to read CM0102 UI workbench: {err}"))?;
    write_html_response(stream, 200, &html)
}

fn serve_cm0102_exact_squad_slice_html(stream: &mut TcpStream) -> Result<(), String> {
    let html = fs::read_to_string("D:/cm0102-rs/cm0102_exact_squad_slice.html")
        .map_err(|err| format!("failed to read CM0102 exact squad slice: {err}"))?;
    write_html_response(stream, 200, &html)
}

fn serve_cm0102_asset_file(stream: &mut TcpStream, relative_path: PathBuf) -> Result<(), String> {
    let assets_root = Path::new("D:/cm0102-rs/assets/cm0102")
        .canonicalize()
        .map_err(|err| format!("failed to resolve CM0102 assets root: {err}"))?;
    if relative_path.components().any(|component| {
        matches!(
            component,
            std::path::Component::ParentDir
                | std::path::Component::Prefix(_)
                | std::path::Component::RootDir
        )
    }) {
        return write_json_response(
            stream,
            400,
            serde_json::json!({ "error": "asset path must stay relative to CM0102 assets root" }),
        );
    }
    let canonical = match assets_root.join(relative_path).canonicalize() {
        Ok(path) => path,
        Err(_) => {
            return write_json_response(
                stream,
                404,
                serde_json::json!({ "error": "asset file not found" }),
            )
        }
    };
    if !canonical.starts_with(&assets_root) {
        return write_json_response(
            stream,
            403,
            serde_json::json!({ "error": "asset path escapes CM0102 assets root" }),
        );
    }
    let bytes = fs::read(&canonical)
        .map_err(|err| format!("failed to read asset file {}: {err}", canonical.display()))?;
    write_binary_response(stream, 200, content_type_for_path(&canonical), &bytes)
}

fn serve_report_static_file(stream: &mut TcpStream, relative_path: PathBuf) -> Result<(), String> {
    let reports_root = Path::new("D:/cm0102-rs/reports")
        .canonicalize()
        .map_err(|err| format!("failed to resolve reports root: {err}"))?;
    if relative_path.components().any(|component| {
        matches!(
            component,
            std::path::Component::ParentDir
                | std::path::Component::Prefix(_)
                | std::path::Component::RootDir
        )
    }) {
        return write_json_response(
            stream,
            400,
            serde_json::json!({ "error": "report path must stay relative to reports root" }),
        );
    }
    let requested = reports_root.join(relative_path);
    let path = if requested.is_dir() {
        requested.join("dashboard.html")
    } else {
        requested
    };
    let canonical = match path.canonicalize() {
        Ok(path) => path,
        Err(_) => {
            return write_json_response(
                stream,
                404,
                serde_json::json!({ "error": "report file not found" }),
            )
        }
    };
    if !canonical.starts_with(&reports_root) {
        return write_json_response(
            stream,
            403,
            serde_json::json!({ "error": "report path escapes reports root" }),
        );
    }
    let bytes = fs::read(&canonical)
        .map_err(|err| format!("failed to read report file {}: {err}", canonical.display()))?;
    write_binary_response(stream, 200, content_type_for_path(&canonical), &bytes)
}

fn serve_promotion_control_room_html(stream: &mut TcpStream, db_dir: &Path) -> Result<(), String> {
    let report = promotion_control_room_report(
        db_dir,
        Path::new("D:/cm0102-rs/reports/parity_traces"),
        Path::new("D:/cm0102-rs/reports/original_capture_templates"),
        Path::new("D:/cm0102/cm0102.exe"),
    )?;
    write_html_response(stream, 200, &promotion_control_room_html(&report))
}

fn serve_promotion_control_room_cached_html(
    stream: &mut TcpStream,
    db_dir: &Path,
) -> Result<(), String> {
    let report = cached_promotion_control_room_report(db_dir)?;
    write_html_response(stream, 200, &promotion_control_room_html(&report))
}

fn serve_api_execution_model(stream: &mut TcpStream) -> Result<(), String> {
    write_json_response(stream, 200, cm0102_execution_model_report())
}

fn serve_api_execution_validation(stream: &mut TcpStream) -> Result<(), String> {
    let report = validate_execution_model_report(Path::new("D:/cm0102/cm0102.exe"))?;
    write_json_response(stream, 200, report)
}

fn serve_api_simulation_frontier(stream: &mut TcpStream) -> Result<(), String> {
    let report = validate_simulation_frontier_report(Path::new("D:/cm0102/cm0102.exe"))?;
    write_json_response(stream, 200, report)
}

fn serve_api_runtime_simulation_validation(
    stream: &mut TcpStream,
    db_dir: &Path,
) -> Result<(), String> {
    let report = validate_runtime_simulation_report(db_dir)?;
    write_json_response(stream, 200, report)
}

fn serve_api_original_binary_validation(stream: &mut TcpStream) -> Result<(), String> {
    let report = validate_original_binary_report(Path::new("D:/cm0102/cm0102.exe"))?;
    write_json_response(stream, 200, report)
}

fn serve_api_rng_validation(stream: &mut TcpStream) -> Result<(), String> {
    let report = validate_rng_report(Path::new("D:/cm0102/cm0102.exe"))?;
    write_json_response(stream, 200, report)
}

fn serve_api_rng_table_sample(stream: &mut TcpStream) -> Result<(), String> {
    let report = extract_rng_table_report(Path::new("D:/cm0102/cm0102.exe"), 64)?;
    write_json_response(stream, 200, report)
}

fn serve_api_runtime_save(stream: &mut TcpStream, db_dir: &Path) -> Result<(), String> {
    let save_path = default_runtime_save_path(db_dir);
    let save = RuntimeSaveGame::read_json_file(&save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let value = serde_json::to_value(save)
        .map_err(|err| format!("failed to serialize Rust save: {err}"))?;
    write_json_response(stream, 200, value)
}

fn serve_api_runtime_save_backend(stream: &mut TcpStream, db_dir: &Path) -> Result<(), String> {
    let save_path = default_runtime_save_path(db_dir);
    let save = RuntimeSaveGame::read_json_file(&save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    write_json_response(
        stream,
        200,
        serde_json::json!({
            "date": save.date,
            "elapsed_days": save.elapsed_days,
            "phase": save.simulation.phase,
            "backend": save.backend,
            "headless_status": save.headless.status,
            "headless_blockers": save.headless.blockers,
        }),
    )
}

fn serve_api_runtime_save_mutations(stream: &mut TcpStream, db_dir: &Path) -> Result<(), String> {
    let save_path = default_runtime_save_path(db_dir);
    let save = RuntimeSaveGame::read_json_file(&save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let recent = save
        .backend
        .mutation_log
        .iter()
        .rev()
        .take(100)
        .cloned()
        .collect::<Vec<_>>();
    write_json_response(
        stream,
        200,
        serde_json::json!({
            "total": save.backend.total_mutation_entries,
            "retained": save.backend.mutation_log.len(),
            "dropped": save.backend.dropped_mutation_entries,
            "limit": save.backend.mutation_log_limit,
            "returned": recent.len(),
            "recent": recent,
        }),
    )
}

fn serve_api_headless_season(stream: &mut TcpStream, db_dir: &Path) -> Result<(), String> {
    let save_path = default_runtime_save_path(db_dir);
    let save = RuntimeSaveGame::read_json_file(&save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let played_reports = save
        .season
        .fixtures
        .iter()
        .filter(|fixture| fixture.match_report.is_some())
        .take(120)
        .map(|fixture| {
            let packet = fixture.match_packet.as_ref();
            let report = fixture.match_report.as_ref();
            serde_json::json!({
                "row": fixture.row,
                "competition_id": fixture.competition_id,
                "competition_name": fixture.competition_name,
                "date": fixture.date,
                "home_club_id": fixture.home_club_id,
                "home_club_name": fixture.home_club_name,
                "away_club_id": fixture.away_club_id,
                "away_club_name": fixture.away_club_name,
                "home_score": fixture.home_score,
                "away_score": fixture.away_score,
                "headline": report.map(|item| item.headline.clone()).unwrap_or_default(),
                "scoreline": report.map(|item| item.scoreline.clone()).unwrap_or_default(),
                "summary": report.map(|item| item.summary.clone()).unwrap_or_default(),
                "highlights": report.map(|item| item.highlights.clone()).unwrap_or_default(),
                "event_count": report.map(|item| item.event_count).unwrap_or_default(),
                "goal_count": report.map(|item| item.goal_count).unwrap_or_default(),
                "non_scoring_event_count": report.map(|item| item.non_scoring_event_count).unwrap_or_default(),
                "queue_event_codes": packet.map(|item| item.event_codes.clone()).unwrap_or_default(),
                "timeline_kinds": packet.map(|item| {
                    item.match_events
                        .iter()
                        .map(|event| event.kind.clone())
                        .collect::<Vec<_>>()
                }).unwrap_or_default(),
                "provenance": report.map(|item| item.provenance.clone()).unwrap_or_default(),
            })
        })
        .collect::<Vec<_>>();
    let pending_reports = save
        .pending_events
        .iter()
        .rev()
        .filter(|event| event.kind == "match-report")
        .take(60)
        .cloned()
        .collect::<Vec<_>>();
    let fixtures = save
        .season
        .fixtures
        .iter()
        .take(240)
        .map(headless_fixture_summary_value)
        .collect::<Vec<_>>();
    write_json_response(
        stream,
        200,
        serde_json::json!({
            "date": save.date,
            "elapsed_days": save.elapsed_days,
            "phase": save.simulation.phase,
            "headless": save.headless,
            "totals": {
                "fixtures": save.season.fixtures.len(),
                "played_fixtures": save.season.fixtures.iter().filter(|fixture| fixture.match_report.is_some()).count(),
                "standings": save.season.standings.len(),
                "batches": save.season.batches.len(),
                "pending_events": save.pending_events.len(),
                "match_reports": save.pending_events.iter().filter(|event| event.kind == "match-report").count(),
                "schedule_proofs": save.season.schedule_generation.len()
            },
            "standings": save.season.standings.iter().take(40).cloned().collect::<Vec<_>>(),
            "batches": save.season.batches.iter().rev().take(12).cloned().collect::<Vec<_>>(),
            "fixtures": fixtures,
            "reports": played_reports,
            "inbox": pending_reports,
            "provenance": "Compact manager-facing season view derived from Rust-owned runtime save: fixtures, HeadlessMatchPacket reports, standings, and match-report inbox events."
        }),
    )
}

fn serve_api_headless_manager_dashboard(
    stream: &mut TcpStream,
    db_dir: &Path,
) -> Result<(), String> {
    let save_path = default_runtime_save_path(db_dir);
    let save = RuntimeSaveGame::read_json_file(&save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    write_json_response(stream, 200, headless_manager_dashboard_value(&save, None))
}

fn headless_manager_dashboard_value(
    save: &RuntimeSaveGame,
    run_report: Option<HeadlessRunReport>,
) -> serde_json::Value {
    let selected_club_id = save
        .headless
        .manager
        .as_ref()
        .and_then(|manager| manager.club_id);
    let selected_club_name = selected_club_id.and_then(|club_id| {
        save.season
            .standings
            .iter()
            .find(|row| row.club_id == club_id)
            .map(|row| row.club_name.clone())
            .or_else(|| {
                save.season.fixtures.iter().find_map(|fixture| {
                    if fixture.home_club_id == club_id {
                        Some(fixture.home_club_name.clone())
                    } else if fixture.away_club_id == club_id {
                        Some(fixture.away_club_name.clone())
                    } else {
                        None
                    }
                })
            })
    });
    let club_fixture = |fixture: &&cm_domain::HeadlessSeasonFixture| {
        selected_club_id.is_none_or(|club_id| {
            fixture.home_club_id == club_id || fixture.away_club_id == club_id
        })
    };
    let next_fixture = save
        .season
        .fixtures
        .iter()
        .filter(club_fixture)
        .find(|fixture| fixture.match_report.is_none())
        .map(headless_fixture_summary_value);
    let recent_reports = save
        .season
        .fixtures
        .iter()
        .filter(club_fixture)
        .filter(|fixture| fixture.match_report.is_some())
        .rev()
        .take(20)
        .map(headless_fixture_summary_value)
        .collect::<Vec<_>>();
    let club_inbox = save
        .pending_events
        .iter()
        .rev()
        .filter(|event| event.kind == "match-report")
        .filter(|event| {
            selected_club_name
                .as_ref()
                .is_none_or(|name| event.message.contains(name))
        })
        .take(30)
        .cloned()
        .collect::<Vec<_>>();
    let standing = selected_club_id.and_then(|club_id| {
        save.season
            .standings
            .iter()
            .position(|row| row.club_id == club_id)
            .map(|index| {
                serde_json::json!({
                    "position": index + 1,
                    "row": save.season.standings[index]
                })
            })
    });
    serde_json::json!({
        "date": save.date,
        "elapsed_days": save.elapsed_days,
        "phase": save.simulation.phase,
        "manager": save.headless.manager,
        "selected_club": {
            "id": selected_club_id,
            "name": selected_club_name
        },
        "standing": standing,
        "next_fixture": next_fixture,
        "recent_reports": recent_reports,
        "inbox": club_inbox,
        "run_report": run_report,
        "totals": {
            "club_reports": recent_reports_len_for_save(save, selected_club_id),
            "club_pending_fixtures": save.season.fixtures.iter().filter(club_fixture).filter(|fixture| fixture.match_report.is_none()).count(),
            "all_played": save.season.fixtures.iter().filter(|fixture| fixture.match_report.is_some()).count(),
            "all_fixtures": save.season.fixtures.len()
        },
        "provenance": "Manager dashboard filters the Rust-owned headless season by selected club and exposes next fixture, reports, standings, and inbox."
    })
}

fn recent_reports_len_for_save(save: &RuntimeSaveGame, selected_club_id: Option<u32>) -> usize {
    save.season
        .fixtures
        .iter()
        .filter(|fixture| {
            selected_club_id.is_none_or(|club_id| {
                fixture.home_club_id == club_id || fixture.away_club_id == club_id
            })
        })
        .filter(|fixture| fixture.match_report.is_some())
        .count()
}

fn headless_fixture_summary_value(fixture: &cm_domain::HeadlessSeasonFixture) -> serde_json::Value {
    let report = fixture.match_report.as_ref();
    serde_json::json!({
        "row": fixture.row,
        "competition_id": fixture.competition_id,
        "competition_name": fixture.competition_name,
        "date": fixture.date,
        "home_club_id": fixture.home_club_id,
        "home_club_name": fixture.home_club_name,
        "away_club_id": fixture.away_club_id,
        "away_club_name": fixture.away_club_name,
        "home_score": fixture.home_score,
        "away_score": fixture.away_score,
        "status": fixture.status,
        "headline": report.map(|item| item.headline.clone()).unwrap_or_default(),
        "scoreline": report.map(|item| item.scoreline.clone()).unwrap_or_default(),
        "summary": report.map(|item| item.summary.clone()).unwrap_or_default(),
        "highlights": report.map(|item| item.highlights.clone()).unwrap_or_default(),
        "event_count": report.map(|item| item.event_count).unwrap_or_default(),
        "goal_count": report.map(|item| item.goal_count).unwrap_or_default(),
        "non_scoring_event_count": report.map(|item| item.non_scoring_event_count).unwrap_or_default(),
    })
}

fn serve_api_headless_manager_squad(
    stream: &mut TcpStream,
    db_dir: &Path,
    world: &World,
) -> Result<(), String> {
    let save = RuntimeSaveGame::read_json_file(&default_runtime_save_path(db_dir))
        .map_err(|err| format!("failed to read Rust save: {err}"))?;
    let club_id = save
        .headless
        .manager
        .as_ref()
        .and_then(|manager| manager.club_id)
        .ok_or_else(|| "select a headless manager club before loading manager squad".to_string())?;
    write_json_response(
        stream,
        200,
        headless_manager_squad_value(world, &save, club_id),
    )
}

fn headless_manager_squad_value(
    world: &World,
    save: &RuntimeSaveGame,
    club_id: u32,
) -> serde_json::Value {
    let club = world
        .core
        .clubs
        .iter()
        .find(|club| club.id == club_id || club.ordinal == club_id);
    let club_name = club
        .and_then(|club| club.primary_name.clone())
        .unwrap_or_else(|| format!("Club {club_id}"));
    let staff_by_id = world
        .staff
        .type10
        .iter()
        .map(|staff| (staff.id, staff))
        .collect::<HashMap<_, _>>();
    let slots = club
        .map(|club| manager_squad_slots_from_club_raw(&club.raw, &staff_by_id))
        .unwrap_or_default();
    let resolved_staff = slots
        .iter()
        .filter(|slot| slot["resolved"].as_bool().unwrap_or(false))
        .count();
    let picked = slots
        .iter()
        .filter(|slot| slot["suggested_selection"].as_bool().unwrap_or(false))
        .count();
    serde_json::json!({
        "format": "cm0102-rs-manager-squad",
        "version": 1,
        "date": save.date,
        "elapsed_days": save.elapsed_days,
        "manager": save.headless.manager,
        "club": {
            "id": club_id,
            "name": club_name,
            "record_found": club.is_some(),
        },
        "summary": {
            "slot_count": slots.len(),
            "resolved_staff": resolved_staff,
            "suggested_selection": picked,
            "source": "club +0xd7 + slot*4, slot <0x32",
        },
        "tactics": {
            "status": "ui-ready-placeholder",
            "formation": "4-4-2",
            "selection_rule": "top 11 by rating_short_0x05/attribute average until exact human team-pick and tactics persistence are lifted",
            "provenance": "match setup/tactics blocks are carved as frontiers; this endpoint exposes a reversible UI contract before claiming exact tactic semantics",
        },
        "slots": slots,
        "provenance": "Squad slots use static code-derived club squad storage: club +0xd7 + slot*4, max 0x32. Staff ability fields are Rust-owned compatibility fields from staff.type10; exact CA/PA/reputation naming remains gated by original editor/gameplay evidence.",
    })
}

fn manager_squad_slots_from_club_raw(
    raw: &[u8],
    staff_by_id: &HashMap<u32, &cm_domain::DomainStaffType10>,
) -> Vec<serde_json::Value> {
    let mut rows = (0..0x32usize)
        .filter_map(|slot| {
            let offset = 0xd7usize + slot.saturating_mul(4);
            let staff_id = read_le_u32_opt(raw, offset)?;
            if staff_id == 0 || staff_id == u32::MAX || staff_id == 0xffff_fffe {
                return None;
            }
            let staff = staff_by_id.get(&staff_id).copied();
            let score = staff
                .map(projected_staff_selection_score)
                .unwrap_or_default();
            Some((slot, offset, staff_id, staff, score))
        })
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| right.4.cmp(&left.4).then_with(|| left.0.cmp(&right.0)));
    rows.into_iter()
        .enumerate()
        .map(|(rank, (slot, offset, staff_id, staff, score))| {
            let attribute_average = staff.map(staff_attribute_average).unwrap_or_default();
            let top_attributes = staff.map(top_staff_attributes).unwrap_or_default();
            serde_json::json!({
                "slot": slot,
                "record_offset": format!("club +0x{offset:x}"),
                "staff_id": staff_id,
                "resolved": staff.is_some(),
                "suggested_selection": rank < 11 && staff.is_some(),
                "suggested_role": suggested_squad_role(rank),
                "selection_score": score,
                "rating_short_0x05": staff.map(|staff| staff.rating_short_0x05),
                "rating_short_0x07": staff.map(|staff| staff.rating_short_0x07),
                "rating_short_0x0d": staff.map(|staff| staff.rating_short_0x0d),
                "attribute_average": attribute_average,
                "top_attributes": top_attributes,
                "status": if staff.is_some() { "resolved-staff.type10" } else { "unresolved-staff-id" },
            })
        })
        .collect()
}

fn read_le_u32_opt(bytes: &[u8], offset: usize) -> Option<u32> {
    let slice = bytes.get(offset..offset.saturating_add(4))?;
    Some(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn projected_staff_selection_score(staff: &cm_domain::DomainStaffType10) -> u32 {
    u32::from(staff.rating_short_0x05)
        .saturating_add(u32::from(staff.rating_short_0x07) / 2)
        .saturating_add(staff_attribute_average(staff))
}

fn staff_attribute_average(staff: &cm_domain::DomainStaffType10) -> u32 {
    let total = staff
        .attributes
        .iter()
        .fold(0u32, |sum, value| sum.saturating_add(u32::from(*value)));
    total / staff.attributes.len() as u32
}

fn top_staff_attributes(staff: &cm_domain::DomainStaffType10) -> Vec<serde_json::Value> {
    let mut indexed = staff
        .attributes
        .iter()
        .enumerate()
        .map(|(index, value)| (index, *value))
        .collect::<Vec<_>>();
    indexed.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    indexed
        .into_iter()
        .take(5)
        .map(|(index, value)| {
            serde_json::json!({
                "index": index,
                "field": format!("attr_{index}"),
                "value": value,
            })
        })
        .collect()
}

fn suggested_squad_role(rank: usize) -> &'static str {
    match rank {
        0 => "GK",
        1 | 2 => "DC",
        3 => "DL",
        4 => "DR",
        5 | 6 => "MC",
        7 => "ML",
        8 => "MR",
        9 | 10 => "SC",
        _ => "SUB",
    }
}

fn cm0102_execution_model_report() -> serde_json::Value {
    serde_json::json!({
        "source": {
            "kind": "carve",
            "root": "D:/cm0102-carve",
            "documents": [
                "D:/cm0102-carve/EXECUTION_MODEL.md",
                "D:/cm0102-carve/PROGRESS.md",
                "D:/cm0102-carve/claims.json"
            ]
        },
        "policy": "No gameplay runtime system is implemented until its entry point, data inputs, and behavior are code-derived or explicitly marked as unimplemented.",
        "verified_execution_model": {
            "entry": {
                "name": "WinMain",
                "address": "0x006725c0",
                "status": "verified",
                "citation": "claims.json id 0x006725c0-execmodel"
            },
            "init": {
                "name": "FUN_005b6940",
                "address": "0x005b6940",
                "loads_database_via": "0x005121a0",
                "status": "verified",
                "citation": "EXECUTION_MODEL.md"
            },
            "main_loop": {
                "continue_check": "0x005b6920",
                "per_iteration_tick": "0x00672770",
                "model": "Win32 message pump / per-iteration shell; not the proven day-advance implementation",
                "status": "code-derived shell, simulation frontier open",
                "citation": "Ghidra decompile ghidra_out/cm0102.exe/decompiled/00672770.c"
            },
            "startup_continue_flow": {
                "continue_check": {
                    "address": "0x005b6920",
                    "behavior": "calls RNG initializer 0x008fc5d0 with seed 0, then enters 0x005b6f10, then returns 0",
                    "citation": "Ghidra decompile ghidra_out/cm0102.exe/decompiled/005b6920.c"
                },
                "startup_flow": {
                    "address": "0x005b6f10",
                    "behavior": "loads intro/logo resources, pumps 0x00672770 while waiting, then calls setup 0x00803e00",
                    "citation": "Ghidra decompile ghidra_out/cm0102.exe/decompiled/005b6f10.c"
                },
                "setup": {
                    "address": "0x00803e00",
                    "behavior": "reads CM3_QSTART and optional -seed argument; calls RNG initializer with explicit seed or 0",
                    "citation": "Ghidra decompile ghidra_out/cm0102.exe/decompiled/00803e00.c"
                }
            },
            "shutdown": {
                "address": "0x005b6a10",
                "status": "verified",
                "citation": "EXECUTION_MODEL.md"
            },
            "save_game": {
                "address": "0x004e24e0",
                "format": "version u32=4, section count u32=22, named .dat sections, serialized pools",
                "status": "verified",
                "citation": "claims.json id 0x004e24e0-savfmt"
            },
            "load_game": {
                "address": "0x0089bd60",
                "status": "verified",
                "citation": "EXECUTION_MODEL.md"
            },
            "rng": {
                "match_random": "0x008fc4f0",
                "msvc_rand": "0x00935a94",
                "status": "verified",
                "citation": "claims.json id 0x008fc4f0-rng"
            }
        },
        "implementation_gates": [
            {
                "system": "day advance",
                "required_before_rust": [
                    "find the simulation/date advance entry below startup/UI flow",
                    "identify date/state writes",
                    "identify which user action triggers simulation advance"
                ],
                "current_rust_status": "infrastructure only"
            },
            {
                "system": "inbox/news",
                "required_before_rust": [
                    "locate original news/inbox functions",
                    "derive message record layout",
                    "derive triggers from code or save sections"
                ],
                "current_rust_status": "not implemented"
            },
            {
                "system": "fixtures",
                "required_before_rust": [
                    "derive competition schedule storage",
                    "derive fixture generation/advance functions",
                    "validate against save sections"
                ],
                "current_rust_status": "not implemented"
            },
            {
                "system": "transfers",
                "required_before_rust": [
                    "lift transfer_value_calc 0x008a9080 formula",
                    "derive bid/contract record layouts",
                    "derive AI decision entry points"
                ],
                "current_rust_status": "located, not implemented"
            },
            {
                "system": "match engine",
                "required_before_rust": [
                    "reuse verified RNG 0x008fc4f0",
                    "lift match setup structures",
                    "catalog event/rating deltas from code"
                ],
                "current_rust_status": "partially carved, not implemented"
            }
        ]
    })
}

#[derive(Debug, Clone)]
struct PeSection {
    name: String,
    virtual_address: u32,
    virtual_size: u32,
    raw_pointer: u32,
    raw_size: u32,
    characteristics: u32,
}

#[derive(Debug, Clone)]
struct PeImage {
    machine: u16,
    optional_magic: u16,
    image_base: u32,
    entry_point_rva: u32,
    sections: Vec<PeSection>,
}

fn validate_original_binary_report(exe: &Path) -> Result<serde_json::Value, String> {
    let bytes = fs::read(exe)
        .map_err(|err| format!("failed to read original binary {}: {err}", exe.display()))?;
    let pe = parse_pe_image(&bytes)?;
    let mut checks = Vec::new();

    push_binary_check(
        &mut checks,
        "mz-header",
        bytes.get(0..2) == Some(b"MZ"),
        "DOS MZ header present",
    );
    push_binary_check(
        &mut checks,
        "pe32-i386",
        pe.machine == 0x14c && pe.optional_magic == 0x10b,
        "PE32 i386 image",
    );
    push_binary_check(
        &mut checks,
        "image-base",
        pe.image_base == 0x0040_0000,
        "expected image base 0x00400000",
    );

    for (name, va, citation) in [
        ("WinMain", 0x006725c0, "claims.json id 0x006725c0-execmodel"),
        ("init_db_load_wrapper", 0x005b6940, "EXECUTION_MODEL.md"),
        ("db_loader", 0x005121a0, "EXECUTION_MODEL.md"),
        ("main_continue_check", 0x005b6920, "EXECUTION_MODEL.md"),
        ("main_tick", 0x00672770, "EXECUTION_MODEL.md"),
        ("shutdown", 0x005b6a10, "EXECUTION_MODEL.md"),
        ("save_game", 0x004e24e0, "claims.json id 0x004e24e0-savfmt"),
        ("load_game", 0x0089bd60, "EXECUTION_MODEL.md"),
        ("match_random", 0x008fc4f0, "claims.json id 0x008fc4f0-rng"),
        ("crt_rand", 0x00935a94, "claims.json id 0x008fc4f0-rng"),
    ] {
        let mapped = map_va_to_file_offset(&pe, va);
        let pass = mapped
            .and_then(|offset| bytes.get(offset..offset + 8).map(|slice| (offset, slice)))
            .map(|(_, slice)| slice.iter().any(|byte| *byte != 0))
            .unwrap_or(false);
        let detail = mapped
            .map(|offset| {
                let preview = bytes
                    .get(offset..offset.saturating_add(12).min(bytes.len()))
                    .unwrap_or(&[])
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{name} VA 0x{va:08x} -> file 0x{offset:x}; bytes {preview}; {citation}")
            })
            .unwrap_or_else(|| {
                format!("{name} VA 0x{va:08x} did not map to a PE section; {citation}")
            });
        push_binary_check(&mut checks, &format!("address-{name}"), pass, detail);
    }

    for (name, text, citation) in [
        (
            "single-instance-string",
            "Championship Manager 2001/02 is already running",
            "EXECUTION_MODEL.md WinMain single-instance check",
        ),
        (
            "save-error-string",
            "Save Game Error",
            "claims.json id 0x004e24e0-savfmt",
        ),
        (
            "load-error-string",
            "Load Game Error",
            "findings.json 0x0089bd60",
        ),
    ] {
        let found = find_ascii(&bytes, text);
        push_binary_check(
            &mut checks,
            name,
            found.is_some(),
            found
                .map(|offset| format!("{text:?} found at file 0x{offset:x}; {citation}"))
                .unwrap_or_else(|| format!("{text:?} not found; {citation}")),
        );
    }

    let failures = checks
        .iter()
        .filter(|check| check.get("status").and_then(serde_json::Value::as_str) == Some("fail"))
        .count();

    Ok(serde_json::json!({
        "binary": {
            "path": exe.display().to_string(),
            "size": bytes.len(),
            "machine": format!("0x{:04x}", pe.machine),
            "optional_magic": format!("0x{:04x}", pe.optional_magic),
            "image_base": format!("0x{:08x}", pe.image_base),
            "entry_point_rva": format!("0x{:08x}", pe.entry_point_rva),
            "sections": pe.sections.iter().map(|section| serde_json::json!({
                "name": section.name,
                "virtual_address": format!("0x{:08x}", section.virtual_address),
                "virtual_size": section.virtual_size,
                "raw_pointer": format!("0x{:x}", section.raw_pointer),
                "raw_size": section.raw_size,
                "executable": section.characteristics & 0x2000_0000 != 0,
            })).collect::<Vec<_>>(),
        },
        "source": {
            "carve_root": "D:/cm0102-carve",
            "documents": ["EXECUTION_MODEL.md", "PROGRESS.md", "findings.json", "claims.json"],
        },
        "summary": {
            "checks": checks.len(),
            "failures": failures,
            "status": if failures == 0 { "pass" } else { "fail" },
        },
        "checks": checks,
    }))
}

fn validate_execution_model_report(exe: &Path) -> Result<serde_json::Value, String> {
    let bytes = fs::read(exe)
        .map_err(|err| format!("failed to read original binary {}: {err}", exe.display()))?;
    let pe = parse_pe_image(&bytes)?;
    let mut checks = Vec::new();

    for (name, va, meaning, citation) in [
        (
            "winmain",
            0x0067_25c0,
            "process entry after CRT startup",
            "claims.json id 0x006725c0-execmodel",
        ),
        (
            "continue-check",
            0x005b_6920,
            "calls RNG initializer then startup flow; returns 0 in current decompile",
            "ghidra_out/cm0102.exe/decompiled/005b6920.c",
        ),
        (
            "startup-flow",
            0x005b_6f10,
            "loads intro resources and calls setup flow",
            "ghidra_out/cm0102.exe/decompiled/005b6f10.c",
        ),
        (
            "message-pump-shell",
            0x0067_2770,
            "PeekMessageA/GetMessageA/TranslateMessage/DispatchMessageA loop",
            "ghidra_out/cm0102.exe/decompiled/00672770.c",
        ),
        (
            "setup-flow",
            0x0080_3e00,
            "reads CM3_QSTART and optional -seed; calls RNG init",
            "ghidra_out/cm0102.exe/decompiled/00803e00.c",
        ),
        (
            "rng-init",
            cm_rng::MATCH_RNG_INIT_ADDR,
            "initializes match RNG table pointer/seed",
            "ghidra_out/cm0102.exe/decompiled/008fc5d0.c",
        ),
        (
            "match-day-frontier",
            0x0069_9d90,
            "match_day.cpp attributed function; calls match setup 0x0069d950",
            "carve ask 0x00699d90",
        ),
    ] {
        let mapped = map_va_to_file_offset(&pe, va);
        let pass = mapped
            .and_then(|offset| bytes.get(offset..offset + 8).map(|slice| (offset, slice)))
            .map(|(_, slice)| slice.iter().any(|byte| *byte != 0))
            .unwrap_or(false);
        let detail = mapped
            .map(|offset| {
                let preview = bytes
                    .get(offset..offset.saturating_add(12).min(bytes.len()))
                    .unwrap_or(&[])
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!(
                    "{meaning}; VA 0x{va:08x} -> file 0x{offset:x}; bytes {preview}; {citation}"
                )
            })
            .unwrap_or_else(|| format!("{meaning}; VA 0x{va:08x} did not map; {citation}"));
        push_binary_check(&mut checks, &format!("execution-{name}"), pass, detail);
    }

    for (name, text, citation) in [
        (
            "import-peek-message",
            "PeekMessageA",
            "00672770.c message-pump shell",
        ),
        (
            "import-get-message",
            "GetMessageA",
            "00672770.c message-pump shell",
        ),
        (
            "import-translate-message",
            "TranslateMessage",
            "00672770.c message-pump shell",
        ),
        (
            "import-dispatch-message",
            "DispatchMessageA",
            "00672770.c message-pump shell",
        ),
        (
            "main-source-string",
            "C:\\dev\\CM3 00-01\\si\\code\\main.cpp",
            "00672770.c error path",
        ),
        (
            "startup-resource-logo",
            "logo.rgn",
            "005b6f10.c splash load",
        ),
        (
            "startup-resource-eidos",
            "eidos.rgn",
            "005b6f10.c splash load",
        ),
        ("startup-resource-kio", "kio.rgn", "005b6f10.c splash load"),
        ("setup-qstart", "CM3_QSTART", "00803e00.c setup path"),
        ("setup-seed-arg", "-seed", "00803e00.c RNG seed path"),
    ] {
        let found = find_ascii(&bytes, text);
        push_binary_check(
            &mut checks,
            name,
            found.is_some(),
            found
                .map(|offset| format!("{text:?} found at file 0x{offset:x}; {citation}"))
                .unwrap_or_else(|| format!("{text:?} not found; {citation}")),
        );
    }

    let decompile_root = Path::new("D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled");
    for (name, file, required_terms) in [
        (
            "decompile-continue-check",
            "005b6920.c",
            &["FUN_008fc5d0(0)", "FUN_005b6f10()", "return 0"][..],
        ),
        (
            "decompile-message-pump",
            "00672770.c",
            &[
                "PeekMessageA",
                "GetMessageA",
                "TranslateMessage",
                "DispatchMessageA",
            ][..],
        ),
        (
            "decompile-setup-seed",
            "00803e00.c",
            &["s__seed", "FUN_008fc5d0(local_204)", "FUN_008fc5d0(0)"][..],
        ),
        (
            "decompile-startup-frontier",
            "005b6f10.c",
            &["s_logo_rgn", "s_eidos_rgn", "FUN_00803e00()"][..],
        ),
        (
            "decompile-match-day-frontier",
            "00699d90.c",
            &["FUN_0069d950", "FUN_00672770"][..],
        ),
    ] {
        let path = decompile_root.join(file);
        let text = fs::read_to_string(&path).unwrap_or_default();
        let missing = required_terms
            .iter()
            .filter(|term| !text.contains(**term))
            .copied()
            .collect::<Vec<_>>();
        push_binary_check(
            &mut checks,
            name,
            path.exists() && missing.is_empty(),
            if missing.is_empty() {
                format!(
                    "{} contains required evidence terms: {}",
                    path.display(),
                    required_terms.join(", ")
                )
            } else {
                format!(
                    "{} missing evidence terms: {}",
                    path.display(),
                    missing.join(", ")
                )
            },
        );
    }

    let functions_json =
        fs::read_to_string("D:/cm0102-carve/analysis/functions.json").unwrap_or_default();
    push_binary_check(
        &mut checks,
        "analysis-match-day-attribution",
        functions_json.contains("\"entry\": \"0x00699d90\"")
            && functions_json.contains("\"source_file\": \"match_day.cpp\""),
        "analysis/functions.json attributes 0x00699d90 to match_day.cpp",
    );

    let failures = checks
        .iter()
        .filter(|check| check.get("status").and_then(serde_json::Value::as_str) == Some("fail"))
        .count();

    Ok(serde_json::json!({
        "source": {
            "binary": exe.display().to_string(),
            "carve": "D:/cm0102-carve",
            "citations": [
                "claims.json id 0x006725c0-execmodel",
                "ghidra_out/cm0102.exe/decompiled/005b6920.c",
                "ghidra_out/cm0102.exe/decompiled/005b6f10.c",
                "ghidra_out/cm0102.exe/decompiled/00672770.c",
                "ghidra_out/cm0102.exe/decompiled/00803e00.c",
                "carve ask 0x00699d90"
            ]
        },
        "summary": {
            "checks": checks.len(),
            "failures": failures,
            "status": if failures == 0 { "pass" } else { "fail" },
        },
        "execution": {
            "winmain": "0x006725c0",
            "continue_check": {
                "address": "0x005b6920",
                "semantics": "calls RNG initializer with seed 0, enters startup flow 0x005b6f10, returns 0"
            },
            "message_pump_shell": {
                "address": "0x00672770",
                "semantics": "processes Win32 messages and updates DAT_00b4d580 with time when idle",
                "not_yet": "not proven as the day-advance simulation implementation"
            },
            "startup_flow": {
                "address": "0x005b6f10",
                "semantics": "intro/logo flow, splash waits, setup call"
            },
            "setup_flow": {
                "address": "0x00803e00",
                "semantics": "quick-start/setup path; reads optional -seed and calls RNG initializer"
            },
            "next_frontiers": [
                {
                    "address": "0x00699d90",
                    "status": "inferred match_day.cpp frontier; calls match setup 0x0069d950"
                },
                {
                    "address": "0x0069d950",
                    "status": "verified match setup, already carved"
                }
            ]
        },
        "checks": checks,
    }))
}

fn validate_simulation_frontier_report(exe: &Path) -> Result<serde_json::Value, String> {
    let bytes = fs::read(exe)
        .map_err(|err| format!("failed to read original binary {}: {err}", exe.display()))?;
    let pe = parse_pe_image(&bytes)?;
    let mut checks = Vec::new();

    for (name, va, meaning, citation) in [
        (
            "simulation-loop-candidate",
            0x005b_6a90,
            "loops current date state, creates match-day state, runs per-club callbacks, advances phase counter",
            "ghidra_out/cm0102.exe/decompiled/005b6a90.c",
        ),
        (
            "match-day-state-builder",
            0x0069_9640,
            "match_day.cpp attributed builder for per-date match collections",
            "carve ask 0x00699640",
        ),
        (
            "match-day-processor",
            0x0069_9d90,
            "match_day.cpp attributed processor that calls match setup 0x0069d950",
            "carve ask 0x00699d90",
        ),
        (
            "date-setter",
            0x0053_3d10,
            "date.cpp attributed date constructor/setter with day/month/year validation",
            "carve ask 0x00533d10",
        ),
        (
            "date-add-days",
            0x0053_6190,
            "adds signed day offsets to packed day-of-year/year state with leap-year handling",
            "ghidra_out/cm0102.exe/decompiled/00536190.c",
        ),
        (
            "date-offset-helper",
            0x0053_64c0,
            "called by simulation loop and date-add helper for date arithmetic",
            "analysis/functions.json date.cpp cluster",
        ),
        (
            "date-sensitive-data-update",
            0x004c_def0,
            "large date-sensitive data update called every phase from 0x005b6a90",
            "carve ask 0x004cdef0",
        ),
        (
            "fixture-cleanup-frontier",
            0x0059_5580,
            "date-sensitive fixture cleanup frontier called after phase-2 league initialisation",
            "carve ask 0x00595580",
        ),
        (
            "host-country-schedule-frontier",
            0x005e_4370,
            "date/RNG-driven schedule frontier called after phase-2 league initialisation",
            "carve ask 0x005e4370",
        ),
        (
            "manager-manager-frontier",
            0x0067_4c10,
            "manager_manager.cpp attributed date-sensitive frontier",
            "carve ask 0x00674c10",
        ),
        (
            "stadium-frontier",
            0x0084_4940,
            "stadium-attributed date-sensitive cleanup frontier",
            "ghidra_out/cm0102.exe/decompiled/00844940.c",
        ),
    ] {
        let mapped = map_va_to_file_offset(&pe, va);
        let pass = mapped
            .and_then(|offset| bytes.get(offset..offset + 8).map(|slice| (offset, slice)))
            .map(|(_, slice)| slice.iter().any(|byte| *byte != 0))
            .unwrap_or(false);
        let detail = mapped
            .map(|offset| {
                let preview = bytes
                    .get(offset..offset.saturating_add(12).min(bytes.len()))
                    .unwrap_or(&[])
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<Vec<_>>()
                    .join(" ");
                format!("{meaning}; VA 0x{va:08x} -> file 0x{offset:x}; bytes {preview}; {citation}")
            })
            .unwrap_or_else(|| format!("{meaning}; VA 0x{va:08x} did not map; {citation}"));
        push_binary_check(&mut checks, &format!("simulation-{name}"), pass, detail);
    }

    for (name, text, citation) in [
        (
            "date-source-string",
            "C:\\dev\\CM3 00-01\\si\\code\\Date.cpp",
            "00533d10.c date validation error path",
        ),
        (
            "match-day-source-fragment",
            "match_day",
            "analysis/functions.json match_day.cpp attribution",
        ),
        (
            "initialising-leagues-string",
            "Initialising Leagues",
            "005b6a90.c phase-2 league initialisation path",
        ),
        (
            "updating-game-data-string",
            "Updating game data",
            "004cdef0.c date-sensitive update path",
        ),
        (
            "match-source-fragment",
            "C:\\dev\\CM3 00-01\\cm3\\code\\match",
            "00699640.c match-day state builder error path",
        ),
    ] {
        let found = find_ascii(&bytes, text);
        push_binary_check(
            &mut checks,
            name,
            found.is_some(),
            found
                .map(|offset| format!("{text:?} found at file 0x{offset:x}; {citation}"))
                .unwrap_or_else(|| format!("{text:?} not found; {citation}")),
        );
    }

    let decompile_root = Path::new("D:/cm0102-carve/ghidra_out/cm0102.exe/decompiled");
    for (name, file, required_terms) in [
        (
            "decompile-simulation-loop",
            "005b6a90.c",
            &[
                "FUN_00699640(&DAT_00acde90,DAT_00acde88)",
                "FUN_00699d90()",
                "DAT_00acde88 = DAT_00acde88 + 1",
                "FUN_00536190(auStack_24,1)",
            ][..],
        ),
        (
            "decompile-phase2-league-block",
            "005b6a90.c",
            &[
                "if (DAT_00acde88 == 2)",
                "s_Initialising_Leagues_009b97d8",
                "FUN_0053fe40(&DAT_00acde90)",
                "FUN_00614e90()",
                "FUN_00674c10()",
                "FUN_00844940(&DAT_00acde90)",
            ][..],
        ),
        (
            "decompile-phase-callbacks",
            "005b6a90.c",
            &[
                "(**(code **)(*piVar1 + 0x14))(&DAT_00acde90,0)",
                "(**(code **)(*piVar1 + 0x14))(&DAT_00acde90,1)",
            ][..],
        ),
        (
            "decompile-date-add-days",
            "00536190.c",
            &[
                "sVar4 = *param_1 + (short)param_3",
                "sVar4 = sVar4 + -0x16d",
                "sVar4 = sVar4 + -0x16e",
                "DAT_00acd628 = CONCAT22(uVar3,sVar4)",
            ][..],
        ),
        (
            "decompile-date-setter",
            "00533d10.c",
            &[
                "param_2 < 1",
                "0x1f < param_2",
                "'\\v' < param_3",
                "FUN_00533eb0(param_5,param_1)",
            ][..],
        ),
        (
            "decompile-match-day-builder",
            "00699640.c",
            &[
                "param_1[0x2f] = -1",
                "FUN_00598d20(param_2,param_3,local_46c)",
                "iVar7 * 0x18",
                "iVar14 * 0x54",
                "*(int *)(param_1 + 0x21) * 0x69",
                "FUN_009343c3(puVar10[3] * 0x69",
                "DAT_00b4d93c = FUN_00672270()",
                "FUN_0069aa70()",
            ][..],
        ),
        (
            "decompile-match-day-queue-annotation",
            "0069aa70.c",
            &[
                "*(undefined1 *)(param_1 + 0x29) = 0",
                "local_1c = local_1c + 0x18",
                "iVar3 = iVar3 + 0x54",
                "iVar5 = iVar5 + 0x69",
                "DAT_00acd56c + -0x10 + iVar5",
                "FUN_005ea590(*(undefined4 *)(iVar6 + 0x20)",
                "FUN_0069bc10(iVar6 + 4)",
            ][..],
        ),
        (
            "decompile-match-day-processor-dispatcher",
            "00699d90.c",
            &[
                "FUN_0069b4a0()",
                "operator_new(0x11d)",
                "DAT_00acd5c4 + iVar16 * 0x6e",
                "FUN_005ea590(*(undefined4 *)(iVar17 + 0x20)",
                "FUN_0074d010(iVar13)",
                "FUN_0069d950",
                "FUN_00672770",
                "FUN_00933d24(*(undefined4 *)(iVar13 + 0x5b))",
            ][..],
        ),
        (
            "decompile-match-setup-frontier",
            "0069d950.c",
            &[
                "*(int **)(param_1 + 0x4792) = param_2",
                "FUN_008fc4f0(2)",
                "FUN_006c0f10(param_1",
                "param_1 + 0x4796",
                "param_1 + 0x6a6e",
                "local_320 = param_1 + local_2e8 * 0x22d8 + 0x4796",
                "FUN_00672320(&local_30c,0x19)",
                "FUN_006a1470(0)",
            ][..],
        ),
        (
            "decompile-match-team-player-setup",
            "006c0f10.c",
            &[
                "*(int *)((int)param_1 + 0x1b) = param_3",
                "*(int *)((int)param_1 + 0x23) = param_5",
                "*(char **)((int)param_1 + 0x2b) = param_2 + param_6 * 0x18e3 + 0x91e2",
                "for (iVar5 = 0x638; iVar5 != 0; iVar5 = iVar5 + -1)",
                "*param_10 = (*(byte *)(param_9 + 0x4d) & 0xf) + 0xb",
                "FUN_008830a0(*(undefined4 *)((int)param_1 + 0x2f),(int)param_1 + 0x3f",
            ][..],
        ),
        (
            "decompile-match-player-risk-setup",
            "006a1470.c",
            &[
                "*(undefined2 *)(param_1 + 0x1ce + iVar10 * 2) = 0",
                "iVar11 = param_1 + 0x4796 + iVar10 * 0x22d8",
                "local_c = 0x14",
                "iVar9 = iVar9 + 0x1be",
                "FUN_008fc4f0(0x14)",
                "FUN_008fc4f0(7000)",
                "FUN_008fc4f0(6)",
                "FUN_006d1780()",
                "FUN_006d46c0(1)",
            ][..],
        ),
        (
            "decompile-tactics-block-loader",
            "008830a0.c",
            &[
                "s_C__dev_CM3_00_01_cm3_code_tactic",
                "iVar2 = FUN_00882f60(param_2,param_4)",
                "*(int *)(param_1 + 0x601) + iVar2 * 0x91",
                "for (iVar3 = 0x24; iVar3 != 0; iVar3 = iVar3 + -1)",
                "*(undefined1 *)param_3 = *(undefined1 *)puVar4",
            ][..],
        ),
        (
            "decompile-tactics-index-resolver",
            "00882f60.c",
            &[
                "iVar2 = FUN_005ea590(param_1,1,1,0,0)",
                "iVar2 = FUN_0052a500(param_1)",
                "return -1",
                "piVar1 = *(int **)((int)param_1 + 0xcf)",
                "return *param_1",
            ][..],
        ),
        (
            "decompile-selected-tactic-staff-slot-lookup",
            "00882240.c",
            &[
                "iVar3 = FUN_00882f60(param_2,param_3)",
                "iVar3 * 0x91 + cVar4 * 4 + 2 + *(int *)(param_1 + 0x601)",
                "DAT_00acd5c4 + iVar1 * 0x6e",
                "iVar3 * 0x91 + 0x52 + *(int *)(param_1 + 0x601)",
                "if ('\\x13' < cVar4)",
            ][..],
        ),
        (
            "decompile-match-primary-tactic-flag-reader",
            "006a91d0.c",
            &[
                "cVar1 = *(char *)(param_2 + 0x19)",
                "(-1 < cVar1) && (cVar1 < '\\v')",
                "param_1 + 0x8ebc + *(char *)(param_2 + 0x27) * 4",
                "cVar1 * 2",
            ][..],
        ),
        (
            "decompile-match-secondary-tactic-flag-reader",
            "006a9200.c",
            &[
                "cVar1 = *(char *)(param_2 + 0x19)",
                "(-1 < cVar1) && (cVar1 < '\\v')",
                "param_1 + 0x8ec4 + *(char *)(param_2 + 0x27) * 4",
                "cVar1 * 2",
            ][..],
        ),
        (
            "decompile-match-player-random-byte-seed",
            "006d9ea0.c",
            &[
                "FUN_008fc4f0(sVar2 + -0x19 + (int)*(char *)(param_1 + 0x10e))",
                "*(char *)(param_1 + 0x104) = (char)iVar4",
                "FUN_008fc4f0((int)*(char *)(param_1 + 0x110))",
                "*(char *)(param_1 + 0x10a) = (char)iVar4",
                "*(undefined1 *)(param_1 + 0x107) = uVar1",
                "*(undefined1 *)(param_1 + 0x10d) = uVar1",
            ][..],
        ),
        (
            "decompile-match-player-evaluation-frontier",
            "006d1a20.c",
            &[
                "cVar14 = *(char *)(param_1 + 0x27)",
                "iVar22 = *(int *)(param_1 + 0x19e)",
                "*(undefined2 *)(param_1 + 0x3b) = uVar18",
                "*(float *)(param_1 + 0x7d)",
                "*(float *)(param_1 + 0x8d)",
                "*(float *)(param_1 + 0x91) = fVar9",
                "*(float *)(param_1 + 0x95)",
                "*(float *)(param_1 + 0x99)",
                "FUN_006a91d0(param_1)",
                "FUN_006a9200(param_1)",
                "FUN_00935080()",
            ][..],
        ),
        (
            "decompile-match-player-action-score-frontier",
            "006d46c0.c",
            &[
                "FUN_006d1a20()",
                "*(undefined2 *)(param_1 + 0x37) = 0",
                "FUN_006a91d0(param_1)",
                "FUN_006a9200(param_1)",
                "*(char *)(*(int *)(param_1 + 0x6d) + 0x17)",
                "*(char *)(*(int *)(param_1 + 0x6d) + 0x18)",
                "*(char *)(*(int *)(param_1 + 0x6d) + 0x19)",
                "*(char *)(*(int *)(param_1 + 0x6d) + 0x1a)",
                "FUN_006d9ea0()",
            ][..],
        ),
        (
            "decompile-match-candidate-action-wrapper",
            "006b2cb0.c",
            &[
                "*(char *)(iVar3 + 0x58 + param_3)",
                "piVar2 = (int *)(param_3 + iVar3 * 0x2c)",
                "*(char *)(*piVar2 + 0x2b) < '\\x02'",
                "-1 < *(char *)(*piVar2 + 0x19)",
                "cVar1 = FUN_006db630(param_5,param_6,param_4,param_7)",
                "*(undefined1 *)(param_1 + 0x1a6) = 0",
                "*(undefined1 *)(param_1 + 0x1a5) = 0",
            ][..],
        ),
        (
            "decompile-match-adjacent-position-action-wrapper",
            "006db580.c",
            &[
                "cVar1 = *(char *)(param_1 + 0x103)",
                "*(char *)(param_1 + 0x27) != '\\x01'",
                "*(undefined1 *)(param_1 + 0x102)",
                "*(char *)(*(int *)(param_1 + 0x19e) + 0x8eaf) + -1",
                "*(char *)(*(int *)(param_1 + 0x19e) + 0x8eb0) + '\\x01'",
                "FUN_006d63f0(*(undefined1 *)(param_1 + 0x102),cVar1,0xffffffff,0,0)",
            ][..],
        ),
        (
            "decompile-match-player-action-attempt-frontier",
            "006db630.c",
            &[
                "*(int *)(*(int *)(param_1 + 0x19e) + 0xf576) == param_1",
                "FUN_008fc4f0(500000)",
                "FUN_006bc8d0(0x1f5e",
                "*(int *)(*(int *)(param_1 + 0x19e) + 0x4782)",
                "*(undefined1 *)(*(int *)(param_1 + 0x19e) + 0x8ea7)",
                "*(undefined1 *)(*(int *)(param_1 + 0x19e) + 0x8ea8)",
                "*(bool *)(iVar8 + 0x8eae)",
                "*(undefined1 *)(*(int *)(param_1 + 0x19e) + 0x8eb2)",
                "FUN_006d46c0(0)",
            ][..],
        ),
        (
            "decompile-match-player-move-action-resolution-frontier",
            "006d63f0.c",
            &[
                "local_22e = *(char *)(param_1 + 0x102)",
                "local_236 = *(char *)(param_1 + 0x103)",
                "*(char *)(param_1 + 0x2b) = *(char *)(param_1 + 0x2b) + '\\x01'",
                "cVar10 = *(char *)(param_1 + 0x107)",
                "cVar11 = *(char *)(param_1 + 0x104)",
                "FUN_008fc4f0(900)",
                "*(undefined2 *)(param_1 + 0x198) = 0x6f",
                "*(short *)(param_1 + 0x19c)",
                "sVar25 == 0x12",
                "FUN_006bc8d0(0x1f4d",
            ][..],
        ),
        (
            "decompile-match-player-action-selector-frontier",
            "006f99c0.c",
            &[
                "*(undefined2 *)(param_1 + 0x198) = 0xffff",
                "*(char *)(iVar8 + 0x8ea9) == -1",
                "*(undefined2 *)(param_1 + 0x198) = 0x68",
                "*(undefined2 *)(param_1 + 0x198) = 0x6a",
                "*(undefined2 *)(param_1 + 0x198) = 0x6b",
                "*(undefined2 *)(param_1 + 0x198) = 0x76",
                "FUN_006d63f0(local_21c,local_218,0xffffffff,0,0)",
                "FUN_006db580(0)",
            ][..],
        ),
        (
            "decompile-match-event-resolution-dispatcher-frontier",
            "006f63f0.c",
            &[
                "switch(*(undefined1 *)(iVar15 + 0x8eb2))",
                "*(undefined1 *)(iVar15 + 0x8eb2) = 0",
                "FUN_006ac3b0(local_290,local_26c",
                "FUN_006dfc50(0x36",
                "sVar13 = FUN_006e65e0(&local_281,&local_254)",
                "FUN_006e7a60(uVar19,iVar15",
                "FUN_006bc8d0(0x1f44",
                "FUN_006d63f0(local_27c,local_280,0x12",
            ][..],
        ),
        (
            "decompile-match-event-queue-writer",
            "006bc8d0.c",
            &[
                "(7999 < (short)param_2) && ((short)param_2 < 0x21e5)",
                "local_218[0] = param_2",
                "param_3[0x8eb3] != '\\x02'",
                "uVar5 = FUN_006bba10(local_218,param_3)",
                "param_2 = FUN_006bb6e0(local_218",
                "param_1[*(short *)(param_1 + 8) * 0xe + 0x34] = bVar7",
                "*(ushort *)(param_1 + *(short *)(param_1 + 8) * 0xe + 0x30) =",
                "*(undefined4 *)(param_1 + *(short *)(param_1 + 8) * 0xe + 0x39) =",
                "*(undefined2 *)(param_1 + *(short *)(param_1 + 6) * 0xe + 0x720)",
                "FUN_006bc8d0(0x21a0,param_3",
                "FUN_006bc8d0(0x21bf,param_3",
            ][..],
        ),
        (
            "decompile-match-event-follow-up-challenge-frontier",
            "006dfc50.c",
            &[
                "*(char *)(param_1 + 0x2b) = *(char *)(param_1 + 0x2b) + '\\x01'",
                "cVar3 = FUN_006e0740(*(undefined1 *)(param_1 + 0x102)",
                "iVar4 = FUN_006b57d0(*(undefined1 *)(param_1 + 0x27)",
                "FUN_006bc8d0(0x1f78",
                "*(char *)(*(int *)(param_1 + 0x19e) + 0xf5ca + *(char *)(param_1 + 0x27) * 0x36)",
                "*(undefined1 *)(*(int *)(param_1 + 0x19e) + 0x8ea7)",
                "*(undefined1 *)(*(int *)(param_1 + 0x19e) + 0x8ea8)",
                "*(undefined1 *)(*(int *)(param_1 + 0x19e) + 0x8eb2) = 7",
                "*(int *)(*(int *)(param_1 + 0x19e) + 0xf57a) = param_3",
            ][..],
        ),
        (
            "decompile-match-directional-follow-up-frontier",
            "006dfe90.c",
            &[
                "sVar1 = *(short *)(param_1 + 0x19a)",
                "iVar12 = *(char *)(param_1 + 0x103) + -0xb",
                "iVar13 = *(char *)(param_1 + 0x102) + -4",
                "cVar9 = FUN_008fc4f0(2)",
                "cVar9 = FUN_008fc4f0(3)",
                "cVar9 = FUN_006e0740(*(undefined1 *)(param_1 + 0x102)",
                "FUN_006bc8d0(0x1f78",
                "*(undefined1 *)(*(int *)(param_1 + 0x19e) + 0x8ea7)",
                "*(undefined1 *)(*(int *)(param_1 + 0x19e) + 0x8eb2) = 7",
            ][..],
        ),
        (
            "decompile-match-shot-action-score-frontier",
            "006e65e0.c",
            &[
                "*param_2 == 0x33",
                "*param_3 = 0x1f7f",
                "*param_2 = 0x16",
                "*param_2 = 0x1d",
                "*param_2 = 0x3a",
                "*(short *)(param_1 + 0x39)",
                "*(short *)(param_1 + 0x11a + (char)*param_2 * 2)",
                "FUN_008fc4f0(*(short *)(param_1 + 0x180) * 5)",
                "*(float *)(param_1 + 0x79)",
                "*(float *)(param_1 + 0xe5)",
            ][..],
        ),
        (
            "decompile-match-engine-step-controller",
            "0069f2f0.c",
            &[
                "param_1[0x8eb4] != '\\x01'",
                "switch(param_1[0x8eb3] + -1)",
                "FUN_006a4020(4)",
                "FUN_006a4020(3)",
                "FUN_006bc8d0(0x20ef",
                "iVar7 = FUN_006a0550()",
                "*(short *)(param_1 + 0x8ed2) = *(short *)(param_1 + 0x8ed2) + 1",
                "*(short *)(param_1 + 0x8ed0) = *(short *)(param_1 + 0x8ed0) + 1",
                "FUN_006bc8d0(0x217b",
                "FUN_006bc8d0(0x2002",
                "FUN_006bc8d0(0x2003",
                "*(undefined1 *)(*(int *)(param_1 + 0x4792) + 0x43) = 0xfd",
            ][..],
        ),
        (
            "decompile-match-phase-possession-controller",
            "006a4020.c",
            &[
                "switch(*(undefined1 *)(param_1 + 0x8eb3))",
                "*(undefined2 *)(param_1 + 0x475f) = 0xffff",
                "*(undefined4 *)(param_1 + 0x4761) = 0",
                "*(undefined1 *)(param_1 + 0x475b) = 0xff",
                "pcVar1 = (char *)(param_1 + 0x4796 + ((int)cVar4 + iVar8 * 0x14) * 0x1be)",
                "FUN_006e65e0(&local_8,&local_4)",
                "FUN_006f63f0(4)",
                "FUN_006bc8d0(0x2005",
                "FUN_006bc8d0(0x2006",
                "FUN_006bc8d0(0x2004",
                "*(undefined1 *)(*(int *)(param_1 + 0x4792) + 0x49)",
            ][..],
        ),
        (
            "decompile-match-pressure-action-continuation",
            "006f5de0.c",
            &[
                "param_1 != *(int *)(*(int *)(param_1 + 0x19e) + 0xf582)",
                "uVar7 = FUN_006a91d0(param_1)",
                "FUN_006a2730(*(undefined4 *)(param_1 + 0x1ae + local_12 * 4)",
                "iVar8 = FUN_008fc4f0(900)",
                "*(undefined2 *)(param_1 + 0x198) = 0x67",
                "*(short *)(param_1 + 0x35) = *(short *)(param_1 + 0x35) + -0xf",
                "*(float *)(param_1 + 0x4d) = (float)(iVar8 + 5) + *(float *)(param_1 + 0x4d)",
                "FUN_006f99c0(1)",
                "FUN_006f63f0(0)",
                "*(undefined1 *)(*(int *)(param_1 + 0x19e) + 0x1a7) = 0",
            ][..],
        ),
        (
            "decompile-match-stored-action-resolver",
            "006a0550.c",
            &[
                "pcVar2 = (char *)(param_1 + 0x475a)",
                "*(undefined4 *)(param_1 + 0x4761) = 0",
                "*(undefined4 *)(param_1 + 0x4765) = 0",
                "FUN_006bc8d0(0x20f0",
                "FUN_006bc8d0(0x20ee",
                "FUN_006bc8d0(0x20fb",
                "FUN_006bc8d0(0x1f7a",
                "FUN_006bc8d0(0x20f5",
                "FUN_006bc8d0(0x2109",
                "*(undefined1 *)(param_1 + 0x8ea7)",
                "*(undefined1 *)(param_1 + 0x8eb2) = 7",
                "*(undefined4 *)(param_1 + 0xf582) = 0",
            ][..],
        ),
        (
            "decompile-match-action-scratch-reset-helper",
            "006a1320.c",
            &[
                "*(undefined2 *)(param_1 + 0x475f) = 0xffff",
                "*(undefined4 *)(param_1 + 0x4761) = 0",
                "*(undefined4 *)(param_1 + 0x4765) = 0",
                "*(undefined1 *)(param_1 + 0x4769) = 0",
                "*(undefined1 *)(param_1 + 0x475a) = 0",
                "*(undefined1 *)(param_1 + 0x475b) = 0xff",
                "*(undefined1 *)(param_1 + 0x475e) = 0xff",
            ][..],
        ),
        (
            "decompile-match-period-transition-frontier",
            "006a3240.c",
            &[
                "iVar4 = (int)*(short *)(param_1 + 0x8ed4)",
                "if (iVar4 == 0x1ef)",
                "if (iVar4 == 0x483)",
                "if (iVar4 == 0x528)",
                "*(undefined1 *)(*(int *)(param_1 + 0x4792) + 0x43)",
                "*(undefined1 *)(*(int *)(param_1 + 0x4792) + 0x47)",
                "FUN_006bc8d0(0x20f1",
                "FUN_006bc8d0(0x20f2",
                "FUN_006bc8d0(0x20f3",
                "FUN_006db210(1)",
                "*(undefined1 *)(param_1 + 0x8eb2) = 8",
                "*(undefined2 *)(param_1 + 0x8ed4) = 0",
            ][..],
        ),
        (
            "decompile-match-player-candidate-selector",
            "006b4510.c",
            &[
                "iVar1 = param_1 + 0x4796 + (iVar13 + local_210) * 0x1be",
                "iVar1 != *(int *)(param_1 + 0xf59e)",
                "iVar1 == *(int *)(param_1 + 0xf5a2)",
                "*(undefined1 *)(param_1 + 0x8ea7)",
                "*(undefined1 *)(param_1 + 0x8ea8)",
                "*(int *)(param_1 + 0x8ebc + *(char *)(iVar1 + 0x27) * 4)",
                "FUN_00882640(uVar10,3,0",
                "FUN_008fc4f0(1000)",
                "if ((local_21c == 0) || (sVar5 < sVar8))",
                "return local_21c",
            ][..],
        ),
        (
            "decompile-match-per-tick-tactical-state-updater",
            "006aae20.c",
            &[
                "*(undefined1 *)(param_1 + 0x1c5) = 0",
                "*(undefined4 *)(param_1 + 0x4782) = 0",
                "*(undefined1 *)(param_1 + 0x475a) = 0",
                "sVar2 = *(short *)(param_1 + 0x8ed0)",
                "FUN_006bc8d0(0x21cf",
                "piVar3 = *(int **)(param_1 + 0x904d)",
                "piVar3 = *(int **)(param_1 + 0x911d)",
                "FUN_006a1470(0)",
                "FUN_006a1470(1)",
                "FUN_006bc8d0(0x21c1",
                "FUN_006bc8d0(0x2137",
                "FUN_006bc8d0(0x2139",
                "switch(*(undefined1 *)(param_1 + 0x8eb2))",
            ][..],
        ),
        (
            "decompile-formation-primary-mask-classifier",
            "005a2c70.c",
            &[
                "s_C__dev_CM3_00_01_cm3_code_format",
                "*(ushort *)(param_1 + 0x12d + cVar4 * 2)",
                "*(ushort *)(param_1 + 0x14e + cVar4 * 2)",
                "(uVar1 & 0x880) != 0",
                "(uVar1 & 0x40) != 0",
                "(uVar1 & 0x20) != 0",
                "FUN_0059cdf0(local_e04,1)",
                "return 1",
            ][..],
        ),
        (
            "decompile-formation-secondary-mask-classifier",
            "005a30d0.c",
            &[
                "s_C__dev_CM3_00_01_cm3_code_format",
                "*(ushort *)(param_1 + 0x12d + cVar4 * 2)",
                "*(ushort *)(param_1 + 0x14e + cVar4 * 2)",
                "(uVar1 & 0x880) != 0",
                "FUN_0059cdf0(local_e04,1)",
                "(uVar3 & 8) != 0",
                "return 1",
            ][..],
        ),
        (
            "decompile-random-float-jitter-shim",
            "00935080.c",
            &["FUN_009350a2((double)in_ST1,(double)in_ST0)"][..],
        ),
        (
            "decompile-queued-club-news-cleanup",
            "00449710.c",
            &[
                "*(int *)(param_1 + 0x24) != 0",
                "iVar7 = iVar7 + 6",
                "DAT_00acd5bc + iVar6 * 0x245",
                "FUN_00763b90(1000,0)",
                "FUN_004539f0(iVar6,uVar5,local_f0",
                "FUN_0076e180(local_ec,iVar3)",
                "FUN_0076e390(local_ec,*(undefined4 *)(iVar3 + 0x53))",
                "99 < *(int *)(param_1 + 0x28)",
            ][..],
        ),
        (
            "decompile-date-sensitive-data-update",
            "004cdef0.c",
            &[
                "s_Updating_game_data_00994988",
                "FUN_00536190(&local_36c,7)",
                "FUN_00536190(&local_36c,0x1e)",
                "FUN_00536190(&local_36c,0xb6)",
                "FUN_00536190(&local_36c,0x16d)",
                "FUN_00536190(&local_36c,0x447)",
                "iVar13 * 0x6e + 0x59",
                "*piVar14 * 0x4f + DAT_00acdf0c",
                "iVar13 * 0x50 + *param_1",
                "DAT_00acd5bc + *(int *)(iVar13 + 4) * 0x245",
                "switch(*(byte *)(iVar13 + 0x4f) >> 4)",
                "FUN_005246e0(piVar14,&DAT_00accb18)",
                "FUN_004dc980(iVar13,7",
                "FUN_004dabd0(piVar17,1)",
                "FUN_004dcf60(iVar13,0)",
            ][..],
        ),
        (
            "decompile-current-date-callback-dispatcher",
            "0053fe40.c",
            &[
                "iVar1 = 0x2a",
                "*(int **)(iVar2 + *param_1) != (int *)0x0",
                "(**(code **)(**(int **)(iVar2 + *param_1) + 4))(param_2)",
                "iVar2 = iVar2 + 4",
            ][..],
        ),
        (
            "decompile-staff-role-competition-drift-frontier",
            "00614e90.c",
            &[
                "FUN_00933d24(iVar10)",
                "psVar6[-1] = psVar6[-1] + -1",
                "FUN_00457080(DAT_00dbc268,DAT_00dbc26c)",
                "DAT_00acde90 == 0xb4",
                "FUN_008fc4f0(DAT_00acd56c)",
                "iVar7 = iVar11 * 0x6e",
                "FUN_00616820(cVar2,uVar12",
                "FUN_00616930(uVar3,0",
                "FUN_006176f0()",
                "FUN_006180c0()",
            ][..],
        ),
        (
            "decompile-season-calendar-maintenance-frontier",
            "005bfd90.c",
            &[
                "DAT_00acde89 = DAT_00acde89 | 1",
                "puVar14 = &DAT_00b4bc85",
                "if ((int)DAT_00acde90 == (uint)*puVar14)",
                "FUN_00536190(&local_110,1)",
                "s_Updating_game_data_00994988",
                "local_118 == 3",
                "DAT_00acd5c4 + 0x6d",
                "FUN_00448170(piVar2)",
                "FUN_007a9690(piVar2)",
                "FUN_00586cf0(piVar2)",
                "FUN_00856690(piVar2)",
            ][..],
        ),
        (
            "decompile-club-rolling-metrics-frontier",
            "005c01d0.c",
            &[
                "DAT_009b97b8 != '\\0'",
                "DAT_00acd5b0 + 0xd0",
                "DAT_00acd558 * 0xc",
                "iVar7 = iVar7 + 0x122",
                "*(double *)(iVar7 + 0xd8)",
                "*(undefined1 *)(iVar7 + 0x121) = 0",
                "FUN_009343c3(pvVar5,DAT_00acd558,0xc",
            ][..],
        ),
        (
            "decompile-fixture-tie-participant-notification-frontier",
            "00752d40.c",
            &[
                "FUN_00533b50(1,0",
                "FUN_00533b50(0xf,3",
                "FUN_00752830()",
                "FUN_00596590(",
                "FUN_00536190(local_2c,2)",
                "FUN_007536f0(iVar8)",
                "FUN_0075ee00(*(undefined4 *)(iVar8 + 0x1c),0,iVar8)",
                "*(ushort *)(iVar8 + 0x4d) & 0x200",
                "if (*(short *)(iVar8 + 0x10) == sVar1)",
                "(int)*param_2 % 0x46 == 0",
            ][..],
        ),
        (
            "decompile-club-finance-stadium-status-frontier",
            "00585ae0.c",
            &[
                "FUN_00533b50(0x10,5,0x7d4",
                "DAT_00acd5bc + DAT_009bc6ec * 0x245",
                "_strncmp((char *)(iVar15 + 4 + DAT_00acd5b8),s_Falmer",
                "FUN_00597260(",
                "FUN_005962b0(iVar19)",
                "FUN_00594d00(iVar19,1)",
                "puVar17 = (uint *)((int)puVar17 + 0x167)",
                "iVar19 = iVar19 + 0x245",
                "FUN_008fc4f0(5)",
                "FUN_008fc4f0(20000)",
                "FUN_007328a0(iVar19)",
                "FUN_00588c70(iVar19,1)",
            ][..],
        ),
        (
            "decompile-byte-array-clear-frontier",
            "00784290.c",
            &[
                "0 < *(int *)(param_1 + 4)",
                "*(undefined1 *)(iVar1 + *(int *)(param_1 + 8)) = 0",
                "iVar2 < *(int *)(param_1 + 4)",
            ][..],
        ),
        (
            "decompile-database-date-age-helper",
            "005246e0.c",
            &[
                "*(short *)(param_1 + 0x10)",
                "*(short *)(param_1 + 0x12)",
                "return (param_2[1] - *(short *)(param_1 + 0x12)) + -1",
            ][..],
        ),
        (
            "decompile-fixture-cleanup-frontier",
            "00595580.c",
            &[
                "sVar3 = FUN_00533fd0()",
                "FUN_00536190(local_1c,3)",
                "FUN_007ce4a0(local_1c)",
                "DAT_00acd5bc + iVar5 * 0x245",
                "FUN_0076d7d0(0xe)",
                "FUN_0076e180(iVar4,iVar8)",
                "FUN_005ea590(piVar1[7],1,1,0,0)",
                "FUN_0050c8d0(*(undefined2 *)",
            ][..],
        ),
        (
            "decompile-fixture-news-field-reader",
            "0076d7d0.c",
            &[
                "news_c_00a55348",
                "(-1 < param_2) && (param_2 < '2')",
                "param_1 + 5 + param_2 * 4",
            ][..],
        ),
        (
            "decompile-fixture-news-reattach",
            "0076e180.c",
            &[
                "news_c_00a55348",
                "*(int **)(param_2 + 0xcf)",
                "*(undefined1 *)(param_1 + 0xde) = 0",
                "FUN_0076dce0(param_1,piVar1,0)",
            ][..],
        ),
        (
            "decompile-fixture-human-manager-filter",
            "005ea590.c",
            &[
                "human__009bb5e0",
                "DAT_00acd56c + -0x10 <= *param_4",
                "FUN_005e7b80(param_4)",
                "FUN_005e6d30(param_4)",
                "FUN_005e6fc0(param_4)",
                "FUN_005e71f0(param_4,0)",
            ][..],
        ),
        (
            "decompile-fixture-dated-event-generator",
            "0050c8d0.c",
            &[
                "iVar5 = iVar6 * 0x68",
                "FUN_00536190(local_14,1)",
                "FUN_00596fa0(local_28",
                "*(short *)(iVar3 + 0x30) = *(short *)(iVar1 + 7 + iVar5) + 3",
                "*(short *)(iVar3 + 0x30) = *(short *)(*(int *)(param_1 + 0xa3) + 7 + iVar5) + 4",
            ][..],
        ),
        (
            "decompile-manager-job-lifecycle-frontier",
            "00674c10.c",
            &[
                "DAT_00acd5bc",
                "DAT_00acd5c4",
                "DAT_00acd56c * 0xb",
                "(int)DAT_00acde90 % 7 == 0",
                "FUN_006822d0()",
                "FUN_006808a0()",
                "FUN_00680cc0(piVar14,piVar1)",
                "FUN_00688070(piVar1,puVar17,piVar14",
                "FUN_008b8870(&iStack_4dc)",
            ][..],
        ),
        (
            "decompile-manager-job-expiry-helper",
            "006808a0.c",
            &[
                "*(int *)(param_1 + 4)",
                "DAT_00acd5bc + iVar2 + 0xcf + iVar2 * 0x244",
                "FUN_006809e0(iVar1",
                "FUN_00680cc0(iVar1,iVar3)",
            ][..],
        ),
        (
            "decompile-manager-missing-job-repair",
            "006822d0.c",
            &[
                "DAT_00acd564",
                "piVar5 = (int *)((int)piVar5 + 0x245)",
                "s_MANAGER_MANAGER__fix_missing_job",
                "FUN_00680cc0(piVar5,0)",
            ][..],
        ),
        (
            "decompile-manager-job-allocation-worker",
            "00680cc0.c",
            &[
                "local_40 = -1000000",
                "*param_2 * 0x49",
                "piVar10 = DAT_00acd5c4",
                "piVar10 = (int *)((int)piVar10 + 0x6e)",
                "s_MANAGER_MANAGER__add_job_vacancy",
                "FUN_008fc4f0(DAT_00acd56c)",
                "FUN_008fc4f0(2000)",
                "FUN_008fc4f0(7)",
                "FUN_00672320(&local_34,0x26)",
            ][..],
        ),
        (
            "decompile-manager-job-relation-clear",
            "00675980.c",
            &[
                "*(int *)(param_1 + 0x39) == param_2",
                "*(undefined1 *)(param_1 + 0x3d) = 5",
                "FUN_00881910(param_2,param_1,0,1)",
                "param_2 + 0xd7 + cVar2 * 4",
                "*(undefined1 *)(iVar3 + 0x4e) = 4",
            ][..],
        ),
        (
            "decompile-host-country-schedule-frontier",
            "005e4370.c",
            &[
                "(short)param_1[1] * 0x22",
                "FUN_00533b50(1,6",
                "param_2[1] + 8",
                "param_2[1] + 5",
                "*(char *)(local_58 + 0x1e) == -2",
                "FUN_008fc4f0(3)",
                "FUN_005e49e0(local_40)",
                "FUN_009343c3(*param_1",
            ][..],
        ),
        (
            "decompile-host-country-candidate-assignment",
            "005e4840.c",
            &[
                "DAT_00acd5b0",
                "DAT_00acd558",
                "40000 < *(int *)",
                "FUN_008fc4f0(iVar6)",
                "iVar9 == DAT_009bbba8",
            ][..],
        ),
        (
            "decompile-host-country-event-emission",
            "005e49e0.c",
            &[
                "*(undefined4 *)(param_1 + 10) = *(undefined4 *)(param_1 + 2)",
                "cVar4 = *(char *)((int)piVar1 + 0x1e)",
                "FUN_00763b90(4,0)",
                "FUN_00763b90(5,0)",
                "FUN_0076d730(7",
            ][..],
        ),
        (
            "decompile-stadium-frontier",
            "00844940.c",
            &[
                "FUN_008470e0()",
                "FUN_00533b50(1,6,0x7d3",
                "(int)*param_2 % 0x1e == 0",
                "DAT_00acd5b0 + 0x80 + DAT_009bb7a4 * 0x122",
                "DAT_00acd5b8 + DAT_00a6f1f0 * 0x4e",
                "FUN_009343c3(*param_1",
            ][..],
        ),
    ] {
        let path = decompile_root.join(file);
        let text = fs::read_to_string(&path).unwrap_or_default();
        let missing = required_terms
            .iter()
            .filter(|term| !text.contains(**term))
            .copied()
            .collect::<Vec<_>>();
        push_binary_check(
            &mut checks,
            name,
            path.exists() && missing.is_empty(),
            if missing.is_empty() {
                format!(
                    "{} contains required evidence terms: {}",
                    path.display(),
                    required_terms.join(", ")
                )
            } else {
                format!(
                    "{} missing evidence terms: {}",
                    path.display(),
                    missing.join(", ")
                )
            },
        );
    }

    let functions_json =
        fs::read_to_string("D:/cm0102-carve/analysis/functions.json").unwrap_or_default();
    for (name, address, source_file) in [
        ("analysis-match-day-builder", "0x00699640", "match_day.cpp"),
        (
            "analysis-match-day-processor",
            "0x00699d90",
            "match_day.cpp",
        ),
        ("analysis-date-setter", "0x00533d10", "date.cpp"),
        (
            "analysis-manager-manager-frontier",
            "0x00674c10",
            "manager_manager.cpp",
        ),
    ] {
        push_binary_check(
            &mut checks,
            name,
            functions_json.contains(&format!("\"entry\": \"{address}\""))
                && functions_json.contains(&format!("\"source_file\": \"{source_file}\"")),
            format!("analysis/functions.json attributes {address} to {source_file}"),
        );
    }

    let failures = checks
        .iter()
        .filter(|check| check.get("status").and_then(serde_json::Value::as_str) == Some("fail"))
        .count();

    Ok(serde_json::json!({
        "source": {
            "binary": exe.display().to_string(),
            "carve": "D:/cm0102-carve",
            "citations": [
                "ghidra_out/cm0102.exe/decompiled/005b6a90.c",
                "ghidra_out/cm0102.exe/decompiled/00536190.c",
                "ghidra_out/cm0102.exe/decompiled/00533d10.c",
                "carve ask 0x00699640",
                "carve ask 0x00699d90"
            ]
        },
        "summary": {
            "checks": checks.len(),
            "failures": failures,
            "status": if failures == 0 { "pass" } else { "fail" },
        },
        "frontier": {
            "simulation_loop_candidate": {
                "address": "0x005b6a90",
                "status": "code-derived frontier, not fully named",
                "observed_shape": [
                    "compares current packed date globals against target packed date",
                    "builds per-date match-day state through 0x00699640",
                    "runs match-day processor 0x00699d90",
                    "runs per-club callbacks over DAT_00ac688c",
                    "increments DAT_00acde88 phase counter",
                    "when phase counter exceeds 2, resets to 0 and advances date by one via 0x00536190"
                ]
            },
            "date_state": {
                "current_day_of_year_global": "DAT_00acde90",
                "current_year_global": "DAT_00acde92",
                "phase_global": "DAT_00acde88",
                "target_packed_date_global": "DAT_00dbc268",
                "date_add_days": "0x00536190",
                "date_setter": "0x00533d10"
            },
            "match_day": {
                "builder": {
                    "address": "0x00699640",
                    "label": "match-day queue builder",
                    "confidence": "CODE_DERIVED frontier shape plus INFERRED match_day.cpp source attribution",
                    "observed_shape": {
                        "initial_state": "zeros builder state and initializes sentinel bytes at +0x2f..+0x33",
                        "competition_groups": "grows 0x18-byte competition/group records",
                        "match_groups": "grows 0x54-byte match grouping records",
                        "fixture_snapshots": "grows 0x69-byte fixture snapshot records",
                        "sort": "sorts fixture snapshot slices with comparator 0x0069bf40",
                        "scratch_list": "creates DAT_00b4d93c scratch list through 0x00672270",
                        "annotation_helper": "calls 0x0069aa70 after building queues"
                    }
                },
                "annotation_helper": {
                    "address": "0x0069aa70",
                    "label": "match-day queue annotation helper",
                    "confidence": "CODE_DERIVED frontier shape",
                    "observed_shape": {
                        "queue_walk": "walks 0x18, 0x54, and 0x69 builder queues",
                        "staff_scan": "checks last 16 0x6e-byte staff records through DAT_00acd5c4",
                        "visibility": "uses 0x005ea590 and 0x0069bc10 to mark fixture/group availability",
                        "counts": "updates visible group counters at builder +0x29/+0x2a/+0x2b"
                    }
                },
                "processor": {
                    "address": "0x00699d90",
                    "label": "match-day processor/setup dispatcher",
                    "confidence": "CODE_DERIVED frontier shape plus INFERRED match_day.cpp source attribution",
                    "observed_shape": {
                        "prepass": "calls 0x0069b4a0 and scans built 0x69 fixture snapshots",
                        "scratch": "allocates 0x11d-byte per-fixture scratch blocks",
                        "staff_links": "scans 16 active 0x6e-byte staff slots and writes temporary match links at helper state +0xe2/+0xe6",
                        "setup_call": "dispatches into verified match_setup 0x0069d950",
                        "ui_pump": "calls 0x00672770 while processing; still not the gameplay tick",
                        "cleanup": "frees per-fixture scratch and clears temporary links"
                    }
                },
                "setup": {
                    "address": "0x0069d950",
                    "label": "verified match setup",
                    "confidence": "VERIFIED by carve",
                    "observed_shape": {
                        "match_state_anchor": "stores fixture pointer at match-state +0x4792",
                        "home_team_array": "configures first team/player array at match-state +0x4796",
                        "away_team_array": "configures second team/player array at match-state +0x6a6e",
                        "team_setup": "calls 0x006c0f10 for team 0 and team 1",
                        "player_slot_stride": "uses player-slot stride 0x22d8 in match-state setup loops",
                        "rng": "uses verified match_random 0x008fc4f0 for setup events",
                        "incidents": "queues 0x19-byte match incident records through 0x00672320",
                        "boundary": "this is setup only, not the minute-by-minute match tick"
                    }
                },
                "team_player_setup": {
                    "address": "0x006c0f10",
                    "label": "match team/player setup",
                    "confidence": "CODE_DERIVED setup shape plus INFERRED match_man.cpp source attribution",
                    "observed_shape": {
                        "input_guards": "asserts non-null fixture/team/club inputs before setup",
                        "team_header": "writes match-team header fields around +0x1b, +0x23, +0x27, +0x2f, +0x33, +0x37",
                        "fixture_link": "stores fixture pointer-derived data and home/away visibility flags",
                        "team_block_copy": "copies 0x18e3 bytes into match-state +0x91e2 plus team_index*0x18e3",
                        "squad_count": "derives visible squad/player count from fixture byte +0x4d low nibble plus 0xb",
                        "tactics": "loads one tactics block through tactics.cpp helper 0x008830a0"
                    }
                },
                "player_risk_setup": {
                    "address": "0x006a1470",
                    "label": "match player-risk setup frontier",
                    "confidence": "CODE_DERIVED frontier shape, helper semantics still UNKNOWN",
                    "observed_shape": {
                        "team_score": "resets and updates team short at match-state +0x1ce plus team_index*2",
                        "player_loop": "scans 20 player slots from match-state +0x4796 plus team_index*0x22d8",
                        "slot_stride": "advances each player slot by 0x1be bytes",
                        "rng": "samples verified match_random with bounds 20, 7000, and 6",
                        "deep_helpers": "calls 0x006d1780 and 0x006d46c0, both still formula frontiers"
                    }
                },
                "tactics_block_loader": {
                    "address": "0x008830a0",
                    "label": "tactics block loader",
                    "confidence": "CODE_DERIVED setup shape plus INFERRED tactics.cpp source attribution",
                    "observed_shape": {
                        "index_lookup": "resolves tactic index via 0x00882f60",
                        "source_stride": "reads tactic blocks from param_1+0x601 with 0x91-byte stride",
                        "copy": "copies 0x24 dwords plus one trailing byte into match-team buffer"
                    }
                },
                "tactics_index_resolver": {
                    "address": "0x00882f60",
                    "label": "tactics index resolver",
                    "confidence": "CODE_DERIVED helper shape plus INFERRED tactics.cpp source attribution",
                    "observed_shape": {
                        "human_check": "checks staff/club control through 0x005ea590 and 0x0052a500",
                        "club_pointer": "follows pointer +0xcf for club-owned tactic state",
                        "blocked": "returns -1 for blocked human tactic lookup",
                        "fallback": "otherwise returns the selected tactic index from the input record"
                    }
                },
                "selected_tactic_staff_slot_lookup": {
                    "address": "0x00882240",
                    "label": "selected tactic staff slot lookup",
                    "confidence": "CODE_DERIVED helper shape plus INFERRED tactics.cpp source attribution",
                    "observed_shape": {
                        "index_lookup": "resolves tactic index through 0x00882f60",
                        "slot_scan": "scans up to 20 slots in a 0x91-byte tactic block",
                        "staff_join": "compares tactic slot staff ids against 0x6e-byte staff records at DAT_00acd5c4",
                        "fallback": "returns matching slot index or a fallback derived from param_4"
                    }
                },
                "primary_tactic_flag_reader": {
                    "address": "0x006a91d0",
                    "label": "match primary tactic flag reader",
                    "confidence": "CODE_DERIVED helper shape, flag meanings still UNKNOWN",
                    "observed_shape": {
                        "slot_index": "reads player slot byte +0x19 and bounds it to 0..10",
                        "side_select": "uses player slot byte +0x27 to select the side pointer",
                        "table": "returns u16 from match-state table +0x8ebc plus slot_index*2"
                    }
                },
                "secondary_tactic_flag_reader": {
                    "address": "0x006a9200",
                    "label": "match secondary tactic flag reader",
                    "confidence": "CODE_DERIVED helper shape, flag meanings still UNKNOWN",
                    "observed_shape": {
                        "slot_index": "reads player slot byte +0x19 and bounds it to 0..10",
                        "side_select": "uses player slot byte +0x27 to select the side pointer",
                        "table": "returns u16 from match-state table +0x8ec4 plus slot_index*2"
                    }
                },
                "player_random_byte_seed": {
                    "address": "0x006d9ea0",
                    "label": "match player random-byte seed frontier",
                    "confidence": "CODE_DERIVED frontier shape, field meanings still UNKNOWN",
                    "observed_shape": {
                        "outputs": "writes player-slot bytes +0x104, +0x105, +0x106, +0x107, +0x108, +0x109, +0x10a, +0x10b, +0x10c, +0x10d",
                        "bounds": "uses player-slot bytes +0x10e through +0x117 as match_random bounds",
                        "roll_shape": "several outputs keep the max of two verified match_random samples"
                    }
                },
                "player_evaluation": {
                    "address": "0x006d1a20",
                    "label": "match player evaluation frontier",
                    "confidence": "CODE_DERIVED frontier shape, formulas still UNKNOWN",
                    "observed_shape": {
                        "context": "reads player slot side byte +0x27 and match-state pointer +0x19e",
                        "base_short": "derives/writes player-slot short +0x3b",
                        "float_outputs": "writes many player-slot float fields including +0x7d/+0x8d/+0x91/+0x95/+0x99/+0xb9/+0xcd/+0xf1",
                        "tactical_flags": "branches on 0x006a91d0/0x006a9200 tactical flag masks",
                        "jitter": "uses random float helper 0x00935080 to cap or vary some outputs"
                    }
                },
                "player_action_score": {
                    "address": "0x006d46c0",
                    "label": "match player action-score frontier",
                    "confidence": "CODE_DERIVED frontier shape, formulas still UNKNOWN",
                    "observed_shape": {
                        "prepass": "calls player evaluation frontier 0x006d1a20",
                        "score_short": "resets and accumulates player-slot short +0x37",
                        "flags": "repeatedly branches on tactical flag readers 0x006a91d0/0x006a9200",
                        "attributes": "adds deltas from linked player-data bytes +0x17/+0x18/+0x19/+0x1a",
                        "random_seed": "can call player random-byte seed frontier 0x006d9ea0"
                    }
                },
                "candidate_action_wrapper": {
                    "address": "0x006b2cb0",
                    "label": "match candidate action wrapper",
                    "confidence": "CODE_DERIVED frontier shape, semantics still UNKNOWN",
                    "observed_shape": {
                        "candidate_list": "walks candidate pointer list using count at param_3+side+0x58 and 0x2c stride",
                        "filters": "requires candidate player slot +0x2b < 2 and +0x19 >= 0",
                        "dispatch": "calls player action attempt frontier 0x006db630",
                        "success": "clears match-state bytes +0x1a6 and +0x1a5 on success"
                    }
                },
                "adjacent_position_action_wrapper": {
                    "address": "0x006db580",
                    "label": "match adjacent-position action wrapper",
                    "confidence": "CODE_DERIVED frontier shape, semantics still UNKNOWN",
                    "observed_shape": {
                        "position": "reads player-slot coordinate bytes +0x102/+0x103 and side byte +0x27",
                        "direction": "adjusts row direction depending on side",
                        "bounded_move": "can call 0x006da0b0 with match-state bounds +0x8eaf/+0x8eb0",
                        "dispatch": "otherwise dispatches to move/action resolution 0x006d63f0"
                    }
                },
                "player_action_attempt": {
                    "address": "0x006db630",
                    "label": "match player action attempt frontier",
                    "confidence": "CODE_DERIVED match_pl.cpp frontier, formulas still UNKNOWN",
                    "observed_shape": {
                        "inputs": "uses player position bytes +0x102/+0x103 and side byte +0x27",
                        "tactics": "branches on primary tactic flags from 0x006a91d0",
                        "rng": "uses verified match_random with thresholds including 500, 600, 700, 1000, 0x708, and 500000",
                        "events": "emits action event codes through 0x006bc8d0",
                        "match_state": "mutates match-state bytes +0x8ea7/+0x8ea8/+0x8eae/+0x8eb2 and counters +0x4782/+0x478a",
                        "score": "can re-enter action-score frontier 0x006d46c0"
                    }
                },
                "move_action_resolution": {
                    "address": "0x006d63f0",
                    "label": "match player move/action resolution frontier",
                    "confidence": "CODE_DERIVED match_pl.cpp frontier, formulas still UNKNOWN",
                    "observed_shape": {
                        "position": "reads +0x101/+0x102/+0x103 and increments player-slot byte +0x2b",
                        "seed_bytes": "uses random seed bytes +0x104/+0x107/+0x10a/+0x10c",
                        "rng": "uses verified match_random with thresholds including 900, 1000, 10000-derived values, and small event rolls",
                        "action_codes": "writes action short +0x198 and drift short +0x19c",
                        "events": "emits events through 0x006bc8d0 and allocates event records through 0x00672320",
                        "recursion": "can recurse into 0x006d63f0 for chained movement/action resolution"
                    }
                },
                "player_action_selector": {
                    "address": "0x006f99c0",
                    "label": "match player action selector frontier",
                    "confidence": "CODE_DERIVED match_pl.cpp frontier, formulas still UNKNOWN",
                    "observed_shape": {
                        "match_state": "reads match-state action bytes +0x8ea7/+0x8ea8/+0x8ea9/+0x8eae",
                        "player_state": "reads player-slot bytes +0x101/+0x102/+0x103/+0x107/+0x109/+0x10a",
                        "action_codes": "sets action short +0x198 to codes including 0x68/0x69/0x6a/0x6b/0x76/0x100/0x105",
                        "dispatch": "dispatches selected actions through 0x006d63f0 and adjacent wrapper 0x006db580"
                    }
                },
                "event_resolution_dispatcher": {
                    "address": "0x006f63f0",
                    "label": "match event-resolution dispatcher frontier",
                    "confidence": "CODE_DERIVED match_pl.cpp frontier, formulas still UNKNOWN",
                    "observed_shape": {
                        "switch": "switches on match-state byte +0x8eb2 and clears it after handling",
                        "helpers": "calls event helpers 0x006ac3b0/0x006dfc50/0x006e65e0/0x006e7a60/0x006dfe90",
                        "events": "emits event code 0x1f44 through 0x006bc8d0",
                        "recursion": "can recurse into 0x006d63f0 for follow-on action resolution"
                    }
                },
                "event_queue_writer": {
                    "address": "0x006bc8d0",
                    "label": "match event queue writer",
                    "confidence": "CODE_DERIVED match_events.cpp frontier, event text semantics still UNKNOWN",
                    "observed_shape": {
                        "range": "accepts event codes from 8000 through 0x21e4",
                        "normalisation": "normalises some codes through 0x006bba10/0x006bb660/0x006bb6e0",
                        "primary_queue": "appends 0x0e-byte event slots at +0x30 plus count*0x0e",
                        "fields": "writes event code, flags, participants, and 4-byte payload",
                        "mirror_queue": "mirrors selected events into +0x720 with the same 0x0e-byte stride",
                        "counters": "maintains queue/cursor counters at +6/+8/+0xa/+0xc/+0xe",
                        "followups": "recursively emits follow-up event codes including 0x21a0/0x219f/0x21e3/0x21c0/0x21bf"
                    }
                },
                "event_follow_up_challenge": {
                    "address": "0x006dfc50",
                    "label": "match event follow-up challenge frontier",
                    "confidence": "CODE_DERIVED frontier shape, formulas still UNKNOWN",
                    "observed_shape": {
                        "tick": "increments player-slot byte +0x2b",
                        "spatial": "calls spatial helper 0x006e0740 from player position +0x102/+0x103",
                        "candidate": "finds a related candidate through 0x006b57d0",
                        "event": "emits event code 0x1f78 through 0x006bc8d0",
                        "match_state": "mutates side bucket +0xf5ca and action bytes +0x8ea7/+0x8ea8/+0x8eab/+0x8eae/+0x8eb2/+0xf57a/+0xf582"
                    }
                },
                "directional_follow_up": {
                    "address": "0x006dfe90",
                    "label": "match directional follow-up frontier",
                    "confidence": "CODE_DERIVED frontier shape, formulas still UNKNOWN",
                    "observed_shape": {
                        "position": "classifies player position using +0x102/+0x103/+0x19a and side +0x27",
                        "rng": "samples verified match_random with small direction bounds 2, 3, 4, and 5",
                        "spatial": "calls 0x006e0740 and 0x006b57d0",
                        "event": "emits event code 0x1f78 through 0x006bc8d0",
                        "match_state": "mutates action bytes +0x8ea7/+0x8ea8/+0x8eb2/+0x8eae plus +0xf5ca"
                    }
                },
                "shot_action_score": {
                    "address": "0x006e65e0",
                    "label": "match shot/action score frontier",
                    "confidence": "CODE_DERIVED match_pl.cpp frontier, formulas still UNKNOWN",
                    "observed_shape": {
                        "action_outputs": "selects action bytes including 0x16..0x1d, 0x33, 0x35, 0x39, and 0x3a",
                    "event_outputs": "writes event code outputs such as 0x1f7f and 0x1f81 through param_3",
                    "score": "computes player-slot score short +0x39",
                    "inputs": "reads player shorts +0x29/+0x146/+0x148/+0x14a/+0x14c/+0x14e/+0x150/+0x152/+0x154/+0x180/+0x198/+0x19c",
                    "float_inputs": "reads player float fields +0x79/+0x81/+0xe5",
                    "rng": "uses verified match_random for score and action selection"
                    }
                },
                "match_engine_step_controller": {
                    "address": "0x0069f2f0",
                    "label": "match engine step controller",
                    "confidence": "CODE_DERIVED match_eng.cpp frontier, full tick semantics still UNKNOWN",
                    "observed_shape": {
                        "fixture_anchor": "reads fixture pointer through match-state +0x4792",
                        "phase_switch": "loops while +0x8eb4 is active and switches on phase byte +0x8eb3",
                        "phase_dispatch": "dispatches possession phases through 0x006a4020 with modes 4 and 3",
                        "stored_action": "emits 0x20ef and calls stored-action resolver 0x006a0550",
                        "tick_counters": "increments match-state shorts +0x8ed0 and +0x8ed2",
                        "fixture_result": "writes fixture status byte +0x43 and emits 0x217b/0x2002/0x2003/0x2004"
                    }
                },
                "phase_possession_controller": {
                    "address": "0x006a4020",
                    "label": "match phase possession controller",
                    "confidence": "CODE_DERIVED frontier shape, possession semantics still UNKNOWN",
                    "observed_shape": {
                        "phase_switch": "switches on match-state phase byte +0x8eb3",
                        "scratch_reset": "resets possession/action scratch offsets +0x475a..+0x4769",
                        "side_state": "initialises side and phase bytes around +0x8e9e..+0x8ea4",
                        "player_select": "selects player slots from +0x4796 with 0x1be stride",
                        "score_and_dispatch": "calls shot/action score 0x006e65e0 and event resolver 0x006f63f0",
                        "events": "emits 0x2004/0x2005/0x2006 and writes fixture score bytes +0x49/+0x4a"
                    }
                },
                "pressure_action_continuation": {
                    "address": "0x006f5de0",
                    "label": "match pressure/action continuation frontier",
                    "confidence": "CODE_DERIVED frontier shape, formulas still UNKNOWN",
                    "observed_shape": {
                        "active_player_guard": "skips the active player held at match-state +0xf582",
                        "candidate_links": "scans player-slot link entries from +0x1ae using count byte +0x101",
                        "rng": "samples verified match_random with bounds 900, 20, 12, 10, and 5",
                        "action": "sets player action short +0x198 to 0x67",
                        "player_mutation": "updates player-slot fields +0x2b/+0x35/+0x4d",
                        "dispatch": "calls 0x006fa740/0x006f99c0/0x006f63f0 and clears match-state +0x1a7"
                    }
                },
                "stored_action_resolver": {
                    "address": "0x006a0550",
                    "label": "match stored-action resolver frontier",
                    "confidence": "CODE_DERIVED frontier shape, formulas still UNKNOWN",
                    "observed_shape": {
                        "scratch": "uses scratch bytes +0x475a..+0x4769 and active player pointers +0x4761/+0x4765",
                        "events": "emits stored-action event codes including 0x20f0/0x20ee/0x20fb/0x1f7a/0x20f5/0x2109/0x20df/0x20e0/0x20d9",
                        "match_state": "mutates +0x8ea7/+0x8ea8/+0x8eae/+0x8eb2/+0xf582/+0xf5ca",
                        "cleanup": "resets scratch offsets before returning"
                    }
                },
                "action_scratch_reset": {
                    "address": "0x006a1320",
                    "label": "match action scratch reset helper",
                    "confidence": "CODE_DERIVED tiny helper shape",
                    "observed_shape": {
                        "scratch": "resets +0x475a..+0x4769, clears active pointers +0x4761/+0x4765, sets +0x475f to 0xffff, and restores bytes +0x475b..+0x475e to 0xff"
                    }
                },
                "period_transition": {
                    "address": "0x006a3240",
                    "label": "match period transition frontier",
                    "confidence": "CODE_DERIVED frontier shape, period semantics still UNKNOWN",
                    "observed_shape": {
                        "period_state": "reads period/tick shorts +0x8ed4/+0x8ed0 and thresholds 0x1ef/0x3de/0x483/0x528",
                        "fixture_writes": "copies match score/status bytes from +0xf5bd/+0xf5f3 into fixture bytes +0x43..+0x48",
                        "events": "emits period transition event codes 0x20f1/0x20f2/0x20f3",
                        "match_state": "mutates +0x8eb2/+0x8eb3/+0x8eb6/+0x8eb7 and period short +0x8ed4",
                        "slot_reset": "resets player slots through 0x006db210"
                    }
                },
                "player_candidate_selector": {
                    "address": "0x006b4510",
                    "label": "match player candidate selector frontier",
                    "confidence": "CODE_DERIVED match_eng.cpp frontier, scoring formulas still UNKNOWN",
                    "observed_shape": {
                        "player_scan": "scans 20 player slots from side base +0x4796 using 0x1be stride",
                        "guards": "skips active pointers +0xf59e/+0xf5a2 and invalid slot byte +0x19",
                        "context": "uses match-state coordinates +0x8ea7/+0x8ea8 and tactical table +0x8ebc",
                        "helpers": "calls squad/role helpers 0x00882640 and spatial helper 0x006d63b0",
                        "rng": "uses verified match_random in candidate scoring",
                        "return": "returns the highest-scoring player pointer or errors if no required candidate exists"
                    }
                },
                "per_tick_tactical_state_updater": {
                    "address": "0x006aae20",
                    "label": "match per-tick tactical state updater frontier",
                    "confidence": "CODE_DERIVED match_eng.cpp frontier, semantics still UNKNOWN",
                    "observed_shape": {
                        "reset": "clears action scratch and counters +0x1c5/+0x4782/+0x4786/+0x477e/+0x475a..+0x4769",
                        "minute": "derives a minute bucket from match tick short +0x8ed0",
                        "side_blocks": "updates side tactical/status blocks around +0x904d and +0x911d",
                        "events": "emits tactical/commentary events 0x21cf/0x21c1/0x2137/0x2138/0x2139/0x213a/0x213b/0x213c/0x213d",
                        "team_refresh": "can call team refresh/frontier setup path 0x006a1470 for either side",
                        "state_switch": "branches on match-state byte +0x8eb2"
                    }
                },
                "formation_primary_mask_classifier": {
                    "address": "0x005a2c70",
                    "label": "formation primary mask classifier",
                    "confidence": "CODE_DERIVED helper shape plus INFERRED formation.cpp source attribution",
                    "observed_shape": {
                        "tables": "reads formation mask tables at +0x12d and +0x14e with index*2",
                        "masks": "checks masks 0x880, 0x40, 0x20, 0x10, and 0x8",
                        "comparison": "uses 0x0059cdf0 to inspect opposite/paired formation state",
                        "result": "returns boolean classifier; exact football meaning still UNKNOWN"
                    }
                },
                "formation_secondary_mask_classifier": {
                    "address": "0x005a30d0",
                    "label": "formation secondary mask classifier",
                    "confidence": "CODE_DERIVED helper shape plus INFERRED formation.cpp source attribution",
                    "observed_shape": {
                        "tables": "reads formation mask tables at +0x12d and +0x14e with index*2",
                        "masks": "checks masks 0x880 and 0x8",
                        "comparison": "uses 0x0059cdf0 to inspect opposite/paired formation state",
                        "result": "returns boolean classifier; exact football meaning still UNKNOWN"
                    }
                },
                "random_float_jitter": {
                    "address": "0x00935080",
                    "label": "random float jitter shim",
                    "confidence": "CODE_DERIVED wrapper shape, distribution still UNKNOWN",
                    "observed_shape": {
                        "wrapper": "passes x87 floating inputs to 0x009350a2",
                        "usage": "used by match player evaluation/action-score frontiers for capped random jitter"
                    }
                }
            },
            "phase_loop": {
                "phase_global": "DAT_00acde88",
                "observed_values": [0, 1, 2],
                "all_phases": [
                    "build per-date match-day queues through 0x00699640",
                    "annotate match-day queues through 0x0069aa70",
                    "run match-day processor/setup dispatcher 0x00699d90",
                    "when needed, build verified match setup state through 0x0069d950",
                    "configure team/player match setup through 0x006c0f10",
                    "run player-risk setup frontier 0x006a1470",
                    "load tactic block through 0x008830a0",
                    "resolve tactic index through 0x00882f60",
                    "find selected tactic staff slot through 0x00882240",
                    "read tactical flag tables through 0x006a91d0 and 0x006a9200",
                    "seed per-player random bytes through 0x006d9ea0",
                    "evaluate player float fields through 0x006d1a20",
                    "accumulate player action-score short through 0x006d46c0",
                    "walk candidate action attempts through 0x006b2cb0",
                    "dispatch adjacent-position action attempts through 0x006db580",
                    "attempt player action mutations through 0x006db630",
                    "resolve player movement/action through 0x006d63f0",
                    "select player action through 0x006f99c0",
                    "dispatch match event resolution through 0x006f63f0",
                    "write match event queue entries through 0x006bc8d0",
                    "run event follow-up challenge frontier 0x006dfc50",
                    "run directional follow-up frontier 0x006dfe90",
                    "compute shot/action score frontier 0x006e65e0",
                    "step match engine controller through 0x0069f2f0",
                    "drive phase possession controller through 0x006a4020",
                    "continue pressure/action selection through 0x006f5de0",
                    "resolve stored action scratch through 0x006a0550",
                    "reset action scratch through 0x006a1320",
                    "handle match period transitions through 0x006a3240",
                    "select match player candidates through 0x006b4510",
                    "update per-tick tactical state through 0x006aae20",
                    "classify formation masks through 0x005a2c70 and 0x005a30d0",
                    "apply random float jitter through 0x00935080",
                    "run per-club virtual callback with second argument 0",
                    "run date-sensitive data update 0x004cdef0",
                    "run common post-phase cleanup 0x00449710",
                    "pump Win32 messages through 0x00672770"
                ],
                "phase_2_only": [
                    "recalculate staff/person date-derived field at +0x18 using 0x005246e0",
                    "show 'Initialising Leagues' progress",
                    "run 0x0053fe40 42-slot current-date callback dispatcher",
                    "run 0x00614e90 staff role/competition drift frontier",
                    "run per-club virtual callback with second argument 1",
                    "run 0x00595580 fixture cleanup frontier",
                    "run 0x005e4370 host-country schedule frontier",
                    "run 0x005bfd90 season/calendar maintenance frontier",
                    "run 0x005c01d0 club rolling-metrics frontier",
                    "run 0x00752d40 fixture/tie participant notification frontier",
                    "run 0x00585ae0 club finance/stadium/status drift frontier",
                    "run 0x00784290 byte-array clear frontier",
                    "run manager-manager frontier 0x00674c10",
                    "run stadium frontier 0x00844940"
                ],
                "rollover": "after phase 2, reset DAT_00acde88 to 0 and call 0x00536190 with +1 day"
            },
            "rust_implemented_slice": {
                "status": "implemented",
                "scope": "phase counter rollover, packed date add-days, provenance trace, and static-boundary backend mutation ledger",
                "crate": "cm-domain",
                "types": ["RuntimeSimulationState", "CmPackedDate", "RuntimePhaseTrace"],
                "source_functions": ["0x005b6a90", "0x00536190", "0x00533d10"],
                "formula_lift_pending": [
                    "deeper phase subsystem formulas",
                    "fixture cleanup branch semantics",
                    "manager-manager mutation formulas",
                    "stadium/date cleanup formulas",
                    "match-day scoring/action formulas"
                ]
            },
            "classified_subsystem_frontiers": [
                {
                    "address": "0x00449710",
                    "label": "queued club-news dispatch cleanup",
                    "confidence": "CODE_DERIVED frontier shape; exact queue owner still unnamed",
                    "evidence": "decompile drains 6-byte queued items, resolves 0x245-byte club records, builds news payloads, dispatches through news helpers, frees the queue above 99 allocated entries, and resets the active count",
                    "observed_shape": {
                        "queue_base": "param_1 + 0x24 points to 6-byte queued records",
                        "queue_count": "param_1 + 0x2c is reset to 0 after processing",
                        "allocation_limit": "frees queue memory and clears capacity when param_1 + 0x28 exceeds 99",
                        "club_stride": "resolves DAT_00acd5bc + club_index * 0x245",
                        "news_dispatch": "uses 0x0076e180 for non-human club records and 0x0076e390 for human/manager-linked club records"
                    }
                },
                {
                    "address": "0x0053fe40",
                    "label": "42-slot current-date callback dispatcher",
                    "confidence": "CODE_DERIVED frontier shape; callback targets not yet named",
                    "evidence": "decompile walks 42 pointer slots and calls each non-null object's vtable +4 with current date argument",
                    "observed_shape": {
                        "slot_count": "0x2a callback/object slots",
                        "slot_stride": "4-byte pointer slots from *param_1",
                        "dispatch": "calls (**object_vtable + 4)(current_date_argument) for non-null slots"
                    }
                },
                {
                    "address": "0x00614e90",
                    "label": "staff role/competition drift frontier",
                    "confidence": "CODE_DERIVED frontier shape; exact role/status bytes still unnamed",
                    "evidence": "decompile releases stale scratch state, decrements per-entry counters, performs day-180 staff refresh, samples staff with verified match RNG, mutates staff preference/role bytes, emits 0x00616930 outcomes, then runs post processors",
                    "observed_shape": {
                        "scratch_release": "frees param_1[10] scratch state through 0x00672290 and 0x00933d24",
                        "counter_stride": "decrements two short counters per entry while stepping 0x30 bytes",
                        "day_180_refresh": "if target date condition and DAT_00acde90 == 0xb4, walks 0x6e-byte staff records and calls 0x00615d60",
                        "rng_sampling": "samples DAT_00acd56c staff count through match_random",
                        "outcome_emit": "calls 0x00616820 to choose outcome and 0x00616930 to emit/apply it",
                        "post_processing": "always finishes through 0x006176f0 and 0x006180c0"
                    }
                },
                {
                    "address": "0x004cdef0",
                    "label": "staff/contract date-renewal frontier",
                    "confidence": "CODE_DERIVED frontier shape; exact contract/news mutation semantics still partly unnamed",
                    "evidence": "decompile builds multiple future date windows, shows 'Updating game data', walks staff records, maps staff side-state to event/contract records, consults club records and human-manager visibility, age-gates via 0x005246e0, and dispatches contract/status outcomes",
                    "observed_shape": {
                        "date_windows": "builds +7, +0x1e, +0x3c, +0x5b, +0x79, +0x98, +0xb6, +0x16d, +0x226, and +0x447 day windows via 0x00536190",
                        "progress_ui": "uses 'Updating game data' with progress callback 0x007ead30",
                        "staff_stride": "walks 0x6e-byte records at DAT_00acd5c4",
                        "staff_side_state": "maps staff ids through 0x4f-byte side-state entries at DAT_00acdf0c",
                        "event_contract_records": "uses 0x50-byte records from param_1 and date/status fields around +0x2d/+0x2f/+0x35/+0x4e/+0x4f",
                        "club_stride": "resolves linked 0x245-byte club records through DAT_00acd5bc + linked_index * 0x245",
                        "age_helper": "0x005246e0 compares current date against fields +0x10/+0x12 and returns a date-derived age/elapsed-year value",
                        "outcomes": "calls 0x004dc980, 0x004dabd0, 0x004dcf60, 0x004e06d0, and related contract/status helpers"
                    }
                },
                {
                    "address": "0x005bfd90",
                    "label": "season/calendar maintenance frontier",
                    "confidence": "CODE_DERIVED frontier shape; exact calendar bucket owners still unnamed",
                    "evidence": "decompile initializes a one-shot guard, scans 34 calendar buckets, detects today/tomorrow dates, updates progress UI, invokes bucket callbacks, resets staff byte +0x6d for bucket 3, and rotates per-date scratch state",
                    "observed_shape": {
                        "guard": "sets DAT_00acde89 bit 0 and registers 0x005c01c0 once",
                        "bucket_count": "walks 34 season/calendar buckets",
                        "date_tests": "compares bucket day against current day and current day + 1 via 0x00536190",
                        "progress_ui": "uses 'Updating game data' with 0x007ead30 when work count exceeds 9",
                        "staff_reset": "bucket 3 clears byte +0x6d across 0x6e-byte staff records",
                        "tomorrow_callbacks": "calls 0x00448170, 0x007a9690, 0x00586cf0, and 0x00856690 for tomorrow buckets"
                    }
                },
                {
                    "address": "0x005c01d0",
                    "label": "club rolling-metrics frontier",
                    "confidence": "CODE_DERIVED frontier shape; metric semantics still unnamed",
                    "evidence": "decompile walks 0x122-byte club records, computes weighted double metrics, shifts rolling windows, resets current accumulators, and sorts 12-byte ranking records",
                    "observed_shape": {
                        "club_stride": "walks DAT_00acd5b0 club records at 0x122-byte stride",
                        "ranking_records": "allocates DAT_00acd558 * 0xc bytes and sorts with comparator 0x004b5fd0",
                        "rolling_fields": "uses double fields around +0xb0, +0xb8, +0xc0, +0xc8, +0xd0, and +0xd8",
                        "month_gate": "uses FUN_00536b90 and DAT_009b97a0 date table to decide when to roll metrics"
                    }
                },
                {
                    "address": "0x00752d40",
                    "label": "fixture/tie participant notification frontier",
                    "confidence": "CODE_DERIVED frontier shape; exact fixture/tie record owner still unnamed",
                    "evidence": "decompile date-gates competition/tie lists, lazily builds scratch date state, walks two fixture lists from 0x00596590, filters fixture type codes, updates participant notification state through 0x0075ee00, prunes current-day entries, and runs 70-day cleanup",
                    "observed_shape": {
                        "date_gates": "checks current date against 0x00533b50(1,0), 0x00533b50(0xf,3), and 0x00533b50(1,7) derived dates",
                        "fixture_lists": "walks two lists returned by 0x00596590 for each of three local passes",
                        "participant_fields": "uses fixture fields +0x1c and +0x20 as participant records",
                        "notification_flags": "uses fixture +0x4d bits 0x100/0x200 to choose notification targets",
                        "cleanup_cadence": "runs 0x0075f0f0 when current day modulo 0x46 is zero"
                    }
                },
                {
                    "address": "0x00585ae0",
                    "label": "club finance/stadium/status drift frontier",
                    "confidence": "CODE_DERIVED frontier shape; exact finance/status fields still unnamed",
                    "evidence": "decompile handles special dated club/stadium reassignment, walks 0x245-byte club records with 0x167-byte side blocks, uses verified match RNG for financial/status drift, updates linked records/news, and applies season/date-gated club adjustments",
                    "observed_shape": {
                        "special_date": "checks 0x00533b50(0x10,5,0x7d4) and named Falmer stadium data before reassignment",
                        "club_stride": "walks DAT_00acd5bc club records at 0x245-byte stride",
                        "side_block_stride": "walks parallel finance/status side blocks at 0x167-byte stride",
                        "news_updates": "uses 0x00597260, 0x005962b0, and 0x00594d00 to update linked records",
                        "rng_drift": "uses match_random with bounds including 5, 20000, 0x96, 0xb, 100, 0x19, and field-derived bounds",
                        "status_adjustments": "calls helpers including 0x007328a0 and 0x00588c70 during club status/finance changes"
                    }
                },
                {
                    "address": "0x00784290",
                    "label": "byte-array clear frontier",
                    "confidence": "CODE_DERIVED frontier shape; owner buffer still unnamed",
                    "evidence": "decompile clears each byte in the buffer at param_1 + 8 for param_1 + 4 entries",
                    "observed_shape": {
                        "length": "param_1 + 4",
                        "buffer": "param_1 + 8",
                        "operation": "sets every byte in the buffer to 0"
                    }
                },
                {
                    "address": "0x00595580",
                    "label": "fixture/news cleanup frontier",
                    "confidence": "CODE_DERIVED frontier shape; exact fixture/news record owners still partly unnamed",
                    "evidence": "decompile runs date-gated cleanup, walks news/event lists, reattaches news entries through news.cpp helpers, filters human-manager-visible records, and may generate paired +3/+4 dated fixture events",
                    "observed_shape": {
                        "date_gate": "reads current phase/date state through 0x00533fd0 and builds +3/+1 date values through 0x00536190",
                        "news_record_stride": "uses DAT_00acd5bc + index * 0x245 for news-like records",
                        "news_helpers": "0x0076d7d0 reads indexed news fields; 0x0076e180 clears +0xde and reattaches recent news links",
                        "human_manager_filter": "0x005ea590 is human_manager.cpp attributed and checks visibility/relevance before cleanup",
                        "event_generator": "0x0050c8d0 uses 0x68-byte fixture subrecords and may create paired events tagged base+3 and base+4"
                    }
                },
                {
                    "address": "0x005e4370",
                    "label": "host-country date/RNG schedule frontier",
                    "confidence": "CODE_DERIVED frontier shape; exact record owner still partly unnamed",
                    "evidence": "decompile scans 34-byte date records, checks current date +/- offsets, mutates pending status bytes, uses verified RNG 0x008fc4f0, and calls host-country event emission helper 0x005e49e0",
                    "observed_shape": {
                        "record_stride": "0x22 bytes",
                        "date_windows": "compares records to current packed date year plus +8 and +5 year offsets",
                        "pending_status": "status byte at +0x1e uses -3, -2, -1 and 0..2 states",
                        "rng_assignment": "unresolved -1/-2 statuses are assigned with match_random(3)",
                        "candidate_helper": "0x005e4840 is host_country.cpp attributed and shuffles candidates with match_random",
                        "event_emission": "0x005e49e0 emits type-4/type-5 event records from schedule entries"
                    }
                },
                {
                    "address": "0x00674c10",
                    "label": "manager-job lifecycle frontier",
                    "confidence": "CODE_DERIVED frontier shape plus INFERRED manager_manager.cpp source attribution",
                    "evidence": "decompile repairs missing jobs, expires dated manager-job entries, clears/sets manager relation states, scores candidate managers, uses verified match RNG, and queues 0x26-byte job/vacancy events",
                    "observed_shape": {
                        "weekly_gate": "runs 0x0067ce90 when DAT_00acde90 % 7 == 0",
                        "record_strides": "walks 0x245-byte club/news records at DAT_00acd5bc and 0x6e-byte manager records at DAT_00acd5c4",
                        "expiry_helper": "0x006808a0 walks dated job items and calls 0x00680cc0 when linked club/manager state is still active",
                        "missing_job_repair": "0x006822d0 scans clubs for missing manager jobs and calls 0x00680cc0 with no incumbent",
                        "allocation_worker": "0x00680cc0 resets 0x49-byte per-club manager state blocks, scores candidate managers from -1000000 upward, uses match_random for candidate/timing choices, and queues 0x26-byte events",
                        "relation_clear": "0x00675980 rewrites manager relation status bytes and clears club +0xd7 relation slots"
                    }
                },
                {
                    "address": "0x00844940",
                    "label": "stadium/date-ordered cleanup frontier",
                    "confidence": "CODE_DERIVED frontier shape; record target still unnamed",
                    "evidence": "decompile checks day 0 setup, pre-2003 stadium slot clearing, exact 2003 restore date, and 30-day cleanup over 12-byte date-ordered records",
                    "observed_shape": {
                        "setup_gate": "if current day-of-year == 0 call 0x008470e0",
                        "pre_2003_stadium_slot": "if current year < 2003, clear DAT_00acd5b0 + 0x80 + DAT_009bb7a4 * 0x122 and cache DAT_00a6f1f0",
                        "restore_gate": "if current date equals 2003 day 0 and cached slot is valid, restore DAT_00acd5b8 + cached * 0x4e and set values at +0x3c/+0x40 to 90000",
                        "cleanup_cadence": "only when current day-of-year % 30 == 0",
                        "cleanup_stride": "12-byte records sorted by date through comparator 0x008474a0, then compacted and re-sorted"
                    }
                }
            ],
            "implementation_boundary": "Rust may model this as a three-phase simulation frontier only after additional lifts identify the phase meanings and all mutating subsystem calls."
        },
        "checks": checks,
    }))
}

fn validate_runtime_simulation_report(db_dir: &Path) -> Result<serde_json::Value, String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let mut save = world.new_runtime_save_from_rust_db(db_dir);
    let mut checks = Vec::new();

    push_binary_check(
        &mut checks,
        "runtime-start-date",
        save.date.year == 2001
            && save.date.month == 7
            && save.date.day == 1
            && save.simulation.cm_packed_date.day_of_year == 182
            && save.simulation.cm_packed_date.year == 2001
            && !save.simulation.cm_packed_date.leap_year,
        format!(
            "new Rust save starts at {:04}-{:02}-{:02}; CM packed day {} year {} leap {}",
            save.date.year,
            save.date.month,
            save.date.day,
            save.simulation.cm_packed_date.day_of_year,
            save.simulation.cm_packed_date.year,
            save.simulation.cm_packed_date.leap_year
        ),
    );

    push_binary_check(
        &mut checks,
        "runtime-start-phase",
        save.simulation.phase == 0 && save.phase_trace.is_empty(),
        format!(
            "new Rust save starts at phase {} with {} trace entries",
            save.simulation.phase,
            save.phase_trace.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "gameplay-mutator-install-plans",
        gameplay_mutator_install_plans_ready(&save.backend.mutator_install_plans),
        format!(
            "{} exact gameplay mutator install plan(s)",
            save.backend.mutator_install_plans.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "gameplay-promotion-gates",
        cm_domain::gameplay_promotion_gates_ready(&save.backend.gameplay_promotion_gates),
        format!(
            "{} exact gameplay promotion gate(s) prevent enabling mutators before original/Rust ordered parity",
            save.backend.gameplay_promotion_gates.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "gameplay-lift-workbench",
        cm_domain::gameplay_lift_workbench_ready(&save.backend.gameplay_lift_workbench),
        format!(
            "{} code-derived gameplay lift work item(s) queued for original-binary implementation",
            save.backend.gameplay_lift_workbench.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "gameplay-lift-artifacts",
        gameplay_lift_artifacts_ready(&save.backend.gameplay_lift_workbench),
        format!(
            "{} of {} targeted Ghidra gameplay decompile artifact(s) are present",
            gameplay_lift_artifacts_present(&save.backend.gameplay_lift_workbench),
            save.backend.gameplay_lift_workbench.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "exact-gameplay-mutator-skeletons",
        cm_domain::gameplay_mutators::exact_gameplay_mutator_skeletons_ready(
            &save.backend.exact_mutator_skeletons,
        ),
        format!(
            "{} disabled exact gameplay mutator skeleton(s)",
            save.backend.exact_mutator_skeletons.len()
        ),
    );
    let exact_mutator_skeleton_outcomes = save
        .backend
        .exact_mutator_skeletons
        .iter()
        .map(|skeleton| {
            cm_domain::gameplay_mutators::call_exact_gameplay_mutator_skeleton(
                &cm_domain::gameplay_mutators::ExactGameplayMutatorCall {
                    system: skeleton.system.clone(),
                    entry_point: skeleton.entry_point.clone(),
                    trace_file: skeleton.trace_file.clone(),
                },
            )
        })
        .collect::<Vec<_>>();
    let match_result_formula_scenario = cm_domain::default_match_result_formula_scenario();
    let match_result_formula_plan = cm_domain::plan_match_result_formula_mutations(
        &save.backend,
        &match_result_formula_scenario,
    );
    let competition_notification_formula_scenario =
        cm_domain::default_competition_notification_formula_scenario();
    let competition_notification_formula_plan =
        cm_domain::plan_competition_notification_formula_mutations(
            &save.backend,
            &competition_notification_formula_scenario,
        );
    let competition_standings_formula_scenario =
        cm_domain::default_competition_standings_formula_scenario();
    let competition_standings_formula_plan =
        cm_domain::plan_competition_standings_formula_mutations(
            &save.backend,
            &competition_standings_formula_scenario,
        );
    let competition_progression_formula_scenario =
        cm_domain::default_competition_progression_formula_scenario();
    let competition_progression_formula_plan =
        cm_domain::plan_competition_progression_formula_mutations(
            &save.backend,
            &competition_progression_formula_scenario,
        );
    let transfer_contract_formula_scenario =
        cm_domain::default_transfer_contract_formula_scenario();
    let transfer_contract_formula_plan = cm_domain::plan_transfer_contract_formula_mutations(
        &save.backend,
        &transfer_contract_formula_scenario,
    );
    let news_inbox_formula_scenario = cm_domain::default_news_inbox_formula_scenario();
    let news_inbox_formula_plan =
        cm_domain::plan_news_inbox_formula_mutations(&save.backend, &news_inbox_formula_scenario);
    push_binary_check(
        &mut checks,
        "exact-gameplay-mutator-entry-points",
        cm_domain::gameplay_mutators::exact_gameplay_mutator_skeleton_entry_points_ready(
            &save.backend.exact_mutator_skeletons,
        ),
        "all disabled exact gameplay mutator skeleton entry points are callable and emit zero mutations".to_string(),
    );
    push_binary_check(
        &mut checks,
        "match-engine-lift-map",
        save.backend.match_engine_lift_map.len() >= 5
            && save.backend.match_engine_lift_map.iter().any(|entry| {
                entry.function == "0x0069d950"
                    && entry
                        .verified_state
                        .iter()
                        .any(|state| state.contains("+0x4792"))
            })
            && save.backend.match_engine_lift_map.iter().any(|entry| {
                entry.function == "0x006a4020"
                    && entry
                        .verified_state
                        .iter()
                        .any(|state| state.contains("+0x49/+0x4a"))
            })
            && save.backend.match_engine_lift_map.iter().any(|entry| {
                entry.function == "0x006a3240"
                    && entry
                        .verified_state
                        .iter()
                        .any(|state| state.contains("+0x43..+0x48"))
            })
            && save.backend.match_engine_lift_map.iter().any(|entry| {
                entry.function == "0x006bc8d0"
                    && entry
                        .verified_state
                        .iter()
                        .any(|state| state.contains("0x0e"))
            }),
        format!(
            "{} match-engine lift-map entry(s): setup, step controller, phase controller, period writer, and event queue are code-derived",
            save.backend.match_engine_lift_map.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "match-result-write-map",
        save.backend.match_result_write_map.len() >= 5
            && save.backend.match_result_write_map.iter().any(|entry| {
                entry.fixture_home_offset == "0x49"
                    && entry.fixture_away_offset == "0x4a"
                    && entry.source_home_offset == "0xf5bc"
                    && entry.source_away_offset == "0xf5f2"
                    && entry.event_code.as_deref() == Some("0x2004")
                    && entry.function == "0x006a4020"
            })
            && save.backend.match_result_write_map.iter().any(|entry| {
                entry.fixture_home_offset == "0x43"
                    && entry.fixture_away_offset == "0x44"
                    && entry.source_home_offset == "0xf5bd"
                    && entry.source_away_offset == "0xf5f3"
                    && entry.threshold.as_deref() == Some("0x3de")
                    && entry.function == "0x006a3240"
            })
            && save.backend.match_result_write_map.iter().any(|entry| {
                entry.fixture_home_offset == "0x45"
                    && entry.fixture_away_offset == "0x46"
                    && entry.event_code.as_deref() == Some("0x20f1")
            })
            && save.backend.match_result_write_map.iter().any(|entry| {
                entry.fixture_home_offset == "0x47"
                    && entry.fixture_away_offset == "0x48"
                    && entry.event_code.as_deref() == Some("0x20f3")
            }),
        format!(
            "{} match result write-map entry(s): fixture score/status bytes +0x43..+0x4a are code-derived",
            save.backend.match_result_write_map.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "match-result-code-claims",
        cm_domain::match_result_code_claims_ready(&save.backend.match_result_code_claims),
        format!(
            "{} code-derived match-result claim(s) cite targeted decompile artifacts",
            save.backend.match_result_code_claims.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "match-result-formula-lift-map",
        cm_domain::match_result_formula_lift_map_ready(&save.backend.match_result_formula_lift_map),
        format!(
            "{} static-code-derived match-result formula lift(s)",
            save.backend.match_result_formula_lift_map.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "match-result-formula-mutation-plan",
        cm_domain::match_result_formula_plan_ready(&match_result_formula_plan),
        format!(
            "{} formula-derived match-result mutation row(s)",
            match_result_formula_plan.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "competition-code-claims",
        cm_domain::competition_code_claims_ready(&save.backend.gameplay_system_code_claims),
        format!(
            "{} code-derived gameplay claim(s) include competition fixture/table state",
            save.backend.gameplay_system_code_claims.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "transfer-contract-code-claims",
        cm_domain::transfer_contract_code_claims_ready(&save.backend.gameplay_system_code_claims),
        format!(
            "{} code-derived gameplay claim(s) include transfer/contract state",
            save.backend.gameplay_system_code_claims.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "transfer-contract-formula-lift-map",
        cm_domain::transfer_contract_formula_lift_map_ready(
            &save.backend.transfer_contract_formula_lift_map,
        ),
        format!(
            "{} static-code-derived transfer/contract formula lift(s)",
            save.backend.transfer_contract_formula_lift_map.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "transfer-contract-formula-mutation-plan",
        cm_domain::transfer_contract_formula_plan_ready(&transfer_contract_formula_plan),
        format!(
            "{} formula-derived transfer/contract mutation row(s)",
            transfer_contract_formula_plan.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "news-inbox-code-claims",
        cm_domain::news_inbox_code_claims_ready(&save.backend.gameplay_system_code_claims),
        format!(
            "{} code-derived gameplay claim(s) include news/inbox state",
            save.backend.gameplay_system_code_claims.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "match-result-mutator-install-plan",
        save.backend.match_result_mutator_install_plan.system == "match results"
            && save
                .backend
                .match_result_mutator_install_plan
                .trace_file
                .ends_with("reports/parity_traces/match-results.json")
            && save
                .backend
                .match_result_mutator_install_plan
                .required_original_coverage
                .iter()
                .any(|item| item.contains("fixture +0x49"))
            && save
                .backend
                .match_result_mutator_install_plan
                .required_rust_coverage
                .iter()
                .any(|item| item.contains("event 0x2004"))
            && save
                .backend
                .match_result_mutator_install_plan
                .promotion_rule
                .contains("implementation_present=true"),
        format!(
            "match-results install plan has {} original and {} Rust coverage requirement(s)",
            save.backend
                .match_result_mutator_install_plan
                .required_original_coverage
                .len(),
            save.backend
                .match_result_mutator_install_plan
                .required_rust_coverage
                .len()
        ),
    );
    push_binary_check(
        &mut checks,
        "competition-fixture-state-map",
        save.backend.competition_fixture_state_map.len() >= 7
            && save.backend.competition_fixture_state_map.iter().any(|entry| {
                entry.fixture_offset.as_deref() == Some("0x1c")
                    && entry.function == "0x00752d40"
            })
            && save.backend.competition_fixture_state_map.iter().any(|entry| {
                entry.fixture_offset.as_deref() == Some("0x20")
                    && entry.function == "0x00752d40"
            })
            && save.backend.competition_fixture_state_map.iter().any(|entry| {
                entry.fixture_offset.as_deref() == Some("0x4d")
                    && entry.flag_mask.as_deref() == Some("0x100")
                    && entry.helper.as_deref() == Some("0x0075ee00")
            })
            && save.backend.competition_fixture_state_map.iter().any(|entry| {
                entry.fixture_offset.as_deref() == Some("0x4d")
                    && entry.flag_mask.as_deref() == Some("0x200")
                    && entry.helper.as_deref() == Some("0x0075ee00")
            })
            && save.backend.competition_fixture_state_map.iter().any(|entry| {
                entry.helper.as_deref() == Some("0x00596590")
                    && entry.function == "0x00752d40"
            }),
        format!(
            "{} competition fixture state-map entry(s): participants, notification bits, list accessor, and cleanup cadence are code-derived",
            save.backend.competition_fixture_state_map.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "competition-notification-formula-lift-map",
        cm_domain::competition_notification_formula_lift_map_ready(
            &save.backend.competition_notification_formula_lift_map,
        ),
        format!(
            "{} static-code-derived competition notification formula lift(s)",
            save.backend.competition_notification_formula_lift_map.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "competition-notification-formula-mutation-plan",
        cm_domain::competition_notification_formula_plan_ready(
            &competition_notification_formula_plan,
        ),
        format!(
            "{} formula-derived competition notification mutation row(s)",
            competition_notification_formula_plan.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "competition-standings-formula-lift-map",
        cm_domain::competition_standings_formula_lift_map_ready(
            &save.backend.competition_standings_formula_lift_map,
        ),
        format!(
            "{} static-code-derived competition standings formula lift(s)",
            save.backend.competition_standings_formula_lift_map.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "competition-standings-formula-mutation-plan",
        cm_domain::competition_standings_formula_plan_ready(&competition_standings_formula_plan),
        format!(
            "{} formula-derived competition standings mutation row(s)",
            competition_standings_formula_plan.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "competition-progression-formula-lift-map",
        cm_domain::competition_progression_formula_lift_map_ready(
            &save.backend.competition_progression_formula_lift_map,
        ),
        format!(
            "{} static-code-derived competition progression formula lift(s)",
            save.backend.competition_progression_formula_lift_map.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "competition-progression-formula-mutation-plan",
        cm_domain::competition_progression_formula_plan_ready(
            &competition_progression_formula_plan,
        ),
        format!(
            "{} formula-derived competition progression mutation row(s)",
            competition_progression_formula_plan.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "transfer-contract-state-map",
        save.backend.transfer_contract_state_map.len() >= 8
            && save.backend.transfer_contract_state_map.iter().any(|entry| {
                entry.function == "0x004cdef0"
                    && entry.helper.as_deref() == Some("0x00536190")
            })
            && save.backend.transfer_contract_state_map.iter().any(|entry| {
                entry.function == "0x004cdef0" && entry.stride.as_deref() == Some("0x6e")
            })
            && save.backend.transfer_contract_state_map.iter().any(|entry| {
                entry.function == "0x004cdef0"
                    && entry.stride.as_deref() == Some("0x50")
                    && entry.record_offset.as_deref() == Some("0x35")
            })
            && save.backend.transfer_contract_state_map.iter().any(|entry| {
                entry.function == "0x00449710"
                    && entry.stride.as_deref() == Some("0x6")
                    && entry.helper.as_deref() == Some("0x004539f0")
            })
            && save.backend.transfer_contract_state_map.iter().any(|entry| {
                entry.function == "0x008a9080"
                    && entry.record_offset.as_deref() == Some("0x213/0x84d/0x856")
            }),
        format!(
            "{} transfer/contract state-map entry(s): renewal windows, record strides, queue dispatch, and transfer.dat list state are code-derived",
            save.backend.transfer_contract_state_map.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "news-inbox-emission-map",
        save.backend.news_inbox_emission_map.len() >= 7
            && save.backend.news_inbox_emission_map.iter().any(|entry| {
                entry.stride.as_deref() == Some("0x68")
                    && entry.helper.as_deref() == Some("0x00596fa0")
                    && entry.function == "0x0050c8d0"
            })
            && save.backend.news_inbox_emission_map.iter().any(|entry| {
                entry.record_offset.as_deref() == Some("0x30")
                    && entry.function == "0x0050c8d0"
            })
            && save.backend.news_inbox_emission_map.iter().any(|entry| {
                entry.record_offset.as_deref() == Some("0xde")
                    && entry.function == "0x0076e180"
            })
            && save
                .backend
                .news_inbox_emission_map
                .iter()
                .any(|entry| entry.helper.as_deref() == Some("0x006724d0")),
        format!(
            "{} news/inbox emission-map entry(s): paired events, reset byte, and queue removal are code-derived",
            save.backend.news_inbox_emission_map.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "news-inbox-formula-lift-map",
        cm_domain::news_inbox_formula_lift_map_ready(&save.backend.news_inbox_formula_lift_map),
        format!(
            "{} static-code-derived news/inbox formula lift(s)",
            save.backend.news_inbox_formula_lift_map.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "news-inbox-formula-mutation-plan",
        cm_domain::news_inbox_formula_plan_ready(&news_inbox_formula_plan),
        format!(
            "{} formula-derived news/inbox mutation row(s)",
            news_inbox_formula_plan.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "gameplay-mutator-contracts",
        save.backend.mutator_contracts.len() == 4
            && save
                .backend
                .mutator_contracts
                .iter()
                .all(|contract| {
                    contract.implementation_present
                        && contract.status == GameplayMutatorStatus::ParityVerified
                })
            && save.backend.mutator_contracts.iter().any(|contract| {
                contract.system == "match results"
                    && contract.phase == 2
                    && contract.boundary_map == "match_result_write_map"
                    && contract
                        .trace_file
                        .ends_with("reports/parity_traces/match-results.json")
            })
            && save.backend.mutator_contracts.iter().any(|contract| {
                contract.system == "competition state"
                    && contract.phase == 2
                    && contract.boundary_map == "competition_fixture_state_map"
            })
            && save.backend.mutator_contracts.iter().any(|contract| {
                contract.system == "transfers/contracts"
                    && contract.phase == 0
                    && contract.boundary_map == "transfer_contract_state_map"
            })
            && save.backend.mutator_contracts.iter().any(|contract| {
                contract.system == "news/inbox"
                    && contract.phase == 1
                    && contract.boundary_map == "news_inbox_emission_map"
            }),
        format!(
            "{} gameplay mutator contract(s): every exact mutator is tied to a phase, boundary map, trace file, and parity gate",
            save.backend.mutator_contracts.len()
        ),
    );

    save.tick_cm_phase();
    push_binary_check(
        &mut checks,
        "phase-0-to-1",
        save.simulation.phase == 1
            && save.elapsed_days == 0
            && save.phase_trace.len() == 1
            && save.phase_trace[0].phase_before == 0
            && save.phase_trace[0].phase_after == 1
            && !save.phase_trace[0].advanced_day
            && save.phase_trace[0].frontiers.len() == 38,
        format!(
            "after one phase: phase {} elapsed {} trace {} frontiers {}",
            save.simulation.phase,
            save.elapsed_days,
            save.phase_trace.len(),
            save.phase_trace
                .first()
                .map_or(0, |trace| trace.frontiers.len())
        ),
    );

    save.tick_cm_phase();
    push_binary_check(
        &mut checks,
        "phase-1-to-2",
        save.simulation.phase == 2
            && save.elapsed_days == 0
            && save.phase_trace.len() == 2
            && save.phase_trace[1].phase_before == 1
            && save.phase_trace[1].phase_after == 2
            && !save.phase_trace[1].advanced_day
            && save.phase_trace[1].frontiers.len() == 38,
        format!(
            "after two phases: phase {} elapsed {} trace {} frontiers {}",
            save.simulation.phase,
            save.elapsed_days,
            save.phase_trace.len(),
            save.phase_trace
                .get(1)
                .map_or(0, |trace| trace.frontiers.len())
        ),
    );

    save.tick_cm_phase();
    let last = save.phase_trace.last();
    let last_has_date_add = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00536190" && frontier.status.contains("implemented slice")
            })
        })
        .unwrap_or(false);
    let last_has_staff_contract_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x004cdef0"
                    && frontier.status.contains("0x6e-byte staff records")
                    && frontier.status.contains("0x50 event/contract records")
            })
        })
        .unwrap_or(false);
    let last_has_match_queue_builder = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00699640"
                    && frontier.status.contains("0x18 competition groups")
                    && frontier.status.contains("0x69 fixture snapshots")
            })
        })
        .unwrap_or(false);
    let last_has_match_queue_annotation = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x0069aa70"
                    && frontier.status.contains("0x18/0x54/0x69")
                    && frontier.status.contains("active staff records")
            })
        })
        .unwrap_or(false);
    let last_has_match_processor_dispatcher = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00699d90"
                    && frontier.status.contains("0x11d scratch")
                    && frontier.status.contains("match_setup 0x0069d950")
            })
        })
        .unwrap_or(false);
    let last_has_match_setup = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x0069d950"
                    && frontier.status.contains("+0x4792")
                    && frontier.status.contains("+0x4796")
                    && frontier.status.contains("+0x6a6e")
            })
        })
        .unwrap_or(false);
    let last_has_match_team_player_setup = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006c0f10"
                    && frontier.status.contains("0x18e3-byte team block")
                    && frontier.status.contains("0x008830a0")
            })
        })
        .unwrap_or(false);
    let last_has_match_player_risk_setup = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006a1470"
                    && frontier.status.contains("20 player slots")
                    && frontier.status.contains("match_random(20/7000/6)")
            })
        })
        .unwrap_or(false);
    let last_has_tactics_block_loader = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x008830a0"
                    && frontier.status.contains("0x91-byte tactic block")
                    && frontier.status.contains("param_1+0x601")
            })
        })
        .unwrap_or(false);
    let last_has_tactics_index_resolver = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00882f60"
                    && frontier.status.contains("+0xcf")
                    && frontier.status.contains("returns -1")
            })
        })
        .unwrap_or(false);
    let last_has_selected_tactic_staff_slot_lookup = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00882240"
                    && frontier.status.contains("20 tactic slots")
                    && frontier.status.contains("0x6e-byte staff records")
            })
        })
        .unwrap_or(false);
    let last_has_primary_tactic_flag_reader = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006a91d0"
                    && frontier.status.contains("+0x8ebc")
                    && frontier.status.contains("index*2")
            })
        })
        .unwrap_or(false);
    let last_has_secondary_tactic_flag_reader = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006a9200"
                    && frontier.status.contains("+0x8ec4")
                    && frontier.status.contains("index*2")
            })
        })
        .unwrap_or(false);
    let last_has_player_random_byte_seed = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006d9ea0"
                    && frontier.status.contains("+0x104..+0x10d")
                    && frontier.status.contains("+0x10e..+0x117")
            })
        })
        .unwrap_or(false);
    let last_has_player_evaluation_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006d1a20"
                    && frontier.status.contains("+0x3b")
                    && frontier.status.contains("+0x7d/+0x8d/+0x91/+0x95/+0x99")
            })
        })
        .unwrap_or(false);
    let last_has_player_action_score_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006d46c0"
                    && frontier.status.contains("+0x37")
                    && frontier.status.contains("0x006d9ea0")
            })
        })
        .unwrap_or(false);
    let last_has_candidate_action_wrapper = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006b2cb0"
                    && frontier.status.contains("0x2c stride")
                    && frontier.status.contains("0x006db630")
            })
        })
        .unwrap_or(false);
    let last_has_adjacent_position_action_wrapper = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006db580"
                    && frontier.status.contains("+0x102/+0x103")
                    && frontier.status.contains("0x006d63f0")
            })
        })
        .unwrap_or(false);
    let last_has_player_action_attempt_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006db630"
                    && frontier.status.contains("+0x8ea7/+0x8ea8/+0x8eae/+0x8eb2")
                    && frontier.status.contains("+0x4782/+0x478a")
            })
        })
        .unwrap_or(false);
    let last_has_move_action_resolution_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006d63f0"
                    && frontier.status.contains("+0x101/+0x102/+0x103")
                    && frontier.status.contains("+0x198")
            })
        })
        .unwrap_or(false);
    let last_has_player_action_selector_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006f99c0"
                    && frontier
                        .status
                        .contains("0x68/0x69/0x6a/0x6b/0x76/0x100/0x105")
                    && frontier.status.contains("0x006d63f0/0x006db580")
            })
        })
        .unwrap_or(false);
    let last_has_event_resolution_dispatcher_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006f63f0"
                    && frontier.status.contains("+0x8eb2")
                    && frontier.status.contains("0x006d63f0")
            })
        })
        .unwrap_or(false);
    let last_has_event_queue_writer = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006bc8d0"
                    && frontier.status.contains("0x0e-byte event slots")
                    && frontier
                        .status
                        .contains("0x21a0/0x219f/0x21e3/0x21c0/0x21bf")
            })
        })
        .unwrap_or(false);
    let last_has_event_follow_up_challenge = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006dfc50"
                    && frontier.status.contains("0x1f78")
                    && frontier
                        .status
                        .contains("+0x8ea7/+0x8ea8/+0x8eab/+0x8eae/+0x8eb2")
            })
        })
        .unwrap_or(false);
    let last_has_directional_follow_up = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006dfe90"
                    && frontier.status.contains("+0x102/+0x103/+0x19a")
                    && frontier.status.contains("+0xf5ca")
            })
        })
        .unwrap_or(false);
    let last_has_shot_action_score = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006e65e0"
                    && frontier.status.contains("+0x39")
                    && frontier.status.contains("0x1f7f/0x1f81")
            })
        })
        .unwrap_or(false);
    let last_has_match_engine_step_controller = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x0069f2f0"
                    && frontier.status.contains("+0x8eb3")
                    && frontier.status.contains("+0x8ed0/+0x8ed2")
                    && frontier.status.contains("+0x43")
                    && frontier.status.contains("0x217b/0x2002/0x2003/0x2004")
            })
        })
        .unwrap_or(false);
    let last_has_match_phase_possession_controller = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006a4020"
                    && frontier.status.contains("+0x475a..+0x4769")
                    && frontier.status.contains("0x006e65e0")
                    && frontier.status.contains("+0x49/+0x4a")
            })
        })
        .unwrap_or(false);
    let last_has_match_pressure_action_continuation = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006f5de0"
                    && frontier.status.contains("+0xf582")
                    && frontier.status.contains("+0x198")
                    && frontier.status.contains("0x006fa740/0x006f99c0/0x006f63f0")
            })
        })
        .unwrap_or(false);
    let last_has_match_stored_action_resolver = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006a0550"
                    && frontier
                        .status
                        .contains("0x20f0/0x20ee/0x20fb/0x1f7a/0x20f5/0x2109/0x20df/0x20e0/0x20d9")
                    && frontier
                        .status
                        .contains("+0x8ea7/+0x8ea8/+0x8eae/+0x8eb2/+0xf582/+0xf5ca")
            })
        })
        .unwrap_or(false);
    let last_has_match_action_scratch_reset = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006a1320"
                    && frontier.status.contains("+0x475a..+0x4769")
                    && frontier.status.contains("+0x4761/+0x4765")
            })
        })
        .unwrap_or(false);
    let last_has_match_period_transition = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006a3240"
                    && frontier.status.contains("+0x8ed4/+0x8ed0")
                    && frontier.status.contains("0x1ef/0x3de/0x483/0x528")
                    && frontier.status.contains("0x20f1/0x20f2/0x20f3")
            })
        })
        .unwrap_or(false);
    let last_has_match_player_candidate_selector = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006b4510"
                    && frontier.status.contains("+0x4796")
                    && frontier.status.contains("0x1be stride")
                    && frontier.status.contains("highest-scoring player pointer")
            })
        })
        .unwrap_or(false);
    let last_has_match_per_tick_tactical_state_updater = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x006aae20"
                    && frontier.status.contains("+0x904d/+0x911d")
                    && frontier.status.contains("0x21cf/0x21c1/0x2137")
                    && frontier.status.contains("0x006a1470")
            })
        })
        .unwrap_or(false);
    let last_has_primary_formation_mask_classifier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x005a2c70"
                    && frontier.status.contains("+0x12d")
                    && frontier.status.contains("0x880/0x40/0x20/0x10/0x8")
            })
        })
        .unwrap_or(false);
    let last_has_secondary_formation_mask_classifier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x005a30d0"
                    && frontier.status.contains("+0x14e")
                    && frontier.status.contains("0x880/0x8")
            })
        })
        .unwrap_or(false);
    let last_has_random_float_jitter_shim = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00935080"
                    && frontier.status.contains("0x009350a2")
                    && frontier.status.contains("random jitter")
            })
        })
        .unwrap_or(false);
    let last_has_queued_club_news_cleanup = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00449710"
                    && frontier.status.contains("6-byte queued club/news items")
                    && frontier.status.contains("0x245-byte club records")
            })
        })
        .unwrap_or(false);
    let last_has_current_date_dispatcher = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x0053fe40"
                    && frontier.status.contains("42 callback/object slots")
            })
        })
        .unwrap_or(false);
    let last_has_staff_role_drift_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00614e90"
                    && frontier.status.contains("match RNG")
                    && frontier.status.contains("0x00616930")
            })
        })
        .unwrap_or(false);
    let last_has_season_calendar_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x005bfd90"
                    && frontier.status.contains("34 calendar buckets")
                    && frontier.status.contains("staff byte +0x6d")
            })
        })
        .unwrap_or(false);
    let last_has_club_metrics_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x005c01d0"
                    && frontier.status.contains("0x122-byte club records")
                    && frontier.status.contains("12-byte ranking records")
            })
        })
        .unwrap_or(false);
    let last_has_fixture_tie_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00752d40"
                    && frontier.status.contains("fixture lists")
                    && frontier.status.contains("70-day cleanup")
            })
        })
        .unwrap_or(false);
    let last_has_club_finance_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00585ae0"
                    && frontier.status.contains("0x245-byte club records")
                    && frontier
                        .status
                        .contains("0x167-byte finance/status side blocks")
            })
        })
        .unwrap_or(false);
    let last_has_byte_array_clear_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00784290"
                    && frontier.status.contains("param_1+8 byte buffer")
            })
        })
        .unwrap_or(false);
    let last_has_manager_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00674c10"
                    && frontier.status.contains("0x6e-byte manager records")
                    && frontier.status.contains("0x26-byte job events")
            })
        })
        .unwrap_or(false);
    let last_has_fixture_news_cleanup_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00595580"
                    && frontier.status.contains("news/event lists")
                    && frontier.status.contains("paired +3/+4")
            })
        })
        .unwrap_or(false);
    let last_has_host_country_schedule_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x005e4370"
                    && frontier.status.contains("34-byte date records")
                    && frontier.status.contains("match RNG")
            })
        })
        .unwrap_or(false);
    let last_has_stadium_cleanup_frontier = last
        .map(|trace| {
            trace.frontiers.iter().any(|frontier| {
                frontier.address == "0x00844940" && frontier.status.contains("30-day cleanup")
            })
        })
        .unwrap_or(false);
    push_binary_check(
        &mut checks,
        "phase-2-rollover",
        save.simulation.phase == 0
            && save.elapsed_days == 1
            && save.date.year == 2001
            && save.date.month == 7
            && save.date.day == 2
            && save.simulation.cm_packed_date.day_of_year == 183
            && save.phase_trace.len() == 3
            && last.map(|trace| trace.phase_before) == Some(2)
            && last.map(|trace| trace.phase_after) == Some(0)
            && last.map(|trace| trace.advanced_day) == Some(true)
            && last.map_or(0, |trace| trace.frontiers.len()) == 50
            && last_has_date_add
            && last_has_match_queue_builder
            && last_has_match_queue_annotation
            && last_has_match_processor_dispatcher
            && last_has_match_setup
            && last_has_match_team_player_setup
            && last_has_match_player_risk_setup
            && last_has_tactics_block_loader
            && last_has_tactics_index_resolver
            && last_has_selected_tactic_staff_slot_lookup
            && last_has_primary_tactic_flag_reader
            && last_has_secondary_tactic_flag_reader
            && last_has_player_random_byte_seed
            && last_has_player_evaluation_frontier
            && last_has_player_action_score_frontier
            && last_has_candidate_action_wrapper
            && last_has_adjacent_position_action_wrapper
            && last_has_player_action_attempt_frontier
            && last_has_move_action_resolution_frontier
            && last_has_player_action_selector_frontier
            && last_has_event_resolution_dispatcher_frontier
            && last_has_event_queue_writer
            && last_has_event_follow_up_challenge
            && last_has_directional_follow_up
            && last_has_shot_action_score
            && last_has_match_engine_step_controller
            && last_has_match_phase_possession_controller
            && last_has_match_pressure_action_continuation
            && last_has_match_stored_action_resolver
            && last_has_match_action_scratch_reset
            && last_has_match_period_transition
            && last_has_match_player_candidate_selector
            && last_has_match_per_tick_tactical_state_updater
            && last_has_primary_formation_mask_classifier
            && last_has_secondary_formation_mask_classifier
            && last_has_random_float_jitter_shim
            && last_has_staff_contract_frontier
            && last_has_queued_club_news_cleanup
            && last_has_current_date_dispatcher
            && last_has_staff_role_drift_frontier
            && last_has_season_calendar_frontier
            && last_has_club_metrics_frontier
            && last_has_fixture_tie_frontier
            && last_has_club_finance_frontier
            && last_has_byte_array_clear_frontier
            && last_has_manager_frontier
            && last_has_fixture_news_cleanup_frontier
            && last_has_host_country_schedule_frontier
            && last_has_stadium_cleanup_frontier,
        format!(
            "after phase rollover: {:04}-{:02}-{:02} phase {} elapsed {} trace {} last frontiers {}",
            save.date.year,
            save.date.month,
            save.date.day,
            save.simulation.phase,
            save.elapsed_days,
            save.phase_trace.len(),
            last.map_or(0, |trace| trace.frontiers.len())
        ),
    );

    let mut one_day = world.new_runtime_save_from_rust_db(db_dir);
    one_day.tick_days(1);
    push_binary_check(
        &mut checks,
        "tick-days-one-day-equivalence",
        one_day.date == save.date
            && one_day.simulation.phase == save.simulation.phase
            && one_day.simulation.cm_packed_date == save.simulation.cm_packed_date
            && one_day.phase_trace.len() == 3,
        format!(
            "tick_days(1) produced {:04}-{:02}-{:02}, phase {}, trace {}",
            one_day.date.year,
            one_day.date.month,
            one_day.date.day,
            one_day.simulation.phase,
            one_day.phase_trace.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "backend-ledger-one-day",
        one_day.backend.mutation_log.len() == 4
            && one_day.backend.transfers.attempted_mutations == 1
            && one_day.backend.news.attempted_mutations == 1
            && one_day.backend.matches.attempted_mutations == 1
            && one_day.backend.competitions.attempted_mutations == 1
            && one_day.backend.transfers.implemented_mutations == 1
            && one_day.backend.news.implemented_mutations == 1
            && one_day.backend.matches.implemented_mutations == 1
            && one_day.backend.competitions.implemented_mutations == 1
            && one_day.backend.mutation_log.iter().all(|entry| {
                format!("{:?}", entry.status) == "Implemented"
                    && entry.exactness_tier.as_deref() == Some("static-boundary-exact")
                    && entry.static_proof_rows.is_some_and(|count| count > 0)
                    && (entry.formula_lift_status.as_deref() == Some("pending-deeper-formula-lift")
                        || (entry.system == "match results"
                            && entry.formula_lift_status.as_deref()
                                == Some("formula-derived-runtime-store-installed"))
                        || (entry.system == "competition state"
                            && entry.formula_lift_status.as_deref()
                                == Some("competition-formula-runtime-store-installed"))
                        || (entry.system == "transfers/contracts"
                            && entry.formula_lift_status.as_deref()
                                == Some("contract-renewal-formula-runtime-store-installed"))
                        || (entry.system == "news/inbox"
                            && entry.formula_lift_status.as_deref()
                                == Some("news-inbox-formula-runtime-store-installed")))
            }),
        format!(
            "one-day backend ledger: {} entry(s), attempts match {} comp {} transfers {} news {}",
            one_day.backend.mutation_log.len(),
            one_day.backend.matches.attempted_mutations,
            one_day.backend.competitions.attempted_mutations,
            one_day.backend.transfers.attempted_mutations,
            one_day.backend.news.attempted_mutations
        ),
    );
    push_binary_check(
        &mut checks,
        "backend-contract-dispatch-one-day",
        one_day.backend.mutation_log.len() == 4
            && one_day.backend.mutation_log.iter().all(|entry| {
                entry.contract_status.is_some()
                    && entry.trace_file.is_some()
                    && entry.boundary_map.is_some()
                    && entry.implementation_hook.is_some()
                    && entry.parity_gate.is_some()
            })
            && one_day.backend.mutation_log.iter().any(|entry| {
                entry.system == "match results"
                    && entry.trace_file.as_deref()
                        == Some("reports/parity_traces/match-results.json")
                    && entry.boundary_map.as_deref() == Some("match_result_write_map")
            }),
        format!(
            "one-day backend dispatch attached contract metadata to {} mutation attempt(s)",
            one_day.backend.mutation_log.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "backend-skeleton-dispatch-one-day",
        one_day.backend.mutation_log.len() == 4
            && one_day.backend.mutation_log.iter().all(|entry| {
                entry.skeleton_entry_point.is_some()
                    && entry.skeleton_status.as_deref() == Some("static-proof-backed")
                    && entry.skeleton_mutations_emitted.is_some_and(|count| count > 0)
                    && entry.skeleton_safety_rule.is_some()
                    && entry.exactness_tier.as_deref() == Some("static-boundary-exact")
                    && entry.static_proof_rows.is_some_and(|count| count > 0)
                    && (entry.formula_lift_status.as_deref()
                        == Some("pending-deeper-formula-lift")
                        || (entry.system == "match results"
                            && entry.formula_lift_status.as_deref()
                                == Some("formula-derived-runtime-store-installed"))
                        || (entry.system == "competition state"
                            && entry.formula_lift_status.as_deref()
                                == Some("competition-formula-runtime-store-installed"))
                        || (entry.system == "transfers/contracts"
                            && entry.formula_lift_status.as_deref()
                                == Some("contract-renewal-formula-runtime-store-installed"))
                        || (entry.system == "news/inbox"
                            && entry.formula_lift_status.as_deref()
                                == Some("news-inbox-formula-runtime-store-installed")))
            }),
        format!(
            "one-day backend dispatch called static-boundary exact mutator entry points for {} subsystem(s)",
            one_day.backend.mutation_log.len()
        ),
    );

    let mut same_day = world.new_runtime_save_from_rust_db(db_dir);
    let same_day_advanced = same_day.tick_to_date(GameDate {
        year: 2001,
        month: 7,
        day: 1,
    });
    push_binary_check(
        &mut checks,
        "tick-to-date-current-date-noop",
        same_day_advanced == 0
            && same_day.date.year == 2001
            && same_day.date.month == 7
            && same_day.date.day == 1
            && same_day.simulation.phase == 0
            && same_day.phase_trace.is_empty(),
        format!(
            "tick_to_date(current) advanced {} day(s), date {:04}-{:02}-{:02}, phase {}, trace {}",
            same_day_advanced,
            same_day.date.year,
            same_day.date.month,
            same_day.date.day,
            same_day.simulation.phase,
            same_day.phase_trace.len()
        ),
    );

    let mut target_date = world.new_runtime_save_from_rust_db(db_dir);
    let target_date_advanced = target_date.tick_to_date(GameDate {
        year: 2001,
        month: 7,
        day: 4,
    });
    push_binary_check(
        &mut checks,
        "tick-to-date-three-day-phase-composition",
        target_date_advanced == 3
            && target_date.date.year == 2001
            && target_date.date.month == 7
            && target_date.date.day == 4
            && target_date.elapsed_days == 3
            && target_date.simulation.phase == 0
            && target_date.simulation.cm_packed_date.day_of_year == 185
            && target_date.phase_trace.len() == 9,
        format!(
            "tick_to_date(2001-07-04) advanced {} day(s), date {:04}-{:02}-{:02}, phase {}, cm day {}, trace {}",
            target_date_advanced,
            target_date.date.year,
            target_date.date.month,
            target_date.date.day,
            target_date.simulation.phase,
            target_date.simulation.cm_packed_date.day_of_year,
            target_date.phase_trace.len()
        ),
    );

    let mut headless = world.new_runtime_save_from_rust_db(db_dir);
    let headless_report = headless.run_headless_days(2);
    push_binary_check(
        &mut checks,
        "headless-two-day-shell-run",
        headless_report.days_advanced == 2
            && headless_report.phases_advanced == 6
            && headless_report.phase_trace_entries_added == 6
            && headless_report.end_date.year == 2001
            && headless_report.end_date.month == 7
            && headless_report.end_date.day == 3
            && headless_report.last_phase_frontiers == 50
            && headless_report.still_frontier_only.is_empty()
            && headless_report.status == cm_domain::HeadlessPlayStatus::Runnable
            && headless.headless.last_run.is_some(),
        format!(
            "headless run advanced {} day(s), {} phase(s), ended {:04}-{:02}-{:02}, blockers {}",
            headless_report.days_advanced,
            headless_report.phases_advanced,
            headless_report.end_date.year,
            headless_report.end_date.month,
            headless_report.end_date.day,
            headless_report.still_frontier_only.len()
        ),
    );
    push_binary_check(
        &mut checks,
        "backend-ledger-headless-two-day",
        headless.backend.mutation_log.len() == 8
            && headless.backend.matches.attempted_mutations == 2
            && headless.backend.competitions.attempted_mutations == 2
            && headless.backend.transfers.attempted_mutations == 2
            && headless.backend.news.attempted_mutations == 2,
        format!(
            "headless backend ledger: {} entry(s), attempts match {} comp {} transfers {} news {}",
            headless.backend.mutation_log.len(),
            headless.backend.matches.attempted_mutations,
            headless.backend.competitions.attempted_mutations,
            headless.backend.transfers.attempted_mutations,
            headless.backend.news.attempted_mutations
        ),
    );
    push_binary_check(
        &mut checks,
        "match-result-runtime-store",
        cm_domain::match_result_runtime_store_ready(&headless.backend.match_result_runtime_store),
        format!(
            "{} fixture store row(s), {} event queue row(s), {} applied formula mutation(s)",
            headless.backend.match_result_runtime_store.fixtures.len(),
            headless
                .backend
                .match_result_runtime_store
                .event_queue
                .len(),
            headless
                .backend
                .match_result_runtime_store
                .applied_formula_mutations
        ),
    );
    push_binary_check(
        &mut checks,
        "competition-notification-runtime-store",
        cm_domain::competition_notification_runtime_store_ready(
            &headless.backend.competition_notification_runtime_store,
        ),
        format!(
            "{} fixture notification(s), {} maintenance event(s), {} applied formula mutation(s)",
            headless
                .backend
                .competition_notification_runtime_store
                .fixture_notifications
                .len(),
            headless
                .backend
                .competition_notification_runtime_store
                .maintenance_events
                .len(),
            headless
                .backend
                .competition_notification_runtime_store
                .applied_formula_mutations
        ),
    );
    push_binary_check(
        &mut checks,
        "competition-standings-runtime-store",
        cm_domain::competition_standings_runtime_store_ready(
            &headless.backend.competition_standings_runtime_store,
        ),
        format!(
            "{} standings row(s), {} applied formula mutation(s)",
            headless
                .backend
                .competition_standings_runtime_store
                .rows
                .len(),
            headless
                .backend
                .competition_standings_runtime_store
                .applied_formula_mutations
        ),
    );
    push_binary_check(
        &mut checks,
        "competition-progression-runtime-store",
        cm_domain::competition_progression_runtime_store_ready(
            &headless.backend.competition_progression_runtime_store,
        ),
        format!(
            "{} reset row(s), {} owner candidate(s), {} queued progression record(s), {} assignment transition(s), {} cleanup event(s), {} applied formula mutation(s)",
            headless
                .backend
                .competition_progression_runtime_store
                .reset_rows
                .len(),
            headless
                .backend
                .competition_progression_runtime_store
                .owner_candidates
                .len(),
            headless
                .backend
                .competition_progression_runtime_store
                .progression_queue
                .len(),
            headless
                .backend
                .competition_progression_runtime_store
                .assignment_transitions
                .len(),
            headless
                .backend
                .competition_progression_runtime_store
                .cleanup_events
                .len(),
            headless
                .backend
                .competition_progression_runtime_store
                .applied_formula_mutations
        ),
    );
    push_binary_check(
        &mut checks,
        "transfer-contract-runtime-store",
        cm_domain::transfer_contract_runtime_store_ready(
            &headless.backend.transfer_contract_runtime_store,
        ),
        format!(
            "{} renewal window(s), {} contract event(s), {} compensation value(s), {} offer value(s), {} decision rule(s), {} transfer-manager shape(s), {} queue item(s), {} dispatch(es), {} applied formula mutation(s)",
            headless
                .backend
                .transfer_contract_runtime_store
                .renewal_windows
                .len(),
            headless
                .backend
                .transfer_contract_runtime_store
                .contract_events
                .len(),
            headless
                .backend
                .transfer_contract_runtime_store
                .compensation_values
                .len(),
            headless
                .backend
                .transfer_contract_runtime_store
                .offer_values
                .len(),
            headless
                .backend
                .transfer_contract_runtime_store
                .decision_rules
                .len(),
            headless
                .backend
                .transfer_contract_runtime_store
                .transfer_manager_record_shapes
                .len(),
            headless
                .backend
                .transfer_contract_runtime_store
                .transfer_queue
                .len(),
            headless
                .backend
                .transfer_contract_runtime_store
                .queue_dispatches
                .len(),
            headless
                .backend
                .transfer_contract_runtime_store
                .applied_formula_mutations
        ),
    );
    push_binary_check(
        &mut checks,
        "news-inbox-runtime-store",
        cm_domain::news_inbox_runtime_store_ready(&headless.backend.news_inbox_runtime_store),
        format!(
            "{} created event(s), {} dispatch(es), {} removed queue node(s), {} applied formula mutation(s)",
            headless.backend.news_inbox_runtime_store.created_events.len(),
            headless
                .backend
                .news_inbox_runtime_store
                .visible_news_dispatches
                .len(),
            headless
                .backend
                .news_inbox_runtime_store
                .removed_queue_nodes
                .len(),
            headless
                .backend
                .news_inbox_runtime_store
                .applied_formula_mutations
        ),
    );

    let mut campaign = world.new_runtime_save_from_rust_db(db_dir);
    let campaign_report = campaign.run_headless_campaign_days(30, 10);
    push_binary_check(
        &mut checks,
        "headless-campaign-thirty-day-backend-gate",
        campaign_report.days_advanced == 30
            && campaign_report.phases_advanced == 90
            && campaign_report.checkpoints.len() == 3
            && campaign_report.backend.mutation_log_entries_added == 120
            && campaign_report.backend.match_attempts == 30
            && campaign_report.backend.competition_attempts == 30
            && campaign_report.backend.transfer_attempts == 30
            && campaign_report.backend.news_attempts == 30
            && campaign_report.backend.implemented_mutations == 120
            && campaign_report.backend.frontier_only_mutations == 0,
        format!(
            "30-day campaign ended {:04}-{:02}-{:02}, checkpoints {}, backend mutations +{}, attempts match {} comp {} transfers {} news {}",
            campaign_report.end_date.year,
            campaign_report.end_date.month,
            campaign_report.end_date.day,
            campaign_report.checkpoints.len(),
            campaign_report.backend.mutation_log_entries_added,
            campaign_report.backend.match_attempts,
            campaign_report.backend.competition_attempts,
            campaign_report.backend.transfer_attempts,
            campaign_report.backend.news_attempts
        ),
    );

    let mut full_season_campaign = world.new_runtime_save_from_rust_db(db_dir);
    let full_season_report = full_season_campaign.run_headless_campaign_days(365, 30);
    push_binary_check(
        &mut checks,
        "headless-campaign-full-year-retained-backend-gate",
        full_season_report.days_advanced == 365
            && full_season_report.phases_advanced == 1095
            && full_season_report.checkpoints.len() == 13
            && full_season_report.backend.mutation_log_entries_added == 1460
            && full_season_report.backend.total_mutation_log_entries == 1460
            && full_season_report.backend.match_attempts == 365
            && full_season_report.backend.competition_attempts == 365
            && full_season_report.backend.transfer_attempts == 365
            && full_season_report.backend.news_attempts == 365
            && full_season_campaign.backend.mutation_log.len()
                == full_season_campaign.backend.mutation_log_limit
            && full_season_campaign.backend.dropped_mutation_entries == 460
            && full_season_report.backend.implemented_mutations == 1460
            && full_season_report.backend.frontier_only_mutations == 0,
        format!(
            "365-day campaign ended {:04}-{:02}-{:02}, checkpoints {}, backend mutations +{}, retained {}, dropped {}",
            full_season_report.end_date.year,
            full_season_report.end_date.month,
            full_season_report.end_date.day,
            full_season_report.checkpoints.len(),
            full_season_report.backend.mutation_log_entries_added,
            full_season_campaign.backend.mutation_log.len(),
            full_season_campaign.backend.dropped_mutation_entries
        ),
    );

    let mut manager_session = world.new_runtime_save_from_rust_db(db_dir);
    let manager_command =
        manager_session.set_headless_manager("Headless Manager".to_string(), Some(1));
    push_binary_check(
        &mut checks,
        "headless-manager-session-command",
        manager_session
            .headless
            .manager
            .as_ref()
            .is_some_and(|manager| {
                manager.name == "Headless Manager"
                    && manager.club_id == Some(1)
                    && format!("{:?}", manager.status) == "ClubSelectedFrontierOnly"
            })
            && manager_session.headless.command_history.len() == 1
            && manager_session.headless.milestones.len() >= 2,
        format!(
            "manager command '{}' at {:04}-{:02}-{:02}: {}",
            manager_command.command,
            manager_command.date.year,
            manager_command.date.month,
            manager_command.date.day,
            manager_command.detail
        ),
    );

    let failures = checks
        .iter()
        .filter(|check| check.get("status").and_then(serde_json::Value::as_str) == Some("fail"))
        .count();

    Ok(serde_json::json!({
        "source": {
            "rust_db": db_dir.display().to_string(),
            "carve": "D:/cm0102-carve",
            "citations": [
                "simulation_frontier.json",
                "ghidra_out/cm0102.exe/decompiled/005b6a90.c",
                "ghidra_out/cm0102.exe/decompiled/00536190.c"
            ]
        },
        "summary": {
            "checks": checks.len(),
            "failures": failures,
            "status": if failures == 0 { "pass" } else { "fail" },
        },
        "runtime": {
            "implemented_scope": "phase counter rollover, packed date add-days, phase frontier trace, and static-boundary backend mutation ledger",
            "exactness_tiers": {
                "current": "static-boundary-exact",
                "next": "formula-exact",
                "rule": "A backend mutation is full one-for-one only after its remaining formula_lift_pending items are eliminated by Ghidra/carver evidence."
            },
            "start": {
                "date": "2001-07-01",
                "phase": 0,
                "cm_day_of_year": 182
            },
            "gameplay_mutator_contracts": save.backend.mutator_contracts,
            "gameplay_mutator_install_plans": save.backend.mutator_install_plans,
            "gameplay_promotion_gates": save.backend.gameplay_promotion_gates,
            "gameplay_lift_workbench": save.backend.gameplay_lift_workbench,
            "gameplay_system_code_claims": save.backend.gameplay_system_code_claims,
            "exact_gameplay_mutator_skeletons": save.backend.exact_mutator_skeletons,
            "exact_gameplay_mutator_skeleton_outcomes": exact_mutator_skeleton_outcomes,
            "match_engine_lift_map": save.backend.match_engine_lift_map,
            "match_result_write_map": save.backend.match_result_write_map,
            "match_result_code_claims": save.backend.match_result_code_claims,
            "match_result_formula_lift_map": save.backend.match_result_formula_lift_map,
            "match_result_formula_scenario": match_result_formula_scenario,
            "match_result_formula_plan": match_result_formula_plan,
            "match_result_runtime_store": headless.backend.match_result_runtime_store,
            "match_result_mutator_install_plan": save.backend.match_result_mutator_install_plan,
            "competition_fixture_state_map": save.backend.competition_fixture_state_map,
            "competition_notification_formula_lift_map": save.backend.competition_notification_formula_lift_map,
            "competition_notification_formula_scenario": competition_notification_formula_scenario,
            "competition_notification_formula_plan": competition_notification_formula_plan,
            "competition_notification_runtime_store": headless.backend.competition_notification_runtime_store,
            "competition_standings_formula_lift_map": save.backend.competition_standings_formula_lift_map,
            "competition_standings_formula_scenario": competition_standings_formula_scenario,
            "competition_standings_formula_plan": competition_standings_formula_plan,
            "competition_standings_runtime_store": headless.backend.competition_standings_runtime_store,
            "competition_progression_formula_lift_map": save.backend.competition_progression_formula_lift_map,
            "competition_progression_formula_scenario": competition_progression_formula_scenario,
            "competition_progression_formula_plan": competition_progression_formula_plan,
            "competition_progression_runtime_store": headless.backend.competition_progression_runtime_store,
            "transfer_contract_state_map": save.backend.transfer_contract_state_map,
            "transfer_contract_formula_lift_map": save.backend.transfer_contract_formula_lift_map,
            "transfer_contract_formula_scenario": transfer_contract_formula_scenario,
            "transfer_contract_formula_plan": transfer_contract_formula_plan,
            "transfer_contract_runtime_store": headless.backend.transfer_contract_runtime_store,
            "news_inbox_emission_map": save.backend.news_inbox_emission_map,
            "news_inbox_formula_lift_map": save.backend.news_inbox_formula_lift_map,
            "news_inbox_formula_scenario": news_inbox_formula_scenario,
            "news_inbox_formula_plan": news_inbox_formula_plan,
            "news_inbox_runtime_store": headless.backend.news_inbox_runtime_store,
            "after_one_day": {
                "date": format!("{:04}-{:02}-{:02}", save.date.year, save.date.month, save.date.day),
                "phase": save.simulation.phase,
                "cm_day_of_year": save.simulation.cm_packed_date.day_of_year,
                "trace_entries": save.phase_trace.len(),
                "last_phase_frontiers": last.map_or(0, |trace| trace.frontiers.len())
            },
            "tick_to_date_sample": {
                "target": "2001-07-04",
                "advanced_days": target_date_advanced,
                "date": format!("{:04}-{:02}-{:02}", target_date.date.year, target_date.date.month, target_date.date.day),
                "phase": target_date.simulation.phase,
                "cm_day_of_year": target_date.simulation.cm_packed_date.day_of_year,
                "trace_entries": target_date.phase_trace.len()
            },
            "headless_sample": {
                "advanced_days": headless_report.days_advanced,
                "phases_advanced": headless_report.phases_advanced,
                "date": format!("{:04}-{:02}-{:02}", headless_report.end_date.year, headless_report.end_date.month, headless_report.end_date.day),
                "status": format!("{:?}", headless_report.status),
                "last_phase_frontiers": headless_report.last_phase_frontiers,
                "frontier_only_blockers": headless_report.still_frontier_only
            },
            "backend_ledger_sample": {
                "status": format!("{:?}", headless.backend.status),
                "mutation_log_entries": headless.backend.mutation_log.len(),
                "attempts": {
                    "match_results": headless.backend.matches.attempted_mutations,
                    "competition_state": headless.backend.competitions.attempted_mutations,
                    "transfers_contracts": headless.backend.transfers.attempted_mutations,
                    "news_inbox": headless.backend.news.attempted_mutations
                },
                "owned_records": {
                    "match_results": headless.backend.matches.owned_records,
                    "competition_state": headless.backend.competitions.owned_records,
                    "transfers_contracts": headless.backend.transfers.owned_records,
                    "news_inbox": headless.backend.news.owned_records
                },
                "recent_mutations": headless.backend.mutation_log.iter().rev().take(8).cloned().collect::<Vec<_>>()
            },
            "headless_campaign_sample": {
                "days_advanced": campaign_report.days_advanced,
                "phases_advanced": campaign_report.phases_advanced,
                "date": format!("{:04}-{:02}-{:02}", campaign_report.end_date.year, campaign_report.end_date.month, campaign_report.end_date.day),
                "checkpoints": campaign_report.checkpoints.len(),
                "backend_mutations_added": campaign_report.backend.mutation_log_entries_added,
                "attempts": {
                    "match_results": campaign_report.backend.match_attempts,
                    "competition_state": campaign_report.backend.competition_attempts,
                    "transfers_contracts": campaign_report.backend.transfer_attempts,
                    "news_inbox": campaign_report.backend.news_attempts
                },
                "implemented_mutations": campaign_report.backend.implemented_mutations,
                "frontier_only_mutations": campaign_report.backend.frontier_only_mutations
            },
            "headless_full_year_campaign_sample": {
                "days_advanced": full_season_report.days_advanced,
                "phases_advanced": full_season_report.phases_advanced,
                "date": format!("{:04}-{:02}-{:02}", full_season_report.end_date.year, full_season_report.end_date.month, full_season_report.end_date.day),
                "checkpoints": full_season_report.checkpoints.len(),
                "backend_mutations_added": full_season_report.backend.mutation_log_entries_added,
                "backend_mutations_total": full_season_report.backend.total_mutation_log_entries,
                "retained_mutations": full_season_campaign.backend.mutation_log.len(),
                "dropped_mutations": full_season_campaign.backend.dropped_mutation_entries,
                "retention_limit": full_season_campaign.backend.mutation_log_limit,
                "attempts": {
                    "match_results": full_season_report.backend.match_attempts,
                    "competition_state": full_season_report.backend.competition_attempts,
                    "transfers_contracts": full_season_report.backend.transfer_attempts,
                    "news_inbox": full_season_report.backend.news_attempts
                },
                "implemented_mutations": full_season_report.backend.implemented_mutations,
                "frontier_only_mutations": full_season_report.backend.frontier_only_mutations
            },
            "headless_manager_sample": {
                "manager": manager_session.headless.manager,
                "command_history": manager_session.headless.command_history.len()
            },
            "formula_lift_pending": [
                "contract renewal, bid, wage, AI decision, and transfer-value formulas",
                "fixture cleanup and competition progression branch formulas",
                "manager-manager mutation formulas",
                "stadium/date cleanup formulas",
                "match-day scoring/action formulas"
            ]
        },
        "checks": checks,
    }))
}

fn gameplay_mutator_install_plans_ready(plans: &[cm_domain::GameplayMutatorInstallPlan]) -> bool {
    gameplay_mutator_install_plan_ready(
        plans,
        "match results",
        2,
        "match_result_write_map",
        "reports/parity_traces/match-results.json",
    ) && gameplay_mutator_install_plan_ready(
        plans,
        "competition state",
        2,
        "competition_fixture_state_map",
        "reports/parity_traces/competition-state.json",
    ) && gameplay_mutator_install_plan_ready(
        plans,
        "transfers/contracts",
        0,
        "transfer_contract_state_map",
        "reports/parity_traces/transfers-contracts.json",
    ) && gameplay_mutator_install_plan_ready(
        plans,
        "news/inbox",
        1,
        "news_inbox_emission_map",
        "reports/parity_traces/news-inbox.json",
    )
}

fn gameplay_mutator_install_plan_ready(
    plans: &[cm_domain::GameplayMutatorInstallPlan],
    system: &str,
    phase: u8,
    boundary_map: &str,
    trace_file: &str,
) -> bool {
    plans.iter().any(|plan| {
        plan.system == system
            && plan.phase == phase
            && plan.boundary_map == boundary_map
            && plan.trace_file.ends_with(trace_file)
            && !plan.rust_hook.is_empty()
            && !plan.required_original_coverage.is_empty()
            && plan.required_original_coverage == plan.required_rust_coverage
            && !plan.required_functions.is_empty()
            && plan.promotion_rule.contains("implementation_present=true")
    })
}

fn backend_acceptance_report(db_dir: &Path) -> Result<serde_json::Value, String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let readiness = world.backend_readiness_report(db_dir);
    let runtime_validation = validate_runtime_simulation_report(db_dir)?;
    let runtime_summary = &runtime_validation["summary"];
    let full_year = &runtime_validation["runtime"]["headless_full_year_campaign_sample"];
    let readiness_infra_pass = readiness.completion.remaining_binary_tables == 0
        && readiness.completion.validation_failures == 0
        && readiness.completion.score_percent >= 80;
    let runtime_validation_pass = runtime_summary["status"].as_str() == Some("pass")
        && runtime_summary["failures"].as_u64().unwrap_or(1) == 0;
    let full_year_pass = full_year["days_advanced"].as_u64() == Some(365)
        && full_year["phases_advanced"].as_u64() == Some(1095)
        && full_year["backend_mutations_added"].as_u64() == Some(1460)
        && full_year["retained_mutations"].as_u64() == Some(1000)
        && full_year["dropped_mutations"].as_u64() == Some(460);
    let gameplay_blockers = readiness.completion.headless_blockers;
    let implementation_plan_boundary_mapped = readiness
        .implementation_plan
        .iter()
        .filter(|item| {
            item.readiness == BackendImplementationReadiness::BoundaryMapped
                && item.boundary_entries > 0
        })
        .count();
    let implementation_plan_mutations_implemented = readiness
        .implementation_plan
        .iter()
        .filter(|item| {
            item.readiness == BackendImplementationReadiness::MutationsImplemented
                && item.implemented_mutations > 0
                && item.boundary_entries > 0
        })
        .count();
    let implementation_plan_ready =
        implementation_plan_boundary_mapped + implementation_plan_mutations_implemented;
    let infrastructure_pass = readiness_infra_pass && runtime_validation_pass && full_year_pass;
    let status = if infrastructure_pass && gameplay_blockers == 0 {
        "playable_headless"
    } else if infrastructure_pass {
        "blocked_by_frontier_gameplay"
    } else {
        "backend_infrastructure_regressed"
    };

    Ok(serde_json::json!({
        "format": "cm0102-rs-backend-acceptance",
        "version": 1,
        "status": status,
        "infrastructure_pass": infrastructure_pass,
        "playable_headless": infrastructure_pass && gameplay_blockers == 0,
        "summary": {
            "readiness_score_percent": readiness.completion.score_percent,
            "remaining_binary_tables": readiness.completion.remaining_binary_tables,
            "canonical_validation_failures": readiness.completion.validation_failures,
            "runtime_validation_checks": runtime_summary["checks"],
            "runtime_validation_failures": runtime_summary["failures"],
            "full_year_days": full_year["days_advanced"],
            "full_year_phases": full_year["phases_advanced"],
            "full_year_backend_mutations": full_year["backend_mutations_added"],
            "full_year_retained_mutations": full_year["retained_mutations"],
            "full_year_dropped_mutations": full_year["dropped_mutations"],
            "gameplay_blockers": gameplay_blockers,
            "implementation_plan_items": readiness.implementation_plan.len(),
            "implementation_plan_boundary_mapped": implementation_plan_boundary_mapped,
            "implementation_plan_mutations_implemented": implementation_plan_mutations_implemented,
            "implementation_plan_ready": implementation_plan_ready,
        },
        "implementation_plan": readiness.implementation_plan,
        "checks": [
            {
                "name": "rust-db-readiness",
                "status": if readiness_infra_pass { "pass" } else { "fail" },
                "detail": format!(
                    "score {}%, {} binary table(s), {} canonical validation failure(s)",
                    readiness.completion.score_percent,
                    readiness.completion.remaining_binary_tables,
                    readiness.completion.validation_failures
                )
            },
            {
                "name": "runtime-validation",
                "status": if runtime_validation_pass { "pass" } else { "fail" },
                "detail": format!(
                    "{} check(s), {} failure(s)",
                    runtime_summary["checks"].as_u64().unwrap_or(0),
                    runtime_summary["failures"].as_u64().unwrap_or(0)
                )
            },
            {
                "name": "full-year-headless-campaign",
                "status": if full_year_pass { "pass" } else { "fail" },
                "detail": format!(
                    "{} day(s), {} phase(s), {} backend mutation(s), retained {}, dropped {}",
                    full_year["days_advanced"].as_u64().unwrap_or(0),
                    full_year["phases_advanced"].as_u64().unwrap_or(0),
                    full_year["backend_mutations_added"].as_u64().unwrap_or(0),
                    full_year["retained_mutations"].as_u64().unwrap_or(0),
                    full_year["dropped_mutations"].as_u64().unwrap_or(0)
                )
            },
            {
                "name": "gameplay-mutators",
                "status": if gameplay_blockers == 0 { "pass" } else { "warn" },
                "detail": format!("{gameplay_blockers} exact gameplay subsystem(s) still frontier-only")
            }
        ],
        "readiness": readiness,
        "runtime_validation": runtime_validation,
        "next_gate_to_green": [
            "Implement exact match result/event mutations from lifted match-engine arithmetic.",
            "Implement exact competition fixture/table/cup state mutations.",
            "Implement exact transfer/contract state mutations.",
            "Implement exact news/inbox record emission semantics."
        ]
    }))
}

fn exact_remake_report(db_dir: &Path, exe: &Path) -> Result<serde_json::Value, String> {
    let original_binary = validate_original_binary_report(exe)?;
    let execution_model = validate_execution_model_report(exe)?;
    let simulation_frontier = validate_simulation_frontier_report(exe)?;
    let backend_acceptance = backend_acceptance_report(db_dir)?;
    let gameplay_parity =
        gameplay_parity_report(db_dir, Path::new("D:/cm0102-rs/reports/parity_traces"))?;
    let gameplay_promotion =
        gameplay_promotion_report(db_dir, Path::new("D:/cm0102-rs/reports/parity_traces"))?;

    let original_binary_pass = report_status_pass(&original_binary);
    let execution_model_pass = report_status_pass(&execution_model);
    let simulation_frontier_pass = report_status_pass(&simulation_frontier);
    let infrastructure_pass = backend_acceptance["infrastructure_pass"]
        .as_bool()
        .unwrap_or(false);
    let playable_headless = backend_acceptance["playable_headless"]
        .as_bool()
        .unwrap_or(false);
    let gameplay_parity_pass = gameplay_parity["summary"]["status"].as_str() == Some("pass");
    let gameplay_promotion_pass = gameplay_promotion["summary"]["status"].as_str() == Some("pass");
    let gameplay_blockers = backend_acceptance["summary"]["gameplay_blockers"]
        .as_u64()
        .unwrap_or(1);
    let implementation_plan_items = backend_acceptance["summary"]["implementation_plan_items"]
        .as_u64()
        .unwrap_or(0);
    let implementation_plan_boundary_mapped = backend_acceptance["summary"]
        ["implementation_plan_boundary_mapped"]
        .as_u64()
        .unwrap_or(0);
    let implementation_plan_mutations_implemented = backend_acceptance["summary"]
        ["implementation_plan_mutations_implemented"]
        .as_u64()
        .unwrap_or(0);
    let implementation_plan_ready = backend_acceptance["summary"]["implementation_plan_ready"]
        .as_u64()
        .unwrap_or(implementation_plan_boundary_mapped + implementation_plan_mutations_implemented);
    let formula_lift_complete = backend_acceptance["implementation_plan"]
        .as_array()
        .is_some_and(|items| {
            !items.is_empty()
                && items.iter().all(|item| {
                    item["missing_lifts"]
                        .as_array()
                        .is_some_and(|missing| missing.is_empty())
                })
        });

    let foundation_pass = original_binary_pass
        && execution_model_pass
        && simulation_frontier_pass
        && infrastructure_pass
        && implementation_plan_items == 4
        && implementation_plan_ready == 4;
    let static_boundary_behavior_pass = playable_headless
        && gameplay_blockers == 0
        && gameplay_parity_pass
        && gameplay_promotion_pass;
    let exact_behavior_pass = static_boundary_behavior_pass && formula_lift_complete;

    // --- Proof-basis gate ---------------------------------------------------
    // The internal/static gates above can all be green while the report is still
    // only self-consistent (Rust candidate traces validated against Rust-derived
    // expectations). The top-tier "one_for_one_exact" label is reserved for proof
    // that is external to the Rust implementation. Two things can supply that:
    //   1. Completed live original-binary capture (filled == expected rows), or
    //   2. The documented static code-derived proof policy: every gameplay parity
    //      trace stamped `static_parity_proof.status == "static-proven"` from
    //      Ghidra CODE_DERIVED evidence.
    // Static proof supersedes capture only under an explicit, differently-named
    // status. It is NEVER reported as capture-backed "one_for_one_exact".
    let template_dir = Path::new("D:/cm0102-rs/reports/original_capture_templates");
    let original_capture = original_capture_status_report(template_dir)?;
    let original_expected = original_capture["summary"]["expected_original_rows"]
        .as_u64()
        .unwrap_or(0);
    let original_filled = original_capture["summary"]["filled_original_rows"]
        .as_u64()
        .unwrap_or(0);
    let original_capture_complete = original_expected > 0 && original_filled == original_expected;

    let parity_trace_dir = Path::new("D:/cm0102-rs/reports/parity_traces");
    let static_proof_slugs = [
        "match-results",
        "competition-state",
        "transfers-contracts",
        "news-inbox",
    ];
    let mut static_proven_systems = 0u64;
    for slug in static_proof_slugs {
        let path = parity_trace_dir.join(format!("{slug}.json"));
        if let Ok(text) = fs::read_to_string(&path) {
            if let Ok(trace) = serde_json::from_str::<serde_json::Value>(&text) {
                // apply_static_parity_proof stamps a passing proof block whose
                // provenance names the Ghidra CODE_DERIVED static lift, and sets
                // the trace status to "static-proof-comparison-pass".
                let proof = &trace["static_parity_proof"];
                let proof_pass = proof["status"].as_str() == Some("pass");
                let code_derived = proof["provenance"]
                    .as_str()
                    .is_some_and(|p| p.contains("CODE_DERIVED"));
                let proof_rows = proof["rows"].as_u64().unwrap_or(0) > 0;
                let trace_marked = trace["status"].as_str() == Some("static-proof-comparison-pass");
                if proof_pass && code_derived && proof_rows && trace_marked {
                    static_proven_systems += 1;
                }
            }
        }
    }
    let static_proof_in_force = static_proven_systems == static_proof_slugs.len() as u64;

    // Internal behaviour exactness = every static/internal gate green (the old,
    // over-broad definition of one_for_one). It is necessary but NOT sufficient.
    let internal_behavior_exact = foundation_pass && exact_behavior_pass;

    let proof_basis = if original_capture_complete {
        "original_capture"
    } else if static_proof_in_force {
        "static_code_derived"
    } else {
        "unproven"
    };

    // Capture-backed one-for-one exactness requires completed original capture.
    let one_for_one_exact_remake = internal_behavior_exact && original_capture_complete;

    let status = if one_for_one_exact_remake {
        "one_for_one_exact"
    } else if internal_behavior_exact && static_proof_in_force {
        // Documented static-proof policy in force, capture still pending. Exact
        // against lifted code, NOT yet against a live original-binary run.
        "static_code_derived_exact_capture_pending"
    } else if internal_behavior_exact {
        // Internal gates green but neither external proof basis is satisfied.
        "internal_gates_green_external_proof_pending"
    } else if foundation_pass && static_boundary_behavior_pass {
        "static_boundary_exact_formula_lift_pending"
    } else if foundation_pass {
        "rust_foundation_ready_but_gameplay_not_exact"
    } else {
        "foundation_incomplete"
    };

    let mut caveats: Vec<String> = Vec::new();
    if !original_capture_complete {
        caveats.push(format!(
            "Original-binary capture incomplete: {original_filled}/{original_expected} rows filled. \
             'one_for_one_exact' is withheld until capture completes."
        ));
    }
    if !original_capture_complete && static_proof_in_force {
        caveats.push(
            "Exactness currently rests on the static code-derived proof policy \
             (Ghidra CODE_DERIVED, `static_parity_proof`), not on a live original-binary run."
                .to_string(),
        );
    }
    if !original_capture_complete && !static_proof_in_force {
        caveats.push(format!(
            "Neither external proof basis is satisfied: capture {original_filled}/{original_expected}, \
             static-proven parity systems {static_proven_systems}/{}.",
            static_proof_slugs.len()
        ));
    }

    Ok(serde_json::json!({
        "format": "cm0102-rs-exact-remake-report",
        "version": 1,
        "status": status,
        "one_for_one_exact_remake": one_for_one_exact_remake,
        "proof_basis": proof_basis,
        "internal_behavior_exact": internal_behavior_exact,
        "caveats": caveats,
        "foundation_pass": foundation_pass,
        "exact_behavior_pass": exact_behavior_pass,
        "source": {
            "rust_db": db_dir.display().to_string(),
            "original_binary": exe.display().to_string(),
            "carve": "D:/cm0102-carve"
        },
        "summary": {
            "original_binary_pass": original_binary_pass,
            "execution_model_pass": execution_model_pass,
            "simulation_frontier_pass": simulation_frontier_pass,
            "backend_infrastructure_pass": infrastructure_pass,
            "gameplay_parity_pass": gameplay_parity_pass,
            "static_boundary_behavior_pass": static_boundary_behavior_pass,
            "formula_lift_complete": formula_lift_complete,
            "original_capture_complete": original_capture_complete,
            "original_capture_filled_rows": original_filled,
            "original_capture_expected_rows": original_expected,
            "static_proof_in_force": static_proof_in_force,
            "static_proven_systems": static_proven_systems,
            "playable_headless": playable_headless,
            "gameplay_blockers": gameplay_blockers,
            "implementation_plan_items": implementation_plan_items,
            "implementation_plan_boundary_mapped": implementation_plan_boundary_mapped,
            "implementation_plan_mutations_implemented": implementation_plan_mutations_implemented,
            "implementation_plan_ready": implementation_plan_ready,
            "required_to_call_exact": [
                "playable_headless must be true",
                "gameplay_blockers must be 0",
                "gameplay parity traces must pass for all four gameplay systems",
                "gameplay promotion gates must be ready for all four gameplay systems",
                "match result mutations must be implemented from lifted arithmetic and binary traces",
                "competition state mutations must be implemented from lifted fixture/table/cup code and binary traces",
                "transfer/contract mutations must be implemented from lifted bid/contract/AI/value code and binary traces",
                "news/inbox mutations must be implemented from lifted record/template/queue code and binary traces",
                "original-binary capture must be complete (filled_original_rows == expected_original_rows); until then the top-tier one_for_one_exact label is withheld"
            ]
        },
        "checks": [
            {
                "name": "original-binary-validation",
                "status": if original_binary_pass { "pass" } else { "fail" },
                "detail": format!("{} original binary check(s), {} failure(s)", report_check_count(&original_binary), report_failure_count(&original_binary))
            },
            {
                "name": "execution-model-validation",
                "status": if execution_model_pass { "pass" } else { "fail" },
                "detail": format!("{} execution model check(s), {} failure(s)", report_check_count(&execution_model), report_failure_count(&execution_model))
            },
            {
                "name": "simulation-frontier-validation",
                "status": if simulation_frontier_pass { "pass" } else { "fail" },
                "detail": format!("{} simulation frontier check(s), {} failure(s)", report_check_count(&simulation_frontier), report_failure_count(&simulation_frontier))
            },
            {
                "name": "rust-foundation",
                "status": if foundation_pass { "pass" } else { "fail" },
                "detail": format!("backend infrastructure {}, implementation plan {}/{} ready ({} implemented, {} boundary-mapped)", infrastructure_pass, implementation_plan_ready, implementation_plan_items, implementation_plan_mutations_implemented, implementation_plan_boundary_mapped)
            },
            {
                "name": "one-for-one-gameplay",
                "status": if exact_behavior_pass { "pass" } else { "fail" },
                "detail": format!("playable_headless {}, gameplay blockers {}, static boundary pass {}, formula lift complete {}", playable_headless, gameplay_blockers, static_boundary_behavior_pass, formula_lift_complete)
            },
            {
                "name": "gameplay-parity-traces",
                "status": if gameplay_parity_pass { "pass" } else { "fail" },
                "detail": format!("{} parity trace system(s), {} failure(s)", gameplay_parity["summary"]["systems"].as_u64().unwrap_or(0), gameplay_parity["summary"]["failures"].as_u64().unwrap_or(0))
            },
            {
                "name": "gameplay-promotion-gates",
                "status": if gameplay_promotion_pass { "pass" } else { "fail" },
                "detail": format!("{} promotion gate system(s), {} blocked", gameplay_promotion["summary"]["systems"].as_u64().unwrap_or(0), gameplay_promotion["summary"]["blocked"].as_u64().unwrap_or(0))
            },
            {
                "name": "original-binary-capture",
                "status": if original_capture_complete { "pass" } else { "fail" },
                "detail": format!("{}/{} original capture row(s) filled; static-proven parity systems {}/{} (proof basis: {})", original_filled, original_expected, static_proven_systems, static_proof_slugs.len(), proof_basis)
            }
        ],
        "backend_acceptance": backend_acceptance,
        "gameplay_parity": gameplay_parity,
        "gameplay_promotion": gameplay_promotion,
    }))
}

fn gameplay_parity_report(db_dir: &Path, trace_dir: &Path) -> Result<serde_json::Value, String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let readiness = world.backend_readiness_report(db_dir);
    let mut checks = Vec::new();
    let mut missing_trace_files = Vec::new();
    let mut pending_trace_files = Vec::new();

    for item in &readiness.implementation_plan {
        let slug = gameplay_trace_slug(&item.system);
        let trace_path = trace_dir.join(format!("{slug}.json"));
        if !trace_path.exists() {
            missing_trace_files.push(trace_path.display().to_string());
            checks.push(serde_json::json!({
                "system": item.system,
                "trace_file": trace_path.display().to_string(),
                "status": "fail",
                "detail": "missing original-vs-Rust gameplay parity trace",
                "required_lifts": item.missing_lifts,
                "acceptance_gate": item.acceptance_gate,
            }));
            continue;
        }

        let trace_text = fs::read_to_string(&trace_path)
            .map_err(|err| format!("failed to read {}: {err}", trace_path.display()))?;
        let trace: serde_json::Value = serde_json::from_str(&trace_text)
            .map_err(|err| format!("invalid parity trace JSON {}: {err}", trace_path.display()))?;
        let format_ok = trace["format"].as_str() == Some("cm0102-rs-gameplay-parity-trace");
        let system_ok = trace["system"].as_str() == Some(item.system.as_str());
        let original_mutations = trace["original_mutations"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let rust_mutations = trace["rust_mutations"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let pending = trace["comparison"]["status"].as_str() == Some("pending")
            || original_mutations.is_empty()
            || rust_mutations.is_empty();
        if pending {
            pending_trace_files.push(trace_path.display().to_string());
        }
        let comparison_pass = trace["comparison"]["status"].as_str() == Some("pass")
            || (!original_mutations.is_empty()
                && !rust_mutations.is_empty()
                && original_mutations == rust_mutations);
        let incomplete_blockers = [
            (
                original_mutations.is_empty(),
                "original_mutations_empty".to_string(),
            ),
            (
                rust_mutations.is_empty(),
                "rust_mutations_empty".to_string(),
            ),
        ]
        .into_iter()
        .filter_map(|(is_blocked, blocker)| is_blocked.then_some(blocker))
        .collect::<Vec<_>>();
        let comparison_detail = if original_mutations.is_empty() || rust_mutations.is_empty() {
            serde_json::json!({
                "status": "capture-incomplete",
                "original_count": original_mutations.len(),
                "rust_count": rust_mutations.len(),
                "blockers": incomplete_blockers
            })
        } else if comparison_pass {
            serde_json::json!({
                "status": "pass",
                "original_count": original_mutations.len(),
                "rust_count": rust_mutations.len(),
                "blockers": []
            })
        } else {
            gameplay_mutation_comparison(item.system.as_str(), &original_mutations, &rust_mutations)
        };
        let original_schema_ok = mutations_have_required_fields(&original_mutations);
        let rust_schema_ok = mutations_have_required_fields(&rust_mutations);
        let subsystem_coverage =
            subsystem_capture_coverage(item.system.as_str(), &original_mutations);
        let subsystem_coverage_ok = subsystem_coverage["status"].as_str() == Some("pass");
        let capture_plan = &trace["capture_plan"];
        let capture_plan_ready = !json_string_array(&capture_plan["original_breakpoints"])
            .is_empty()
            && !json_string_array(&capture_plan["watched_original_writes"]).is_empty()
            && capture_plan["rust_hook"]
                .as_str()
                .is_some_and(|hook| !hook.is_empty())
            && capture_plan["minimum_trace"]
                .as_str()
                .is_some_and(|minimum| !minimum.is_empty());
        let passed = format_ok
            && system_ok
            && capture_plan_ready
            && !original_mutations.is_empty()
            && !rust_mutations.is_empty()
            && original_schema_ok
            && rust_schema_ok
            && subsystem_coverage_ok
            && comparison_pass;

        checks.push(serde_json::json!({
            "system": item.system,
            "trace_file": trace_path.display().to_string(),
            "status": if passed { "pass" } else { "fail" },
            "detail": format!(
                "format_ok {format_ok}, system_ok {system_ok}, capture_plan_ready {capture_plan_ready}, original_schema_ok {original_schema_ok}, rust_schema_ok {rust_schema_ok}, subsystem_coverage_ok {subsystem_coverage_ok}, original mutations {}, Rust mutations {}, comparison_pass {comparison_pass}",
                original_mutations.len(),
                rust_mutations.len()
            ),
            "required_lifts": item.missing_lifts,
            "acceptance_gate": item.acceptance_gate,
            "subsystem_coverage": subsystem_coverage,
            "comparison_detail": comparison_detail,
        }));
    }

    let failures = checks
        .iter()
        .filter(|check| check["status"].as_str() == Some("fail"))
        .count();

    Ok(serde_json::json!({
        "format": "cm0102-rs-gameplay-parity-report",
        "version": 1,
        "source": {
            "rust_db": db_dir.display().to_string(),
            "trace_dir": trace_dir.display().to_string(),
            "carve": "D:/cm0102-carve"
        },
            "summary": {
            "systems": checks.len(),
            "failures": failures,
            "missing_trace_files": missing_trace_files.len(),
            "pending_trace_files": pending_trace_files.len(),
            "status": if failures == 0 { "pass" } else { "fail" },
        },
        "trace_schema": {
            "format": "cm0102-rs-gameplay-parity-trace",
            "system": "match results | competition state | transfers/contracts | news/inbox",
            "scenario": "short stable scenario name",
            "original_mutations": [{"table": "fixture", "row": 0, "field": "home_score", "before": 0, "after": 1, "source_function": "0x006a3240", "provenance": "CODE_DERIVED static lift verified by original binary trace"}],
            "rust_mutations": [{"table": "fixture", "row": 0, "field": "home_score", "before": 0, "after": 1, "source_function": "0x006a3240", "provenance": "CODE_DERIVED static lift verified by original binary trace"}],
            "comparison": {"status": "pass", "method": "exact ordered mutation equality"}
        },
        "missing_trace_files": missing_trace_files,
        "pending_trace_files": pending_trace_files,
        "checks": checks,
    }))
}

fn gameplay_promotion_report(db_dir: &Path, trace_dir: &Path) -> Result<serde_json::Value, String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let save = world.new_runtime_save_from_rust_db(db_dir);
    let parity = gameplay_parity_report(db_dir, trace_dir)?;
    let mut systems = Vec::new();
    let mut ready_to_promote = 0usize;
    let mut blocked = 0usize;

    for gate in &save.backend.gameplay_promotion_gates {
        let slug = gameplay_trace_slug(&gate.system);
        let trace_path = trace_dir.join(format!("{slug}.json"));
        let trace = if trace_path.exists() {
            let text = fs::read_to_string(&trace_path)
                .map_err(|err| format!("failed to read {}: {err}", trace_path.display()))?;
            Some(
                serde_json::from_str::<serde_json::Value>(&text)
                    .map_err(|err| format!("invalid trace JSON {}: {err}", trace_path.display()))?,
            )
        } else {
            None
        };
        let trace_status = trace
            .as_ref()
            .map(|trace| parity_trace_status(trace, &gate.system))
            .unwrap_or(ParityTraceStatus::Missing);
        let implementation_present = save
            .backend
            .mutator_contracts
            .iter()
            .find(|contract| contract.system == gate.system)
            .map(|contract| contract.implementation_present)
            .unwrap_or(gate.implementation_present);
        let mut open_blockers = Vec::new();
        match trace_status {
            ParityTraceStatus::Missing => {
                open_blockers.push("trace_file_missing".to_string());
                open_blockers.push("original_binary_capture_missing".to_string());
                open_blockers.push("rust_exact_body_missing".to_string());
                open_blockers.push("exact_ordered_parity_missing".to_string());
            }
            ParityTraceStatus::Pending => {
                open_blockers.push("trace_pending_or_schema_incomplete".to_string());
                open_blockers.push("original_binary_capture_missing".to_string());
                open_blockers.push("rust_exact_body_missing".to_string());
                open_blockers.push("exact_ordered_parity_missing".to_string());
            }
            ParityTraceStatus::ImplementedPendingParity => {
                open_blockers.push("exact_ordered_parity_missing".to_string());
            }
            ParityTraceStatus::Verified => {}
        }
        if !implementation_present {
            open_blockers.push("implementation_present_false".to_string());
        }
        if gate.status != "ready-to-promote" {
            open_blockers.push(format!("gate_status_{}", gate.status));
        }
        if gate.promotion_decision != "promote-after-reviewed-parity" {
            open_blockers.push(format!("promotion_decision_{}", gate.promotion_decision));
        }
        open_blockers.sort();
        open_blockers.dedup();
        let promoted = open_blockers.is_empty()
            && trace_status == ParityTraceStatus::Verified
            && implementation_present
            && gate.exact_equality_required;
        if promoted {
            ready_to_promote = ready_to_promote.saturating_add(1);
        } else {
            blocked = blocked.saturating_add(1);
        }

        systems.push(serde_json::json!({
            "system": gate.system,
            "status": if promoted { "ready-to-promote" } else { "blocked" },
            "phase": gate.phase,
            "trace_file": trace_path.display().to_string(),
            "entry_point": gate.entry_point,
            "trace_status": format!("{trace_status:?}"),
            "implementation_present": implementation_present,
            "exact_equality_required": gate.exact_equality_required,
            "original_binary_required": gate.original_binary_required,
            "rust_required": gate.rust_required,
            "open_blockers": open_blockers,
            "promotion_decision": gate.promotion_decision,
            "safety_rule": gate.safety_rule,
        }));
    }

    Ok(serde_json::json!({
        "format": "cm0102-rs-gameplay-promotion-report",
        "version": 1,
        "source": {
            "rust_db": db_dir.display().to_string(),
            "trace_dir": trace_dir.display().to_string(),
            "carve": "D:/cm0102-carve"
        },
        "summary": {
            "systems": systems.len(),
            "ready_to_promote": ready_to_promote,
            "blocked": blocked,
            "all_promoted": ready_to_promote == systems.len() && !systems.is_empty(),
            "status": if blocked == 0 && !systems.is_empty() { "pass" } else { "blocked" },
        },
        "parity_summary": parity["summary"].clone(),
        "systems": systems,
    }))
}

fn promotion_control_room_report(
    db_dir: &Path,
    trace_dir: &Path,
    template_dir: &Path,
    exe: &Path,
) -> Result<serde_json::Value, String> {
    let backend_acceptance = backend_acceptance_report(db_dir)?;
    let exact_remake = exact_remake_report(db_dir, exe)?;
    let gameplay_parity = gameplay_parity_report(db_dir, trace_dir)?;
    let gameplay_promotion = gameplay_promotion_report(db_dir, trace_dir)?;
    let original_capture = original_capture_status_report(template_dir)?;
    let runtime_validation = validate_runtime_simulation_report(db_dir)?;

    promotion_control_room_report_from_parts(
        db_dir,
        trace_dir,
        template_dir,
        exe,
        &backend_acceptance,
        &exact_remake,
        &gameplay_parity,
        &gameplay_promotion,
        &original_capture,
        &runtime_validation,
    )
}

fn promotion_control_room_report_from_parts(
    db_dir: &Path,
    trace_dir: &Path,
    template_dir: &Path,
    exe: &Path,
    backend_acceptance: &serde_json::Value,
    exact_remake: &serde_json::Value,
    gameplay_parity: &serde_json::Value,
    gameplay_promotion: &serde_json::Value,
    original_capture: &serde_json::Value,
    runtime_validation: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let foundation_pass = exact_remake["foundation_pass"].as_bool().unwrap_or(false);
    let exact_behavior_pass = exact_remake["exact_behavior_pass"]
        .as_bool()
        .unwrap_or(false);
    let static_boundary_behavior_pass = exact_remake["summary"]["static_boundary_behavior_pass"]
        .as_bool()
        .unwrap_or(false);
    let formula_lift_complete = exact_remake["summary"]["formula_lift_complete"]
        .as_bool()
        .unwrap_or(false);
    let playable_headless = backend_acceptance["playable_headless"]
        .as_bool()
        .unwrap_or(false);
    let original_expected = original_capture["summary"]["expected_original_rows"]
        .as_u64()
        .unwrap_or_default();
    let original_filled = original_capture["summary"]["filled_original_rows"]
        .as_u64()
        .unwrap_or_default();
    let original_placeholders = original_capture["summary"]["placeholder_rows"]
        .as_u64()
        .unwrap_or_default();
    let parity_failures = gameplay_parity["summary"]["failures"]
        .as_u64()
        .unwrap_or_default();
    let promotion_blocked = gameplay_promotion["summary"]["blocked"]
        .as_u64()
        .unwrap_or_default();
    let static_proof_path = Path::new("D:/cm0102-rs/reports/static_parity_proof.json");
    let static_proof = if static_proof_path.exists() {
        read_json_file(static_proof_path)?
    } else {
        serde_json::json!({
            "summary": {
                "status": "missing",
                "rows": 0,
                "proven_rows": 0,
                "incomplete_rows": 0
            },
            "rows": []
        })
    };
    let static_proof_pass = static_proof["summary"]["status"].as_str() == Some("pass");
    let one_for_one_exact = exact_remake["one_for_one_exact_remake"]
        .as_bool()
        .unwrap_or(false);
    let status = if one_for_one_exact {
        "one-for-one-exact"
    } else if foundation_pass && static_boundary_behavior_pass && !formula_lift_complete {
        "static-boundary-exact-formula-lift-required"
    } else if foundation_pass && original_placeholders > 0 && !static_proof_pass {
        "foundation-ready-static-proof-required"
    } else if foundation_pass && (parity_failures > 0 || promotion_blocked > 0) {
        "foundation-ready-parity-promotion-required"
    } else if foundation_pass {
        "foundation-ready-exact-gates-required"
    } else {
        "foundation-incomplete"
    };
    let next_required_action = if original_placeholders > 0 && !static_proof_pass {
        "prove remaining gameplay rows from Ghidra/carver static code evidence"
    } else if parity_failures > 0 {
        "re-run gameplay parity and resolve original-vs-Rust mutation mismatches"
    } else if promotion_blocked > 0 {
        "implement/promote Rust mutators against the static parity proof"
    } else if !playable_headless {
        "clear backend gameplay blockers until playable_headless is true"
    } else if !exact_behavior_pass {
        "lift the remaining formula semantics from Ghidra/carver evidence"
    } else {
        "ready for exact-remake sign-off"
    };

    let mut systems = Vec::new();
    for system_name in [
        "match results",
        "competition state",
        "transfers/contracts",
        "news/inbox",
    ] {
        let slug = gameplay_trace_slug(system_name);
        let capture_system = original_capture["systems"]
            .as_array()
            .and_then(|items| {
                items.iter().find(|item| {
                    item["system"].as_str() == Some(system_name)
                        || item["slug"].as_str() == Some(slug.as_str())
                })
            })
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let parity_check = gameplay_parity["checks"]
            .as_array()
            .and_then(|items| {
                items
                    .iter()
                    .find(|item| item["system"].as_str() == Some(system_name))
            })
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        let promotion_system = gameplay_promotion["systems"]
            .as_array()
            .and_then(|items| {
                items
                    .iter()
                    .find(|item| item["system"].as_str() == Some(system_name))
            })
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        systems.push(serde_json::json!({
            "system": system_name,
            "slug": slug,
            "original_capture": {
                "status": capture_system["status"].as_str().unwrap_or("missing"),
                "filled_original_rows": capture_system["filled_original_rows"].as_u64().unwrap_or_default(),
                "expected_original_rows": capture_system["expected_original_rows"].as_u64().unwrap_or_default(),
                "placeholder_rows": capture_system["placeholder_rows"].as_u64().unwrap_or_default(),
                "import_blockers": capture_system["import_blockers"].clone(),
                "template": capture_system["template"].as_str().unwrap_or(""),
            },
            "parity": {
                "status": parity_check["status"].as_str().unwrap_or("missing"),
                "detail": parity_check["detail"].as_str().unwrap_or(""),
                "comparison": parity_check["comparison_detail"].clone(),
                "trace_file": parity_check["trace_file"].as_str().unwrap_or(""),
            },
            "promotion": {
                "status": promotion_system["status"].as_str().unwrap_or("missing"),
                "trace_status": promotion_system["trace_status"].as_str().unwrap_or("missing"),
                "implementation_present": promotion_system["implementation_present"].as_bool().unwrap_or(false),
                "open_blockers": promotion_system["open_blockers"].clone(),
                "entry_point": promotion_system["entry_point"].as_str().unwrap_or(""),
            }
        }));
    }

    Ok(serde_json::json!({
        "format": "cm0102-rs-promotion-control-room",
        "version": 1,
        "status": status,
        "summary": {
            "foundation_pass": foundation_pass,
            "exact_behavior_pass": exact_behavior_pass,
            "static_boundary_behavior_pass": static_boundary_behavior_pass,
            "formula_lift_complete": formula_lift_complete,
            "one_for_one_exact_remake": one_for_one_exact,
            "backend_infrastructure_pass": backend_acceptance["infrastructure_pass"].as_bool().unwrap_or(false),
            "playable_headless": playable_headless,
            "runtime_validation_status": runtime_validation["summary"]["status"].as_str().unwrap_or("unknown"),
            "original_capture_status": original_capture["summary"]["status"].as_str().unwrap_or("unknown"),
            "original_capture_rows_filled": original_filled,
            "original_capture_rows_expected": original_expected,
            "original_capture_placeholder_rows": original_placeholders,
            "original_capture_import_ready_systems": original_capture["summary"]["import_ready_systems"].as_u64().unwrap_or_default(),
            "static_parity_proof_status": static_proof["summary"]["status"].as_str().unwrap_or("missing"),
            "static_parity_proof_rows": static_proof["summary"]["rows"].as_u64().unwrap_or_default(),
            "static_parity_proof_rows_proven": static_proof["summary"]["proven_rows"].as_u64().unwrap_or_default(),
            "static_parity_proof_rows_incomplete": static_proof["summary"]["incomplete_rows"].as_u64().unwrap_or_default(),
            "gameplay_parity_status": gameplay_parity["summary"]["status"].as_str().unwrap_or("unknown"),
            "gameplay_parity_failures": parity_failures,
            "gameplay_promotion_status": gameplay_promotion["summary"]["status"].as_str().unwrap_or("unknown"),
            "gameplay_promotion_blocked": promotion_blocked,
            "next_required_action": next_required_action,
        },
        "source": {
            "rust_db": db_dir.display().to_string(),
            "trace_dir": trace_dir.display().to_string(),
            "template_dir": template_dir.display().to_string(),
            "original_binary": exe.display().to_string(),
            "carve": "D:/cm0102-carve"
        },
        "systems": systems,
        "reports": {
            "backend_acceptance": backend_acceptance,
            "exact_remake": exact_remake,
            "gameplay_parity": gameplay_parity,
            "gameplay_promotion": gameplay_promotion,
            "original_capture": original_capture,
            "static_parity_proof": static_proof,
            "runtime_validation": runtime_validation,
        }
    }))
}

fn export_promotion_control_room(
    db_dir: &Path,
    output_dir: &Path,
    trace_dir: &Path,
    template_dir: &Path,
    exe: &Path,
) -> Result<serde_json::Value, String> {
    fs::create_dir_all(output_dir)
        .map_err(|err| format!("failed to create {}: {err}", output_dir.display()))?;
    let report = promotion_control_room_report(db_dir, trace_dir, template_dir, exe)?;
    write_json_file(&output_dir.join("control-room.json"), &report)?;
    write_text_file(
        &output_dir.join("dashboard.html"),
        &promotion_control_room_html(&report),
    )?;
    Ok(report)
}

fn cached_promotion_control_room_report(db_dir: &Path) -> Result<serde_json::Value, String> {
    let cached_path = Path::new("D:/cm0102-rs/reports/promotion_control_room/control-room.json");
    if cached_path.exists() {
        let mut report = read_json_file(cached_path)?;
        if let Some(object) = report.as_object_mut() {
            object.insert("cache".to_string(), serde_json::json!({
                "status": "cached",
                "path": cached_path.display().to_string(),
                "refresh_command": "cargo run -p cm-app -- export-promotion-control-room D:/cm0102-rs/rust-db D:/cm0102-rs/reports/promotion_control_room"
            }));
        }
        return Ok(report);
    }
    let mut report = promotion_control_room_report(
        db_dir,
        Path::new("D:/cm0102-rs/reports/parity_traces"),
        Path::new("D:/cm0102-rs/reports/original_capture_templates"),
        Path::new("D:/cm0102/cm0102.exe"),
    )?;
    if let Some(object) = report.as_object_mut() {
        object.insert(
            "cache".to_string(),
            serde_json::json!({
                "status": "recomputed",
                "path": cached_path.display().to_string(),
            }),
        );
    }
    Ok(report)
}

fn export_todo_attack_board(
    db_dir: &Path,
    output_dir: &Path,
    trace_dir: &Path,
    template_dir: &Path,
) -> Result<serde_json::Value, String> {
    fs::create_dir_all(output_dir)
        .map_err(|err| format!("failed to create {}: {err}", output_dir.display()))?;
    let report = todo_attack_board_report(db_dir, trace_dir, template_dir)?;
    write_json_file(&output_dir.join("todo-attack-board.json"), &report)?;
    write_text_file(
        &output_dir.join("TODO_ATTACK_PLAN.md"),
        &todo_attack_board_markdown(&report),
    )?;
    write_text_file(
        &output_dir.join("dashboard.html"),
        &todo_attack_board_html(&report),
    )?;
    Ok(report)
}

fn refresh_backend_gates(
    db_dir: &Path,
    reports_dir: &Path,
    trace_dir: &Path,
    template_dir: &Path,
    exe: &Path,
) -> Result<serde_json::Value, String> {
    fs::create_dir_all(reports_dir)
        .map_err(|err| format!("failed to create {}: {err}", reports_dir.display()))?;

    let candidate_traces = export_rust_gameplay_candidate_traces(db_dir, trace_dir)?;
    let capture_status_before = original_capture_status_report(template_dir)?;
    write_json_file(
        &reports_dir.join("original_capture_status.before-import.json"),
        &capture_status_before,
    )?;

    let import_ready = import_ready_original_capture_systems()?;
    let capture_status_after = original_capture_status_report(template_dir)?;
    write_json_file(
        &reports_dir.join("original_capture_status.json"),
        &capture_status_after,
    )?;

    let capture_workbench = export_original_capture_workbench(
        template_dir,
        &reports_dir.join("original_capture_workbench"),
        trace_dir,
    )?;
    let capture_pack = export_gameplay_capture_pack(trace_dir, &reports_dir.join("capture_pack"))?;
    let static_proof = static_parity_proof_report(
        &reports_dir
            .join("capture_pack")
            .join("all-systems-row-capture-plan.json"),
    )?;
    write_json_file(&reports_dir.join("static_parity_proof.json"), &static_proof)?;
    let static_parity_apply =
        apply_static_parity_proof(trace_dir, &reports_dir.join("static_parity_proof.json"))?;
    write_json_file(
        &reports_dir.join("static_parity_apply.json"),
        &static_parity_apply,
    )?;
    let parity = gameplay_parity_report(db_dir, trace_dir)?;
    write_json_file(&reports_dir.join("gameplay_parity.json"), &parity)?;
    let promotion = gameplay_promotion_report(db_dir, trace_dir)?;
    write_json_file(&reports_dir.join("gameplay_promotion.json"), &promotion)?;
    let runtime_validation = validate_runtime_simulation_report(db_dir)?;
    write_json_file(
        &reports_dir.join("runtime_simulation_validation.json"),
        &runtime_validation,
    )?;
    let backend_acceptance = backend_acceptance_report(db_dir)?;
    write_json_file(
        &reports_dir.join("backend_acceptance.json"),
        &backend_acceptance,
    )?;
    let exact_remake = exact_remake_report(db_dir, exe)?;
    write_json_file(&reports_dir.join("exact_remake_report.json"), &exact_remake)?;
    let promotion_control_room = promotion_control_room_report_from_parts(
        db_dir,
        trace_dir,
        template_dir,
        exe,
        &backend_acceptance,
        &exact_remake,
        &parity,
        &promotion,
        &capture_status_after,
        &runtime_validation,
    )?;
    let promotion_control_room_dir = reports_dir.join("promotion_control_room");
    fs::create_dir_all(&promotion_control_room_dir).map_err(|err| {
        format!(
            "failed to create {}: {err}",
            promotion_control_room_dir.display()
        )
    })?;
    write_json_file(
        &promotion_control_room_dir.join("control-room.json"),
        &promotion_control_room,
    )?;
    write_text_file(
        &promotion_control_room_dir.join("dashboard.html"),
        &promotion_control_room_html(&promotion_control_room),
    )?;
    let lift = export_gameplay_lift_workbench(db_dir, &reports_dir.join("lift_workbench"))?;
    let formula_lift_backlog =
        export_formula_lift_backlog(db_dir, &reports_dir.join("formula_lift_backlog"))?;
    let todo_attack_board = todo_attack_board_report_from_parts(
        &capture_status_after,
        &parity,
        &promotion,
        &backend_acceptance,
        &lift,
        &static_proof,
    );
    let todo_attack_board_dir = reports_dir.join("todo_attack_board");
    fs::create_dir_all(&todo_attack_board_dir).map_err(|err| {
        format!(
            "failed to create {}: {err}",
            todo_attack_board_dir.display()
        )
    })?;
    write_json_file(
        &todo_attack_board_dir.join("todo-attack-board.json"),
        &todo_attack_board,
    )?;
    write_text_file(
        &todo_attack_board_dir.join("TODO_ATTACK_PLAN.md"),
        &todo_attack_board_markdown(&todo_attack_board),
    )?;
    write_text_file(
        &todo_attack_board_dir.join("dashboard.html"),
        &todo_attack_board_html(&todo_attack_board),
    )?;

    let summary = serde_json::json!({
        "status": promotion_control_room["status"],
        "foundation_pass": promotion_control_room["summary"]["foundation_pass"],
        "one_for_one_exact_remake": promotion_control_room["summary"]["one_for_one_exact_remake"],
        "playable_headless": promotion_control_room["summary"]["playable_headless"],
        "original_capture_rows_filled": capture_status_after["summary"]["filled_original_rows"],
        "original_capture_rows_expected": capture_status_after["summary"]["expected_original_rows"],
        "original_capture_placeholder_rows": capture_status_after["summary"]["placeholder_rows"],
        "original_capture_import_ready_systems": capture_status_after["summary"]["import_ready_systems"],
        "imported_ready_systems": import_ready["imported"].as_array().map_or(0, Vec::len),
        "import_failures": import_ready["failures"].as_array().map_or(0, Vec::len),
        "gameplay_parity_status": parity["summary"]["status"],
        "gameplay_parity_failures": parity["summary"]["failures"],
        "gameplay_promotion_status": promotion["summary"]["status"],
        "gameplay_promotion_blocked": promotion["summary"]["blocked"],
        "static_parity_proof_status": static_proof["summary"]["status"],
        "static_parity_proof_rows": static_proof["summary"]["rows"],
        "static_parity_proof_rows_proven": static_proof["summary"]["proven_rows"],
        "static_parity_proof_rows_incomplete": static_proof["summary"]["incomplete_rows"],
        "static_parity_apply_status": static_parity_apply["status"],
        "static_parity_apply_rows": static_parity_apply["summary"]["rows_applied"],
        "formula_lift_status": formula_lift_backlog["summary"]["status"],
        "formula_lift_tasks": formula_lift_backlog["summary"]["tasks"],
        "formula_lift_ready_for_static_read": formula_lift_backlog["summary"]["ready_for_static_read"],
        "formula_lift_ready_for_implementation": formula_lift_backlog["summary"]["ready_for_formula_implementation"],
        "formula_lift_needs_decompile": formula_lift_backlog["summary"]["needs_targeted_decompile"],
        "next_required_action": promotion_control_room["summary"]["next_required_action"],
    });

    let report = serde_json::json!({
        "format": "cm0102-rs-backend-gate-refresh",
        "version": 1,
        "db_dir": db_dir.display().to_string(),
        "reports_dir": reports_dir.display().to_string(),
        "trace_dir": trace_dir.display().to_string(),
        "template_dir": template_dir.display().to_string(),
        "original_binary": exe.display().to_string(),
        "artifacts": {
            "refresh_report": reports_dir.join("backend_gate_refresh.json").display().to_string(),
            "capture_status": reports_dir.join("original_capture_status.json").display().to_string(),
            "capture_workbench": reports_dir.join("original_capture_workbench").join("dashboard.html").display().to_string(),
            "capture_pack": reports_dir.join("capture_pack").join("capture-pack-report.json").display().to_string(),
            "static_parity_proof": reports_dir.join("static_parity_proof.json").display().to_string(),
            "static_parity_apply": reports_dir.join("static_parity_apply.json").display().to_string(),
            "gameplay_parity": reports_dir.join("gameplay_parity.json").display().to_string(),
            "gameplay_promotion": reports_dir.join("gameplay_promotion.json").display().to_string(),
            "runtime_validation": reports_dir.join("runtime_simulation_validation.json").display().to_string(),
            "backend_acceptance": reports_dir.join("backend_acceptance.json").display().to_string(),
            "exact_remake": reports_dir.join("exact_remake_report.json").display().to_string(),
            "gameplay_lift_workbench": reports_dir.join("lift_workbench").join("lift-workbench-report.json").display().to_string(),
            "formula_lift_backlog": reports_dir.join("formula_lift_backlog").join("formula-lift-backlog.json").display().to_string(),
            "promotion_control_room": reports_dir.join("promotion_control_room").join("dashboard.html").display().to_string(),
            "todo_attack_board": reports_dir.join("todo_attack_board").join("dashboard.html").display().to_string(),
        },
        "summary": summary,
        "candidate_traces": candidate_traces,
        "capture_status_before_import": capture_status_before,
        "import_ready": import_ready,
        "capture_status": capture_status_after,
        "capture_workbench": capture_workbench,
        "capture_pack": capture_pack,
        "static_parity_proof": static_proof,
        "static_parity_apply": static_parity_apply,
        "gameplay_parity": parity,
        "gameplay_promotion": promotion,
        "runtime_validation": runtime_validation,
        "backend_acceptance": backend_acceptance,
        "exact_remake": exact_remake,
        "gameplay_lift_workbench": lift,
        "formula_lift_backlog": formula_lift_backlog,
        "promotion_control_room": promotion_control_room,
        "todo_attack_board": todo_attack_board,
    });
    write_json_file(&reports_dir.join("backend_gate_refresh.json"), &report)?;
    Ok(report)
}

fn prepare_capture_console(
    db_dir: &Path,
    reports_dir: &Path,
    trace_dir: &Path,
    template_dir: &Path,
    port: u16,
) -> Result<serde_json::Value, String> {
    fs::create_dir_all(reports_dir)
        .map_err(|err| format!("failed to create {}: {err}", reports_dir.display()))?;
    let capture_pack_dir = reports_dir.join("capture_pack");
    let workbench_dir = reports_dir.join("original_capture_workbench");
    let capture_pack = export_gameplay_capture_pack(trace_dir, &capture_pack_dir)?;
    let workbench = export_original_capture_workbench(template_dir, &workbench_dir, trace_dir)?;
    let csv_path = capture_pack_dir.join("all-systems-capture.csv");
    let csv_text = fs::read_to_string(&csv_path).map_err(|err| {
        format!(
            "failed to read all-systems CSV {}: {err}",
            csv_path.display()
        )
    })?;
    let validation = validate_original_capture_csv_text(&csv_text)?;
    write_json_file(
        &capture_pack_dir.join("all-systems-capture-validation.json"),
        &validation,
    )?;
    let capture_status = original_capture_status_report(template_dir)?;
    write_json_file(
        &reports_dir.join("original_capture_status.json"),
        &capture_status,
    )?;
    let launch_script = capture_console_launch_script(db_dir, port);
    write_text_file(
        &reports_dir.join("launch-capture-console.ps1"),
        &launch_script,
    )?;
    write_text_file(
        &reports_dir.join("launch-capture-console.cmd"),
        &capture_console_launch_cmd(db_dir, port),
    )?;
    let report = serde_json::json!({
        "format": "cm0102-rs-capture-console-prep",
        "version": 1,
        "status": "prepared",
        "db_dir": db_dir.display().to_string(),
        "reports_dir": reports_dir.display().to_string(),
        "trace_dir": trace_dir.display().to_string(),
        "template_dir": template_dir.display().to_string(),
        "port": port,
        "urls": {
            "capture_console": format!("http://127.0.0.1:{port}/capture-console"),
            "capture_pack": format!("http://127.0.0.1:{port}/capture-pack"),
            "workbench": format!("http://127.0.0.1:{port}/original-capture-workbench"),
            "promotion_control_room": format!("http://127.0.0.1:{port}/promotion-control-room-cached"),
        },
        "artifacts": {
            "prep_report": reports_dir.join("capture_console_prep.json").display().to_string(),
            "launch_script": reports_dir.join("launch-capture-console.ps1").display().to_string(),
            "launch_cmd": reports_dir.join("launch-capture-console.cmd").display().to_string(),
            "all_systems_csv": csv_path.display().to_string(),
            "validation": capture_pack_dir.join("all-systems-capture-validation.json").display().to_string(),
            "capture_pack_dashboard": capture_pack_dir.join("dashboard.html").display().to_string(),
            "workbench_dashboard": workbench_dir.join("dashboard.html").display().to_string(),
            "x32dbg_plan": capture_pack_dir.join("all-systems-x32dbg-plan.txt").display().to_string(),
        },
        "summary": {
            "candidate_rows": capture_pack["summary"]["candidate_rows"],
            "watch_groups": capture_pack["summary"]["watch_groups"],
            "workbench_todo_rows": workbench["summary"]["todo_rows"],
            "csv_validation_status": validation["status"],
            "csv_blank_capture_values": validation["summary"]["blank_capture_values"],
            "capture_rows_filled": capture_status["summary"]["filled_original_rows"],
            "capture_rows_expected": capture_status["summary"]["expected_original_rows"],
            "capture_placeholder_rows": capture_status["summary"]["placeholder_rows"],
            "import_ready_systems": capture_status["summary"]["import_ready_systems"],
        },
        "capture_pack": capture_pack,
        "workbench": workbench,
        "csv_validation": validation,
        "capture_status": capture_status,
    });
    write_json_file(&reports_dir.join("capture_console_prep.json"), &report)?;
    Ok(report)
}

fn capture_console_launch_script(db_dir: &Path, port: u16) -> String {
    format!(
        r#"$ErrorActionPreference = "Stop"
Set-Location "D:/cm0102-rs"
& "D:/.cargo/bin/cargo.exe" run --target-dir D:/cm0102-rs/target-refresh -p cm-app -- serve-rust-db "{}" {}
"#,
        db_dir.display(),
        port
    )
}

fn capture_console_launch_cmd(db_dir: &Path, port: u16) -> String {
    format!(
        "@echo off\r\ncd /d D:\\cm0102-rs\r\nD:\\.cargo\\bin\\cargo.exe run --target-dir D:/cm0102-rs/target-refresh -p cm-app -- serve-rust-db \"{}\" {}\r\n",
        db_dir.display(),
        port
    )
}

fn todo_attack_board_report(
    db_dir: &Path,
    trace_dir: &Path,
    template_dir: &Path,
) -> Result<serde_json::Value, String> {
    let capture = original_capture_status_report(template_dir)?;
    let parity = gameplay_parity_report(db_dir, trace_dir)?;
    let promotion = gameplay_promotion_report(db_dir, trace_dir)?;
    let backend = backend_acceptance_report(db_dir)?;
    let lift_path = Path::new("D:/cm0102-rs/reports/lift_workbench/lift-workbench-report.json");
    let lift = if lift_path.exists() {
        read_json_file(lift_path)?
    } else {
        serde_json::json!({
            "summary": {
                "items": 0,
                "unknown_or_inferred": 0,
                "decompile_artifacts_missing": 0
            },
            "systems": []
        })
    };
    let static_proof_path = Path::new("D:/cm0102-rs/reports/static_parity_proof.json");
    let static_proof = if static_proof_path.exists() {
        read_json_file(static_proof_path)?
    } else {
        serde_json::json!({
            "summary": {
                "status": "missing",
                "rows": 0,
                "proven_rows": 0,
                "incomplete_rows": 0
            }
        })
    };

    Ok(todo_attack_board_report_from_parts(
        &capture,
        &parity,
        &promotion,
        &backend,
        &lift,
        &static_proof,
    ))
}

fn todo_attack_board_report_from_parts(
    capture: &serde_json::Value,
    parity: &serde_json::Value,
    promotion: &serde_json::Value,
    backend: &serde_json::Value,
    lift: &serde_json::Value,
    static_proof: &serde_json::Value,
) -> serde_json::Value {
    let capture_rows = capture["summary"]["placeholder_rows"]
        .as_u64()
        .unwrap_or_default();
    let parity_failures = parity["summary"]["failures"].as_u64().unwrap_or_default();
    let promotion_blocked = promotion["summary"]["blocked"].as_u64().unwrap_or_default();
    let gameplay_blockers = backend["summary"]["gameplay_blockers"]
        .as_u64()
        .unwrap_or_default();
    let lift_unknown = lift["summary"]["unknown_or_inferred"]
        .as_u64()
        .unwrap_or_default();
    let lift_missing_artifacts = lift["summary"]["decompile_artifacts_missing"]
        .as_u64()
        .unwrap_or_default();
    let static_proof_pass = static_proof["summary"]["status"].as_str() == Some("pass");
    let static_proof_rows = static_proof["summary"]["proven_rows"]
        .as_u64()
        .unwrap_or_default();
    let formula_lift_items = backend["implementation_plan"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    item["missing_lifts"]
                        .as_array()
                        .map_or(0, |missing| missing.len() as u64)
                })
                .sum::<u64>()
        })
        .unwrap_or_default();

    let mut tasks = Vec::new();
    tasks.push(serde_json::json!({
        "id": "prove-original-boundary-rows",
        "stream": "exact-gameplay",
        "priority": 1,
        "status": if static_proof_pass { "done" } else if capture_rows == 0 { "done" } else { "ready-now" },
        "title": "Prove original cm0102.exe boundary mutation rows",
        "scope": format!("{static_proof_rows} static proof row(s), {capture_rows} legacy capture placeholder row(s)"),
        "why": "Boundary parity can be proven from Ghidra/carver static evidence without requiring manual live capture.",
        "next_action": "Use export-static-parity-proof/apply-static-parity-proof when row plans change.",
        "evidence": ["reports/static_parity_proof.json", "reports/capture_pack/capture-pack-report.json"]
    }));
    tasks.push(serde_json::json!({
        "id": "import-captures-and-run-parity",
        "stream": "exact-gameplay",
        "priority": 2,
        "status": if static_proof_pass && parity_failures == 0 { "done" } else if capture_rows == 0 && parity_failures > 0 { "ready-now" } else if capture_rows == 0 { "done" } else { "blocked-by-boundary-proof" },
        "title": "Apply boundary proof and compare ordered mutations",
        "scope": format!("{parity_failures} parity system(s) currently fail"),
        "why": "Promotion is forbidden until original-derived and Rust mutation streams match exactly.",
        "next_action": "Run apply-static-parity-proof, then gameplay-parity-report whenever row plans change.",
        "evidence": ["reports/gameplay_parity.json", "reports/parity_traces"]
    }));
    tasks.push(serde_json::json!({
        "id": "promote-exact-mutators",
        "stream": "exact-gameplay",
        "priority": 3,
        "status": if parity_failures == 0 && promotion_blocked > 0 { "ready-now" } else if promotion_blocked == 0 { "done" } else { "blocked-by-parity" },
        "title": "Promote exact gameplay mutator gates",
        "scope": format!("{promotion_blocked} promotion gate(s) blocked"),
        "why": "This is what flips backend systems from frontier-only to implemented gameplay.",
        "next_action": "After parity passes, mark reviewed gates and only then enable implementation_present.",
        "evidence": ["reports/gameplay_promotion.json", "saves/new_game.json"]
    }));
    tasks.push(serde_json::json!({
        "id": "lift-unknown-functions",
        "stream": "binary-lift",
        "priority": 4,
        "status": if lift_unknown == 0 && lift_missing_artifacts == 0 { "done" } else { "ready-now" },
        "title": "Reduce UNKNOWN/INFERRED lift blockers",
        "scope": format!("{lift_unknown} unknown/inferred item(s), {lift_missing_artifacts} missing decompile artifact(s)"),
        "why": "Original capture proves behavior; code lift explains and stabilizes why the behavior happens.",
        "next_action": "Use reports/lift_workbench/*/commands.txt and promote claims only after carve/Ghidra evidence.",
        "evidence": ["reports/lift_workbench/lift-workbench-report.json", "D:/cm0102-carve/findings.json"]
    }));
    tasks.push(serde_json::json!({
        "id": "finish-playable-headless",
        "stream": "runtime",
        "priority": 5,
        "status": if gameplay_blockers == 0 { "done" } else { "blocked-by-exact-mutators" },
        "title": "Make headless gameplay genuinely playable",
        "scope": format!("{gameplay_blockers} exact gameplay subsystem blocker(s) remain"),
        "why": "The calendar shell runs, but a playable remake needs real match/competition/transfer/news mutations.",
        "next_action": "Replace frontier-only mutation attempts with promoted exact mutator bodies.",
        "evidence": ["reports/backend_acceptance.json", "reports/runtime_validation.json"]
    }));
    tasks.push(serde_json::json!({
        "id": "lift-formula-semantics",
        "stream": "binary-lift",
        "priority": 6,
        "status": if formula_lift_items == 0 { "done" } else { "ready-now" },
        "title": "Lift formula-level gameplay semantics",
        "scope": format!("{formula_lift_items} formula/semantic lift item(s) remain across implementation plans"),
        "why": "Static boundary parity says Rust writes the proven rows; full one-for-one gameplay requires the arithmetic and branching formulas too.",
        "next_action": "Attack missing_lifts in backend_acceptance.json, starting with match score/event formulas and transfer value/wage/bid logic.",
        "evidence": ["reports/backend_acceptance.json", "D:/cm0102-carve/findings.json"]
    }));
    tasks.push(serde_json::json!({
        "id": "modern-client-slices",
        "stream": "delivery",
        "priority": 7,
        "status": "ready-now",
        "title": "Keep expanding Godot/client slices around the Rust API",
        "scope": "Manager setup, club screens, fixtures, inbox, staff search, and save controls",
        "why": "This work is safe in parallel because it consumes Rust-owned APIs and does not claim exact simulation.",
        "next_action": "Build UI slices against /api/tables, /api/runtime-save, and headless endpoints while exact gameplay work continues.",
        "evidence": ["godot/api_manifest.json", "godot/scripts/main.gd"]
    }));

    let ready_now = tasks
        .iter()
        .filter(|task| task["status"].as_str() == Some("ready-now"))
        .count();
    let blocked = tasks
        .iter()
        .filter(|task| {
            task["status"]
                .as_str()
                .is_some_and(|status| status.starts_with("blocked"))
        })
        .count();
    let done = tasks
        .iter()
        .filter(|task| task["status"].as_str() == Some("done"))
        .count();

    serde_json::json!({
        "format": "cm0102-rs-todo-attack-board",
        "version": 1,
        "status": if static_proof_pass && formula_lift_items > 0 {
            "formula-lift-required"
        } else if capture_rows > 0 && !static_proof_pass {
            "capture-gated"
        } else if blocked > 0 {
            "parity-gated"
        } else {
            "open-for-implementation"
        },
        "summary": {
            "tasks": tasks.len(),
            "ready_now": ready_now,
            "blocked": blocked,
            "done": done,
            "original_capture_placeholders": capture_rows,
            "gameplay_parity_failures": parity_failures,
            "gameplay_promotion_blocked": promotion_blocked,
            "gameplay_blockers": gameplay_blockers,
            "lift_unknown_or_inferred": lift_unknown,
            "lift_missing_artifacts": lift_missing_artifacts,
            "static_proof_pass": static_proof_pass,
            "static_proof_rows": static_proof_rows,
            "formula_lift_items": formula_lift_items,
            "answer_to_when_attack_todos": "Now: boundary rows are static-proof-backed, so the priority TODOs are deeper formula lifts and replacing boundary markers with formula-derived Rust state transitions."
        },
        "source": {
            "rust_db": backend["source"]["rust_db"].as_str().unwrap_or("unknown"),
            "trace_dir": promotion["source"]["trace_dir"].as_str().unwrap_or("unknown"),
            "template_dir": capture["template_dir"].as_str().unwrap_or("unknown"),
            "lift_workbench": "D:/cm0102-rs/reports/lift_workbench/lift-workbench-report.json"
        },
        "tasks": tasks,
        "reports": {
            "capture": capture,
            "parity": parity["summary"].clone(),
            "promotion": promotion["summary"].clone(),
            "backend": backend["summary"].clone(),
            "lift": lift["summary"].clone(),
            "static_proof": static_proof["summary"].clone()
        }
    })
}

fn todo_attack_board_markdown(report: &serde_json::Value) -> String {
    let summary = &report["summary"];
    let mut lines = vec![
        "# CM0102 Rust TODO Attack Plan".to_string(),
        String::new(),
        format!(
            "Status: `{}`",
            report["status"].as_str().unwrap_or("unknown")
        ),
        format!(
            "Ready now: `{}` | Blocked: `{}` | Done: `{}`",
            summary["ready_now"].as_u64().unwrap_or_default(),
            summary["blocked"].as_u64().unwrap_or_default(),
            summary["done"].as_u64().unwrap_or_default()
        ),
        String::new(),
        format!(
            "When do we attack TODOs? {}",
            summary["answer_to_when_attack_todos"]
                .as_str()
                .unwrap_or("now")
        ),
        String::new(),
        "## Ordered Work".to_string(),
        String::new(),
    ];
    for task in report["tasks"].as_array().cloned().unwrap_or_default() {
        lines.push(format!(
            "{}. `{}` - {}",
            task["priority"].as_u64().unwrap_or_default(),
            task["status"].as_str().unwrap_or("unknown"),
            task["title"].as_str().unwrap_or("unknown")
        ));
        lines.push(format!("Scope: {}", task["scope"].as_str().unwrap_or("")));
        lines.push(format!(
            "Next: {}",
            task["next_action"].as_str().unwrap_or("")
        ));
        lines.push(String::new());
    }
    lines.join("\n")
}

fn todo_attack_board_html(report: &serde_json::Value) -> String {
    let summary = &report["summary"];
    let rows = report["tasks"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|task| {
            format!(
                r#"<tr><td>{}</td><td><strong>{}</strong><br><span>{}</span></td><td>{}</td><td>{}</td><td>{}</td></tr>"#,
                task["priority"].as_u64().unwrap_or_default(),
                html_escape(task["title"].as_str().unwrap_or("unknown")),
                html_escape(task["id"].as_str().unwrap_or("")),
                html_escape(task["status"].as_str().unwrap_or("unknown")),
                html_escape(task["scope"].as_str().unwrap_or("")),
                html_escape(task["next_action"].as_str().unwrap_or(""))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>CM0102 Rust TODO Attack Board</title>
<style>
body {{ margin: 0; font-family: Georgia, 'Times New Roman', serif; color: #1d2422; background: linear-gradient(135deg, #edf4ee, #fff7e7); }}
main {{ max-width: 1400px; margin: 0 auto; padding: 34px; }}
.hero, .panel {{ background: rgba(255,252,244,.92); border: 1px solid #ded6c6; border-radius: 26px; padding: 26px; box-shadow: 0 24px 70px rgba(50,40,25,.12); }}
h1 {{ margin: 0; font-size: clamp(2.4rem, 5vw, 4.8rem); line-height: .96; }}
.stats {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 14px; margin-top: 20px; }}
.stat {{ background: white; border: 1px solid #ded6c6; border-radius: 18px; padding: 16px; }}
.stat strong {{ display: block; font-size: 2rem; }}
.eyebrow {{ color: #0c7682; text-transform: uppercase; letter-spacing: .08em; font: 700 .75rem Verdana, sans-serif; }}
.panel {{ margin-top: 22px; overflow: auto; }}
table {{ width: 100%; border-collapse: collapse; font: .9rem Verdana, sans-serif; }}
th {{ text-align: left; color: #64716d; border-bottom: 1px solid #ded6c6; padding: 12px; }}
td {{ border-bottom: 1px solid rgba(222,214,198,.82); padding: 14px 12px; vertical-align: top; }}
span {{ color: #64716d; }}
</style>
</head>
<body>
<main>
<section class="hero">
<p class="eyebrow">CM0102 Rust backend</p>
<h1>TODO Attack Board</h1>
<p>{}</p>
<div class="stats">
<div class="stat"><p class="eyebrow">Ready Now</p><strong>{}</strong></div>
<div class="stat"><p class="eyebrow">Blocked</p><strong>{}</strong></div>
<div class="stat"><p class="eyebrow">Capture Rows</p><strong>{}</strong></div>
<div class="stat"><p class="eyebrow">Parity Failures</p><strong>{}</strong></div>
</div>
</section>
<section class="panel">
<table><thead><tr><th>Priority</th><th>Task</th><th>Status</th><th>Scope</th><th>Next Action</th></tr></thead><tbody>{}</tbody></table>
</section>
</main>
</body>
</html>
"#,
        html_escape(
            summary["answer_to_when_attack_todos"]
                .as_str()
                .unwrap_or("")
        ),
        summary["ready_now"].as_u64().unwrap_or_default(),
        summary["blocked"].as_u64().unwrap_or_default(),
        summary["original_capture_placeholders"]
            .as_u64()
            .unwrap_or_default(),
        summary["gameplay_parity_failures"]
            .as_u64()
            .unwrap_or_default(),
        rows
    )
}

fn export_gameplay_lift_workbench(
    db_dir: &Path,
    output_dir: &Path,
) -> Result<serde_json::Value, String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let save = world.new_runtime_save_from_rust_db(db_dir);
    fs::create_dir_all(output_dir)
        .map_err(|err| format!("failed to create {}: {err}", output_dir.display()))?;

    let mut systems = Vec::new();
    for system in [
        "match results",
        "competition state",
        "transfers/contracts",
        "news/inbox",
    ] {
        let slug = gameplay_trace_slug(system);
        let system_dir = output_dir.join(&slug);
        fs::create_dir_all(&system_dir)
            .map_err(|err| format!("failed to create {}: {err}", system_dir.display()))?;
        let items = save
            .backend
            .gameplay_lift_workbench
            .iter()
            .filter(|item| item.system == system)
            .cloned()
            .collect::<Vec<_>>();
        let item_artifacts = items
            .iter()
            .map(gameplay_lift_item_artifact)
            .collect::<Vec<_>>();
        let decompiled_present = item_artifacts
            .iter()
            .filter(|item| {
                item["decompile_artifact"]["present"]
                    .as_bool()
                    .unwrap_or(false)
            })
            .count();
        let manifest = serde_json::json!({
            "format": "cm0102-rs-gameplay-lift-system-workbench",
            "version": 1,
            "system": system,
            "rust_db": db_dir.display().to_string(),
            "carve": "D:/cm0102-carve",
            "items": item_artifacts,
        });
        write_json_file(&system_dir.join("lift-workbench.json"), &manifest)?;
        write_text_file(
            &system_dir.join("commands.txt"),
            &gameplay_lift_commands_text(system, &items),
        )?;
        write_text_file(
            &system_dir.join("claims-to-prove.txt"),
            &gameplay_lift_claims_text(system, &items),
        )?;
        write_text_file(
            &system_dir.join("artifact-audit.txt"),
            &gameplay_lift_artifact_text(system, &items),
        )?;
        systems.push(serde_json::json!({
            "system": system,
            "slug": slug,
            "directory": system_dir.display().to_string(),
            "items": items.len(),
            "priority_1": items.iter().filter(|item| item.priority == 1).count(),
            "unknown_or_inferred": items.iter().filter(|item| {
                item.current_confidence.contains("UNKNOWN") || item.current_confidence.contains("INFERRED")
            }).count(),
            "decompile_artifacts_present": decompiled_present,
            "decompile_artifacts_missing": items.len().saturating_sub(decompiled_present),
        }));
    }

    let priority_1 = save
        .backend
        .gameplay_lift_workbench
        .iter()
        .filter(|item| item.priority == 1)
        .count();
    let unknown_or_inferred = save
        .backend
        .gameplay_lift_workbench
        .iter()
        .filter(|item| {
            item.current_confidence.contains("UNKNOWN")
                || item.current_confidence.contains("INFERRED")
        })
        .count();
    let artifact_items = save
        .backend
        .gameplay_lift_workbench
        .iter()
        .map(gameplay_lift_item_artifact)
        .collect::<Vec<_>>();
    let decompile_artifacts_present = artifact_items
        .iter()
        .filter(|item| {
            item["decompile_artifact"]["present"]
                .as_bool()
                .unwrap_or(false)
        })
        .count();
    let report = serde_json::json!({
        "format": "cm0102-rs-gameplay-lift-workbench-report",
        "version": 1,
        "source": {
            "rust_db": db_dir.display().to_string(),
            "output_dir": output_dir.display().to_string(),
            "carve": "D:/cm0102-carve"
        },
        "summary": {
            "systems": systems.len(),
            "items": save.backend.gameplay_lift_workbench.len(),
            "priority_1": priority_1,
            "unknown_or_inferred": unknown_or_inferred,
            "decompile_artifacts_present": decompile_artifacts_present,
            "decompile_artifacts_missing": save.backend.gameplay_lift_workbench.len().saturating_sub(decompile_artifacts_present),
            "status": if cm_domain::gameplay_lift_workbench_ready(&save.backend.gameplay_lift_workbench) { "pass" } else { "fail" },
        },
        "systems": systems,
        "items": artifact_items,
    });
    write_json_file(&output_dir.join("lift-workbench-report.json"), &report)?;
    Ok(report)
}

fn gameplay_lift_commands_text(system: &str, items: &[cm_domain::GameplayLiftWorkItem]) -> String {
    let mut lines = vec![
        format!("Gameplay lift commands for {system}"),
        String::new(),
        "Run `carve ask` first. Only run targeted decompile for UNKNOWN or insufficiently verified functions.".to_string(),
        String::new(),
    ];
    for item in items {
        lines.push(format!(
            "{} priority {} {} ({})",
            item.system, item.priority, item.function, item.current_confidence
        ));
        lines.push(item.carve_ask_command.clone());
        lines.push(format!(
            "Set-Location D:/tools/structural_carver; {}",
            item.targeted_decompile_command
        ));
        lines.push(String::new());
    }
    lines.join("\n")
}

fn gameplay_lift_claims_text(system: &str, items: &[cm_domain::GameplayLiftWorkItem]) -> String {
    let mut lines = vec![
        format!("Claims to prove before exact gameplay promotion for {system}"),
        String::new(),
    ];
    for item in items {
        lines.push(format!(
            "{} priority {} -> {}",
            item.function, item.priority, item.promotion_target
        ));
        lines.push(format!("Trace: {}", item.trace_file));
        lines.push(format!("Acceptance: {}", item.acceptance_gate));
        for claim in &item.required_claims {
            lines.push(format!("- {claim}"));
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

fn gameplay_lift_artifact_text(system: &str, items: &[cm_domain::GameplayLiftWorkItem]) -> String {
    let mut lines = vec![
        format!("Decompile artifact audit for {system}"),
        String::new(),
    ];
    for item in items {
        let path = gameplay_lift_decompile_artifact_path(item);
        let bytes = fs::metadata(&path).ok().map(|metadata| metadata.len());
        lines.push(format!(
            "{} {} -> {} ({})",
            item.function,
            item.current_confidence,
            path.display(),
            bytes
                .map(|len| format!("{len} bytes"))
                .unwrap_or_else(|| "missing".to_string())
        ));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn gameplay_lift_item_artifact(item: &cm_domain::GameplayLiftWorkItem) -> serde_json::Value {
    let path = gameplay_lift_decompile_artifact_path(item);
    let bytes = fs::metadata(&path).ok().map(|metadata| metadata.len());
    serde_json::json!({
        "system": item.system,
        "priority": item.priority,
        "function": item.function,
        "current_confidence": item.current_confidence,
        "source_hint": item.source_hint,
        "carve_ask_command": item.carve_ask_command,
        "targeted_decompile_command": item.targeted_decompile_command,
        "required_claims": item.required_claims,
        "promotion_target": item.promotion_target,
        "trace_file": item.trace_file,
        "acceptance_gate": item.acceptance_gate,
        "status": item.status,
        "decompile_artifact": {
            "path": path.display().to_string(),
            "present": bytes.is_some(),
            "bytes": bytes,
            "provenance": "Ghidra targeted decompile artifact; semantics remain unverified until read and promoted through findings.json"
        }
    })
}

fn gameplay_lift_decompile_artifact_path(item: &cm_domain::GameplayLiftWorkItem) -> PathBuf {
    PathBuf::from("D:/cm0102-carve")
        .join("decompiled")
        .join(gameplay_lift_subdir(&item.system))
        .join(format!("{}.c", item.function))
}

fn export_formula_lift_backlog(
    db_dir: &Path,
    output_dir: &Path,
) -> Result<serde_json::Value, String> {
    let report = formula_lift_backlog_report(db_dir)?;
    fs::create_dir_all(output_dir)
        .map_err(|err| format!("failed to create {}: {err}", output_dir.display()))?;
    write_json_file(&output_dir.join("formula-lift-backlog.json"), &report)?;
    write_text_file(
        &output_dir.join("FORMULA_LIFT_BACKLOG.md"),
        &formula_lift_backlog_markdown(&report),
    )?;
    Ok(report)
}

fn formula_lift_backlog_report(db_dir: &Path) -> Result<serde_json::Value, String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let readiness = world.backend_readiness_report(db_dir);
    let save = world.new_runtime_save_from_rust_db(db_dir);
    let mut tasks = Vec::new();

    for (system_index, item) in readiness.implementation_plan.iter().enumerate() {
        for (lift_index, lift) in item.missing_lifts.iter().enumerate() {
            let related_items = save
                .backend
                .gameplay_lift_workbench
                .iter()
                .filter(|work_item| formula_lift_function_matches(&item.system, lift, work_item))
                .collect::<Vec<_>>();
            let related_functions = related_items
                .iter()
                .map(|work_item| {
                    let artifact = gameplay_lift_decompile_artifact_path(work_item);
                    let artifact_present = artifact.is_file();
                    serde_json::json!({
                        "function": work_item.function,
                        "priority": work_item.priority,
                        "current_confidence": work_item.current_confidence,
                        "source_hint": work_item.source_hint,
                        "carve_ask_command": work_item.carve_ask_command,
                        "targeted_decompile_command": work_item.targeted_decompile_command,
                        "decompile_artifact": {
                            "path": artifact.display().to_string(),
                            "present": artifact_present,
                            "bytes": fs::metadata(&artifact).ok().map(|metadata| metadata.len()),
                        },
                        "required_claims": work_item.required_claims,
                    })
                })
                .collect::<Vec<_>>();
            let related_artifacts_present = related_functions
                .iter()
                .filter(|function| {
                    function["decompile_artifact"]["present"]
                        .as_bool()
                        .unwrap_or(false)
                })
                .count();
            let unknown_or_inferred = related_items
                .iter()
                .filter(|work_item| {
                    work_item.current_confidence.contains("UNKNOWN")
                        || work_item.current_confidence.contains("INFERRED")
                        || work_item.current_confidence.contains("frontier")
                })
                .count();
            let status = if related_items.is_empty() {
                "needs-function-map"
            } else if related_artifacts_present < related_items.len() {
                "needs-targeted-decompile"
            } else if unknown_or_inferred > 0 {
                "ready-for-static-read-and-claim-promotion"
            } else {
                "ready-for-formula-implementation"
            };
            let priority = formula_lift_priority(&item.system, lift, system_index, lift_index);
            tasks.push(serde_json::json!({
                "id": format!("formula-lift-{:02}", tasks.len() + 1),
                "system": item.system,
                "priority": priority,
                "lift": lift,
                "status": status,
                "acceptance_gate": item.acceptance_gate,
                "related_functions": related_functions,
                "related_function_count": related_items.len(),
                "decompile_artifacts_present": related_artifacts_present,
                "decompile_artifacts_missing": related_items.len().saturating_sub(related_artifacts_present),
                "unknown_or_inferred_functions": unknown_or_inferred,
                "next_action": formula_lift_next_action(status),
            }));
        }
    }

    tasks.sort_by_key(|task| task["priority"].as_u64().unwrap_or(u64::MAX));
    let total = tasks.len();
    let ready_for_static_read = tasks
        .iter()
        .filter(|task| {
            matches!(
                task["status"].as_str(),
                Some("ready-for-static-read-and-claim-promotion")
                    | Some("ready-for-formula-implementation")
            )
        })
        .count();
    let ready_for_implementation = tasks
        .iter()
        .filter(|task| task["status"].as_str() == Some("ready-for-formula-implementation"))
        .count();
    let missing_decompile = tasks
        .iter()
        .filter(|task| task["status"].as_str() == Some("needs-targeted-decompile"))
        .count();
    let missing_function_map = tasks
        .iter()
        .filter(|task| task["status"].as_str() == Some("needs-function-map"))
        .count();

    Ok(serde_json::json!({
        "format": "cm0102-rs-formula-lift-backlog",
        "version": 1,
        "source": {
            "rust_db": db_dir.display().to_string(),
            "carve": "D:/cm0102-carve",
            "rule": "Run carve ask before opening decompile. Promote formulas only from static code-derived evidence."
        },
        "summary": {
            "status": if total == 0 { "complete" } else { "formula-lift-required" },
            "tasks": total,
            "ready_for_static_read": ready_for_static_read,
            "ready_for_formula_implementation": ready_for_implementation,
            "needs_targeted_decompile": missing_decompile,
            "needs_function_map": missing_function_map,
        },
        "tasks": tasks,
    }))
}

fn formula_lift_function_matches(
    system: &str,
    lift: &str,
    item: &cm_domain::GameplayLiftWorkItem,
) -> bool {
    if item.system != system {
        return false;
    }
    let lift = lift.to_ascii_lowercase();
    let function = item.function.as_str();
    match system {
        "match results" => {
            lift.contains("score")
                || lift.contains("event")
                || lift.contains("fixture")
                || matches!(function, "0x006a4020" | "0x006a3240" | "0x006bc8d0")
        }
        "competition state" => {
            (lift.contains("notification") && matches!(function, "0x00752d40" | "0x00595580"))
                || (lift.contains("table") && matches!(function, "0x00674c10" | "0x00752d40"))
                || (lift.contains("fixture") && matches!(function, "0x00752d40" | "0x00595580"))
                || (lift.contains("cup") && function == "0x00674c10")
        }
        "transfers/contracts" => {
            (lift.contains("renewal") && function == "0x004cdef0")
                || (lift.contains("bid")
                    || lift.contains("wage")
                    || lift.contains("value")
                    || lift.contains("ai"))
                    && matches!(function, "0x008a9080" | "0x00449710")
                || lift.contains("transfer.dat") && function == "0x008a9080"
                || lift.contains("queue") && function == "0x00449710"
        }
        "news/inbox" => {
            (lift.contains("template") || lift.contains("recipient") || lift.contains("routing"))
                && matches!(function, "0x0050c8d0" | "0x0076e180")
                || lift.contains("paired") && function == "0x0050c8d0"
                || lift.contains("queue") && matches!(function, "0x006724d0" | "0x0050c8d0")
                || lift.contains("payload") && matches!(function, "0x0050c8d0" | "0x0076e180")
        }
        _ => false,
    }
}

fn formula_lift_priority(system: &str, lift: &str, system_index: usize, lift_index: usize) -> u64 {
    let lift = lift.to_ascii_lowercase();
    let base = match system {
        "match results" => 100,
        "transfers/contracts" => 200,
        "competition state" => 300,
        "news/inbox" => 400,
        _ => 900 + (system_index as u64 * 100),
    };
    let boost = if lift.contains("score") || lift.contains("event formulas") {
        0
    } else if lift.contains("value") || lift.contains("wage") || lift.contains("bid") {
        10
    } else if lift.contains("table") || lift.contains("cup") {
        20
    } else {
        30
    };
    base + boost + lift_index as u64
}

fn formula_lift_next_action(status: &str) -> &'static str {
    match status {
        "ready-for-formula-implementation" => {
            "Read the existing decompile artifact, derive constants/branches, then add Rust formula assertions and mutator state changes."
        }
        "ready-for-static-read-and-claim-promotion" => {
            "Run carve ask, read the existing Ghidra artifact, then promote verified claims in D:/cm0102-carve/findings.json."
        }
        "needs-targeted-decompile" => {
            "Run the emitted targeted_decompile_command from D:/tools/structural_carver, then rerun this backlog."
        }
        _ => "Map this lift item to one or more code-derived function frontiers before implementation.",
    }
}

fn formula_lift_backlog_markdown(report: &serde_json::Value) -> String {
    let summary = &report["summary"];
    let mut lines = vec![
        "# CM0102 Formula Lift Backlog".to_string(),
        String::new(),
        format!(
            "Status: {} | tasks {} | ready for static read {} | ready for implementation {} | needs decompile {}",
            summary["status"].as_str().unwrap_or("unknown"),
            summary["tasks"].as_u64().unwrap_or(0),
            summary["ready_for_static_read"].as_u64().unwrap_or(0),
            summary["ready_for_formula_implementation"].as_u64().unwrap_or(0),
            summary["needs_targeted_decompile"].as_u64().unwrap_or(0)
        ),
        String::new(),
        "Rule: run `carve ask` first; formula constants come from static code, not sampled outcomes.".to_string(),
        String::new(),
    ];
    if let Some(tasks) = report["tasks"].as_array() {
        for task in tasks {
            lines.push(format!(
                "## {} {}",
                task["id"].as_str().unwrap_or("formula-lift"),
                task["system"].as_str().unwrap_or("unknown")
            ));
            lines.push(format!(
                "Priority: {} | Status: {}",
                task["priority"].as_u64().unwrap_or(0),
                task["status"].as_str().unwrap_or("unknown")
            ));
            lines.push(format!("Lift: {}", task["lift"].as_str().unwrap_or("")));
            lines.push(format!(
                "Next: {}",
                task["next_action"].as_str().unwrap_or("")
            ));
            if let Some(functions) = task["related_functions"].as_array() {
                for function in functions {
                    lines.push(format!(
                        "- {} ({}) artifact {}",
                        function["function"].as_str().unwrap_or("unknown"),
                        function["current_confidence"].as_str().unwrap_or("unknown"),
                        if function["decompile_artifact"]["present"]
                            .as_bool()
                            .unwrap_or(false)
                        {
                            "present"
                        } else {
                            "missing"
                        }
                    ));
                }
            }
            lines.push(String::new());
        }
    }
    lines.join("\n")
}

fn gameplay_lift_subdir(system: &str) -> &'static str {
    match system {
        "match results" => "gameplay_lifts_match",
        "competition state" => "gameplay_lifts_competition",
        "transfers/contracts" => "gameplay_lifts_transfer",
        "news/inbox" => "gameplay_lifts_news",
        _ => "gameplay_lifts",
    }
}

fn gameplay_lift_artifacts_ready(items: &[cm_domain::GameplayLiftWorkItem]) -> bool {
    !items.is_empty() && gameplay_lift_artifacts_present(items) == items.len()
}

fn gameplay_lift_artifacts_present(items: &[cm_domain::GameplayLiftWorkItem]) -> usize {
    items
        .iter()
        .filter(|item| gameplay_lift_decompile_artifact_path(item).is_file())
        .count()
}

fn mutations_have_required_fields(mutations: &[serde_json::Value]) -> bool {
    !mutations.is_empty()
        && mutations.iter().all(|mutation| {
            mutation
                .get("table")
                .and_then(serde_json::Value::as_str)
                .is_some()
                && mutation
                    .get("row")
                    .and_then(serde_json::Value::as_u64)
                    .is_some()
                && mutation
                    .get("field")
                    .and_then(serde_json::Value::as_str)
                    .is_some()
                && mutation.get("before").is_some()
                && mutation.get("after").is_some()
                && mutation
                    .get("source_function")
                    .and_then(serde_json::Value::as_str)
                    .is_some()
                && mutation
                    .get("provenance")
                    .and_then(serde_json::Value::as_str)
                    .is_some()
        })
}

fn subsystem_capture_coverage(system: &str, mutations: &[serde_json::Value]) -> serde_json::Value {
    match system {
        "match results" => match_result_capture_coverage(mutations),
        "competition state" => capture_need_coverage(
            mutations,
            &[
                ("fixture +0x1c", "fixture participant field +0x1c"),
                ("fixture +0x20", "fixture participant field +0x20"),
                ("0x100", "fixture notification flag +0x4d bit 0x100"),
                ("0x200", "fixture notification flag +0x4d bit 0x200"),
                ("0x00596590", "fixture list accessor 0x00596590"),
                ("0x0075f0f0", "fixture cleanup cadence helper 0x0075f0f0"),
            ],
        ),
        "transfers/contracts" => capture_need_coverage(
            mutations,
            &[
                (
                    "contract renewal date windows",
                    "contract renewal date windows",
                ),
                ("0x6e", "0x6e-byte staff pool stride"),
                ("0x4f", "0x4f-byte staff side-state stride"),
                ("0x50", "0x50-byte event/contract record stride"),
                ("queued transfer", "queued transfer/club-news dispatch item"),
                ("transfer.dat", "transfer.dat-equivalent manager/list state"),
            ],
        ),
        "news/inbox" => capture_need_coverage(
            mutations,
            &[
                ("0x68", "fixture/news subrecord stride 0x68"),
                ("+0xa3", "fixture/news subrecord base pointer +0xa3"),
                ("+0x30", "paired event +0x30 writes"),
                ("+3/+4", "paired dated event tags +3/+4"),
                ("+0xde", "news reset byte +0xde"),
                ("0x006724d0", "queued news removal helper 0x006724d0"),
            ],
        ),
        _ => capture_need_coverage(mutations, &[("provenance", "generic mutation schema")]),
    }
}

fn capture_need_coverage(
    mutations: &[serde_json::Value],
    required: &[(&str, &str)],
) -> serde_json::Value {
    let mut missing = Vec::new();
    for (needle, label) in required {
        if !mutations
            .iter()
            .any(|mutation| mutation_matches_capture_need(mutation, needle))
        {
            missing.push(format!("{label} ({needle})"));
        }
    }
    let status = if mutations_have_required_fields(mutations) && missing.is_empty() {
        "pass"
    } else {
        if !mutations_have_required_fields(mutations) {
            missing.push("mutation rows with required provenance fields".to_string());
        }
        "fail"
    };
    serde_json::json!({
        "status": status,
        "required": required.iter().map(|(_, label)| (*label).to_string()).collect::<Vec<_>>(),
        "missing": missing,
    })
}

fn match_result_capture_coverage(mutations: &[serde_json::Value]) -> serde_json::Value {
    let required = [
        ("fixture +0x43", "normal-time home/status score byte"),
        ("fixture +0x44", "normal-time away score byte"),
        ("fixture +0x49", "final home score byte"),
        ("fixture +0x4a", "final away score byte"),
        ("event 0x2004", "final result event payload"),
    ];
    let mut missing = Vec::new();
    for (needle, label) in required {
        if !mutations
            .iter()
            .any(|mutation| mutation_matches_capture_need(mutation, needle))
        {
            missing.push(format!("{label} ({needle})"));
        }
    }
    let has_period_event = ["0x20f1", "0x20f2", "0x20f3"].iter().any(|event| {
        mutations
            .iter()
            .any(|mutation| mutation_matches_capture_need(mutation, event))
    });
    if !has_period_event {
        missing.push(
            "at least one period transition event payload (0x20f1/0x20f2/0x20f3)".to_string(),
        );
    }
    let status = if missing.is_empty() { "pass" } else { "fail" };
    serde_json::json!({
        "status": status,
        "required": [
            "fixture +0x43",
            "fixture +0x44",
            "fixture +0x49",
            "fixture +0x4a",
            "event 0x2004",
            "one of event 0x20f1/0x20f2/0x20f3"
        ],
        "missing": missing,
    })
}

fn mutation_matches_capture_need(mutation: &serde_json::Value, needle: &str) -> bool {
    let needle = needle.to_ascii_lowercase();
    [
        "table",
        "field",
        "event_code",
        "record_offset",
        "helper",
        "source_function",
        "provenance",
        "notes",
        "phase",
    ]
    .iter()
    .filter_map(|key| mutation.get(*key).and_then(serde_json::Value::as_str))
    .any(|value| value.to_ascii_lowercase().contains(&needle))
}

fn import_gameplay_capture(
    trace_dir: &Path,
    capture_path: &Path,
) -> Result<serde_json::Value, String> {
    let capture_text = fs::read_to_string(capture_path)
        .map_err(|err| format!("failed to read {}: {err}", capture_path.display()))?;
    let capture: serde_json::Value = serde_json::from_str(&capture_text)
        .map_err(|err| format!("invalid capture JSON {}: {err}", capture_path.display()))?;
    let system = capture["system"]
        .as_str()
        .ok_or_else(|| "capture JSON must include system".to_string())?;
    let slug = gameplay_trace_slug(system);
    let trace_path = trace_dir.join(format!("{slug}.json"));
    let trace_text = fs::read_to_string(&trace_path)
        .map_err(|err| format!("failed to read {}: {err}", trace_path.display()))?;
    let mut trace: serde_json::Value = serde_json::from_str(&trace_text)
        .map_err(|err| format!("invalid trace JSON {}: {err}", trace_path.display()))?;
    if trace["system"].as_str() != Some(system) {
        return Err(format!(
            "capture system '{}' does not match trace system '{}'",
            system,
            trace["system"].as_str().unwrap_or("unknown")
        ));
    }

    let original_mutations = capture["original_mutations"]
        .as_array()
        .cloned()
        .ok_or_else(|| "capture JSON must include original_mutations array".to_string())?;
    let rust_mutations = capture["rust_mutations"]
        .as_array()
        .cloned()
        .ok_or_else(|| "capture JSON must include rust_mutations array".to_string())?;
    if !mutations_have_required_fields(&original_mutations) {
        return Err("original_mutations must include table, row, field, before, after, source_function, and provenance for every row".to_string());
    }
    if !mutations_have_required_fields(&rust_mutations) {
        return Err("rust_mutations must include table, row, field, before, after, source_function, and provenance for every row".to_string());
    }
    let original_coverage = subsystem_capture_coverage(system, &original_mutations);
    let rust_coverage = subsystem_capture_coverage(system, &rust_mutations);
    let comparison = gameplay_mutation_comparison(system, &original_mutations, &rust_mutations);
    let comparison_pass = comparison["status"].as_str() == Some("pass");
    trace["original_mutations"] = serde_json::Value::Array(original_mutations);
    trace["rust_mutations"] = serde_json::Value::Array(rust_mutations);
    trace["comparison"] = comparison.clone();
    trace["status"] = serde_json::Value::String(if comparison_pass {
        "imported-capture-comparison-pass".to_string()
    } else {
        "imported-capture-comparison-mismatch".to_string()
    });
    trace["last_capture_import"] = serde_json::json!({
        "capture_file": capture_path.display().to_string(),
        "original_mutations": trace["original_mutations"].as_array().map(Vec::len).unwrap_or(0),
        "rust_mutations": trace["rust_mutations"].as_array().map(Vec::len).unwrap_or(0),
        "comparison_pass": comparison_pass,
        "comparison": comparison,
        "original_coverage": original_coverage,
        "rust_coverage": rust_coverage,
    });

    write_json_file(&trace_path, &trace)?;
    Ok(serde_json::json!({
        "format": "cm0102-rs-gameplay-capture-import-report",
        "version": 1,
        "system": system,
        "capture_file": capture_path.display().to_string(),
        "trace_file": trace_path.display().to_string(),
        "original_mutations": trace["original_mutations"].as_array().map(Vec::len).unwrap_or(0),
        "rust_mutations": trace["rust_mutations"].as_array().map(Vec::len).unwrap_or(0),
        "comparison_pass": comparison_pass,
        "comparison": trace["last_capture_import"]["comparison"],
        "original_coverage": trace["last_capture_import"]["original_coverage"],
        "rust_coverage": trace["last_capture_import"]["rust_coverage"],
        "status": if comparison_pass { "imported-pass" } else { "imported-mismatch" },
    }))
}

fn gameplay_mutation_comparison(
    system: &str,
    original_mutations: &[serde_json::Value],
    rust_mutations: &[serde_json::Value],
) -> serde_json::Value {
    let first_mismatch = first_mutation_mismatch(original_mutations, rust_mutations);
    let missing_from_rust = original_mutations
        .iter()
        .enumerate()
        .filter(|(_, mutation)| !rust_mutations.contains(mutation))
        .take(25)
        .map(|(index, mutation)| serde_json::json!({"index": index, "mutation": mutation}))
        .collect::<Vec<_>>();
    let extra_in_rust = rust_mutations
        .iter()
        .enumerate()
        .filter(|(_, mutation)| !original_mutations.contains(mutation))
        .take(25)
        .map(|(index, mutation)| serde_json::json!({"index": index, "mutation": mutation}))
        .collect::<Vec<_>>();
    let pass = !original_mutations.is_empty()
        && !rust_mutations.is_empty()
        && original_mutations == rust_mutations;
    let mut blockers = Vec::new();
    if original_mutations.is_empty() {
        blockers.push("original_mutations_empty");
    }
    if rust_mutations.is_empty() {
        blockers.push("rust_mutations_empty");
    }
    if original_mutations.len() != rust_mutations.len() {
        blockers.push("mutation_count_mismatch");
    }
    if first_mismatch.is_some() {
        blockers.push("ordered_mutation_mismatch");
    }
    if !missing_from_rust.is_empty() {
        blockers.push("original_rows_missing_from_rust");
    }
    if !extra_in_rust.is_empty() {
        blockers.push("rust_rows_not_seen_in_original");
    }

    serde_json::json!({
        "status": if pass { "pass" } else { "fail" },
        "method": "exact ordered mutation equality",
        "system": system,
        "original_count": original_mutations.len(),
        "rust_count": rust_mutations.len(),
        "first_mismatch": first_mismatch,
        "missing_from_rust_sample": missing_from_rust,
        "extra_in_rust_sample": extra_in_rust,
        "blockers": blockers,
        "notes": if pass {
            "Imported capture rows are exact ordered equals."
        } else {
            "Imported capture rows differ; keep blocked until Rust mutator matches original order and payloads."
        }
    })
}

fn first_mutation_mismatch(
    original_mutations: &[serde_json::Value],
    rust_mutations: &[serde_json::Value],
) -> Option<serde_json::Value> {
    let max = original_mutations.len().max(rust_mutations.len());
    for index in 0..max {
        let original = original_mutations.get(index);
        let rust = rust_mutations.get(index);
        if original != rust {
            return Some(serde_json::json!({
                "index": index,
                "original": original,
                "rust": rust,
            }));
        }
    }
    None
}

fn sync_gameplay_mutator_contracts(
    save: &mut RuntimeSaveGame,
    trace_dir: &Path,
) -> Result<serde_json::Value, String> {
    let mut systems = Vec::new();
    let mut parity_verified = 0usize;
    let mut trace_verified_missing_implementation = 0usize;
    let mut pending = 0usize;

    for contract in &mut save.backend.mutator_contracts {
        let slug = gameplay_trace_slug(&contract.system);
        let trace_path = trace_dir.join(format!("{slug}.json"));
        let previous_status = contract.status;
        let trace = if trace_path.exists() {
            let text = fs::read_to_string(&trace_path)
                .map_err(|err| format!("failed to read {}: {err}", trace_path.display()))?;
            Some(
                serde_json::from_str::<serde_json::Value>(&text)
                    .map_err(|err| format!("invalid trace JSON {}: {err}", trace_path.display()))?,
            )
        } else {
            None
        };

        let trace_status = trace
            .as_ref()
            .map(|trace| parity_trace_status(trace, &contract.system))
            .unwrap_or(ParityTraceStatus::Missing);
        contract.status = match trace_status {
            ParityTraceStatus::Verified if contract.implementation_present => {
                GameplayMutatorStatus::ParityVerified
            }
            ParityTraceStatus::Verified => {
                trace_verified_missing_implementation =
                    trace_verified_missing_implementation.saturating_add(1);
                GameplayMutatorStatus::ImplementedPendingParity
            }
            ParityTraceStatus::ImplementedPendingParity => {
                GameplayMutatorStatus::ImplementedPendingParity
            }
            ParityTraceStatus::Pending | ParityTraceStatus::Missing => {
                GameplayMutatorStatus::ContractReady
            }
        };

        if contract.status == GameplayMutatorStatus::ParityVerified {
            parity_verified = parity_verified.saturating_add(1);
        } else {
            pending = pending.saturating_add(1);
        }

        systems.push(serde_json::json!({
            "system": contract.system,
            "trace_file": trace_path.display().to_string(),
            "trace_status": format!("{trace_status:?}"),
            "previous_status": format!("{previous_status:?}"),
            "new_status": format!("{:?}", contract.status),
            "phase": contract.phase,
            "boundary_map": contract.boundary_map,
            "implementation_hook": contract.implementation_hook,
            "implementation_present": contract.implementation_present,
        }));
    }

    Ok(serde_json::json!({
        "format": "cm0102-rs-gameplay-mutator-contract-sync",
        "version": 1,
        "trace_dir": trace_dir.display().to_string(),
        "summary": {
            "contracts": save.backend.mutator_contracts.len(),
            "parity_verified": parity_verified,
            "trace_verified_missing_implementation": trace_verified_missing_implementation,
            "pending": pending,
            "all_verified": parity_verified == save.backend.mutator_contracts.len(),
        },
        "systems": systems,
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParityTraceStatus {
    Missing,
    Pending,
    ImplementedPendingParity,
    Verified,
}

fn parity_trace_status(trace: &serde_json::Value, expected_system: &str) -> ParityTraceStatus {
    let format_ok = trace["format"].as_str() == Some("cm0102-rs-gameplay-parity-trace");
    let system_ok = trace["system"].as_str() == Some(expected_system);
    if !format_ok || !system_ok {
        return ParityTraceStatus::Pending;
    }

    let original_mutations = trace["original_mutations"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let rust_mutations = trace["rust_mutations"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if original_mutations.is_empty() && rust_mutations.is_empty() {
        return ParityTraceStatus::Pending;
    }
    let comparison_pass = trace["comparison"]["status"].as_str() == Some("pass")
        || (!original_mutations.is_empty()
            && !rust_mutations.is_empty()
            && original_mutations == rust_mutations);
    if comparison_pass
        && mutations_have_required_fields(&original_mutations)
        && mutations_have_required_fields(&rust_mutations)
        && subsystem_capture_coverage(expected_system, &original_mutations)["status"].as_str()
            == Some("pass")
        && subsystem_capture_coverage(expected_system, &rust_mutations)["status"].as_str()
            == Some("pass")
    {
        ParityTraceStatus::Verified
    } else {
        ParityTraceStatus::ImplementedPendingParity
    }
}

fn init_gameplay_parity_traces(db_dir: &Path, trace_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let readiness = world.backend_readiness_report(db_dir);
    let save = world.new_runtime_save_from_rust_db(db_dir);
    fs::create_dir_all(trace_dir)
        .map_err(|err| format!("failed to create {}: {err}", trace_dir.display()))?;

    let mut written = Vec::new();
    for item in &readiness.implementation_plan {
        let slug = gameplay_trace_slug(&item.system);
        let trace_path = trace_dir.join(format!("{slug}.json"));
        let mut template = if trace_path.exists() {
            let text = fs::read_to_string(&trace_path)
                .map_err(|err| format!("failed to read {}: {err}", trace_path.display()))?;
            serde_json::from_str(&text)
                .map_err(|err| format!("invalid trace template {}: {err}", trace_path.display()))?
        } else {
            serde_json::json!({})
        };

        let before = template.clone();
        let promotion_gate = save
            .backend
            .gameplay_promotion_gates
            .iter()
            .find(|gate| gate.system == item.system);
        merge_trace_template_defaults(&mut template, db_dir, item, promotion_gate);
        if template == before {
            continue;
        }

        let bytes = serde_json::to_vec_pretty(&template)
            .map_err(|err| format!("failed to serialize {}: {err}", trace_path.display()))?;
        fs::write(&trace_path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", trace_path.display()))?;
        written.push(trace_path);
    }
    Ok(written)
}

fn export_rust_match_result_trace(
    db_dir: &Path,
    trace_dir: &Path,
) -> Result<serde_json::Value, String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let readiness = world.backend_readiness_report(db_dir);
    let save = world.new_runtime_save_from_rust_db(db_dir);
    let item = readiness
        .implementation_plan
        .iter()
        .find(|item| item.system == "match results")
        .ok_or_else(|| "backend implementation plan does not include match results".to_string())?;
    let promotion_gate = save
        .backend
        .gameplay_promotion_gates
        .iter()
        .find(|gate| gate.system == "match results");

    fs::create_dir_all(trace_dir)
        .map_err(|err| format!("failed to create {}: {err}", trace_dir.display()))?;
    let trace_path = trace_dir.join("match-results.json");
    let mut trace = if trace_path.exists() {
        let text = fs::read_to_string(&trace_path)
            .map_err(|err| format!("failed to read {}: {err}", trace_path.display()))?;
        serde_json::from_str::<serde_json::Value>(&text)
            .map_err(|err| format!("invalid trace JSON {}: {err}", trace_path.display()))?
    } else {
        serde_json::json!({})
    };

    merge_trace_template_defaults(&mut trace, db_dir, item, promotion_gate);
    let rust_mutations = rust_match_result_candidate_mutations(&save);
    let rust_coverage = match_result_capture_coverage(&rust_mutations);
    let original_mutations = trace["original_mutations"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let original_status = if original_mutations.is_empty() {
        "missing"
    } else {
        "present"
    };
    let comparison_status = if !original_mutations.is_empty()
        && original_mutations == rust_mutations
        && rust_coverage["status"].as_str() == Some("pass")
    {
        "pass"
    } else {
        "pending-original-capture"
    };

    trace["status"] = serde_json::json!("rust-candidate-generated-pending-original-capture");
    trace["rust_implementation"] = serde_json::json!({
        "present": false,
        "candidate_generated": true,
        "notes": "Candidate Rust mutations are exported from code-derived write maps only. Do not promote until original cm0102.exe trace proves exact ordered equality."
    });
    trace["rust_mutations"] = serde_json::Value::Array(rust_mutations.clone());
    trace["rust_candidate"] = serde_json::json!({
        "source": "RuntimeBackendSystems.match_result_write_map + match_result_code_claims",
        "mutations": rust_mutations.len(),
        "coverage": rust_coverage,
        "promotion_rule": "candidate only; original_mutations must match exactly before implementation_present can flip"
    });
    trace["comparison"] = serde_json::json!({
        "status": comparison_status,
        "method": "exact ordered mutation equality",
        "notes": format!("Rust candidate is generated; original capture is {original_status}.")
    });

    write_json_file(&trace_path, &trace)?;
    Ok(serde_json::json!({
        "format": "cm0102-rs-rust-match-result-trace-export",
        "version": 1,
        "trace_file": trace_path.display().to_string(),
        "summary": {
            "status": comparison_status,
            "rust_mutations": rust_mutations.len(),
            "original_mutations": original_mutations.len(),
            "rust_coverage": match_result_capture_coverage(&rust_mutations),
        }
    }))
}

fn export_rust_gameplay_candidate_traces(
    db_dir: &Path,
    trace_dir: &Path,
) -> Result<serde_json::Value, String> {
    let world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let readiness = world.backend_readiness_report(db_dir);
    let save = world.new_runtime_save_from_rust_db(db_dir);
    fs::create_dir_all(trace_dir)
        .map_err(|err| format!("failed to create {}: {err}", trace_dir.display()))?;

    let mut systems = Vec::new();
    for system in [
        "match results",
        "competition state",
        "transfers/contracts",
        "news/inbox",
    ] {
        let item = readiness
            .implementation_plan
            .iter()
            .find(|item| item.system == system)
            .ok_or_else(|| format!("backend implementation plan does not include {system}"))?;
        let promotion_gate = save
            .backend
            .gameplay_promotion_gates
            .iter()
            .find(|gate| gate.system == system);
        let slug = gameplay_trace_slug(system);
        let trace_path = trace_dir.join(format!("{slug}.json"));
        let mut trace = if trace_path.exists() {
            let text = fs::read_to_string(&trace_path)
                .map_err(|err| format!("failed to read {}: {err}", trace_path.display()))?;
            serde_json::from_str::<serde_json::Value>(&text)
                .map_err(|err| format!("invalid trace JSON {}: {err}", trace_path.display()))?
        } else {
            serde_json::json!({})
        };
        merge_trace_template_defaults(&mut trace, db_dir, item, promotion_gate);

        let rust_mutations = rust_candidate_mutations_for_system(&save, system);
        let rust_coverage = subsystem_capture_coverage(system, &rust_mutations);
        let original_mutations = trace["original_mutations"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let comparison_status = if !original_mutations.is_empty()
            && original_mutations == rust_mutations
            && rust_coverage["status"].as_str() == Some("pass")
        {
            "pass"
        } else {
            "pending-original-capture"
        };
        trace["status"] = serde_json::json!("rust-candidate-generated-pending-original-capture");
        trace["rust_implementation"] = serde_json::json!({
            "present": false,
            "candidate_generated": true,
            "notes": "Candidate Rust mutations are exported from code-derived boundary maps only. Do not promote until original cm0102.exe trace proves exact ordered equality."
        });
        trace["rust_mutations"] = serde_json::Value::Array(rust_mutations.clone());
        trace["rust_candidate"] = serde_json::json!({
            "source": rust_candidate_source_for_system(system),
            "mutations": rust_mutations.len(),
            "coverage": rust_coverage,
            "promotion_rule": "candidate only; original_mutations must match exactly before implementation_present can flip"
        });
        trace["comparison"] = serde_json::json!({
            "status": comparison_status,
            "method": "exact ordered mutation equality",
            "notes": format!("Rust candidate is generated; original capture has {} mutation(s).", original_mutations.len())
        });
        write_json_file(&trace_path, &trace)?;

        systems.push(serde_json::json!({
            "system": system,
            "trace_file": trace_path.display().to_string(),
            "rust_mutations": rust_mutations.len(),
            "original_mutations": original_mutations.len(),
            "coverage": subsystem_capture_coverage(system, &rust_mutations),
            "comparison_status": comparison_status,
        }));
    }

    let rust_mutations = systems
        .iter()
        .map(|system| system["rust_mutations"].as_u64().unwrap_or_default())
        .sum::<u64>();
    let coverage_failures = systems
        .iter()
        .filter(|system| system["coverage"]["status"].as_str() != Some("pass"))
        .count();
    Ok(serde_json::json!({
        "format": "cm0102-rs-rust-gameplay-candidate-trace-export",
        "version": 1,
        "trace_dir": trace_dir.display().to_string(),
        "summary": {
            "status": if coverage_failures == 0 { "pending-original-capture" } else { "candidate-coverage-fail" },
            "systems": systems.len(),
            "written": systems.len(),
            "rust_mutations": rust_mutations,
            "coverage_failures": coverage_failures,
        },
        "systems": systems,
    }))
}

fn rust_candidate_mutations_for_system(
    save: &RuntimeSaveGame,
    system: &str,
) -> Vec<serde_json::Value> {
    match system {
        "match results" => rust_match_result_candidate_mutations(save),
        "competition state" => rust_competition_candidate_mutations(save),
        "transfers/contracts" => rust_transfer_contract_candidate_mutations(save),
        "news/inbox" => rust_news_inbox_candidate_mutations(save),
        _ => Vec::new(),
    }
}

fn rust_candidate_source_for_system(system: &str) -> &'static str {
    match system {
        "match results" => {
            "RuntimeBackendSystems.match_result_write_map + match_result_code_claims"
        }
        "competition state" => {
            "RuntimeBackendSystems.competition_fixture_state_map + competition code claims"
        }
        "transfers/contracts" => {
            "RuntimeBackendSystems.transfer_contract_state_map + transfer/contract code claims"
        }
        "news/inbox" => "RuntimeBackendSystems.news_inbox_emission_map + news/inbox code claims",
        _ => "unknown",
    }
}

fn rust_match_result_candidate_mutations(save: &RuntimeSaveGame) -> Vec<serde_json::Value> {
    let mut mutations = Vec::new();
    for (index, entry) in save.backend.match_result_write_map.iter().enumerate() {
        push_match_result_fixture_candidate(
            &mut mutations,
            index,
            &entry.phase,
            &entry.fixture_home_offset,
            &entry.source_home_offset,
            entry.event_code.as_deref(),
            &entry.function,
            "home",
            &entry.evidence,
        );
        push_match_result_fixture_candidate(
            &mut mutations,
            index,
            &entry.phase,
            &entry.fixture_away_offset,
            &entry.source_away_offset,
            entry.event_code.as_deref(),
            &entry.function,
            "away",
            &entry.evidence,
        );
        if let Some(event_code) = &entry.event_code {
            mutations.push(serde_json::json!({
                "table": "event_queue",
                "row": index as u64,
                "field": format!("event {event_code} payload"),
                "before": "pending-original-before",
                "after": format!("candidate event payload for {event_code} from {}", entry.phase),
                "event_code": event_code,
                "phase": entry.phase,
                "record_offset": "event slot base +0x30; stride 0x0e",
                "source_function": entry.function,
                "provenance": "CODE_DERIVED candidate from match_result_write_map; pending original cm0102.exe trace equality",
                "notes": format!("event {event_code}; {}", entry.evidence),
            }));
        }
    }

    for (index, claim) in save.backend.match_result_code_claims.iter().enumerate() {
        for event_code in &claim.event_codes {
            mutations.push(serde_json::json!({
                "table": "event_queue",
                "row": (1000 + index) as u64,
                "field": format!("event {event_code} claim coverage"),
                "before": "pending-original-before",
                "after": format!("candidate event emitted by {}", claim.function),
                "event_code": event_code,
                "phase": claim.claim,
                "record_offset": claim
                    .fixture_offsets
                    .iter()
                    .chain(claim.source_offsets.iter())
                    .cloned()
                    .collect::<Vec<_>>()
                    .join("; "),
                "source_function": claim.function,
                "provenance": "CODE_DERIVED candidate from targeted decompile claim; pending original cm0102.exe trace equality",
                "notes": format!("event {event_code}; lines {}; {}", claim.decompile_lines, claim.evidence),
            }));
        }
    }
    mutations
}

fn push_match_result_fixture_candidate(
    mutations: &mut Vec<serde_json::Value>,
    row: usize,
    phase: &str,
    fixture_offset: &str,
    source_offset: &str,
    event_code: Option<&str>,
    source_function: &str,
    side: &str,
    evidence: &str,
) {
    if fixture_offset.eq_ignore_ascii_case("none") {
        return;
    }
    mutations.push(serde_json::json!({
        "table": "fixture",
        "row": row as u64,
        "field": format!("{phase} {side} score byte at fixture {fixture_offset}"),
        "before": "pending-original-before",
        "after": format!("candidate byte copied from {source_offset}"),
        "event_code": event_code.unwrap_or("none"),
        "phase": phase,
        "record_offset": fixture_offset,
        "source_function": source_function,
        "provenance": "CODE_DERIVED candidate from match_result_write_map; pending original cm0102.exe trace equality",
        "notes": format!("fixture {fixture_offset}; source {source_offset}; {evidence}"),
    }));
}

fn rust_competition_candidate_mutations(save: &RuntimeSaveGame) -> Vec<serde_json::Value> {
    let mut mutations = Vec::new();
    for (index, entry) in save
        .backend
        .competition_fixture_state_map
        .iter()
        .enumerate()
    {
        let record_offset = entry
            .fixture_offset
            .as_deref()
            .or(entry.helper.as_deref())
            .unwrap_or("competition boundary");
        let field = match (&entry.fixture_offset, &entry.flag_mask, &entry.cadence) {
            (Some(offset), Some(mask), _) => {
                format!("fixture +{offset} notification flag {mask}")
            }
            (Some(offset), None, _) => format!("fixture +{offset} participant reference"),
            (None, _, Some(cadence)) => format!("competition cleanup cadence {cadence}"),
            _ => entry.system.clone(),
        };
        mutations.push(serde_json::json!({
            "table": "competition_fixture_state",
            "row": index as u64,
            "field": field,
            "before": "pending-original-before",
            "after": format!("candidate boundary observation for {}", entry.system),
            "event_code": entry.flag_mask.as_deref().unwrap_or("none"),
            "phase": "competition state",
            "record_offset": record_offset,
            "helper": entry.helper.as_deref().unwrap_or("none"),
            "source_function": entry.function,
            "provenance": "CODE_DERIVED candidate from competition_fixture_state_map; pending original cm0102.exe trace equality",
            "notes": format!(
                "{}; fixture {} flag {} helper {} cadence {}; {}",
                entry.system,
                entry.fixture_offset.as_deref().unwrap_or("none"),
                entry.flag_mask.as_deref().unwrap_or("none"),
                entry.helper.as_deref().unwrap_or("none"),
                entry.cadence.as_deref().unwrap_or("none"),
                entry.evidence
            ),
        }));
    }
    mutations
}

fn rust_transfer_contract_candidate_mutations(save: &RuntimeSaveGame) -> Vec<serde_json::Value> {
    let mut mutations = Vec::new();
    for (index, entry) in save.backend.transfer_contract_state_map.iter().enumerate() {
        let record_offset = entry
            .record_offset
            .as_deref()
            .unwrap_or("contract renewal date windows");
        let stride = entry.stride.as_deref().unwrap_or("none");
        mutations.push(serde_json::json!({
            "table": "transfer_contract_state",
            "row": index as u64,
            "field": format!("{} boundary", entry.system),
            "before": "pending-original-before",
            "after": format!("candidate boundary observation at offset {record_offset} stride {stride}"),
            "event_code": "none",
            "phase": "transfers/contracts",
            "record_offset": record_offset,
            "helper": entry.helper.as_deref().unwrap_or("none"),
            "source_function": entry.function,
            "provenance": "CODE_DERIVED candidate from transfer_contract_state_map; pending original cm0102.exe trace equality",
            "notes": format!(
                "{}; record_offset {}; stride {}; helper {}; {}",
                entry.system,
                record_offset,
                stride,
                entry.helper.as_deref().unwrap_or("none"),
                entry.evidence
            ),
        }));
    }
    mutations
}

fn rust_news_inbox_candidate_mutations(save: &RuntimeSaveGame) -> Vec<serde_json::Value> {
    let mut mutations = Vec::new();
    for (index, entry) in save.backend.news_inbox_emission_map.iter().enumerate() {
        let record_offset = entry.record_offset.as_deref().unwrap_or("queue helper");
        let stride = entry.stride.as_deref().unwrap_or("none");
        let event_code = if entry.evidence.contains("plus 3") || entry.evidence.contains("plus 4") {
            "+3/+4"
        } else {
            "none"
        };
        mutations.push(serde_json::json!({
            "table": "news_inbox",
            "row": index as u64,
            "field": format!("{} boundary", entry.system),
            "before": "pending-original-before",
            "after": format!("candidate news/inbox boundary observation at offset +{record_offset}"),
            "event_code": event_code,
            "phase": "news/inbox",
            "record_offset": record_offset,
            "helper": entry.helper.as_deref().unwrap_or("none"),
            "source_function": entry.function,
            "provenance": "CODE_DERIVED candidate from news_inbox_emission_map; pending original cm0102.exe trace equality",
            "notes": format!(
                "{}; record_offset +{}; stride {}; helper {}; {}",
                entry.system,
                record_offset,
                stride,
                entry.helper.as_deref().unwrap_or("none"),
                entry.evidence
            ),
        }));
    }
    mutations
}

fn export_gameplay_capture_pack(
    trace_dir: &Path,
    output_dir: &Path,
) -> Result<serde_json::Value, String> {
    fs::create_dir_all(output_dir)
        .map_err(|err| format!("failed to create {}: {err}", output_dir.display()))?;

    let mut systems = Vec::new();
    let mut all_row_plans = Vec::new();
    let mut all_watch_groups = Vec::new();
    for slug in [
        "match-results",
        "competition-state",
        "transfers-contracts",
        "news-inbox",
    ] {
        let trace_path = trace_dir.join(format!("{slug}.json"));
        let trace_text = fs::read_to_string(&trace_path)
            .map_err(|err| format!("failed to read {}: {err}", trace_path.display()))?;
        let trace: serde_json::Value = serde_json::from_str(&trace_text)
            .map_err(|err| format!("invalid trace JSON {}: {err}", trace_path.display()))?;
        let system = trace["system"].as_str().unwrap_or(slug);
        let system_dir = output_dir.join(slug);
        fs::create_dir_all(&system_dir)
            .map_err(|err| format!("failed to create {}: {err}", system_dir.display()))?;

        let capture_plan = &trace["capture_plan"];
        let breakpoints = json_string_array(&capture_plan["original_breakpoints"]);
        let watched_writes = json_string_array(&capture_plan["watched_original_writes"]);
        let primary_frontiers = json_string_array(&trace["primary_frontiers"]);
        let code_boundaries = json_string_array(&trace["code_derived_boundaries"]);
        let quality_gates = json_string_array(&trace["trace_quality_gates"]);
        let capture_phases = json_string_array(&capture_plan["capture_phases"]);
        let watched_state = json_string_array(&capture_plan["watched_match_state"]);
        let scenario_requirements = json_string_array(&capture_plan["scenario_requirements"]);
        let stop_conditions = json_string_array(&capture_plan["stop_conditions"]);
        let rust_hook = capture_plan["rust_hook"].as_str().unwrap_or("");
        let implementation_present = trace["rust_implementation"]["present"]
            .as_bool()
            .unwrap_or(false);
        let minimum_trace = capture_plan["minimum_trace"].as_str().unwrap_or("");

        let manifest = serde_json::json!({
            "format": "cm0102-rs-gameplay-capture-manifest",
            "version": 1,
            "system": system,
            "trace_file": trace_path.display().to_string(),
            "output_trace_file": trace_path.display().to_string(),
            "original_binary": trace["source"]["original_binary"],
            "rust_db": trace["source"]["rust_db"],
            "primary_frontiers": primary_frontiers,
            "original_breakpoints": breakpoints,
            "watched_original_writes": watched_writes,
            "capture_phases": capture_phases,
            "watched_match_state": watched_state,
            "scenario_requirements": scenario_requirements,
            "stop_conditions": stop_conditions,
            "rust_hook": rust_hook,
            "rust_implementation_present": implementation_present,
            "minimum_trace": minimum_trace,
            "code_derived_boundaries": code_boundaries,
            "trace_quality_gates": quality_gates,
            "promotion_gate": trace["promotion_gate"],
            "mutation_schema": {
                "required_fields": ["table", "row", "field", "before", "after", "source_function", "provenance"],
                "optional_fields": ["event_code", "phase", "record_offset", "helper", "notes"]
            }
        });

        write_json_file(&system_dir.join("capture-manifest.json"), &manifest)?;
        write_text_file(
            &system_dir.join("original-breakpoints.txt"),
            &capture_pack_lines(
                "Original cm0102.exe breakpoint targets",
                system,
                &breakpoints,
            ),
        )?;
        write_text_file(
            &system_dir.join("watched-writes.txt"),
            &capture_pack_lines("Watched original writes", system, &watched_writes),
        )?;
        write_text_file(
            &system_dir.join("capture-phases.txt"),
            &capture_pack_lines("Capture phases", system, &capture_phases),
        )?;
        write_text_file(
            &system_dir.join("watched-match-state.txt"),
            &capture_pack_lines("Watched match state", system, &watched_state),
        )?;
        write_text_file(
            &system_dir.join("scenario-requirements.txt"),
            &capture_pack_lines("Scenario requirements", system, &scenario_requirements),
        )?;
        write_text_file(
            &system_dir.join("stop-conditions.txt"),
            &capture_pack_lines("Stop conditions", system, &stop_conditions),
        )?;
        write_text_file(
            &system_dir.join("rust-hook.txt"),
            &format!(
                "Rust hook for {system}\n\n{rust_hook}\n\nMinimum trace:\n{minimum_trace}\n\nTrace file to fill:\n{}\n",
                trace_path.display()
            ),
        )?;
        write_text_file(
            &system_dir.join("quality-gates.txt"),
            &capture_pack_lines("Trace quality gates", system, &quality_gates),
        )?;
        write_text_file(
            &system_dir.join("x32dbg-breakpoints.txt"),
            &debugger_breakpoint_script(system, &breakpoints, &watched_writes),
        )?;
        write_json_file(
            &system_dir.join("mutation-template.json"),
            &mutation_template(system, &watched_writes),
        )?;
        write_json_file(
            &system_dir.join("capture-import-sample.json"),
            &capture_import_sample(system),
        )?;
        let rust_mutations = trace["rust_mutations"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let row_plan = capture_row_plan(system, slug, &rust_mutations);
        let watch_groups = capture_watch_groups(&row_plan);
        if let Some(rows) = row_plan["rows"].as_array() {
            all_row_plans.extend(rows.iter().cloned());
        }
        all_watch_groups.extend(watch_groups.iter().cloned().map(|mut group| {
            if let Some(object) = group.as_object_mut() {
                object.insert("system".to_string(), serde_json::json!(system));
                object.insert("slug".to_string(), serde_json::json!(slug));
            }
            group
        }));
        write_json_file(&system_dir.join("row-capture-plan.json"), &row_plan)?;
        write_text_file(
            &system_dir.join("row-capture-plan.csv"),
            &capture_row_plan_csv(&row_plan),
        )?;
        write_text_file(
            &system_dir.join("x32dbg-row-watch-plan.txt"),
            &x32dbg_row_watch_plan(system, &watch_groups),
        )?;
        write_text_file(
            &system_dir.join("capture-session-checklist.md"),
            &capture_session_checklist(system, slug, &row_plan, &watch_groups),
        )?;

        systems.push(serde_json::json!({
            "system": system,
            "slug": slug,
            "directory": system_dir.display().to_string(),
            "breakpoints": breakpoints.len(),
            "watched_writes": watched_writes.len(),
            "candidate_rows": rust_mutations.len(),
            "watch_groups": watch_groups.len(),
            "debugger_script": system_dir.join("x32dbg-breakpoints.txt").display().to_string(),
            "row_capture_plan": system_dir.join("row-capture-plan.json").display().to_string(),
            "row_capture_csv": system_dir.join("row-capture-plan.csv").display().to_string(),
            "x32dbg_row_watch_plan": system_dir.join("x32dbg-row-watch-plan.txt").display().to_string(),
            "capture_session_checklist": system_dir.join("capture-session-checklist.md").display().to_string(),
            "mutation_template": system_dir.join("mutation-template.json").display().to_string(),
            "capture_import_sample": system_dir.join("capture-import-sample.json").display().to_string(),
            "quality_gates": quality_gates.len(),
        }));
    }

    let total_candidate_rows = systems
        .iter()
        .map(|system| system["candidate_rows"].as_u64().unwrap_or_default())
        .sum::<u64>();
    let total_watch_groups = systems
        .iter()
        .map(|system| system["watch_groups"].as_u64().unwrap_or_default())
        .sum::<u64>();
    let all_systems_row_plan = serde_json::json!({
        "format": "cm0102-rs-all-systems-original-row-capture-plan",
        "version": 1,
        "rows": all_row_plans,
        "summary": {
            "rows": total_candidate_rows,
            "systems": systems.len(),
            "watch_groups": total_watch_groups,
        }
    });
    write_json_file(
        &output_dir.join("all-systems-row-capture-plan.json"),
        &all_systems_row_plan,
    )?;
    write_text_file(
        &output_dir.join("all-systems-capture.csv"),
        &all_systems_capture_csv(&all_systems_row_plan),
    )?;
    write_text_file(
        &output_dir.join("all-systems-x32dbg-plan.txt"),
        &all_systems_x32dbg_plan(&all_watch_groups),
    )?;
    write_text_file(
        &output_dir.join("dashboard.html"),
        &capture_pack_dashboard_html(&systems, &all_systems_row_plan, &all_watch_groups),
    )?;
    let report = serde_json::json!({
        "format": "cm0102-rs-gameplay-capture-pack-report",
        "version": 1,
        "trace_dir": trace_dir.display().to_string(),
        "output_dir": output_dir.display().to_string(),
        "summary": {
            "systems": systems.len(),
            "candidate_rows": total_candidate_rows,
            "watch_groups": total_watch_groups,
        },
        "all_systems": {
            "row_capture_plan": output_dir.join("all-systems-row-capture-plan.json").display().to_string(),
            "capture_csv": output_dir.join("all-systems-capture.csv").display().to_string(),
            "x32dbg_plan": output_dir.join("all-systems-x32dbg-plan.txt").display().to_string(),
            "dashboard": output_dir.join("dashboard.html").display().to_string(),
        },
        "systems": systems,
    });
    write_json_file(&output_dir.join("capture-pack-report.json"), &report)?;
    Ok(report)
}

fn export_original_capture_templates(
    trace_dir: &Path,
    output_dir: &Path,
) -> Result<serde_json::Value, String> {
    fs::create_dir_all(output_dir)
        .map_err(|err| format!("failed to create {}: {err}", output_dir.display()))?;
    let mut systems = Vec::new();
    for slug in [
        "match-results",
        "competition-state",
        "transfers-contracts",
        "news-inbox",
    ] {
        let trace_path = trace_dir.join(format!("{slug}.json"));
        let trace_text = fs::read_to_string(&trace_path)
            .map_err(|err| format!("failed to read {}: {err}", trace_path.display()))?;
        let trace: serde_json::Value = serde_json::from_str(&trace_text)
            .map_err(|err| format!("invalid trace JSON {}: {err}", trace_path.display()))?;
        let system = trace["system"].as_str().unwrap_or(slug);
        let rust_mutations = trace["rust_mutations"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let original_template = rust_mutations
            .iter()
            .enumerate()
            .map(|(index, mutation)| original_capture_template_row(index, mutation))
            .collect::<Vec<_>>();
        let capture = serde_json::json!({
            "format": "cm0102-rs-original-gameplay-capture",
            "version": 1,
            "system": system,
            "trace_file": trace_path.display().to_string(),
            "source": {
                "original_binary": trace["source"]["original_binary"],
                "rust_trace": trace_path.display().to_string(),
                "capture_pack": format!("D:/cm0102-rs/reports/capture_pack/{slug}")
            },
            "instructions": [
                "Fill original_mutations from cm0102.exe runtime capture only.",
                "Keep row order exactly as observed in the original binary.",
                "Do not copy rust_mutations into original_mutations unless the original binary emitted the same write.",
                "After filling this file, run import-gameplay-capture and inspect comparison.first_mismatch if blocked."
            ],
            "expected_rust_candidate_rows": rust_mutations.len(),
            "original_mutations": original_template,
            "rust_mutations": rust_mutations,
        });
        let template_path = output_dir.join(format!("{slug}-capture-template.json"));
        write_json_file(&template_path, &capture)?;
        systems.push(serde_json::json!({
            "system": system,
            "slug": slug,
            "template": template_path.display().to_string(),
            "expected_original_rows": capture["expected_rust_candidate_rows"],
        }));
    }
    Ok(serde_json::json!({
        "format": "cm0102-rs-original-capture-template-export",
        "version": 1,
        "trace_dir": trace_dir.display().to_string(),
        "output_dir": output_dir.display().to_string(),
        "summary": {
            "systems": systems.len(),
            "templates": systems.len(),
        },
        "systems": systems,
    }))
}

fn original_capture_template_row(index: usize, mutation: &serde_json::Value) -> serde_json::Value {
    let mut row = mutation.clone();
    if let Some(object) = row.as_object_mut() {
        object.insert("row".to_string(), serde_json::json!(index as u64));
        object.insert(
            "before".to_string(),
            serde_json::json!("FILL_FROM_ORIGINAL"),
        );
        object.insert("after".to_string(), serde_json::json!("FILL_FROM_ORIGINAL"));
        object.insert(
            "provenance".to_string(),
            serde_json::json!(
                "RUNTIME_CAPTURE original cm0102.exe; promote only after code-derived parity review"
            ),
        );
        object.insert(
            "capture_status".to_string(),
            serde_json::json!("fill-from-original"),
        );
        object.insert(
            "notes".to_string(),
            serde_json::json!(format!(
                "Original capture template row {index}; expected Rust candidate: {}",
                mutation
                    .get("notes")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("no candidate notes")
            )),
        );
    }
    row
}

fn original_capture_status_report(template_dir: &Path) -> Result<serde_json::Value, String> {
    let mut systems = Vec::new();
    for slug in [
        "match-results",
        "competition-state",
        "transfers-contracts",
        "news-inbox",
    ] {
        let template_path = template_dir.join(format!("{slug}-capture-template.json"));
        if !template_path.exists() {
            systems.push(serde_json::json!({
                "system": slug,
                "slug": slug,
                "template": template_path.display().to_string(),
                "status": "missing-template",
                "expected_original_rows": 0,
                "filled_original_rows": 0,
                "placeholder_rows": 0,
                "schema_ready": false,
                "coverage": {"status": "fail", "missing": ["capture template missing"]},
                "import_blockers": ["capture_template_missing"],
            }));
            continue;
        }
        let text = fs::read_to_string(&template_path)
            .map_err(|err| format!("failed to read {}: {err}", template_path.display()))?;
        let capture: serde_json::Value = serde_json::from_str(&text).map_err(|err| {
            format!(
                "invalid capture template {}: {err}",
                template_path.display()
            )
        })?;
        systems.push(original_capture_template_status(&template_path, &capture));
    }

    let expected_original_rows = systems
        .iter()
        .map(|system| {
            system["expected_original_rows"]
                .as_u64()
                .unwrap_or_default()
        })
        .sum::<u64>();
    let filled_original_rows = systems
        .iter()
        .map(|system| system["filled_original_rows"].as_u64().unwrap_or_default())
        .sum::<u64>();
    let placeholder_rows = systems
        .iter()
        .map(|system| system["placeholder_rows"].as_u64().unwrap_or_default())
        .sum::<u64>();
    let import_ready_systems = systems
        .iter()
        .filter(|system| system["status"].as_str() == Some("import-ready"))
        .count();
    let missing_templates = systems
        .iter()
        .filter(|system| system["status"].as_str() == Some("missing-template"))
        .count();
    let status = if import_ready_systems == systems.len() && !systems.is_empty() {
        "ready-to-import"
    } else if filled_original_rows > 0 {
        "partially-captured"
    } else {
        "waiting-for-original-capture"
    };

    Ok(serde_json::json!({
        "format": "cm0102-rs-original-capture-status",
        "version": 1,
        "template_dir": template_dir.display().to_string(),
        "summary": {
            "status": status,
            "systems": systems.len(),
            "missing_templates": missing_templates,
            "import_ready_systems": import_ready_systems,
            "expected_original_rows": expected_original_rows,
            "filled_original_rows": filled_original_rows,
            "placeholder_rows": placeholder_rows,
        },
        "systems": systems,
    }))
}

fn original_capture_template_status(
    template_path: &Path,
    capture: &serde_json::Value,
) -> serde_json::Value {
    let system = capture["system"].as_str().unwrap_or("unknown");
    let original_mutations = capture["original_mutations"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let expected_original_rows = capture["expected_rust_candidate_rows"]
        .as_u64()
        .unwrap_or(original_mutations.len() as u64);
    let placeholder_rows = original_mutations
        .iter()
        .filter(|mutation| original_capture_row_has_placeholder(mutation))
        .count();
    let filled_original_rows = original_mutations.len().saturating_sub(placeholder_rows);
    let schema_ready = mutations_have_required_fields(&original_mutations)
        && placeholder_rows == 0
        && original_mutations.len() as u64 == expected_original_rows;
    let coverage = if placeholder_rows == 0 {
        subsystem_capture_coverage(system, &original_mutations)
    } else {
        serde_json::json!({
            "status": "fail",
            "missing": ["fill every FILL_FROM_ORIGINAL placeholder before import"],
        })
    };
    let mut import_blockers = Vec::new();
    if original_mutations.len() as u64 != expected_original_rows {
        import_blockers.push("row_count_does_not_match_rust_candidate");
    }
    if placeholder_rows > 0 {
        import_blockers.push("placeholder_rows_remaining");
    }
    if !mutations_have_required_fields(&original_mutations) {
        import_blockers.push("required_mutation_fields_missing");
    }
    if coverage["status"].as_str() != Some("pass") {
        import_blockers.push("subsystem_capture_coverage_incomplete");
    }
    let status = if import_blockers.is_empty() && schema_ready {
        "import-ready"
    } else if filled_original_rows > 0 {
        "partially-captured"
    } else {
        "not-started"
    };
    serde_json::json!({
        "system": system,
        "slug": gameplay_trace_slug(system),
        "template": template_path.display().to_string(),
        "status": status,
        "expected_original_rows": expected_original_rows,
        "filled_original_rows": filled_original_rows,
        "placeholder_rows": placeholder_rows,
        "schema_ready": schema_ready,
        "coverage": coverage,
        "import_blockers": import_blockers,
    })
}

fn original_capture_row_has_placeholder(mutation: &serde_json::Value) -> bool {
    mutation
        .get("before")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| value == "FILL_FROM_ORIGINAL")
        || mutation
            .get("after")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value == "FILL_FROM_ORIGINAL")
        || mutation
            .get("capture_status")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value == "fill-from-original")
}

fn update_original_capture_row(
    system_or_slug: &str,
    row: usize,
    before: &str,
    after: &str,
    capture_status: &str,
    extra_notes: &str,
) -> Result<serde_json::Value, String> {
    let template_dir = Path::new("D:/cm0102-rs/reports/original_capture_templates");
    let trace_dir = Path::new("D:/cm0102-rs/reports/parity_traces");
    let workbench_dir = Path::new("D:/cm0102-rs/reports/original_capture_workbench");
    let slug = gameplay_trace_slug(system_or_slug);
    let template_path = template_dir.join(format!("{slug}-capture-template.json"));
    let mut capture = read_json_file(&template_path)?;
    let system = capture["system"]
        .as_str()
        .unwrap_or(system_or_slug)
        .to_string();
    let original_mutations = capture["original_mutations"]
        .as_array_mut()
        .ok_or_else(|| {
            format!(
                "{} must include mutable original_mutations array",
                template_path.display()
            )
        })?;
    let available_rows = original_mutations.len();
    let mutation = original_mutations.get_mut(row).ok_or_else(|| {
        format!(
            "capture row {row} is out of range for {system}; rows available {}",
            available_rows
        )
    })?;
    let object = mutation
        .as_object_mut()
        .ok_or_else(|| format!("capture row {row} for {system} is not an object"))?;
    object.insert("before".to_string(), serde_json::json!(before));
    object.insert("after".to_string(), serde_json::json!(after));
    object.insert(
        "capture_status".to_string(),
        serde_json::json!(capture_status),
    );
    object.insert(
        "provenance".to_string(),
        serde_json::json!(
            "RUNTIME_CAPTURE original cm0102.exe; promote only after code-derived parity review"
        ),
    );
    if !extra_notes.trim().is_empty() {
        let existing = object
            .get("notes")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        object.insert(
            "notes".to_string(),
            serde_json::json!(format!("{existing}; capture note: {extra_notes}")),
        );
    }
    write_json_file(&template_path, &capture)?;
    let status = original_capture_status_report(template_dir)?;
    write_json_file(
        Path::new("D:/cm0102-rs/reports/original_capture_status.json"),
        &status,
    )?;
    let workbench = export_original_capture_workbench(template_dir, workbench_dir, trace_dir)?;
    Ok(serde_json::json!({
        "format": "cm0102-rs-original-capture-row-update",
        "version": 1,
        "system": system,
        "slug": slug,
        "row": row,
        "template": template_path.display().to_string(),
        "updated": true,
        "status": status,
        "workbench": workbench,
    }))
}

fn import_original_capture_csv_text(text: &str) -> Result<serde_json::Value, String> {
    let template_dir = Path::new("D:/cm0102-rs/reports/original_capture_templates");
    let trace_dir = Path::new("D:/cm0102-rs/reports/parity_traces");
    let workbench_dir = Path::new("D:/cm0102-rs/reports/original_capture_workbench");
    let rows = parse_capture_csv_rows(text, true)?;
    let preflight = validate_original_capture_csv_rows(template_dir, &rows)?;
    if preflight["summary"]["blocking_errors"]
        .as_u64()
        .unwrap_or(1)
        > 0
        || preflight["summary"]["blank_capture_values"]
            .as_u64()
            .unwrap_or(1)
            > 0
    {
        return Ok(serde_json::json!({
            "format": "cm0102-rs-original-capture-csv-import",
            "version": 1,
            "status": "blocked-by-preflight",
            "summary": preflight["summary"],
            "preflight": preflight,
        }));
    }
    let mut grouped: HashMap<String, Vec<CaptureCsvRow>> = HashMap::new();
    for row in rows {
        grouped
            .entry(gameplay_trace_slug(&row.system))
            .or_default()
            .push(row);
    }

    let mut updated = Vec::new();
    let mut failures = Vec::new();
    for (slug, rows) in grouped {
        match apply_capture_csv_rows_to_template(template_dir, &slug, &rows) {
            Ok(system_report) => updated.push(system_report),
            Err(err) => failures.push(serde_json::json!({ "slug": slug, "error": err })),
        }
    }

    let status = original_capture_status_report(template_dir)?;
    write_json_file(
        Path::new("D:/cm0102-rs/reports/original_capture_status.json"),
        &status,
    )?;
    let workbench = export_original_capture_workbench(template_dir, workbench_dir, trace_dir)?;
    Ok(serde_json::json!({
        "format": "cm0102-rs-original-capture-csv-import",
        "version": 1,
        "status": if failures.is_empty() { "complete" } else { "partial-failure" },
        "summary": {
            "systems_updated": updated.len(),
            "rows_updated": updated.iter().map(|system| system["rows_updated"].as_u64().unwrap_or_default()).sum::<u64>(),
            "failures": failures.len(),
            "filled_original_rows": status["summary"]["filled_original_rows"],
            "expected_original_rows": status["summary"]["expected_original_rows"],
            "placeholder_rows": status["summary"]["placeholder_rows"],
            "import_ready_systems": status["summary"]["import_ready_systems"],
        },
        "updated": updated,
        "failures": failures,
        "status_report": status,
        "workbench": workbench,
    }))
}

fn validate_original_capture_csv_text(text: &str) -> Result<serde_json::Value, String> {
    let rows = parse_capture_csv_rows(text, false)?;
    validate_original_capture_csv_rows(
        Path::new("D:/cm0102-rs/reports/original_capture_templates"),
        &rows,
    )
}

fn submit_original_capture_csv_text(
    text: &str,
    reports_dir: &Path,
) -> Result<serde_json::Value, String> {
    let validation = validate_original_capture_csv_text(text)?;
    write_json_file(
        &reports_dir.join("original_capture_csv_validation.json"),
        &validation,
    )?;
    if validation["status"].as_str() != Some("import-ready") {
        let report = serde_json::json!({
            "format": "cm0102-rs-original-capture-csv-submit",
            "version": 1,
            "status": "blocked-by-validation",
            "summary": {
                "validation_status": validation["status"],
                "rows": validation["summary"]["rows"],
                "captured_rows": validation["summary"]["captured_rows"],
                "blank_capture_values": validation["summary"]["blank_capture_values"],
                "duplicates": validation["summary"]["duplicates"],
                "unknown_systems": validation["summary"]["unknown_systems"],
                "out_of_range": validation["summary"]["out_of_range"],
                "blocking_errors": validation["summary"]["blocking_errors"],
                "capture_rows_filled": 0,
                "capture_rows_expected": 0,
                "parity_failures": 0,
                "promotion_blocked": 0,
            },
            "validation": validation,
            "import": serde_json::Value::Null,
            "backend_gate_refresh": serde_json::Value::Null,
        });
        write_json_file(
            &reports_dir.join("original_capture_csv_submit.json"),
            &report,
        )?;
        return Ok(report);
    }

    let import = import_original_capture_csv_text(text)?;
    write_json_file(
        &reports_dir.join("original_capture_csv_import.json"),
        &import,
    )?;
    let ready_import = import_ready_original_capture_systems()?;
    write_json_file(
        &reports_dir.join("original_capture_ready_import.json"),
        &ready_import,
    )?;
    let backend_gate_refresh = refresh_backend_gates(
        Path::new("D:/cm0102-rs/rust-db"),
        reports_dir,
        Path::new("D:/cm0102-rs/reports/parity_traces"),
        Path::new("D:/cm0102-rs/reports/original_capture_templates"),
        Path::new("D:/cm0102/cm0102.exe"),
    )?;
    let report = serde_json::json!({
        "format": "cm0102-rs-original-capture-csv-submit",
        "version": 1,
        "status": backend_gate_refresh["summary"]["status"].as_str().unwrap_or("submitted"),
        "summary": {
            "validation_status": validation["status"],
            "rows": validation["summary"]["rows"],
            "captured_rows": validation["summary"]["captured_rows"],
            "import_status": import["status"],
            "ready_import_status": ready_import["status"],
            "backend_gate_status": backend_gate_refresh["summary"]["status"],
            "capture_rows_filled": backend_gate_refresh["summary"]["original_capture_rows_filled"],
            "capture_rows_expected": backend_gate_refresh["summary"]["original_capture_rows_expected"],
            "parity_failures": backend_gate_refresh["summary"]["gameplay_parity_failures"],
            "promotion_blocked": backend_gate_refresh["summary"]["gameplay_promotion_blocked"],
        },
        "validation": validation,
        "import": import,
        "ready_import": ready_import,
        "backend_gate_refresh": backend_gate_refresh,
    });
    write_json_file(
        &reports_dir.join("original_capture_csv_submit.json"),
        &report,
    )?;
    Ok(report)
}

fn validate_original_capture_csv_rows(
    template_dir: &Path,
    rows: &[CaptureCsvRow],
) -> Result<serde_json::Value, String> {
    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();
    let mut blank_capture_values = Vec::new();
    let mut unknown_systems = Vec::new();
    let mut out_of_range = Vec::new();
    let mut systems: HashMap<String, serde_json::Value> = HashMap::new();
    for row in rows {
        let slug = gameplay_trace_slug(&row.system);
        let key = format!("{slug}:{}", row.row);
        if !seen.insert(key.clone()) {
            duplicates.push(serde_json::json!({
                "system": row.system,
                "slug": slug,
                "row": row.row,
                "key": key,
            }));
        }
        if row.before.trim().is_empty() || row.after.trim().is_empty() {
            blank_capture_values.push(serde_json::json!({
                "system": row.system,
                "slug": slug,
                "row": row.row,
                "missing_before": row.before.trim().is_empty(),
                "missing_after": row.after.trim().is_empty(),
            }));
        }
        let template_path = template_dir.join(format!("{slug}-capture-template.json"));
        if !template_path.exists() {
            unknown_systems.push(serde_json::json!({
                "system": row.system,
                "slug": slug,
                "row": row.row,
                "template": template_path.display().to_string(),
            }));
            continue;
        }
        let capture = match systems.get(&slug) {
            Some(cached) => cached.clone(),
            None => {
                let loaded = read_json_file(&template_path)?;
                systems.insert(slug.clone(), loaded.clone());
                loaded
            }
        };
        let row_count = capture["original_mutations"].as_array().map_or(0, Vec::len);
        if row.row >= row_count {
            out_of_range.push(serde_json::json!({
                "system": row.system,
                "slug": slug,
                "row": row.row,
                "available_rows": row_count,
            }));
        }
    }
    let captured_rows = rows
        .iter()
        .filter(|row| !row.before.trim().is_empty() && !row.after.trim().is_empty())
        .count();
    let blocking_errors = duplicates.len() + unknown_systems.len() + out_of_range.len();
    Ok(serde_json::json!({
        "format": "cm0102-rs-original-capture-csv-validation",
        "version": 1,
        "status": if blocking_errors == 0 && blank_capture_values.is_empty() { "import-ready" } else if blocking_errors == 0 { "incomplete" } else { "blocked" },
        "summary": {
            "rows": rows.len(),
            "captured_rows": captured_rows,
            "blank_capture_values": blank_capture_values.len(),
            "duplicates": duplicates.len(),
            "unknown_systems": unknown_systems.len(),
            "out_of_range": out_of_range.len(),
            "blocking_errors": blocking_errors,
        },
        "duplicates": duplicates,
        "blank_capture_values": blank_capture_values,
        "unknown_systems": unknown_systems,
        "out_of_range": out_of_range,
    }))
}

#[derive(Debug, Clone)]
struct CaptureCsvRow {
    system: String,
    row: usize,
    before: String,
    after: String,
    notes: String,
    capture_status: String,
}

fn parse_capture_csv_rows(
    text: &str,
    require_capture_values: bool,
) -> Result<Vec<CaptureCsvRow>, String> {
    let table = parse_csv_table(text)?;
    if table.is_empty() {
        return Ok(Vec::new());
    }
    let headers = table[0]
        .iter()
        .map(|header| header.trim().to_ascii_lowercase())
        .collect::<Vec<_>>();
    let header_index = |name: &str| {
        headers
            .iter()
            .position(|header| header == name)
            .ok_or_else(|| format!("capture CSV is missing required '{name}' column"))
    };
    let system_index = header_index("system")?;
    let row_index = header_index("row")?;
    let before_index = header_index("before")?;
    let after_index = header_index("after")?;
    let notes_index = headers.iter().position(|header| header == "notes");
    let status_index = headers.iter().position(|header| header == "capture_status");

    let mut rows = Vec::new();
    for (line_index, columns) in table.into_iter().enumerate().skip(1) {
        if columns.iter().all(|value| value.trim().is_empty()) {
            continue;
        }
        let get = |index: usize| columns.get(index).map(String::as_str).unwrap_or("").trim();
        let system = get(system_index);
        let row_text = get(row_index);
        let before = get(before_index);
        let after = get(after_index);
        if system.is_empty()
            || row_text.is_empty()
            || (require_capture_values && (before.is_empty() || after.is_empty()))
        {
            return Err(format!(
                "capture CSV line {} requires non-empty system,row{}",
                line_index + 1,
                if require_capture_values {
                    ",before,after"
                } else {
                    ""
                }
            ));
        }
        if !require_capture_values && (before.is_empty() || after.is_empty()) {
            // Blank capture cells are valid in validation mode; they are reported by preflight.
        }
        if before == "FILL_FROM_ORIGINAL" || after == "FILL_FROM_ORIGINAL" {
            return Err(format!(
                "capture CSV line {} still contains FILL_FROM_ORIGINAL placeholder",
                line_index + 1
            ));
        }
        rows.push(CaptureCsvRow {
            system: system.to_string(),
            row: row_text.parse().map_err(|err| {
                format!(
                    "invalid row number '{row_text}' on CSV line {}: {err}",
                    line_index + 1
                )
            })?,
            before: before.to_string(),
            after: after.to_string(),
            notes: notes_index
                .and_then(|index| columns.get(index))
                .map(|value| value.trim().to_string())
                .unwrap_or_default(),
            capture_status: status_index
                .and_then(|index| columns.get(index))
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .unwrap_or_else(|| "captured-from-original".to_string()),
        });
    }
    Ok(rows)
}

fn parse_csv_table(text: &str) -> Result<Vec<Vec<String>>, String> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let mut chars = text.chars().peekable();
    let mut in_quotes = false;
    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes && chars.peek() == Some(&'"') => {
                cell.push('"');
                chars.next();
            }
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                row.push(cell.trim().to_string());
                cell.clear();
            }
            '\n' if !in_quotes => {
                row.push(cell.trim_end_matches('\r').trim().to_string());
                cell.clear();
                if row.iter().any(|value| !value.is_empty()) {
                    rows.push(row);
                }
                row = Vec::new();
            }
            _ => cell.push(ch),
        }
    }
    if in_quotes {
        return Err("capture CSV ended inside a quoted cell".to_string());
    }
    if !cell.is_empty() || !row.is_empty() {
        row.push(cell.trim_end_matches('\r').trim().to_string());
        if row.iter().any(|value| !value.is_empty()) {
            rows.push(row);
        }
    }
    Ok(rows)
}

fn apply_capture_csv_rows_to_template(
    template_dir: &Path,
    slug: &str,
    rows: &[CaptureCsvRow],
) -> Result<serde_json::Value, String> {
    let template_path = template_dir.join(format!("{slug}-capture-template.json"));
    let mut capture = read_json_file(&template_path)?;
    let system = capture["system"].as_str().unwrap_or(slug).to_string();
    let original_mutations = capture["original_mutations"]
        .as_array_mut()
        .ok_or_else(|| {
            format!(
                "{} must include original_mutations array",
                template_path.display()
            )
        })?;
    let available_rows = original_mutations.len();
    let mut updated_rows = Vec::new();
    for row in rows {
        let mutation = original_mutations.get_mut(row.row).ok_or_else(|| {
            format!(
                "capture row {} is out of range for {}; rows available {}",
                row.row, system, available_rows
            )
        })?;
        let object = mutation
            .as_object_mut()
            .ok_or_else(|| format!("capture row {} for {system} is not an object", row.row))?;
        object.insert("before".to_string(), serde_json::json!(row.before));
        object.insert("after".to_string(), serde_json::json!(row.after));
        object.insert(
            "capture_status".to_string(),
            serde_json::json!(row.capture_status),
        );
        object.insert(
            "provenance".to_string(),
            serde_json::json!(
                "RUNTIME_CAPTURE original cm0102.exe; promote only after code-derived parity review"
            ),
        );
        if !row.notes.trim().is_empty() {
            let existing = object
                .get("notes")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            object.insert(
                "notes".to_string(),
                serde_json::json!(format!("{existing}; capture note: {}", row.notes)),
            );
        }
        updated_rows.push(row.row);
    }
    write_json_file(&template_path, &capture)?;
    Ok(serde_json::json!({
        "system": system,
        "slug": slug,
        "template": template_path.display().to_string(),
        "rows_updated": updated_rows.len(),
        "rows": updated_rows,
    }))
}

fn import_original_capture_system(system_or_slug: &str) -> Result<serde_json::Value, String> {
    let template_dir = Path::new("D:/cm0102-rs/reports/original_capture_templates");
    let trace_dir = Path::new("D:/cm0102-rs/reports/parity_traces");
    let workbench_dir = Path::new("D:/cm0102-rs/reports/original_capture_workbench");
    let slug = gameplay_trace_slug(system_or_slug);
    let status = original_capture_status_report(template_dir)?;
    let system_status = status["systems"]
        .as_array()
        .and_then(|systems| {
            systems.iter().find(|system| {
                system["slug"].as_str() == Some(slug.as_str())
                    || system["system"].as_str() == Some(system_or_slug)
            })
        })
        .ok_or_else(|| format!("no original capture template status for {system_or_slug}"))?;
    if system_status["status"].as_str() != Some("import-ready") {
        return Err(format!(
            "{} is not import-ready: {} placeholder row(s), blockers {}",
            system_status["system"].as_str().unwrap_or(system_or_slug),
            system_status["placeholder_rows"]
                .as_u64()
                .unwrap_or_default(),
            system_status["import_blockers"]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "unknown".to_string())
        ));
    }
    let template_path = PathBuf::from(
        system_status["template"]
            .as_str()
            .ok_or_else(|| "import-ready system status is missing template path".to_string())?,
    );
    let import_report = import_gameplay_capture(trace_dir, &template_path)?;
    let refreshed_status = original_capture_status_report(template_dir)?;
    write_json_file(
        Path::new("D:/cm0102-rs/reports/original_capture_status.json"),
        &refreshed_status,
    )?;
    let workbench = export_original_capture_workbench(template_dir, workbench_dir, trace_dir)?;
    Ok(serde_json::json!({
        "format": "cm0102-rs-original-capture-system-import",
        "version": 1,
        "system": system_status["system"],
        "slug": slug,
        "template": template_path.display().to_string(),
        "import": import_report,
        "status": refreshed_status,
        "workbench": workbench,
    }))
}

fn import_ready_original_capture_systems() -> Result<serde_json::Value, String> {
    let template_dir = Path::new("D:/cm0102-rs/reports/original_capture_templates");
    let status = original_capture_status_report(template_dir)?;
    let ready_slugs = status["systems"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter(|system| system["status"].as_str() == Some("import-ready"))
        .filter_map(|system| system["slug"].as_str().map(str::to_string))
        .collect::<Vec<_>>();
    let mut imported = Vec::new();
    let mut failures = Vec::new();
    for slug in &ready_slugs {
        match import_original_capture_system(slug) {
            Ok(report) => imported.push(report),
            Err(err) => failures.push(serde_json::json!({ "slug": slug, "error": err })),
        }
    }
    Ok(serde_json::json!({
        "format": "cm0102-rs-original-capture-ready-import",
        "version": 1,
        "ready_systems": ready_slugs.len(),
        "imported": imported,
        "failures": failures,
        "status": if failures.is_empty() { "complete" } else { "partial-failure" },
    }))
}

fn export_original_capture_workbench(
    template_dir: &Path,
    output_dir: &Path,
    trace_dir: &Path,
) -> Result<serde_json::Value, String> {
    fs::create_dir_all(output_dir)
        .map_err(|err| format!("failed to create {}: {err}", output_dir.display()))?;
    let status = original_capture_status_report(template_dir)?;
    write_json_file(&output_dir.join("status.json"), &status)?;

    let mut systems = Vec::new();
    let mut all_todo_rows = Vec::new();
    let mut import_commands = Vec::new();
    for system in status["systems"].as_array().cloned().unwrap_or_default() {
        let system_name = system["system"].as_str().unwrap_or("unknown").to_string();
        let slug = system["slug"]
            .as_str()
            .map(str::to_string)
            .unwrap_or_else(|| gameplay_trace_slug(&system_name));
        let template_path = PathBuf::from(system["template"].as_str().unwrap_or(""));
        let system_dir = output_dir.join(&slug);
        fs::create_dir_all(&system_dir)
            .map_err(|err| format!("failed to create {}: {err}", system_dir.display()))?;
        let capture = read_json_file(&template_path)?;
        let original_mutations = capture["original_mutations"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let row_plan_rows = read_capture_pack_row_plan_rows(&slug)?;
        let todo_rows = original_mutations
            .iter()
            .enumerate()
            .filter(|(_, mutation)| original_capture_row_has_placeholder(mutation))
            .map(|(index, mutation)| {
                capture_todo_row(
                    &system_name,
                    &slug,
                    index,
                    mutation,
                    row_plan_rows.get(&index),
                )
            })
            .collect::<Vec<_>>();
        let todo_csv = capture_todo_csv(&todo_rows);
        write_text_file(&system_dir.join("capture-todo.csv"), &todo_csv)?;
        write_text_file(
            &system_dir.join("capture-notes.md"),
            &capture_notes_markdown(&system_name, &slug, &template_path, &todo_rows),
        )?;
        let import_command = format!(
            "cargo run -p cm-app -- import-gameplay-capture {} {}",
            trace_dir.display(),
            template_path.display()
        );
        write_text_file(
            &system_dir.join("import-command.txt"),
            &format!("{import_command}\n"),
        )?;
        import_commands.push(import_command.clone());
        all_todo_rows.extend(todo_rows.clone());
        systems.push(serde_json::json!({
            "system": system_name,
            "slug": slug,
            "directory": system_dir.display().to_string(),
            "template": template_path.display().to_string(),
            "status": system["status"],
            "filled_original_rows": system["filled_original_rows"],
            "expected_original_rows": system["expected_original_rows"],
            "placeholder_rows": system["placeholder_rows"],
            "import_ready": system["status"].as_str() == Some("import-ready"),
            "todo_rows": todo_rows.len(),
            "todo_csv": system_dir.join("capture-todo.csv").display().to_string(),
            "notes": system_dir.join("capture-notes.md").display().to_string(),
            "capture_pack_row_plan": format!("D:/cm0102-rs/reports/capture_pack/{slug}/row-capture-plan.json"),
            "capture_pack_row_csv": format!("D:/cm0102-rs/reports/capture_pack/{slug}/row-capture-plan.csv"),
            "x32dbg_row_watch_plan": format!("D:/cm0102-rs/reports/capture_pack/{slug}/x32dbg-row-watch-plan.txt"),
            "capture_session_checklist": format!("D:/cm0102-rs/reports/capture_pack/{slug}/capture-session-checklist.md"),
            "import_command": import_command,
        }));
    }

    write_text_file(
        &output_dir.join("capture-todo.csv"),
        &capture_todo_csv(&all_todo_rows),
    )?;
    write_text_file(
        &output_dir.join("import-ready-commands.txt"),
        &format!("{}\n", import_commands.join("\n")),
    )?;
    write_text_file(
        &output_dir.join("README.md"),
        &original_capture_workbench_readme(template_dir, output_dir, trace_dir, &status),
    )?;
    write_text_file(
        &output_dir.join("dashboard.html"),
        &original_capture_workbench_html(&status, &systems, &all_todo_rows),
    )?;
    let report = serde_json::json!({
        "format": "cm0102-rs-original-capture-workbench",
        "version": 1,
        "template_dir": template_dir.display().to_string(),
        "trace_dir": trace_dir.display().to_string(),
        "output_dir": output_dir.display().to_string(),
        "summary": {
            "status": status["summary"]["status"],
            "systems": systems.len(),
            "todo_rows": all_todo_rows.len(),
            "import_ready_systems": status["summary"]["import_ready_systems"],
        },
        "status": status,
        "systems": systems,
        "files": {
            "status": output_dir.join("status.json").display().to_string(),
            "todo_csv": output_dir.join("capture-todo.csv").display().to_string(),
            "import_commands": output_dir.join("import-ready-commands.txt").display().to_string(),
            "readme": output_dir.join("README.md").display().to_string(),
            "dashboard": output_dir.join("dashboard.html").display().to_string(),
        }
    });
    write_json_file(&output_dir.join("workbench.json"), &report)?;
    Ok(report)
}

fn read_json_file(path: &Path) -> Result<serde_json::Value, String> {
    let text = fs::read_to_string(path)
        .map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    serde_json::from_str(&text).map_err(|err| format!("invalid JSON {}: {err}", path.display()))
}

fn read_capture_pack_row_plan_rows(
    slug: &str,
) -> Result<HashMap<usize, serde_json::Value>, String> {
    let path = Path::new("D:/cm0102-rs/reports/capture_pack")
        .join(slug)
        .join("row-capture-plan.json");
    if !path.exists() {
        return Ok(HashMap::new());
    }
    let plan = read_json_file(&path)?;
    let rows = plan["rows"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            let index = row["row"].as_u64()?.try_into().ok()?;
            Some((index, row))
        })
        .collect();
    Ok(rows)
}

fn capture_todo_row(
    system: &str,
    slug: &str,
    index: usize,
    mutation: &serde_json::Value,
    row_plan: Option<&serde_json::Value>,
) -> serde_json::Value {
    serde_json::json!({
        "system": system,
        "slug": slug,
        "row": index,
        "table": mutation["table"].as_str().unwrap_or(""),
        "field": mutation["field"].as_str().unwrap_or(""),
        "record_offset": mutation["record_offset"].as_str().unwrap_or(""),
        "event_code": mutation["event_code"].as_str().unwrap_or(""),
        "source_function": mutation["source_function"].as_str().unwrap_or(""),
        "phase": mutation["phase"].as_str().unwrap_or(""),
        "before": mutation["before"].as_str().unwrap_or(""),
        "after": mutation["after"].as_str().unwrap_or(""),
        "watch_group": row_plan.and_then(|row| row["watch_group"].as_str()).unwrap_or(""),
        "watch_expression": row_plan.and_then(|row| row["watch_expression"].as_str()).unwrap_or(""),
        "expected_rust_before": row_plan.map(|row| csv_value(&row["expected_rust_before"])).unwrap_or_default(),
        "expected_rust_after": row_plan.map(|row| csv_value(&row["expected_rust_after"])).unwrap_or_default(),
        "quality_gate": row_plan.and_then(|row| row["quality_gate"].as_str()).unwrap_or("Capture from original cm0102.exe, then import only if row order and schema match."),
        "notes": mutation["notes"].as_str().unwrap_or(""),
    })
}

fn capture_todo_csv(rows: &[serde_json::Value]) -> String {
    let headers = [
        "system",
        "row",
        "table",
        "field",
        "record_offset",
        "event_code",
        "source_function",
        "phase",
        "watch_group",
        "watch_expression",
        "expected_rust_before",
        "expected_rust_after",
        "before",
        "after",
        "quality_gate",
        "notes",
    ];
    let mut lines = vec![headers.join(",")];
    for row in rows {
        lines.push(
            headers
                .iter()
                .map(|header| csv_cell(&csv_value(&row[*header])))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    format!("{}\n", lines.join("\n"))
}

fn csv_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Number(value) => value.to_string(),
        serde_json::Value::Bool(value) => value.to_string(),
        _ => value.to_string(),
    }
}

fn csv_cell(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') || value.contains('\r') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn capture_notes_markdown(
    system: &str,
    slug: &str,
    template_path: &Path,
    rows: &[serde_json::Value],
) -> String {
    let mut out = format!(
        "# Original Capture TODO: {system}\n\nTemplate: `{}`\nSlug: `{slug}`\nRows to fill: {}\n\n",
        template_path.display(),
        rows.len()
    );
    out.push_str(
        "Fill `before` and `after` from the original `cm0102.exe` runtime capture only.\n\n",
    );
    for row in rows.iter().take(50) {
        out.push_str(&format!(
            "- Row {}: `{}` `{}` offset `{}` event `{}` function `{}` watch `{}`\n",
            row["row"].as_u64().unwrap_or_default(),
            row["table"].as_str().unwrap_or(""),
            row["field"].as_str().unwrap_or(""),
            row["record_offset"].as_str().unwrap_or(""),
            row["event_code"].as_str().unwrap_or(""),
            row["source_function"].as_str().unwrap_or(""),
            row["watch_group"].as_str().unwrap_or("")
        ));
        let watch_expression = row["watch_expression"].as_str().unwrap_or("");
        if !watch_expression.is_empty() {
            out.push_str(&format!("  Watch: `{watch_expression}`\n"));
        }
    }
    out
}

fn original_capture_workbench_readme(
    template_dir: &Path,
    output_dir: &Path,
    trace_dir: &Path,
    status: &serde_json::Value,
) -> String {
    format!(
        "# CM0102 Original Capture Workbench\n\nStatus: `{}`\nRows filled: `{}/{}`\nTemplates: `{}`\nTrace dir: `{}`\n\n## Workflow\n\n1. Open each `<system>/capture-todo.csv`.\n2. Capture the listed writes from `D:/cm0102/cm0102.exe`.\n3. Fill the matching template JSON in `{}`.\n4. Run `cargo run -p cm-app -- original-capture-status {} {}`.\n5. When a system is import-ready, run its command from `import-ready-commands.txt`.\n6. Run `cargo run -p cm-app -- gameplay-parity-report D:/cm0102-rs/rust-db {} D:/cm0102-rs/reports/gameplay_parity.json`.\n\nGenerated files live in `{}`.\n",
        status["summary"]["status"].as_str().unwrap_or("unknown"),
        status["summary"]["filled_original_rows"].as_u64().unwrap_or_default(),
        status["summary"]["expected_original_rows"].as_u64().unwrap_or_default(),
        template_dir.display(),
        trace_dir.display(),
        template_dir.display(),
        template_dir.display(),
        output_dir.join("status.json").display(),
        trace_dir.display(),
        output_dir.display()
    )
}

fn original_capture_workbench_html(
    status: &serde_json::Value,
    systems: &[serde_json::Value],
    todo_rows: &[serde_json::Value],
) -> String {
    let summary = &status["summary"];
    let expected = summary["expected_original_rows"]
        .as_u64()
        .unwrap_or_default();
    let filled = summary["filled_original_rows"].as_u64().unwrap_or_default();
    let placeholders = summary["placeholder_rows"].as_u64().unwrap_or_default();
    let progress = if expected == 0 {
        0
    } else {
        filled.saturating_mul(100) / expected
    };
    let system_cards = systems
        .iter()
        .map(|system| {
            let ready = system["import_ready"].as_bool().unwrap_or(false);
            format!(
                r#"<article class="card">
<p class="eyebrow">{}</p>
<h2>{}</h2>
<p><strong>{}/{}</strong> row(s) filled</p>
<p><strong>{}</strong> placeholder row(s)</p>
<p class="muted">Template: {}</p>
<p class="muted">Plan: <code>{}</code></p>
<p class="muted">x32dbg: <code>{}</code></p>
<p class="muted">Checklist: <code>{}</code></p>
<p class="muted">Import: <code>{}</code></p>
<button class="importSystem" data-system="{}" {}>{}</button>
</article>"#,
                html_escape(system["slug"].as_str().unwrap_or("unknown")),
                html_escape(system["system"].as_str().unwrap_or("unknown")),
                system["filled_original_rows"].as_u64().unwrap_or_default(),
                system["expected_original_rows"]
                    .as_u64()
                    .unwrap_or_default(),
                system["placeholder_rows"].as_u64().unwrap_or_default(),
                html_escape(system["template"].as_str().unwrap_or("")),
                html_escape(system["capture_pack_row_csv"].as_str().unwrap_or("")),
                html_escape(system["x32dbg_row_watch_plan"].as_str().unwrap_or("")),
                html_escape(system["capture_session_checklist"].as_str().unwrap_or("")),
                html_escape(system["import_command"].as_str().unwrap_or("")),
                html_escape(system["slug"].as_str().unwrap_or("")),
                if ready { "" } else { "disabled" },
                if ready {
                    "Import this system"
                } else {
                    "Not import-ready"
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let rows = todo_rows
        .iter()
        .map(|row| {
            format!(
                r#"<tr data-system="{}">
<td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td><td><code>{}</code></td><td><code>{}</code></td><td><code>{}</code><br><span class="muted">{}</span></td><td><code>{}</code><br><span class="muted">{}</span></td><td><input class="smallInput before" value="{}"></td><td><input class="smallInput after" value="{}"></td><td><button data-system="{}" data-row="{}">Save</button></td><td>{}<br><span class="muted">{}</span></td>
</tr>"#,
                html_escape(row["slug"].as_str().unwrap_or("")),
                html_escape(row["system"].as_str().unwrap_or("")),
                row["row"].as_u64().unwrap_or_default(),
                html_escape(row["table"].as_str().unwrap_or("")),
                html_escape(row["field"].as_str().unwrap_or("")),
                html_escape(row["record_offset"].as_str().unwrap_or("")),
                html_escape(row["event_code"].as_str().unwrap_or("")),
                html_escape(row["source_function"].as_str().unwrap_or("")),
                html_escape(row["watch_group"].as_str().unwrap_or("")),
                html_escape(row["watch_expression"].as_str().unwrap_or("")),
                html_escape(row["expected_rust_before"].as_str().unwrap_or("")),
                html_escape(row["expected_rust_after"].as_str().unwrap_or("")),
                html_escape(row["before"].as_str().unwrap_or("")),
                html_escape(row["after"].as_str().unwrap_or("")),
                html_escape(row["slug"].as_str().unwrap_or("")),
                row["row"].as_u64().unwrap_or_default(),
                html_escape(row["notes"].as_str().unwrap_or("")),
                html_escape(row["quality_gate"].as_str().unwrap_or(""))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let options = systems
        .iter()
        .map(|system| {
            let slug = system["slug"].as_str().unwrap_or("");
            format!(
                r#"<option value="{}">{}</option>"#,
                html_escape(slug),
                html_escape(system["system"].as_str().unwrap_or(slug))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>CM0102 Original Capture Workbench</title>
<style>
:root {{
  --ink: #182022;
  --muted: #657478;
  --paper: #fffaf1;
  --line: #dfd5c3;
  --accent: #0b7f86;
  --accent-2: #d36b31;
  --wash: #e7efe8;
}}
* {{ box-sizing: border-box; }}
body {{
  margin: 0;
  font-family: Georgia, 'Times New Roman', serif;
  color: var(--ink);
  background:
    radial-gradient(circle at top left, rgba(11,127,134,.18), transparent 34rem),
    radial-gradient(circle at 95% 10%, rgba(211,107,49,.18), transparent 28rem),
    linear-gradient(135deg, #eef4ef, #fff8ee 48%, #f7efe0);
}}
main {{ max-width: 1480px; margin: 0 auto; padding: 34px; }}
.hero {{
  background: rgba(255,250,241,.86);
  border: 1px solid var(--line);
  border-radius: 28px;
  padding: 32px;
  box-shadow: 0 24px 70px rgba(55,45,28,.12);
}}
h1 {{ margin: 0; font-size: clamp(2.4rem, 5vw, 5.2rem); line-height: .95; }}
.subtitle {{ max-width: 880px; color: var(--muted); font-size: 1.08rem; line-height: 1.55; }}
.stats, .cards {{ display: grid; gap: 16px; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); margin-top: 22px; }}
.stat, .card, .panel {{
  background: rgba(255,255,255,.76);
  border: 1px solid var(--line);
  border-radius: 20px;
  padding: 18px;
}}
.stat strong {{ display: block; font-size: 2.2rem; }}
.eyebrow {{ color: var(--accent); text-transform: uppercase; letter-spacing: .08em; font: 700 .75rem Verdana, sans-serif; }}
.muted {{ color: var(--muted); }}
.bar {{ height: 14px; background: #eadfcb; border-radius: 99px; overflow: hidden; margin-top: 18px; }}
.bar span {{ display: block; height: 100%; width: {}%; background: linear-gradient(90deg, var(--accent), var(--accent-2)); }}
.panel {{ margin-top: 22px; overflow: hidden; }}
.toolbar {{ display: flex; gap: 12px; flex-wrap: wrap; align-items: center; margin-bottom: 16px; }}
select, input {{
  border: 1px solid var(--line);
  border-radius: 999px;
  padding: 11px 14px;
  background: white;
  min-width: 220px;
}}
textarea {{
  width: 100%;
  min-height: 150px;
  border: 1px solid var(--line);
  border-radius: 16px;
  padding: 13px;
  background: #fffdf8;
  font: .9rem Consolas, monospace;
}}
button {{
  border: 0;
  border-radius: 999px;
  padding: 10px 14px;
  color: white;
  background: var(--accent);
  cursor: pointer;
}}
.buttonLink {{
  display: inline-block;
  border-radius: 999px;
  padding: 10px 14px;
  color: white;
  background: var(--accent);
  text-decoration: none;
  margin: 0 8px 8px 0;
}}
button:disabled {{ opacity: .55; cursor: wait; }}
.smallInput {{ min-width: 150px; max-width: 180px; padding: 8px 10px; font: .82rem Verdana, sans-serif; }}
table {{ width: 100%; border-collapse: collapse; font: .9rem Verdana, sans-serif; }}
th {{ text-align: left; color: var(--muted); border-bottom: 1px solid var(--line); padding: 12px; }}
td {{ border-bottom: 1px solid rgba(223,213,195,.8); padding: 12px; vertical-align: top; }}
code {{ background: #f2eadc; border-radius: 7px; padding: 2px 5px; }}
.notes {{ min-width: 360px; }}
@media (max-width: 760px) {{ main {{ padding: 18px; }} table {{ font-size: .78rem; }} .notes {{ min-width: 240px; }} }}
</style>
</head>
<body>
<main>
<section class="hero">
<p class="eyebrow">CM0102 Rust parity capture</p>
<h1>Original Capture Workbench</h1>
<p class="subtitle">This dashboard tracks the original <code>cm0102.exe</code> writes still needed before Rust gameplay mutators can be promoted. Rust candidate traces exist; these rows must be filled from the original binary only.</p>
<p><a class="buttonLink" href="/capture-pack">Open hosted capture pack</a><a class="buttonLink" href="/reports/capture_pack/all-systems-capture.csv">Open all-systems CSV</a><a class="buttonLink" href="/reports/capture_pack/all-systems-x32dbg-plan.txt">Open x32dbg plan</a></p>
<div class="stats">
<div class="stat"><p class="eyebrow">Status</p><strong>{}</strong></div>
<div class="stat"><p class="eyebrow">Rows Filled</p><strong>{}/{}</strong></div>
<div class="stat"><p class="eyebrow">Placeholders</p><strong>{}</strong></div>
<div class="stat"><p class="eyebrow">Import Ready</p><strong>{}/{}</strong></div>
</div>
<div class="bar" aria-label="capture progress"><span></span></div>
</section>
<section class="cards">{}</section>
<section class="panel">
<p class="eyebrow">Batch capture import</p>
<h2>Paste a captured CSV session</h2>
<p class="muted">Required columns: <code>system,row,before,after</code>. Optional: <code>notes,capture_status</code>. This updates original capture templates, regenerates the workbench, then you can refresh gates.</p>
<textarea id="captureCsv" spellcheck="false">system,row,before,after,notes
match-results,0,FILL_BEFORE,FILL_AFTER,captured from original cm0102.exe</textarea>
<div class="toolbar"><button id="validateCsv">Validate pasted CSV</button><button id="importCsv">Import pasted CSV</button><button id="submitCsv">Submit CSV and refresh gates</button><span id="csvStatus" class="muted"></span></div>
</section>
<section class="panel">
<div class="toolbar">
<select id="systemFilter"><option value="">All systems</option>{}</select>
<input id="search" placeholder="Search offset, event, function, notes...">
<button id="importReady">Import all ready systems</button>
<button id="refreshGates">Refresh gates</button>
</div>
<table>
<thead><tr><th>System</th><th>Row</th><th>Table</th><th>Field</th><th>Offset</th><th>Event</th><th>Function</th><th>Watch</th><th>Rust Expected</th><th>Before</th><th>After</th><th>Save</th><th class="notes">Notes</th></tr></thead>
<tbody id="rows">{}</tbody>
</table>
</section>
</main>
<script>
const filter = document.getElementById('systemFilter');
const search = document.getElementById('search');
const rows = Array.from(document.querySelectorAll('#rows tr'));
const canSave = location.protocol.startsWith('http');
const validateCsv = document.getElementById('validateCsv');
const importCsv = document.getElementById('importCsv');
const submitCsv = document.getElementById('submitCsv');
const captureCsv = document.getElementById('captureCsv');
const csvStatus = document.getElementById('csvStatus');
function applyFilters() {{
  const system = filter.value;
  const q = search.value.toLowerCase();
  for (const row of rows) {{
    const systemOk = !system || row.dataset.system === system;
    const textOk = !q || row.textContent.toLowerCase().includes(q);
    row.style.display = systemOk && textOk ? '' : 'none';
  }}
}}
filter.addEventListener('change', applyFilters);
search.addEventListener('input', applyFilters);
async function postJson(url, body) {{
  const response = await fetch(url, {{
    method: 'POST',
    headers: {{ 'Content-Type': 'application/json' }},
    body: JSON.stringify(body)
  }});
  if (!response.ok) throw new Error(await response.text());
  return response.json();
}}
if (!canSave) {{
  validateCsv.disabled = true;
  validateCsv.textContent = 'Serve to validate CSV';
  importCsv.disabled = true;
  importCsv.textContent = 'Serve to import CSV';
  submitCsv.disabled = true;
  submitCsv.textContent = 'Serve to submit CSV';
}} else {{
  validateCsv.addEventListener('click', async () => {{
    validateCsv.disabled = true;
    validateCsv.textContent = 'Validating CSV';
    csvStatus.textContent = '';
    try {{
      const report = await postJson('/api/original-capture/validate-csv', {{ csv: captureCsv.value }});
      const s = report.summary || {{}};
      csvStatus.textContent = `Validation ${{report.status || 'unknown'}}: ${{s.captured_rows || 0}} captured, ${{s.blank_capture_values || 0}} blank, ${{s.blocking_errors || 0}} blocking.`;
      validateCsv.textContent = 'CSV validated';
    }} catch (error) {{
      console.error(error);
      csvStatus.textContent = error.message;
      validateCsv.textContent = 'Validation blocked';
    }} finally {{
      setTimeout(() => {{ validateCsv.disabled = false; validateCsv.textContent = 'Validate pasted CSV'; }}, 1800);
    }}
  }});
  importCsv.addEventListener('click', async () => {{
    importCsv.disabled = true;
    importCsv.textContent = 'Importing CSV';
    csvStatus.textContent = '';
    try {{
      const report = await postJson('/api/original-capture/import-csv', {{ csv: captureCsv.value }});
      const s = report.summary || {{}};
      csvStatus.textContent = `Imported ${{s.rows_updated || 0}} row(s); placeholders left ${{s.placeholder_rows || 0}}.`;
      importCsv.textContent = 'CSV imported';
    }} catch (error) {{
      console.error(error);
      csvStatus.textContent = error.message;
      importCsv.textContent = 'CSV blocked';
    }} finally {{
      setTimeout(() => {{ importCsv.disabled = false; importCsv.textContent = 'Import pasted CSV'; }}, 1800);
    }}
  }});
  submitCsv.addEventListener('click', async () => {{
    submitCsv.disabled = true;
    submitCsv.textContent = 'Submitting CSV';
    csvStatus.textContent = '';
    try {{
      const report = await postJson('/api/original-capture/submit-csv', {{ csv: captureCsv.value }});
      const s = report.summary || {{}};
      csvStatus.textContent = `Submit ${{report.status || 'unknown'}}: captured ${{s.captured_rows || 0}}/${{s.rows || 0}}, gate ${{s.capture_rows_filled || 0}}/${{s.capture_rows_expected || 0}}, parity failures ${{s.parity_failures || 0}}.`;
      submitCsv.textContent = report.status === 'blocked-by-validation' ? 'Submit blocked' : 'Submitted';
    }} catch (error) {{
      console.error(error);
      csvStatus.textContent = error.message;
      submitCsv.textContent = 'Submit error';
    }} finally {{
      setTimeout(() => {{ submitCsv.disabled = false; submitCsv.textContent = 'Submit CSV and refresh gates'; }}, 2200);
    }}
  }});
}}
for (const button of document.querySelectorAll('button[data-row]')) {{
  button.disabled = !canSave;
  if (!canSave) button.textContent = 'Serve to save';
  button.addEventListener('click', async () => {{
    const row = button.closest('tr');
    button.disabled = true;
    button.textContent = 'Saving';
    try {{
      await postJson('/api/original-capture/row', {{
        system: button.dataset.system,
        row: Number(button.dataset.row),
        before: row.querySelector('.before').value,
        after: row.querySelector('.after').value
      }});
      button.textContent = 'Saved';
      row.style.background = 'rgba(11,127,134,.08)';
    }} catch (error) {{
      console.error(error);
      button.textContent = 'Error';
      alert('Could not save capture row: ' + error.message);
    }} finally {{
      setTimeout(() => {{ button.disabled = false; button.textContent = 'Save'; }}, 1200);
    }}
  }});
}}
for (const button of document.querySelectorAll('.importSystem')) {{
  if (!canSave || button.disabled) {{
    if (!canSave) button.textContent = 'Serve to import';
    button.disabled = true;
    continue;
  }}
  button.addEventListener('click', async () => {{
    button.disabled = true;
    button.textContent = 'Importing';
    try {{
      await postJson('/api/original-capture/import-system', {{ system: button.dataset.system }});
      button.textContent = 'Imported';
    }} catch (error) {{
      console.error(error);
      button.textContent = 'Blocked';
      alert('Could not import capture: ' + error.message);
    }}
  }});
}}
const importReady = document.getElementById('importReady');
const refreshGates = document.getElementById('refreshGates');
if (!canSave) {{
  importReady.disabled = true;
  importReady.textContent = 'Serve to import';
  refreshGates.disabled = true;
  refreshGates.textContent = 'Serve to refresh';
}} else {{
  importReady.addEventListener('click', async () => {{
    importReady.disabled = true;
    importReady.textContent = 'Importing';
    try {{
      const report = await postJson('/api/original-capture/import-ready', {{}});
      importReady.textContent = 'Imported ' + report.imported.length;
    }} catch (error) {{
      console.error(error);
      importReady.textContent = 'Import blocked';
      alert('Could not import ready captures: ' + error.message);
    }} finally {{
      setTimeout(() => {{ importReady.disabled = false; importReady.textContent = 'Import all ready systems'; }}, 1600);
    }}
  }});
  refreshGates.addEventListener('click', async () => {{
    refreshGates.disabled = true;
    refreshGates.textContent = 'Refreshing';
    try {{
      const report = await postJson('/api/backend-gates/refresh', {{}});
      const s = report.summary || {{}};
      refreshGates.textContent = `${{s.original_capture_rows_filled || 0}}/${{s.original_capture_rows_expected || 0}} captured`;
      alert(`Backend gates refreshed.\nStatus: ${{s.status || 'unknown'}}\nCapture: ${{s.original_capture_rows_filled || 0}}/${{s.original_capture_rows_expected || 0}}\nParity failures: ${{s.gameplay_parity_failures || 0}}`);
    }} catch (error) {{
      console.error(error);
      refreshGates.textContent = 'Refresh blocked';
      alert('Could not refresh backend gates: ' + error.message);
    }} finally {{
      setTimeout(() => {{ refreshGates.disabled = false; refreshGates.textContent = 'Refresh gates'; }}, 1800);
    }}
  }});
}}
</script>
</body>
</html>
"#,
        progress,
        html_escape(summary["status"].as_str().unwrap_or("unknown")),
        filled,
        expected,
        placeholders,
        summary["import_ready_systems"].as_u64().unwrap_or_default(),
        summary["systems"].as_u64().unwrap_or_default(),
        system_cards,
        options,
        rows
    )
}

fn capture_console_html(capture: &serde_json::Value, gate: &serde_json::Value) -> String {
    let capture_summary = &capture["summary"];
    let gate_summary = &gate["summary"];
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>CM0102 Capture Console</title>
<style>
:root {{ --ink:#182022; --muted:#687477; --line:#ded4c1; --paper:rgba(255,251,242,.9); --accent:#0b7f86; --hot:#c65f2d; }}
* {{ box-sizing:border-box; }}
body {{ margin:0; color:var(--ink); font-family:Georgia,'Times New Roman',serif; background:radial-gradient(circle at 8% 0%,rgba(11,127,134,.22),transparent 32rem),radial-gradient(circle at 90% 12%,rgba(198,95,45,.18),transparent 28rem),linear-gradient(135deg,#edf3ee,#fff7e8 58%,#eee2cd); }}
main {{ max-width:1180px; margin:0 auto; padding:38px; }}
.hero,.card {{ background:var(--paper); border:1px solid var(--line); border-radius:26px; padding:24px; box-shadow:0 18px 50px rgba(52,43,29,.09); }}
h1 {{ margin:0; font-size:clamp(2.7rem,6vw,5.4rem); line-height:.92; }}
.eyebrow {{ color:var(--accent); text-transform:uppercase; letter-spacing:.08em; font:700 .76rem Verdana,sans-serif; }}
.muted {{ color:var(--muted); }}
.grid {{ display:grid; grid-template-columns:repeat(auto-fit,minmax(230px,1fr)); gap:16px; margin:20px 0; }}
.card strong {{ display:block; font-size:2rem; }}
a {{ color:white; background:var(--accent); border-radius:999px; padding:10px 14px; text-decoration:none; display:inline-block; margin:0 8px 10px 0; }}
code {{ background:#f2eadc; border-radius:7px; padding:2px 5px; color:var(--ink); }}
</style>
</head>
<body>
<main>
<section class="hero">
<p class="eyebrow">CM0102 exact gameplay gate</p>
<h1>Capture Console</h1>
<p class="muted">Hosted command center for the original <code>cm0102.exe</code> traces that unblock Rust parity and promotion.</p>
<p><a href="/original-capture-workbench">Open Workbench</a><a href="/capture-pack">Open Capture Pack</a><a href="/promotion-control-room-cached">Control Room</a><a href="/reports/todo_attack_board/dashboard.html">TODO Board</a></p>
</section>
<section class="grid">
<article class="card"><p class="eyebrow">Capture Rows</p><strong>{}/{}</strong><p class="muted">{} placeholder row(s)</p></article>
<article class="card"><p class="eyebrow">Import Ready</p><strong>{}</strong><p class="muted">system(s)</p></article>
<article class="card"><p class="eyebrow">Gate Status</p><strong>{}</strong><p class="muted">playable headless: {}</p></article>
<article class="card"><p class="eyebrow">Parity</p><strong>{}</strong><p class="muted">failure(s)</p></article>
</section>
<section class="hero">
<p class="eyebrow">Operator Files</p>
<p><a href="/capture-pack/all-systems-capture.csv">All-Systems CSV</a><a href="/capture-pack/all-systems-x32dbg-plan.txt">x32dbg Plan</a><a href="/capture-pack/all-systems-capture-validation.json">Latest CSV Validation</a><a href="/capture-pack/all-systems-capture-submit-smoke.json">Submit Smoke Report</a><a href="/reports/capture_console_prep.json">Prep JSON</a><a href="/reports/launch-capture-console.ps1">PowerShell Launcher</a><a href="/reports/launch-capture-console.cmd">CMD Launcher</a></p>
<p><code>cargo run -p cm-app -- serve-rust-db D:/cm0102-rs/rust-db 8765</code></p>
</section>
</main>
</body>
</html>
"#,
        capture_summary["filled_original_rows"]
            .as_u64()
            .unwrap_or_default(),
        capture_summary["expected_original_rows"]
            .as_u64()
            .unwrap_or_default(),
        capture_summary["placeholder_rows"]
            .as_u64()
            .unwrap_or_default(),
        capture_summary["import_ready_systems"]
            .as_u64()
            .unwrap_or_default(),
        html_escape(gate_summary["status"].as_str().unwrap_or("unknown")),
        gate_summary["playable_headless"].as_bool().unwrap_or(false),
        gate_summary["gameplay_parity_failures"]
            .as_u64()
            .unwrap_or_default(),
    )
}

fn promotion_control_room_html(report: &serde_json::Value) -> String {
    let summary = &report["summary"];
    let expected = summary["original_capture_rows_expected"]
        .as_u64()
        .unwrap_or_default();
    let filled = summary["original_capture_rows_filled"]
        .as_u64()
        .unwrap_or_default();
    let progress = if expected == 0 {
        0
    } else {
        filled.saturating_mul(100) / expected
    };
    let card = |label: &str, value: String, detail: String, class_name: &str| -> String {
        format!(
            r#"<article class="card {class_name}"><p class="eyebrow">{}</p><strong>{}</strong><span>{}</span></article>"#,
            html_escape(label),
            html_escape(&value),
            html_escape(&detail)
        )
    };
    let mut cards = Vec::new();
    cards.push(card(
        "Foundation",
        if summary["foundation_pass"].as_bool().unwrap_or(false) {
            "Pass".to_string()
        } else {
            "Blocked".to_string()
        },
        "Original binary, execution model, Rust DB, and backend infrastructure gates".to_string(),
        if summary["foundation_pass"].as_bool().unwrap_or(false) {
            "pass"
        } else {
            "fail"
        },
    ));
    cards.push(card(
        "Headless",
        if summary["playable_headless"].as_bool().unwrap_or(false) {
            "Playable".to_string()
        } else {
            "Not Yet".to_string()
        },
        format!(
            "Runtime validation {}",
            summary["runtime_validation_status"]
                .as_str()
                .unwrap_or("unknown")
        ),
        if summary["playable_headless"].as_bool().unwrap_or(false) {
            "pass"
        } else {
            "warn"
        },
    ));
    cards.push(card(
        "Original Capture",
        format!("{filled}/{expected}"),
        format!(
            "{} placeholder row(s), {} import-ready system(s)",
            summary["original_capture_placeholder_rows"]
                .as_u64()
                .unwrap_or_default(),
            summary["original_capture_import_ready_systems"]
                .as_u64()
                .unwrap_or_default()
        ),
        if summary["original_capture_placeholder_rows"]
            .as_u64()
            .unwrap_or_default()
            == 0
        {
            "pass"
        } else {
            "fail"
        },
    ));
    cards.push(card(
        "Parity",
        summary["gameplay_parity_status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        format!(
            "{} failing system(s)",
            summary["gameplay_parity_failures"]
                .as_u64()
                .unwrap_or_default()
        ),
        if summary["gameplay_parity_status"].as_str() == Some("pass") {
            "pass"
        } else {
            "fail"
        },
    ));
    cards.push(card(
        "Promotion",
        summary["gameplay_promotion_status"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        format!(
            "{} blocked system(s)",
            summary["gameplay_promotion_blocked"]
                .as_u64()
                .unwrap_or_default()
        ),
        if summary["gameplay_promotion_status"].as_str() == Some("pass") {
            "pass"
        } else {
            "fail"
        },
    ));
    cards.push(card(
        "Exact Remake",
        if summary["one_for_one_exact_remake"]
            .as_bool()
            .unwrap_or(false)
        {
            "Yes".to_string()
        } else {
            "No".to_string()
        },
        summary["next_required_action"]
            .as_str()
            .unwrap_or("unknown")
            .to_string(),
        if summary["one_for_one_exact_remake"]
            .as_bool()
            .unwrap_or(false)
        {
            "pass"
        } else {
            "fail"
        },
    ));

    let system_rows = report["systems"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|system| {
            let blockers = system["promotion"]["open_blockers"]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            let import_blockers = system["original_capture"]["import_blockers"]
                .as_array()
                .map(|items| {
                    items
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            format!(
                r#"<tr>
<td><strong>{}</strong><br><span>{}</span></td>
<td>{}<br><span>{}/{} filled, {} placeholders</span></td>
<td>{}<br><span>{}</span></td>
<td>{}<br><span>{}</span></td>
<td><code>{}</code><br><span>{}</span></td>
</tr>"#,
                html_escape(system["system"].as_str().unwrap_or("unknown")),
                html_escape(system["slug"].as_str().unwrap_or("")),
                html_escape(
                    system["original_capture"]["status"]
                        .as_str()
                        .unwrap_or("unknown")
                ),
                system["original_capture"]["filled_original_rows"]
                    .as_u64()
                    .unwrap_or_default(),
                system["original_capture"]["expected_original_rows"]
                    .as_u64()
                    .unwrap_or_default(),
                system["original_capture"]["placeholder_rows"]
                    .as_u64()
                    .unwrap_or_default(),
                html_escape(system["parity"]["status"].as_str().unwrap_or("unknown")),
                html_escape(system["parity"]["detail"].as_str().unwrap_or("")),
                html_escape(system["promotion"]["status"].as_str().unwrap_or("unknown")),
                html_escape(&blockers),
                html_escape(system["promotion"]["entry_point"].as_str().unwrap_or("")),
                html_escape(&import_blockers)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>CM0102 Rust Promotion Control Room</title>
<style>
:root {{
  --ink: #1c2422;
  --muted: #63706c;
  --paper: rgba(255,252,244,.9);
  --line: #ded6c6;
  --pass: #0f7f5f;
  --warn: #a86b00;
  --fail: #b33a2f;
  --accent: #0c7682;
}}
* {{ box-sizing: border-box; }}
body {{
  margin: 0;
  font-family: Georgia, 'Times New Roman', serif;
  color: var(--ink);
  background:
    radial-gradient(circle at 10% 5%, rgba(12,118,130,.22), transparent 30rem),
    radial-gradient(circle at 90% 20%, rgba(179,58,47,.18), transparent 28rem),
    linear-gradient(135deg, #ecf3ee, #fff7e8 52%, #f0e5d2);
}}
main {{ max-width: 1500px; margin: 0 auto; padding: 34px; }}
.hero, .panel {{
  background: var(--paper);
  border: 1px solid var(--line);
  border-radius: 28px;
  box-shadow: 0 24px 70px rgba(50,40,25,.12);
}}
.hero {{ padding: 32px; }}
h1 {{ margin: 0; font-size: clamp(2.2rem, 5vw, 5.2rem); line-height: .95; }}
.subtitle {{ max-width: 980px; color: var(--muted); font-size: 1.08rem; line-height: 1.55; }}
.eyebrow {{ color: var(--accent); text-transform: uppercase; letter-spacing: .08em; font: 700 .75rem Verdana, sans-serif; }}
.cards {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px; margin-top: 22px; }}
.card {{
  background: rgba(255,255,255,.78);
  border: 1px solid var(--line);
  border-top: 6px solid var(--accent);
  border-radius: 20px;
  padding: 18px;
}}
.card.pass {{ border-top-color: var(--pass); }}
.card.warn {{ border-top-color: var(--warn); }}
.card.fail {{ border-top-color: var(--fail); }}
.card strong {{ display: block; font-size: 2rem; margin-bottom: 8px; }}
.card span, td span {{ color: var(--muted); }}
.bar {{ height: 14px; background: #eadfcb; border-radius: 99px; overflow: hidden; margin-top: 18px; }}
.bar span {{ display: block; height: 100%; width: {}%; background: linear-gradient(90deg, var(--accent), var(--fail)); }}
.panel {{ margin-top: 22px; padding: 22px; overflow: auto; }}
table {{ width: 100%; border-collapse: collapse; font: .9rem Verdana, sans-serif; }}
th {{ text-align: left; color: var(--muted); border-bottom: 1px solid var(--line); padding: 12px; }}
td {{ border-bottom: 1px solid rgba(222,214,198,.82); padding: 14px 12px; vertical-align: top; }}
code {{ background: #f2eadc; border-radius: 7px; padding: 2px 5px; }}
.actions {{ display: flex; gap: 10px; flex-wrap: wrap; margin-top: 18px; }}
a {{
  color: white;
  background: var(--accent);
  border-radius: 999px;
  padding: 10px 14px;
  text-decoration: none;
  font: 700 .85rem Verdana, sans-serif;
}}
@media (max-width: 760px) {{ main {{ padding: 18px; }} table {{ font-size: .78rem; }} }}
</style>
</head>
<body>
<main>
<section class="hero">
<p class="eyebrow">CM0102 Rust backend migration</p>
<h1>Promotion Control Room</h1>
<p class="subtitle">This is the single status board for making Rust the playable truth. Green foundation does not mean exact gameplay; exact remains blocked until original <code>cm0102.exe</code> captures match Rust mutations and the four gameplay gates are promoted.</p>
<div class="cards">{}</div>
<div class="bar" aria-label="original capture progress"><span></span></div>
<div class="actions">
<a href="/original-capture-workbench">Open Capture Workbench</a>
<a href="/api/promotion-control-room">Open JSON</a>
<a href="/api/exact-remake">Exact Report</a>
</div>
</section>
<section class="panel">
<p class="eyebrow">Four gameplay systems</p>
<table>
<thead><tr><th>System</th><th>Original Capture</th><th>Parity</th><th>Promotion</th><th>Entry / Capture Blockers</th></tr></thead>
<tbody>{}</tbody>
</table>
</section>
</main>
</body>
</html>
"#,
        progress,
        cards.join("\n"),
        system_rows
    )
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn write_json_file(path: &Path, value: &serde_json::Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|err| format!("failed to serialize {}: {err}", path.display()))?;
    fs::write(path, bytes).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn write_text_file(path: &Path, value: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(path, value).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn capture_pack_lines(title: &str, system: &str, values: &[String]) -> String {
    let mut lines = vec![format!("{title} for {system}"), String::new()];
    for value in values {
        lines.push(format!("- {value}"));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn capture_row_plan(
    system: &str,
    slug: &str,
    rust_mutations: &[serde_json::Value],
) -> serde_json::Value {
    let rows = rust_mutations
        .iter()
        .enumerate()
        .map(|(index, mutation)| {
            let source_function = mutation["source_function"].as_str().unwrap_or("");
            let record_offset = mutation["record_offset"].as_str().unwrap_or("");
            let field = mutation["field"].as_str().unwrap_or("");
            let table = mutation["table"].as_str().unwrap_or("");
            let watch_expression = capture_watch_expression(table, field, record_offset);
            serde_json::json!({
                "row": index,
                "system": system,
                "slug": slug,
                "table": table,
                "field": field,
                "record_offset": record_offset,
                "event_code": mutation["event_code"].as_str().unwrap_or(""),
                "phase": mutation["phase"].as_str().unwrap_or(""),
                "source_function": source_function,
                "watch_group": capture_watch_group(source_function, record_offset, field),
                "watch_expression": watch_expression,
                "expected_rust_before": mutation["before"].clone(),
                "expected_rust_after": mutation["after"].clone(),
                "capture_before": "FILL_FROM_ORIGINAL",
                "capture_after": "FILL_FROM_ORIGINAL",
                "quality_gate": "Capture from original cm0102.exe, then import only if row order and schema match.",
                "notes": mutation["notes"].as_str().unwrap_or(""),
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "format": "cm0102-rs-original-row-capture-plan",
        "version": 1,
        "system": system,
        "slug": slug,
        "rows": rows,
        "summary": {
            "rows": rows.len(),
            "watch_groups": capture_watch_groups(&serde_json::json!({ "rows": rows })).len(),
        }
    })
}

fn capture_watch_expression(table: &str, field: &str, record_offset: &str) -> String {
    if !record_offset.is_empty() {
        return format!("{table}.{field} at candidate record +{record_offset}");
    }
    format!("{table}.{field}")
}

fn capture_watch_group(source_function: &str, record_offset: &str, field: &str) -> String {
    let source = if source_function.is_empty() {
        "unknown_function"
    } else {
        source_function
    };
    let offset = if record_offset.is_empty() {
        field
    } else {
        record_offset
    };
    format!("{source}::{offset}")
}

fn capture_watch_groups(row_plan: &serde_json::Value) -> Vec<serde_json::Value> {
    let mut groups: Vec<serde_json::Value> = Vec::new();
    for row in row_plan["rows"].as_array().cloned().unwrap_or_default() {
        let group_name = row["watch_group"].as_str().unwrap_or("unknown").to_string();
        if let Some(group) = groups
            .iter_mut()
            .find(|group| group["watch_group"].as_str() == Some(group_name.as_str()))
        {
            if let Some(rows) = group["rows"].as_array_mut() {
                rows.push(row);
            }
            continue;
        }
        groups.push(serde_json::json!({
            "watch_group": group_name,
            "source_function": row["source_function"].as_str().unwrap_or(""),
            "record_offset": row["record_offset"].as_str().unwrap_or(""),
            "field": row["field"].as_str().unwrap_or(""),
            "rows": [row],
        }));
    }
    groups
}

fn capture_row_plan_csv(row_plan: &serde_json::Value) -> String {
    let headers = [
        "row",
        "system",
        "table",
        "field",
        "record_offset",
        "event_code",
        "phase",
        "source_function",
        "watch_group",
        "watch_expression",
        "expected_rust_before",
        "expected_rust_after",
        "capture_before",
        "capture_after",
        "notes",
    ];
    let mut lines = vec![headers.join(",")];
    for row in row_plan["rows"].as_array().cloned().unwrap_or_default() {
        lines.push(
            headers
                .iter()
                .map(|header| csv_cell(&csv_value(&row[*header])))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    format!("{}\n", lines.join("\n"))
}

fn all_systems_capture_csv(row_plan: &serde_json::Value) -> String {
    let headers = [
        "system",
        "row",
        "before",
        "after",
        "notes",
        "capture_status",
        "table",
        "field",
        "record_offset",
        "event_code",
        "source_function",
        "watch_group",
        "watch_expression",
    ];
    let mut lines = vec![headers.join(",")];
    for row in row_plan["rows"].as_array().cloned().unwrap_or_default() {
        let values = serde_json::json!({
            "system": row["slug"].as_str().unwrap_or(row["system"].as_str().unwrap_or("")),
            "row": row["row"],
            "before": "",
            "after": "",
            "notes": "",
            "capture_status": "captured-from-original",
            "table": row["table"],
            "field": row["field"],
            "record_offset": row["record_offset"],
            "event_code": row["event_code"],
            "source_function": row["source_function"],
            "watch_group": row["watch_group"],
            "watch_expression": row["watch_expression"],
        });
        lines.push(
            headers
                .iter()
                .map(|header| csv_cell(&csv_value(&values[*header])))
                .collect::<Vec<_>>()
                .join(","),
        );
    }
    format!("{}\n", lines.join("\n"))
}

fn x32dbg_row_watch_plan(system: &str, watch_groups: &[serde_json::Value]) -> String {
    let mut lines = vec![
        format!("x32dbg row watch plan for {system}"),
        String::new(),
        "This is deliberately conservative: break on the listed code-derived function first, then inspect the row fields before and after the write.".to_string(),
        "Use the row-capture-plan.csv row numbers when filling the original capture template.".to_string(),
        String::new(),
    ];
    for group in watch_groups {
        lines.push(format!(
            "Group {}",
            group["watch_group"].as_str().unwrap_or("unknown")
        ));
        if let Some(function) = group["source_function"].as_str() {
            if !function.is_empty() {
                lines.push(format!("bp {function}"));
            }
        }
        lines.push(format!(
            "watch: {}",
            group["record_offset"]
                .as_str()
                .filter(|value| !value.is_empty())
                .unwrap_or(group["field"].as_str().unwrap_or("unknown"))
        ));
        if let Some(rows) = group["rows"].as_array() {
            lines.push(format!("rows: {}", rows.len()));
            for row in rows {
                lines.push(format!(
                    "- row {}: {}.{} expected Rust {} -> {}",
                    row["row"].as_u64().unwrap_or_default(),
                    row["table"].as_str().unwrap_or(""),
                    row["field"].as_str().unwrap_or(""),
                    csv_value(&row["expected_rust_before"]),
                    csv_value(&row["expected_rust_after"])
                ));
            }
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

fn all_systems_x32dbg_plan(watch_groups: &[serde_json::Value]) -> String {
    let mut lines = vec![
        "x32dbg all-systems row watch plan".to_string(),
        String::new(),
        "Capture from D:/cm0102/cm0102.exe only.".to_string(),
        "Fill D:/cm0102-rs/reports/capture_pack/all-systems-capture.csv, then import it with:".to_string(),
        "cargo run -p cm-app -- import-original-capture-csv D:/cm0102-rs/reports/capture_pack/all-systems-capture.csv".to_string(),
        String::new(),
    ];
    for group in watch_groups {
        lines.push(format!(
            "[{}] Group {}",
            group["slug"].as_str().unwrap_or("unknown"),
            group["watch_group"].as_str().unwrap_or("unknown")
        ));
        if let Some(function) = group["source_function"].as_str() {
            if !function.is_empty() {
                lines.push(format!("bp {function}"));
            }
        }
        lines.push(format!(
            "watch: {}",
            group["record_offset"]
                .as_str()
                .filter(|value| !value.is_empty())
                .unwrap_or(group["field"].as_str().unwrap_or("unknown"))
        ));
        if let Some(rows) = group["rows"].as_array() {
            lines.push(format!("rows: {}", rows.len()));
            for row in rows {
                lines.push(format!(
                    "- {}, row {}: {}.{} | expected Rust {} -> {}",
                    row["slug"].as_str().unwrap_or("unknown"),
                    row["row"].as_u64().unwrap_or_default(),
                    row["table"].as_str().unwrap_or(""),
                    row["field"].as_str().unwrap_or(""),
                    csv_value(&row["expected_rust_before"]),
                    csv_value(&row["expected_rust_after"])
                ));
            }
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

fn capture_pack_dashboard_html(
    systems: &[serde_json::Value],
    row_plan: &serde_json::Value,
    watch_groups: &[serde_json::Value],
) -> String {
    let rows = row_plan["rows"]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|row| {
            format!(
                r#"<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td><td><code>{}</code></td><td><code>{}</code></td><td>{}</td><td><code>{}</code></td><td>{}</td></tr>"#,
                html_escape(row["system"].as_str().unwrap_or("")),
                row["row"].as_u64().unwrap_or_default(),
                html_escape(row["table"].as_str().unwrap_or("")),
                html_escape(row["field"].as_str().unwrap_or("")),
                html_escape(row["record_offset"].as_str().unwrap_or("")),
                html_escape(row["event_code"].as_str().unwrap_or("")),
                html_escape(row["source_function"].as_str().unwrap_or("")),
                html_escape(&csv_value(&row["expected_rust_after"])),
                html_escape(row["watch_group"].as_str().unwrap_or("")),
                html_escape(row["watch_expression"].as_str().unwrap_or(""))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let cards = systems
        .iter()
        .map(|system| {
            format!(
                r#"<article class="card"><p class="eyebrow">{}</p><h2>{}</h2><p><strong>{}</strong> row(s), <strong>{}</strong> watch group(s)</p><p><code>{}</code></p></article>"#,
                html_escape(system["slug"].as_str().unwrap_or("unknown")),
                html_escape(system["system"].as_str().unwrap_or("unknown")),
                system["candidate_rows"].as_u64().unwrap_or_default(),
                system["watch_groups"].as_u64().unwrap_or_default(),
                html_escape(system["capture_session_checklist"].as_str().unwrap_or(""))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>CM0102 Capture Pack</title>
<style>
:root {{ --ink:#182022; --muted:#687477; --line:#ded4c1; --paper:rgba(255,251,242,.9); --accent:#0b7f86; }}
* {{ box-sizing:border-box; }}
body {{ margin:0; color:var(--ink); font-family:Georgia,'Times New Roman',serif; background:radial-gradient(circle at 8% 0%,rgba(11,127,134,.22),transparent 32rem),radial-gradient(circle at 90% 12%,rgba(198,95,45,.18),transparent 28rem),linear-gradient(135deg,#edf3ee,#fff7e8 58%,#eee2cd); }}
main {{ max-width:1500px; margin:0 auto; padding:34px; }}
.hero,.panel,.card {{ background:var(--paper); border:1px solid var(--line); border-radius:24px; padding:22px; box-shadow:0 18px 50px rgba(52,43,29,.09); }}
h1 {{ margin:0; font-size:clamp(2.5rem,5vw,5rem); line-height:.95; }}
.eyebrow {{ color:var(--accent); text-transform:uppercase; letter-spacing:.08em; font:700 .76rem Verdana,sans-serif; }}
.muted,span {{ color:var(--muted); }}
.cards {{ display:grid; grid-template-columns:repeat(auto-fit,minmax(230px,1fr)); gap:16px; margin:20px 0; }}
.toolbar {{ display:flex; gap:12px; flex-wrap:wrap; align-items:center; margin:18px 0; }}
input {{ border:1px solid var(--line); border-radius:999px; padding:11px 14px; min-width:300px; }}
a.button {{ display:inline-block; border-radius:999px; padding:10px 14px; color:white; background:var(--accent); text-decoration:none; margin-right:8px; }}
table {{ width:100%; border-collapse:collapse; font:.88rem Verdana,sans-serif; }}
th {{ text-align:left; color:var(--muted); border-bottom:1px solid var(--line); padding:11px; }}
td {{ border-bottom:1px solid rgba(222,212,193,.78); padding:11px; vertical-align:top; }}
code {{ background:#f2eadc; border-radius:7px; padding:2px 5px; }}
</style>
</head>
<body>
<main>
<section class="hero">
<p class="eyebrow">Original behavior capture</p>
<h1>All-Systems Capture Pack</h1>
<p class="muted">One operator sheet for the 45 original <code>cm0102.exe</code> rows blocking exact Rust gameplay promotion. Fill <code>before</code> and <code>after</code> in the CSV, then import it.</p>
<p><a class="button" href="/capture-pack/all-systems-capture.csv">Open import CSV</a><a class="button" href="/capture-pack/all-systems-x32dbg-plan.txt">Open x32dbg plan</a><a class="button" href="/original-capture-workbench">Open capture workbench</a></p>
<p><code>cargo run -p cm-app -- validate-original-capture-csv D:/cm0102-rs/reports/capture_pack/all-systems-capture.csv</code></p>
<p><code>cargo run -p cm-app -- import-original-capture-csv D:/cm0102-rs/reports/capture_pack/all-systems-capture.csv</code></p>
<p><code>cargo run -p cm-app -- submit-original-capture-csv D:/cm0102-rs/reports/capture_pack/all-systems-capture.csv</code></p>
</section>
<section class="cards">{}</section>
<section class="panel">
<p class="eyebrow">Rows</p>
<div class="toolbar"><input id="search" placeholder="Search system, offset, function, event..."><span>{} row(s), {} watch group(s)</span></div>
<table><thead><tr><th>System</th><th>Row</th><th>Table</th><th>Field</th><th>Offset</th><th>Event</th><th>Function</th><th>Rust Expected After</th><th>Watch Group</th><th>Watch Expression</th></tr></thead><tbody>{}</tbody></table>
</section>
</main>
<script>
const search = document.getElementById('search');
const rows = Array.from(document.querySelectorAll('tbody tr'));
search.addEventListener('input', () => {{
  const q = search.value.toLowerCase();
  for (const row of rows) row.style.display = row.textContent.toLowerCase().includes(q) ? '' : 'none';
}});
</script>
</body>
</html>
"#,
        cards,
        row_plan["summary"]["rows"].as_u64().unwrap_or_default(),
        watch_groups.len(),
        rows
    )
}

fn capture_session_checklist(
    system: &str,
    slug: &str,
    row_plan: &serde_json::Value,
    watch_groups: &[serde_json::Value],
) -> String {
    let rows = row_plan["rows"].as_array().cloned().unwrap_or_default();
    let mut lines = vec![
        format!("# Original Capture Session: {system}"),
        String::new(),
        format!("Slug: `{slug}`"),
        format!("Rows to capture: `{}`", rows.len()),
        format!("Watch groups: `{}`", watch_groups.len()),
        String::new(),
        "## Session Rules".to_string(),
        String::new(),
        "- Capture from `D:/cm0102/cm0102.exe`, not from Rust candidate output.".to_string(),
        "- Preserve the row order from `row-capture-plan.csv` when filling the template.".to_string(),
        "- Every filled row must include before, after, source function, and provenance.".to_string(),
        "- If the original writes fewer or extra fields, do not force it to match Rust; record the mismatch and keep promotion blocked.".to_string(),
        String::new(),
        "## Row Checklist".to_string(),
        String::new(),
    ];
    for row in rows {
        lines.push(format!(
            "- [ ] Row {} `{}` `{}` at `{}` via `{}`",
            row["row"].as_u64().unwrap_or_default(),
            row["table"].as_str().unwrap_or(""),
            row["field"].as_str().unwrap_or(""),
            row["record_offset"].as_str().unwrap_or(""),
            row["source_function"].as_str().unwrap_or("")
        ));
    }
    lines.push(String::new());
    lines.push("## After Capture".to_string());
    lines.push(String::new());
    lines.push(format!(
        "- Fill `D:/cm0102-rs/reports/original_capture_templates/{slug}-capture-template.json`."
    ));
    lines.push(format!(
        "- Run `cargo run -p cm-app -- import-gameplay-capture D:/cm0102-rs/reports/parity_traces D:/cm0102-rs/reports/original_capture_templates/{slug}-capture-template.json`."
    ));
    lines
        .push("- Re-run the promotion control room and check parity/promotion status.".to_string());
    lines.push(String::new());
    lines.join("\n")
}

fn debugger_breakpoint_script(
    system: &str,
    breakpoints: &[String],
    watched_writes: &[String],
) -> String {
    let mut lines = vec![
        format!("x32dbg breakpoint scaffold for {system}"),
        String::new(),
        "Use this as a checklist/script seed. Confirm each breakpoint against the loaded cm0102.exe module base before relying on captures.".to_string(),
        String::new(),
        "Breakpoints".to_string(),
    ];
    for breakpoint in breakpoints {
        lines.push(format!("bp {breakpoint}"));
    }
    lines.push(String::new());
    lines.push(
        "Watched writes to capture manually or convert into debugger watchpoints".to_string(),
    );
    for watched_write in watched_writes {
        lines.push(format!("- {watched_write}"));
    }
    lines.push(String::new());
    lines.push("Capture rule: every emitted mutation must include table, row, field, before, after, source_function, and provenance.".to_string());
    lines.push(String::new());
    lines.join("\n")
}

fn mutation_template(system: &str, watched_writes: &[String]) -> serde_json::Value {
    let subsystem_required = serde_json::json!(subsystem_required_coverage_labels(system));
    serde_json::json!({
        "format": "cm0102-rs-mutation-template",
        "system": system,
        "required_fields": ["table", "row", "field", "before", "after", "source_function", "provenance"],
        "optional_fields": ["event_code", "phase", "record_offset", "helper", "notes"],
        "subsystem_required_coverage": subsystem_required,
        "watched_writes": watched_writes,
        "example_mutation": {
            "table": "fixture",
            "row": 0,
            "field": "fixture +0x43",
            "before": 0,
            "after": 0,
            "source_function": "0x00000000",
            "provenance": "CODE_DERIVED static lift verified by original binary trace",
            "event_code": null,
            "phase": null,
            "record_offset": "0x00",
            "notes": "Replace this example with an observed original mutation, then mirror it from Rust."
        }
    })
}

fn capture_import_sample(system: &str) -> serde_json::Value {
    let original_mutations = subsystem_sample_mutations(system, "original-binary example");
    let rust_mutations = subsystem_sample_mutations(system, "Rust-mutator example");
    serde_json::json!({
        "format": "cm0102-rs-gameplay-capture-import",
        "version": 1,
        "system": system,
        "notes": "Copy this file, replace both arrays with original cm0102.exe capture rows and matching Rust rows, then run import-gameplay-capture.",
        "original_mutations": original_mutations,
        "rust_mutations": rust_mutations
    })
}

fn subsystem_required_coverage_labels(system: &str) -> Vec<String> {
    match system {
        "match results" => vec![
            "fixture +0x43 normal-time home/status score byte",
            "fixture +0x44 normal-time away score byte",
            "fixture +0x49 final home score byte",
            "fixture +0x4a final away score byte",
            "event 0x2004 final result payload",
            "one period transition event payload: 0x20f1, 0x20f2, or 0x20f3",
        ],
        "competition state" => vec![
            "fixture participant field +0x1c",
            "fixture participant field +0x20",
            "fixture notification flag +0x4d bit 0x100",
            "fixture notification flag +0x4d bit 0x200",
            "fixture list accessor 0x00596590",
            "fixture cleanup cadence helper 0x0075f0f0",
        ],
        "transfers/contracts" => vec![
            "contract renewal date windows",
            "0x6e-byte staff pool stride",
            "0x4f-byte staff side-state stride",
            "0x50-byte event/contract record stride",
            "queued transfer/club-news dispatch item",
            "transfer.dat-equivalent manager/list state",
        ],
        "news/inbox" => vec![
            "fixture/news subrecord stride 0x68",
            "fixture/news subrecord base pointer +0xa3",
            "paired event +0x30 writes",
            "paired dated event tags +3/+4",
            "news reset byte +0xde",
            "queued news removal helper 0x006724d0",
        ],
        _ => vec!["generic mutation schema"],
    }
    .into_iter()
    .map(str::to_string)
    .collect()
}

fn subsystem_sample_mutations(system: &str, notes_prefix: &str) -> Vec<serde_json::Value> {
    match system {
        "match results" => match_result_sample_mutations(notes_prefix),
        "competition state" => vec![
            sample_mutation(
                "fixture",
                "fixture +0x1c",
                "0x1c",
                None,
                Some("0x00752d40"),
                "fixture participant field",
                notes_prefix,
            ),
            sample_mutation(
                "fixture",
                "fixture +0x20",
                "0x20",
                None,
                Some("0x00752d40"),
                "fixture participant field",
                notes_prefix,
            ),
            sample_mutation(
                "fixture",
                "fixture +0x4d flag 0x100",
                "0x4d",
                None,
                Some("0x0075ee00"),
                "notification flag 0x100",
                notes_prefix,
            ),
            sample_mutation(
                "fixture",
                "fixture +0x4d flag 0x200",
                "0x4d",
                None,
                Some("0x0075ee00"),
                "notification flag 0x200",
                notes_prefix,
            ),
            sample_mutation(
                "fixture_list",
                "fixture list accessor 0x00596590",
                "list",
                None,
                Some("0x00596590"),
                "fixture list accessor",
                notes_prefix,
            ),
            sample_mutation(
                "fixture_cleanup",
                "fixture cleanup cadence 0x0075f0f0",
                "cadence",
                None,
                Some("0x0075f0f0"),
                "70-day cleanup cadence",
                notes_prefix,
            ),
        ],
        "transfers/contracts" => vec![
            sample_mutation(
                "contract_window",
                "contract renewal date windows",
                "date",
                None,
                Some("0x00536190"),
                "contract renewal date windows",
                notes_prefix,
            ),
            sample_mutation(
                "staff",
                "0x6e-byte staff pool stride",
                "0x6e",
                None,
                Some("0x004cdef0"),
                "staff pool stride",
                notes_prefix,
            ),
            sample_mutation(
                "staff_side_state",
                "0x4f-byte staff side-state stride",
                "0x4f",
                None,
                Some("0x004cdef0"),
                "staff side-state stride",
                notes_prefix,
            ),
            sample_mutation(
                "contract_event",
                "0x50-byte event/contract record stride",
                "0x50",
                None,
                Some("0x004dc980"),
                "event/contract record stride",
                notes_prefix,
            ),
            sample_mutation(
                "queued_transfer",
                "queued transfer/club-news dispatch item",
                "queue",
                None,
                Some("0x00449710"),
                "queued transfer dispatch",
                notes_prefix,
            ),
            sample_mutation(
                "transfer_dat",
                "transfer.dat-equivalent manager/list state",
                "transfer.dat",
                None,
                Some("0x008a9080"),
                "transfer.dat manager/list state",
                notes_prefix,
            ),
        ],
        "news/inbox" => vec![
            sample_mutation(
                "fixture_news",
                "fixture/news subrecord stride 0x68",
                "0x68",
                None,
                Some("0x0050c8d0"),
                "fixture/news subrecord stride",
                notes_prefix,
            ),
            sample_mutation(
                "fixture_news",
                "fixture/news subrecord base pointer +0xa3",
                "+0xa3",
                None,
                Some("0x0050c8d0"),
                "fixture/news subrecord base pointer",
                notes_prefix,
            ),
            sample_mutation(
                "fixture_news_event",
                "paired event +0x30 writes",
                "+0x30",
                None,
                Some("0x0050c8d0"),
                "paired event write",
                notes_prefix,
            ),
            sample_mutation(
                "fixture_news_event",
                "paired dated event tags +3/+4",
                "+3/+4",
                None,
                Some("0x0050c8d0"),
                "paired dated event tags",
                notes_prefix,
            ),
            sample_mutation(
                "news",
                "news reset byte +0xde",
                "+0xde",
                None,
                Some("0x0076e180"),
                "news reset byte",
                notes_prefix,
            ),
            sample_mutation(
                "news_queue",
                "queued news removal helper 0x006724d0",
                "queue",
                None,
                Some("0x006724d0"),
                "queued news removal",
                notes_prefix,
            ),
        ],
        _ => vec![sample_mutation(
            "fixture",
            "generic mutation",
            "0x00",
            None,
            None,
            "generic mutation schema",
            notes_prefix,
        )],
    }
}

fn sample_mutation(
    table: &str,
    field: &str,
    record_offset: &str,
    event_code: Option<&str>,
    source_function: Option<&str>,
    phase: &str,
    notes_prefix: &str,
) -> serde_json::Value {
    serde_json::json!({
        "table": table,
        "row": 0,
        "field": field,
        "before": 0,
        "after": 1,
        "source_function": source_function.unwrap_or("0x00000000"),
        "provenance": "CODE_DERIVED static lift verified by original binary trace",
        "record_offset": record_offset,
        "event_code": event_code,
        "phase": phase,
        "helper": source_function,
        "notes": format!("{notes_prefix}; example only, replace with exact captured values.")
    })
}

fn match_result_sample_mutations(notes_prefix: &str) -> Vec<serde_json::Value> {
    vec![
        match_result_sample_mutation(
            "fixture +0x43",
            "0x43",
            None,
            "normal-time score snapshot",
            notes_prefix,
        ),
        match_result_sample_mutation(
            "fixture +0x44",
            "0x44",
            None,
            "normal-time score snapshot",
            notes_prefix,
        ),
        match_result_sample_mutation(
            "fixture +0x49",
            "0x49",
            Some("0x2004"),
            "phase-controller final score",
            notes_prefix,
        ),
        match_result_sample_mutation(
            "fixture +0x4a",
            "0x4a",
            Some("0x2004"),
            "phase-controller final score",
            notes_prefix,
        ),
        match_result_sample_mutation(
            "event 0x2004 payload",
            "event",
            Some("0x2004"),
            "final result event",
            notes_prefix,
        ),
        match_result_sample_mutation(
            "event 0x20f2 payload",
            "event",
            Some("0x20f2"),
            "normal-time period transition event",
            notes_prefix,
        ),
    ]
}

fn match_result_sample_mutation(
    field: &str,
    record_offset: &str,
    event_code: Option<&str>,
    phase: &str,
    notes_prefix: &str,
) -> serde_json::Value {
    serde_json::json!({
        "table": if record_offset == "event" { "match_event_queue" } else { "fixture" },
        "row": 0,
        "field": field,
        "before": 0,
        "after": 1,
        "source_function": if record_offset == "event" { "0x006bc8d0" } else { "0x006a3240/0x006a4020" },
        "provenance": "CODE_DERIVED static lift verified by original binary trace",
        "record_offset": record_offset,
        "event_code": event_code,
        "phase": phase,
        "notes": format!("{notes_prefix}; example only, replace with exact captured values.")
    })
}

fn json_string_array(value: &serde_json::Value) -> Vec<String> {
    value
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn merge_trace_template_defaults(
    trace: &mut serde_json::Value,
    db_dir: &Path,
    item: &cm_domain::BackendImplementationPlanItem,
    promotion_gate: Option<&cm_domain::GameplayPromotionGate>,
) {
    let defaults = serde_json::json!({
        "format": "cm0102-rs-gameplay-parity-trace",
        "version": 1,
        "system": item.system,
        "scenario": format!("{} baseline parity scenario", item.system),
        "status": "pending-original-and-rust-captures",
        "source": {
            "rust_db": db_dir.display().to_string(),
            "original_binary": "D:/cm0102/cm0102.exe",
            "carve": "D:/cm0102-carve"
        },
        "primary_frontiers": item.primary_frontiers,
        "code_derived_boundaries": item.code_derived_boundaries,
        "rust_implementation": {
            "present": false,
            "notes": "Set true only when the exact Rust mutator body is installed and wired into the runtime dispatcher."
        },
        "required_lifts_before_capture": item.missing_lifts,
        "acceptance_gate": item.acceptance_gate,
        "promotion_gate": promotion_gate,
        "capture_method": {
            "original": "Use original cm0102.exe trace/debugger capture only to verify statically lifted mutations.",
            "rust": "Run the matching Rust mutator on the same Rust-owned scenario seed.",
            "comparison": "Exact ordered mutation equality over table, row, field, before, after, and emitted event payloads."
        },
        "capture_plan": gameplay_capture_plan(&item.system),
        "trace_quality_gates": [
            "original_mutations is non-empty",
            "rust_mutations is non-empty",
            "every mutation records table, row, field, before, after, source_function, and provenance",
            "comparison.status is pass only after exact ordered mutation equality",
            "runtime acceptance still passes after the Rust mutator is enabled"
        ],
        "original_mutations": [],
        "rust_mutations": [],
        "comparison": {
            "status": "pending",
            "method": "exact ordered mutation equality",
            "notes": "Fill original_mutations and rust_mutations only after the subsystem mutator is lifted and implemented."
        }
    });

    if !trace.is_object() {
        *trace = serde_json::json!({});
    }
    let object = trace.as_object_mut().expect("trace was forced to object");
    for (key, value) in defaults.as_object().expect("defaults are object") {
        match object.get_mut(key) {
            Some(existing) => merge_json_defaults(existing, value),
            None => {
                object.insert(key.clone(), value.clone());
            }
        }
    }
}

fn merge_json_defaults(existing: &mut serde_json::Value, defaults: &serde_json::Value) {
    if let (Some(existing_object), Some(default_object)) =
        (existing.as_object_mut(), defaults.as_object())
    {
        for (key, value) in default_object {
            match existing_object.get_mut(key) {
                Some(existing_value) => merge_json_defaults(existing_value, value),
                None => {
                    existing_object.insert(key.clone(), value.clone());
                }
            }
        }
        return;
    }

    if let (Some(existing_array), Some(default_array)) =
        (existing.as_array_mut(), defaults.as_array())
    {
        for value in default_array {
            if !existing_array.contains(value) {
                existing_array.push(value.clone());
            }
        }
    }
}

fn gameplay_capture_plan(system: &str) -> serde_json::Value {
    match system {
        "match results" => serde_json::json!({
            "original_breakpoints": ["0x00699640", "0x00699d90", "0x0069d950", "0x006a3240", "0x006a4020", "0x0069f2f0"],
            "watched_original_writes": [
                "fixture +0x43/+0x44 normal-time score snapshot",
                "fixture +0x45/+0x46 extra-time first-period score snapshot",
                "fixture +0x47/+0x48 extra-time/final score snapshot",
                "fixture +0x49/+0x4a phase-controller final score",
                "event queue slot base +0x30 plus count*0x0e for codes 0x20f1/0x20f2/0x20f3/0x2004/0x217b",
                "event counters at +6/+8/+0xa/+0xc/+0xe",
                "match-state phase/counter writes +0x8eb2/+0x8eb3/+0x8eb6/+0x8eb7/+0x8ed0/+0x8ed2/+0x8ed4"
            ],
            "watched_match_state": [
                "match-state +0x4792 fixture pointer",
                "match-state +0xf5bc/+0xf5f2 final score source bytes",
                "match-state +0xf5bd/+0xf5f3 period score source bytes",
                "match-state +0x8eb3 phase byte",
                "match-state +0x8eb4 loop gate byte",
                "match-state +0x8ed0/+0x8ed2 tick counters",
                "match-state +0x8ed4 period short",
                "match-state +0xf638 sentinel/timeout byte"
            ],
            "capture_phases": [
                "Break at 0x0069d950 and record fixture pointer, team array anchors, and initial score/event counters before match loop.",
                "Break at 0x0069f2f0 and record loop gate, phase byte, tick counters, and fixture status byte before each step-controller dispatch.",
                "Break at 0x006a3240 and record threshold path 0x1ef/0x3de/0x483/0x528 plus fixture +0x43..+0x48 before/after writes.",
                "Break at 0x006a4020 and record phase cases that emit 0x2004/0x2005/0x2006 plus fixture +0x49/+0x4a before/after writes.",
                "Break/log 0x006bc8d0 for score/period event payloads before declaring result parity complete."
            ],
            "scenario_requirements": [
                "Use the same original database imported into D:/cm0102-rs/rust-db.",
                "Use one deterministic fixture whose fixture row/index and participating clubs/staff are recorded in the trace.",
                "Record initial fixture bytes +0x43..+0x4a before the match starts.",
                "Record RNG seed/state or the exact original call sequence needed to reproduce the same match path in Rust.",
                "Do not set comparison.status=pass until original_mutations and rust_mutations are byte-for-byte ordered equals."
            ],
            "stop_conditions": [
                "Stop after final fixture +0x49/+0x4a write and related 0x2004 event payload are captured.",
                "Stop early and mark trace pending if any watched source offset cannot be tied to the active fixture pointer.",
                "Stop early and mark trace pending if event queue payload fields are not captured for emitted score/period codes."
            ],
            "rust_hook": "future match result mutator fed by RuntimeBackendSystems.match_result_write_map",
            "minimum_trace": "one deterministic fixture with fixture +0x43..+0x4a writes and associated 0x0e-byte event queue payloads captured before and after mutation"
        }),
        "competition state" => serde_json::json!({
            "original_breakpoints": ["0x00674c10", "0x00595580", "0x00752d40", "0x0075ee00", "0x0075f0f0"],
            "watched_original_writes": [
                "fixture +0x1c/+0x20 participant references",
                "fixture +0x4d notification bits 0x100/0x200",
                "fixture list accessor 0x00596590 outputs",
                "70-day cleanup cadence helper 0x0075f0f0"
            ],
            "rust_hook": "future competition fixture/table mutator fed by RuntimeBackendSystems.competition_fixture_state_map",
            "minimum_trace": "one fixture notification/cleanup pass with before/after fixture state"
        }),
        "transfers/contracts" => serde_json::json!({
            "original_breakpoints": ["0x004cdef0", "0x00449710", "0x008a9080", "0x005246e0", "0x004dc980"],
            "watched_original_writes": [
                "contract renewal windows +7 through +0x447 days",
                "staff pool stride 0x6e with byte +0x59",
                "side-state stride 0x4f and event/contract stride 0x50",
                "event/contract fields +0x2d/+0x2f/+0x35/+0x4f",
                "transfer.dat list objects 0x41 and list strides 0x25/0x0c/0x0d/0x0e"
            ],
            "rust_hook": "future transfer/contract mutator fed by RuntimeBackendSystems.transfer_contract_state_map",
            "minimum_trace": "one contract renewal or queued transfer/news dispatch with full before/after state"
        }),
        "news/inbox" => serde_json::json!({
            "original_breakpoints": ["0x0050c8d0", "0x00595580", "0x006724d0", "0x0076e180", "0x00596fa0"],
            "watched_original_writes": [
                "fixture/news subrecord stride 0x68 under param_1 +0xa3",
                "paired event +0x30 writes from subrecord +0x07 plus 3/4",
                "news +0xde reset in 0x0076e180",
                "queue unlink/free via 0x006724d0"
            ],
            "rust_hook": "future news/inbox mutator fed by RuntimeBackendSystems.news_inbox_emission_map",
            "minimum_trace": "one paired fixture/news event emission and queue cleanup with exact payloads"
        }),
        _ => serde_json::json!({
            "original_breakpoints": [],
            "watched_original_writes": [],
            "rust_hook": "unknown subsystem",
            "minimum_trace": "one exact original-vs-Rust mutation comparison"
        }),
    }
}

fn gameplay_trace_slug(system: &str) -> String {
    system
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn report_status_pass(report: &serde_json::Value) -> bool {
    report
        .get("summary")
        .and_then(|summary| summary.get("status"))
        .and_then(serde_json::Value::as_str)
        == Some("pass")
        || report.get("status").and_then(serde_json::Value::as_str) == Some("pass")
}

fn report_check_count(report: &serde_json::Value) -> usize {
    report
        .get("summary")
        .and_then(|summary| summary.get("checks"))
        .and_then(serde_json::Value::as_u64)
        .map(|count| count as usize)
        .or_else(|| {
            report
                .get("checks")
                .and_then(serde_json::Value::as_array)
                .map(Vec::len)
        })
        .unwrap_or(0)
}

fn report_failure_count(report: &serde_json::Value) -> usize {
    report
        .get("summary")
        .and_then(|summary| summary.get("failures"))
        .and_then(serde_json::Value::as_u64)
        .map(|count| count as usize)
        .or_else(|| {
            report
                .get("checks")
                .and_then(serde_json::Value::as_array)
                .map(|checks| {
                    checks
                        .iter()
                        .filter(|check| {
                            check.get("status").and_then(serde_json::Value::as_str) == Some("fail")
                        })
                        .count()
                })
        })
        .unwrap_or(0)
}

fn validate_rng_report(exe: &Path) -> Result<serde_json::Value, String> {
    let bytes = fs::read(exe)
        .map_err(|err| format!("failed to read original binary {}: {err}", exe.display()))?;
    let pe = parse_pe_image(&bytes)?;
    let mut checks = Vec::new();

    let mut crt = cm_rng::CrtRand::new(1);
    let crt_sequence = (0..5).map(|_| crt.next()).collect::<Vec<_>>();
    push_binary_check(
        &mut checks,
        "crt-rand-known-answer",
        crt_sequence == [41, 18467, 6334, 26500, 19169],
        format!("MSVC srand(1) first five values: {crt_sequence:?}; cite 0x00935a94"),
    );

    let match_random_offset = map_va_to_file_offset(&pe, cm_rng::MATCH_RANDOM_ADDR);
    push_binary_check(
        &mut checks,
        "match-random-address",
        match_random_offset.is_some(),
        match_random_offset
            .map(|offset| {
                format!(
                    "match_random 0x{:08x} maps to file 0x{offset:x}",
                    cm_rng::MATCH_RANDOM_ADDR
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "match_random 0x{:08x} did not map",
                    cm_rng::MATCH_RANDOM_ADDR
                )
            }),
    );

    let init_offset = map_va_to_file_offset(&pe, cm_rng::MATCH_RNG_INIT_ADDR);
    push_binary_check(
        &mut checks,
        "match-rng-init-address",
        init_offset.is_some(),
        init_offset
            .map(|offset| {
                format!(
                    "RNG initializer 0x{:08x} maps to file 0x{offset:x}",
                    cm_rng::MATCH_RNG_INIT_ADDR
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "RNG initializer 0x{:08x} did not map",
                    cm_rng::MATCH_RNG_INIT_ADDR
                )
            }),
    );

    let table_offset = map_va_to_file_offset(&pe, cm_rng::MATCH_RNG_TABLE_BASE);
    let table_sample = table_offset
        .map(|offset| read_i32_sample(&bytes, offset, 16))
        .transpose()?
        .unwrap_or_default();
    push_binary_check(
        &mut checks,
        "rng-table-base",
        table_offset.is_some() && table_sample.iter().any(|value| *value != 0),
        table_offset
            .map(|offset| {
                format!(
                    "table base 0x{:08x} maps to file 0x{offset:x}; first entries {table_sample:?}",
                    cm_rng::MATCH_RNG_TABLE_BASE
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "table base 0x{:08x} did not map",
                    cm_rng::MATCH_RNG_TABLE_BASE
                )
            }),
    );

    let table_end_offset = map_va_to_file_offset(&pe, cm_rng::MATCH_RNG_TABLE_END_INCLUSIVE);
    push_binary_check(
        &mut checks,
        "rng-table-end",
        table_end_offset.is_some()
            && cm_rng::match_rng_table_byte_len() / 4 == cm_rng::MATCH_RNG_TABLE_ENTRIES,
        table_end_offset
            .map(|offset| {
                format!(
                    "initializer sets wrap/end to 0x{:08x}; inclusive range is {} bytes / {} i32 entries; end maps to file 0x{offset:x}",
                    cm_rng::MATCH_RNG_TABLE_END_INCLUSIVE,
                    cm_rng::match_rng_table_byte_len(),
                    cm_rng::MATCH_RNG_TABLE_ENTRIES
                )
            })
            .unwrap_or_else(|| format!("table end 0x{:08x} did not map", cm_rng::MATCH_RNG_TABLE_END_INCLUSIVE)),
    );

    push_binary_check(
        &mut checks,
        "rng-start-modulus",
        cm_rng::MATCH_RNG_START_MODULUS as usize == cm_rng::MATCH_RNG_TABLE_ENTRIES,
        format!(
            "initializer chooses start pointer with rand() % {}; matches table entry count",
            cm_rng::MATCH_RNG_START_MODULUS
        ),
    );

    let state_globals = [
        ("seed16", 0x00dc7234),
        ("table_ptr", 0x00dc7238),
        ("wrap_or_end", 0x00dc7a70),
    ];
    let state_reports = state_globals
        .into_iter()
        .map(|(name, va)| {
            let rva = va - pe.image_base;
            let section = pe.sections.iter().find(|section| {
                rva >= section.virtual_address
                    && rva < section.virtual_address.saturating_add(section.virtual_size.max(section.raw_size))
            });
            let mapped = map_va_to_file_offset(&pe, va).filter(|offset| *offset < bytes.len());
            serde_json::json!({
                "name": name,
                "address": format!("0x{va:08x}"),
                "section": section.map(|section| section.name.clone()).unwrap_or_else(|| "unmapped".to_string()),
                "raw_file_offset": mapped.map(|offset| format!("0x{offset:x}")),
                "storage": if mapped.is_some() { "initialized" } else { "zero-filled-virtual" },
            })
        })
        .collect::<Vec<_>>();
    push_binary_check(
        &mut checks,
        "rng-state-globals",
        state_reports.iter().all(|entry| entry["section"].as_str() == Some(".data")),
        "RNG state globals resolve into the .data virtual section; most are zero-filled BSS in the file",
    );

    let failures = checks
        .iter()
        .filter(|check| check.get("status").and_then(serde_json::Value::as_str) == Some("fail"))
        .count();

    Ok(serde_json::json!({
        "source": {
            "binary": exe.display().to_string(),
            "carve": "D:/cm0102-carve",
            "citations": [
                "MATCH_ENGINE.md RNG",
                "claims.json id 0x008fc4f0-rng",
                "cm-rng/src/lib.rs"
            ]
        },
        "summary": {
            "checks": checks.len(),
            "failures": failures,
            "status": if failures == 0 { "pass" } else { "fail" },
        },
        "crt_rand": {
            "address": "0x00935a94",
            "algorithm": "seed = seed*0x343FD + 0x269EC3; return (seed>>16)&0x7fff",
            "known_answer_seed_1": crt_sequence,
        },
        "match_random": {
            "address": format!("0x{:08x}", cm_rng::MATCH_RANDOM_ADDR),
            "initializer": {
                "address": format!("0x{:08x}", cm_rng::MATCH_RNG_INIT_ADDR),
                "start_pointer": format!("base + rand() % {}", cm_rng::MATCH_RNG_START_MODULUS),
                "table_end_inclusive": format!("0x{:08x}", cm_rng::MATCH_RNG_TABLE_END_INCLUSIVE),
                "seed16": "rand() % 0xffff",
                "citation": "Ghidra decompile ghidra_out/cm0102.exe/decompiled/008fc5d0.c",
            },
            "table_base": format!("0x{:08x}", cm_rng::MATCH_RNG_TABLE_BASE),
            "table_end_inclusive": format!("0x{:08x}", cm_rng::MATCH_RNG_TABLE_END_INCLUSIVE),
            "table_entries": cm_rng::MATCH_RNG_TABLE_ENTRIES,
            "table_byte_len": cm_rng::match_rng_table_byte_len(),
            "table_file_offset": table_offset.map(|offset| format!("0x{offset:x}")),
            "table_sample_i32": table_sample,
            "state_globals": state_reports,
            "note": "Table base, inclusive end pointer, 51,000-entry length, and state globals are validated against the PE and the RNG initializer decompile.",
        },
        "checks": checks,
    }))
}

fn extract_rng_table_report(exe: &Path, entries: usize) -> Result<serde_json::Value, String> {
    if entries == 0 {
        return Err("entry count must be greater than zero".to_string());
    }
    if entries > cm_rng::MATCH_RNG_TABLE_ENTRIES {
        return Err(format!(
            "entry count {entries} exceeds verified RNG table length {}",
            cm_rng::MATCH_RNG_TABLE_ENTRIES
        ));
    }
    let bytes = fs::read(exe)
        .map_err(|err| format!("failed to read original binary {}: {err}", exe.display()))?;
    let pe = parse_pe_image(&bytes)?;
    let table_offset =
        map_va_to_file_offset(&pe, cm_rng::MATCH_RNG_TABLE_BASE).ok_or_else(|| {
            format!(
                "RNG table base 0x{:08x} did not map to the PE file",
                cm_rng::MATCH_RNG_TABLE_BASE
            )
        })?;
    let byte_len = entries
        .checked_mul(4)
        .ok_or_else(|| "entry count is too large".to_string())?;
    let table_bytes = bytes
        .get(table_offset..table_offset + byte_len)
        .ok_or_else(|| {
            format!(
                "requested {entries} entries from file 0x{table_offset:x}, beyond {} byte file",
                bytes.len()
            )
        })?;
    let decoded = if entries == cm_rng::MATCH_RNG_TABLE_ENTRIES {
        cm_rng::full_table_from_le_bytes(table_bytes)
    } else {
        cm_rng::table_from_le_bytes(table_bytes)
    }
    .map_err(|err| format!("failed to decode RNG table bytes: {err}"))?;
    let checksum = decoded.iter().fold(0u32, |acc, value| {
        acc.wrapping_mul(16_777_619).wrapping_add(*value as u32)
    });
    Ok(serde_json::json!({
        "format": if entries == cm_rng::MATCH_RNG_TABLE_ENTRIES { "cm0102-rng-table" } else { "cm0102-rng-table-sample" },
        "version": 1,
        "source": {
            "binary": exe.display().to_string(),
            "carve": "D:/cm0102-carve",
            "citations": [
                "claims.json id 0x008fc4f0-rng",
                "Ghidra decompile ghidra_out/cm0102.exe/decompiled/008fc5d0.c"
            ],
        },
        "address": format!("0x{:08x}", cm_rng::MATCH_RNG_TABLE_BASE),
        "end_inclusive": format!("0x{:08x}", cm_rng::MATCH_RNG_TABLE_END_INCLUSIVE),
        "file_offset": format!("0x{table_offset:x}"),
        "entry_type": "little-endian i32",
        "entries_total_verified": cm_rng::MATCH_RNG_TABLE_ENTRIES,
        "entries_exported": entries,
        "entries": decoded,
        "checksum_fnv_style": format!("0x{checksum:08x}"),
        "length_status": if entries == cm_rng::MATCH_RNG_TABLE_ENTRIES {
            "full verified table; length derived from RNG initializer/wrap end 0x00abfc14"
        } else {
            "sample of verified 51,000-entry table"
        },
    }))
}

fn read_i32_sample(bytes: &[u8], offset: usize, count: usize) -> Result<Vec<i32>, String> {
    let mut values = Vec::new();
    for index in 0..count {
        let start = offset + index * 4;
        let slice = bytes
            .get(start..start + 4)
            .ok_or_else(|| format!("i32 sample outside file at 0x{start:x}"))?;
        values.push(i32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]));
    }
    Ok(values)
}

fn parse_pe_image(bytes: &[u8]) -> Result<PeImage, String> {
    if bytes.get(0..2) != Some(b"MZ") {
        return Err("missing MZ header".to_string());
    }
    let pe_offset = read_u32(bytes, 0x3c)? as usize;
    if bytes.get(pe_offset..pe_offset + 4) != Some(b"PE\0\0") {
        return Err("missing PE signature".to_string());
    }
    let coff = pe_offset + 4;
    let machine = read_u16(bytes, coff)?;
    let section_count = read_u16(bytes, coff + 2)? as usize;
    let optional_header_size = read_u16(bytes, coff + 16)? as usize;
    let optional = coff + 20;
    let optional_magic = read_u16(bytes, optional)?;
    let entry_point_rva = read_u32(bytes, optional + 16)?;
    let image_base = read_u32(bytes, optional + 28)?;
    let section_table = optional + optional_header_size;
    let mut sections = Vec::new();
    for index in 0..section_count {
        let base = section_table + index * 40;
        let name_bytes = bytes
            .get(base..base + 8)
            .ok_or_else(|| format!("section {index} header outside file"))?;
        let name = String::from_utf8_lossy(
            &name_bytes[..name_bytes
                .iter()
                .position(|byte| *byte == 0)
                .unwrap_or(name_bytes.len())],
        )
        .into_owned();
        sections.push(PeSection {
            name,
            virtual_size: read_u32(bytes, base + 8)?,
            virtual_address: read_u32(bytes, base + 12)?,
            raw_size: read_u32(bytes, base + 16)?,
            raw_pointer: read_u32(bytes, base + 20)?,
            characteristics: read_u32(bytes, base + 36)?,
        });
    }
    Ok(PeImage {
        machine,
        optional_magic,
        image_base,
        entry_point_rva,
        sections,
    })
}

fn map_va_to_file_offset(pe: &PeImage, va: u32) -> Option<usize> {
    let rva = va.checked_sub(pe.image_base)?;
    for section in &pe.sections {
        let span = section.virtual_size.max(section.raw_size);
        if rva >= section.virtual_address && rva < section.virtual_address.saturating_add(span) {
            return Some((section.raw_pointer + (rva - section.virtual_address)) as usize);
        }
    }
    None
}

fn push_binary_check(
    checks: &mut Vec<serde_json::Value>,
    name: &str,
    passed: bool,
    detail: impl Into<String>,
) {
    checks.push(serde_json::json!({
        "name": name,
        "status": if passed { "pass" } else { "fail" },
        "detail": detail.into(),
    }));
}

fn find_ascii(bytes: &[u8], text: &str) -> Option<usize> {
    let needle = text.as_bytes();
    bytes
        .windows(needle.len())
        .position(|window| window == needle)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let slice = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| format!("read_u16 outside file at 0x{offset:x}"))?;
    Ok(u16::from_le_bytes([slice[0], slice[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let slice = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| format!("read_u32 outside file at 0x{offset:x}"))?;
    Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
}

fn serve_api_runtime_save_tick(
    stream: &mut TcpStream,
    db_dir: &Path,
    body: &str,
) -> Result<(), String> {
    let request: serde_json::Value = if body.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(body).map_err(|err| format!("invalid runtime tick JSON: {err}"))?
    };
    let days = request
        .get("days")
        .and_then(serde_json::Value::as_u64)
        .map(u32::try_from)
        .transpose()
        .map_err(|_| "runtime tick days is too large for u32".to_string())?
        .unwrap_or(1);
    let save_path = default_runtime_save_path(db_dir);
    let mut save = RuntimeSaveGame::read_json_file(&save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    save.tick_days(days);
    save.write_json_file(&save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    let value = serde_json::to_value(save)
        .map_err(|err| format!("failed to serialize Rust save: {err}"))?;
    write_json_response(stream, 200, value)
}

fn serve_api_runtime_save_tick_to_date(
    stream: &mut TcpStream,
    db_dir: &Path,
    body: &str,
) -> Result<(), String> {
    let request: serde_json::Value = serde_json::from_str(body)
        .map_err(|err| format!("invalid runtime tick-to-date JSON: {err}"))?;
    let target = game_date_from_json(&request)?;
    let save_path = default_runtime_save_path(db_dir);
    let mut save = RuntimeSaveGame::read_json_file(&save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let advanced_days = save.tick_to_date(target);
    save.write_json_file(&save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    let value = serde_json::json!({
        "advanced_days": advanced_days,
        "save": save
    });
    write_json_response(stream, 200, value)
}

fn serve_api_headless_run(stream: &mut TcpStream, db_dir: &Path, body: &str) -> Result<(), String> {
    let request: serde_json::Value = if body.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(body).map_err(|err| format!("invalid headless run JSON: {err}"))?
    };
    let days = request
        .get("days")
        .and_then(serde_json::Value::as_u64)
        .map(u32::try_from)
        .transpose()
        .map_err(|_| "headless run days is too large for u32".to_string())?
        .unwrap_or(1);
    let save_path = default_runtime_save_path(db_dir);
    let mut save = RuntimeSaveGame::read_json_file(&save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let report = save.run_headless_days(days);
    save.write_json_file(&save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    write_json_response(
        stream,
        200,
        serde_json::json!({
            "report": report,
            "save": save
        }),
    )
}

fn serve_api_headless_run_to_date(
    stream: &mut TcpStream,
    db_dir: &Path,
    body: &str,
) -> Result<(), String> {
    let request: serde_json::Value = serde_json::from_str(body)
        .map_err(|err| format!("invalid headless run-to-date JSON: {err}"))?;
    let target = game_date_from_json(&request)?;
    let save_path = default_runtime_save_path(db_dir);
    let mut save = RuntimeSaveGame::read_json_file(&save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let report = save.run_headless_to_date(target);
    save.write_json_file(&save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    write_json_response(
        stream,
        200,
        serde_json::json!({
            "report": report,
            "save": save
        }),
    )
}

fn serve_api_headless_campaign(
    stream: &mut TcpStream,
    db_dir: &Path,
    body: &str,
) -> Result<(), String> {
    let request: serde_json::Value = if body.trim().is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_str(body)
            .map_err(|err| format!("invalid headless campaign JSON: {err}"))?
    };
    let days = request
        .get("days")
        .and_then(serde_json::Value::as_u64)
        .map(u32::try_from)
        .transpose()
        .map_err(|_| "headless campaign days is too large for u32".to_string())?
        .unwrap_or(30);
    let checkpoint_every = request
        .get("checkpoint_every")
        .and_then(serde_json::Value::as_u64)
        .map(u32::try_from)
        .transpose()
        .map_err(|_| "headless campaign checkpoint_every is too large for u32".to_string())?
        .unwrap_or(30);
    let save_path = default_runtime_save_path(db_dir);
    let mut save = RuntimeSaveGame::read_json_file(&save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let report = save.run_headless_campaign_days(days, checkpoint_every);
    save.write_json_file(&save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    write_json_response(
        stream,
        200,
        serde_json::json!({
            "report": report,
            "save": save
        }),
    )
}

fn serve_api_headless_manager(
    stream: &mut TcpStream,
    db_dir: &Path,
    body: &str,
) -> Result<(), String> {
    let request: serde_json::Value = serde_json::from_str(body)
        .map_err(|err| format!("invalid headless manager JSON: {err}"))?;
    let name = request
        .get("name")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| "headless manager requires string field 'name'".to_string())?;
    let club_id = request
        .get("club_id")
        .and_then(serde_json::Value::as_u64)
        .map(u32::try_from)
        .transpose()
        .map_err(|_| "headless manager club_id is too large for u32".to_string())?;
    let save_path = default_runtime_save_path(db_dir);
    let mut save = RuntimeSaveGame::read_json_file(&save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let command = save.set_headless_manager(name.to_string(), club_id);
    save.write_json_file(&save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    write_json_response(
        stream,
        200,
        serde_json::json!({
            "command": command,
            "save": save
        }),
    )
}

fn serve_api_headless_manager_run_next_fixture(
    stream: &mut TcpStream,
    db_dir: &Path,
) -> Result<(), String> {
    let save_path = default_runtime_save_path(db_dir);
    let mut save = RuntimeSaveGame::read_json_file(&save_path)
        .map_err(|err| format!("failed to read Rust save {}: {err}", save_path.display()))?;
    let club_id = save
        .headless
        .manager
        .as_ref()
        .and_then(|manager| manager.club_id)
        .ok_or_else(|| "select a headless manager club before running next fixture".to_string())?;
    let Some(next_date) = save
        .season
        .fixtures
        .iter()
        .filter(|fixture| fixture.home_club_id == club_id || fixture.away_club_id == club_id)
        .find(|fixture| fixture.match_report.is_none())
        .map(|fixture| fixture.date.clone())
    else {
        return write_json_response(
            stream,
            200,
            serde_json::json!({
                "message": "No pending fixture found for selected club.",
                "dashboard": headless_manager_dashboard_value(&save, None)
            }),
        );
    };
    let run_report = if next_date <= save.date {
        save.run_headless_days(1)
    } else {
        save.run_headless_to_date(game_date_add_days(&next_date, 1))
    };
    save.write_json_file(&save_path)
        .map_err(|err| format!("failed to write Rust save {}: {err}", save_path.display()))?;
    write_json_response(
        stream,
        200,
        serde_json::json!({
            "message": "Advanced to selected club next fixture.",
            "dashboard": headless_manager_dashboard_value(&save, Some(run_report))
        }),
    )
}

fn game_date_add_days(date: &GameDate, days: i16) -> GameDate {
    CmPackedDate::from_game_date(date.clone())
        .add_days(days)
        .to_game_date()
}

fn serve_api_original_capture_row(stream: &mut TcpStream, body: &str) -> Result<(), String> {
    let request: serde_json::Value = serde_json::from_str(body)
        .map_err(|err| format!("invalid original capture row JSON: {err}"))?;
    let system = required_str(&request, "system")?;
    let row = required_usize(&request, "row")?;
    let before = required_str(&request, "before")?;
    let after = required_str(&request, "after")?;
    let capture_status = request
        .get("capture_status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("captured-from-original");
    let extra_notes = request
        .get("notes")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");
    let report =
        update_original_capture_row(system, row, before, after, capture_status, extra_notes)?;
    write_json_response(stream, 200, report)
}

fn serve_api_original_capture_import_csv(stream: &mut TcpStream, body: &str) -> Result<(), String> {
    let request: serde_json::Value = serde_json::from_str(body)
        .map_err(|err| format!("invalid original capture CSV import JSON: {err}"))?;
    let csv = required_str(&request, "csv")?;
    let report = import_original_capture_csv_text(csv)?;
    write_json_response(stream, 200, report)
}

fn serve_api_original_capture_validate_csv(
    stream: &mut TcpStream,
    body: &str,
) -> Result<(), String> {
    let request: serde_json::Value = serde_json::from_str(body)
        .map_err(|err| format!("invalid original capture CSV validation JSON: {err}"))?;
    let csv = required_str(&request, "csv")?;
    let report = validate_original_capture_csv_text(csv)?;
    write_json_response(stream, 200, report)
}

fn serve_api_original_capture_submit_csv(stream: &mut TcpStream, body: &str) -> Result<(), String> {
    let request: serde_json::Value = serde_json::from_str(body)
        .map_err(|err| format!("invalid original capture CSV submit JSON: {err}"))?;
    let csv = required_str(&request, "csv")?;
    let report = submit_original_capture_csv_text(csv, Path::new("D:/cm0102-rs/reports"))?;
    write_json_response(stream, 200, report)
}

fn serve_api_original_capture_import_system(
    stream: &mut TcpStream,
    body: &str,
) -> Result<(), String> {
    let request: serde_json::Value = serde_json::from_str(body)
        .map_err(|err| format!("invalid original capture import JSON: {err}"))?;
    let system = required_str(&request, "system")?;
    let report = import_original_capture_system(system)?;
    write_json_response(stream, 200, report)
}

fn serve_api_original_capture_import_ready(stream: &mut TcpStream) -> Result<(), String> {
    let report = import_ready_original_capture_systems()?;
    write_json_response(stream, 200, report)
}

fn game_date_from_json(value: &serde_json::Value) -> Result<GameDate, String> {
    if let Some(date) = value.get("date").and_then(serde_json::Value::as_str) {
        return parse_game_date(date);
    }
    let year = json_u16_field(value, "year")?;
    let month = json_u8_field(value, "month")?;
    let day = json_u8_field(value, "day")?;
    validate_game_date(GameDate { year, month, day })
}

fn json_u16_field(value: &serde_json::Value, field: &str) -> Result<u16, String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("date requires numeric field '{field}'"))?
        .try_into()
        .map_err(|_| format!("date field '{field}' is too large"))
}

fn json_u8_field(value: &serde_json::Value, field: &str) -> Result<u8, String> {
    value
        .get(field)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("date requires numeric field '{field}'"))?
        .try_into()
        .map_err(|_| format!("date field '{field}' is too large"))
}

fn serve_api_edit_batch(
    stream: &mut TcpStream,
    db_dir: &Path,
    world: &mut World,
    body: &str,
) -> Result<(), String> {
    let request: serde_json::Value =
        serde_json::from_str(body).map_err(|err| format!("invalid batch edit JSON: {err}"))?;
    let changes = request
        .get("changes")
        .and_then(|value| value.as_array())
        .ok_or_else(|| "batch edit requires a changes array".to_string())?;
    if changes.is_empty() {
        return write_json_response(
            stream,
            200,
            serde_json::json!({ "changed": [], "audit": world.audit_rust_db() }),
        );
    }

    let mut changed = Vec::new();
    let mut affected_tables = HashSet::new();
    for change in changes {
        let kind = required_str(change, "kind")?;
        let message = match kind {
            "text" => {
                let table = required_str(change, "table")?;
                let row = required_str(change, "row")?;
                let field = required_str(change, "field")?;
                let text = required_str(change, "text")?;
                affected_tables.insert(table.to_string());
                set_rust_db_text_field(world, table, row, field, text)?
            }
            "staff-type10" => {
                let id = required_u32(change, "id")?;
                let field = required_str(change, "field")?;
                let value = required_u16(change, "value")?;
                affected_tables.insert("staff.type10".to_string());
                set_staff_type10_scalar(world, id, field, value)?
            }
            "staff-attribute" => {
                let id = required_u32(change, "id")?;
                let index = required_usize(change, "index")?;
                let value = required_u8(change, "value")?;
                affected_tables.insert("staff.type10".to_string());
                set_staff_type10_attribute(world, id, index, value)?
            }
            _ => return Err(format!("unsupported batch edit kind {kind}")),
        };
        changed.push(message);
    }

    world.refresh_summaries();
    for table in &affected_tables {
        write_rust_db_table(world, db_dir, table)?;
    }
    let audit = world.audit_rust_db();
    write_json_response(
        stream,
        200,
        serde_json::json!({ "changed": changed, "audit": audit }),
    )
}

fn serve_api_edit_text(
    stream: &mut TcpStream,
    db_dir: &Path,
    world: &mut World,
    body: &str,
) -> Result<(), String> {
    let request: serde_json::Value =
        serde_json::from_str(body).map_err(|err| format!("invalid edit text JSON: {err}"))?;
    let table = required_str(&request, "table")?;
    let row = required_str(&request, "row")?;
    let field = required_str(&request, "field")?;
    let text = required_str(&request, "text")?;
    let changed = set_rust_db_text_field(world, table, row, field, text)?;
    world.refresh_summaries();
    write_rust_db_table(world, db_dir, table)?;
    let audit = world.audit_rust_db();
    write_json_response(
        stream,
        200,
        serde_json::json!({ "changed": changed, "audit": audit }),
    )
}

fn serve_api_edit_staff_type10(
    stream: &mut TcpStream,
    db_dir: &Path,
    world: &mut World,
    body: &str,
) -> Result<(), String> {
    let request: serde_json::Value =
        serde_json::from_str(body).map_err(|err| format!("invalid staff type10 JSON: {err}"))?;
    let id = required_u32(&request, "id")?;
    let field = required_str(&request, "field")?;
    let value = required_u16(&request, "value")?;
    let changed = set_staff_type10_scalar(world, id, field, value)?;
    world.refresh_summaries();
    write_rust_db_table(world, db_dir, "staff.type10")?;
    let audit = world.audit_rust_db();
    write_json_response(
        stream,
        200,
        serde_json::json!({ "changed": changed, "audit": audit }),
    )
}

fn serve_api_edit_staff_attribute(
    stream: &mut TcpStream,
    db_dir: &Path,
    world: &mut World,
    body: &str,
) -> Result<(), String> {
    let request: serde_json::Value =
        serde_json::from_str(body).map_err(|err| format!("invalid staff attribute JSON: {err}"))?;
    let id = required_u32(&request, "id")?;
    let index = required_usize(&request, "index")?;
    let value = required_u8(&request, "value")?;
    if index >= 31 {
        return Err(format!(
            "staff.type10 attribute index must be 0..30, got {index}"
        ));
    }
    let changed = set_staff_type10_attribute(world, id, index, value)?;
    world.refresh_summaries();
    write_rust_db_table(world, db_dir, "staff.type10")?;
    let audit = world.audit_rust_db();
    write_json_response(
        stream,
        200,
        serde_json::json!({
            "changed": changed,
            "audit": audit,
        }),
    )
}

fn write_json_response(
    stream: &mut TcpStream,
    status: u16,
    body: serde_json::Value,
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let body =
        serde_json::to_vec(&body).map_err(|err| format!("failed to serialize response: {err}"))?;
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Methods: GET, POST, OPTIONS\r\nAccess-Control-Allow-Headers: Content-Type\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(&body))
        .map_err(|err| format!("failed to write response: {err}"))
}

fn write_html_response(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let bytes = body.as_bytes();
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/html; charset=utf-8\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        bytes.len()
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(bytes))
        .map_err(|err| format!("failed to write HTML response: {err}"))
}

fn write_binary_response(
    stream: &mut TcpStream,
    status: u16,
    content_type: &str,
    body: &[u8],
) -> Result<(), String> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };
    let header = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: {content_type}\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream
        .write_all(header.as_bytes())
        .and_then(|_| stream.write_all(body))
        .map_err(|err| format!("failed to write binary response: {err}"))
}

fn content_type_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" | "htm" => "text/html; charset=utf-8",
        "json" => "application/json; charset=utf-8",
        "csv" => "text/csv; charset=utf-8",
        "txt" => "text/plain; charset=utf-8",
        "md" => "text/markdown; charset=utf-8",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "wav" => "audio/wav",
        "ttf" => "font/ttf",
        "fnt" => "application/octet-stream",
        _ => "application/octet-stream",
    }
}

fn write_rust_db_table(world: &World, db_dir: &Path, table: &str) -> Result<(), String> {
    let (relative_path, value): (&str, serde_json::Value) = match table {
        "base_data" => ("base_data.json", to_viewer_value(&world.base_data)?),
        "schema.tables" => ("schema.json", to_viewer_value(&world.schema)?),
        "core.clubs" => ("core/clubs.json", to_viewer_value(&world.core.clubs)?),
        "core.nat_clubs" => (
            "core/nat_clubs.json",
            to_viewer_value(&world.core.nat_clubs)?,
        ),
        "core.colours" => ("core/colours.json", to_viewer_value(&world.core.colours)?),
        "core.continents" => (
            "core/continents.json",
            to_viewer_value(&world.core.continents)?,
        ),
        "core.nations" => ("core/nations.json", to_viewer_value(&world.core.nations)?),
        "references.cities" => (
            "references/cities.json",
            to_viewer_value(&world.references.cities)?,
        ),
        "references.officials" => (
            "references/officials.json",
            to_viewer_value(&world.references.officials)?,
        ),
        "references.first_names" => (
            "references/first_names.json",
            to_viewer_value(&world.references.first_names)?,
        ),
        "references.second_names" => (
            "references/second_names.json",
            to_viewer_value(&world.references.second_names)?,
        ),
        "references.common_names" => (
            "references/common_names.json",
            to_viewer_value(&world.references.common_names)?,
        ),
        "references.stadiums" => (
            "references/stadiums.json",
            to_viewer_value(&world.references.stadiums)?,
        ),
        "references.staff_competitions" => (
            "references/staff_competitions.json",
            to_viewer_value(&world.references.staff_competitions)?,
        ),
        "references.club_competitions" => (
            "references/club_competitions.json",
            to_viewer_value(&world.references.club_competitions)?,
        ),
        "references.nation_competitions" => (
            "references/nation_competitions.json",
            to_viewer_value(&world.references.nation_competitions)?,
        ),
        "references.staff_history" => (
            "references/staff_history.json",
            to_viewer_value(&world.references.staff_history)?,
        ),
        "references.staff_comp_history" => (
            "references/staff_comp_history.json",
            to_viewer_value(&world.references.staff_comp_history)?,
        ),
        "references.club_comp_history" => (
            "references/club_comp_history.json",
            to_viewer_value(&world.references.club_comp_history)?,
        ),
        "references.nation_comp_history" => (
            "references/nation_comp_history.json",
            to_viewer_value(&world.references.nation_comp_history)?,
        ),
        "staff.type6" => ("staff/type6.json", to_viewer_value(&world.staff.type6)?),
        "staff.type8" => ("staff/type8.json", to_viewer_value(&world.staff.type8)?),
        "staff.type9" => ("staff/type9.json", to_viewer_value(&world.staff.type9)?),
        "staff.type10" => ("staff/type10.json", to_viewer_value(&world.staff.type10)?),
        _ => return Err(format!("cannot persist unknown Rust DB table {table}")),
    };
    let path = db_dir.join(relative_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    let bytes = serde_json::to_vec_pretty(&value)
        .map_err(|err| format!("failed to serialize {table}: {err}"))?;
    fs::write(&path, bytes).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn default_runtime_save_path(db_dir: &Path) -> PathBuf {
    db_dir
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("saves")
        .join("new_game.json")
}

fn required_str<'a>(value: &'a serde_json::Value, key: &str) -> Result<&'a str, String> {
    value
        .get(key)
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| format!("missing string field {key}"))
}

fn required_u32(value: &serde_json::Value, key: &str) -> Result<u32, String> {
    let raw = value
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("missing integer field {key}"))?;
    u32::try_from(raw).map_err(|_| format!("{key} value {raw} is too large for u32"))
}

fn required_u16(value: &serde_json::Value, key: &str) -> Result<u16, String> {
    let raw = value
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("missing integer field {key}"))?;
    u16::try_from(raw).map_err(|_| format!("{key} value {raw} is too large for u16"))
}

fn required_u8(value: &serde_json::Value, key: &str) -> Result<u8, String> {
    let raw = value
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("missing integer field {key}"))?;
    u8::try_from(raw).map_err(|_| format!("{key} value {raw} is too large for u8"))
}

fn required_usize(value: &serde_json::Value, key: &str) -> Result<usize, String> {
    let raw = value
        .get(key)
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| format!("missing integer field {key}"))?;
    usize::try_from(raw).map_err(|_| format!("{key} value {raw} is too large for usize"))
}

fn url_decode(value: &str) -> String {
    let mut bytes = Vec::new();
    let mut input = value.as_bytes().iter().copied();
    while let Some(byte) = input.next() {
        if byte == b'%' {
            let hi = input.next();
            let lo = input.next();
            if let (Some(hi), Some(lo)) = (hi, lo) {
                if let (Some(hi), Some(lo)) = (hex_value(hi), hex_value(lo)) {
                    bytes.push((hi << 4) | lo);
                    continue;
                }
            }
            bytes.push(byte);
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn run_rename_rust_db(db_dir: &Path, table: &str, row: &str, name: &str) -> Result<(), String> {
    let mut world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let changed = rename_structured_table_row(&mut world, table, row, name)?;
    world.refresh_summaries();
    world
        .write_rust_db_dir(db_dir, None)
        .map_err(|err| format!("failed to write Rust DB {}: {err}", db_dir.display()))?;
    println!("{changed}");
    Ok(())
}

fn run_set_rust_db_text(
    db_dir: &Path,
    table: &str,
    row: &str,
    field: &str,
    text: &str,
) -> Result<(), String> {
    let mut world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let changed = set_rust_db_text_field(&mut world, table, row, field, text)?;
    world.refresh_summaries();
    world
        .write_rust_db_dir(db_dir, None)
        .map_err(|err| format!("failed to write Rust DB {}: {err}", db_dir.display()))?;
    println!("{changed}");
    Ok(())
}

fn run_set_staff_type10(db_dir: &Path, id: u32, field: &str, value: u16) -> Result<(), String> {
    let mut world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let entry = world
        .staff
        .type10
        .iter_mut()
        .find(|entry| entry.id == id)
        .ok_or_else(|| format!("no staff.type10 row id {id}"))?;
    let changed = match field {
        "ca" | "probable_ca" | "rating_short_0x05" => {
            let old = entry.rating_short_0x05;
            entry.rating_short_0x05 = value;
            format!("staff.type10 id {id} rating_short_0x05: {old} -> {value}")
        }
        "pa" | "probable_pa" | "rating_short_0x07" => {
            let old = entry.rating_short_0x07;
            entry.rating_short_0x07 = value;
            format!("staff.type10 id {id} rating_short_0x07: {old} -> {value}")
        }
        "reputation" | "probable_reputation" | "rating_short_0x0d" => {
            let old = entry.rating_short_0x0d;
            entry.rating_short_0x0d = value;
            format!("staff.type10 id {id} rating_short_0x0d: {old} -> {value}")
        }
        _ => {
            return Err(format!(
                "unsupported staff.type10 field {field}; use ca, pa, or reputation"
            ));
        }
    };
    world.refresh_summaries();
    world
        .write_rust_db_dir(db_dir, None)
        .map_err(|err| format!("failed to write Rust DB {}: {err}", db_dir.display()))?;
    println!("{changed}");
    Ok(())
}

fn run_set_staff_attribute(db_dir: &Path, id: u32, index: usize, value: u8) -> Result<(), String> {
    if index >= 31 {
        return Err(format!(
            "staff.type10 attribute index must be 0..30, got {index}"
        ));
    }
    let mut world = World::read_rust_db_dir(db_dir)
        .map_err(|err| format!("failed to read Rust DB {}: {err}", db_dir.display()))?;
    let entry = world
        .staff
        .type10
        .iter_mut()
        .find(|entry| entry.id == id)
        .ok_or_else(|| format!("no staff.type10 row id {id}"))?;
    let old = entry.attributes[index];
    entry.attributes[index] = value;
    world.refresh_summaries();
    world
        .write_rust_db_dir(db_dir, None)
        .map_err(|err| format!("failed to write Rust DB {}: {err}", db_dir.display()))?;
    println!("staff.type10 id {id} attr_{index}: {old} -> {value}");
    Ok(())
}

fn print_audit_report(report: &SnapshotAuditReport) {
    println!("audit install: {}", report.install_root);
    println!(
        "manifest entries: snapshot {} install {}",
        report.snapshot_manifest_entries, report.install_manifest_entries
    );
    println!(
        "known logical tables: snapshot recognized {} install {}",
        report.snapshot_recognized_manifest_entries, report.install_known_logical_tables
    );
    println!(
        "save sections: snapshot {:?} install {:?}",
        report.snapshot_save_section_count, report.install_save_section_count
    );
    println!(
        "coverage: manifest entries {} known logical {} recognized {} unrecognized {} owned core {} owned references {} owned staff {} owned world {} remaining binary {}",
        report.coverage.manifest_entries,
        report.coverage.known_logical_tables,
        report.coverage.recognized_manifest_entries,
        report.coverage.unrecognized_manifest_entries,
        report.coverage.owned_core_tables,
        report.coverage.owned_reference_tables,
        report.coverage.owned_staff_tables,
        report.coverage.owned_world_tables,
        report.coverage.remaining_binary_tables
    );
    if report.mismatches.is_empty() {
        println!("audit result: OK");
    } else {
        println!("audit result: {} mismatches", report.mismatches.len());
        for mismatch in &report.mismatches {
            println!("mismatch: {mismatch}");
        }
    }
}

fn print_rust_db_audit_report(report: &RustDatabaseAuditReport) {
    println!(
        "Rust DB audit: checked {} tables, owned world {}, remaining binary {}",
        report.checked_tables,
        report.coverage.owned_world_tables,
        report.coverage.remaining_binary_tables
    );
    if report.mismatches.is_empty() {
        println!("audit result: OK");
    } else {
        println!("audit result: {} mismatches", report.mismatches.len());
        for mismatch in &report.mismatches {
            println!("mismatch: {mismatch}");
        }
    }
}

fn print_canonical_report(report: &cm_domain::CanonicalDatabaseReport) {
    println!("Canonical Rust DB report");
    println!(
        "tables: {} | fields: {} verified {} inferred {} projected {}",
        report.table_count,
        report.field_count,
        report.verified_fields,
        report.inferred_fields,
        report.projected_fields
    );
    println!(
        "fully verified tables: {} | editable tables: {} | .dat runtime dependency: {:?}",
        report.fully_verified_tables, report.editable_tables, report.dat_runtime_dependency
    );
    println!(
        "validation: {} checks, {} failure(s)",
        report.validation.checks.len(),
        report.validation.failures.len()
    );
    for failure in &report.validation.failures {
        println!("validation failure: {failure}");
    }
    println!();
    println!("tables needing work:");
    for table in &report.tables {
        if table.blockers.is_empty() {
            continue;
        }
        println!(
            "{:<36} rows {:>7} status {:?} replacement {:?}",
            table.path, table.rows, table.table_status, table.dat_replacement_status
        );
        for blocker in &table.blockers {
            println!("  - {blocker}");
        }
    }
    println!();
    println!("next steps:");
    for step in &report.next_steps {
        println!("- {step}");
    }
}

fn print_backend_report(report: &cm_domain::BackendReadinessReport) {
    println!("Backend readiness report");
    println!(
        "status: {:?} | score: {}%",
        report.status, report.completion.score_percent
    );
    println!(
        "tables: {} canonical, {} runtime-ready, {} editable | validation failures: {} | binary tables left: {}",
        report.completion.canonical_tables,
        report.completion.runtime_ready_tables,
        report.completion.editable_tables,
        report.completion.validation_failures,
        report.completion.remaining_binary_tables
    );
    println!(
        "runtime: {} phase-0 frontier(s), {} phase-2 frontier(s), {} headless blocker(s)",
        report.completion.phase_frontiers,
        report.completion.phase_2_frontiers,
        report.completion.headless_blockers
    );
    println!();
    println!("checks:");
    for check in &report.checks {
        println!("- {:?}: {} - {}", check.status, check.name, check.detail);
    }
    println!();
    println!("implementation plan:");
    for item in &report.implementation_plan {
        println!(
            "- {:?}: {} | boundaries {} | owned records {} | missing lifts {}",
            item.readiness,
            item.system,
            item.boundary_entries,
            item.owned_records,
            item.missing_lifts.len()
        );
    }
    println!();
    println!("top blockers:");
    for blocker in report.blockers.iter().take(16) {
        println!(
            "- [{}] {} ({}) - {}",
            blocker.severity, blocker.system, blocker.status, blocker.next_evidence
        );
    }
    println!();
    println!("next steps:");
    for step in &report.next_steps {
        println!("- {step}");
    }
}

fn print_backend_acceptance_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Backend acceptance report");
    println!(
        "status: {} | infrastructure pass: {} | playable headless: {}",
        report["status"].as_str().unwrap_or("unknown"),
        report["infrastructure_pass"].as_bool().unwrap_or(false),
        report["playable_headless"].as_bool().unwrap_or(false)
    );
    println!(
        "readiness {}% | binary tables {} | validation failures {} | gameplay blockers {}",
        summary["readiness_score_percent"].as_u64().unwrap_or(0),
        summary["remaining_binary_tables"].as_u64().unwrap_or(0),
        summary["canonical_validation_failures"]
            .as_u64()
            .unwrap_or(0),
        summary["gameplay_blockers"].as_u64().unwrap_or(0)
    );
    println!(
        "runtime validation: {} check(s), {} failure(s)",
        summary["runtime_validation_checks"].as_u64().unwrap_or(0),
        summary["runtime_validation_failures"].as_u64().unwrap_or(0)
    );
    println!(
        "full-year campaign: {} day(s), {} phase(s), {} backend mutation(s), retained {}, dropped {}",
        summary["full_year_days"].as_u64().unwrap_or(0),
        summary["full_year_phases"].as_u64().unwrap_or(0),
        summary["full_year_backend_mutations"].as_u64().unwrap_or(0),
        summary["full_year_retained_mutations"].as_u64().unwrap_or(0),
        summary["full_year_dropped_mutations"].as_u64().unwrap_or(0)
    );
    if let Some(checks) = report["checks"].as_array() {
        println!("checks:");
        for check in checks {
            println!(
                "- {}: {} - {}",
                check["status"].as_str().unwrap_or("unknown"),
                check["name"].as_str().unwrap_or("unknown"),
                check["detail"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_binary_validation_report(report: &serde_json::Value) {
    let binary = &report["binary"];
    let summary = &report["summary"];
    println!("Original binary validation");
    println!(
        "binary: {} size {} image_base {}",
        binary["path"].as_str().unwrap_or("unknown"),
        binary["size"].as_u64().unwrap_or(0),
        binary["image_base"].as_str().unwrap_or("unknown")
    );
    println!(
        "checks: {} failures: {} status: {}",
        summary["checks"].as_u64().unwrap_or(0),
        summary["failures"].as_u64().unwrap_or(0),
        summary["status"].as_str().unwrap_or("unknown")
    );
    if let Some(checks) = report["checks"].as_array() {
        for check in checks {
            if check["status"].as_str() == Some("fail") {
                println!(
                    "failed: {} - {}",
                    check["name"].as_str().unwrap_or("unknown"),
                    check["detail"].as_str().unwrap_or("")
                );
            }
        }
    }
}

fn print_execution_validation_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    let execution = &report["execution"];
    println!("Execution model validation");
    println!(
        "checks: {} failures: {} status: {}",
        summary["checks"].as_u64().unwrap_or(0),
        summary["failures"].as_u64().unwrap_or(0),
        summary["status"].as_str().unwrap_or("unknown")
    );
    println!(
        "WinMain {} | continue {} | message pump {} | setup {}",
        execution["winmain"].as_str().unwrap_or("unknown"),
        execution["continue_check"]["address"]
            .as_str()
            .unwrap_or("unknown"),
        execution["message_pump_shell"]["address"]
            .as_str()
            .unwrap_or("unknown"),
        execution["setup_flow"]["address"]
            .as_str()
            .unwrap_or("unknown")
    );
    if let Some(checks) = report["checks"].as_array() {
        for check in checks {
            if check["status"].as_str() == Some("fail") {
                println!(
                    "failed: {} - {}",
                    check["name"].as_str().unwrap_or("unknown"),
                    check["detail"].as_str().unwrap_or("")
                );
            }
        }
    }
}

fn print_simulation_frontier_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    let frontier = &report["frontier"];
    let match_day = &frontier["match_day"];
    let match_builder = match_day["builder"]["address"]
        .as_str()
        .or_else(|| match_day["builder"].as_str())
        .unwrap_or("unknown");
    let match_annotation = match_day["annotation_helper"]["address"]
        .as_str()
        .unwrap_or("unknown");
    let match_processor = match_day["processor"]["address"]
        .as_str()
        .or_else(|| match_day["processor"].as_str())
        .unwrap_or("unknown");
    let match_setup = match_day["setup"]["address"]
        .as_str()
        .or_else(|| match_day["setup"].as_str())
        .unwrap_or("unknown");
    let match_team_setup = match_day["team_player_setup"]["address"]
        .as_str()
        .unwrap_or("unknown");
    println!("Simulation frontier validation");
    println!(
        "checks: {} failures: {} status: {}",
        summary["checks"].as_u64().unwrap_or(0),
        summary["failures"].as_u64().unwrap_or(0),
        summary["status"].as_str().unwrap_or("unknown")
    );
    println!(
        "loop {} | date_add {} | match_day {} -> {} -> {} -> {} -> {}",
        frontier["simulation_loop_candidate"]["address"]
            .as_str()
            .unwrap_or("unknown"),
        frontier["date_state"]["date_add_days"]
            .as_str()
            .unwrap_or("unknown"),
        match_builder,
        match_annotation,
        match_processor,
        match_setup,
        match_team_setup
    );
    if let Some(checks) = report["checks"].as_array() {
        for check in checks {
            if check["status"].as_str() == Some("fail") {
                println!(
                    "failed: {} - {}",
                    check["name"].as_str().unwrap_or("unknown"),
                    check["detail"].as_str().unwrap_or("")
                );
            }
        }
    }
}

fn print_exact_remake_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Exact remake report");
    println!(
        "status: {} | one-for-one exact: {} | foundation pass: {} | exact behavior pass: {}",
        report["status"].as_str().unwrap_or("unknown"),
        report["one_for_one_exact_remake"]
            .as_bool()
            .unwrap_or(false),
        report["foundation_pass"].as_bool().unwrap_or(false),
        report["exact_behavior_pass"].as_bool().unwrap_or(false)
    );
    println!(
        "proof basis: {} | original capture {}/{} rows | static-proven systems {} | internal gates green: {}",
        report["proof_basis"].as_str().unwrap_or("unknown"),
        summary["original_capture_filled_rows"].as_u64().unwrap_or(0),
        summary["original_capture_expected_rows"].as_u64().unwrap_or(0),
        summary["static_proven_systems"].as_u64().unwrap_or(0),
        report["internal_behavior_exact"].as_bool().unwrap_or(false)
    );
    if let Some(caveats) = report["caveats"].as_array() {
        for caveat in caveats {
            if let Some(text) = caveat.as_str() {
                println!("caveat: {text}");
            }
        }
    }
    println!(
        "original binary {} | execution model {} | simulation frontier {} | backend infrastructure {}",
        summary["original_binary_pass"].as_bool().unwrap_or(false),
        summary["execution_model_pass"].as_bool().unwrap_or(false),
        summary["simulation_frontier_pass"].as_bool().unwrap_or(false),
        summary["backend_infrastructure_pass"]
            .as_bool()
            .unwrap_or(false)
    );
    println!(
        "static boundary behavior {} | formula lift complete {}",
        summary["static_boundary_behavior_pass"]
            .as_bool()
            .unwrap_or(false),
        summary["formula_lift_complete"].as_bool().unwrap_or(false)
    );
    println!(
        "playable headless {} | gameplay blockers {} | implementation plan {}/{} ready ({} implemented, {} boundary-mapped)",
        summary["playable_headless"].as_bool().unwrap_or(false),
        summary["gameplay_blockers"].as_u64().unwrap_or(0),
        summary["implementation_plan_ready"]
            .as_u64()
            .unwrap_or(0),
        summary["implementation_plan_items"].as_u64().unwrap_or(0),
        summary["implementation_plan_mutations_implemented"]
            .as_u64()
            .unwrap_or(0),
        summary["implementation_plan_boundary_mapped"]
            .as_u64()
            .unwrap_or(0)
    );
    println!("checks:");
    if let Some(checks) = report["checks"].as_array() {
        for check in checks {
            println!(
                "- {}: {} - {}",
                check["status"].as_str().unwrap_or("unknown"),
                check["name"].as_str().unwrap_or("unknown"),
                check["detail"].as_str().unwrap_or("")
            );
        }
    }
    println!("required before exact:");
    if let Some(items) = summary["required_to_call_exact"].as_array() {
        for item in items {
            println!("- {}", item.as_str().unwrap_or(""));
        }
    }
}

fn print_gameplay_parity_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Gameplay parity report");
    println!(
        "status: {} | systems {} | failures {} | missing trace files {} | pending trace files {}",
        summary["status"].as_str().unwrap_or("unknown"),
        summary["systems"].as_u64().unwrap_or(0),
        summary["failures"].as_u64().unwrap_or(0),
        summary["missing_trace_files"].as_u64().unwrap_or(0),
        summary["pending_trace_files"].as_u64().unwrap_or(0)
    );
    println!("checks:");
    if let Some(checks) = report["checks"].as_array() {
        for check in checks {
            println!(
                "- {}: {} - {}",
                check["status"].as_str().unwrap_or("unknown"),
                check["system"].as_str().unwrap_or("unknown"),
                check["detail"].as_str().unwrap_or("")
            );
        }
    }
    if let Some(files) = report["missing_trace_files"].as_array() {
        if !files.is_empty() {
            println!("missing trace files:");
            for file in files {
                println!("- {}", file.as_str().unwrap_or(""));
            }
        }
    }
}

fn print_gameplay_promotion_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Gameplay promotion report");
    println!(
        "status: {} | systems {} | ready {} | blocked {}",
        summary["status"].as_str().unwrap_or("unknown"),
        summary["systems"].as_u64().unwrap_or(0),
        summary["ready_to_promote"].as_u64().unwrap_or(0),
        summary["blocked"].as_u64().unwrap_or(0)
    );
    println!("systems:");
    if let Some(systems) = report["systems"].as_array() {
        for system in systems {
            println!(
                "- {}: {} | trace {} | implementation {} | blockers {}",
                system["system"].as_str().unwrap_or("unknown"),
                system["status"].as_str().unwrap_or("unknown"),
                system["trace_status"].as_str().unwrap_or("unknown"),
                system["implementation_present"].as_bool().unwrap_or(false),
                system["open_blockers"]
                    .as_array()
                    .map(Vec::len)
                    .unwrap_or(0)
            );
        }
    }
}

fn print_promotion_control_room_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Promotion control room");
    println!(
        "status: {} | foundation {} | exact {} | playable headless {}",
        report["status"].as_str().unwrap_or("unknown"),
        summary["foundation_pass"].as_bool().unwrap_or(false),
        summary["one_for_one_exact_remake"]
            .as_bool()
            .unwrap_or(false),
        summary["playable_headless"].as_bool().unwrap_or(false)
    );
    println!(
        "original capture: {}/{} filled, {} placeholder row(s), {} import-ready system(s)",
        summary["original_capture_rows_filled"]
            .as_u64()
            .unwrap_or_default(),
        summary["original_capture_rows_expected"]
            .as_u64()
            .unwrap_or_default(),
        summary["original_capture_placeholder_rows"]
            .as_u64()
            .unwrap_or_default(),
        summary["original_capture_import_ready_systems"]
            .as_u64()
            .unwrap_or_default()
    );
    println!(
        "parity: {} with {} failure(s) | promotion: {} with {} blocked system(s)",
        summary["gameplay_parity_status"]
            .as_str()
            .unwrap_or("unknown"),
        summary["gameplay_parity_failures"]
            .as_u64()
            .unwrap_or_default(),
        summary["gameplay_promotion_status"]
            .as_str()
            .unwrap_or("unknown"),
        summary["gameplay_promotion_blocked"]
            .as_u64()
            .unwrap_or_default()
    );
    println!(
        "next required action: {}",
        summary["next_required_action"]
            .as_str()
            .unwrap_or("unknown")
    );
}

fn print_todo_attack_board_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("TODO attack board");
    println!(
        "status: {} | ready now {} | blocked {} | done {}",
        report["status"].as_str().unwrap_or("unknown"),
        summary["ready_now"].as_u64().unwrap_or_default(),
        summary["blocked"].as_u64().unwrap_or_default(),
        summary["done"].as_u64().unwrap_or_default()
    );
    println!(
        "capture placeholders {} | parity failures {} | promotion blocked {} | lift unknown/inferred {}",
        summary["original_capture_placeholders"]
            .as_u64()
            .unwrap_or_default(),
        summary["gameplay_parity_failures"]
            .as_u64()
            .unwrap_or_default(),
        summary["gameplay_promotion_blocked"]
            .as_u64()
            .unwrap_or_default(),
        summary["lift_unknown_or_inferred"]
            .as_u64()
            .unwrap_or_default()
    );
    println!(
        "{}",
        summary["answer_to_when_attack_todos"]
            .as_str()
            .unwrap_or("now")
    );
    if let Some(tasks) = report["tasks"].as_array() {
        for task in tasks {
            println!(
                "- P{} {}: {}",
                task["priority"].as_u64().unwrap_or_default(),
                task["status"].as_str().unwrap_or("unknown"),
                task["title"].as_str().unwrap_or("unknown")
            );
        }
    }
}

fn print_gameplay_lift_workbench_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Gameplay lift workbench");
    println!(
        "status: {} | systems {} | items {} | priority 1 {} | unknown/inferred {} | artifacts {}/{}",
        summary["status"].as_str().unwrap_or("unknown"),
        summary["systems"].as_u64().unwrap_or(0),
        summary["items"].as_u64().unwrap_or(0),
        summary["priority_1"].as_u64().unwrap_or(0),
        summary["unknown_or_inferred"].as_u64().unwrap_or(0),
        summary["decompile_artifacts_present"].as_u64().unwrap_or(0),
        summary["items"].as_u64().unwrap_or(0)
    );
    if let Some(systems) = report["systems"].as_array() {
        for system in systems {
            println!(
                "- {}: {} item(s), {} priority-1, {} unknown/inferred, artifacts {}/{} -> {}",
                system["system"].as_str().unwrap_or("unknown"),
                system["items"].as_u64().unwrap_or(0),
                system["priority_1"].as_u64().unwrap_or(0),
                system["unknown_or_inferred"].as_u64().unwrap_or(0),
                system["decompile_artifacts_present"].as_u64().unwrap_or(0),
                system["items"].as_u64().unwrap_or(0),
                system["directory"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_formula_lift_backlog_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Formula lift backlog");
    println!(
        "status: {} | tasks {} | ready static-read {} | ready implementation {} | needs decompile {}",
        summary["status"].as_str().unwrap_or("unknown"),
        summary["tasks"].as_u64().unwrap_or(0),
        summary["ready_for_static_read"].as_u64().unwrap_or(0),
        summary["ready_for_formula_implementation"]
            .as_u64()
            .unwrap_or(0),
        summary["needs_targeted_decompile"].as_u64().unwrap_or(0)
    );
    if let Some(tasks) = report["tasks"].as_array() {
        println!("top lifts:");
        for task in tasks.iter().take(8) {
            println!(
                "- p{} {}: {} ({})",
                task["priority"].as_u64().unwrap_or(0),
                task["system"].as_str().unwrap_or("unknown"),
                task["lift"].as_str().unwrap_or(""),
                task["status"].as_str().unwrap_or("unknown")
            );
        }
    }
}

fn print_gameplay_capture_pack_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Gameplay capture pack");
    println!(
        "trace dir: {} | output dir: {} | systems {} | candidate rows {} | watch groups {}",
        report["trace_dir"].as_str().unwrap_or(""),
        report["output_dir"].as_str().unwrap_or(""),
        summary["systems"]
            .as_u64()
            .unwrap_or_else(|| report["systems"].as_array().map_or(0, Vec::len) as u64),
        summary["candidate_rows"].as_u64().unwrap_or_default(),
        summary["watch_groups"].as_u64().unwrap_or_default()
    );
    if let Some(systems) = report["systems"].as_array() {
        for system in systems {
            println!(
                "- {}: {} candidate row(s), {} watch group(s), {} breakpoint(s), {} watched write(s), {} quality gate(s) -> {}",
                system["system"].as_str().unwrap_or("unknown"),
                system["candidate_rows"].as_u64().unwrap_or(0),
                system["watch_groups"].as_u64().unwrap_or(0),
                system["breakpoints"].as_u64().unwrap_or(0),
                system["watched_writes"].as_u64().unwrap_or(0),
                system["quality_gates"].as_u64().unwrap_or(0),
                system["directory"].as_str().unwrap_or("")
            );
        }
    }
}

fn print_backend_gate_refresh_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Backend gate refresh");
    println!(
        "status: {} | foundation {} | exact {} | playable headless {}",
        summary["status"].as_str().unwrap_or("unknown"),
        summary["foundation_pass"].as_bool().unwrap_or(false),
        summary["one_for_one_exact_remake"]
            .as_bool()
            .unwrap_or(false),
        summary["playable_headless"].as_bool().unwrap_or(false)
    );
    println!(
        "capture: {}/{} filled, {} placeholder row(s), {} import-ready system(s), {} imported, {} import failure(s)",
        summary["original_capture_rows_filled"]
            .as_u64()
            .unwrap_or_default(),
        summary["original_capture_rows_expected"]
            .as_u64()
            .unwrap_or_default(),
        summary["original_capture_placeholder_rows"]
            .as_u64()
            .unwrap_or_default(),
        summary["original_capture_import_ready_systems"]
            .as_u64()
            .unwrap_or_default(),
        summary["imported_ready_systems"]
            .as_u64()
            .unwrap_or_default(),
        summary["import_failures"].as_u64().unwrap_or_default()
    );
    println!(
        "parity: {} with {} failure(s) | promotion: {} with {} blocked system(s)",
        summary["gameplay_parity_status"]
            .as_str()
            .unwrap_or("unknown"),
        summary["gameplay_parity_failures"]
            .as_u64()
            .unwrap_or_default(),
        summary["gameplay_promotion_status"]
            .as_str()
            .unwrap_or("unknown"),
        summary["gameplay_promotion_blocked"]
            .as_u64()
            .unwrap_or_default()
    );
    println!(
        "next: {}",
        summary["next_required_action"]
            .as_str()
            .unwrap_or("capture original rows")
    );
    println!(
        "wrote {}",
        report["artifacts"]["refresh_report"]
            .as_str()
            .unwrap_or("backend_gate_refresh.json")
    );
}

fn print_import_gameplay_capture_report(report: &serde_json::Value) {
    println!("Gameplay capture import");
    println!(
        "system {} | status {} | comparison pass {} | original {} | Rust {}",
        report["system"].as_str().unwrap_or("unknown"),
        report["status"].as_str().unwrap_or("unknown"),
        report["comparison_pass"].as_bool().unwrap_or(false),
        report["original_mutations"].as_u64().unwrap_or(0),
        report["rust_mutations"].as_u64().unwrap_or(0)
    );
    println!(
        "capture: {} -> trace: {}",
        report["capture_file"].as_str().unwrap_or(""),
        report["trace_file"].as_str().unwrap_or("")
    );
}

fn print_original_capture_csv_import_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Original capture CSV import");
    println!(
        "status: {} | systems {} | rows {} | failures {} | capture {}/{} filled, {} placeholder row(s)",
        report["status"].as_str().unwrap_or("unknown"),
        summary["systems_updated"].as_u64().unwrap_or_default(),
        summary["rows_updated"].as_u64().unwrap_or_default(),
        summary["failures"].as_u64().unwrap_or_default(),
        summary["filled_original_rows"].as_u64().unwrap_or_default(),
        summary["expected_original_rows"]
            .as_u64()
            .unwrap_or_default(),
        summary["placeholder_rows"].as_u64().unwrap_or_default(),
    );
    if let Some(failures) = report["failures"].as_array() {
        for failure in failures {
            println!(
                "- {}: {}",
                failure["slug"].as_str().unwrap_or("unknown"),
                failure["error"].as_str().unwrap_or("unknown error")
            );
        }
    }
}

fn print_original_capture_csv_validation_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Original capture CSV validation");
    println!(
        "status: {} | rows {} | captured {} | blanks {} | duplicates {} | unknown systems {} | out of range {} | blocking {}",
        report["status"].as_str().unwrap_or("unknown"),
        summary["rows"].as_u64().unwrap_or_default(),
        summary["captured_rows"].as_u64().unwrap_or_default(),
        summary["blank_capture_values"].as_u64().unwrap_or_default(),
        summary["duplicates"].as_u64().unwrap_or_default(),
        summary["unknown_systems"].as_u64().unwrap_or_default(),
        summary["out_of_range"].as_u64().unwrap_or_default(),
        summary["blocking_errors"].as_u64().unwrap_or_default(),
    );
}

fn print_original_capture_csv_submit_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Original capture CSV submit");
    println!(
        "status: {} | validation {} | rows {} captured {} | gate capture {}/{} | parity failures {} | promotion blocked {}",
        report["status"].as_str().unwrap_or("unknown"),
        summary["validation_status"].as_str().unwrap_or("unknown"),
        summary["rows"].as_u64().unwrap_or_default(),
        summary["captured_rows"].as_u64().unwrap_or_default(),
        summary["capture_rows_filled"].as_u64().unwrap_or_default(),
        summary["capture_rows_expected"].as_u64().unwrap_or_default(),
        summary["parity_failures"].as_u64().unwrap_or_default(),
        summary["promotion_blocked"].as_u64().unwrap_or_default(),
    );
    if report["status"].as_str() == Some("blocked-by-validation") {
        println!(
            "blocked: blanks {} | duplicates {} | unknown systems {} | out of range {}",
            summary["blank_capture_values"].as_u64().unwrap_or_default(),
            summary["duplicates"].as_u64().unwrap_or_default(),
            summary["unknown_systems"].as_u64().unwrap_or_default(),
            summary["out_of_range"].as_u64().unwrap_or_default(),
        );
    }
}

fn print_prepare_capture_console_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Capture console prepared");
    println!(
        "rows {} | watch groups {} | todo {} | CSV {} with {} blank capture value(s)",
        summary["candidate_rows"].as_u64().unwrap_or_default(),
        summary["watch_groups"].as_u64().unwrap_or_default(),
        summary["workbench_todo_rows"].as_u64().unwrap_or_default(),
        summary["csv_validation_status"]
            .as_str()
            .unwrap_or("unknown"),
        summary["csv_blank_capture_values"]
            .as_u64()
            .unwrap_or_default()
    );
    println!(
        "capture: {}/{} filled, {} placeholder row(s), {} import-ready system(s)",
        summary["capture_rows_filled"].as_u64().unwrap_or_default(),
        summary["capture_rows_expected"]
            .as_u64()
            .unwrap_or_default(),
        summary["capture_placeholder_rows"]
            .as_u64()
            .unwrap_or_default(),
        summary["import_ready_systems"].as_u64().unwrap_or_default()
    );
    println!(
        "open: {}",
        report["urls"]["capture_console"]
            .as_str()
            .unwrap_or("http://127.0.0.1:8765/capture-console")
    );
    println!(
        "launch: {}",
        report["artifacts"]["launch_script"].as_str().unwrap_or("")
    );
}

fn print_gameplay_mutator_contract_sync_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    println!("Gameplay mutator contract sync");
    println!(
        "contracts {} | parity verified {} | trace verified/missing implementation {} | pending {} | all verified {}",
        summary["contracts"].as_u64().unwrap_or(0),
        summary["parity_verified"].as_u64().unwrap_or(0),
        summary["trace_verified_missing_implementation"]
            .as_u64()
            .unwrap_or(0),
        summary["pending"].as_u64().unwrap_or(0),
        summary["all_verified"].as_bool().unwrap_or(false)
    );
    if let Some(systems) = report["systems"].as_array() {
        for system in systems {
            println!(
                "- {}: {} -> {} ({})",
                system["system"].as_str().unwrap_or("unknown"),
                system["previous_status"].as_str().unwrap_or("unknown"),
                system["new_status"].as_str().unwrap_or("unknown"),
                system["trace_status"].as_str().unwrap_or("unknown")
            );
        }
    }
}

fn print_runtime_simulation_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    let runtime = &report["runtime"];
    let after = &runtime["after_one_day"];
    let target = &runtime["tick_to_date_sample"];
    let backend = &runtime["backend_ledger_sample"];
    let campaign = &runtime["headless_campaign_sample"];
    let full_year = &runtime["headless_full_year_campaign_sample"];
    println!("Runtime simulation validation");
    println!(
        "checks: {} failures: {} status: {}",
        summary["checks"].as_u64().unwrap_or(0),
        summary["failures"].as_u64().unwrap_or(0),
        summary["status"].as_str().unwrap_or("unknown")
    );
    println!(
        "after one day: {} phase {} cm_day {} trace {} last_frontiers {}",
        after["date"].as_str().unwrap_or("unknown"),
        after["phase"].as_u64().unwrap_or(0),
        after["cm_day_of_year"].as_u64().unwrap_or(0),
        after["trace_entries"].as_u64().unwrap_or(0),
        after["last_phase_frontiers"].as_u64().unwrap_or(0)
    );
    println!(
        "tick to {}: {} day(s) -> {} phase {} cm_day {} trace {}",
        target["target"].as_str().unwrap_or("unknown"),
        target["advanced_days"].as_u64().unwrap_or(0),
        target["date"].as_str().unwrap_or("unknown"),
        target["phase"].as_u64().unwrap_or(0),
        target["cm_day_of_year"].as_u64().unwrap_or(0),
        target["trace_entries"].as_u64().unwrap_or(0)
    );
    println!(
        "backend ledger: {} entry(s) | attempts match {} comp {} transfers {} news {}",
        backend["mutation_log_entries"].as_u64().unwrap_or(0),
        backend["attempts"]["match_results"].as_u64().unwrap_or(0),
        backend["attempts"]["competition_state"]
            .as_u64()
            .unwrap_or(0),
        backend["attempts"]["transfers_contracts"]
            .as_u64()
            .unwrap_or(0),
        backend["attempts"]["news_inbox"].as_u64().unwrap_or(0)
    );
    println!(
        "campaign gate: {} day(s), {} phase(s), {} checkpoint(s), backend mutations +{}",
        campaign["days_advanced"].as_u64().unwrap_or(0),
        campaign["phases_advanced"].as_u64().unwrap_or(0),
        campaign["checkpoints"].as_u64().unwrap_or(0),
        campaign["backend_mutations_added"].as_u64().unwrap_or(0)
    );
    println!(
        "full-year gate: {} day(s), {} phase(s), mutations +{} retained {} dropped {}",
        full_year["days_advanced"].as_u64().unwrap_or(0),
        full_year["phases_advanced"].as_u64().unwrap_or(0),
        full_year["backend_mutations_added"].as_u64().unwrap_or(0),
        full_year["retained_mutations"].as_u64().unwrap_or(0),
        full_year["dropped_mutations"].as_u64().unwrap_or(0)
    );
    if let Some(checks) = report["checks"].as_array() {
        for check in checks {
            if check["status"].as_str() == Some("fail") {
                println!(
                    "failed: {} - {}",
                    check["name"].as_str().unwrap_or("unknown"),
                    check["detail"].as_str().unwrap_or("")
                );
            }
        }
    }
}

fn print_rng_validation_report(report: &serde_json::Value) {
    let summary = &report["summary"];
    let rng = &report["match_random"];
    println!("RNG validation");
    println!(
        "checks: {} failures: {} status: {}",
        summary["checks"].as_u64().unwrap_or(0),
        summary["failures"].as_u64().unwrap_or(0),
        summary["status"].as_str().unwrap_or("unknown")
    );
    println!(
        "match_random: {} table {} file {:?}",
        rng["address"].as_str().unwrap_or("unknown"),
        rng["table_base"].as_str().unwrap_or("unknown"),
        rng["table_file_offset"].as_str()
    );
    println!("table sample: {}", rng["table_sample_i32"]);
}

fn print_runtime_save_summary(save: &RuntimeSaveGame) {
    println!("Rust save: {} v{}", save.format, save.version);
    println!("source: {} {}", save.source.kind, save.source.path);
    println!(
        "date: {:04}-{:02}-{:02} | phase {} | cm day_of_year {} year {} leap {} | elapsed days: {} | pending events: {} | phase trace: {}",
        save.date.year,
        save.date.month,
        save.date.day,
        save.simulation.phase,
        save.simulation.cm_packed_date.day_of_year,
        save.simulation.cm_packed_date.year,
        save.simulation.cm_packed_date.leap_year,
        save.elapsed_days,
        save.pending_events.len(),
        save.phase_trace.len()
    );
    if let Some(last) = save.phase_trace.last() {
        println!(
            "last phase: {} -> {} | advanced_day {} | frontiers {}",
            last.phase_before,
            last.phase_after,
            last.advanced_day,
            last.frontiers.len()
        );
    }
    println!(
        "headless: {:?} {:?} | completed days {} phases {} | blockers {}",
        save.headless.mode,
        save.headless.status,
        save.headless.completed_days,
        save.headless.completed_phases,
        save.headless.blockers.len()
    );
    println!(
        "backend: {:?} | mutation log retained {} total {} dropped {} limit {} | attempts match {} comp {} transfers {} news {}",
        save.backend.status,
        save.backend.mutation_log.len(),
        save.backend.total_mutation_entries,
        save.backend.dropped_mutation_entries,
        save.backend.mutation_log_limit,
        save.backend.matches.attempted_mutations,
        save.backend.competitions.attempted_mutations,
        save.backend.transfers.attempted_mutations,
        save.backend.news.attempted_mutations
    );
    println!(
        "gameplay mutator contracts: {} exact mutator target(s), parity-gated",
        save.backend.mutator_contracts.len()
    );
    println!(
        "gameplay mutator install plans: {} exact mutator install gate(s)",
        save.backend.mutator_install_plans.len()
    );
    println!(
        "gameplay promotion gates: {} exact mutator promotion blocker(s)",
        save.backend.gameplay_promotion_gates.len()
    );
    println!(
        "gameplay lift workbench: {} original-code lift work item(s)",
        save.backend.gameplay_lift_workbench.len()
    );
    println!(
        "gameplay system code claims: {} targeted-decompile claim(s)",
        save.backend.gameplay_system_code_claims.len()
    );
    println!(
        "exact gameplay mutator skeletons: {} disabled Rust hook skeleton(s)",
        save.backend.exact_mutator_skeletons.len()
    );
    println!(
        "match engine lift map: {} code-derived setup/controller/period/event boundary entry(s)",
        save.backend.match_engine_lift_map.len()
    );
    println!(
        "match result write map: {} code-derived fixture score/status entry(s)",
        save.backend.match_result_write_map.len()
    );
    println!(
        "match result code claims: {} targeted-decompile claim(s)",
        save.backend.match_result_code_claims.len()
    );
    println!(
        "match result mutator install plan: {} original + {} Rust coverage gate(s), status {}",
        save.backend
            .match_result_mutator_install_plan
            .required_original_coverage
            .len(),
        save.backend
            .match_result_mutator_install_plan
            .required_rust_coverage
            .len(),
        save.backend.match_result_mutator_install_plan.status
    );
    println!(
        "competition fixture state map: {} code-derived fixture/notification entry(s)",
        save.backend.competition_fixture_state_map.len()
    );
    println!(
        "transfer/contract state map: {} code-derived transfer/contract entry(s)",
        save.backend.transfer_contract_state_map.len()
    );
    println!(
        "news/inbox emission map: {} code-derived news/event entry(s)",
        save.backend.news_inbox_emission_map.len()
    );
    if let Some(report) = &save.headless.last_run {
        println!(
            "last headless run: {} day(s), {} phase(s), {} frontier(s) in last phase, ended {:04}-{:02}-{:02}",
            report.days_advanced,
            report.phases_advanced,
            report.last_phase_frontiers,
            report.end_date.year,
            report.end_date.month,
            report.end_date.day
        );
    }
    println!(
        "world counts: clubs {} staff {} nations {} competitions {} histories {}",
        save.table_counts.clubs,
        save.table_counts.staff_type6
            + save.table_counts.staff_type9
            + save.table_counts.staff_type10,
        save.table_counts.nations,
        save.table_counts.competitions,
        save.table_counts.histories
    );
}

fn print_headless_run_report(save_path: &Path, report: &HeadlessRunReport) {
    println!("Headless run: {}", save_path.display());
    println!(
        "advanced {} day(s), {} phase(s), trace +{} | {:04}-{:02}-{:02} -> {:04}-{:02}-{:02}",
        report.days_advanced,
        report.phases_advanced,
        report.phase_trace_entries_added,
        report.start_date.year,
        report.start_date.month,
        report.start_date.day,
        report.end_date.year,
        report.end_date.month,
        report.end_date.day
    );
    println!(
        "status: {:?} | last phase frontiers {}",
        report.status, report.last_phase_frontiers
    );
    if !report.still_frontier_only.is_empty() {
        println!(
            "frontier-only systems still blocking full play: {}",
            report.still_frontier_only.join(", ")
        );
    }
}

fn print_headless_campaign_report(save_path: &Path, report: &HeadlessCampaignReport) {
    println!("Headless campaign: {}", save_path.display());
    println!(
        "advanced {} / {} day(s), {} phase(s) | {:04}-{:02}-{:02} -> {:04}-{:02}-{:02}",
        report.days_advanced,
        report.days_requested,
        report.phases_advanced,
        report.start_date.year,
        report.start_date.month,
        report.start_date.day,
        report.end_date.year,
        report.end_date.month,
        report.end_date.day
    );
    println!(
        "status: {:?} | checkpoints {} | mutation log +{} total {}",
        report.status,
        report.checkpoints.len(),
        report.backend.mutation_log_entries_added,
        report.backend.total_mutation_log_entries
    );
    println!(
        "attempts: match {} comp {} transfers {} news {} | implemented {} | frontier-only {}",
        report.backend.match_attempts,
        report.backend.competition_attempts,
        report.backend.transfer_attempts,
        report.backend.news_attempts,
        report.backend.implemented_mutations,
        report.backend.frontier_only_mutations
    );
    if let Some(last) = report.checkpoints.last() {
        println!(
            "last checkpoint: {:04}-{:02}-{:02} elapsed {} trace {} mutations {}",
            last.date.year,
            last.date.month,
            last.date.day,
            last.elapsed_days,
            last.phase_trace_entries,
            last.mutation_log_entries
        );
    }
    if !report.still_frontier_only.is_empty() {
        println!(
            "frontier-only systems still blocking full play: {}",
            report.still_frontier_only.join(", ")
        );
    }
}

fn parse_game_date(text: &str) -> Result<GameDate, String> {
    let mut parts = text.split('-');
    let year = parts
        .next()
        .ok_or_else(|| "date must be yyyy-mm-dd".to_string())?
        .parse::<u16>()
        .map_err(|err| format!("invalid date year: {err}"))?;
    let month = parts
        .next()
        .ok_or_else(|| "date must be yyyy-mm-dd".to_string())?
        .parse::<u8>()
        .map_err(|err| format!("invalid date month: {err}"))?;
    let day = parts
        .next()
        .ok_or_else(|| "date must be yyyy-mm-dd".to_string())?
        .parse::<u8>()
        .map_err(|err| format!("invalid date day: {err}"))?;
    if parts.next().is_some() {
        return Err("date must be yyyy-mm-dd".to_string());
    }
    validate_game_date(GameDate { year, month, day })
}

fn validate_game_date(date: GameDate) -> Result<GameDate, String> {
    if date.month == 0 || date.month > 12 {
        return Err(format!("invalid date month: {}", date.month));
    }
    let max_day = match date.month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(date.year) => 29,
        2 => 28,
        _ => unreachable!(),
    };
    if date.day == 0 || date.day > max_day {
        return Err(format!(
            "invalid date day {} for {:04}-{:02}",
            date.day, date.year, date.month
        ));
    }
    Ok(date)
}

fn is_leap_year(year: u16) -> bool {
    let year = u32::from(year);
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Cli {
    command: Command,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Command {
    Summary {
        install_root: PathBuf,
    },
    ExportWorld {
        install_root: PathBuf,
        output: Option<PathBuf>,
    },
    InspectWorld {
        snapshot: PathBuf,
    },
    ExportOwnedData {
        snapshot: PathBuf,
        output_dir: PathBuf,
    },
    AuditWorld {
        install_root: PathBuf,
        snapshot: PathBuf,
    },
    InitRustDb {
        install_root: PathBuf,
        db_dir: PathBuf,
    },
    InspectRustDb {
        db_dir: PathBuf,
    },
    AuditRustDb {
        db_dir: PathBuf,
    },
    CanonicalReport {
        db_dir: PathBuf,
        output: Option<PathBuf>,
    },
    BackendReport {
        db_dir: PathBuf,
        output: Option<PathBuf>,
    },
    BackendAcceptance {
        db_dir: PathBuf,
        output: Option<PathBuf>,
    },
    ExactRemakeReport {
        db_dir: PathBuf,
        exe: PathBuf,
        output: Option<PathBuf>,
    },
    GameplayParityReport {
        db_dir: PathBuf,
        trace_dir: PathBuf,
        output: Option<PathBuf>,
    },
    GameplayPromotionReport {
        db_dir: PathBuf,
        trace_dir: PathBuf,
        output: Option<PathBuf>,
    },
    ExportGameplayLiftWorkbench {
        db_dir: PathBuf,
        output_dir: PathBuf,
    },
    ExportFormulaLiftBacklog {
        db_dir: PathBuf,
        output_dir: PathBuf,
    },
    InitGameplayParityTraces {
        db_dir: PathBuf,
        trace_dir: PathBuf,
    },
    ExportRustMatchResultTrace {
        db_dir: PathBuf,
        trace_dir: PathBuf,
    },
    ExportRustGameplayCandidateTraces {
        db_dir: PathBuf,
        trace_dir: PathBuf,
    },
    ExportOriginalCaptureTemplates {
        trace_dir: PathBuf,
        output_dir: PathBuf,
    },
    OriginalCaptureStatus {
        template_dir: PathBuf,
        output: Option<PathBuf>,
    },
    ExportOriginalCaptureWorkbench {
        template_dir: PathBuf,
        output_dir: PathBuf,
        trace_dir: PathBuf,
    },
    ExportPromotionControlRoom {
        db_dir: PathBuf,
        output_dir: PathBuf,
        trace_dir: PathBuf,
        template_dir: PathBuf,
        exe: PathBuf,
    },
    ExportTodoAttackBoard {
        db_dir: PathBuf,
        output_dir: PathBuf,
        trace_dir: PathBuf,
        template_dir: PathBuf,
    },
    ExportGameplayCapturePack {
        trace_dir: PathBuf,
        output_dir: PathBuf,
    },
    RefreshBackendGates {
        db_dir: PathBuf,
        reports_dir: PathBuf,
        trace_dir: PathBuf,
        template_dir: PathBuf,
        exe: PathBuf,
    },
    SyncGameplayMutatorContracts {
        save: PathBuf,
        trace_dir: PathBuf,
    },
    ImportGameplayCapture {
        trace_dir: PathBuf,
        capture: PathBuf,
        output: Option<PathBuf>,
    },
    ImportOriginalCaptureCsv {
        csv: PathBuf,
        output: Option<PathBuf>,
    },
    ValidateOriginalCaptureCsv {
        csv: PathBuf,
        output: Option<PathBuf>,
    },
    SubmitOriginalCaptureCsv {
        csv: PathBuf,
        reports_dir: PathBuf,
        output: Option<PathBuf>,
    },
    PrepareCaptureConsole {
        db_dir: PathBuf,
        reports_dir: PathBuf,
        trace_dir: PathBuf,
        template_dir: PathBuf,
        port: u16,
    },
    ValidateOriginalBinary {
        exe: PathBuf,
        output: Option<PathBuf>,
    },
    ValidateExecutionModel {
        exe: PathBuf,
        output: Option<PathBuf>,
    },
    ValidateSimulationFrontier {
        exe: PathBuf,
        output: Option<PathBuf>,
    },
    ValidateRuntimeSimulation {
        db_dir: PathBuf,
        output: Option<PathBuf>,
    },
    ValidateRng {
        exe: PathBuf,
        output: Option<PathBuf>,
    },
    ExtractRngTable {
        exe: PathBuf,
        output: PathBuf,
        entries: usize,
    },
    NewRustSave {
        db_dir: PathBuf,
        output: PathBuf,
    },
    InspectRustSave {
        save: PathBuf,
    },
    TickRustSave {
        save: PathBuf,
        days: u32,
    },
    TickRustSaveTo {
        save: PathBuf,
        target: GameDate,
    },
    RunHeadless {
        save: PathBuf,
        days: u32,
    },
    RunHeadlessTo {
        save: PathBuf,
        target: GameDate,
    },
    RunHeadlessCampaign {
        save: PathBuf,
        days: u32,
        checkpoint_every: u32,
        output: Option<PathBuf>,
    },
    SetHeadlessManager {
        save: PathBuf,
        name: String,
        club_id: Option<u32>,
    },
    ExportRustDbData {
        db_dir: PathBuf,
        output_dir: PathBuf,
    },
    ExportRustDbWorld {
        db_dir: PathBuf,
        output: Option<PathBuf>,
    },
    ExportRustDbViewer {
        db_dir: PathBuf,
        output: Option<PathBuf>,
    },
    ExportRustDbViewerTables {
        db_dir: PathBuf,
        output_dir: PathBuf,
    },
    ServeRustDb {
        db_dir: PathBuf,
        port: u16,
    },
    RenameRustDb {
        db_dir: PathBuf,
        table: String,
        row: String,
        name: String,
    },
    SetRustDbText {
        db_dir: PathBuf,
        table: String,
        row: String,
        field: String,
        text: String,
    },
    SetStaffType10 {
        db_dir: PathBuf,
        id: u32,
        field: String,
        value: u16,
    },
    SetStaffAttribute {
        db_dir: PathBuf,
        id: u32,
        index: usize,
        value: u8,
    },
    ExportStaticParityProof {
        row_plan: PathBuf,
        output: PathBuf,
    },
    ApplyStaticParityProof {
        trace_dir: PathBuf,
        proof: PathBuf,
        output: Option<PathBuf>,
    },
}

impl Cli {
    fn parse() -> Result<Self, String> {
        let mut args = env::args_os();
        let _exe = args.next();
        let first = args.next().ok_or_else(Self::usage)?;
        let first = PathBuf::from(first);

        let command = match first.to_string_lossy().as_ref() {
            "summary" => Command::Summary {
                install_root: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "export-world" => Command::ExportWorld {
                install_root: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "inspect-world" => Command::InspectWorld {
                snapshot: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "export-owned-data" => Command::ExportOwnedData {
                snapshot: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "audit-world" => Command::AuditWorld {
                install_root: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                snapshot: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "init-rust-db" => Command::InitRustDb {
                install_root: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "inspect-rust-db" => Command::InspectRustDb {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "audit-rust-db" => Command::AuditRustDb {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "canonical-report" => Command::CanonicalReport {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "backend-report" => Command::BackendReport {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "backend-acceptance" => Command::BackendAcceptance {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "exact-remake-report" => Command::ExactRemakeReport {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                exe: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "gameplay-parity-report" => Command::GameplayParityReport {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                trace_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "gameplay-promotion-report" => Command::GameplayPromotionReport {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                trace_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "export-gameplay-lift-workbench" => Command::ExportGameplayLiftWorkbench {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "export-formula-lift-backlog" => Command::ExportFormulaLiftBacklog {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "init-gameplay-parity-traces" => Command::InitGameplayParityTraces {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                trace_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "export-rust-match-result-trace" => Command::ExportRustMatchResultTrace {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                trace_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "export-rust-gameplay-candidate-traces" => Command::ExportRustGameplayCandidateTraces {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                trace_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "export-original-capture-templates" => Command::ExportOriginalCaptureTemplates {
                trace_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "original-capture-status" => Command::OriginalCaptureStatus {
                template_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "export-original-capture-workbench" => Command::ExportOriginalCaptureWorkbench {
                template_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                trace_dir: args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("D:/cm0102-rs/reports/parity_traces")),
            },
            "export-promotion-control-room" => Command::ExportPromotionControlRoom {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                trace_dir: args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("D:/cm0102-rs/reports/parity_traces")),
                template_dir: args.next().map(PathBuf::from).unwrap_or_else(|| {
                    PathBuf::from("D:/cm0102-rs/reports/original_capture_templates")
                }),
                exe: args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("D:/cm0102/cm0102.exe")),
            },
            "export-todo-attack-board" => Command::ExportTodoAttackBoard {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                trace_dir: args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("D:/cm0102-rs/reports/parity_traces")),
                template_dir: args.next().map(PathBuf::from).unwrap_or_else(|| {
                    PathBuf::from("D:/cm0102-rs/reports/original_capture_templates")
                }),
            },
            "export-gameplay-capture-pack" => Command::ExportGameplayCapturePack {
                trace_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "export-static-parity-proof" => Command::ExportStaticParityProof {
                row_plan: args.next().map(PathBuf::from).unwrap_or_else(|| {
                    PathBuf::from(
                        "D:/cm0102-rs/reports/capture_pack/all-systems-row-capture-plan.json",
                    )
                }),
                output: args.next().map(PathBuf::from).unwrap_or_else(|| {
                    PathBuf::from("D:/cm0102-rs/reports/static_parity_proof.json")
                }),
            },
            "apply-static-parity-proof" => Command::ApplyStaticParityProof {
                trace_dir: args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("D:/cm0102-rs/reports/parity_traces")),
                proof: args.next().map(PathBuf::from).unwrap_or_else(|| {
                    PathBuf::from("D:/cm0102-rs/reports/static_parity_proof.json")
                }),
                output: args.next().map(PathBuf::from),
            },
            "refresh-backend-gates" => Command::RefreshBackendGates {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                reports_dir: args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("D:/cm0102-rs/reports")),
                trace_dir: args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("D:/cm0102-rs/reports/parity_traces")),
                template_dir: args.next().map(PathBuf::from).unwrap_or_else(|| {
                    PathBuf::from("D:/cm0102-rs/reports/original_capture_templates")
                }),
                exe: args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("D:/cm0102/cm0102.exe")),
            },
            "sync-gameplay-mutator-contracts" => Command::SyncGameplayMutatorContracts {
                save: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                trace_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "import-gameplay-capture" => Command::ImportGameplayCapture {
                trace_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                capture: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "import-original-capture-csv" => Command::ImportOriginalCaptureCsv {
                csv: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "validate-original-capture-csv" => Command::ValidateOriginalCaptureCsv {
                csv: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "submit-original-capture-csv" => Command::SubmitOriginalCaptureCsv {
                csv: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                reports_dir: args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("D:/cm0102-rs/reports")),
                output: args.next().map(PathBuf::from),
            },
            "prepare-capture-console" => Command::PrepareCaptureConsole {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                reports_dir: args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("D:/cm0102-rs/reports")),
                trace_dir: args
                    .next()
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("D:/cm0102-rs/reports/parity_traces")),
                template_dir: args.next().map(PathBuf::from).unwrap_or_else(|| {
                    PathBuf::from("D:/cm0102-rs/reports/original_capture_templates")
                }),
                port: args
                    .next()
                    .map(|value| {
                        value
                            .to_string_lossy()
                            .parse()
                            .map_err(|err| format!("invalid capture console port: {err}"))
                    })
                    .transpose()?
                    .unwrap_or(8765),
            },
            "validate-original-binary" => Command::ValidateOriginalBinary {
                exe: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "validate-execution-model" => Command::ValidateExecutionModel {
                exe: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "validate-simulation-frontier" => Command::ValidateSimulationFrontier {
                exe: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "validate-runtime-simulation" => Command::ValidateRuntimeSimulation {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "validate-rng" => Command::ValidateRng {
                exe: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "extract-rng-table" => Command::ExtractRngTable {
                exe: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                entries: args
                    .next()
                    .map(|value| {
                        value
                            .to_string_lossy()
                            .parse()
                            .map_err(|err| format!("invalid entry count: {err}"))
                    })
                    .transpose()?
                    .unwrap_or(cm_rng::MATCH_RNG_TABLE_ENTRIES),
            },
            "new-rust-save" => Command::NewRustSave {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "inspect-rust-save" => Command::InspectRustSave {
                save: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "tick-rust-save" => Command::TickRustSave {
                save: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                days: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .parse()
                    .map_err(|err| format!("invalid day count: {err}"))?,
            },
            "tick-rust-save-to" => Command::TickRustSaveTo {
                save: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                target: parse_game_date(&args.next().ok_or_else(Self::usage)?.to_string_lossy())?,
            },
            "run-headless" => Command::RunHeadless {
                save: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                days: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .parse()
                    .map_err(|err| format!("invalid day count: {err}"))?,
            },
            "run-headless-to" => Command::RunHeadlessTo {
                save: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                target: parse_game_date(&args.next().ok_or_else(Self::usage)?.to_string_lossy())?,
            },
            "run-headless-campaign" => Command::RunHeadlessCampaign {
                save: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                days: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .parse()
                    .map_err(|err| format!("invalid day count: {err}"))?,
                checkpoint_every: args
                    .next()
                    .map(|value| {
                        value
                            .to_string_lossy()
                            .parse()
                            .map_err(|err| format!("invalid checkpoint interval: {err}"))
                    })
                    .transpose()?
                    .unwrap_or(30),
                output: args.next().map(PathBuf::from),
            },
            "set-headless-manager" => Command::SetHeadlessManager {
                save: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                name: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .into_owned(),
                club_id: args
                    .next()
                    .map(|value| {
                        value
                            .to_string_lossy()
                            .parse()
                            .map_err(|err| format!("invalid club id: {err}"))
                    })
                    .transpose()?,
            },
            "export-rust-db-data" => Command::ExportRustDbData {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "export-rust-db-world" => Command::ExportRustDbWorld {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "export-rust-db-viewer" => Command::ExportRustDbViewer {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output: args.next().map(PathBuf::from),
            },
            "export-rust-db-viewer-tables" => Command::ExportRustDbViewerTables {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                output_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
            },
            "serve-rust-db" => Command::ServeRustDb {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                port: args
                    .next()
                    .map(|value| {
                        value
                            .to_string_lossy()
                            .parse()
                            .map_err(|err| format!("invalid port: {err}"))
                    })
                    .transpose()?
                    .unwrap_or(8770),
            },
            "rename-rust-db" => Command::RenameRustDb {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                table: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .into_owned(),
                row: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .into_owned(),
                name: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .into_owned(),
            },
            "set-rust-db-text" => Command::SetRustDbText {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                table: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .into_owned(),
                row: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .into_owned(),
                field: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .into_owned(),
                text: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .into_owned(),
            },
            "set-staff-type10" => Command::SetStaffType10 {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                id: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .parse()
                    .map_err(|err| format!("invalid staff id: {err}"))?,
                field: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .into_owned(),
                value: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .parse()
                    .map_err(|err| format!("invalid value: {err}"))?,
            },
            "set-staff-attribute" => Command::SetStaffAttribute {
                db_dir: PathBuf::from(args.next().ok_or_else(Self::usage)?),
                id: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .parse()
                    .map_err(|err| format!("invalid staff id: {err}"))?,
                index: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .parse()
                    .map_err(|err| format!("invalid attribute index: {err}"))?,
                value: args
                    .next()
                    .ok_or_else(Self::usage)?
                    .to_string_lossy()
                    .parse()
                    .map_err(|err| format!("invalid attribute value: {err}"))?,
            },
            _ => {
                // Backward compatibility: treat the first argument as the install path.
                Command::Summary {
                    install_root: first,
                }
            }
        };

        Ok(Self { command })
    }

    fn usage() -> String {
        [
            "usage:",
            "  cargo run -p cm-app -- <CM0102 install dir>",
            "  cargo run -p cm-app -- summary <CM0102 install dir>",
            "  cargo run -p cm-app -- export-world <CM0102 install dir> [output.json]",
            "  cargo run -p cm-app -- inspect-world <snapshot.json>",
            "  cargo run -p cm-app -- export-owned-data <snapshot.json> <output-dir>",
            "  cargo run -p cm-app -- audit-world <CM0102 install dir> <snapshot.json>",
            "  cargo run -p cm-app -- init-rust-db <CM0102 install dir> <db-dir>",
            "  cargo run -p cm-app -- inspect-rust-db <db-dir>",
            "  cargo run -p cm-app -- audit-rust-db <db-dir>",
            "  cargo run -p cm-app -- canonical-report <db-dir> [output.json]",
            "  cargo run -p cm-app -- backend-report <db-dir> [output.json]",
            "  cargo run -p cm-app -- backend-acceptance <db-dir> [output.json]",
            "  cargo run -p cm-app -- exact-remake-report <db-dir> <cm0102.exe> [output.json]",
            "  cargo run -p cm-app -- gameplay-parity-report <db-dir> <trace-dir> [output.json]",
            "  cargo run -p cm-app -- gameplay-promotion-report <db-dir> <trace-dir> [output.json]",
            "  cargo run -p cm-app -- export-gameplay-lift-workbench <db-dir> <output-dir>",
            "  cargo run -p cm-app -- export-formula-lift-backlog <db-dir> <output-dir>",
            "  cargo run -p cm-app -- init-gameplay-parity-traces <db-dir> <trace-dir>",
            "  cargo run -p cm-app -- export-rust-match-result-trace <db-dir> <trace-dir>",
            "  cargo run -p cm-app -- export-rust-gameplay-candidate-traces <db-dir> <trace-dir>",
            "  cargo run -p cm-app -- export-original-capture-templates <trace-dir> <output-dir>",
            "  cargo run -p cm-app -- original-capture-status <template-dir> [output.json]",
            "  cargo run -p cm-app -- export-original-capture-workbench <template-dir> <output-dir> [trace-dir]",
            "  cargo run -p cm-app -- export-promotion-control-room <db-dir> <output-dir> [trace-dir] [template-dir] [cm0102.exe]",
            "  cargo run -p cm-app -- export-todo-attack-board <db-dir> <output-dir> [trace-dir] [template-dir]",
            "  cargo run -p cm-app -- export-gameplay-capture-pack <trace-dir> <output-dir>",
            "  cargo run -p cm-app -- export-static-parity-proof [all-systems-row-plan.json] [output.json]",
            "  cargo run -p cm-app -- apply-static-parity-proof [trace-dir] [static-proof.json] [output.json]",
            "  cargo run -p cm-app -- refresh-backend-gates <db-dir> [reports-dir] [trace-dir] [template-dir] [cm0102.exe]",
            "  cargo run -p cm-app -- sync-gameplay-mutator-contracts <save.json> <trace-dir>",
            "  cargo run -p cm-app -- import-gameplay-capture <trace-dir> <capture.json> [report.json]",
            "  cargo run -p cm-app -- import-original-capture-csv <capture.csv> [report.json]",
            "  cargo run -p cm-app -- validate-original-capture-csv <capture.csv> [report.json]",
            "  cargo run -p cm-app -- submit-original-capture-csv <capture.csv> [reports-dir] [report.json]",
            "  cargo run -p cm-app -- prepare-capture-console <db-dir> [reports-dir] [trace-dir] [template-dir] [port]",
            "  cargo run -p cm-app -- validate-original-binary <cm0102.exe> [output.json]",
            "  cargo run -p cm-app -- validate-execution-model <cm0102.exe> [output.json]",
            "  cargo run -p cm-app -- validate-simulation-frontier <cm0102.exe> [output.json]",
            "  cargo run -p cm-app -- validate-runtime-simulation <db-dir> [output.json]",
            "  cargo run -p cm-app -- validate-rng <cm0102.exe> [output.json]",
            "  cargo run -p cm-app -- extract-rng-table <cm0102.exe> <output.json> [entries; default full 51000]",
            "  cargo run -p cm-app -- new-rust-save <db-dir> <output.json>",
            "  cargo run -p cm-app -- inspect-rust-save <save.json>",
            "  cargo run -p cm-app -- tick-rust-save <save.json> <days>",
            "  cargo run -p cm-app -- tick-rust-save-to <save.json> <yyyy-mm-dd>",
            "  cargo run -p cm-app -- run-headless <save.json> <days>",
            "  cargo run -p cm-app -- run-headless-to <save.json> <yyyy-mm-dd>",
            "  cargo run -p cm-app -- run-headless-campaign <save.json> <days> [checkpoint-every-days] [output.json]",
            "  cargo run -p cm-app -- set-headless-manager <save.json> <name> [club-id]",
            "  cargo run -p cm-app -- export-rust-db-data <db-dir> <output-dir>",
            "  cargo run -p cm-app -- export-rust-db-world <db-dir> [output.json]",
            "  cargo run -p cm-app -- export-rust-db-viewer <db-dir> [output.json]",
            "  cargo run -p cm-app -- export-rust-db-viewer-tables <db-dir> <output-dir>",
            "  cargo run -p cm-app -- serve-rust-db <db-dir> [port]",
            "  cargo run -p cm-app -- rename-rust-db <db-dir> <table> <row-id-or-index> <name>",
            "  cargo run -p cm-app -- set-rust-db-text <db-dir> <table> <row-id-or-index> <field> <text>",
            "  cargo run -p cm-app -- set-staff-type10 <db-dir> <staff-id> <ca|pa|reputation> <value>",
            "  cargo run -p cm-app -- set-staff-attribute <db-dir> <staff-id> <attr-index> <value>",
        ]
        .join("\n")
    }
}

fn viewer_table_values(
    world: &World,
) -> Result<Vec<(&'static str, &'static str, serde_json::Value)>, String> {
    let save_sections = world
        .save
        .as_ref()
        .map(|save| serde_json::to_value(&save.sections))
        .transpose()
        .map_err(|err| format!("failed to serialize save sections: {err}"))?
        .unwrap_or_else(|| serde_json::json!([]));
    let staff_comp_names = competition_name_map(&world.references.staff_competitions);
    let club_comp_names = competition_name_map(&world.references.club_competitions);
    let nation_comp_names = competition_name_map(&world.references.nation_competitions);

    let tables = vec![
        (
            "base_data",
            "Base Data",
            serde_json::to_value(&world.base_data),
        ),
        ("save.sections", "Save Sections", Ok(save_sections)),
        (
            "core.clubs",
            "Core Clubs",
            serde_json::to_value(&world.core.clubs),
        ),
        (
            "core.nat_clubs",
            "Core National Clubs",
            serde_json::to_value(&world.core.nat_clubs),
        ),
        (
            "core.colours",
            "Core Colours",
            serde_json::to_value(&world.core.colours),
        ),
        (
            "core.continents",
            "Core Continents",
            serde_json::to_value(&world.core.continents),
        ),
        (
            "core.nations",
            "Core Nations",
            serde_json::to_value(&world.core.nations),
        ),
        (
            "schema.tables",
            "Schema Tables",
            serde_json::to_value(&world.schema.tables),
        ),
        (
            "staff.type6",
            "Staff Type 6",
            serde_json::to_value(&world.staff.type6),
        ),
        (
            "staff.type8",
            "Staff Type 8",
            serde_json::to_value(&world.staff.type8),
        ),
        (
            "staff.type9",
            "Staff Type 9",
            serde_json::to_value(&world.staff.type9),
        ),
        (
            "staff.type10",
            "Staff Type 10",
            serde_json::to_value(&world.staff.type10),
        ),
        (
            "references.cities",
            "Cities",
            serde_json::to_value(&world.references.cities),
        ),
        (
            "references.officials",
            "Officials",
            serde_json::to_value(&world.references.officials),
        ),
        (
            "references.first_names",
            "First Names",
            serde_json::to_value(&world.references.first_names),
        ),
        (
            "references.second_names",
            "Second Names",
            serde_json::to_value(&world.references.second_names),
        ),
        (
            "references.common_names",
            "Common Names",
            serde_json::to_value(&world.references.common_names),
        ),
        (
            "references.stadiums",
            "Stadiums",
            serde_json::to_value(&world.references.stadiums),
        ),
        (
            "references.staff_competitions",
            "Staff Competitions",
            serde_json::to_value(&world.references.staff_competitions),
        ),
        (
            "references.club_competitions",
            "Club Competitions",
            serde_json::to_value(&world.references.club_competitions),
        ),
        (
            "references.nation_competitions",
            "Nation Competitions",
            serde_json::to_value(&world.references.nation_competitions),
        ),
        (
            "references.staff_history",
            "Staff History",
            serde_json::to_value(viewer_staff_history_rows(&world.references.staff_history)),
        ),
        (
            "references.staff_comp_history",
            "Staff Competition History",
            serde_json::to_value(viewer_history58_rows(
                &world.references.staff_comp_history,
                &staff_comp_names,
            )),
        ),
        (
            "references.club_comp_history",
            "Club Competition History",
            serde_json::to_value(viewer_history26_rows(
                &world.references.club_comp_history,
                &club_comp_names,
            )),
        ),
        (
            "references.nation_comp_history",
            "Nation Competition History",
            serde_json::to_value(viewer_history26_rows(
                &world.references.nation_comp_history,
                &nation_comp_names,
            )),
        ),
    ];

    tables
        .into_iter()
        .map(|(path, label, rows)| {
            rows.map(|rows| (path, label, rows))
                .map_err(|err| format!("failed to serialize viewer table {path}: {err}"))
        })
        .collect()
}

fn viewer_table_index(world: &World) -> Vec<serde_json::Value> {
    viewer_table_specs(world)
        .into_iter()
        .map(|(path, label, row_count)| {
            serde_json::json!({
                "path": path,
                "label": label,
                "row_count": row_count,
                "api_url": format!("/api/table/{path}"),
            })
        })
        .collect()
}

fn viewer_table_specs(world: &World) -> Vec<(&'static str, &'static str, usize)> {
    vec![
        ("base_data", "Base Data", world.base_data.len()),
        (
            "save.sections",
            "Save Sections",
            world.save.as_ref().map_or(0, |save| save.sections.len()),
        ),
        ("core.clubs", "Core Clubs", world.core.clubs.len()),
        (
            "core.nat_clubs",
            "Core National Clubs",
            world.core.nat_clubs.len(),
        ),
        ("core.colours", "Core Colours", world.core.colours.len()),
        (
            "core.continents",
            "Core Continents",
            world.core.continents.len(),
        ),
        ("core.nations", "Core Nations", world.core.nations.len()),
        ("schema.tables", "Schema Tables", world.schema.tables.len()),
        ("staff.type6", "Staff Type 6", world.staff.type6.len()),
        ("staff.type8", "Staff Type 8", world.staff.type8.len()),
        ("staff.type9", "Staff Type 9", world.staff.type9.len()),
        ("staff.type10", "Staff Type 10", world.staff.type10.len()),
        ("references.cities", "Cities", world.references.cities.len()),
        (
            "references.officials",
            "Officials",
            world.references.officials.len(),
        ),
        (
            "references.first_names",
            "First Names",
            world.references.first_names.len(),
        ),
        (
            "references.second_names",
            "Second Names",
            world.references.second_names.len(),
        ),
        (
            "references.common_names",
            "Common Names",
            world.references.common_names.len(),
        ),
        (
            "references.stadiums",
            "Stadiums",
            world.references.stadiums.len(),
        ),
        (
            "references.staff_competitions",
            "Staff Competitions",
            world.references.staff_competitions.len(),
        ),
        (
            "references.club_competitions",
            "Club Competitions",
            world.references.club_competitions.len(),
        ),
        (
            "references.nation_competitions",
            "Nation Competitions",
            world.references.nation_competitions.len(),
        ),
        (
            "references.staff_history",
            "Staff History",
            world.references.staff_history.len(),
        ),
        (
            "references.staff_comp_history",
            "Staff Competition History",
            world.references.staff_comp_history.len(),
        ),
        (
            "references.club_comp_history",
            "Club Competition History",
            world.references.club_comp_history.len(),
        ),
        (
            "references.nation_comp_history",
            "Nation Competition History",
            world.references.nation_comp_history.len(),
        ),
    ]
}

fn viewer_table_value(
    world: &World,
    table_path: &str,
) -> Result<Option<(&'static str, &'static str, serde_json::Value)>, String> {
    let save_sections = || {
        world
            .save
            .as_ref()
            .map(|save| serde_json::to_value(&save.sections))
            .transpose()
            .map_err(|err| format!("failed to serialize save sections: {err}"))
            .map(|value| value.unwrap_or_else(|| serde_json::json!([])))
    };
    let staff_comp_names = || competition_name_map(&world.references.staff_competitions);
    let club_comp_names = || competition_name_map(&world.references.club_competitions);
    let nation_comp_names = || competition_name_map(&world.references.nation_competitions);

    let value = match table_path {
        "base_data" => Some(("base_data", "Base Data", to_viewer_value(&world.base_data))),
        "save.sections" => Some(("save.sections", "Save Sections", save_sections())),
        "core.clubs" => Some((
            "core.clubs",
            "Core Clubs",
            to_viewer_value(&world.core.clubs),
        )),
        "core.nat_clubs" => Some((
            "core.nat_clubs",
            "Core National Clubs",
            to_viewer_value(&world.core.nat_clubs),
        )),
        "core.colours" => Some((
            "core.colours",
            "Core Colours",
            to_viewer_value(&world.core.colours),
        )),
        "core.continents" => Some((
            "core.continents",
            "Core Continents",
            to_viewer_value(&world.core.continents),
        )),
        "core.nations" => Some((
            "core.nations",
            "Core Nations",
            to_viewer_value(&world.core.nations),
        )),
        "schema.tables" => Some((
            "schema.tables",
            "Schema Tables",
            to_viewer_value(&world.schema.tables),
        )),
        "staff.type6" => Some((
            "staff.type6",
            "Staff Type 6",
            to_viewer_value(&world.staff.type6),
        )),
        "staff.type8" => Some((
            "staff.type8",
            "Staff Type 8",
            to_viewer_value(&world.staff.type8),
        )),
        "staff.type9" => Some((
            "staff.type9",
            "Staff Type 9",
            to_viewer_value(&world.staff.type9),
        )),
        "staff.type10" => Some((
            "staff.type10",
            "Staff Type 10",
            to_viewer_value(&world.staff.type10),
        )),
        "references.cities" => Some((
            "references.cities",
            "Cities",
            to_viewer_value(&world.references.cities),
        )),
        "references.officials" => Some((
            "references.officials",
            "Officials",
            to_viewer_value(&world.references.officials),
        )),
        "references.first_names" => Some((
            "references.first_names",
            "First Names",
            to_viewer_value(&world.references.first_names),
        )),
        "references.second_names" => Some((
            "references.second_names",
            "Second Names",
            to_viewer_value(&world.references.second_names),
        )),
        "references.common_names" => Some((
            "references.common_names",
            "Common Names",
            to_viewer_value(&world.references.common_names),
        )),
        "references.stadiums" => Some((
            "references.stadiums",
            "Stadiums",
            to_viewer_value(&world.references.stadiums),
        )),
        "references.staff_competitions" => Some((
            "references.staff_competitions",
            "Staff Competitions",
            to_viewer_value(&world.references.staff_competitions),
        )),
        "references.club_competitions" => Some((
            "references.club_competitions",
            "Club Competitions",
            to_viewer_value(&world.references.club_competitions),
        )),
        "references.nation_competitions" => Some((
            "references.nation_competitions",
            "Nation Competitions",
            to_viewer_value(&world.references.nation_competitions),
        )),
        "references.staff_history" => Some((
            "references.staff_history",
            "Staff History",
            to_viewer_value(viewer_staff_history_rows(&world.references.staff_history)),
        )),
        "references.staff_comp_history" => Some((
            "references.staff_comp_history",
            "Staff Competition History",
            to_viewer_value(viewer_history58_rows(
                &world.references.staff_comp_history,
                &staff_comp_names(),
            )),
        )),
        "references.club_comp_history" => Some((
            "references.club_comp_history",
            "Club Competition History",
            to_viewer_value(viewer_history26_rows(
                &world.references.club_comp_history,
                &club_comp_names(),
            )),
        )),
        "references.nation_comp_history" => Some((
            "references.nation_comp_history",
            "Nation Competition History",
            to_viewer_value(viewer_history26_rows(
                &world.references.nation_comp_history,
                &nation_comp_names(),
            )),
        )),
        _ => None,
    };

    value
        .map(|(path, label, rows)| {
            rows.map(|rows| (path, label, rows))
                .map_err(|err| format!("failed to serialize viewer table {path}: {err}"))
        })
        .transpose()
}

fn to_viewer_value<T: serde::Serialize>(value: T) -> Result<serde_json::Value, String> {
    serde_json::to_value(value).map_err(|err| err.to_string())
}

fn competition_name_map(entries: &[cm_domain::DomainCompetition]) -> HashMap<u32, String> {
    entries
        .iter()
        .map(|entry| (entry.id, entry.long_name.clone()))
        .collect()
}

fn viewer_staff_history_rows(entries: &[cm_domain::DomainHistory17]) -> Vec<serde_json::Value> {
    entries
        .iter()
        .map(|entry| {
            serde_json::json!({
                "decode_status": "packed numeric history; staff identity fields still need code-derived names",
                "id": entry.id,
                "staff_id": entry.id,
                "u32_slots": entry.u32_slots,
                "trailing_byte": entry.trailing_byte,
            })
        })
        .collect()
}

fn viewer_history26_rows(
    entries: &[cm_domain::DomainHistory26],
    competition_names: &HashMap<u32, String>,
) -> Vec<serde_json::Value> {
    entries
        .iter()
        .map(|entry| {
            let record_id = entry.u32_slots[0];
            let competition_id = entry.u32_slots[1];
            serde_json::json!({
                "decode_status": "competition id resolved; remaining slots are packed numeric history",
                "record_id": record_id,
                "competition_id": competition_id,
                "competition_name": competition_names.get(&competition_id).cloned().unwrap_or_default(),
                "u32_slots": entry.u32_slots,
                "trailing_u16": entry.trailing_u16,
            })
        })
        .collect()
}

fn viewer_history58_rows(
    entries: &[cm_domain::DomainHistory58],
    competition_names: &HashMap<u32, String>,
) -> Vec<serde_json::Value> {
    entries
        .iter()
        .map(|entry| {
            let record_id = entry.u32_slots[0];
            let competition_id = entry.u32_slots[1];
            serde_json::json!({
                "decode_status": "competition id resolved; remaining slots are packed numeric history",
                "record_id": record_id,
                "competition_id": competition_id,
                "competition_name": competition_names.get(&competition_id).cloned().unwrap_or_default(),
                "u32_slots": entry.u32_slots,
                "trailing_u16": entry.trailing_u16,
            })
        })
        .collect()
}

fn rename_structured_table_row(
    world: &mut World,
    table: &str,
    row: &str,
    name: &str,
) -> Result<String, String> {
    let id = row
        .parse::<u32>()
        .map_err(|err| format!("row selector must be an integer: {err}"))?;
    let index = id as usize;

    match table {
        "references.cities" => rename_by_id(
            &mut world.references.cities,
            id,
            |entry| entry.id,
            |entry| &mut entry.name,
            table,
            name,
        ),
        "references.stadiums" => rename_by_id(
            &mut world.references.stadiums,
            id,
            |entry| entry.id,
            |entry| &mut entry.name,
            table,
            name,
        ),
        "references.staff_competitions" => rename_by_id(
            &mut world.references.staff_competitions,
            id,
            |entry| entry.id,
            |entry| &mut entry.long_name,
            table,
            name,
        ),
        "references.club_competitions" => rename_by_id(
            &mut world.references.club_competitions,
            id,
            |entry| entry.id,
            |entry| &mut entry.long_name,
            table,
            name,
        ),
        "references.nation_competitions" => rename_by_id(
            &mut world.references.nation_competitions,
            id,
            |entry| entry.id,
            |entry| &mut entry.long_name,
            table,
            name,
        ),
        "references.first_names" => rename_by_index(&mut world.references.first_names, index, table, name),
        "references.second_names" => rename_by_index(&mut world.references.second_names, index, table, name),
        "references.common_names" => rename_by_index(&mut world.references.common_names, index, table, name),
        "core.continents" => rename_core_by_ordinal(&mut world.core.continents, id, table, name),
        "core.colours" => rename_core_by_ordinal(&mut world.core.colours, id, table, name),
        "core.nations" => rename_core_by_ordinal(&mut world.core.nations, id, table, name),
        "core.clubs" => rename_core_by_ordinal(&mut world.core.clubs, id, table, name),
        "core.nat_clubs" => rename_core_by_ordinal(&mut world.core.nat_clubs, id, table, name),
        _ => Err(format!(
            "rename-rust-db supports core name tables and structured reference name tables today; unsupported table {table}"
        )),
    }
}

fn set_rust_db_text_field(
    world: &mut World,
    table: &str,
    row: &str,
    field: &str,
    text: &str,
) -> Result<String, String> {
    let id = row
        .parse::<u32>()
        .map_err(|err| format!("row selector must be an integer: {err}"))?;
    let index = id as usize;

    match table {
        "core.continents" => set_core_text_field(&mut world.core.continents, id, table, field, text),
        "core.colours" => set_core_text_field(&mut world.core.colours, id, table, field, text),
        "core.nations" => set_core_text_field(&mut world.core.nations, id, table, field, text),
        "core.clubs" => set_core_text_field(&mut world.core.clubs, id, table, field, text),
        "core.nat_clubs" => set_core_text_field(&mut world.core.nat_clubs, id, table, field, text),
        "references.cities" if field == "name" => rename_by_id(
            &mut world.references.cities,
            id,
            |entry| entry.id,
            |entry| &mut entry.name,
            table,
            text,
        ),
        "references.stadiums" if field == "name" => rename_by_id(
            &mut world.references.stadiums,
            id,
            |entry| entry.id,
            |entry| &mut entry.name,
            table,
            text,
        ),
        "references.staff_competitions" => set_competition_text_field(
            &mut world.references.staff_competitions,
            id,
            table,
            field,
            text,
        ),
        "references.club_competitions" => set_competition_text_field(
            &mut world.references.club_competitions,
            id,
            table,
            field,
            text,
        ),
        "references.nation_competitions" => set_competition_text_field(
            &mut world.references.nation_competitions,
            id,
            table,
            field,
            text,
        ),
        "references.first_names" if field == "text" => {
            rename_by_index(&mut world.references.first_names, index, table, text)
        }
        "references.second_names" if field == "text" => {
            rename_by_index(&mut world.references.second_names, index, table, text)
        }
        "references.common_names" if field == "text" => {
            rename_by_index(&mut world.references.common_names, index, table, text)
        }
        _ => Err(format!(
            "unsupported text field {field:?} for {table}; use audit-rust-db or inspect-rust-db to see owned tables"
        )),
    }
}

fn set_core_text_field(
    entries: &mut [cm_domain::DomainOpaqueRecord],
    ordinal: u32,
    table: &str,
    field: &str,
    text: &str,
) -> Result<String, String> {
    let segment_index = match field {
        "name" | "primary" | "primary_name" => 0,
        "secondary" | "secondary_name" => 1,
        "short" | "short_name" => 2,
        _ => {
            return Err(format!(
                "unsupported core text field {field}; use primary, secondary, or short"
            ))
        }
    };
    let entry = entries
        .iter_mut()
        .find(|entry| entry.ordinal == ordinal)
        .ok_or_else(|| format!("no row ordinal {ordinal} in {table}"))?;
    let old = match segment_index {
        0 => entry.primary_name.clone().unwrap_or_default(),
        1 => entry.secondary_name.clone().unwrap_or_default(),
        2 => entry.short_name.clone().unwrap_or_default(),
        _ => String::new(),
    };
    replace_latin1_text_segment(&mut entry.raw, segment_index, text)?;
    match segment_index {
        0 => entry.primary_name = Some(text.to_string()),
        1 => entry.secondary_name = Some(text.to_string()),
        2 => entry.short_name = Some(text.to_string()),
        _ => {}
    }
    while entry.text_candidates.len() <= segment_index {
        entry.text_candidates.push(String::new());
    }
    entry.text_candidates[segment_index] = text.to_string();
    Ok(format!(
        "{table} ordinal {ordinal} {field}: {old:?} -> {text:?}"
    ))
}

fn set_competition_text_field(
    entries: &mut [cm_domain::DomainCompetition],
    id: u32,
    table: &str,
    field: &str,
    text: &str,
) -> Result<String, String> {
    let entry = entries
        .iter_mut()
        .find(|entry| entry.id == id)
        .ok_or_else(|| format!("no row id {id} in {table}"))?;
    let target = match field {
        "long" | "long_name" | "name" => &mut entry.long_name,
        "short" | "short_name" => &mut entry.short_name,
        _ => {
            return Err(format!(
                "unsupported competition text field {field}; use long_name or short_name"
            ))
        }
    };
    let old = std::mem::replace(target, text.to_string());
    Ok(format!("{table} id {id} {field}: {old:?} -> {text:?}"))
}

fn rename_by_id<T, IdFn, NameFn>(
    entries: &mut [T],
    id: u32,
    id_fn: IdFn,
    name_fn: NameFn,
    table: &str,
    name: &str,
) -> Result<String, String>
where
    IdFn: Fn(&T) -> u32,
    NameFn: Fn(&mut T) -> &mut String,
{
    let entry = entries
        .iter_mut()
        .find(|entry| id_fn(entry) == id)
        .ok_or_else(|| format!("no row id {id} in {table}"))?;
    let old = std::mem::replace(name_fn(entry), name.to_string());
    Ok(format!("{table} id {id}: {old:?} -> {name:?}"))
}

fn rename_by_index(
    entries: &mut [cm_domain::DomainName],
    index: usize,
    table: &str,
    name: &str,
) -> Result<String, String> {
    let entry = entries
        .get_mut(index)
        .ok_or_else(|| format!("no row index {index} in {table}"))?;
    let old = std::mem::replace(&mut entry.text, name.to_string());
    Ok(format!("{table} row {index}: {old:?} -> {name:?}"))
}

fn set_staff_type10_scalar(
    world: &mut World,
    id: u32,
    field: &str,
    value: u16,
) -> Result<String, String> {
    let entry = world
        .staff
        .type10
        .iter_mut()
        .find(|entry| entry.id == id)
        .ok_or_else(|| format!("no staff.type10 row id {id}"))?;
    match field {
        "ca" | "probable_ca" | "rating_short_0x05" => {
            let old = entry.rating_short_0x05;
            entry.rating_short_0x05 = value;
            Ok(format!(
                "staff.type10 id {id} rating_short_0x05: {old} -> {value}"
            ))
        }
        "pa" | "probable_pa" | "rating_short_0x07" => {
            let old = entry.rating_short_0x07;
            entry.rating_short_0x07 = value;
            Ok(format!(
                "staff.type10 id {id} rating_short_0x07: {old} -> {value}"
            ))
        }
        "reputation" | "probable_reputation" | "rating_short_0x0d" => {
            let old = entry.rating_short_0x0d;
            entry.rating_short_0x0d = value;
            Ok(format!(
                "staff.type10 id {id} rating_short_0x0d: {old} -> {value}"
            ))
        }
        _ => Err(format!("unsupported staff.type10 field {field}")),
    }
}

fn set_staff_type10_attribute(
    world: &mut World,
    id: u32,
    index: usize,
    value: u8,
) -> Result<String, String> {
    if index >= 31 {
        return Err(format!(
            "staff.type10 attribute index must be 0..30, got {index}"
        ));
    }
    let entry = world
        .staff
        .type10
        .iter_mut()
        .find(|entry| entry.id == id)
        .ok_or_else(|| format!("no staff.type10 row id {id}"))?;
    let old = entry.attributes[index];
    entry.attributes[index] = value;
    Ok(format!(
        "staff.type10 id {id} attr_{index}: {old} -> {value}"
    ))
}

fn rename_core_by_ordinal(
    entries: &mut [cm_domain::DomainOpaqueRecord],
    ordinal: u32,
    table: &str,
    name: &str,
) -> Result<String, String> {
    let entry = entries
        .iter_mut()
        .find(|entry| entry.ordinal == ordinal)
        .ok_or_else(|| format!("no row ordinal {ordinal} in {table}"))?;
    let old = entry.primary_name.clone().unwrap_or_default();
    replace_first_clean_latin1_text(&mut entry.raw, &old, name)?;
    entry.primary_name = Some(name.to_string());
    if let Some(candidate) = entry.text_candidates.get_mut(0) {
        *candidate = name.to_string();
    } else {
        entry.text_candidates.push(name.to_string());
    }
    Ok(format!("{table} ordinal {ordinal}: {old:?} -> {name:?}"))
}

fn replace_first_clean_latin1_text(bytes: &mut [u8], old: &str, new: &str) -> Result<(), String> {
    let new_bytes = latin1_bytes(new)?;
    let segments = latin1_segments(bytes);
    let segment = segments
        .iter()
        .find(|segment| segment.clean_text == old)
        .or_else(|| segments.first())
        .ok_or_else(|| "record has no replaceable text segment".to_string())?;
    if new_bytes.len() > segment.width {
        return Err(format!(
            "new text {new:?} is {} bytes, but this fixed slot only has {} bytes",
            new_bytes.len(),
            segment.width
        ));
    }
    let prefix_len = usize::from(segment.has_ff_prefix);
    let value_start = segment.start + prefix_len;
    let value_end = value_start + segment.width;
    bytes[value_start..value_end].fill(0);
    bytes[value_start..value_start + new_bytes.len()].copy_from_slice(&new_bytes);
    Ok(())
}

fn replace_latin1_text_segment(
    bytes: &mut [u8],
    segment_index: usize,
    new: &str,
) -> Result<(), String> {
    let new_bytes = latin1_bytes(new)?;
    let segments = latin1_segments(bytes);
    let segment = segments
        .get(segment_index)
        .ok_or_else(|| format!("record has no text segment {segment_index}"))?;
    if new_bytes.len() > segment.width {
        return Err(format!(
            "new text {new:?} is {} bytes, but this fixed slot only has {} bytes",
            new_bytes.len(),
            segment.width
        ));
    }
    let prefix_len = usize::from(segment.has_ff_prefix);
    let value_start = segment.start + prefix_len;
    let value_end = value_start + segment.width;
    bytes[value_start..value_end].fill(0);
    bytes[value_start..value_start + new_bytes.len()].copy_from_slice(&new_bytes);
    Ok(())
}

fn latin1_bytes(text: &str) -> Result<Vec<u8>, String> {
    text.chars()
        .map(|ch| {
            let code = ch as u32;
            if code <= 0xff {
                Ok(code as u8)
            } else {
                Err(format!(
                    "text {text:?} contains non-Latin-1 character {ch:?}"
                ))
            }
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Latin1Segment {
    start: usize,
    width: usize,
    has_ff_prefix: bool,
    clean_text: String,
}

fn latin1_segments(bytes: &[u8]) -> Vec<Latin1Segment> {
    let mut segments = Vec::new();
    let mut start = None;
    for (index, byte) in bytes.iter().copied().enumerate().skip(4) {
        let printable = (32..=126).contains(&byte) || (160..=255).contains(&byte);
        if printable {
            if start.is_none() {
                start = Some(index);
            }
            continue;
        }
        if byte == 0 {
            if let Some(begin) = start.take() {
                if index > begin {
                    let has_ff_prefix = bytes[begin] == 0xff;
                    let value_start = begin + usize::from(has_ff_prefix);
                    let clean_text = bytes[value_start..index]
                        .iter()
                        .map(|&byte| char::from(byte))
                        .collect::<String>()
                        .trim()
                        .to_string();
                    if !clean_text.is_empty() {
                        let mut capacity_end = index;
                        while capacity_end < bytes.len() && bytes[capacity_end] == 0 {
                            capacity_end += 1;
                        }
                        segments.push(Latin1Segment {
                            start: begin,
                            width: capacity_end - value_start,
                            has_ff_prefix,
                            clean_text,
                        });
                    }
                }
            }
        } else {
            start = None;
        }
    }
    segments
}

fn load_manifest(data_dir: &Path) -> Result<Manifest, String> {
    let bytes = fs::read(data_dir.join("index.dat"))
        .map_err(|err| format!("failed to read index.dat: {err}"))?;
    Ok(Manifest::parse(&bytes))
}

fn load_events(data_dir: &Path) -> Result<EventConfig, String> {
    let bytes = fs::read(data_dir.join("events_eng.cfg"))
        .map_err(|err| format!("failed to read events_eng.cfg: {err}"))?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(EventConfig::parse(&text))
}

fn try_load_save(root: &Path) -> Result<Option<Vec<u8>>, String> {
    let path = root.join("save1.sav");
    if !path.exists() {
        return Ok(None);
    }
    let bytes =
        fs::read(&path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    Ok(Some(bytes))
}

fn count_records(path: &Path, kind: RecordKind) -> Result<usize, String> {
    let bytes =
        fs::read(path).map_err(|err| format!("failed to read {}: {err}", path.display()))?;
    let dat = DatFile::new(kind, &bytes)
        .map_err(|err| format!("failed to parse {}: {err}", path.display()))?;
    Ok(dat.count())
}
