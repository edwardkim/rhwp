//! [Issue #6646] 묶음 빈칸 전진폭이 글꼴마다 달라 문항 번호 뒤 본문이 당겨진다.
//!
//! 근인: 묶음 빈칸(HWP5 문자 컨트롤 30 → `U+00A0`)을 일반 글자처럼 글꼴 글리프
//! 폭으로 재고 있었다. 묶음 빈칸은 **줄바꿈만 막는 공백**이라 전진폭이 일반 공백과
//! 같아야 하는데, 글꼴 표의 두 값이 제각각이다.
//!
//! `exam_eng.hwp` 1쪽 실측에서 묶음 빈칸과 일반 공백의 폭이 달랐다.
//! 글꼴 표 자체도 567개 중 149개에서 두 값이 다르고 50개는 0 이다.
//!
//! 수정: `measure_char_width_embedded_decision` 에서 묶음 빈칸을 일반 공백과 같은
//! 갈래로 넣는다.
//!
//! 한컴 PDF와의 시각 대조는 별도 Visual Sweep 증적에 남기고, 이 검사는
//! 글줄 내용 및 공백 측정 갈래를 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/exam_eng.hwp";
/// 7번 문항 줄을 고르는 표식. 번호와 본문이 한 줄에 있다.
const LINE_PREFIX: &str = "7.";
const LINE_CONTAINS: &str = "대화를";
fn find_question_line(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let text = node
            .children
            .iter()
            .filter_map(|child| match &child.node_type {
                RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                _ => None,
            })
            .collect::<String>();
        if text.starts_with(LINE_PREFIX) && text.contains(LINE_CONTAINS) {
            return Some(node);
        }
    }
    node.children.iter().find_map(find_question_line)
}

#[test]
fn issue_6646_nbsp_advance_matches_plain_space() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let tree = core.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let question_line = find_question_line(&tree.root)
        .unwrap_or_else(|| panic!("1쪽에서 `{LINE_PREFIX}…{LINE_CONTAINS}` 글줄을 찾아야 한다"));
    let line_text = question_line
        .children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect::<String>();
    assert!(
        line_text.contains("7.\u{00a0}\u{2007}대화를"),
        "7번 문항 내용과 공백 제어문자가 같은 글줄에 있어야 함: {line_text:?}"
    );

    let trace: serde_json::Value = serde_json::from_str(
        &core
            .get_font_decision_trace_native(0, r#"{"maxCharacters":4096}"#)
            .expect("1쪽 글꼴 결정 추적"),
    )
    .expect("글꼴 결정 추적 JSON");
    let records = trace["records"].as_array().expect("글꼴 결정 기록");
    let nbsp: Vec<_> = records
        .iter()
        .filter(|record| record["source"]["character"] == "\u{00a0}")
        .collect();
    assert!(!nbsp.is_empty(), "원문에 묶음 빈칸이 있어야 함");
    for record in nbsp {
        assert!(
            matches!(
                record["layoutMetric"]["widthSource"].as_str(),
                Some("metricHalfSpace" | "metricSpaceOverlay")
            ),
            "묶음 빈칸은 일반 공백과 같은 측정 갈래를 사용해야 함: {record:?}"
        );
        assert!(
            record["layoutMetric"]["finalAdvanceHwpunit"]
                .as_i64()
                .is_some_and(|advance| advance > 0),
            "묶음 빈칸은 실제 전진폭을 가져야 함: {record:?}"
        );
    }
}
