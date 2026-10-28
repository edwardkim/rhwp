//! Issue #1835: TAC 표에서 셀 내용이 저장 높이(common.height)를 크게(>1.5×) 초과하면
//! 비례 축소 대신 내용 높이로 확장한다 (한글 2022 편집기 오라클 정합).
//!
//! 기존 픽스처는 저장 높이7126→3950HU를 변경한4×3 TAC 표를 포함한다.
//! 독립 정본은 `pdf/issue1835_tac_stale_height-hwp-2020.pdf` 전3쪽이다.
//! Native/fresh WASM 전쪽 최저97.59989%와 직접 판독 근거는
//! `mydocs/pr/assets/planet6897_green_20261002/stale1835_body_border_validation.json`에 있다.
//! 높이 픽셀 핀 대신 칸 안의 내용 포함·행 비겹침과 본문/전체 표의 쪽 소속을 검사한다.

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue1835_tac_stale_height.hwp";

fn load_trees() -> Vec<rhwp::renderer::render_tree::PageRenderTree> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("{SAMPLE} 읽기: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("{SAMPLE} 해석: {e:?}"));
    assert_eq!(doc.page_count(), 3, "독립 한컴 정본은3쪽이다");
    (0..doc.page_count())
        .map(|page| {
            doc.build_page_render_tree(page)
                .unwrap_or_else(|e| panic!("{SAMPLE} {}쪽 출력: {e}", page + 1))
        })
        .collect()
}

fn find_table<'a>(node: &'a RenderNode, pi: usize, ci: usize) -> Option<&'a RenderNode> {
    if let RenderNodeType::Table(t) = &node.node_type {
        if t.para_index == Some(pi) && t.control_index == Some(ci) {
            return Some(node);
        }
    }
    node.children.iter().find_map(|c| find_table(c, pi, ci))
}

/// 실제 글자가 있는 출력 노드만 수집한다. 빈 칸의 편집용 줄은 본문이 아니다.
fn collect_text_runs<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if matches!(&node.node_type, RenderNodeType::TextRun(run) if !run.text.trim().is_empty()) {
        out.push(node);
    }
    for child in &node.children {
        collect_text_runs(child, out);
    }
}

/// 칸의 원문 소속과 칸 안의 내용 포함을 검사한다. 절대 위치/높이를 고정하지 않는다.
fn assert_whole_table_cells(table: &RenderNode, rows: u16, cols: u16) {
    let cells: Vec<_> = table
        .children
        .iter()
        .filter_map(|node| {
            if let RenderNodeType::TableCell(cell) = &node.node_type {
                Some((node, cell))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        cells.len(),
        usize::from(rows) * usize::from(cols),
        "전체 칸이 있어야 한다"
    );
    for row in 0..rows {
        for col in 0..cols {
            let (node, cell) = cells
                .iter()
                .find(|(_, cell)| cell.row == row && cell.col == col)
                .expect("원문의 모든 행/열 칸이 한 번씩 있어야 한다");
            assert!(
                !cell.page_fragment,
                "통째 표의 칸이 쪽 조각이어서는 안 된다"
            );
            if row > 0 {
                let (above, _) = cells
                    .iter()
                    .find(|(_, cell)| cell.row == row - 1 && cell.col == col)
                    .expect("앞 행의 같은 열");
                assert!(
                    node.bbox.y >= above.bbox.y + above.bbox.height,
                    "앞뒤 행이 겹치면 안 된다"
                );
            }
            let mut runs = Vec::new();
            collect_text_runs(node, &mut runs);
            for text in runs {
                assert!(
                    text.bbox.y >= node.bbox.y
                        && text.bbox.y + text.bbox.height <= node.bbox.y + node.bbox.height,
                    "칸 내용이 축소된 행의 위아래로 벗어났다: {:?}, 칸 {:?}",
                    text.bbox,
                    node.bbox
                );
            }
        }
    }
}

/// 저장 개체 높이가 작아도 첫 표의 모든 행 내용은 자기 칸에 들어간다.
#[test]
fn tac_stale_height_table_expands_to_content() {
    let pages = load_trees();
    let table = find_table(&pages[0].root, 3, 0).expect("첫쪽4×3 TAC 표");
    assert_whole_table_cells(table, 4, 3);
    let mut runs = Vec::new();
    collect_text_runs(table, &mut runs);
    let text: String = runs
        .iter()
        .filter_map(|node| {
            if let RenderNodeType::TextRun(run) = &node.node_type {
                Some(
                    run.text
                        .chars()
                        .filter(|ch| !ch.is_whitespace())
                        .collect::<String>(),
                )
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        text, "제목담당자세부내용",
        "원문 첫 행의 제목이 누락/중복되면 안 된다"
    );
}

/// 후속 제목은 첫 표 뒤에 있고, 마지막 제목과 통째 표는 정본의 쪽 소속을 유지한다.
#[test]
fn tac_stale_height_following_paragraph_flows_below_table() {
    let pages = load_trees();
    let table = find_table(&pages[0].root, 3, 0).expect("첫쪽4×3 TAC 표");
    let mut runs = Vec::new();
    collect_text_runs(&pages[0].root, &mut runs);
    let next = runs.iter().find(|node| {
        matches!(&node.node_type, RenderNodeType::TextRun(run)
            if run.para_index == Some(4) && run.cell_context.is_none() && run.text.contains("셀 편집"))
    }).expect("원문 후속 셀 편집 제목");
    assert!(
        next.bbox.y >= table.bbox.y + table.bbox.height,
        "후속 제목은 첫 표 아래에 있어야 한다"
    );

    let mut second_page_runs = Vec::new();
    collect_text_runs(&pages[1].root, &mut second_page_runs);
    assert!(
        second_page_runs.iter().any(|node| {
            matches!(&node.node_type, RenderNodeType::TextRun(run)
            if run.para_index == Some(20) && run.cell_context.is_none()
                && run.text.contains("표 다음 페이지로 넘기기"))
        }),
        "마지막 표의 제목은 정본2쪽에 남아야 한다"
    );
    assert!(
        find_table(&pages[1].root, 20, 0).is_none(),
        "마지막 표를2쪽에서 나누면 안 된다"
    );
    let last_table = find_table(&pages[2].root, 20, 0).expect("정본3쪽의 통째11행 표");
    assert_whole_table_cells(last_table, 11, 3);
}
