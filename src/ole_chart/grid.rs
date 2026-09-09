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
const VALUE_MARKER: &[u8] = b"VtValue\0";

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
    /// 문서가 **선언한** 머리행·머리열 수. 1/1 로 못박지 않는다(#6922).
    pub label_rows: u16,
    pub label_cols: u16,
    /// `index` 오름차순. 코너 셀(index 1)은 없다.
    pub cells: Vec<GridCell>,
}

impl LegacyChartGrid {
    /// 머리행을 뺀 데이터 행 수.
    ///
    /// 문서가 선언한 `label_rows` 를 뺀다. `scan_legacy_grid` 는 `rows >= 2` 만
    /// 내보내지만 필드가 공개라 직접 구성한 값에서도 감산이 넘치지 않게 한다.
    pub fn data_rows(&self) -> usize {
        (self.rows as usize).saturating_sub(self.label_rows as usize)
    }

    /// 머리열을 뺀 데이터 열 수.
    pub fn data_cols(&self) -> usize {
        (self.cols as usize).saturating_sub(self.label_cols as usize)
    }

    /// 데이터 행 `index`(1-based) 가 놓인 격자 행.
    ///
    /// 머리행이 하나인 문서에서는 `index` 그대로다. 코퍼스의 레거시 그리드 74개가 전부
    /// 1/1 이지만, `data_rows()` 가 선언값을 쓰므로 서수도 같은 기준을 써야 어긋나지
    /// 않는다(#6922).
    pub fn data_row_at(&self, index: u16) -> u16 {
        self.label_rows.saturating_add(index).saturating_sub(1)
    }

    /// 데이터 열 `index`(1-based) 가 놓인 격자 열.
    pub fn data_col_at(&self, index: u16) -> u16 {
        self.label_cols.saturating_add(index).saturating_sub(1)
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
    PrologueMismatch { at: usize },
    /// 데이터 셀이 없는 치수다.
    EmptyGrid { rows: u16, cols: u16 },
    /// 수치 셀 개수가 `(rows - 1) * (cols - 1)` 과 다르다.
    NumberCellCountMismatch { found: usize, expected: usize },
}

impl GridScanError {
    /// 기존 `OleChartParseError::UnsupportedContentsLayout` 의 `reason` 어휘로 옮긴다.
    pub(crate) fn reason(&self) -> &'static str {
        match self {
            Self::MarkerNotFound => "legacy HWP chart data grid marker not found",
            Self::PrologueMismatch { .. } => "legacy HWP chart data grid prologue not recognized",
            Self::EmptyGrid { .. } => "legacy HWP chart data grid is empty",
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
    // `<i32 typeId><u16 nameLen>"VtDataGrid\0"` — 이름 앞 6바이트가 선언의 시작이다.
    let at = marker
        .checked_sub(6)
        .ok_or(GridScanError::PrologueMismatch { at: marker })?;

    let mut reader = ArchiveReader::new(contents, at);
    let name = reader.read_type()?;
    if name != GRID_MARKER {
        return Err(GridScanError::PrologueMismatch { at });
    }
    reader.read_from(GRID_MARKER)?;

    let rows = reader.rows;
    let cols = reader.cols;
    if rows < 2 || cols < 2 {
        return Err(GridScanError::EmptyGrid { rows, cols });
    }
    let (label_rows, label_cols, data_cols, data_rows) = reader
        .counts
        .ok_or(GridScanError::PrologueMismatch { at })?;

    // 선언 치수가 슬롯 치수와 맞아떨어져야 모양을 신뢰할 수 있다.
    if usize::from(label_rows) + usize::from(data_rows) != usize::from(rows)
        || usize::from(label_cols) + usize::from(data_cols) != usize::from(cols)
    {
        return Err(GridScanError::NumberCellCountMismatch {
            found: usize::from(data_rows) * usize::from(data_cols),
            expected: (usize::from(rows) - 1) * (usize::from(cols) - 1),
        });
    }

    let cells = reader.cells;
    // 수치는 머리행·머리열에 오지 않는다. 오면 모양을 잘못 읽은 것이다. 데이터 칸이
    // **비어 있는 것**은 정상이므로(코퍼스 74개 중 5개) 개수 일치는 요구하지 않는다.
    let numbers = cells
        .iter()
        .filter(|cell| matches!(cell.value, GridValue::Number { .. }))
        .count();
    let misplaced = cells
        .iter()
        .filter(|cell| matches!(cell.value, GridValue::Number { .. }))
        .filter(|cell| cell.row < label_rows || cell.col < label_cols)
        .count();
    let capacity = usize::from(data_rows) * usize::from(data_cols);
    if misplaced > 0 || numbers > capacity {
        return Err(GridScanError::NumberCellCountMismatch {
            found: numbers,
            expected: capacity,
        });
    }

    Ok(LegacyChartGrid {
        window,
        rows,
        cols,
        label_rows,
        label_cols,
        cells,
    })
}

/// 판독기가 아는 클래스. 목록에 없는 이름을 만나면 모양을 신뢰할 수 없다.
const KNOWN_CLASSES: &[&[u8]] = &[
    GRID_MARKER,
    MATRIX_MARKER,
    COLLECTION_MARKER,
    OBJECT_MARKER,
    DOUBLE_MARKER,
    STRING_MARKER,
    VALUE_MARKER,
];

/// `VtArchive` 순차 판독기 (#6922).
///
/// `Int` 는 2바이트, `Long` 은 4바이트다. 아래 문법은 코퍼스 10,000건의 레거시 그리드
/// **74개 전수**에서 성립한다(실패 0).
///
/// ```text
/// ReadObject : i32 objectId   (-1 = NULL 슬롯 · 이미 본 id = 역참조만)
///              (처음) ReadType + ReadFrom
/// ReadType   : i32 typeId     (처음 보는 타입이면 <u16 len><name><u16 ver>)
///
/// VtDataGrid : base VtMatrix ; i16 ×4 (columnLabel, rowLabel, dataColumn, dataRow)
/// VtMatrix   : base VtCollection ; i16 rowCount ; i16 columnCount ; data[0..rows*cols]
/// VtCollection: i16 m_count ; base VtObject
/// VtDouble   : f64 value ; i16 precision ; base VtValue
/// VtString   : i16 len ; (len>0) len+1 바이트(NUL 포함) ; base VtValue
/// VtValue    : base VtObject
/// VtObject   : 없음
/// ```
///
/// 종전 스캐너는 셀 위치를 **아카이브 객체 id** 로 잡아, id 가 셀 밖 객체와 공유되는
/// 문서에서 절반이 머리행·머리열로 밀렸다(148735526: id 최대 264 · 수치 204개 중 74개
/// 오배치). 슬롯 순서로 걷는 것이 맞다.
struct ArchiveReader<'a> {
    bytes: &'a [u8],
    at: usize,
    types: Vec<(i32, &'static [u8])>,
    objects: Vec<i32>,
    rows: u16,
    cols: u16,
    cells: Vec<GridCell>,
    counts: Option<(u16, u16, u16, u16)>,
}

impl<'a> ArchiveReader<'a> {
    fn new(bytes: &'a [u8], at: usize) -> Self {
        Self {
            bytes,
            at,
            types: Vec::new(),
            objects: Vec::new(),
            rows: 0,
            cols: 0,
            cells: Vec::new(),
            counts: None,
        }
    }

    fn fail(&self) -> GridScanError {
        GridScanError::PrologueMismatch { at: self.at }
    }

    fn u16(&mut self) -> Result<u16, GridScanError> {
        let value = read_u16(self.bytes, self.at).ok_or_else(|| self.fail())?;
        self.at += 2;
        Ok(value)
    }

    fn i16(&mut self) -> Result<i16, GridScanError> {
        Ok(self.u16()? as i16)
    }

    fn i32(&mut self) -> Result<i32, GridScanError> {
        let value = read_u32(self.bytes, self.at).ok_or_else(|| self.fail())?;
        self.at += 4;
        Ok(value as i32)
    }

    fn f64(&mut self) -> Result<(f64, usize), GridScanError> {
        let raw = self
            .bytes
            .get(self.at..self.at + 8)
            .ok_or_else(|| self.fail())?;
        let offset = self.at;
        self.at += 8;
        Ok((f64::from_le_bytes(raw.try_into().expect("8바이트")), offset))
    }

    fn read_type(&mut self) -> Result<&'static [u8], GridScanError> {
        let type_id = self.i32()?;
        if let Some((_, name)) = self.types.iter().find(|(id, _)| *id == type_id) {
            return Ok(name);
        }
        let len = usize::from(self.u16()?);
        let raw = self
            .bytes
            .get(self.at..self.at + len)
            .ok_or_else(|| self.fail())?;
        let name = KNOWN_CLASSES
            .iter()
            .copied()
            .find(|known| *known == raw)
            .ok_or_else(|| self.fail())?;
        self.at += len + 2; // 이름 + <u16 version>
        self.types.push((type_id, name));
        Ok(name)
    }

    /// 기반 클래스 한 단계.
    fn read_base(&mut self) -> Result<(), GridScanError> {
        let name = self.read_type()?;
        self.read_from(name)?;
        Ok(())
    }

    fn read_from(&mut self, name: &[u8]) -> Result<Option<GridValue>, GridScanError> {
        if name == GRID_MARKER {
            self.read_base()?; // VtMatrix — rows/cols/data 를 그 안에서 읽는다
            self.counts = Some((self.u16()?, self.u16()?, self.u16()?, self.u16()?));
            return Ok(None);
        }
        if name == MATRIX_MARKER {
            self.read_base()?; // VtCollection
            self.rows = self.u16()?;
            self.cols = self.u16()?;
            let cols = usize::from(self.cols);
            let slots = usize::from(self.rows).saturating_mul(cols);
            for slot in 0..slots {
                if let Some(value) = self.read_object()? {
                    self.cells.push(GridCell {
                        index: slot as u32 + 1,
                        row: (slot / cols) as u16,
                        col: (slot % cols) as u16,
                        value,
                    });
                }
            }
            return Ok(None);
        }
        if name == COLLECTION_MARKER {
            let _count = self.i16()?;
            self.read_base()?; // VtObject
            return Ok(None);
        }
        if name == DOUBLE_MARKER {
            let (value, offset) = self.f64()?;
            let _precision = self.i16()?;
            self.read_base()?; // VtValue
            return Ok(Some(GridValue::Number { value, offset }));
        }
        if name == STRING_MARKER {
            let len = self.i16()?;
            let mut record = self.at - 2..self.at;
            let mut text = String::new();
            if len > 0 {
                let len = usize::from(len as u16);
                let payload = self
                    .bytes
                    .get(self.at..self.at + len)
                    .ok_or_else(|| self.fail())?;
                text = decode_cell_text(payload).unwrap_or_default();
                // `record` 는 `<u16 len> + payload` 다. 저장은 그 뒤 NUL 까지 `length + 1`
                // 바이트를 읽으므로 **커서만** 한 바이트 더 나아간다.
                record = record.start..self.at + len;
                self.at += len + 1;
            }
            self.read_base()?; // VtValue
            return Ok(Some(GridValue::Text { text, record }));
        }
        if name == VALUE_MARKER {
            self.read_base()?; // VtObject
            return Ok(None);
        }
        if name == OBJECT_MARKER {
            return Ok(None);
        }
        Err(self.fail())
    }

    fn read_object(&mut self) -> Result<Option<GridValue>, GridScanError> {
        let object_id = self.i32()?;
        if object_id == -1 {
            return Ok(None);
        }
        if self.objects.contains(&object_id) {
            // 역참조 — 뒤에 아무것도 오지 않는다.
            return Ok(None);
        }
        self.objects.push(object_id);
        let name = self.read_type()?;
        self.read_from(name)
    }
}

/// 셀 문자열 페이로드 — `cp949 \0\0 utf16le \0\0`.
///
/// **UTF-16 절반을 정본으로 읽는다.** cp949 절반은 ASCII 라벨(`0.7`)에서 길이가 홀수라
/// 짝수 정렬을 가정하면 안 되고, 확장 문자에서 EUC-KR 왕복이 손실될 수 있다. UTF-16
/// 절반이 없는 작성기를 위해 cp949 로 폴백한다.
fn decode_cell_text(payload: &[u8]) -> Option<String> {
    let Some(split) = payload.windows(2).position(|pair| pair == [0, 0]) else {
        // `\0\0` 경계가 없는 작성기 — 페이로드 전체가 cp949 한 벌이다(#6922).
        let (decoded, _, had_errors) = EUC_KR.decode(payload);
        if had_errors {
            return None;
        }
        let text = decoded.replace('\u{3000}', " ").trim().to_string();
        if text.is_empty() || text.chars().any(char::is_control) {
            return None;
        }
        return Some(text);
    };
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

    /// 셀 하나의 합성 명세. `index` 는 **1-based 행우선 슬롯 서수**다.
    pub(crate) enum Cell {
        Num(u32, f64),
        Text(u32, &'static str),
    }

    impl Cell {
        fn index(&self) -> u32 {
            match self {
                Cell::Num(index, _) | Cell::Text(index, _) => *index,
            }
        }
    }

    /// 머리행·머리열 1/1 짜리 `VtDataGrid` 를 합성한다.
    pub(crate) fn synth_grid(rows: u16, cols: u16, cells: &[Cell]) -> Vec<u8> {
        synth_grid_labeled(rows, cols, 1, 1, cells)
    }

    /// 실측 문법 그대로 `VtDataGrid` 를 합성한다.
    ///
    /// 코퍼스가 훑지 못하는 경로(0·음수·거대값, 창 밖 값, 어긋난 선언)를 픽스처 없이
    /// 재현하기 위한 것이다. **슬롯 순서로** 적고, 명세에 없는 슬롯에는 `-1`(NULL)을
    /// 넣는다.
    pub(crate) fn synth_grid_labeled(
        rows: u16,
        cols: u16,
        label_rows: u16,
        label_cols: u16,
        cells: &[Cell],
    ) -> Vec<u8> {
        /// 판독기가 되읽을 수 있는 바이트만 내는 기록기.
        struct Writer {
            out: Vec<u8>,
            types: Vec<&'static [u8]>,
            next_object: i32,
        }

        impl Writer {
            fn i16(&mut self, value: i16) {
                self.out.extend_from_slice(&value.to_le_bytes());
            }

            fn i32(&mut self, value: i32) {
                self.out.extend_from_slice(&value.to_le_bytes());
            }

            /// `ReadType` — 처음 보는 타입에만 이름을 붙인다.
            fn ty(&mut self, name: &'static [u8]) {
                if let Some(at) = self.types.iter().position(|known| *known == name) {
                    self.i32(at as i32 + 2);
                    return;
                }
                self.types.push(name);
                let type_id = self.types.len() as i32 + 1;
                self.i32(type_id);
                self.out
                    .extend_from_slice(&(name.len() as u16).to_le_bytes());
                self.out.extend_from_slice(name);
                self.out.extend_from_slice(&1u16.to_le_bytes()); // version
            }

            /// 새 객체 하나를 연다 — 역참조가 되지 않게 매번 새 id 를 쓴다.
            fn object(&mut self) {
                let id = self.next_object;
                self.next_object += 1;
                self.i32(id);
            }

            /// `VtValue` → `VtObject` 기반 사슬.
            fn value_base(&mut self) {
                self.ty(VALUE_MARKER);
                self.ty(OBJECT_MARKER);
            }
        }

        let mut w = Writer {
            out: vec![0u8; 16],
            types: Vec::new(),
            next_object: 1_000,
        };

        w.ty(GRID_MARKER); // 판독기는 이 선언의 6바이트 앞에서 시작한다
        w.ty(MATRIX_MARKER); // VtDataGrid 의 기반
        w.ty(COLLECTION_MARKER); // VtMatrix 의 기반
        w.i16(cells.len() as i16); // VtCollection::m_count
        w.ty(OBJECT_MARKER); // VtCollection 의 기반
        w.i16(rows as i16);
        w.i16(cols as i16);

        // 손상 스트림이 거대 치수를 주장하는 경우를 위해 실제로 적는 슬롯을 제한한다.
        // 그러면 스트림이 선언보다 짧아져, 판독기가 예약 없이 거부하는지 잴 수 있다.
        let slots = (rows as u32 * cols as u32).min(4_096);
        for slot in 0..slots {
            match cells.iter().find(|cell| cell.index() == slot + 1) {
                None => w.i32(-1),
                Some(Cell::Num(_, value)) => {
                    w.object();
                    w.ty(DOUBLE_MARKER);
                    w.out.extend_from_slice(&value.to_le_bytes());
                    w.i16(-1); // precision
                    w.value_base();
                }
                Some(Cell::Text(_, text)) => {
                    w.object();
                    w.ty(STRING_MARKER);
                    let (cp949, _, _) = EUC_KR.encode(text);
                    let mut payload = cp949.into_owned();
                    payload.extend_from_slice(&[0, 0]);
                    for unit in text.encode_utf16() {
                        payload.extend_from_slice(&unit.to_le_bytes());
                    }
                    payload.extend_from_slice(&[0, 0]);
                    w.i16(payload.len() as i16);
                    w.out.extend_from_slice(&payload);
                    w.out.push(0); // 저장은 NUL 까지 length + 1 바이트를 읽는다
                    w.value_base();
                }
            }
        }

        // `VtDataGrid` 자신의 꼬리 — 셀 payload **뒤에** 선언된다.
        w.i16(label_rows as i16);
        w.i16(label_cols as i16);
        w.i16(cols.saturating_sub(label_cols) as i16);
        w.i16(rows.saturating_sub(label_rows) as i16);

        w.out.extend_from_slice(b"VtPlot\0");
        w.out
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
    fn slot_order_places_cells_even_when_object_ids_are_far_apart() {
        // **[#6922] 뒤집힘 자물쇠.** 종전 스캐너는 셀 위치를 아카이브 객체 id 로 잡아,
        // id 가 셀 밖 개체와 번호를 나눠 쓰는 문서에서 셀이 통째로 밀렸다. 좌표는
        // 슬롯 서수에서만 나와야 하므로, 객체 id 를 아무리 띄워도 결과가 같아야 한다.
        let dense = control_like_grid();
        let mut sparse = dense.clone();
        // 첫 객체 id(1000)를 훨씬 큰 값으로 바꾼다 — 좌표에 영향이 없어야 한다.
        let at = find_from(&sparse, &1_000i32.to_le_bytes(), 0).expect("첫 객체 id");
        sparse[at..at + 4].copy_from_slice(&900_000i32.to_le_bytes());

        let grid = scan_legacy_grid(&sparse).expect("scan");
        assert_eq!(
            grid.cells,
            scan_legacy_grid(&dense).expect("scan").cells,
            "객체 id 는 셀 좌표가 아니다"
        );
    }

    #[test]
    fn declared_label_counts_are_honoured() {
        // **[#6922]** 머리행·머리열은 1/1 로 못박힌 값이 아니라 문서가 선언한다.
        let bytes = synth_grid_labeled(
            4,
            3,
            2,
            1,
            &[
                Cell::Text(2, "머리 1"),
                Cell::Text(3, "머리 2"),
                Cell::Text(5, "머리 3"),
                Cell::Text(6, "머리 4"),
                Cell::Text(7, "계열 1"),
                Cell::Num(8, 1.0),
                Cell::Num(9, 2.0),
                Cell::Text(10, "계열 2"),
                Cell::Num(11, 3.0),
                Cell::Num(12, 4.0),
            ],
        );
        let grid = scan_legacy_grid(&bytes).expect("scan");
        assert_eq!((grid.label_rows, grid.label_cols), (2, 1));
        assert_eq!((grid.data_rows(), grid.data_cols()), (2, 2));
        assert_eq!(grid.number(2, 1), Some(1.0));
        assert_eq!(grid.number(3, 2), Some(4.0));
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
    fn cp949_only_labels_without_a_utf16_half_are_decoded() {
        // **[#6922]** `\0\0` 경계 없이 cp949 한 벌만 싣는 작성기가 있다(148759031).
        // 종전에는 이 라벨 15칸이 통째로 버려졌다.
        assert_eq!(decode_cell_text(b"4\xbf\xf9"), Some("4월".to_string()));
        assert_eq!(
            decode_cell_text(b"'12\xb3\xe2 3\xbf\xf9"),
            Some("'12년 3월".to_string())
        );
    }

    #[test]
    fn values_outside_the_window_are_ignored() {
        let mut bytes = control_like_grid();
        // 창을 닫는 `VtPlot` 뒤에 축 눈금처럼 보이는 값을 심는다.
        bytes.extend_from_slice(&9999.0f64.to_le_bytes());
        bytes.extend_from_slice(&[0xFF, 0xFF, 0x06, 0x00, 0x00, 0x00]);

        let grid = scan_legacy_grid(&bytes).expect("scan");
        assert_eq!(
            grid.value_offsets().count(),
            12,
            "창 밖 값을 주워 오면 안 된다"
        );
    }

    #[test]
    fn empty_data_cells_are_accepted() {
        // **[#6922]** 데이터 칸이 비어 있는 것은 정상이다. 코퍼스의 레거시 그리드 74개
        // 중 5개가 그렇고, 종전 스캐너는 이것을 `NumberCellCountMismatch` 로 거부했다.
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
                // (2,1) 이 비었다.
                Cell::Num(9, 3.0),
            ],
        );
        let grid = scan_legacy_grid(&bytes).expect("빈 데이터 칸은 결함이 아니다");
        assert_eq!(grid.number(2, 1), None);
        assert_eq!(grid.number(2, 2), Some(3.0));
    }

    #[test]
    fn numbers_in_the_label_band_are_rejected() {
        // **음성 대조** — 수치가 머리행·머리열에 오면 모양을 잘못 읽은 것이다.
        let bytes = synth_grid(
            2,
            2,
            &[
                Cell::Num(2, 1.0), // 머리행에 수치
                Cell::Text(3, "계열 1"),
                Cell::Num(4, 2.0),
            ],
        );
        assert!(matches!(
            scan_legacy_grid(&bytes),
            Err(GridScanError::NumberCellCountMismatch { .. })
        ));
    }

    #[test]
    fn declared_counts_that_disagree_with_the_dimensions_are_rejected() {
        // **음성 대조** — 선언 치수(`label + data`)가 슬롯 치수와 어긋나면 거부한다.
        let mut bytes = synth_grid(3, 3, &[Cell::Text(2, "항목 1"), Cell::Num(5, 1.0)]);
        // 꼬리 네 값 중 `dataRow` 를 부풀린다.
        let at = bytes.len() - b"VtPlot\0".len() - 2;
        bytes[at..at + 2].copy_from_slice(&9i16.to_le_bytes());
        assert!(matches!(
            scan_legacy_grid(&bytes),
            Err(GridScanError::NumberCellCountMismatch { .. })
        ));
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
        // 예약하지 않고 바이트가 떨어지는 자리에서 거부해야 한다.
        let bytes = synth_grid(u16::MAX, u16::MAX, &[]);
        assert!(matches!(
            scan_legacy_grid(&bytes),
            Err(GridScanError::PrologueMismatch { .. })
        ));
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
