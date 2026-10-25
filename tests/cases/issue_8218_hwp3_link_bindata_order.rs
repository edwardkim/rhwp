#![cfg(not(target_arch = "wasm32"))]

//! [#8218] HWP3 의 연결(Link) 그림 BinData 는 실행마다 같은 순서로, binDataId 위치에 놓인다.
//! 저장한 HWP5 를 다시 열어도 그림마다 원래 외부 경로를 가리켜야 한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::bin_data::BinDataType;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::parse_document;

const FIXTURE: &str = "samples/hwp3-sample10.hwp";

fn read_fixture() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE))
        .unwrap_or_else(|error| panic!("read {FIXTURE}: {error}"))
}

/// 본문 그림의 (binDataId, 외부 경로) — 등장 순서.
fn picture_links(doc: &Document) -> Vec<(u16, Option<String>)> {
    let mut out = Vec::new();
    for section in &doc.sections {
        for para in &section.paragraphs {
            for control in &para.controls {
                let picture = match control {
                    Control::Picture(picture) => picture,
                    Control::Shape(shape) => match shape.as_ref() {
                        ShapeObject::Picture(picture) => picture,
                        _ => continue,
                    },
                    _ => continue,
                };
                out.push((
                    picture.image_attr.bin_data_id,
                    picture.image_attr.external_path.clone(),
                ));
            }
        }
    }
    out
}

#[test]
fn link_bin_data_is_ordered_by_id_every_parse() {
    let bytes = read_fixture();
    let first = parse_document(&bytes).expect("parse");
    let links = first
        .doc_info
        .bin_data_list
        .iter()
        .filter(|bin| matches!(bin.data_type, BinDataType::Link))
        .count();
    assert!(
        links >= 2,
        "{FIXTURE} 에 연결 그림이 둘 이상 있어야 순서를 검사한다"
    );

    let ids = |doc: &Document| -> Vec<(u16, Option<String>)> {
        doc.doc_info
            .bin_data_list
            .iter()
            .map(|bin| (bin.storage_id, bin.abs_path.clone()))
            .collect()
    };
    let expected = ids(&first);
    for (index, (storage_id, _)) in expected.iter().enumerate() {
        assert_eq!(
            usize::from(*storage_id),
            index + 1,
            "BIN_DATA 위치 {} 의 storage_id 가 {storage_id} 다",
            index + 1
        );
    }
    // 맵마다 해시 시드가 달라 종전 구현은 여러 번 열면 순서가 바뀐다.
    for _ in 0..16 {
        let again = parse_document(&bytes).expect("parse again");
        assert_eq!(ids(&again), expected, "열 때마다 BinData 순서가 바뀐다");
    }
}

#[test]
fn saved_hwp5_keeps_each_picture_external_path() {
    let bytes = read_fixture();
    let original = parse_document(&bytes).expect("parse");
    let before = picture_links(&original);
    assert!(
        before.iter().filter(|(_, path)| path.is_some()).count() >= 2,
        "외부 경로 그림이 둘 이상 있어야 한다"
    );

    let core = DocumentCore::from_bytes(&bytes).expect("open");
    let saved = parse_document(&core.export_hwp_native().expect("save")).expect("reparse");
    assert_eq!(
        picture_links(&saved),
        before,
        "저장한 HWP5 를 다시 열면 그림이 다른 외부 경로를 가리킨다"
    );
}
