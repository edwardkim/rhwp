//! 용지 기준 자리차지 사각형의 바깥 여백·캡션은 개체 상자에 속한다 (#8189).
//!
//! 한/글 2020 이 열어 HWP 로 저장한 합성 입력(`samples/stored_reset_float/gen_field_float_reset.py`,
//! `P_WORDS=14 A_CAP=1 A_M=1417 A_IN0=1 A_REL=PAPER A_Y=8000 FILL=18`). 구역 첫 빈 문단(pi=0)이
//! 위·아래 바깥 여백 1417 HU 와 아래 캡션을 가진 용지 기준 자리차지 사각형 A 를 소유한다.
//! 두 줄짜리 문단 P(pi=20)는 첫 쪽 끝에 걸친다. `_field` 는 P 가 하이퍼링크 필드로 시작하고
//! 경계 뒤에 용지 기준·글 앞으로 사각형 B 를 둔다.
//!
//! ```text
//!   정본(한/글 2020 Print)          두 입력 모두
//!   사각형 A 위끝                    용지 위 + 세로 오프셋 + 위 여백 (125.1px)
//!   pi=0 저장 vpos                  A 상자 바닥 = 오프셋+위 여백+높이+아래 여백+캡션 간격·줄
//!   쪽 나눔                          2쪽 · P 줄 0 은 1쪽(…낱말11 낱말), 줄 1 은 2쪽(12 낱말13…)
//! ```
//!
//! 수정 전 rhwp 는 도형을 위 여백 없이 오프셋 자리에 그리고(106.7px), 자리차지 밴드에 캡션을
//! 빼 본문 전체를 캡션 높이(24.6px)만큼 위에 두었다. 그 결과 P 두 줄이 모두 1쪽에 들어갔다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::shape::ShapeObject;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLES: [&str; 2] = [
    "samples/stored_reset_float/paper_band_margin_caption.hwp",
    "samples/stored_reset_float/paper_band_margin_caption_field.hwp",
];
const P: u64 = 20;

fn core(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("read")).expect("parse")
}

fn px(hu: i32) -> f64 {
    rhwp::renderer::hwpunit_to_px(hu, 96.0)
}

fn text(node: &RenderNode) -> String {
    let mut out = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_string(),
        _ => String::new(),
    };
    for child in &node.children {
        out.push_str(&text(child));
    }
    out
}

/// 1쪽의 (사각형 위·아래, 글줄 위끝·글자) 목록.
fn page_one(core: &DocumentCore) -> (Vec<(f64, f64)>, Vec<(f64, String)>) {
    fn walk(node: &RenderNode, rects: &mut Vec<(f64, f64)>, lines: &mut Vec<(f64, String)>) {
        match &node.node_type {
            RenderNodeType::Rectangle(_) => {
                rects.push((node.bbox.y, node.bbox.y + node.bbox.height))
            }
            RenderNodeType::TextLine(_) => lines.push((node.bbox.y, text(node))),
            _ => {}
        }
        for child in &node.children {
            walk(child, rects, lines);
        }
    }
    let tree = core.build_page_render_tree(0).expect("page");
    let (mut rects, mut lines) = (Vec::new(), Vec::new());
    walk(&tree.root, &mut rects, &mut lines);
    (rects, lines)
}

/// 개체 잉크는 바깥 여백 상자 안쪽에 그려지고, 뒤 본문은 캡션까지 포함한 상자 아래에서 시작한다.
#[test]
fn paper_topbottom_shape_box_owns_outer_margin_and_caption() {
    for sample in SAMPLES {
        let core = core(sample);
        let section = &core.document().sections[0];
        let anchor = &section.paragraphs[0];
        let rect = anchor
            .controls
            .iter()
            .find_map(|control| match control {
                Control::Shape(shape) => match shape.as_ref() {
                    ShapeObject::Rectangle(rect) => Some(rect),
                    _ => None,
                },
                _ => None,
            })
            .expect("사각형 A");
        let common = &rect.common;
        let caption = rect.drawing.caption.as_ref().expect("A 의 아래 캡션");
        let caption_line = caption.paragraphs[0].line_segs[0].line_height;

        // 한/글 저장 사다리: pi=0 의 첫 줄은 캡션까지 포함한 A 상자 바닥에 놓인다(본문 기준).
        let page = &section.section_def.page_def;
        let body_top_hu = (page.margin_top + page.margin_header) as i32;
        let box_bottom_hu = common.vertical_offset as i32
            + i32::from(common.margin.top)
            + common.height as i32
            + i32::from(common.margin.bottom)
            + i32::from(caption.spacing)
            + caption_line;
        assert!(
            (anchor.line_segs[0].vertical_pos - (box_bottom_hu - body_top_hu)).abs() <= 2,
            "{sample}: 입력 가정 — pi=0 저장 vpos 가 캡션 포함 A 상자 바닥"
        );

        let (rects, lines) = page_one(&core);
        let a_top = rects
            .iter()
            .map(|(top, _)| *top)
            .fold(f64::INFINITY, f64::min);
        let ink_top = px(common.vertical_offset as i32 + i32::from(common.margin.top));
        assert!(
            (a_top - ink_top).abs() <= 1.0,
            "{sample}: A 잉크 위끝은 오프셋 + 위 여백 {ink_top:.1} — {a_top:.1}"
        );

        let caption_line_top = lines
            .iter()
            .find(|(_, line)| line.contains("캡션 A"))
            .unwrap_or_else(|| panic!("{sample}: 캡션 줄 — {lines:?}"))
            .0;
        let body = lines
            .iter()
            .find(|(_, line)| line.contains("field-float"))
            .unwrap_or_else(|| panic!("{sample}: pi=1 글줄 — {lines:?}"))
            .0;
        let stored_body = px(body_top_hu + section.paragraphs[1].line_segs[0].vertical_pos);
        assert!(
            caption_line_top > a_top && body > caption_line_top,
            "{sample}: A → 캡션 → 본문 순서 — A {a_top:.1}, 캡션 {caption_line_top:.1}, 본문 {body:.1}"
        );
        assert!(
            (body - stored_body).abs() <= 1.0,
            "{sample}: pi=1 은 저장 사다리 자리 {stored_body:.1} 에서 시작 — {body:.1}"
        );
    }
}

/// 캡션 포함 밴드 아래에서 흐르면 P 는 정본처럼 첫 줄만 1쪽에 둔다.
#[test]
fn paragraph_below_captioned_paper_band_splits_like_hancom() {
    for sample in SAMPLES {
        let core = core(sample);
        assert_eq!(core.page_count(), 2, "{sample}: 정본 2쪽");
        let pages = core.dump_page_items_json(None);
        let p_parts: Vec<_> = pages
            .as_array()
            .unwrap()
            .iter()
            .enumerate()
            .flat_map(|(page, value)| {
                value["columns"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .flat_map(|column| column["items"].as_array().unwrap().iter())
                    .filter(|item| {
                        item["paraIndex"].as_u64() == Some(P)
                            && item["kind"]
                                .as_str()
                                .is_some_and(|kind| kind.ends_with("aragraph"))
                    })
                    .map(move |item| {
                        (
                            page,
                            item["kind"].as_str().unwrap().to_string(),
                            item["startLine"].as_u64(),
                            item["endLine"].as_u64(),
                        )
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(
            p_parts,
            vec![
                (0, "partialParagraph".to_string(), Some(0), Some(1)),
                (1, "partialParagraph".to_string(), Some(1), Some(2)),
            ],
            "{sample}: P 는 줄 1에서 나뉜다"
        );
        let page_text = |page: u32| text(&core.build_page_render_tree(page).expect("page").root);
        let (first, second) = (page_text(0), page_text(1));
        assert!(
            first.contains("낱말11") && !first.contains("낱말13"),
            "{sample}: 1쪽은 P 첫 줄에서 끝난다"
        );
        assert!(
            second.contains("낱말13")
                && second.contains("뒤낱말08")
                && second.contains("P 뒤 문단"),
            "{sample}: 2쪽은 P 둘째 줄부터"
        );
    }
}
