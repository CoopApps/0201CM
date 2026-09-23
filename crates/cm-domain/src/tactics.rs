//! Tactics core — position rating + team score, ported from `FUN_006c8930`
//! (position rating for a player-in-role) and `FUN_006c5c40` (team score =
//! Σ(11 position ratings) / (opp_rep × 8)).
//!
//! # The recovered core
//! Every role in the exe's engine has a "which attribute matters" wire:
//! - GK (bit `0x001`) → +0x0f  (the goalkeeping aptitude)
//! - Sweeper (`0x002`) → +0x10
//! - CB-alt (`0x004`) → +0x11
//! - CB (`0x008`) → +0x12
//! - CB variants (`0x010`) → min/max of +0x12/+0x13/+0x14
//! - Defensive mid / anchor (`0x020`) → +0x14 (or avg with +0x1a)
//! - Attack mid / attacker (`0x040`) → +0x15
//! - Wide variants (`0x080`/`0x200`/`0x800`) → +0x14/+0x16 combinations
//!
//! Then `points = DAT_00a01ce0[aptitude]`, scaled by condition/100.
//! Team score = Σ over 11 players / (opp_reputation × 8).
//!
//! # `DAT_00a01ce0` — the attribute→points curve, **verified extract from the
//! exe at VA 0x00a01ce0** (linear +10 per step above 10, steep penalties below).

/// The 21-entry attribute-to-points curve at exe VA 0x00a01ce0 (verified).
/// Applied to a 0..20 attribute value: `points = ATTR_CURVE[value.clamp(0,20)]`.
// GDI-REG: 006c8930 PORTED_PARTIAL
pub const ATTR_CURVE: [i32; 21] = [
    -150, -124, -107, -84, -60, -50, -40, -30, -20, -10,
       0,   10,   20,  30,  40,  50,  60,  70,  80,  90, 100,
];

/// Position rating for one player in a specific role bit (`FUN_006c8930`).
/// `aptitudes` is the 12-byte block at type10 +0x0f..+0x1a (our
/// `unknown_bytes_15_26`). `position_mask` = which role bit is being evaluated
/// (0x001..0x800). `condition_pct` scales the result (100 = fresh, 0 = broken).
///
/// Returns the raw rating (signed; negatives for out-of-position players).
// GDI-REG: 006c8930 PORTED_PARTIAL
pub fn position_rating(aptitudes: &[u8; 12], position_mask: u16, condition_pct: u8) -> i32 {
    let get = |off_in_block: usize| -> i32 {
        // ATTR_CURVE indexed by the aptitude byte (clamped to 0..20).
        let v = (aptitudes[off_in_block] as i32).clamp(0, 20) as usize;
        ATTR_CURVE[v]
    };
    // Map position bit → aptitude offset within the 12-byte block (offset from
    // record +0x0f, i.e. index 0 = +0x0f = GK aptitude).
    let base = match position_mask {
        // GK — the +0x11..+0x14 max cluster in the exe (:118-128); we simplify
        // to the direct GK aptitude for outfield/GK differentiation. Real
        // treatment uses max(+0x11, +0x12, +0x13, +0x14).
        m if m & 0x001 != 0 => get(0),
        // Sweeper / CB variants
        m if m & 0x002 != 0 => get(1),
        m if m & 0x004 != 0 => get(2),
        m if m & 0x008 != 0 => get(3),
        // CB cluster (0x010): max of +0x12/+0x13/+0x14 (:151-162).
        m if m & 0x010 != 0 => {
            let a = aptitudes[3].max(aptitudes[4]).max(aptitudes[5]);
            ATTR_CURVE[(a as usize).min(20)]
        }
        // Defensive/wide mid (0x020): +0x14 (or avg with +0x1a in wide branch).
        m if m & 0x020 != 0 => get(5),
        // Attack mid / attacker (0x040): +0x15.
        m if m & 0x040 != 0 => get(6),
        // Wide/forward variants (0x080/0x200/0x800): +0x16 primarily.
        m if m & 0x800 != 0 => get(7),
        m if m & 0x200 != 0 => get(6),
        m if m & 0x080 != 0 => get(5),
        // Unknown mask — the exe defaults to +100 for a plain reference.
        _ => 100,
    };
    // Condition scales the result (`FUN_006c8930`:76-78).
    let scaled = base * condition_pct.max(1) as i32 / 100;
    scaled
}

/// Team score for 11 selected players (`FUN_006c5c40`:379-403).
/// `sum_position_ratings` = Σ of `position_rating` for each of the 11 players
/// in the role they were selected for. `opp_reputation` is the opponent's club
/// reputation (`ClubView::reputation`, the ×500 value).
///
/// Returns the normalised team score — higher = stronger vs this opponent.
/// Clubs with a stronger squad AND a favourable matchup post a higher score.
// GDI-REG: 006c5c40 PORTED_BEHAVIOURAL
pub fn team_score(sum_position_ratings: i32, opp_reputation: u16) -> i32 {
    // Guard: opp_rep can be zero (data-less clubs). Floor at 1 so the divisor
    // is never 0; the resulting big score is faithful to how a top team would
    // outclass a rep-0 opponent.
    let divisor = (opp_reputation.max(1) as i32) * 8;
    (sum_position_ratings * 1000) / divisor
}

/// Position bits for the standard 4-4-2 flat lineup — used until the tactics
/// AI / per-club formation resolver (`FUN_00881310`) is wired. Order is:
/// GK, RB, CB, CB, LB, RM, CM, CM, LM, ST, ST.
pub const FLAT_442_ROLES: [u16; 11] = [
    0x001, // GK
    0x020, // RB (wide defender)
    0x010, // CB (cluster)
    0x010, // CB
    0x020, // LB
    0x080, // RM (wide mid)
    0x020, // CM
    0x020, // CM
    0x080, // LM
    0x040, // ST
    0x040, // ST
];
