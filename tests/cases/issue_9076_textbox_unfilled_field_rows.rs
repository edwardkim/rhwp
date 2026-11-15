//! 관계형·고정형 회귀: 글상자 안 글자 없는 문단에 미기입 누름틀이 있어도 글자처럼 취급 표는 한/글
//! 인쇄처럼 한 줄에 하나씩 글상자 안에 놓인다.
//! 주석 갱신: 2026-11-16T03:27:25+09:00 (Asia/Seoul).
//! 직전 파일 변경: 2026-11-15T05:28:15+09:00, commit 3a2fcbf7d12d4f57fa3fcff33721ff7ed9059738.
//!
//! [#9076] 한/글은 미기입 누름틀의 안내문을 문단 글자로 저장하고 저장 줄(LINE_SEG)도 그 글자를
//! 넣고 나눈다. 적재할 때 안내문은 값에서 빠지고 `Field::guide_residue` 로만 남으므로, 그
//! 저장 줄은 지금 문단의 배치가 아니다. 한/글 인쇄도 안내문 없이 줄을 다시 나눈다(#9005).
//!
//! # 무엇이 깨져 있었나
//!
//! 글상자 배치는 개체의 줄을 저장 줄 시작과 컨트롤의 스트림 위치를 비교해 정한다(#9021).
//! 안내문이 든 문단의 저장 줄 시작 `[0, 17, 25]` 은 안내문 1유닛과 누름틀 끝 8유닛을 포함한
//! 값이고 컨트롤 위치는 `[0, 8, 16]` 이라, 두 표가 모두 첫 줄(안내문만 있던 줄)로 판정됐다.
//! 표는 글상자 폭에 가까워서 둘째 표가 글상자 오른쪽 끝에서 시작해 밖으로 나갔다.
//!
//! # 기대값의 출처
//!
//! 두 문서 모두 한/글 2022(12.0.0.535)에서 직접 만들었다.
//! - `textbox_unfilled_field_tac_tables.hwp`: 빈 문서에 글상자를 그리고 그 안 한 문단에
//!   누름틀(안내문 「.」)을 넣은 뒤 Enter 없이 표 2개(각 165.4mm, 글상자 169.4mm)를 넣었다.
//!   원시 PARA_TEXT 는 `누름틀 시작 0 · 안내문 8 · 누름틀 끝 9 · 표1 17 · 표2 25`, 저장 줄은 3개다.
//! - `textbox_unfilled_field_tac_tables_nofield.hwp`: 위 문서에서 누름틀만 지워 다시 저장했다.
//!   저장 줄은 2개(`[0, 8]`)다.
//!
//! 같은 한/글에서 인쇄(Microsoft Print to PDF)한 `pdf/issue9076/` 의 두 PDF 는 표 괘선 선분
//! 20개가 서로 완전히 같다. 96dpi 로 본 표 위 괘선 y 는 표1 469.92px, 표2 536.64px, 왼쪽
//! 괘선 x 는 둘 다 125.12px 다. 한/글은 누름틀 유무와 상관없이 두 문서를 똑같이 인쇄한다.
//!
//! # 검사하는 것
//!
//! fixture 계약(안내문 잔재와 저장 줄 수), 둘째 표가 첫째 표 아래 줄에서 같은 x 로 시작하고
//! 두 표가 글상자 안에 있음, 누름틀 유무로 표 위치가 달라지지 않음(관계 검사).
//! 글상자 탐색 시 전체 1쪽을 고정하고, 절대 좌표는 고정하지 않는다. 실제 위치는 독립 PDF 와의 Visual Sweep 으로 확인한다.
//!
//! # 측정
//!
//! 2026-11-16, `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96`, 판정값
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣). 두 문서 모두 전체 1쪽 =
//! PDF 1쪽이고, 인쇄되는 글자가 없어 글꼴이 결과에 관여하지 않는다.
//! - 누름틀 있음: Native 100.00 / fresh WASM 100.00 (수정 전 devel `7ee4fe0327`: 29.43 / 29.43)
//! - 누름틀 없음: Native 100.00 / fresh WASM 100.00 (수정 전과 같음)
//!
//! 증거: `mydocs/pr/assets/johndoekim-20261116/`(전 쪽 TSV, review·overlay PNG).

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const FIELD: &str = "samples/issue9076/textbox_unfilled_field_tac_tables.hwp";
const NOFIELD: &str = "samples/issue9076/textbox_unfilled_field_tac_tables_nofield.hwp";
/// 관계 판정의 부동소수 허용 오차(px).
const EPS: f64 = 0.5;

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

impl Rect {
    fn of(node: &RenderNode) -> Self {
        Self {
            x: node.bbox.x,
            y: node.bbox.y,
            w: node.bbox.width,
            h: node.bbox.height,
        }
    }
    fn right(&self) -> f64 {
        self.x + self.w
    }
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
}

fn load(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("재현물 읽기 {sample}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 문서의 글상자 안 첫 문단(표가 든 문단).
fn textbox_paragraph(doc: &DocumentCore) -> Paragraph {
    doc.document().sections[0]
        .paragraphs
        .iter()
        .flat_map(|para| para.controls.iter())
        .find_map(|control| match control {
            Control::Shape(shape) => shape.drawing()?.text_box.as_ref()?.paragraphs.first(),
            _ => None,
        })
        .expect("글상자가 없다 — 시험 설정 오류")
        .clone()
}

/// (글상자, 그 글상자가 직접 가진 표들을 문단 내 컨트롤 순서로).
fn find_textbox(node: &RenderNode, best: &mut Option<(Rect, Vec<Rect>)>) {
    if matches!(node.node_type, RenderNodeType::TextBox) {
        let mut tables: Vec<(usize, Rect)> = node
            .children
            .iter()
            .filter_map(|child| match &child.node_type {
                RenderNodeType::Table(table) => Some((table.control_index?, Rect::of(child))),
                _ => None,
            })
            .collect();
        tables.sort_by_key(|(control_index, _)| *control_index);
        if best.as_ref().is_none_or(|(_, t)| tables.len() > t.len()) {
            *best = Some((
                Rect::of(node),
                tables.into_iter().map(|(_, rect)| rect).collect(),
            ));
        }
    }
    for child in &node.children {
        find_textbox(child, best);
    }
}

/// 글상자와 그 안의 표1·표2.
fn textbox_tables(sample: &str) -> (Rect, [Rect; 2]) {
    let doc = load(sample);
    assert_eq!(doc.page_count(), 1, "{sample}: 한/글 인쇄와 같이 1쪽이다");
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let mut best = None;
    find_textbox(&tree.root, &mut best);
    let (textbox, tables) = best.expect("표가 든 글상자가 없다 — 시험 설정 오류");
    let tables: [Rect; 2] = tables
        .try_into()
        .unwrap_or_else(|tables| panic!("{sample}: 글상자 안 표는 2개여야 한다: {tables:?}"));
    (textbox, tables)
}

fn tac_table_count(para: &Paragraph) -> usize {
    para.controls
        .iter()
        .filter(|control| matches!(control, Control::Table(table) if table.common.treat_as_char))
        .count()
}

fn guide_residue_len(para: &Paragraph) -> usize {
    para.controls
        .iter()
        .filter_map(|control| match control {
            Control::Field(field) => field.guide_residue.as_ref(),
            _ => None,
        })
        .map(|residue| residue.text.chars().count())
        .sum()
}

/// 두 재현물이 이 검사가 전제하는 구성인지 먼저 확인한다.
#[test]
fn fixtures_differ_only_by_the_unfilled_field() {
    let with_field = textbox_paragraph(&load(FIELD));
    assert!(with_field.text.is_empty(), "글자 없는 문단이다");
    assert_eq!(tac_table_count(&with_field), 2);
    assert_eq!(
        guide_residue_len(&with_field),
        1,
        "미기입 누름틀의 안내문 「.」 이 잔재로 남아 있다"
    );
    assert_eq!(
        with_field.line_segs.len(),
        3,
        "한/글은 안내문 줄 · 표1 줄 · 표2 줄로 저장했다"
    );

    let without_field = textbox_paragraph(&load(NOFIELD));
    assert!(without_field.text.is_empty());
    assert_eq!(tac_table_count(&without_field), 2);
    assert_eq!(guide_residue_len(&without_field), 0);
    assert_eq!(without_field.line_segs.len(), 2, "표1 줄 · 표2 줄");
}

#[test]
fn second_table_starts_a_new_row_inside_the_textbox() {
    let (textbox, [first, second]) = textbox_tables(FIELD);
    assert!(
        second.y + EPS >= first.bottom(),
        "표2 는 표1 아래 줄에서 시작한다 — 한/글 인쇄는 두 표를 위아래로 놓는다: \
         {first:?} / {second:?}"
    );
    assert!(
        (second.x - first.x).abs() <= EPS,
        "표2 는 표1 과 같은 줄 시작 x 에서 시작한다: {first:?} / {second:?}"
    );
    for (index, table) in [first, second].iter().enumerate() {
        assert!(
            table.x + EPS >= textbox.x
                && table.right() <= textbox.right() + EPS
                && table.y + EPS >= textbox.y
                && table.bottom() <= textbox.bottom() + EPS,
            "표{} 가 글상자 밖으로 나간다: 상자 {textbox:?}, 표 {table:?}",
            index + 1
        );
    }
}

/// 한/글은 두 문서를 똑같이 인쇄한다(표 괘선 선분 20개 일치). 미기입 누름틀이 표를 옮기면 안 된다.
#[test]
fn unfilled_field_does_not_move_the_tables() {
    let (_, with_field) = textbox_tables(FIELD);
    let (_, without_field) = textbox_tables(NOFIELD);
    for (index, (a, b)) in with_field.iter().zip(&without_field).enumerate() {
        assert!(
            (a.x - b.x).abs() <= 1.0 && (a.y - b.y).abs() <= 1.0,
            "표{} 위치가 누름틀 유무로 달라진다: 누름틀 있음 {a:?}, 없음 {b:?}",
            index + 1
        );
    }
}
