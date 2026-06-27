pub const EMU_PER_POINT: f32 = 12_700.0;

pub fn pt_to_emu(value: f32) -> i64 {
    (value * EMU_PER_POINT).round() as i64
}

pub fn emu_to_pt(value: i64) -> f32 {
    value as f32 / EMU_PER_POINT
}
