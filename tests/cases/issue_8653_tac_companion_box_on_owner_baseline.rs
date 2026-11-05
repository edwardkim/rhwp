//! [#8653] 관계형·고정형 회귀 — 글자처럼 취급(TAC) 표만 담은 문단의 한 저장 줄을 한 표가
//! 소유하면(줄 높이 = 그 표의 위 바깥여백 + 본체 + 아래 바깥여백), 같은 줄의 다른 TAC 표는
//! 소유자의 기준선에 높이의 85% 로 앉는다.
//!
//! 주석 갱신: 2026-11-06 (KST) — 측정값은 아래 표
//!
//! - 입력: `samples/issue8653/tac_companion_box_on_owner_baseline.hwpx` — 서울시 결재문서
//!   (36355589, 한컴 2020 저장, 1쪽)를 한/글 2022 로 다시 저장한 뒤 최상위 문단 0·1 만 남겨
//!   다시 한/글 2022 로 저장한 축소본. 문단 0 한 줄(lh 7844, 기준선 6667)에 시행 상자(4×2,
//!   7564HU + 바깥여백 140/140 = 7844, 줄 소유자)와 결재 상자(3×5, 4996HU)가 함께 있다.
//! - 독립 기준: 같은 입력의 한/글 2022 Print PDF
//!   `pdf/issue8653_tac_companion_box_on_owner_baseline-2020.pdf`(1쪽). 960dpi 래스터에서
//!   결재 상자 위 괘선 중심 87.55px, 같은 방식의 시행 상자 위 괘선 중심 58.45px(rhwp 58.6)
//!   → 보정한 결재 상자 위끝 87.7px = 줄 위끝 56.7 + 1.87 + 0.85 × (100.85 − 66.61).
//! - 수정 전 rhwp(devel ece9c3383b): TAC 표 문단 경로가 결재 상자 본체 하단을 `기준선 +
//!   아래 바깥여백` 에 붙여 위끝 80.8px(6.9px 위).
//!
//! 측정(2026-11-06, `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값
//! `silhouette_raw_match_percent` = 색상 경계 보정 전 2px 이웃 내용 실루엣):
//!
//! | 입력 | 전체 쪽수(한컴=rhwp) | Native | fresh WASM | 수정 전 Native |
//! |---|---|---|---|---|
//! | tac_companion_box_on_owner_baseline | 1 | 100.00 | 100.00 | 65.75 |
//!
//! 두 경로 전쪽 최저: 100.00.
//!
//! 증거: `mydocs/pr/assets/issue_8653/{native,native_base,wasm}/*_silhouette.tsv`,
//! `review/*.png`. 검사는 두 표의 쪽·문단 소속, 소유자와 동반 표 위끝의 기준선 관계와 하단
//! 차(높이차의 0.15배)(관계형), 독립 PDF 전체 쪽수(고정형)다. 절대 px 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8653/tac_companion_box_on_owner_baseline.hwpx";

fn px(hu: i64) -> f64 {
    hu as f64 * 96.0 / 7200.0
}

fn find_table(node: &RenderNode, para: usize, control: usize) -> Option<BoundingBox> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.para_index == Some(para) && table.control_index == Some(control) {
            return Some(node.bbox.clone());
        }
    }
    node.children
        .iter()
        .find_map(|child| find_table(child, para, control))
}

#[test]
fn companion_tac_table_sits_on_line_owner_baseline() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("parse: {e:?}"));
    assert_eq!(core.page_count(), 1, "한컴 Print PDF 1쪽");

    let para = &core.document().sections[0].paragraphs[0];
    assert_eq!(para.line_segs.len(), 1, "문단 0 은 저장 줄 하나");
    let line = &para.line_segs[0];
    let tables: Vec<(usize, _)> = para
        .controls
        .iter()
        .enumerate()
        .filter_map(|(ci, c)| match c {
            Control::Table(t) if t.common.treat_as_char => Some((ci, t)),
            _ => None,
        })
        .collect();
    assert_eq!(tables.len(), 2, "한 줄에 TAC 표 둘");
    let outer = |t: &rhwp::model::table::Table| {
        i64::from(t.outer_margin_top)
            + i64::from(t.common.height)
            + i64::from(t.outer_margin_bottom)
    };
    let (owner_ci, owner) = tables
        .iter()
        .copied()
        .find(|(_, t)| outer(t) == i64::from(line.line_height))
        .unwrap_or_else(|| panic!("줄 높이 = 바깥 상자인 소유 표"));
    let (companion_ci, companion) = tables
        .iter()
        .copied()
        .find(|(ci, _)| *ci != owner_ci)
        .unwrap();

    let tree = core
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("1쪽 렌더 트리: {e:?}"));
    let owner_box = find_table(&tree.root, 0, owner_ci).expect("소유 표를 그린다");
    let companion_box = find_table(&tree.root, 0, companion_ci).expect("동반 표를 그린다");

    let owner_h = px(i64::from(owner.common.height));
    let companion_h = px(i64::from(companion.common.height));
    let owner_om_top = px(i64::from(owner.outer_margin_top));
    let line_top = owner_box.y - owner_om_top;
    let expected = line_top + owner_om_top + 0.85 * (owner_h - companion_h);
    assert!(
        (companion_box.y - expected).abs() <= 0.5,
        "동반 표 위끝 {:.1} = 줄 위끝 {line_top:.1} + 소유자 위 바깥여백 + 0.85×높이차 = {expected:.1}",
        companion_box.y
    );
    let bottom_gap = (owner_box.y + owner_box.height) - (companion_box.y + companion_box.height);
    assert!(
        (bottom_gap - 0.15 * (owner_h - companion_h)).abs() <= 0.5,
        "두 표 하단 차 {bottom_gap:.1} = 높이차의 0.15배 {:.1}",
        0.15 * (owner_h - companion_h)
    );
}
