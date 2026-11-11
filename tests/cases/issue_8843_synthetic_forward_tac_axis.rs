//! [#8843] 관계형·고정형 회귀 — 가린 문단(합성 줄) 뒤 저장 앞 점프를 커서가 거절하면, 뒤 저장 TAC
//! 표도 같은 축에서 순차 흐름에 놓인다(거절한 빈자리를 단 첫 표 축으로 되살리지 않는다).
//!
//! 주석 갱신: 2026-11-09 17:36 KST(+09:00), 새 파일.
//!
//! - 입력: `samples/issue8843/synthetic_forward_tac_axis.hwpx` — 공개 결재문서 36355707(HWPX, 2쪽).
//!   문단 8 은 가린(합성, bit31) 줄이고 그 끝과 문단 9 저장 vpos 사이에 옛 줄 수로 잰 1920HU
//!   빈자리가 있다. 문단 10 은 TAC 표(2×8) 하나만 든 문단이다.
//! - 독립 기준: `pdf/issue8843/synthetic_forward_tac_axis-2020.pdf`(한/글 2022 12.0.0.4547 Print, 2쪽).
//!   한/글은 빈자리를 두지 않고 순차 흐름으로 그린다(표 위끝 621px, 「라.」 733.6px).
//! - 수정 전 rhwp(devel `024d610633`)는 커서가 문단 9 에서 이 앞 점프를 거절하고도(`#1811`), 문단 10
//!   의 합성 TAC 줄 경로가 단 첫 표 축으로 빈자리를 되살려 표·뒤 본문이 25.6px 아래였다
//!   (TAC 표 − 문단 9 48.8px, 기대 23.2px).
//!
//! 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96`(4열
//! `silhouette_raw_match_percent`, 색상 경계 보정 전 2px 실루엣), 소스 devel `024d610633` + 이 수정.
//! 1쪽 Native 99.90 / fresh WASM 99.90(수정 전 Native 61.03), 2쪽 99.99 / 100.00. 전체 2쪽 =
//! PDF 2쪽, 전쪽 최저 Native 99.90(1쪽)·fresh WASM 99.90(1쪽).
//! 증거: `mydocs/pr/assets/issue_8843/{native,wasm,native-base}/`(TSV·review·overlay PNG).
//!
//! 검사: 원본 계약(문단 8 합성 줄, 문단 8·9 사이 저장 빈자리, 문단 10 TAC 표), 쪽수 2 = 한/글
//! (고정형), TAC 표 − 문단 9 = 저장 vpos 차이 + 바깥 위 여백, 문단 11 − 문단 9 = 저장 vpos 차이
//! (관계형, ±1px — 양쪽 raw 99.90% 쪽).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::LineSeg;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8843/synthetic_forward_tac_axis.hwpx";
const DPI: f64 = 96.0;

fn px(hu: i32) -> f64 {
    f64::from(hu) * DPI / 7200.0
}

fn load() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"))
}

/// 본문 흐름(표 칸 안 제외)의 문단별 첫 글줄 위끝과 첫 표 위끝.
#[derive(Default)]
struct Flow {
    lines: Vec<(usize, f64)>,
    tables: Vec<(usize, f64)>,
}

fn collect(node: &RenderNode, flow: &mut Flow) {
    match &node.node_type {
        RenderNodeType::Table(table) => {
            if let Some(para) = table.para_index {
                if !flow.tables.iter().any(|(owner, _)| *owner == para) {
                    flow.tables.push((para, node.bbox.y));
                }
            }
            return;
        }
        RenderNodeType::TextLine(line) => {
            if let Some(para) = line.para_index {
                if !flow.lines.iter().any(|(owner, _)| *owner == para) {
                    flow.lines.push((para, node.bbox.y));
                }
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, flow);
    }
}

fn find(items: &[(usize, f64)], para: usize, what: &str) -> f64 {
    items
        .iter()
        .find(|(owner, _)| *owner == para)
        .map(|(_, y)| *y)
        .unwrap_or_else(|| panic!("문단 {para} {what}"))
}

#[test]
fn tac_table_after_rejected_synthetic_gap_follows_the_flow() {
    let core = load();
    let paragraphs = &core.document().sections[0].paragraphs;
    let first = |index: usize| &paragraphs[index].line_segs[0];

    // 원본 계약: 문단 8 은 가린(합성) 줄이고, 그 끝과 문단 9 저장 vpos 사이에 옛 줄 수로 잰 빈자리가
    // 있다. 문단 10 은 TAC 표 하나만 든 문단이다.
    let masked = first(8);
    assert_ne!(
        masked.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY,
        0,
        "문단 8 합성 줄"
    );
    let masked_end = masked.vertical_pos + masked.line_height + masked.line_spacing;
    assert!(
        first(9).vertical_pos > masked_end,
        "문단 8·9 사이 저장 빈자리"
    );
    let table = paragraphs[10]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) if table.common.treat_as_char => Some(table.as_ref()),
            _ => None,
        })
        .expect("문단 10 TAC 표");
    assert_eq!(core.page_count(), 2, "전체 쪽수 = 한/글 2쪽");

    let page = core.build_page_render_tree(0).expect("1쪽");
    let mut flow = Flow::default();
    collect(&page.root, &mut flow);
    let unit = find(&flow.lines, 9, "글줄");

    // 문단 9 → 표: 저장 vpos 차이 + 바깥 위 여백(빈자리는 문단 9 앞에서 이미 거절됐다).
    let expected =
        px(first(10).vertical_pos - first(9).vertical_pos) + px(i32::from(table.outer_margin_top));
    let got = find(&flow.tables, 10, "표") - unit;
    assert!(
        (got - expected).abs() <= 1.0,
        "TAC 표 − 문단 9 {got:.1} = 저장 vpos 차이 + 바깥 위 여백 {expected:.1}"
    );

    // 문단 9 → 문단 11: 같은 축의 저장 vpos 차이.
    let expected = px(first(11).vertical_pos - first(9).vertical_pos);
    let got = find(&flow.lines, 11, "글줄") - unit;
    assert!(
        (got - expected).abs() <= 1.0,
        "문단 11 − 문단 9 {got:.1} = 저장 vpos 차이 {expected:.1}"
    );
}
