//! Issue #530: treatise sample.hwp 5페이지 TAC 표 Top caption 이 머리행 위에 겹치는 회귀.
//!
//! 본질: `treat_as_char=true` 표가 inline 위치로 렌더링될 때 Top caption 높이가
//! 표 본문 y 에 반영되지 않아 caption 과 머리행이 같은 y 영역에 렌더링됨.

use std::cmp::Ordering;
use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

#[test]
fn issue_530_tac_top_caption_does_not_overlap_header_row() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join("samples/basic/treatise sample.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse treatise sample.hwp");

    // page=5, global_idx=4
    let tree = doc
        .build_page_render_tree(4)
        .expect("render treatise sample.hwp page 5");

    let mut issue_tables = Vec::new();
    collect_issue_530_tables(&tree.root, &mut issue_tables);
    assert!(
        !issue_tables.is_empty(),
        "section=0 pi=60 ci=0 표 노드를 찾지 못함"
    );

    let table = issue_tables
        .into_iter()
        .min_by(|a, b| a.bbox.y.partial_cmp(&b.bbox.y).unwrap_or(Ordering::Equal))
        .expect("issue #530 표 노드 선택 실패");

    let mut caption_baselines = Vec::new();
    collect_issue_530_caption_baselines(&tree.root, &mut caption_baselines);
    assert!(
        caption_baselines.len() >= 2,
        "issue #530 표 caption TextRun 을 충분히 찾지 못함: {:?}",
        caption_baselines
    );

    let max_caption_baseline = caption_baselines
        .iter()
        .map(|(_, baseline)| *baseline)
        .fold(f64::NEG_INFINITY, f64::max);

    println!(
        "issue #530 table_top={:.2}, max_caption_baseline={:.2}, caption_runs={:?}",
        table.bbox.y, max_caption_baseline, caption_baselines
    );

    assert!(
        table.bbox.y > max_caption_baseline + 0.1,
        "TAC Top caption 이 표 머리행 위에 겹침: table_top={:.2}, \
         max_caption_baseline={:.2}, caption_runs={:?}",
        table.bbox.y,
        max_caption_baseline,
        caption_baselines
    );
}

fn collect_issue_530_tables<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.section_index == Some(0)
            && table.para_index == Some(60)
            && table.control_index == Some(0)
        {
            out.push(node);
        }
    }

    for child in &node.children {
        collect_issue_530_tables(child, out);
    }
}

fn collect_issue_530_caption_baselines(node: &RenderNode, out: &mut Vec<(String, f64)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if let Some(ctx) = &run.cell_context {
            if ctx.parent_para_index == 60
                && ctx
                    .path
                    .first()
                    .map(|entry| entry.control_index == 0 && entry.cell_index == 65534)
                    .unwrap_or(false)
            {
                out.push((run.text.clone(), node.bbox.y + run.baseline));
            }
        }
    }

    for child in &node.children {
        collect_issue_530_caption_baselines(child, out);
    }
}
