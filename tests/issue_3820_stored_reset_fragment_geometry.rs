//! Issue #3820 Stage 120 — native-HWP5 stored-reset table fragment paint geometry.
//!
//! The policy authority fixture contains three top-level empty-host 1x1 RowBreak tables whose
//! cell LINE_SEG ladder restarts at zero across a paragraph boundary.  Hancom keeps the existing
//! page owners/cuts, paints the first frame at the declared stored head height (without the
//! reset-preceding trailing line spacing), and restores the equal outer-left/top margin on both
//! physical fragments.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{BoundingBox, LineNode, RenderNode, RenderNodeType};
use rhwp::renderer::{hwpunit_to_px, DEFAULT_DPI};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str =
    "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp";

#[derive(Clone, Copy)]
struct FragmentExpectation {
    page_index: u32,
    para_index: usize,
    host_vpos_hu: Option<i32>,
    cell_height_hu: i32,
    cell_para_start: usize,
    cell_para_end: usize,
}

fn document() -> HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|err| panic!("read {SAMPLE}: {err}"));
    HwpDocument::from_bytes(&bytes).expect("parse policy authority fixture")
}

fn body_geometry(node: &RenderNode) -> Option<(BoundingBox, BoundingBox)> {
    if let RenderNodeType::Body {
        clip_rect: Some(clip),
    } = &node.node_type
    {
        return Some((node.bbox, *clip));
    }
    node.children.iter().find_map(body_geometry)
}

fn tables_for_paragraph<'a>(
    node: &'a RenderNode,
    para_index: usize,
    out: &mut Vec<&'a RenderNode>,
) {
    if matches!(
        &node.node_type,
        RenderNodeType::Table(table) if table.para_index == Some(para_index)
    ) {
        out.push(node);
    }
    for child in &node.children {
        tables_for_paragraph(child, para_index, out);
    }
}

fn direct_cell(table: &RenderNode) -> &RenderNode {
    let cells: Vec<_> = table
        .children
        .iter()
        .filter(|child| {
            matches!(
                &child.node_type,
                RenderNodeType::TableCell(cell) if cell.row == 0 && cell.col == 0
            )
        })
        .collect();
    assert_eq!(
        cells.len(),
        1,
        "stored-reset fragment must paint one direct 1x1 cell"
    );
    cells[0]
}

fn direct_frame_lines(table: &RenderNode) -> Vec<&LineNode> {
    table
        .children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::Line(line) if child.visible => Some(line),
            _ => None,
        })
        .collect()
}

fn line_owners(table: &RenderNode) -> BTreeSet<(usize, u32)> {
    fn collect(node: &RenderNode, out: &mut BTreeSet<(usize, u32)>) {
        if let RenderNodeType::TextLine(line) = &node.node_type {
            if let (Some(para_index), Some(line_index)) = (line.para_index, line.line_index) {
                out.insert((para_index, line_index));
            }
        }
        for child in &node.children {
            collect(child, out);
        }
    }

    let mut owners = BTreeSet::new();
    collect(table, &mut owners);
    owners
}

fn assert_fragment_geometry(
    doc: &HwpDocument,
    expected: FragmentExpectation,
) -> BTreeSet<(usize, u32)> {
    let human_page = expected.page_index + 1;
    let tree = doc
        .build_page_render_tree(expected.page_index)
        .unwrap_or_else(|err| panic!("render policy p{human_page}: {err}"));
    let (body, clip) =
        body_geometry(&tree.root).unwrap_or_else(|| panic!("p{human_page} Body geometry"));
    let mut tables = Vec::new();
    tables_for_paragraph(&tree.root, expected.para_index, &mut tables);
    assert_eq!(
        tables.len(),
        1,
        "p{human_page} pi={} must have one physical table owner",
        expected.para_index,
    );
    let table = tables[0];
    let cell = direct_cell(table);

    let outer = hwpunit_to_px(283, DEFAULT_DPI);
    let expected_x = body.x + outer;
    let expected_y = body.y
        + expected
            .host_vpos_hu
            .map(|vpos| hwpunit_to_px(vpos, DEFAULT_DPI))
            .unwrap_or(0.0)
        + outer;
    let expected_width = hwpunit_to_px(41_954, DEFAULT_DPI);
    let expected_cell_height = hwpunit_to_px(expected.cell_height_hu, DEFAULT_DPI);

    assert!(
        (table.bbox.x - expected_x).abs() <= 0.2
            && (table.bbox.y - expected_y).abs() <= 0.2,
        "p{human_page} pi={} paint origin must restore outer-left/top: body={body:?}, table={:?}, expected=({expected_x},{expected_y})",
        expected.para_index,
        table.bbox,
    );
    assert!(
        (table.bbox.width - expected_width).abs() <= 0.2,
        "p{human_page} pi={} width changed: {:?}",
        expected.para_index,
        table.bbox,
    );
    assert!(
        (cell.bbox.x - table.bbox.x).abs() <= 0.2
            && (cell.bbox.y - table.bbox.y).abs() <= 0.2
            && (cell.bbox.width - expected_width).abs() <= 0.2
            && (cell.bbox.height - expected_cell_height).abs() <= 0.2,
        "p{human_page} pi={} cell paint/clip geometry mismatch: table={:?}, cell={:?}, expected_h={expected_cell_height}",
        expected.para_index,
        table.bbox,
        cell.bbox,
    );
    // The final Table bbox contains the bottom border's half-stroke; the cell viewport is the
    // exact stored/measured fragment height.
    assert!(
        (table.bbox.height - expected_cell_height - 0.5).abs() <= 0.2,
        "p{human_page} pi={} table frame height mismatch: table={:?}, cell={:?}",
        expected.para_index,
        table.bbox,
        cell.bbox,
    );

    let frame = direct_frame_lines(table);
    let table_right = table.bbox.x + table.bbox.width;
    let cell_bottom = cell.bbox.y + cell.bbox.height;
    let has_top = frame
        .iter()
        .any(|line| (line.y1 - line.y2).abs() <= 0.01 && (line.y1 - table.bbox.y).abs() <= 0.2);
    let has_bottom = frame
        .iter()
        .any(|line| (line.y1 - line.y2).abs() <= 0.01 && (line.y1 - cell_bottom).abs() <= 0.2);
    let has_left = frame
        .iter()
        .any(|line| (line.x1 - line.x2).abs() <= 0.01 && (line.x1 - table.bbox.x).abs() <= 0.2);
    let has_right = frame
        .iter()
        .any(|line| (line.x1 - line.x2).abs() <= 0.01 && (line.x1 - table_right).abs() <= 0.2);
    assert!(
        has_top && has_bottom && has_left && has_right,
        "p{human_page} pi={} physical frame incomplete: top={has_top} bottom={has_bottom} left={has_left} right={has_right}",
        expected.para_index,
    );
    let painted_top = frame
        .iter()
        .filter(|line| (line.y1 - line.y2).abs() <= 0.01 && (line.y1 - table.bbox.y).abs() <= 0.2)
        .map(|line| line.y1 - line.style.width / 2.0)
        .fold(f64::INFINITY, f64::min);
    assert!(
        painted_top >= clip.y,
        "p{human_page} pi={} top frame paint escaped Body clip: painted={painted_top}, clip={clip:?}",
        expected.para_index,
    );

    let owners = line_owners(table);
    assert!(
        !owners.is_empty(),
        "p{human_page} pi={} visible cell line owners missing",
        expected.para_index,
    );
    let actual_paragraphs: BTreeSet<_> = owners.iter().map(|(para_index, _)| *para_index).collect();
    let expected_paragraphs: BTreeSet<_> =
        (expected.cell_para_start..expected.cell_para_end).collect();
    assert_eq!(
        actual_paragraphs, expected_paragraphs,
        "p{human_page} pi={} cell paragraph owners changed",
        expected.para_index,
    );
    owners
}

#[test]
fn issue_3820_stored_reset_tables_match_hancom_fragment_geometry_without_moving_owners() {
    let doc = document();
    assert_eq!(
        doc.page_count(),
        215,
        "policy fixture page ownership changed"
    );

    // First-fragment heights are the exact stored head heights.  Successor heights are the
    // measured reset tails; their x/y origin alone receives the physical outer-margin inset.
    let pairs = [
        (
            FragmentExpectation {
                page_index: 166,
                para_index: 1775,
                host_vpos_hu: Some(45_848),
                cell_height_hu: 23_282,
                cell_para_start: 0,
                cell_para_end: 5,
            },
            FragmentExpectation {
                page_index: 167,
                para_index: 1775,
                host_vpos_hu: None,
                cell_height_hu: 41_282,
                cell_para_start: 5,
                cell_para_end: 12,
            },
        ),
        (
            FragmentExpectation {
                page_index: 171,
                para_index: 1806,
                host_vpos_hu: Some(45_848),
                cell_height_hu: 15_282,
                cell_para_start: 0,
                cell_para_end: 6,
            },
            FragmentExpectation {
                page_index: 172,
                para_index: 1806,
                host_vpos_hu: None,
                cell_height_hu: 25_282,
                cell_para_start: 6,
                cell_para_end: 19,
            },
        ),
        (
            FragmentExpectation {
                page_index: 212,
                para_index: 2548,
                host_vpos_hu: Some(58_200),
                cell_height_hu: 11_282,
                cell_para_start: 0,
                cell_para_end: 6,
            },
            FragmentExpectation {
                page_index: 213,
                para_index: 2548,
                host_vpos_hu: None,
                cell_height_hu: 15_282,
                cell_para_start: 6,
                cell_para_end: 12,
            },
        ),
    ];

    for (first, successor) in pairs {
        let first_owners = assert_fragment_geometry(&doc, first);
        let successor_owners = assert_fragment_geometry(&doc, successor);
        assert!(
            first_owners.is_disjoint(&successor_owners),
            "pi={} duplicated cell line owners across p{}→p{}: first={first_owners:?}, successor={successor_owners:?}",
            first.para_index,
            first.page_index + 1,
            successor.page_index + 1,
        );
    }
}
