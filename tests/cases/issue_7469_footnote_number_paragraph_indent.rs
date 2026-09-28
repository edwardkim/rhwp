//! [#7469] 각주 번호가 붙은 첫 문단도 문단 모양의 내어쓰기를 받는다.
//!
//! 각주 문단은 번호 자리에 autoNum 컨트롤을 두고 문단 모양(내어쓰기·정렬)을 그대로 쓴다.
//! 종전 rhwp 는 번호가 붙은 첫 문단만 전용 경로에서 모든 줄을 영역 왼쪽 끝에 그리고, 번호
//! 서식의 끝 공백을 자리표시 공백 앞에 덧붙였다. 그래서 같은 각주의 둘째 문단은 내어쓰기를
//! 받는데 번호 문단의 둘째 줄부터는 받지 못했고, 번호 뒤 간격이 공백 세 칸으로 벌어졌다.
//!
//! 기대값은 구현과 독립이다.
//! - 문단 모양: 각주 문단(paraPr 3) 내어쓰기 −1310HU = 17.47px(96dpi).
//! - 한/글 2024 출력 `pdf/정책연구용역사업 …-hwp-2024.pdf` 67쪽: 각주 78 첫 줄 x=94.6,
//!   둘째 줄 x=112.0(차 17.4), 본문 `CFR` 은 `78)` 뒤 공백 한 칸.

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const POLICY: &str =
    "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwp";
/// 한/글 정본과 쪽수(215)가 같은 원본의 67쪽.
const POLICY_PAGE: u32 = 66;
const EMPTY_NOTES: &str = "samples/task1725/text_footnote_tail_overpagination.hwp";
/// 내용 없이 번호만 있는 각주 46·47 이 놓이는 쪽.
const EMPTY_NOTES_PAGE: u32 = 155;
/// 각주 문단 모양의 내어쓰기 1310HU 를 96dpi px 로 바꾼 값.
const HANGING_INDENT_PX: f64 = 1310.0 * 96.0 / 7200.0;

struct NoteLine {
    x: f64,
    y: f64,
    height: f64,
    text: String,
}

fn core(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 각주 영역의 글줄을 그려진 순서대로 모은다. 글자는 표시 문자열(`display_text`)로 잇는다.
fn footnote_lines(core: &DocumentCore, page: u32) -> Vec<NoteLine> {
    fn walk(node: &RenderNode, in_footnote: bool, out: &mut Vec<NoteLine>) {
        let in_footnote = in_footnote || matches!(node.node_type, RenderNodeType::FootnoteArea);
        if in_footnote && matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let runs: Vec<&RenderNode> = node
                .children
                .iter()
                .filter(|child| matches!(child.node_type, RenderNodeType::TextRun(_)))
                .collect();
            let text = runs
                .iter()
                .map(|child| match &child.node_type {
                    RenderNodeType::TextRun(run) => {
                        run.display_text.clone().unwrap_or_else(|| run.text.clone())
                    }
                    _ => String::new(),
                })
                .collect();
            out.push(NoteLine {
                x: runs.first().map_or(node.bbox.x, |run| run.bbox.x),
                y: node.bbox.y,
                height: node.bbox.height,
                text,
            });
        }
        for child in &node.children {
            walk(child, in_footnote, out);
        }
    }
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|e| panic!("{}쪽 render tree: {e:?}", page + 1));
    let mut out = Vec::new();
    walk(&tree.root, false, &mut out);
    assert!(!out.is_empty(), "{}쪽에 각주 글줄이 없다", page + 1);
    out
}

fn line_starting_with<'a>(lines: &'a [NoteLine], prefix: &str) -> (usize, &'a NoteLine) {
    lines
        .iter()
        .enumerate()
        .find(|(_, line)| line.text.starts_with(prefix))
        .unwrap_or_else(|| {
            let texts: Vec<&str> = lines.iter().map(|line| line.text.as_str()).collect();
            panic!("`{prefix}` 로 시작하는 각주 글줄이 없다: {texts:?}")
        })
}

/// 번호 문단의 둘째 줄은 내어쓰기만큼 들어간다(수정 전: 첫 줄과 같은 x).
#[test]
fn numbered_footnote_paragraph_continuation_uses_hanging_indent() {
    let lines = footnote_lines(&core(POLICY), POLICY_PAGE);
    let (index, first) = line_starting_with(&lines, "78)");
    let second = &lines[index + 1];
    let indent = second.x - first.x;
    assert!(
        (indent - HANGING_INDENT_PX).abs() <= 0.5,
        "각주 78 둘째 줄 들여쓰기 {indent:.2}px — 문단 모양 내어쓰기 {HANGING_INDENT_PX:.2}px \
         (한/글 정본 94.6 → 112.0) 와 달라야 할 이유가 없다. 첫 줄 x={:.1}, 둘째 줄 x={:.1} `{}`",
        first.x,
        second.x,
        second.text
    );
}

/// 번호는 autoNum 자리표시 위에 놓여 본문과 공백 한 칸으로 떨어진다(수정 전: 세 칸).
#[test]
fn footnote_number_replaces_the_autonum_placeholder() {
    let lines = footnote_lines(&core(POLICY), POLICY_PAGE);
    let (_, first) = line_starting_with(&lines, "78)");
    assert!(
        first.text.starts_with("78) CFR"),
        "각주 78 첫 줄이 `78) CFR` 로 시작해야 한다(한/글 정본). 실제: {:?}",
        first.text
    );
}

/// 대조군 — 번호가 없는 둘째 문단(`출처: …`)은 종전에도 내어쓰기를 받았고 그대로다.
#[test]
fn unnumbered_footnote_paragraph_keeps_hanging_indent() {
    let lines = footnote_lines(&core(POLICY), POLICY_PAGE);
    let (index, first) = lines
        .iter()
        .enumerate()
        .find(|(index, line)| {
            line.text.starts_with("출처: U.S.")
                && lines
                    .get(index + 1)
                    .is_some_and(|next| next.text.starts_with("https://"))
        })
        .expect("둘째 줄이 이어지는 `출처: U.S.` 문단");
    let indent = lines[index + 1].x - first.x;
    assert!(
        (indent - HANGING_INDENT_PX).abs() <= 0.5,
        "번호 없는 각주 문단의 둘째 줄 들여쓰기 {indent:.2}px ≠ {HANGING_INDENT_PX:.2}px"
    );
}

/// 대조군 — 내용 없이 번호만 있는 각주는 종전 번호 경로를 그대로 쓴다. 번호가 남고 줄
/// 상자가 내용 있는 각주와 같아야 한다(번호를 기본 글자 모양으로 붙이면 줄이 커진다).
#[test]
fn empty_footnote_keeps_its_number_and_line_box() {
    let lines = footnote_lines(&core(EMPTY_NOTES), EMPTY_NOTES_PAGE);
    let (_, empty) = line_starting_with(&lines, "46)");
    let (_, filled) = line_starting_with(&lines, "48)");
    assert_eq!(
        empty.text.trim_end(),
        "46)",
        "빈 각주 46 의 번호가 사라지거나 바뀌었다"
    );
    assert!(
        (empty.height - filled.height).abs() <= 0.01,
        "빈 각주 46 줄 높이 {:.2} ≠ 내용 있는 각주 48 줄 높이 {:.2}",
        empty.height,
        filled.height
    );
    let (_, next_empty) = line_starting_with(&lines, "47)");
    assert!(
        next_empty.y > empty.y,
        "빈 각주 47 이 46 아래에 놓여야 한다 (46 y={:.1}, 47 y={:.1})",
        empty.y,
        next_empty.y
    );
}
