//! [#7288] 앞 프레임에 속한 빈 문단이 새 프레임의 첫 줄을 밀지 않는다.
//!
//! # 무엇이 깨져 있었나
//!
//! 「나누지 않음」 표가 통째로 다음 쪽에 놓이면, 그 뒤를 따르는 빈 문단 하나가 새 쪽
//! 상단에서 줄 상자를 **새로** 차지했다. 그 빈 문단의 저장 `vpos` 는 우리가 막 떠난
//! 프레임의 좌표인데도 현재 프레임에서 한 줄을 먹어, 이후 모든 줄이 25.6px 씩 밀리고
//! 그 쪽의 마지막 줄이 본문 아래로 12.2px 넘쳤다.
//!
//! ```text
//!   pi=1372 빈 문단  저장 vpos 73300 -> 977.3px  (앞 프레임 바닥)
//!   pi=1373 빈 문단  저장 vpos 19720 -> 262.9px  = 표 바닥 = 조판 위치
//!   pi=1374 "주) …"  저장 vpos 21640 -> 288.5px
//! ```
//!
//! # 기대값의 출처 — 저장 사다리와 한컴 PDF 가 서로를 확증한다
//!
//! 이 문서의 저장 사다리는 `pi=1374` 를 본문 상단 기준 `21640HU = 288.53px` 에 둔다.
//! 본문 상단이 `75.59px` 이므로 절대 좌표는 **`364.12px`** 이다.
//!
//! 저장소 정본 `pdf/text_footnote_tail_overpagination-2024.pdf` 의 같은 쪽에서 같은 줄
//! ("주) 대각선 양측의 숫자는 …")은 **`363.9px`**(PDF 272.93pt × 96/72)에 있다. 두
//! 독립 출처가 0.3px 안에서 일치하므로, 이 좌표는 구현이 계산한 값의 재인용이 아니다.
//!
//! 종전 rhwp 는 이 줄을 `389.7px` — 정확히 한 줄(25.6px) 아래 — 에 그렸다.
//!
//! # 대조군
//!
//! 같은 쪽에서 본문 아래로 넘치는 요소가 없어야 한다. 밀림이 되살아나면 마지막 줄이
//! 다시 본문을 넘으므로, 위치 검사와 넘침 검사가 같은 결함을 두 각도로 잡는다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/task1725/text_footnote_tail_overpagination.hwp";
/// 검사 대상 쪽 (0-based). 7×7 «나누지 않음» 표 `표 7.4-2` 가 통째로 놓이는 쪽.
const PAGE: u32 = 63;
/// 그 쪽에서 표 뒤 첫 본문 문단.
const PARA: usize = 1374;
/// 저장 사다리가 지시하는 위치 — `21640HU / 75 + 본문 상단 75.5867px`.
const LADDER_Y_PX: f64 = 21640.0 / 75.0 + 75.586_666_67;
/// 한컴 2024 정본 PDF 의 같은 줄 — `272.93pt × 96/72`.
const ORACLE_Y_PX: f64 = 363.9;
/// 종전 rhwp 위치 — 한 줄(25.6px) 아래.
const REGRESSED_Y_PX: f64 = 389.7;

fn collect_lines(node: &RenderNode, out: &mut Vec<(usize, f64, f64)>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if let Some(pi) = line.para_index {
            out.push((pi, node.bbox.y, node.bbox.y + node.bbox.height));
        }
    }
    for child in &node.children {
        collect_lines(child, out);
    }
}

fn body_bottom(node: &RenderNode) -> Option<f64> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox.y + node.bbox.height);
    }
    node.children.iter().find_map(body_bottom)
}

fn page() -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    core.build_page_render_tree(PAGE)
        .expect("대상 쪽 render tree")
        .root
}

/// 표 뒤 첫 본문 줄이 저장 사다리가 지시한 자리에 선다 — 한컴 PDF 와 같은 자리다.
#[test]
fn the_line_after_the_table_sits_where_the_ladder_says() {
    let root = page();
    let mut lines = Vec::new();
    collect_lines(&root, &mut lines);

    let first = lines
        .iter()
        .filter(|(pi, _, _)| *pi == PARA)
        .map(|(_, top, _)| *top)
        .fold(f64::INFINITY, f64::min);
    assert!(
        first.is_finite(),
        "문단 {PARA} 의 줄을 찾지 못했다 — 시험 설정 오류. 줄 수={}",
        lines.len()
    );

    assert!(
        (first - LADDER_Y_PX).abs() <= 1.0,
        "표 뒤 첫 본문 줄이 저장 사다리 자리({LADDER_Y_PX:.1}px, 한컴 PDF {ORACLE_Y_PX:.1}px)에 \
         서야 한다 — 앞 프레임 빈 문단이 줄을 차지하면 {REGRESSED_Y_PX:.1}px 로 한 줄 밀린다. \
         got {first:.1}px"
    );
}

/// 같은 쪽에 본문 아래로 넘치는 줄이 없다.
#[test]
fn nothing_on_that_page_overflows_the_body() {
    let root = page();
    let bottom = body_bottom(&root).expect("Body 노드");
    let mut lines = Vec::new();
    collect_lines(&root, &mut lines);

    let over: Vec<String> = lines
        .iter()
        .filter(|(_, _, line_bottom)| *line_bottom > bottom + 2.0)
        .map(|(pi, top, line_bottom)| format!("pi={pi} y={top:.1}..{line_bottom:.1}"))
        .collect();
    assert!(
        over.is_empty(),
        "본문 바닥({bottom:.1}px) 아래로 그려진 줄이 있다 — 앞 프레임 빈 문단이 다시 줄을 \
         차지해 전부 밀린 것이다. {over:?}"
    );
}
