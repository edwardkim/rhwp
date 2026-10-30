//! 원본 HU 좌표의 합·차에서 생긴 부동소수점 오차와 장치 반 픽셀을 구분한다.

use crate::renderer::{DEFAULT_DPI, HWPUNIT_PER_INCH};

/// 원점과 상대 위치는 각각 장치 정수로 바뀐다. 원본 단위 변환 두 연산과
/// 차·역변환 두 연산의 반올림 오차 안에 있는 정수 HU만 복원한다.
/// 실제 소수 HU나 연속 좌표를 정수화하는 시각 허용치는 사용하지 않는다.
pub(super) fn relative_device(value: f64, origin: f64, pixel: f64) -> f64 {
    let units = HWPUNIT_PER_INCH / DEFAULT_DPI;
    let relative = value - origin;
    let hu = relative * units;
    let integral = hu.round();
    let error = (value.abs() + origin.abs()) * units * f64::EPSILON * 4.0;
    if hu.is_finite()
        && integral >= f64::from(i32::MIN)
        && integral <= f64::from(i32::MAX)
        && (hu - integral).abs() <= error
    {
        integral / (pixel * units)
    } else {
        relative / pixel
    }
}
