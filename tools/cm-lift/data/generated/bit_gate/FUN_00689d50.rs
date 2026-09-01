// AUTO-GENERATED from FUN_00689d50.c by cm-lift codegen
// Pattern: bit_gate — first-match-wins over 5 mask bits
//
// Semantic labels for each branch aren't recoverable from the
// decompile alone; the human port should replace `Variant<N>`
// with the enum this decoder discriminates.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant_00689d50 {
    B0,
    B1,
    B2,
    B3,
    B4,
    None,
}

pub fn decode(mask: u32) -> Variant_00689d50 {
    if mask & 0x400 != 0 { return Variant_00689d50::B0; }
    if mask & 0x2 != 0 { return Variant_00689d50::B1; }
    if mask & 0x85 != 0 { return Variant_00689d50::B2; }
    if mask & 0x30 != 0 { return Variant_00689d50::B3; }
    if mask & 0x308 != 0 { return Variant_00689d50::B4; }
    Variant_00689d50::None
}
