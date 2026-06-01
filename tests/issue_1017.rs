//! Issue #1017: BehindText / InFrontOfText direct replay z-order contract.
//!
//! `복학원서.hwp` 1페이지의 중앙 baked watermark는 PageLayerTree raw order상
//! 본문 textRun 뒤에 나오지만, `wrap=behindText` 이므로 direct replay backend에서는
//! flow text보다 먼저 그려야 한다.

use serde_json::Value;

#[derive(Debug)]
struct OrderedOp<'a> {
    seq: usize,
    path: String,
    op: &'a Value,
}

#[test]
fn issue_1017_baked_watermark_is_raw_ordered_after_text_but_marked_behind_text() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = std::path::Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = std::fs::read(&hwp_path).expect("read 복학원서.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse 복학원서.hwp");

    let json = doc
        .get_page_layer_tree_native(0)
        .expect("layer tree page 1");
    let parsed: Value = serde_json::from_str(&json).expect("PageLayerTree JSON");
    let mut ops = Vec::new();
    collect_ops_in_tree_order(&parsed["root"], "root".to_string(), &mut ops);

    let first_text = ops
        .iter()
        .find(|entry| op_type(entry.op) == Some("textRun"))
        .expect("first textRun op");
    let baked_watermarks: Vec<_> = ops
        .iter()
        .filter(|entry| {
            op_type(entry.op) == Some("image")
                && entry.op["bakedWatermark"].as_bool() == Some(true)
                && entry.op["watermark"]["preset"].as_str().is_some()
        })
        .collect();

    assert_eq!(
        baked_watermarks.len(),
        1,
        "fixture should expose exactly one baked watermark image op"
    );
    let watermark = baked_watermarks[0];

    assert!(
        watermark.seq > first_text.seq,
        "raw PageLayerTree order should demonstrate the #1017 failure mode: \
         first text seq={}, watermark seq={}, watermark path={}",
        first_text.seq,
        watermark.seq,
        watermark.path
    );
    assert_eq!(watermark.op["wrap"], "behindText");
    assert_eq!(watermark.op["mime"], "image/png");
    assert_eq!(watermark.op["bakedWatermark"], true);
}

#[test]
fn issue_1017_canvaskit_replay_plan_exposes_baked_watermark_plane() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = std::path::Path::new(repo_root).join("samples/복학원서.hwp");
    let bytes = std::fs::read(&hwp_path).expect("read 복학원서.hwp");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse 복학원서.hwp");

    let json = doc
        .get_canvaskit_replay_plan_native(0, "default")
        .expect("CanvasKit replay plan page 1");
    let plan: Value = serde_json::from_str(&json).expect("CanvasKit replay plan JSON");
    let items = plan["items"]
        .as_array()
        .expect("CanvasKit replay plan items");

    let page_background = items
        .iter()
        .find(|item| item["opType"].as_str() == Some("pageBackground"))
        .expect("pageBackground replay item");
    let text_run = items
        .iter()
        .find(|item| item["opType"].as_str() == Some("textRun"))
        .expect("textRun replay item");
    let baked_watermark = items
        .iter()
        .find(|item| {
            item["opType"].as_str() == Some("image")
                && item["detail"]
                    .as_str()
                    .is_some_and(|detail| detail.contains("resolved=bakedWatermark"))
        })
        .expect("baked watermark replay item");

    assert_eq!(page_background["replayPlane"].as_str(), Some("background"));
    assert_eq!(text_run["replayPlane"].as_str(), Some("flow"));
    assert_eq!(baked_watermark["replayPlane"].as_str(), Some("behindText"));
    assert!(
        baked_watermark["detail"]
            .as_str()
            .is_some_and(|detail| detail.contains("wrap=behindText")),
        "baked watermark item should keep wrap detail: {baked_watermark:?}"
    );
}

fn collect_ops_in_tree_order<'a>(node: &'a Value, path: String, out: &mut Vec<OrderedOp<'a>>) {
    match node["kind"].as_str() {
        Some("group") => {
            let children = node["children"]
                .as_array()
                .expect("group node must have children");
            for (idx, child) in children.iter().enumerate() {
                collect_ops_in_tree_order(child, format!("{path}/g{idx}"), out);
            }
        }
        Some("clipRect") => {
            collect_ops_in_tree_order(&node["child"], format!("{path}/clip"), out);
        }
        Some("leaf") => {
            let ops = node["ops"].as_array().expect("leaf node must have ops");
            for (idx, op) in ops.iter().enumerate() {
                out.push(OrderedOp {
                    seq: out.len() + 1,
                    path: format!("{path}/op{idx}"),
                    op,
                });
            }
        }
        other => panic!("unexpected layer node kind at {path}: {other:?}"),
    }
}

fn op_type(op: &Value) -> Option<&str> {
    op["type"].as_str()
}
