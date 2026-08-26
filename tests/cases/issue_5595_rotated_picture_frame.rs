//! Issue #5595: 90° 회전된 그림을 긴 변 기준 정사각형으로 조판해 지면 밖으로 내보내던
//! 회귀의 가드.
//!
//! 회전 그림에서 `CommonObjAttr.width/height`(HWPX `hp:sz`)는 한컴이 저장한 **회전 후
//! 외접 프레임**이고 `SHAPE_COMPONENT.current_width/height`(`hp:curSz`)는 **회전 전
//! 원본 표시 크기**다. 표시 크기 helper 가 두 값을 축별 max 로 합치면 90°/270° 처럼 두
//! 축이 뒤바뀐 그림에서 긴 변이 가로·세로 양쪽에 들어가 정사각형이 된다.
//!
//! 재현 문서 (tracked 합성 샘플): `samples/issue5595_rotated_picture_topbottom.hwpx`
//! - 그림 1개, `angle=90`, `sz`=53420×37986(188.5×134.0mm), `curSz`=37986×53420,
//!   `treatAsChar=0` + `TOP_AND_BOTTOM` (원 보고 문서 00493 과 같은 축).
//! - 수정 전: Image bbox 712.3×712.3px (정사각형, 긴 변을 두 축에 사용).
//! - 수정 후: Image bbox 712.3×506.5px = 선언 188.5×134.0mm.
//!
//! 글자처럼(TAC)·SQUARE 어울림 경로는 회전 프레임을 이미 바르게 썼다. 이 테스트가
//! 지키는 것은 그 두 경로와 TopAndBottom float 경로의 **일치**다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5595_rotated_picture_topbottom.hwpx";

/// 선언 크기 53420×37986 HWPUNIT → 96dpi 픽셀.
const EXPECTED_W: f64 = 712.3;
const EXPECTED_H: f64 = 506.5;

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

fn collect_images(node: &serde_json::Value, out: &mut Vec<(f64, f64)>) {
    if node.get("type").and_then(|t| t.as_str()) == Some("Image") {
        if let Some(bbox) = node.get("bbox") {
            let w = bbox.get("w").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let h = bbox.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0);
            out.push((w, h));
        }
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        collect_images(child, out);
    }
}

#[test]
fn issue_5595_rotated_picture_keeps_declared_frame() {
    let doc = load_doc();
    let json = doc.get_page_render_tree(0).expect("render tree page 0");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let mut images = Vec::new();
    collect_images(&tree, &mut images);
    let (w, h) = *images
        .first()
        .expect("회전 그림 Image 노드가 렌더 트리에 있어야 한다");

    assert!(
        (w - EXPECTED_W).abs() < 1.0 && (h - EXPECTED_H).abs() < 1.0,
        "회전 그림은 선언 프레임 {EXPECTED_W}×{EXPECTED_H}px 로 조판되어야 한다 \
         (긴 변 기준 정사각형 금지), got {w}×{h}"
    );
    assert!(
        (w - h).abs() > 100.0,
        "회전 그림이 정사각형으로 조판됐다 — #5595 회귀, got {w}×{h}"
    );
}
