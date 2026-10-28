//! [Issue #5820 축2] 글상자 안 로고 줄의 세로 배치 2결함 — 156560092 2쪽,
//! 한글 2022 오라클 실측(r12): 로고 A 상자-상대 top +12.8 vs rhwp +24.9.
//!
//! 1. 글상자 세로 CENTER 정렬의 콘텐츠 높이 계산이 **TAC 인라인 개체를
//!    제외**해, lineseg 없는 로고 문단이 높이 0 으로 계산됨 → 오프셋이
//!    반칸(+21.2px) 과대해지고 로고가 글상자 아래로 흘러넘쳤다.
//! 2. 같은 줄의 인라인 그림 형제를 **상단 정렬** — 한글은 하단(베이스라인)
//!    정렬이라 키가 작은 로고 A 가 12.1px 낮게 그려졌다.
//!
//! 수정 후: A 상자-상대 top +14.1(한글 +12.8), B +3.8(한글 +2.4), B 하단이
//! 상자 안(446.1 ≤ 449.9). 결함 상태에서는 A top +24.9·B 하단 467(상자 밖)
//! 로 어서션이 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5820/156560092_ecard_meeting_press.hwpx";

#[test]
fn issue_5820_textbox_inline_logos_bottom_aligned_inside_box() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 2, "한글 2022 정본은 2쪽이다");

    let tree = core.build_page_render_tree(1).expect("page 2 render tree");
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("render tree JSON");

    let mut rect: Option<(f64, f64)> = None; // (y, h) — 글상자 프레임(w≈342.6)
    let mut logo_a: Option<(f64, f64)> = None; // w≈91.5
    let mut logo_b: Option<(f64, f64)> = None; // w≈119.3
    walk(&json, &mut rect, &mut logo_a, &mut logo_b);
    let (rect_y, rect_h) = rect.expect("글상자 Rect");
    let (a_y, _a_h) = logo_a.expect("로고 A");
    let (b_y, b_h) = logo_b.expect("로고 B");

    let a_rel = a_y - rect_y;
    assert!(
        (11.0..=17.0).contains(&a_rel),
        "로고 A 는 하단 정렬로 상자-상대 top ≈ +14 여야 한다 (한글 +12.8, 결함 시 +24.9): {a_rel:.1}"
    );
    assert!(
        b_y + b_h <= rect_y + rect_h + 0.5,
        "로고 B 하단은 글상자 안이어야 한다 (결함 시 +17px 넘침): bottom={:.1} box_bottom={:.1}",
        b_y + b_h,
        rect_y + rect_h
    );
}

#[allow(clippy::type_complexity)]
fn walk(
    node: &serde_json::Value,
    rect: &mut Option<(f64, f64)>,
    logo_a: &mut Option<(f64, f64)>,
    logo_b: &mut Option<(f64, f64)>,
) {
    if let Some(obj) = node.as_object() {
        let ty = obj.get("type").and_then(|v| v.as_str()).unwrap_or("");
        let bbox = obj.get("bbox").and_then(|b| {
            Some((
                b.get("y")?.as_f64()?,
                b.get("h")?.as_f64()?,
                b.get("w")?.as_f64()?,
            ))
        });
        if let Some((y, h, w)) = bbox {
            if ty == "Rect" && (340.0..=346.0).contains(&w) {
                *rect = Some((y, h));
            }
            if ty == "Image" && (89.0..=94.0).contains(&w) {
                *logo_a = Some((y, h));
            }
            if ty == "Image" && (117.0..=122.0).contains(&w) {
                *logo_b = Some((y, h));
            }
        }
        if let Some(children) = obj.get("children").and_then(|c| c.as_array()) {
            for child in children {
                walk(child, rect, logo_a, logo_b);
            }
        }
    }
}
