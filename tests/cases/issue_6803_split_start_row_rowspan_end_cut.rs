//! [Issue #6803] 쪽 분할 조각의 **시작 행에서 시작해 끝 행을 넘는** rowspan 셀이
//! 끝 컷을 못 받아 **다음 쪽이 다시 그릴 몫까지** 배치했다.
//!
//! `1376496` (전주·완주 혁신도시 [별표 6]) 의 `pi=6 ci=0` 표는 14행×5열
//! `쪽나눔=RowBreak`. 셀 `r=8,c=0 rs=6` 이 문단 67개(빈 문단으로 세로 가운데를 만든
//! 수작업 서식, 156,749HU = 2,090px)를 담고 3·4쪽에 걸친다.
//!
//! ```text
//! DIAG_FRAG pi=6 ci=0 rows=8..11  start_cut=[1] end_cut=[1]   ← end_cut 은 행 10 의 부기
//! DIAG_FRAG pi=6 ci=0 rows=10..14 start_cut=[1] end_cut=[5]
//! ```
//!
//! `table_partial.rs` 의 행 술어가 셀을 **시작 행으로만** 분류한다.
//!
//! ```text
//! is_split_start_row = !start_cut.is_empty() && cell_row == start_row      // 8 == 8  → true
//! is_split_end_row   = !end_cut.is_empty()   && cell_row == end_row - 1    // 8 == 10 → false
//! ```
//!
//! `apply_end == false` 라 `cell_cut_window` 가 `eu = usize::MAX` 를 내고, `#1748` 의
//! 높이-기반 구제(`is_rowbreak_straddle`)는 `!is_in_split_row` 조건에서 막힌다.
//! 그래서 3쪽이 `pi=0..66` 을 전부 그렸다 — 글줄이 용지 아래 **1,015.1px**,
//! `export-text` 3쪽의 `A1` 이 **2 대신 6**.
//!
//! ⭐ 같은 셀이 4쪽에서는 `is_in_split_row=false` → `is_rowbreak_straddle` 로 높이 컷을
//! 받아 `pi=34` 에서 **정확히 이어받는다**. 컷 위치는 처음부터 옳았고 3쪽만 멈추지
//! 않았다 — 두 경로의 비대칭만 메운다.
//!
//! 아래 시험은 **양쪽을 잠근다**: 늦게 끊으면(중복) 3쪽 마지막이 33을 넘고, 일찍
//! 끊으면(소실) 3쪽 끝과 4쪽 시작 사이에 구멍이 난다.
//!
//! 재현 원본은 samples/issue6803/에 정식 등록했다. 파일 부재는 실패다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::PathBuf;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6803/1376496-neighborhood-facility-land-table.hwp";

/// 작업 디렉터리나 비공개 환경 변수와 무관한 정식 회귀 입력.
fn document_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// 문단 67개를 담은 `r=8,c=0 rs=6` 병합 셀.
const TALL_CELL_ROW: u16 = 8;
const TALL_CELL_COL: u16 = 0;
const TALL_CELL_ROW_SPAN: u16 = 6;

/// 그 셀이 이 쪽에 그린 직계 글줄의 문단 인덱스.
fn tall_cell_para_indices(node: &RenderNode, out: &mut Vec<usize>) {
    if let RenderNodeType::TableCell(meta) = &node.node_type {
        if meta.row == TALL_CELL_ROW
            && meta.col == TALL_CELL_COL
            && meta.row_span == TALL_CELL_ROW_SPAN
        {
            for child in &node.children {
                if let RenderNodeType::TextLine(line) = &child.node_type {
                    if let Some(pi) = line.para_index {
                        out.push(pi);
                    }
                }
            }
            return;
        }
    }
    for child in &node.children {
        tall_cell_para_indices(child, out);
    }
}

fn paper_bottom(node: &RenderNode) -> f64 {
    node.bbox.y + node.bbox.height
}

/// 이 쪽에서 용지 아래로 가장 많이 내려간 글줄의 초과폭.
fn worst_text_line_overhang(node: &RenderNode, bottom: f64) -> f64 {
    let mut worst = 0.0f64;
    if matches!(node.node_type, RenderNodeType::TextLine(_)) {
        worst = worst.max(node.bbox.y + node.bbox.height - bottom);
    }
    for child in &node.children {
        worst = worst.max(worst_text_line_overhang(child, bottom));
    }
    worst
}

/// 이 셀이 조각별로 그린 문단 인덱스 — 2·3·4쪽 순서대로.
fn fragment_ranges() -> Vec<Vec<usize>> {
    let path = document_path();
    let bytes = std::fs::read(&path).expect("#6803 정식 회귀 sample 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");
    assert_eq!(
        core.page_count(),
        5,
        "쪽수 형상이 바뀌면 이 시험의 전제가 깨진다"
    );

    (1u32..=3)
        .map(|page| {
            let tree = core.build_page_render_tree(page).expect("render tree");
            let mut pis = Vec::new();
            tall_cell_para_indices(&tree.root, &mut pis);
            pis.sort_unstable();
            pis.dedup();
            pis
        })
        .collect()
}

/// 조각들이 이 셀의 문단을 **빠짐없이·겹침없이** 나눠 갖는다.
///
/// 수정 전: 3쪽 조각이 `pi=0..66` 을 **전부** 그려 4쪽(`34..66`)과 33개가 겹쳤고,
/// 그중 `pi=34..66` 은 용지 아래로 나갔다(최대 +1,015.1px).
///
/// 이 하나로 양쪽이 잠긴다 — 늦게 끊으면 **중복**이, 일찍 끊으면 **구멍**이 생긴다.
#[test]
fn the_fragments_partition_the_cell_exactly_once() {
    let ranges = fragment_ranges();
    let mut seen: Vec<usize> = ranges.iter().flatten().copied().collect();
    let total = seen.len();
    seen.sort_unstable();
    seen.dedup();

    assert_eq!(
        total,
        seen.len(),
        "조각들이 같은 문단을 두 번 그렸다 — #6803 회귀 (조각별 범위 {ranges:?})"
    );
    assert_eq!(
        seen.first().copied(),
        Some(0),
        "첫 조각은 이 셀의 첫 문단부터 그린다 (조각별 범위 {ranges:?})"
    );
    assert_eq!(
        seen.len(),
        seen.last().copied().expect("마지막") + 1,
        "조각 사이에 구멍이 있다 — #6803 회귀 (조각별 범위 {ranges:?})"
    );
}

/// 각 조각은 **자기 몫에서 멈춘다** — 다음 조각이 이어받을 문단을 미리 그리지 않는다.
///
/// 수정 전: 3쪽 조각이 `pi=66` 까지 그렸다(4쪽이 다시 그리는 `pi=34..66` 포함).
#[test]
fn each_fragment_stops_where_the_next_one_resumes() {
    let ranges = fragment_ranges();
    for pair in ranges.windows(2) {
        let (Some(end), Some(next_start)) = (pair[0].last(), pair[1].first()) else {
            continue;
        };
        assert_eq!(
            *next_start,
            end + 1,
            "조각 이음매가 어긋났다 — #6803 회귀              (앞 조각 끝 pi={end}, 다음 조각 시작 pi={next_start};               수정 전 3쪽 조각이 pi=66 까지 그렸다)"
        );
    }
}

/// 용지 밖 글줄이 남지 않는다 (`layout-anomaly` off-canvas 1 → 0 과 같은 축).
#[test]
fn no_text_line_falls_off_the_paper() {
    let path = document_path();
    let bytes = std::fs::read(&path).expect("#6803 정식 회귀 sample 읽기");
    let core = DocumentCore::from_bytes(&bytes).expect("문서 로드");

    let mut worst = 0.0f64;
    let mut worst_page = 0u32;
    for page in 0..core.page_count() {
        let tree = core
            .build_page_render_tree(page as u32)
            .expect("render tree");
        let overhang = worst_text_line_overhang(&tree.root, paper_bottom(&tree.root));
        if overhang > worst {
            worst = overhang;
            worst_page = page as u32 + 1;
        }
    }

    assert!(
        worst < 40.0,
        "글줄이 용지 밖으로 나갔다 — #6803 회귀          (최대 초과 {worst:.1}px @ {worst_page}쪽; 수정 전 +1,015.1px @ 3쪽)"
    );
}
