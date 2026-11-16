//! 관계형·고정형 회귀: 누름틀 끝 표식 뒤에 오는 글자처럼 취급 표는 한/글이 저장한 줄에 놓인다.
//! 주석 갱신: 2026-11-16T22:20:14+09:00 (Asia/Seoul). 직전 파일 변경: 2026-11-16T21:07:36+09:00,
//! commit 8348b7ce20.
//!
//! 2026-11-16 측정 raw(96dpi, `scripts/visual_sweep.py --silhouette-only --compat 2022`, 판정값
//! `silhouette_raw_match_percent`). 세 문서 모두 전체 1쪽 = PDF 1쪽이라 1쪽 값이 전쪽 최저다.
//! - 빈 누름틀: Native 100.00% / fresh WASM 100.00% (수정 전 devel `443dafb081`: 64.87 / 64.87)
//! - 값 4자: Native 100.00% / fresh WASM 100.00% (수정 전 58.36 / 58.42)
//! - 값 1자: Native 100.00% / fresh WASM 100.00% (수정 전 57.23 / 57.24)
//!
//! 근거: `mydocs/pr/assets/issue_9058_field_end_slot_object_rows/README.md`(전 쪽 TSV, review·overlay PNG).
//!
//! [#9058] 누름틀 끝 표식(FIELD_END)은 `controls[]` 에 들지 않지만 문단 스트림에서 8유닛을
//! 차지한다. 한/글이 저장한 줄 시작(LINE_SEG `text_start`)은 그 8유닛을 센 값이다.
//!
//! # 무엇이 깨져 있었나
//!
//! 개체의 줄은 저장 줄 시작과 컨트롤의 스트림 위치를 비교해 정한다. 컨트롤 위치를 셀 때
//! 짝 있는 끝 표식을 빼먹어, 끝 표식 뒤 개체가 8유닛씩 앞에 있는 것으로 계산됐다.
//!
//! - 글자 없는 문단(빈 누름틀 + 표 2개): 위치 `[0, 8, 16]`, 저장 줄 `[0, 24]` → 두 표가 모두
//!   첫 줄. 둘째 표가 글상자 오른쪽 밖으로 나갔다.
//! - 글자 있는 문단(값이 든 누름틀 + 표 2개): 값 4자는 위치 `[0, 12, 20]`, 저장 줄
//!   `[0, 20, 28]` → 표1 이 글자 줄(값 오른쪽)에, 표2 가 표1 의 줄에 놓였다. 두 표가 한 줄씩
//!   앞이었다. 값 1자도 같다(위치 `[0, 9, 17]`, 저장 줄 `[0, 17, 25]`).
//!
//! # 기대값의 출처
//!
//! 세 문서 모두 한/글 2022(12.0.0.535)에서 직접 만들었다. 글상자 한 문단에 누름틀과 글자처럼
//! 취급 표 2개(각 165.4mm, 글상자 169.4mm)가 있다(#9076 재현물과 같은 구성). #9058 에 첨부한
//! 원본을 메인테이너가 `mydocs/pr/assets/johndoekim-20261116/issue9058/` 에 보존했고, 값 1자
//! 문서는 같은 바이트의 `samples/issue9082/` 를 쓴다.
//! - `field_end_slot_textbox_with_cursor.hwp`: 누름틀 안에 커서를 둔 채 저장했다. 안내문 글자가
//!   없다. 원시 PARA_TEXT 는 `누름틀 시작 0 · 누름틀 끝 8 · 표1 16 · 표2 24`, 저장 줄 `[0, 24]`.
//! - `field_end_slot_textbox_with_text.hwp`: 누름틀에 「가나다라」를 입력해 저장했다. 원시
//!   PARA_TEXT 는 `누름틀 시작 0 · 가나다라 8–11 · 누름틀 끝 12 · 표1 20 · 표2 28`, 저장 줄
//!   `[0, 20, 28]`.
//! - `samples/issue9082/textbox_filled_field_tac_tables.hwp`: 누름틀에 「값」을 입력해 저장했다.
//!   원시 PARA_TEXT 는 `누름틀 시작 0 · 값 8 · 누름틀 끝 9 · 표1 17 · 표2 25`, 저장 줄 `[0, 17, 25]`.
//!
//! 같은 한/글의 인쇄(Microsoft Print to PDF, 위 `issue9058/` 의 `*-hwp2022-print.pdf`)에서 96dpi
//! 로 본 표 위 괘선 y 는 빈 누름틀 문서가 표1 469.92px · 표2 536.64px, 값이 든 두 문서가 모두
//! 표1 480.48px · 표2 547.36px 이고 글자 줄은 그 위(466.72px)다. 표 왼쪽 괘선 x 는 모두
//! 125.12px 다.
//!
//! # 검사하는 것
//!
//! - 고정형: 세 문서 모두 전체 1쪽(한/글 인쇄와 같음). fixture 계약은 원본에 저장된 값을
//!   모델에서 직접 읽어 시험 설정 오류를 거른다(끝 표식이 표 앞에 있고 저장 줄 시작이 끝 표식
//!   8유닛을 센 값).
//! - 관계형: 표가 저장 줄 순서대로 위아래에 놓이고 같은 x 에서 시작하며 글상자 안에 있음, 값이
//!   든 문서에서는 표1 이 글자 줄 아래에 있음. 절대 좌표는 고정하지 않는다. 실제 위치는 독립
//!   PDF 와의 Visual Sweep 으로 확인한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const EMPTY_FIELD: &str =
    "mydocs/pr/assets/johndoekim-20261116/issue9058/field_end_slot_textbox_with_cursor.hwp";
const FOUR_CHAR_VALUE: &str =
    "mydocs/pr/assets/johndoekim-20261116/issue9058/field_end_slot_textbox_with_text.hwp";
const ONE_CHAR_VALUE: &str = "samples/issue9082/textbox_filled_field_tac_tables.hwp";
/// 값이 든 누름틀 문서와 그 값.
const FILLED: [(&str, &str); 2] = [(FOUR_CHAR_VALUE, "가나다라"), (ONE_CHAR_VALUE, "값")];
/// 관계 판정의 부동소수 허용 오차(px).
const EPS: f64 = 0.5;
/// 확장 제어 문자 하나가 문단 스트림에서 차지하는 유닛 수.
const SLOT: u32 = 8;

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

/// 문서의 글상자 안 첫 문단(누름틀과 표가 든 문단).
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

/// 글상자와 그 안의 조판 결과.
struct TextBoxLayout {
    textbox: Rect,
    /// 글상자가 직접 가진 표 2개(문단 내 컨트롤 순서).
    tables: [Rect; 2],
    /// 글상자 첫 문단에서 글자가 그려진 줄.
    text_lines: Vec<Rect>,
}

fn find_textbox<'a>(node: &'a RenderNode, best: &mut Option<&'a RenderNode>) {
    if matches!(node.node_type, RenderNodeType::TextBox) {
        let tables = |n: &RenderNode| {
            n.children
                .iter()
                .filter(|child| matches!(child.node_type, RenderNodeType::Table(_)))
                .count()
        };
        if best.is_none_or(|found| tables(node) > tables(found)) {
            *best = Some(node);
        }
    }
    for child in &node.children {
        find_textbox(child, best);
    }
}

fn layout(sample: &str) -> TextBoxLayout {
    let doc = load(sample);
    assert_eq!(doc.page_count(), 1, "{sample}: 한/글 인쇄와 같이 1쪽이다");
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let mut found = None;
    find_textbox(&tree.root, &mut found);
    let node = found.expect("글상자가 없다 — 시험 설정 오류");

    let mut tables: Vec<(usize, Rect)> = node
        .children
        .iter()
        .filter_map(|child| match &child.node_type {
            RenderNodeType::Table(table) => Some((table.control_index?, Rect::of(child))),
            _ => None,
        })
        .collect();
    tables.sort_by_key(|(control_index, _)| *control_index);
    let tables: Vec<Rect> = tables.into_iter().map(|(_, rect)| rect).collect();
    let tables: [Rect; 2] = tables
        .try_into()
        .unwrap_or_else(|tables| panic!("{sample}: 글상자 안 표는 2개여야 한다: {tables:?}"));

    let text_lines = node
        .children
        .iter()
        .filter(|child| {
            matches!(child.node_type, RenderNodeType::TextLine(_))
                && child.children.iter().any(|run| {
                    matches!(&run.node_type, RenderNodeType::TextRun(run) if !run.text.trim().is_empty())
                })
        })
        .map(Rect::of)
        .collect();

    TextBoxLayout {
        textbox: Rect::of(node),
        tables,
        text_lines,
    }
}

fn assert_tables_stack_inside(sample: &str, found: &TextBoxLayout) {
    let [first, second] = found.tables;
    assert!(
        second.y + EPS >= first.bottom(),
        "{sample}: 표2 는 표1 아래 줄에서 시작한다: {first:?} / {second:?}"
    );
    assert!(
        (second.x - first.x).abs() <= EPS,
        "{sample}: 표1 과 표2 는 같은 줄 시작 x 에서 시작한다: {first:?} / {second:?}"
    );
    for (index, table) in found.tables.iter().enumerate() {
        assert!(
            table.x + EPS >= found.textbox.x
                && table.right() <= found.textbox.right() + EPS
                && table.y + EPS >= found.textbox.y
                && table.bottom() <= found.textbox.bottom() + EPS,
            "{sample}: 표{} 가 글상자 밖으로 나간다: 상자 {:?}, 표 {table:?}",
            index + 1,
            found.textbox
        );
    }
}

/// fixture 계약 — 시험 설정 오류를 먼저 걸러낸다. 제품의 위치 계산을 쓰지 않고 모델에서 직접 센다.
#[test]
fn fixtures_store_rows_that_count_the_field_end_slot() {
    let empty = textbox_paragraph(&load(EMPTY_FIELD));
    assert!(empty.text.is_empty(), "글자 없는 문단이다");
    assert!(
        matches!(empty.controls.as_slice(), [Control::Field(field), Control::Table(_), Control::Table(_)] if field.guide_residue.is_none()),
        "안내문 잔재 없는 누름틀 + 표 2개"
    );
    assert_eq!(empty.field_ranges.len(), 1);
    assert_eq!(
        empty.field_ranges[0].control_idx + empty.field_ranges[0].inner_slot_count,
        0
    );
    // 표2 는 셋째 컨트롤이고 그 앞에 끝 표식이 하나 있다: (2 + 1) × 8.
    let starts: Vec<u32> = empty.line_segs.iter().map(|seg| seg.text_start).collect();
    assert_eq!(starts, [0, (2 + 1) * SLOT]);

    for (sample, value) in FILLED {
        let para = textbox_paragraph(&load(sample));
        assert_eq!(para.text, value, "{sample}");
        assert!(
            matches!(
                para.controls.as_slice(),
                [Control::Field(_), Control::Table(_), Control::Table(_)]
            ),
            "{sample}: 누름틀 + 표 2개"
        );
        assert_eq!(para.field_ranges.len(), 1, "{sample}");
        let range = &para.field_ranges[0];
        let chars = value.chars().count();
        assert_eq!(
            (
                range.start_char_idx,
                range.end_char_idx,
                range.inner_slot_count
            ),
            (0, chars, 0),
            "{sample}: 누름틀은 값만 감싼다"
        );
        // 누름틀 시작 8 + 값 + 끝 표식 8 에서 표1, 그 8유닛 뒤에서 표2.
        let table1 = SLOT + chars as u32 + SLOT;
        let starts: Vec<u32> = para.line_segs.iter().map(|seg| seg.text_start).collect();
        assert_eq!(starts, [0, table1, table1 + SLOT], "{sample}");
    }
}

/// 결함 검출(글자 없는 문단) — 수정 전에는 표2 가 표1 옆, 글상자 밖에 놓인다.
#[test]
fn table_after_an_empty_field_end_starts_the_second_row() {
    let found = layout(EMPTY_FIELD);
    assert_tables_stack_inside(EMPTY_FIELD, &found);
}

/// 결함 검출(글자 있는 문단) — 수정 전에는 표1 이 글자 줄에, 표2 가 표1 의 줄에 놓인다.
#[test]
fn tables_after_a_filled_field_end_follow_the_text_row() {
    for (sample, value) in FILLED {
        let found = layout(sample);
        assert_eq!(
            found.text_lines.len(),
            1,
            "{sample}: 글자가 그려진 줄은 「{value}」 한 줄이다: {:?}",
            found.text_lines
        );
        let text_line = found.text_lines[0];
        let [first, _] = found.tables;
        assert!(
            first.y + EPS >= text_line.bottom(),
            "{sample}: 표1 은 글자 줄 아래에서 시작한다 — 한/글 인쇄는 글자 줄 · 표1 · 표2 세 \
             줄이다: 글자 줄 {text_line:?}, 표1 {first:?}"
        );
        assert_tables_stack_inside(sample, &found);
    }
}
