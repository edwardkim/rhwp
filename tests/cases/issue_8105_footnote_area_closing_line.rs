//! [#8105] 각주가 이미 놓인 쪽에서 쪽을 넘는 표의 첫 조각은 각주 영역 위까지 담는다.
//!
//! 한/글 실측(사다리 D, 각주 변형 66/66): 쪽 끝 경계는 각주 구분선 위 여백까지 포함한 각주 영역의
//! 위다 — 표 경로가 따로 두던 보수 안전 띠(3000HU)는 없다.
//! 1130000 가맹 보고서 50쪽(정본 `pdf/pr_6088_6144/hancom2020/…franchise_review_report-2020.pdf`):
//! 표 `pi=532` 행 0 이 각주 위에 다 들어 51쪽은 행 1 로 시작한다. 수정 전 rhwp 는 띠만큼 일찍 잘라
//! 행 0 의 끝 줄 «…해석함이 타당하다.» 가 51쪽으로 넘어갔고 51–54쪽이 한 행씩 밀렸다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE_1130000: &str = "samples/issue5966/1130000-202100008_franchise_review_report.hwp";

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE_1130000);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("parse 1130000")
}

fn tree(core: &DocumentCore, page: u32) -> RenderNode {
    core.build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("{}쪽 렌더 트리: {error:?}", page + 1))
        .root
}

/// 쪽의 글줄 텍스트(공백 제거).
fn page_lines(core: &DocumentCore, page: u32) -> Vec<String> {
    fn text(node: &RenderNode, out: &mut String) {
        if let RenderNodeType::TextRun(run) = &node.node_type {
            out.extend(run.text.chars().filter(|c| !c.is_whitespace()));
        }
        for child in &node.children {
            text(child, out);
        }
    }
    fn walk(node: &RenderNode, out: &mut Vec<String>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            let mut line = String::new();
            text(node, &mut line);
            out.push(line);
            return;
        }
        for child in &node.children {
            walk(child, out);
        }
    }
    let mut out = Vec::new();
    walk(&tree(core, page), &mut out);
    out
}

/// 표 `(구역, 문단, 개체)` 가 이 쪽에 그린 칸의 행 번호.
fn table_rows(core: &DocumentCore, page: u32, table: (usize, usize, usize)) -> Vec<u16> {
    fn walk(node: &RenderNode, inside: bool, table: (usize, usize, usize), out: &mut Vec<u16>) {
        let inside = match &node.node_type {
            RenderNodeType::Table(t) => {
                (t.section_index, t.para_index, t.control_index)
                    == (Some(table.0), Some(table.1), Some(table.2))
            }
            RenderNodeType::TableCell(cell) if inside => {
                out.push(cell.row);
                false
            }
            _ => inside,
        };
        for child in &node.children {
            walk(child, inside, table, out);
        }
    }
    let mut out = Vec::new();
    walk(&tree(core, page), false, table, &mut out);
    out
}

#[test]
fn footnote_page_limit_is_the_footnote_area_top() {
    let core = core();
    let line = "경우에한하여공정한거래를저해할우려가있는것으로해석함이타당하다.";
    assert!(
        page_lines(&core, 49).iter().any(|l| l == line),
        "50쪽이 행 0 의 끝 줄 `{line}` 을 담는다(정본)"
    );
    assert!(
        !page_lines(&core, 50).iter().any(|l| l == line),
        "51쪽은 그 줄을 되풀이하지 않는다"
    );
    let rows_on_51 = table_rows(&core, 50, (0, 532, 0));
    assert!(
        !rows_on_51.is_empty() && !rows_on_51.contains(&0),
        "51쪽은 행 1 로 시작한다(행 0 은 50쪽에서 끝남, 정본): {rows_on_51:?}"
    );
}
