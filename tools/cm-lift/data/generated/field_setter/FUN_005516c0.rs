// AUTO-GENERATED from FUN_005516c0.c by cm-lift codegen
// Pattern: field_setter at +0x10

#[inline]
pub fn set_field_at_0x10(record: &mut [u8], val: u32) {
    let b = val.to_le_bytes();
    record[0x10..0x10+4].copy_from_slice(&b);
}
