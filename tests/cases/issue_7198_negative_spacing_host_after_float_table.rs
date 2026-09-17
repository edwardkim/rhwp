//! [#7198] 빈 host 문단의 자리차지 표 뒤 첫 본문은 저장 사다리대로 host 줄 전진을 받는다 —
//! host 줄간격이 **음수**여도.
//!
//! #2439·#6147 은 `next.vpos - host.vpos == lh + ls` 인 저장 사다리를 "한글이 표 높이를 접고
//! host 줄 전진만 흐름에 계상했다"는 증거로 받아 표 뒤에 host 줄(과 양수 offset)을 계상한다.
//! 그런데 전진량을 `lh + max(ls, 0)` 으로 계산해, 줄간격이 음수인 문서는 등식에서 떨어지고
//! 흐름이 표 높이만 전진했다.
//!
//! 정답지 — 한/글 2020 MCP PDF(`samples/issue7198/*-2020.pdf`) 1쪽 첫 본문 기준선:
//!
//! | 문서 | host `lh`/`ls` | 한/글 기준선 | 저장 vpos+bl | 수정 전 rhwp 줄 위치 차 |
//! | --- | --- | ---: | ---: | ---: |
//! | 156403546 | 1100 / −112 | 305.9 | 306.2 | −18.9px |
//! | 156086935 (대조군) | 1500 / +600 | 305.3 | 305.6 | 0.0 |
//!
//! 이슈 원문서 156467175(10MB, 코퍼스 `korea_downloads/공정거래위원회`, sha256 6b6ba5d4…)는
//! `ls=-1052` 로 같은 형상이며 수정 전 −11.7px, 수정 후 0.0 이다(한/글 기준선 271.5 / 저장
//! 271.8). 이슈 본문의 "+82.8px 아래" 는 저장 vpos(본문 기준)를 쪽 절대 좌표와 비교한 착오다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const NEGATIVE: &str = "samples/issue7198/156403546_negative_spacing_host_after_table.hwp";
const POSITIVE: &str = "samples/issue7198/156086935_positive_spacing_host_after_table.hwp";

fn first_body_run_y(node: &RenderNode, para_index: usize) -> Option<f64> {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.section_index == Some(0)
            && run.para_index == Some(para_index)
            && run.cell_context.is_none()
        {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| first_body_run_y(child, para_index))
}

fn body_top(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y);
    }
    node.children.iter().find_map(body_top)
}

/// 문단 0.1 첫 줄의 렌더 y 와 `본문 윗변 + 저장 vpos` 의 차(px).
fn title_offset_from_stored_vpos(rel: &str) -> f64 {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {rel}: {e:?}"));
    let stored_vpos = core.document().sections[0].paragraphs[1]
        .line_segs
        .first()
        .expect("문단 0.1 저장 줄")
        .vertical_pos;
    let page = core.build_page_render_tree(0).expect("1쪽");
    let top = body_top(&page.root).expect("Body 노드");
    let y = first_body_run_y(&page.root, 1).expect("문단 0.1 글자");
    y - (top + f64::from(stored_vpos) * 96.0 / 7200.0)
}

#[test]
fn issue_7198_negative_host_spacing_keeps_stored_title_position() {
    let delta = title_offset_from_stored_vpos(NEGATIVE);
    assert!(
        delta.abs() <= 0.5,
        "음수 줄간격 host 뒤 첫 본문이 저장 vpos 에서 {delta:+.1}px 어긋났다 \
         (한/글은 저장 위치에 그린다 — 수정 전 −18.9px)"
    );
}

#[test]
fn issue_7198_positive_host_spacing_is_unchanged() {
    let delta = title_offset_from_stored_vpos(POSITIVE);
    assert!(
        delta.abs() <= 0.5,
        "양수 줄간격 대조군이 저장 vpos 에서 {delta:+.1}px 어긋났다"
    );
}
