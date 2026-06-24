//! Issue #1486: HWPX 분할 표 내부 TAC 중첩 표가 오른쪽 밖으로 밀리는 회귀 방지.

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/hwpx_sample2.hwpx";
const TARGET_PAGE: u32 = 8; // 9쪽, 0-based

fn find_body_bbox(node: &RenderNode) -> Option<BoundingBox> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox);
    }

    node.children.iter().find_map(find_body_bbox)
}

fn collect_issue_1486_tables<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        let b = &node.bbox;
        if table.para_index.is_none()
            && table.control_index.is_none()
            && b.y < 220.0
            && b.width > 600.0
            && b.width < 680.0
            && b.height > 100.0
            && b.height < 220.0
        {
            out.push(node);
        }
    }

    for child in &node.children {
        collect_issue_1486_tables(child, out);
    }
}

fn collect_render_tree_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }

    for child in &node.children {
        collect_render_tree_text(child, out);
    }
}

fn render_tree_contains_text(node: &RenderNode, needle: &str) -> bool {
    let mut text = String::new();
    collect_render_tree_text(node, &mut text);
    text.contains(needle)
}

fn collect_images<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Image(_)) {
        out.push(node);
    }

    for child in &node.children {
        collect_images(child, out);
    }
}

fn find_image_bbox_by_ref(
    node: &RenderNode,
    para_index: usize,
    control_index: usize,
) -> Option<BoundingBox> {
    if let RenderNodeType::Image(image) = &node.node_type {
        if image.para_index == Some(para_index) && image.control_index == Some(control_index) {
            return Some(node.bbox);
        }
    }

    node.children
        .iter()
        .find_map(|child| find_image_bbox_by_ref(child, para_index, control_index))
}

fn collect_tables<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push(node);
    }

    for child in &node.children {
        collect_tables(child, out);
    }
}

fn collect_text_run_bboxes(node: &RenderNode, needle: &str, out: &mut Vec<BoundingBox>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == needle {
            out.push(node.bbox);
        }
    }

    for child in &node.children {
        collect_text_run_bboxes(child, needle, out);
    }
}

#[test]
fn issue_1486_partial_table_tac_nested_table_stays_inside_page_body() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let tree = doc
        .build_page_render_tree(TARGET_PAGE)
        .unwrap_or_else(|e| panic!("render {SAMPLE} page {}: {e}", TARGET_PAGE + 1));

    let body = find_body_bbox(&tree.root).expect("Body bbox");
    let page_right = tree.root.bbox.x + tree.root.bbox.width;
    let expected_body_right = page_right - body.x;

    let mut candidates = Vec::new();
    collect_issue_1486_tables(&tree.root, &mut candidates);
    assert!(
        !candidates.is_empty(),
        "9쪽 상단의 문제 TAC 중첩 표를 찾지 못함"
    );

    let table = candidates
        .into_iter()
        .min_by(|a, b| a.bbox.y.partial_cmp(&b.bbox.y).unwrap())
        .expect("candidate table");
    let table_right = table.bbox.x + table.bbox.width;

    eprintln!(
        "[issue_1486] page={} body_x={:.2} page_right={:.2} table=[x={:.2} y={:.2} w={:.2} h={:.2}] right={:.2} expected_body_right={:.2}",
        TARGET_PAGE + 1,
        body.x,
        page_right,
        table.bbox.x,
        table.bbox.y,
        table.bbox.width,
        table.bbox.height,
        table_right,
        expected_body_right,
    );

    assert!(
        table.bbox.x < body.x + 120.0,
        "분할 표 내부 TAC 중첩 표가 본문 좌측에서 과도하게 밀림: table_x={:.2}, body_x={:.2}",
        table.bbox.x,
        body.x,
    );
    assert!(
        table_right <= expected_body_right + 1.0,
        "분할 표 내부 TAC 중첩 표가 페이지 본문 오른쪽을 초과함: table_right={:.2}, expected_body_right={:.2}",
        table_right,
        expected_body_right,
    );
}

#[test]
fn issue_1486_terminal_rowbreak_sliver_does_not_push_pdf_page22_content() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let page22 = doc
        .build_page_render_tree(21)
        .expect("render issue #1486 page 22");
    let page23 = doc
        .build_page_render_tree(22)
        .expect("render issue #1486 page 23");

    assert!(
        render_tree_contains_text(&page22.root, "lisfranc"),
        "한컴 PDF 기준 22쪽 하단의 lisfranc 줄이 rhwp 22쪽에 있어야 함"
    );
    assert!(
        !render_tree_contains_text(&page23.root, "lisfranc"),
        "무가시 RowBreak terminal sliver 때문에 lisfranc 줄이 23쪽으로 밀리면 안 됨"
    );
}

#[test]
fn issue_1486_rowspan_block_tail_stays_on_pdf_page14() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let page13 = doc
        .build_page_render_tree(12)
        .expect("render issue #1486 page 13");
    let page14 = doc
        .build_page_render_tree(13)
        .expect("render issue #1486 page 14");

    assert!(
        render_tree_contains_text(&page13.root, "아동복지시설"),
        "한컴 PDF 기준 13쪽 끝에는 사회취약 계층 (아) 항목까지 보여야 함"
    );
    assert!(
        !render_tree_contains_text(&page13.root, "제2조제10호"),
        "사회취약 계층 하단 행이 13쪽에서 먼저 소비되면 안 됨"
    );
    assert!(
        render_tree_contains_text(&page14.root, "(자)"),
        "한컴 PDF 기준 14쪽은 사회취약 계층 (자) 항목으로 이어져야 함"
    );
    assert!(
        render_tree_contains_text(&page14.root, "제2조제10호"),
        "국민기초생활 보장법 행은 14쪽에 남아야 함"
    );
}

#[test]
fn issue_1486_page19_nested_square_picture_is_not_page_clipped() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let page19 = doc
        .build_page_render_tree(18)
        .expect("render issue #1486 page 19");
    let page_bottom = page19.root.bbox.y + page19.root.bbox.height;

    let mut images = Vec::new();
    collect_images(&page19.root, &mut images);
    let diagram = images
        .into_iter()
        .filter(|n| n.bbox.width > 300.0 && n.bbox.height > 90.0)
        .max_by(|a, b| a.bbox.y.partial_cmp(&b.bbox.y).unwrap())
        .expect("19쪽 하단 무허가건축물 확인 절차 그림");
    let diagram_bottom = diagram.bbox.y + diagram.bbox.height;

    assert!(
        diagram_bottom <= page_bottom - 4.0,
        "19쪽 하단 그림이 페이지 아래로 잘리면 안 됨: bottom={diagram_bottom:.2}, page_bottom={page_bottom:.2}"
    );
}

#[test]
fn issue_1486_page13_page_number_keeps_footer_gap() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let page13 = doc
        .build_page_render_tree(12)
        .expect("render issue #1486 page 13");

    let mut tables = Vec::new();
    collect_tables(&page13.root, &mut tables);
    let bottom_table = tables
        .into_iter()
        .filter(|n| n.bbox.y > 500.0 && n.bbox.width > 700.0)
        .max_by(|a, b| {
            let ab = a.bbox.y + a.bbox.height;
            let bb = b.bbox.y + b.bbox.height;
            ab.partial_cmp(&bb).unwrap()
        })
        .expect("13쪽 하단 배점기준표 조각");
    let table_bottom = bottom_table.bbox.y + bottom_table.bbox.height;

    let mut page_numbers = Vec::new();
    collect_text_run_bboxes(&page13.root, "-13-", &mut page_numbers);
    let page_number = page_numbers.into_iter().next().expect("13쪽 쪽번호");

    assert!(
        page_number.y - table_bottom >= 12.0,
        "13쪽 하단 표와 쪽번호 사이 간격이 너무 좁음: table_bottom={table_bottom:.2}, page_number_y={:.2}",
        page_number.y
    );
}

#[test]
fn issue_1486_page29_tac_logo_aligns_with_text_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let page29 = doc
        .build_page_render_tree(28)
        .expect("render issue #1486 page 29");
    let logo = find_image_bbox_by_ref(&page29.root, 218, 0).expect("29쪽 LH 로고 TAC 그림 bbox");

    assert!(
        (logo.y - 417.6).abs() <= 4.0,
        "29쪽 LH 로고 y가 한컴 PDF 기준에서 벗어남: y={:.2}, bbox={:?}",
        logo.y,
        logo
    );
    assert!(
        (logo.x - 38.4).abs() <= 3.0 && (logo.width - 115.6).abs() <= 3.0,
        "29쪽 LH 로고 x/폭이 한컴 PDF 기준에서 벗어남: bbox={:?}",
        logo
    );
}
