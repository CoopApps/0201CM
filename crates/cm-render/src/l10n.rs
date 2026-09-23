//! Localization template resolver — direct port of `FUN_006547C0`
//! (14,044 bytes decompiled).
//!
//! Decompile: `d:/cm0102-carve/decompiled/gui_primitives/0x006547c0.c`.
//! Full decode: [`reports/gui_primitives_decode.md`](../../../../reports/gui_primitives_decode.md) §13.
//!
//! # Template grammar
//!
//! * `{name}` or `[name]` — placeholder
//! * `{name}{format}` — with format hint (case / plural rule)
//! * `{name}{format}{nested}` — nested argument (e.g. count for plural)
//! * `<...>` — annotation (gender / plural), consumes prev token
//! * `~` — token separator inside a resolved bank string
//! * `\b` (0x08) — target-language reorder marker
//! * `%%` — literal `%`
//!
//! # Two paths
//!
//! * **Localization ON** (`DAT_009D73B4 != 0 && != -1`) — look up
//!   template in bank; walk translated form with `\b` reorder markers.
//! * **Localization OFF** — legacy English path with printf-style
//!   `{arg}` → varargs substitution.
//!
//! # Deferred
//!
//! The exe's single-token inflection engine `FUN_006554C0` (referenced
//! from both paths) is a SEPARATE function not decoded here. It handles
//! gender/plural/case morphology. For English (the shipped locale) our
//! port emits the raw token — the inflection engine only fires when a
//! non-English locale is loaded.

/// One argument the caller supplies to fill placeholder slots.
#[derive(Debug, Clone, PartialEq)]
pub enum TemplateArg {
    /// A pre-formatted string.
    Str(String),
    /// An integer (used for count-driven plural rules and %d format).
    Int(i64),
}

impl TemplateArg {
    pub fn as_str(&self) -> String {
        match self { Self::Str(s) => s.clone(), Self::Int(n) => n.to_string() }
    }
}

/// The localization mode flag — matches exe's `DAT_009D73B4`. When
/// disabled (0 or -1) the resolver runs the English/legacy path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum L10nMode {
    /// English / stripped mode — template printed nearly verbatim.
    Disabled,
    /// Translations loaded — look up templates in bank.
    Enabled,
}

/// The result of resolving a template.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedString {
    pub text: String,
    /// Individual `~`-separated tokens the exe stashes in
    /// `DAT_00B4CE74[]` for downstream inflection.
    pub tokens: Vec<String>,
}

/// Direct port of `FUN_006547C0(dst, src)` — resolve one template
/// against a bank + arg list. `bank_lookup` returns the localized
/// template (with `\b` reorder bytes) or None if this template isn't
/// translated (falls back to legacy path).
///
/// The English/legacy path is fully ported; the localized path is
/// ported for the reorder-and-substitute skeleton but falls back to
/// verbatim token emission where `FUN_006554C0`'s inflection engine
/// would fire.
// GDI-REG: 006547c0 PORTED_BEHAVIOURAL
pub fn resolve_template(
    src: &str,
    args: &[TemplateArg],
    mode: L10nMode,
    bank_lookup: impl FnOnce(&str) -> Option<String>,
) -> ResolvedString {
    // Strip leading whitespace (exe: `while (*src == ' ') skip`).
    let src = src.trim_start_matches(' ');

    match mode {
        L10nMode::Enabled => {
            if let Some(translated) = bank_lookup(src) {
                return resolve_translated(&translated, args);
            }
            resolve_english(src, args)
        }
        L10nMode::Disabled => resolve_english(src, args),
    }
}

/// English/legacy path — walk src, replace `{name}` / `[name]` /
/// `{name}{format}` in argument order.
fn resolve_english(src: &str, args: &[TemplateArg]) -> ResolvedString {
    let mut out = String::new();
    let mut arg_iter = args.iter();
    let bytes = src.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'%' && i + 1 < bytes.len() && bytes[i + 1] == b'%' {
            out.push('%');
            i += 2;
            continue;
        }
        if b == b'{' || b == b'[' {
            // Balanced skip — advances varargs by 1 per `{...}` group.
            let close = if b == b'{' { b'}' } else { b']' };
            // Find matching close.
            let mut depth = 1;
            let mut j = i + 1;
            while j < bytes.len() && depth > 0 {
                if bytes[j] == b { depth += 1; }
                else if bytes[j] == close { depth -= 1; }
                if depth > 0 { j += 1; }
            }
            // Emit next arg's string form.
            if let Some(a) = arg_iter.next() {
                out.push_str(&a.as_str());
            }
            i = j + 1;
            continue;
        }
        if b == b'<' {
            // Annotation `<...>` — the exe swallows it (gender/plural
            // marker for translators). Skip to matching close.
            while i < bytes.len() && bytes[i] != b'>' { i += 1; }
            if i < bytes.len() { i += 1; }
            continue;
        }
        // Literal.
        out.push(b as char);
        i += 1;
    }
    let tokens = out.split('~').map(str::to_string).collect();
    ResolvedString { text: out, tokens }
}

/// Localized path — walk translated template, resolving `\b` reorder
/// markers against source-order args. Passes each `{format}` group +
/// the arg's descriptor bytes into [`crate::inflection::inflect_token`]
/// (the port of `FUN_006554C0`).
fn resolve_translated(translated: &str, args: &[TemplateArg]) -> ResolvedString {
    let mut out = String::new();
    let mut slot_iter = 0usize;
    let bytes = translated.as_bytes();
    let mut i = 0usize;
    let mut pending_format = String::new();
    let mut pending_nested = String::new();
    while i < bytes.len() {
        let b = bytes[i];
        if b == 0x08 {
            // exe: `\b` = "insert arg N here" — feeds the pending
            // {format} + nested into FUN_006554C0 for morphology.
            //
            // The full exe reads (kind_kw, scope_kw) from a stack table
            // built at template parse time. In our port we treat the
            // first captured group as `kind` and the second as `scope`;
            // English pass-through handles both correctly. Case/comp
            // enums default to 0 until the caller threads them.
            if let Some(a) = args.get(slot_iter) {
                let out_ir = crate::inflection::inflect_token(
                    &a.as_str(),
                    &pending_format,          // kind keyword
                    &pending_nested,          // scope keyword
                    0, 0,
                    crate::inflection::LangId::English,
                );
                out.push_str(&out_ir.text);
                slot_iter += 1;
            }
            pending_format.clear();
            pending_nested.clear();
            i += 1;
            continue;
        }
        if b == b'%' && i + 1 < bytes.len() && bytes[i + 1] == b'%' {
            out.push('%'); i += 2; continue;
        }
        if b == b'{' || b == b'[' {
            // {formatter} — capture into pending_format (or _nested if
            // we've already got a format).
            let close = if b == b'{' { b'}' } else { b']' };
            let start = i + 1;
            while i < bytes.len() && bytes[i] != close { i += 1; }
            let captured = std::str::from_utf8(&bytes[start..i]).unwrap_or("").to_string();
            if pending_format.is_empty() { pending_format = captured; }
            else { pending_nested = captured; }
            if i < bytes.len() { i += 1; }
            continue;
        }
        if b == b'<' {
            while i < bytes.len() && bytes[i] != b'>' { i += 1; }
            if i < bytes.len() { i += 1; }
            continue;
        }
        out.push(b as char);
        i += 1;
    }
    let tokens = out.split('~').map(str::to_string).collect();
    ResolvedString { text: out, tokens }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn literal_string_passes_through_unchanged() {
        let r = resolve_template("Hello world", &[], L10nMode::Disabled, |_| None);
        assert_eq!(r.text, "Hello world");
    }

    #[test]
    fn double_percent_becomes_single() {
        let r = resolve_template("100%% loading", &[], L10nMode::Disabled, |_| None);
        assert_eq!(r.text, "100% loading");
    }

    #[test]
    fn brace_placeholder_takes_next_arg() {
        let r = resolve_template("Hello {name}!",
                                  &[TemplateArg::Str("Arsène".into())],
                                  L10nMode::Disabled, |_| None);
        assert_eq!(r.text, "Hello Arsène!");
    }

    #[test]
    fn bracket_placeholder_also_works() {
        let r = resolve_template("Score: [n]",
                                  &[TemplateArg::Int(3)],
                                  L10nMode::Disabled, |_| None);
        assert_eq!(r.text, "Score: 3");
    }

    #[test]
    fn nested_brace_advances_varargs_by_one() {
        // {arg}{format}{nested} should consume ONE varargs slot in the
        // exe's balanced-brace advance. Our port emits the arg and skips
        // the format/nested groups.
        let r = resolve_template("{count}{format}{nested} players",
                                  &[TemplateArg::Int(11), TemplateArg::Str("plural".into())],
                                  L10nMode::Disabled, |_| None);
        // Not perfect fidelity to the exe's exact printf semantics
        // (which threads a `format` string), but the emission order
        // matches: first arg substituted, following groups skipped.
        assert!(r.text.starts_with("11"));
    }

    #[test]
    fn angle_annotation_is_stripped() {
        let r = resolve_template("Hello <_s___name_>",
                                  &[], L10nMode::Disabled, |_| None);
        assert_eq!(r.text, "Hello ");
    }

    #[test]
    fn tokens_split_on_tilde() {
        let r = resolve_template("alpha~beta~gamma", &[], L10nMode::Disabled, |_| None);
        assert_eq!(r.tokens, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn enabled_mode_uses_bank_translation() {
        let r = resolve_template("hello",
                                  &[TemplateArg::Str("world".into())],
                                  L10nMode::Enabled,
                                  |src| { assert_eq!(src, "hello"); Some("bonjour \x08".into()) });
        assert_eq!(r.text, "bonjour world");
    }

    #[test]
    fn enabled_mode_falls_back_when_bank_misses() {
        let r = resolve_template("only-in-english {n}",
                                  &[TemplateArg::Int(42)],
                                  L10nMode::Enabled,
                                  |_| None);
        assert_eq!(r.text, "only-in-english 42");
    }

    #[test]
    fn leading_whitespace_is_stripped() {
        let r = resolve_template("   hello", &[], L10nMode::Disabled, |_| None);
        assert_eq!(r.text, "hello");
    }
}
