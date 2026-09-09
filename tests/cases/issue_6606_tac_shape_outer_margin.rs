//! Issue #6606: 글자처럼(TAC) 도형·묶음의 바깥 여백이 줄 안 위치에 반영되지 않아
//! 왼쪽 여백만큼 어긋나던 결함의 가드 (#6603 TAC 그림의 도형 형제).
//!
//! `samples/draw-group.hwp` 1쪽의 묶음은 `글자처럼=true`, 바깥 여백 left/right 3.20mm
//! (907HU = 12.09px). paragraph_layout 의 TAC 도형 분기가 `set_inline_shape_position`
//! 에 상자 원점을 그대로 등록해, 묶음의 자식 그림 10장이 전부 한컴 PDF 보다
//! 12.05px 왼쪽에 그려졌다 (`pdf/draw-group-2022.pdf` 이미지 bbox 대조, 96DPI px):
//!
//! ```text
//! 첫 자식(폭 61.4)     (255.2, 132.3) → (267.22, 132.17)
//! 가장 왼쪽 자식       x 113.8         → 125.85
//! ```

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/draw-group.hwp";
const TOLERANCE_PX: f64 = 1.5;
/// 한컴 PDF 실측: 묶음 자식 중 폭 61.4px 그림의 좌상단. 결함 시 (255.2, 132.3).
const EXPECTED_FIRST: (f64, f64) = (267.22, 132.17);
const FIRST_WIDTH: f64 = 61.4;
/// 한컴 PDF 실측: 묶음 자식(선 포함) 중 가장 왼쪽 x. 결함 시 113.4.
const EXPECTED_MIN_X: f64 = 125.5;
/// 묶음 `children=17` — render tree 는 자식 도형도 Image 로 낸다.
const GROUP_CHILDREN: usize = 17;

fn collect_group_images(node: &serde_json::Value, in_group: bool, out: &mut Vec<(f64, f64, f64)>) {
    let ty = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    if ty == "Image" && in_group {
        let bbox = &node["bbox"];
        let get = |k: &str| bbox.get(k).and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
        out.push((get("x"), get("y"), get("w")));
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        collect_group_images(child, in_group || ty == "Group", out);
    }
}

#[test]
fn tac_group_children_sit_inside_the_group_outer_margin_box() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));
    let json = document
        .get_page_render_tree(0)
        .expect("render tree page 1");
    let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

    let mut images = Vec::new();
    collect_group_images(&tree, false, &mut images);
    assert_eq!(
        images.len(),
        GROUP_CHILDREN,
        "1쪽 묶음 안에 자식 {GROUP_CHILDREN}개가 있어야 한다: {images:?}"
    );

    let (fx, fy, _) = images
        .iter()
        .copied()
        .find(|(_, _, w)| (w - FIRST_WIDTH).abs() < 1.0)
        .expect("폭 61.4px 자식 그림");
    assert!(
        (fx - EXPECTED_FIRST.0).abs() < TOLERANCE_PX
            && (fy - EXPECTED_FIRST.1).abs() < TOLERANCE_PX,
        "묶음 첫 자식 그림 ({fx:.2}, {fy:.2}) — 한컴 PDF 실측 {EXPECTED_FIRST:?} 이어야 한다. \
         결함 시 (255.2, 132.3): 묶음의 왼쪽 바깥 여백 12.09px 이 빠진 값"
    );

    let min_x = images
        .iter()
        .map(|(x, _, _)| *x)
        .fold(f64::INFINITY, f64::min);
    assert!(
        (min_x - EXPECTED_MIN_X).abs() < TOLERANCE_PX,
        "묶음 자식 중 가장 왼쪽 x={min_x:.2} — 한컴 PDF 실측 {EXPECTED_MIN_X} 이어야 한다 (결함 시 113.4)"
    );
}
