//! 칸 안 문단이 인라인(글자처럼) 중첩 표를 **둘 이상** 품을 때의 줄바꿈.
//!
//! `nested_tac_table_wrap.hwp`(bizbc_11060 (양식)사업계획서(신청기업), 6쪽)의
//! '1. 수행기관 주요 업무실적' 칸은 한 문단에 안내 상자(1×1)와 실적표(12×6)를
//! 나란히 담는다. 저장 `LINE_SEG` 는 두 줄(`lh=14179`, `lh=19921`)로 둘이 서로
//! 다른 줄에 있음을 증언한다.
//!
//! 수정 전: `table_layout` 의 인라인 표 분기가 `inline_x` 를 오른쪽으로만 밀어
//! 둘째 표가 x≈718px(용지 폭 793.7px) 에서 시작했고, 칸 clip 밖이라 괘선도
//! 글자도 남지 않았다 — PDF 에서 12×6 표가 통째로 사라진다(한/글 2024 는
//! 정상 출력이라는 원 PR의 보고). 이번 검증은 engine 2020으로 생성한 독립 PDF를 사용한다.
//!
//! 메인터너 보정: 저장 UTF-16 줄 소속을 폭·가로 정렬·vpos·여백에서 함께
//! 소비한다. 둘째 표의 전체 프레임과 내용이 첫 표 아래 부모 셀 안에 남는지 검사한다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/nested_tac_table_wrap.hwp";
/// 뷰어 6쪽 = 0-based 5.
const PAGE_INDEX: u32 = 5;

fn load_doc() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("parse sample")
}

fn collect_text_runs(node: &RenderNode, out: &mut Vec<(f64, f64, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((node.bbox.x, node.bbox.y, run.display_or_text().to_string()));
    }
    for child in &node.children {
        collect_text_runs(child, out);
    }
}

/// 본문 영역의 오른쪽 끝 — 인라인 접기가 없으면 둘째 표가 이 밖으로 나간다.
fn body_right(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.x + node.bbox.width);
    }
    node.children.iter().find_map(body_right)
}

fn host_cell(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::TableCell(_))
        && node
            .children
            .iter()
            .filter(|child| matches!(child.node_type, RenderNodeType::Table(_)))
            .count()
            == 2
    {
        return Some(node);
    }
    node.children.iter().find_map(host_cell)
}

fn direct_tables(cell: &RenderNode) -> Vec<&RenderNode> {
    cell.children
        .iter()
        .filter(|child| matches!(child.node_type, RenderNodeType::Table(_)))
        .collect()
}

#[test]
fn second_inline_nested_table_stays_inside_the_body_column() {
    let doc = load_doc();
    let page = doc
        .build_page_render_tree(PAGE_INDEX)
        .expect("page 6 render tree");
    let right_edge = body_right(&page.root).expect("본문 영역 노드가 없다");
    let host = host_cell(&page.root).expect("두 중첩 표를 소유한 셀");
    let tables = direct_tables(host);
    assert_eq!(tables.len(), 2);
    for table in tables {
        let bbox = table.bbox;
        assert!(bbox.width > 0.0 && bbox.height > 0.0);
        assert!(bbox.x >= host.bbox.x && bbox.y >= host.bbox.y);
        assert!(bbox.x + bbox.width <= host.bbox.x + host.bbox.width);
        assert!(bbox.x + bbox.width <= right_edge);
        assert!(bbox.y + bbox.height <= host.bbox.y + host.bbox.height);
    }
    let mut runs = Vec::new();
    collect_text_runs(&page.root, &mut runs);

    // 둘째 중첩 표(12×6)의 제목 행과 본문 행 글자. 수정 전에는 첫 표 폭만큼
    // 밀려 x≈754px(본문 오른쪽 끝 737px 밖)에 있었고 칸 clip 으로 소실됐다.
    for needle in ["사업명칭", "총 건수", "비고"] {
        let (x, _, _) = runs
            .iter()
            .find(|(_, _, text)| text.contains(needle))
            .unwrap_or_else(|| panic!("6쪽에 '{needle}' 글자가 없다 — 표본 쪽 번호를 확인하라"));
        assert!(
            *x < right_edge,
            "둘째 인라인 중첩 표의 '{needle}' 이 본문 밖이다 — x={x:.1}, 본문 오른쪽 끝 {right_edge:.1}px. \
             수정 전 결함은 첫 표 폭만큼 `inline_x` 를 밀어 표를 통째로 clip 밖으로 보냈다",
        );
    }
}

#[test]
fn second_inline_nested_table_is_below_the_first() {
    let doc = load_doc();
    let page = doc
        .build_page_render_tree(PAGE_INDEX)
        .expect("page 6 render tree");
    let host = host_cell(&page.root).expect("두 중첩 표를 소유한 셀");
    let tables = direct_tables(host);
    assert_eq!(tables.len(), 2);
    assert!(
        tables[1].bbox.y >= tables[0].bbox.y + tables[0].bbox.height,
        "독립 한컴 PDF와 저장 줄 순서: 둘째 표 전체 프레임은 안내 상자 아래에 있어야 한다"
    );
    let mut runs = Vec::new();
    collect_text_runs(&page.root, &mut runs);

    // 첫 중첩 표(안내 상자)의 마지막 줄과 둘째 표 제목 행의 세로 순서.
    let guide_bottom = runs
        .iter()
        .filter(|(_, _, text)| text.contains("제안과제와"))
        .map(|(_, y, _)| *y)
        .fold(f64::MIN, f64::max);
    assert!(
        guide_bottom > f64::MIN,
        "안내 상자 마지막 내용이 있어야 한다"
    );
    let header_y = runs
        .iter()
        .find(|(_, _, text)| text.contains("사업명칭"))
        .map(|(_, y, _)| *y)
        .expect("'사업명칭' 없음");
    assert!(
        header_y > guide_bottom,
        "둘째 표가 첫 표 아래 줄에 와야 한다 — 안내 상자 마지막 줄 y={guide_bottom:.1}, \
         표 제목 행 y={header_y:.1}",
    );
}
