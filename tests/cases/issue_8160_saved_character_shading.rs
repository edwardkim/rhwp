//! #8160: 원본 팔레트 0 x 15%의 독립 한컴 회색과 실제 HWP5 저장 바이트.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::parser::{cfb_reader::CfbReader, record::Record, tags, FileFormat};
use std::path::Path;
const SHADE_RANGE: std::ops::Range<usize> = 60..64;

/// CLI `convert` 와 같은 경로로 HWP3 를 HWP5 바이트로 만든다.
fn convert_to_hwp5_bytes(path: &Path) -> Option<Vec<u8>> {
    let raw = std::fs::read(path).expect("표본 읽기");
    let mut doc = rhwp::parser::hwp3::parse_hwp3(&raw).ok()?;
    rhwp::document_core::converters::hwpx_to_hwp::convert_if_hwpx_source(
        &mut doc,
        FileFormat::Hwp3,
    );
    rhwp::serializer::cfb_writer::serialize_hwp(&doc).ok()
}

/// public 저장 경로 (`DocumentCore::export_hwp_with_adapter`).
fn convert_to_hwp5_bytes_via_document_core(path: &Path) -> Vec<u8> {
    let raw = std::fs::read(path).expect("표본 읽기");
    let mut core = rhwp::document_core::DocumentCore::from_bytes(&raw).expect("HWP3 파싱");
    core.export_hwp_with_adapter().expect("HWP5 직렬화")
}

/// 저장된 HWP5 바이트의 DocInfo 에서 CHAR_SHAPE payload 를 꺼낸다.
///
/// `Record::read_all` 이 레코드 순회의 정본이다 — 테스트마다 손으로 다시 쓰지 않는다.
fn char_shape_payloads(bytes: &[u8]) -> Vec<Vec<u8>> {
    let mut cfb = CfbReader::open(bytes).expect("CFB 열기");
    let file_header = cfb.read_file_header().expect("FileHeader 읽기");
    let compressed = file_header.get(36).is_some_and(|b| b & 0x01 != 0);
    let doc_info = cfb.read_doc_info(compressed).expect("DocInfo 읽기");
    Record::read_all(&doc_info)
        .expect("DocInfo record 파싱")
        .into_iter()
        .filter(|record| record.tag_id == tags::HWPTAG_CHAR_SHAPE)
        .map(|record| record.data)
        .collect()
}

/// 저장 바이트의 CHAR_SHAPE 음영색을 id 순으로 뽑는다.
fn shade_colors(label: &str, hwp5: &[u8]) -> Vec<u32> {
    let payloads = char_shape_payloads(hwp5);
    assert!(
        !payloads.is_empty(),
        "{label}: CHAR_SHAPE 레코드가 없다 — 저장 경로를 확인하라"
    );
    payloads
        .into_iter()
        .enumerate()
        .map(|(id, payload)| {
            assert!(
                payload.len() >= SHADE_RANGE.end,
                "{label}: CHAR_SHAPE id={id} payload 가 {}바이트로 짧다 \
                 (음영색은 오프셋 {}..{} 에 있어야 한다)",
                payload.len(),
                SHADE_RANGE.start,
                SHADE_RANGE.end
            );
            u32::from_le_bytes(payload[SHADE_RANGE].try_into().expect("4바이트"))
        })
        .collect()
}
#[test]
fn issue_8160_saved_character_shading_matches_independent_hancom_colors() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let input = root.join("samples/hwp3-sample16.hwp");
    let oracle = shade_colors(
        "한컴 저장본",
        &std::fs::read(root.join("samples/hwp3-sample16-hwp5.hwp")).unwrap(),
    );
    assert!(
        oracle.contains(&0x00D8_D8D8),
        "기존 독립 기대값: 팔레트 0, 15% 음영"
    );
    for bytes in [
        convert_to_hwp5_bytes(&input).expect("변환기 경로"),
        convert_to_hwp5_bytes_via_document_core(&input),
    ] {
        let shades = shade_colors("원본 변환", &bytes);
        assert!(
            !shades.contains(&0),
            "검정 막대를 만드는 저장 음영을 방지한다"
        );
        assert!(
            shades.contains(&0x00D8_D8D8),
            "독립 한컴 회색의 반올림 보존"
        );
        assert!(
            shades.iter().all(|color| oracle.contains(color)),
            "한컴이 내지 않은 음영색을 생성하지 않는다"
        );
    }
}
