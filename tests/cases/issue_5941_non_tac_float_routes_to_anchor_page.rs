//! [#5941] 앵커 문단이 쪽 분할되면 **비-TAC 그림/도형**도 앵커 줄이 있는 쪽에 등록한다.
//!
//! ## 종전 결함
//!
//! `section/controls.rs` 의 개체 방출은 `is_routable_treat_as_char_picture_or_shape`
//! (= `treat_as_char` 그대로)가 참일 때만 `find_inline_control_target_page` 로
//! 라우팅했다. 비-TAC 개체는 `routed = None` 이 되어 `append_item` 으로 **그 시점의
//! 현재 쪽**, 곧 앵커 문단이 **끝난** 쪽에 붙는다.
//!
//! 바로 그 실패 모드가 같은 자리의 `#476/#4092` 주석에 적혀 있다 —
//! *"paragraph 가 페이지 분할되면 이 시점의 `st.current_items` 는 마지막 페이지 상태이므로,
//! 그대로 push 하면 박스가 잘못된 페이지에 떠 있게 된다"*. 진단이 맞는데 적용 범위만 좁았다.
//!
//! ## 이 문서 (`samples/issue5941/1490000-201600081_roadmap_research.hwp`, 304쪽)
//!
//! 구역 4 `pi=23` 은 **용지 기준 자리차지 묶음**(세로 오프셋 9544HU = 127.3px, 높이
//! 45807HU = 610.8px)을 달고 있고, 그 문단이 161~162쪽으로 나뉜다.
//!
//! ```text
//!   개체 앵커 줄            line 0        -> 161쪽
//!   종전 rhwp 개체 배치      162쪽         (문단이 끝난 쪽)
//!   한/글 정본 배치          161쪽
//! ```
//!
//! 161쪽에 개체 자리가 예약되지 않아 본문이 34줄 채워지고 하단(1009.1px)을 **555.8px**
//! 넘겼다. 정본은 `Hancom PDF 1.3.0.534` 로 출력해 대조했다 — 그 쪽 본문은 그림 아래
//! 624.9px 부터 시작하는 캡션 6줄뿐이다.
//!
//! ## 수정 후 (같은 빌드 env A/B 실측)
//!
//! | | 수정 전 | 수정 후 |
//! | --- | ---: | ---: |
//! | 이 문서 `overflow` | 52 | **30** |
//! | 이 문서 `offCanvas` | 17 | **0** |
//! | 쪽수 | 304 | 304 |
//! | 이 쪽 최대 하단 초과 | +555.8px | **+56.7px** |
//!
//! **잔여 +56.7px 는 이 수정의 범위가 아니다.** 개체가 제 쪽으로 돌아와 본문이 34줄에서
//! 줄었지만 여전히 조금 넘는다 — 정본 대비 쪽수도 304 vs 302 로 +2 가 남는다.
//! 이 시험은 **개체 소유 쪽 교정**이 만든 크기 변화를 잠그고, 잔여는 미해결로 남긴다.
//!
//! `samples/` 전수(1,141 문서) A/B 에서 **악화 0** — 움직인 문서는 이 하나뿐이고
//! 쪽수 합계(15,510)·겹침(125)·빈쪽(95)은 전부 불변이다. 앵커 줄이 현재 쪽에 있으면
//! `find_inline_control_target_page` 가 `None` 을 돌려주므로 제자리 개체는 안 움직인다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue5941/1490000-201600081_roadmap_research.hwp";
/// 문제의 쪽 (0-based). 신고된 하단 초과가 여기 22노드 몰려 있었다.
const PAGE: u32 = 161;

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

#[test]
fn non_tac_float_stays_on_its_anchor_page_so_the_text_does_not_overflow() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    let tree = core.build_page_render_tree(PAGE).expect("render tree");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);

    // 본문 하한은 Body 노드가 정한다 — 상수로 박지 않는다.
    let body_bottom = nodes
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Body { .. }))
        .map(|n| n.bbox.y + n.bbox.height)
        .expect("Body 노드");

    // 1. 하단 초과가 한 자릿수 배로 줄어든다. 수정 전 +555.8px · 수정 후 +56.7px.
    //    0 이 아니다 — 잔여는 다른 축이며 이 시험이 주장하지 않는다.
    let worst = nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::TextLine(_)))
        .map(|n| n.bbox.y + n.bbox.height - body_bottom)
        .fold(f64::MIN, f64::max);
    assert!(
        worst <= 100.0,
        "#5941: 하단 초과가 수정 전(+555.8px) 규모로 돌아가면 안 된다 — 본문 하한 {body_bottom:.1}, 지금 {worst:+.1}px"
    );

    // 2. 이 수정이 한 일을 직접 검사한다 — **그 쪽에 개체가 있다.**
    //    종전에는 이 묶음(45807HU = 610.8px)이 다음 쪽에 붙어 이 쪽에 없었다.
    let tallest = nodes
        .iter()
        .filter(|n| !matches!(n.node_type, RenderNodeType::TextLine(_)))
        .map(|n| n.bbox.height)
        .fold(0.0f64, f64::max);
    assert!(
        tallest > 500.0,
        "#5941: 앵커 줄이 이 쪽에 있으므로 610.8px 묶음도 이 쪽에 있어야 한다          — 가장 큰 비-글줄 노드 {tallest:.1}px"
    );
}
