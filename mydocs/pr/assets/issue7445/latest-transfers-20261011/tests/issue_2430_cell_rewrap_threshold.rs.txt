//! [#2430] PR #2309(커밋 6d910836) 의 셀 저장-ls1 문단 폭초과 재래핑 허용이
//! 한글과 정합이던 분할 표 문서를 과다분할로 회귀시킨 건에 대한 회귀 가드.
//!
//! 근인: `recompose_stored_single_line_if_overflowing` 의 발동 임계가 실폭 >
//! 내폭 ×1.05 로 너무 느슨해, 측정/렌더 패딩 발산(#2237)으로 살짝(1.05~1.35×)
//! 초과한 정합 셀까지 재래핑해 줄수를 부풀리고 쪽당 표 행 적재를 떨어뜨렸다.
//! 임계를 ×1.8 로 좁혀(#2525 body 판과 동일) 거짓 재래핑을 제거한다. #2291
//! 원 타깃(76자 1-lineseg = ~7.6× 초과)은 임계 위라 계속 재래핑(절단 방지
//! 유지 — issue_2287/issue_2291 테스트가 별도 고정).
//!
//! 대표: `1382000_중간보고자료_2022_가정폭력실태조사` — 한글 39쪽, 임계
//! ×1.05 에서 40쪽(+1) 과다분할, ×1.8 에서 39쪽 정합. (10k 서베이 순변화
//! +4/회귀 0 으로 무회귀 확인.)

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

fn find_table(root: &RenderNode, para_index: usize, control_index: usize) -> Option<&RenderNode> {
    if matches!(
        &root.node_type,
        RenderNodeType::Table(table)
            if table.para_index == Some(para_index)
                && table.control_index == Some(control_index)
    ) {
        return Some(root);
    }
    root.children
        .iter()
        .find_map(|child| find_table(child, para_index, control_index))
}

fn text_line_bbox_containing(root: &RenderNode, needle: &str) -> Option<BoundingBox> {
    if matches!(root.node_type, RenderNodeType::TextLine(_))
        && root.children.iter().any(
            |child| matches!(&child.node_type, RenderNodeType::TextRun(run) if run.text.contains(needle)),
        )
    {
        return Some(root.bbox);
    }
    root.children
        .iter()
        .find_map(|child| text_line_bbox_containing(child, needle))
}

#[test]
fn issue_2430_cell_rewrap_threshold_no_oversplit() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/task2430/1382000_domestic_violence_survey.hwp");
    let bytes = std::fs::read(&path).expect("read fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    let pages = core.page_count();
    // 한글 정답 39쪽. 셀 재래핑 임계가 ×1.05 로 되돌아가면 40쪽(+1) 과다분할한다.
    // #4069 저장 프레임 전파가 p14 셀의 무시되는 빈 Enter를 경계로 오인하면
    // 반대로 38쪽(-1)으로 줄어드는 것도 이 계약이 함께 검출한다.
    assert_eq!(
        pages, 39,
        "1382000은 한글 기준 39쪽이어야 함 (40쪽=재래핑, 38쪽=빈 Enter 프레임 오인). #2430 #4069"
    );
}

#[test]
fn issue_2430_page16_keeps_leading_blank_paragraph_in_cell() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/task2430/1382000_domestic_violence_survey.hwp");
    let bytes = std::fs::read(&path).expect("read fixture");
    let core = DocumentCore::from_bytes(&bytes).expect("parse");
    let page16 = core
        .build_page_render_tree(15)
        .expect("render physical page 16");
    let table = find_table(&page16.root, 90, 0).expect("page 16 outer 1x1 table pi=90 ci=0");
    let cell = table
        .children
        .iter()
        .find(|node| matches!(&node.node_type, RenderNodeType::TableCell(cell) if cell.row == 0 && cell.col == 0))
        .expect("outer table cell (0,0)");
    let first_text = text_line_bbox_containing(cell, "연구과제명")
        .expect("first visible paragraph after the leading empty Enter");
    let top_gap = first_text.y - cell.bbox.y;

    // 한컴 2020 PDF 물리 16쪽은 셀의 p[0] 빈 Enter를 한 줄로 조판한 뒤
    // p[1] `○ 연구과제명`을 시작한다. 수정 전에는 빈 문단을 0높이로 접어
    // top_gap=0.9px였고, 정본은 약 27px다.
    assert!(
        top_gap >= 20.0,
        "셀 (0,0)의 선두 빈 Enter가 조판되지 않았다: table_top={:.2}, text_top={:.2}, gap={top_gap:.2}",
        cell.bbox.y,
        first_text.y
    );
}
