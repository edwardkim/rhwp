//! CLI 표시용 HWPUNIT 변환.

/// HWPUNIT(u32)을 mm로 변환한다.
pub(crate) fn hu_to_mm(hu: u32) -> f64 {
    hu as f64 * 25.4 / 7200.0
}

/// HWPUNIT(i32)을 mm로 변환한다.
pub(crate) fn hu_to_mm_i(hu: i32) -> f64 {
    hu as f64 * 25.4 / 7200.0
}
