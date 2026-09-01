// AUTO-GENERATED from FUN_006da000.c by cm-lift codegen
// Pattern: field_getter at +0x108

#[inline]
pub fn get_field_at_0x108(record: &[u8]) -> u32 {
    u32::from_le_bytes([
        record[0x108], record[0x108+1],
        record[0x108+2], record[0x108+3],
    ])
}
