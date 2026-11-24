//! 관계형·고정형 회귀 — #7776 채용 공고의 조각 순서·중첩 표와 독립 Print의 전체 쪽수.
//! 주석 갱신 2026-11-23 KST. 직전 파일 변경 2026-10-04T19:03:36+09:00
//! `731de9e1b4bb946d76f35108ed7e186ebe4ebecb`.
//! 2026-11-23 한컴 2020 Print, 96 DPI 2px raw 실루엣. 35쪽 Native/fresh WASM 최저 98.38/98.51%.
//! 3쪽 99.70/99.69%, 5쪽 99.99/100.00%, 7쪽 99.54/99.53%, 8쪽 98.98/98.98%.
//! 근거는 `mydocs/pr/assets/issue7776/`의 전쪽 TSV와 확대 비교다.
//!
//! Issue #1686: 같은 문단에 co-anchored 된 다중 RowBreak 표에서 선행 표가
//! continuation으로 분할될 때, 후행 표가 다음 섹션 제목보다 먼저 배치되면 안 된다.
//!
//! `pr-1674` 샘플의 0.27 문단에는 7x5 직위별 요건 표와 3x3
//! `[응시자격요건 고려사항]` 표가 함께 있다. 한컴 2020 PDF 기준 page 3은
//! 직위별 요건 표 continuation 뒤에 0.28 `다. 우대요건...` 섹션이 와야 한다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

const TARGET_PAGE: u32 = 2;
const PAGE_4: u32 = 3;
const PAGE_5: u32 = 4;
const EXPECTED_SECTION: &str = "다.우대요건등[원서접수마감일기준]";
const DEFERRED_TABLE_HEADING: &str = "[응시자격요건고려사항]";
const EXPECTED_PAGE_COUNT: u32 = 35;
const PAGE5_FIRST_CAREER_LINE: &str = "동일기간에경력이중복될경우유리한경력1개만인정함";
const FOLLOWUP_NOTICE: &str = "임용예정직위,응시자격요건및우대요건등관련사항은";

fn load_doc(sample: &str) -> HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {}: {}", sample, e))
}

fn collect_page_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_page_text(child, out);
    }
}

fn normalized_page_text(doc: &HwpDocument, page: u32) -> String {
    let tree = doc
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("build_page_render_tree({page}): {e}"));
    let mut text = String::new();
    collect_page_text(&tree.root, &mut text);
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn assert_page3_keeps_reward_section_before_following_table(sample: &str) {
    let doc = load_doc(sample);
    assert!(
        doc.page_count() > TARGET_PAGE,
        "{sample} should have at least three pages"
    );

    let page_text = normalized_page_text(&doc, TARGET_PAGE);
    let expected_idx = page_text.find(EXPECTED_SECTION).unwrap_or_else(|| {
        panic!(
            "{sample} page 3 should contain `{EXPECTED_SECTION}` before the following co-anchored table. page_text={page_text}"
        )
    });

    if let Some(deferred_idx) = page_text.find(DEFERRED_TABLE_HEADING) {
        assert!(
            expected_idx < deferred_idx,
            "{sample} page 3 placed `{DEFERRED_TABLE_HEADING}` before `{EXPECTED_SECTION}`. page_text={page_text}"
        );
    }
}

fn assert_pr1674_keeps_pdf_page_count_and_page5_boundary(sample: &str) {
    let doc = load_doc(sample);
    assert_eq!(
        doc.page_count(),
        EXPECTED_PAGE_COUNT,
        "{sample} should match the HWP 2024/PDF oracle page count"
    );

    let page4_text = normalized_page_text(&doc, PAGE_4);
    assert!(
        !page4_text.contains(PAGE5_FIRST_CAREER_LINE),
        "{sample} page 4 consumed the first page-5 career line. page4_text={page4_text}"
    );

    let page5_text = normalized_page_text(&doc, PAGE_5);
    assert!(
        page5_text.contains(PAGE5_FIRST_CAREER_LINE),
        "{sample} page 5 should start with the career-range continuation. page5_text={page5_text}"
    );
    assert!(
        !page5_text.contains(FOLLOWUP_NOTICE),
        "{sample} page 5 rendered the follow-up notice before the RowBreak table finished. page5_text={page5_text}"
    );
}

#[test]
fn issue_1686_hwpx_page3_keeps_reward_section_before_following_coanchored_table() {
    assert_page3_keeps_reward_section_before_following_table("samples/hwpx/pr-1674.hwpx");
}

#[test]
fn issue_1686_hwpx_matches_pdf_page_count_and_page5_boundary() {
    assert_pr1674_keeps_pdf_page_count_and_page5_boundary("samples/hwpx/pr-1674.hwpx");
}

#[test]
fn issue_1686_hwp_page3_keeps_reward_section_before_following_coanchored_table() {
    assert_page3_keeps_reward_section_before_following_table("samples/pr-1674.hwp");
}

#[test]
fn issue_1686_hwp_matches_pdf_page_count_and_page5_boundary() {
    assert_pr1674_keeps_pdf_page_count_and_page5_boundary("samples/pr-1674.hwp");
}

fn body_table(page: &RenderNode, paragraph: usize) -> &RenderNode {
    page.children.iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .flat_map(|body| &body.children)
        .filter(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .flat_map(|column| &column.children)
        .find(|node| matches!(&node.node_type, RenderNodeType::Table(table) if table.para_index == Some(paragraph)))
        .expect("본문 단에 속한 표")
}

fn cell_line<'a>(cell: &'a RenderNode, needle: &str) -> &'a RenderNode {
    let mut owners = cell.children.iter().filter(|node| {
        if !matches!(node.node_type, RenderNodeType::TextLine(_)) {
            return false;
        }
        let text: String = node
            .children
            .iter()
            .filter_map(|run| match &run.node_type {
                RenderNodeType::TextRun(text) => Some(text.text.as_str()),
                _ => None,
            })
            .collect();
        text.contains(needle)
    });
    let line = owners.next().expect("지정한 셀의 설명 글줄");
    assert!(
        owners.next().is_none(),
        "{needle} 글줄은 셀에서 한 번만 놓인다"
    );
    line
}

#[test]
fn pr1674_nested_grids_and_the_following_text_share_the_saved_cell_origin() {
    let source = include_bytes!("../samples/pr-1674.hwp");
    let document = DocumentCore::from_bytes(source).expect("채용 공고 HWP");
    let pages = [
        document.build_page_render_tree(6).unwrap(),
        document.build_page_render_tree(7).unwrap(),
    ];
    let section = &document.document().sections[0];
    let host7 = &section.paragraphs[41];
    let [Control::Table(source7)] = host7.controls.as_slice() else {
        panic!("7쪽 외곽 표")
    };
    let model_cell7 = &source7.cells[5];
    let body7 = body_table(&pages[0].root, 41);
    let cell7 = body7
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type,
        RenderNodeType::TableCell(cell) if cell.row == 2 && cell.col == 0)
        })
        .expect("경력기준 셀");
    let grid7 = cell7
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type,
        RenderNodeType::Table(table) if table.row_count == 5 && table.col_count == 5)
        })
        .expect("경력기준 중첩 표");
    let note1 = cell_line(cell7, "(우대요건1)");
    let note2 = cell_line(cell7, "(우대요건2)");
    let stored_note_distance = f64::from(
        model_cell7.paragraphs[3].line_segs[0].vertical_pos
            - model_cell7.paragraphs[2].line_segs[0].vertical_pos,
    ) / 75.0;
    assert!(grid7.bbox.y + grid7.bbox.height <= note1.bbox.y);
    assert!(
        (note2.bbox.y - note1.bbox.y - stored_note_distance).abs() <= 1.0 / 75.0,
        "중첩 표 뒤의 두 설명이 같은 셀의 저장 글줄 간격을 유지한다"
    );
    assert!(note2.bbox.y + note2.bbox.height <= cell7.bbox.y + cell7.bbox.height);

    let host8 = &section.paragraphs[58];
    let [Control::Table(source8)] = host8.controls.as_slice() else {
        panic!("8쪽 외곽 표")
    };
    let model_cell8 = &source8.cells[0];
    let [Control::Table(source_grid8)] = model_cell8.paragraphs[1].controls.as_slice() else {
        panic!("8쪽 저장 내부 표")
    };
    let title_frame = &model_cell8.paragraphs[0];
    let title_line = &title_frame.line_segs[0];
    // HWP5 문단 간격의 저장 단위는 1/2 HU다.
    let gap_after_title =
        document.document().doc_info.para_shapes[title_frame.para_shape_id as usize].spacing_after
            / 2;
    let body8 = body_table(&pages[1].root, 58);
    let cell8 = body8
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::TableCell(_)))
        .expect("세부평가 셀");
    let grid8 = cell8
        .children
        .iter()
        .find(|node| {
            matches!(&node.node_type,
        RenderNodeType::Table(table) if table.row_count==2 && table.col_count==2)
        })
        .expect("세부평가 중첩 표");
    let title8 = cell_line(cell8, "전문지식과 그 응용능력");
    let followup = cell_line(cell8, "평가방법:");
    let stored_title_to_grid = f64::from(
        title_line.line_height
            + title_line.line_spacing
            + gap_after_title
            + i32::from(source_grid8.outer_margin_top),
    ) / 75.0;
    assert!(
        (grid8.bbox.y - title8.bbox.y - stored_title_to_grid).abs() <= 1.0 / 75.0,
        "제목 전진과 문단 뒤 간격이 중첩 표의 원점을 한 번 정한다"
    );
    assert!(grid8.bbox.y + grid8.bbox.height <= followup.bbox.y);
    assert!(followup.bbox.y + followup.bbox.height <= cell8.bbox.y + cell8.bbox.height);
}
