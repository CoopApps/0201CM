use cm_domain::match_engine_exe::{simulate_one_fixture_token_model, EngineTeamSnapshot, EngineTeamPlayer};

fn mk(id: u32, ca: u16) -> EngineTeamSnapshot {
    EngineTeamSnapshot {
        club_id: id, reputation: 1200, grudge_score: 0,
        sum_position_ratings: 0, out_of_position_ids: Vec::new(),
        team_settings: cm_domain::tactic_file::TeamSettings {
            passing:        cm_domain::tactic_file::Passing::Unset,
            mentality:      cm_domain::tactic_file::Mentality::Unset,
            counter_attack: false, offside_trap: false,
            pressing:       cm_domain::tactic_file::Pressing::Unset,
            marking:        cm_domain::tactic_file::Marking::Unset,
            tackling:       cm_domain::tactic_file::Tackling::Unset,
        },
        players: (0..16).map(|i| EngineTeamPlayer {
            player_id: id*100+i, is_not_injured: true, position: (2+(i%8)) as u8,
            jumping_heading: 10, aggression: 8, bravery: 10, dirtiness: 5,
            current_ability: ca, age: 25, injury_proneness: 8, form: 12,
            is_first_choice_gk: i==0, speciality_a: 0, speciality_b: 0,
            position_natural: (2+(i%8)) as u8, position_learn: 0,
        }).collect(),
    }
}

fn main() {
    let h = mk(1, 12000);
    let a = mk(2, 12000);
    let r = simulate_one_fixture_token_model(&h, &a, 42);
    println!("score {}-{}, shots {}-{}, on {}-{}, goal_mins h={} a={}",
        r.home_score, r.away_score, r.home_shots, r.away_shots,
        r.home_shots_on, r.away_shots_on,
        r.home_goal_minutes.len(), r.away_goal_minutes.len());
}
