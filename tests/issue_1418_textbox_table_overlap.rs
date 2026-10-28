//! Issue #1418: Paper 기준 InFrontOfText 글상자 host 빈 문단의 line advance 보존.
//!
//! `samples/2026_oss_rst.hwp` 1페이지에서 제목 글상자는 Paper 기준으로 고정되고,
//! 바로 뒤 큰 1x1 TAC 표는 직전 빈 host 문단의 line advance 뒤에서 시작해야 한다.
//! 정답 PDF 기준 큰 표 상단선은 제목 흰 배경 중앙 근처인 y≈153.4 px를 지난다.
//! 회귀 상태에서는 host 문단 진행량 21.3 px가 layout에서 사라져 표가 y=132.3 px에 시작한다.

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/2026_oss_rst.hwp";
const TARGET_PAGE: u32 = 0;
const TITLE_TEXT: &str = "< 결과보고서 작성 안내 >";

fn find_table_bbox(root: &RenderNode, target_pi: usize, target_ci: usize) -> Option<BoundingBox> {
    if let RenderNodeType::Table(table) = &root.node_type {
        if table.para_index == Some(target_pi) && table.control_index == Some(target_ci) {
            return Some(root.bbox);
        }
    }

    for child in &root.children {
        if let Some(found) = find_table_bbox(child, target_pi, target_ci) {
            return Some(found);
        }
    }

    None
}

fn node_text(node: &RenderNode) -> String {
    let mut text = String::new();
    collect_text(node, &mut text);
    text
}

fn collect_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_text(child, out);
    }
}

fn find_title_textbox_bbox(root: &RenderNode) -> Option<BoundingBox> {
    if matches!(root.node_type, RenderNodeType::TextBox) && node_text(root).contains(TITLE_TEXT) {
        return Some(root.bbox);
    }

    for child in &root.children {
        if let Some(found) = find_title_textbox_bbox(child) {
            return Some(found);
        }
    }

    None
}

#[test]
fn issue_1418_first_page_table_top_overlaps_title_textbox_center() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));

    assert_eq!(
        doc.page_count(),
        6,
        "교체된 정답 fixture는 6페이지 문서여야 함"
    );

    let tree = doc
        .build_page_render_tree(TARGET_PAGE)
        .expect("build_page_render_tree page 1");

    let table_bbox =
        find_table_bbox(&tree.root, 1, 0).expect("pi=1 ci=0 큰 1x1 안내 표 Table 노드 누락");
    let textbox_bbox = find_title_textbox_bbox(&tree.root).expect("제목 글상자 TextBox 노드 누락");

    eprintln!(
        "[issue_1418] table_top={:.2} table_h={:.2} title_textbox=[x={:.2} y={:.2} w={:.2} h={:.2}]",
        table_bbox.y,
        table_bbox.height,
        textbox_bbox.x,
        textbox_bbox.y,
        textbox_bbox.width,
        textbox_bbox.height,
    );

    assert!(
        (table_bbox.y - 153.4).abs() <= 2.0,
        "큰 표 상단선이 제목 글상자 중앙 근처(y≈153.4)에 있어야 함. 실제 y={:.2}. \
         회귀 상태는 host 빈 문단 21.3px 진행량이 빠져 y≈132.3에 시작한다.",
        table_bbox.y,
    );

    assert!(
        (textbox_bbox.y - 137.2).abs() <= 2.0,
        "Paper 기준 제목 글상자 자체는 기존 위치(y≈137.2)에 남아야 함. 실제 y={:.2}",
        textbox_bbox.y,
    );
}

/// 독립 Print의 마지막 세 행은3쪽에서 닫히고 다음 구역은 곧바로4쪽에서 시작한다.
/// 전6쪽 Native 최저95.88324/fresh WASM96.56689%, 3쪽 양쪽99.99680%와 직접 판독.
/// 근거: stage8 C446 및 oss-terminal-cursor 증적의 원본 TSV/review/overlay.
#[test]
fn completed_terminal_cut_keeps_the_next_section_on_the_adjacent_page() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let bytes = fs::read(root.join(SAMPLE)).expect("원본 OSS 결과보고서");
    let core = rhwp::DocumentCore::from_bytes(&bytes).expect("원본 문서 파싱");
    let ending = core.build_page_render_tree(2).expect("표의 마지막 쪽");
    let following = core
        .build_page_render_tree(3)
        .expect("바로 뒤 구역의 첫 쪽");
    let compact = |node: &RenderNode| node_text(node).split_whitespace().collect::<String>();
    let last_text = compact(&ending.root);
    let next_text = compact(&following.root);
    for marker in ["프로젝트주요기능", "기대효과및활용분야", "소감및후기"] {
        assert!(
            last_text.contains(marker),
            "마지막 표 조각의 원문 누락: {marker}"
        );
    }
    assert!(
        next_text.contains("SBOM(소프트웨어자재명세서)"),
        "내용 컷·물리 잔여가 모두 끝난 뒤 항목 없는 중간 쪽을 만들지 않는다"
    );
    assert!(!next_text.contains("소감및후기"), "끝 행의 중복 이월 금지");
    let table = find_table_bbox(&ending.root, 7, 0).expect("원본 문단7 표0의 끝 조각");
    // pdf/hwpx/2026_oss_rst.pdf 3쪽의 원래 벡터 선, pt→96CSSpx.
    // 독립 폭·끝 행 프레임을 내용 높이로 압축해 다음 구역 쪽수만 맞추지 않는다.
    for (label, actual, independent) in [
        ("left", table.x, 113.058_666_666_666_67),
        ("top", table.y, 132.176),
        ("right", table.x + table.width, 680.434_666_666_666_7),
        ("bottom", table.y + table.height, 999.226_666_666_666_7),
    ] {
        assert!(
            (actual - independent).abs() <= 2.0,
            "독립 Print의 {label} ±2CSSpx: {actual} vs {independent}"
        );
    }
}
