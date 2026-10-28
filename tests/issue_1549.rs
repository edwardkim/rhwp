//! Issue #1549: 공동 앵커 표·제목·후속 본문의 실제 순서를 보존한다.
//! 원 이슈의 실문서 설명을 합성 입력의 정답으로 대신하지 않는다. 정확히 같은
//! HWPX의 한컴 2020 Print는 2쪽이며 세 표 아래에 제목, 그 아래에 본문을 둔다.
//! 전쪽 Native/fresh WASM 최저 97.31566/97.44110%와 직접 판독 근거:
//! `mydocs/pr/assets/issue_1549/stored_title_contract/`.
//!
//! fixture: samples/issue1549_multipositive_float_tables.hwpx
//!   = issue1510 구조에서 float 표 offset 을 모두 작은 양수로 narrow
//!     (A=+200, B=+500, C=+800 HWPUNIT) — 표 top 이 제목 라인과 겹치는 multi-positive 케이스.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const HWPX_SAMPLE: &str = "samples/issue1549_multipositive_float_tables.hwpx";
const EMPTY_HOST_SAMPLE: &str = "samples/issue1549_empty_host_float_clamp.hwpx";
const TARGET_PI: usize = 0;
const TITLE_NEEDLE: &str = "MULTI POSITIVE TITLE";
const TARGET_TABLES: [usize; 3] = [2, 3, 4];

fn load_doc(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", sample, e))
}

fn find_table_bbox(root: &RenderNode, target_ci: usize) -> Option<(f64, f64)> {
    if let RenderNodeType::Table(table) = &root.node_type {
        if table.para_index == Some(TARGET_PI) && table.control_index == Some(target_ci) {
            return Some((root.bbox.y, root.bbox.y + root.bbox.height));
        }
    }
    for child in &root.children {
        if let Some(found) = find_table_bbox(child, target_ci) {
            return Some(found);
        }
    }
    None
}

fn find_table_bbox_by_para(root: &RenderNode, para_index: usize) -> Option<(f64, f64)> {
    if let RenderNodeType::Table(table) = &root.node_type {
        if table.para_index == Some(para_index) {
            return Some((root.bbox.y, root.bbox.y + root.bbox.height));
        }
    }
    for child in &root.children {
        if let Some(found) = find_table_bbox_by_para(child, para_index) {
            return Some(found);
        }
    }
    None
}

fn find_title_bbox(root: &RenderNode, needle: &str) -> Option<(f64, f64)> {
    if let RenderNodeType::TextRun(run) = &root.node_type {
        if run.para_index.is_some() && run.text.contains(needle) {
            return Some((root.bbox.y, root.bbox.y + root.bbox.height));
        }
    }
    for child in &root.children {
        if let Some(found) = find_title_bbox(child, needle) {
            return Some(found);
        }
    }
    None
}

#[test]
fn issue_1549_saved_coanchored_tables_precede_their_host_title_and_body() {
    let doc = load_doc(HWPX_SAMPLE);
    assert_eq!(doc.page_count(), 2, "정확한 원본의 독립 Print 전체 쪽수");
    let tree = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");

    let (title_top, title_bottom) =
        find_title_bbox(&tree.root, TITLE_NEEDLE).expect("host title text bbox");
    let tables: Vec<(f64, f64)> = TARGET_TABLES
        .iter()
        .map(|&ci| find_table_bbox(&tree.root, ci).unwrap_or_else(|| panic!("table ci={ci} bbox")))
        .collect();
    for adjacent in tables.windows(2) {
        assert!(
            adjacent[0].1 <= adjacent[1].0 + 0.5,
            "원본 A·B·C 표의 순서·비겹침"
        );
    }
    let last_table_bottom = tables.last().expect("원본 세 표").1;
    assert!(
        title_top + 0.5 >= last_table_bottom,
        "독립 Print의 제목은 마지막 표 아래: 표 하단 {last_table_bottom}, 제목 {title_top}"
    );
    let (body_top, _) = find_title_bbox(&tree.root, "filler paragraph 01").expect("첫 후속 본문");
    assert!(body_top + 0.5 >= title_bottom, "후속 본문은 제목 아래");

    fn text(node: &RenderNode) -> String {
        let mut out = match &node.node_type {
            RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
            _ => String::new(),
        };
        for child in &node.children {
            out.push_str(&text(child));
        }
        out
    }
    let content = text(&tree.root);
    for label in [
        "A11",
        "A12",
        "A21",
        "A22",
        "B11",
        "B12",
        "B21",
        "B22",
        "C11",
        TITLE_NEEDLE,
    ] {
        assert_eq!(
            content.matches(label).count(),
            1,
            "1쪽 내용 단일 소유: {label}"
        );
    }
    let mut previous = 0;
    for number in 1..=30 {
        let label = format!("filler paragraph {number:02}");
        assert_eq!(
            content.matches(&label).count(),
            1,
            "후속 본문 누락·중복: {label}"
        );
        let index = content.find(&label).unwrap();
        assert!(index >= previous, "후속 문단 원문 순서: {label}");
        previous = index + label.len();
    }
    let last = doc
        .build_page_render_tree(1)
        .expect("독립 Print의 마지막 쪽");
    let tail = text(&last.root);
    assert_eq!(content.matches("filler paragraph").count(), 30);
    assert_eq!(tail.matches("filler paragraph").count(), 2);
    for number in 1..=32 {
        let label = format!("filler paragraph {number:02}");
        let (owner, other) = if number <= 30 {
            (&content, &tail)
        } else {
            (&tail, &content)
        };
        assert_eq!(
            owner.matches(&label).count(),
            1,
            "독립 Print의 쪽 소속: {label}"
        );
        assert!(!other.contains(&label), "쪽 경계를 넘은 중복: {label}");
    }
    assert!(tail.find("filler paragraph 31") < tail.find("filler paragraph 32"));

    // 같은 종료 동작의 완료 2쪽은 양쪽 raw 100%다. PDF 글자 원점·기준선을
    // 72pt→96dpi로 환산하며, 98% 미만인 1쪽에는 절대 좌표를 추가하지 않는다.
    fn check_tail_baseline(node: &RenderNode, found: &mut usize) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            let expected = match run.display_or_text() {
                "filler paragraph 31" => Some((31, 143.523_153)),
                "filler paragraph 32" => Some((32, 164.779_917)),
                _ => None,
            };
            if let Some((paragraph, baseline)) = expected {
                assert_eq!(run.para_index, Some(paragraph), "원본 문단 소유");
                assert!(
                    (node.bbox.x - 113.378_550).abs() <= 2.0,
                    "독립 Print 글자 원점"
                );
                assert!(
                    (node.bbox.y + run.baseline - baseline).abs() <= 2.0,
                    "독립 Print 기준선"
                );
                *found += 1;
            }
        }
        for child in &node.children {
            check_tail_baseline(child, found);
        }
    }
    let mut found = 0;
    check_tail_baseline(&last.root, &mut found);
    assert_eq!(found, 2, "마지막 두 문단의 줄이 한 번씩 있어야 한다");
}

/// 별도 빈 host의 후행 표는 선행 float 점유 영역과 겹치지 않는다.
/// 위 합성 입력의 제목 순서 교정을 이 독립적인 표 간 관계에 확대 적용하지 않는다.
///
/// fixture: 문단0 = visible host 제목 + 양수 offset float 표 A,
///          문단1 = 빈 host(text 0) + 양수 offset float 표 C (A 밴드와 겹치는 자연위치).
#[test]
fn issue_1549_empty_host_following_float_clears_preceding_band() {
    let doc = load_doc(EMPTY_HOST_SAMPLE);
    let tree = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");

    let (a_top, a_bottom) =
        find_table_bbox_by_para(&tree.root, 0).expect("para0 visible-host float table A");
    let (c_top, _) =
        find_table_bbox_by_para(&tree.root, 1).expect("para1 empty-host float table C");

    assert!(
        c_top + 0.5 >= a_bottom,
        "empty-host following float table must clear the preceding float band, \
         not overlap it: a=({a_top:.1}..{a_bottom:.1}), c_top={c_top:.1}",
    );
}
