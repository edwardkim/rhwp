//! [#8573] 1단 영역이 위아래로 쌓인 쪽에서 공백 host 자리차지 표가 앞 본문을 밀지 않는다.
//!
//! 재현물(공개 샘플 변형, 저장 LINE_SEG 없음):
//!
//! ```text
//! 문단 0: "Q"
//! 문단 1: 공백 7칸 host + 자리차지 표(문단 기준 세로 559HU, 6행)
//! 문단 2: "A"
//! 문단 3: 단 정의(1단) → 같은 쪽 둘째 1단 영역의 첫 문단 "Z"
//! 문단 4: "Y"
//! ```
//!
//! 웹한글기안기 인쇄본(`pdf/issue8573/whitespace-host-topbottom-stacked-zones-webhwp-print.pdf`)
//! 에서 "Q" 는 본문 맨 위, 표는 "Q" 아래, "A"·"Z"·"Y" 는 표 뒤에 차례로 놓인다. 단이 나란히
//! 서지 않는 쪽에서 문단 기준 표를 단 위끝에서 잰 전폭 예약이 "Q" 를 표 높이만큼 밀면 안 되고,
//! 둘째 영역 첫 문단이 앞 영역 표 높이만큼 다시 밀려도 안 된다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8573/whitespace-host-topbottom-stacked-zones.hwp";
const TOL_PX: f64 = 0.5;

#[derive(Debug, Clone, Copy)]
struct Span {
    top: f64,
    bottom: f64,
}

#[derive(Default, Debug)]
struct Page {
    columns: Vec<Span>,
    tables: Vec<(usize, Span)>,
    lines: Vec<(usize, Span)>,
}

fn first_page() -> Page {
    fn span(n: &RenderNode) -> Span {
        Span {
            top: n.bbox.y,
            bottom: n.bbox.y + n.bbox.height,
        }
    }
    fn walk(n: &RenderNode, page: &mut Page) {
        match &n.node_type {
            RenderNodeType::Column(_) => page.columns.push(span(n)),
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
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut page = Page::default();
    walk(&tree.root, &mut page);
    page
}

fn line(page: &Page, pi: usize) -> Span {
    page.lines
        .iter()
        .filter(|(p, _)| *p == pi)
        .map(|(_, s)| *s)
        .min_by(|a, b| a.top.total_cmp(&b.top))
        .unwrap_or_else(|| panic!("1쪽에 문단 {pi} 글줄: {:?}", page.lines))
}

fn table(page: &Page) -> Span {
    page.tables
        .iter()
        .find(|(p, _)| *p == 1)
        .map(|(_, s)| *s)
        .unwrap_or_else(|| panic!("1쪽에 문단 1 표: {:?}", page.tables))
}

#[test]
fn stacked_single_column_zones_share_one_page() {
    let page = first_page();
    assert!(
        page.columns.len() >= 2,
        "재현물 1쪽은 1단 영역 둘이 위아래로 쌓인다: {:?}",
        page.columns
    );
}

#[test]
fn first_paragraph_is_not_pushed_by_paragraph_table_reservation() {
    let page = first_page();
    let column_top = page
        .columns
        .iter()
        .map(|c| c.top)
        .reduce(f64::min)
        .expect("단");
    let q = line(&page, 0);
    let t = table(&page);
    assert!(
        (q.top - column_top).abs() <= TOL_PX,
        "문단 0 글줄 위({:.2})는 본문 맨 위({column_top:.2})다 — 표 높이 예약에 밀리지 않는다",
        q.top
    );
    assert!(
        t.top + TOL_PX >= q.bottom,
        "문단 1 표 위({:.2})는 문단 0 글줄 아래({:.2}) 뒤다",
        t.top,
        q.bottom
    );
}

#[test]
fn second_zone_first_paragraph_follows_previous_zone_without_table_height() {
    let page = first_page();
    let t = table(&page);
    let a = line(&page, 2);
    let z = line(&page, 3);
    assert!(
        a.top + TOL_PX >= t.bottom,
        "문단 2 는 표 아래({:.2})에 온다: {:.2}",
        t.bottom,
        a.top
    );
    assert!(
        z.top + TOL_PX >= a.bottom,
        "둘째 영역 첫 문단({:.2})은 문단 2 아래({:.2}) 뒤다",
        z.top,
        a.bottom
    );
    let table_height = t.bottom - t.top;
    assert!(
        z.top - a.bottom < table_height,
        "둘째 영역 첫 문단({:.2})이 앞 영역 표 높이({table_height:.2})만큼 다시 밀리지 않는다(문단 2 아래 {:.2})",
        z.top,
        a.bottom
    );
}
