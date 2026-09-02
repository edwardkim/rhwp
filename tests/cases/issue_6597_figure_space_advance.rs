//! [Issue #6597] 고정폭 빈칸(HWP5 문자 컨트롤 31 → `U+2007`)의 전진폭이 한/글의 두
//! 배(0.5em)여서 글머리 본문이 오른쪽으로 밀리던 결함의 가드.
//!
//! 한/글 오라클 PDF 를 `rawdict` 글자 origin 델타로 재면 **일반 공백의 정확히 절반**
//! 이다 (문서 `30307`, 글꼴 14.99pt):
//!
//! ```text
//! `권고일자<FW> : `(13쪽)          `자` advance 14.992 → 다음까지 18.710 ⇒ 3.718pt
//! ` ○<FW><FW>국민소통창구와`(5쪽)   `○` advance 14.992 → 22.428 ⇒ 7.436/2 = 3.718pt
//! (대조) 일반 공백 U+0020                                        7.436pt
//! ```
//!
//! 3.718 / 14.992 = 0.248em. 종전 `font_size * 0.5` 는 정확히 두 배였다.
//!
//! 수정 뒤 실측 — 3쪽 제목 `「` x 107.91 → 97.91 (한/글 98.67), 5쪽 글머리 63줄이
//! 정확히 −10.0px 씩 제자리로 돌아온다.
//!
//! ⚠ 이 시험은 절대 폭이 아니라 **일반 공백과의 비**를 잠근다. 폰트 메트릭이 바뀌어도
//! "고정폭 빈칸 = 공백의 절반"이라는 계약은 그대로여야 한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::document::{Document, Section};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::{CharShape, ParaShape};
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::serializer::hwpx::serialize_hwpx;

/// 같은 글자 두 개 사이에 무엇을 끼우느냐만 다른 세 문단.
const NONE: &str = "가가";
const SPACE: &str = "가 가";
const FIGURE: &str = "가\u{2007}가";

fn document() -> Document {
    let mut section = Section::default();
    section.section_def.page_def = PageDef {
        width: 59528,
        height: 84188,
        ..Default::default()
    };
    for text in [NONE, SPACE, FIGURE] {
        section.paragraphs.push(Paragraph {
            text: text.to_string(),
            char_count: text.chars().count() as u32,
            ..Default::default()
        });
    }

    let mut doc = Document::default();
    doc.doc_info.para_shapes = vec![ParaShape::default()];
    doc.doc_info.char_shapes = vec![CharShape::default()];
    doc.doc_properties.section_count = 1;
    doc.sections.push(section);
    doc
}

/// 줄마다 그 줄 `TextRun` 들의 가로 범위(글자가 실제로 차지한 폭)를 돌려준다.
/// `TextLine` 의 bbox 폭은 단 폭이라 이 축에 못 쓴다.
fn line_text_extents(node: &RenderNode, out: &mut Vec<f64>) {
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        let mut lo = f64::MAX;
        let mut hi = f64::MIN;
        collect_runs(node, &mut lo, &mut hi);
        if hi > lo {
            out.push(hi - lo);
        }
        return;
    }
    for child in &node.children {
        line_text_extents(child, out);
    }
}

fn collect_runs(node: &RenderNode, lo: &mut f64, hi: &mut f64) {
    if matches!(node.node_type, RenderNodeType::TextRun(_)) {
        *lo = lo.min(node.bbox.x);
        *hi = hi.max(node.bbox.x + node.bbox.width);
    }
    for child in &node.children {
        collect_runs(child, lo, hi);
    }
}

#[test]
fn figure_space_advance_is_half_of_a_normal_space() {
    let bytes = serialize_hwpx(&document()).expect("serialize");
    let core = DocumentCore::from_bytes(&bytes).expect("reload");
    let page = core.build_page_render_tree(0).expect("render tree");

    let mut widths = Vec::new();
    line_text_extents(&page.root, &mut widths);
    assert!(
        widths.len() >= 3,
        "세 문단의 줄 폭이 있어야 한다 — 시험 설정 오류. widths={widths:?}"
    );

    let (none_w, space_w, figure_w) = (widths[0], widths[1], widths[2]);
    let space_advance = space_w - none_w;
    let figure_advance = figure_w - none_w;

    assert!(
        space_advance > 1.0 && figure_advance > 0.5,
        "공백/고정폭 빈칸이 폭을 갖고 있어야 한다 — 시험 설정 오류. \
         none={none_w:.2} space={space_w:.2} figure={figure_w:.2}"
    );

    let ratio = figure_advance / space_advance;
    assert!(
        (ratio - 0.5).abs() <= 0.06,
        "고정폭 빈칸(U+2007)은 일반 공백의 **절반**이어야 한다 — #6597 회귀. \
         ratio={ratio:.3} (figure={figure_advance:.2}px space={space_advance:.2}px)"
    );
}
