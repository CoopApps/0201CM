// AUTO-GENERATED from FUN_0093e588.c by cm-lift codegen
// Pattern: bit_gate — first-match-wins over 8 mask bits
//
// Semantic labels for each branch aren't recoverable from the
// decompile alone; the human port should replace `Variant<N>`
// with the enum this decoder discriminates.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant_0093e588 {
    B0,
    B1,
    B2,
    B3,
    B4,
    B5,
    B6,
    B7,
    None,
}

pub fn decode(mask: u32) -> Variant_0093e588 {
    if mask & 0x8 != 0 { return Variant_0093e588::B0; }
    if mask & 0x4 != 0 { return Variant_0093e588::B1; }
    if mask & 0x2 != 0 { return Variant_0093e588::B2; }
    if mask & 0x1 != 0 { return Variant_0093e588::B3; }
    if mask & 0x80000 != 0 { return Variant_0093e588::B4; }
    if mask & 0x30000 != 0 { return Variant_0093e588::B5; }
    if mask & 0x30000 != 0 { return Variant_0093e588::B6; }
    if mask & 0x40000 != 0 { return Variant_0093e588::B7; }
    Variant_0093e588::None
}
