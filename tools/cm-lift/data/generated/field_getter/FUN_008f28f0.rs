// AUTO-GENERATED from FUN_008f28f0.c by cm-lift codegen
// Pattern: field_getter at +0x438

#[inline]
pub fn get_field_at_0x438(record: &[u8]) -> u32 {
    u32::from_le_bytes([
        record[0x438], record[0x438+1],
        record[0x438+2], record[0x438+3],
    ])
}
