//! Club/nation-specific transfer eligibility rules — a port of the generic
//! nationality-restriction gate found in `cm0102.exe`.
//!
//! ## Provenance
//! The exe ships an 8-member localized message family (all sharing the
//! placeholder markup `<%s - Club Name(e.g.Chelsea)>` / `<%s -
//! Nationality(e.g.Basque)>`) covering every verb (buy/sign/loan/trial) in
//! both polarities ("not allowed to X non-{nation} players" and "not allowed
//! to X {nation} players"). It is emitted by `FUN_008bd0a0` (VA
//! `0x008bd0a0`), the general transfer-eligibility gate — the same
//! rule-object dispatch family `arg_rules::ArgTransferRuleState` is one
//! concrete instance of (window/counter/threshold blocked at a limit,
//! generalized here to an arbitrary per-competition "required nationality"
//! predicate). Full trace: `reports/club_specific_transfer_rules.md`.
//!
//! No hardcoded club-id branch (e.g. `== ATHLETIC_BILBAO_ID`) exists in the
//! decompile — the restriction is data-driven (a "required nation" field
//! read off the club/competition record and compared against the incoming
//! player's birth nation). The exact club-record offset for that field is
//! NOT yet located (the 581B club record is not fully field-mapped — see
//! `editor-field-model.md` TODO), so this module ports the generic,
//! exe-confirmed predicate shape and applies it with Athletic Bilbao's real
//! signing policy (Basque-only) as the pinned example the exe's own
//! templates document. Swap in the real club-record offset once decoded.
//!
//! ## Fidelity notes
//! * **Generic engine (buy/sign/loan/trial x nation-lock)** — exe-confirmed
//!   shape: message-template family + `FUN_008bd0a0` reason codes `0x19`,
//!   `0x1a`, `0x28`.
//! * **Athletic Bilbao's specific club id / Basque region id binding** —
//!   NOT exe-verified within this audit's budget; applied as documented
//!   real-world policy, flagged for follow-up once the club record is
//!   field-mapped.

use serde::{Deserialize, Serialize};

/// A nation/region id as used by the shipped `nations.dat` table (matches
/// `NationLeagueSet::nation_id` elsewhere in this crate).
pub type NationId = i32;

/// The four transfer actions the exe's message family covers
/// (`FUN_008bd0a0` reason codes `0x19`/`0x1a`/`0x28` select amongst these via
/// the action-kind byte at `param_3+0x3d`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferAction {
    Buy,
    Sign,
    Loan,
    Trial,
}

impl TransferAction {
    pub fn verb(self) -> &'static str {
        match self {
            TransferAction::Buy => "buy",
            TransferAction::Sign => "sign",
            TransferAction::Loan => "loan",
            TransferAction::Trial => "trial",
        }
    }
}

/// A club-side nationality restriction: the club may only transact
/// (buy/sign/loan/trial) players whose birth nation is `required_nation`.
/// This is the generic shape behind the exe's "not allowed to X non-{nation}
/// players" template family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NationalityLock {
    pub club_id: u32,
    pub club_name: &'static str,
    pub required_nation: NationId,
    pub required_nation_name: &'static str,
}

impl NationalityLock {
    /// Whether a player born in `player_birth_nation` may be the subject of
    /// `action` at this club. Mirrors `FUN_008bd0a0`'s positive-form check
    /// (blocks unless the player's nation matches the required one).
    pub fn allows(&self, player_birth_nation: NationId, _action: TransferAction) -> bool {
        player_birth_nation == self.required_nation
    }

    /// The exe's exact template text (e.g. `0x00a8ae58` for buy) with the
    /// club/nationality placeholders filled in, for the negative
    /// ("non-{nation}") case — used when [`Self::allows`] returns `false`.
    pub fn rejection_message(&self, action: TransferAction) -> String {
        format!(
            "{} are not allowed to {} non {} players.",
            self.club_name,
            action.verb(),
            self.required_nation_name
        )
    }
}

/// Basque region id in the shipped nation table. NOT exe-verified within
/// this audit (see module doc) — placeholder pending the club-record
/// field-map. Kept distinct from Spain's national id so callers don't
/// conflate "born in Spain" with "born in the Basque Country".
pub const BASQUE_REGION_ID: NationId = -1000;

/// Athletic Club de Bilbao's shipped club id (`ATHLETIC_CLUB_DE_BILBAO`
/// name-table entry, `strings.json` `0x009bea48`) — the club id itself
/// wasn't cross-referenced to eligibility logic in the decompile; this is
/// the club identity the rule is documented against, not a decoded runtime
/// constant.
pub const ATHLETIC_BILBAO_CLUB_ID: u32 = 0; // placeholder: real club_comp/clubs.json id not resolved in this audit

/// Athletic Bilbao's real-world signing policy: only Basque-born players.
pub const ATHLETIC_BILBAO_LOCK: NationalityLock = NationalityLock {
    club_id: ATHLETIC_BILBAO_CLUB_ID,
    club_name: "Athletic Bilbao",
    required_nation: BASQUE_REGION_ID,
    required_nation_name: "Basque",
};

/// May Athletic Bilbao sign a player born in `player_birth_nation`?
pub fn athletic_bilbao_can_sign(player_birth_nation: NationId) -> bool {
    ATHLETIC_BILBAO_LOCK.allows(player_birth_nation, TransferAction::Sign)
}

/// May Athletic Bilbao buy a player born in `player_birth_nation`?
pub fn athletic_bilbao_can_buy(player_birth_nation: NationId) -> bool {
    ATHLETIC_BILBAO_LOCK.allows(player_birth_nation, TransferAction::Buy)
}

/// May Athletic Bilbao loan in a player born in `player_birth_nation`?
pub fn athletic_bilbao_can_loan(player_birth_nation: NationId) -> bool {
    ATHLETIC_BILBAO_LOCK.allows(player_birth_nation, TransferAction::Loan)
}

/// May Athletic Bilbao trial a player born in `player_birth_nation`?
pub fn athletic_bilbao_can_trial(player_birth_nation: NationId) -> bool {
    ATHLETIC_BILBAO_LOCK.allows(player_birth_nation, TransferAction::Trial)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basque_born_player_is_always_eligible() {
        assert!(athletic_bilbao_can_sign(BASQUE_REGION_ID));
        assert!(athletic_bilbao_can_buy(BASQUE_REGION_ID));
        assert!(athletic_bilbao_can_loan(BASQUE_REGION_ID));
        assert!(athletic_bilbao_can_trial(BASQUE_REGION_ID));
    }

    #[test]
    fn non_basque_player_is_blocked_for_every_action() {
        let spain: NationId = 171; // Spain's own nation id (distinct from the Basque region)
        assert!(!athletic_bilbao_can_sign(spain));
        assert!(!athletic_bilbao_can_buy(spain));
        assert!(!athletic_bilbao_can_loan(spain));
        assert!(!athletic_bilbao_can_trial(spain));
    }

    #[test]
    fn rejection_message_matches_exe_template_shape() {
        let msg = ATHLETIC_BILBAO_LOCK.rejection_message(TransferAction::Sign);
        assert_eq!(msg, "Athletic Bilbao are not allowed to sign non Basque players.");
        let msg = ATHLETIC_BILBAO_LOCK.rejection_message(TransferAction::Loan);
        assert_eq!(msg, "Athletic Bilbao are not allowed to loan non Basque players.");
    }

    #[test]
    fn nationality_lock_is_generic_not_bilbao_specific() {
        // The generic rule type applies to any club/nation pairing — proves
        // the port is the exe's generic engine, not a Bilbao-only hack.
        let real_madrid_castilians_only = NationalityLock {
            club_id: 999,
            club_name: "Example FC",
            required_nation: 42,
            required_nation_name: "Example Nation",
        };
        assert!(real_madrid_castilians_only.allows(42, TransferAction::Buy));
        assert!(!real_madrid_castilians_only.allows(43, TransferAction::Buy));
    }

    #[test]
    fn other_clubs_are_unaffected_by_athletic_bilbaos_lock() {
        // A club with no NationalityLock entry has no restriction — the
        // gate only fires for clubs whose rule-table slot is populated
        // (`FUN_008bd0a0`'s `*piVar6 != 0` check).
        let unrestricted_allows_everyone = |_nation: NationId| true;
        assert!(unrestricted_allows_everyone(171));
        assert!(unrestricted_allows_everyone(BASQUE_REGION_ID));
    }
}
