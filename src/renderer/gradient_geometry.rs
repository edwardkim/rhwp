//! 도형과 같은 물리 좌표계에서 채우기의 원형 축을 계산한다.

/// 원형 채우기는 중심에서 가장 먼 상자 모서리까지를 반지름으로 사용한다.
/// 독립 Print의 정사각형·가로/세로 직사각형·이동한 중심도 같은 물리 원을
/// 사용한다. 축별 정규화나 긴 축 절반은 모서리까지의 색상 띠를 잘라 낸다.
pub(crate) fn radial_circle(
    bbox: (f64, f64, f64, f64),
    center_x: i16,
    center_y: i16,
) -> Option<(f64, f64, f64)> {
    let (x, y, width, height) = bbox;
    if ![x, y, width, height].into_iter().all(f64::is_finite) || width <= 0.0 || height <= 0.0 {
        return None;
    }
    let local_x = width * f64::from(center_x) / 100.0;
    let local_y = height * f64::from(center_y) / 100.0;
    let dx = local_x.abs().max((width - local_x).abs());
    let dy = local_y.abs().max((height - local_y).abs());
    let geometry = (x + local_x, y + local_y, dx.hypot(dy));
    [geometry.0, geometry.1, geometry.2]
        .into_iter()
        .all(f64::is_finite)
        .then_some(geometry)
}
