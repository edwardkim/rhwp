//! Issue #3931 — native HWP5 RowBreak 표의 저장 높이와 셀 줄높이 회계.
//!
//! Stage 1은 동작을 바꾸지 않고 두 결함 서명을 고정했다.
//! - section 10, paragraph 23의 6행 표에서 r4/c2는 저장 줄 16개(12+4)와
//!   문단 사이 vpos reset을 갖는다.
//! - #4763은 전체 HWP/HWPX를 한컴 2020 PDF와 같은 383쪽으로 맞췄다.
//!
//! Stage 2는 표 구조와 저장 pitch를 바꾸지 않은 채 pi=23의 12+4 소유권을,
//! Stage 3은 pi=14의 declared 선이월을 물리 fragment 계약으로 전환한다. 최신
//! 회귀는 #4763의 383쪽 계약을 유지하면서 두 HWP 표 조각이 본문 하단 안에
//! 머무는지를 함께 고정한다.

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::table::{Cell, Table, TablePageBreak};
use rhwp::parser::parse_document;
use rhwp::renderer::composer::compose_paragraph;
use rhwp::renderer::height_measurer::{HeightMeasurer, MeasuredTable};
use rhwp::renderer::style_resolver::resolve_styles_with_variant;
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const FIXTURE: &str = "samples/2025 행정업무운영 편람(최종).hwp";
const SECTION_INDEX: usize = 10;
const TARGET_PARA_INDEX: usize = 23;
const TARGET_ROW: usize = 4;
const TARGET_COL: usize = 2;
const HEAD_FRAGMENT_TEXT: &str = "문서결재와 업무분장 등을 공무원에게 부여하고 있습니다.";
const TAIL_FRAGMENT_TEXT: &str = "채용목적에 따른 업무범위 내에서";
const PI14_QUESTION_TEXT: &str = "문서등록번호가 빠진 공문서는 효력이 없는 것이 아닌가요?";
const PI14_HEAD_TEXT: &str = "그렇지 않습니다. 공문서에 문서등록번호가 빠졌다고";
const PI14_TAIL_TEXT: &str = "따라서, 해당 문서를 생산한 행정기관에";

fn read_fixture() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE);
    fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn parse_fixture() -> Document {
    parse_document(&read_fixture()).expect("parse #3931 HWP fixture")
}

fn target_table(document: &Document) -> (usize, &Table) {
    document.sections[SECTION_INDEX].paragraphs[TARGET_PARA_INDEX]
        .controls
        .iter()
        .enumerate()
        .find_map(|(control_index, control)| match control {
            Control::Table(table) if table.row_count == 6 => Some((control_index, table.as_ref())),
            _ => None,
        })
        .expect("section 10 paragraph 23 six-row table")
}

fn target_cell(table: &Table) -> (usize, &Cell) {
    table
        .cells
        .iter()
        .enumerate()
        .find(|(_, cell)| cell.row as usize == TARGET_ROW && cell.col as usize == TARGET_COL)
        .expect("paragraph 23 table r4/c2")
}

fn measured_target_table(document: &Document, control_index: usize) -> MeasuredTable {
    let section = &document.sections[SECTION_INDEX];
    let styles =
        resolve_styles_with_variant(&document.doc_info, DEFAULT_DPI, document.is_hwp3_variant);
    let composed = section
        .paragraphs
        .iter()
        .map(compose_paragraph)
        .collect::<Vec<_>>();
    let profile = document.layout_profile();
    HeightMeasurer::new(DEFAULT_DPI)
        .with_hwp3_variant(profile.hwp3_layout())
        .with_native_hwp5(profile.native_hwp5_layout())
        .measure_section(&section.paragraphs, &composed, &styles, None)
        .tables
        .into_iter()
        .find(|table| table.para_index == TARGET_PARA_INDEX && table.control_index == control_index)
        .expect("measured section 10 paragraph 23 table")
}

fn page_index_from_json(json: &str, field: &str) -> u32 {
    json.split(&format!("\"{field}\":"))
        .nth(1)
        .and_then(|value| value.split([',', '}']).next())
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or_else(|| panic!("JSON has no {field}: {json}"))
}

fn paragraph_text_pages(
    document: &HwpDocument,
    section_index: u32,
    para_index: u32,
    head_text: &str,
    tail_text: &str,
) -> (u32, u32) {
    let first_page_json = document
        .get_page_of_position(section_index, para_index)
        .expect("target outer paragraph page");
    let first_page = page_index_from_json(&first_page_json, "page");
    let trees = [first_page, first_page + 1].map(|page| {
        (
            page,
            document
                .get_page_render_tree(page)
                .unwrap_or_else(|_| panic!("render target table page {page}")),
        )
    });
    let page_for = |needle: &str| {
        trees
            .iter()
            .find_map(|(page, tree)| tree.contains(needle).then_some(*page))
            .unwrap_or_else(|| {
                panic!("target text {needle:?} absent from first two table fragments")
            })
    };
    (page_for(head_text), page_for(tail_text))
}

fn target_paragraph_pages(document: &HwpDocument) -> (u32, u32) {
    paragraph_text_pages(
        document,
        SECTION_INDEX as u32,
        TARGET_PARA_INDEX as u32,
        HEAD_FRAGMENT_TEXT,
        TAIL_FRAGMENT_TEXT,
    )
}

fn find_node<'a>(value: &'a Value, predicate: &impl Fn(&Value) -> bool) -> Option<&'a Value> {
    if predicate(value) {
        return Some(value);
    }
    value
        .get("children")
        .and_then(Value::as_array)
        .and_then(|children| {
            children
                .iter()
                .find_map(|child| find_node(child, predicate))
        })
}

fn bbox_bottom(node: &Value) -> f64 {
    let bbox = node.get("bbox").expect("render node bbox");
    bbox.get("y").and_then(Value::as_f64).expect("bbox y")
        + bbox.get("h").and_then(Value::as_f64).expect("bbox h")
}

fn assert_table_fragment_inside_body(
    document: &HwpDocument,
    page: u32,
    para_index: usize,
    fragment_text: &str,
) {
    let tree_json = document
        .get_page_render_tree(page)
        .expect("render target table fragment");
    let tree: Value = serde_json::from_str(&tree_json).expect("parse target render tree");
    let body = find_node(&tree, &|node| {
        node.get("type") == Some(&Value::from("Body"))
    })
    .expect("page body");
    let target_fragment = find_node(&tree, &|node| {
        node.get("type") == Some(&Value::from("Table"))
            && node.get("pi").and_then(Value::as_u64) == Some(para_index as u64)
            && node.to_string().contains(fragment_text)
    })
    .expect("target table fragment");
    assert!(
        bbox_bottom(target_fragment) <= bbox_bottom(body) + 0.5,
        "target fragment must stay inside the page body: table bottom {:.1}, body bottom {:.1}",
        bbox_bottom(target_fragment),
        bbox_bottom(body)
    );
}

#[test]
fn issue_3931_stage1_pins_pi23_stored_line_geometry() {
    let document = parse_fixture();
    assert!(
        document.layout_profile().native_hwp5_layout(),
        "fixture must keep the native HWP5 layout profile"
    );
    let (control_index, table) = target_table(&document);
    assert_eq!(table.page_break, TablePageBreak::RowBreak);

    let (_, cell) = target_cell(table);
    let stored_line_count = cell
        .paragraphs
        .iter()
        .map(|paragraph| paragraph.line_segs.len())
        .sum::<usize>();
    assert_eq!(stored_line_count, 16, "target cell stored LINE_SEG count");

    let measured = measured_target_table(&document, control_index);
    let measured_cell = measured
        .cells
        .iter()
        .find(|cell| cell.row == TARGET_ROW && cell.col == TARGET_COL)
        .expect("measured r4/c2");
    let declared_height = hwpunit_to_px(cell.height as i32, DEFAULT_DPI);
    let measured_required = measured_cell.total_content_height
        + measured_cell.padding_top
        + measured_cell.padding_bottom;

    assert_eq!(
        measured_cell.line_heights.len(),
        16,
        "measurement must still identify the same 16 stored lines"
    );
    assert!(
        (declared_height - 39.3).abs() <= 0.2,
        "baseline declared cell height changed: {declared_height:.2}px"
    );
    assert!(
        (measured_cell.total_content_height - 392.3).abs() <= 0.5,
        "baseline content height changed: {:.2}px",
        measured_cell.total_content_height
    );
    assert!(
        (measured.row_heights[TARGET_ROW] - 422.5).abs() <= 0.5,
        "baseline row height changed: {:.2}px (cell required {:.2}px)",
        measured.row_heights[TARGET_ROW],
        measured_required
    );
    assert!(
        cell.paragraphs
            .iter()
            .flat_map(|paragraph| paragraph.line_segs.iter())
            .all(|line| {
                let pitch = hwpunit_to_px(
                    line.line_height.saturating_add(line.line_spacing),
                    DEFAULT_DPI,
                );
                (pitch - 24.51).abs() <= 0.1
            }),
        "Hancom PDF uses the stored 24.5px pitch; the fix must not shrink global line height"
    );
}

#[test]
fn issue_3931_pi23_stored_reset_splits_across_adjacent_pages() {
    let document = HwpDocument::from_bytes(&read_fixture()).expect("paginate #3931 HWP fixture");
    let (head_page, tail_page) = target_paragraph_pages(&document);
    assert_eq!(
        tail_page,
        head_page + 1,
        "Hancom 2020 puts the first 12 lines at the previous page tail and the last 4 lines at the next page head"
    );

    assert_table_fragment_inside_body(&document, head_page, TARGET_PARA_INDEX, HEAD_FRAGMENT_TEXT);
}

#[test]
fn issue_3931_keeps_pr4763_hwp_page_count_contract() {
    let document = HwpDocument::from_bytes(&read_fixture()).expect("paginate #3931 HWP fixture");
    assert_eq!(
        document.page_count(),
        383,
        "#3931 fragment containment must preserve the #4763 HWP page-count contract"
    );
}

#[test]
fn issue_3931_pi14_declared_overflow_enters_fragment_scan() {
    let document = HwpDocument::from_bytes(&read_fixture()).expect("paginate #3931 HWP fixture");
    let (head_page, tail_page) = paragraph_text_pages(
        &document,
        SECTION_INDEX as u32,
        14,
        PI14_HEAD_TEXT,
        PI14_TAIL_TEXT,
    );
    assert_eq!(
        tail_page,
        head_page + 1,
        "Hancom 2020 keeps the first answer paragraph with question 7, then continues the answer on the next page"
    );
    assert_table_fragment_inside_body(&document, head_page, 14, PI14_HEAD_TEXT);
}

#[test]
fn issue_3931_hwpx_keeps_existing_fragment_route() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/2025 행정업무운영 편람(최종).hwpx");
    let bytes = fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("paginate #3931 HWPX fixture");
    assert_eq!(
        document.page_count(),
        383,
        "#3931 fragment containment must preserve the #4763 HWPX page-count contract"
    );
    let (question_page, answer_page) = paragraph_text_pages(
        &document,
        SECTION_INDEX as u32,
        14,
        PI14_QUESTION_TEXT,
        PI14_HEAD_TEXT,
    );
    assert_eq!(
        answer_page, question_page,
        "#4763 keeps the HWPX question and first answer on the same physical page"
    );
}
