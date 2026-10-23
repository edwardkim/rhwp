//! [#7994] HWP3→HWP5 변환본을 HWPX 로 저장할 때 문단 위·아래 간격은 **저장값**으로 쓴다.
//!
//! HWP5 파서는 HWP3 변환본(`is_hwp3_variant`)의 PARA_SHAPE 문단 위·아래 간격을 렌더용으로
//! 반감한다. 수정 전 HWPX 직렬화기는 그 내부 값을 그대로 써서, 한/글이 간격을 절반으로 읽고
//! 쪽수를 줄였다(10k 07821: 원본 111쪽, rhwp h2x 106쪽). 수정 후에는 원본 레코드 값을 쓰고,
//! rhwp 가 그 HWPX 를 다시 열 때는 HWP3 마커의 «간격 저장값» 표지로 같은 반감을 다시 건다.
//!
//! 입력 `samples/HWP5-nopassword-123456.hwp`(HWP3 변환본 HWP5). 기대값의 근거는 파일의 PARA_SHAPE
//! 원본 레코드(오프셋 16·20 의 문단 위·아래 간격)다.
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
        "표본은 HWP3 변환본이어야 한다"
    );

    let shapes = &core.document().doc_info.para_shapes;
    let stored: Vec<(i32, i32)> = shapes
        .iter()
        .map(|ps| raw_spacing(ps.raw_data.as_deref().expect("원본 PARA_SHAPE 레코드")))
        .collect();
    let rendered: Vec<(i32, i32)> = shapes
        .iter()
        .map(|ps| (ps.spacing_before, ps.spacing_after))
        .collect();
    assert!(
        stored
            .iter()
            .zip(&rendered)
            .any(|(s, r)| s.0 != r.0 || s.1 != r.1),
        "표본에 반감된 간격이 있어야 시험이 성립한다"
    );

    let hwpx = core.export_hwpx_native().expect("export");

    // 1) 파일에는 저장값 — rhwp 마커를 모르는 소비자(한/글)가 읽는 값.
    let foreign =
        DocumentCore::from_bytes(&rewrite_entry(&hwpx, HWP3_MARKER, None)).expect("마커 없는 사본");
    let written: Vec<(i32, i32)> = foreign
        .document()
        .doc_info
        .para_shapes
        .iter()
        .map(|ps| (ps.spacing_before, ps.spacing_after))
        .collect();
    assert_eq!(
        written, stored,
        "HWPX 의 문단 위·아래 간격은 원본 PARA_SHAPE 저장값이어야 한다"
    );

    // 2) rhwp 가 다시 열면 원본 렌더와 같은 내부 값·쪽 배치.
    let reopened = DocumentCore::from_bytes(&hwpx).expect("reopen");
    let reopened_spacing: Vec<(i32, i32)> = reopened
        .document()
        .doc_info
        .para_shapes
        .iter()
        .map(|ps| (ps.spacing_before, ps.spacing_after))
        .collect();
    assert_eq!(
        reopened_spacing, rendered,
        "재열기는 HWP5 파서와 같은 렌더용 간격을 써야 한다"
    );
    assert_eq!(reopened.page_count(), core.page_count(), "왕복 쪽수");
    // 쪽별 소유·컷만 비교한다(컨테이너별 저장 줄 메타데이터는 비교하지 않는다 — #5128 과 같은 축).
    let owners = |doc: &DocumentCore| {
        doc.dump_page_items_json(None)
            .as_array()
            .unwrap()
            .iter()
            .map(|page| {
                page["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|column| {
                        column["items"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|item| {
                                [
                                    "kind",
                                    "paraIndex",
                                    "controlIndex",
                                    "startLine",
                                    "endLine",
                                    "startRow",
                                    "endRow",
                                ]
                                .map(|key| item.get(key).cloned().unwrap_or_default())
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(owners(&reopened), owners(&core), "왕복 쪽별 항목·컷");

    // 3) 옛 표지(`1`)의 HWPX 는 이미 반감된 값을 담고 있으므로 다시 반감하지 않는다.
    let legacy = DocumentCore::from_bytes(&rewrite_entry(&hwpx, HWP3_MARKER, Some(b"1")))
        .expect("옛 표지 사본");
    let legacy_spacing: Vec<(i32, i32)> = legacy
        .document()
        .doc_info
        .para_shapes
        .iter()
        .map(|ps| (ps.spacing_before, ps.spacing_after))
        .collect();
    assert_eq!(
        legacy_spacing, stored,
        "옛 표지 파일의 값은 그대로 읽는다(재반감 없음)"
    );
}
