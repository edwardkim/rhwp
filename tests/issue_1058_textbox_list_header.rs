//! Issue #1058: 글상자 (TextBox) LIST_HEADER 13 byte contract 정합 회귀 가드.
//!
//! 결함 본질: HWPX → HWP 저장 시 글상자 LIST_HEADER 가 마지막 13 byte (zero 8 +
//! editableAtFormMode 4 + fieldName flag 1) 누락 → 한컴편집기가 글상자 안 paragraph
//! 를 본문 list 로 인식하여 신규 paragraph (각주) 추가 시 본문 다단계 목록
//! "1.1.1.1.1.1" 자동 부여.
//!
//! 참조: `hwplib::ForTextBox::listHeader`.
//!
//! 정정 (Stage 2): src/serializer/control.rs::serialize_text_box_if_present —
//! raw_list_header_extra 가 비어 있을 때 (HWPX 출처) 13 byte zero default 적용.

use std::fs;
use std::path::Path;

fn load(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse")
}

/// CFB stream BodyText/Section0 의 LIST_HEADER 레코드 모두 추출 (tag=72).
fn collect_list_header_sizes(hwp_bytes: &[u8]) -> Vec<(u16, usize)> {
    use std::io::Read;
    let cursor = std::io::Cursor::new(hwp_bytes);
    let mut comp = cfb::CompoundFile::open(cursor).expect("cfb open");
    let mut fh = comp.open_stream("/FileHeader").expect("FileHeader");
    let mut fh_data = Vec::new();
    fh.read_to_end(&mut fh_data).expect("read FileHeader");
    let is_compressed = fh_data.get(36).map(|b| (b & 0x01) != 0).unwrap_or(false);
    drop(fh);

    let mut sec0 = comp.open_stream("/BodyText/Section0").expect("Section0");
    let mut raw = Vec::new();
    sec0.read_to_end(&mut raw).expect("read Section0");

    let data = if is_compressed {
        let mut decoder = flate2::read::DeflateDecoder::new(&raw[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).expect("inflate");
        decompressed
    } else {
        raw
    };

    let mut out = Vec::new();
    let mut pos = 0;
    while pos + 4 <= data.len() {
        let hdr = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let tag = (hdr & 0x3FF) as u16;
        let level = ((hdr >> 10) & 0x3FF) as u16;
        let mut size = ((hdr >> 20) & 0xFFF) as usize;
        pos += 4;
        if size == 0xFFF && pos + 4 <= data.len() {
            size = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            pos += 4;
        }
        if tag == 72 {
            out.push((level, size));
        }
        pos += size;
    }
    out
}

/// HWPX 출처 footnote-tbox-01.hwpx 의 글상자 LIST_HEADER 가 33 byte 한컴 정합.
#[test]
fn issue_1058_textbox_list_header_size_33() {
    let mut doc = load("samples/hwpx/footnote-tbox-01.hwpx");
    let hwp_bytes = doc.export_hwp_with_adapter().expect("export");
    let lh_sizes = collect_list_header_sizes(&hwp_bytes);

    // 글상자 LIST_HEADER size=33 가 적어도 1개 존재해야 함 (본 sample 글상자 1개)
    let has_textbox_lh = lh_sizes.iter().any(|(_, sz)| *sz == 33);
    assert!(
        has_textbox_lh,
        "글상자 LIST_HEADER size=33 (한컴 contract) 가 적어도 1개 존재해야 함. \
         actual sizes: {:?}",
        lh_sizes
    );
}

/// HWP 출처 라운드트립 회귀 부재 — table-in-tbox.hwp 의 글상자 LIST_HEADER 33 유지.
#[test]
fn issue_1058_hwp_textbox_roundtrip() {
    let mut doc = load("samples/table-in-tbox.hwp");
    let hwp_bytes = doc.export_hwp_with_adapter().expect("export");
    let lh_sizes = collect_list_header_sizes(&hwp_bytes);

    // HWP 출처는 raw_list_header_extra 보존으로 size 정합
    let has_textbox_lh = lh_sizes.iter().any(|(_, sz)| *sz == 33);
    assert!(
        has_textbox_lh,
        "HWP 출처 table-in-tbox 의 글상자 LIST_HEADER 33 보존. actual sizes: {:?}",
        lh_sizes
    );
}

/// Task #1050 회귀 가드 양립 — footnote LIST_HEADER size=16 (글상자 33 과 별개).
#[test]
fn issue_1058_footnote_list_header_size_16_preserved() {
    let mut doc = load("samples/hwpx/footnote-tbox-01.hwpx");
    let hwp_bytes = doc.export_hwp_with_adapter().expect("export");
    let lh_sizes = collect_list_header_sizes(&hwp_bytes);

    // footnote LIST_HEADER size=16 가 적어도 2개 존재 (글상자 안 + 본문 각주)
    let footnote_lh_count = lh_sizes.iter().filter(|(_, sz)| *sz == 16).count();
    assert!(
        footnote_lh_count >= 2,
        "footnote LIST_HEADER size=16 적어도 2개 (Task #1050 정합). actual sizes: {:?}",
        lh_sizes
    );
}

/// 글상자 LIST_HEADER 의 정확한 raw byte 매핑 — hwplib::ForTextBox::listHeader 정합.
#[test]
fn issue_1058_textbox_list_header_byte_contract() {
    let mut doc = load("samples/hwpx/footnote-tbox-01.hwpx");
    let hwp_bytes = doc.export_hwp_with_adapter().expect("export");

    // CFB 직접 읽어 LIST_HEADER tag=72 size=33 payload 추출
    use std::io::Read;
    let cursor = std::io::Cursor::new(&hwp_bytes);
    let mut comp = cfb::CompoundFile::open(cursor).expect("cfb open");
    let mut fh = comp.open_stream("/FileHeader").expect("FileHeader");
    let mut fh_data = Vec::new();
    fh.read_to_end(&mut fh_data).expect("read FileHeader");
    let is_compressed = fh_data.get(36).map(|b| (b & 0x01) != 0).unwrap_or(false);
    drop(fh);

    let mut sec0 = comp.open_stream("/BodyText/Section0").expect("Section0");
    let mut raw = Vec::new();
    sec0.read_to_end(&mut raw).expect("read Section0");
    let data = if is_compressed {
        let mut decoder = flate2::read::DeflateDecoder::new(&raw[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).expect("inflate");
        decompressed
    } else {
        raw
    };

    let mut pos = 0;
    let mut found = None;
    while pos + 4 <= data.len() {
        let hdr = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
        let tag = hdr & 0x3FF;
        let mut size = ((hdr >> 20) & 0xFFF) as usize;
        pos += 4;
        if size == 0xFFF && pos + 4 <= data.len() {
            size = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            pos += 4;
        }
        if tag == 72 && size == 33 {
            found = Some(data[pos..pos + size].to_vec());
            break;
        }
        pos += size;
    }

    let payload = found.expect("글상자 LIST_HEADER (size=33) 찾기");

    // hwplib::ForTextBox::listHeader 정합 체크:
    // offset 20..28: zero 8 byte
    assert_eq!(
        &payload[20..28],
        &[0u8; 8],
        "TextBox LIST_HEADER offset 20-27: zero 8 byte padding"
    );
    // offset 28..32: editableAtFormMode = 0 (false)
    let editable = i32::from_le_bytes([payload[28], payload[29], payload[30], payload[31]]);
    assert_eq!(
        editable, 0,
        "TextBox LIST_HEADER offset 28-31: editableAtFormMode = 0"
    );
    // offset 32: fieldName flag = 0 (no fieldName)
    assert_eq!(
        payload[32], 0,
        "TextBox LIST_HEADER offset 32: fieldName flag = 0"
    );
}
