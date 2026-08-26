//! [#4100] 차트 데이터 ↔ CSV 행렬.
//!
//! 레이아웃은 **행 = 카테고리, 열 = 계열**이다. 원본 데이터 시트와 같은 모양이라
//! 사람이 스프레드시트에서 고치기 쉽고, 되돌릴 때 자리 대응이 자명하다.
//!
//! ```text
//! ,계열 1,계열 2,계열 3          분산형:  X,계열 1,계열 2
//! 항목 1,4.3,2.4,2                       0.7,2.7,1
//! 항목 2,2.5,4.4,2                       1.8,3.2,2
//! ```
//!
//! 첫 열의 뜻이 축에 따라 갈린다 — 카테고리형이면 **라벨**(대조만 하고 바꾸지 않는다),
//! 분산형이면 **X 값**(수치이고 편집 대상이다). 머리 행 첫 칸이 그 표식이다(`` vs `X`).
//!
//! ## `c:f` 를 레이아웃 근거로 쓰지 않는다
//!
//! 이슈 본문은 `c:f` 가 `Sheet1!$A$2:$A$5`(카테고리 열)/`$B$2:$B$5`(계열 열)로 적혀 있다는
//! 것을 이 레이아웃의 근거로 들었는데, 코퍼스 실측에서 **분산형 5건 전부** ser 0 의 yVal
//! 참조가 `Sheet1!$B$2:$A$4` 였다 — 시작 열 B, 끝 열 A 의 비대칭 범위다. 결론(행=카테고리)은
//! 맞지만 `c:f` 는 파싱 근거로 쓸 수 없다. 레이아웃은 요소 구조에서 유도한다.
//!
//! 인용·파싱은 [`super::table_csv`] 를 그대로 쓴다 — RFC 4180·CRLF·BOM 규약이 표 CSV 와
//! 갈리면 같은 도구로 왕복시킬 수 없다.

use super::table_csv::{matrix_to_csv, parse_csv};

/// 분산형 CSV 의 머리 행 첫 칸.
pub const SCATTER_CORNER: &str = "X";

/// CSV 에서 읽어낸 차트 행렬.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartCsv {
    /// 첫 열 — 카테고리 라벨 또는 분산형 X.
    pub labels: Vec<String>,
    /// 머리 행의 계열명.
    pub names: Vec<String>,
    /// 계열별 값. `values[계열][점]`.
    pub values: Vec<Vec<String>>,
}

/// CSV 구조 자체의 문제. **차트와의 대조는 코어 검증기가 한다** — 여기서는 CSV 가 표로
/// 성립하는지만 본다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChartCsvError {
    /// RFC 4180 파싱 실패.
    Parse(String),
    /// 레코드가 없다.
    Empty,
    /// 머리 행만 있고 데이터 행이 없다.
    NoDataRows,
    /// 행마다 열 수가 다르다.
    RaggedRow {
        row: usize,
        expected: usize,
        actual: usize,
    },
    /// 열이 하나뿐이다 — 라벨만 있고 계열이 없다.
    NoSeriesColumn,
}

impl std::fmt::Display for ChartCsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(msg) => write!(f, "CSV 파싱 실패: {msg}"),
            Self::Empty => write!(f, "CSV 가 비어 있습니다."),
            Self::NoDataRows => write!(f, "CSV 에 머리 행만 있고 데이터 행이 없습니다."),
            Self::RaggedRow {
                row,
                expected,
                actual,
            } => write!(
                f,
                "CSV {row}행의 열 수 {actual} 가 머리 행의 {expected} 와 다릅니다."
            ),
            Self::NoSeriesColumn => write!(f, "CSV 에 계열 열이 없습니다 — 첫 열은 라벨입니다."),
        }
    }
}

/// 차트 데이터를 CSV 본문으로 만든다.
///
/// 행 수는 **값이 정한다** — `labels.len()` 이 아니라 `max(labels, 각 계열 값 개수)` 다.
/// 라벨로 행 수를 정하면 `c:cat` 이 없는 차트에서 **값이 통째로 사라진 CSV** 가 나온다.
/// 실사용 문서에서 실제로 그랬다(`samples/issue2006/1790387_prep_final_report.hwpx` —
/// 6계열 × 6값인데 첫 계열에 `c:cat` 이 없어 CSV 가 머리 행만 남았다). 죽지 않고 틀린
/// 산출이라 눈으로 알아채기 어렵다.
///
/// 모자란 칸은 빈 문자열로 채운다. 여기서 거부하지 않는 이유는 이 함수가 **내보내기**이고,
/// 되돌릴 때 코어 검증기가 개수를 다시 보기 때문이다.
pub fn to_csv(
    labels: &[String],
    names: &[String],
    values: &[Vec<String>],
    scatter: bool,
) -> String {
    let rows = values
        .iter()
        .map(|s| s.len())
        .chain(std::iter::once(labels.len()))
        .max()
        .unwrap_or(0);

    let mut matrix: Vec<Vec<String>> = Vec::with_capacity(rows + 1);

    let mut header = Vec::with_capacity(names.len() + 1);
    header.push(if scatter {
        SCATTER_CORNER.to_string()
    } else {
        String::new()
    });
    header.extend(names.iter().cloned());
    matrix.push(header);

    for row in 0..rows {
        let mut record = Vec::with_capacity(values.len() + 1);
        record.push(labels.get(row).cloned().unwrap_or_default());
        for series in values {
            record.push(series.get(row).cloned().unwrap_or_default());
        }
        matrix.push(record);
    }

    matrix_to_csv(&matrix)
}

/// CSV 본문을 차트 행렬로 읽는다.
pub fn from_csv(text: &str) -> Result<ChartCsv, ChartCsvError> {
    let records = parse_csv(text).map_err(|e| ChartCsvError::Parse(e.to_string()))?;
    let Some((header, rows)) = records.split_first() else {
        return Err(ChartCsvError::Empty);
    };
    if header.len() < 2 {
        return Err(ChartCsvError::NoSeriesColumn);
    }
    if rows.is_empty() {
        return Err(ChartCsvError::NoDataRows);
    }
    for (i, record) in rows.iter().enumerate() {
        if record.len() != header.len() {
            return Err(ChartCsvError::RaggedRow {
                // 머리 행이 1행이므로 사람이 세는 행 번호는 +2 다.
                row: i + 2,
                expected: header.len(),
                actual: record.len(),
            });
        }
    }

    let names: Vec<String> = header[1..].to_vec();
    let labels: Vec<String> = rows.iter().map(|r| r[0].clone()).collect();
    let values: Vec<Vec<String>> = (1..header.len())
        .map(|col| rows.iter().map(|r| r[col].clone()).collect())
        .collect();

    Ok(ChartCsv {
        labels,
        names,
        values,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    #[test]
    fn category_layout_puts_series_in_columns() {
        let csv = to_csv(
            &s(&["항목 1", "항목 2"]),
            &s(&["계열 1", "계열 2"]),
            &[s(&["4.3", "2.5"]), s(&["2.4", "4.4"])],
            false,
        );
        assert_eq!(
            csv,
            ",계열 1,계열 2\r\n항목 1,4.3,2.4\r\n항목 2,2.5,4.4\r\n"
        );
    }

    #[test]
    fn scatter_layout_marks_the_first_column_as_x() {
        let csv = to_csv(
            &s(&["0.7", "1.8"]),
            &s(&["계열 1"]),
            &[s(&["2.7", "3.2"])],
            true,
        );
        assert_eq!(csv, "X,계열 1\r\n0.7,2.7\r\n1.8,3.2\r\n");
    }

    #[test]
    fn round_trip_preserves_every_cell() {
        let labels = s(&["항목 1", "항목 2", "항목 3"]);
        let names = s(&["계열 1", "계열 2"]);
        let values = vec![s(&["1", "2", "3"]), s(&["4.5", "6", "7.25"])];
        let csv = to_csv(&labels, &names, &values, false);
        let back = from_csv(&csv).expect("되읽기");
        assert_eq!(back.labels, labels);
        assert_eq!(back.names, names);
        assert_eq!(back.values, values);
    }

    /// 쉼표·따옴표가 든 계열명도 왕복한다 — 인용은 `table_csv` 가 한다.
    #[test]
    fn quoting_is_delegated_to_table_csv() {
        let names = s(&["매출, 원", "\"특판\""]);
        let csv = to_csv(&s(&["1월"]), &names, &[s(&["10"]), s(&["20"])], false);
        assert!(csv.contains("\"매출, 원\""), "{csv}");
        assert_eq!(from_csv(&csv).expect("되읽기").names, names);
    }

    #[test]
    fn bom_is_tolerated_on_read() {
        let csv = "\u{feff},계열 1\r\n항목 1,4.3\r\n";
        assert_eq!(from_csv(csv).expect("BOM").names, s(&["계열 1"]));
    }

    #[test]
    fn ragged_rows_are_refused_with_a_human_row_number() {
        let csv = ",계열 1,계열 2\r\n항목 1,1,2\r\n항목 2,3\r\n";
        assert_eq!(
            from_csv(csv),
            Err(ChartCsvError::RaggedRow {
                row: 3,
                expected: 3,
                actual: 2
            })
        );
    }

    #[test]
    fn structural_refusals() {
        assert_eq!(from_csv(""), Err(ChartCsvError::Empty));
        assert_eq!(from_csv("항목\r\n"), Err(ChartCsvError::NoSeriesColumn));
        assert_eq!(from_csv(",계열 1\r\n"), Err(ChartCsvError::NoDataRows));
    }

    /// 값 개수가 라벨 수보다 적으면 내보내기는 빈 칸으로 채운다 — 거부는 되돌릴 때
    /// 코어 검증기가 한다(검증기를 두 곳에 두지 않는다).
    #[test]
    fn export_pads_short_series_instead_of_refusing() {
        let csv = to_csv(&s(&["a", "b"]), &s(&["계열 1"]), &[s(&["1"])], false);
        assert_eq!(csv, ",계열 1\r\na,1\r\nb,\r\n");
    }

    /// **라벨이 없어도 값은 전부 나온다.**
    ///
    /// 행 수를 `labels.len()` 으로 잡으면 `c:cat` 없는 차트에서 값이 통째로 사라진 CSV 가
    /// 나온다 — 실사용 문서에서 실제로 그랬다(6계열 × 6값이 머리 행만 남았다). 죽지 않고
    /// 틀린 산출이라 회귀 가드를 여기 둔다.
    #[test]
    fn values_survive_even_when_labels_are_missing() {
        let csv = to_csv(
            &[],
            &s(&["계열 1", "계열 2"]),
            &[s(&["1", "2"]), s(&["3", "4"])],
            false,
        );
        assert_eq!(csv, ",계열 1,계열 2\r\n,1,3\r\n,2,4\r\n");
    }

    /// 라벨이 값보다 짧아도 값이 잘리지 않는다.
    #[test]
    fn short_labels_do_not_truncate_values() {
        let csv = to_csv(&s(&["a"]), &s(&["계열 1"]), &[s(&["1", "2", "3"])], false);
        assert_eq!(csv, ",계열 1\r\na,1\r\n,2\r\n,3\r\n");
    }
}
