//! [#8080] 저장 줄이 없는 문서에서 본문 위에 떠 있는 작은 표(글앞/글뒤)가 제목·본문 표와 겹치던
//! 결함과, 그 수정이 함께 막아야 하는 부작용 세 가지의 회귀 가드.
//!
//! # 무엇이 깨져 있었나
//!
//! 저장 LINE_SEG 가 없는 문서의 구역 첫머리에는 글자 없는 빈 문단 하나가 오른쪽 위 작은
//! 글앞 표를 소유하고, 그 뒤에 제목 문단과 본문 표가 온다. rhwp 는 이 빈 host 문단의 줄 높이와
//! 작은 표의 바깥 여백을 한/글과 다르게 잡아, 본문 표가 작은 표 아래가 아니라 작은 표와 같은
//! 높이에서 시작해 두 표가 겹쳤다.
//!
//! 이 수정이 빈 host 문단을 한/글처럼 다루면 다음 세 경로가 함께 바뀐다.
//!
//! - 셀 첫 문단이 소유한 글앞 중첩 표: 셀 높이를 글줄 높이로만 잡으면 중첩 표 마지막 행의
//!   글자가 셀 밖으로 넘친다.
//! - 양옆에 글앞 도형만 떠 있는 제목 문단: 실제 폭 재조판을 받지 못하고 45자 합성 줄바꿈으로
//!   두 줄이 되어 뒤 표가 다음 쪽으로 밀린다.
//! - 그림이 앵커된 빈 문단 뒤에 빈 문단이 이어지는 작은 표 칸: 행 예약 높이를 두 문단을
//!   쌓은 값으로 잡아 측정·그리기 높이보다 커지고, 그 차이만큼 뒤 본문 표가 밀린다.
//!
//! # 기대값의 출처
//!
//! 재현물 4개는 실문서(기관 서식, 저장 LINE_SEG 없음)를 글자 치환·그림 교체로 익명화한 것이다.
//! 같은 파일을 한컴 웹한글기안기로 연 인쇄 출력(PDF)에서:
//!
//! - 작은 표가 있는 문서는 5쪽이다. 1쪽 제목은 작은 표 왼쪽에 작은 표와 같은 높이로 놓이고,
//!   본문 표는 작은 표 아래에서 시작한다.
//! - 셀 안 중첩 표 문서에서 중첩 표의 마지막 행 글자는 행 안에 모두 보인다.
//! - 떠 있는 도형 사이 제목 문서의 4쪽 맨 위에는 제목이 한 줄로 두 도형 사이에 있고, 그 아래
//!   본문 표가 같은 쪽에 놓인다.
//! - 그림 칸 문서는 5쪽이다. 5쪽 제목은 작은 표 아래에, 본문 표는 제목 아래에 놓인다.
//!   작은 표 아래끝과 바로 아래 개체 사이 간격은 1쪽(본문 표)과 5쪽(제목)이 같다.
//!
//! 떠 있는 도형 사이 제목 문서는 3쪽의 다른 결함(본문 줄 간격) 때문에 3쪽 실루엣 일치율이
//! 90% 미만이라 쪽 수는 검사하지 않고 4쪽만 검사한다. 나머지 쪽은 모두 90% 이상이다.
//!
//! # 검사하는 것
//!
//! 쪽 수, 작은 표·제목 줄·본문 표의 위아래 순서와 겹침 여부, 중첩 표 글줄의 행·셀 포함,
//! 제목 줄 수와 도형과의 같은 띠 여부, 같은 문서 안 두 쪽의 작은 표 아래 간격 일치.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const OVERLAY_SAMPLE: &str = "samples/issue8080/overlay_table_beside_title.hwp";
const NESTED_SAMPLE: &str = "samples/issue8080/cell_nested_table_rows.hwp";
const TITLE_SAMPLE: &str = "samples/issue8080/title_between_floating_shapes.hwp";
const PICTURE_CELL_SAMPLE: &str = "samples/issue8080/picture_cell_row_height.hwp";

/// 1쪽 오른쪽 위 작은 글앞 표를 소유한 빈 host 문단.
const OVERLAY_HOST_PARA: usize = 0;
const OVERLAY_TITLE_PARA: usize = 1;
const OVERLAY_BODY_TABLE_PARA: usize = 2;
/// 셀 안에 글앞 중첩 표가 있는 본문 표.
const NESTED_OUTER_TABLE_PARA: usize = 3;
/// 4쪽 맨 위 제목(양옆 글앞 도형 2개를 소유).
const TITLE_PAGE: u32 = 3;
const TITLE_PARA: usize = 83;
const TITLE_TABLE_PARA: usize = 85;
/// 5쪽 작은 표·제목·본문 표.
const PICTURE_CELL_PAGE: u32 = 4;
const PICTURE_CELL_TABLE_PARA: usize = 129;
const PICTURE_CELL_TITLE_PARA: usize = 130;
const PICTURE_CELL_BODY_TABLE_PARA: usize = 131;
/// 같은 문서 1쪽의 작은 표와 바로 아래 본문 표.
const PICTURE_CELL_FIRST_SMALL_TABLE_PARA: usize = 0;
const PICTURE_CELL_FIRST_BODY_TABLE_PARA: usize = 1;

#[derive(Clone, Copy, Debug)]
struct Rect {
    y: f64,
    h: f64,
}

impl Rect {
    fn of(node: &RenderNode) -> Self {
        Self {
            y: node.bbox.y,
            h: node.bbox.height,
        }
    }
    fn bottom(&self) -> f64 {
        self.y + self.h
    }
    fn center_y(&self) -> f64 {
        self.y + self.h / 2.0
    }
}

fn load(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

/// 본문 직속 표(셀 안 표 제외).
fn top_level_table(node: &RenderNode, para: usize, in_table: bool) -> Option<Rect> {
    if let RenderNodeType::Table(t) = &node.node_type {
        if !in_table && t.para_index == Some(para) {
            return Some(Rect::of(node));
        }
    }
    let inside = in_table || matches!(node.node_type, RenderNodeType::Table(_));
    node.children
        .iter()
        .find_map(|child| top_level_table(child, para, inside))
}

fn top_level_table_node(node: &RenderNode, para: usize) -> Option<&RenderNode> {
    if let RenderNodeType::Table(t) = &node.node_type {
        return (t.para_index == Some(para)).then_some(node);
    }
    node.children
        .iter()
        .find_map(|child| top_level_table_node(child, para))
}

/// 단 직속 글줄(표·도형 안 글줄 제외) 중 `para` 문단의 것.
fn column_lines(node: &RenderNode, para: usize, out: &mut Vec<Rect>) {
    if matches!(node.node_type, RenderNodeType::Column(_)) {
        for child in &node.children {
            if let RenderNodeType::TextLine(tl) = &child.node_type {
                if tl.para_index == Some(para) {
                    out.push(Rect::of(child));
                }
            }
        }
        return;
    }
    for child in &node.children {
        column_lines(child, para, out);
    }
}

fn images_of_para(node: &RenderNode, para: usize, out: &mut Vec<Rect>) {
    if let RenderNodeType::Image(img) = &node.node_type {
        if img.para_index == Some(para) {
            out.push(Rect::of(node));
        }
    }
    for child in &node.children {
        images_of_para(child, para, out);
    }
}

fn body(node: &RenderNode) -> Option<Rect> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(Rect::of(node));
    }
    node.children.iter().find_map(body)
}

fn text_lines(node: &RenderNode, out: &mut Vec<Rect>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        out.push(Rect::of(node));
    }
    for child in &node.children {
        text_lines(child, out);
    }
}

/// 셀 안 첫 중첩 표와 그 중첩 표를 담은 셀.
fn nested_table_in_cell(
    node: &RenderNode,
    cell: Option<&RenderNode>,
) -> Option<(Rect, Rect, Vec<Rect>)> {
    let cell = if matches!(node.node_type, RenderNodeType::TableCell(_)) {
        Some(node)
    } else {
        cell
    };
    if let (RenderNodeType::Table(_), Some(c)) = (&node.node_type, cell) {
        let mut lines = Vec::new();
        text_lines(node, &mut lines);
        return Some((Rect::of(node), Rect::of(c), lines));
    }
    node.children
        .iter()
        .find_map(|child| nested_table_in_cell(child, cell))
}

#[test]
fn overlay_sample_keeps_five_pages() {
    assert_eq!(
        load(OVERLAY_SAMPLE).page_count(),
        5,
        "웹한글기안기 기준 5쪽"
    );
}

#[test]
fn body_table_starts_below_overlay_table_and_title_sits_beside_it() {
    let doc = load(OVERLAY_SAMPLE);
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let overlay = top_level_table(&tree.root, OVERLAY_HOST_PARA, false).expect("작은 글앞 표");
    let body_table =
        top_level_table(&tree.root, OVERLAY_BODY_TABLE_PARA, false).expect("1쪽 본문 표");
    let mut titles = Vec::new();
    column_lines(&tree.root, OVERLAY_TITLE_PARA, &mut titles);
    let title = titles.first().copied().expect("1쪽 제목 줄");
    assert!(
        body_table.y + 0.5 >= overlay.bottom(),
        "본문 표는 작은 글앞 표 아래에서 시작해야 한다: 작은 표 아래끝 {:.1}, 본문 표 위 {:.1} \
         — 빈 host 문단 줄 높이와 바깥 여백이 어긋나면 두 표가 겹친다",
        overlay.bottom(),
        body_table.y
    );
    assert!(
        title.center_y() >= overlay.y && title.center_y() <= overlay.bottom(),
        "제목 줄은 작은 표와 같은 높이 띠에 놓여야 한다: 작은 표 {overlay:?}, 제목 {title:?}"
    );
    assert!(
        title.bottom() <= body_table.y + 0.5,
        "제목 줄이 본문 표와 겹친다: 제목 {title:?}, 본문 표 {body_table:?}"
    );
}

#[test]
fn nested_overlay_table_rows_stay_inside_cell() {
    let doc = load(NESTED_SAMPLE);
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let outer = top_level_table_node(&tree.root, NESTED_OUTER_TABLE_PARA).expect("바깥 표");
    let (nested, cell, lines) = outer
        .children
        .iter()
        .find_map(|child| nested_table_in_cell(child, None))
        .expect("셀 안 중첩 표");
    assert!(
        nested.bottom() <= cell.bottom() + 0.5,
        "중첩 표가 셀 밖으로 나간다: 중첩 표 {nested:?}, 셀 {cell:?}"
    );
    let last = lines.iter().map(Rect::bottom).fold(f64::MIN, f64::max);
    assert!(
        last <= nested.bottom() + 0.5,
        "중첩 표 마지막 행 글자가 표 아래로 넘친다: 마지막 글줄 아래끝 {last:.1}, 중첩 표 아래끝 \
         {:.1} — 셀 높이가 첫 문단 글앞 중첩 표 바닥을 담지 못한다",
        nested.bottom()
    );
}

#[test]
fn title_between_floating_shapes_stays_one_line_above_table() {
    let doc = load(TITLE_SAMPLE);
    let tree = doc
        .build_page_render_tree(TITLE_PAGE)
        .expect("4쪽 렌더 트리");
    let mut titles = Vec::new();
    column_lines(&tree.root, TITLE_PARA, &mut titles);
    assert_eq!(
        titles.len(),
        1,
        "4쪽 제목은 한 줄이어야 한다 — 실제 폭 재조판 없이 45자 합성 줄바꿈을 쓰면 두 줄로 갈라진다. \
         제목 줄 {titles:?}"
    );
    let title = titles[0];
    let mut shapes = Vec::new();
    images_of_para(&tree.root, TITLE_PARA, &mut shapes);
    assert_eq!(
        shapes.len(),
        2,
        "제목 양옆 도형 2개가 4쪽에 있어야 한다: {shapes:?}"
    );
    for shape in &shapes {
        assert!(
            shape.y < title.bottom() && title.y < shape.bottom(),
            "도형과 제목 줄은 같은 띠에 있어야 한다: 도형 {shape:?}, 제목 {title:?}"
        );
    }
    let table = top_level_table(&tree.root, TITLE_TABLE_PARA, false).expect("4쪽 본문 표");
    let page_body = body(&tree.root).expect("본문 영역");
    assert!(
        table.y + 0.5 >= title.bottom(),
        "본문 표는 제목 아래에서 시작해야 한다: 제목 {title:?}, 표 {table:?}"
    );
    assert!(
        table.bottom() <= page_body.bottom() + 0.5,
        "본문 표가 4쪽 본문 영역 안에 다 들어가야 한다: 표 {table:?}, 본문 {page_body:?}"
    );
}

#[test]
fn picture_cell_sample_keeps_five_pages() {
    assert_eq!(
        load(PICTURE_CELL_SAMPLE).page_count(),
        5,
        "웹한글기안기 기준 5쪽 — 그림 칸 행 예약 높이가 커지면 본문 표가 6쪽으로 밀린다"
    );
}

#[test]
fn picture_cell_page_title_and_body_table_follow_small_table() {
    let doc = load(PICTURE_CELL_SAMPLE);
    let tree = doc
        .build_page_render_tree(PICTURE_CELL_PAGE)
        .expect("5쪽 렌더 트리");
    let small = top_level_table(&tree.root, PICTURE_CELL_TABLE_PARA, false).expect("5쪽 작은 표");
    let mut titles = Vec::new();
    column_lines(&tree.root, PICTURE_CELL_TITLE_PARA, &mut titles);
    let title = titles.first().copied().expect("5쪽 제목 줄");
    let body_table =
        top_level_table(&tree.root, PICTURE_CELL_BODY_TABLE_PARA, false).expect("5쪽 본문 표");
    let page_body = body(&tree.root).expect("본문 영역");
    assert!(
        title.y + 0.5 >= small.bottom(),
        "제목은 작은 표 아래에 놓여야 한다: 작은 표 {small:?}, 제목 {title:?}"
    );
    assert!(
        body_table.y + 0.5 >= title.bottom(),
        "본문 표는 제목 아래에서 시작해야 한다: 제목 {title:?}, 본문 표 {body_table:?}"
    );
    assert!(
        body_table.bottom() <= page_body.bottom() + 0.5,
        "본문 표가 5쪽 본문 영역 안에 다 들어가야 한다: 표 {body_table:?}, 본문 {page_body:?}"
    );
}

#[test]
fn picture_cell_row_does_not_widen_gap_below_small_table() {
    let doc = load(PICTURE_CELL_SAMPLE);
    let first = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let first_small = top_level_table(&first.root, PICTURE_CELL_FIRST_SMALL_TABLE_PARA, false)
        .expect("1쪽 작은 표");
    let first_body = top_level_table(&first.root, PICTURE_CELL_FIRST_BODY_TABLE_PARA, false)
        .expect("1쪽 본문 표");
    let last = doc
        .build_page_render_tree(PICTURE_CELL_PAGE)
        .expect("5쪽 렌더 트리");
    let last_small =
        top_level_table(&last.root, PICTURE_CELL_TABLE_PARA, false).expect("5쪽 작은 표");
    let mut titles = Vec::new();
    column_lines(&last.root, PICTURE_CELL_TITLE_PARA, &mut titles);
    let title = titles.first().copied().expect("5쪽 제목 줄");
    let first_gap = first_body.y - first_small.bottom();
    let last_gap = title.y - last_small.bottom();
    assert!(
        (last_gap - first_gap).abs() <= 1.0,
        "작은 표 아래 간격은 1쪽과 5쪽이 같아야 한다: 1쪽 {first_gap:.1}, 5쪽 {last_gap:.1} \
         — 그림이 앵커된 칸의 행 예약 높이를 문단을 쌓은 값으로 잡으면 5쪽 간격만 벌어진다"
    );
}
