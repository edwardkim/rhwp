//! [#7336] 조각 컷 회계와 조각 페인트가 어긋나 본문이 통째로 사라지던 두 갈래.
//!
//! 공통 증상은 같다 — 페이지네이터가 조각에 배정한 높이와 페인터가 실제로 그리는
//! 상자 높이가 발산해, 그 차이만큼의 내용이 clip·쪽 밖으로 사라지고 다음 조각은
//! 이미 소비한 컷 **다음**부터 재개하므로 사라진 내용이 어느 쪽에도 남지 않는다.
//!
//! 1. `nested_table_fragment_cut.hwp` — 조각이 rowspan 블록 **안쪽**에서 끝나면
//!    (`end_row < block_end`) 블록-합 보정이 차액을 이 조각이 그리지 않는 행에 실었다.
//!    4쪽 실측: 예산 `consumed=603.6` vs 그린 상자 `tbl_h=81.8` — 걸침 셀이 소비한
//!    10×2 중첩 표가 칸 하단(486.2) 밖 `y=535.3` 에 놓여 통째로 clip 됐다.
//!    **클립은 페인트 단계라 `extract_page_text_native` 에는 남는다** — 그래서 이
//!    갈래는 텍스트가 아니라 조각 셀 bbox 와 중첩 표 bbox 의 포함 관계로 잠근다.
//! 2. `stored_frame_page_larger_rowbreak.hwpx` — 저장 RowBreak object frame 이
//!    선언 901.2px 로 "이 쪽을 소유한다"고 말하는데 실측 표는 3,676px(약 4쪽)였다.
//!    통째 배치로 본문 아래 2,742px 가 넘쳐 `2-3. 추진일정` 절 전체가 사라졌다.
//!
//! 두 문서 모두 `rhwp dump` 에는 내용이 남아 있어 파서가 아니라 조판·레이아웃 결함이다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const NESTED_CUT_SAMPLE: &str = "samples/issue7336/nested_table_fragment_cut.hwp";
const STORED_FRAME_SAMPLE: &str = "samples/issue7336/stored_frame_page_larger_rowbreak.hwpx";

#[test]
fn page_number_uses_the_document_page_number_style() {
    let doc = load_doc(NESTED_CUT_SAMPLE);
    let source_style = doc
        .document()
        .doc_info
        .styles
        .iter()
        .find(|style| style.local_name == "쪽 번호")
        .expect("원본 쪽 번호 스타일");
    let source_shape = &doc.document().doc_info.char_shapes[source_style.char_shape_id as usize];
    let source_font =
        &doc.document().doc_info.font_faces[1][source_shape.font_ids[1] as usize].name;
    let page = doc.build_page_render_tree(4).expect("5쪽 render tree");
    let footer = page
        .root
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Footer))
        .expect("footer");
    let number_node = footer
        .children
        .iter()
        .flat_map(|line| &line.children)
        .find(|node| matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.contains('5')))
        .expect("5쪽 쪽번호");
    let RenderNodeType::TextRun(number) = &number_node.node_type else {
        unreachable!("위에서 TextRun 으로 선택");
    };
    assert!(
        number.style.font_family.contains(source_font),
        "쪽번호의 영문 글꼴은 원본 스타일에서 가져와야 한다: source={source_font}, actual={}",
        number.style.font_family
    );
    // 한컴 2020/2024 PDF 양쪽에서 5쪽의 glyph 은 y=1052..1061px 이다.
    // 현재 글꼴의 렌더 잉크는 TextRun bbox 상단보다 약 3px 아래에 시작한다.
    assert!(
        (1048.0..=1049.5).contains(&number_node.bbox.y),
        "5쪽 쪽번호 세로 기준이 한컴 PDF 보다 높다: {:.1}px",
        number_node.bbox.y,
    );
}

fn load_doc(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&fs::read(&path).expect("read sample")).expect("open")
}

/// 쪽 텍스트에서 공백을 지운 형태로 비교한다 — 줄바꿈·칸 경계의 공백은 이 계약의
/// 대상이 아니고, 소실 여부만 판정한다.
fn squashed_page_text(doc: &DocumentCore, page: u32) -> String {
    doc.extract_page_text_native(page)
        .unwrap_or_else(|e| panic!("{}쪽 텍스트: {e}", page + 1))
        .split_whitespace()
        .collect()
}

fn assert_page_has(doc: &DocumentCore, page: u32, needle: &str) {
    let text = squashed_page_text(doc, page);
    let wanted: String = needle.split_whitespace().collect();
    assert!(
        text.contains(&wanted),
        "#7336: {}쪽에 {needle:?} 가 있어야 한다\n--- {}쪽 ---\n{text}",
        page + 1,
        page + 1
    );
}

fn bottom(bbox: &BoundingBox) -> f64 {
    bbox.y + bbox.height
}

/// 클립이 걸린 조각 셀 중 **중첩 표를 품은** 첫 셀과 그 중첩 표를 찾는다.
fn find_clipping_fragment_cell_with_nested_table(
    node: &RenderNode,
) -> Option<(&RenderNode, &RenderNode)> {
    if let RenderNodeType::TableCell(cell) = &node.node_type {
        if cell.clip && cell.page_fragment {
            if let Some(nested) = node
                .children
                .iter()
                .find(|child| matches!(child.node_type, RenderNodeType::Table(_)))
            {
                return Some((node, nested));
            }
        }
    }
    node.children
        .iter()
        .find_map(find_clipping_fragment_cell_with_nested_table)
}

fn has_exact_run(node: &RenderNode, text: &str) -> bool {
    matches!(&node.node_type, RenderNodeType::TextRun(run) if run.text.trim() == text)
        || node.children.iter().any(|child| has_exact_run(child, text))
}

fn table_for_para(node: &RenderNode, para: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(table) if table.para_index == Some(para)) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| table_for_para(child, para))
}

fn table_for_para_control(node: &RenderNode, para: usize, control: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(table) if table.para_index == Some(para) && table.control_index == Some(control))
    {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| table_for_para_control(child, para, control))
}

#[test]
fn nested_table_fragment_box_covers_the_cut_it_consumed() {
    let doc = load_doc(NESTED_CUT_SAMPLE);
    let page = doc.build_page_render_tree(3).expect("4쪽 render tree");
    let (cell, nested) =
        find_clipping_fragment_cell_with_nested_table(&page.root).expect("4쪽 조각 칸 + 중첩 표");
    assert!(
        bottom(&nested.bbox) <= bottom(&cell.bbox) + 1.0,
        "#7336: 조각 칸이 컷으로 소비한 중첩 표는 칸 클립 안에 있어야 한다 — \
         중첩 표 하단 {:.1}, 칸 하단 {:.1} (칸 y={:.1} h={:.1})",
        bottom(&nested.bbox),
        bottom(&cell.bbox),
        cell.bbox.y,
        cell.bbox.height,
    );
}

#[test]
fn nested_table_fragment_keeps_the_straddling_cell_content_on_page_four() {
    let doc = load_doc(NESTED_CUT_SAMPLE);
    assert_eq!(
        doc.page_count(),
        6,
        "#7336: 한/글 2024 는 이 문서를 6쪽으로 조판한다"
    );
    // 한/글 정본과 같은 4쪽(0-기반 3). 컷은 처음부터 이 쪽에 배정돼 있었고,
    // 결함은 그린 상자가 짧아 clip 으로 지워지는 것이었다.
    for needle in ["무형자산취득비", "기계장치비", "회계정산비"] {
        assert_page_has(&doc, 3, needle);
    }
    // 5쪽 이어짐 조각은 4쪽이 소비한 컷 **다음**부터 재개해야 한다.
    let fifth = squashed_page_text(&doc, 4);
    assert!(
        !fifth.contains("무형자산취득비"),
        "#7336: 5쪽이 4쪽 조각의 내용을 다시 그리면 안 된다\n--- 5쪽 ---\n{fifth}"
    );
}

#[test]
fn inline_shape_heading_follows_the_saved_page_reset() {
    let doc = load_doc(NESTED_CUT_SAMPLE);
    let fifth = doc.build_page_render_tree(4).expect("5쪽 render tree");
    let sixth = doc.build_page_render_tree(5).expect("6쪽 render tree");
    assert!(
        !has_exact_run(&fifth.root, "추진일정"),
        "#7336: 저장 vpos가 새 쪽으로 되감긴 인라인 제목은 5쪽 꼬리에 남으면 안 된다"
    );
    assert!(
        has_exact_run(&sixth.root, "추진일정"),
        "#7336: 인라인 제목은 한컴 기준과 같이 6쪽 첫 절이어야 한다"
    );
}

#[test]
fn text_after_inline_heading_keeps_page_relative_saved_vpos() {
    let doc = load_doc(NESTED_CUT_SAMPLE);
    let page = doc.build_page_render_tree(5).expect("6쪽 render tree");
    let body = page
        .root
        .children
        .iter()
        .find(|child| matches!(child.node_type, RenderNodeType::Body { .. }))
        .expect("body");
    fn text_line_for_para(node: &RenderNode, para: usize) -> Option<&RenderNode> {
        if matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(para))
        {
            return Some(node);
        }
        node.children
            .iter()
            .find_map(|child| text_line_for_para(child, para))
    }
    let schedule = text_line_for_para(body, 69).expect("heading-following body line");
    let saved_y = body.bbox.y + 6152.0 * 96.0 / 7200.0;
    assert!(
        (schedule.bbox.y - saved_y).abs() < 1.0,
        "#7336: 6쪽 첫 본문은 쪽 원점 + 6152HU이어야 한다: expected={saved_y:.1}, actual={:.1}",
        schedule.bbox.y,
    );
}

#[test]
fn native_multirow_fragment_reopens_saved_outer_top_margin() {
    let doc = load_doc(NESTED_CUT_SAMPLE);
    for (page, anchor_hu) in [(3, 21148.0), (4, 0.0)] {
        let tree = doc.build_page_render_tree(page).expect("table page");
        let body = tree
            .root
            .children
            .iter()
            .find(|child| matches!(child.node_type, RenderNodeType::Body { .. }))
            .expect("body");
        let table = table_for_para(body, 57).expect("continued RowBreak table");
        let expected = body.bbox.y + (anchor_hu + 283.0) * 96.0 / 7200.0;
        assert!(
            (table.bbox.y - expected).abs() < 0.3,
            "#7336: {}쪽 저장 RowBreak 조각 위 여백 283HU: expected={expected:.1}, actual={:.1}",
            page + 1,
            table.bbox.y,
        );
    }
}

#[test]
fn centered_rowspan_fragments_keep_the_saved_cell_height() {
    let doc = load_doc(NESTED_CUT_SAMPLE);
    let fourth = doc.build_page_render_tree(3).expect("4쪽 render tree");
    let fifth = doc.build_page_render_tree(4).expect("5쪽 render tree");
    let fourth_table = table_for_para(&fourth.root, 57).expect("first table fragment");
    let fifth_table = table_for_para(&fifth.root, 57).expect("last table fragment");
    fn spanning_cell(table: &RenderNode) -> Option<&RenderNode> {
        table.children.iter().find(|child| {
            matches!(&child.node_type, RenderNodeType::TableCell(cell) if cell.row == 1 && cell.col == 1)
        })
    }
    let first = spanning_cell(fourth_table).expect("4쪽 걸침 셀");
    let last = spanning_cell(fifth_table).expect("5쪽 걸침 셀");
    let saved_height = 64752.0 * 96.0 / 7200.0;
    assert!(
        (first.bbox.height + last.bbox.height - saved_height).abs() < 2.0,
        "#7336: 가운데 정렬 걸침 셀의 두 조각은 저장 높이 64752HU를 나눠 가져야 한다: first={:.1}, last={:.1}, saved={saved_height:.1}",
        first.bbox.height,
        last.bbox.height,
    );
}

#[test]
fn stored_frame_page_larger_rowbreak_table_is_split_across_pages() {
    let doc = load_doc(STORED_FRAME_SAMPLE);
    assert_eq!(
        doc.page_count(),
        7,
        "#7336: 한/글 2024 는 이 문서를 7쪽으로 조판한다 — 결함 시 4쪽(표를 통째로 얹고 쪽 밖으로 넘침)"
    );
}

#[test]
fn stored_frame_page_larger_rowbreak_keeps_the_schedule_section() {
    let doc = load_doc(STORED_FRAME_SAMPLE);
    // 정본과 같은 쪽 소속: 4쪽 '2-3. 추진일정', 5쪽 '총 합계'.
    assert_page_has(&doc, 3, "2-3. 추진일정");
    assert_page_has(&doc, 4, "총 합계");
}

#[test]
fn hwpx_continuation_fragment_reopens_saved_outer_top_margin() {
    let doc = load_doc(STORED_FRAME_SAMPLE);
    let second = doc.build_page_render_tree(1).expect("first fragment page");
    let second_body = second
        .root
        .children
        .iter()
        .find(|child| matches!(child.node_type, RenderNodeType::Body { .. }))
        .expect("body");
    let first = table_for_para(second_body, 3).expect("first RowBreak fragment");
    assert!(
        (first.bbox.y - second_body.bbox.y - (1760.0 + 283.0) * 96.0 / 7200.0).abs() < 0.3,
        "#7336: 2쪽 첫 표 조각은 저장 앵커 1760HU + 위 여백 283HU에 그려야 한다: body={:.1}, table={:.1}",
        second_body.bbox.y,
        first.bbox.y,
    );
    for page in [2, 3, 4] {
        let tree = doc.build_page_render_tree(page).expect("continuation page");
        let body = tree
            .root
            .children
            .iter()
            .find(|child| matches!(child.node_type, RenderNodeType::Body { .. }))
            .expect("body");
        let table = table_for_para(body, 3).expect("continued RowBreak table");
        assert!(
            (table.bbox.y - body.bbox.y - 283.0 * 96.0 / 7200.0).abs() < 0.3,
            "#7336: {}쪽 이어지는 표 상단은 본문 + 저장 283HU 여백이어야 한다: body={:.1}, table={:.1}",
            page + 1,
            body.bbox.y,
            table.bbox.y,
        );
    }
}

#[test]
fn hwpx_stored_inline_table_does_not_double_charge_last_line_spacing() {
    let doc = load_doc(STORED_FRAME_SAMPLE);
    let page = doc.build_page_render_tree(5).expect("6쪽 render tree");
    let table = table_for_para_control(&page.root, 4, 1).expect("6쪽 동의서 표");
    let header_cell = table
        .children
        .iter()
        .find(|child| matches!(&child.node_type, RenderNodeType::TableCell(cell) if cell.row == 0))
        .expect("first row cell");
    let saved_row_height = 3040.0 * 96.0 / 7200.0;
    assert!(
        (header_cell.bbox.height - saved_row_height).abs() < 0.3,
        "#7336: 저장 첫 행 3040HU와 PDF는 40.5px인데 마지막 줄간격을 재가산하면 안 된다: {:.1}px",
        header_cell.bbox.height,
    );
}
