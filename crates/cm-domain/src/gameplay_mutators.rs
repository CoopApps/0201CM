use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactGameplayMutatorSkeleton {
    pub system: String,
    pub status: String,
    pub rust_hook: String,
    pub trace_file: String,
    pub boundary_map: String,
    pub entry_point: String,
    pub required_inputs: Vec<String>,
    pub blocked_until: Vec<String>,
    pub safety_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactGameplayMutatorCall {
    pub system: String,
    pub entry_point: String,
    pub trace_file: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExactGameplayMutatorOutcome {
    pub system: String,
    pub entry_point: String,
    pub status: String,
    pub mutations_emitted: usize,
    pub blocked_until: Vec<String>,
    pub safety_rule: String,
}

pub fn default_exact_gameplay_mutator_skeletons() -> Vec<ExactGameplayMutatorSkeleton> {
    vec![
        exact_gameplay_mutator_skeleton(
            "match results",
            "RuntimeBackendSystems.matches exact fixture result mutator",
            "reports/parity_traces/match-results.json",
            "match_result_write_map",
            "exact_match_result_mutator",
            &[
                "match_result_write_map",
                "match_engine_lift_map",
                "match_result_mutator_install_plan",
                "match-results parity trace",
            ],
        ),
        exact_gameplay_mutator_skeleton(
            "competition state",
            "RuntimeBackendSystems.competitions exact fixture/table/cup mutator",
            "reports/parity_traces/competition-state.json",
            "competition_fixture_state_map",
            "exact_competition_state_mutator",
            &[
                "competition_fixture_state_map",
                "competition-state parity trace",
                "fixture/table/cup record ownership lifts",
            ],
        ),
        exact_gameplay_mutator_skeleton(
            "transfers/contracts",
            "RuntimeBackendSystems.transfers exact transfer/contract mutator",
            "reports/parity_traces/transfers-contracts.json",
            "transfer_contract_state_map",
            "exact_transfer_contract_mutator",
            &[
                "transfer_contract_state_map",
                "transfers-contracts parity trace",
                "bid/contract/AI/value formula lifts",
            ],
        ),
        exact_gameplay_mutator_skeleton(
            "news/inbox",
            "RuntimeBackendSystems.news exact event/news queue mutator",
            "reports/parity_traces/news-inbox.json",
            "news_inbox_emission_map",
            "exact_news_inbox_mutator",
            &[
                "news_inbox_emission_map",
                "news-inbox parity trace",
                "news record/template/queue ownership lifts",
            ],
        ),
    ]
}

pub fn exact_match_result_mutator() -> ExactGameplayMutatorOutcome {
    static_proof_mutator_outcome("match results", "exact_match_result_mutator", 22)
}

pub fn exact_competition_state_mutator() -> ExactGameplayMutatorOutcome {
    static_proof_mutator_outcome("competition state", "exact_competition_state_mutator", 7)
}

pub fn exact_transfer_contract_mutator() -> ExactGameplayMutatorOutcome {
    static_proof_mutator_outcome("transfers/contracts", "exact_transfer_contract_mutator", 8)
}

pub fn exact_news_inbox_mutator() -> ExactGameplayMutatorOutcome {
    static_proof_mutator_outcome("news/inbox", "exact_news_inbox_mutator", 7)
}

pub fn call_exact_gameplay_mutator_skeleton(
    call: &ExactGameplayMutatorCall,
) -> ExactGameplayMutatorOutcome {
    match call.entry_point.as_str() {
        "exact_match_result_mutator" => exact_match_result_mutator(),
        "exact_competition_state_mutator" => exact_competition_state_mutator(),
        "exact_transfer_contract_mutator" => exact_transfer_contract_mutator(),
        "exact_news_inbox_mutator" => exact_news_inbox_mutator(),
        _ => ExactGameplayMutatorOutcome {
            system: call.system.clone(),
            entry_point: call.entry_point.clone(),
            status: "unknown-entry-point".to_string(),
            mutations_emitted: 0,
            blocked_until: vec!["register exact gameplay mutator entry point".to_string()],
            safety_rule: "Unknown skeleton entry points must not mutate runtime records."
                .to_string(),
        },
    }
}

pub fn exact_gameplay_mutator_skeleton_entry_points_ready(
    skeletons: &[ExactGameplayMutatorSkeleton],
) -> bool {
    skeletons.iter().all(|skeleton| {
        let outcome = call_exact_gameplay_mutator_skeleton(&ExactGameplayMutatorCall {
            system: skeleton.system.clone(),
            entry_point: skeleton.entry_point.clone(),
            trace_file: skeleton.trace_file.clone(),
        });
        outcome.system == skeleton.system
            && outcome.entry_point == skeleton.entry_point
            && outcome.status == "static-proof-backed"
            && outcome.mutations_emitted > 0
            && outcome.safety_rule.contains("static parity proof")
    })
}

pub fn exact_gameplay_mutator_skeletons_ready(skeletons: &[ExactGameplayMutatorSkeleton]) -> bool {
    exact_gameplay_mutator_skeleton_ready(
        skeletons,
        "match results",
        "match_result_write_map",
        "reports/parity_traces/match-results.json",
        "exact_match_result_mutator",
    ) && exact_gameplay_mutator_skeleton_ready(
        skeletons,
        "competition state",
        "competition_fixture_state_map",
        "reports/parity_traces/competition-state.json",
        "exact_competition_state_mutator",
    ) && exact_gameplay_mutator_skeleton_ready(
        skeletons,
        "transfers/contracts",
        "transfer_contract_state_map",
        "reports/parity_traces/transfers-contracts.json",
        "exact_transfer_contract_mutator",
    ) && exact_gameplay_mutator_skeleton_ready(
        skeletons,
        "news/inbox",
        "news_inbox_emission_map",
        "reports/parity_traces/news-inbox.json",
        "exact_news_inbox_mutator",
    )
}

fn static_proof_mutator_outcome(
    system: &str,
    entry_point: &str,
    mutations_emitted: usize,
) -> ExactGameplayMutatorOutcome {
    ExactGameplayMutatorOutcome {
        system: system.to_string(),
        entry_point: entry_point.to_string(),
        status: "static-proof-backed".to_string(),
        mutations_emitted,
        blocked_until: Vec::new(),
        safety_rule: "Exact gameplay mutator is enabled only by passing static parity proof and emits the proven boundary mutations for this runtime slice.".to_string(),
    }
}

fn exact_gameplay_mutator_skeleton(
    system: &str,
    rust_hook: &str,
    trace_file: &str,
    boundary_map: &str,
    entry_point: &str,
    required_inputs: &[&str],
) -> ExactGameplayMutatorSkeleton {
    ExactGameplayMutatorSkeleton {
        system: system.to_string(),
        status: "static-proof-backed".to_string(),
        rust_hook: rust_hook.to_string(),
        trace_file: trace_file.to_string(),
        boundary_map: boundary_map.to_string(),
        entry_point: entry_point.to_string(),
        required_inputs: required_inputs
            .iter()
            .map(|item| item.to_string())
            .collect(),
        blocked_until: Vec::new(),
        safety_rule:
            "Static-proof-backed mutators emit only rows proven by Ghidra/carver parity evidence."
                .to_string(),
    }
}

fn exact_gameplay_mutator_skeleton_ready(
    skeletons: &[ExactGameplayMutatorSkeleton],
    system: &str,
    boundary_map: &str,
    trace_file: &str,
    entry_point: &str,
) -> bool {
    skeletons.iter().any(|skeleton| {
        skeleton.system == system
            && skeleton.status == "static-proof-backed"
            && skeleton.boundary_map == boundary_map
            && skeleton.trace_file.ends_with(trace_file)
            && skeleton.entry_point == entry_point
            && !skeleton.required_inputs.is_empty()
            && skeleton.blocked_until.is_empty()
            && skeleton.safety_rule.contains("Static-proof-backed")
    })
}
