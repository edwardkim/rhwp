//! [#6866] `angle=270` 회전 그림의 렌더 노드 상자를 rhwp 가 다시 90° 돌려 눕힌다.
//!
//! `hp:sz` 는 **회전을 반영한 최종 상자**다. 한컴은 그 상자를 그대로 쓰고 비트맵만 안에서
//! 돌린다. rhwp 는 상자까지 돌려 세로 29.3 × 53.7 을 가로 53.7 × 29.3 으로 눕혔다.
//!
//! ```text
//!   156627451 12쪽  12번째 그림 (angle=270, 문서 안 유일)
//!     선언 hp:sz     29.3 × 53.7
//!     수정 전        53.7 × 29.3   ← 눕혀졌다
//!     수정 후        29.3 × 53.7
//!     정본(2020)     Im42 배치 행렬 a=21.95 b=0 c=0 d=40.28  → 회전 행렬 없이 29.3 × 53.7
//! ```
//!
//! ⭐ 근인은 **전제의 어긋남**이다. 페인터(`ShapeTransform::effective_image_bbox`)는
//! 90/270° 회전 그림의 bbox 가로세로를 **먼저 뒤집은 뒤** 회전을 건다 — 노드 상자가
//! **회전 후** 상자라는 전제다. 그런데 `layout_picture` 는 회전 프레임 그림에서
//! `current_*`(회전 **전** 비트맵)를 노드에 실어 그 전제를 깼다.
//!
//! ⭐ 같은 문서의 `angle=0` 그림 15장이 **문서 안 통제군**이다 — 전부 선언 상자와 맞고,
//! 이 수정으로 움직이지 않아야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6866/156627451-quantum-science-press-note.hwpx";
/// 회전 그림이 있는 쪽(0 기준).
const ROTATED_PAGE: u32 = 11;

/// 선언 `hp:sz` — 2196 × 4030 HWPUNIT.
const DECLARED_W_PX: f64 = 29.3;
const DECLARED_H_PX: f64 = 53.7;

fn open_sample() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드")
}

/// 이 쪽의 모든 이미지 노드를 `(width, height, rotation)` 으로 모은다.
fn images(node: &RenderNode, out: &mut Vec<(f64, f64, f64)>) {
    if let RenderNodeType::Image(image) = &node.node_type {
        out.push((
            node.bbox.width,
            node.bbox.height,
            image.transform.rotation.rem_euclid(360.0),
        ));
    }
    for child in &node.children {
        images(child, out);
    }
}

fn page_images(page: u32) -> Vec<(f64, f64, f64)> {
    let core = open_sample();
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("{}쪽 render tree: {e}", page + 1));
    let mut out = Vec::new();
    images(&tree.root, &mut out);
    out
}

#[test]
fn issue_6866_rotated_picture_keeps_its_declared_box() {
    let rotated: Vec<(f64, f64, f64)> = page_images(ROTATED_PAGE)
        .into_iter()
        .filter(|(_, _, rotation)| *rotation > 1.0)
        .collect();
    assert_eq!(
        rotated.len(),
        1,
        "이 쪽에는 회전 그림이 정확히 하나 있어야 한다 (실측 {rotated:?})"
    );
    let (width, height, rotation) = rotated[0];
    assert!(
        (rotation - 270.0).abs() < 1.0,
        "대상은 angle=270 그림이다 (실측 {rotation})"
    );
    assert!(
        (width - DECLARED_W_PX).abs() <= 0.5 && (height - DECLARED_H_PX).abs() <= 0.5,
        "회전 그림의 상자는 선언 `hp:sz`({DECLARED_W_PX} x {DECLARED_H_PX}) 그대로여야 \
         한다 — 결함 시 가로세로가 뒤바뀌어 {width:.1} x {height:.1} 이었다"
    );
}

#[test]
fn issue_6866_unrotated_pictures_are_untouched() {
    // 같은 문서의 `angle=0` 그림들이 통제군이다. 이 쪽의 나머지 두 장은 선언 상자
    // 247.0 x 165.3 · 285.0 x 168.0 이고 이 수정과 무관하다.
    let upright: Vec<(f64, f64)> = page_images(ROTATED_PAGE)
        .into_iter()
        .filter(|(_, _, rotation)| *rotation <= 1.0)
        .map(|(w, h, _)| ((w * 10.0).round() / 10.0, (h * 10.0).round() / 10.0))
        .collect();
    assert!(
        upright.contains(&(247.0, 165.3)),
        "회전 없는 그림은 그대로여야 한다 (실측 {upright:?})"
    );
    assert!(
        upright.contains(&(285.0, 168.0)),
        "회전 없는 그림은 그대로여야 한다 (실측 {upright:?})"
    );
}
