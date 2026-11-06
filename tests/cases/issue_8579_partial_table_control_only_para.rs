//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-06T11:54:54+09:00; 직전 파일 커밋: bf7a74526f6ff3290a1672ffee5bb86e3cb741a8 (2026-11-04T21:00:19+09:00).
//! 2026-11-06 Visual Sweep: 96dpi·PDFium·색상 경계 보정 전 2px raw 실루엣.
//! 같은 입력의 저장 버전에 맞는 Desktop Print(2020→compat 2022, 2024→compat 2024)와 대조.
//! partial_table_control_only_para: 전 2쪽 Native 100.00000%,100.00000% / fresh optimized WASM 100.00000%,100.00000%; 전쪽 최저 100.00000% / 100.00000%.
//! partial_table_following_text: 전 2쪽 Native 95.27771%,100.00000% / fresh optimized WASM 95.18540%,100.00000%; 전쪽 최저 95.27771% / 95.18540%.
//! 증거: mydocs/pr/assets/hyunmyungjanelee-20261106/{native,wasm}-<입력-stem>.tsv,
//! input-reference-pairs.json·측정 JSON·대표 PNG 및 해당 원 PR review 문서.
//! Native/fresh WASM의 직접 확대 판독과 입력·PDF·제품 해시는 공통 기록에 연결한다.
//!
//! #8579: 쪽 나눔 표 경로의 저장 줄 없는 개체 전용 칸 문단.
//!
//! HWP5 칸 문단이 글자 없이 개체만 들고 `LINE_SEG` 가 없으면 줄 수가 0 이다.
//! 쪽에 걸친 표(행 14개)의 온전한 행에서 이 문단이
//! - 글자처럼 취급 직선만 가지면 건너뛰어 직선 노드가 생기지 않았고,
//! - 문단 기준 어울림 그림을 가지면 원점이 문단 끝(빈 줄 하나 아래)으로 잡혔다.
//!
//! 재현물은 공개 샘플(test-image.hwpx, issue3738, issue5789)을 조합해 저장 줄을
//! 걷어 낸 HWP5 이다. 행 높이는 내용보다 넉넉히 선언했고(칸 세로 정렬 위),
//! 그림은 모두 칸 폭보다 좁으며 그림 뒤에 글 문단을 두지 않았다. 웹한글기안기 인쇄본:
//! `pdf/issue8579/partial_table_control_only_para-webhwp-print.pdf` — 화살표가
//! 칸마다 있고, 그림 위는 소유 문단 위 + 저장 세로 오프셋이다.
//!
//! 짝수 행: c0 직선 | c1 그림(p0) | c2 캡션(p0) + 그림 문단(p1, 55HU) | c3 그림(p0).
//! 홀수 행: 네 칸 모두 직선.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

const HU_PER_PX: f64 = 7200.0 / 96.0;

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8579/partial_table_control_only_para.hwp");
    DocumentCore::from_bytes(&std::fs::read(path).expect("required fixture")).expect("HWP parse")
}

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = vec![node];
    for child in &node.children {
        out.extend(nodes(child));
    }
    out
}

struct Cell<'a> {
    row: u16,
    col: u16,
    bbox: BoundingBox,
    node: &'a RenderNode,
}

fn cells(root: &RenderNode) -> Vec<Cell<'_>> {
    nodes(root)
        .into_iter()
        .filter_map(|n| match &n.node_type {
            RenderNodeType::TableCell(c) => Some(Cell {
                row: c.row,
                col: c.col,
                bbox: n.bbox,
                node: n,
            }),
            _ => None,
        })
        .collect()
}

fn line_count(cell: &RenderNode) -> usize {
    nodes(cell)
        .into_iter()
        .filter(|n| matches!(n.node_type, RenderNodeType::Line(_)))
        .count()
}

fn first_line_top(cell: &RenderNode, para_index: usize) -> Option<f64> {
    nodes(cell).into_iter().find_map(|n| match &n.node_type {
        RenderNodeType::TextLine(line) if line.para_index == Some(para_index) => Some(n.bbox.y),
        _ => None,
    })
}

fn image_top(cell: &RenderNode) -> Option<f64> {
    nodes(cell).into_iter().find_map(|n| match &n.node_type {
        RenderNodeType::Image(_) => Some(n.bbox.y),
        _ => None,
    })
}

struct Probe {
    page: u32,
    row: u16,
    col: u16,
    bbox: BoundingBox,
    lines: usize,
    image: Option<f64>,
    anchors: [Option<f64>; 2],
}

/// 행 전체가 이 쪽에 실린 칸만 검사한다(쪽 경계 조각은 별도 계약).
fn whole_row_cells(core: &DocumentCore) -> Vec<Probe> {
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("page tree");
        for cell in cells(&tree.root) {
            out.push(Probe {
                page,
                row: cell.row,
                col: cell.col,
                bbox: cell.bbox,
                lines: line_count(cell.node),
                image: image_top(cell.node),
                anchors: [first_line_top(cell.node, 0), first_line_top(cell.node, 1)],
            });
        }
    }
    // 두 쪽에 나뉘어 실린 행은 조각 소유 계약이 따로 있어 제외한다.
    let split_rows: Vec<u16> = out
        .iter()
        .filter(|p| out.iter().any(|q| q.row == p.row && q.page != p.page))
        .map(|p| p.row)
        .collect();
    out.retain(|p| !split_rows.contains(&p.row));
    out
}

#[test]
fn tac_line_only_cell_paragraph_emits_its_line_in_every_whole_row() {
    let core = core();
    assert!(core.page_count() > 1, "표가 쪽 나눔 경로를 타야 한다");
    let mut checked = 0;
    for p in whole_row_cells(&core) {
        if p.row % 2 == 1 || p.col == 0 {
            assert!(
                p.lines >= 1,
                "p{} r{}c{}: 글자처럼 취급 직선이 그려져야 한다 (bbox={:?})",
                p.page + 1,
                p.row,
                p.col,
                p.bbox
            );
            checked += 1;
        }
    }
    assert!(checked >= 10, "검사한 직선 칸이 너무 적다: {checked}");
}

#[test]
fn unstored_picture_paragraph_anchors_at_its_paragraph_top() {
    let core = core();
    let mut checked = 0;
    for p in whole_row_cells(&core) {
        if p.row % 2 == 1 || p.col == 0 {
            continue;
        }
        let (anchor_para, offset_hu) = match p.col {
            1 | 3 => (0, 0.0),
            2 => (1, 55.0),
            _ => continue,
        };
        let at = format!("p{} r{}c{}", p.page + 1, p.row, p.col);
        let image = p.image.unwrap_or_else(|| panic!("{at}: 그림 없음"));
        let top = p.anchors[anchor_para].unwrap_or_else(|| panic!("{at}: 소유 문단 줄 없음"));
        let expected = top + offset_hu / HU_PER_PX;
        assert!(
            (image - expected).abs() < 1.0,
            "{at}: 그림 위 {image:.1} 는 소유 문단 위 {top:.1} + 오프셋 = {expected:.1} 이어야 한다"
        );
        assert!(image >= p.bbox.y, "{at}: 그림이 칸 위를 넘는다");
        checked += 1;
    }
    assert!(checked >= 6, "검사한 그림 칸이 너무 적다: {checked}");
}
