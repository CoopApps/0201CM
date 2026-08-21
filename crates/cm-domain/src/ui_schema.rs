use serde::{Deserialize, Serialize};

pub const SCREEN_WIDTH: u16 = 800;
pub const SCREEN_HEIGHT: u16 = 600;
pub const MAX_X: u16 = 799;
pub const MAX_Y: u16 = 599;

pub const DRAW_QUEUE_COUNT_OFFSET: u32 = 0x12e9a0;
pub const DRAW_RECORD_BASE_OFFSET: u32 = 0x0ba95e;
pub const DRAW_RECORD_STRIDE: u32 = 0x018c;
pub const DRAW_RECORD_MAX_INDEX: u16 = 0x04ae;
pub const DRAW_RECORD_CAPACITY: u16 = DRAW_RECORD_MAX_INDEX + 1;
pub const DRAW_SECONDARY_QUEUE_OFFSET: u32 = 0x12eb96;
pub const DRAW_SECONDARY_QUEUE_COUNT_OFFSET: u32 = 0x12f4f8;

pub const REGION_COUNT_OFFSET: u32 = 0x12e99e;
pub const REGION_RECORD_BASE_OFFSET: u32 = 0x000004;
pub const REGION_RECORD_STRIDE: u32 = 0x0bf1;
pub const REGION_RECORD_MAX_INDEX: u16 = 0x00f8;
pub const REGION_RECORD_CAPACITY: u16 = REGION_RECORD_MAX_INDEX + 1;
pub const CUSTOM_FONT_SLOT_STRIDE: u32 = 0x1404;
pub const CUSTOM_FONT_FIRST_GLYPH: u16 = 0x20;
pub const CUSTOM_FONT_LAST_GLYPH: u16 = 0xff;
pub const CUSTOM_FONT_GLYPH_COUNT: u16 = CUSTOM_FONT_LAST_GLYPH - CUSTOM_FONT_FIRST_GLYPH + 1;
pub const CUSTOM_FONT_GLYPH_STRIDE: u32 = 0x14;
pub const SYSTEM_FONT_FIXED_POINT_ROUND: i32 = 0x8000;
pub const SYSTEM_FONT_FIXED_POINT_SHIFT: u8 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cm0102UiEvidenceTier {
    CodeDerived,
    StaticLift,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cm0102UiField {
    pub offset: u32,
    pub size: u16,
    pub name: String,
    pub semantics: String,
    pub evidence: Cm0102UiEvidenceTier,
    pub source_function: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cm0102UiLayoutSchema {
    pub screen_width: u16,
    pub screen_height: u16,
    pub max_x: u16,
    pub max_y: u16,
    pub draw_record_stride: u32,
    pub draw_record_capacity: u16,
    pub region_record_stride: u32,
    pub region_record_capacity: u16,
    pub draw_record_fields: Vec<Cm0102UiField>,
    pub region_record_fields: Vec<Cm0102UiField>,
    pub font_slots: Vec<Cm0102FontSlot>,
    pub font_metric_facts: Vec<Cm0102FontMetricFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cm0102FontSlot {
    pub slot: i16,
    pub file: Option<String>,
    pub metric_path: Cm0102FontMetricPath,
    pub fallback_height: Option<u16>,
    pub fallback_width_size: Option<u16>,
    pub evidence: Cm0102UiEvidenceTier,
    pub source_function: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Cm0102FontMetricPath {
    CustomFntTable,
    SystemOrT2kFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cm0102FontMetricFact {
    pub name: String,
    pub semantics: String,
    pub evidence: Cm0102UiEvidenceTier,
    pub source_function: String,
}

pub fn clamp_original_x(value: i32) -> u16 {
    value.clamp(0, i32::from(MAX_X)) as u16
}

pub fn clamp_original_y(value: i32) -> u16 {
    value.clamp(0, i32::from(MAX_Y)) as u16
}

pub fn cm0102_ui_layout_schema() -> Cm0102UiLayoutSchema {
    Cm0102UiLayoutSchema {
        screen_width: SCREEN_WIDTH,
        screen_height: SCREEN_HEIGHT,
        max_x: MAX_X,
        max_y: MAX_Y,
        draw_record_stride: DRAW_RECORD_STRIDE,
        draw_record_capacity: DRAW_RECORD_CAPACITY,
        region_record_stride: REGION_RECORD_STRIDE,
        region_record_capacity: REGION_RECORD_CAPACITY,
        draw_record_fields: draw_record_fields(),
        region_record_fields: region_record_fields(),
        font_slots: font_slots(),
        font_metric_facts: font_metric_facts(),
    }
}

pub fn custom_font_width_from_advances<I>(font_height: i32, chars: I) -> i32
where
    I: IntoIterator<Item = Cm0102CustomGlyphMetric>,
{
    let mut width = 0;
    let mut previous_right_bearing = 0;
    for (index, glyph) in chars.into_iter().enumerate() {
        let kerning = if index == 0 {
            0
        } else {
            custom_font_pair_spacing(font_height, previous_right_bearing, glyph.left_bearing)
        };
        width += glyph.advance + kerning;
        previous_right_bearing = glyph.right_bearing;
    }
    width
}

pub fn custom_font_pair_spacing(
    font_height: i32,
    previous_right_bearing: i32,
    next_left_bearing: i32,
) -> i32 {
    let spacing = (((font_height * 3) + (((font_height * 3) >> 31) & 3)) >> 2)
        - next_left_bearing
        - previous_right_bearing;
    spacing.max(0)
}

pub fn system_font_width_from_fixed_point_advances<I>(advances: I) -> u32
where
    I: IntoIterator<Item = i32>,
{
    let total: i32 = advances.into_iter().sum();
    ((total + SYSTEM_FONT_FIXED_POINT_ROUND) >> SYSTEM_FONT_FIXED_POINT_SHIFT) as u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cm0102CustomGlyphMetric {
    pub advance: i32,
    pub left_bearing: i32,
    pub right_bearing: i32,
}

fn draw_record_fields() -> Vec<Cm0102UiField> {
    vec![
        field(
            0x00,
            4,
            "display_context_base",
            "param_1 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x04,
            4,
            "draw_record_base",
            "param_2 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x08,
            4,
            "parent_or_payload_a",
            "param_19 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x0c,
            4,
            "draw_flags",
            "param_4 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x10,
            4,
            "x1",
            "left coordinate, clamped to >= 0",
            "0x005d7bd0",
        ),
        field(
            0x14,
            4,
            "y1",
            "top coordinate, clamped to >= 0",
            "0x005d7bd0",
        ),
        field(
            0x18,
            4,
            "x2",
            "right coordinate, clamped to <= 799",
            "0x005d7bd0",
        ),
        field(
            0x1c,
            4,
            "y2",
            "bottom coordinate, clamped to <= 599",
            "0x005d7bd0",
        ),
        field(
            0x20,
            4,
            "style_or_type",
            "param_9 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x24,
            4,
            "row_or_toggle",
            "derived row/index byte when attached to region",
            "0x005d7bd0",
        ),
        field(0x28, 4, "x1_copy", "left coordinate copy", "0x005d7bd0"),
        field(0x2c, 4, "x2_copy", "right coordinate copy", "0x005d7bd0"),
        field(0x30, 4, "y1_copy", "top coordinate copy", "0x005d7bd0"),
        field(0x34, 4, "y2_copy", "bottom coordinate copy", "0x005d7bd0"),
        field(
            0x38,
            4,
            "render_flags",
            "param_11 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x3c,
            4,
            "colour_or_group",
            "param_14 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x40,
            4,
            "payload_pointer",
            "param_18 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x44,
            4,
            "text_length_or_minus_one",
            "computed when flags include 0x8 and 0x10",
            "0x005d7bd0",
        ),
        field(
            0x48,
            4,
            "payload_b",
            "param_20 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x50,
            4,
            "frame_counter",
            "DAT_00b4d580 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x54,
            4,
            "external_text_pointer",
            "param_17 retained when flag 0x8 is set",
            "0x005d7bd0",
        ),
        field(
            0x70,
            2,
            "draw_index",
            "param_3 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x72,
            2,
            "short_param_12",
            "param_12 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x74,
            2,
            "short_param_13",
            "param_13 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x76,
            2,
            "font_or_colour_short",
            "param_15 copied as short",
            "0x005d7bd0",
        ),
        field(
            0x78,
            2,
            "short_param_16",
            "param_16 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x7a,
            2,
            "owning_region_index",
            "param_21 copied into record",
            "0x005d7bd0",
        ),
        field(
            0x7c,
            2,
            "sentinel_short",
            "initialized to 0xffff",
            "0x005d7bd0",
        ),
        field(
            0x7e,
            2,
            "sentinel_short_2",
            "initialized to 0xffff",
            "0x005d7bd0",
        ),
        field(
            0x80,
            256,
            "text_buffer",
            "copied text or formatted region-attached text",
            "0x005d7bd0",
        ),
        field(
            0x180,
            4,
            "dirty_or_alternate_row",
            "set when attached region row alternates",
            "0x005d7bd0",
        ),
        field(
            0x184,
            4,
            "zeroed_state_61",
            "initialized to zero",
            "0x005d7bd0",
        ),
        field(
            0x188,
            4,
            "zeroed_state_62",
            "initialized to zero",
            "0x005d7bd0",
        ),
    ]
}

fn region_record_fields() -> Vec<Cm0102UiField> {
    vec![
        field(
            0x00,
            4,
            "x1",
            "left coordinate, clamped to >= 0",
            "0x00402b00",
        ),
        field(
            0x04,
            4,
            "y1",
            "top coordinate, clamped to >= 0",
            "0x00402b00",
        ),
        field(
            0x08,
            4,
            "x2",
            "right coordinate, clamped to <= 799",
            "0x00402b00",
        ),
        field(
            0x0c,
            4,
            "y2",
            "bottom coordinate, clamped to <= 599",
            "0x00402b00",
        ),
        field(
            0x10,
            4,
            "min_or_large_default",
            "initialized to 0x7fffffff",
            "0x00402b00",
        ),
        field(
            0x14,
            4,
            "zeroed_region_state",
            "initialized to zero",
            "0x00402b00",
        ),
        field(
            0x18,
            4,
            "region_flags",
            "param_12 copied into record",
            "0x00402b00",
        ),
        field(
            0x1c,
            4,
            "region_unknown_index",
            "initialized to 0xffffffff",
            "0x00402b00",
        ),
        field(
            0x200,
            4,
            "zeroed_state_80",
            "initialized to zero",
            "0x00402b00",
        ),
        field(
            0x204,
            4,
            "display_context_base",
            "param_1 copied into region",
            "0x00402b00",
        ),
        field(
            0x208,
            4,
            "draw_record_base",
            "param_2 copied into region",
            "0x00402b00",
        ),
        field(
            0x20c,
            2,
            "region_index",
            "param_3 copied into region",
            "0x00402b00",
        ),
        field(
            0x20e,
            2,
            "parent_region_index",
            "param_14 copied into region",
            "0x00402b00",
        ),
        field(
            0x20f,
            1,
            "parent_region_short_high_byte_overlap",
            "overlaps parent region short",
            "0x00402b00",
        ),
        field(
            0x20c + 8,
            2,
            "short_param_13",
            "param_13 copied into region",
            "0x00402b00",
        ),
        field(
            0xb72,
            2,
            "region_index_copy",
            "param_3 copied into region",
            "0x00402b00",
        ),
        field(
            0xb76,
            2,
            "sentinel_short",
            "initialized to 0xffff",
            "0x00402b00",
        ),
        field(
            0xb7a,
            49,
            "scratch_zero_bytes",
            "zero-filled scratch area",
            "0x00402b00",
        ),
        field(
            0xba6,
            4,
            "zeroed_state_ba6",
            "initialized to zero",
            "0x00402b00",
        ),
        field(
            0xbab,
            1,
            "primary_row_count",
            "param_8 copied into region",
            "0x00402b00",
        ),
        field(
            0xbac,
            1,
            "secondary_row_count",
            "param_10 copied into region",
            "0x00402b00",
        ),
        field(
            0xbad,
            30,
            "primary_row_bytes",
            "copied from param_9 or filled with 1",
            "0x00402b00",
        ),
        field(
            0xbcb,
            30,
            "secondary_row_bytes",
            "copied from param_11 or filled with 1",
            "0x00402b00",
        ),
        field(0xbe9, 4, "enabled_state", "initialized to 1", "0x00402b00"),
        field(
            0xbed,
            4,
            "zeroed_state_bed",
            "initialized to zero",
            "0x00402b00",
        ),
    ]
}

fn font_slots() -> Vec<Cm0102FontSlot> {
    vec![
        custom_font_slot(1, "arial_narrow_10.fnt"),
        custom_font_slot(2, "arial_narrow_11.fnt"),
        custom_font_slot(3, "arial_14.fnt"),
        custom_font_slot(4, "arial_16.fnt"),
        custom_font_slot(5, "arial_18.fnt"),
        custom_font_slot(6, "trade_cond_24_bold.fnt"),
        custom_font_slot(7, "trade_cond_28_bold.fnt"),
        fallback_font_slot(0, Some("small.t2k"), 0x0f, 9),
        fallback_font_slot(1, Some("medium.t2k"), 0x0f, 9),
        fallback_font_slot(2, Some("large.t2k"), 0x12, 10),
        fallback_font_slot(3, Some("symbol.ttf"), 0x15, 0x0e),
        fallback_font_slot(4, None, 0x18, 0x10),
        fallback_font_slot(5, None, 0x1b, 0x12),
        fallback_font_slot(6, None, 0x27, 0x14),
        fallback_font_slot(7, None, 0x2d, 0x16),
    ]
}

fn font_metric_facts() -> Vec<Cm0102FontMetricFact> {
    vec![
        metric_fact(
            "custom_font_height",
            "FUN_005cf7b0 returns *(DAT_00accb9c + slot * 0x1404) for custom .fnt slots.",
            "0x005cf7b0",
        ),
        metric_fact(
            "custom_font_width",
            "FUN_005cf610 sums glyph advance at DAT_00accba0 + char * 0x14 + slot * 0x1404, treating '|' as space.",
            "0x005cf610",
        ),
        metric_fact(
            "custom_pair_spacing",
            "FUN_005cf840 adds max(0, floor((font_height * 3) / 4) - next_left_bearing - previous_right_bearing) between adjacent printable glyphs.",
            "0x005cf840",
        ),
        metric_fact(
            "system_font_height",
            "When DAT_009b88f8 != 0, FUN_005cf7b0 returns hardcoded heights for slots 0..7.",
            "0x005cf7b0",
        ),
        metric_fact(
            "system_font_width",
            "When DAT_009b88f8 != 0, FUN_005cf610 delegates to FUN_0059bed0 with point sizes 9, 10, 14, 16, 18, 20, 22.",
            "0x005cf610",
        ),
        metric_fact(
            "system_width_rounding",
            "FUN_0059bed0 accumulates 16.16 glyph advances and returns (sum + 0x8000) >> 16.",
            "0x0059bed0",
        ),
    ]
}

fn field(
    offset: u32,
    size: u16,
    name: &'static str,
    semantics: &'static str,
    source_function: &'static str,
) -> Cm0102UiField {
    Cm0102UiField {
        offset,
        size,
        name: name.to_string(),
        semantics: semantics.to_string(),
        evidence: Cm0102UiEvidenceTier::CodeDerived,
        source_function: source_function.to_string(),
    }
}

fn custom_font_slot(slot: i16, file: &str) -> Cm0102FontSlot {
    Cm0102FontSlot {
        slot,
        file: Some(file.to_string()),
        metric_path: Cm0102FontMetricPath::CustomFntTable,
        fallback_height: None,
        fallback_width_size: None,
        evidence: Cm0102UiEvidenceTier::CodeDerived,
        source_function: "0x005ce750".to_string(),
    }
}

fn fallback_font_slot(
    slot: i16,
    file: Option<&str>,
    height: u16,
    width_size: u16,
) -> Cm0102FontSlot {
    Cm0102FontSlot {
        slot,
        file: file.map(str::to_string),
        metric_path: Cm0102FontMetricPath::SystemOrT2kFallback,
        fallback_height: Some(height),
        fallback_width_size: Some(width_size),
        evidence: Cm0102UiEvidenceTier::CodeDerived,
        source_function: "0x0059b1d0/0x005cf610/0x005cf7b0".to_string(),
    }
}

fn metric_fact(name: &str, semantics: &str, source_function: &str) -> Cm0102FontMetricFact {
    Cm0102FontMetricFact {
        name: name.to_string(),
        semantics: semantics.to_string(),
        evidence: Cm0102UiEvidenceTier::CodeDerived,
        source_function: source_function.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cm0102_ui_schema_preserves_lifted_caps_and_strides() {
        let schema = cm0102_ui_layout_schema();
        assert_eq!(schema.screen_width, 800);
        assert_eq!(schema.screen_height, 600);
        assert_eq!(schema.draw_record_stride, 0x18c);
        assert_eq!(schema.draw_record_capacity, 0x4af);
        assert_eq!(schema.region_record_stride, 0x0bf1);
        assert_eq!(schema.region_record_capacity, 0x00f9);
        assert_eq!(schema.font_slots.len(), 15);
    }

    #[test]
    fn cm0102_ui_schema_clamps_to_original_screen_bounds() {
        assert_eq!(clamp_original_x(-20), 0);
        assert_eq!(clamp_original_x(900), 799);
        assert_eq!(clamp_original_y(-20), 0);
        assert_eq!(clamp_original_y(900), 599);
    }

    #[test]
    fn cm0102_custom_font_pair_spacing_matches_lifted_formula() {
        assert_eq!(custom_font_pair_spacing(20, 3, 4), 8);
        assert_eq!(custom_font_pair_spacing(20, 20, 20), 0);
    }

    #[test]
    fn cm0102_system_font_width_rounds_16_16_advances() {
        assert_eq!(system_font_width_from_fixed_point_advances([0x10000]), 1);
        assert_eq!(
            system_font_width_from_fixed_point_advances([0x18000, 0x18000]),
            3
        );
    }
}
