//! [#8573] 앞 문단 어울림 표 상자와 겹치는 문단 기준 자리차지 표는 표만 그 상자 아래로 내려간다.
//!
//! 재현물(공개 샘플 변형, 저장 LINE_SEG 없음, 두 문서는 문단 0 host 글자만 다르다):
//!
//! ```text
//! 문단 0: host 글자(공백 3칸 / "XXX") + 어울림 표 "SQ"(단 오른쪽, 바깥 여백 1mm)
//! 문단 1: 자리차지 표 "BELOW"(문단 기준 세로 93HU, 단 거의 전폭, 바깥 여백 1mm) + 글자 "NEXT"
//! 문단 2: 글자 "END"
//! ```
//!
//! 웹한글기안기 인쇄본(`pdf/issue8573/*-host-square-then-topbottom-push-webhwp-print.pdf`)은 두
//! 문서가 같다. 문단 1 표는 어울림 표 아래끝 + 아래 바깥 여백 + 자기 위 바깥 여백에서 시작하고
//! (괘선 191.0 → 199.0), "NEXT"·"END" 글줄은 어울림 표 옆 문단 위치에 그대로 남아 표보다 위에
//! 놓인다.
//!
//! 수정 전에는 표가 문단 1 원점 + 오프셋에 그려져 어울림 표와 겹쳤고 "NEXT"·"END" 가 표 아래로
//! 갔다. 기대값은 모델 바깥 여백과 개체 사이 관계로 적는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLES: [&str; 2] = [
    "samples/issue8573/whitespace-host-square-then-topbottom-push.hwp",
    "samples/issue8573/text-host-square-then-topbottom-push.hwp",
];
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
                    page.lines.push((pi, span(n)));
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

/// 문단 `pi` 표의 (위 바깥 여백, 아래 바깥 여백) px.
fn margins(core: &DocumentCore, pi: usize) -> (f64, f64) {
    core.document().sections[0].paragraphs[pi]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some((
                table.outer_margin_top as f64 * HWPUNIT_PX,
                table.outer_margin_bottom as f64 * HWPUNIT_PX,
            )),
            _ => None,
        })
        .unwrap_or_else(|| panic!("문단 {pi} 표"))
}

#[test]
fn topbottom_table_moves_below_preceding_square_box() {
    for sample in SAMPLES {
        let core = core(sample);
        let page = first_page(&core);
        let square = table_of(&page, 0);
        let table = table_of(&page, 1);
        let (_, square_bottom) = margins(&core, 0);
        let (own_top, _) = margins(&core, 1);
        let floor = square.bottom + square_bottom + own_top;
        assert!(
            (table.top - floor).abs() <= TOL_PX,
            "{sample}: 문단 1 표 위({:.2})는 어울림 표 아래 + 두 바깥 여백({floor:.2})이다",
            table.top
        );
    }
}

#[test]
fn host_and_following_lines_stay_beside_square_table() {
    for sample in SAMPLES {
        let core = core(sample);
        let page = first_page(&core);
        let square = table_of(&page, 0);
        let table = table_of(&page, 1);
        for pi in [1, 2] {
            let line = first_line(&page, pi);
            assert!(
                line.top < square.bottom,
                "{sample}: 문단 {pi} 글줄 위({:.2})는 어울림 표 옆(표 아래끝 {:.2} 위)에 남는다",
                line.top,
                square.bottom
            );
            assert!(
                line.bottom <= table.top + TOL_PX,
                "{sample}: 문단 {pi} 글줄 아래({:.2})는 내려간 자리차지 표 위({:.2}) 위다",
                line.bottom,
                table.top
            );
        }
    }
}
