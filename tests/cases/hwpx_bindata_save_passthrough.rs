#![cfg(not(target_arch = "wasm32"))]

//! HWPX 저장은 바뀌지 않은 그림(BinData)을 다시 압축하지 않고 원본 deflate 바이트를
//! 옮기고, 바꾼 그림은 새 내용으로 쓴다.

use std::io::{Cursor, Read};

use rhwp::document_core::DocumentCore;
use rhwp::model::bin_data::BinDataBytes;
use rhwp::parser::parse_document;

fn read_fixture(path: &str) -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

/// BinData/ 아래 deflate 엔트리의 (이름, 압축 바이트, 해제 바이트).
fn bin_data_entries(zip_bytes: &[u8]) -> Vec<(String, Vec<u8>, Vec<u8>)> {
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_bytes.to_vec())).expect("zip");
    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let (name, raw) = {
            let mut entry = archive.by_index_raw(index).expect("raw entry");
            let name = entry.name().expect("entry name").into_owned();
            if !name.starts_with("BinData/")
                || entry.compression() != zip::CompressionMethod::Deflated
            {
                continue;
            }
            let mut raw = Vec::new();
            entry.read_to_end(&mut raw).expect("read raw");
            (name, raw)
        };
        let mut content = Vec::new();
        archive
            .by_index(index)
            .expect("entry")
            .read_to_end(&mut content)
            .expect("read content");
        entries.push((name, raw, content));
    }
    entries
}

#[test]
fn hwpx_source_keeps_original_deflate_bytes() {
    let source = read_fixture("samples/hwpx/exam_kor.hwpx");
    let original = bin_data_entries(&source);
    assert!(
        !original.is_empty(),
        "원본에 deflate 그림 엔트리가 있어야 한다"
    );

    let core = DocumentCore::from_bytes(&source).expect("open");
    let saved = bin_data_entries(&core.export_hwpx_native().expect("save hwpx"));
    for (name, raw, content) in &original {
        let (_, saved_raw, saved_content) = saved
            .iter()
            .find(|(saved_name, ..)| saved_name == name)
            .unwrap_or_else(|| panic!("{name} 가 저장본에 없다"));
        assert_eq!(saved_content, content, "{name} 내용이 달라졌다");
        assert!(saved_raw == raw, "{name} 압축 바이트가 원본과 다르다");
    }
}

#[test]
fn hwp_source_reuses_stored_deflate_stream() {
    let source = read_fixture("samples/exam_kor.hwp");
    let original = parse_document(&source).expect("parse hwp");
    let core = DocumentCore::from_bytes(&source).expect("open");
    let saved = bin_data_entries(&core.export_hwpx_native().expect("save hwpx"));
    assert!(!saved.is_empty());

    for content in &original.bin_data_content {
        let Some(stored) = content.data.load_raw().filter(|stored| stored.compressed) else {
            continue;
        };
        let loaded = content.data.load();
        let (name, saved_raw, saved_content) = saved
            .iter()
            .find(|(_, _, saved_content)| *saved_content == loaded)
            .unwrap_or_else(|| panic!("BIN{:04X} 내용의 엔트리가 저장본에 없다", content.id));
        assert!(
            *saved_raw == stored.bytes,
            "{name} 가 HWP 저장 스트림을 그대로 쓰지 않았다"
        );
        assert_eq!(*saved_content, loaded);
    }
}

#[test]
fn replaced_image_is_written_with_new_content() {
    let source = read_fixture("samples/hwpx/exam_kor.hwpx");
    let mut core = DocumentCore::from_bytes(&source).expect("open");
    let mut replacement = core.document().bin_data_content[0].data.load();
    let last = replacement.len() - 1;
    replacement[last] ^= 0xff;
    core.document_mut().bin_data_content[0].data = BinDataBytes::Loaded(replacement.clone());

    let saved = bin_data_entries(&core.export_hwpx_native().expect("save hwpx"));
    assert!(
        saved.iter().any(|(_, _, content)| *content == replacement),
        "바꾼 그림이 원본 엔트리로 덮였다"
    );
}
