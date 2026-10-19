//! 표준 16개 BorderFill 굵기의 한컴 Print 반복 구간.
//!
//! 2020/2024 각각 80사례의 독립 Print 벡터를 1/600 inch 단위로 대조했다.
//! 선 굵기의 반올림 결과로 반복 길이를 역산하면 얇은 선에서 원문 정보를 잃는다.
//! 따라서 BORDER_WIDTHS의 선언 index로 물리 패턴을 선택하고 출력 좌표로 변환한다.
//! 근거: mydocs/working/task_m100_7642_stage1238.md. 화면 px나 문서 ID에 맞춘 표가 아니다.

use crate::model::style::BorderLineType;
use crate::renderer::StrokeDash;

// 열: 짧은 점/공백, 긴 점/공백, 쇄선의 긴 점/공백. 단위는 1/600 inch다.
const PRINT_INTERVALS: [[u16; 6]; 16] = [
    [3, 4, 17, 9, 34, 9],
    [4, 6, 20, 12, 40, 12],
    [5, 7, 26, 14, 51, 15],
    [7, 10, 34, 21, 68, 21],
    [9, 13, 43, 27, 86, 27],
    [10, 15, 52, 29, 103, 30],
    [14, 21, 69, 42, 138, 42],
    [17, 25, 86, 51, 172, 51],
    [21, 31, 104, 62, 207, 63],
    [24, 36, 121, 72, 242, 72],
    [35, 52, 173, 105, 346, 105],
    [52, 78, 259, 156, 518, 156],
    [69, 103, 347, 206, 693, 207],
    [104, 156, 520, 311, 1039, 312],
    [139, 208, 693, 417, 1386, 417],
    [173, 259, 866, 519, 1732, 519],
];

pub(crate) fn border_dash(kind: BorderLineType, width: u8) -> Option<StrokeDash> {
    let &[short, short_gap, long, long_gap, chain, chain_gap] =
        PRINT_INTERVALS.get(usize::from(width))?;
    let intervals = match kind {
        BorderLineType::Dot => [short, short_gap, 0, 0, 0, 0],
        BorderLineType::Dash => [long, long_gap, 0, 0, 0, 0],
        BorderLineType::DashDot => [chain, chain_gap, short, chain_gap, 0, 0],
        BorderLineType::DashDotDot => [chain, chain_gap, short, chain_gap, short, chain_gap],
        // 실선·복선의 굵기/배치 및 이번 독립 대조에 없는 종류는 기존 계약을 보존한다.
        _ => return None,
    };
    Some(StrokeDash::Explicit(
        intervals.map(|value| f64::from(value) * 96.0 / 600.0),
    ))
}
