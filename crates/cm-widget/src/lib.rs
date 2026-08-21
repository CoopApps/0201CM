//! Data-driven widget evaluator for CM0102 screens.
//!
//! # Pipeline
//!
//! ```text
//! cm0102.exe → carver (Ghidra + ui/recover_screens) → analysis/screens/<va>.json
//!                                                                     │
//!                                          cm-widget::ScreenSpec::load
//!                                                                     ▼
//!    GameState  ─────►  StateProvider  ─────►  ScreenSpec::render(&mut Surface)
//! ```
//!
//! One evaluator, N screens. Every screen ships as data (a JSON spec produced
//! by the carver). No per-screen hand-porting. The scraper's JSON schema is
//! the source of truth; this crate honours it faithfully.
//!
//! # The `Value` types (from the scraper)
//!
//! Each arg in a widget spec is a permissive object. Any of these keys may be
//! present:
//! - `literal` / `literal_hex` / `literal_signed` — a concrete integer.
//! - `string` `{addr, text}` — a Latin-1 C-string literal read from `.rdata`.
//! - `scratch_ref` `{addr, contents}` — a scratch-buffer address; `contents`
//!   present iff the scraper could resolve what's currently in it.
//! - `stack_addr` `{frame_offset, resolved_bytes?, resolved_dword?}` —
//!   pointer to a stack slot; `resolved_bytes` present for weight arrays.
//! - `global_ref` — `"DAT_00xxxxxx"`, a runtime global (BSS palette etc.)
//! - `reg` `{reg, last_set}` — unresolved register (should be rare; if hit,
//!   scraper needs an extension).
//! - `unknown` — the scraper explicitly failed to resolve.
//!
//! The `Value::as_i32()` etc helpers keep evaluator code short. A missing key
//! yields `None` — the render pass then knows to fall back or skip cleanly.

use cm_render::font::Fonts;
use cm_render::image::Image;
use cm_render::layout::{rebuild_layout, Layout};
use cm_render::panel::{
    F_BEVEL, F_HGRADIENT, F_SOLID_FILL, F_SUNKEN, F_TRANSPARENT, F_VGRADIENT,
};
use cm_render::{pack565, Surface};

/// Pack a (r,g,b) tuple into RGB565 u32.
fn pack565_rgb(rgb: (u8, u8, u8)) -> u32 {
    pack565(rgb.0, rgb.1, rgb.2) as u32
}

/// Scrollbar arrow sprites — extracted verbatim from cm0102.exe .data:
///   DAT_009b9c0c (up-arrow)   at file offset 0x5b9c0c
///   DAT_009b9c2c (down-arrow) at file offset 0x5b9c2c
/// Both are 7×4 pixel, 1-byte-per-pixel (0 = transparent, 1 = ink). Drawn
/// by FUN_00403cc0 via FUN_005cd9d0 (the 1-bit sprite blitter). Ink colour
/// comes from DAT_00ad6bda in the exe — passed through as arrow_ink here.
const ARROW_UP: [u8; 7 * 4] = [
    0, 0, 0, 1, 0, 0, 0,
    0, 0, 1, 1, 1, 0, 0,
    0, 1, 1, 1, 1, 1, 0,
    1, 1, 1, 1, 1, 1, 1,
];
const ARROW_DOWN: [u8; 7 * 4] = [
    1, 1, 1, 1, 1, 1, 1,
    0, 1, 1, 1, 1, 1, 0,
    0, 0, 1, 1, 1, 0, 0,
    0, 0, 0, 1, 0, 0, 0,
];

fn draw_scrollbar_arrow(surface: &mut Surface, x: i32, y: i32, down: bool, ink: (u8, u8, u8)) {
    let sprite = if down { &ARROW_DOWN } else { &ARROW_UP };
    let p = pack565(ink.0, ink.1, ink.2);
    for row in 0..4 {
        for col in 0..7 {
            if sprite[row * 7 + col] != 0 {
                let px = x + col as i32;
                let py = y + row as i32;
                if px >= 0 && py >= 0 && (px as usize) < Surface::W && (py as usize) < Surface::H {
                    surface.buf[py as usize * Surface::W + px as usize] = p;
                }
            }
        }
    }
}
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

// ------- spec deserialisation -------

/// One argument in a widget call — permissive, any of the fields may be present.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Value {
    pub literal: Option<i64>,
    pub literal_hex: Option<String>,
    pub literal_signed: Option<i64>,
    pub string: Option<StringVal>,
    pub scratch_ref: Option<ScratchRef>,
    pub stack_addr: Option<StackAddr>,
    pub global_ref: Option<String>,
    pub reg: Option<RegRef>,
    pub unknown: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StringVal {
    pub addr: String,
    pub text: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScratchRef {
    pub addr: String,
    #[serde(default)]
    pub contents: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StackAddr {
    pub frame_offset: i64,
    #[serde(default)]
    pub resolved_bytes: Option<Vec<i64>>,
    #[serde(default)]
    pub resolved_dword: Option<Box<Value>>,
    /// For a `menu_list.labels_ptr`: the ordered array of label Values the
    /// preceding `strcpy_scratch` calls emitted.
    #[serde(default)]
    pub resolved_labels: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegRef {
    pub reg: String,
    pub last_set: String,
}

impl Value {
    /// Try to read this value as a small integer (rect coord, flag word, event code).
    pub fn as_i32(&self) -> Option<i32> {
        if let Some(v) = self.literal_signed.or(self.literal) {
            return Some(v as i32);
        }
        None
    }

    /// Try to read this value as a u32 (for global/scratch addresses).
    pub fn as_u32(&self) -> Option<u32> {
        self.literal.map(|v| v as u32)
    }

    /// Try to read this value as a small unsigned flag word.
    pub fn as_flags(&self) -> u32 {
        self.literal.unwrap_or(0) as u32
    }

    /// If this value is a string or resolved scratch buffer, return its text.
    /// Falls back to the provider for unresolved scratch/global refs.
    pub fn as_text<'a>(&'a self, provider: &'a dyn StateProvider) -> Option<String> {
        if let Some(s) = &self.string {
            return Some(s.text.clone());
        }
        if let Some(sr) = &self.scratch_ref {
            if let Some(contents) = &sr.contents {
                return Some(contents.clone());
            }
            if let Some(addr) = u32_from_hex(&sr.addr) {
                return provider.scratch(addr);
            }
        }
        None
    }

    /// Weight arrays for area col/row weights come as `stack_addr.resolved_bytes`.
    pub fn as_weight_array(&self) -> Option<Vec<i32>> {
        self.stack_addr
            .as_ref()
            .and_then(|s| s.resolved_bytes.as_ref())
            .map(|v| v.iter().map(|w| *w as i32).collect())
    }
}

fn u32_from_hex(s: &str) -> Option<u32> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u32::from_str_radix(s, 16).ok()
}

#[derive(Debug, Clone, Deserialize)]
pub struct WidgetArg {
    pub pos: u32,
    pub name: String,
    #[serde(default)]
    pub push_at: Option<String>,
    #[serde(default)]
    pub value: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BranchCtx {
    pub at_va: String,
    pub cmp: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Widget {
    pub at_va: String,
    pub kind: String,
    pub call_target: String,
    #[serde(default)]
    pub args: Vec<WidgetArg>,
    #[serde(default)]
    pub ecx: Value,
    #[serde(default)]
    pub branch_context: Vec<BranchCtx>,
}

impl Widget {
    /// Positional lookup by argument name (handles rename tolerantly).
    pub fn arg(&self, name: &str) -> Option<&Value> {
        self.args
            .iter()
            .find(|a| a.name == name)
            .map(|a| &a.value)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScreenSpec {
    #[serde(default)]
    pub exe: String,
    pub callback_va: String,
    #[serde(default)]
    pub widget_count: u32,
    pub widgets: Vec<Widget>,
}

impl ScreenSpec {
    /// Load a spec from a `analysis/screens/<va>.json` file.
    ///
    /// Contract gate: unless the caller opts out via `CM_ALLOW_UNSIGNED=1`,
    /// this refuses to load a spec that has no matching signed contract at
    /// `<va>.contract.json`. The contract records the sha256 of the spec at
    /// sign-off time and is produced by `tools/screen_audit.py <va> --sign`,
    /// which itself refuses to sign until every arg the spec uses is
    /// decoded AND ported (per `analysis/helpers.json`).
    ///
    /// The goal: no screen ships until someone has read the full spec and
    /// committed to implementing every arg it actually uses.
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let s: Self = serde_json::from_slice(&bytes).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())
        })?;

        // Per-screen opt-out only. `CM_ALLOW_UNSIGNED` used to accept "1"
        // as a global kill switch — a hole big enough to fit the whole
        // project through. Now it accepts a comma-separated list of VAs
        // (as they appear in the filename, e.g. "008055e0,00804020").
        // Anything else, including "1", is ignored. Every allowance is
        // named and visible in shell history.
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        let allow_this = std::env::var("CM_ALLOW_UNSIGNED")
            .ok()
            .map(|v| v.split(',').any(|va| va.trim().to_ascii_lowercase() == filename))
            .unwrap_or(false);

        if !allow_this {
            let contract_path = path.with_extension("contract.json");
            // Contract identity = SHA(spec || helpers.json). Any helper edit
            // rehashes every dependent contract → auto-invalidation.
            let helpers_path = path
                .parent()
                .and_then(|p| p.parent())
                .map(|p| p.join("helpers.json"))
                .unwrap_or_else(|| std::path::PathBuf::from("helpers.json"));
            let helpers_bytes = std::fs::read(&helpers_path).unwrap_or_default();
            let mut combined = bytes.clone();
            combined.extend_from_slice(b"||");
            combined.extend_from_slice(&helpers_bytes);
            let sha = sha256_hex(&combined);
            match verify_contract(&contract_path, &sha) {
                Ok(()) => {}
                Err(reason) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::PermissionDenied,
                        format!(
                            "SCREEN GATE HELD for {}: {reason}\n\
                             Run:  python D:/cm0102-carve/tools/screen_audit.py <va>\n\
                             then: python D:/cm0102-carve/tools/screen_audit.py <va> --sign\n\
                             Override for THIS screen only: CM_ALLOW_UNSIGNED={filename} \
                             (comma-separated for multiple; global \"1\" no longer works)",
                            path.display()
                        ),
                    ));
                }
            }
        }

        Ok(s)
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    // Tiny SHA-256 implementation — kept inline so cm-widget doesn't pick up
    // an extra crate dep just for the gate. Ported from FIPS 180-4.
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1,
        0x923f82a4, 0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
        0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147,
        0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
        0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];
    let bit_len = (bytes.len() as u64) * 8;
    let mut msg = bytes.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 { msg.push(0); }
    msg.extend_from_slice(&bit_len.to_be_bytes());
    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(chunk[i*4..i*4+4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i-15].rotate_right(7) ^ w[i-15].rotate_right(18) ^ (w[i-15] >> 3);
            let s1 = w[i-2].rotate_right(17) ^ w[i-2].rotate_right(19) ^ (w[i-2] >> 10);
            w[i] = w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1);
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh.wrapping_add(s1).wrapping_add(ch).wrapping_add(K[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let mj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(mj);
            hh = g; g = f; f = e; e = d.wrapping_add(t1);
            d = c; c = b; b = a; a = t1.wrapping_add(t2);
        }
        h[0] = h[0].wrapping_add(a); h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c); h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e); h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g); h[7] = h[7].wrapping_add(hh);
    }
    let mut s = String::with_capacity(64);
    for w in h { s.push_str(&format!("{:08x}", w)); }
    s
}

fn verify_contract(contract_path: &Path, spec_sha: &str) -> Result<(), String> {
    if !contract_path.exists() {
        return Err(format!("no contract at {}", contract_path.display()));
    }
    let bytes = std::fs::read(contract_path).map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let recorded = v.get("spec_sha").and_then(|s| s.as_str()).unwrap_or("");
    if recorded != spec_sha {
        return Err(format!(
            "contract sha mismatch (spec has changed since sign-off): contract={} spec={}",
            &recorded[..recorded.len().min(16)], &spec_sha[..16]
        ));
    }
    Ok(())
}

// ------- runtime state substitution -------

/// The evaluator asks the game for values the spec references at runtime
/// (scratch-buffer contents, per-DAT globals, guard conditions). One
/// implementation per screen (or one big one for the whole app) provides them.
pub trait StateProvider {
    /// Contents of a scratch buffer at `addr` (e.g. `0xdbc380` for the banner
    /// title). Return `None` to fall back to whatever the spec captured.
    fn scratch(&self, addr: u32) -> Option<String> {
        let _ = addr;
        None
    }
    /// Value of a runtime BSS/global at `DAT_00xxxxxx`. Numeric — colours are
    /// RGB565 packed, flags are bit words, etc.
    fn global(&self, name: &str) -> Option<u32> {
        let _ = name;
        None
    }
    /// Evaluate a guard condition (as it appears in `branch_context[*].cmp`).
    /// Default is "always true" — the render walks every widget linearly.
    /// Override when a screen has an if/else zone.
    fn cond(&self, cmp: &str) -> bool {
        let _ = cmp;
        true
    }
    /// For row items whose `col`/`row` come from a `<reg …>` value in the spec,
    /// provide the current iteration indices. The evaluator passes `at_va` so
    /// the provider can key on the widget site.
    fn indexed(&self, at_va: &str, arg_name: &str) -> Option<i32> {
        let _ = (at_va, arg_name);
        None
    }
    /// Row-loop template expansion: for an item that lives in a multi-row
    /// area (nrows > 1) and appears once at row=0, the exe emits it N times
    /// inside a loop. The evaluator asks the provider for how many rows to
    /// stamp and (via `row_text`) what text to substitute per-row-per-column.
    ///
    /// `area_rect` identifies the area; `col` identifies the item; return the
    /// number of rows to render. Zero means "just one row" (default). Return
    /// value should typically match the area's nrows for the header/name
    /// column and be conditional (only if slot has secondary) for others.
    fn row_count(&self, area_rect: (i32, i32, i32, i32)) -> Option<usize> {
        let _ = area_rect;
        None
    }
    /// Text for a specific `(area, row, col)` cell — country name for col 0
    /// of the picker's row 3, etc. Return `None` to fall back to the spec's
    /// captured static text.
    fn row_text(&self, area_rect: (i32, i32, i32, i32), row: usize, col: usize) -> Option<String> {
        let _ = (area_rect, row, col);
        None
    }
    /// Per-cell text ink override (RGB565 packed as u32). This is how the
    /// exe's selection highlight works: when a slot is selected the exe
    /// re-emits the item with a different `aux_b` DAT (typically
    /// `DAT_00ad6bc4` = yellow). We surface that swap through the provider
    /// so screens can encode "this row/col is selected → yellow".
    fn cell_ink(&self, area_rect: (i32, i32, i32, i32), row: usize, col: usize) -> Option<u32> {
        let _ = (area_rect, row, col);
        None
    }
    /// Per-cell highlight box override — return `Some(rgb565)` to draw a
    /// bevelled 1px outline around this cell in the given colour. Ports
    /// the exe's `flags |= 0x800` selection-box path.
    fn cell_box(&self, area_rect: (i32, i32, i32, i32), row: usize, col: usize) -> Option<u32> {
        let _ = (area_rect, row, col);
        None
    }
    /// Ask the provider whether a specific item cell should be hidden
    /// entirely (no panel, no text, no highlight). Ports the exe's runtime
    /// skip of whole item groups — e.g. FUN_008055e0 emits the Attribute
    /// Masking widgets ONLY when [0x9a2051] (use_real_players) is 1, so the
    /// port hides those cells when the option is off. Returning true means
    /// skip this cell for the current frame.
    fn cell_hidden(&self, area_rect: (i32, i32, i32, i32), row: usize, col: usize) -> bool {
        let _ = (area_rect, row, col);
        false
    }
    /// Index of the currently-selected label in a menu_list widget. Ports
    /// the exe's `1 << selected_idx` bitmask (built in FUN_00807280:0x807488)
    /// that FUN_005d6bf0 tests per label at line 141 to pick Path A
    /// (unselected: aux_b=DAT_00acdf74, flags=0x22) vs Path B (selected:
    /// aux_b=DAT_00ad6bdc, flags=0x822 which adds the 0x800 highlight box).
    fn menu_selected_index(&self, area_rect: (i32, i32, i32, i32)) -> Option<usize> {
        let _ = area_rect;
        None
    }
    /// Runtime labels for a `menu_list` widget. When the spec's `labels_ptr`
    /// is a stack pointer without resolved labels (built at runtime by the
    /// exe's own code — e.g. Start Season's per-league season options), the
    /// provider supplies the list. Return `None` to defer to whatever the
    /// spec captured statically. `area_rect` = `(l, t, r, b)` from the
    /// menu_list widget so the provider can key by location.
    fn menu_labels(&self, area_rect: (i32, i32, i32, i32)) -> Option<Vec<String>> {
        let _ = area_rect;
        None
    }
    /// Runtime override for the item highlight box. The exe re-emits an
    /// item with `flags |= 0x800` when the option it represents is the
    /// currently-selected one; the static spec captures whatever state was
    /// live at scrape time. To render the current state we override that
    /// per (area, row, col).
    ///
    /// Returns:
    ///   `HighlightHint::Default` (default): honour the spec's `flags & 0x800`
    ///   `HighlightHint::Suppress`: don't draw a highlight even if spec asks
    ///   `HighlightHint::Force(rgb)`: draw with this colour even if spec doesn't
    fn cell_highlight(&self, area_rect: (i32, i32, i32, i32), row: usize, col: usize) -> HighlightHint {
        let _ = (area_rect, row, col);
        HighlightHint::Default
    }
    /// Per-cell panel fill override — return `Some(rgb565)` to force this
    /// cell to draw as a solid-fill bevelled button in the given colour,
    /// regardless of the item's own flags. Used for picker-row cells that
    /// are visually buttons even though the item's flags field doesn't
    /// carry F_SOLID_FILL (bit 0x1 in `flags` remains undecoded; this hook
    /// bridges the gap until we decode it).
    fn cell_fill(&self, area_rect: (i32, i32, i32, i32), row: usize, col: usize) -> Option<u32> {
        let _ = (area_rect, row, col);
        None
    }
    /// Scroll offset (in rows) for a multi-row list area. Return 0 for the
    /// top-of-list case; increment as the user scrolls.
    fn scroll_offset(&self, area_rect: (i32, i32, i32, i32)) -> usize {
        let _ = area_rect;
        0
    }
    /// Total row count backing the list — separate from the visible
    /// `row_count`, which is capped by the area's `nrows`. Used to size the
    /// scrollbar thumb. Return `None` when the list is not scrollable.
    fn total_rows(&self, area_rect: (i32, i32, i32, i32)) -> Option<usize> {
        let _ = area_rect;
        None
    }
}

/// A trivial provider that returns nothing — the evaluator falls back to
/// whatever the spec statically captured. Useful for smoke-testing a spec's
/// static content before wiring real GameState.
pub struct NullProvider;
impl StateProvider for NullProvider {}

/// Highlight-box decision per cell — allows the provider to override the
/// spec's static `flags & 0x800` in either direction.
#[derive(Debug, Clone, Copy)]
pub enum HighlightHint {
    Default,
    Suppress,
    Force(u32),
}

// ------- evaluator -------

/// Palette bindings. Kept out of the spec because these are runtime BSS
/// globals — we let a `StateProvider` inject them where the spec says
/// `global_ref: DAT_00xxxxxx`, and fall back to these when the provider
/// doesn't know.
pub struct Palette {
    /// `DAT_00ad6bc4` — selected/highlight foreground.
    pub highlight_fg: (u8, u8, u8),
    /// `DAT_00acdf6e` — default text foreground.
    pub default_fg: (u8, u8, u8),
    /// `DAT_00acdf74` — title / header text foreground.
    pub title_fg: (u8, u8, u8),
    /// `DAT_00acdf98` — title / bevel background baseline.
    pub title_bg: (u8, u8, u8),
    /// `DAT_00ad6bf4` — navy button base.
    pub btn_blue: (u8, u8, u8),
    /// `DAT_00ad6bdc` — grey/disabled.
    pub grey: (u8, u8, u8),
    /// Banner red (used in `banner_and_title`; there's no single DAT_).
    pub banner_red: (u8, u8, u8),
    /// Sidebar gradient base blue (matches `sidebar_gradient_matches_game_mbr`).
    pub sidebar_blue: (u8, u8, u8),
    /// Near-white text ink.
    pub near_white: (u8, u8, u8),
    /// Dark ink for dark-on-light text (Back/Next label).
    pub dark_ink: (u8, u8, u8),
}

impl Default for Palette {
    /// Values locked by `banner_bevel_matches_grab` and
    /// `sidebar_gradient_matches_game_mbr` tests in `cm-render`.
    fn default() -> Self {
        Palette {
            highlight_fg: (255, 255, 0),
            default_fg: (231, 227, 231),
            title_fg: (231, 227, 231),
            title_bg: (0, 0, 0),
            btn_blue: (0, 0, 132),
            grey: (132, 130, 132),
            banner_red: (255, 0, 0),
            sidebar_blue: (0, 0, 132),
            near_white: (231, 227, 231),
            dark_ink: (30, 30, 30),
        }
    }
}

/// A registered area (from a `FUN_00549790` call) — indexed later by items
/// via their `area_handle`. Rects are absolute-pixel; layout is precomputed
/// so per-item lookup is O(1).
struct AreaState {
    layout: Layout,
    #[allow(dead_code)]
    rect: (i32, i32, i32, i32),
    #[allow(dead_code)]
    flags: u32,
}

/// State carried through a render pass — palette, current areas, scratch snapshots.
struct EvalCtx<'a> {
    palette: &'a Palette,
    provider: &'a dyn StateProvider,
    /// Areas keyed by their `ecx` VA (the GUI-pool slot pointer the exe uses
    /// as the area handle). Item widgets pass `area_handle` referencing one.
    areas: HashMap<u32, AreaState>,
    /// Most-recently-registered area — items with an unresolved `area_handle`
    /// (a `<reg …>` value) fall back to this. Matches the exe's convention
    /// of items following immediately after their area declaration.
    last_area: Option<u32>,
}

impl<'a> EvalCtx<'a> {
    fn area_for(&self, area_handle_arg: Option<&Value>) -> Option<&AreaState> {
        if let Some(v) = area_handle_arg {
            if let Some(addr) = v.as_u32() {
                if let Some(a) = self.areas.get(&addr) {
                    return Some(a);
                }
            }
        }
        self.last_area.and_then(|k| self.areas.get(&k))
    }
}

impl ScreenSpec {
    /// Render this screen into `surface`. Every widget is visited in the order
    /// the exe emits them. Background image (if any) is blitted first.
    pub fn render(
        &self,
        surface: &mut Surface,
        fonts: &mut Fonts,
        bg: Option<&Image>,
        palette: &Palette,
        provider: &dyn StateProvider,
    ) {
        match bg {
            Some(img) => surface.blit_image(img, 0, 0),
            None => surface.fill(12, 12, 16),
        }
        let mut ctx = EvalCtx {
            palette,
            provider,
            areas: HashMap::new(),
            last_area: None,
        };
        for w in &self.widgets {
            // Honour guard conditions the scraper attached. Provider decides
            // truth; unknown conditions default to True (render everything).
            if !w
                .branch_context
                .iter()
                .all(|c| ctx.provider.cond(&c.cmp))
            {
                continue;
            }
            match w.kind.as_str() {
                "area" => self.eval_area(w, surface, &mut ctx),
                "item" => self.eval_item(w, surface, fonts, &mut ctx),
                "sidebar" => self.eval_sidebar(w, surface, fonts, &ctx),
                "nav_bar" => self.eval_nav_bar(w, surface, fonts, &ctx),
                "menu_list" => self.eval_menu_list(w, surface, fonts, &ctx),
                // strcpy_scratch / sprintf_scratch update scratch buffers —
                // the scraper baked their side effects into subsequent
                // `text_ptr` args (`scratch_ref.contents`), so we don't need
                // to re-run them at eval time.
                _ => {}
            }
        }
    }

    fn eval_area(&self, w: &Widget, surface: &mut Surface, ctx: &mut EvalCtx) {
        let l = w.arg("l").and_then(|v| v.as_i32()).unwrap_or(0);
        let t = w.arg("t").and_then(|v| v.as_i32()).unwrap_or(0);
        let r = w.arg("r").and_then(|v| v.as_i32()).unwrap_or(0);
        let b = w.arg("b").and_then(|v| v.as_i32()).unwrap_or(0);
        let ncols = w.arg("ncols").and_then(|v| v.as_i32()).unwrap_or(1).max(1) as usize;
        let nrows = w.arg("nrows").and_then(|v| v.as_i32()).unwrap_or(1).max(1) as usize;
        let flags = w.arg("flags").map(|v| v.as_flags()).unwrap_or(0);
        // Column weights: prefer resolved bytes; else equal-weight fallback.
        let cw: Vec<i32> = w
            .arg("col_weights_ptr")
            .and_then(|v| v.as_weight_array())
            .filter(|xs| xs.len() >= ncols)
            .map(|xs| xs.into_iter().take(ncols).collect())
            .unwrap_or_else(|| vec![1; ncols]);
        let rw = vec![1i32; nrows]; // TODO: row-weights resolution (rare; scraper doesn't emit them yet).
        let scrollbar = false;
        let layout = rebuild_layout((l, t, r, b), flags, &cw, &rw, scrollbar);
        // Handle key: use the ecx VA if we have one, else the widget's own VA
        // (parsed from `at_va`), so items can look us up.
        let key = ctx_area_key(w, ctx);
        ctx.areas.insert(
            key,
            AreaState {
                layout,
                rect: (l, t, r, b),
                flags,
            },
        );
        ctx.last_area = Some(key);

        // Areas paint their own background when they carry a fill flag. The
        // toggle strip on Select League(s) (110,145,525,165) has flags=0x2
        // (F_TRANSPARENT — dim underlying pixels), which is what puts the
        // grey bar behind "Use Real Players: Yes No" and "Attribute Masking:
        // Yes No". Without this, the area only registers layout and the
        // Yes/No text sits on the bare background image. Colour_a defaults
        // to grey when unresolved — matches the exe's neutral panel colour.
        let paints_panel =
            flags & (F_TRANSPARENT | F_HGRADIENT | F_VGRADIENT | F_SOLID_FILL | F_BEVEL) != 0;
        if paints_panel {
            let color_a = self.resolve_color(w.arg("color_a"), flags, ctx);
            surface.draw_panel(l, t, r, b, flags, color_a);
        }

        // Scrollbar — ported from FUN_00403640 (geometry) + FUN_00403cc0
        // (drawing) in cm0102.exe. Not a guess: reads the exe's decompile
        // directly.
        //
        // Trigger: FUN_00403390 calls FUN_00403640 when `visible_rows <
        // total_rows + 1`. Reserves 21 px of area width (0x15) and draws
        // the scrollbar 19 px wide (`r - reserve_x - 19` … `r - reserve_x`).
        //
        // Reserve padding (from FUN_00403390 line 51-55):
        //   flags & 0x1  → reserve_x = 0, reserve_y = 0
        //   otherwise    → reserve_x = 2; reserve_y = 2 if (flags & 0x10)
        //                                or nrows == 1, else 8.
        //
        // Track / arrow buttons (from FUN_00403640 line 12-22):
        //   scrollbar_l = r - reserve_x - 19
        //   scrollbar_r = r - reserve_x
        //   scrollbar_t = t + reserve_y
        //   scrollbar_b = b - reserve_y
        //   arrow_up_bottom = scrollbar_t + 20    (up-button is 20 px tall)
        //   arrow_down_top  = scrollbar_b - 20    (down-button is 20 px tall)
        //
        // Thumb (from FUN_00403640 line 45-59):
        //   scroll_range = total_rows - visible_rows + 1
        //   track_h      = arrow_down_top - arrow_up_bottom + 1
        //   thumb_h      = track_h * visible / (scroll_range + visible)
        //                  (min 10)
        //   thumb_top    = arrow_up_bottom +
        //                  (track_h - thumb_h) * scroll_pos / scroll_range
        //   thumb_bottom = thumb_top - 1 + thumb_h
        //
        // Drawing sequence (FUN_00403cc0 line 41-66):
        //   1. Up-arrow button:   F_SOLID_FILL|F_BEVEL, DAT_00acdf6e (grey)
        //   2. Up-arrow icon:     sprite DAT_009b9c0c, ink DAT_00ad6bda
        //   3. Track:             F_TRANSPARENT, DAT_00ad6bda (dimming)
        //   4. Thumb:             F_SOLID_FILL|F_BEVEL, DAT_00acdf6e (grey)
        //   5. Down-arrow button: F_SOLID_FILL|F_BEVEL, DAT_00acdf6e (grey)
        //   6. Down-arrow icon:   sprite DAT_009b9c2c, ink DAT_00ad6bda
        if let Some(total) = ctx.provider.total_rows((l, t, r, b)) {
            let visible = nrows;
            if total > visible {
                let (reserve_x, reserve_y) = if flags & 0x1 != 0 {
                    (0, 0)
                } else {
                    let ry = if flags & 0x10 != 0 || nrows == 1 { 2 } else { 8 };
                    (2, ry)
                };
                let sb_l = r - reserve_x - 19;
                let sb_r = r - reserve_x;
                let sb_t = t + reserve_y;
                let sb_b = b - reserve_y;
                let arrow_up_bot = sb_t + 20;
                let arrow_dn_top = sb_b - 20;
                let track_h = (arrow_dn_top - arrow_up_bot + 1).max(1);
                let scroll_range = (total - visible + 1) as i32;
                let mut thumb_h = track_h * (visible as i32) / (scroll_range + visible as i32);
                if thumb_h < 11 { thumb_h = 10; }
                let scroll_pos = ctx.provider.scroll_offset((l, t, r, b)) as i32;
                let thumb_top = arrow_up_bot
                    + (track_h - thumb_h) * scroll_pos / scroll_range.max(1);
                let thumb_bot = thumb_top - 1 + thumb_h;

                let grey = ctx.palette.grey;
                // Track: DAT_00ad6bda not yet decoded; fall back to dim grey.
                let track_dim = (60, 60, 60);
                let arrow_ink = (0, 0, 0);

                // Up-arrow button
                surface.draw_panel(sb_l, sb_t, sb_r, arrow_up_bot - 1, F_SOLID_FILL | F_BEVEL, grey);
                draw_scrollbar_arrow(
                    surface,
                    (sb_l + sb_r) / 2 - 3,
                    (sb_t + arrow_up_bot - 1) / 2 - 1,
                    /*down=*/ false,
                    arrow_ink,
                );
                // Track between arrows
                surface.draw_panel(sb_l, arrow_up_bot, sb_r, arrow_dn_top, F_TRANSPARENT, track_dim);
                // Thumb
                surface.draw_panel(sb_l, thumb_top, sb_r, thumb_bot, F_SOLID_FILL | F_BEVEL, grey);
                // Down-arrow button
                surface.draw_panel(sb_l, arrow_dn_top + 1, sb_r, sb_b, F_SOLID_FILL | F_BEVEL, grey);
                draw_scrollbar_arrow(
                    surface,
                    (sb_l + sb_r) / 2 - 3,
                    (arrow_dn_top + 1 + sb_b) / 2 - 1,
                    /*down=*/ true,
                    arrow_ink,
                );
            }
        }
    }

    fn eval_item(
        &self,
        w: &Widget,
        surface: &mut Surface,
        fonts: &mut Fonts,
        ctx: &mut EvalCtx,
    ) {
        let (l0, t0, r0, b0) = (
            w.arg("l").and_then(|v| v.as_i32()).unwrap_or(0),
            w.arg("t").and_then(|v| v.as_i32()).unwrap_or(0),
            w.arg("r").and_then(|v| v.as_i32()).unwrap_or(0),
            w.arg("b").and_then(|v| v.as_i32()).unwrap_or(0),
        );
        let col = w.arg("col").and_then(|v| v.as_i32()).unwrap_or(0) as usize;
        let declared_row = w.arg("row").and_then(|v| v.as_i32()).unwrap_or(0) as usize;

        let uses_area = (l0, t0, r0, b0) == (0, 0, 0, 0);
        let area = if uses_area { ctx.area_for(w.arg("area_handle")) } else { None };

        // Row-template N-plication: when the item lives in a multi-row area
        // (nrows > 1) and appears at row=0, the exe emits it inside a loop
        // once per data row. Ask the provider for the row count.
        let (row_start, row_end) = match area {
            Some(a) if uses_area && declared_row == 0 && a.layout.row_top.len() > 1 => {
                let n = ctx.provider.row_count(a.rect).unwrap_or(1);
                (0usize, n.min(a.layout.row_top.len()))
            }
            _ => (declared_row, declared_row + 1),
        };

        let flags = w.arg("flags").map(|v| v.as_flags()).unwrap_or(0);
        let color_a = self.resolve_color(w.arg("color_a"), flags, ctx);
        let font_slot = w.arg("font").and_then(|v| v.as_i32()).unwrap_or(3).max(0) as u8;
        let slot = if font_slot <= 8 { font_slot } else { 3 };
        let default_text = w
            .arg("text_ptr")
            .and_then(|v| v.as_text(ctx.provider));

        for row in row_start..row_end {
            let rect = if uses_area {
                match area {
                    Some(a) if col < a.layout.col_left.len() && row < a.layout.row_top.len() => (
                        a.layout.col_left[col],
                        a.layout.row_top[row],
                        a.layout.col_right[col],
                        a.layout.row_bottom[row],
                    ),
                    _ => continue,
                }
            } else {
                (l0, t0, r0, b0)
            };

            // Runtime hide: if the provider says this cell is hidden for the
            // current frame, skip drawing it entirely. Ports the exe's
            // conditional widget skips (e.g. the Attribute Masking group
            // vanishes when use_real_players is 0).
            if uses_area {
                if let Some(a) = area {
                    if ctx.provider.cell_hidden(a.rect, row, col) {
                        continue;
                    }
                }
            }

            // Cell fill override: provider can force this cell to draw as a
            // solid grey button (or any colour) even when the spec's flags
            // don't ask for a fill. Picker-row buttons use this because
            // their captured flags=0x1 doesn't set F_SOLID_FILL.
            let cell_fill_override = if uses_area {
                area.and_then(|a| ctx.provider.cell_fill(a.rect, row, col))
            } else {
                None
            };
            let (effective_flags, effective_color) = if let Some(rgb) = cell_fill_override {
                (flags | F_SOLID_FILL | F_BEVEL, unpack565(rgb as u16))
            } else {
                (flags, color_a)
            };
            let paints_panel = effective_flags & (F_TRANSPARENT | F_HGRADIENT | F_VGRADIENT | F_SOLID_FILL | F_BEVEL) != 0;
            if paints_panel {
                surface.draw_panel(rect.0, rect.1, rect.2, rect.3, effective_flags, effective_color);
            }

            // Text: prefer per-row provider text (for country names etc.),
            // fall back to whatever the spec captured statically.
            let text = if uses_area && row_end > 1 {
                area.and_then(|a| ctx.provider.row_text(a.rect, row, col))
                    .or_else(|| default_text.clone())
            } else {
                default_text.clone()
            };
            if let Some(text) = text {
                // Text ink resolution order:
                //   1. Provider cell_ink override (per-row selection state).
                //   2. `aux_b` DAT resolved via palette map — the exe's real
                //      text-ink parameter. Selection highlight is done by
                //      swapping aux_b to DAT_00ad6bc4 (yellow).
                //   3. Fallback: for F_SOLID_FILL buttons pick white or dark
                //      by luminance; for non-fill items use palette.default_fg.
                let per_cell = if uses_area {
                    area.and_then(|a| ctx.provider.cell_ink(a.rect, row, col))
                } else {
                    None
                };
                let aux_b_ink = if per_cell.is_none() {
                    self.resolve_ink(w.arg("aux_b"), ctx)
                } else {
                    None
                };
                // Text-box alignment flags from aux_a (FUN_005d0870 param_5):
                // 0x1 = F_LEFT, 0x40 = F_RIGHT, 0x2 = F_NOVCENTER, 0x20 = F_SHADOW.
                let text_flags = w.arg("aux_a").and_then(|v| v.as_i32()).unwrap_or(0) as u32;
                // Ink resolution order:
                //   1. Provider cell_ink override.
                //   2. Provider-forced cell_fill → white ink.
                //   3. Shadowed-text button (aux_a has F_SHADOW=0x20 AND
                //      panel is solid-filled) → WHITE ink; aux_b becomes the
                //      shadow colour, not the text colour. This is what
                //      distinguishes buttons (Select All: aux_a=0x2c, WHITE)
                //      from banner titles (banner: aux_a=0xc, YELLOW).
                //      In CM01/02: enabled button = white on grey; NEVER yellow.
                //   4. `aux_b` DAT for non-shadow items (banners, headers).
                //   5. Luminance fallback.
                let ink = if let Some(rgb) = per_cell {
                    unpack565(rgb as u16)
                } else if cell_fill_override.is_some() {
                    ctx.palette.near_white
                } else if text_flags & 0x20 != 0 && effective_flags & F_SOLID_FILL != 0 {
                    ctx.palette.near_white
                } else if let Some(rgb) = aux_b_ink {
                    rgb
                } else {
                    let (fr, fg, fb) = effective_color;
                    let very_light = fr >= 240 && fg >= 240 && fb >= 240;
                    if effective_flags & F_SOLID_FILL != 0 {
                        if very_light { ctx.palette.dark_ink } else { ctx.palette.near_white }
                    } else {
                        ctx.palette.default_fg
                    }
                };
                let f = fonts.slot(slot);
                surface.draw_text_box(rect.0, rect.1, rect.2, rect.3, text_flags, f, ink, &text);

                // Selection highlight box — provider decides via HighlightHint,
                // else fall back to the spec's static `flags & 0x800`.
                let hint = if uses_area {
                    area.map(|a| ctx.provider.cell_highlight(a.rect, row, col))
                        .unwrap_or(HighlightHint::Default)
                } else {
                    HighlightHint::Default
                };
                let box_rgb = match hint {
                    HighlightHint::Force(rgb) => Some(rgb),
                    HighlightHint::Suppress => None,
                    HighlightHint::Default => {
                        // Legacy cell_box path (still used by some providers)
                        // then the static spec highlight bit.
                        let legacy = if uses_area {
                            area.and_then(|a| ctx.provider.cell_box(a.rect, row, col))
                        } else {
                            None
                        };
                        legacy.or_else(|| {
                            if flags & 0x800 != 0 {
                                Some(pack565_rgb(ctx.palette.highlight_fg))
                            } else {
                                None
                            }
                        })
                    }
                };
                if let Some(rgb) = box_rgb {
                    surface.draw_hollow_rect(rect.0, rect.1, rect.2, rect.3, unpack565(rgb as u16));
                }
            }
        }
    }

    /// Resolve `aux_b` (or any other DAT-ref-or-literal) as a TEXT INK.
    /// Distinct from resolve_color which is for panel fills: text-ink args
    /// are the exe's mechanism for selection highlight (swap DAT →
    /// DAT_00ad6bc4 = yellow). Returns None when the value is missing or
    /// a null literal, so the caller can fall through to its own default.
    fn resolve_ink(&self, v: Option<&Value>, ctx: &EvalCtx) -> Option<(u8, u8, u8)> {
        let val = v?;
        if let Some(name) = &val.global_ref {
            if let Some(rgb) = ctx.provider.global(name) {
                return Some(unpack565(rgb as u16));
            }
            // Same DAT map as resolve_color. Text-ink DATs commonly used:
            //   DAT_00acdf74 → yellow header text (title/label emphasis)
            //   DAT_00ad6bc4 → highlight yellow (SELECTED state)
            //   DAT_00ad6bdc → grey label text (unselected picker names)
            //   DAT_00acdf6e → light grey (unselected state text)
            return Some(match name.as_str() {
                "DAT_00ad6bc4" => ctx.palette.highlight_fg,
                "DAT_00acdf74" => ctx.palette.highlight_fg,
                "DAT_00ad6bdc" => (200, 200, 200),
                "DAT_00acdf6e" => (200, 200, 200),
                "DAT_00acdf98" => ctx.palette.near_white,
                _ => return None,
            });
        }
        if let Some(lit) = val.as_u32() {
            if lit == 0 { return None; }
            if lit <= 0xffff {
                return Some(unpack565(lit as u16));
            }
        }
        None
    }

    fn resolve_color(&self, v: Option<&Value>, flags: u32, ctx: &EvalCtx) -> (u8, u8, u8) {
        if let Some(val) = v {
            if let Some(name) = &val.global_ref {
                if let Some(rgb565) = ctx.provider.global(name) {
                    return unpack565(rgb565 as u16);
                }
                // Palette DAT bindings. Established by observation:
                // - `acdf98` = red banner background (locked by banner_bevel test).
                // - `acdf6e` = **grey button fill** — the Select All / De-Select All
                //   pills and other neutral-action buttons pass this as color_a
                //   with F_SOLID_FILL. Result should be white text on grey.
                // - `acdf74` = title / header text (yellow in most CM screens).
                // - `ad6bc4` = highlight (yellow on selected).
                // - `ad6bdc` = also grey / disabled variant.
                // - `ad6bf4` = navy blue (setup menu buttons).
                match name.as_str() {
                    "DAT_00ad6bc4" => return ctx.palette.highlight_fg,
                    "DAT_00acdf6e" => return ctx.palette.grey,
                    "DAT_00acdf74" => return ctx.palette.highlight_fg,
                    "DAT_00acdf98" => return ctx.palette.banner_red,
                    "DAT_00ad6bf4" => return ctx.palette.btn_blue,
                    "DAT_00ad6bdc" => return ctx.palette.grey,
                    _ => {}
                }
            }
            if let Some(lit) = val.as_u32() {
                // Literal RGB565 (rare — most colours come via DAT_ globals).
                if lit <= 0xffff {
                    return unpack565(lit as u16);
                }
            }
        }
        // Flag-driven default: solid fill = red banner (the only "colour" a
        // static banner_and_title item asks for); bevel-and-transparent = navy.
        if flags & F_SOLID_FILL != 0 {
            ctx.palette.banner_red
        } else {
            ctx.palette.btn_blue
        }
    }

    fn eval_sidebar(
        &self,
        w: &Widget,
        surface: &mut Surface,
        fonts: &mut Fonts,
        ctx: &EvalCtx,
    ) {
        // FUN_00745540(mode, aux). Layout comes from that helper: area (5,10,85,590),
        // 1 col × 13 rows, flags=1. Mode affects which rows are populated;
        // provider decides that. We paint the gradient + a default 5-row layout
        // matching what the hand-porting used.
        let _mode = w.arg("mode").and_then(|v| v.as_i32()).unwrap_or(0);
        surface.draw_panel(0, 0, 89, 599, F_VGRADIENT, ctx.palette.sidebar_blue);
        let side = rebuild_layout((5, 10, 85, 590), 1, &[1], &[1; 13], false);
        let side2 = rebuild_layout((5, 10, 85, 590), 1, &[1, 1], &[1; 13], false);
        let f = fonts.slot(1);
        let yellow = (255, 255, 0);
        surface.draw_text_box(
            side.col_left[0], side.row_top[0], side.col_right[0], side.row_bottom[0],
            0, f, yellow, "Version\n3.9.60",
        );
        surface.draw_text_box(
            side2.col_left[0], side2.row_top[1], side2.col_right[0], side2.row_bottom[1],
            0, f, yellow, "<<<",
        );
        surface.draw_text_box(
            side2.col_left[1], side2.row_top[1], side2.col_right[1], side2.row_bottom[1],
            0, f, yellow, ">>>",
        );
        surface.draw_text_box(
            side.col_left[0], side.row_top[2], side.col_right[0], side.row_bottom[2],
            0, f, (140, 140, 140), "Add\nManager",
        );
        surface.draw_text_box(
            side.col_left[0], side.row_top[3], side.col_right[0], side.row_bottom[3],
            0, f, yellow, "Restart\nGame",
        );
        surface.draw_text_box(
            side.col_left[0], side.row_top[4], side.col_right[0], side.row_bottom[4],
            0, f, yellow, "Exit\nGame",
        );
    }

    /// `FUN_005d6bf0(l, t, r, b, ncols, nrows, font, label_count, labels_ptr, flags)`.
    /// The exe builds a `ncols × nrows` grid inside `(l..r, t..b)` using our layout
    /// engine and emits one text-only item per label (F_TRANSPARENT | F_BEVEL for
    /// hoverable feel — matches the setup screen's blue-bordered menu buttons).
    /// Labels come from `labels_ptr.resolved_labels`, one per menu row.
    fn eval_menu_list(
        &self,
        w: &Widget,
        surface: &mut Surface,
        fonts: &mut Fonts,
        ctx: &EvalCtx,
    ) {
        let l = w.arg("l").and_then(|v| v.as_i32()).unwrap_or(0);
        let t = w.arg("t").and_then(|v| v.as_i32()).unwrap_or(0);
        let r = w.arg("r").and_then(|v| v.as_i32()).unwrap_or(0);
        let b = w.arg("b").and_then(|v| v.as_i32()).unwrap_or(0);
        let ncols = w.arg("ncols").and_then(|v| v.as_i32()).unwrap_or(2).max(1) as usize;
        let nrows = w.arg("nrows").and_then(|v| v.as_i32()).unwrap_or(1).max(1) as usize;
        let font_slot = w.arg("font").and_then(|v| v.as_i32()).unwrap_or(4).max(0) as u8;
        let font_slot = if font_slot <= 8 { font_slot } else { 3 };
        // Labels: prefer whatever the provider supplies at runtime (Start
        // Season's per-league season list, e.g.), else fall back to the
        // static list the spec captured. Empty → nothing to draw.
        let labels: Vec<String> = ctx
            .provider
            .menu_labels((l, t, r, b))
            .unwrap_or_else(|| {
                w.arg("labels_ptr")
                    .and_then(|v| v.stack_addr.as_ref())
                    .and_then(|s| s.resolved_labels.as_ref())
                    .map(|xs| {
                        xs.iter()
                            .filter_map(|lv| lv.string.as_ref().map(|s| s.text.clone()))
                            .collect()
                    })
                    .unwrap_or_default()
            });
        if labels.is_empty() {
            return; // nothing to render
        }
        let layout = rebuild_layout((l, t, r, b), 1, &vec![1; ncols], &vec![1; nrows], false);
        // The item ctors under FUN_005d6bf0 place items row-major, spilling the
        // last-partial row into the centre. Approximation: N items where N =
        // ncols*rows_used, then leftovers centred on the next row (matches the
        // setup menu's "Web Sites" centred on row 4 of 6).
        for (idx, label) in labels.iter().enumerate() {
            let full_rows = labels.len() / ncols;
            let (col, row, cell) = if idx < full_rows * ncols {
                let r = idx / ncols;
                let c = idx % ncols;
                (c, r, (layout.col_left[c], layout.row_top[r], layout.col_right[c], layout.row_bottom[r]))
            } else {
                // Last partial row centered (Web Sites style).
                let r = full_rows;
                let leftover = labels.len() - full_rows * ncols;
                let mid_l = (layout.col_left[0] + layout.col_right[ncols - 1]) / 2;
                let half = ((layout.col_right[0] - layout.col_left[0]) + 3) / 2;
                let cell_l = mid_l - half + ((idx - full_rows * ncols) as i32 - leftover as i32 / 2) * (half * 2 + 2);
                let cell_r = cell_l + half * 2 - 1;
                (0, r, (cell_l, layout.row_top[r], cell_r, layout.row_bottom[r]))
            };
            let (l2, t2, r2, b2) = cell;
            let _ = (col, row);
            // Per-label item emission mirrors FUN_005d6bf0:141-165:
            //   Unselected (Path A): flags=0x22 (F_TRANSPARENT|F_BEVEL),
            //                        aux_b = DAT_00acdf74 → white text
            //   Selected   (Path B): flags=0x822 (adds 0x800 highlight),
            //                        aux_b = DAT_00ad6bdc → yellow text +
            //                        1px yellow outline around the bevel
            let selected_idx = ctx.provider.menu_selected_index((l, t, r, b));
            let is_selected = selected_idx == Some(idx);
            surface.draw_panel(l2, t2, r2, b2, F_TRANSPARENT | F_BEVEL, ctx.palette.btn_blue);
            let f = fonts.slot(font_slot);
            let ink = if is_selected {
                ctx.palette.highlight_fg   // yellow
            } else {
                ctx.palette.near_white     // white for the unselected majority
            };
            surface.draw_text_box(l2, t2, r2, b2, 0, f, ink, label);
            if is_selected {
                // Yellow 1px outline OUTSIDE the blue bevel (per user
                // description). Draws one pixel outboard of the button rect.
                surface.draw_hollow_rect(l2 - 1, t2 - 1, r2 + 1, b2 + 1, ctx.palette.highlight_fg);
            }
        }
    }

    fn eval_nav_bar(
        &self,
        w: &Widget,
        surface: &mut Surface,
        fonts: &mut Fonts,
        ctx: &EvalCtx,
    ) {
        let back = w.arg("back_flag").and_then(|v| v.as_i32()).unwrap_or(0) != 0;
        let next = w.arg("next_flag").and_then(|v| v.as_i32()).unwrap_or(0) != 0;
        let _ = (back, next); // future: dim disabled buttons
        let nav = rebuild_layout((100, 555, 790, 590), 1, &[3, 1], &[1], false);
        let f = fonts.slot(3);
        for (rect, label) in [(nav.cell(0, 0), "Back"), (nav.cell(1, 0), "Next")].iter() {
            let (l, t, r, b) = *rect;
            surface.draw_panel(l, t, r, b, F_SOLID_FILL | F_BEVEL, ctx.palette.grey);
            surface.draw_text_box(l, t, r, b, 0, f, ctx.palette.dark_ink, label);
        }
    }
}

fn ctx_area_key(w: &Widget, ctx: &EvalCtx) -> u32 {
    // Prefer the `ecx` value the exe used as the area handle (the GUI-pool
    // slot pointer). If it isn't a static literal, fall back to the widget's
    // own `at_va` — items pass area_handle=this-VA in practice.
    if let Some(v) = w.ecx.as_u32() {
        return v;
    }
    let _ = ctx;
    u32_from_hex(&w.at_va).unwrap_or(0)
}

// ------- helpers -------

#[inline]
fn unpack565(v: u16) -> (u8, u8, u8) {
    let r5 = (v >> 11) & 0x1f;
    let g6 = (v >> 5) & 0x3f;
    let b5 = v & 0x1f;
    (
        ((r5 << 3) | (r5 >> 2)) as u8,
        ((g6 << 2) | (g6 >> 4)) as u8,
        ((b5 << 3) | (b5 >> 2)) as u8,
    )
}

// ------- tests -------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_loads_from_carve_json() {
        // Setup screen spec produced by the carver.
        let path = Path::new("D:/cm0102-carve/analysis/screens/00804020.json");
        if !path.exists() {
            eprintln!("skip: {} not present", path.display());
            return;
        }
        let spec = ScreenSpec::load(path).expect("load setup screen spec");
        assert!(!spec.widgets.is_empty());
        assert_eq!(spec.callback_va, "0x804020");
        // Must have at least the banner item (rect 100,10,790,70) and the sidebar helper.
        let has_banner = spec.widgets.iter().any(|w| {
            w.kind == "item"
                && w.arg("l").and_then(|v| v.as_i32()) == Some(100)
                && w.arg("t").and_then(|v| v.as_i32()) == Some(10)
                && w.arg("r").and_then(|v| v.as_i32()) == Some(790)
                && w.arg("b").and_then(|v| v.as_i32()) == Some(70)
        });
        let has_sidebar = spec.widgets.iter().any(|w| w.kind == "sidebar");
        assert!(has_banner, "spec should contain the red banner item");
        assert!(has_sidebar, "spec should contain the sidebar call");
    }
}
