use std::path::Path;

use cm_domain::{NewGameOptions, World};

fn main() {
    let rust_db = Path::new("D:/cm0102-rs/rust-db");
    let world = World::read_rust_db_dir(rust_db).expect("read rust-db");
    let options = NewGameOptions {
        selected_nations: vec!["England".to_string()],
        background_nations: vec![],
        use_real_players: true,
        attribute_masking: true,
        start_year: 2001,
    };
    let save = world.new_game_from_rust_db(rust_db, &options);

    let prem = save
        .simple_leagues
        .iter()
        .find(|l| l.real_comp_id == 7)
        .expect("Premier Division not found");

    let club_ids: Vec<u32> = prem.teams.iter().map(|t| t.club_id).collect();
    println!("{} clubs in the Premier Division", club_ids.len());

    let mut total_shots_h = 0u32;
    let mut total_shots_a = 0u32;
    let mut total_goals_h = 0u32;
    let mut total_goals_a = 0u32;
    let mut n = 0u32;

    for pair in club_ids.chunks(2).take(10) {
        if pair.len() < 2 {
            continue;
        }
        let home = save.snapshot_team_for_engine(pair[0]);
        let away = save.snapshot_team_for_engine(pair[1]);
        let (Some(home), Some(away)) = (home, away) else {
            println!("skip {} v {} — snapshot failed", pair[0], pair[1]);
            continue;
        };
        for seed in 0..3u64 {
            let r = cm_domain::match_engine_exe::simulate_one_fixture_token_model(
                &home, &away, seed * 1000 + n as u64,
            );
            println!(
                "{} v {}: score {}-{} shots {}-{} on {}-{}",
                pair[0], pair[1], r.home_score, r.away_score,
                r.home_shots, r.away_shots, r.home_shots_on, r.away_shots_on
            );
            total_shots_h += r.home_shots as u32;
            total_shots_a += r.away_shots as u32;
            total_goals_h += r.home_score as u32;
            total_goals_a += r.away_score as u32;
            n += 1;
        }
    }

    println!(
        "\n{} matches: avg shots {:.1}-{:.1}  avg goals {:.2}-{:.2}",
        n,
        total_shots_h as f32 / n as f32, total_shots_a as f32 / n as f32,
        total_goals_h as f32 / n as f32, total_goals_a as f32 / n as f32,
    );
}
