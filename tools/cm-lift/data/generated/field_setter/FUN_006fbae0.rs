// AUTO-GENERATED from FUN_006fbae0.c by cm-lift codegen
// Pattern: field_setter at +0x2b

#[inline]
pub fn set_field_at_0x2b(record: &mut [u8], val: u32) {
    let b = val.to_le_bytes();
    record[0x2b..0x2b+4].copy_from_slice(&b);
}
