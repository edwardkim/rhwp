//! [Issue #6184] 남은 17.9px 에 16px 줄이 들어가는데도 다음 쪽으로 넘겨, 그 쪽에
//! 이미 놓인 흐름표 위에 겹쳐 그린다 (156489124 인쇄 10쪽).
//!
//! 문단은 **텍스트 줄 + 자리차지 RowBreak 표**를 함께 가진 visible-host float 이고,
//! 저장 사다리는 그 줄을 이 쪽 안에 둔다(`vpos 71604 + lh 1200 = 72804` ≤ 본문
//! 72847, 다음 문단 `vpos 6863` 으로 리셋). 한글도 그 줄을 1031.2..1047.2 로
//! 놓는다 — 본문 하단(1046.9)을 0.3px 넘겨서라도 이 쪽 마지막 줄로 삼는다.
//!
//! 근인 세 겹:
//!  1. `prefill_before_deferred_table` 이 **음수 세로 오프셋**에서 통째로 빠져나가
//!     host 줄 pre-emit 자체를 안 했다(이 표는 `vertOffset = -1700`).
//!  2. pre-emit fit 을 **말미 줄간격까지 포함**해 24.0px 로 재서 잔여 16.6px 를
//!     넘겨 탈락했다(바로 아래 후속 문단 루프는 이미 `height_for_fit` 을 쓴다).
//!  3. 중복 방지 가드가 조판·페인트 양쪽에서 **현재 쪽 항목만** 훑어, 쪽을 넘긴 뒤
//!     앞 쪽의 pre-emit 을 못 보고 같은 줄을 두 쪽에 그렸다.
//!
//! 재현물은 원본(1.5MB)의 문단 286..325 창을 잘라낸 IR 슬라이스(20KB)다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6184/156489124_tail_line_before_deferred_table.hwp";
/// 슬라이스에서 host 문단의 인덱스(원본 pi=324).
const HOST_PARA: usize = 38;
/// 이 줄에만 있는 글자.
const TAIL_TEXT: &str = "국민적 관심이 큰";

#[test]
fn issue_6184_tail_line_stays_on_the_page_and_is_not_duplicated() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let first = core.build_page_render_tree(0).expect("page 1");
    let second = core.build_page_render_tree(1).expect("page 2");

    let on_first = host_line_tops(&first.root);
    let on_second = host_line_tops(&second.root);

    assert_eq!(
        on_first.len(),
        1,
        "host 줄은 이 쪽 마지막 줄로 한 번 놓여야 한다 — 1쪽 {on_first:?}"
    );
    assert!(
        on_first[0] > 900.0,
        "host 줄은 쪽 말미에 와야 한다 — 실측 {:.1}",
        on_first[0]
    );
    assert!(
        on_second.is_empty(),
        "같은 줄이 다음 쪽에 또 그려지면 안 된다 — 2쪽 {on_second:?}"
    );

    // 다음 쪽은 흐름표부터 깨끗하게 시작한다.
    let table_top = first_table_top(&second.root).expect("다음 쪽 흐름표");
    assert!(
        table_top < 200.0,
        "다음 쪽은 표부터 시작해야 한다 — 표 위끝 {table_top:.1}"
    );
}

/// host 문단의 꼬리 줄 위끝들.
fn host_line_tops(node: &RenderNode) -> Vec<f64> {
    let mut out = Vec::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.para_index == Some(HOST_PARA) && run.text.contains(TAIL_TEXT) {
            out.push(node.bbox.y);
        }
    }
    for child in &node.children {
        out.extend(host_line_tops(child));
    }
    out
}

/// 쪽에서 가장 위에 놓인 표의 위끝.
fn first_table_top(node: &RenderNode) -> Option<f64> {
    let own = matches!(node.node_type, RenderNodeType::Table(_)).then_some(node.bbox.y);
    node.children
        .iter()
        .filter_map(first_table_top)
        .chain(own)
        .fold(None, |acc: Option<f64>, top| {
            Some(acc.map_or(top, |best: f64| best.min(top)))
        })
}
