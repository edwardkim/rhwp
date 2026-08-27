//! [Issue #6143] 자리차지 표가 다음 쪽으로 넘어갈 때 문단 기준 세로 오프셋을
//! 그대로 다시 더해 쪽 상단이 비고 표가 한 쪽 더 갈라진다 (156555538 9쪽).
//!
//! 근인: 문단 기준 오프셋의 기준점은 **앵커 문단이 놓인 자리**다. 이 문서의
//! 앵커 pi=16 은 저장 사다리상 8쪽에 있고(vpos=32514), 표 상단은 거기서
//! +41592HU 인 988.1px — 8쪽 바닥이라 한 행도 못 들어간다. 한글은 표를 9쪽으로
//! 넘기고 **오프셋은 이미 소진된 것으로 보아 쪽 상단에 붙여** 그린다.
//!
//! rhwp 는 앵커 문단을 통째로 9쪽으로 옮긴 뒤 오프셋 554.6px 를 **새 쪽 상단
//! 기준으로 다시** 얹었다. 그 결과 9쪽 위 554.6px 가 비고, 조각 예산도 그만큼
//! 짧아져(page_avail 990.3 − 1.9 − 554.6 = 433.8) 행 1 을 20줄째에서 자르고
//! 나머지를 10쪽으로 넘겨 전체가 한글 17쪽 대비 18쪽이 됐다.
//!
//! 수정은 조판(typeset 예산)과 배치(layout) **양쪽 대칭**이다 — 한쪽만 고치면
//! 표는 위로 올라가되 컷은 그대로라 쪽 하단이 빈 채 남는다(#2015 감액이 두 축을
//! 짝으로 유지하는 것과 같은 이유).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6143/156555538_securities_settlement_review.hwpx";
/// 결함이 나타나는 쪽(0-based) — 한글 쪽번호 `- 8 -`.
const PAGE: u32 = 8;
/// 결함 상태에서 표가 시작하던 자리. 본문 상단 75.6 + 오프셋 554.6.
const DEFECT_TABLE_TOP_PX: f64 = 630.1;

#[test]
fn issue_6143_deferred_float_table_starts_at_page_top() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    // 한글 2020 오라클 17쪽. 결함 상태는 18쪽이었다 — 오프셋을 두 번 세면서
    // 조각 예산이 짧아져 표가 한 번 더 갈라진 몫이다.
    assert_eq!(core.page_count(), 17, "한글 2020 오라클과 같은 17쪽이어야 한다");

    let page = core.build_page_render_tree(PAGE).expect("9쪽 render tree");
    let mut table_tops: Vec<f64> = Vec::new();
    collect_table_tops(&page.root, &mut table_tops);
    let first_top = table_tops.iter().copied().fold(f64::INFINITY, f64::min);
    assert!(
        first_top.is_finite(),
        "9쪽에 자리차지 표가 그려져야 한다 (표 노드 없음)"
    );

    // 넘어온 조각은 쪽 본문 상단(이 구역 75.6px)에서 시작한다. 결함이면 거기서
    // 오프셋 554.6px 아래인 630.1px 이라, 두 상태는 한참 떨어져 구별된다.
    // 상한은 그 중간보다 훨씬 낮게 잡아 조판 미세 변동에는 걸리지 않게 한다.
    const BODY_TOP_PX: f64 = 75.6;
    const TOLERANCE_PX: f64 = 100.0;
    assert!(
        first_top <= BODY_TOP_PX + TOLERANCE_PX,
        "넘어온 자리차지 표는 쪽 본문 상단({BODY_TOP_PX:.1})에서 시작해야 한다 \
         (표 상단 {first_top:.1}, 결함 시 {DEFECT_TABLE_TOP_PX:.1})"
    );
}

fn collect_table_tops(node: &RenderNode, out: &mut Vec<f64>) {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push(node.bbox.y);
    }
    for child in &node.children {
        collect_table_tops(child, out);
    }
}
