//! [Issue #6173] 오른쪽 정렬 글상자에서 **로고 사이 공백**까지 말미 공백으로 걷어내
//! 로고 두 장이 27px 우측으로 밀리고, 뒤 로고가 글상자 우단을 넘어 잘린다.
//!
//! 문단은 `[로고A][공백4][로고B][공백2]` 오른쪽 정렬이다. 인라인(자리차지) 개체는
//! run 을 쪼개지 않고 run 안 char 위치에 놓이므로 composer 는 이것을 **공백 6칸짜리
//! run 하나**로 합성한다. 오른쪽 정렬이 폭에서 제외할 말미 공백을 run 뒤에서부터만
//! 세면 로고 사이 4칸(26.7px)까지 말미로 걷혀 앵커가 그만큼 우측으로 밀린다.
//!
//! | | rhwp(수정 전) | rhwp(수정 후) | 한글 2020 |
//! |---|---|---|---|
//! | 로고A 좌변 | 507.9 | 480.9 | 480.87 |
//! | 로고B 좌변 | 635.7 | 608.7 | 608.31 |
//! | 로고B 우변 | 742.1 (글상자 715.1 밖) | 715.1 | 714.65 |
//!
//! 같은 계약이 `paragraph_layout`(본문 흐름)과 `shape_layout`(글상자 개체) 두 곳에
//! 복제돼 있었고 **양쪽 다 같은 구멍**이었다 — 그래서 서로 어긋나지 않아 조용했다.
//! 이 테스트는 두 경로가 함께 옳은 것을 잠근다: 글자 run 은 로고 사이·뒤에 놓이고,
//! 로고는 글상자 안에 온전히 들어간다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6173/textbox_right_align_logos.hwpx";
/// 글상자 안쪽 우단(px). 로고 우변이 이보다 크면 clip 에 잘린다.
const BOX_INNER_RIGHT_PX: f64 = 715.1;
/// 한글 2020 실측 로고A 좌변.
const ORACLE_FIRST_LOGO_X: f64 = 480.87;

#[test]
fn issue_6173_space_before_inline_object_is_not_trailing() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1 render tree");

    let mut logos = Vec::new();
    collect_images(&page.root, &mut logos);
    logos.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("finite"));
    assert_eq!(
        logos.len(),
        2,
        "글상자에 로고 두 장이 있어야 한다 — 실측 {logos:?}"
    );

    let (first_x, _) = logos[0];
    let (last_x, last_w) = logos[1];
    let last_right = last_x + last_w;

    assert!(
        last_right <= BOX_INNER_RIGHT_PX + 1.0,
        "뒤 로고가 글상자 안쪽 우단({BOX_INNER_RIGHT_PX})을 넘어 잘린다 — \
         x={last_x:.1} w={last_w:.1} 우변={last_right:.1}"
    );
    assert!(
        (first_x - ORACLE_FIRST_LOGO_X).abs() <= 2.0,
        "로고 사이 공백은 말미 공백이 아니다 — 앞 로고 좌변 실측 {first_x:.1}, \
         한글 {ORACLE_FIRST_LOGO_X}. 사이 공백 4칸(26.7px)까지 걷어내면 507.9 가 된다."
    );
}

/// 쪽 안 모든 이미지의 `(x, width)`.
fn collect_images(node: &RenderNode, out: &mut Vec<(f64, f64)>) {
    if let RenderNodeType::Image(_) = &node.node_type {
        out.push((node.bbox.x, node.bbox.width));
    }
    for child in &node.children {
        collect_images(child, out);
    }
}
