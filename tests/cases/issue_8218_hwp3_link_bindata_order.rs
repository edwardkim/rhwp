//! 검사 유형: 관계형·고정형. 그림의 소속·순서·내용 보존을 검사한다.
//! 주석 갱신: 2026-11-17T18:51:52+09:00; 직전 변경: 062e7a4db540861e4d0eebea16d714465484528a 2026-11-17T13:50:54+09:00.
//! 독립 측정 이력: 2026-11-17, source a27afaf7c9d7e2444b2ad1b14550da4ab5ab7202, hft2ttf 공급 조건.
//! samples/hwp3-sample4.hwp 및 독립 한컴 2020 Print pdf/hwp3-sample4-2020.pdf 각 36쪽.
//! 96 DPI, 색상 경계 보정 전 2px raw: Native 전쪽 최저 99.32%, fresh WASM 99.33%.
//! 전쪽 TSV: mydocs/pr/assets/issue_8371/sample4-source-layout-20261117/native-hft2ttf.tsv, wasm-hft2ttf.tsv.
//! 위 값은 과거 실행의 출처이며 현재 정리 head의 재실행 결과로 표시하지 않는다.
//! 현재 검증과 커밋 선별 근거는 같은 경로의 validation.md를 따른다.
//! 기존 21쪽 좌표의 독립 출처: mydocs/pr/assets/issue8219/sample4/evidence.json.
//! NASM 그림은 독립 Print의 26쪽 소속과 그림→설명→본문 관계로 검사한다.
//! 외부 연결 그림을 삽입한 메모리 합성 입력은 별도의 비시각 저장 계약이다.
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

fn sample4_nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut result = vec![node];
    for child in &node.children {
        result.extend(sample4_nodes(child));
    }
    result
}

#[test]
fn sample4_nasm_picture_and_following_text_keep_their_source_page() {
    let core = DocumentCore::from_bytes(&read_fixture()).unwrap();
    let trees: Vec<_> = (0..core.page_count())
        .map(|p| core.build_page_render_tree(p).unwrap())
        .collect();
    let mut owners = Vec::new();
    for (page, tree) in trees.iter().enumerate() {
        for node in sample4_nodes(&tree.root) {
            if matches!(&node.node_type, RenderNodeType::Image(image) if image.para_index == Some(853))
            {
                owners.push((page, node));
            }
        }
    }
    assert_eq!(owners.len(), 1, "NASM 그림 누락·중복");
    let (page, image) = owners[0];
    assert_eq!(page, 25, "독립 Print의 26쪽에서만 그림을 소유한다");
    let text_at = |page: usize, pi: usize| {
        sample4_nodes(&trees[page].root).into_iter().filter(|node| {
            matches!(&node.node_type, RenderNodeType::TextRun(run) if run.para_index == Some(pi) && !run.text.is_empty())
        }).collect::<Vec<_>>()
    };
    let caption = text_at(page, 857);
    let body = text_at(page, 859);
    assert!(
        !caption.is_empty() && !body.is_empty(),
        "그림 설명·다음 본문 보존"
    );
    assert!(caption
        .iter()
        .all(|n| image.bbox.y + image.bbox.height <= n.bbox.y));
    let caption_bottom = caption
        .iter()
        .map(|n| n.bbox.y + n.bbox.height)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        body.iter().all(|n| caption_bottom <= n.bbox.y),
        "그림→설명→본문 순서"
    );
    assert!(
        !text_at(24, 852).is_empty(),
        "앞 쪽 제목도 자기 쪽에 보존한다"
    );
}
