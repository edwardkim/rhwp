//! HWP OLE `Contents` 차트 스트림 최소 파서.
//!
//! 한컴 차트 사양(revision 1.2)의 `ChartOBJ` 기본 구조를 기준으로 legacy
//! HWP chart payload를 식별한다. 전체 object graph는 아직 해석하지 않고,
//! `VtDataGrid` 구간에서 라벨과 연속된 f64 값 배열만 추출한다.

use std::fmt;

use encoding_rs::EUC_KR;
use serde::Serialize;

use super::grid::{self, LegacyChartGrid};
use super::orientation::{self, SeriesAxis, SeriesAxisEvidence};

/// OLE `Contents`에서 추출한 차트 IR.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OleChart {
    pub chart_type: OleChartType,
    pub title: Option<String>,
    pub categories: Vec<String>,
    pub series: Vec<OleChartSeries>,
    /// 그리드의 어느 축이 계열이었는가(#4098).
    pub series_axis: SeriesAxis,
    /// [`Self::series_axis`] 를 무엇으로 정했는가.
    ///
    /// [`SeriesAxisEvidence::Inconclusive`] 는 **판정이 아니라 관례 폴백**이라는 선언이다.
    /// 소비자가 결정과 추정을 구별할 수 있어야 하므로 값과 함께 싣는다.
    pub series_axis_evidence: SeriesAxisEvidence,
}

/// OLE `Contents` 차트 종류.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OleChartType {
    Column,
    Bar,
    Line,
    Pie,
    #[default]
    Unknown,
}

/// OLE `Contents` 차트 시리즈.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OleChartSeries {
    pub name: Option<String>,
    pub values: Vec<f64>,
}

/// OLE `Contents` 스트림 진단 정보.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OleChartContentsProbe {
    pub len: usize,
    pub signature: [u8; 16],
    pub first_words_le: [u32; 4],
    pub has_cfb_magic: bool,
    pub has_ooxml_chart_marker: bool,
    pub legacy_chart_object_start: Option<usize>,
    pub has_vt_chart_marker: bool,
    pub has_vt_data_grid_marker: bool,
    pub has_vt_chart_title_marker: bool,
    pub likely_legacy_hwp_chart_contents: bool,
}

/// OLE `Contents` 차트 파싱 오류.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OleChartParseError {
    Empty,
    TooShort {
        len: usize,
    },
    UnsupportedContentsLayout {
        len: usize,
        signature: [u8; 16],
        reason: &'static str,
    },
}

impl OleChartParseError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Empty => "EMPTY_CONTENTS",
            Self::TooShort { .. } => "CONTENTS_TOO_SHORT",
            Self::UnsupportedContentsLayout { .. } => "UNSUPPORTED_CONTENTS_LAYOUT",
        }
    }

    pub fn stable_message(&self) -> String {
        match self {
            Self::Empty => "OLE 차트 미지원: Contents 스트림이 비어 있음".to_string(),
            Self::TooShort { len } => {
                format!("OLE 차트 미지원: Contents 스트림이 너무 짧음 (len={len})")
            }
            Self::UnsupportedContentsLayout { reason, .. } => {
                format!("OLE 차트 미지원: {reason}")
            }
        }
    }
}

impl fmt::Display for OleChartParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.stable_message())
    }
}

impl std::error::Error for OleChartParseError {}

/// OLE `Contents` 스트림을 차트 IR로 파싱한다.
pub fn parse_ole_chart_contents(bytes: &[u8]) -> Result<OleChart, OleChartParseError> {
    let probe = probe_ole_chart_contents(bytes)?;

    if probe.likely_legacy_hwp_chart_contents {
        return parse_legacy_hwp_chart_contents(bytes, &probe);
    }

    Err(OleChartParseError::UnsupportedContentsLayout {
        len: probe.len,
        signature: probe.signature,
        reason: "unknown OLE Contents layout",
    })
}

/// 레거시 그리드를 차트 IR 로 옮긴다.
///
/// 권위 순서는 **① `VtObject` 가 명시한 치수가 모양 → ② 셀 인덱스가 배치 → ③ 라벨은
/// 이름일 뿐** 이다(#4098). 예전에는 라벨 분류가 값 개수를 정했기 때문에 라벨을 잘못
/// 가르면 값 탐색까지 같이 틀렸다.
fn parse_legacy_hwp_chart_contents(
    bytes: &[u8],
    probe: &OleChartContentsProbe,
) -> Result<OleChart, OleChartParseError> {
    let grid = grid::scan_legacy_grid(bytes)
        .map_err(|error| unsupported_legacy_layout(probe, error.reason()))?;
    let (series_axis, series_axis_evidence) = orientation::decide_series_axis(bytes, &grid);

    let (series_count, category_count) = match series_axis {
        SeriesAxis::Rows => (grid.data_rows(), grid.data_cols()),
        SeriesAxis::Columns => (grid.data_cols(), grid.data_rows()),
    };

    Ok(OleChart {
        chart_type: OleChartType::Unknown,
        title: extract_chart_title(bytes),
        categories: (1..=category_count)
            .map(|index| grid_category_label(&grid, series_axis, index))
            .collect(),
        series: (1..=series_count)
            .map(|index| OleChartSeries {
                name: grid_series_name(&grid, series_axis, index),
                values: (1..=category_count)
                    .map(|category| grid_value(&grid, series_axis, index, category))
                    .collect(),
            })
            .collect(),
        series_axis,
        series_axis_evidence,
    })
}

/// 계열 이름. 라벨 셀이 비어 있어도 파싱을 실패시키지 않는다 — 이름이 없는 것과 모양을
/// 모르는 것은 다르다.
fn grid_series_name(grid: &LegacyChartGrid, axis: SeriesAxis, index: usize) -> Option<String> {
    let label = match axis {
        SeriesAxis::Rows => grid.row_label(index as u16),
        SeriesAxis::Columns => grid.column_label(index as u16),
    };
    label.map(str::to_string)
}

/// 카테고리 이름. 라벨이 없으면 1-based 서수로 채워 `categories.len()` 이 데이터 치수와
/// 어긋나지 않게 한다 — 렌더러가 카테고리 개수로 축을 잡는다.
fn grid_category_label(grid: &LegacyChartGrid, axis: SeriesAxis, index: usize) -> String {
    let label = match axis {
        SeriesAxis::Rows => grid.column_label(index as u16),
        SeriesAxis::Columns => grid.row_label(index as u16),
    };
    label
        .map(str::to_string)
        .unwrap_or_else(|| index.to_string())
}

/// 계열 `series` 의 카테고리 `category` 값.
///
/// `scan_legacy_grid` 가 데이터 칸과 수치 셀의 일대일 대응을 이미 보장하므로 `None` 은
/// 도달하지 않는다.
fn grid_value(grid: &LegacyChartGrid, axis: SeriesAxis, series: usize, category: usize) -> f64 {
    let (row, col) = match axis {
        SeriesAxis::Rows => (series as u16, category as u16),
        SeriesAxis::Columns => (category as u16, series as u16),
    };
    grid.number(row, col).unwrap_or(0.0)
}

fn unsupported_legacy_layout(
    probe: &OleChartContentsProbe,
    reason: &'static str,
) -> OleChartParseError {
    OleChartParseError::UnsupportedContentsLayout {
        len: probe.len,
        signature: probe.signature,
        reason,
    }
}

/// OLE `Contents` 스트림의 안정적인 진단 정보를 만든다.
pub fn probe_ole_chart_contents(bytes: &[u8]) -> Result<OleChartContentsProbe, OleChartParseError> {
    if bytes.is_empty() {
        return Err(OleChartParseError::Empty);
    }
    if bytes.len() < 16 {
        return Err(OleChartParseError::TooShort { len: bytes.len() });
    }

    let mut signature = [0u8; 16];
    signature.copy_from_slice(&bytes[..16]);

    let mut first_words_le = [0u32; 4];
    for (i, chunk) in bytes[..16].chunks_exact(4).enumerate() {
        first_words_le[i] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
    }

    let has_cfb_magic = bytes.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
    let has_ooxml_chart_marker = bytes
        .windows(b"chartSpace".len())
        .any(|w| w == b"chartSpace");
    let legacy_chart_object_start = if first_words_le[0] == 0x0001_0000
        && first_words_le[1] == first_words_le[2]
        && first_words_le[3] >= 0x20
        && (first_words_le[3] as usize) < bytes.len()
    {
        Some(first_words_le[3] as usize)
    } else {
        None
    };
    let has_vt_chart_marker = find_bytes(bytes, b"VtChart\0").is_some();
    let has_vt_data_grid_marker = find_bytes(bytes, b"VtDataGrid\0").is_some();
    let has_vt_chart_title_marker = find_title_marker(bytes, 0).map(|_| ()).is_some();
    let likely_legacy_hwp_chart_contents =
        legacy_chart_object_start.is_some() && has_vt_data_grid_marker;

    Ok(OleChartContentsProbe {
        len: bytes.len(),
        signature,
        first_words_le,
        has_cfb_magic,
        has_ooxml_chart_marker,
        legacy_chart_object_start,
        has_vt_chart_marker,
        has_vt_data_grid_marker,
        has_vt_chart_title_marker,
        likely_legacy_hwp_chart_contents,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LabelCandidate {
    text: String,
    end: usize,
}

fn extract_string_labels(bytes: &[u8], start: usize, end: usize) -> Vec<LabelCandidate> {
    let mut labels = Vec::new();
    let mut offset = start;
    while offset + 4 <= end {
        let len = read_u16(bytes, offset) as usize;
        if (2..=128).contains(&len) && offset + 2 + len <= end {
            if let Some(text) = decode_legacy_text_payload(&bytes[offset + 2..offset + 2 + len]) {
                if !labels
                    .iter()
                    .any(|label: &LabelCandidate| label.text == text)
                {
                    labels.push(LabelCandidate {
                        text,
                        end: offset + 2 + len,
                    });
                }
                offset += 2 + len;
                continue;
            }
        }
        offset += 1;
    }
    labels
}

fn decode_legacy_text_payload(payload: &[u8]) -> Option<String> {
    let text_end = payload.windows(2).position(|w| w == [0, 0])?;
    if text_end < 2 {
        return None;
    }

    let (text, _, had_errors) = EUC_KR.decode(&payload[..text_end]);
    if had_errors {
        return None;
    }

    let normalized = text.replace('\u{3000}', " ");
    let text = normalized.trim().to_string();
    if !is_plausible_chart_label(&text) {
        return None;
    }

    Some(text)
}

fn is_plausible_chart_label(text: &str) -> bool {
    if text.is_empty() || text.starts_with("Vt") || text.len() > 64 {
        return false;
    }

    let mut has_data_char = false;
    for ch in text.chars() {
        if ch.is_control() {
            return false;
        }
        if ch.is_ascii_digit() || ('가'..='힣').contains(&ch) {
            has_data_char = true;
            continue;
        }
        if matches!(ch, ' ' | ',' | '.' | '-' | '_' | '(' | ')' | '/' | '%') {
            continue;
        }
        if ch.is_ascii_alphabetic() {
            continue;
        }
        return false;
    }

    has_data_char
}

fn extract_chart_title(bytes: &[u8]) -> Option<String> {
    let title_marker = find_title_marker(bytes, 0)?;
    let title_start =
        legacy_chart_object_data_start(bytes, title_marker.start, title_marker.marker);
    let title_end = find_next_legacy_object_marker(bytes, title_start)
        .or_else(|| find_bytes_from(bytes, b"VtList\0", title_start))
        .unwrap_or(bytes.len());
    extract_string_labels(bytes, title_start, title_end)
        .into_iter()
        .max_by_key(|label| label.text.chars().count())
        .map(|label| label.text)
}

struct MarkerMatch {
    start: usize,
    marker: &'static [u8],
}

fn legacy_chart_object_data_start(bytes: &[u8], marker_start: usize, marker: &[u8]) -> usize {
    // 한컴 차트 사양의 ChartOBJ는 StoredName(char*) 다음 StoredVersion(int)을 둔다.
    // StoredName은 marker 문자열의 NUL까지 포함하므로, 최초 선언 객체에서는 4바이트
    // version 뒤부터 ChartObjData가 시작된다.
    let data_start = marker_start.saturating_add(marker.len()).saturating_add(4);
    if data_start <= bytes.len() {
        data_start
    } else {
        marker_start
    }
}

fn find_title_marker(bytes: &[u8], start: usize) -> Option<MarkerMatch> {
    find_first_marker(bytes, start, &[b"VtChartTitle\0", b"VtTitle\0"])
}

fn find_next_legacy_object_marker(bytes: &[u8], start: usize) -> Option<usize> {
    // 마커 목록의 정본은 `grid` 다(#4098 중복 제거).
    grid::next_object_marker(bytes, start)
}

fn find_first_marker(bytes: &[u8], start: usize, markers: &[&'static [u8]]) -> Option<MarkerMatch> {
    markers
        .iter()
        .filter_map(|marker| {
            find_bytes_from(bytes, marker, start).map(|pos| MarkerMatch { start: pos, marker })
        })
        .min_by_key(|marker| marker.start)
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    find_bytes_from(haystack, needle, 0)
}

fn find_bytes_from(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    if needle.is_empty() || start >= haystack.len() || needle.len() > haystack.len() {
        return None;
    }
    haystack[start..]
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| start + offset)
}

fn read_u16(bytes: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([bytes[offset], bytes[offset + 1]])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_rejects_empty_contents() {
        let err = probe_ole_chart_contents(&[]).expect_err("empty contents should fail");
        assert_eq!(err.code(), "EMPTY_CONTENTS");
    }

    #[test]
    fn probe_rejects_too_short_contents() {
        let err = probe_ole_chart_contents(&[0u8; 8]).expect_err("short contents should fail");
        assert_eq!(err.code(), "CONTENTS_TOO_SHORT");
    }

    #[test]
    fn probe_detects_legacy_hwp_chart_contents_shape() {
        let mut bytes = vec![0u8; 0x80];
        bytes[0..4].copy_from_slice(&0x0001_0000u32.to_le_bytes());
        bytes[4..8].copy_from_slice(&0x0000_0020u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&0x0000_0020u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&0x0000_0060u32.to_le_bytes());

        let probe = probe_ole_chart_contents(&bytes).expect("probe");
        assert!(!probe.likely_legacy_hwp_chart_contents);

        let err =
            parse_ole_chart_contents(&bytes).expect_err("minimal legacy bytes should fail stable");
        assert_eq!(err.code(), "UNSUPPORTED_CONTENTS_LAYOUT");
    }

    #[test]
    fn probe_detects_legacy_hwp_chart_object_markers() {
        let mut bytes = vec![0u8; 0x100];
        bytes[0..4].copy_from_slice(&0x0001_0000u32.to_le_bytes());
        bytes[4..8].copy_from_slice(&0x0000_0020u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&0x0000_0020u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&0x0000_0060u32.to_le_bytes());
        bytes[0x60..0x68].copy_from_slice(b"VtChart\0");
        bytes[0x80..0x8b].copy_from_slice(b"VtDataGrid\0");

        let probe = probe_ole_chart_contents(&bytes).expect("probe");
        assert_eq!(probe.legacy_chart_object_start, Some(0x60));
        assert!(probe.has_vt_chart_marker);
        assert!(probe.has_vt_data_grid_marker);
        assert!(probe.likely_legacy_hwp_chart_contents);
    }

    #[test]
    fn legacy_text_payload_decodes_cp949_until_null_pair() {
        let payload = [
            0xC0, 0xFB, 0xB8, 0xB3, 0xB1, 0xDD, 0x00, 0x00, 0x01, 0xC8, 0xBD, 0xB9,
        ];

        let text = decode_legacy_text_payload(&payload).expect("decode payload");
        assert_eq!(text, "적립금");
    }

    #[test]
    fn legacy_text_payload_normalizes_full_width_spaces() {
        let payload = [
            0xBF, 0xAC, 0xB1, 0xDD, 0xA1, 0xA1, 0xC0, 0xE7, 0xC1, 0xA4, 0xA1, 0xA1, 0xC0, 0xFC,
            0xB8, 0xC1, 0x00, 0x00,
        ];

        let text = decode_legacy_text_payload(&payload).expect("decode payload");
        assert_eq!(text, "연금 재정 전망");
    }

    /// 값 필터가 사라졌으므로 소수·0·음수가 그대로 살아 나온다(#4098 결함 1).
    ///
    /// 예전 `is_plausible_grid_value` 는 이 중 어느 것도 통과시키지 못했고, 값을 거르는
    /// 데 그치지 않고 연속 런을 끊어 **파싱 전체**를 실패시켰다.
    #[test]
    fn legacy_grid_reads_values_the_old_filter_rejected() {
        use crate::ole_chart::grid::tests::{synth_grid, Cell};

        let mut bytes = vec![0u8; 0x60];
        bytes[0..4].copy_from_slice(&0x0001_0000u32.to_le_bytes());
        bytes[4..8].copy_from_slice(&0x0000_0020u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&0x0000_0020u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&0x0000_0060u32.to_le_bytes());
        bytes.extend_from_slice(&synth_grid(
            2,
            4,
            &[
                Cell::Text(2, "봄"),
                Cell::Text(3, "여름"),
                Cell::Text(4, "가을"),
                Cell::Text(5, "판매"),
                Cell::Num(6, 4.3),
                Cell::Num(7, 0.0),
                Cell::Num(8, -2.5),
            ],
        ));

        let chart = parse_ole_chart_contents(&bytes).expect("parse");
        assert_eq!(chart.categories, ["봄", "여름", "가을"]);
        assert_eq!(chart.series.len(), 1);
        assert_eq!(chart.series[0].name.as_deref(), Some("판매"));
        assert_eq!(chart.series[0].values, [4.3, 0.0, -2.5]);
    }

    /// 라벨에 숫자가 섞여 있어도 계열·카테고리가 뒤바뀌지 않는다(#4098 결함 3).
    ///
    /// 예전 digit 휴리스틱은 `항목 1`·`계열 1` 을 전부 카테고리로 몰아 계열을 0개로
    /// 만들었고, 코퍼스 28종이 전건 이 경로로 실패했다.
    #[test]
    fn numeric_labels_do_not_collapse_series_and_categories() {
        use crate::ole_chart::grid::tests::{synth_grid, Cell};

        let mut bytes = vec![0u8; 0x60];
        bytes[0..4].copy_from_slice(&0x0001_0000u32.to_le_bytes());
        bytes[4..8].copy_from_slice(&0x0000_0020u32.to_le_bytes());
        bytes[8..12].copy_from_slice(&0x0000_0020u32.to_le_bytes());
        bytes[12..16].copy_from_slice(&0x0000_0060u32.to_le_bytes());
        bytes.extend_from_slice(&synth_grid(
            3,
            3,
            &[
                Cell::Text(2, "항목 1"),
                Cell::Text(3, "항목 2"),
                Cell::Text(4, "계열 1"),
                Cell::Num(5, 1.5),
                Cell::Num(6, 2.5),
                Cell::Text(7, "계열 2"),
                Cell::Num(8, 3.5),
                Cell::Num(9, 4.5),
            ],
        ));

        let chart = parse_ole_chart_contents(&bytes).expect("parse");
        assert_eq!(chart.categories, ["항목 1", "항목 2"]);
        assert_eq!(chart.series.len(), 2);
        assert_eq!(chart.series[0].name.as_deref(), Some("계열 1"));
        assert_eq!(chart.series[0].values, [1.5, 2.5]);
        assert_eq!(chart.series[1].name.as_deref(), Some("계열 2"));
        assert_eq!(chart.series[1].values, [3.5, 4.5]);
    }
}
