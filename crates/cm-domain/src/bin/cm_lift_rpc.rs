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
        "shot_in_box" => {
            if args.len() < 3 { return Err("need 3 args".into()); }
            Ok(match_engine_exe::shot_in_box(
                args[0] as i8, args[1] as i8, args[2] as u8) as i64)
        }
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
        _ => Err(format!("unknown fn: {}", name)),
    }
}
