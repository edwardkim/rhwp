//! 쪽에서 나뉜 행의 칸은 위아래가 잘려 그려진다. 잘려 보이지 않는 줄은 클릭으로
//! 고를 수 없고, 그 자리를 누르면 같은 칸의 보이는 줄로 간다.
//!
//! `rowbreak-problem-pages.hwpx` 16쪽(0부터 15)의 나뉜 칸은 문단 5의 두 줄을 칸 clip
//! 아래에 둔다. 그림에서는 보이지 않는데, 그 자리를 누르면 보이지 않는 글자에 캐럿이 갔다.

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::Value;

const SAMPLE: &str = "samples/rowbreak-problem-pages.hwpx";
const PAGE: u32 = 15;

/// 쪽 조각 칸의 clip(위, 아래) 안팎으로 나눈 글자 런 bbox.
fn clipped_runs(
    node: &RenderNode,
    clip: Option<(f64, f64)>,
    out: &mut Vec<(BoundingBox, f64, f64)>,
) {
    let clip = match &node.node_type {
        RenderNodeType::TableCell(cell) if cell.clip => {
            let (top, bottom) = (node.bbox.y, node.bbox.y + node.bbox.height);
            Some(clip.map_or((top, bottom), |(t, b): (f64, f64)| {
                (t.max(top), b.min(bottom))
            }))
        }
        _ => clip,
    };
    if let (RenderNodeType::TextRun(run), Some((top, bottom))) = (&node.node_type, clip) {
        let b = &node.bbox;
        if !run.text.trim().is_empty() && (b.y > bottom || b.y + b.height < top) {
            out.push((b.clone(), top, bottom));
        }
    }
    for child in &node.children {
        clipped_runs(child, clip, out);
    }
}

#[test]
fn clicking_a_line_clipped_off_a_split_row_lands_on_a_visible_line() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let doc = HwpDocument::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let tree = doc.build_page_render_tree(PAGE).unwrap();
    let mut hidden = Vec::new();
    clipped_runs(&tree.root, None, &mut hidden);
    assert!(
        !hidden.is_empty(),
        "재현 문서가 더는 칸 clip 밖에 줄을 두지 않는다"
    );
    for (bbox, top, bottom) in hidden {
        let (x, y) = (bbox.x + bbox.width / 2.0, bbox.y + bbox.height / 2.0);
        let hit: Value = serde_json::from_str(&doc.hit_test_native(PAGE, x, y).unwrap()).unwrap();
        let rect = &hit["cursorRect"];
        assert_eq!(rect["pageIndex"], PAGE, "{hit}");
        let caret_mid = rect["y"].as_f64().unwrap() + rect["height"].as_f64().unwrap() / 2.0;
        assert!(
            (top..=bottom).contains(&caret_mid),
            "({x:.1}, {y:.1}) 클릭이 clip {top:.1}..{bottom:.1} 밖 캐럿으로 갔다: {hit}"
        );
    }
}
