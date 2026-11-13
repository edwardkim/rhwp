//! [#8981] 관계형·고정형 회귀 — 겹침 걸음 사다리 칸의 빈 줄 전진은 다음 문단 앞 간격을 한 번만 담는다.
//!
//! 주석 갱신: 2026-11-12 15:30 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8981/overlap_spacer_paragraph_bands.hwp` — 해양수산부 공개 문서
//!   「평가결과서(선박금융 강화)」(HWP5, 1쪽). 본문 표 「평가 결과」 칸(행 6, 열 2)의 문단 12개 중
//!   빈 줄 문단 2·5·9 는 줄간격 60%(`lh=1300 ls=-520`)라 다음 문단 vpos 가 줄 상자 안에서 시작한다
//!   (겹침 걸음 셋 → `cell_uses_overlapping_line_boxes`). 그 걸음(예: 6456 → 7536HU)은 빈 줄
//!   전진 780HU 와 다음 문단 앞 간격 300HU 를 함께 담는다.
//! - 독립 기준: `pdf/issue8981/overlap_spacer_paragraph_bands-2020.pdf`(한/글 2022 12.0.0.4547 Print,
//!   1쪽). 한/글은 칸 안 문단 원점을 저장 사다리대로 둔다.
//! - 수정 전 rhwp(devel `eb84afc036`·`80b1c354c2`)는 빈 줄 전진에 저장 차이 전체를 쓰고 다음 문단에서 앞 간격을
//!   다시 더해, 빈 줄마다 4px 씩(칸 끝 12px) 아래로 밀렸다.
//!
//! 측정: 2026-11-12, `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣). 전체 1쪽 = 한/글 1쪽.
//! 수정 후(devel `80b1c354c2` 기준) 1쪽 Native 99.70%, fresh WASM 100.00%(두 경로 전쪽 최저값도
//! 같음). 수정 전 Native 82.80%. 증거: `mydocs/pr/assets/issue_8981/{native,wasm,base-native}/`.
//!
//! 검사: 원본 계약(겹침 걸음 셋 이상, 빈 줄 뒤 저장 걸음 > 빈 줄 `lh + ls`), 쪽수 1 = 한/글(고정형),
//! 칸 안 각 문단 첫 줄 위끝의 첫 문단 대비 거리 = 저장 vpos 차, 마지막 줄이 칸 안(관계형).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8981/overlap_spacer_paragraph_bands.hwp";
/// 표를 품은 본문 문단.
const HOST: usize = 1;
/// 「평가 결과」 내용 칸.
const ROW: u16 = 6;
const COL: u16 = 2;

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

fn hu_to_px(hu: i32) -> f64 {
    f64::from(hu) * 96.0 / 7200.0
}

fn find_cell(node: &RenderNode, cell_index: u32) -> Option<&RenderNode> {
    if let RenderNodeType::TableCell(cell) = &node.node_type {
        if cell.model_cell_index == Some(cell_index) && cell.row == ROW && cell.col == COL {
            return Some(node);
        }
    }
    node.children
        .iter()
        .find_map(|child| find_cell(child, cell_index))
}

/// (문단, 줄) → 줄 상자 위끝·아래끝.
fn cell_lines(node: &RenderNode, out: &mut Vec<(usize, u32, f64, f64)>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if let (Some(para), Some(index)) = (line.para_index, line.line_index) {
            out.push((para, index, node.bbox.y, node.bbox.y + node.bbox.height));
        }
    }
    for child in &node.children {
        cell_lines(child, out);
    }
}

#[test]
fn overlap_ladder_empty_line_advance_keeps_one_spacing_band() {
    let core = load();
    assert_eq!(core.page_count(), 1, "한/글 Print 와 같은 1쪽");

    let host = &core.document().sections[0].paragraphs[HOST];
    let table = host
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table),
            _ => None,
        })
        .expect("본문 표");
    let cell_index = table
        .cells
        .iter()
        .position(|cell| cell.row == ROW && cell.col == COL)
        .expect("평가 결과 칸");
    let cell = &table.cells[cell_index];

    // 원본 계약: 겹침 걸음(다음 vpos 가 앞 줄 상자 안)이 셋 이상이고, 빈 줄 문단 뒤 걸음은
    // 빈 줄 자신의 `lh + ls` 보다 커서 문단 간격 띠를 담는다.
    let segs: Vec<_> = cell.paragraphs.iter().flat_map(|p| &p.line_segs).collect();
    let overlapping = segs
        .windows(2)
        .filter(|w| {
            w[1].vertical_pos >= w[0].vertical_pos
                && w[1].vertical_pos < w[0].vertical_pos + w[0].line_height
        })
        .count();
    assert!(overlapping >= 3, "겹침 걸음 {overlapping} ≥ 3");
    let mut banded_empty = 0;
    for (index, para) in cell.paragraphs.iter().enumerate() {
        let [line] = para.line_segs.as_slice() else {
            continue;
        };
        if !para.text.trim().is_empty() || line.line_spacing >= 0 {
            continue;
        }
        let next = &cell.paragraphs[index + 1].line_segs[0];
        let step = next.vertical_pos - line.vertical_pos;
        assert!(
            step > line.line_height + line.line_spacing && step <= line.line_height,
            "빈 줄 문단 {index}: 걸음 {step} 은 lh+ls {} 보다 크고 lh {} 이하",
            line.line_height + line.line_spacing,
            line.line_height
        );
        banded_empty += 1;
    }
    assert_eq!(banded_empty, 3, "문단 간격 띠를 담은 음수 줄간격 빈 줄 3개");

    let page = core.build_page_render_tree(0).expect("1쪽");
    let cell_node = find_cell(&page.root, cell_index as u32).expect("칸 상자");
    let mut lines = Vec::new();
    cell_lines(cell_node, &mut lines);

    let first_top = |para: usize| {
        lines
            .iter()
            .find(|(p, line, _, _)| *p == para && *line == 0)
            .map(|(_, _, top, _)| *top)
            .unwrap_or_else(|| panic!("칸 문단 {para} 첫 줄"))
    };
    let origin_top = first_top(0);
    let origin_vpos = cell.paragraphs[0].line_segs[0].vertical_pos;
    for (index, para) in cell.paragraphs.iter().enumerate() {
        let stored = hu_to_px(para.line_segs[0].vertical_pos - origin_vpos);
        let drawn = first_top(index) - origin_top;
        assert!(
            (drawn - stored).abs() <= 0.5,
            "칸 문단 {index} 첫 줄 거리 {drawn:.1} = 저장 vpos 차 {stored:.1}"
        );
    }

    let cell_bottom = cell_node.bbox.y + cell_node.bbox.height;
    let last_bottom = lines
        .iter()
        .map(|(_, _, _, bottom)| *bottom)
        .fold(f64::NEG_INFINITY, f64::max);
    assert!(
        last_bottom <= cell_bottom + 0.5,
        "마지막 줄 아래끝 {last_bottom:.1} 은 칸 아래끝 {cell_bottom:.1} 안"
    );
}
