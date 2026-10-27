//! 정상 한컴 저장 RowBreak의 50/10행·뒤 본문을 독립 Print 좌표와 함께 보호한다.
//! 양쪽 전쪽 raw100·직접 판독을 마친 페이지의 ±2px이며 내용·소유 관계도 유지한다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;
use serde_json::Value;
use sha2::{Digest, Sha256};

fn collect<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        collect(child, out);
    }
}

fn assert_sha(bytes: &[u8], expected: &str) {
    let actual: String = Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(actual, expected, "독립 Print와 같은 원본·기준 파일");
}

#[test]
fn verified_rowbreak_fragments_and_successor_keep_print_frames() {
    let reference: Value = serde_json::from_str(include_str!(
        "../../mydocs/pr/assets/cell_picture_layer_qualified_20261026/rowbreak-expected.json"
    ))
    .unwrap();
    assert_eq!(reference["dpi"], 96);
    assert_eq!(reference["compat"], "2022");
    assert_eq!(reference["profile"], "print");
    assert_eq!(reference["native_raw"], "100.00000");
    assert_eq!(reference["fresh_wasm_raw"], "100.00000");
    let input = include_bytes!(
        "../../mydocs/pr/assets/float_fragment_frames_20261020/inputs/hwp/public-rowbreak-normalized-2020.hwp"
    );
    assert_sha(input, reference["input_sha256"].as_str().unwrap());
    let pdf = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(reference["pdf"].as_str().unwrap()),
    )
    .unwrap();
    assert_sha(&pdf, reference["pdf_sha256"].as_str().unwrap());
    let mut core = DocumentCore::from_bytes(input).unwrap();
    core.set_hangul2024_compat(false);
    assert_eq!(core.page_count(), 2, "독립 Print의 전체 쪽수");
    let mut all_cells = BTreeSet::new();
    let mut successor_count = 0;
    for page in 0..2 {
        let tree = core.build_page_render_tree(page).unwrap();
        let mut nodes = Vec::new();
        collect(&tree.root, &mut nodes);
        let tables: Vec<_> = nodes
            .iter()
            .filter(|n| matches!(n.node_type, RenderNodeType::Table(_)))
            .collect();
        assert_eq!(tables.len(), 1, "각 쪽의 표 조각 하나");
        let frame = tables[0].bbox;
        let expected = &reference["pages"][page as usize];
        let edges = expected["table_border_centerline_edges_px"]
            .as_array()
            .unwrap();
        for (actual, independent) in [
            frame.x,
            frame.y,
            frame.x + frame.width,
            frame.y + frame.height,
        ]
        .into_iter()
        .zip(edges)
        {
            assert!(
                (actual - independent.as_f64().unwrap()).abs() <= 2.0,
                "p{}: 표 경계 actual={actual}, independent Print={independent}",
                page + 1
            );
        }
        let cells: Vec<_> = nodes
            .iter()
            .filter_map(|n| match &n.node_type {
                RenderNodeType::TableCell(cell) => Some((cell.row, cell.col)),
                _ => None,
            })
            .collect();
        assert_eq!(
            cells.len(),
            expected["printed_rows"].as_u64().unwrap() as usize * 2
        );
        for address in cells {
            assert!(
                all_cells.insert(address),
                "쪽 사이 셀 중복 없음: {address:?}"
            );
        }
        for n in nodes {
            let RenderNodeType::TextRun(run) = &n.node_type else {
                continue;
            };
            if !run.text.contains("다음 문단") {
                continue;
            }
            successor_count += 1;
            assert_eq!(page, 1, "뒤 본문은 마지막 조각과 같은 2쪽");
            assert!(
                n.bbox.y >= frame.y + frame.height,
                "뒤 본문이 표를 덮지 않는다"
            );
            let first = &reference["successor_text_baselines"][0]["baseline_xy_px"];
            assert!((n.bbox.x - first[0].as_f64().unwrap()).abs() <= 2.0);
            assert!(
                (n.bbox.y + run.baseline - first[1].as_f64().unwrap()).abs() <= 2.0,
                "독립 Print의 뒤 본문 baseline을 보존한다"
            );
        }
    }
    let expected_cells: BTreeSet<_> = (0..60).flat_map(|row| [(row, 0), (row, 1)]).collect();
    assert_eq!(
        all_cells, expected_cells,
        "60행의 셀 소유·내용을 누락 없이 한 번씩 보존"
    );
    assert_eq!(successor_count, 1, "뒤 본문 누락·중복 없음");
}
