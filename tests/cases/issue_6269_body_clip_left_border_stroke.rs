//! Issue #6269: 본문 좌단에 붙은 세로 테두리선의 획 절반이 body clip 에 잘리던 회귀 가드.
//!
//! 세로선 render 노드의 bbox 는 `x .. x+width` 로 잡히지만 실제 획은 경로 중심 기준이라
//! `x-width/2 .. x+width/2` 를 덮는다. body clip 을 bbox 합집합으로만 잡으면 본문 좌단
//! (`body_area.x`)에 놓인 세로선은 왼쪽 절반이 clip 밖으로 나가 잘린다. PDF 직접 출력은
//! clip 을 적용하지 않아 온전했고 clip 을 지키는 studio(CanvasKit)·SVG 만 잉크가 절반이
//! 됐다(156739836 2·3쪽: studio 잉크 51 vs PDF 94).
//!
//! 여기서는 body clipRect 아래의 모든 line op 에 대해 **획 왼쪽 끝**이 clip 안에 있는지
//! 본다. 오른쪽·아래는 부동 개체 상한(#5855)이 따로 걸려 있어 왼쪽 경계만 판정한다.

use serde_json::Value;
use std::fs;
use std::path::Path;

const TOLERANCE: f64 = 1e-6;
const MAX_PAGES: u32 = 3;

/// 획 왼쪽 끝이 body clip 왼쪽 경계보다 얼마나 밖으로 나갔는지(px). 양수면 잘린다.
struct Scan {
    worst_deficit: f64,
    worst_label: String,
    lines_checked: usize,
}

fn walk(node: &Value, body_left: Option<f64>, page: u32, scan: &mut Scan) {
    match node.get("kind").and_then(Value::as_str) {
        Some("clipRect") => {
            let clip_kind = node.get("clipKind").and_then(Value::as_str).unwrap_or("");
            // 셀 clip 안의 자손은 body clip 확장 대상이 아니다(#42065 RowBreak 중첩 표).
            if clip_kind == "tableCell" {
                return;
            }
            let next = if clip_kind == "body" {
                node.get("clip")
                    .and_then(|clip| clip.get("x"))
                    .and_then(Value::as_f64)
            } else {
                body_left
            };
            if let Some(child) = node.get("child") {
                walk(child, next, page, scan);
            }
        }
        Some("group") => {
            for child in node
                .get("children")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                walk(child, body_left, page, scan);
            }
        }
        Some("leaf") => {
            let Some(left) = body_left else {
                return;
            };
            for op in node
                .get("ops")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
            {
                if op.get("type").and_then(Value::as_str) != Some("line") {
                    continue;
                }
                let x1 = op.get("x1").and_then(Value::as_f64).unwrap_or(0.0);
                let x2 = op.get("x2").and_then(Value::as_f64).unwrap_or(0.0);
                let width = op
                    .get("style")
                    .and_then(|style| style.get("width"))
                    .and_then(Value::as_f64)
                    .unwrap_or(0.0);
                if !width.is_finite() || width <= 0.0 {
                    continue;
                }
                scan.lines_checked += 1;
                let stroke_left = x1.min(x2) - width / 2.0;
                let deficit = left - stroke_left;
                if deficit > scan.worst_deficit {
                    scan.worst_deficit = deficit;
                    scan.worst_label = format!(
                        "page {page}: line x={:.2} width={:.2} 획 왼쪽 {stroke_left:.2} < clip {left:.2}",
                        x1.min(x2),
                        width
                    );
                }
            }
        }
        _ => {}
    }
}

fn scan_sample(relative: &str) -> Scan {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let bytes = fs::read(&path).unwrap_or_else(|err| panic!("read {relative}: {err}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|err| panic!("parse {relative}: {err:?}"));
    let mut scan = Scan {
        worst_deficit: 0.0,
        worst_label: String::new(),
        lines_checked: 0,
    };
    for page in 0..doc.page_count().min(MAX_PAGES) {
        let json = doc
            .get_page_layer_tree_native(page)
            .unwrap_or_else(|err| panic!("{relative} page {page} layer tree: {err:?}"));
        let value: Value = serde_json::from_str(&json)
            .unwrap_or_else(|err| panic!("{relative} page {page} layer tree json: {err}"));
        if let Some(root) = value.get("root") {
            walk(root, None, page, &mut scan);
        }
    }
    scan
}

fn assert_left_stroke_inside_body_clip(relative: &str) {
    let scan = scan_sample(relative);
    assert!(
        scan.lines_checked > 0,
        "{relative}: body clip 아래에서 검사한 line op 가 없다 — 가드가 비어 있다"
    );
    assert!(
        scan.worst_deficit <= TOLERANCE,
        "{relative}: 세로선 획 왼쪽 {:.3}px 이 body clip 밖이다 ({})",
        scan.worst_deficit,
        scan.worst_label
    );
}

#[test]
fn issue_6269_multi_column_doc_keeps_left_border_stroke_inside_body_clip() {
    // 1.5px 세로 테두리선 — 이슈 원문(156739836)과 같은 굵기.
    assert_left_stroke_inside_body_clip("samples/hwp-multi-001.hwp");
}

#[test]
fn issue_6269_thin_border_keeps_left_stroke_inside_body_clip() {
    // 0.5px 얇은 선도 절반이 잘리면 화면에서 사라진다.
    assert_left_stroke_inside_body_clip("samples/basic/BlogForm_Recipe.hwp");
}

#[test]
fn issue_6269_thick_border_keeps_left_stroke_inside_body_clip() {
    // 굵은 선(5.7px)은 잘림 폭이 커 육안으로도 드러났다.
    assert_left_stroke_inside_body_clip("samples/pr-1674.hwp");
}

#[test]
fn issue_6269_table_border_keeps_left_stroke_inside_body_clip() {
    assert_left_stroke_inside_body_clip("samples/issue_1133.hwp");
}
