//! 관계형·고정형 — [#9216] 칸 첫 문단 바로 뒤의 저장 재시작은 쪽 경계다.
//! 주석 갱신: 2026-11-20 11:06 KST (새 파일)
//!
//! 입력: 공개 입법예고 첨부(교육부 「규제영향분석서」, hwp5, 10쪽)
//! `samples/issue9216/regulatory-impact-nested-row-restart.hwp`.
//! 기준: 한컴 engine 2020 Print PDF `pdf/issue9216/regulatory-impact-nested-row-restart-2020.pdf`(10쪽).
//! `pi=37` 은 RowBreak 1×1 표이고 칸 안에 3×3 중첩 표가 있다. 중첩 표 3행 가운데 칸의 저장 줄은
//! p[0] `vpos 0·1624·3248`, p[1] `vpos 0`(높이 600·간격 152), p[2] `vpos 752·2376·4000` 이다. p[1] 이
//! 0 으로 돌아가고 p[2] 가 p[1] 의 끝 752 에서 이어지므로 한/글은 p[0] 뒤에서 쪽을 넘겼다(PDF 4쪽은
//! p[0] 세 줄로 끝나고 5쪽은 p[2] 로 시작한다). 수정 전 rhwp 는 앞 문단 첫 줄이 양수 원점일 때만
//! 재시작을 인정해 이 행을 통째로 5쪽으로 넘겼고, 넘어온 행이 다음 제목 문단과 겹쳤다.
//! - 고정형: 전체 쪽수 10(= PDF).
//! - 관계형: p[0] 글은 4쪽, p[2] 글은 5쪽에 있다. 5쪽에서 다음 제목 문단(`pi=40`)은 이어진 표 조각
//!   아래에 있다.
//!
//! 측정(2026-11-20, 이 파일과 같은 커밋의 코드, base = upstream/devel `0a437a7eef`,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴 `ttfs/opensource` — 문서 본 글꼴 없는 환경):
//! - Native: 10쪽 / 10쪽, 4쪽 96.30(base 92.21)·5쪽 97.86(base 87.96), 전쪽 최저 96.30(4쪽)
//! - fresh WASM(`scripts/wasm-pack-locked.sh --target web`): 10쪽 / 10쪽, 4쪽 96.98·5쪽 99.21, 전쪽 최저 96.98(4쪽)
//! - 증거: `mydocs/pr/assets/issue_9216/`(native·wasm 전쪽 TSV, 4·5쪽 review·overlay PNG)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue9216/regulatory-impact-nested-row-restart.hwp";
/// 한컴 engine 2020 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 10;
/// 나뉜 표의 호스트 문단과 그 뒤 제목 문단.
const SPLIT_TABLE_PARA: usize = 37;
/// 중첩 표 3행 가운데 칸의 p[0]·p[2] 에만 있는 글.
const BEFORE_RESTART: &str = "미비점";
const AFTER_RESTART: &str = "교육여건";
const NEXT_HEADING: &str = "이해관계자";

fn runs(node: &RenderNode, out: &mut Vec<(f64, f64, String)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push((
            node.bbox.y,
            node.bbox.y + node.bbox.height,
            run.text.clone(),
        ));
    }
    for child in &node.children {
        runs(child, out);
    }
}

fn find<'a>(node: &'a RenderNode, pred: &dyn Fn(&RenderNode) -> bool) -> Option<&'a RenderNode> {
    if pred(node) {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, pred))
}

/// 나뉜 표 칸 안 글줄 가운데 `needle` 을 품은 줄의 수.
fn table_lines(core: &DocumentCore, page: u32, needle: &str) -> usize {
    let tree = core.build_page_render_tree(page).expect("쪽 렌더 트리");
    let Some(table) = find(&tree.root, &|n| {
        matches!(&n.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(SPLIT_TABLE_PARA) && t.cell_context.is_none())
    }) else {
        return 0;
    };
    let mut all = Vec::new();
    runs(table, &mut all);
    all.iter()
        .filter(|(_, _, text)| text.contains(needle))
        .count()
}

#[test]
fn nested_row_splits_at_restart_after_first_cell_paragraph() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );

    // 재시작 앞 문단은 4쪽, 뒤 문단은 5쪽에 한 번씩만 있다.
    assert_eq!(
        (
            table_lines(&core, 3, BEFORE_RESTART),
            table_lines(&core, 4, BEFORE_RESTART)
        ),
        (1, 0),
        "재시작 앞 문단은 4쪽 표 조각에만 있어야 한다"
    );
    assert_eq!(
        (
            table_lines(&core, 3, AFTER_RESTART),
            table_lines(&core, 4, AFTER_RESTART)
        ),
        (0, 1),
        "재시작 뒤 문단은 5쪽 표 조각에만 있어야 한다"
    );

    // 5쪽의 다음 제목 문단은 이어진 표 조각 아래에 있다.
    let tree = core.build_page_render_tree(4).expect("5쪽");
    let table = find(&tree.root, &|n| {
        matches!(&n.node_type, RenderNodeType::Table(t)
            if t.para_index == Some(SPLIT_TABLE_PARA) && t.cell_context.is_none())
    })
    .expect("5쪽에 이어진 표 조각이 있어야 한다");
    let mut all = Vec::new();
    runs(&tree.root, &mut all);
    let heading_top = all
        .iter()
        .filter(|(_, _, text)| text.contains(NEXT_HEADING))
        .map(|(top, _, _)| *top)
        .fold(f64::INFINITY, f64::min);
    let table_bottom = table.bbox.y + table.bbox.height;
    assert!(
        heading_top.is_finite() && heading_top >= table_bottom - 0.5,
        "다음 제목 문단 위 {heading_top:.2} 은 이어진 표 조각 아래 {table_bottom:.2} 보다 아래여야 한다"
    );
}
