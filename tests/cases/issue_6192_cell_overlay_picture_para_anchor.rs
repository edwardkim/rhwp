//! [Issue #6192] 표 칸 안 **글 뒤로 배치**(BehindText) 말풍선이 호스트 문단이 아니라
//! 칸 콘텐츠 상단을 기준으로 놓여, 감싸야 할 문장 위로 28~79px 올라간다.
//!
//! 근인은 `#2226` 의 `displaced_empty_line_para` 분기다. 그 규칙은 "그림이 글줄을
//! 밀어 그 줄의 `vpos` 가 그림 자신의 변위인" 형상을 겨냥한 것인데(→ 앵커를 칸
//! 콘텐츠 상단으로 되돌린다), **overlay 그림은 흐름을 밀지 않으므로** 빈 호스트
//! 문단의 `vpos` 는 진짜 흐름 위치다.
//!
//! 이 문서 STEP1 칸 실측:
//!
//! ```text
//! 칸 콘텐츠 상단  content_cell_y + pad_top = 254.79   ← 수정 전 기준점
//! 호스트 문단 top para_y_before_compose    = 285.30   ← 옳은 기준점
//! vOffset 478HU = 6.37px
//!   수정 전: 254.79 + 6.37 = 261.16
//!   수정 후: 285.30 + 6.37 = 291.67     한글 291.36 (Δ 0.31)
//! ```
//!
//! 한글 2020 오라클(COM PDF, `Hancom PDF`) 대조 — **말풍선 14개 전부 1.2px 이내**:
//!
//! | 쪽 | 개수 | 수정 전 Δ | 수정 후 Δ |
//! |---|---|---|---|
//! | 물리 4쪽 | 6 | −28.4 ~ −30.8 | +0.34 ~ +1.10 |
//! | 물리 6쪽 | 8 | −18.7 ~ −79.1 | +0.33 ~ +1.16 |
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6192/cell_behind_text_para_anchor.hwpx";
/// 한글 2020 실측 말풍선 y (물리 4쪽 6개).
const ORACLE_Y: [f64; 6] = [291.36, 417.62, 537.97, 664.87, 794.17, 915.80];
/// 허용 오차 — 오라클 대비 최대 편차 1.16px 에 여유를 둔다.
const TOL_PX: f64 = 2.5;

#[test]
fn issue_6192_cell_overlay_picture_anchors_to_host_paragraph() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page = core.build_page_render_tree(0).expect("page 1 render tree");

    let mut balloons = Vec::new();
    collect_balloons(&page.root, &mut balloons);
    balloons.sort_by(|a, b| a.partial_cmp(b).expect("finite"));

    assert_eq!(
        balloons.len(),
        ORACLE_Y.len(),
        "말풍선 {}개가 있어야 한다 — 실측 {balloons:?}",
        ORACLE_Y.len()
    );

    let bad: Vec<String> = balloons
        .iter()
        .zip(ORACLE_Y)
        .filter(|(got, want)| (**got - *want).abs() > TOL_PX)
        .map(|(got, want)| format!("한글 {want:.2} ↔ rhwp {got:.2} (Δ{:+.2})", got - want))
        .collect();
    assert!(
        bad.is_empty(),
        "글 뒤로 배치 말풍선은 호스트 문단 top + vOffset 에 놓여야 한다 \
         (칸 콘텐츠 상단 기준이면 28~31px 위로 올라간다) — 어긋난 것: {bad:?}"
    );
}

/// 말풍선 그림(폭 300px 초과)의 위끝.
fn collect_balloons(node: &RenderNode, out: &mut Vec<f64>) {
    if matches!(node.node_type, RenderNodeType::Image(_)) && node.bbox.width > 300.0 {
        out.push(node.bbox.y);
    }
    for child in &node.children {
        collect_balloons(child, out);
    }
}
