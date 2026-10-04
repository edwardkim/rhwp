//! Issue #2319 — 저장 lineseg 없는(기계생성) 문서의 텍스트-host tac 표 높이 붕괴 회귀.
//!
//! `samples/task2319/20544835_jinan_apt_form.hwp` (진안군 공동주택 지원 신청서)
//! 전 문단 lineseg 부재. pi=0 = 제목 텍스트 + 13×11 신청서 표(tac, 실측 858px).
//!
//! 회귀 시그니처 (수정 전, 10k r15 재검증 −1 계열 26건):
//! - typeset_tac_table 의 fmt 기반 높이 분기가 표 높이를 제목 줄높이(17.3px)로 채택
//! - "TAC 표 높이 보정" cap 이 tac_seg_total=0(lineseg 부재) → fmt.total_height(34.7px)
//!   로 누적을 되감음
//! - 결과: 후속 표 4개(133/120/120/371px)까지 전부 1쪽에 스택 — rhwp 1쪽 vs 한글 2쪽
//!
//! 고정: 총 2쪽 + 신청서 표(pi=0)는 p1, 안내 표(pi=3)는 p2.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

fn core() -> DocumentCore {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join("samples/task2319/20544835_jinan_apt_form.hwp");
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", path.display(), e));
    DocumentCore::from_bytes(&bytes).expect("parse 20544835_jinan_apt_form.hwp")
}

#[test]
fn issue_2319_form_doc_paginates_to_two_pages() {
    let core = core();
    assert_eq!(
        core.page_count(),
        2,
        "한글 오라클 2쪽 정합 (수정 전 1쪽: tac 표 높이 붕괴로 전부 스택)"
    );
}

#[test]
fn issue_2319_form_table_and_guide_tables_split() {
    let core = core();
    // Inspect the final painted tables, regardless of which PageItem owns
    // their paragraph. The independent contract is page and object ownership.
    fn tables<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(node.node_type, RenderNodeType::Table(_)) {
            out.push(node);
            return;
        }
        for child in &node.children {
            tables(child, out);
        }
    }
    assert_eq!(core.page_count(), 2, "한컴 기준 2쪽");
    let first = core.build_page_render_tree(0).expect("신청서 쪽");
    let second = core.build_page_render_tree(1).expect("안내 쪽");
    let mut first_tables = Vec::new();
    let mut second_tables = Vec::new();
    tables(&first.root, &mut first_tables);
    tables(&second.root, &mut second_tables);
    let form: Vec<_> = first_tables
        .iter()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(0) && t.control_index == Some(2)
                && t.row_count == 13 && t.col_count == 11)
        })
        .collect();
    assert_eq!(form.len(), 1, "신청서 표의 쪽 소속·누락·중복");
    assert!(form[0].bbox.height > 800.0, "신청서 표 높이 붕괴 방지");
    assert!(
        matches!(&second_tables.first().expect("안내 표").node_type,
        RenderNodeType::Table(t) if t.para_index == Some(3) && t.control_index == Some(0)),
        "2쪽 첫 표는 안내 표여야 한다"
    );
    assert!(
        !second_tables.iter().any(
            |node| matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(0))
        ),
        "신청서 표를 다음 쪽에 중복하지 않는다"
    );
}
