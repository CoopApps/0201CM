//! AUTHENTIC injury-type table, byte-exact from cm0102.exe .data
//! (names via FUN_00618610 switch; table at 0x009d3dd9, stride 13).
//! id -> name/region/weight/min_days/var_days/pct/severity.

#[derive(Debug, Clone, Copy)]
pub struct InjuryType {
    pub name: &'static str,
    pub region: u8,
    pub weight: u8,
    pub min_days: i16,
    pub var_days: i16,
    pub pct: u8,
    pub severity: i8,
}

pub const INJURY_TYPES: [InjuryType; 82] = [
    InjuryType { name: "cold", region: 0, weight: 6, min_days: 1, var_days: 3, pct: 50, severity: -1 }, // 0
    InjuryType { name: "flu", region: 0, weight: 7, min_days: 2, var_days: 5, pct: 80, severity: -1 }, // 1
    InjuryType { name: "virus", region: 0, weight: 8, min_days: 2, var_days: 5, pct: 80, severity: -1 }, // 2
    InjuryType { name: "food poisoning", region: 0, weight: 9, min_days: 2, var_days: 5, pct: 80, severity: -1 }, // 3
    InjuryType { name: "serious viral infection", region: 0, weight: 80, min_days: 170, var_days: 20, pct: 90, severity: 2 }, // 4
    InjuryType { name: "twisted ankle", region: 1, weight: 6, min_days: 15, var_days: 25, pct: 60, severity: 0 }, // 5
    InjuryType { name: "sprained ankle", region: 1, weight: 6, min_days: 10, var_days: 20, pct: 60, severity: 0 }, // 6
    InjuryType { name: "damaged foot", region: 1, weight: 6, min_days: 10, var_days: 15, pct: 60, severity: 0 }, // 7
    InjuryType { name: "stubbed toe", region: 1, weight: 6, min_days: 10, var_days: 10, pct: 40, severity: 0 }, // 8
    InjuryType { name: "damaged heel", region: 1, weight: 6, min_days: 5, var_days: 10, pct: 60, severity: 0 }, // 9
    InjuryType { name: "broken toe", region: 1, weight: 7, min_days: 10, var_days: 15, pct: 60, severity: 1 }, // 10
    InjuryType { name: "strained ankle ligaments", region: 1, weight: 7, min_days: 15, var_days: 30, pct: 60, severity: 1 }, // 11
    InjuryType { name: "damaged achilles tendon", region: 1, weight: 8, min_days: 20, var_days: 40, pct: 60, severity: 2 }, // 12
    InjuryType { name: "torn ankle ligaments", region: 1, weight: 8, min_days: 30, var_days: 150, pct: 60, severity: 2 }, // 13
    InjuryType { name: "broken ankle", region: 1, weight: 8, min_days: 60, var_days: 150, pct: 60, severity: 2 }, // 14
    InjuryType { name: "broken foot", region: 1, weight: 8, min_days: 60, var_days: 60, pct: 60, severity: 2 }, // 15
    InjuryType { name: "bruised shin", region: 2, weight: 6, min_days: 3, var_days: 11, pct: 20, severity: 0 }, // 16
    InjuryType { name: "calf strain", region: 2, weight: 6, min_days: 15, var_days: 25, pct: 60, severity: 0 }, // 17
    InjuryType { name: "torn calf muscle", region: 2, weight: 7, min_days: 30, var_days: 65, pct: 60, severity: 2 }, // 18
    InjuryType { name: "gashed leg", region: 2, weight: 7, min_days: 5, var_days: 16, pct: 60, severity: 1 }, // 19
    InjuryType { name: "shin splints", region: 2, weight: 8, min_days: 25, var_days: 100, pct: 60, severity: 2 }, // 20
    InjuryType { name: "broken leg", region: 2, weight: 8, min_days: 90, var_days: 170, pct: 60, severity: 2 }, // 21
    InjuryType { name: "twisted knee", region: 3, weight: 6, min_days: 10, var_days: 15, pct: 60, severity: 0 }, // 22
    InjuryType { name: "strained knee ligaments", region: 3, weight: 7, min_days: 10, var_days: 20, pct: 60, severity: 1 }, // 23
    InjuryType { name: "damaged knee cap", region: 3, weight: 8, min_days: 20, var_days: 40, pct: 60, severity: 2 }, // 24
    InjuryType { name: "damaged knee cartilage", region: 3, weight: 8, min_days: 10, var_days: 35, pct: 60, severity: 2 }, // 25
    InjuryType { name: "torn knee ligaments", region: 3, weight: 8, min_days: 50, var_days: 60, pct: 60, severity: 2 }, // 26
    InjuryType { name: "damaged cruciate ligaments", region: 3, weight: 9, min_days: 180, var_days: 180, pct: 60, severity: 2 }, // 27
    InjuryType { name: "pulled hamstring", region: 4, weight: 6, min_days: 15, var_days: 25, pct: 60, severity: 1 }, // 28
    InjuryType { name: "bruised thigh", region: 4, weight: 6, min_days: 3, var_days: 10, pct: 60, severity: 0 }, // 29
    InjuryType { name: "thigh strain", region: 4, weight: 6, min_days: 5, var_days: 20, pct: 60, severity: 0 }, // 30
    InjuryType { name: "torn hamstring", region: 4, weight: 7, min_days: 50, var_days: 45, pct: 60, severity: 2 }, // 31
    InjuryType { name: "dead leg", region: 4, weight: 7, min_days: 3, var_days: 5, pct: 60, severity: 1 }, // 32
    InjuryType { name: "gashed leg", region: 4, weight: 7, min_days: 5, var_days: 12, pct: 60, severity: 1 }, // 33
    InjuryType { name: "broken leg", region: 4, weight: 8, min_days: 90, var_days: 100, pct: 60, severity: 2 }, // 34
    InjuryType { name: "groin strain", region: 5, weight: 6, min_days: 15, var_days: 25, pct: 60, severity: 0 }, // 35
    InjuryType { name: "torn groin muscle", region: 5, weight: 7, min_days: 50, var_days: 45, pct: 60, severity: 1 }, // 36
    InjuryType { name: "hip injury", region: 5, weight: 7, min_days: 20, var_days: 100, pct: 60, severity: 2 }, // 37
    InjuryType { name: "broken pelvis", region: 5, weight: 50, min_days: 90, var_days: 150, pct: 60, severity: 2 }, // 38
    InjuryType { name: "bruised rib", region: 6, weight: 6, min_days: 3, var_days: 5, pct: 0, severity: 0 }, // 39
    InjuryType { name: "chest injury", region: 6, weight: 6, min_days: 5, var_days: 20, pct: 50, severity: 0 }, // 40
    InjuryType { name: "back strain", region: 6, weight: 6, min_days: 10, var_days: 10, pct: 60, severity: 1 }, // 41
    InjuryType { name: "fractured ribs", region: 6, weight: 7, min_days: 20, var_days: 40, pct: 60, severity: 1 }, // 42
    InjuryType { name: "broken ribs", region: 6, weight: 7, min_days: 20, var_days: 40, pct: 60, severity: 2 }, // 43
    InjuryType { name: "slipped disc", region: 6, weight: 9, min_days: 45, var_days: 180, pct: 60, severity: 2 }, // 44
    InjuryType { name: "damaged spine", region: 6, weight: 9, min_days: 45, var_days: 290, pct: 60, severity: 2 }, // 45
    InjuryType { name: "stubbed finger", region: 7, weight: 6, min_days: 1, var_days: 5, pct: 0, severity: 0 }, // 46
    InjuryType { name: "cut hand", region: 7, weight: 7, min_days: 1, var_days: 5, pct: 0, severity: 0 }, // 47
    InjuryType { name: "broken finger", region: 7, weight: 7, min_days: 3, var_days: 5, pct: 0, severity: 1 }, // 48
    InjuryType { name: "broken hand", region: 7, weight: 8, min_days: 10, var_days: 10, pct: 50, severity: 2 }, // 49
    InjuryType { name: "sprained wrist", region: 8, weight: 6, min_days: 3, var_days: 3, pct: 0, severity: 0 }, // 50
    InjuryType { name: "strained wrist", region: 8, weight: 6, min_days: 5, var_days: 5, pct: 0, severity: 0 }, // 51
    InjuryType { name: "damaged elbow", region: 8, weight: 6, min_days: 5, var_days: 10, pct: 30, severity: 1 }, // 52
    InjuryType { name: "fractured wrist", region: 8, weight: 7, min_days: 20, var_days: 15, pct: 30, severity: 1 }, // 53
    InjuryType { name: "gashed arm", region: 8, weight: 8, min_days: 3, var_days: 5, pct: 60, severity: 1 }, // 54
    InjuryType { name: "fractured arm", region: 8, weight: 8, min_days: 20, var_days: 15, pct: 60, severity: 1 }, // 55
    InjuryType { name: "broken wrist", region: 8, weight: 8, min_days: 20, var_days: 30, pct: 60, severity: 2 }, // 56
    InjuryType { name: "broken arm", region: 8, weight: 8, min_days: 20, var_days: 30, pct: 60, severity: 2 }, // 57
    InjuryType { name: "damaged shoulder", region: 9, weight: 6, min_days: 5, var_days: 25, pct: 60, severity: 0 }, // 58
    InjuryType { name: "dislocated shoulder", region: 9, weight: 8, min_days: 5, var_days: 10, pct: 20, severity: 1 }, // 59
    InjuryType { name: "broken shoulder", region: 9, weight: 8, min_days: 20, var_days: 45, pct: 60, severity: 2 }, // 60
    InjuryType { name: "strained neck", region: 10, weight: 6, min_days: 5, var_days: 10, pct: 30, severity: 0 }, // 61
    InjuryType { name: "damaged neck", region: 10, weight: 6, min_days: 10, var_days: 25, pct: 60, severity: 1 }, // 62
    InjuryType { name: "broken collarbone", region: 10, weight: 8, min_days: 30, var_days: 45, pct: 60, severity: 2 }, // 63
    InjuryType { name: "bruised head", region: 11, weight: 6, min_days: 5, var_days: 5, pct: 10, severity: 0 }, // 64
    InjuryType { name: "facial injury", region: 11, weight: 6, min_days: 3, var_days: 10, pct: 10, severity: 0 }, // 65
    InjuryType { name: "gashed head", region: 11, weight: 7, min_days: 3, var_days: 10, pct: 40, severity: 0 }, // 66
    InjuryType { name: "bruised jaw", region: 11, weight: 7, min_days: 3, var_days: 10, pct: 20, severity: 0 }, // 67
    InjuryType { name: "concussion", region: 11, weight: 7, min_days: 3, var_days: 6, pct: 40, severity: 0 }, // 68
    InjuryType { name: "broken nose", region: 11, weight: 7, min_days: 3, var_days: 6, pct: 30, severity: 0 }, // 69
    InjuryType { name: "fractured jaw", region: 11, weight: 8, min_days: 20, var_days: 20, pct: 60, severity: 1 }, // 70
    InjuryType { name: "dislocated jaw", region: 11, weight: 8, min_days: 20, var_days: 20, pct: 60, severity: 1 }, // 71
    InjuryType { name: "fractured cheekbone", region: 11, weight: 8, min_days: 20, var_days: 20, pct: 60, severity: 1 }, // 72
    InjuryType { name: "broken cheekbone", region: 11, weight: 8, min_days: 20, var_days: 60, pct: 60, severity: 2 }, // 73
    InjuryType { name: "broken jaw", region: 11, weight: 8, min_days: 20, var_days: 60, pct: 60, severity: 2 }, // 74
    InjuryType { name: "fractured skull", region: 11, weight: 9, min_days: 60, var_days: 180, pct: 60, severity: 2 }, // 75
    InjuryType { name: "arthritis", region: 3, weight: 100, min_days: 14, var_days: 90, pct: 60, severity: 2 }, // 76
    InjuryType { name: "faith healing", region: 12, weight: 0, min_days: 3, var_days: 4, pct: 0, severity: 3 }, // 77
    InjuryType { name: "physiotherapy", region: 12, weight: 0, min_days: 14, var_days: 14, pct: 50, severity: 3 }, // 78
    InjuryType { name: "surgery", region: 12, weight: 0, min_days: 60, var_days: 120, pct: 25, severity: 3 }, // 79
    InjuryType { name: "radiotherapy", region: 12, weight: 0, min_days: 210, var_days: 120, pct: 25, severity: 3 }, // 80
    InjuryType { name: "holiday", region: 12, weight: 0, min_days: 14, var_days: 0, pct: 25, severity: 3 }, // 81
];

/// Body-region -> inclusive injury-id range (0x009d4205, stride 3).
pub const INJURY_REGION_RANGES: [(u8, u8); 13] = [
    (0,4),(5,15),(16,21),(22,27),(28,34),(35,38),(39,45),(46,49),(50,57),(58,60),(61,63),(64,75),(77,81),
];

/// "Period Out" text — exact FUN_007cd7f0 rule.
pub fn period_out(days: u16) -> String {
    let d = days as i32;
    if d < 1 { String::new() }
    else if d == 1 { "1 day".to_string() }
    else if d < 14 { format!("{d} days") }
    else if d < 43 { format!("{} weeks", (d + 3) / 7) }
    else { format!("{} months", (d + 15) / 30) }
}
