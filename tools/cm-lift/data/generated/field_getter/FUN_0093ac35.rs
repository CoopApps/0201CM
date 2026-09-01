// AUTO-GENERATED from FUN_0093ac35.c by cm-lift codegen
// Pattern: field_getter at +0x4

#[inline]
pub fn get_field_at_0x4(record: &[u8]) -> u32 {
    u32::from_le_bytes([
        record[0x4], record[0x4+1],
        record[0x4+2], record[0x4+3],
    ])
}
