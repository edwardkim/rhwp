//! 같은 before 원본의 독립 Print도 자르는 나누지 않음 표를 보호한다.
//! 전쪽 Native/fresh raw100%와 직접 판독: issue7216_print_20261024 자산 참조.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions};
use rhwp::model::{control::Control, table::TablePageBreak};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use std::collections::BTreeMap;
use std::path::Path;

const INPUT: &str = "samples/issue7216/tall_table_before.hwpx";
const LAST_PRINTED_ROW: u16 = 53;

fn compact(text: &str) -> String {
    text.chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect()
}

fn text(node: &RenderNode) -> String {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        return run.text.clone();
    }
    node.children.iter().map(text).collect()
}

fn owned_table(node: &RenderNode, owner: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(owner)) {
        return Some(node);
    }
    node.children
        .iter()
        .find_map(|child| owned_table(child, owner))
}

fn baseline(root: &Path, name: &str) -> u64 {
    std::fs::read_to_string(root.join("tests/fixtures").join(name))
        .unwrap()
        .lines()
        .find_map(|line| {
            let (input, value) = line.split_once('\t')?;
            (input == INPUT.strip_prefix("samples/").unwrap()).then(|| value.parse().unwrap())
        })
        .expect("독립 Print로 입증한 의도된 넘침의 원장 행")
}

#[test]
fn no_divide_preserves_owned_rows_print_clipping_and_following_body() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let core = DocumentCore::from_bytes(&std::fs::read(root.join(INPUT)).unwrap()).unwrap();
    assert_eq!(core.page_count(), 2, "독립 Print 전체 쪽수");
    let (owner, source) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find_map(|(pi, p)| {
            p.controls.iter().find_map(|c| match c {
                Control::Table(t) => Some((pi, t.as_ref())),
                _ => None,
            })
        })
        .unwrap();
    assert_eq!(source.page_break, TablePageBreak::None);
    assert_eq!((source.row_count, source.col_count), (61, 3));
    let _ = core.take_overflow_cell_lines();
    let first = core.build_page_render_tree(0).unwrap();
    let first_overflow = u64::from(core.take_overflow_cell_lines());
    let second = core.build_page_render_tree(1).unwrap();
    let overflow_lines = first_overflow + u64::from(core.take_overflow_cell_lines());
    let table = owned_table(&first.root, owner).expect("표는 첫 쪽에 통째로 소유");
    assert!(
        owned_table(&second.root, owner).is_none(),
        "표 행을 다음 쪽에 중복하지 않음"
    );
    assert_eq!(compact(&text(&second.root)), "표다음문단");
    let cells: BTreeMap<_, _> = table
        .children
        .iter()
        .filter_map(|node| {
            if let RenderNodeType::TableCell(c) = &node.node_type {
                Some(((c.row, c.col), node))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        cells.len(),
        source.cells.len(),
        "잘린 원문도 모델·표 소유에서 제거하지 않음"
    );
    let page_bottom = first.root.bbox.y + first.root.bbox.height;
    let mut hidden = 0;
    for cell in &source.cells {
        let rendered = cells[&(cell.row, cell.col)];
        let expected: String = cell.paragraphs.iter().map(|p| p.text.as_str()).collect();
        assert_eq!(
            compact(&text(rendered)),
            compact(&expected),
            "셀 {}:{} 원문 보존",
            cell.row,
            cell.col
        );
        if cell.row > LAST_PRINTED_ROW {
            assert!(
                rendered.bbox.y > page_bottom,
                "Print에서도 잘린 행의 같은 물리 소유"
            );
            hidden += 1;
        } else if cell.row == LAST_PRINTED_ROW {
            assert!(rendered.bbox.y < page_bottom, "Print의 마지막 표시 행");
        }
    }
    assert_eq!(hidden, usize::from((60 - LAST_PRINTED_ROW) * 3));
    assert_eq!(
        overflow_lines, hidden as u64,
        "의도된 잘림도 진단에서는 계속 보고"
    );
    let options = AnomalyOptions {
        overflow_tolerance_px: 2.0,
        ..Default::default()
    };
    let scans = [
        scan_page(0, &first.root, 2, &options),
        scan_page(1, &second.root, 2, &options),
    ];
    let off_canvas = scans.iter().map(|s| s.off_canvas.len() as u64).sum::<u64>();
    let over_bottom = scans
        .iter()
        .flat_map(|s| s.overflow.iter())
        .filter(|a| a.over_bottom > 2.0)
        .count() as u64;
    assert_eq!(off_canvas, 1, "한 원본 표의 용지 밖 신호");
    assert_eq!(over_bottom, 1, "한 원본 표의 본문 밖 신호");
    assert_eq!(baseline(root, "overflow_cell_baseline.tsv"), overflow_lines);
    assert_eq!(baseline(root, "off_canvas_baseline.tsv"), off_canvas);
    assert_eq!(baseline(root, "body_overflow_baseline.tsv"), over_bottom);
}
