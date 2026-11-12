//! 쪽 나눔 표의 저장 줄 없는 칸 문단에 글자처럼 취급이 **아닌** 직선(화살표)만 있을 때.
//!
//! #8579 는 같은 칸 문단(글자 없음, `LINE_SEG` 없음 → 줄 수 0)이 쪽에 걸친 표의 온전한 행에서
//! 건너뛰어지던 결함을 표·그림·글자처럼 취급 도형에 대해 고쳤다. 주간 일과표 서식은 칸 안
//! 화살표를 문단 기준 자리차지 직선(세로 오프셋 600HU)으로 두므로 그 구제를 받지 못해, 쪽 경계
//! 조각이 아닌 행의 화살표가 모두 사라졌다.
//!
//! 재현물은 #8579 생성기를 변형해 공개 샘플(test-image.hwpx, issue3738, issue5789)을 조합하고
//! 저장 줄을 걷어 낸 HWP5 다. 행 14개, 4열. 짝수 행: c0 글자 | c1 직선 | c2 그림 | c3 직선,
//! 홀수 행: c0 글자 | c1 그림 | c2 직선 | c3 직선. 칸 세로 정렬은 위(`valign_top`)와
//! 가운데(`valign_center`) 두 벌이다. Windows 한컴 2020 MCP Print
//! (`pdf/issue8690/*-hwp-2020.pdf`)에서 직선은 두 쪽의 모든 칸에 그려진다.
//!
//! 셋째·넷째 재현물(`rowspan_infront_top`, `rowspan_infront_center`)은 직선을 글앞으로 두고,
//! 두 행 묶음마다 c1·c3 를 2행 병합 칸으로 둔다(짝수 행: c0 글자 | c1 직선(병합) | c2 직선 |
//! c3 직선(병합), 홀수 행: c0 글자 | c2 직선). 칸 세로 정렬은 위와 가운데다. 병합 칸은 쪽 경계에 걸리지 않아도 칸 전체를 덮는 컷을
//! 받고, 글앞 도형만 든 문단은 빈 줄로 분류되어 그 컷의 빈 줄 소유에서 빠져 건너뛰어졌다.
//! 같은 Windows 한컴 Print에서 병합 칸 직선도 모두 그려진다.
//!
//! 검사: 쪽 경계 조각이 아닌 행의 직선 칸마다 직선이 하나 이상 있고, 그 직선이 자기 칸 안에
//! 있다. 위 정렬에서는 칸 위쪽 절반, 가운데 정렬에서는 칸 세로 가운데 띠 안에 있다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn core(name: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8690")
        .join(name);
    DocumentCore::from_bytes(&std::fs::read(path).expect("required fixture")).expect("HWP parse")
}

fn nodes(node: &RenderNode) -> Vec<&RenderNode> {
    let mut out = vec![node];
    for child in &node.children {
        out.extend(nodes(child));
    }
    out
}

struct Probe {
    page: u32,
    row: u16,
    col: u16,
    cell: BoundingBox,
    lines: Vec<BoundingBox>,
}

fn whole_row_cells(core: &DocumentCore) -> Vec<Probe> {
    let mut out = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("page tree");
        for n in nodes(&tree.root) {
            if let RenderNodeType::TableCell(c) = &n.node_type {
                out.push(Probe {
                    page,
                    row: c.row,
                    col: c.col,
                    cell: n.bbox,
                    lines: nodes(n)
                        .into_iter()
                        .filter(|l| matches!(l.node_type, RenderNodeType::Line(_)))
                        .map(|l| l.bbox)
                        .collect(),
                });
            }
        }
    }
    let split_rows: Vec<u16> = out
        .iter()
        .filter(|p| out.iter().any(|q| q.row == p.row && q.page != p.page))
        .map(|p| p.row)
        .collect();
    out.retain(|p| !split_rows.contains(&p.row));
    out
}

fn is_line_cell(row: u16, col: u16) -> bool {
    if row.is_multiple_of(2) {
        col == 1 || col == 3
    } else {
        col == 2 || col == 3
    }
}

/// 직선 칸마다 칸 안에 직선이 있는지 보고, 직선 세로 중심의 칸 안 상대 위치(0=위, 1=아래)를 돌려준다.
fn check_lines(name: &str) -> Vec<f64> {
    check_lines_in(name, is_line_cell)
}

fn check_lines_in(name: &str, is_line_cell: fn(u16, u16) -> bool) -> Vec<f64> {
    let core = core(name);
    assert!(
        core.page_count() > 1,
        "{name}: 표가 쪽 나눔 경로를 타야 한다"
    );
    let mut rel = Vec::new();
    for p in whole_row_cells(&core) {
        if !is_line_cell(p.row, p.col) {
            continue;
        }
        let at = format!("{name} p{} r{}c{}", p.page + 1, p.row, p.col);
        let line = p.lines.first().unwrap_or_else(|| {
            panic!(
                "{at}: 글자처럼 취급이 아닌 직선이 그려져야 한다 (칸 {:?})",
                p.cell
            )
        });
        assert!(
            line.x >= p.cell.x - 0.5
                && line.x + line.width <= p.cell.x + p.cell.width + 0.5
                && line.y >= p.cell.y - 0.5
                && line.y + line.height <= p.cell.y + p.cell.height + 0.5,
            "{at}: 직선 {line:?} 이 칸 {:?} 안에 있어야 한다",
            p.cell
        );
        rel.push((line.y + line.height / 2.0 - p.cell.y) / p.cell.height);
    }
    assert!(
        rel.len() >= 16,
        "{name}: 검사한 직선 칸이 너무 적다: {}",
        rel.len()
    );
    rel
}

#[test]
fn float_line_cell_paragraph_emits_line_in_every_whole_row_top_aligned() {
    for r in check_lines("float_line_valign_top.hwp") {
        assert!(
            r < 0.5,
            "위 정렬 칸의 직선은 칸 위쪽 절반에 있어야 한다: {r:.2}"
        );
    }
}

#[test]
fn float_line_cell_paragraph_emits_line_in_every_whole_row_center_aligned() {
    for r in check_lines("float_line_valign_center.hwp") {
        assert!(
            (0.25..=0.75).contains(&r),
            "가운데 정렬 칸의 직선은 칸 세로 가운데 띠에 있어야 한다: {r:.2}"
        );
    }
}

fn is_rowspan_line_cell(row: u16, col: u16) -> bool {
    if row.is_multiple_of(2) {
        (1..=3).contains(&col)
    } else {
        col == 2
    }
}

#[test]
fn infront_line_in_rowspan_cell_is_emitted_top_aligned() {
    for r in check_lines_in("float_line_rowspan_infront_top.hwp", is_rowspan_line_cell) {
        assert!(
            r < 0.5,
            "위 정렬 칸의 직선은 칸 위쪽 절반에 있어야 한다: {r:.2}"
        );
    }
}

#[test]
fn infront_line_in_rowspan_cell_is_emitted_center_aligned() {
    let rel = check_lines_in(
        "float_line_rowspan_infront_center.hwp",
        is_rowspan_line_cell,
    );
    for r in rel {
        assert!(
            (0.25..=0.75).contains(&r),
            "가운데 정렬 칸의 직선은 칸 세로 가운데 띠에 있어야 한다: {r:.2}"
        );
    }
}

#[test]
fn centered_shape_anchor_keeps_the_source_font_line_box() {
    // 관계형: 독립 Windows 한컴 Print에서 확인한 원본 글자 줄과 도형 띠의 소유다.
    // 고정한 PDF 좌표가 아니라 소스 em·안 여백·오프셋과 실제 셀 상자를 대조한다.
    for name in [
        "float_line_valign_center.hwp",
        "float_line_rowspan_infront_center.hwp",
    ] {
        let core = core(name);
        let document = core.document();
        let table = document
            .sections
            .iter()
            .flat_map(|section| &section.paragraphs)
            .flat_map(|para| &para.controls)
            .find_map(|control| match control {
                rhwp::model::control::Control::Table(table) => Some(table),
                _ => None,
            })
            .expect("원본 표");
        let mut checked = 0;
        for probe in whole_row_cells(&core) {
            let Some(line) = probe.lines.first() else {
                continue;
            };
            let cell = table
                .cells
                .iter()
                .find(|cell| cell.row == probe.row && cell.col == probe.col)
                .expect("실제 칸의 원본 소속");
            let para = &cell.paragraphs[0];
            let shape = para
                .controls
                .iter()
                .find_map(|control| match control {
                    rhwp::model::control::Control::Shape(shape) => Some(shape.common()),
                    _ => None,
                })
                .expect("직선 앵커");
            let font = &document.doc_info.char_shapes[para.char_shapes[0].char_shape_id as usize];
            let em = f64::from(font.base_size) * 96.0 / 7200.0;
            let offset = f64::from(shape.vertical_offset) * 96.0 / 7200.0;
            let extent = offset + f64::from(shape.height) * 96.0 / 7200.0;
            let padding = cell.effective_padding(&table.padding);
            let top = f64::from(padding.top) * 96.0 / 7200.0;
            let bottom = f64::from(padding.bottom) * 96.0 / 7200.0;
            let inner = probe.cell.height - top - bottom;
            let content = em.max(extent);
            let expected = probe.cell.y + top + (inner - content).max(0.0) / 2.0 + offset;
            assert!((line.y - expected).abs() <= 0.5,
                "{name} p{} r{}c{}: 원본 글자 줄과 개체의 큰 띠로 정렬해야 한다; 실제 {}, 원본 관계 {}",
                probe.page + 1, probe.row, probe.col, line.y, expected);
            checked += 1;
        }
        assert!(checked >= 16, "{name}: 전체 행과 병합 칸의 앵커를 검사한다");
    }
}
