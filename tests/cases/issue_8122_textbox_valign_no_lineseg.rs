//! [#8122] 저장 줄 없는 글상자의 세로 가운데 정렬이 내용 높이를 0으로 보던 결함의 회귀 가드.
//!
//! # 무엇이 깨져 있었나
//!
//! `layout_textbox_content` 는 세로 정렬에 쓸 내용 높이를 문단의 저장 LINE_SEG
//! (`vertical_pos + line_height`)로 셌다. 저장 줄이 없는 글상자 문단은 이 값이 없어 내용 높이가
//! 0 이 되고, 가운데 정렬이 안쪽 높이의 절반을 위 여백으로 써서 글이 상자 아래로 넘쳤다.
//!
//! # 기대값의 출처
//!
//! 재현물은 실문서(저장 LINE_SEG 없음)를 글자 치환·그림 교체·문단 삭제로 익명화한 것이다. 같은
//! 파일을 한컴 웹한글기안기로 인쇄한 PDF 에서 글상자 안 두 줄은 모두 상자 안에 있고, 내용이
//! 안쪽 높이를 채워 위아래 남는 공간이 같다.
//!
//! # 검사하는 것
//!
//! 글상자 안 줄이 모두 상자 안에 있는지, 세로 가운데 정렬로 위아래 남는 공간이 같은지.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8122/textbox_valign_no_lineseg.hwp";
/// 저장 줄 없는 두 문단을 담은 세로 가운데 정렬 글앞 글상자(0번 문단 컨트롤 2).
const HOST_PARA: usize = 0;
const TEXTBOX_CONTROL: usize = 2;

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
}

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("재현물 읽기");
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn find_textbox_shape(node: &RenderNode) -> Option<&RenderNode> {
    if let RenderNodeType::Rectangle(rect) = &node.node_type {
        if rect.para_index == Some(HOST_PARA) && rect.control_index == Some(TEXTBOX_CONTROL) {
            return Some(node);
        }
    }
    node.children.iter().find_map(find_textbox_shape)
}

#[test]
fn no_lineseg_textbox_centers_its_lines_inside_the_box() {
    let doc = load();
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");

    let shape = find_textbox_shape(&tree.root).expect("글상자 사각형");
    let outer = Rect::of(shape);
    let inner_node = shape
        .children
        .iter()
        .find(|c| matches!(c.node_type, RenderNodeType::TextBox))
        .expect("글상자 안쪽 영역");
    let inner = Rect::of(inner_node);
    let lines: Vec<Rect> = inner_node
        .children
        .iter()
        .filter(|c| matches!(c.node_type, RenderNodeType::TextLine(_)))
        .map(Rect::of)
        .collect();
    assert_eq!(lines.len(), 2, "글상자 문단 두 개가 한 줄씩: {lines:?}");

    for line in &lines {
        assert!(
            line.y + 0.5 >= outer.y && line.bottom() <= outer.bottom() + 0.5,
            "글상자 줄은 상자 안에 있어야 한다 — 내용 높이를 0으로 보면 가운데 정렬이 \
             안쪽 높이의 절반만큼 내려 상자 아래로 넘친다: 상자 {outer:?}, 줄 {line:?}"
        );
    }

    let top_gap = lines[0].y - inner.y;
    let bottom_gap = inner.bottom() - lines[lines.len() - 1].bottom();
    assert!(
        (top_gap - bottom_gap).abs() <= 1.0,
        "세로 가운데 정렬은 위아래 남는 공간이 같아야 한다: 위 {top_gap:.1}, 아래 {bottom_gap:.1}"
    );
}

// ── 정렬 이동과 실제 위쪽 배치의 공통 결과 ─────────────────────────────────────────
//
// 저장 줄이 없는 글상자의 가운데·아래쪽 정렬은 위쪽 정렬로 실제 배치한 줄 끝을 내용 높이로 써야
// 한다. 문단 앞 간격·줄 간격·빈 문단을 측정 쪽에서 따로 세면, 배치가 그 간격을 소비하지 않는
// 경계(단 맨 위 앞 간격 등)에서 정렬 이동이 실제 흐름과 어긋난다. 아래 검사는 한컴 절대 좌표가
// 아니라 같은 입력의 위쪽 배치를 기준으로 한 정렬 불변식이다.
//   가운데: 줄 위치 = 위쪽 위치 + (안쪽 아래끝 − 위쪽 마지막 줄 아래끝) / 2
//   아래쪽: 마지막 줄 아래끝 = 안쪽 아래끝 (마지막 줄 간격은 점유로 세지 않는다)

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::table::VerticalAlign;

/// 원 재현물의 글상자를 키우고(안쪽 높이 여유 확보) 문단 구성을 바꾼 뒤 정렬별 줄 위치를 얻는다.
fn layout_variant(
    align: VerticalAlign,
    spacing_before_hu: i32,
    append_empty_para: bool,
) -> (Rect, Vec<Rect>) {
    let mut doc = load();
    let mut model = doc.document().clone();
    let new_shape_id = model.doc_info.para_shapes.len() as u16;
    {
        let host = &mut model.sections[0].paragraphs[HOST_PARA];
        let Control::Shape(shape) = &mut host.controls[TEXTBOX_CONTROL] else {
            panic!("0번 문단 컨트롤 2 는 글상자 도형이어야 한다");
        };
        let common = shape.common_mut();
        common.height *= 2;
        let text_box = shape
            .drawing_mut()
            .and_then(|d| d.text_box.as_mut())
            .expect("글상자");
        text_box.vertical_align = align;
        if append_empty_para {
            // `new_empty_like` 는 편집용 기본 줄 하나를 넣는다. 재현물처럼 저장 줄 없는 빈 문단으로 둔다.
            let mut empty = Paragraph::new_empty_like(text_box.paragraphs.last().unwrap());
            empty.line_segs.clear();
            text_box.paragraphs.push(empty);
        }
        let template_id = text_box.paragraphs[0].para_shape_id as usize;
        for para in &mut text_box.paragraphs {
            assert!(
                para.line_segs.is_empty(),
                "재현물의 글상자 문단은 저장 줄이 없어야 한다"
            );
            para.para_shape_id = new_shape_id;
        }
        let mut shape_style = model.doc_info.para_shapes[template_id].clone();
        shape_style.spacing_before = spacing_before_hu;
        model.doc_info.para_shapes.push(shape_style);
    }
    doc.set_document(model);
    let tree = doc.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let shape = find_textbox_shape(&tree.root).expect("글상자 사각형");
    let inner_node = shape
        .children
        .iter()
        .find(|c| matches!(c.node_type, RenderNodeType::TextBox))
        .expect("글상자 안쪽 영역");
    let lines = inner_node
        .children
        .iter()
        .filter(|c| matches!(c.node_type, RenderNodeType::TextLine(_)))
        .map(Rect::of)
        .collect();
    (Rect::of(inner_node), lines)
}

fn assert_alignment_follows_top_flow(spacing_before_hu: i32, append_empty_para: bool) {
    let case = format!("앞 간격 {spacing_before_hu}HU, 빈 문단 추가 {append_empty_para}");
    let (inner, top) = layout_variant(VerticalAlign::Top, spacing_before_hu, append_empty_para);
    let (_, center) = layout_variant(VerticalAlign::Center, spacing_before_hu, append_empty_para);
    let (_, bottom) = layout_variant(VerticalAlign::Bottom, spacing_before_hu, append_empty_para);
    let expected_lines = 2 + usize::from(append_empty_para);
    assert_eq!(top.len(), expected_lines, "{case}: 위쪽 줄 수 {top:?}");
    assert_eq!(center.len(), top.len(), "{case}: 가운데 줄 수");
    assert_eq!(bottom.len(), top.len(), "{case}: 아래쪽 줄 수");

    let top_end = top.last().unwrap().bottom();
    let free = inner.bottom() - top_end;
    assert!(
        free > 10.0,
        "{case}: 상자를 키운 변형은 정렬 여유가 있어야 한다 (여유 {free:.2})"
    );
    for (i, ((t, c), b)) in top.iter().zip(&center).zip(&bottom).enumerate() {
        assert!(
            (c.y - (t.y + free / 2.0)).abs() <= 0.5,
            "{case}: 가운데 정렬 {i}번 줄은 위쪽 배치에서 남는 공간의 절반만큼 내려가야 한다 \
             — 위쪽 {:.2}, 기대 {:.2}, 실제 {:.2}",
            t.y,
            t.y + free / 2.0,
            c.y
        );
        assert!(
            (b.y - (t.y + free)).abs() <= 0.5,
            "{case}: 아래쪽 정렬 {i}번 줄은 위쪽 배치에서 남는 공간만큼 내려가야 한다 \
             — 위쪽 {:.2}, 기대 {:.2}, 실제 {:.2}",
            t.y,
            t.y + free,
            b.y
        );
    }
    assert!(
        (bottom.last().unwrap().bottom() - inner.bottom()).abs() <= 0.5,
        "{case}: 아래쪽 정렬의 마지막 줄 아래끝은 안쪽 아래끝과 같아야 한다 \
         (마지막 줄 간격·뒤 간격은 점유가 아니다): 줄 {:.2}, 안쪽 {:.2}",
        bottom.last().unwrap().bottom(),
        inner.bottom()
    );
}

#[test]
fn enlarged_box_without_spacing_aligns_from_top_flow() {
    assert_alignment_follows_top_flow(0, false);
}

#[test]
fn spacing_before_on_every_paragraph_aligns_from_top_flow() {
    assert_alignment_follows_top_flow(600, false);
}

#[test]
fn trailing_empty_paragraph_aligns_from_top_flow() {
    assert_alignment_follows_top_flow(600, true);
}
