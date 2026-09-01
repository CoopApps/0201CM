// AUTO-GENERATED from FUN_006e5da0.c by cm-lift codegen
// Pattern: field_setter at +0x8eaa

#[inline]
pub fn set_field_at_0x8eaa(record: &mut [u8], val: u32) {
    let b = val.to_le_bytes();
    record[0x8eaa..0x8eaa+4].copy_from_slice(&b);
}
