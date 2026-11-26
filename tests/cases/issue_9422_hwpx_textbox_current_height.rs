//! 관계형·고정형 — [#9422] 한/글이 저장한 HWPX 의 넘치는 글상자는 바뀌지 않은 폭을 `curSz width="0"` 으로 적어도 저장 현재 높이로 인쇄 경계를 그린다.
//! 주석 갱신: 2026-11-26 KST_TIME KST (처음 작성)
//!
//! 입력: `samples/issue9419/kaf_appendix_2020.hwpx` — `samples/issue9379/kaf_appendix.hwp` 를 한/글 2020(11.0.0.1623)으로
//! HWPX 로 다시 저장한 파일. 기준 `pdf/issue9419/kaf_appendix_2020-2020.pdf`(54쪽).
//!
//! 36쪽 용어 글상자(구역 1 문단 46)는 `orgSz 47685×7855`, `curSz width="0" height="8743"`, 종축 배율 1.113049 를 저장한다.
//! 저장 글줄(vpos 0·3243·6486, 줄 높이 1691)과 안 여백 283·283 의 끝이 8743 HU 로 현재 높이와 같다. 한/글 2020 Print 는
//! 셋째 줄까지 상자 안에 그린다. #9415 는 같은 저장값의 HWP 재저장본에서 현재 높이를 쓰지만, HWPX 의 폭 0 은 바뀌지 않은
//! 축의 표지인데도 파서가 남긴 재직렬화 표지(`current_width_was_zero`)를 거절 조건으로 읽어 개체 높이(7855 HU)로 잘랐다.
//! - 반례: 크기 보호 상자는 선언 높이를 유지한다(같은 파일에서 보호 비트를 켜 확인).
//!
//! 측정: MEASURE_NOTE
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::{RectangleShape, ShapeObject};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const HWPX: &str = "samples/issue9419/kaf_appendix_2020.hwpx";
const HANCOM_PAGES: u32 = 54;
const PAGE: u32 = 35;
const HU_TO_PX: f64 = 96.0 / 7200.0;

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(HWPX);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {HWPX}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {HWPX}: {e:?}"));
    assert_eq!(core.page_count(), HANCOM_PAGES, "한컴 PDF 의 전체 쪽수");
    core
}

fn terminology_textbox(core: &mut DocumentCore) -> &mut RectangleShape {
    let Control::Shape(shape) = &mut core.document_mut().sections[1].paragraphs[46].controls[0]
    else {
        panic!("36쪽 용어 글상자 도형");
    };
    let ShapeObject::Rectangle(rectangle) = shape.as_mut() else {
        panic!("36쪽 용어 글상자 사각형");
    };
    rectangle
}

fn find_textbox_with_line<'a>(
    node: &'a RenderNode,
    first_word: &str,
) -> Option<(&'a RenderNode, &'a RenderNode)> {
    if matches!(node.node_type, RenderNodeType::TextBox) {
        let line = node.children.iter().find(|line| {
            matches!(line.node_type, RenderNodeType::TextLine(_))
                && line.children.iter().any(|child| {
                    matches!(&child.node_type, RenderNodeType::TextRun(run)
                        if run.text.trim_start().starts_with(first_word))
                })
        });
        if let Some(line) = line {
            return Some((node, line));
        }
    }
    node.children
        .iter()
        .find_map(|child| find_textbox_with_line(child, first_word))
}

#[test]
fn hwpx_resaved_textbox_uses_the_stored_current_height_for_its_print_frame() {
    let mut core = open();
    let rectangle = terminology_textbox(&mut core);
    let attr = &rectangle.drawing.shape_attr;
    assert!(
        attr.current_width_was_zero,
        "fixture 는 한/글이 바뀌지 않은 폭을 curSz width=\"0\" 으로 저장한 HWPX 다"
    );
    let declared = rectangle.common.height;
    let current = attr.current_height;
    assert!(current > declared, "저장 현재 높이가 개체 높이보다 크다");
    let text_box = rectangle.drawing.text_box.as_ref().unwrap();
    let content_end = text_box.paragraphs[0]
        .line_segs
        .iter()
        .map(|line| line.vertical_pos + line.line_height)
        .max()
        .unwrap()
        + i32::from(text_box.margin_top)
        + i32::from(text_box.margin_bottom);
    assert_eq!(
        content_end as u32, current,
        "저장 글줄과 안 여백의 끝이 현재 높이를 증언한다"
    );

    let tree = core.build_page_render_tree(PAGE).unwrap();
    let (textbox, first_line) = find_textbox_with_line(&tree.root, "In each").expect("첫 줄");
    let (last_box, last_line) =
        find_textbox_with_line(&tree.root, "and Cadastre").expect("셋째 줄");
    assert_eq!(last_box.id, textbox.id, "세 줄이 한 글상자에 있다");
    let clip = textbox.print_clip_bbox.expect("글상자 인쇄 경계");
    assert!(
        (clip.height - f64::from(current) * HU_TO_PX).abs() < 1e-6,
        "인쇄 경계 높이 {:.2} 는 저장 현재 높이 {:.2}px 다(개체 높이 {:.2}px 가 아니다)",
        clip.height,
        f64::from(current) * HU_TO_PX,
        f64::from(declared) * HU_TO_PX
    );
    assert!(first_line.bbox.y >= clip.y);
    assert!(
        last_line.bbox.y + last_line.bbox.height < clip.y + clip.height,
        "셋째 줄 아래에 아래 안 여백이 남는다"
    );
}

#[test]
fn hwpx_size_protected_textbox_keeps_its_declared_frame() {
    let mut core = open();
    let rectangle = terminology_textbox(&mut core);
    rectangle.common.size_protect = true;
    let declared = f64::from(rectangle.common.height) * HU_TO_PX;
    core.set_document(core.document().clone());
    let tree = core.build_page_render_tree(PAGE).unwrap();
    let (textbox, _) = find_textbox_with_line(&tree.root, "In each").expect("첫 줄");
    let clip = textbox.print_clip_bbox.expect("글상자 인쇄 경계");
    assert!((clip.height - declared).abs() < 1e-6);
}
