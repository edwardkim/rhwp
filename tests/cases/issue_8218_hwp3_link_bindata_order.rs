//! 검사 유형: 관계형·고정형.
//! 2026-10-28 측정 raw(96dpi, PDFium): hwp3-sample4.hwp 1쪽 Native 64.32418% / fresh WASM 64.81033%,
//! 21쪽 Native·fresh WASM 각 100.00000%; 전36쪽 최저 64.32418% / 64.81033%.
//! 1쪽은 90% 미만이므로 관계 검사만 사용한다. 독립 Print의 21쪽 내장 그림 좌표를 고정 검사한다.
//! 연결 그림 두 개를 삽입한 비시각 합성 입력은 별도이며 원본의 측정값으로 대체하지 않는다.
//! 근거: mydocs/pr/assets/issue8219/sample4/evidence.json.

#![cfg(not(target_arch = "wasm32"))]

//! [#8218] HWP3의 그림 BinData는 실행마다 같은 순서로, binDataId 위치에 놓인다.
//! sample4는 내장 그림 문서다. 외부 연결 순서는 이 원본의 두 그림 레코드만 메모리에서
//! 연결 그림으로 바꾼 별도 합성 입력으로 검사한다. 합성 입력을 원본 Print로 취급하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::bin_data::BinDataType;
use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::shape::ShapeObject;
use rhwp::parser::parse_document;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::io::{Read, Write};

const FIXTURE: &str = "samples/hwp3-sample4.hwp";

fn read_fixture() -> Vec<u8> {
    std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE))
        .unwrap_or_else(|error| panic!("read {FIXTURE}: {error}"))
}

/// 원본 sample4는 수정하지 않는다. 압축 본문에서 지정한 두 그림의 종류·이름만 바꿔
/// 내장과 연결이 섞인 HWP3를 메모리에서 만든다. 길이·형식 가정을 먼저 확인한다.
fn sample_with_synthetic_links() -> Vec<u8> {
    let source = read_fixture();
    assert!(source.starts_with(b"HWP Document File V3.00"));
    assert_eq!(source[154], 1, "sample4 body uses raw deflate");
    let info_len = u16::from_le_bytes([source[156], source[157]]) as usize;
    let body_offset = 1166 + info_len;
    let mut body = Vec::new();
    flate2::read::DeflateDecoder::new(&source[body_offset..])
        .read_to_end(&mut body)
        .expect("decompress HWP3 body");
    for (name, link) in [
        (&b"E$$00210.WMF"[..], &b"C:\\rhwp-fixture\\linked-1.wmf"[..]),
        (&b"E$$00211.WMF"[..], &b"C:\\rhwp-fixture\\linked-2.wmf"[..]),
    ] {
        let pos = body
            .windows(name.len())
            .position(|bytes| bytes == name)
            .expect("picture name");
        assert_eq!(body[pos - 9], 2, "source picture must be embedded");
        assert!(pos + 256 <= body.len() && link.len() < 256);
        body[pos - 9] = 0;
        body[pos..pos + 256].fill(0);
        body[pos..pos + link.len()].copy_from_slice(link);
    }
    let mut compressor =
        flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    compressor.write_all(&body).expect("compress HWP3 body");
    let mut result = source[..body_offset].to_vec();
    result.extend(compressor.finish().expect("finish HWP3 body"));
    result
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
    let bytes = sample_with_synthetic_links();
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
    let bytes = sample_with_synthetic_links();
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

#[test]
fn sample4_embedded_pictures_keep_ids_and_contents_after_hwp5_save() {
    let bytes = read_fixture();
    let original = parse_document(&bytes).expect("parse sample4");
    let pictures = picture_links(&original);
    assert_eq!(pictures.len(), 13, "실제 sample4의 그림 수");
    assert!(
        pictures.iter().all(|(_, path)| path.is_none()),
        "sample4 그림은 내장이다"
    );
    let core = DocumentCore::from_bytes(&bytes).expect("open sample4");
    let saved = parse_document(&core.export_hwp_native().expect("save sample4")).expect("reparse");
    assert_eq!(
        picture_links(&saved),
        pictures,
        "저장 뒤 그림 id와 순서 보존"
    );
    for (id, _) in pictures {
        let before = original
            .bin_data_content
            .iter()
            .find(|content| content.id == id)
            .expect("original picture");
        let after = saved
            .bin_data_content
            .iter()
            .find(|content| content.id == id)
            .expect("saved picture");
        assert_eq!(
            before.data.load(),
            after.data.load(),
            "내장 그림 {id} 바이트 보존"
        );
    }
}

#[test]
fn sample4_page21_embedded_picture_matches_independent_print() {
    fn image(node: &RenderNode) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::Image(_)) {
            return Some(node);
        }
        node.children.iter().find_map(image)
    }
    let core = DocumentCore::from_bytes(&read_fixture()).expect("sample4");
    let tree = core.build_page_render_tree(20).expect("21쪽");
    let frame = image(&tree.root).expect("21쪽 내장 그림").bbox;
    for (actual, print, name) in [
        (frame.x, 255.70, "left"),
        (frame.y, 954.16, "top"),
        (frame.x + frame.width, 496.06, "right"),
        (frame.y + frame.height, 1002.77, "bottom"),
    ] {
        assert!(
            (actual - print).abs() <= 2.0,
            "21쪽 그림 {name}: {actual:.2}px, Print {print:.2}px"
        );
    }
}
