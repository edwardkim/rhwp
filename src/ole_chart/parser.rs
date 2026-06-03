//! HWP OLE `Contents` 차트 스트림 최소 파서.
//!
//! 한컴 차트 사양(revision 1.2)의 `ChartOBJ` 기본 구조를 기준으로 legacy
//! HWP chart payload를 식별한다. 전체 object graph는 아직 해석하지 않고,
//! `VtDataGrid` 구간에서 라벨과 연속된 f64 값 배열만 추출한다.

use std::fmt;

use encoding_rs::EUC_KR;
use serde::Serialize;

/// OLE `Contents`에서 추출한 차트 IR.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OleChart {
    pub chart_type: OleChartType,
    pub title: Option<String>,
    pub categories: Vec<String>,
    pub series: Vec<OleChartSeries>,
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

fn parse_legacy_hwp_chart_contents(
    bytes: &[u8],
    probe: &OleChartContentsProbe,
) -> Result<OleChart, OleChartParseError> {
    let grid_marker_start = find_bytes(bytes, b"VtDataGrid\0").ok_or_else(|| {
        unsupported_legacy_layout(probe, "legacy HWP chart data grid marker not found")
    })?;
    let grid_start = legacy_chart_object_data_start(bytes, grid_marker_start, b"VtDataGrid\0");
    let grid_end = find_next_legacy_object_marker(bytes, grid_start).ok_or_else(|| {
        unsupported_legacy_layout(probe, "legacy HWP chart data grid boundary not found")
    })?;

    let labels = extract_string_labels(bytes, grid_start, grid_end);
    let mut series_names = Vec::new();
    let mut categories = Vec::new();
    let mut saw_category = false;

    for label in labels {
        if label.text.chars().any(|ch| ch.is_ascii_digit()) {
            saw_category = true;
            categories.push(label.text);
        } else if !saw_category {
            series_names.push(label.text);
        }
    }

    series_names.dedup();
    categories.dedup();

    if series_names.is_empty() || categories.is_empty() {
        return Err(unsupported_legacy_layout(
            probe,
            "legacy HWP chart data grid labels are incomplete",
        ));
    }

    let series_count = series_names.len();
    let expected_values = series_count * categories.len();
    let values = extract_grid_values(bytes, grid_start, grid_end, expected_values);
    if values.len() != expected_values {
        return Err(unsupported_legacy_layout(
            probe,
            "legacy HWP chart data grid shape not recognized",
        ));
    }

    let mut series = Vec::with_capacity(series_count);
    for (series_idx, name) in series_names.into_iter().enumerate() {
        let mut series_values = Vec::with_capacity(categories.len());
        for category_idx in 0..categories.len() {
            series_values.push(values[category_idx * series_count + series_idx]);
        }
        series.push(OleChartSeries {
            name: Some(name),
            values: series_values,
        });
    }

    Ok(OleChart {
        chart_type: OleChartType::Unknown,
        title: extract_chart_title(bytes),
        categories,
        series,
    })
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

fn extract_grid_values(bytes: &[u8], start: usize, end: usize, expected_count: usize) -> Vec<f64> {
    if expected_count == 0 || start >= end || end > bytes.len() {
        return Vec::new();
    }

    let mut best = Vec::new();
    let scan_end = end.saturating_sub(8);
    for offset in start..=scan_end {
        let value = read_f64(bytes, offset);
        if !is_plausible_grid_value(value) {
            continue;
        }

        let mut run = Vec::new();
        let mut cursor = offset;
        while cursor + 8 <= end {
            let value = read_f64(bytes, cursor);
            if !is_plausible_grid_value(value) {
                break;
            }
            run.push(value);
            cursor += 8;
        }

        if run.len() == expected_count {
            return run;
        }
        if run.len() > best.len() {
            best = run;
        }
    }

    if best.len() >= expected_count {
        best.truncate(expected_count);
        best
    } else {
        extract_sparse_grid_values(bytes, start, end, expected_count)
    }
}

fn extract_sparse_grid_values(
    bytes: &[u8],
    start: usize,
    end: usize,
    expected_count: usize,
) -> Vec<f64> {
    let mut values = Vec::new();
    let mut offset = start;
    while offset + 8 <= end {
        let value = read_f64(bytes, offset);
        if is_plausible_grid_value(value) {
            values.push(value);
            offset += 8;
            continue;
        }
        offset += 1;
    }

    if values.len() == expected_count {
        values
    } else {
        Vec::new()
    }
}

fn is_plausible_grid_value(value: f64) -> bool {
    value.is_finite()
        && value >= 1.0
        && value <= 1_000_000.0
        && (value.fract().abs() < 1e-9 || (1.0 - value.fract()).abs() < 1e-9)
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
    const MARKERS: &[&[u8]] = &[
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

    find_first_marker(bytes, start, MARKERS).map(|marker| marker.start)
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

fn read_f64(bytes: &[u8], offset: usize) -> f64 {
    f64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
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

    #[test]
    fn legacy_grid_values_require_dense_f64_run() {
        let mut bytes = vec![0u8; 96];
        bytes[4..12].copy_from_slice(&999.0f64.to_le_bytes());
        bytes[17..25].copy_from_slice(&1.0f64.to_le_bytes());
        bytes[25..33].copy_from_slice(&2.0f64.to_le_bytes());
        bytes[33..41].copy_from_slice(&3.0f64.to_le_bytes());

        let values = extract_grid_values(&bytes, 0, bytes.len(), 3);
        assert_eq!(values, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn legacy_grid_values_fall_back_to_exact_sparse_candidates() {
        let mut bytes = vec![0u8; 96];
        bytes[7..15].copy_from_slice(&1.0f64.to_le_bytes());
        bytes[29..37].copy_from_slice(&2.0f64.to_le_bytes());
        bytes[53..61].copy_from_slice(&3.0f64.to_le_bytes());

        let values = extract_grid_values(&bytes, 0, bytes.len(), 3);
        assert_eq!(values, [1.0, 2.0, 3.0]);

        let values = extract_grid_values(&bytes, 0, bytes.len(), 2);
        assert!(values.is_empty());
    }
}
