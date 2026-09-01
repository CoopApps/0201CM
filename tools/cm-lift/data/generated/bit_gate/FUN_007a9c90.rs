// AUTO-GENERATED from FUN_007a9c90.c by cm-lift codegen
// Pattern: bit_gate — first-match-wins over 4 mask bits
//
// Semantic labels for each branch aren't recoverable from the
// decompile alone; the human port should replace `Variant<N>`
// with the enum this decoder discriminates.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant_007a9c90 {
    B0,
    B1,
    B2,
    B3,
    None,
}

pub fn decode(mask: u32) -> Variant_007a9c90 {
    if mask & 0xF != 0 { return Variant_007a9c90::B0; }
    if mask & 0xF0 != 0 { return Variant_007a9c90::B1; }
    if mask & 0xF != 0 { return Variant_007a9c90::B2; }
    if mask & 0xF0 != 0 { return Variant_007a9c90::B3; }
    Variant_007a9c90::None
}
