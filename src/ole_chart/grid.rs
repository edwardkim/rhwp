//! [#4098] 레거시 `Contents` 의 `VtDataGrid` 를 **구조로** 읽는다.
//!
//! ## 왜 별도 스캐너인가
//!
//! 같은 모듈의 [`parser`](super::parser) 는 그리드를 **값의 성질로 짐작**했다. "정수 ·
//! 1 이상 · 100만 이하" 만 값으로 인정하는 필터가 값을 거르는 데 그치지 않고 연속 f64
//! 런의 **프레임 기준**이어서, 실제 차트의 `4.3` 하나가 런을 끊어 파싱 전체를 무너뜨렸다.
//! 개수도 라벨에서 역산했기 때문에 라벨 분류가 틀리면 값 탐색까지 같이 틀렸다.
//!
//! 이 스캐너는 값을 **보지 않는다.** 크기·부호·정수 여부 어느 것도 판단 재료가 아니다.
//!
//! ## 문법 (실측: 코퍼스 28종 + 레거시 단독 대조군)
//!
//! `VtDataGrid` 프롤로그가 치수를 **명시**한다. 행 pitch 추론도 stride 가정도 필요 없다.
//!
//! ```text
//! <u16 11> "VtDataGrid\0"   <u16 ver> <u32>
//! <u16  9> "VtMatrix\0"     <u16 ver> <u32>
//! <u16 13> "VtCollection\0" <u16 ver> <u16> <u32>
//! <u16  9> "VtObject\0"     <u16 ver> <u16 ROWS> <u16 COLS>
//! ```
//!
//! 이어서 셀이 온다. 셀마다 **1-based 행우선 인덱스**를 싣고 코너 셀(index 1)은 없다 —
//! `(row, col) = ((index - 1) / COLS, (index - 1) % COLS)`. 0 행은 열 이름, 0 열은 행 이름,
//! 나머지가 수치다.
//!
//! ```text
//! <u32 owner> <u32 index> <u32 typeId>   typeId 5 = 문자, 7 = 수치
//! [<u16 9> "VtDouble\0"|"VtString\0" <u16 ver>]   최초 사용 시에만
//! <payload>                              f64 8B  |  <u16 len> cp949 \0\0 utf16le \0\0
//! <separator>                            수치 뒤에만 `FF FF 06 00 00 00`
//! ```
//!
//! 그래서 수치는 구분자로, 문자는 형상으로 찾고 **양쪽 다 12바이트 헤더를 되읽어
//! `typeId` 로 확인**한다. 형상만 맞는 우연은 헤더에서 걸린다.
//!
//! ## 구간 제한은 필수다
//!
//! `VtDataGrid` 창 밖에는 축 눈금 같은 무관한 `VtDouble` 이 있다. 대조군 실측으로
//! 제한 12 · 무제한 14 다(#4055 Stage 1).

use std::ops::Range;

use encoding_rs::EUC_KR;

const GRID_MARKER: &[u8] = b"VtDataGrid\0";
const MATRIX_MARKER: &[u8] = b"VtMatrix\0";
const COLLECTION_MARKER: &[u8] = b"VtCollection\0";
const OBJECT_MARKER: &[u8] = b"VtObject\0";
const DOUBLE_MARKER: &[u8] = b"VtDouble\0";
const STRING_MARKER: &[u8] = b"VtString\0";

/// 수치 셀 f64 바로 뒤에 오는 구분자. 문자 셀에는 붙지 않는다.
const VALUE_SEPARATOR: &[u8] = &[0xFF, 0xFF, 0x06, 0x00, 0x00, 0x00];

const TYPE_STRING: u32 = 5;
const TYPE_DOUBLE: u32 = 7;

/// `<u16 nameLen> <name> <u16 version>` 인라인 클래스 선언의 길이.
const fn declaration_len(marker: &[u8]) -> usize {
    2 + marker.len() + 2
}

/// `VtDataGrid` 다음에 오는 형제 오브젝트 마커 — 그리드 구간의 끝을 정한다.
///
/// 이 목록의 **유일한 사본**이다. `parser.rs` 와 `tests/support/` 에 있던 중복 2벌을
/// 여기로 접었다(#4098).
const OBJECT_MARKERS: &[&[u8]] = &[
    b"VtBackdrop\0",
    b"VtBackDrop\0",
    b"VtChartSection\0",
    b"VtFootnote\0",
    b"VtLegend\0",
    b"VtPlot\0",
    b"VtPrintInformation\0",
    b"VtChartTitle\0",
    b"VtTitle\0",
];

/// 셀이 싣고 있는 값.
#[derive(Debug, Clone, PartialEq)]
pub enum GridValue {
    /// 수치 셀.
    ///
    /// `offset` 은 f64 8바이트의 시작이다. 길이가 변하지 않으므로 **in-place 패치 주소**로
    /// 그대로 쓸 수 있다 — 레거시 값 쓰기(#4100 후속)가 이 주소를 필요로 한다.
    Number { value: f64, offset: usize },
    /// 문자 셀.
    ///
    /// `record` 는 `<u16 len>` 접두어를 **포함한** 원본 구간이다. 길이가 바뀌므로 in-place
    /// 패치 대상이 아니고, 방향 판정의 바이트 대조에 쓴다.
    Text { text: String, record: Range<usize> },
}

/// 셀 하나.
#[derive(Debug, Clone, PartialEq)]
pub struct GridCell {
    /// 원본이 실은 1-based 행우선 인덱스.
    pub index: u32,
    pub row: u16,
    pub col: u16,
    pub value: GridValue,
}

/// `VtDataGrid` 하나를 구조로 읽은 결과.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyChartGrid {
    /// 그리드 데이터 구간 `[start, end)`.
    pub window: Range<usize>,
    /// 머리행·머리열을 **포함한** 치수. `VtObject` 가 명시한 값이다.
    pub rows: u16,
    pub cols: u16,
    /// `index` 오름차순. 코너 셀(index 1)은 없다.
    pub cells: Vec<GridCell>,
}

impl LegacyChartGrid {
    /// 머리행을 뺀 데이터 행 수.
    ///
    /// `scan_legacy_grid` 는 `rows >= 2` 만 내보내지만 필드가 공개라 직접 구성한
    /// 값에서도 감산이 넘치지 않게 한다.
    pub fn data_rows(&self) -> usize {
        (self.rows as usize).saturating_sub(1)
    }

    /// 머리열을 뺀 데이터 열 수.
    pub fn data_cols(&self) -> usize {
        (self.cols as usize).saturating_sub(1)
    }

    pub fn cell(&self, row: u16, col: u16) -> Option<&GridCell> {
        self.cells
            .iter()
            .find(|cell| cell.row == row && cell.col == col)
    }

    /// 데이터 행 `row`(1-based) 의 이름 — 셀 `(row, 0)`.
    pub fn row_label(&self, row: u16) -> Option<&str> {
        match &self.cell(row, 0)?.value {
            GridValue::Text { text, .. } => Some(text.as_str()),
            GridValue::Number { .. } => None,
        }
    }

    /// 데이터 열 `col`(1-based) 의 이름 — 셀 `(0, col)`.
    pub fn column_label(&self, col: u16) -> Option<&str> {
        match &self.cell(0, col)?.value {
            GridValue::Text { text, .. } => Some(text.as_str()),
            GridValue::Number { .. } => None,
        }
    }

    pub fn number(&self, row: u16, col: u16) -> Option<f64> {
        match &self.cell(row, col)?.value {
            GridValue::Number { value, .. } => Some(*value),
            GridValue::Text { .. } => None,
        }
    }

    /// 수치 셀의 `(패치 주소, 값)` 을 인덱스 순서로.
    pub fn value_offsets(&self) -> impl Iterator<Item = (usize, f64)> + '_ {
        self.cells.iter().filter_map(|cell| match &cell.value {
            GridValue::Number { value, offset } => Some((*offset, *value)),
            GridValue::Text { .. } => None,
        })
    }

    /// 라벨 셀의 원본 레코드 바이트(`<u16 len>` 포함).
    pub(crate) fn label_record<'a>(
        &self,
        contents: &'a [u8],
        row: u16,
        col: u16,
    ) -> Option<&'a [u8]> {
        match &self.cell(row, col)?.value {
            GridValue::Text { record, .. } => contents.get(record.clone()),
            GridValue::Number { .. } => None,
        }
    }
}

/// 그리드를 구조로 읽지 못한 사유.
///
/// 전부 **모양을 신뢰할 수 없다**는 뜻이다. 이름을 못 정하는 것과 다르다 — 이름 모호는
/// 판정 표지로 싣고 통과시킨다([`super::orientation`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridScanError {
    /// `VtDataGrid` 마커가 없다.
    MarkerNotFound,
    /// 프롤로그의 선언 순서가 실측 규약과 다르다 — 알려지지 않은 작성기다.
    PrologueMismatch {
        at: usize,
    },
    /// 데이터 셀이 없는 치수다.
    EmptyGrid {
        rows: u16,
        cols: u16,
    },
    CellIndexOutOfRange {
        index: u32,
        rows: u16,
        cols: u16,
    },
    DuplicateCellIndex {
        index: u32,
    },
    /// 수치 셀 개수가 `(rows - 1) * (cols - 1)` 과 다르다.
    NumberCellCountMismatch {
        found: usize,
        expected: usize,
    },
}

impl GridScanError {
    /// 기존 `OleChartParseError::UnsupportedContentsLayout` 의 `reason` 어휘로 옮긴다.
    pub(crate) fn reason(&self) -> &'static str {
        match self {
            Self::MarkerNotFound => "legacy HWP chart data grid marker not found",
            Self::PrologueMismatch { .. } => "legacy HWP chart data grid prologue not recognized",
            Self::EmptyGrid { .. } => "legacy HWP chart data grid is empty",
            Self::CellIndexOutOfRange { .. } | Self::DuplicateCellIndex { .. } => {
                "legacy HWP chart data grid cell index is inconsistent"
            }
            Self::NumberCellCountMismatch { .. } => {
                "legacy HWP chart data grid shape not recognized"
            }
        }
    }
}

/// `start` 이후 최초의 형제 오브젝트 마커 위치.
pub(crate) fn next_object_marker(contents: &[u8], start: usize) -> Option<usize> {
    OBJECT_MARKERS
        .iter()
        .filter_map(|marker| find_from(contents, marker, start))
        .min()
}

/// `VtDataGrid` 데이터 구간 `[start, end)`.
///
/// 시작은 `VtDataGrid` 이름 뒤의 `version(u16) + payload(u32)` 뒤다. 끝은 최초의 형제
/// 마커이고, 없으면 스트림 끝으로 닫는다.
pub fn legacy_grid_window(contents: &[u8]) -> Option<Range<usize>> {
    let marker = find_from(contents, GRID_MARKER, 0)?;
    grid_window_from(contents, marker)
}

fn grid_window_from(contents: &[u8], marker: usize) -> Option<Range<usize>> {
    let start = marker
        .checked_add(GRID_MARKER.len())?
        .checked_add(2)?
        .checked_add(4)?;
    if start > contents.len() {
        return None;
    }
    let end = next_object_marker(contents, start).unwrap_or(contents.len());
    if end < start {
        return None;
    }
    Some(start..end)
}

/// `Contents` 의 `VtDataGrid` 를 구조로 읽는다.
pub fn scan_legacy_grid(contents: &[u8]) -> Result<LegacyChartGrid, GridScanError> {
    let marker = find_from(contents, GRID_MARKER, 0).ok_or(GridScanError::MarkerNotFound)?;
    let window = grid_window_from(contents, marker).ok_or(GridScanError::MarkerNotFound)?;
    let (rows, cols) = read_prologue(contents, marker)?;

    if rows < 2 || cols < 2 {
        return Err(GridScanError::EmptyGrid { rows, cols });
    }
    let expected = (rows as usize - 1) * (cols as usize - 1);
    let cell_count = rows as usize * cols as usize;

    // rows/cols 는 문서가 선언한 값이므로, 그 값만으로 대규모 메모리를 예약하지 않는다.
    // 실제 셀 수는 아래에서 입력 바이트를 훑어 수집하고, 마지막에 선언 치수와 대조한다.
    let mut cells = Vec::new();
    collect_number_cells(contents, &window, &mut cells);
    collect_text_cells(contents, &window, &mut cells);
    cells.sort_by_key(|cell| cell.index);

    for (position, cell) in cells.iter().enumerate() {
        if cell.index < 2 || cell.index as usize > cell_count {
            return Err(GridScanError::CellIndexOutOfRange {
                index: cell.index,
                rows,
                cols,
            });
        }
        if position > 0 && cells[position - 1].index == cell.index {
            return Err(GridScanError::DuplicateCellIndex { index: cell.index });
        }
    }

    // 좌표는 인덱스와 `cols` 에서 나온다.
    for cell in &mut cells {
        let zero_based = cell.index - 1;
        cell.row = (zero_based / cols as u32) as u16;
        cell.col = (zero_based % cols as u32) as u16;
    }

    // 수치는 머리행·머리열에 오지 않는다. 오면 모양을 잘못 읽은 것이다.
    let numbers = cells
        .iter()
        .filter(|cell| matches!(cell.value, GridValue::Number { .. }))
        .filter(|cell| cell.row > 0 && cell.col > 0)
        .count();
    let misplaced = cells
        .iter()
        .filter(|cell| matches!(cell.value, GridValue::Number { .. }))
        .count()
        - numbers;
    if misplaced > 0 || numbers != expected {
        return Err(GridScanError::NumberCellCountMismatch {
            found: numbers,
            expected,
        });
    }

    Ok(LegacyChartGrid {
        window,
        rows,
        cols,
        cells,
    })
}

fn read_prologue(contents: &[u8], marker: usize) -> Result<(u16, u16), GridScanError> {
    // `VtDataGrid` 이름 뒤 version(u16) + payload(u32).
    let mut at = marker + GRID_MARKER.len() + 2 + 4;
    for (name, extra) in [
        (MATRIX_MARKER, 4usize),
        (COLLECTION_MARKER, 6usize),
        (OBJECT_MARKER, 0usize),
    ] {
        if !declares(contents, at, name) {
            return Err(GridScanError::PrologueMismatch { at });
        }
        at += declaration_len(name) + extra;
    }
    let rows = read_u16(contents, at).ok_or(GridScanError::PrologueMismatch { at })?;
    let cols = read_u16(contents, at + 2).ok_or(GridScanError::PrologueMismatch { at })?;
    Ok((rows, cols))
}

/// `at` 에 `<u16 nameLen> <name>` 인라인 선언이 있는가.
fn declares(contents: &[u8], at: usize, name: &[u8]) -> bool {
    read_u16(contents, at) == Some(name.len() as u16)
        && contents.get(at + 2..at + 2 + name.len()) == Some(name)
}

fn collect_number_cells(contents: &[u8], window: &Range<usize>, out: &mut Vec<GridCell>) {
    // 첫 값은 `VtDouble` 선언 뒤에 온다. 그 앞의 구분자 유사 바이트는 값이 아니다.
    let Some(anchor) = find_from(&contents[..window.end], DOUBLE_MARKER, window.start) else {
        return;
    };

    let mut cursor = anchor;
    while let Some(hit) = find_from(&contents[..window.end], VALUE_SEPARATOR, cursor + 1) {
        cursor = hit;
        let Some(offset) = hit.checked_sub(8) else {
            continue;
        };
        if offset < anchor {
            continue;
        }
        let Some((index, type_id)) = cell_header(contents, offset) else {
            continue;
        };
        if type_id != TYPE_DOUBLE {
            continue;
        }
        let Some(raw) = contents.get(offset..hit) else {
            continue;
        };
        let value = f64::from_le_bytes(raw.try_into().expect("8바이트"));
        out.push(GridCell {
            index,
            row: 0,
            col: 0,
            value: GridValue::Number { value, offset },
        });
    }
}

fn collect_text_cells(contents: &[u8], window: &Range<usize>, out: &mut Vec<GridCell>) {
    let mut at = window.start;
    while at + 2 < window.end {
        if let Some(len) = read_u16(contents, at).map(usize::from) {
            let payload_end = at + 2 + len;
            if (2..=256).contains(&len) && payload_end <= window.end {
                if let Some(text) = decode_cell_text(&contents[at + 2..payload_end]) {
                    if let Some((index, TYPE_STRING)) = cell_header(contents, at) {
                        out.push(GridCell {
                            index,
                            row: 0,
                            col: 0,
                            value: GridValue::Text {
                                text,
                                record: at..payload_end,
                            },
                        });
                        at = payload_end;
                        continue;
                    }
                }
            }
        }
        at += 1;
    }
}

/// 페이로드 앞의 `<u32 owner> <u32 index> <u32 typeId>` 를 되읽는다.
///
/// 최초 사용 셀에는 페이로드 바로 앞에 인라인 클래스 선언이 끼므로 그만큼 더 물러선다.
fn cell_header(contents: &[u8], payload_start: usize) -> Option<(u32, u32)> {
    let mut at = payload_start;
    for marker in [DOUBLE_MARKER, STRING_MARKER] {
        let len = declaration_len(marker);
        if at >= len && declares(contents, at - len, marker) {
            at -= len;
            break;
        }
    }
    let header = at.checked_sub(12)?;
    let index = read_u32(contents, header + 4)?;
    let type_id = read_u32(contents, header + 8)?;
    Some((index, type_id))
}

/// 셀 문자열 페이로드 — `cp949 \0\0 utf16le \0\0`.
///
/// **UTF-16 절반을 정본으로 읽는다.** cp949 절반은 ASCII 라벨(`0.7`)에서 길이가 홀수라
/// 짝수 정렬을 가정하면 안 되고, 확장 문자에서 EUC-KR 왕복이 손실될 수 있다. UTF-16
/// 절반이 없는 작성기를 위해 cp949 로 폴백한다.
fn decode_cell_text(payload: &[u8]) -> Option<String> {
    let split = payload.windows(2).position(|pair| pair == [0, 0])?;
    let tail = payload.get(split + 2..)?;

    let text = if tail.len() >= 2 && tail.ends_with(&[0, 0]) && (tail.len() - 2) % 2 == 0 {
        let units: Vec<u16> = tail[..tail.len() - 2]
            .chunks_exact(2)
            .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
            .collect();
        String::from_utf16(&units).ok()?
    } else {
        let (decoded, _, had_errors) = EUC_KR.decode(&payload[..split]);
        if had_errors {
            return None;
        }
        decoded.into_owned()
    };

    let text = text.replace('\u{3000}', " ").trim().to_string();
    if text.is_empty() || text.chars().any(char::is_control) {
        return None;
    }
    Some(text)
}

fn find_from(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    if needle.is_empty() || start >= haystack.len() || needle.len() > haystack.len() - start {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| start + offset)
}

fn read_u16(bytes: &[u8], at: usize) -> Option<u16> {
    let raw = bytes.get(at..at + 2)?;
    Some(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], at: usize) -> Option<u32> {
    let raw = bytes.get(at..at + 4)?;
    Some(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// 셀 하나의 합성 명세.
    pub(crate) enum Cell {
        Num(u32, f64),
        Text(u32, &'static str),
    }

    /// 실측 문법 그대로 `VtDataGrid` 를 합성한다.
    ///
    /// 코퍼스가 훑지 못하는 경로(0·음수·거대값, 창 밖 값, 깨진 인덱스)를 픽스처 없이
    /// 재현하기 위한 것이다.
    pub(crate) fn synth_grid(rows: u16, cols: u16, cells: &[Cell]) -> Vec<u8> {
        fn declare(out: &mut Vec<u8>, name: &[u8]) {
            out.extend_from_slice(&(name.len() as u16).to_le_bytes());
            out.extend_from_slice(name);
            out.extend_from_slice(&1u16.to_le_bytes());
        }

        let mut out = vec![0u8; 16];
        declare(&mut out, GRID_MARKER);
        out.extend_from_slice(&2u32.to_le_bytes());
        declare(&mut out, MATRIX_MARKER);
        out.extend_from_slice(&3u32.to_le_bytes());
        declare(&mut out, COLLECTION_MARKER);
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&4u32.to_le_bytes());
        declare(&mut out, OBJECT_MARKER);
        out.extend_from_slice(&rows.to_le_bytes());
        out.extend_from_slice(&cols.to_le_bytes());

        let (mut declared_double, mut declared_string) = (false, false);
        for cell in cells {
            match cell {
                Cell::Num(index, value) => {
                    out.extend_from_slice(&4u32.to_le_bytes());
                    out.extend_from_slice(&index.to_le_bytes());
                    out.extend_from_slice(&TYPE_DOUBLE.to_le_bytes());
                    if !declared_double {
                        declare(&mut out, DOUBLE_MARKER);
                        declared_double = true;
                    }
                    out.extend_from_slice(&value.to_le_bytes());
                    out.extend_from_slice(VALUE_SEPARATOR);
                }
                Cell::Text(index, text) => {
                    out.extend_from_slice(&4u32.to_le_bytes());
                    out.extend_from_slice(&index.to_le_bytes());
                    out.extend_from_slice(&TYPE_STRING.to_le_bytes());
                    if !declared_string {
                        declare(&mut out, STRING_MARKER);
                        declared_string = true;
                    }
                    let (cp949, _, _) = EUC_KR.encode(text);
                    let mut payload = cp949.into_owned();
                    payload.extend_from_slice(&[0, 0]);
                    for unit in text.encode_utf16() {
                        payload.extend_from_slice(&unit.to_le_bytes());
                    }
                    payload.extend_from_slice(&[0, 0]);
                    out.extend_from_slice(&(payload.len() as u16).to_le_bytes());
                    out.extend_from_slice(&payload);
                    // 문자 셀 뒤에는 수치 구분자가 붙지 않는다(실측).
                    out.extend_from_slice(&[0x00, 0x06, 0x00, 0x00, 0x00]);
                }
            }
        }
        out.extend_from_slice(b"VtPlot\0");
        out
    }

    /// 대조군과 같은 3계열 × 4카테고리, 카테고리-major 배치.
    pub(crate) fn control_like_grid() -> Vec<u8> {
        synth_grid(
            5,
            4,
            &[
                Cell::Text(2, "적립금"),
                Cell::Text(3, "수입"),
                Cell::Text(4, "지출"),
                Cell::Text(5, "2010년"),
                Cell::Num(6, 328.0),
                Cell::Num(7, 50.0),
                Cell::Num(8, 11.0),
                Cell::Text(9, "2020년"),
                Cell::Num(10, 812.0),
                Cell::Num(11, 70.0),
                Cell::Num(12, 15.0),
                Cell::Text(13, "2030년"),
                Cell::Num(14, 1702.0),
                Cell::Num(15, 189.0),
                Cell::Num(16, 201.0),
                Cell::Text(17, "2040년"),
                Cell::Num(18, 1477.0),
                Cell::Num(19, 191.0),
                Cell::Num(20, 289.0),
            ],
        )
    }

    #[test]
    fn reads_dimensions_and_places_cells_row_major() {
        let grid = scan_legacy_grid(&control_like_grid()).expect("scan");
        assert_eq!((grid.rows, grid.cols), (5, 4));
        assert_eq!((grid.data_rows(), grid.data_cols()), (4, 3));
        assert_eq!(grid.column_label(1), Some("적립금"));
        assert_eq!(grid.row_label(1), Some("2010년"));
        assert_eq!(grid.number(1, 1), Some(328.0));
        assert_eq!(grid.number(1, 2), Some(50.0));
        assert_eq!(grid.number(4, 3), Some(289.0));
    }

    #[test]
    fn grid_window_starts_after_datagrid_declaration() {
        let bytes = control_like_grid();
        let marker = find_from(&bytes, GRID_MARKER, 0).expect("VtDataGrid");
        let expected_start = marker + GRID_MARKER.len() + 2 + 4;
        let expected_end = find_from(&bytes, b"VtPlot\0", expected_start).expect("VtPlot");

        assert_eq!(
            legacy_grid_window(&bytes),
            Some(expected_start..expected_end),
            "window는 VtDataGrid 선언 전체 뒤에서 시작해야 한다"
        );
    }

    #[test]
    fn zero_negative_and_fractional_values_are_read() {
        // 전부 옛 `is_plausible_grid_value` 가 거부하던 값이다.
        let bytes = synth_grid(
            2,
            5,
            &[
                Cell::Text(2, "봄"),
                Cell::Text(3, "여름"),
                Cell::Text(4, "가을"),
                Cell::Text(5, "겨울"),
                Cell::Text(6, "판매"),
                Cell::Num(7, 0.0),
                Cell::Num(8, -3.5),
                Cell::Num(9, 1.0e12),
                Cell::Num(10, 1_000_001.0),
            ],
        );
        let grid = scan_legacy_grid(&bytes).expect("scan");
        let values: Vec<f64> = grid.value_offsets().map(|(_, value)| value).collect();
        assert_eq!(values, [0.0, -3.5, 1.0e12, 1_000_001.0]);
    }

    #[test]
    fn decimal_values_are_read() {
        let bytes = synth_grid(
            2,
            3,
            &[
                Cell::Text(2, "항목 1"),
                Cell::Text(3, "항목 2"),
                Cell::Text(4, "계열 1"),
                Cell::Num(5, 4.3),
                Cell::Num(6, 2.5),
            ],
        );
        let grid = scan_legacy_grid(&bytes).expect("scan");
        assert_eq!(grid.number(1, 1), Some(4.3));
        assert_eq!(grid.number(1, 2), Some(2.5));
    }

    #[test]
    fn value_offsets_round_trip_to_the_bytes() {
        let bytes = control_like_grid();
        let grid = scan_legacy_grid(&bytes).expect("scan");
        let mut seen = 0;
        for (offset, value) in grid.value_offsets() {
            let raw: [u8; 8] = bytes[offset..offset + 8].try_into().expect("8바이트");
            assert_eq!(
                f64::from_le_bytes(raw),
                value,
                "offset {offset} 재독 불일치"
            );
            seen += 1;
        }
        assert_eq!(seen, 12);
    }

    #[test]
    fn ascii_labels_with_odd_length_cp949_half_are_decoded() {
        // `0.7` 은 cp949 절반이 3바이트라 짝수 정렬을 가정하면 놓친다(분산형 실측).
        let bytes = synth_grid(
            2,
            4,
            &[
                Cell::Text(2, "0.7"),
                Cell::Text(3, "1.8"),
                Cell::Text(4, "2.6"),
                Cell::Text(5, "Y1 값"),
                Cell::Num(6, 1.5),
                Cell::Num(7, 2.5),
                Cell::Num(8, 3.5),
            ],
        );
        let grid = scan_legacy_grid(&bytes).expect("scan");
        assert_eq!(grid.column_label(1), Some("0.7"));
        assert_eq!(grid.column_label(3), Some("2.6"));
        assert_eq!(grid.row_label(1), Some("Y1 값"));
    }

    #[test]
    fn values_outside_the_window_are_ignored() {
        let mut bytes = control_like_grid();
        // 창을 닫는 `VtPlot` 뒤에 축 눈금처럼 보이는 값을 심는다.
        bytes.extend_from_slice(&4u32.to_le_bytes());
        bytes.extend_from_slice(&21u32.to_le_bytes());
        bytes.extend_from_slice(&TYPE_DOUBLE.to_le_bytes());
        bytes.extend_from_slice(&9999.0f64.to_le_bytes());
        bytes.extend_from_slice(VALUE_SEPARATOR);

        let grid = scan_legacy_grid(&bytes).expect("scan");
        assert_eq!(
            grid.value_offsets().count(),
            12,
            "창 밖 값을 주워 오면 안 된다"
        );
    }

    #[test]
    fn number_count_mismatch_is_rejected() {
        let bytes = synth_grid(
            3,
            3,
            &[
                Cell::Text(2, "항목 1"),
                Cell::Text(3, "항목 2"),
                Cell::Text(4, "계열 1"),
                Cell::Num(5, 1.0),
                Cell::Num(6, 2.0),
                Cell::Text(7, "계열 2"),
                // (2,1) 과 (2,2) 중 하나가 빠졌다.
                Cell::Num(8, 3.0),
            ],
        );
        assert_eq!(
            scan_legacy_grid(&bytes),
            Err(GridScanError::NumberCellCountMismatch {
                found: 3,
                expected: 4
            })
        );
    }

    #[test]
    fn out_of_range_cell_index_is_rejected() {
        let bytes = synth_grid(
            2,
            2,
            &[
                Cell::Text(2, "항목 1"),
                Cell::Text(3, "계열 1"),
                Cell::Num(99, 1.0),
            ],
        );
        assert!(matches!(
            scan_legacy_grid(&bytes),
            Err(GridScanError::CellIndexOutOfRange { index: 99, .. })
        ));
    }

    #[test]
    fn duplicate_cell_index_is_rejected() {
        let bytes = synth_grid(
            2,
            3,
            &[
                Cell::Text(2, "항목 1"),
                Cell::Text(3, "항목 2"),
                Cell::Text(4, "계열 1"),
                Cell::Num(5, 1.0),
                Cell::Num(5, 2.0),
            ],
        );
        assert_eq!(
            scan_legacy_grid(&bytes),
            Err(GridScanError::DuplicateCellIndex { index: 5 })
        );
    }

    #[test]
    fn prologue_mismatch_is_rejected() {
        let mut bytes = control_like_grid();
        let at = find_from(&bytes, MATRIX_MARKER, 0).expect("VtMatrix");
        bytes[at..at + 2].copy_from_slice(b"Xx");
        assert!(matches!(
            scan_legacy_grid(&bytes),
            Err(GridScanError::PrologueMismatch { .. })
        ));
    }

    #[test]
    fn missing_marker_is_rejected() {
        assert_eq!(
            scan_legacy_grid(&[0u8; 64]),
            Err(GridScanError::MarkerNotFound)
        );
    }

    #[test]
    fn degenerate_dimensions_are_rejected() {
        let bytes = synth_grid(1, 4, &[Cell::Text(2, "항목 1")]);
        assert_eq!(
            scan_legacy_grid(&bytes),
            Err(GridScanError::EmptyGrid { rows: 1, cols: 4 })
        );
    }

    #[test]
    fn oversized_declared_dimensions_do_not_preallocate_cells() {
        // 작은 손상 스트림이 최대 u16 치수를 주장해도, 입력에 없는 셀만큼 메모리를
        // 예약하지 않고 최종 구조 검증으로 거부해야 한다.
        let bytes = synth_grid(u16::MAX, u16::MAX, &[]);
        assert_eq!(
            scan_legacy_grid(&bytes),
            Err(GridScanError::NumberCellCountMismatch {
                found: 0,
                expected: (u16::MAX as usize - 1) * (u16::MAX as usize - 1),
            })
        );
    }

    #[test]
    fn single_cell_grid_is_read() {
        let bytes = synth_grid(
            2,
            2,
            &[
                Cell::Text(2, "항목 1"),
                Cell::Text(3, "계열 1"),
                Cell::Num(4, 4.3),
            ],
        );
        let grid = scan_legacy_grid(&bytes).expect("scan");
        assert_eq!((grid.data_rows(), grid.data_cols()), (1, 1));
        assert_eq!(grid.number(1, 1), Some(4.3));
    }
}
