//! 독립 한컴 2024 Print의 합성 굵기 계약.
//! 14 크기 × 일반/정체/굵은 이탤릭/굵은 정체, 56개 정상 저장 수식.
//! Print job ba86c360-281b-4c0f-ba34-5b3c8b876926, 13.0.0.3901.
//! 같은 공개 일반 윤곽에 획과 장평을 적용한다. 사설 윤곽·바이트는 포함하지 않는다.

use super::FontGlyphPaint;

// BaseUnit(HWPUNIT), Print의 Tz/100. 이탤릭과 정체의 값이 같다.
const BOLD_SCALE: [(u32, f64); 14] = [
    (600, 0.96153847),
    (700, 0.96666664),
    (800, 0.94285713),
    (900, 0.97435898),
    (1000, 0.97674416),
    (1100, 0.97872337),
    (1200, 0.98039215),
    (1300, 0.9818182400000001),
    (1400, 0.96666664),
    (1500, 0.984375),
    (1600, 0.98529411),
    (1800, 0.98684212),
    (2000, 0.97647057),
    (2400, 0.99009903),
];

pub(super) fn bold_paint(fs: f64, dpi: f64) -> Option<FontGlyphPaint> {
    let size = super::super::font_frame_metrics::base_unit_at_dpi(fs, dpi);
    let (_, scale_x) = BOLD_SCALE.iter().find(|(base, _)| *base == size)?;
    Some(FontGlyphPaint {
        stroke_width: fs * 0.02,
        scale_x: *scale_x,
    })
}

/// 같은 전진 격자의 물리 한 단위. 화면 픽셀이나 문서 좌표가 아니다.
pub(super) fn bold_extra_advance(dpi: f64) -> f64 {
    crate::renderer::hwpunit_to_px(75, dpi)
}
