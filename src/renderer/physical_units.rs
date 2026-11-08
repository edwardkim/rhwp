//! 정수 문서 단위에서 온 픽셀은 같은 정수로 되돌리고 연속 좌표의 절삭은 유지한다.

use super::{hwpunit_to_px, HWPUNIT_PER_INCH};

#[inline]
pub(super) fn px_to_hwpunit(px: f64, dpi: f64) -> i32 {
    let scaled = px * HWPUNIT_PER_INCH / dpi;
    let nearest = scaled.round() as i32;
    // 임의 허용 오차로 좌표를 반올림하지 않는다. 원래 정수를 픽셀로 만든
    // 값과 비트까지 같을 때만 역변환의 부동소수점 손실을 복원한다.
    if hwpunit_to_px(nearest, dpi).to_bits() == px.to_bits() {
        nearest
    } else {
        scaled as i32
    }
}
