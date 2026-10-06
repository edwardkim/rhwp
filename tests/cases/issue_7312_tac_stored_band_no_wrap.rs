//! [#7312] 저장 밴드가 `om_top + 선언높이 + om_bottom` 로 **표 하나를** 담고 있으면
//! 중간앵커 줄바꿈을 적용하지 않는다.
//!
//! ## 종전 결함
//!
//! `treat_as_char=true` 표를 배치하기 직전 `should_wrap_middle_anchored_table` 이
//! "남은 줄 폭에 표가 안 들어간다" 고 판정하면 `current_y += line_step` 으로 표를 **다음
//! 줄**에 내려놓는다. 그런데 바로 다음 줄에서 부르는 `tac_table_stored_outer_band_top`
//! (`#5729`)은 "저장 밴드가 `om_top + 선언높이 + om_bottom` 이면 한/글은 표 상단을
//! **줄 상단 + om_top** 에 앉힌다" 는 계약이고, 내려간 `current_y` 를 그대로 받는다.
//! 곧 **저장 사다리가 "이 표가 이 줄을 통째로 차지한다" 고 증명하는데 폭 판정이 그것을
//! 덮는** 구조였다.
//!
//! ## 이 문서 (`samples/issue7312/`, 1쪽)
//!
//! ```text
//! 문단 0.2  ls[0] vpos=4863(64.8px) lh=6896  ← 6896 == om_top 283 + 선언 6330 + om_bottom 283
//!   결재란 표 3행×5열  treat_as_char=true  outer_margin 283 사방
//!   줄바꿈 판정 입력: occupied 294.00 + footprint 362.09 > line_w 642.53  → 발동
//! ```
//!
//! | | 수정 전 | 수정 후 | 한/글 2022 정본 |
//! | --- | ---: | ---: | ---: |
//! | 결재란 표 상단 | 237.5 | **144.2** | 145.7 |
//! | 결재란 표 좌단 | 79.4 | **373.4** | 373.9 |
//! | `pi=11` 마지막 줄 | 1050.3 | **868.9** | 869.6 |
//!
//! 정본 PDF 는 저장소 밖 캐시(한/글 2022)라 이 시험은 **저장 사다리와의 정합**을 계약으로
//! 삼는다 — 사다리 값 `vpos/75 + 본문 상단` 이 정본과 ±1.7px 로 맞는 것은 이슈 #7312 에 있다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue7312/36494702_approval_tac_band.hwpx";

/// 결재란 표의 저장 바깥 여백 (HU) → px.
const OM_TOP_PX: f64 = 283.0 / 75.0;
/// host 줄(`문단 0.2`)의 저장 vpos (HU) → px.
const HOST_LINE_VPOS_PX: f64 = 4863.0 / 75.0;
/// 마지막 문단(`pi=11`)의 저장 vpos (HU) → px.
const LAST_LINE_VPOS_PX: f64 = 59497.0 / 75.0;

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

fn page_nodes(core: &DocumentCore) -> Vec<RenderNode> {
    let page = core.build_page_render_tree(0).expect("render tree");
    let mut refs = Vec::new();
    walk(&page.root, &mut refs);
    refs.into_iter().cloned().collect()
}

/// 결재란(3×5) 표 상자.
fn approval_table(nodes: &[RenderNode]) -> BoundingBox {
    nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::Table(t) if t.row_count == 3 && t.col_count == 5 => Some(n.bbox),
            _ => None,
        })
        .next()
        .expect("결재란 3×5 표")
}

#[test]
fn stored_outer_band_keeps_the_tac_table_on_its_host_line() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_eq!(core.page_count(), 1, "#7312: 이 fixture 는 1쪽이다");

    let nodes = page_nodes(&core);
    let body = nodes
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .expect("본문 상자")
        .bbox;
    let table = approval_table(&nodes);

    // host 줄 상단 = 본문 상단 + 저장 vpos. 표 상단은 거기에 바깥 위 여백만 더한 자리다.
    let host_line_top = body.y + HOST_LINE_VPOS_PX;
    let expected = host_line_top + OM_TOP_PX;
    assert!(
        (table.y - expected).abs() < 0.5,
        "#7312: 저장 밴드가 om_top+선언높이+om_bottom 이면 표 상단은 host 줄 상단 + om_top \
         ({expected:.2}) 이어야 한다 — 수정 전에는 줄 하나(93.28px)를 더 내려가 237.5 였다: {:.2}",
        table.y
    );

    // 줄바꿈을 타면 표가 줄 시작(x≈79.4)으로 되돌아간다. 타지 않으면 앞 글자 뒤에 남는다.
    assert!(
        table.x > body.x + 200.0,
        "#7312: 줄바꿈을 타지 않으면 표 좌단은 앞 글자 뒤(본문 상자보다 200px 이상 오른쪽)여야 \
         한다 — 수정 전 79.4: {:.2}",
        table.x
    );

    // 그 문단 뒤 본문 전체가 저장 사다리에 붙는다.
    let last_line = nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) if run.text.contains("특이사항 없음") => {
                Some(n.bbox.y)
            }
            _ => None,
        })
        .next()
        .expect("마지막 문단");
    let expected_last = body.y + LAST_LINE_VPOS_PX;
    assert!(
        (last_line - expected_last).abs() < 1.0,
        "#7312: 뒤 본문은 저장 사다리 자리({expected_last:.2})에 있어야 한다 \
         — 수정 전 1050.3(사다리보다 181.4px 아래): {last_line:.2}"
    );
}
