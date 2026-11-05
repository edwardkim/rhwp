//! 관계형: split_row_clip_hit의 공개 입력·편집·저장 계약.
//! 주석 갱신: 2026-11-04T00:23:31+09:00; 직전 파일 변경 커밋: 2026-11-03T21:45:03+09:00 39b4f81f6aab4cc08c6145ed6911db2a8fe5125e.
//! 측정: 2026-11-03, 96 DPI·색상 경계 보정 전 2px raw 실루엣. rowbreak-problem-pages.hwpx 전 18쪽: Native/fresh WASM 각각 최저 92.88%; 16쪽 98.37%/98.49%. 독립 Print 확대 판독 완료. 절대 좌표 핀 없이 실제 clip과 셀 경로·마지막 내용 보존을 검사한다.
//! 정식 증거: mydocs/pr/assets/semanticist21-20261103/validation.md 및 전쪽 TSV·확대 PNG.
use rhwp::renderer::layout::CellContext;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;
use serde_json::{json, Value};

const SAMPLE: &str = "samples/rowbreak-problem-pages.hwpx";
const PAGE: u32 = 15;

struct ClippedRun {
    bbox: BoundingBox,
    top: f64,
    bottom: f64,
    context: CellContext,
    text: String,
}
fn clipped_runs(node: &RenderNode, clip: Option<(f64, f64)>, out: &mut Vec<ClippedRun>) {
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
        if !run.text.trim().is_empty() {
            if let Some(context) = &run.cell_context {
                out.push(ClippedRun {
                    bbox: node.bbox,
                    top,
                    bottom,
                    context: context.clone(),
                    text: run.text.clone(),
                });
            }
        }
    }
    for child in &node.children {
        clipped_runs(child, clip, out);
    }
}
#[test]
fn split_row_visible_lines_keep_content_and_hit_ownership() {
    let doc = HwpDocument::from_bytes(&std::fs::read(SAMPLE).unwrap()).unwrap();
    let tree = doc.build_page_render_tree(PAGE).unwrap();
    let mut runs = Vec::new();
    clipped_runs(&tree.root, None, &mut runs);
    let visible: Vec<_> = runs
        .iter()
        .filter(|r| r.bbox.y < r.bottom && r.bbox.y + r.bbox.height > r.top)
        .collect();
    assert!(
        !visible.is_empty(),
        "분할 셀의 보이는 내용은 반드시 검사한다"
    );
    let text: String = visible.iter().map(|r| r.text.as_str()).collect();
    assert!(text.contains("공공데이터"), "마지막 문단 보존: {text}");
    assert!(text.contains("고려하여 추진"), "마지막 문단 보존: {text}");
    for run in &runs {
        let b = &run.bbox;
        if b.width <= 0.0 {
            continue;
        }
        let top = b.y.max(run.top);
        let bottom = (b.y + b.height).min(run.bottom);
        let visible = top < bottom;
        let x = b.x + b.width / 2.0;
        let y = if visible {
            (top + bottom) / 2.0
        } else {
            b.y + b.height / 2.0
        };
        let hit: Value = serde_json::from_str(&doc.hit_test_native(PAGE, x, y).unwrap()).unwrap();
        let rect = &hit["cursorRect"];
        assert_eq!(rect["pageIndex"], PAGE, "{hit}");
        let mid = rect["y"].as_f64().unwrap() + rect["height"].as_f64().unwrap() / 2.0;
        assert!((run.top..=run.bottom).contains(&mid), "clip 밖 캐럿: {hit}");
        if visible {
            let expected: Vec<_> = run.context.path.iter().map(|p| json!({"controlIndex":p.control_index,"cellIndex":p.cell_index,"cellParaIndex":p.cell_para_index})).collect();
            assert_eq!(
                hit["parentParaIndex"], run.context.parent_para_index,
                "{hit}"
            );
            assert_eq!(hit["cellPath"], json!(expected), "{hit}");
        }
    }
}
