//! [#8573] 저장 줄 없는 공백 host 문단의 줄은 글자 있는 host 줄과 똑같이 놓인다.
//!
//! 재현물(공개 샘플 변형, 저장 LINE_SEG 없음, 두 문서는 host 글자만 다르다):
//!
//! ```text
//! 문단 0: "Q" 한 줄
//! 문단 1: host 글자(공백 7칸 / "X", 20pt 160%) + 자리차지 표(문단 기준 세로 26HU, 바깥 여백 1mm)
//! 문단 2: "Z" 한 줄
//! ```
//!
//! 웹한글기안기 인쇄본(`pdf/issue8573/*-topbottom-line-below-webhwp-print.pdf`)에서 두 문서의
//! Q→Z 거리는 112.00px 로 같다. 세로 오프셋이 host 줄 높이보다 작아 host 줄이 표 위에 들지
//! 못하고 표 바깥 상자 **아래**로 가며, 문단 2 는 그 줄 뒤에 온다. 공백 host 도 이 줄을 쓴다.
//!
//! 수정 전에는 공백 host 만 다른 경로를 타서 host 줄 높이 일부만 셌다(Q→Z 88.00px, 글자 host 는
//! 111.89px). 기대값은 픽셀 좌표가 아니라 두 문서의 관계와 모델 바깥 여백으로 적는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const WHITESPACE: &str = "samples/issue8573/whitespace-host-topbottom-line-below.hwp";
const TEXT: &str = "samples/issue8573/text-host-topbottom-line-below.hwp";
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

fn line_top(page: &Page, pi: usize) -> f64 {
    page.lines
        .iter()
        .filter(|(p, _)| *p == pi)
        .map(|(_, s)| s.top)
        .reduce(f64::min)
        .unwrap_or_else(|| panic!("1쪽에 문단 {pi} 글줄: {:?}", page.lines))
}

fn table_of(page: &Page, pi: usize) -> Span {
    page.tables
        .iter()
        .find(|(p, _)| *p == pi)
        .map(|(_, s)| *s)
        .unwrap_or_else(|| panic!("1쪽에 문단 {pi} 표: {:?}", page.tables))
}

fn outer_bottom(core: &DocumentCore) -> f64 {
    core.document().sections[0].paragraphs[1]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.outer_margin_bottom as f64 * HWPUNIT_PX),
            _ => None,
        })
        .expect("문단 1 표")
}

#[test]
fn whitespace_host_places_next_paragraph_like_text_host() {
    let ws = first_page(&core(WHITESPACE));
    let tx = first_page(&core(TEXT));
    let ws_gap = line_top(&ws, 2) - line_top(&ws, 0);
    let tx_gap = line_top(&tx, 2) - line_top(&tx, 0);
    assert!(
        (ws_gap - tx_gap).abs() <= TOL_PX,
        "공백 host 의 Q→Z 거리({ws_gap:.2})는 글자 host({tx_gap:.2})와 같다"
    );
    let ws_table = table_of(&ws, 1);
    let tx_table = table_of(&tx, 1);
    assert!(
        (ws_table.top - tx_table.top).abs() <= TOL_PX,
        "공백 host 표 위({:.2})는 글자 host 표 위({:.2})와 같다",
        ws_table.top,
        tx_table.top
    );
}

#[test]
fn whitespace_host_line_sits_below_table_outer_box() {
    let ws_core = core(WHITESPACE);
    let ws = first_page(&ws_core);
    let tx = first_page(&core(TEXT));
    // 글자 host 의 host 줄 높이(표 아래 줄 상자)를 공백 host 의 줄 높이로 쓴다.
    let host_line = tx
        .lines
        .iter()
        .find(|(p, _)| *p == 1)
        .map(|(_, s)| s.bottom - s.top)
        .expect("글자 host 줄");
    let table = table_of(&ws, 1);
    let floor = table.bottom + outer_bottom(&ws_core) + host_line;
    let next = line_top(&ws, 2);
    assert!(
        next + TOL_PX >= floor,
        "문단 2 글줄 위({next:.2})는 표 아래 + 아래 바깥 여백 + host 줄({floor:.2}) 아래다"
    );
}
