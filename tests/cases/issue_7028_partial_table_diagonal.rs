//! #7028: 제목행 반복과 셀 대각선은 서로 다른 계약이다.
//! 실물 PDF의 온전한 제목 셀을 기준으로 검사한다. IR 변형은 경계 검사일 뿐
//! 한컴에서 생성한 정상 문서나 실제 분할 셀의 대각선 정답지로 쓰지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::{
    control::Control,
    document::Document,
    table::{Table, TableZone},
};
use rhwp::renderer::render_tree::{BoundingBox, LineNode, RenderNode, RenderNodeType};
use rhwp::renderer::StrokeDash;
use rhwp::DocumentCore;

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/task2146/21761835_jeonjik_exemption_table.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).expect("required fixture")).expect("HWP parse")
}

fn table_mut(doc: &mut Document) -> &mut Table {
    let Control::Table(table) = &mut doc.sections[0].paragraphs[4].controls[0] else {
        panic!("fixture table pi=4 ci=0")
    };
    table
}

fn variant(edit: impl FnOnce(&mut Document)) -> DocumentCore {
    let mut core = core();
    let mut doc = core.document().clone();
    edit(&mut doc);
    core.set_document(doc);
    core
}

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = vec![node];
    for child in &node.children {
        out.extend(nodes(child));
    }
    out
}

fn cell_box(root: &RenderNode, row: u16, col: u16) -> Option<BoundingBox> {
    nodes(root).into_iter().find_map(|n| match &n.node_type {
        RenderNodeType::TableCell(c) if c.row == row && c.col == col => Some(n.bbox),
        _ => None,
    })
}

fn diagonals(root: &RenderNode) -> Vec<&LineNode> {
    nodes(root)
        .into_iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Line(line)
                if (line.x2 - line.x1).abs() > 1.0 && (line.y2 - line.y1).abs() > 1.0 =>
            {
                Some(line)
            }
            _ => None,
        })
        .collect()
}

fn assert_diagonal(root: &RenderNode, row: u16, col: u16) {
    let b = cell_box(root, row, col).expect("cell must be rendered");
    let near = |a: f64, b: f64| (a - b).abs() < 0.02;
    let matching: Vec<_> = diagonals(root)
        .into_iter()
        .filter(|l| {
            near(l.x1, b.x)
                && near(l.y1, b.y)
                && near(l.x2, b.x + b.width)
                && near(l.y2, b.y + b.height)
        })
        .collect();
    assert_eq!(
        matching.len(),
        1,
        "r{row}c{col} must have exactly one corner-to-corner diagonal, bbox={b:?}"
    );
    assert_eq!(matching[0].style.color, 0);
    assert_eq!(matching[0].style.dash, StrokeDash::Solid);
    assert!(matching[0].style.width > 0.0);
}

#[test]
fn real_header_diagonal_is_present_once_on_all_six_pages() {
    let core = core();
    let mut doc = core.document().clone();
    let table = table_mut(&mut doc);
    assert_eq!((table.row_count, table.col_count), (78, 5));
    assert!(table.repeat_header);
    assert_eq!(table.leading_header_rows(), vec![0]);
    assert_eq!(table.cells[0].border_fill_id, 13);
    assert_eq!(doc.doc_info.border_fills[12].attr, 0x40);
    assert_eq!(core.page_count(), 6);
    for page in 0..6 {
        let tree = core.build_page_render_tree(page).expect("page tree");
        let cells: Vec<_> = nodes(&tree.root)
            .into_iter()
            .filter(|n| matches!(&n.node_type,RenderNodeType::TableCell(c) if c.row==0))
            .collect();
        assert_eq!(cells.len(), 5, "p{} repeated header", page + 1);
        for cell in cells {
            assert!(
                (cell.bbox.height - 52.4).abs() < 0.1,
                "#2146 height protection"
            );
            let y = if page == 0 { 225.3 } else { 113.4 };
            assert!((cell.bbox.y - y).abs() < 0.1, "header y changed");
        }
        assert_diagonal(&tree.root, 0, 0);
        assert_eq!(
            diagonals(&tree.root).len(),
            1,
            "duplicate or stray diagonal"
        );
        let svg = core.render_page_svg_native(page).expect("SVG export");
        let diagonal_count = svg
            .split("<line ")
            .skip(1)
            .filter(|part| {
                let tag = part.split('>').next().unwrap();
                let coord = |key: &str| {
                    let needle = format!("{key}=\"");
                    let rest = &tag[tag.find(&needle).expect("line coordinate") + needle.len()..];
                    rest.split('"').next().unwrap().parse::<f64>().unwrap()
                };
                (coord("x2") - coord("x1")).abs() > 1.0 && (coord("y2") - coord("y1")).abs() > 1.0
            })
            .count();
        assert_eq!(diagonal_count, 1, "SVG p{}", page + 1);
    }
}

#[test]
fn disabling_header_repeat_does_not_copy_first_row() {
    let core = variant(|d| table_mut(d).repeat_header = false);
    assert!(core.page_count() > 1);
    assert_diagonal(&core.build_page_render_tree(0).unwrap().root, 0, 0);
    for p in 1..core.page_count() {
        let tree = core.build_page_render_tree(p).unwrap();
        assert!(cell_box(&tree.root, 0, 0).is_none());
        assert!(diagonals(&tree.root).is_empty());
    }
}

#[test]
fn no_diagonal_style_stays_absent() {
    for remove_type in [false, true] {
        let core = variant(|d| {
            if remove_type {
                d.doc_info.border_fills[12].diagonal.diagonal_type = 0;
            } else {
                d.doc_info.border_fills[12].attr = 0;
            }
        });
        for p in 0..core.page_count() {
            assert!(diagonals(&core.build_page_render_tree(p).unwrap().root).is_empty());
        }
    }
}

#[test]
fn vertical_header_uses_the_same_diagonal_emission() {
    let core = variant(|d| table_mut(d).cells[0].text_direction = 1);
    assert_diagonal(&core.build_page_render_tree(0).unwrap().root, 0, 0);
    assert_diagonal(&core.build_page_render_tree(1).unwrap().root, 0, 0);
}

#[test]
fn complete_body_cell_is_not_restricted_to_headers() {
    let core = variant(|d| {
        let t = table_mut(d);
        let c = t
            .cells
            .iter_mut()
            .find(|c| c.row == 1 && c.col == 2)
            .unwrap();
        assert!(!c.is_header);
        c.border_fill_id = 13;
    });
    assert_diagonal(&core.build_page_render_tree(0).unwrap().root, 1, 2);
}

#[test]
fn active_zone_only_excludes_intersecting_cells() {
    for (row, col, expected) in [(0, 0, 0), (1, 2, 1)] {
        let core = variant(|d| {
            table_mut(d).zones.push(TableZone {
                start_row: row,
                end_row: row,
                start_col: col,
                end_col: col,
                border_fill_id: 13,
            })
        });
        let tree = core.build_page_render_tree(0).unwrap();
        assert_eq!(diagonals(&tree.root).len(), expected, "zone {row},{col}");
    }
}

#[test]
fn inactive_zone_does_not_disable_a_cell_diagonal() {
    let core = variant(|d| {
        table_mut(d).zones.push(TableZone {
            start_row: 0,
            end_row: 0,
            start_col: 0,
            end_col: 0,
            border_fill_id: 14,
        })
    });
    assert_diagonal(&core.build_page_render_tree(0).unwrap().root, 0, 0);
}

#[test]
fn complete_rowspan_keeps_its_whole_cell_diagonal() {
    let core = variant(|d| {
        let c = table_mut(d)
            .cells
            .iter_mut()
            .find(|c| c.row == 6 && c.col == 2)
            .unwrap();
        assert_eq!(c.row_span, 2);
        c.border_fill_id = 13;
    });
    assert_diagonal(&core.build_page_render_tree(0).unwrap().root, 6, 2);
}

#[test]
fn cut_row_and_straddling_rowspan_do_not_gain_fragment_diagonals() {
    // Row 9 has end-cut metadata but finishes on p1 (no second instance).
    // The rowspan 6..11 actually crosses p1/p2. Do not equate cut with repetition.
    // Only the diagonal property is varied; no synthetic paper size is introduced.
    for (row, col) in [(9, 2), (6, 0)] {
        let core = variant(|d| {
            table_mut(d)
                .cells
                .iter_mut()
                .find(|c| c.row == row && c.col == col)
                .unwrap()
                .border_fill_id = 13;
        });
        let mut instances = 0;
        let mut page_fragments = 0;
        for p in 0..core.page_count() {
            let tree = core.build_page_render_tree(p).unwrap();
            if cell_box(&tree.root, row, col).is_some() {
                instances += 1;
                page_fragments += nodes(&tree.root)
                    .iter()
                    .filter(|n| {
                        matches!(&n.node_type, RenderNodeType::TableCell(c)
                        if c.row == row && c.col == col && c.page_fragment)
                    })
                    .count();
                assert_eq!(
                    diagonals(&tree.root).len(),
                    1,
                    "only intact header may emit: r{row}c{col}, p{p}"
                );
                assert_diagonal(&tree.root, 0, 0);
            }
        }
        assert!(
            page_fragments > 0,
            "r{row}c{col} must exercise fragment metadata"
        );
        if row == 6 {
            assert!(
                instances >= 2,
                "rowspan must actually cross a page boundary"
            );
        } else {
            assert_eq!(instances, 1, "completed end-cut row is not repeated");
        }
    }
}
