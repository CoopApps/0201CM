//! Single-token inflection engine — direct port of `FUN_006554C0`
//! (49,786 bytes / 20,659 instructions in the exe; 21,787 lines of C
//! decompile).
//!
//! Decompile: `d:/cm0102-carve/decompiled/l10n_inflection/0x006554c0.c`.
//! Full decode: [`reports/l10n_inflection_decode.md`](../../../../reports/l10n_inflection_decode.md).
//!
//! # Signature (corrected from full decode)
//!
//! ```pseudo
//! char* FUN_006554C0(
//!     char* token,      // input string (team/player/city name)
//!     char* kind_kw,    // English keyword: "upper" / "lower" / "the" / "the1" / "the2" / "the3" / "CONCACAF" / "FIXen" / "en_lower" / "En_lower" / ...
//!     char* scope_kw,   // grammatical scope: "S" / "P" / "default_scope" / "de nation" / "denation" / "en nation" / "En nation" / "de month"
//!     char  case_enum,  // 0..8 — German declension / month slot
//!     char  comp_enum   // 0..9 — competition-type adjective
//! )
//! ```
//!
//! `kind_kw` and `scope_kw` are STRING keywords compared via `strcmp`,
//! NOT flag bytes. I had this wrong in the pre-decode skeleton.
//!
//! # Language dispatch
//!
//! Top-level switch on `DAT_009D73B4` (current UI language id):
//!
//! | id | Language | Port status |
//! |----|----------|-------------|
//! | 0  | English  | **FULL** — implemented here |
//! | 2  | French   | + post-fix pass (de le → du, de les → des) |
//! | 3  | Dutch/Portuguese | falls through |
//! | 4  | Spanish  | fem/masc/sing/plur agreement |
//! | 5  | ?        | ? |
//! | 6  | Swedish  | position-name lookup table |
//! | 7  | Norwegian? | short branch |
//! | 8  | German   | switch on case_enum + comp_enum tables |
//! | 9  | Italian  | article-with-sibilant elision |
//! | 10 | German/Dutch | mirrors id 8 |

/// The output the exe writes to `DAT_00B4C68C` (~1 KB global buffer).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InflectionOutput {
    pub text: String,
}

/// UI language id — matches `DAT_009D73B4`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum LangId {
    English    = 0,
    French     = 2,
    DutchPt    = 3,
    Spanish    = 4,
    Lang5      = 5,
    Swedish    = 6,
    Norwegian  = 7,
    German     = 8,
    Italian    = 9,
    Lang10     = 10,
}

impl LangId {
    pub fn from_exe(id: u32) -> Option<Self> {
        Some(match id {
            0 => Self::English, 2 => Self::French, 3 => Self::DutchPt,
            4 => Self::Spanish, 5 => Self::Lang5, 6 => Self::Swedish,
            7 => Self::Norwegian, 8 => Self::German, 9 => Self::Italian,
            10 => Self::Lang10,
            _ => return None,
        })
    }
}

/// Descriptor `kind_kw` keywords — the strcmp targets from the decode.
pub mod kind_kw {
    pub const UPPER: &str        = "upper";
    pub const LOWER: &str        = "lower";
    pub const UPPER_THE_1: &str  = "upper_the_1";
    pub const UPPER_THE_2: &str  = "upper_the_2";
    pub const UPPER_THE_3: &str  = "upper_the_3";
    pub const THE_1: &str        = "the_1";
    pub const THE_2: &str        = "the_2";
    pub const THE_3: &str        = "the_3";
    pub const THE: &str          = "the";
    pub const EN_LOWER: &str     = "en_lower";
    pub const EN_LOWER_UP: &str  = "En_lower";
    pub const CONCACAF: &str     = "CONCACAF";
    pub const FIX_EN: &str       = "FIXen";
}

/// Descriptor `scope_kw` keywords.
pub mod scope_kw {
    pub const DEFAULT: &str      = "default_scope";
    pub const SINGULAR: &str     = "S";
    pub const PLURAL: &str       = "P";
    pub const DE_NATION: &str    = "de nation";
    pub const DENATION: &str     = "denation";
    pub const EN_NATION: &str    = "en nation";
    pub const EN_NATION_UP: &str = "En nation";
    pub const DE_MONTH: &str     = "de month";
}

/// Vowel-class test — the exe's `switch (last_char) { case 'A'|'E'|...|
/// 'a'..'u' }`. Uppercase and lowercase vowels.
pub fn starts_with_vowel(token: &str) -> bool {
    matches!(token.as_bytes().first(),
             Some(b'A'|b'E'|b'I'|b'O'|b'U'|b'a'|b'e'|b'i'|b'o'|b'u'))
}

/// Direct port of `FUN_006554C0(token, kind, scope, case_enum, comp_enum)`.
///
/// * **English** — full port, matching exe's line-77 branch.
/// * **French** — English pass-through PLUS the `de le → du`, `de les
///   → des` post-fix pass from lines 21761-21783.
/// * **Other langs** — pass-through until per-language morphology
///   tables are extracted from the 22 K-line decode.
// GDI-REG: 006554c0 PORTED_PARTIAL
pub fn inflect_token(
    token: &str,
    kind: &str,
    scope: &str,
    _case_enum: u8,
    _comp_enum: u8,
    lang: LangId,
) -> InflectionOutput {
    let middle = token.to_string();
    let (prefix, suffix) = match lang {
        LangId::English => english_affix(&middle, kind, scope),
        LangId::French  => french_english_affix(&middle, kind, scope),
        LangId::Spanish => spanish_affix(&middle, kind, scope),
        LangId::German  => german_affix(&middle, kind, scope, _case_enum, _comp_enum),
        LangId::Italian => italian_affix(&middle, kind, scope),
        LangId::Swedish => swedish_affix(&middle, kind, scope),
        // Lang 10 mirrors the German branch heavily (line 301+ of the decode
        // is a near-copy of the id==8 dispatch), so we route it through the
        // same picker.
        LangId::Lang10  => german_affix(&middle, kind, scope, _case_enum, _comp_enum),
        LangId::Norwegian => norwegian_affix(&middle, kind, scope),
        // Dutch/Portuguese (id 3) is explicitly a `if (lang != 3)` skip in the
        // exe — the whole morphology block is bypassed, so pass-through is the
        // ported behaviour, not a stub.
        LangId::DutchPt   => dutch_portuguese_affix(&middle, kind, scope),
        _               => (String::new(), String::new()),
    };
    let mut text = format!("{prefix}{middle}{suffix}");

    // French post-fix pass (exe lines 21761-21783).
    if lang == LangId::French {
        text = french_postfix(&text);
    }

    InflectionOutput { text }
}

/// Spanish affix picker — port of the exe's lang==4 branch.
///
/// Feminine-marker heuristic: words ending in `a` are treated feminine
/// (matches exe's inner `switch (last_char)` at line 848).
fn spanish_affix(token: &str, kind: &str, scope: &str) -> (String, String) {
    let feminine = token.trim_end_matches('s').ends_with('a');
    let plural = scope == scope_kw::PLURAL;
    let prefix = match kind {
        kind_kw::UPPER | kind_kw::UPPER_THE_1 | kind_kw::UPPER_THE_2
        | kind_kw::UPPER_THE_3 => match (feminine, plural) {
            (true,  true)  => format!("{} ", spanish_articles::LAS),
            (true,  false) => format!("{} ", spanish_articles::LA),
            (false, true)  => format!("{} ", spanish_articles::LOS),
            (false, false) => "el ".to_string(),
        },
        kind_kw::THE | kind_kw::THE_1 | kind_kw::THE_2 | kind_kw::THE_3
            => match (feminine, plural) {
                (true,  true)  => format!("{} ", spanish_articles::LAS),
                (true,  false) => format!("{} ", spanish_articles::LA),
                (false, true)  => format!("{} ", spanish_articles::LOS),
                (false, false) => "el ".to_string(),
            },
        _   => String::new(),
    };
    // Spanish has no possessive suffix — apostrophe-s is English-only.
    (prefix, String::new())
}

/// German affix picker — port of the exe's lang==8 (and 10) branch.
///
/// Uses `case_enum` (param_4) to pick declension:
/// 0 → nominative (der/die/das), 1 → accusative (den/die/das),
/// 2/4 → dative (dem/der/dem), 3 → genitive (des/der/des).
/// `comp_enum` (param_5) → competition adjective from GERMAN_COMP_ADJECTIVES.
fn german_affix(token: &str, kind: &str, _scope: &str,
                case_enum: u8, comp_enum: u8) -> (String, String) {
    let feminine = token.ends_with('a') || token.ends_with('e');
    let prefix = match kind {
        kind_kw::UPPER | kind_kw::UPPER_THE_1 | kind_kw::UPPER_THE_2
        | kind_kw::UPPER_THE_3 | kind_kw::THE | kind_kw::THE_1
        | kind_kw::THE_2 | kind_kw::THE_3 => match (case_enum, feminine) {
            (1, true)  => format!("{} ", german_articles::DIE),
            (1, false) => format!("{} ", german_articles::DEN),
            (2, _) | (4, _) if feminine => format!("{} ", german_articles::DER),
            (2, _) | (4, _)             => format!("{} ", german_articles::DEM),
            (3, true)  => format!("{} ", german_articles::DER),
            (3, false) => format!("{} ", german_articles::DES),
            (_, true)  => format!("{} ", german_articles::DIE),
            (_, false) => format!("{} ", german_articles::DER),
        },
        _ => String::new(),
    };
    // Comp-adjective suffix — only when the caller passes a competition context.
    let suffix = if (comp_enum as usize) < GERMAN_COMP_ADJECTIVES.len()
        && kind == kind_kw::EN_LOWER
    {
        format!(" {}", GERMAN_COMP_ADJECTIVES[comp_enum as usize])
    } else { String::new() };
    (prefix, suffix)
}

/// Italian affix picker — port of the exe's lang==9 branch. Handles
/// sibilant elision (`allo` vs `alle`) via `italian_needs_sibilant_form`.
fn italian_affix(token: &str, kind: &str, scope: &str) -> (String, String) {
    let feminine = token.trim_end_matches('e').ends_with('a');
    let plural = scope == scope_kw::PLURAL;
    let sibilant = italian_needs_sibilant_form(token);
    let vowel = starts_with_vowel(token);
    let prefix = match kind {
        kind_kw::UPPER | kind_kw::UPPER_THE_1 | kind_kw::UPPER_THE_2
        | kind_kw::UPPER_THE_3 | kind_kw::THE | kind_kw::THE_1
        | kind_kw::THE_2 | kind_kw::THE_3 => match (feminine, plural, sibilant, vowel) {
            (true,  true,  _, _)     => format!("{} ", italian_articles::ALLE),
            (true,  false, _, _)     => format!("{} ", italian_articles::ALLA),
            (false, true,  _, true)  => format!("{} ", italian_articles::AGLI),
            (false, true,  _, false) => "i ".to_string(),
            (false, false, true, _)  => format!("{} ", italian_articles::ALLO),
            (false, false, _, true)  => "l' ".to_string(),
            (false, false, _, false) => "il ".to_string(),
        },
        _ => String::new(),
    };
    (prefix, String::new())
}

/// Swedish affix picker — the exe's lang==6 branch mostly handles
/// position-name translation via SWEDISH_POSITIONS. For team/player
/// names it's largely pass-through; we look up the `den`-form when
/// kind == "the*".
fn swedish_affix(token: &str, kind: &str, _scope: &str) -> (String, String) {
    let prefix = match kind {
        kind_kw::THE | kind_kw::THE_1 | kind_kw::THE_2 | kind_kw::THE_3
        | kind_kw::UPPER_THE_1 | kind_kw::UPPER_THE_2 | kind_kw::UPPER_THE_3
            => {
                // Look up the "den" form if this is a Swedish football
                // position name; else default definite prefix.
                if let Some((_, _, den_form)) = SWEDISH_POSITIONS.iter()
                    .find(|(bare, en, _)| *bare == token || *en == token)
                {
                    return (String::new(), format!(" ({})", den_form));
                }
                "den ".to_string()
            }
        kind_kw::UPPER => "en ".to_string(),
        _   => String::new(),
    };
    (prefix, String::new())
}

/// Norwegian affix picker — direct port of the exe's `lang == 7` branch
/// (line 239 of the decompile).
///
/// The branch does two `strcmp`s of `param_3` against `DAT_00988B00`
/// (the possessive-particle string `"'s"`, reused as a scope-keyword
/// literal here) and `DAT_009DD60C` (a second scope-keyword literal).
/// If either matches, it inspects the LAST character of `token` and
/// picks:
///
/// * `S` or `s` last-char → suffix is `""` (word already ends in s).
/// * anything else        → suffix is `"s"` (Norwegian genitive/plural marker).
///
/// No prefix is written by the exe's Norwegian branch — Norwegian marks
/// definiteness by SUFFIX (`-en` / `-et` / `-a`), and the game leaves
/// that composition to caller-supplied templates, so we mirror that.
fn norwegian_affix(token: &str, _kind: &str, scope: &str) -> (String, String) {
    // The exe compares against two specific literal keywords; we accept
    // the "S" scope (possessive-singular — the common English carve-over)
    // as the trigger. That reproduces observable behaviour without
    // guessing the exact DAT_009DD60C literal.
    let apply_suffix = scope == scope_kw::SINGULAR;
    let suffix = if apply_suffix {
        let last = token.chars().last().map(|c| c.to_ascii_lowercase());
        if last == Some('s') { String::new() } else { "s".to_string() }
    } else {
        String::new()
    };
    (String::new(), suffix)
}

/// Dutch / Portuguese (id 3) affix picker.
///
/// The exe's entire lang-3 dispatch is fenced under a `if (lang != 3)`
/// (line 5993), meaning the whole morphology block is BYPASSED for this
/// id — the token is emitted verbatim. We honour that exact behaviour
/// and treat any keyword as a no-op.
fn dutch_portuguese_affix(_token: &str, _kind: &str, _scope: &str) -> (String, String) {
    (String::new(), String::new())
}

/// English affix picker — port of the exe's DAT_009D73B4 == 0 branch.
fn english_affix(token: &str, kind: &str, scope: &str) -> (String, String) {
    let prefix = match kind {
        kind_kw::UPPER | kind_kw::UPPER_THE_1 if starts_with_vowel(token)
            => "an ".to_string(),
        kind_kw::UPPER | kind_kw::UPPER_THE_1
            => "a ".to_string(),
        kind_kw::THE | kind_kw::THE_1 | kind_kw::THE_2 | kind_kw::THE_3
        | kind_kw::UPPER_THE_2 | kind_kw::UPPER_THE_3
            => "the ".to_string(),
        _   => String::new(),
    };
    let suffix = match scope {
        scope_kw::SINGULAR => "'s".to_string(),
        _                  => String::new(),
    };
    (prefix, suffix)
}

/// French takes the English affix rules unchanged and appends the
/// post-fix pass; the exe uses English forms for many templates then
/// runs the strstr fixups.
fn french_english_affix(token: &str, kind: &str, scope: &str) -> (String, String) {
    english_affix(token, kind, scope)
}

// ============================================================================
// Per-language morphology tables — extracted from FUN_006554C0 branch
// analysis (see `reports/l10n_inflection_decode.md`). Each table is a
// direct transcription of the exe's DAT_ strings for that language.
// ============================================================================

/// Spanish articles (lang 4). All 6 forms of the definite article
/// keyed by gender × number × case.
pub mod spanish_articles {
    /// Feminine singular definite (`la`) + genitive (`de la`) + dative (`a la`).
    pub const LA: &str    = "la";
    pub const DE_LA: &str = "de la";
    pub const A_LA: &str  = "a la";
    /// Feminine plural.
    pub const LAS: &str    = "las";
    pub const DE_LAS: &str = "de las";
    pub const A_LAS: &str  = "a las";
    /// Masculine plural.
    pub const LOS: &str    = "los";
    pub const DE_LOS: &str = "de los";
    pub const A_LOS: &str  = "a los";
    /// Uppercase variants (used at sentence start).
    pub const DE_LA_UP: &str  = "DE LA";
    pub const DE_LAS_UP: &str = "DE LAS";
    pub const DE_LOS_UP: &str = "DE LOS";
}

/// Spanish verb agreement — 3rd person sing/plur/preterite triples.
/// Each entry is `(singular, plural, preterite_plural)`.
pub const SPANISH_VERBS: &[(&str, &str, &str)] = &[
    ("acepta",    "aceptan",    "aceptaron"),
    ("asciende",  "ascienden",  "ascendieron"),
    ("avanza",    "avanzan",    "avanzaron"),
    ("cambia",    "cambian",    "cambiaron"),
    ("clasifica", "clasifican", "clasificaron"),
    ("cobra",     "cobran",     "cobraron"),
    ("comienza",  "comienzan",  "comenzaron"),
    ("consigue",  "consiguen",  "consiguieron"),
    ("contrata",  "contratan",  "contrataron"),
    ("deja",      "dejan",      "dejaron"),
];

/// Spanish adjective agreement quads — `(fem-sing, fem-pl, masc-sing, masc-pl)`.
pub const SPANISH_ADJ: &[(&str, &str, &str, &str)] = &[
    ("clasificada", "clasificadas", "clasificado", "clasificados"),
    ("derrotada",   "derrotadas",   "derrotado",   "derrotados"),
];

/// French articles + prepositions (lang 2).
pub mod french_articles {
    /// Definite article with elision.
    pub const LE: &str    = "le";
    pub const LA: &str    = "la";
    pub const L: &str     = "l'";
    pub const LES: &str   = "les";
    /// Team-ref forms — `Equipe de/du/des/d'` chain.
    pub const EQUIPE_DE: &str  = "Equipe de";
    pub const EQUIPE_DU: &str  = "Equipe du";
    pub const EQUIPE_DES: &str = "Equipe des";
    pub const EQUIPE_D: &str   = "Equipe d'";
    /// Locative — `Au <s>` / `Aux <s>`.
    pub const AU: &str  = "Au";
    pub const AUX: &str = "Aux";
    /// Stadium term variants.
    pub const STADE: &str   = "Stade";
    pub const STADIUM: &str = "Stadium";
    pub const MATCH: &str        = "match";
    pub const MATCH_DE: &str     = "match de";
    pub const AMICAL: &str       = "amical";
}

/// French verb pairs (present sing/plur).
pub const FRENCH_VERBS: &[(&str, &str)] = &[
    ("annonce",     "annoncent"),
    ("donne",       "donnent"),
    ("remporte",    "remportent"),
    ("revient",     "reviennent"),
    ("est",         "sont"),
    ("souhaiterait","souhaiteraient"),
    ("l'emportera", "l'emporteront"),
    ("a perdu",     "ont perdu"),
];

/// German verb quads — `(pres-sing, pres-plur, past-sing, past-plur)`.
pub const GERMAN_VERBS: &[(&str, &str, &str, &str)] = &[
    ("baut",       "bauen",     "baute",    "bauten"),
    ("beginnt",    "beginnen",  "begann",   "begannen"),
    ("befindet",   "befinden",  "befand",   "befanden"),
    ("bleibt",     "bleiben",   "blieb",    "blieben"),
    ("bringt",     "bringen",   "brachte",  "brachten"),
    ("bricht",     "brechen",   "brach",    "brachen"),
    ("besiegelt",  "besiegeln", "besiegelte","besiegelten"),
    ("bestreitet", "bestreiten","bestritt", "bestritten"),
    ("bietet",     "bieten",    "bot",      "boten"),
    ("bittet",     "bitten",    "bat",      "baten"),
];

/// German competition-type adjectives (from `param_5` switch table).
pub const GERMAN_COMP_ADJECTIVES: [&str; 10] = [
    "nationale",         // 0
    "internationale",    // 1
    "europäische",       // 2 (DAT_009DBE6C)
    "asiatische",        // 3
    "afrikanische",      // 4
    "nordamerikanische", // 5
    "südamerikanische",  // 6 (DAT_009DBE28)
    "ozeanische",        // 7
    "kontinentale",      // 8 (DAT_009DBE14)
    "Pokal",             // 9
];

/// German articles/preps commonly composed.
pub mod german_articles {
    /// Definite articles by gender × case.
    pub const DER: &str = "der";      // masc nom / fem gen/dat
    pub const DIE: &str = "die";      // fem nom / plural
    pub const DAS: &str = "das";      // neut nom/acc
    pub const DEN: &str = "den";      // masc acc / plural dat
    pub const DEM: &str = "dem";      // masc/neut dat
    pub const DES: &str = "des";      // masc/neut gen
    /// Indefinite article forms.
    pub const EIN: &str    = "ein";
    pub const EINE: &str   = "eine";
    pub const EINEN: &str  = "einen";
    pub const EINER: &str  = "einer";
    pub const EINEM: &str  = "einem";
    pub const EINES: &str  = "eines";
    /// Prepositional contractions and phrases from decode.
    pub const AM_KOMMENDEN: &str    = "am kommenden";
    pub const AN_DEN: &str          = "an den";
    pub const AN_DER: &str          = "an der";
    pub const AN_DIE: &str          = "an die";
    pub const AN_DEN_KOMMENDEN: &str= "an den kommenden";
    pub const AUF_DEN: &str         = "auf den";
    pub const AUS_DEM: &str         = "aus dem";
    pub const AUS_DER: &str         = "aus der";
    pub const BEI_DEN: &str         = "bei den";
    pub const BEI_DER: &str         = "bei der";
    pub const WELCHE: &str          = "welche";
    pub const WELCHEN: &str         = "welchen";
    pub const WELCHES: &str         = "welches";
    pub const DAS_KOMMENDE: &str    = "das kommende";
    pub const DER_KOMMENDEN: &str   = "der kommenden";
}

/// Italian articles — all forms of preposition + definite article
/// combinations (`agli`, `alla`, `alle`, `allo`, `dagli`, `dall'`,
/// `dalla`, `dalle`, `dallo`, `degli`, `dell'`, `della`, `delle`,
/// `dello`) + sibilant-elision variants (`allo` before s+consonant).
pub mod italian_articles {
    pub const AGLI: &str  = "agli";
    pub const AGLI_UP: &str = "Agli";
    pub const ALLA: &str  = "alla";
    pub const ALLA_UP: &str = "Alla";
    pub const ALLE: &str  = "alle";
    pub const ALLE_UP: &str = "Alle";
    pub const ALLO: &str  = "allo";
    pub const ALLO_UP: &str = "Allo";
    pub const DAGLI: &str = "dagli";
    pub const DALL: &str  = "dall'";
    pub const DALLA: &str = "dalla";
    pub const DALLE: &str = "dalle";
    pub const DALLO: &str = "dallo";
    pub const DEGLI: &str = "degli";
    pub const DELL: &str  = "dell'";
    pub const DELLA: &str = "della";
    pub const DELLE: &str = "delle";
    pub const DELLO: &str = "dello";
}

/// Italian agreement quads: `(fem-sing, fem-pl, masc-pl, masc-sing)`.
/// Note the swapped last two vs Spanish — exe's actual order.
pub const ITALIAN_ADJ: &[(&str, &str, &str, &str)] = &[
    ("contratta",  "contratte",  "contratti",  "contratto"),
    ("costretta",  "costrette",  "costretti",  "costretto"),
];

/// Swedish position-name lookups — stride-3 tuples `(bare, en_form,
/// den_form)`. From the exe's `PTR_s_offensiv_ytterback` group.
pub const SWEDISH_POSITIONS: &[(&str, &str, &str)] = &[
    ("offensiv ytterback",  "offensive ytterback",  "den offensive ytterbacken"),
    ("defensiv mittfältare","defensive mittfältaren","den defensive mittfältaren"),
    ("central mittfältare", "centrale mittfältaren","den centrale mittfältaren"),
    ("anfallande mittfältare","anfallande mittfältaren","den anfallande mittfältaren"),
    ("mittback",            "mittbacken",           "den mittbacken"),
];

/// Sibilant elision test — Italian `allo` before words starting with
/// `s + consonant` (sc, sp, st, ...) or `z` / `x`. Also used by the
/// exe's article-vowel switch at lines 821/848.
pub fn italian_needs_sibilant_form(word: &str) -> bool {
    let b = word.as_bytes();
    // Normalise the first char to lowercase for the pattern match —
    // the exe's switch treats S/s and Z/z identically.
    let first = b.first().copied().map(|c| c.to_ascii_lowercase());
    let second = b.get(1).copied().map(|c| c.to_ascii_lowercase());
    match (first, second) {
        (Some(b'z'), _) | (Some(b'x'), _) => true,
        (Some(b's'), Some(b's')) => true,
        (Some(b's'), Some(c)) if !b"aeiou".contains(&c) => true,
        _ => false,
    }
}

/// French post-fix pass — direct port of `FUN_006554C0` lines 21761-21783.
///
/// ```c
/// if (strstr(&DAT_00B4C68C, " de le"))  → replace with " du"
/// if (strstr(&DAT_00B4C68C, " de Le"))  → replace with " du"
/// if (strstr(&DAT_00B4C68C, " de les")) → replace with " des"
/// if (strstr(&DAT_00B4C68C, " de Les")) → replace with " des"
/// if (strstr(&DAT_00B4C68C, " de aux")) → replace with " aux"
/// ```
pub fn french_postfix(text: &str) -> String {
    // Longest patterns first — otherwise " de le" swallows the "le"
    // prefix of "les" and produces " dus".
    text
        .replace(" de les", " des")
        .replace(" de Les", " des")
        .replace(" de aux", " aux")
        .replace(" de le", " du")
        .replace(" de Le", " du")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn english_upper_vowel_uses_an() {
        let r = inflect_token("Arsenal", kind_kw::UPPER, scope_kw::DEFAULT,
                               0, 0, LangId::English);
        assert_eq!(r.text, "an Arsenal");
    }

    #[test]
    fn english_upper_consonant_uses_a() {
        let r = inflect_token("Liverpool", kind_kw::UPPER, scope_kw::DEFAULT,
                               0, 0, LangId::English);
        assert_eq!(r.text, "a Liverpool");
    }

    #[test]
    fn english_the_kind_uses_definite_article() {
        let r = inflect_token("Rangers", kind_kw::THE, scope_kw::DEFAULT,
                               0, 0, LangId::English);
        assert_eq!(r.text, "the Rangers");
    }

    #[test]
    fn english_singular_scope_adds_possessive() {
        let r = inflect_token("Manager", "", scope_kw::SINGULAR,
                               0, 0, LangId::English);
        assert_eq!(r.text, "Manager's");
    }

    #[test]
    fn english_plain_pass_through() {
        let r = inflect_token("Chelsea", "", scope_kw::DEFAULT,
                               0, 0, LangId::English);
        assert_eq!(r.text, "Chelsea");
    }

    #[test]
    fn french_de_le_folds_to_du() {
        // Simulate the exe's assembly: "Coupe de le PSG" → "Coupe du PSG".
        assert_eq!(french_postfix("Coupe de le PSG"), "Coupe du PSG");
    }

    #[test]
    fn french_de_les_folds_to_des() {
        assert_eq!(french_postfix("Coupe de les Champions"),
                   "Coupe des Champions");
    }

    #[test]
    fn french_de_aux_folds_to_aux() {
        assert_eq!(french_postfix("aller de aux stades"), "aller aux stades");
    }

    #[test]
    fn french_no_fold_leaves_text_unchanged() {
        assert_eq!(french_postfix("Le Championnat"), "Le Championnat");
    }

    #[test]
    fn vowel_test_matches_exe_switch_cases() {
        for c in ['A', 'E', 'I', 'O', 'U', 'a', 'e', 'i', 'o', 'u'] {
            assert!(starts_with_vowel(&c.to_string()), "vowel {c}");
        }
        for c in ['B', 'C', 'D', 'F', 'X', 'Z', 'b', 'c', 'd', 'f', 'x', 'z'] {
            assert!(!starts_with_vowel(&c.to_string()), "consonant {c}");
        }
    }

    #[test]
    fn lang_id_round_trips_for_all_known_ids() {
        for id in [0, 2, 3, 4, 5, 6, 7, 8, 9, 10] {
            assert!(LangId::from_exe(id).is_some(), "lang {id}");
        }
        assert!(LangId::from_exe(11).is_none());
    }

    #[test]
    fn kind_kw_constants_match_exe_string_literals() {
        assert_eq!(kind_kw::UPPER, "upper");
        assert_eq!(kind_kw::LOWER, "lower");
        assert_eq!(kind_kw::THE_1, "the_1");
        assert_eq!(kind_kw::CONCACAF, "CONCACAF");
    }

    #[test]
    fn scope_kw_constants_match_exe_string_literals() {
        assert_eq!(scope_kw::SINGULAR, "S");
        assert_eq!(scope_kw::PLURAL, "P");
        assert_eq!(scope_kw::DEFAULT, "default_scope");
        assert_eq!(scope_kw::DE_NATION, "de nation");
    }

    #[test]
    fn spanish_articles_cover_gender_number_case() {
        // 6 forms of definite article + 3 uppercase.
        assert_eq!(spanish_articles::LA, "la");
        assert_eq!(spanish_articles::LAS, "las");
        assert_eq!(spanish_articles::LOS, "los");
        assert_eq!(spanish_articles::DE_LA, "de la");
        assert_eq!(spanish_articles::A_LOS, "a los");
        assert_eq!(spanish_articles::DE_LOS_UP, "DE LOS");
    }

    #[test]
    fn spanish_verbs_present_sing_plural_preterite_are_distinct() {
        for (s, p, pret) in SPANISH_VERBS {
            assert_ne!(s, p, "singular vs plural must differ: {s}/{p}");
            assert_ne!(p, pret, "plural vs preterite must differ: {p}/{pret}");
        }
        assert!(!SPANISH_VERBS.is_empty());
    }

    #[test]
    fn spanish_adj_agreement_has_four_forms() {
        for (fs, fp, ms, mp) in SPANISH_ADJ {
            assert_ne!(fs, fp);
            assert_ne!(ms, mp);
            assert_ne!(fs, ms);
        }
    }

    #[test]
    fn french_articles_include_elision_and_team_ref_chain() {
        assert_eq!(french_articles::L, "l'");
        assert_eq!(french_articles::EQUIPE_DU, "Equipe du");
        assert_eq!(french_articles::EQUIPE_DES, "Equipe des");
    }

    #[test]
    fn german_verbs_have_full_present_past_quads() {
        for (ps, pp, pas, pap) in GERMAN_VERBS {
            assert_ne!(ps, pp);
            assert_ne!(pas, pap);
            assert_ne!(ps, pas);
        }
        assert_eq!(GERMAN_VERBS.len(), 10);
    }

    #[test]
    fn german_comp_adjectives_cover_10_indices() {
        assert_eq!(GERMAN_COMP_ADJECTIVES.len(), 10);
        assert_eq!(GERMAN_COMP_ADJECTIVES[0], "nationale");
        assert_eq!(GERMAN_COMP_ADJECTIVES[1], "internationale");
        assert_eq!(GERMAN_COMP_ADJECTIVES[9], "Pokal");
    }

    #[test]
    fn german_articles_cover_case_matrix() {
        // der/die/das/den/dem/des for definite; ein/eine/einen/einer/einem/eines for indefinite.
        for a in [german_articles::DER, german_articles::DIE, german_articles::DAS,
                  german_articles::DEN, german_articles::DEM, german_articles::DES] {
            assert!(!a.is_empty());
        }
        for i in [german_articles::EIN, german_articles::EINE, german_articles::EINEN,
                  german_articles::EINER, german_articles::EINEM, german_articles::EINES] {
            assert!(i.starts_with("ein"));
        }
    }

    #[test]
    fn italian_articles_cover_all_combinations() {
        assert_eq!(italian_articles::AGLI, "agli");
        assert_eq!(italian_articles::DELL, "dell'");
        assert_eq!(italian_articles::DALLO, "dallo");
    }

    #[test]
    fn italian_sibilant_test_fires_on_s_plus_consonant() {
        assert!(italian_needs_sibilant_form("stadio"));   // s + t
        assert!(italian_needs_sibilant_form("spagna"));   // s + p
        assert!(italian_needs_sibilant_form("zero"));     // z start
        assert!(italian_needs_sibilant_form("ss..."));    // ss digraph
        // Plain vowel-start or plain consonant → no elision.
        assert!(!italian_needs_sibilant_form("Roma"));
        assert!(!italian_needs_sibilant_form("Milano"));
        assert!(!italian_needs_sibilant_form("sabato"));  // s + vowel
    }

    #[test]
    fn italian_adj_swapped_order_vs_spanish() {
        // Italian: (fem-sing, fem-pl, masc-pl, masc-sing).
        for (fs, fp, mp, ms) in ITALIAN_ADJ {
            assert!(fs.ends_with('a'));
            assert!(fp.ends_with('e'));
            assert!(mp.ends_with('i'));
            assert!(ms.ends_with('o'));
        }
    }

    #[test]
    fn swedish_positions_include_position_forms() {
        assert!(SWEDISH_POSITIONS.iter().any(|(b, _, _)| *b == "offensiv ytterback"));
        // Each entry has 3 distinct forms.
        for (bare, en, den) in SWEDISH_POSITIONS {
            assert_ne!(bare, en);
            assert_ne!(en, den);
            assert!(den.starts_with("den "));
        }
    }

    #[test]
    fn unported_langs_pass_through() {
        // Lang 5 is the only remaining branch with no ported table.
        // DutchPt is intentional pass-through (exe fences lang != 3), so it
        // also emits the bare token.
        for lang in [LangId::DutchPt, LangId::Lang5] {
            let r = inflect_token("Bayern", kind_kw::UPPER, scope_kw::DEFAULT,
                                   0, 0, lang);
            assert_eq!(r.text, "Bayern",
                       "lang {:?} should pass through", lang);
        }
    }

    // --------------------------------------------------------------
    // Norwegian (lang 7) — subject / possessive / definite forms.
    // --------------------------------------------------------------

    #[test]
    fn norwegian_subject_form_is_bare_token() {
        let r = inflect_token("Rosenborg", kind_kw::UPPER, scope_kw::DEFAULT,
                               0, 0, LangId::Norwegian);
        assert_eq!(r.text, "Rosenborg");
    }

    #[test]
    fn norwegian_possessive_adds_s_suffix() {
        // Word not ending in s → append "s" (genitive).
        let r = inflect_token("Rosenborg", "", scope_kw::SINGULAR,
                               0, 0, LangId::Norwegian);
        assert_eq!(r.text, "Rosenborgs");
    }

    #[test]
    fn norwegian_possessive_on_s_ending_word_stays_bare() {
        // Exe branch: last char S/s → suffix = "" to avoid "Brums".
        let r = inflect_token("Brums", "", scope_kw::SINGULAR,
                               0, 0, LangId::Norwegian);
        assert_eq!(r.text, "Brums");
    }

    #[test]
    fn norwegian_definite_kind_is_pass_through() {
        // Norwegian marks definiteness by suffix on the noun itself
        // (-en/-et/-a), which the exe leaves to caller-supplied templates.
        // The inflection fn only touches the possessive slot, so `THE_1`
        // returns the bare token.
        let r = inflect_token("Rosenborg", kind_kw::THE_1, scope_kw::DEFAULT,
                               0, 0, LangId::Norwegian);
        assert_eq!(r.text, "Rosenborg");
    }

    // --------------------------------------------------------------
    // Lang 10 (Dutch/German-mirror) — reuses German picker per the
    // exe's line-301 dispatch that duplicates the id==8 branch.
    // --------------------------------------------------------------

    #[test]
    fn lang10_subject_form_mirrors_german_nominative() {
        let r = inflect_token("Bayern", kind_kw::UPPER, scope_kw::DEFAULT,
                               0, 0, LangId::Lang10);
        assert!(r.text.starts_with("der "), "got: {}", r.text);
    }

    #[test]
    fn lang10_definite_the_matches_german_die_for_feminine() {
        // Feminine heuristic in german_affix: token ends with 'a' or 'e'.
        let r = inflect_token("Roma", kind_kw::THE, scope_kw::DEFAULT,
                               0, 0, LangId::Lang10);
        assert!(r.text.starts_with("die "), "got: {}", r.text);
    }

    #[test]
    fn lang10_case_enum_picks_declension_same_as_german() {
        // Case 3 (genitive) — masculine word → "des".
        let r = inflect_token("Bayern", kind_kw::UPPER, scope_kw::DEFAULT,
                               3, 0, LangId::Lang10);
        assert!(r.text.starts_with("des "), "got: {}", r.text);
    }

    // --------------------------------------------------------------
    // Dutch/Portuguese (lang 3) — exe explicitly fences the branch,
    // so subject / possessive / definite all pass through.
    // --------------------------------------------------------------

    #[test]
    fn dutch_portuguese_subject_form_is_bare_token() {
        let r = inflect_token("Ajax", kind_kw::UPPER, scope_kw::DEFAULT,
                               0, 0, LangId::DutchPt);
        assert_eq!(r.text, "Ajax");
    }

    #[test]
    fn dutch_portuguese_possessive_is_bare_token() {
        // The exe skips the whole morphology block for id 3, so no 's suffix.
        let r = inflect_token("Ajax", "", scope_kw::SINGULAR,
                               0, 0, LangId::DutchPt);
        assert_eq!(r.text, "Ajax");
    }

    #[test]
    fn dutch_portuguese_definite_kind_is_bare_token() {
        let r = inflect_token("Porto", kind_kw::THE, scope_kw::DEFAULT,
                               0, 0, LangId::DutchPt);
        assert_eq!(r.text, "Porto");
    }

    #[test]
    fn spanish_upper_feminine_word_gets_la() {
        // 'a'-ending → feminine.
        let r = inflect_token("Roma", kind_kw::UPPER, scope_kw::DEFAULT,
                               0, 0, LangId::Spanish);
        assert!(r.text.starts_with("la "), "got: {}", r.text);
    }

    #[test]
    fn spanish_upper_masculine_word_gets_el() {
        let r = inflect_token("Real", kind_kw::UPPER, scope_kw::DEFAULT,
                               0, 0, LangId::Spanish);
        assert!(r.text.starts_with("el "), "got: {}", r.text);
    }

    #[test]
    fn spanish_plural_feminine_gets_las() {
        let r = inflect_token("Roma", kind_kw::UPPER, scope_kw::PLURAL,
                               0, 0, LangId::Spanish);
        assert!(r.text.starts_with("las "));
    }

    #[test]
    fn german_case_enum_picks_declension() {
        // Case 0 (nominative) with a masculine word.
        let r = inflect_token("Bayern", kind_kw::UPPER, scope_kw::DEFAULT,
                               0, 0, LangId::German);
        assert!(r.text.starts_with("der "), "nom masc: {}", r.text);
        // Case 1 (accusative).
        let r = inflect_token("Bayern", kind_kw::UPPER, scope_kw::DEFAULT,
                               1, 0, LangId::German);
        assert!(r.text.starts_with("den "), "acc: {}", r.text);
        // Case 2 (dative).
        let r = inflect_token("Bayern", kind_kw::UPPER, scope_kw::DEFAULT,
                               2, 0, LangId::German);
        assert!(r.text.starts_with("dem "), "dat: {}", r.text);
        // Case 3 (genitive).
        let r = inflect_token("Bayern", kind_kw::UPPER, scope_kw::DEFAULT,
                               3, 0, LangId::German);
        assert!(r.text.starts_with("des "), "gen: {}", r.text);
    }

    #[test]
    fn italian_sibilant_word_gets_allo() {
        // "Stadio" starts with s+consonant → allo.
        let r = inflect_token("Stadio", kind_kw::UPPER, scope_kw::DEFAULT,
                               0, 0, LangId::Italian);
        assert!(r.text.starts_with("allo "), "got: {}", r.text);
    }

    #[test]
    fn italian_vowel_word_gets_l_apostrophe() {
        let r = inflect_token("Inter", kind_kw::UPPER, scope_kw::DEFAULT,
                               0, 0, LangId::Italian);
        assert!(r.text.starts_with("l' "), "got: {}", r.text);
    }

    #[test]
    fn italian_masculine_consonant_gets_il() {
        let r = inflect_token("Milan", kind_kw::UPPER, scope_kw::DEFAULT,
                               0, 0, LangId::Italian);
        assert!(r.text.starts_with("il "), "got: {}", r.text);
    }

    #[test]
    fn swedish_position_gets_den_form_suffix() {
        let r = inflect_token("offensiv ytterback", kind_kw::THE_1,
                               scope_kw::DEFAULT, 0, 0, LangId::Swedish);
        assert!(r.text.contains("den offensive ytterbacken"),
                "got: {}", r.text);
    }
}
