//! [#8573] 저장 줄 없는 **빈** host 의 어울림 표는 흐름을 표 높이가 아니라 host 줄 하나만큼만
//! 전진시킨다. 어울림 표 상자는 뒤 문단이 피하는 영역으로 남는다.
//!
//! 재현물(공개 샘플 변형, 저장 LINE_SEG 없음, `tools/make_square_host_fixture.py --host-text ""
//! --sq-in-column --overlay-rect --extra-square-pic --no-behind-pic --sq-run-charpr 5
//! --p1-text "  NEXT" --below-voff {5300|1074}`):
//!
//! ```text
//! 문단 0: 글자 없음 + 어울림 표 "SQ"(16pt run, 단 오른쪽, 왼쪽에 글자 띠) + 종이 기준 글앞으로
//!         사각형 + 종이 기준 어울림 작은 그림(어울림 개체 둘). 문단 끝 글자는 10pt.
//! 문단 1: 자리차지 표 "BELOW"(문단 기준 세로 오프셋, 바깥 여백 1mm) + 글자 "  NEXT"
//! 문단 2: 글자 "END"
//! ```
//!
//! 웹한글기안기 인쇄본(`pdf/issue8573/empty-host-square-*-webhwp-print.pdf`):
//! - 문단 0 은 어울림 표 옆에 줄 하나만 두고, 그 줄 높이는 개체 부호의 글자 모양까지 든 가장 큰
//!   글자(16pt)로 정한다. host 글자를 `"X"` 로 바꾼 문서도 같은 자리다.
//! - 오프셋이 큰 문서(`beside-line`): "NEXT"·"END" 는 어울림 표 옆, "BELOW" 표는 문단 1
//!   원점(= "NEXT" 줄 위) + 오프셋 + 위 바깥 여백.
//! - 오프셋이 작은 문서(`push`): "BELOW" 표는 어울림 표 아래 + 두 바깥 여백으로 내려가고, "END"
//!   는 그 표 위 틈에 들지 못해 표 아래로 간다.
//!
//! - 글자 host(`text-host-square-paper-companion-beside-line`, 종이 기준 어울림 그림 동반)도
//!   같다. host 글줄은 어울림 표 옆에 남는다.
//!
//! 수정 전에는 문단 0 이 어울림 표 높이만큼 흐름을 전진시켜 뒤 내용이 모두 어울림 표 아래로
//! 밀렸다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::style::LineSpacingType;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const BESIDE: &str = "samples/issue8573/empty-host-square-beside-line.hwp";
const PUSH: &str = "samples/issue8573/empty-host-square-then-topbottom-push.hwp";
/// 같은 형상의 글자("XX") host. 글자 크기는 한 가지(10pt)다.
const TEXT_HOST: &str = "samples/issue8573/text-host-square-paper-companion-beside-line.hwp";
const HWPUNIT_PX: f64 = 96.0 / 7200.0;
const TOL_PX: f64 = 0.5;

#[derive(Debug, Clone, Copy)]
struct Span {
    top: f64,
    bottom: f64,
}

#[derive(Default)]
struct Page {
    tables: Vec<(usize, Span)>,
    lines: Vec<(usize, Span)>,
}

fn core(sample: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn first_page(core: &DocumentCore) -> Page {
    fn span(n: &RenderNode) -> Span {
        Span {
            top: n.bbox.y,
            bottom: n.bbox.y + n.bbox.height,
        }
    }
    fn walk(n: &RenderNode, page: &mut Page) {
        match &n.node_type {
            RenderNodeType::Table(table) => {
                if let Some(pi) = table.para_index {
                    page.tables.push((pi, span(n)));
                }
                return;
            }
            RenderNodeType::TextLine(line) => {
                if let Some(pi) = line.para_index {
                    if n.bbox.height > 0.0 {
                        page.lines.push((pi, span(n)));
                    }
                }
            }
            _ => {}
        }
        for c in &n.children {
            walk(c, page);
        }
    }
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut page = Page::default();
    walk(&tree.root, &mut page);
    page
}

fn table_of(page: &Page, pi: usize) -> Span {
    page.tables
        .iter()
        .find(|(p, _)| *p == pi)
        .map(|(_, s)| *s)
        .unwrap_or_else(|| panic!("1쪽에 문단 {pi} 표: {:?}", page.tables))
}

fn first_line(page: &Page, pi: usize) -> Span {
    page.lines
        .iter()
        .filter(|(p, _)| *p == pi)
        .map(|(_, s)| *s)
        .reduce(|a, b| if a.top <= b.top { a } else { b })
        .unwrap_or_else(|| panic!("1쪽에 문단 {pi} 글줄: {:?}", page.lines))
}

/// 문단 `pi` 표의 (세로 오프셋, 위 바깥 여백, 아래 바깥 여백) px.
fn table_geometry(core: &DocumentCore, pi: usize) -> (f64, f64, f64) {
    core.document().sections[0].paragraphs[pi]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some((
                table.common.vertical_offset as i32 as f64 * HWPUNIT_PX,
                table.outer_margin_top as f64 * HWPUNIT_PX,
                table.outer_margin_bottom as f64 * HWPUNIT_PX,
            )),
            _ => None,
        })
        .unwrap_or_else(|| panic!("문단 {pi} 표"))
}

/// 문단 0 의 가장 큰 글자 모양 한 줄의 전진량 px(글자 크기 × 퍼센트 줄 간격).
fn largest_char_line_advance(core: &DocumentCore) -> f64 {
    let doc = core.document();
    let para = &doc.sections[0].paragraphs[0];
    let size_pt = para
        .char_shapes
        .iter()
        .map(|r| doc.doc_info.char_shapes[r.char_shape_id as usize].base_size)
        .max()
        .expect("문단 0 글자 모양") as f64
        / 100.0;
    let shape = &doc.doc_info.para_shapes[para.para_shape_id as usize];
    assert!(matches!(shape.line_spacing_type, LineSpacingType::Percent));
    size_pt * 96.0 / 72.0 * shape.line_spacing as f64 / 100.0
}

#[test]
fn empty_host_advances_one_line_of_largest_char_shape() {
    for sample in [BESIDE, PUSH, TEXT_HOST] {
        let core = core(sample);
        let page = first_page(&core);
        let square = table_of(&page, 0);
        let (square_offset, square_top, _) = table_geometry(&core, 0);
        let origin = square.top - square_top - square_offset;
        let next = first_line(&page, 1);
        let advance = largest_char_line_advance(&core);
        assert!(
            ((next.top - origin) - advance).abs() <= TOL_PX,
            "{sample}: 문단 1 원점({:.2})은 빈 host 원점({origin:.2}) + 가장 큰 글자 한 줄({advance:.2})이다",
            next.top
        );
        assert!(
            next.top < square.bottom,
            "{sample}: 문단 1 글줄 위({:.2})는 어울림 표 옆(표 아래끝 {:.2} 위)이다",
            next.top,
            square.bottom
        );
    }
}

#[test]
fn topbottom_table_starts_from_paragraph_origin_beside_square_table() {
    for sample in [BESIDE, TEXT_HOST] {
        topbottom_table_starts_from_paragraph_origin(sample);
    }
}

#[test]
fn text_host_line_stays_beside_square_table_with_paper_companion() {
    let core = core(TEXT_HOST);
    let page = first_page(&core);
    let square = table_of(&page, 0);
    let (square_offset, square_top, _) = table_geometry(&core, 0);
    let origin = square.top - square_top - square_offset;
    // 문단 0 의 글줄 중 본문 흐름의 것(글상자 안 글줄은 문단 원점보다 위에 있다).
    let host = page
        .lines
        .iter()
        .filter(|(p, s)| *p == 0 && s.top >= origin - TOL_PX)
        .map(|(_, s)| *s)
        .reduce(|a, b| if a.top <= b.top { a } else { b })
        .unwrap_or_else(|| panic!("문단 0 본문 글줄: {:?}", page.lines));
    assert!(
        (host.top - origin).abs() <= TOL_PX && host.bottom <= square.bottom,
        "host 글줄({:.2}..{:.2})은 문단 원점({origin:.2})에서 어울림 표({:.2}..{:.2}) 옆에 놓인다",
        host.top,
        host.bottom,
        square.top,
        square.bottom
    );
}

fn topbottom_table_starts_from_paragraph_origin(sample: &str) {
    let core = core(sample);
    assert_eq!(core.page_count(), 1);
    let page = first_page(&core);
    let square = table_of(&page, 0);
    let next = first_line(&page, 1);
    let end = first_line(&page, 2);
    let table = table_of(&page, 1);
    let (offset, own_top, _) = table_geometry(&core, 1);
    let expected = next.top + offset + own_top;
    assert!(
        (table.top - expected).abs() <= TOL_PX,
        "문단 1 표 위({:.2})는 문단 1 원점({:.2}) + 오프셋 + 위 바깥 여백({expected:.2})이다",
        table.top,
        next.top
    );
    assert!(
        end.top < square.bottom && end.bottom <= table.top + TOL_PX,
        "문단 2 글줄({:.2}..{:.2})은 어울림 표 옆, 자리차지 표({:.2}) 위다",
        end.top,
        end.bottom,
        table.top
    );
}

#[test]
fn topbottom_table_moves_below_empty_host_square_box() {
    let core = core(PUSH);
    assert_eq!(core.page_count(), 1);
    let page = first_page(&core);
    let square = table_of(&page, 0);
    let table = table_of(&page, 1);
    let (_, _, square_bottom) = table_geometry(&core, 0);
    let (_, own_top, _) = table_geometry(&core, 1);
    let floor = square.bottom + square_bottom + own_top;
    assert!(
        (table.top - floor).abs() <= TOL_PX,
        "문단 1 표 위({:.2})는 어울림 표 아래 + 두 바깥 여백({floor:.2})이다",
        table.top
    );
    let end = first_line(&page, 2);
    assert!(
        end.top >= table.bottom,
        "문단 2 글줄 위({:.2})는 표 위 틈에 들지 못해 자리차지 표 아래({:.2})다",
        end.top,
        table.bottom
    );
}
