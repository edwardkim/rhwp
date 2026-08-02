//! 표 ↔ CSV 변환 코어 — 데이터 보고서 자동화의 입출구 (로드맵 #3719 §6-7).
//!
//! `export-tables` 의 격자 JSON 은 병합을 `rowSpan`/`colSpan` 으로 **보존**하지만, 표
//! 계산기(엑셀·pandas)가 먹는 것은 직사각 격자뿐이다. 모델의 `Table.cells` 는 앵커 셀만
//! 담고 병합으로 덮인 좌표는 목록에 아예 없으므로(`table_extract` 참조), 앵커를 순서대로
//! 이어 붙이면 **병합이 있는 행에서 열이 통째로 밀린다** — 오류 없이 어긋난 데이터가
//! 나온다. 그래서 여기서는 격자를 `rows`×`cols` 로 **채워서** 내보낸다: 앵커 위치에 값,
//! 덮인 칸은 빈 문자열.
//!
//! 인용은 RFC 4180 이다. 필드에 `,`·`"`·CR·LF 가 있으면 큰따옴표로 감싸고 내부 `"` 는
//! `""` 로 두 번 쓴다. 레코드 구분자는 CRLF(RFC 4180 §2.1)로 쓰고, 파서는 LF 단독도
//! 받는다(유닉스 도구가 만든 CSV 호환).
//!
//! 파서/렌더 무변경의 순수 변환 모듈이다 — 문서 모델을 읽기만 하고 쓰지 않는다.

use std::fmt;

use super::table_extract::TableGrid;

/// RFC 4180 §2.1 의 레코드 구분자.
pub const CSV_RECORD_SEPARATOR: &str = "\r\n";

/// UTF-8 BOM — 엑셀이 CP949 로 오해해 한글이 깨지는 것을 막는 선택 표식.
pub const UTF8_BOM: &str = "\u{feff}";

/// CSV 파싱 실패 위치와 사유. 봉투의 `invalid[]` 항목으로 그대로 옮겨 쓴다.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvParseError {
    /// 0 기준 레코드(행) 번호.
    pub record: usize,
    /// 0 기준 필드(열) 번호.
    pub field: usize,
    /// 사람이 읽는 사유.
    pub reason: &'static str,
}

impl fmt::Display for CsvParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "CSV {}행 {}열: {}",
            self.record + 1,
            self.field + 1,
            self.reason
        )
    }
}

/// 필드 하나를 RFC 4180 규칙으로 인용한다.
///
/// 인용이 필요 없는 값은 그대로 둔다 — 불필요한 따옴표는 사람이 읽는 진단(diff)을
/// 어지럽히고, 어떤 소비자는 값의 일부로 오독한다.
pub fn quote_field(field: &str) -> String {
    if !field.contains([',', '"', '\r', '\n']) {
        return field.to_string();
    }
    let mut out = String::with_capacity(field.len() + 2);
    out.push('"');
    for ch in field.chars() {
        if ch == '"' {
            out.push('"');
        }
        out.push(ch);
    }
    out.push('"');
    out
}

/// 병합을 채운 `rows`×`cols` 문자열 행렬.
///
/// 앵커 셀의 텍스트는 앵커 좌표에만 놓이고 덮인 칸은 빈 문자열이다. 값을 병합 범위에
/// 복제하지 않는 이유는 `csv-to-table` 왕복 때문이다 — 복제하면 되돌릴 때 덮인 칸에
/// 값이 있는 CSV 가 되어 "쓸 수 없는 칸에 값이 있다"로 거부된다.
pub fn grid_matrix(grid: &TableGrid) -> Vec<Vec<String>> {
    let rows = grid.rows as usize;
    let cols = grid.cols as usize;
    let mut matrix = vec![vec![String::new(); cols]; rows];
    for cell in &grid.cells {
        let (r, c) = (cell.row as usize, cell.col as usize);
        // `table_extract` 가 범위 밖 앵커를 이미 걸러내지만, 격자 크기가 0 인 손상
        // 문서에서도 패닉하지 않도록 여기서도 확인한다.
        if let Some(slot) = matrix.get_mut(r).and_then(|row| row.get_mut(c)) {
            slot.clone_from(&cell.text);
        }
    }
    matrix
}

/// 문자열 행렬을 CSV 본문으로 직렬화한다 (마지막 레코드에도 구분자를 붙인다).
pub fn matrix_to_csv(matrix: &[Vec<String>]) -> String {
    let mut out = String::new();
    for row in matrix {
        let mut first = true;
        for field in row {
            if !first {
                out.push(',');
            }
            first = false;
            out.push_str(&quote_field(field));
        }
        out.push_str(CSV_RECORD_SEPARATOR);
    }
    out
}

/// 표 격자 하나를 CSV 본문으로 변환한다 (병합 채움 + RFC 4180 인용).
pub fn grid_to_csv(grid: &TableGrid) -> String {
    matrix_to_csv(&grid_matrix(grid))
}

/// CSV 본문을 레코드 배열로 판독한다 (RFC 4180, LF 단독 구분자 허용, 선두 BOM 허용).
///
/// 빈 입력은 레코드 0건이다. 마지막 레코드 뒤의 구분자 하나는 빈 레코드를 만들지
/// 않는다 — 그러지 않으면 정상 CSV 가 항상 "행이 하나 많다"로 거부된다.
pub fn parse_csv(input: &str) -> Result<Vec<Vec<String>>, CsvParseError> {
    let body = input.strip_prefix(UTF8_BOM).unwrap_or(input);
    let chars: Vec<char> = body.chars().collect();
    let mut records: Vec<Vec<String>> = Vec::new();
    if chars.is_empty() {
        return Ok(records);
    }

    let mut record: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut i = 0usize;
    loop {
        if chars.get(i) == Some(&'"') {
            i += 1;
            loop {
                match chars.get(i) {
                    None => {
                        return Err(CsvParseError {
                            record: records.len(),
                            field: record.len(),
                            reason: "따옴표가 닫히지 않았습니다",
                        })
                    }
                    Some('"') => {
                        if chars.get(i + 1) == Some(&'"') {
                            field.push('"');
                            i += 2;
                        } else {
                            i += 1;
                            break;
                        }
                    }
                    Some(c) => {
                        field.push(*c);
                        i += 1;
                    }
                }
            }
            // 닫는 따옴표 뒤에 올 수 있는 것은 구분자·줄바꿈·끝뿐이다. 그 밖의 문자를
            // 조용히 이어 붙이면 `"a"b` 가 `ab` 로 통과해 원본과 다른 값이 표에 들어간다.
            match chars.get(i) {
                None | Some(',') | Some('\r') | Some('\n') => {}
                Some(_) => {
                    return Err(CsvParseError {
                        record: records.len(),
                        field: record.len(),
                        reason: "닫는 따옴표 뒤에 구분자가 아닌 문자가 있습니다 (값 안의 \" 는 \"\" 로 쓰세요)",
                    })
                }
            }
        } else {
            while let Some(c) = chars.get(i) {
                if matches!(c, ',' | '\r' | '\n') {
                    break;
                }
                field.push(*c);
                i += 1;
            }
        }
        record.push(std::mem::take(&mut field));

        match chars.get(i) {
            Some(',') => i += 1,
            Some('\r') | Some('\n') => {
                if chars.get(i) == Some(&'\r') && chars.get(i + 1) == Some(&'\n') {
                    i += 2;
                } else {
                    i += 1;
                }
                records.push(std::mem::take(&mut record));
                if i >= chars.len() {
                    break;
                }
            }
            _ => {
                records.push(std::mem::take(&mut record));
                break;
            }
        }
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quotes_only_when_needed() {
        assert_eq!(quote_field("보통값"), "보통값");
        assert_eq!(quote_field("가,나"), "\"가,나\"");
        assert_eq!(quote_field("따\"옴"), "\"따\"\"옴\"");
        assert_eq!(quote_field("줄\n바꿈"), "\"줄\n바꿈\"");
    }

    #[test]
    fn round_trip_preserves_special_characters() {
        let matrix = vec![vec![
            "a,b".to_string(),
            "c\"d".to_string(),
            "e\nf".to_string(),
            String::new(),
        ]];
        let csv = matrix_to_csv(&matrix);
        assert_eq!(parse_csv(&csv).expect("파싱"), matrix);
    }

    #[test]
    fn trailing_separator_does_not_add_a_record() {
        assert_eq!(
            parse_csv("a,b\r\nc,d\r\n").expect("파싱"),
            vec![vec!["a", "b"], vec!["c", "d"]]
        );
        assert_eq!(
            parse_csv("a,b\nc,d").expect("파싱"),
            vec![vec!["a", "b"], vec!["c", "d"]]
        );
    }

    #[test]
    fn empty_fields_are_kept() {
        assert_eq!(parse_csv("a,,c").expect("파싱"), vec![vec!["a", "", "c"]]);
        assert_eq!(parse_csv("a,").expect("파싱"), vec![vec!["a", ""]]);
        assert_eq!(parse_csv(",").expect("파싱"), vec![vec!["", ""]]);
    }

    #[test]
    fn bom_is_stripped() {
        let csv = format!("{UTF8_BOM}값");
        assert_eq!(parse_csv(&csv).expect("파싱"), vec![vec!["값"]]);
    }

    #[test]
    fn empty_input_is_zero_records() {
        assert!(parse_csv("").expect("파싱").is_empty());
    }

    #[test]
    fn unterminated_quote_is_an_error() {
        let e = parse_csv("a,\"b").expect_err("오류여야 함");
        assert_eq!(e.record, 0);
        assert_eq!(e.field, 1);
    }

    #[test]
    fn garbage_after_closing_quote_is_an_error() {
        let e = parse_csv("\"a\"b").expect_err("오류여야 함");
        assert_eq!(e.record, 0);
        assert_eq!(e.field, 0);
    }
}
