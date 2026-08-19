//! Issue #5593: 세로 가운데 정렬 칸이 **글자 줄 높이만으로** 중앙을 잡아, 줄보다 큰
//! 비-flow 개체(바코드·도장)가 줄 위치에 그려지며 칸 밖으로 밀리던 회귀의 가드.
//!
//! 셀 세로 정렬 기준 콘텐츠 높이는 `cell_wrap_object_visual_bottom` 이 모으는데,
//! 종전 필터는 `Square|Tight|Through` 만 셌다. 글 앞으로(`InFrontOfText`)·글 뒤로
//! (`BehindText`) 개체는 줄 흐름을 밀지 않아 저장 LINE_SEG 에도 흡수되지 않으므로,
//! 이 필터에서 빠지면 어디에도 계상되지 않는다.
//!
//! 재현 문서 (tracked 합성 샘플): `samples/issue5593_cell_center_front_object.hwpx`
//! - 1×1 표, 칸 `vertAlign="CENTER"`, 칸 높이 6400HU(85.3px).
//! - 칸 문단: 글자 줄(13.3px) + `IN_FRONT_OF_TEXT` 사각형 5830HU(77.7px).
//! - 수정 전: 개체 y=134.2 bottom=211.9 → 칸 bottom 183.5 를 28.4px 넘김.
//! - 수정 후: 개체 y=100.1 bottom=177.8 → 칸 안. 같은 기하의 `SQUARE` 개체와 일치.
//!
//! 보고 문서 00425(칸 85.0px · 줄 16.0px · 그림 77.5px → 칸 밖 27px)와 같은 산술이다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5593_cell_center_front_object.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

/// (y, height) of the first node of the given render-tree type.
fn first_box(node: &serde_json::Value, ty: &str) -> Option<(f64, f64)> {
    if node.get("type").and_then(|t| t.as_str()) == Some(ty) {
        let bbox = node.get("bbox")?;
        return Some((bbox.get("y")?.as_f64()?, bbox.get("h")?.as_f64()?));
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        if let Some(found) = first_box(child, ty) {
            return Some(found);
        }
    }
    None
}

#[test]
fn issue_5593_center_cell_counts_front_object_height() {
    let doc = load_doc();
    let json = doc.get_page_render_tree(0).expect("render tree page 0");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let (cell_y, cell_h) = first_box(&tree, "Cell").expect("Cell 노드");
    let (obj_y, obj_h) = first_box(&tree, "Rect").expect("개체(Rect) 노드");

    let cell_bottom = cell_y + cell_h;
    let obj_bottom = obj_y + obj_h;
    assert!(
        obj_bottom <= cell_bottom + 0.5,
        "글 앞으로 개체가 칸 밖으로 밀렸다 — #5593 회귀. \
         칸 {cell_y:.1}..{cell_bottom:.1}, 개체 {obj_y:.1}..{obj_bottom:.1}"
    );

    // 개체 높이가 정렬 기준에 들어가면 개체가 칸 안에서 중앙에 온다 —
    // 줄 높이만 셌을 때의 위치(칸 중앙 + 개체가 아래로 삐져나감)와 구분한다.
    let expected_y = cell_y + (cell_h - obj_h) / 2.0;
    assert!(
        (obj_y - expected_y).abs() < 3.0,
        "개체가 칸 세로 중앙에 오지 않았다 (기대 ≈{expected_y:.1}, 실제 {obj_y:.1})"
    );
}
