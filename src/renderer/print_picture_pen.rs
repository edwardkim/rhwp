//! 인쇄 그림의 물리 띠와 최종 raster 외곽 pen은 서로 다른 단위를 소비한다.

/// 독립 2022·2024 Print 그림의 기준 장치다. 메타파일·비트맵의 재생 세대와 구분한다.
pub(crate) const DEVICE_DPI: f64 = 600.0;

pub(crate) fn raster_width([band, outline]: [f64; 2], minimum: Option<f64>) -> Option<f64> {
    let minimum = minimum?;
    if ![band, outline, minimum]
        .into_iter()
        .all(|value| value.is_finite() && value > 0.0)
    {
        return None;
    }
    let width = band + outline.max(minimum);
    width.is_finite().then_some(width)
}
