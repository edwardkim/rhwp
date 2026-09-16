//! #3798: 쪽말의 비가시 간격과 실제 글자 영역을 구분한다.
//!
//! #7195 작업지시자 시각 판정(2026-09-16): 이 샘플의 현재 WASM 배치는
//! 한컴편집기와 동일하다. 경계 문단은 1쪽, 뒤 문단은 2쪽에 배치한다.
//! 이전의 "말미 간격 trim 상수 한도" 실험은 채택되지 않았다.
//! 실험/반증은 mydocs/report/task3798/ 및 Git 이력에 보존한다.
//!
//! 페이지 fit은 실제 줄 영역으로, 후속 배치는 소비 높이로 판정해야 한다.
//! 이 계약은 비가시 간격만 넘는 경우를 허용하되 글자/후속 문단 초과는 금지한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue3798/page_end_trailing_spill.hwpx";
const FIXTURE_TRAILING_SPACING_PX: f64 = 40.0;
const BOUNDARY: &str = "경계문단";

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return run.text.clone();
    }
    node.children.iter().map(text).collect()
}

fn normalized(s: &str) -> String {
    s.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn body(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node);
    }
    node.children.iter().find_map(body)
}

fn check_lines(node: &RenderNode, top: f64, bottom: f64) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && !text(node).trim().is_empty() {
        assert!(
            node.visible
                && node.bbox.y >= top - 0.5
                && node.bbox.y + node.bbox.height <= bottom + 0.5,
            "실제 글줄은 본문 안이어야 함: {:?}, body={top}..{bottom}",
            node.bbox
        );
    }
    for child in &node.children {
        check_lines(child, top, bottom);
    }
}

fn check_layout(bottom_margin_delta: u32, boundary_page: usize) {
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let mut core = DocumentCore::from_bytes(&bytes).expect("파싱");
    if bottom_margin_delta != 0 {
        let mut doc = core.document().clone();
        doc.sections[0].section_def.page_def.margin_bottom += bottom_margin_delta;
        core.set_document(doc);
    }
    let expected: String = core.document().sections[0]
        .paragraphs
        .iter()
        .map(|p| normalized(&p.text))
        .collect();
    assert_eq!(core.page_count(), 2, "합성 표본의 앞채움/경계/뒤 문단 구성");
    let mut page_texts = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("render");
        let b = body(&tree.root).expect("body");
        check_lines(b, b.bbox.y, b.bbox.y + b.bbox.height);
        page_texts.push(normalized(&text(b)));
    }
    // IR 원문 전체와 비교하여 어느 문단의 중복/누락도 허용하지 않는다.
    assert_eq!(page_texts.concat(), expected, "원문 순서/중복/누락");
    assert!(page_texts[0].contains("앞채움28"));
    assert!(page_texts[boundary_page].contains(BOUNDARY));
    assert!(!page_texts[1 - boundary_page].contains(BOUNDARY));
    for i in 1..=6 {
        let marker = format!("다음쪽본문{i:02}");
        assert!(
            !page_texts[0].contains(&marker),
            "말미 소비 간격을 무시하지 않음"
        );
        assert!(page_texts[1].contains(&marker));
    }
}

#[test]
fn invisible_trailing_spacing_keeps_the_fitting_line_and_advances_following_content() {
    check_layout(0, 0);
}

/// 반례: 가용 높이를32px 줄이면 896+16px의 실제 경계 줄조차 들어가지 않는다.
/// 비가시 말미 간격을 흘리는 규칙을 글자 영역 초과 허용으로 확대하지 않는다.
/// 이는 합성 계약이며 변형 표본의 한컴 시각 검증을 주장하지 않는다.
#[test]
fn actual_line_overflow_moves_the_boundary_paragraph() {
    check_layout(2400, 1);
}

#[test]
fn fixture_has_forty_pixel_trailing_spacing() {
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let doc = rhwp::parser::parse_document(&bytes).expect("파싱");
    let widest = doc.sections[0]
        .paragraphs
        .iter()
        .filter_map(|p| p.line_segs.last().map(|s| s.line_spacing as f64 / 75.0))
        .fold(0.0_f64, f64::max);
    assert!(
        (widest - FIXTURE_TRAILING_SPACING_PX).abs() < 1.0,
        "독립 생성된 표본의 말미 간격: {widest}px"
    );
}
