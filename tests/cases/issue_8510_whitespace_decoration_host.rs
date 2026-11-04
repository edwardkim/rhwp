//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8512의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T19:42:50+09:00 (`7aba215a29e97651d3c6baf195b55d1b1abc2b6b`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8510/156500952-k-startup-league.hwpx`; 독립 Print `pdf/issue8510/156500952-k-startup-league-2022.pdf`.
//! 전체 5쪽, 전쪽 최저 Native 99.26% / fresh WASM 99.88%.
//! 주장 4쪽 raw: Native 99.89% / fresh WASM 99.99%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8512_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-03T12:30:00+09:00 (새 파일, 직전 파일 커밋 없음).
//! 2026-11-03 측정 raw(`silhouette_raw_match_percent`, 96dpi, visual_sweep --silhouette-only,
//! base 4e2cc6bb36 + 이 수정, Native skia 11:48+09:00 / fresh WASM 11:49+09:00):
//! - 156500952(HWPX, 한컴 5쪽 = rhwp 5쪽): Native 99.91/99.97/99.85/99.94/99.97%, fresh WASM
//!   99.97/99.98/99.89/99.96/100.00%; 전5쪽 최저 Native 99.85% / WASM 99.89%(3쪽).
//! - 수정 전(base) Native 4쪽 72.54%(다른 쪽은 같은 값).
//!
//! 근거: mydocs/pr/assets/issue_8510/{native,wasm}/*_silhouette.tsv.
//!
//! 공백만 있는 문단에 데코레이션 표(글앞으로/글뒤로)가 앵커되면, 그 문단 줄은 저장 사다리가
//! 증언하는 만큼 흐름을 차지한다(#8510).
//!
//! 수정 전 rhwp 는 `decoration_host::plan` 의 두 분기에서 모두 빠졌다 — 첫 분기는 공백 아닌
//! 글자가 있어야 하고, #7047 분기는 `para_has_visible_text`(공백도 글자로 셈)로 걸렀다. 그 문단
//! 줄이 흐름 0 이 되어 뒤 비-TAC 표가 그 줄높이만큼 위로 올라가 앞 글줄 위에 겹쳤다.
//!
//! 기대값은 원본 저장 LINE_SEG·개체 속성과 독립 한컴 2022 12.0.0.4547 Print(`pdf/issue8510/*-2022.pdf`)로 정한다.
//! - 156500952 K-스타트업 참가자 모집(HWPX, 한컴 5쪽) 4쪽: 문단 0.74(공백 1 + 글앞으로 표) vpos 34798·
//!   lh 1500·ls 600, 문단 0.75 vpos 36898. 문단 0.79 줄 vpos 43618, 문단 0.81 비-TAC 자리차지 표
//!   (세로=문단 오프셋 63, 바깥 위 140) 줄 vpos 46418.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn hu(v: f64) -> f64 {
    v * 96.0 / 7200.0
}

/// 본문(표 밖) 글줄의 (문단, 위끝, 아래끝)과 표의 (문단, 컨트롤, 위끝).
fn collect(
    node: &RenderNode,
    in_table: bool,
    lines: &mut Vec<(usize, f64, f64)>,
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
                lines.push((p, node.bbox.y, node.bbox.y + node.bbox.height));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, inside, lines, tables);
    }
}

#[test]
fn whitespace_only_decoration_host_line_occupies_stored_advance() {
    let sample = "samples/issue8510/156500952-k-startup-league.hwpx";
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(
        core.page_count(),
        5,
        "{sample}: 독립 한/글 2022 Print 전체 쪽수"
    );

    let tree = core.build_page_render_tree(3).expect("4쪽 render tree");
    let (mut lines, mut tables) = (Vec::new(), Vec::new());
    collect(&tree.root, false, &mut lines, &mut tables);
    let line = |para: usize| {
        lines
            .iter()
            .find(|l| l.0 == para)
            .copied()
            .unwrap_or_else(|| panic!("{sample}: 4쪽 문단 {para} 글줄 없음 {lines:?}"))
    };
    let table_top = tables
        .iter()
        .find(|t| t.0 == 81 && t.1 == 0)
        .map(|t| t.2)
        .unwrap_or_else(|| panic!("{sample}: 4쪽 문단 81 표 없음 {tables:?}"));

    // 공백만 있는 문단 0.74 의 줄(1500 + 600 HU)이 흐름을 차지하면, 문단 0.81 표 위끝은
    // 문단 0.79 줄 위끝에서 저장 vpos 차 + 세로 오프셋 + 바깥 위 여백만큼 아래다.
    let (p79, p80) = (line(79), line(80));
    let expected = hu(46418.0 - 43618.0 + 63.0 + 140.0);
    assert!(
        (table_top - p79.1 - expected).abs() <= 1.0,
        "{sample}: 문단 0.79 줄 위끝→문단 0.81 표 위끝 {:.2} ≠ 저장 사다리 {expected:.2}",
        table_top - p79.1
    );
    // 표는 앞 문단 0.80 글줄 아래에서 시작한다(겹침 없음).
    assert!(
        table_top >= p80.2 - 0.5,
        "{sample}: 문단 0.81 표 위끝 {table_top:.2} 이 문단 0.80 글줄 아래끝 {:.2} 보다 위(겹침)",
        p80.2
    );
}
