//! [#7994/#8064] HWP3 계보를 가진 HWP5의 원본 문단 간격을 HWPX에 보존한다.
//!
//! #8064의 b9bb438d99 이후 HWP5 파서는 위·아래 간격을 IR에서 반감하지 않는다.
//! 기대값은 입력의 PARA_SHAPE raw 레코드에서 직접 읽는다. 계보 표시를 이유로 이를
//! 두 배 복원하거나, rhwp 전용 마커로 재열기에서만 다시 반감하면 외부 소비자가 달라진다.
//!
//! 이 검사는 원본 속성과 저장·재열기의 IR 계약을 보호한다. 쪽수·페이지 소유의 렌더링
//! 기대값은 고정하지 않는다. 이전 검사 원문과 독립 인쇄 결과는
//! mydocs/pr/assets/issue7445/deferred-20261020/source-pr8028/에 보존한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::{Cursor, Read, Write};

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/HWP5-nopassword-123456.hwp";
const HWP3_MARKER: &str = "META-INF/rhwp-hwp3-origin";

fn raw_spacing(raw: &[u8]) -> (i32, i32) {
    let at = |o: usize| i32::from_le_bytes([raw[o], raw[o + 1], raw[o + 2], raw[o + 3]]);
    (at(16), at(20))
}

/// HWPX 에서 한 항목을 빼거나 바꾼 사본(rhwp 마커를 모르는 소비자를 흉내 낸다).
fn rewrite_entry(hwpx: &[u8], path: &str, replacement: Option<&[u8]>) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(hwpx)).expect("zip");
    let mut out = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for i in 0..input.len() {
        let mut entry = input.by_index(i).expect("entry");
        let name = entry.name().expect("HWPX ZIP 엔트리 이름").to_string();
        let mut data = Vec::new();
        entry.read_to_end(&mut data).expect("read entry");
        let data = if name == path {
            match replacement {
                Some(value) => value.to_vec(),
                None => continue,
            }
        } else {
            data
        };
        let options =
            zip::write::SimpleFileOptions::default().compression_method(if name == "mimetype" {
                zip::CompressionMethod::Stored
            } else {
                zip::CompressionMethod::Deflated
            });
        out.start_file(name, options).expect("start");
        out.write_all(&data).expect("write");
    }
    out.finish().expect("finish").into_inner()
}

#[test]
fn hwp3_variant_hwpx_export_writes_stored_paragraph_spacing() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(&path).expect("read")).expect("parse");
    assert!(
        core.document().is_hwp3_variant,
        "HWP3 계보를 가진 실제 HWP5 표본"
    );
    let spacing = |document: &DocumentCore| {
        document
            .document()
            .doc_info
            .para_shapes
            .iter()
            .map(|shape| (shape.spacing_before, shape.spacing_after))
            .collect::<Vec<_>>()
    };
    let stored: Vec<_> = core
        .document()
        .doc_info
        .para_shapes
        .iter()
        .map(|shape| raw_spacing(shape.raw_data.as_deref().expect("원본 PARA_SHAPE 레코드")))
        .collect();
    assert!(
        stored
            .iter()
            .any(|&(before, after)| before != 0 || after != 0),
        "0뿐인 간격으로는 두 배 복원을 검출할 수 없다"
    );
    assert_eq!(
        spacing(&core),
        stored,
        "#8064: HWP5 계보와 문단 간격의 저장 단위는 별도 계약이다"
    );

    let hwpx = core.export_hwpx_native().expect("export");
    let foreign = DocumentCore::from_bytes(&rewrite_entry(&hwpx, HWP3_MARKER, None))
        .expect("rhwp 마커를 모르는 소비자의 IR");
    assert_eq!(
        spacing(&foreign),
        stored,
        "외부 소비자에 전달하는 간격은 원본 PARA_SHAPE 저장값이어야 한다"
    );
    let reopened = DocumentCore::from_bytes(&hwpx).expect("reopen");
    assert_eq!(
        spacing(&reopened),
        stored,
        "rhwp 전용 마커의 해석 여부가 저장 간격을 바꾸면 안 된다"
    );
    let legacy = DocumentCore::from_bytes(&rewrite_entry(&hwpx, HWP3_MARKER, Some(b"1")))
        .expect("기존 계보 표지가 있는 사본");
    assert_eq!(
        spacing(&legacy),
        stored,
        "기존 표지 파일도 원본 속성을 다시 반감하지 않는다"
    );
}
