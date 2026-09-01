// AUTO-GENERATED from FUN_005516a0.c by cm-lift codegen
// Pattern: field_setter at +0x28

#[inline]
pub fn set_field_at_0x28(record: &mut [u8], val: u32) {
    let b = val.to_le_bytes();
    record[0x28..0x28+4].copy_from_slice(&b);
}
