//! 저장 줄이 없는 **빈** 캡션 문단의 높이는 그 문단 글자 크기의 줄 하나이고, 계산 글줄 host 의
//! 자리차지 표는 첫 조각에서도 바깥 위 여백을 연다.
//!
//! 두 문서 모두 「- 규제 차등화 예비분석 결과표」 제목 줄과 자리차지 표(문단 기준 세로 오프셋
//! 1784 HU, 바깥 위 여백 141 HU)를 한 문단에 담는다. 표 위 캡션은 빈 문단 하나(글자 2pt =
//! 200 HU, 저장 줄 없음), 캡션 간격은 850 HU 다. 제목 줄 상단 → 표 위 괘선은
//! 23.79(오프셋) + 1.88(바깥 위 여백) + 2.67(캡션) + 11.33(캡션 간격) = **39.67px** 다.
//!
//! - `samples/80168_regulatory_analysis.hwp` 10쪽 문단 61(통째 표). 한/글 2022 정본
//!   (`pdf/80168_regulatory_analysis-2022.pdf`): 제목 글자 위끝 203.22px, 표 위 괘선 243.09px,
//!   아래 괘선 830.29px. 수정 전 rhwp 는 빈 캡션을 기본 400 HU 로 재 42.33px 였다.
//! - `samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp` 12쪽 문단 83
//!   (분할 첫 조각, 저장 줄 없는 HWP 라 host 글줄을 계산한다). 한/글 2020 정본
//!   (`pdf/70833-electrical-safety-rule-regulatory-analysis-2020.pdf`): 제목 글자 위끝 235.46px,
//!   표 위 괘선 275.06px. 수정 전 rhwp 는 캡션만 고치면 바깥 위 여백이 빠져 37.8px 였다.
//!
//! 반례: 표 자체 높이(행 합)는 이 변경과 무관하다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

/// 제목 줄 상단 → 표 위 괘선 (px).
const EXPECTED_GAP: f64 = 39.67;

fn host_line_top(node: &RenderNode, para: usize) -> Option<f64> {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.para_index == Some(para) && line.caption_owner.is_none() {
            return Some(node.bbox.y);
        }
    }
    node.children
        .iter()
        .find_map(|child| host_line_top(child, para))
}

fn host_table(node: &RenderNode, para: usize) -> Option<(f64, f64)> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para) {
            return Some((node.bbox.y, node.bbox.height));
        }
    }
    node.children
        .iter()
        .find_map(|child| host_table(child, para))
}

/// 문단의 제목 줄과 표가 처음 함께 나오는 쪽: (쪽, 줄 상단, 표 위끝, 표 높이).
fn title_and_table(sample: &str, para: usize) -> (u32, f64, f64, f64) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {sample}: {e}"));
    let doc = DocumentCore::from_bytes(&bytes).expect("parse sample");
    for page in 0..doc.page_count() {
        let tree = doc.build_page_render_tree(page).expect("render tree");
        if let (Some(line_top), Some((table_top, table_height))) = (
            host_line_top(&tree.root, para),
            host_table(&tree.root, para),
        ) {
            return (page, line_top, table_top, table_height);
        }
    }
    panic!("{sample}: 문단 {para} 의 제목 줄과 표가 같은 쪽에 있어야 한다");
}

#[test]
fn empty_top_caption_reserves_its_own_glyph_height() {
    let (page, line_top, table_top, table_height) =
        title_and_table("samples/80168_regulatory_analysis.hwp", 61);
    let gap = table_top - line_top;
    assert!(
        (gap - EXPECTED_GAP).abs() <= 0.5,
        "80168 {}쪽: 제목 줄 상단→표 위끝 {gap:.2}px, 기대 {EXPECTED_GAP}px \
         (한/글 2022 글자 위끝 기준 39.87px; 빈 캡션을 400 HU 로 재면 42.33px)",
        page + 1
    );
    // 반례: 표 높이(행 7개 합)는 바뀌지 않는다 — 정본 830.29 − 243.09 = 587.20px.
    assert!(
        (table_height - 587.2).abs() <= 1.0,
        "80168 표 높이 {table_height:.2}px 가 정본 587.20px 와 달라졌다"
    );
}

#[test]
fn split_first_fragment_of_computed_host_opens_outer_top_margin() {
    let (page, line_top, table_top, _) = title_and_table(
        "samples/issue6854/70833-electrical-safety-rule-regulatory-analysis.hwp",
        83,
    );
    let gap = table_top - line_top;
    assert!(
        (gap - EXPECTED_GAP).abs() <= 0.5,
        "70833 {}쪽 첫 조각: 제목 줄 상단→표 위끝 {gap:.2}px, 기대 {EXPECTED_GAP}px \
         (한/글 2020 글자 위끝 기준 39.60px; 바깥 위 여백이 빠지면 37.8px)",
        page + 1
    );
}
