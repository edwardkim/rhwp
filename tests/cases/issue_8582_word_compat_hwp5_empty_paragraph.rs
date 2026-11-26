//! #8582: 호환 대상이 MS 워드인 **HWP5** 문서의 저장 줄 없는 빈 문단도 그리기 y 가
//! 배율로 잰 줄 높이를 따른다.
//!
//! HWPX 재현물(`word_hcr_empty`)은 읽을 때 줄을 다시 짜서 저장 줄이 생기므로 그리기의
//! 빈 문단 폴백 경로를 지나지 않는다. 실문서(HWP5, 저장 줄 없음)는 그 경로를 지나는데,
//! 조판(쪽 나눔 장부)은 배율을 썼지만 그리기는 글자 크기 그대로 써서 빈 문단 뒤 글줄이
//! 위로 당겨졌다(빈 문단 하나에 11pt 기준 약 4.4px). 이 시험은 같은 재현물의 저장 줄을
//! 지우고 호환 대상을 MS 워드로 둔 HWP5 로 다시 저장해 그 경로를 잰다.
//!
//! 기준: 데스크톱 한컴 인쇄. 레이아웃 호환 플래그가 켜진 HWP5(`word_hcr_*_hwp5b`)는 2020 엔진
//! 인쇄가 HWPX 2024 인쇄와 기준선까지 같다. `TOP` 에서 `END` 까지 32460 HU (`TOP` 줄 10pt 160%
//! 2706 HU + 빈 문단 16개 × 1860 HU). 플래그가 0 이면 한/글 줄 높이(19200 HU, 인쇄 19172 HU)다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::{
    parser::tags,
    renderer::render_tree::{RenderNode, RenderNodeType},
    DocumentCore,
};

const HU_PER_PX: f64 = 75.0;

/// 호환 대상이 MS 워드인 실문서 HWP5 의 레이아웃 호환 플래그(사내 302건 중 279건의 값).
const WORD_LAYOUT_FLAGS: [u8; 20] = [
    0xe3, 0x01, 0, 0, 0xff, 0xf8, 0xcf, 0, 0x4f, 0, 0, 0, 0xec, 0x01, 0, 0, 0, 0, 0, 0,
];

fn hwp5_without_stored_lines(name: &str, layout_flags: [u8; 20]) -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8582")
        .join(format!("{name}.hwpx"));
    let bytes = std::fs::read(&path).expect("#8582 재현물");
    let mut doc = rhwp::parser::parse_document(&bytes).expect("HWPX 읽기");
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            para.line_segs.clear();
        }
    }
    // 실문서 HWP5 처럼 호환 대상(MS 워드)과 레이아웃 호환 플래그를 제자리에 둔다.
    for record in &mut doc.doc_info.extra_records {
        if record.tag_id == tags::HWPTAG_COMPATIBLE_DOCUMENT {
            record.data = 2u32.to_le_bytes().to_vec();
        }
        if record.tag_id == tags::HWPTAG_LAYOUT_COMPATIBILITY {
            record.data = layout_flags.to_vec();
        }
    }
    rhwp::serializer::serialize_document(&doc).expect("HWP5 저장")
}

fn line_top(node: &RenderNode, para: usize) -> Option<f64> {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para) && line.section_index == Some(0) {
            return Some(node.bbox.y);
        }
    }
    node.children.iter().find_map(|child| line_top(child, para))
}

#[test]
fn word_target_hwp5_empty_paragraphs_draw_with_scaled_height() {
    let bytes = hwp5_without_stored_lines("word_hcr_empty", WORD_LAYOUT_FLAGS);
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let reread = rhwp::parser::parse_hwp(&bytes).unwrap();
    assert!(
        reread.doc_info.extra_records.iter().any(|record| {
            record.tag_id == tags::HWPTAG_COMPATIBLE_DOCUMENT
                && record.data.get(..4) == Some(&2u32.to_le_bytes()[..])
        }),
        "호환 대상 MS 워드"
    );
    assert!(
        reread.sections[0]
            .paragraphs
            .iter()
            .all(|para| para.line_segs.is_empty()),
        "저장 줄 없는 HWP5"
    );
    assert_eq!(core.page_count(), 1, "한 쪽");
    let tree = core.build_page_render_tree(0).unwrap();
    let top = line_top(&tree.root, 0).expect("TOP 줄");
    let end = line_top(&tree.root, 17).expect("END 줄");
    let span = (end - top) * HU_PER_PX;
    assert!(
        (span - 32460.0).abs() <= 60.0,
        "빈 문단 16개를 지난 TOP→END {span:.0} HU, 데스크톱 한컴 32460 HU"
    );
}

/// 레이아웃 호환 플래그가 모두 0 인 HWP5 는 호환 대상이 MS 워드여도 한/글 줄 높이다.
#[test]
fn word_target_hwp5_without_layout_flags_keeps_hangul_height() {
    let bytes = hwp5_without_stored_lines("word_hcr_empty", [0; 20]);
    let core = DocumentCore::from_bytes(&bytes).unwrap();
    let tree = core.build_page_render_tree(0).unwrap();
    let top = line_top(&tree.root, 0).expect("TOP 줄");
    let end = line_top(&tree.root, 17).expect("END 줄");
    let span = (end - top) * HU_PER_PX;
    assert!(
        (span - 19200.0).abs() <= 60.0,
        "플래그 0 HWP5 의 TOP→END {span:.0} HU, 데스크톱 한컴 19172 HU"
    );
}

/// 실문서 짜임의 합성 서식(`word_real_layout_hwp5{b,c}.hwp`, 맑은 고딕 7·10·20pt·함초롬바탕
/// 11pt·글자처럼 취급한 1행 2열 표). 데스크톱 한컴 2020 엔진 인쇄는 레이아웃 호환 플래그가 켜진
/// 판(b)이 2쪽, 플래그 0 판(c)이 1쪽이다.
#[test]
fn word_real_layout_hwp5_page_count_follows_layout_flags() {
    for (name, pages) in [
        ("word_real_layout_hwp5b.hwp", 2),
        ("word_real_layout_hwp5c.hwp", 1),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue8582")
            .join(name);
        let bytes = std::fs::read(&path).expect("#8582 HWP5 재현물");
        let core = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(core.page_count(), pages, "{name}: 데스크톱 한컴 {pages}쪽");
    }
}

/// 레이아웃 호환 플래그 중 문단 수준(둘째 UINT32)의 비트 4(0x10)만 워드 줄 높이를 켠다. 그 비트만
/// 켠 판(`word_hcr_p100_hwp5_lcp_b4.hwp`)은 데스크톱 한컴 인쇄 피치가 약 1858 HU 이고, 같은 바이트의
/// 이웃 비트 3 만 켠 판(`..._lcp_b3.hwp`)은 한/글 줄 높이(약 1100 HU)다.
#[test]
fn paragraph_layout_flag_bit4_enables_word_line_height() {
    for (name, pitch) in [
        ("word_hcr_p100_hwp5_lcp_b4.hwp", 1860.0),
        ("word_hcr_p100_hwp5_lcp_b3.hwp", 1100.0),
    ] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("samples/issue8582")
            .join(name);
        let bytes = std::fs::read(&path).expect("#8582 HWP5 재현물");
        let core = DocumentCore::from_bytes(&bytes).unwrap();
        let tree = core.build_page_render_tree(0).unwrap();
        let first = line_top(&tree.root, 1).expect("첫 대상 줄");
        let last = line_top(&tree.root, 16).expect("마지막 대상 줄");
        let actual = (last - first) / 15.0 * HU_PER_PX;
        assert!(
            (actual - pitch).abs() <= 8.0,
            "{name}: 줄 피치 {actual:.0} HU, 데스크톱 한컴 {pitch:.0} HU"
        );
    }
}
