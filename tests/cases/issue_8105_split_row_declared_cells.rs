//! [#8105] 행 안 내용 컷으로 나뉜 일반 행의 이어지는 조각은 그 행 칸의 선언 높이로 잰다.
//!
//! 기준: 한/글 2020 Print `pdf/1613000-202200037-air-traffic-controller-cbta-2020.pdf`.
//! - 141·142쪽 구역 1 표(250×7) 행 88: 칸 선언 1282HU(17.09px), 내용 두 줄. 141쪽이 첫 줄을 닫는 선까지
//!   담고 142쪽 조각은 남은 한 줄 17.1px 이다(괘선 88.8 → 105.9). 측정 행 높이(38.4px)에서 141쪽 상자를
//!   빼면 21.3px 이 되어, 수정 전 rhwp 는 142쪽 이후 행이 모두 4.3px 아래로 밀렸다.
//!
//! - 86712 32–34쪽 표(2×2) 행 1: 세 쪽에 걸친 행의 가운데 조각(33쪽)은 앞 쪽 상자를 장부로 받지 않았다.
//!   측정 행 높이(2262.5px)에서 33쪽 상자만 빼 1332.8px 을 34쪽 끝 조각에 넘기면 32쪽이 그린 몫까지 다시
//!   예약해 34쪽 조각이 본문 아래까지 늘어났다. 기준 `pdf/86712_regulatory_analysis-hwp-2024.pdf` 34쪽
//!   행 1 아래 괘선 535.1 = 남은 내용 끝.
//!
//! 경계(이 규칙을 쓰지 않는 행): 그 행에서 끝나는 병합 칸의 선언이 지나는 행들의 칸 선언 합보다 크면
//! 남는 몫이 측정 행 높이에 실려 다음 조각으로 이어진다 — 1342000 127·128쪽 행 78–80 칸(19744HU >
//! 5782 + 4300 + 2618). 이 경계는 1342000 전쪽 Visual Sweep 에서 90% 미만 0쪽으로 확인했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp";
const SAMPLE_86712: &str = "samples/86712_regulatory_analysis.hwp";

fn load(sample: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {sample}: {error:?}"))
}

fn core() -> DocumentCore {
    load(SAMPLE)
}

struct CellBox {
    row: u16,
    top: f64,
    bottom: f64,
    /// 칸 안 내용(글줄·개체)의 가장 낮은 아래.
    content_bottom: Option<f64>,
}

/// 칸 안 내용(글줄·개체)의 가장 낮은 아래.
fn content_bottom(node: &RenderNode) -> Option<f64> {
    node.children
        .iter()
        .flat_map(|child| {
            std::iter::once(child.bbox.y + child.bbox.height).chain(content_bottom(child))
        })
        .reduce(f64::max)
}

/// 표 `(구역, 문단, 개체)` 가 쪽 `page`(0-기준)에 그린 칸 상자.
fn table_cells(core: &DocumentCore, page: u32, table: (usize, usize, usize)) -> Vec<CellBox> {
    fn walk(node: &RenderNode, inside: bool, table: (usize, usize, usize), out: &mut Vec<CellBox>) {
        let inside = match &node.node_type {
            RenderNodeType::Table(t) => {
                (t.section_index, t.para_index, t.control_index)
                    == (Some(table.0), Some(table.1), Some(table.2))
            }
            RenderNodeType::TableCell(cell) if inside => {
                out.push(CellBox {
                    row: cell.row,
                    top: node.bbox.y,
                    bottom: node.bbox.y + node.bbox.height,
                    content_bottom: content_bottom(node),
                });
                false
            }
            _ => inside,
        };
        for child in &node.children {
            walk(child, inside, table, out);
        }
    }
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("{}쪽 렌더 트리: {error:?}", page + 1));
    let mut out = Vec::new();
    walk(&tree.root, false, table, &mut out);
    out
}

fn px(hu: f64) -> f64 {
    hu / 75.0
}

#[test]
fn continued_row_fragment_keeps_to_its_declared_cell_height() {
    let core = core();
    let table = (1, 2, 0);
    let on_141 = table_cells(&core, 140, table);
    assert!(
        on_141.iter().any(|b| b.row == 88),
        "141쪽이 행 88 첫 조각을 담아야 한다"
    );
    let on_142 = table_cells(&core, 141, table);
    let row_88 = on_142
        .iter()
        .find(|b| b.row == 88)
        .expect("142쪽이 행 88 이어지는 조각으로 시작해야 한다");
    let height = row_88.bottom - row_88.top;
    let declared = px(1282.0);
    assert!(
        height <= declared + 0.5,
        "142쪽 행 88 조각 = 남은 한 줄(정본 17.1px, 칸 선언 {declared:.2}px), 실제 {height:.2}px"
    );
    let row_89_top = on_142
        .iter()
        .filter(|b| b.row == 89)
        .map(|b| b.top)
        .fold(f64::INFINITY, f64::min);
    assert!(
        (row_89_top - row_88.bottom).abs() <= 0.5,
        "행 89 는 행 88 조각 바로 아래에서 시작한다: 행 88 아래 {:.2} / 행 89 위 {row_89_top:.2}",
        row_88.bottom
    );
}

#[test]
fn middle_fragment_without_carried_boxes_does_not_stretch_the_last_fragment() {
    let core = load(SAMPLE_86712);
    let cells = table_cells(&core, 33, (0, 195, 0));
    let row_1: Vec<_> = cells.iter().filter(|b| b.row == 1).collect();
    assert!(!row_1.is_empty(), "34쪽이 행 1 끝 조각을 담아야 한다");
    let bottom = row_1.iter().map(|b| b.bottom).fold(f64::MIN, f64::max);
    let content = row_1
        .iter()
        .filter_map(|b| b.content_bottom)
        .fold(f64::MIN, f64::max);
    assert!(
        bottom - content <= 20.0,
        "34쪽 행 1 끝 조각은 남은 내용에서 닫힌다(정본 아래 괘선 535.1): 행 아래 {bottom:.1}, 내용 아래 {content:.1}"
    );
}
