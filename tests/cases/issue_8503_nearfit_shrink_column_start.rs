//! 관계형: PR #8509의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:42:47+09:00 (`6b6572416a8b5d022ea9e027b9c23b14e6afcd98`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8503/1190000-200700045-nearfit-table-indent.hwp`; 독립 Print `pdf/issue8503/1190000-200700045-nearfit-table-indent-2022.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 98.80% / fresh WASM 98.84%.
//! 주장 1쪽 raw: Native 98.80% / fresh WASM 98.84%.
//! 입력 `samples/issue8503/1490000-201000053-nearfit-table-indent.hwp`; 독립 Print `pdf/issue8503/1490000-201000053-nearfit-table-indent-2022.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8509_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-03T09:51:32+09:00 (새 파일).
//! 2026-11-03 측정 raw(96dpi, PDFium, 색상 경계 보정 전 2px 실루엣, `visual_sweep.py --silhouette-only`),
//! 기준은 각 원본의 독립 한컴 2022 12.0.0.4547 Print(pdf/issue8503/*-2022.pdf):
//! - 1490000-201000053(HWP, 1쪽): 1쪽 Native 99.93%, fresh WASM 99.90%. 수정 전 1쪽 Native 79.43%.
//! - 1190000-200700045(HWP, 1쪽): 1쪽 Native 98.62%, fresh WASM 98.69%. 수정 전 1쪽 Native 82.26%.
//!
//! 증거: mydocs/pr/assets/issue_8503/{native,wasm}/*_silhouette.tsv.
//!
//! 기대값은 독립 한컴 Print 괘선과 원본 저장값으로 정한다. 두 원본의 TAC 표는 선언 폭이 본문 폭보다
//! 근소하게 넓고(0.9 ≤ 본문/선언 < 1), 첫 칸 저장 줄이 문단 왼쪽 여백 252 HU 에서 시작한다.
//! 줄 오른끝(252+10724=10976, 252+11572=11824)은 선언 가정의 칸 안쪽 폭에 몇 HU 차로 붙고 축소 가정
//! 쪽과는 수백 HU 떨어진다. 줄 폭만 보면 반대로 축소 쪽에 가깝다 — 한/글은 표를
//! 축소하지 않았다. Print 괘선 폭도 선언 폭이다(620px, 654px). `segment_width` 만 보면 왼쪽 여백만큼
//! 좁게 읽혀 축소로 오판한다. 축소가 맞는 반례(hwpx_sample2 3쪽)와 미해결 상태 고정(Recipe)은
//! `issue_7059_nearfit_shrink_stored_ladder` 가 지킨다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SHORT_FORM: &str = "samples/issue8503/1490000-201000053-nearfit-table-indent.hwp";
const REPORT_FORM: &str = "samples/issue8503/1190000-200700045-nearfit-table-indent.hwp";

fn first_table(node: &RenderNode) -> Option<&RenderNode> {
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return Some(node);
    }
    node.children.iter().find_map(first_table)
}

/// 1쪽 첫 표를 그린 폭과 원본의 선언 폭·본문 폭(px).
fn verify(sample: &str) {
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    let px = |hu: i64| rhwp::renderer::hwpunit_to_px(hu as i32, rhwp::renderer::DEFAULT_DPI);
    let section = &core.document().sections[0];
    let page_def = &section.section_def.page_def;
    let body_hu = i64::from(page_def.width)
        - i64::from(page_def.margin_left)
        - i64::from(page_def.margin_right);
    let table = section
        .paragraphs
        .iter()
        .flat_map(|p| p.controls.iter())
        .find_map(|c| match c {
            rhwp::model::control::Control::Table(t) => Some(t),
            _ => None,
        })
        .expect("TAC 표");
    let declared_hu = i64::from(table.common.width);
    // 전제: 근사-축소 규칙이 판정하는 표(본문/선언 ≥ 0.9, 선언 > 본문)이고, 첫 칸 저장 줄이
    // 왼쪽 여백에서 시작해 오른끝이 선언 안쪽 폭에 닿는다.
    assert!(table.common.treat_as_char, "{sample}: TAC 표");
    assert!(
        declared_hu > body_hu && body_hu * 10 >= declared_hu * 9,
        "{sample}: 근사-초과 표(선언 {declared_hu} HU, 본문 {body_hu} HU)"
    );
    let cell = table
        .cells
        .iter()
        .find(|c| c.row == 0 && c.col == 0)
        .expect("첫 칸");
    let seg = &cell.paragraphs[0].line_segs[0];
    let pad = cell.effective_padding(&table.padding);
    let pad_h = i64::from(pad.left) + i64::from(pad.right);
    let declared_inner = i64::from(cell.width) - pad_h;
    let shrunk_inner = i64::from(cell.width) * body_hu / declared_hu - pad_h;
    let line_right = i64::from(seg.column_start) + i64::from(seg.segment_width);
    let line_width = i64::from(seg.segment_width);
    // 이 시험이 지키는 입력: 줄 폭만 보면 축소 쪽에 가깝고, 줄 오른끝은 선언 쪽에 가깝다.
    assert!(
        seg.column_start > 0,
        "{sample}: 첫 칸 줄은 왼쪽 여백에서 시작한다"
    );
    assert!(
        (line_width - shrunk_inner).abs() < (line_width - declared_inner).abs(),
        "{sample}: 줄 폭 {line_width} 만 보면 축소 안쪽 {shrunk_inner} 쪽(선언 {declared_inner})"
    );
    assert!(
        (line_right - declared_inner).abs() < (line_right - shrunk_inner).abs(),
        "{sample}: 줄 오른끝 {line_right} 은 선언 안쪽 {declared_inner} 쪽(축소 {shrunk_inner})"
    );

    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let drawn = first_table(&tree.root).expect("1쪽 표").bbox.width;
    let (declared, body) = (px(declared_hu), px(body_hu));
    // 한/글은 축소하지 않는다: 그린 폭은 선언 폭이고, 본문 폭보다 넓다.
    assert!(
        (drawn - declared).abs() <= 1.0,
        "{sample}: 그린 표 폭 {drawn:.1}px 은 선언 폭 {declared:.1}px 이어야 한다(본문 {body:.1}px)"
    );
    assert!(
        drawn > body + 5.0,
        "{sample}: 표가 본문 폭 {body:.1}px 로 축소되면 안 된다(그린 폭 {drawn:.1}px)"
    );
}

/// 1490000-201000053(HWP): 그린 폭 620.6px(선언), 본문 604.7px. 한컴 Print 괘선 96→716px(620px).
#[test]
fn nearfit_table_with_indented_first_cell_line_keeps_declared_width() {
    verify(SHORT_FORM);
}

/// 1190000-200700045(HWP, rhwp-issue-docs#658): 선언 49089 HU(654.5px), 본문 642.5px.
/// 한컴 Print 괘선 77→731px(654px).
#[test]
fn nearfit_table_with_indented_first_cell_line_keeps_declared_width_report_form() {
    verify(REPORT_FORM);
}
