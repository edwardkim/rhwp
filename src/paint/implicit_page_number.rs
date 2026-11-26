//! 한컴 2020 Print의 기본 꼬리말 쪽번호를 참조 장치에서 물리 종이로 투영한다.

pub(super) fn project_footer_number(
    logical_height: f64,
    paper_height: f64,
    bottom_margin: f64,
    font_size: f64,
    dpi: f64,
) -> Option<(f64, f64)> {
    const REFERENCE_DPI: f64 = 600.0;
    // Haansoft Batang의 원본과 한컴 Print의 내장 부분 글꼴은 hhea -145, UPEM 1024다.
    const DESCENT_EM: f64 = 145.0 / 1024.0;

    let pixels_per_unit = REFERENCE_DPI / dpi;
    let paper = (logical_height * pixels_per_unit).round();
    let margin = (bottom_margin * pixels_per_unit).round();
    let em = (font_size * pixels_per_unit).round();
    let baseline = paper - margin - (em * DESCENT_EM).ceil();
    if !paper_height.is_finite()
        || paper_height <= 0.0
        || !paper.is_finite()
        || paper <= 0.0
        || !baseline.is_finite()
        || baseline <= 0.0
        || baseline > paper
        || em <= 0.0
    {
        return None;
    }

    let physical_unit = paper_height / paper;
    Some((baseline * physical_unit, em * physical_unit))
}
