// AUTO-GENERATED from FUN_006e5d80.c by cm-lift codegen
// Pattern: field_setter at +0x8ea8

#[inline]
pub fn set_field_at_0x8ea8(record: &mut [u8], val: u32) {
    let b = val.to_le_bytes();
    record[0x8ea8..0x8ea8+4].copy_from_slice(&b);
}
