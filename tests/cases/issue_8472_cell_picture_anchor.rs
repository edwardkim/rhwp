//! [#8472] 관계형 회귀 — 칸 안 자리차지(TopAndBottom, vert=문단) 그림.
//!
//! 1. 본문 직속 비-TAC 표의 칸 높이는 저장 사다리(다음 문단 vpos = 앵커 vpos + 그림 높이)를 따른다.
//!    그림을 단 빈 문단의 줄 전진을 그림 위에 한 번 더 쌓지 않는다.
//! 2. 그런 행을 쪽 끝에서 나눌 때 그림을 단 문단의 글줄과 그림 몫은 한 조각에 함께 간다.
//!
//! - 갱신: 2026-11-02 (task/8472-cell-picture-anchor)
//! - 입력: `samples/issue8472/cell_picture_row_split.hwp` — 1382000-202100018 종합보고서를 한/글 2022 로
//!   HWPX 저장 → 문단 0·178·179·179·180·182·183 만 남김 → 마지막 차트 표 두 칸의 그림 앞에 라벨
//!   줄 2개 복제(`samples/issue8472/gen_picrow.py`, fills=0 before=2 after=0) → 한/글 2022 가 HWP 로
//!   다시 저장(저장 줄은 한/글이 조판).
//! - 독립 기준: 한/글 2022(12.0.0.4547) Print `pdf/issue8472_cell_picture_row_split-2020.pdf` —
//!   1쪽 끝에 둘째 표 라벨 «[ 남성 ]»·«[ 여성 ]» 각 3줄, 2쪽 첫머리에 그 두 그림.
//! - 측정(code 06cbf26801, `silhouette_raw_match_percent` 1/2쪽): Native 90.21/100 · fresh WASM
//!   90.55/100 — 최저 90.21(1쪽). 1쪽 남은 차이: 한/글은 뒤 표 첫 조각 상자를 쪽 바닥까지 닫고
//!   라벨을 그 안에서 세로 가운데 정렬하는데, rhwp 는 라벨 높이로만 닫는다(미해결, #8472 비범위).
//!   수정 전 Native 92.00/70.61(2쪽 그림이 조각 밖).
//!   증거 `mydocs/pr/assets/issue_8472/{native,wasm}/fixture.tsv`.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8472/cell_picture_row_split.hwp";

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse {SAMPLE}: {e:?}"))
}

fn walk<'a>(node: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
    out.push(node);
    for child in &node.children {
        walk(child, out);
    }
}

/// 쪽 `page` 에서 문단 `para` 가 소유한 표 노드들.
fn tables_of<'a>(nodes: &[&'a RenderNode], para: usize) -> Vec<&'a RenderNode> {
    nodes
        .iter()
        .copied()
        .filter(|node| {
            matches!(&node.node_type, RenderNodeType::Table(t)
                if t.para_index == Some(para) && t.cell_context.is_none())
        })
        .collect()
}

fn subtree_text(node: &RenderNode) -> String {
    let mut nodes = Vec::new();
    walk(node, &mut nodes);
    nodes
        .iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TextRun(run) => Some(run.text.as_str()),
            _ => None,
        })
        .collect()
}

fn subtree_images(node: &RenderNode) -> usize {
    let mut nodes = Vec::new();
    walk(node, &mut nodes);
    nodes
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Image(_)))
        .count()
}

/// 본문의 1×2 차트 표를 단 문단 번호(앞 표, 뒤 표).
fn chart_paras(core: &DocumentCore) -> (usize, usize) {
    let paras: Vec<usize> = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .filter(|(_, p)| {
            p.controls.iter().any(|c| {
                matches!(c, Control::Table(t) if t.row_count == 1 && t.col_count == 2
                    && !t.common.treat_as_char)
            })
        })
        .map(|(i, _)| i)
        .collect();
    assert_eq!(paras.len(), 2, "차트 표 두 개: {paras:?}");
    (paras[0], paras[1])
}

#[test]
fn chart_cell_height_follows_stored_ladder() {
    let core = core();
    let (first, _) = chart_paras(&core);
    let para = &core.document().sections[0].paragraphs[first];
    let table = para
        .controls
        .iter()
        .find_map(|c| match c {
            Control::Table(t) => Some(t),
            _ => None,
        })
        .unwrap();
    // 저장 사다리 끝 + 칸 위아래 여백(HU) — 한/글이 저장한 내용 상자.
    let stored_hu = table
        .cells
        .iter()
        .map(|cell| {
            let end = cell
                .paragraphs
                .iter()
                .flat_map(|p| p.line_segs.iter())
                .map(|seg| seg.vertical_pos + seg.line_height)
                .max()
                .unwrap_or(0);
            end + i32::from(cell.padding.top) + i32::from(cell.padding.bottom)
        })
        .max()
        .unwrap();
    let stored_px = stored_hu as f64 / 75.0;
    let tree = core.build_page_render_tree(0).expect("렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let tables = tables_of(&nodes, first);
    assert_eq!(tables.len(), 1, "앞 차트 표는 1쪽에 통째로");
    let painted = tables[0].bbox.height;
    assert!(
        (painted - stored_px).abs() <= 1.0,
        "앞 차트 표 높이 {painted:.1}px 는 저장 내용 상자 {stored_px:.1}px 와 같아야 한다(앵커 줄 이중 계상 금지)"
    );
}

#[test]
fn picture_stays_with_its_anchor_paragraph_across_page_split() {
    let core = core();
    let (_, second) = chart_paras(&core);
    assert_eq!(core.page_count(), 2, "한/글 Print 쪽수 2");
    let page_tables = |page: u32| {
        let tree = core.build_page_render_tree(page).expect("렌더 트리");
        let mut nodes = Vec::new();
        walk(&tree.root, &mut nodes);
        tables_of(&nodes, second)
            .iter()
            .map(|t| (subtree_text(t), subtree_images(t)))
            .collect::<Vec<_>>()
    };
    let first_page = page_tables(0);
    let second_page = page_tables(1);
    assert_eq!(first_page.len(), 1, "뒤 차트 표의 첫 조각은 1쪽 끝");
    assert_eq!(second_page.len(), 1, "이어지는 조각은 2쪽");
    let (text0, images0) = &first_page[0];
    let (text1, images1) = &second_page[0];
    assert_eq!(
        text0.matches("남성").count(),
        3,
        "1쪽에 라벨 «[ 남성 ]» 3줄: {text0}"
    );
    assert_eq!(
        text0.matches("여성").count(),
        3,
        "1쪽에 라벨 «[ 여성 ]» 3줄: {text0}"
    );
    assert_eq!(
        *images0, 0,
        "그림은 앵커 문단과 함께 2쪽으로 간다(1쪽에 그림 없음)"
    );
    assert_eq!(*images1, 2, "2쪽 조각이 두 그림을 소유한다");
    assert!(
        !text1.contains("남성") && !text1.contains("여성"),
        "라벨 줄은 다시 그리지 않는다: {text1}"
    );

    // 그림은 앵커 문단과 같은 조각의 칸 상자 안에 있다 — 앵커 줄을 앞 조각에 두면 그림이
    // 이어지는 조각의 위변 밖으로 올라가고 조각 상자가 그림보다 작아진다.
    let tree = core.build_page_render_tree(1).expect("렌더 트리");
    let mut nodes = Vec::new();
    walk(&tree.root, &mut nodes);
    let table = tables_of(&nodes, second)[0];
    let mut inside = Vec::new();
    walk(table, &mut inside);
    let top = table.bbox.y;
    let bottom = table.bbox.y + table.bbox.height;
    for image in inside
        .iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Image(_)))
    {
        let (y0, y1) = (image.bbox.y, image.bbox.y + image.bbox.height);
        assert!(
            y0 >= top - 0.01 && y1 <= bottom + 0.01,
            "그림 {y0:.1}..{y1:.1} 은 이어지는 표 조각 {top:.1}..{bottom:.1} 안에 있어야 한다"
        );
    }
}
