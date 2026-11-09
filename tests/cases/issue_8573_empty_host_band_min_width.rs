//! [#8573] 저장 줄 없는 빈 host 의 어울림 표 옆 줄은 한/글 최소 줄 폭(1440HU) 띠에서만 생긴다.
//!
//! 재현물(공개 샘플 변형, 저장 LINE_SEG 없음, 세 문서는 표 폭만 다르다):
//!
//! ```text
//! 문단 0: 구역·단 정의만 든 빈 host + 어울림 표(문단 기준, 왼쪽, 오른쪽 띠 600HU / 1430HU / 1450HU)
//! 문단 1: "A"
//! 문단 2: "B"
//! ```
//!
//! 웹한글기안기 인쇄본(`pdf/issue8573/empty-host-square-band-*-webhwp-print.pdf`)에서 띠 600HU 는
//! "A" 가 표 바로 아래(빈 host 줄 없이)에 오고, 띠 1430HU 는 "A"·"B" 가 표 아래에, 띠 1450HU 는 표 오른쪽 띠 안에 놓인다(한/글이 줄에
//! 보장하는 최소 너비 1440HU = 0.2인치). 단 폭에 가까운 표 옆에 줄이 생기면 안 되고, 최소 줄
//! 폭 이상 띠가 남으면 빈 host 도 글자 host 처럼 표 옆으로 흘러야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const NO_BAND: &str = "samples/issue8573/empty-host-square-band-600-no-extra-line.hwp";
const NARROW: &str = "samples/issue8573/empty-host-square-band-1430-below.hwp";
const WIDE: &str = "samples/issue8573/empty-host-square-band-1450-beside.hwp";
const TOL_PX: f64 = 0.5;

#[derive(Debug, Clone, Copy)]
struct Rect {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

#[derive(Default, Debug)]
struct Page {
    tables: Vec<(usize, Rect)>,
    lines: Vec<(usize, Rect)>,
}

fn first_page(sample: &str) -> Page {
    fn rect(n: &RenderNode) -> Rect {
        Rect {
            left: n.bbox.x,
            top: n.bbox.y,
            right: n.bbox.x + n.bbox.width,
            bottom: n.bbox.y + n.bbox.height,
        }
    }
    fn walk(n: &RenderNode, page: &mut Page) {
        match &n.node_type {
            RenderNodeType::Table(table) => {
                if let Some(pi) = table.para_index {
                    page.tables.push((pi, rect(n)));
                }
                return;
            }
            RenderNodeType::TextLine(line) => {
                if let Some(pi) = line.para_index {
                    page.lines.push((pi, rect(n)));
                }
            }
            _ => {}
        }
        for c in &n.children {
            walk(c, page);
        }
    }
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut page = Page::default();
    walk(&tree.root, &mut page);
    page
}

fn table(page: &Page) -> Rect {
    page.tables
        .iter()
        .find(|(p, _)| *p == 0)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("1쪽에 문단 0 표: {:?}", page.tables))
}

fn line(page: &Page, pi: usize) -> Rect {
    page.lines
        .iter()
        .find(|(p, _)| *p == pi)
        .map(|(_, r)| *r)
        .unwrap_or_else(|| panic!("1쪽에 문단 {pi} 글줄: {:?}", page.lines))
}

#[test]
fn band_narrower_than_min_line_width_keeps_lines_below_table() {
    let page = first_page(NARROW);
    let t = table(&page);
    for pi in [1, 2] {
        let l = line(&page, pi);
        assert!(
            l.top + TOL_PX >= t.bottom,
            "띠 1430HU: 문단 {pi} 글줄 위({:.2})는 표 아래({:.2}) 뒤다 — 표 옆에 줄이 생기지 않는다",
            l.top,
            t.bottom
        );
    }
    assert!(
        !page
            .lines
            .iter()
            .any(|(_, l)| l.top < t.bottom - TOL_PX && l.left >= t.right - TOL_PX),
        "띠 1430HU: 표 옆 띠에 글줄이 없다: {:?}",
        page.lines
    );
}

#[test]
fn band_at_least_min_line_width_flows_beside_table() {
    let page = first_page(WIDE);
    let t = table(&page);
    for pi in [1, 2] {
        let l = line(&page, pi);
        assert!(
            l.top < t.bottom - TOL_PX && l.left + TOL_PX >= t.right,
            "띠 1450HU: 문단 {pi} 글줄({:.2},{:.2})은 표(오른쪽 {:.2}, 아래 {:.2}) 옆 띠에 놓인다",
            l.left,
            l.top,
            t.right,
            t.bottom
        );
    }
    let a = line(&page, 1);
    let b = line(&page, 2);
    assert!(
        b.top + TOL_PX >= a.bottom,
        "띠 1450HU: 문단 2({:.2})는 문단 1 아래({:.2}) 뒤다",
        b.top,
        a.bottom
    );
}

#[test]
fn near_column_width_table_adds_no_empty_host_line_below() {
    let page = first_page(NO_BAND);
    let t = table(&page);
    let a = line(&page, 1);
    let line_height = a.bottom - a.top;
    assert!(
        a.top + TOL_PX >= t.bottom && a.top - t.bottom < line_height,
        "띠 600HU: 문단 1 글줄 위({:.2})는 표 아래({:.2})에서 빈 host 줄 하나({line_height:.2})만큼 더 밀리지 않는다",
        a.top,
        t.bottom
    );
}
