//! [#4098] 그리드의 어느 축이 계열인가를 **판정**한다. 가정하지 않는다.
//!
//! ## 왜 판정해야 하는가
//!
//! [`parser`](super::parser) 는 카테고리-major 를 하드코딩했다. 실측은 그렇지 않다 —
//! `samples/chart/**` 코퍼스 28종(한컴 2022 작성)은 **행이 계열**이고, 레거시 단독 대조군
//! `samples/143E433F503322BD33.hwp` 만 **열이 계열**이다. 고정 가정은 코퍼스 문서에서
//! 계열과 카테고리를 전치시킨다.
//!
//! ## 무엇을 증거로 삼는가
//!
//! #4055 의 `classify()` 는 OOXML 정답지와 대조하는 **오라클**이라 레거시 단독 문서에는
//! 쓸 수 없다. 대신 그리드 창 **뒤**의 계열·범례 구간을 본다. 거기에는 계열 이름
//! `VtString` 레코드가 바이트 그대로 다시 실리고, 카테고리는 실리지 않는다. 그래서
//! "머리열이 전부 재등장했는가 / 머리행이 전부 재등장했는가" 가 구조적 상호참조가 된다.
//! 라벨이 무슨 **뜻**인지는 보지 않는다.
//!
//! ## 두 세부 규칙은 실측이 강제했다
//!
//! - **가납 조건 — 데이터 행·열이 둘 다 2 이상일 때만 증거로 인정한다.** 원형 차트는
//!   데이터 행이 1개인데 범례가 슬라이스(=열)를 나열하므로 열 라벨이 전부 재등장한다.
//!   가납 조건이 없으면 원형 10건을 **정확히 반대로** 판정한다.
//! - **반대 축은 "0" 이 아니라 "불완전" 이면 된다.** 대조군은 카테고리 `2040년` 이
//!   오탐으로 재등장해 머리행 echo 가 4개 중 1개다.
//!
//! ## 증거가 없으면
//!
//! 관례(행=계열)로 접고 [`SeriesAxisEvidence::Inconclusive`] 로 **그 사실을 싣는다.**
//! 값은 어느 쪽이든 같고 이름·묶음만 달라지므로, 읽기 전체를 거부하면 범위가 틀린
//! 보수성이 된다(`mydocs/report/task_m100_4100_report.md` §3-3). 모양 자체를 못 믿는
//! 경우는 [`super::grid::GridScanError`] 가 이미 닫는다.

use serde::Serialize;

use super::grid::LegacyChartGrid;

/// 계열이 놓인 축.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SeriesAxis {
    /// 한 행이 한 계열 — 머리열이 계열 이름, 머리행이 카테고리.
    Rows,
    /// 한 열이 한 계열 — 머리행이 계열 이름, 머리열이 카테고리.
    Columns,
}

/// 무엇을 보고 [`SeriesAxis`] 를 정했는가.
///
/// [`Self::Inconclusive`] 는 **판정이 아니라 관례 폴백**이라는 선언이다. 소비자가 결정과
/// 추정을 구별할 수 있어야 하므로 IR 에 싣는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SeriesAxisEvidence {
    /// 머리열(행 이름)이 그리드 뒤 계열 구간에 전부 재등장했다.
    RowLabelsEchoed,
    /// 머리행(열 이름)이 전부 재등장했다.
    ColumnLabelsEchoed,
    /// 증거가 가납되지 않았다 — 작성기 관례(행=계열)로 접었다.
    Inconclusive,
}

/// 계열 축을 판정한다.
pub fn decide_series_axis(
    contents: &[u8],
    grid: &LegacyChartGrid,
) -> (SeriesAxis, SeriesAxisEvidence) {
    let rows = grid.data_rows();
    let cols = grid.data_cols();

    // 한 축의 데이터 항목이 1개면 "재등장하지 않음" 이 증거가 되지 않는다.
    if rows < 2 || cols < 2 {
        return (SeriesAxis::Rows, SeriesAxisEvidence::Inconclusive);
    }

    let tail = match contents.get(grid.window.end..) {
        Some(tail) => tail,
        None => return (SeriesAxis::Rows, SeriesAxisEvidence::Inconclusive),
    };

    let row_hits = (1..=rows)
        .filter(|row| echoed(grid.label_record(contents, *row as u16, 0), tail))
        .count();
    let col_hits = (1..=cols)
        .filter(|col| echoed(grid.label_record(contents, 0, *col as u16), tail))
        .count();

    if row_hits == rows && col_hits < cols {
        (SeriesAxis::Rows, SeriesAxisEvidence::RowLabelsEchoed)
    } else if col_hits == cols && row_hits < rows {
        (SeriesAxis::Columns, SeriesAxisEvidence::ColumnLabelsEchoed)
    } else {
        (SeriesAxis::Rows, SeriesAxisEvidence::Inconclusive)
    }
}

/// 라벨 레코드가 그리드 뒤에 바이트 그대로 다시 나오는가.
///
/// `<u16 len>` 접두어를 포함한 채로 비교한다 — 접두어를 빼면 `2010년` 이 `12010년` 안에서
/// 잡히는 식의 부분일치가 생긴다.
fn echoed(record: Option<&[u8]>, tail: &[u8]) -> bool {
    match record {
        Some(record) if !record.is_empty() && record.len() <= tail.len() => {
            tail.windows(record.len()).any(|window| window == record)
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ole_chart::grid::scan_legacy_grid;
    use crate::ole_chart::grid::tests::{control_like_grid, synth_grid, Cell};

    /// 지정한 셀들의 라벨 레코드를 그리드 창 **뒤**에 덧붙인다(계열 구간 모사).
    fn echo_labels(bytes: &[u8], cells: &[(u16, u16)]) -> Vec<u8> {
        let grid = scan_legacy_grid(bytes).expect("scan");
        let mut out = bytes.to_vec();
        for (row, col) in cells {
            let record = grid
                .label_record(bytes, *row, *col)
                .expect("label record")
                .to_vec();
            out.extend_from_slice(&record);
        }
        out
    }

    /// 데이터 3행 × 4열, 행이 계열.
    fn series_major_grid() -> Vec<u8> {
        synth_grid(
            4,
            5,
            &[
                Cell::Text(2, "항목 1"),
                Cell::Text(3, "항목 2"),
                Cell::Text(4, "항목 3"),
                Cell::Text(5, "항목 4"),
                Cell::Text(6, "계열 1"),
                Cell::Num(7, 4.3),
                Cell::Num(8, 2.5),
                Cell::Num(9, 3.5),
                Cell::Num(10, 4.5),
                Cell::Text(11, "계열 2"),
                Cell::Num(12, 2.4),
                Cell::Num(13, 4.4),
                Cell::Num(14, 1.8),
                Cell::Num(15, 2.8),
                Cell::Text(16, "계열 3"),
                Cell::Num(17, 2.0),
                Cell::Num(18, 2.0),
                Cell::Num(19, 3.0),
                Cell::Num(20, 5.0),
            ],
        )
    }

    #[test]
    fn row_labels_echoed_decides_rows() {
        let bytes = echo_labels(&series_major_grid(), &[(1, 0), (2, 0), (3, 0)]);
        let grid = scan_legacy_grid(&bytes).expect("scan");
        assert_eq!(
            decide_series_axis(&bytes, &grid),
            (SeriesAxis::Rows, SeriesAxisEvidence::RowLabelsEchoed)
        );
    }

    #[test]
    fn column_labels_echoed_decides_columns() {
        // 대조군 모양 — 열이 계열이다.
        let bytes = echo_labels(&control_like_grid(), &[(0, 1), (0, 2), (0, 3)]);
        let grid = scan_legacy_grid(&bytes).expect("scan");
        assert_eq!(
            decide_series_axis(&bytes, &grid),
            (SeriesAxis::Columns, SeriesAxisEvidence::ColumnLabelsEchoed)
        );
    }

    #[test]
    fn partial_echo_on_the_other_axis_does_not_block() {
        // 대조군 실측: 카테고리 `2040년` 하나가 오탐으로 재등장한다(행 echo 1/4).
        let bytes = echo_labels(&control_like_grid(), &[(0, 1), (0, 2), (0, 3), (4, 0)]);
        let grid = scan_legacy_grid(&bytes).expect("scan");
        assert_eq!(
            decide_series_axis(&bytes, &grid),
            (SeriesAxis::Columns, SeriesAxisEvidence::ColumnLabelsEchoed)
        );
    }

    #[test]
    fn single_data_row_ignores_column_echo() {
        // 원형 실측: 데이터 1행 × 4열인데 범례가 슬라이스(=열)를 나열한다.
        // 가납 조건이 없으면 여기서 `Columns` 로 뒤집힌다.
        let pie = synth_grid(
            2,
            5,
            &[
                Cell::Text(2, "1 분기"),
                Cell::Text(3, "2 분기"),
                Cell::Text(4, "3 분기"),
                Cell::Text(5, "4 분기"),
                Cell::Text(6, "판매"),
                Cell::Num(7, 4.3),
                Cell::Num(8, 2.5),
                Cell::Num(9, 3.5),
                Cell::Num(10, 4.5),
            ],
        );
        let bytes = echo_labels(&pie, &[(0, 1), (0, 2), (0, 3), (0, 4)]);
        let grid = scan_legacy_grid(&bytes).expect("scan");
        assert_eq!(
            decide_series_axis(&bytes, &grid),
            (SeriesAxis::Rows, SeriesAxisEvidence::Inconclusive)
        );
    }

    #[test]
    fn no_echo_is_inconclusive() {
        let bytes = series_major_grid();
        let grid = scan_legacy_grid(&bytes).expect("scan");
        assert_eq!(
            decide_series_axis(&bytes, &grid),
            (SeriesAxis::Rows, SeriesAxisEvidence::Inconclusive)
        );
    }

    #[test]
    fn both_axes_echoed_is_inconclusive() {
        let bytes = echo_labels(
            &series_major_grid(),
            &[(1, 0), (2, 0), (3, 0), (0, 1), (0, 2), (0, 3), (0, 4)],
        );
        let grid = scan_legacy_grid(&bytes).expect("scan");
        assert_eq!(
            decide_series_axis(&bytes, &grid),
            (SeriesAxis::Rows, SeriesAxisEvidence::Inconclusive)
        );
    }
}
