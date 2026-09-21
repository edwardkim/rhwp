//! [#6761] 문단 경계의 저장 `vpos` 되감김을 분할 경로가 쪽 경계로 쓰지 않는다.
//!
//! # 무엇이 깨져 있었나
//!
//! 저장 사다리가 문단 경계에서 **쪽 위쪽 띠로 되돌아가면** 한/글이 거기서 쪽을 끊었다는
//! 뜻이다. `typeset` 은 그 신호를 이미 계산한다(`stored_vpos_rewind_break`) — 계산되면
//! 「문단 전체 배치」 분기를 건너뛴다. 그런데 이어지는 **줄 단위 분할 경로**의 쪽 넘김
//! 조건이 그 값을 보지 않아, 남은 여백에 첫 줄이 들어가기만 하면 같은 쪽에 얹었다.
//! 그래서 되감김이 말한 경계가 무시되고 본문 바닥 아래에 글줄이 그려진다.
//!
//! ```text
//!   samples/task2070/1130000-201900011_…_2017년기준 시장구조조사.hwp  4쪽(0-based 3)
//!     저장 vpos  pi=41 53956  ->  pi=42 500      되감김(= 다음 쪽 상단에서 재시작)
//!     진단       DIAG_COMPAT24 rewind-site pi=42 break=true cur=694.6 avail=744.5
//!     수정 전    pi=42·43 을 4쪽 969.5 / 995.7 에 그린다 — 본문 바닥 961.8 **밖**
//! ```
//!
//! # 독립 기대값 — 한/글 정본
//!
//! `pdf/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사-2022.pdf`
//! (315쪽, 이 문서의 정본)에서 그 두 줄은 **5쪽 첫 두 줄**이다.
//!
//! ```text
//!   정본 p4 마지막 줄  '2. 중분류 산업별 집중률…'        905.1..920.4 px
//!   정본 p5 첫 줄      '제Ⅴ장 산업집중도 …'             221.4 px
//!   정본 p5 둘째 줄    '제1절 산업집중률의 현황과 추이…'  249.8 px
//! ```
//!
//! # 반례 — 되감김이라고 다 쪽 경계는 아니다
//!
//! `stored_vpos_rewinds` 는 값이 **줄기만 하면** 참이라 같은 쪽 안의 부분 후퇴도 포함한다.
//! `samples/task2287/1342000_edu_curriculum_map.hwp` 는 표 아래 주석이 앞 문단보다 위에서
//! 시작한다(`pi=219 70880 -> pi=220 66140`). 쪽 경계의 되감김은 **쪽 상단 띠에서 다시
//! 시작**하므로, 앞 값에 쓰는 `> 5000` 의 거울로 새 값에 `<= 5000` 을 요구한다. 그 조건이
//! 없으면 이 문서가 413 -> 414쪽이 되어 정본(415쪽) 오프셋 정렬이 깨진다(`#7226` 핀).

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 되감김이 쪽 경계인 문서. 정본은 `pdf/task2070/…-2022.pdf`(315쪽).
const SAMPLE_MARKET: &str =
    "samples/task2070/1130000-201900011_D0150004-1-002_2017년기준 시장구조조사.hwp";
/// 반례 — 같은 쪽 안의 부분 후퇴가 있는 문서. 정본 415쪽, 1..144쪽 오프셋 0.
const SAMPLE_EDU: &str = "samples/task2287/1342000_edu_curriculum_map.hwp";

fn core(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("정식 원본")).expect("문서 로드")
}

fn line_text(node: &RenderNode) -> String {
    let mut out = String::new();
    fn walk(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.push_str(&run.text);
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    walk(node, &mut out);
    out
}

/// (줄 상단, 줄 하단, 텍스트) — y 오름차순.
fn text_lines(root: &RenderNode) -> Vec<(f64, f64, String)> {
    let mut out = Vec::new();
    fn walk(node: &RenderNode, out: &mut Vec<(f64, f64, String)>) {
        if matches!(node.node_type, RenderNodeType::TextLine { .. }) {
            out.push((node.bbox.y, node.bbox.y + node.bbox.height, line_text(node)));
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    walk(root, &mut out);
    out.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("유한값"));
    out
}

fn body_bottom(root: &RenderNode) -> f64 {
    fn walk(node: &RenderNode) -> Option<f64> {
        if matches!(node.node_type, RenderNodeType::Body { .. }) {
            return Some(node.bbox.y + node.bbox.height);
        }
        node.children.iter().find_map(walk)
    }
    walk(root).expect("Body 노드")
}

/// 되감김 경계의 두 문단은 다음 쪽 머리로 간다 — 본문 바닥 아래에 그리지 않는다.
#[test]
fn stored_rewind_paragraph_starts_the_next_page() {
    let core = core(SAMPLE_MARKET);
    assert_eq!(core.page_count(), 315, "정본과 같은 쪽수여야 한다");

    let p4 = core.build_page_render_tree(3).expect("4쪽 렌더 트리").root;
    let bottom = body_bottom(&p4);
    let escaped: Vec<_> = text_lines(&p4)
        .into_iter()
        .filter(|(_, line_bottom, text)| *line_bottom > bottom + 2.0 && !text.trim().is_empty())
        .collect();
    assert!(
        escaped.is_empty(),
        "4쪽 본문 바닥({bottom:.1}) 아래에 글줄이 그려졌다 — 되감김 경계가 무시됐다: {escaped:?}"
    );

    let p5 = core.build_page_render_tree(4).expect("5쪽 렌더 트리").root;
    let heads: Vec<String> = text_lines(&p5)
        .into_iter()
        .map(|(_, _, text)| text.trim().to_string())
        .filter(|text| !text.is_empty())
        .take(2)
        .collect();
    assert!(
        heads
            .first()
            .is_some_and(|line| line.starts_with("제Ⅴ장 산업집중도")),
        "5쪽 첫 줄은 정본대로 '제Ⅴ장 산업집중도' 여야 한다 — got {heads:?}"
    );
    assert!(
        heads
            .get(1)
            .is_some_and(|line| line.starts_with("제1절 산업집중률의 현황과 추이")),
        "5쪽 둘째 줄은 정본대로 '제1절 산업집중률의 현황과 추이' 여야 한다 — got {heads:?}"
    );
}

/// 반례 — 같은 쪽 안의 부분 후퇴는 쪽을 끊지 않는다.
#[test]
fn partial_retreat_inside_a_page_does_not_break() {
    let core = core(SAMPLE_EDU);
    assert_eq!(
        core.page_count(),
        413,
        "쪽 상단 재시작이 아닌 부분 후퇴(pi=219 70880 -> pi=220 66140)에서 쪽을 끊으면 \
         이 문서가 414쪽이 되어 정본(415쪽) 1..144쪽 오프셋 0 정렬이 깨진다"
    );
}
