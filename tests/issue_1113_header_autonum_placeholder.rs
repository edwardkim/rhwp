//! Issue #1113: HWPX 머리말 글상자 페이지번호(AutoNumber) 문단의 placeholder
//! 공백(U+0020) 제거 회귀 가드.
//!
//! 재현 문서: `samples/hwpx/exam_social.hwpx`.
//!
//! 배경: HWPX 머리말 글상자는 `<hp:autoNum>` + 빈 `<hp:t/>` 로 페이지번호만
//! 담는다. rhwp 파서는 HWP5 PARA_TEXT 조립 시 AutoNumber placeholder 공백
//! (U+0020) 을 합성한다. 어댑터의 `materialize_master_page_autonum_placeholder`
//! 가 이 공백을 제거하는데, 종전에는 바탕쪽(MasterPage) 에만 적용되어 머리말
//! 홀수쪽 글상자(폭 4252)에서 잉여 공백이 남았다. 한컴 에디터가 이 공백을
//! 페이지번호와 함께 줄나눔하여 글상자 높이가 비정상적으로 커졌다.
//!
//! 정답지(`samples/exam_social.hwp`)는 해당 페이지번호 문단을 AutoNumber-only
//! (PARA_HEADER char_count=9, PARA_TEXT size=18) 로 저장한다.

use rhwp::parser::cfb_reader::CfbReader;
use rhwp::parser::record::Record;
use rhwp::parser::tags::{
    CTRL_AUTO_NUMBER, CTRL_HEADER, HWPTAG_CTRL_HEADER, HWPTAG_PARA_HEADER, HWPTAG_PARA_TEXT,
};
use std::fs;
use std::path::Path;

// record payload 의 ctrl_id 4 byte 를 LE u32 로 읽으면 tags::ctrl_id (BE 정의) 와 일치.
// CTRL_HEADER head, CTRL_AUTO_NUMBER 를 그대로 사용.
// PARA_TEXT 안 AUTO_NUMBER 제어문자 마커 (HWP5: 0x0012, 8 utf16 폭 점유).
const PARA_TEXT_AUTONUM_MARKER: u16 = 0x0012;

fn save_hwp_section0(hwpx_rel: &str) -> Vec<Record> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(hwpx_rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", hwpx_rel, e));
    let mut doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse hwpx");
    let hwp = doc.export_hwp_with_adapter().expect("export hwp");

    let mut cfb = CfbReader::open(&hwp).expect("cfb open");
    let header = cfb.read_file_header().expect("filehdr");
    let compressed = (header[36] & 0x01) != 0;
    let data = cfb
        .read_body_text_section(0, compressed, false)
        .expect("section0");
    Record::read_all(&data).expect("records")
}

fn ctrl_id_le(payload: &[u8]) -> Option<u32> {
    (payload.len() >= 4)
        .then(|| u32::from_le_bytes([payload[0], payload[1], payload[2], payload[3]]))
}

/// CTRL_HEADER head 의 apply_type (payload byte[4..8]). HWP5: 1=Even, 2=Odd.
fn head_apply_type(payload: &[u8]) -> Option<u32> {
    (payload.len() >= 8)
        .then(|| u32::from_le_bytes([payload[4], payload[5], payload[6], payload[7]]))
}

/// 주어진 apply_type 의 head subtree 범위 [start, end) 를 찾는다.
fn head_subtree(records: &[Record], apply_type: u32) -> Option<(usize, usize)> {
    for (i, r) in records.iter().enumerate() {
        if r.tag_id == HWPTAG_CTRL_HEADER
            && ctrl_id_le(&r.data) == Some(CTRL_HEADER)
            && head_apply_type(&r.data) == Some(apply_type)
        {
            let lv = r.level;
            let mut j = i + 1;
            while j < records.len() && records[j].level > lv {
                j += 1;
            }
            return Some((i, j));
        }
    }
    None
}

/// PARA_TEXT 의 첫 u16 들에서 AUTO_NUMBER 마커(0x0012) 위치를 찾는다.
/// AutoNumber-only: [0x0012]... (첫 u16 == 마커)
/// placeholder 공백 잔존 시: [0x0020][0x0012]... (둘째 u16 == 마커)
fn para_text_autonum_marker_index(pt: &Record) -> Option<usize> {
    let u16s: Vec<u16> = pt
        .data
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    u16s.iter()
        .take(2)
        .position(|&v| v == PARA_TEXT_AUTONUM_MARKER)
}

/// subtree 안에서 페이지번호(AutoNumber) 문단의 (PARA_HEADER, PARA_TEXT) 를 찾는다.
/// 같은 subtree 에 CTRL_HEADER aton 이 있고, PARA_TEXT 가 0x0012 마커로 시작(또는
/// 잉여 공백 뒤)하는 문단.
fn find_autonum_para<'a>(
    records: &'a [Record],
    range: (usize, usize),
) -> Option<(&'a Record, &'a Record)> {
    let (s, e) = range;
    let ctrl_autonum_nearby = records[s..e]
        .iter()
        .any(|r| r.tag_id == HWPTAG_CTRL_HEADER && ctrl_id_le(&r.data) == Some(CTRL_AUTO_NUMBER));
    if !ctrl_autonum_nearby {
        return None;
    }
    for i in s..e.saturating_sub(1) {
        if records[i].tag_id == HWPTAG_PARA_HEADER
            && records[i + 1].tag_id == HWPTAG_PARA_TEXT
            && para_text_autonum_marker_index(&records[i + 1]).is_some()
        {
            return Some((&records[i], &records[i + 1]));
        }
    }
    None
}

#[test]
fn odd_header_autonum_paragraph_has_no_placeholder_space() {
    let records = save_hwp_section0("samples/hwpx/exam_social.hwpx");

    // 홀수쪽 = apply_type 2 (HWP5: 1=Even, 2=Odd).
    let odd = head_subtree(&records, 2).expect("odd head subtree (apply_type=2)");
    let (para_header, para_text) =
        find_autonum_para(&records, odd).expect("odd header autonum paragraph");

    // PARA_TEXT: AutoNumber-only (placeholder U+0020 없음).
    // 정답지 size=18: [0x0012][aton 4byte][zero...][0x0012][0x000d]
    // placeholder 공백 있으면 맨 앞 u16 == 0x0020 + size=20.
    let first_u16 = u16::from_le_bytes([para_text.data[0], para_text.data[1]]);
    assert_ne!(
        first_u16,
        0x0020,
        "홀수쪽 머리말 페이지번호 문단 PARA_TEXT 에 placeholder 공백(U+0020) 이 남아있음: {:02x?}",
        &para_text.data[..para_text.data.len().min(8)]
    );
    assert_eq!(
        para_text.size, 18,
        "PARA_TEXT size 가 정답지(18, AutoNumber-only) 와 다름. 공백 추가 시 20."
    );

    // PARA_HEADER char_count (첫 u16) == 9 (autonum 8 utf16 + 끝마커 1).
    let char_count = u16::from_le_bytes([para_header.data[0], para_header.data[1]]);
    assert_eq!(
        char_count, 9,
        "PARA_HEADER char_count 가 정답지(9) 와 다름. placeholder 공백 시 10."
    );
}
