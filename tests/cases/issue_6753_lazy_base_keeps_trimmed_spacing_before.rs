//! [Issue #6753] lazy vpos 기준 역산이 **트림된 `spacing_before`** 를 기준점에 실어
//! 쪽 적합 판정을 쪽마다 상수만큼 낙관하던 결함의 가드.
//!
//! ## 무엇이 어긋났나 — 페인트와 적합 판정이 다른 기준을 쓴다
//!
//! 쪽 base 확립(`typeset.rs` `st.current_items.is_empty()` 분기)은 쪽이 **연속 조각으로
//! 시작**하면 돌지 않는다. 조각이 이미 항목을 차지해 뒤따르는 첫 문단이 그 분기에 못
//! 들어가기 때문이다. 그러면 `vpos_page_base` 가 `None` 인 채로 lazy 역산이 돌고,
//! 그 역산은 sequential `y_offset` 을 쓰는데 **거기서 `#2279 ①` 트림이 `sb` 를 빼 갔다.**
//!
//! `27469` 5쪽 실측 — `pi=26`(`4. 판단`, `sb 40.00px`)이 트림된 뒤 `pi=27` 에서 역산:
//!
//! ```text
//!   현행   lazy_base = 16560 − (11962 + 1600) = 2998 HU   ← 트림된 sb 그 자체
//!   교정   lazy_base = 16560 − (14962 + 1600) =   −2 HU   → 0 (반올림 잡음)
//! ```
//!
//! 페인트는 기준 0 으로 그린다 — `pi=35` 첫 줄 실측 `969.40 = Body.y 113.40 + 사다리 856.00`.
//! 즉 **적합 판정만** 40px 낙관해, 안 들어가는 줄이 그 쪽에 남아 본문 하한을 넘었다.
//!
//! ```text
//!   종전  cur_h(pi=35) = 809.3 → 두 줄(70.4px)이 다 들어간다고 본다
//!   교정  cur_h(pi=35) = 849.3 → 한 줄만 들어간다 (한/글 2020 오라클과 같은 배분)
//! ```
//!
//! ## 좁힌 범위
//!
//! * 되돌림은 **역산에만** 건다. 전진량(`flow_advance_height`)은 그대로다.
//! * `sa` 만 트림된 자리는 0 이다(`flow_trimmed_spacing_before` 가 같은 판정을 다시 묻는다).
//! * `−16 HU` 클램프는 **되돌림이 실제로 있었을 때만** — 자리차지 표 등에서 크게 음수인
//!   역산 무효 가드는 그대로 살아 있어야 한다.
//! * **네이티브 HWP5 한정.** 전 포맷에 켠 판은 `issue1880_*.hwpx` 2건을 악화시켰다
//!   (5쪽 넘침 1 → 4, 최대 +82.65px). HWPX 의 `vpos` 리셋은 writer-local 재시작일 수
//!   있어 별도 기계가 다룬다.
//!
//! ## 지키는 핀
//!
//! `tests/issue_1733.rs` 의 `242`(한컴 2020 PDF 오라클)는 이 축에서 **먼저 시도한
//! "조각 시작 줄 `vpos == 0` 을 쪽 기준으로 못박기" 를 기각시킨 핀**이다(242 → 243).
//! 기준을 단정하지 않고 역산식만 고치면 242 가 유지된다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6718/27469-child-allowance-retroactive-support.hwp";
/// 0-based — 조각으로 시작하는 5쪽.
const PAGE_INDEX: u32 = 4;
/// `body_area y=113.40 + h=895.80`.
const BODY_BOTTOM_PX: f64 = 1009.20;
const TOLERANCE_PX: f64 = 8.0;

/// 꼬리말(쪽번호)은 본문 하한 아래에 있는 것이 정상이다 — `Body` 아래만 모은다.
fn collect_body_runs<'a>(node: &'a RenderNode, in_body: bool, out: &mut Vec<&'a RenderNode>) {
    let in_body = in_body || matches!(node.node_type, RenderNodeType::Body { .. });
    if in_body {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            if !run.text.trim().is_empty() {
                out.push(node);
            }
        }
    }
    for child in &node.children {
        collect_body_runs(child, in_body, out);
    }
}

#[test]
fn fragment_led_page_does_not_lose_the_trimmed_spacing_before() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue6718 sample");
    assert_eq!(
        document.page_count(),
        12,
        "쪽수는 한/글 2020 오라클과 같은 12쪽이어야 한다 — 이 축은 배분만 바꾼다"
    );

    let tree = document
        .build_page_render_tree(PAGE_INDEX)
        .expect("render p5");
    let mut nodes = Vec::new();
    collect_body_runs(&tree.root, false, &mut nodes);

    let worst = nodes
        .iter()
        .map(|node| node.bbox.y + node.bbox.height)
        .fold(f64::NEG_INFINITY, f64::max);

    assert!(
        worst.is_finite(),
        "5쪽에 글자가 있어야 한다 — 표본이나 쪽 인덱스가 어긋났다"
    );
    assert!(
        worst <= BODY_BOTTOM_PX + TOLERANCE_PX,
        "5쪽 글자가 본문 하한을 넘었다 — 최하단 {worst:.1}px, 하한 {BODY_BOTTOM_PX:.1}px \
         (회귀 시 1058.4px = +49.3px, 쪽번호 자리에 그려진다)"
    );

    // 6쪽 첫 글줄은 한/글이 넘긴 그 줄이어야 한다.
    let next = document
        .build_page_render_tree(PAGE_INDEX + 1)
        .expect("render p6");
    let mut next_nodes = Vec::new();
    collect_body_runs(&next.root, false, &mut next_nodes);
    let first_text = next_nodes
        .iter()
        .min_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y))
        .and_then(|node| match &node.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.clone()),
            _ => None,
        })
        .unwrap_or_default();
    assert!(
        first_text.contains("비용"),
        "6쪽 첫 글줄은 한/글 2020 오라클과 같이 `비용의 지원을 신청할 수 있다.` 여야 한다 — got {first_text:?}"
    );
}
