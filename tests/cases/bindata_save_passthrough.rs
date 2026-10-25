#![cfg(not(target_arch = "wasm32"))]

//! HWP5 저장은 바뀌지 않은 그림(BinData)의 저장 스트림을 원본 그대로 쓰고,
//! 바꾼 그림은 새 내용으로 쓴다.

use rhwp::document_core::DocumentCore;
use rhwp::model::bin_data::BinDataBytes;
use rhwp::parser::parse_document;

const FIXTURE: &str = "samples/exam_kor.hwp";

fn read_fixture() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE))
        .unwrap_or_else(|error| panic!("read {FIXTURE}: {error}"))
}

#[test]
fn unchanged_bin_data_keeps_stored_stream_and_content() {
    let bytes = read_fixture();
    let original = parse_document(&bytes).expect("parse fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("open fixture");
    let saved = parse_document(&core.export_hwp_native().expect("save")).expect("reparse");

    assert!(
        !original.bin_data_content.is_empty(),
        "{FIXTURE} 에 그림이 있어야 검사가 의미 있다"
    );
    assert_eq!(
        saved.bin_data_content.len(),
        original.bin_data_content.len()
    );
    for (before, after) in original
        .bin_data_content
        .iter()
        .zip(&saved.bin_data_content)
    {
        assert_eq!(after.id, before.id);
        assert_eq!(
            after.data.load(),
            before.data.load(),
            "BIN{:04X} 내용이 저장 뒤 달라졌다",
            before.id
        );
        let (Some(stored_before), Some(stored_after)) =
            (before.data.load_raw(), after.data.load_raw())
        else {
            panic!("BIN{:04X} 저장 형태를 읽지 못했다", before.id);
        };
        assert_eq!(stored_after.compressed, stored_before.compressed);
        assert!(
            stored_after.bytes == stored_before.bytes,
            "BIN{:04X} 저장 스트림이 원본과 다르다",
            before.id
        );
    }
}

#[test]
fn replaced_bin_data_is_written_with_new_content() {
    let bytes = read_fixture();
    let mut core = DocumentCore::from_bytes(&bytes).expect("open fixture");
    let target = core.document().bin_data_content[0].id;
    let mut replacement = core.document().bin_data_content[0].data.load();
    // 같은 형식을 유지하되 내용을 바꾼다(끝 바이트 반전).
    let last = replacement.len() - 1;
    replacement[last] ^= 0xff;
    core.document_mut().bin_data_content[0].data = BinDataBytes::Loaded(replacement.clone());

    let saved = parse_document(&core.export_hwp_native().expect("save")).expect("reparse");
    let after = saved
        .bin_data_content
        .iter()
        .find(|content| content.id == target)
        .expect("saved bin data");
    assert_eq!(
        after.data.load(),
        replacement,
        "바꾼 그림이 원본 저장 스트림으로 덮였다"
    );
}
