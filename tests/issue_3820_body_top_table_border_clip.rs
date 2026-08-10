//! Issue #3820 — Body-top table frame paint clipping.
//!
//! SVG and Canvas clip strokes by painted area. A table top-border centreline
//! exactly on the Body clip loses its outer half even though its table/cell
//! owner geometry is correct. The paint-only repair must move just that border
//! group inward; owner boxes and the bottom frame stay on their source geometry.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, LineNode, RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};

const SAMPLE: &str =
    "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp";

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|err| panic!("read {SAMPLE}: {err}"));
    DocumentCore::from_bytes(&bytes).expect("parse policy authority fixture")
}

fn body_clip(node: &RenderNode) -> Option<BoundingBox> {
    if let RenderNodeType::Body {
        clip_rect: Some(clip),
    } = &node.node_type
    {
        return Some(*clip);
    }
    node.children.iter().find_map(body_clip)
}

fn table_for_paragraph(node: &RenderNode, para_index: usize) -> Option<&RenderNode> {
    if matches!(
        &node.node_type,
        RenderNodeType::Table(table) if table.para_index == Some(para_index)
    ) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| table_for_paragraph(child, para_index))
}

fn horizontal_lines(table: &RenderNode) -> Vec<&LineNode> {
    table
        .children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::Line(line) if child.visible && (line.y1 - line.y2).abs() <= 0.01 => {
                Some(line)
            }
            _ => None,
        })
        .collect()
}

fn assert_body_top_frame_contract(
    core: &DocumentCore,
    page_index: u32,
    para_index: usize,
    expected_paint_top_inset_hu: i32,
) {
    let human_page = page_index + 1;
    let tree = core
        .build_page_render_tree(page_index)
        .unwrap_or_else(|err| panic!("#3820 p{human_page} render failed: {err}"));
    let clip =
        body_clip(&tree.root).unwrap_or_else(|| panic!("#3820 p{human_page} Body clip missing"));
    let table = table_for_paragraph(&tree.root, para_index)
        .unwrap_or_else(|| panic!("#3820 p{human_page} pi={para_index} table missing"));

    let expected_paint_top = clip.y + hwpunit_to_px(expected_paint_top_inset_hu, DEFAULT_DPI);
    assert!(
        (table.bbox.y - expected_paint_top).abs() <= 0.5,
        "table paint bbox mismatch: p{human_page} table_y={} expected={} clip_y={}",
        table.bbox.y,
        expected_paint_top,
        clip.y,
    );
    let top_cells: Vec<_> = table
        .children
        .iter()
        .filter(
            |child| matches!(&child.node_type, RenderNodeType::TableCell(cell) if cell.row == 0),
        )
        .collect();
    assert!(
        !top_cells.is_empty(),
        "#3820 p{human_page} pi={para_index} top-row cell missing",
    );
    assert!(
        top_cells
            .iter()
            .all(|cell| (cell.bbox.y - table.bbox.y).abs() <= 0.5),
        "cell owner bbox moved away from table top on p{human_page}",
    );

    let top_frame_lines: Vec<_> = horizontal_lines(table)
        .into_iter()
        .filter(|line| line.y1 >= table.bbox.y && line.y1 <= table.bbox.y + 8.0)
        .collect();
    assert!(
        !top_frame_lines.is_empty(),
        "#3820 p{human_page} pi={para_index} top frame line missing",
    );
    let painted_top = top_frame_lines
        .iter()
        .map(|line| line.y1 - line.style.width / 2.0)
        .fold(f64::INFINITY, f64::min);
    assert!(
        painted_top >= clip.y,
        "top frame paint is clipped on p{human_page}: painted_top={painted_top} clip_y={}",
        clip.y,
    );
}

#[test]
fn issue_3820_p33_table_frame_is_paint_only_inset() {
    let core = core();
    let tree = core
        .build_page_render_tree(32)
        .expect("render policy PDF p33");
    let clip = body_clip(&tree.root).expect("p33 Body clip");
    let table = table_for_paragraph(&tree.root, 428).expect("p33 pi=428 table");
    let table_bottom = table.bbox.y + table.bbox.height;
    let bottom_line_y = horizontal_lines(table)
        .into_iter()
        .filter(|line| (line.y1 - table_bottom).abs() <= 8.0)
        .map(|line| line.y1)
        .fold(f64::NEG_INFINITY, f64::max);

    assert!(
        (269.20..=269.30).contains(&table_bottom),
        "p33 table bbox changed: bottom={table_bottom}",
    );
    assert!(
        (bottom_line_y - table_bottom).abs() <= 0.5,
        "p33 bottom frame must remain on source geometry: line={bottom_line_y} table={table_bottom}",
    );
    assert_body_top_frame_contract(&core, 32, 428, 0);
}

#[test]
fn issue_3820_successor_body_top_fragments_keep_full_top_stroke() {
    let core = core();
    // Render-page indices are 0-based: Hancom physical p168 and p214.
    // Stage 120 restores the stored-reset table's already-reserved physical outer-top margin on
    // the paint subtree.  The stroke must still remain wholly inside the Body clip.
    assert_body_top_frame_contract(&core, 167, 1775, 283);
    assert_body_top_frame_contract(&core, 213, 2548, 283);
}
