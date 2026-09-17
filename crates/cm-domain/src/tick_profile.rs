//! Tick performance instrumentation.
//!
//! Zero-cost when disabled: every span is behind a cached `bool` read,
//! and the timing call is skipped entirely when off. Enable by setting
//! `CM_TICK_PROFILE=1`.
//!
//! This measures **implementation cost only**. It must never change
//! what the tick does — no span may be placed so that its presence
//! alters RNG consumption, mutation order, or scheduling. See the
//! project's fidelity contract: optimisation may change implementation
//! radically, but not observable behaviour.
//!
//! Usage:
//!
//! ```ignore
//! let _s = tick_profile::span("fixtures");   // times until end of scope
//! ```

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Instant;

/// 0 = not yet checked, 1 = disabled, 2 = enabled.
static STATE: AtomicU8 = AtomicU8::new(0);

/// Is profiling switched on for this process?
#[inline]
pub fn enabled() -> bool {
    match STATE.load(Ordering::Relaxed) {
        1 => false,
        2 => true,
        _ => {
            let on = std::env::var("CM_TICK_PROFILE")
                .map(|v| v != "0" && !v.is_empty())
                .unwrap_or(false);
            STATE.store(if on { 2 } else { 1 }, Ordering::Relaxed);
            on
        }
    }
}

thread_local! {
    static ACC: RefCell<BTreeMap<&'static str, (u128, u64)>> =
        RefCell::new(BTreeMap::new());
}

/// An open timing span. Records on drop.
pub struct Span {
    label: &'static str,
    start: Instant,
}

impl Drop for Span {
    fn drop(&mut self) {
        let nanos = self.start.elapsed().as_nanos();
        let label = self.label;
        ACC.with(|a| {
            let mut a = a.borrow_mut();
            let e = a.entry(label).or_insert((0, 0));
            e.0 += nanos;
            e.1 += 1;
        });
    }
}

/// Start a span. Returns `None` (and costs one atomic load) when
/// profiling is off.
#[inline]
pub fn span(label: &'static str) -> Option<Span> {
    if enabled() {
        Some(Span { label, start: Instant::now() })
    } else {
        None
    }
}

/// One accumulated measurement.
pub struct Entry {
    pub label: &'static str,
    pub millis: f64,
    pub calls: u64,
}

/// Drain the accumulated timings, heaviest first.
pub fn take() -> Vec<Entry> {
    let mut out: Vec<Entry> = ACC.with(|a| {
        a.borrow()
            .iter()
            .map(|(label, (nanos, calls))| Entry {
                label,
                millis: *nanos as f64 / 1.0e6,
                calls: *calls,
            })
            .collect()
    });
    out.sort_by(|x, y| y.millis.partial_cmp(&x.millis).unwrap_or(std::cmp::Ordering::Equal));
    out
}

/// Clear accumulated timings (e.g. between measured days).
pub fn reset() {
    ACC.with(|a| a.borrow_mut().clear());
}

/// Render the current timings as a table, heaviest first.
pub fn report(header: &str) -> String {
    let entries = take();
    let total: f64 = entries.iter().map(|e| e.millis).sum();
    let mut s = format!("\n=== {header} ===\n");
    s.push_str(&format!(
        "{:<38} {:>10} {:>8} {:>7}\n", "span", "ms", "calls", "%"));
    for e in &entries {
        let pct = if total > 0.0 { e.millis / total * 100.0 } else { 0.0 };
        s.push_str(&format!(
            "{:<38} {:>10.1} {:>8} {:>6.1}%\n", e.label, e.millis, e.calls, pct));
    }
    s.push_str(&format!("{:<38} {:>10.1}\n", "TOTAL (sum of spans)", total));
    s
}
