//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8508의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:42:45+09:00 (`b24c9a7430e71bf975d3f9227d6a9ee88f3134e2`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8502/36316493-approval-body.hwpx`; 독립 Print `pdf/issue8502/36316493-approval-body-2022.pdf`.
//! 전체 5쪽, 전쪽 최저 Native 99.87% / fresh WASM 99.72%.
//! 주장 3쪽 raw: Native 99.90% / fresh WASM 99.96%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8508_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-03T09:40:00+09:00 (새 파일, 직전 파일 커밋 없음).
//! 2026-11-03 측정 raw(`silhouette_raw_match_percent`, 96dpi, visual_sweep --silhouette-only,
//! base 4e2cc6bb36 + 이 수정, Native skia 09:13+09:00 / fresh WASM 09:14+09:00):
//! - 36316493(HWPX, 한컴 5쪽 = rhwp 5쪽): Native 100.00/100.00/99.68/99.87/99.87%, fresh WASM
//!   100.00/99.96/99.96/99.91/99.68%; 전5쪽 최저 Native 99.68%(3쪽) / WASM 99.68%(5쪽).
//! - 36440652(HWPX, 한컴 3쪽 = rhwp 3쪽, 대조 문서·저장소 미포함 — 10k 코퍼스): Native 100.00/92.93/100.00%,
//!   fresh WASM 100.00/92.11/100.00%; 최저 92.93% / 92.11%(2쪽).
//! - 수정 전(base) Native: 36316493 2쪽 87.84%·3쪽 37.27%, 36440652 2쪽 54.80%.
//!
//! 근거: mydocs/pr/assets/issue_8502/{native,wasm}/*_silhouette.tsv.
//!
//! 저장 줄 하나에 TAC 표 하나만 있는 문단은, 다음 문단 저장 원점이 `줄 높이 + 줄 간격 +
//! 이 문단 아래 간격 + 다음 문단 위 간격` 으로 정확히 닫히면 줄 간격 전량을 소비한다(#8502).
//!
//! 수정 전 rhwp 는 이 닫힘 등식에서 문단 간격을 빼 등식이 거짓이 되었고, 줄 간격의 절반만
//! 흐름에 더했다. 다음 문단이 구현 태그 줄이면 지연 기준축이 그 끝으로 다시 잡혀 쪽 끝까지
//! 위로 당겨졌다.
//!
//! 기대값은 원본 저장 LINE_SEG 와 독립 한컴 2022 12.0.0.4547 Print(`pdf/issue8502/*-2022.pdf`)로 정한다.
//! - 36316493(HWPX, 한컴 5쪽) 2쪽: 문단 10 TAC 표(4×7, 바깥 위 141 HU) 줄 vpos 108047·lh 10903·
//!   ls 1360, 문단 아래 1000·문단 11 위 1000(각 저장값의 절반 500 HU 를 소비), 문단 11 줄 vpos 121310.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn hu(v: f64) -> f64 {
    v * 96.0 / 7200.0
}

/// 본문(표 밖) 글줄의 (문단, 위끝)과 표의 (문단, 컨트롤, 위끝).
fn collect(
    node: &RenderNode,
    in_table: bool,
    lines: &mut Vec<(usize, f64)>,
    tables: &mut Vec<(usize, usize, f64)>,
) {
    let mut inside = in_table;
    match &node.node_type {
        RenderNodeType::Table(t) => {
            if let (Some(p), Some(c)) = (t.para_index, t.control_index) {
                if !in_table {
                    tables.push((p, c, node.bbox.y));
                }
            }
            inside = true;
        }
        RenderNodeType::TextLine(line) if !in_table => {
            if let Some(p) = line.para_index {
                lines.push((p, node.bbox.y));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, inside, lines, tables);
    }
}

#[test]
fn single_tac_table_line_consumes_full_spacing_when_stored_ladder_closes() {
    let sample = "samples/issue8502/36316493-approval-body.hwpx";
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(
        core.page_count(),
        5,
        "{sample}: 독립 한/글 2022 Print 전체 쪽수"
    );

    let tree = core.build_page_render_tree(1).expect("2쪽 render tree");
    let (mut lines, mut tables) = (Vec::new(), Vec::new());
    collect(&tree.root, false, &mut lines, &mut tables);
    let table_top = tables
        .iter()
        .find(|t| t.0 == 10 && t.1 == 0)
        .map(|t| t.2)
        .unwrap_or_else(|| panic!("{sample}: 2쪽 문단 10 표 없음 {tables:?}"));
    let heading_top = lines
        .iter()
        .filter(|l| l.0 == 11)
        .map(|l| l.1)
        .fold(f64::INFINITY, f64::min);
    assert!(
        heading_top.is_finite(),
        "{sample}: 2쪽 문단 11 글줄 없음 {lines:?}"
    );

    // 표 상자 위끝 = 줄 위끝 + 바깥 위 여백. 문단 11 줄 위끝 = 문단 10 줄 위끝 + 저장 vpos 차.
    let expected = hu(121310.0 - 108047.0 - 141.0);
    assert!(
        (heading_top - table_top - expected).abs() <= 1.0,
        "{sample}: 표 위끝→다음 문단 줄 위끝 {:.2} ≠ 저장 사다리 {expected:.2}",
        heading_top - table_top
    );
}
