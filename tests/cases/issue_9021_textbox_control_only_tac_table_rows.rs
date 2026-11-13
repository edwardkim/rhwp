//! 관계형 회귀: 글자 없는 글상자 문단에서 글자처럼 취급 표의 줄 소속·공유 기준선·글상자 포함.
//! 주석 갱신: 2026-11-13T03:50:57+09:00; 새 파일이라 직전 파일 커밋 없음.
//! 2026-11-13 측정: Visual Sweep raw, 96dpi, PDFium, 색상 경계 보정 전 2px 내용 실루엣.
//! textbox_tac_tables_two_lines.hwp 전 1쪽(1쪽): Native 100.00000% / fresh WASM 100.00000%; 각 전쪽 최저 동일.
//! 근거: mydocs/pr/assets/issue_9021_textbox_tac_tables/README.md 와 같은 폴더의 TSV·PNG; 독립 PDF는 pdf/issue9021/.
//!
//! [#9021] 글상자 안 글자 없는 문단 하나에 글자처럼 취급 표 3개가 있고 한/글이 두 줄로
//! 저장한 문서에서, 세 표가 한 줄에 이어 붙어 셋째 표가 글상자 오른쪽 밖으로 나가던 결함의
//! 회귀 가드.
//!
//! # 무엇이 깨져 있었나
//!
//! 글상자 배치는 개체의 줄을 조합 줄의 `char_start`(글자 번호)로 정했다. 글자 없는 문단은
//! 저장 줄 시작(`text_start` 0·16, 스트림 위치)이 글자 번호로는 모두 0 이 되어 줄들이
//! 겹치고, 세 표가 같은 줄로 판정돼 첫 줄 하나에 x 를 누적했다. 같은 문단의 글줄 방출은
//! 이미 저장 줄대로 두 줄을 내고 있어, 글줄과 개체의 줄 소속이 갈라져 있었다.
//!
//! 또 글상자 배치는 같은 줄의 표를 모두 줄 상단에 붙였다. 한/글은 줄 높이를 정한 표의
//! 기준선에 나머지 표를 앉히므로, 낮은 표2 가 한/글보다 3.2px 위에 그려졌다.
//!
//! # 기대값의 출처
//!
//! 재현물은 한/글 2022(12.0.0.535)에서 직접 만든 문서다. 빈 문서에 글상자 하나를 넣고 그 안
//! 한 문단에 Enter 없이 표 3개(표1·표2 는 좁게, 표3 은 상자 폭에 가깝게)를 넣었다. 같은
//! 파일을 같은 한/글에서 인쇄한 PDF(`pdf/issue9021/`)에서 표1·표2 는 첫 줄에 나란히, 표3 은
//! 그 아래 둘째 줄에 있고 세 표 모두 글상자 안에 있다. 96dpi 로 본 괘선 y 는 표1 540–561px,
//! 표2 543.5–561px, 표3 576.5–594px 다.
//!
//! # 검사하는 것
//!
//! 표1·표2 가 같은 줄, 낮은 표2 가 공유 기준선에 앉음, 표3 이 그 아래 줄, 세 표의 글상자
//! 포함과 서로 겹치지 않음(관계 검사).
//! 절대 좌표는 고정하지 않는다. 실제 위치·모양은 독립 PDF 와의 Visual Sweep 으로 확인한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue9021/textbox_tac_tables_two_lines.hwp";
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
    fn overlaps(&self, other: &Rect) -> bool {
        self.x + EPS < other.right()
            && other.x + EPS < self.right()
            && self.y + EPS < other.bottom()
            && other.y + EPS < self.bottom()
    }
}

/// (글상자, 그 글상자가 직접 가진 표들을 문단 내 컨트롤 순서로) — 표가 가장 많은 글상자.
fn table_textbox(node: &RenderNode, best: &mut Option<(Rect, Vec<Rect>)>) {
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
        table_textbox(child, best);
    }
}

/// 글상자와 그 안의 표1·표2·표3.
fn load_tables() -> (Rect, [Rect; 3]) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    let doc = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(doc.page_count(), 1, "재현물은 한/글 인쇄와 같이 1쪽이다");
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let mut best = None;
    table_textbox(&tree.root, &mut best);
    let (textbox, tables) = best.expect("표가 든 글상자가 없다 — 시험 설정 오류");
    let tables: [Rect; 3] = tables
        .try_into()
        .unwrap_or_else(|tables| panic!("글상자 안 표는 3개여야 한다: {tables:?}"));
    (textbox, tables)
}

#[test]
fn first_two_tables_share_the_first_row() {
    let (_, [first, second, _]) = load_tables();
    assert!(
        first.y + EPS < second.bottom() && second.y + EPS < first.bottom(),
        "표1·표2 는 저장 첫 줄에 함께 있다(세로 범위가 겹친다): {first:?} / {second:?}"
    );
    assert!(
        first.right() <= second.x + EPS,
        "같은 줄에서 표2 는 표1 의 오른쪽에 온다: {first:?} / {second:?}"
    );
}

/// 한/글 인쇄에서 첫 줄의 두 표는 윗변이 아니라 기준선을 공유한다. 줄 높이를 정한 표1
/// (괘선 y 540–561px)보다 낮은 표2(괘선 y 543.5–561px)는 그만큼 아래에서 시작하고, 두 표의
/// 아랫변은 1px 안에서 만난다.
#[test]
fn shorter_table_sits_on_the_shared_baseline() {
    let (_, [first, second, _]) = load_tables();
    assert!(
        first.h > second.h + 2.0,
        "재현물의 표1 은 표2 보다 높다 — 시험 설정 오류: {first:?} / {second:?}"
    );
    assert!(
        second.y >= first.y + 2.0,
        "낮은 표2 는 표1 과 윗변을 맞추지 않고 기준선에 앉아 더 아래에서 시작한다: \
         {first:?} / {second:?}"
    );
    assert!(
        (first.bottom() - second.bottom()).abs() <= 1.5,
        "같은 줄 두 표의 아랫변은 높이 차의 일부만큼만 벌어진다: {first:?} / {second:?}"
    );
}

#[test]
fn third_table_starts_on_the_row_below() {
    let (_, [first, second, third]) = load_tables();
    let first_row_bottom = first.bottom().max(second.bottom());
    assert!(
        third.y + EPS >= first_row_bottom,
        "표3 은 저장 둘째 줄(스트림 위치 16)의 개체다 — 첫 줄 표들 아래에서 시작해야 한다. \
         첫 줄 아랫변 {first_row_bottom:.1}, 표3 {third:?}"
    );
}

#[test]
fn tables_stay_inside_the_textbox() {
    let (textbox, tables) = load_tables();
    for (index, table) in tables.iter().enumerate() {
        assert!(
            table.x + EPS >= textbox.x
                && table.right() <= textbox.right() + EPS
                && table.y + EPS >= textbox.y
                && table.bottom() <= textbox.bottom() + EPS,
            "표{} 가 글상자 밖으로 나간다 — 둘째 줄 개체를 첫 줄 끝에 이어 붙였다. \
             상자 {textbox:?}, 표 {table:?}",
            index + 1
        );
    }
}

#[test]
fn tables_do_not_overlap() {
    let (_, tables) = load_tables();
    for (i, a) in tables.iter().enumerate() {
        for b in &tables[i + 1..] {
            assert!(!a.overlaps(b), "표끼리 겹친다: {a:?} / {b:?}");
        }
    }
}
