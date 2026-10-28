//! [#7193] 그림 안쪽 여백(`hp:inMargin` / HWP 그림 padding)은 틀 안에서 그림이 그려질
//! 자리를 줄인다. 틀(`hp:sz`)은 그대로 흐름·선택 기준이다.
//!
//! 재현 문서 (`samples/issue7193/`, 코퍼스 원본 그대로):
//! - `2983289_picture_inner_margin.hwpx` — '첨부파일' 안내 그림 1개, `tac=true`,
//!   `sz`=5400×4251(72.0×56.7px), `curSz`=0×1984, `inMargin top=2267`(30.2px).
//! - `3184393_picture_inner_margin.hwp` — 같은 그림을 담은 HWP5 저장본.
//!
//! 세 값은 `sz.height = curSz.height + inMargin.top`(4251 = 1984 + 2267)으로 맞물린다.
//! 코퍼스 HWPX 12,256개 그림 중 안쪽 여백이 있는 6개가 전부 이 관계를 지켰다.
//!
//! 독립 기준 — 한/글 2020 PDF(같은 폴더 `*-2020.pdf`)를 96dpi 로 래스터화해 버튼의 파란
//! 화소 범위를 실측했다.
//! - hwpx: x 411..481, y 105..131 — 틀 윗변 75.6 + 30.2 = 105.8 에서 시작.
//! - hwp:  x 562..632, y 108..133 — 틀 윗변 78.3 + 30.2 = 108.5 에서 시작.
//!
//! 수정 전 rhwp 는 틀 전체(72.0×56.7px)에 그림을 늘여 그려 세로가 2.1배였다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

const HWPX: &str = "samples/issue7193/2983289_picture_inner_margin.hwpx";
const HWP: &str = "samples/issue7193/3184393_picture_inner_margin.hwp";
/// 안쪽 여백이 모두 0인 `tac` 그림 하나 — 규칙이 적용되지 않아야 할 대조군.
const NO_MARGIN: &str = "samples/issue3587/d-pi2-import.hwpx";

/// HWPUNIT → 96dpi px.
fn px(hu: f64) -> f64 {
    hu * 96.0 / 7200.0
}

fn load(rel: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {rel}: {e:?}"))
}

#[derive(Debug, Clone, Copy)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

/// SVG `<image>` 요소의 실제 그림 사각형.
fn svg_images(svg: &str) -> Vec<Rect> {
    let mut out = Vec::new();
    for chunk in svg.split("<image").skip(1) {
        let head = &chunk[..chunk.find('>').unwrap_or(chunk.len())];
        let attr = |k: &str| -> Option<f64> {
            let pat = format!(" {k}=\"");
            let i = head.find(&pat)? + pat.len();
            let rest = &head[i..];
            rest[..rest.find('"')?].parse().ok()
        };
        if let (Some(x), Some(y), Some(w), Some(h)) =
            (attr("x"), attr("y"), attr("width"), attr("height"))
        {
            out.push(Rect { x, y, w, h });
        }
    }
    out
}

/// 렌더 트리 `Image` 노드의 bbox(개체 틀).
fn tree_images(node: &serde_json::Value, out: &mut Vec<Rect>) {
    if node.get("type").and_then(|t| t.as_str()) == Some("Image") {
        let b = &node["bbox"];
        let f = |k: &str| b.get(k).and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
        out.push(Rect {
            x: f("x"),
            y: f("y"),
            w: f("w"),
            h: f("h"),
        });
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        tree_images(child, out);
    }
}

fn frame_and_painted(rel: &str) -> (Rect, Rect) {
    let doc = load(rel);
    let tree: serde_json::Value =
        serde_json::from_str(&doc.get_page_render_tree(0).expect("render tree"))
            .expect("render tree json");
    let mut frames = Vec::new();
    tree_images(&tree, &mut frames);
    let painted = svg_images(&doc.render_page_svg(0).expect("svg"));
    assert_eq!(
        frames.len(),
        1,
        "{rel}: Image 노드는 1개여야 한다, got {frames:?}"
    );
    assert_eq!(
        painted.len(),
        1,
        "{rel}: <image> 는 1개여야 한다, got {painted:?}"
    );
    (frames[0], painted[0])
}

fn assert_close(label: &str, got: f64, want: f64) {
    assert!(
        (got - want).abs() < 0.05,
        "{label}: got {got:.3}, want {want:.3}"
    );
}

fn assert_inset_by_top_margin(rel: &str) {
    let (frame, painted) = frame_and_painted(rel);

    // 틀은 선언 크기 그대로 — 흐름·선택 기준은 바뀌지 않는다.
    assert_close("frame w", frame.w, px(5400.0));
    assert_close("frame h", frame.h, px(4251.0));

    // 그림은 틀에서 안쪽 여백(위 2267)을 뺀 자리에 그려진다.
    assert_close("painted x", painted.x, frame.x);
    assert_close("painted y", painted.y, frame.y + px(2267.0));
    assert_close("painted w", painted.w, px(5400.0));
    assert_close("painted h", painted.h, px(1984.0));
}

#[test]
fn issue_7193_hwpx_picture_is_painted_inside_inner_margin() {
    assert_inset_by_top_margin(HWPX);
}

#[test]
fn issue_7193_hwp_picture_is_painted_inside_inner_margin() {
    assert_inset_by_top_margin(HWP);
}

#[test]
fn issue_7193_picture_without_inner_margin_fills_its_frame() {
    let (frame, painted) = frame_and_painted(NO_MARGIN);
    assert!(
        frame.w > 1.0 && frame.h > 1.0,
        "대조군 틀이 비었다: {frame:?}"
    );
    assert_close("painted x", painted.x, frame.x);
    assert_close("painted y", painted.y, frame.y);
    assert_close("painted w", painted.w, frame.w);
    assert_close("painted h", painted.h, frame.h);
}

/// 레이어 트리 JSON 의 `image` op bbox 들(CanvasKit·Studio 소비 경로).
fn layer_image_ops(node: &serde_json::Value, out: &mut Vec<Rect>) {
    match node {
        serde_json::Value::Object(map) => {
            if map.get("type").and_then(|t| t.as_str()) == Some("image") {
                let b = &map["bbox"];
                let f = |k: &str| b.get(k).and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
                out.push(Rect {
                    x: f("x"),
                    y: f("y"),
                    w: f("width"),
                    h: f("height"),
                });
            }
            map.values().for_each(|v| layer_image_ops(v, out));
        }
        serde_json::Value::Array(items) => items.iter().for_each(|v| layer_image_ops(v, out)),
        _ => {}
    }
}

#[test]
fn issue_7193_layer_tree_image_op_is_painted_inside_inner_margin() {
    let doc = load(HWPX);
    let (frame, _) = frame_and_painted(HWPX);
    let layer: serde_json::Value =
        serde_json::from_str(&doc.get_page_layer_tree(0).expect("layer tree"))
            .expect("layer tree json");
    let mut ops = Vec::new();
    layer_image_ops(&layer, &mut ops);
    assert_eq!(ops.len(), 1, "image op 는 1개여야 한다, got {ops:?}");
    assert_close("op x", ops[0].x, frame.x);
    assert_close("op y", ops[0].y, frame.y + px(2267.0));
    assert_close("op w", ops[0].w, px(5400.0));
    assert_close("op h", ops[0].h, px(1984.0));
}
