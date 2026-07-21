//! Issue #2439: a later para-relative TopAndBottom RowBreak float can be deferred whole to a
//! fresh page by the co-anchored orphan guard. Its placement/exclusion anchor must be rebased to
//! that fresh page; retaining the previous page's `para_start_height` lets following text flow at
//! the top of the new page and overlap the deferred table.
//!
//! The synthetic fixture is narrowed from `issue1663_coanchored_float_orphan.hwpx`:
//! - a preceding paragraph gives the shared float host a non-zero page-local start;
//! - small float A remains on page 1;
//! - page-fitting float B cannot fit the remainder and is deferred whole to page 2;
//! - `AFTER FLOAT` must resume below B's exclusion, never inside B.

use rhwp::model::control::Control;
use rhwp::model::paragraph::LineSeg;
use rhwp::model::provenance::{SourceFormat, SourceProvenance};
use rhwp::model::table::TablePageBreak;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/hwpx/issue2439_page_local_float_exclusion.hwpx";
const ZERO_OFFSET_STACK_SAMPLE: &str =
    "samples/issue2439_zero_offset_coanchored_float_exclusion.hwp";
const POSITIVE_EMPTY_HOST_SAMPLE: &str = "samples/issue1549_empty_host_float_clamp.hwpx";
const HOST_PI: usize = 1;
const TABLE_A_CI: usize = 0;
const TABLE_B_CI: usize = 1;

fn load_doc(sample: &str) -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {sample}: {e}"))
}

fn load_with_native_hwp5_provenance(sample: &str) -> HwpDocument {
    let mut doc = load_doc(sample);
    let mut model = doc.document().clone();
    // The compact geometry fixture is HWPX so it can be reviewed as source. Rebinding only its
    // provenance exercises the exact native-HWP5 compatibility gate without checking the
    // user-provided reproduction document into the repository.
    model.provenance = SourceProvenance {
        format: SourceFormat::Hwp5,
        hwp3_lineage: false,
        hwpx_lineage: false,
    };
    let host = &mut model.sections[0].paragraphs[1];
    host.line_segs[0].tag &= !LineSeg::TAG_IMPLEMENTATION_PROPERTY;
    let Control::Table(table) = &mut host.controls[0] else {
        panic!("positive empty-host fixture must contain a table control");
    };
    table.page_break = TablePageBreak::RowBreak;
    doc.set_document(model);
    assert!(doc.document().layout_profile().native_hwp5_layout());
    doc
}

fn isolated_positive_empty_host_doc(top_margin_delta: i16) -> HwpDocument {
    let mut doc = load_with_native_hwp5_provenance(POSITIVE_EMPTY_HOST_SAMPLE);
    let mut model = doc.document().clone();
    // Remove the unrelated float in the preceding title paragraph so the full-table coordinate
    // is controlled only by this host's paragraph top, offset, and strict outer-top margin.
    model.sections[0].paragraphs[0]
        .controls
        .retain(|control| !matches!(control, Control::Table(_)));
    let Control::Table(table) = &mut model.sections[0].paragraphs[1].controls[0] else {
        panic!("positive empty-host fixture must contain a table control");
    };
    table.outer_margin_top += top_margin_delta;
    doc.set_document(model);
    doc
}

fn split_positive_empty_host_doc() -> HwpDocument {
    const ROW_COUNT: u16 = 30;

    let mut doc = load_with_native_hwp5_provenance(POSITIVE_EMPTY_HOST_SAMPLE);
    let mut model = doc.document().clone();
    model.sections[0].paragraphs[0]
        .controls
        .retain(|control| !matches!(control, Control::Table(_)));

    let Control::Table(table) = &mut model.sections[0].paragraphs[1].controls[0] else {
        panic!("positive empty-host fixture must contain a table control");
    };
    let template_cell = table
        .cells
        .first()
        .cloned()
        .expect("positive empty-host fixture must contain one cell");
    let row_size = table.row_sizes.first().copied().unwrap_or(1);
    let mut cells = Vec::with_capacity(usize::from(ROW_COUNT));
    for row in 0..ROW_COUNT {
        let mut cell = template_cell.clone();
        cell.row = row;
        cell.is_header = row == 0;
        if row > 0 {
            let paragraph = cell
                .paragraphs
                .first_mut()
                .expect("template cell must contain one paragraph");
            let text_len = paragraph.text.chars().count();
            paragraph.delete_text_at(0, text_len);
            paragraph.insert_text_at(0, &format!("split row {row:02}"));
        }
        cells.push(cell);
    }

    table.row_count = ROW_COUNT;
    table.row_sizes = vec![row_size; usize::from(ROW_COUNT)];
    table.cells = cells;
    table.cell_grid = (0..usize::from(ROW_COUNT)).map(Some).collect();
    table.repeat_header = true;
    table.common.height = template_cell.height.saturating_mul(u32::from(ROW_COUNT));
    table.dirty = true;
    doc.set_document(model);
    doc
}

fn find_table_bbox(
    root: &RenderNode,
    host_para_index: usize,
    control_index: usize,
) -> Option<(f64, f64)> {
    if let RenderNodeType::Table(table) = &root.node_type {
        if table.para_index == Some(host_para_index) && table.control_index == Some(control_index) {
            return Some((root.bbox.y, root.bbox.y + root.bbox.height));
        }
    }
    root.children
        .iter()
        .find_map(|child| find_table_bbox(child, host_para_index, control_index))
}

fn find_text_bbox(root: &RenderNode, needle: &str) -> Option<(f64, f64)> {
    if let RenderNodeType::TextRun(run) = &root.node_type {
        if run.text == needle {
            return Some((root.bbox.y, root.bbox.y + root.bbox.height));
        }
    }
    root.children
        .iter()
        .find_map(|child| find_text_bbox(child, needle))
}

fn find_body_bottom(root: &RenderNode) -> Option<f64> {
    if matches!(root.node_type, RenderNodeType::Body { .. }) {
        return Some(root.bbox.y + root.bbox.height);
    }
    root.children.iter().find_map(find_body_bottom)
}

#[test]
fn deferred_coanchored_float_uses_fresh_page_local_exclusion_anchor() {
    let doc = load_doc(SAMPLE);
    let page1 = doc
        .build_page_render_tree(0)
        .expect("build page 1 render tree");
    let page2 = doc
        .build_page_render_tree(1)
        .expect("build page 2 render tree");

    assert!(
        find_table_bbox(&page1.root, HOST_PI, TABLE_A_CI).is_some(),
        "small preceding float A must remain on page 1",
    );
    assert!(
        find_table_bbox(&page1.root, HOST_PI, TABLE_B_CI).is_none(),
        "page-fitting co-anchored float B must defer whole instead of leaving a fragment on page 1",
    );

    let (table_top, table_bottom) =
        find_table_bbox(&page2.root, HOST_PI, TABLE_B_CI).expect("deferred table B bbox on page 2");
    assert!(
        table_bottom > table_top,
        "deferred table must have positive height: table=[{table_top:.1},{table_bottom:.1}]",
    );

    if let Some((after_top, after_bottom)) = find_text_bbox(&page2.root, "AFTER FLOAT") {
        let body_bottom = find_body_bottom(&page2.root).expect("page 2 body bbox");
        assert!(
            after_top + 0.5 >= table_bottom,
            "following text must resume below the deferred table's fresh-page exclusion: \
             table=[{table_top:.1},{table_bottom:.1}], after_top={after_top:.1}",
        );
        assert!(
            after_bottom <= body_bottom + 0.5,
            "following text must remain inside the page body or paginate later: \
             after=[{after_top:.1},{after_bottom:.1}], body_bottom={body_bottom:.1}",
        );
    } else {
        let later_page_has_text = (2..doc.page_count()).any(|page_index| {
            doc.build_page_render_tree(page_index)
                .ok()
                .and_then(|tree| find_text_bbox(&tree.root, "AFTER FLOAT"))
                .is_some()
        });
        assert!(
            later_page_has_text,
            "following text may move to a later page, but it must not disappear",
        );
    }
}

#[test]
fn zero_offset_coanchored_float_reserves_its_full_zone_for_later_siblings() {
    let doc = load_doc(ZERO_OFFSET_STACK_SAMPLE);
    let page = doc
        .build_page_render_tree(0)
        .expect("build zero-offset stack render tree");

    let (first_top, first_bottom) =
        find_table_bbox(&page.root, 0, 2).expect("zero-offset table A bbox");
    let (second_top, second_bottom) =
        find_table_bbox(&page.root, 0, 3).expect("positive-offset table B bbox");

    assert!(
        first_bottom > first_top && second_bottom > second_top,
        "both co-anchored tables must retain positive height: A=[{first_top:.1},{first_bottom:.1}], \
         B=[{second_top:.1},{second_bottom:.1}]",
    );
    assert!(
        second_top + 0.5 >= first_bottom,
        "a zero-offset first float must reserve an exclusion zone so its positive-offset sibling \
         is stacked below it: A=[{first_top:.1},{first_bottom:.1}], B=[{second_top:.1},{second_bottom:.1}]",
    );
    let (host_text_top, _) =
        find_text_bbox(&page.root, "ISSUE 1510 CENTER TITLE").expect("visible host text bbox");
    assert!(
        host_text_top + 0.5 >= second_bottom,
        "visible host text emitted after the co-anchored table group must resume below the last \
         table: B=[{second_top:.1},{second_bottom:.1}], text_top={host_text_top:.1}",
    );
}

#[test]
fn positive_offset_empty_host_float_advances_flow_to_its_painted_bottom() {
    let doc = load_with_native_hwp5_provenance(POSITIVE_EMPTY_HOST_SAMPLE);
    let page = doc
        .build_page_render_tree(0)
        .expect("build positive-offset empty-host render tree");

    let (table_top, table_bottom) =
        find_table_bbox(&page.root, 1, 0).expect("single positive-offset table bbox");
    let (following_top, _) =
        find_text_bbox(&page.root, "filler paragraph 01").expect("following paragraph bbox");

    assert!(
        following_top + 0.5 >= table_bottom,
        "following flow must start at or below the actual painted bottom of a positive-offset \
         empty-host TopAndBottom float: table=[{table_top:.1},{table_bottom:.1}], \
         following_top={following_top:.1}",
    );

    const TOP_MARGIN_DELTA_HU: i16 = 567;
    let isolated_base_page = isolated_positive_empty_host_doc(0)
        .build_page_render_tree(0)
        .expect("build isolated base render tree");
    let (isolated_base_top, _) =
        find_table_bbox(&isolated_base_page.root, 1, 0).expect("isolated base table bbox");
    let larger_top_margin_doc = isolated_positive_empty_host_doc(TOP_MARGIN_DELTA_HU);
    let larger_margin_page = larger_top_margin_doc
        .build_page_render_tree(0)
        .expect("build larger-top-margin render tree");
    let (larger_margin_top, _) =
        find_table_bbox(&larger_margin_page.root, 1, 0).expect("larger-top-margin table bbox");
    let expected_delta = hwpunit_to_px(TOP_MARGIN_DELTA_HU as i32, DEFAULT_DPI);
    assert!(
        (larger_margin_top - isolated_base_top - expected_delta).abs() < 0.01,
        "the full PageItem::Table path must repeat the same strict outer-top margin as the first \
         PartialTable fragment: base_top={isolated_base_top:.2}, larger_top={larger_margin_top:.2}, \
         expected_delta={expected_delta:.2}",
    );
}

#[test]
fn positive_offset_empty_host_rowbreak_continuation_advances_following_flow() {
    let doc = split_positive_empty_host_doc();
    let mut table_fragments = Vec::new();
    let mut following_text = None;

    for page_index in 0..doc.page_count() {
        let page = doc
            .build_page_render_tree(page_index)
            .unwrap_or_else(|e| panic!("build split-table page {}: {e}", page_index + 1));
        if let Some((table_top, table_bottom)) = find_table_bbox(&page.root, 1, 0) {
            let repeated_header = find_text_bbox(&page.root, "C11").is_some();
            table_fragments.push((page_index, table_top, table_bottom, repeated_header));
        }
        if following_text.is_none() {
            following_text =
                find_text_bbox(&page.root, "filler paragraph 01").map(|bbox| (page_index, bbox));
        }
    }

    assert!(
        table_fragments.len() >= 2,
        "the synthetic RowBreak table must produce PartialTable continuation fragments"
    );
    assert!(
        table_fragments
            .iter()
            .all(|(_, top, bottom, _)| bottom > top),
        "every split-table fragment must retain positive painted height: {table_fragments:?}"
    );
    assert!(
        table_fragments
            .iter()
            .all(|(_, _, _, repeated_header)| *repeated_header),
        "the header row must repeat on every RowBreak fragment: {table_fragments:?}"
    );

    let &(last_table_page, last_table_top, last_table_bottom, _) = table_fragments
        .last()
        .expect("split table must have a final fragment");
    let (following_page, (following_top, _)) =
        following_text.expect("following paragraph must remain visible after the split table");
    assert!(
        following_page >= last_table_page,
        "following text must not precede the final PartialTable fragment: table_page={}, \
         following_page={}",
        last_table_page + 1,
        following_page + 1,
    );
    if following_page == last_table_page {
        assert!(
            following_top + 0.5 >= last_table_bottom,
            "following flow must start below the final positive-offset PartialTable fragment: \
             table=[{last_table_top:.1},{last_table_bottom:.1}], \
             following_top={following_top:.1}"
        );
    }
}
