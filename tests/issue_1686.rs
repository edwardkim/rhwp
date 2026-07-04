//! Issue #1686: 같은 문단에 co-anchored 된 다중 RowBreak 표에서 선행 표가
//! continuation으로 분할될 때, 후행 표가 다음 섹션 제목보다 먼저 배치되면 안 된다.
//!
//! `pr-1674` 샘플의 0.27 문단에는 7x5 직위별 요건 표와 3x3
//! `[응시자격요건 고려사항]` 표가 함께 있다. 한컴 2020 PDF 기준 page 3은
//! 직위별 요건 표 continuation 뒤에 0.28 `다. 우대요건...` 섹션이 와야 한다.

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
fn issue_1686_hwp_page3_keeps_reward_section_before_following_coanchored_table() {
    assert_page3_keeps_reward_section_before_following_table("samples/pr-1674.hwp");
}

#[test]
fn issue_1686_hwpx_matches_pdf_page_count_and_page5_boundary() {
    assert_pr1674_keeps_pdf_page_count_and_page5_boundary("samples/hwpx/pr-1674.hwpx");
}

#[test]
fn issue_1686_hwp_matches_pdf_page_count_and_page5_boundary() {
    assert_pr1674_keeps_pdf_page_count_and_page5_boundary("samples/pr-1674.hwp");
}
