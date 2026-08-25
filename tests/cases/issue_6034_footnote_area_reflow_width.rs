//! [#6034] 각주 구분선이 본문 마지막 줄을 관통하고 각주가 그 줄에 밀착하던
//! 결함 가드.
//!
//! 2912735(법원 서식, 12KB HWP5, 한글 6.x 대 저장본) — 각주 문단에 저장
//! LINE_SEG 가 없어 compose_lines 의 45자 고정 휴리스틱 폴백으로 래핑됐고,
//! 실제 각주 영역 폭(604.7px)보다 좁게 꺾여 각주 블록이 부풀었다(rhwp 15줄
//! vs 한글 9줄). 블록은 본문 하단 bottom-anchor 라 부푼 만큼 영역 상단이
//! 본문 마지막 줄 위로 올라가(764.8 < 본문 766.0~780.7) 구분선(772.1)이
//! 글줄 한가운데를 지났다(한글 2024 실측: 구분선 791.7px = 본문 밑 +11px).
//! 수정 = LINE_SEG 없는 각주 문단을 각주 영역 폭으로 재조판(reflow)해
//! compose — 편집 경로(footnote_ops)의 reflow 계약과 같은 상자. LINE_SEG 가
//! 있는 문단은 종전 그대로다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6034/2912735_court_report_form.hwp";

fn find_footnote_area<'a>(node: &'a RenderNode) -> Option<&'a RenderNode> {
    if matches!(&node.node_type, RenderNodeType::FootnoteArea) {
        return Some(node);
    }
    node.children.iter().find_map(find_footnote_area)
}

fn body_text_bottom(node: &RenderNode, in_footnote: bool) -> f64 {
    let in_footnote = in_footnote || matches!(&node.node_type, RenderNodeType::FootnoteArea);
    let own = match &node.node_type {
        RenderNodeType::TextRun(run) if !in_footnote && !run.text.trim().is_empty() => {
            node.bbox.y + node.bbox.height
        }
        _ => f64::MIN,
    };
    node.children
        .iter()
        .map(|child| body_text_bottom(child, in_footnote))
        .fold(own, f64::max)
}

fn footnote_text_lines(node: &RenderNode) -> usize {
    let own = match &node.node_type {
        RenderNodeType::TextLine(_) => usize::from(node.children.iter().any(
            |c| matches!(&c.node_type, RenderNodeType::TextRun(r) if !r.text.trim().is_empty()),
        )),
        _ => 0,
    };
    own + node.children.iter().map(footnote_text_lines).sum::<usize>()
}

#[test]
fn issue_6034_footnote_area_starts_below_body_text() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("parse #6034 fixture");

    let p1 = core.build_page_render_tree(0).expect("render p1");
    let body_bottom = body_text_bottom(&p1.root, false);
    assert!(
        (770.0..=790.0).contains(&body_bottom),
        "본문 마지막 줄 하단 앵커 {body_bottom:.1} — 표본 형상이 달라짐"
    );

    let area = find_footnote_area(&p1.root).expect("p1 각주 영역");
    // 결함 상태: 영역 상단 764.8 < 본문 하단 780.7 → 구분선이 글줄 관통.
    assert!(
        area.bbox.y >= body_bottom - 0.5,
        "각주 영역 상단({:.1})이 본문 마지막 줄 하단({body_bottom:.1}) 위에 있음 — \
         구분선이 본문을 관통한다",
        area.bbox.y,
    );

    // 45자 폴백이면 12줄(+빈 3줄)로 부푼다 — 영역 폭 재조판이면 한글과 같은 9줄.
    let lines = footnote_text_lines(area);
    assert!(
        (8..=10).contains(&lines),
        "각주 텍스트 줄 수 {lines} — 폭 재조판이 아니라 45자 폴백이면 12줄로 부풂 \
         (한글 2024 실측 9줄)",
    );
}
