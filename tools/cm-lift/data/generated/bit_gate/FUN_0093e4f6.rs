// AUTO-GENERATED from FUN_0093e4f6.c by cm-lift codegen
// Pattern: bit_gate — first-match-wins over 9 mask bits
//
// Semantic labels for each branch aren't recoverable from the
// decompile alone; the human port should replace `Variant<N>`
// with the enum this decoder discriminates.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant_0093e4f6 {
    B0,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
    B8,
    None,
}

pub fn decode(mask: u32) -> Variant_0093e4f6 {
    if mask & 0x1 != 0 { return Variant_0093e4f6::B0; }
    if mask & 0x4 != 0 { return Variant_0093e4f6::B1; }
    if mask & 0x8 != 0 { return Variant_0093e4f6::B2; }
    if mask & 0x10 != 0 { return Variant_0093e4f6::B3; }
    if mask & 0x20 != 0 { return Variant_0093e4f6::B4; }
    if mask & 0x2 != 0 { return Variant_0093e4f6::B5; }
    if mask & 0x300 != 0 { return Variant_0093e4f6::B6; }
    if mask & 0x300 != 0 { return Variant_0093e4f6::B7; }
    if mask & 0x1000 != 0 { return Variant_0093e4f6::B8; }
    Variant_0093e4f6::None
}
