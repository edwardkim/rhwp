//! 점 연산자의 독립 em 잉크 상자를 모든 출력에서 같은 수학 타원으로 표시한다.
//! 사설 글꼴의 윤곽을 복제하지 않는다. bbox 메타데이터와 Print 원점을 대조한다.

use super::GlyphOutlineCommand as Command;

pub(super) fn commands(text: &str, fs: f64, dpi: f64, baseline: f64) -> Option<Vec<Command>> {
    // HYhwpEQ UPM 1024의 잉크 상자. BULLET과 CDOT의 의미·표시 크기를 구분한다.
    let (left, bottom, right, top) = match text {
        "∙" => (386.0, 174.0, 638.0, 427.0),
        "⋅" => (424.0, 218.0, 598.0, 381.0),
        _ => return None,
    };
    let scale = fs / 1024.0;
    let cx = (left + right) * scale / 2.0;
    let cy = baseline
        - super::negation_geometry::math_origin_shift(fs, dpi)
        - (bottom + top) * scale / 2.0;
    let rx = (right - left) * scale / 2.0;
    let ry = (top - bottom) * scale / 2.0;
    // 단위 원의 표준 4분원 cubic 근사. 특정 글꼴의 제어점을 가져오지 않는다.
    let kappa = 4.0 * (2.0_f64.sqrt() - 1.0) / 3.0;
    let kx = rx * kappa;
    let ky = ry * kappa;
    Some(vec![
        Command::Move((cx + rx, cy)),
        Command::Cubic([(cx + rx, cy + ky), (cx + kx, cy + ry), (cx, cy + ry)]),
        Command::Cubic([(cx - kx, cy + ry), (cx - rx, cy + ky), (cx - rx, cy)]),
        Command::Cubic([(cx - rx, cy - ky), (cx - kx, cy - ry), (cx, cy - ry)]),
        Command::Cubic([(cx + kx, cy - ry), (cx + rx, cy - ky), (cx + rx, cy)]),
        Command::Close,
    ])
}
