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
//! 이 수정이 빈 host 문단을 한/글처럼 다루면 다음 경로가 함께 바뀐다.
//!
//! - 셀 첫 문단이 소유한 글앞 중첩 표: 셀 높이를 글줄 높이로만 잡으면 중첩 표 마지막 행의
//!   글자가 셀 밖으로 넘친다.
//! - 양옆에 글앞 도형만 떠 있는 제목 문단: 실제 폭 재조판을 받지 못하고 45자 합성 줄바꿈으로
//!   두 줄이 되어 뒤 표가 다음 쪽으로 밀린다.
//! - 그림이 앵커된 빈 문단 뒤에 빈 문단이 이어지는 작은 표 칸: 행 예약 높이를 두 문단을
//!   쌓은 값으로 잡아 측정·그리기 높이보다 커지고, 그 차이만큼 뒤 본문 표가 밀린다.
//! - 글뒤 그림을 함께 가진 빈 문단의 자리차지 표: 바깥 여백을 빠뜨리고 host 빈 줄도 세지
//!   않아 뒤 흐름이 한 줄 짧아진다. 반대로 표만 가진 빈 어울림 host 뒤에 빈 줄을 더하면
//!   뒤 흐름이 한 줄 길어진다.
//!
//! # 기대값의 출처
//!
//! 재현물 4개는 실문서(기관 서식, 저장 LINE_SEG 없음)를 글자 치환·그림 교체로 익명화한 것이다.
//! 같은 파일을 한컴 웹한글기안기로 연 인쇄 출력(PDF)에서:
//!
//! - 작은 표가 있는 문서는 5쪽이다. 모든 쪽에 작은 표가 오른쪽 위에 있고 본문 표는 그 아래에서
//!   시작한다. 1쪽 제목은 작은 표 왼쪽에 작은 표와 같은 높이로 놓인다.
//! - 셀 안 중첩 표 문서에서 중첩 표의 마지막 행 글자는 행 안에 모두 보인다.
//! - 떠 있는 도형 사이 제목 문서는 4쪽이다. 2쪽 글뒤 그림을 가진 표 host 의 빈 줄이 표 아래에
//!   놓여 2쪽 마지막 빈 문단이 3쪽 첫 줄이 된다. 3쪽 어울림 표 뒤 안내 줄은 표 바로 아래에 온다.
//!   4쪽 맨 위에는 제목이 한 줄로 두 도형 사이에 있고, 그 아래 본문 표가 같은 쪽에 놓인다.
//! - 그림 칸 문서는 5쪽이다. 5쪽 제목은 작은 표 아래에, 본문 표는 제목 아래에 놓인다.
//!   작은 표 아래끝과 바로 아래 개체 사이 간격은 1쪽(본문 표)과 5쪽(제목)이 같다.
//!
//! 재현물 4개의 모든 쪽 실루엣 일치율은 90% 이상이다.
//!
//! # 검사하는 것
//!
//! 쪽 수, 작은 표·제목 줄·본문 표의 위아래 순서와 겹침 여부, 중첩 표 글줄의 행·셀 포함,
//! 제목 줄 수와 도형과의 같은 띠 여부, 같은 문서 안 두 쪽의 작은 표 아래 간격 일치,
//! 빈 표 host 뒤 문단의 쪽 소속과 표 아래 간격.

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
const DECORATED_HOST_PAGE: u32 = 1;
const DECORATED_HOST_PARA: usize = 43;
const PUSHED_EMPTY_PARA: usize = 52;
const SQUARE_HOST_PARA: usize = 67;
const SQUARE_CAPTION_PARA: usize = 68;
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

/// 본문 직속 표(셀 안 표 제외)의 (폭, 위치).
fn top_level_tables(node: &RenderNode, in_table: bool, out: &mut Vec<(f64, Rect)>) {
    if !in_table && matches!(node.node_type, RenderNodeType::Table(_)) {
        out.push((node.bbox.width, Rect::of(node)));
    }
    let inside = in_table || matches!(node.node_type, RenderNodeType::Table(_));
    for child in &node.children {
        top_level_tables(child, inside, out);
    }
}

#[test]
fn every_page_draws_its_overlay_table_above_body_table() {
    let doc = load(OVERLAY_SAMPLE);
    for page in 0..doc.page_count() {
        let tree = doc.build_page_render_tree(page).expect("렌더 트리");
        let mut tables = Vec::new();
        top_level_tables(&tree.root, false, &mut tables);
        tables.sort_by(|a, b| a.0.total_cmp(&b.0));
        assert_eq!(
            tables.len(),
            2,
            "{}쪽에는 작은 글앞 표와 본문 표가 하나씩 있어야 한다 — 빈 host 문단이 앞 쪽에 \
             남으면 작은 표가 앞 쪽 아래에 그려진다: {tables:?}",
            page + 1
        );
        let (overlay, body_table) = (tables[0].1, tables[1].1);
        assert!(
            body_table.y + 0.5 >= overlay.bottom(),
            "{}쪽 본문 표는 작은 글앞 표 아래에서 시작해야 한다: 작은 표 {overlay:?}, \
             본문 표 {body_table:?}",
            page + 1
        );
    }
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
fn title_sample_keeps_four_pages() {
    assert_eq!(
        load(TITLE_SAMPLE).page_count(),
        4,
        "웹한글기안기 기준 4쪽 — 글뒤 그림을 함께 가진 빈 표 host 의 빈 줄을 빠뜨리면 \
         3쪽이 한 줄 위로 당겨지고, 넣고 어울림 표 뒤에도 빈 줄을 더하면 4쪽 표가 5쪽으로 넘친다"
    );
}

#[test]
fn decorated_empty_table_host_line_pushes_last_empty_paragraph_to_next_page() {
    let doc = load(TITLE_SAMPLE);
    let page2 = doc
        .build_page_render_tree(DECORATED_HOST_PAGE)
        .expect("2쪽 렌더 트리");
    let page3 = doc
        .build_page_render_tree(DECORATED_HOST_PAGE + 1)
        .expect("3쪽 렌더 트리");
    let mut host = Vec::new();
    column_lines(&page2.root, DECORATED_HOST_PARA, &mut host);
    let table = top_level_table(&page2.root, DECORATED_HOST_PARA, false).expect("2쪽 표");
    assert!(
        host.iter().all(|line| line.y + 0.5 >= table.bottom()),
        "글뒤 그림을 함께 가진 빈 host 의 빈 줄은 표 위 틈(오프셋+위 여백)이 줄 높이보다 \
         좁아 표 아래에 놓여야 한다: 줄 {host:?}, 표 {table:?}"
    );
    let mut on_page2 = Vec::new();
    column_lines(&page2.root, PUSHED_EMPTY_PARA, &mut on_page2);
    let mut on_page3 = Vec::new();
    column_lines(&page3.root, PUSHED_EMPTY_PARA, &mut on_page3);
    assert!(
        on_page2.is_empty() && on_page3.len() == 1,
        "host 빈 줄만큼 밀린 2쪽 마지막 빈 문단은 3쪽 첫 줄이어야 한다: 2쪽 {on_page2:?}, 3쪽 {on_page3:?}"
    );
}

#[test]
fn table_only_square_host_adds_no_empty_line_below_table() {
    let doc = load(TITLE_SAMPLE);
    let tree = doc
        .build_page_render_tree(DECORATED_HOST_PAGE + 1)
        .expect("3쪽 렌더 트리");
    let table = top_level_table(&tree.root, SQUARE_HOST_PARA, false).expect("3쪽 어울림 표");
    let mut caption = Vec::new();
    column_lines(&tree.root, SQUARE_CAPTION_PARA, &mut caption);
    let first = *caption.first().expect("어울림 표 뒤 안내 줄");
    assert!(
        first.y + 0.5 >= table.bottom() && first.y - table.bottom() < first.h,
        "표만 가진 빈 어울림 host 는 자기 빈 줄이 없어 뒤 안내 줄이 표 바로 아래(바깥 여백만 사이)에 와야 한다: \
         표 {table:?}, 안내 줄 {first:?}"
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

// ── 글앞 도형 host 의 줄 소유는 앞 공백 수와 무관하다 ─────────────────────────────────
//
// 글앞/글뒤 도형은 종이층에 놓여 줄 폭에 중립이다. 저장 줄 없는 문단에서 이런 도형만 가진
// host 는 개체 없는 문단과 같이 실제 폭 프레임이 줄을 나눈다. 같은 폭·같은 글자 모양에서
// 앞 공백 수만 바꾼 세 변형(24·25·0개 + `가` 24자)은 모두 본문 폭 한 줄에 들어가므로,
// 줄 수·쪽 수·뒤 표의 쪽 소속이 같아야 한다. 공백 비율로 조판 경로를 고르면 24/25 사이에서
// 줄 수와 뒤 표의 쪽이 갈린다.

use rhwp::model::control::Control;

/// 4쪽 제목 문단(83)을 같은 문단 모양·글앞 도형 2개를 유지한 채 `앞 공백 n + 가×24` 단일
/// 글자 모양 글자로 바꾼다.
fn title_variant(leading_spaces: usize) -> DocumentCore {
    let mut doc = load(TITLE_SAMPLE);
    let mut model = doc.document().clone();
    let para = &mut model.sections[0].paragraphs[TITLE_PARA];
    let overlay_shapes = para
        .controls
        .iter()
        .filter(|c| matches!(c, Control::Shape(s) if !s.common().treat_as_char))
        .count();
    assert_eq!(overlay_shapes, 2, "제목 문단은 글앞 도형 2개를 가진다");
    let body_char_shape = para.char_shapes.last().expect("글자 모양").char_shape_id;
    let visible = para.text.chars().count();
    para.delete_text_at(0, visible);
    let text = format!("{}{}", " ".repeat(leading_spaces), "가".repeat(24));
    para.insert_text_at(0, &text);
    para.apply_char_shape_to_entire_text(body_char_shape);
    para.line_segs.clear();
    assert_eq!(para.text, text, "변형 제목 글자");
    assert_eq!(para.controls.len(), 2, "글앞 도형 2개 유지");
    doc.set_document(model);
    doc
}

/// 변형 제목이 놓인 쪽, 그 쪽의 제목 줄 수, 뒤 본문 표가 놓인 쪽.
fn title_variant_layout(leading_spaces: usize) -> (u32, usize, u32, u32) {
    let doc = title_variant(leading_spaces);
    let pages = doc.page_count();
    let mut title_page = None;
    let mut table_page = None;
    for page in 0..pages {
        let tree = doc.build_page_render_tree(page).expect("렌더 트리");
        let mut lines = Vec::new();
        column_lines(&tree.root, TITLE_PARA, &mut lines);
        if !lines.is_empty() && title_page.is_none() {
            title_page = Some((page, lines.len()));
        }
        if table_page.is_none() && top_level_table(&tree.root, TITLE_TABLE_PARA, false).is_some() {
            table_page = Some(page);
        }
    }
    let (page, lines) = title_page.expect("변형 제목 줄");
    (page, lines, table_page.expect("제목 뒤 본문 표"), pages)
}

#[test]
fn overlay_shape_title_line_ownership_ignores_leading_space_count() {
    let control = title_variant_layout(0);
    let (page, lines, table_page, pages) = control;
    assert_eq!(
        lines, 1,
        "앞 공백 없는 대조군: 24자는 본문 폭 한 줄 ({control:?})"
    );
    assert_eq!(
        table_page, page,
        "앞 공백 없는 대조군: 뒤 표는 제목과 같은 쪽 ({control:?})"
    );
    for leading in [24, 25] {
        let variant = title_variant_layout(leading);
        assert_eq!(
            variant, control,
            "앞 공백 {leading}개 변형의 (제목 쪽, 줄 수, 뒤 표 쪽, 전체 쪽수)는 앞 공백 없는 \
             대조군과 같아야 한다 — 글앞 도형은 줄 폭에 중립이고 두 변형 모두 한 줄에 들어간다"
        );
    }
    assert_eq!(pages, 4, "변형도 원본처럼 4쪽");
}

/// `host_text` 면 앞 host 에 글자를 넣는다. 앞 글뒤 표를 문단 기준 Top/Left·세로 오프셋 2000HU·위 여백 1500HU 로
/// 둔 뒤, 뒤 본문 표만 자리차지(비 TAC)·위 여백 `next_top` 으로 바꿔 HWPX 로 저장·재파싱한다.
/// `next_on_next_page` 면 뒤 본문 표 문단을 쪽 나누기로 다음 쪽에 보낸다.
fn anchor_variant(host_text: bool, next_top: i16, next_on_next_page: bool) -> DocumentCore {
    use rhwp::model::shape::{HorzAlign, TextWrap, VertAlign, VertRelTo};
    let mut doc = load(OVERLAY_SAMPLE);
    let mut model = doc.document().clone();
    let paras = &mut model.sections[0].paragraphs;
    if host_text {
        paras[OVERLAY_HOST_PARA].insert_text_at(0, "가");
    }
    let front = paras[OVERLAY_HOST_PARA]
        .controls
        .iter_mut()
        .find_map(|c| match c {
            Control::Table(t) if !t.common.treat_as_char => Some(t),
            _ => None,
        })
        .expect("앞 host 의 글뒤 표");
    front.common.vert_rel_to = VertRelTo::Para;
    front.common.vert_align = VertAlign::Top;
    front.common.vertical_offset = 2000;
    front.common.horz_align = HorzAlign::Left;
    front.outer_margin_top = 1500;
    let Control::Table(next) = &mut paras[OVERLAY_BODY_TABLE_PARA].controls[0] else {
        panic!("뒤 본문 표")
    };
    next.common.treat_as_char = false;
    next.common.text_wrap = TextWrap::TopAndBottom;
    next.outer_margin_top = next_top;
    doc.set_document(model);
    if next_on_next_page {
        doc.insert_page_break_native(0, OVERLAY_BODY_TABLE_PARA, 0)
            .expect("뒤 표 앞 쪽 나누기");
    }
    let bytes = doc.export_hwpx_native().expect("HWPX 저장");
    DocumentCore::from_bytes(&bytes).expect("HWPX 재파싱")
}

fn front_table_y(doc: &DocumentCore) -> f64 {
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    top_level_table(&tree.root, OVERLAY_HOST_PARA, false)
        .expect("1쪽 앞 글뒤 표")
        .y
}

/// 앞 표의 원점은 자기 host·자기 속성으로 정해진다. 문서 순서상 뒤 표의 위 여백이 바뀌어도
/// 앞 표가 움직이면 안 된다(같은 쪽, 뒤 표가 다음 쪽인 경우 모두).
#[test]
fn front_overlay_table_origin_ignores_following_table_top_margin() {
    for host_text in [false, true] {
        for next_on_next_page in [false, true] {
            let without = front_table_y(&anchor_variant(host_text, 0, next_on_next_page));
            let with = front_table_y(&anchor_variant(host_text, 1200, next_on_next_page));
            assert!(
                (without - with).abs() < 0.01,
                "뒤 표 위 여백 0/1200HU 에서 앞 표 y 가 같아야 한다 \
                 (host 글자={host_text}, 뒤 표 다음 쪽={next_on_next_page}): {without} vs {with}"
            );
        }
    }
}

/// 문단 기준 위 정렬 글뒤 표의 세로 오프셋은 표 괘선 상단을 가리킨다. host 에 글자가 있든
/// 없든 바깥여백 위는 원점에 더하지 않는다(같은 두 변형의 한컴 웹한글기안기 인쇄에서 앞 표
/// 괘선 상단이 같은 높이).
#[test]
fn front_overlay_table_top_does_not_depend_on_host_text() {
    let empty_host = front_table_y(&anchor_variant(false, 0, false));
    let text_host = front_table_y(&anchor_variant(true, 0, false));
    assert!(
        (empty_host - text_host).abs() < 0.01,
        "host 글자 유무와 관계없이 앞 표 y 가 같아야 한다: {empty_host} vs {text_host}"
    );
}

fn square_variant(v_off: u32, top: i16, h_off: u32, left: i16) -> DocumentCore {
    let doc = load(TITLE_SAMPLE);
    let mut model = doc.document().clone();
    let Control::Table(t) = &mut model.sections[0].paragraphs[SQUARE_HOST_PARA].controls[0] else {
        panic!("빈 host 의 어울림 표")
    };
    t.common.vertical_offset = v_off;
    t.common.horizontal_offset = h_off;
    t.outer_margin_top = top;
    t.outer_margin_left = left;
    let mut doc = doc;
    doc.set_document(model);
    let bytes = doc.export_hwpx_native().expect("HWPX 저장");
    DocumentCore::from_bytes(&bytes).expect("HWPX 재파싱")
}

/// (표 x, 표 위끝, 표 아래끝, 안내 줄 위끝)
fn square_geometry(doc: &DocumentCore) -> (f64, f64, f64, f64) {
    let tree = doc
        .build_page_render_tree(TITLE_PAGE - 1)
        .expect("3쪽 렌더 트리");
    let table = top_level_table_node(&tree.root, SQUARE_HOST_PARA).expect("3쪽 어울림 표");
    let mut caption = Vec::new();
    column_lines(&tree.root, SQUARE_CAPTION_PARA, &mut caption);
    let first = caption.first().expect("어울림 표 뒤 안내 줄");
    (
        table.bbox.x,
        table.bbox.y,
        table.bbox.y + table.bbox.height,
        first.y,
    )
}

/// 빈 host 의 문단 기준 어울림 표는 바깥여백 상자의 위·왼끝이 오프셋 자리에 온다. 표 괘선은
/// 거기서 위·왼 바깥여백만큼 안쪽이고, 뒤 안내 줄은 아래 바깥여백 아래에서 시작한다. 세로 오프셋
/// 0/1500/736HU, 위 바깥여백 0/1500/1000HU, 가로 오프셋 0/1000HU, 왼 바깥여백 283/1500HU 를
/// 바꾼 변형의 한컴 웹한글기안기 인쇄에서 표 괘선이 오프셋과 바깥여백의 합만큼 옮겨 가고, 표
/// 아래끝과 안내 줄 사이 간격은 변형마다 같다.
#[test]
fn empty_square_host_table_origin_follows_offsets_and_outer_margins() {
    let hu = |v: i32| v as f64 * 96.0 / 7200.0;
    let (base_x, base_top, _, _) = square_geometry(&square_variant(0, 0, 0, 0));
    for (v_off, top) in [(0u32, 0i16), (1500, 0), (0, 1500), (736, 1000), (736, 283)] {
        let (_, table_top, table_bottom, caption_top) =
            square_geometry(&square_variant(v_off, top, 0, 0));
        let expected = base_top + hu(v_off as i32 + top as i32);
        assert!(
            (table_top - expected).abs() < 0.5,
            "세로 오프셋 {v_off}HU·위 바깥여백 {top}HU: 표 위끝 {table_top:.2}, 기대 {expected:.2}"
        );
        let gap = caption_top - table_bottom;
        assert!(
            (gap - hu(283)).abs() < 0.5,
            "세로 오프셋 {v_off}HU·위 바깥여백 {top}HU: 표 아래끝과 안내 줄 사이 {gap:.2}px, \
             기대 아래 바깥여백 {:.2}px",
            hu(283)
        );
    }
    for (h_off, left) in [(1000u32, 283i16), (0, 1500), (1500, 1500)] {
        let (table_x, ..) = square_geometry(&square_variant(0, 0, h_off, left));
        let expected = base_x + hu(h_off as i32 + left as i32);
        assert!(
            (table_x - expected).abs() < 0.5,
            "가로 오프셋 {h_off}HU·왼 바깥여백 {left}HU: 표 x {table_x:.2}, 기대 {expected:.2}"
        );
    }
}
