//! [#6929] 문단 기준 자리차지 표의 `vertOffset` 을 push-down 이 통째로 덮어쓰지 않는다.
//!
//! # 무엇이 깨져 있었나
//!
//! `#347` 의 push-down(`raw_y.max(y_start)`)은 자리차지(TopAndBottom) 표를 **앞선 표·텍스트
//! 아래로** 민다. 그런데 앵커 문단이 단 최상단에 있고 그 문단이 잉크를 하나도 안 내면,
//! `y_start` 가 앵커보다 아래인 것은 **앵커 문단 자신의 줄 예약**뿐이다. 그 아래로 밀면
//! 문단 기준 `vertOffset` 이 통째로 무시된다.
//!
//! ```text
//!   본문 상단 94.5 · vertOffset 433 HU = 5.77px
//!   raw_y   = 94.5 + 5.77 = 100.3      ← 선언대로
//!   y_start = 118.5 (= 94.5 + 24.0, 앵커 문단의 줄 예약)
//!   pushed  = max(100.3, 118.5) = 118.5   ← +18.2px, 표 아래끝이 제목을 침범
//! ```
//!
//! # 좁힘 — 공동 앵커 형제가 있으면 그대로 둔다
//!
//! 한 문단에 자리차지 표가 여러 개 달리면(co-anchored) 그 쌓임은 `y_start` 가 만든다.
//! `#1639`(`issue1639_empty_host_negative_offset_float.hwpx`)가 그 순서를 잠그고 있고,
//! 완화를 넓히면 형제들이 같은 자리로 모여 겹친다. 그래서 **단에 아직 아무것도 안 놓였을
//! 때**(`col_node.children.is_empty()`)만 앵커를 바닥으로 쓴다. 보이는 host 제목이 있는
//! `#1549` 형상은 앵커가 단 최상단이어도 제목 줄이 먼저 놓이므로 조건에서 빠진다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6929/148776468_search_ad_terms_press_release.hwp";

/// 한/글 2020 정본(#6929 본문, `hwp2024Convert` job `f382c2c1-…`)이 잰 1쪽 좌표.
///
/// 저장 선언과도 맞물린다 — 본문 상단 94.5 + `vertOffset` 433 HU(5.77px) = **100.27**,
/// 정본 100.2 와 0.07px 안이다. 곧 이 값은 외부 측정과 저장 선언이 같이 가리키는 자리다.
const ORACLE_TABLE_TOP: f64 = 100.2;
/// 정본의 제목 문단 상단. 표가 이 아래로 내려오면 글자가 겹친다.
const ORACLE_TITLE_TOP: f64 = 268.2;
const TITLE: &str = "인터넷포털 검색광고서비스";

fn page0(sample: &str) -> RenderNode {
    let bytes = std::fs::read(sample).unwrap_or_else(|e| panic!("재현체 {sample}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("로드 {sample}: {e}"));
    core.build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("렌더 {sample}: {e}"))
        .root
}

fn line_text(node: &RenderNode) -> String {
    let mut s = String::new();
    if let RenderNodeType::TextRun(run) = &node.node_type {
        s.push_str(run.display_or_text());
    }
    for c in &node.children {
        s.push_str(&line_text(c));
    }
    s
}

/// 본문 최상단 자리차지 표 — 쪽 맨 위의 머리 표.
fn head_table(node: &RenderNode, out: &mut Option<(f64, f64)>) {
    if !node.visible || node.editor_only {
        return;
    }
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        let top = node.bbox.y;
        if out.is_none_or(|(cur, _)| top < cur) {
            *out = Some((top, top + node.bbox.height));
        }
        return;
    }
    for c in &node.children {
        head_table(c, out);
    }
}

fn title_top(node: &RenderNode, out: &mut Option<f64>) {
    if !node.visible || node.editor_only {
        return;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && line_text(node).contains(TITLE) {
        if out.is_none_or(|cur| node.bbox.y < cur) {
            *out = Some(node.bbox.y);
        }
        return;
    }
    for c in &node.children {
        title_top(c, out);
    }
}

#[test]
fn para_relative_float_table_starts_at_its_declared_offset() {
    let root = page0(SAMPLE);
    let mut table = None;
    head_table(&root, &mut table);
    let (top, _) = table.expect("1쪽 머리 표를 못 찾았습니다");
    assert!(
        (top - ORACLE_TABLE_TOP).abs() <= 1.0,
        "머리 표 상단이 정본과 다릅니다 — 정본 {ORACLE_TABLE_TOP:.1} · 현재 {top:.1} \
         (저장 vertOffset 433 HU = 5.77px, 본문 상단 94.5)"
    );
}

#[test]
fn the_head_table_does_not_reach_into_the_title() {
    let root = page0(SAMPLE);
    let mut table = None;
    head_table(&root, &mut table);
    let (_, bottom) = table.expect("1쪽 머리 표를 못 찾았습니다");
    let mut title = None;
    title_top(&root, &mut title);
    let title = title.expect("제목 문단을 못 찾았습니다");

    assert!(
        (title - ORACLE_TITLE_TOP).abs() <= 1.0,
        "제목 상단이 정본과 다릅니다 — 정본 {ORACLE_TITLE_TOP:.1} · 현재 {title:.1}"
    );
    assert!(
        bottom <= title + 0.5,
        "표 아래끝이 제목을 {:.1}px 침범합니다 — 표 바닥 {bottom:.1} · 제목 상단 {title:.1}",
        bottom - title
    );
}

/// 반례 — 보이는 host 제목이 있으면 표는 제목 **아래**로 간다(`#1549` 계약).
///
/// 이 완화가 넓어지면 제목과 표가 같은 자리에서 시작해 겹친다.
#[test]
fn visible_host_title_still_pushes_its_float_table_down() {
    let root = page0("samples/issue1549_multipositive_float_tables.hwpx");
    let mut title = None;
    title_top_of(&root, "MULTI POSITIVE TITLE", &mut title);
    let title = title.expect("host 제목을 못 찾았습니다");
    let mut table = None;
    head_table(&root, &mut table);
    let (top, _) = table.expect("자리차지 표를 못 찾았습니다");
    assert!(
        top > title,
        "보이는 host 제목이 있으면 표가 그 아래여야 합니다 — 제목 {title:.1} · 표 {top:.1}"
    );
}

fn title_top_of(node: &RenderNode, needle: &str, out: &mut Option<f64>) {
    if !node.visible || node.editor_only {
        return;
    }
    if matches!(node.node_type, RenderNodeType::TextLine(_)) && line_text(node).contains(needle) {
        if out.is_none_or(|cur| node.bbox.y < cur) {
            *out = Some(node.bbox.y);
        }
        return;
    }
    for c in &node.children {
        title_top_of(c, needle, out);
    }
}
