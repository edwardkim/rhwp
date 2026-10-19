//! 일반 표와 부분 표가 같은 행 구획의 외곽 폭을 소비한다.

pub(super) fn resolved_table_width(
    rows: &[Vec<f64>],
    fallback_width: f64,
    declared_width: f64,
) -> f64 {
    let width = rows
        .iter()
        .map(|row| row.last().copied().unwrap_or(0.0))
        .fold(fallback_width, f64::max);
    // 모든 행이 선언 폭으로 닫히면 더 큰 전역 폴백은 실제 외곽이 아니다.
    // 기존 일반 표의 반올림 경계를 부분 표에도 동일하게 적용한다.
    if declared_width > 0.0
        && (width - declared_width).abs() > 0.5
        && !rows.is_empty()
        && rows
            .iter()
            .all(|row| (row.last().copied().unwrap_or(0.0) - declared_width).abs() <= 0.5)
    {
        declared_width
    } else {
        width
    }
}
