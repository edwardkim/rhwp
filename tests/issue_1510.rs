//! Issue #1510: visible text 가 있는 한 문단에 co-anchored para-relative
//! TopAndBottom floating 표가 여러 개 있을 때, vertical_offset 정렬/누적으로
//! 페이지가 늘어나거나 표 순서가 뒤집히는 회귀를 막는다.

use rhwp::model::shape::{CommonObjAttr, TextWrap, VertRelTo};
use rhwp::model::table::Table;
use rhwp::renderer::layout::para_relative_float_table_lead;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const HWP_SAMPLE: &str = "samples/issue1510_coanchored_float_tables.hwp";
const HWPX_SAMPLE: &str = "samples/issue1510_coanchored_float_tables.hwpx";
const TARGET_PI: usize = 0;
const TARGET_TABLES: [usize; 3] = [2, 3, 4];

fn load_doc(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", sample, e))
}

fn collect_table_order(root: &RenderNode, out: &mut Vec<usize>) {
    if let RenderNodeType::Table(table) = &root.node_type {
        if table.para_index == Some(TARGET_PI) {
            if let Some(ci) = table.control_index {
                if TARGET_TABLES.contains(&ci) {
                    out.push(ci);
                }
            }
        }
    }
    for child in &root.children {
        collect_table_order(child, out);
    }
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

fn find_text_bbox(root: &RenderNode, needle: &str) -> Option<(f64, f64)> {
    if let RenderNodeType::TextRun(run) = &root.node_type {
        if run.para_index.is_some() && run.text == needle {
            return Some((root.bbox.y, root.bbox.y + root.bbox.height));
        }
    }
    for child in &root.children {
        if let Some(found) = find_text_bbox(child, needle) {
            return Some(found);
        }
    }
    None
}

#[test]
fn issue_1510_coanchored_visible_para_float_tables_stay_on_one_page() {
    let doc = load_doc(HWP_SAMPLE);

    assert_eq!(
        doc.page_count(),
        1,
        "{} should match the Hancom 2024 HWP PDF baseline as a one-page document",
        HWP_SAMPLE,
    );
}

#[test]
fn issue_1510_visible_para_float_tables_apply_offsets_without_text_overlap() {
    let doc = load_doc(HWP_SAMPLE);
    let tree = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");

    let (a_top, a_bottom) = find_table_bbox(&tree.root, 2).expect("A table bbox");
    let (b_top, _) = find_table_bbox(&tree.root, 3).expect("B table bbox");
    let (c_top, _) = find_table_bbox(&tree.root, 4).expect("C table bbox");
    let (_, filler_07_bottom) =
        find_text_bbox(&tree.root, "filler paragraph 07").expect("filler 07 bbox");
    let (filler_08_top, _) =
        find_text_bbox(&tree.root, "filler paragraph 08").expect("filler 08 bbox");

    assert!(
        b_top + 0.5 < c_top,
        "negative vertical_offset table should render above the zero-offset sibling: b_top={b_top:.1}, c_top={c_top:.1}",
    );
    assert!(
        filler_07_bottom <= a_top + 12.0,
        "text before the positive-offset table should remain above the table zone: filler07_bottom={filler_07_bottom:.1}, a_top={a_top:.1}",
    );
    assert!(
        filler_08_top >= a_bottom - 0.5,
        "text after reaching the positive-offset table should resume below it: filler08_top={filler_08_top:.1}, a_bottom={a_bottom:.1}",
    );
}

#[test]
fn issue_1510_visible_para_float_tables_keep_document_order() {
    let doc = load_doc(HWP_SAMPLE);
    let tree = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");
    let mut order = Vec::new();
    collect_table_order(&tree.root, &mut order);

    assert_eq!(
        order, TARGET_TABLES,
        "co-anchored visible-host float tables should retain document/control order",
    );
}

#[test]
fn issue_1510_hwpx_unsigned_negative_offset_and_visible_flow_match_two_page_baseline() {
    let doc = load_doc(HWPX_SAMPLE);

    assert_eq!(
        doc.page_count(),
        2,
        "{} should match the Hancom 2024 HWPX PDF baseline as a two-page document",
        HWPX_SAMPLE,
    );

    let page0 = doc
        .build_page_render_tree(0)
        .expect("build_page_render_tree(0)");
    let page1 = doc
        .build_page_render_tree(1)
        .expect("build_page_render_tree(1)");

    let (_, b_bottom) = find_table_bbox(&page0.root, 3).expect("B table bbox");
    let (c_top, _) = find_table_bbox(&page0.root, 4).expect("C table bbox");
    assert!(
        c_top + 0.5 >= b_bottom,
        "HWPX visible-host non-positive float siblings should stack vertically: b_bottom={b_bottom:.1}, c_top={c_top:.1}",
    );

    assert!(
        find_text_bbox(&page0.root, "filler paragraph 29").is_some(),
        "filler 29 should remain on page 1",
    );
    assert!(
        find_text_bbox(&page0.root, "filler paragraph 30").is_none(),
        "filler 30 should move to page 2",
    );
    assert!(
        find_text_bbox(&page1.root, "filler paragraph 30").is_some()
            && find_text_bbox(&page1.root, "filler paragraph 32").is_some(),
        "page 2 should contain filler 30..32",
    );
}

// --- #6697 후속: Square 어울림도 para-relative vertOffset 리드를 받는다 ---
//
// #6697 은 셀 안 제목 줄이 안은 문단 기준 자리차지 중첩 표를 호스트 문단 아래로
// 내리는 vertOffset 리드(`para_relative_float_table_lead`)를 `TopAndBottom` 어울림
// 하나로 좁혔다. 텍스트를 옆으로 흘리는 `Square` 어울림도 같은 문단 기준 vertOffset
// 규칙을 따르는데 리드에서 빠져, #6697 이 그 제목 줄을 그리기 시작한 뒤 Square 중첩
// 표가 제목 줄과 같은 y 에 겹쳤다. 아래 가드는 두 어울림이 같은 리드를 받고, 세로
// 배치 계약이 다른 글 앞/뒤 overlay 는 계속 제외되며(편람 안내 상자 text_overlap
// 회귀 방지), 부호 있는 음수 오프셋과 글자처럼 표는 리드 0 임을 고정한다.

/// 1200 HU vertOffset = 16.0 px @ 96 dpi.
fn para_float_table(text_wrap: TextWrap, vertical_offset: u32, treat_as_char: bool) -> Table {
    Table {
        common: CommonObjAttr {
            treat_as_char,
            vert_rel_to: VertRelTo::Para,
            vertical_offset,
            text_wrap,
            ..CommonObjAttr::default()
        },
        ..Table::default()
    }
}

#[test]
fn issue_6697_top_and_bottom_and_square_wrap_share_vertical_offset_lead() {
    let top = para_float_table(TextWrap::TopAndBottom, 1200, false);
    assert!(
        (para_relative_float_table_lead(&top, 96.0) - 16.0).abs() < 0.05,
        "TopAndBottom 어울림은 종전대로 vertOffset 리드를 받아야 한다",
    );

    let square = para_float_table(TextWrap::Square, 1200, false);
    assert!(
        (para_relative_float_table_lead(&square, 96.0) - 16.0).abs() < 0.05,
        "Square 어울림도 같은 vertOffset 리드를 받아야 한다 (#6697 누락분)",
    );
}

#[test]
fn issue_6697_front_and_behind_overlay_wraps_stay_excluded_from_lead() {
    for overlay in [TextWrap::BehindText, TextWrap::InFrontOfText] {
        let table = para_float_table(overlay, 1200, false);
        assert_eq!(
            para_relative_float_table_lead(&table, 96.0),
            0.0,
            "글 앞/뒤 overlay 는 세로 배치 계약이 달라 리드에서 제외된다",
        );
    }
}

#[test]
fn issue_6697_signed_negative_offset_and_treat_as_char_take_no_lead() {
    // 부호 있는 u32 음수 오프셋(−22613 HU)은 표를 위로 올리지 않는다.
    let negative = para_float_table(TextWrap::Square, 4_294_944_683, false);
    assert_eq!(para_relative_float_table_lead(&negative, 96.0), 0.0);

    // 글자처럼 표는 문단 기준 어울림 규칙 밖이다.
    let as_char = para_float_table(TextWrap::Square, 1200, true);
    assert_eq!(para_relative_float_table_lead(&as_char, 96.0), 0.0);
}
