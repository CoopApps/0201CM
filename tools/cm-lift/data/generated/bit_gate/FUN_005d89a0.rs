// AUTO-GENERATED from FUN_005d89a0.c by cm-lift codegen
// Pattern: bit_gate — first-match-wins over 5 mask bits
//
// Semantic labels for each branch aren't recoverable from the
// decompile alone; the human port should replace `Variant<N>`
// with the enum this decoder discriminates.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant_005d89a0 {
    B0,
    B1,
    B2,
    B3,
    B4,
    None,
}

pub fn decode(mask: u32) -> Variant_005d89a0 {
    if mask & 0x1 != 0 { return Variant_005d89a0::B0; }
    if mask & 0x2 != 0 { return Variant_005d89a0::B1; }
    if mask & 0x4 != 0 { return Variant_005d89a0::B2; }
    if mask & 0x2 != 0 { return Variant_005d89a0::B3; }
    if mask & 0x4 != 0 { return Variant_005d89a0::B4; }
    Variant_005d89a0::None
}
