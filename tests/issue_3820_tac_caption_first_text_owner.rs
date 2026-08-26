//! Issue #3820 Stage 121 — Bottom-caption TAC 표가 자신의 첫 저장 줄을 소유할 때
//! 표 뒤 host text를 오른쪽 잔여 폭으로 빼앗지 않는 회귀를 고정한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str =
    "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp";
const PAGE_33: u32 = 32;
const PARA_428: usize = 428;
const URL: &str = "statistics.eurotransplant.org)";

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

fn collect_url_runs(node: &RenderNode, out: &mut Vec<(String, BoundingBox)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        let is_target_paragraph = run.para_index == Some(PARA_428);
        let is_url_fragment =
            run.text.contains("statistics.euro") || run.text.contains("ansplant.org)");
        if is_target_paragraph && is_url_fragment {
            out.push((run.text.clone(), node.bbox));
        }
    }
    for child in &node.children {
        collect_url_runs(child, out);
    }
}

#[test]
fn issue_3820_bottom_caption_tac_keeps_following_url_below_the_table() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|err| panic!("read {SAMPLE}: {err}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse policy authority fixture");
    assert_eq!(core.page_count(), 215, "Hancom PDF의 215쪽을 유지해야 함");

    let tree = core
        .build_page_render_tree(PAGE_33)
        .expect("render physical p33");
    let table = table_for_paragraph(&tree.root, PARA_428).expect("p33 leading TAC table");
    let table_right = table.bbox.x + table.bbox.width;
    let table_bottom = table.bbox.y + table.bbox.height;

    let mut url_runs = Vec::new();
    collect_url_runs(&tree.root, &mut url_runs);
    url_runs.sort_by(|(_, left), (_, right)| {
        left.y
            .total_cmp(&right.y)
            .then_with(|| left.x.total_cmp(&right.x))
    });
    let rendered_url: String = url_runs.iter().map(|(text, _)| text.as_str()).collect();
    assert_eq!(rendered_url, URL, "p33 host URL이 누락·중복·분할되면 안 됨");

    let first = url_runs.first().expect("p33 URL first run").1;
    assert!(
        first.x <= table.bbox.x + 12.0,
        "URL은 표 오른쪽 잔여 폭이 아니라 다음 줄 시작점에서 시작해야 함: url_x={} table_x={} table_right={table_right}",
        first.x,
        table.bbox.x,
    );
    assert!(
        first.y >= table_bottom - 0.5,
        "URL은 Bottom caption TAC 표 아래에서 시작해야 함: url_y={} table_bottom={table_bottom}",
        first.y,
    );
    assert!(
        url_runs
            .iter()
            .all(|(_, bbox)| (bbox.y - first.y).abs() <= 0.5),
        "URL은 표 오른쪽과 다음 줄로 나뉘면 안 됨: {url_runs:?}",
    );
    assert!(
        url_runs.iter().all(|(_, bbox)| bbox.x < table_right),
        "URL 조각이 표 오른쪽 잔여 폭을 침범하면 안 됨: {url_runs:?}",
    );
}
