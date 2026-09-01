// AUTO-GENERATED from FUN_008b3260.c by cm-lift codegen
// Pattern: field_getter at +0xc

#[inline]
pub fn get_field_at_0xc(record: &[u8]) -> u32 {
    u32::from_le_bytes([
        record[0xc], record[0xc+1],
        record[0xc+2], record[0xc+3],
    ])
}
