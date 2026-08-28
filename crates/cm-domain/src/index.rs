//! Localised region names — port of `index.cpp`
//! (`C:\dev\CM3 00-01\cm3\code\index.cpp`).
//!
//! The TU has 3 functions:
//!
//! * **`FUN_005eb410`** — 40995 bytes. Ghidra refused to decompile it (too big);
//!   the disasm walks a giant switch that dispatches text lookups over shipped
//!   `.idx` files (multi-language string catalogues). Signature and payload
//!   need per-language dumps to be portable; ledgered as decoded-shape-only.
//! * **`FUN_005f7140`** — 129 bytes. Error reporter: emits
//!   `"Unable to find the %s index"` and pops the standard error dialog.
//! * **`FUN_00600680`** — 2019 bytes. Ported HERE: given `(category 1..14,
//!   out_buf, min_id >= 100, nationality_or_0xff)` it returns the localised
//!   REGION name — "African"/"Africa"/"Britannico e Irlandese"/... etc.
//!
//! # FUN_00600680 mechanics (verified from the decompile switch)
//!
//! `param_1` = 1..14 region category
//! `param_4` = nationality (u8, `0xff` = "unknown") — chooses adjective vs
//! noun form for languages that inflect it
//! `param_3` = a min_id (>= 100 = 0x64) — an early input guard
//! `FUN_006547b0()` = current UI language id. When language ∈ {4=Italian,
//! 5=Portuguese, 9=?}, a hand-crafted table of localised strings is used;
//! otherwise the English string table is picked with adjective/noun based
//! on nationality being non-`0xff`.
//!
//! Regions (verified from the switch case labels):
//!   1  African / Africa
//!   2  Asian / Asia
//!   3  Caribbean
//!   4  Central American / Central America
//!   5  Central European / Central Europe
//!   6  Eastern European / Eastern Europe
//!   7  Middle Eastern / Middle East
//!   8  North African / North Africa
//!   9  North American / North America
//!   10 Oceanic / Oceania
//!   11 Scandinavian / Scandinavia
//!   12 South American / South America
//!   13 Southern European / Southern Europe
//!   14 UK / Irish  vs  UK & Ireland
//!
//! This module ships the English variant (adjective + noun forms). The
//! localised variants live in the shipped `.idx` files and remain deferred.

/// The 14 region categories in [`region_name`].
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionCategory {
    African            = 1,
    Asian              = 2,
    Caribbean          = 3,
    CentralAmerican    = 4,
    CentralEuropean    = 5,
    EasternEuropean    = 6,
    MiddleEastern      = 7,
    NorthAfrican       = 8,
    NorthAmerican      = 9,
    Oceanic            = 10,
    Scandinavian       = 11,
    SouthAmerican      = 12,
    SouthernEuropean   = 13,
    UkIrish            = 14,
}

impl RegionCategory {
    pub fn from_u8(v: u8) -> Option<Self> {
        Some(match v {
            1 => Self::African, 2 => Self::Asian, 3 => Self::Caribbean,
            4 => Self::CentralAmerican, 5 => Self::CentralEuropean,
            6 => Self::EasternEuropean, 7 => Self::MiddleEastern,
            8 => Self::NorthAfrican, 9 => Self::NorthAmerican,
            10 => Self::Oceanic, 11 => Self::Scandinavian,
            12 => Self::SouthAmerican, 13 => Self::SouthernEuropean,
            14 => Self::UkIrish,
            _ => return None,
        })
    }
}

/// Sentinel value the exe uses for "unknown nationality" — triggers noun
/// forms in the non-English branches, adjective forms in English.
pub const NATIONALITY_UNKNOWN: u8 = 0xff;

/// English localisation of a region name.
///
/// * `nationality == None`  — the exe passes `0xff`, returns noun forms
///   ("Africa", "North America", "UK & Ireland"). English is a special case:
///   the exe still uses adjective forms for most regions here, matching its
///   switch layout in the English (default) branch.
/// * `nationality == Some(_)` — adjective form ("African", "North American").
///
/// Ports the English (default-language) branch of FUN_00600680.
pub fn region_name(cat: RegionCategory, nationality: Option<u8>) -> &'static str {
    let is_adj = nationality.map_or(false, |n| n != NATIONALITY_UNKNOWN);
    match cat {
        RegionCategory::African         => if is_adj { "African" }         else { "Africa" },
        RegionCategory::Asian           => if is_adj { "Asian" }           else { "Asia" },
        RegionCategory::Caribbean       => "Caribbean",
        RegionCategory::CentralAmerican => if is_adj { "Central American" } else { "Central America" },
        RegionCategory::CentralEuropean => if is_adj { "Central European" } else { "Central Europe" },
        RegionCategory::EasternEuropean => if is_adj { "Eastern European" } else { "Eastern Europe" },
        RegionCategory::MiddleEastern   => if is_adj { "Middle Eastern" }   else { "Middle East" },
        RegionCategory::NorthAfrican    => if is_adj { "North African" }    else { "North Africa" },
        RegionCategory::NorthAmerican   => if is_adj { "North American" }   else { "North America" },
        RegionCategory::Oceanic         => if is_adj { "Oceanic" }          else { "Oceania" },
        RegionCategory::Scandinavian    => if is_adj { "Scandinavian" }     else { "Scandinavia" },
        RegionCategory::SouthAmerican   => if is_adj { "South American" }   else { "South America" },
        RegionCategory::SouthernEuropean => if is_adj { "Southern European" } else { "Southern Europe" },
        RegionCategory::UkIrish         => if is_adj { "UK / Irish" }       else { "UK & Ireland" },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adjective_vs_noun() {
        assert_eq!(region_name(RegionCategory::African, Some(1)), "African");
        assert_eq!(region_name(RegionCategory::African, None), "Africa");
        assert_eq!(region_name(RegionCategory::NorthAmerican, Some(0)), "North American");
        assert_eq!(region_name(RegionCategory::NorthAmerican, None), "North America");
        assert_eq!(region_name(RegionCategory::UkIrish, Some(0)), "UK / Irish");
        assert_eq!(region_name(RegionCategory::UkIrish, None), "UK & Ireland");
    }

    #[test]
    fn caribbean_has_only_one_form() {
        assert_eq!(region_name(RegionCategory::Caribbean, Some(0)), "Caribbean");
        assert_eq!(region_name(RegionCategory::Caribbean, None), "Caribbean");
    }

    #[test]
    fn from_u8_round_trip() {
        for v in 1u8..=14 {
            let c = RegionCategory::from_u8(v).unwrap();
            assert_eq!(c as u8, v);
        }
        assert!(RegionCategory::from_u8(0).is_none());
        assert!(RegionCategory::from_u8(15).is_none());
    }
}
