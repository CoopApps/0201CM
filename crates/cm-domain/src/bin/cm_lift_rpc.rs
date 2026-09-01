//! `cm-lift-rpc` — thin JSON-over-stdout wrapper around ported cm-domain
//! fns. Called by `cm_lift.diff_harness` to compare each Rust port
//! against the exe emulated under Unicorn.
//!
//! Usage: `cargo run --release --bin cm_lift_rpc -- <fn_name> <args...>`
//!
//! Prints JSON like `{"ret": 1, "exc": null}` on stdout.
//!
//! Add new fns to `dispatch()` as their ports gain probes. Every entry
//! becomes a paired diff test in `cm-lift diff`.

use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!(r#"{{"ret": null, "exc": "usage: cm_lift_rpc <fn_name> <args...>"}}"#);
        return;
    }
    let fn_name = args[1].as_str();
    let call_args: Vec<i64> = args[2..].iter()
        .filter_map(|s| s.parse::<i64>().ok())
        .collect();
    let out = match dispatch(fn_name, &call_args) {
        Ok(v)  => format!(r#"{{"ret": {}, "exc": null}}"#, v),
        Err(e) => format!(r#"{{"ret": null, "exc": {:?}}}"#, e),
    };
    println!("{}", out);
}

/// Dispatch table — one arm per ported fn we can diff against the exe.
fn dispatch(name: &str, args: &[i64]) -> Result<i64, String> {
    use cm_domain::match_engine_exe;
    use cm_domain::finance;
    use cm_domain::transfer;
    use cm_domain::tactic_file;
    use cm_domain::formation;

    match name {
        "finalize_rating" => {
            if args.len() < 1 { return Err("need 1 arg".into()); }
            Ok(match_engine_exe::finalize_rating(args[0] as i16) as i64)
        }
        "gk_save_rating_delta_milli" => {
            if args.len() < 2 { return Err("need 2 args".into()); }
            Ok(match_engine_exe::gk_save_rating_delta_milli(
                args[0] != 0, args[1] as u8) as i64)
        }
        "shot_tier_bucket" => {
            // rng_delta, flag — RNG-dependent so this diff is smoke only
            if args.len() < 2 { return Err("need 2 args".into()); }
            let mut rng = match_engine_exe::MatchRng::new(0);
            Ok(match_engine_exe::shot_tier_bucket(
                args[0] as i16, args[1] as i8, &mut rng) as i64)
        }
        "role_mask_to_position" => {
            if args.len() < 1 { return Err("need 1 arg".into()); }
            let pos = formation::FormationCode::role_mask_to_position(args[0] as u16);
            Ok(pos.map(|p| p as i64).unwrap_or(-1))
        }
        "chairman_will_sack" => {
            if args.len() < 2 { return Err("need 2 args".into()); }
            let cs = finance::ChairmanState::default();
            Ok(finance::chairman_will_sack(&cs, args[0] as i32, args[1] as i32) as i64)
        }
        "chairman_takeover_fires" => {
            if args.len() < 1 { return Err("need 1 arg".into()); }
            let cs = finance::ChairmanState::default();
            Ok(finance::chairman_takeover_fires(&cs, args[0] as i32) as i64)
        }
        "team_tempo_mask" => {
            if args.len() < 1 { return Err("need 1 arg".into()); }
            Ok(tactic_file::team_tempo_mask(args[0] as i8)
                .map(|m| m as i64).unwrap_or(-1))
        }
        "team_mentality_mask" => {
            if args.len() < 1 { return Err("need 1 arg".into()); }
            Ok(tactic_file::team_mentality_mask(args[0] as i8) as i64)
        }
        "age_wage_cap" => {
            if args.len() < 2 { return Err("need 2 args".into()); }
            Ok(finance::age_wage_cap(args[0] as i32, args[1] as i32) as i64)
        }
        "star_floor" => {
            if args.len() < 1 { return Err("need 1 arg".into()); }
            Ok(finance::star_floor(args[0] as usize) as i64)
        }
        "away_goals_verdict" => {
            if args.len() < 4 { return Err("need 4 args".into()); }
            let v = transfer::away_goals_verdict(
                args[0] as u8, args[1] as u8, args[2] as u32, args[3] as u32);
            Ok(v.map(|id| id as i64).unwrap_or(-1))
        }
        "scout_bucket_range" => {
            if args.len() < 1 { return Err("need 1 arg".into()); }
            let (lo, hi) = transfer::scout_bucket_range(args[0] as u8);
            Ok(((lo as i64) << 8) | (hi as i64))
        }
        "fifa_score" => {
            if args.len() < 6 { return Err("need 6 args (all as milli-i64)".into()); }
            let f = |i: usize| args[i] as f64 / 1000.0;
            Ok((cm_domain::fifa_rankings::fifa_score(f(0), f(1), f(2), f(3), f(4), f(5)) * 1000.0) as i64)
        }
        "season_avg_rating" => {
            if args.len() < 2 { return Err("need 2 args (appearances, sum)".into()); }
            let v = cm_domain::player_rating::season_avg_rating(args[0] as u8, args[1] as i16);
            Ok(v.map(|f| (f * 1000.0) as i64).unwrap_or(-1))
        }
        "predict_wage" => {
            if args.len() < 8 {
                return Err("need 8 args (contract_field, agent_q, rep_bucket, age, is_gk, player_id, game_day, mode)".into());
            }
            Ok(cm_domain::transfer::predict_wage(
                args[0] as i32, args[1] as i32, args[2] as i32,
                args[3] as u8, args[4] != 0,
                args[5] as u32, args[6] as u32, args[7] as u8,
            ) as i64)
        }
        "assist_bonus_milli" => {
            if args.len() < 1 { return Err("need 1 arg".into()); }
            Ok(match_engine_exe::assist_bonus_milli(args[0] as i16) as i64)
        }
        "scout_throttle" => {
            // (club_rep, [7 x (attr5, attr6, or -1 if empty)], academy, top5)
            if args.len() < 16 { return Err("need 16 args".into()); }
            let coach_attrs: [Option<(i8, i8)>; 7] = [
                if args[1] < 0 { None } else { Some((args[1] as i8, args[2] as i8)) },
                if args[3] < 0 { None } else { Some((args[3] as i8, args[4] as i8)) },
                if args[5] < 0 { None } else { Some((args[5] as i8, args[6] as i8)) },
                if args[7] < 0 { None } else { Some((args[7] as i8, args[8] as i8)) },
                if args[9] < 0 { None } else { Some((args[9] as i8, args[10] as i8)) },
                if args[11] < 0 { None } else { Some((args[11] as i8, args[12] as i8)) },
                if args[13] < 0 { None } else { Some((args[13] as i8, args[14] as i8)) },
            ];
            Ok(cm_domain::transfer::scout_throttle(
                args[0] as i16, &coach_attrs,
                args[15] != 0, false, // academy, top5 — simplified
            ) as i64)
        }
        "foreign_player_permit" => {
            if args.len() < 5 { return Err("need 5 args".into()); }
            Ok(cm_domain::transfer::foreign_player_permit(
                args[0] as i16, args[1] as i32, args[2] as i32,
                args[3] as i32, args[4] != 0,
            ) as i64)
        }
        "mentor_loyalty_bypass" => {
            // args are unused — just a smoke test of the const list
            let count = cm_domain::transfer::MENTOR_LOYALTY_OVERRIDE_MANAGERS.len();
            Ok(count as i64)
        }
        "chairman_approves_overrun" => {
            if args.len() < 2 { return Err("need 2 args (amount_gbp, would_go_negative)".into()); }
            let mut cs = finance::ChairmanState::default();
            Ok(finance::chairman_approves_overrun(&mut cs, args[0], args[1] != 0) as i64)
        }
        "club_status_byte" => {
            // (id, num_clubs, table_len) — table is synthesized deterministically
            if args.len() < 3 { return Err("need 3 args".into()); }
            let id = args[0] as u32;
            let num_clubs = args[1] as u32;
            let table_len = args[2] as usize;
            let mut table = vec![0u8; table_len];
            for (i, b) in table.iter_mut().enumerate() { *b = (i & 0xff) as u8; }
            Ok(finance::club_status_byte(&table, Some(id), num_clubs) as i64)
        }
        "shot_in_box" => {
            if args.len() < 3 { return Err("need 3 args (x, y, target_side)".into()); }
            Ok(match_engine_exe::shot_in_box(
                args[0] as i8, args[1] as i8, args[2] as u8) as i64)
        }
        "morale_label_id" => {
            if args.len() < 1 { return Err("need 1 arg (morale)".into()); }
            // Returns the ordinal of the label bucket
            let label = cm_domain::transfer::morale_label(args[0] as u8);
            let ord = ["Very Low", "Low", "Ok", "Good", "Very Good", "Superb"]
                .iter().position(|&s| s == label).unwrap_or(0);
            Ok(ord as i64)
        }
        _ => Err(format!("unknown fn: {}", name)),
    }
}
