// AUTO-GENERATED from FUN_006b1040.c by cm-lift codegen
// Pattern: field_getter at +0x8ed8

#[inline]
pub fn get_field_at_0x8ed8(record: &[u8]) -> u32 {
    u32::from_le_bytes([
        record[0x8ed8], record[0x8ed8+1],
        record[0x8ed8+2], record[0x8ed8+3],
    ])
}
