//! [#8573] 저장 줄 없는 공백 host 의 음수 세로 오프셋 자리차지 표는 쪽 위 밖으로 사라지지 않는다.
//!
//! 재현물(공개 샘플 변형, 저장 LINE_SEG 없음):
//!
//! ```text
//! (1) 표 하나: 문단 0 "Q" / 문단 1 공백 7칸 host + 자리차지 표(문단 기준 세로 -54774HU, 바깥 여백 0) /
//!     문단 2 "A"
//! (2) 표 둘:   문단 0 "Q" / 문단 1 공백 7칸 host + 결재 표(단 기준 오른쪽, 세로 -54774HU) +
//!     제목 띠 표(문단 기준 왼쪽, 세로 3200HU = 결재 표 높이) / 두 표 바깥 여백 0
//! ```
//!
//! 웹한글기안기 인쇄본(`pdf/issue8573/whitespace-host-{negative-offset-table,two-tables-negative-offset}
//! -webhwp-print.pdf`)은 음수 오프셋 표를 문단 위끝("Q" 바로 아래)에 놓는다. 저장 줄 없는 "X" host
//! 로 바꿔도 같다. rhwp 의 글자 host 경로는 오프셋을 그대로 써 표를 쪽 위 밖(-576px)으로
//! 보내므로, 공백 host 를 그 경로로 보내면 표가 화면에서 사라진다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const ONE: &str = "samples/issue8573/whitespace-host-negative-offset-table.hwp";
const TWO: &str = "samples/issue8573/whitespace-host-two-tables-negative-offset.hwp";
const TOL_PX: f64 = 0.5;

#[derive(Debug, Clone, Copy)]
struct Span {
    top: f64,
    bottom: f64,
}

#[derive(Default, Debug)]
struct Page {
    body: Option<Span>,
    tables: Vec<(usize, Span)>,
    lines: Vec<(usize, Span)>,
}

fn first_page(sample: &str) -> Page {
    fn span(n: &RenderNode) -> Span {
        Span {
            top: n.bbox.y,
            bottom: n.bbox.y + n.bbox.height,
        }
    }
    fn walk(n: &RenderNode, page: &mut Page) {
        match &n.node_type {
            RenderNodeType::Body { .. } => page.body = Some(span(n)),
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
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
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
        .find(|(p, _)| *p == pi)
        .map(|(_, s)| *s)
        .unwrap_or_else(|| panic!("1쪽에 문단 {pi} 글줄: {:?}", page.lines))
}

fn assert_host_tables_stay_below_first_line(sample: &str, expected_tables: usize) {
    let page = first_page(sample);
    let body = page.body.expect("본문 영역");
    let q = line(&page, 0);
    let host_tables: Vec<Span> = page
        .tables
        .iter()
        .filter(|(p, _)| *p == 1)
        .map(|(_, s)| *s)
        .collect();
    assert_eq!(
        host_tables.len(),
        expected_tables,
        "{sample}: 문단 1 표 {expected_tables}개가 1쪽에 그려진다: {:?}",
        page.tables
    );
    for t in host_tables {
        assert!(
            t.top + TOL_PX >= q.bottom && t.bottom <= body.bottom + TOL_PX,
            "{sample}: 음수 오프셋 표({:.2}~{:.2})는 문단 0 글줄 아래({:.2})부터 본문 안에 놓인다",
            t.top,
            t.bottom,
            q.bottom
        );
    }
}

#[test]
fn single_negative_offset_table_stays_on_page() {
    assert_host_tables_stay_below_first_line(ONE, 1);
}

#[test]
fn two_tables_with_negative_offset_sign_table_stay_on_page() {
    assert_host_tables_stay_below_first_line(TWO, 2);
}
