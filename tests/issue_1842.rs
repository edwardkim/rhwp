//! Issue #1842 — 서식 표 셀 내부 인라인 개체 라인높이: 저장 LINE_SEG lh 재계산 드리프트.
//!
//! 3114781 (한방 병·의원 수입금액 검토부표) p2 의 전면 1×1 서식 표에서, 셀 내부
//! 첫 문단(텍스트 없는 tac 묶음 전용, 저장 lh=3401HU=45.3px)의 라인높이가
//! `has_tac_shape` 축소 분기로 퇴화(max_fs=0 → font_lh=0)해 후속 블록 전체가
//! −33pt 당겨졌다 (한글 2022 대비 p2 Δbaseline median −33.9pt). "Shape 와
//! 텍스트의 baseline 정렬" 보정은 텍스트가 있는 줄(max_fs>0)에만 적용한다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue1842_cell_tac_group_lineheight.hwp";

/// p2 서식 셀의 첫 텍스트 문단(p[1])이 저장 vpos 사다리(작성요령 개체 라인
/// lh=3401 + ls=500 + 간격 = 5901HU) 아래에서 시작한다. 종전에는 개체 라인이
/// 0 높이로 퇴화해 p[1] 이 개체 위로 당겨졌다.
#[test]
fn issue_1842_cell_tac_only_line_keeps_stored_height() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let data = fs::read(Path::new(repo_root).join(SAMPLE)).unwrap_or_else(|e| panic!("read: {e}"));
    let core = DocumentCore::from_bytes(&data).expect("load");
    let tree = core.build_page_render_tree(1).expect("render p2");

    // 가장 큰 표(전면 서식 표) 첫 셀의 직속 TextLine y 목록 (내부표 제외)
    fn tables<'a>(n: &'a RenderNode, out: &mut Vec<&'a RenderNode>) {
        if matches!(n.node_type, RenderNodeType::Table(_)) {
            out.push(n);
        }
        for c in &n.children {
            tables(c, out);
        }
    }
    let mut ts = Vec::new();
    tables(&tree.root, &mut ts);
    let big = ts
        .iter()
        .max_by(|a, b| a.bbox.height.partial_cmp(&b.bbox.height).unwrap())
        .expect("전면 서식 표");

    fn cell_lines(n: &RenderNode, in_inner_table: bool, depth: usize, out: &mut Vec<(f64, f64)>) {
        let in_inner =
            in_inner_table || (depth > 0 && matches!(n.node_type, RenderNodeType::Table(_)));
        if !in_inner {
            if let RenderNodeType::TextLine(_) = n.node_type {
                out.push((n.bbox.y, n.bbox.height));
            }
        }
        for c in &n.children {
            cell_lines(c, in_inner, depth + 1, out);
        }
    }
    let mut lines = Vec::new();
    cell_lines(big, false, 0, &mut lines);
    lines.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    // tac 묶음 전용 라인: 저장 lh=3401HU=45.3px 가 렌더 TextLine 높이로 보존되어야
    // 한다. 종전(max_fs=0 축소 퇴화)에는 이 높이의 라인이 존재하지 않았다.
    let tac_line = lines
        .iter()
        .find(|(_, h)| (*h - 45.3).abs() <= 2.0)
        .unwrap_or_else(|| {
            panic!("저장 lh(45.3px) tac-전용 라인 부재 — 축소 퇴화 회귀. lines={lines:?}")
        });

    // 저장 사다리: tac 라인(vpos=0) → 첫 텍스트 문단(vpos=5901HU) = 78.7px.
    // 종전(퇴화): ≈23px. 그룹 내부 텍스트 라인(같은 y 대역)은 +60px 문턱으로 제외.
    let next_y = lines
        .iter()
        .map(|(y, _)| *y)
        .filter(|y| *y >= tac_line.0 + 60.0)
        .fold(f64::INFINITY, f64::min);
    let delta = next_y - tac_line.0;
    assert!(
        (delta - 78.7).abs() <= 6.0,
        "tac-전용 개체 라인 아래 첫 텍스트 줄 간격 {delta:.1}px ≠ 저장 78.7px \
         (max_fs=0 축소 퇴화 회귀: 종전 ~23px)"
    );
}

/// [Issue #1842 / #2063 인계] CellBreak 초대형 표의 **저장 LINE_SEG 부재 셀** 라인높이 회귀.
///
/// `samples/issue2063_huge_cellbreak_table.hwp` (화성시 [별표2], 5,277행×10열 CellBreak)의
/// 데이터 셀들은 저장 LINE_SEG 가 없어(`line_segs.is_empty()`) composer 가 placeholder
/// (line_height=400) 로 합성한다. 종전엔 `corrected_line_height` 가 이를 `max_fs*ls%(160%)`
/// 로 팽창(행 25px)해 rhwp 213쪽으로 과분할(한글 2022 = 162쪽). 수정: CellBreak 표의
/// synthetic 셀 라인높이를 폰트 em(max_fs)으로 — 행 17px, 41행/쪽(한글 정합). RowBreak
/// 규제영향분석서(76076 등)는 현행 유지(#1891 공식 PDF 쪽수 불변).
#[test]
fn issue_1842_cellbreak_synthetic_lineheight_em_not_inflated() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join("samples/issue2063_huge_cellbreak_table.hwp");
    let data = fs::read(&path).unwrap_or_else(|e| panic!("read: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&data).expect("parse");
    let pages = doc.page_count();
    // 수정 전 213쪽(synthetic 셀 ×1.6 팽창), 수정 후 159쪽. 한글 2022 = 162쪽.
    // 하한(150)은 과소, 상한(175)은 synthetic 팽창 재발(→213)을 잡는다.
    assert!(
        (150..=175).contains(&pages),
        "issue1842: CellBreak synthetic 셀 페이지 수 {pages} 가 기대(150..=175, 한글 162) 밖 \
         — 부재 LINE_SEG 셀 라인높이 팽창(수정 전 213) 회귀 의심",
    );
}
