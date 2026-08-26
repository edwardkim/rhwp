//! Regression guards for `samples/rowbreak-problem-pages.hwpx`.
//!
//! The first chart-like TAC table on page 2 (`pi=5 ci=0`) must start below the
//! preceding `<민간 SaaS 연계공통기반 운영체계>` title line. Otherwise the chart
//! border and image are painted under that title text.

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/rowbreak-problem-pages.hwpx";
const HWP_SAMPLE: &str = "samples/rowbreak-problem-pages.hwp";
const PAGE_INDEX: u32 = 1;

fn load_doc(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let sample_path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&sample_path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {sample}: {e:?}"))
}

fn find_table_bbox(root: &RenderNode, target_pi: usize, target_ci: usize) -> Option<BoundingBox> {
    if let RenderNodeType::Table(t) = &root.node_type {
        if t.para_index == Some(target_pi) && t.control_index == Some(target_ci) {
            return Some(root.bbox);
        }
    }

    root.children
        .iter()
        .find_map(|child| find_table_bbox(child, target_pi, target_ci))
}

fn find_table_node(root: &RenderNode, target_pi: usize, target_ci: usize) -> Option<&RenderNode> {
    if let RenderNodeType::Table(t) = &root.node_type {
        if t.para_index == Some(target_pi) && t.control_index == Some(target_ci) {
            return Some(root);
        }
    }

    root.children
        .iter()
        .find_map(|child| find_table_node(child, target_pi, target_ci))
}

fn find_body_bbox(root: &RenderNode) -> Option<BoundingBox> {
    if matches!(root.node_type, RenderNodeType::Body { .. }) {
        return Some(root.bbox);
    }

    root.children.iter().find_map(find_body_bbox)
}

fn find_body_node(root: &RenderNode) -> Option<&RenderNode> {
    if matches!(root.node_type, RenderNodeType::Body { .. }) {
        return Some(root);
    }

    root.children.iter().find_map(find_body_node)
}

fn find_textrun_bbox_containing(root: &RenderNode, needle: &str) -> Option<BoundingBox> {
    if let RenderNodeType::TextRun(run) = &root.node_type {
        if run.text.contains(needle) {
            return Some(root.bbox);
        }
    }

    root.children
        .iter()
        .find_map(|child| find_textrun_bbox_containing(child, needle))
}

fn max_text_line_bottom(root: &RenderNode) -> Option<f64> {
    let own_bottom = if matches!(root.node_type, RenderNodeType::TextLine(_)) {
        Some(root.bbox.y + root.bbox.height)
    } else {
        None
    };

    root.children
        .iter()
        .filter_map(max_text_line_bottom)
        .fold(own_bottom, |acc, bottom| {
            Some(acc.map_or(bottom, |current| current.max(bottom)))
        })
}

fn collect_rectangles_with_text<'a>(root: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(root.node_type, RenderNodeType::Rectangle(_))
        && max_text_line_bottom(root).is_some()
    {
        out.push(root);
    }

    for child in &root.children {
        collect_rectangles_with_text(child, out);
    }
}

#[test]
fn rowbreak_page11_partial_table_stays_inside_body() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let sample_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&sample_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", SAMPLE, e));
    let tree = doc
        .build_page_render_tree(10)
        .unwrap_or_else(|e| panic!("render page 11: {e}"));

    let body = find_body_bbox(&tree.root).expect("page 11 body should render");
    let table = find_table_bbox(&tree.root, 5, 0).expect("page 11 table pi=5 ci=0 should render");

    let body_bottom = body.y + body.height;
    let table_bottom = table.y + table.height;
    assert!(
        table_bottom <= body_bottom + 0.5,
        "page 11 table is clipped: table bottom={table_bottom:.2}, body bottom={body_bottom:.2}"
    );
}

#[test]
fn rowbreak_page13_following_reference_strip_stays_below_table() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let sample_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&sample_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", SAMPLE, e));
    let tree = doc
        .build_page_render_tree(12)
        .unwrap_or_else(|e| panic!("render page 13: {e}"));

    let reference_strip =
        find_table_bbox(&tree.root, 11, 0).expect("page 13 reference strip pi=11 ci=0");
    let table = find_table_bbox(&tree.root, 11, 1).expect("page 13 table pi=11 ci=1");

    let table_bottom = table.y + table.height;
    assert!(
        reference_strip.y >= table_bottom - 0.5,
        "page 13 reference strip overlaps table: table=[{:.2}..{:.2}], strip_y={:.2}",
        table.y,
        table_bottom,
        reference_strip.y
    );
}

#[test]
fn rowbreak_page13_textbox_shapes_cover_their_text() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let sample_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&sample_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", SAMPLE, e));
    let tree = doc
        .build_page_render_tree(12)
        .unwrap_or_else(|e| panic!("render page 13: {e}"));
    let table = find_table_node(&tree.root, 13, 0).expect("page 13 excerpt table pi=13 ci=0");

    let mut rectangles = Vec::new();
    collect_rectangles_with_text(table, &mut rectangles);
    let wide_text_rectangles: Vec<_> = rectangles
        .into_iter()
        .filter(|node| node.bbox.width > 300.0 && node.bbox.height > 20.0)
        .collect();

    assert!(
        !wide_text_rectangles.is_empty(),
        "page 13 should render textbox-backed rectangles inside the excerpt table"
    );
    for rect in wide_text_rectangles {
        let rect_bottom = rect.bbox.y + rect.bbox.height;
        let text_bottom = max_text_line_bottom(rect).expect("rectangle should contain text lines");
        assert!(
            rect_bottom >= text_bottom - 0.5,
            "textbox-backed rectangle clips text: rect=[{:.2}..{:.2}], text_bottom={text_bottom:.2}",
            rect.bbox.y,
            rect_bottom
        );
    }
}

#[test]
fn rowbreak_page13_preserves_linear_empty_spacer_in_excerpt_table() {
    for sample in [SAMPLE, HWP_SAMPLE] {
        let doc = load_doc(sample);
        let tree = doc
            .build_page_render_tree(12)
            .unwrap_or_else(|e| panic!("render {sample} page 13: {e}"));
        let table = find_table_node(&tree.root, 13, 0).expect("page 13 excerpt table pi=13 ci=0");

        let mut rectangles = Vec::new();
        collect_rectangles_with_text(table, &mut rectangles);
        let mut wide_text_rectangles: Vec<_> = rectangles
            .into_iter()
            .filter(|node| node.bbox.width > 300.0 && node.bbox.height > 20.0)
            .collect();
        wide_text_rectangles.sort_by(|a, b| a.bbox.y.partial_cmp(&b.bbox.y).unwrap());
        let first_textbox = wide_text_rectangles
            .first()
            .unwrap_or_else(|| panic!("{sample} page 13 should render the first blue textbox"));
        let table_bottom = table.bbox.y + table.bbox.height;

        assert!(
            first_textbox.bbox.y >= 572.0,
            "{sample} page 13 collapses the caption spacer before the first textbox: first_textbox_y={:.2}",
            first_textbox.bbox.y
        );
        assert!(
            table_bottom >= 995.0,
            "{sample} page 13 excerpt table is too short after spacer collapse: table_bottom={table_bottom:.2}"
        );
    }
}

#[test]
fn rowbreak_page17_split_table_covers_visible_textbox_shape() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let sample_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&sample_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", SAMPLE, e));
    let tree = doc
        .build_page_render_tree(16)
        .unwrap_or_else(|e| panic!("render page 17: {e}"));
    let table = find_table_node(&tree.root, 28, 0).expect("page 17 table pi=28 ci=0");

    let mut rectangles = Vec::new();
    collect_rectangles_with_text(table, &mut rectangles);
    let wide_text_rectangles: Vec<_> = rectangles
        .into_iter()
        .filter(|node| node.bbox.width > 300.0 && node.bbox.height > 100.0)
        .collect();

    assert!(
        !wide_text_rectangles.is_empty(),
        "page 17 should render the large textbox-backed rectangle in table pi=28 ci=0"
    );

    let table_bottom = table.bbox.y + table.bbox.height;
    for rect in wide_text_rectangles {
        let rect_bottom = rect.bbox.y + rect.bbox.height;
        assert!(
            table_bottom >= rect_bottom - 0.5,
            "page 17 split table clips visible textbox shape: table=[{:.2}..{:.2}], rect=[{:.2}..{:.2}]",
            table.bbox.y,
            table_bottom,
            rect.bbox.y,
            rect_bottom
        );
    }
}

#[test]
fn rowbreak_page18_does_not_emit_tiny_empty_table_continuation() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let sample_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&sample_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", SAMPLE, e));
    let tree = doc
        .build_page_render_tree(17)
        .unwrap_or_else(|e| panic!("render page 18: {e}"));

    if let Some(table) = find_table_node(&tree.root, 28, 0) {
        assert!(
            table.bbox.height > 100.0,
            "page 18 should not be a tiny empty continuation of table pi=28 ci=0: height={:.2}",
            table.bbox.height
        );
        assert!(
            max_text_line_bottom(table).is_some(),
            "page 18 continuation of table pi=28 ci=0 should contain visible content"
        );
    }
}

#[test]
fn rowbreak_final_pages_match_hancom_pdf_page_count() {
    for sample in [SAMPLE, HWP_SAMPLE] {
        let doc = load_doc(sample);
        assert_eq!(
            doc.page_count(),
            18,
            "{sample} should match the 18-page Hancom PDF reference"
        );
    }
}

#[test]
fn rowbreak_page17_keeps_final_database_table_tail_like_hancom_pdf() {
    let doc = load_doc(SAMPLE);
    let page17 = doc
        .build_page_render_tree(16)
        .unwrap_or_else(|e| panic!("render page 17: {e}"));
    let page18 = doc
        .build_page_render_tree(17)
        .unwrap_or_else(|e| panic!("render page 18: {e}"));

    assert!(
        text_line_exists(&page17.root, "오픈API 개발"),
        "Hancom PDF page 17 contains the final database table tail; rhwp must not defer it"
    );
    assert!(
        text_line_exists(&page18.root, "보안 분야"),
        "Hancom PDF page 18 starts the security section"
    );
    assert!(
        find_table_node(&page18.root, 28, 0).is_none(),
        "page 18 should not be another continuation of table pi=28 ci=0"
    );
}

#[test]
fn rowbreak_page17_keeps_database_separation_line_before_example_box() {
    for sample in [SAMPLE, HWP_SAMPLE] {
        let doc = load_doc(sample);
        let page17 = doc
            .build_page_render_tree(16)
            .unwrap_or_else(|e| panic!("render {sample} page 17: {e}"));
        // `별도`만 검색하면 후속 예시 표의 "별도 개방DB"를 잘못 집을 수 있다.
        // PDF p17에서 예시 상자 바로 앞에 있어야 하는 정확한 source line을 고정한다.
        let database_line =
            text_line_bbox_containing(&page17.root, "공공데이터를 별도 테이블(table)로 구성·설계")
                .unwrap_or_else(|| {
                    panic!("{sample} page 17 should render the separate-table line")
                });
        let example_caption = text_line_bbox_containing(&page17.root, "예시")
            .unwrap_or_else(|| panic!("{sample} page 17 should render the example caption"));
        let database_line_bottom = database_line.y + database_line.height;

        assert!(
            example_caption.y >= database_line_bottom - 0.5,
            "{sample} page 17 overlaps the separate-table line with the example caption: line=[{:.2}..{database_line_bottom:.2}], caption_y={:.2}",
            database_line.y,
            example_caption.y
        );
    }
}

fn collect_table_cells<'a>(
    root: &'a RenderNode,
    target_pi: usize,
    target_ci: usize,
) -> Vec<&'a RenderNode> {
    if let RenderNodeType::Table(t) = &root.node_type {
        if t.para_index == Some(target_pi) && t.control_index == Some(target_ci) {
            return root
                .children
                .iter()
                .filter(|child| matches!(child.node_type, RenderNodeType::TableCell(_)))
                .collect();
        }
    }

    root.children
        .iter()
        .find_map(|child| {
            let cells = collect_table_cells(child, target_pi, target_ci);
            (!cells.is_empty()).then_some(cells)
        })
        .unwrap_or_default()
}

fn collect_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        collect_text(child, out);
    }
}

fn text_line_exists(root: &RenderNode, needle: &str) -> bool {
    if matches!(root.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        collect_text(root, &mut text);
        if text.contains(needle) {
            return true;
        }
    }

    root.children
        .iter()
        .any(|child| text_line_exists(child, needle))
}

fn text_line_bbox_containing(root: &RenderNode, needle: &str) -> Option<BoundingBox> {
    if matches!(root.node_type, RenderNodeType::TextLine(_)) {
        let mut text = String::new();
        collect_text(root, &mut text);
        if text.contains(needle) {
            return Some(root.bbox);
        }
    }

    root.children
        .iter()
        .find_map(|child| text_line_bbox_containing(child, needle))
}

/// [#4334] "중첩 표"를 원래 para/control 인덱스가 없는 표(`is_none()`)로 찾았는데,
/// #4334 가 `layout_embedded_table`(TAC 중첩 표)/재귀 중첩 표의 host 경로 플러밍
/// 결손을 고치면서 그 인덱스가 채워져 후보가 0개가 됐다 — 이 실패 자체가 수정이
/// 실제로 이 표(page 8 "이어지는 참조 각주" 셀 안 TAC 중첩 표)에 적용됐다는 증거다.
///
/// `cell_context.is_some()` 로 바꿨다 — 이게 원래 `is_none()` 이 대신 쓰던 신호의
/// 진짜 의미다: "이 표가 다른 셀/글상자 안에 중첩돼 있는가?" (#4334 가 `TableNode`
/// 에 새로 채운 필드, `ImageNode.cell_context` 와 같은 다단계 경로). 이 함수는
/// 이미 `root`(특정 셀 서브트리)로 스코프가 좁혀진 채 호출되므로(두 호출부 모두
/// `row26_detail` 셀), 실측(HWPX: para=1,ctrl=0,cell_context=Some) 으로 그 스코프
/// 안 유일한 표가 실제로 중첩 표임을 확인했다 — #1486 케이스와 달리 특정 인덱스
/// 값을 하드코딩하지 않는다: 이 함수는 HWP/HWPX 두 변형에서 재사용되는데 같은
/// 의미 내용이라도 포맷별 내부 문단 인덱스가 다를 수 있어(#1486 은 한 포맷 한
/// 표라 값 고정이 안전했지만 여기는 아니다) 인덱스 유무가 아니라 "중첩 여부"라는
/// 원래 의도를 그대로 유지하는 조건이 맞다.
fn first_nested_table_bbox(root: &RenderNode) -> Option<BoundingBox> {
    for child in &root.children {
        if let RenderNodeType::Table(t) = &child.node_type {
            if t.cell_context.is_some() {
                return Some(child.bbox);
            }
        }
        if let Some(bbox) = first_nested_table_bbox(child) {
            return Some(bbox);
        }
    }

    None
}

#[test]
fn rowbreak_page2_chart_starts_below_title_line() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let sample_path = Path::new(repo_root).join(SAMPLE);
    let bytes = fs::read(&sample_path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {:?}", SAMPLE, e));
    let tree = doc
        .build_page_render_tree(PAGE_INDEX)
        .unwrap_or_else(|e| panic!("render page {}: {}", PAGE_INDEX + 1, e));

    let chart =
        find_table_bbox(&tree.root, 5, 0).expect("page 2 chart table pi=5 ci=0 should render");
    let title = find_textrun_bbox_containing(&tree.root, "연계공통기반 운영체계")
        .expect("page 2 chart title text should render");

    let title_bottom = title.y + title.height;
    assert!(
        chart.y >= title_bottom - 0.5,
        "page 2 chart overlaps title text: title=[{:.2}..{:.2}], chart_y={:.2}",
        title.y,
        title_bottom,
        chart.y,
    );
}

#[test]
fn rowbreak_page7_nested_table_paragraph_keeps_host_text() {
    let doc = load_doc(SAMPLE);
    let page7 = doc
        .build_page_render_tree(6)
        .unwrap_or_else(|e| panic!("render page 7: {e}"));
    let page8 = doc
        .build_page_render_tree(7)
        .unwrap_or_else(|e| panic!("render page 8: {e}"));

    let cells = collect_table_cells(&page7.root, 21, 0);
    assert!(
        !cells.is_empty(),
        "page 7 rowbreak table pi=21 ci=0 should render cells"
    );
    assert!(
        cells
            .iter()
            .any(|cell| text_line_exists(cell, "1. 「정보통신망")),
        "row 25 should keep the host paragraph text before its nested reference table"
    );
    let row25_detail = cells
        .iter()
        .find(|cell| matches!(&cell.node_type, RenderNodeType::TableCell(c) if c.row == 2 && c.col == 1))
        .expect("page 7 row 25 detail cell should render");
    let row26_detail = cells
        .iter()
        .find(|cell| matches!(&cell.node_type, RenderNodeType::TableCell(c) if c.row == 3 && c.col == 1))
        .expect("page 7 row 26 detail cell should render");
    let row25_text_bottom =
        max_text_line_bottom(row25_detail).expect("page 7 row 25 detail cell should contain text");
    assert!(
        row25_text_bottom <= row26_detail.bbox.y + 0.5,
        "row 25 text overlaps row 26 on page 7: row25 text bottom={:.2}, row26 top={:.2}",
        row25_text_bottom,
        row26_detail.bbox.y
    );

    let page8_cells = collect_table_cells(&page8.root, 21, 0);
    let page8_top_detail = page8_cells
        .iter()
        .find(|cell| matches!(&cell.node_type, RenderNodeType::TableCell(c) if c.row == 3 && c.col == 1))
        .expect("page 8 continued row detail cell should render");
    let following = text_line_bbox_containing(page8_top_detail, "과학기술정보통신부장관")
        .expect("page 8 continued row should render the paragraph after the dotted fragment");
    assert!(
        following.y >= page8_top_detail.bbox.y - 0.5,
        "page 8 continued paragraph is clipped above the detail cell: text_top={:.2}, cell_top={:.2}",
        following.y,
        page8_top_detail.bbox.y
    );
    let page8_cell_bottom = page8_top_detail.bbox.y + page8_top_detail.bbox.height;
    let following_bottom = following.y + following.height;
    assert!(
        following_bottom <= page8_cell_bottom + 0.5,
        "page 8 continued paragraph is clipped below the detail cell: text_bottom={following_bottom:.2}, cell_bottom={page8_cell_bottom:.2}"
    );
}

#[test]
fn rowbreak_hwpx_page1_ignores_stale_initial_vpos() {
    let doc = load_doc(SAMPLE);
    let page1 = doc
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render HWPX page 1: {e}"));
    let body = find_body_node(&page1.root).expect("HWPX page 1 body should render");
    let definition = text_line_bbox_containing(&page1.root, "(정의)")
        .expect("HWPX page 1 definition paragraph should render");

    assert!(
        definition.y <= body.bbox.y + 90.0,
        "HWPX page 1 stale vpos pushes the first content down: body_top={:.2}, definition_y={:.2}",
        body.bbox.y,
        definition.y
    );
}

#[test]
fn rowbreak_hwpx_page5_table_split_keeps_page6_notes_off_page5() {
    let doc = load_doc(SAMPLE);
    let page4 = doc
        .build_page_render_tree(3)
        .unwrap_or_else(|e| panic!("render HWPX page 4: {e}"));
    let page5 = doc
        .build_page_render_tree(4)
        .unwrap_or_else(|e| panic!("render HWPX page 5: {e}"));
    let page6 = doc
        .build_page_render_tree(5)
        .unwrap_or_else(|e| panic!("render HWPX page 6: {e}"));

    assert!(
        !text_line_exists(&page4.root, "테스트키 배포"),
        "HWPX page 4 should not pull the next-page table header/row forward"
    );
    assert!(
        text_line_exists(&page5.root, "테스트키 배포"),
        "HWPX page 5 should start the continued table at 연계 개발 및 테스트"
    );
    assert!(
        !text_line_exists(&page5.root, "가용성, 응답성"),
        "HWPX page 5 should not pull the page 6 quality notes forward"
    );
    assert!(
        text_line_exists(&page6.root, "가용성, 응답성"),
        "HWPX page 6 should start with the quality/performance note"
    );
}

#[test]
fn rowbreak_hwpx_page8_keeps_continued_nested_reference_line() {
    let doc = load_doc(SAMPLE);
    let page8 = doc
        .build_page_render_tree(7)
        .unwrap_or_else(|e| panic!("render HWPX page 8: {e}"));

    let cells = collect_table_cells(&page8.root, 21, 0);
    let row26_detail = cells
        .iter()
        .find(|cell| matches!(&cell.node_type, RenderNodeType::TableCell(c) if c.row == 3 && c.col == 1))
        .expect("HWPX page 8 row 26 detail cell should render");
    let line = text_line_bbox_containing(row26_detail, "매개하는 자를")
        .expect("HWPX page 8 should keep the first continued nested reference line");
    let following = text_line_bbox_containing(row26_detail, "과학기술정보통신부장관")
        .expect("HWPX page 8 should render the paragraph after the continued nested reference");
    let nested_table =
        first_nested_table_bbox(row26_detail).expect("HWPX page 8 continued nested table bbox");
    let cell_bottom = row26_detail.bbox.y + row26_detail.bbox.height;
    let line_bottom = line.y + line.height;
    let nested_bottom = nested_table.y + nested_table.height;

    assert!(
        line.y >= row26_detail.bbox.y - 0.5,
        "HWPX page 8 continued line is clipped above the cell: line_top={:.2}, cell_top={:.2}",
        line.y,
        row26_detail.bbox.y
    );
    assert!(
        line_bottom <= cell_bottom + 0.5,
        "HWPX page 8 continued line is clipped below the cell: line_bottom={:.2}, cell_bottom={cell_bottom:.2}",
        line_bottom
    );
    assert!(
        following.y >= line_bottom - 0.5,
        "HWPX page 8 continued line overlaps the following paragraph: line_bottom={line_bottom:.2}, following_top={:.2}",
        following.y
    );
    assert!(
        nested_bottom <= following.y + 0.5,
        "HWPX page 8 continued nested table border includes the following paragraph: nested_bottom={nested_bottom:.2}, following_top={:.2}",
        following.y
    );
}

#[test]
fn rowbreak_hwpx_page10_keeps_csap_table_near_title() {
    let doc = load_doc(SAMPLE);
    let page10 = doc
        .build_page_render_tree(9)
        .unwrap_or_else(|e| panic!("render HWPX page 10: {e}"));
    let body = find_body_node(&page10.root).expect("HWPX page 10 body should render");
    let title = text_line_bbox_containing(&page10.root, "서비스 보안인증제도")
        .expect("HWPX page 10 title should render");
    let table = find_table_bbox(&page10.root, 1, 0).expect("HWPX page 10 CSAP table should render");

    assert!(
        table.y <= body.bbox.y + 120.0,
        "HWPX page 10 stale vpos pushes the CSAP table to the page bottom: body_top={:.2}, table_y={:.2}",
        body.bbox.y,
        table.y
    );
    assert!(
        table.y >= title.y + title.height - 0.5,
        "HWPX page 10 CSAP table overlaps the title: title=[{:.2}..{:.2}], table_y={:.2}",
        title.y,
        title.y + title.height,
        table.y
    );
}

#[test]
fn rowbreak_page12_reference_text_stays_inside_body() {
    let doc = load_doc(SAMPLE);
    let page12 = doc
        .build_page_render_tree(11)
        .unwrap_or_else(|e| panic!("render page 12: {e}"));
    let body = find_body_node(&page12.root).expect("page 12 body should render");
    let body_bottom = body.bbox.y + body.bbox.height;
    let text_bottom =
        max_text_line_bottom(body).expect("page 12 body should contain visible text lines");

    assert!(
        text_bottom <= body_bottom + 0.5,
        "page 12 text is clipped by the Body clip: text_bottom={text_bottom:.2}, body_bottom={body_bottom:.2}"
    );
}

#[test]
fn rowbreak_hwp_page8_keeps_continued_nested_reference_line() {
    let doc = load_doc(HWP_SAMPLE);
    let page8 = doc
        .build_page_render_tree(7)
        .unwrap_or_else(|e| panic!("render HWP page 8: {e}"));

    let cells = collect_table_cells(&page8.root, 21, 0);
    let row26_detail = cells
        .iter()
        .find(|cell| matches!(&cell.node_type, RenderNodeType::TableCell(c) if c.row == 3 && c.col == 1))
        .expect("HWP page 8 row 26 detail cell should render");
    let line = text_line_bbox_containing(row26_detail, "매개하는 자를")
        .expect("HWP page 8 should keep the first continued nested reference line");
    let following = text_line_bbox_containing(row26_detail, "과학기술정보통신부장관")
        .expect("HWP page 8 should render the paragraph after the continued nested reference");
    let nested_table =
        first_nested_table_bbox(row26_detail).expect("HWP page 8 continued nested table bbox");
    let cell_bottom = row26_detail.bbox.y + row26_detail.bbox.height;
    let line_bottom = line.y + line.height;
    let nested_bottom = nested_table.y + nested_table.height;

    assert!(
        line.y >= row26_detail.bbox.y - 0.5,
        "HWP page 8 continued line is clipped above the cell: line_top={:.2}, cell_top={:.2}",
        line.y,
        row26_detail.bbox.y
    );
    assert!(
        line_bottom <= cell_bottom + 0.5,
        "HWP page 8 continued line is clipped below the cell: line_bottom={:.2}, cell_bottom={cell_bottom:.2}",
        line_bottom
    );
    assert!(
        following.y >= line_bottom - 0.5,
        "HWP page 8 continued line overlaps the following paragraph: line_bottom={line_bottom:.2}, following_top={:.2}",
        following.y
    );
    assert!(
        nested_bottom <= following.y + 0.5,
        "HWP page 8 continued nested table border includes the following paragraph: nested_bottom={nested_bottom:.2}, following_top={:.2}",
        following.y
    );
}

#[test]
fn rowbreak_hwp_page12_reference_text_stays_inside_body() {
    let doc = load_doc(HWP_SAMPLE);
    let page12 = doc
        .build_page_render_tree(11)
        .unwrap_or_else(|e| panic!("render HWP page 12: {e}"));
    let body = find_body_node(&page12.root).expect("HWP page 12 body should render");
    let body_bottom = body.bbox.y + body.bbox.height;
    let text_bottom =
        max_text_line_bottom(body).expect("HWP page 12 body should contain visible text lines");

    assert!(
        text_bottom <= body_bottom + 0.5,
        "HWP page 12 text is clipped by the Body clip: text_bottom={text_bottom:.2}, body_bottom={body_bottom:.2}"
    );
}

#[test]
fn rowbreak_page7_starts_article_26_like_hancom_pdf() {
    let doc = load_doc(SAMPLE);
    let page7 = doc
        .build_page_render_tree(6)
        .unwrap_or_else(|e| panic!("render page 7: {e}"));

    let cells = collect_table_cells(&page7.root, 21, 0);
    assert!(
        cells.iter().any(|cell| text_line_exists(cell, "제26조")),
        "Hancom PDF page 7 starts article 26 in table pi=21; rhwp should not stop at article 25"
    );
}

#[test]
fn rowbreak_page7_keeps_tail_line_before_large_table_like_hancom_pdf() {
    let doc = load_doc(SAMPLE);
    let page7 = doc
        .build_page_render_tree(6)
        .unwrap_or_else(|e| panic!("render page 7: {e}"));

    assert!(
        text_line_exists(&page7.root, "보호에 관한 법률」 및"),
        "Hancom PDF page 7 starts with the tail of paragraph 20 before table pi=21"
    );
}
