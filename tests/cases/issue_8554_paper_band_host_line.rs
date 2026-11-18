//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-04T19:41:21+09:00 (새 파일, 직전 파일 커밋 없음).
//! 2026-11-04 측정 raw(`silhouette_raw_match_percent`, 96dpi, visual_sweep --silhouette-only, compat 2022,
//! base 1924d371d6 + 이 수정, Native skia / fresh WASM 16:40+09:00):
//! - 2913671 별지 1(HWP, 한컴 2쪽 = rhwp 2쪽): Native 95.98/100.00%, fresh WASM 99.44/100.00%;
//!   전2쪽 최저 Native 95.98%(1쪽) / WASM 99.44%(1쪽).
//!   Native 1쪽 잔여는 base 에도 있는 #8565 한글 subset 웹폰트의 한자 누락(#8590; rebase 전 base
//!   7d06d5c706 에서는 99.51%).
//! - 수정 전(base 1924d371d6) 1쪽 Native 47.45% / WASM 47.26%(2쪽은 같은 값).
//!
//! 근거: mydocs/pr/assets/issue_8554/{head,base}/{native,wasm}/i39_silhouette.tsv.
//!
//! 쪽·용지 기준 자리차지(비-TAC) 표의 빈 host 줄이 저장 자리에서 그 표 띠에 걸려 띠 아래로
//! 밀리면, 그 줄은 표가 대신한 줄이 아니라 띠 아래의 독립 줄이다 — 줄높이와 줄간격을 모두
//! 차지한다(#8554). 수정 전 rhwp 는 띠 아래로 점프한 뒤 줄간격만 전진해 뒤 본문 전체가
//! 줄높이(17.3 px)만큼 올라갔다.
//!
//! 기대값은 원본 저장값과 독립 한/글 Print(`pdf/issue8554/*-2022.pdf`, Hwp 2022 12.0.0.4547)로 정한다.
//! - 2913671 「별지 1」(docs#39, 2쪽): 문단 0.2 는 빈 문단(저장 vpos 5200, 줄높이 1300, 줄간격 1300)에
//!   1×2 표 하나(용지 기준 세로 15192 HU, 높이 6876 HU, 바깥 여백 140 HU, 자리차지). 표 띠는
//!   204.4–298.0 px 이고 host 저장 줄 위끝 197.8 px 가 띠에 걸린다. 한/글 Print 의 「2.」(문단 0.3)
//!   줄 위끝은 띠 아래끝 + 34.7 px(= 1300 + 1300 HU) 이다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn hu(v: f64) -> f64 {
    v * 96.0 / 7200.0
}

/// 본문(표 밖) 글줄의 (문단, 위끝)과 표의 (문단, 위끝, 아래끝).
fn collect(
    node: &RenderNode,
    in_table: bool,
    lines: &mut Vec<(usize, f64)>,
    tables: &mut Vec<(usize, f64, f64)>,
) {
    let mut inside = in_table;
    match &node.node_type {
        RenderNodeType::Table(t) => {
            if let Some(p) = t.para_index {
                if !in_table {
                    tables.push((p, node.bbox.y, node.bbox.y + node.bbox.height));
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
fn host_line_pushed_below_paper_band_keeps_its_line_height() {
    let sample = "samples/issue8554/2913671-appendix1-family-register.hwp";
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(core.page_count(), 2, "{sample}: 독립 한/글 Print 전체 쪽수");

    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let (mut lines, mut tables) = (Vec::new(), Vec::new());
    collect(&tree.root, false, &mut lines, &mut tables);
    let table = |para: usize| {
        tables
            .iter()
            .find(|t| t.0 == para)
            .copied()
            .unwrap_or_else(|| panic!("{sample}: 1쪽 문단 {para} 표 없음 {tables:?}"))
    };
    let line = |para: usize| {
        lines
            .iter()
            .find(|l| l.0 == para)
            .map(|l| l.1)
            .unwrap_or_else(|| panic!("{sample}: 1쪽 문단 {para} 글줄 없음 {lines:?}"))
    };
    let (band_table, next_table) = (table(2), table(4));
    // 용지 기준 표 띠 아래끝 = 표 아래끝 + 바깥 아래 여백 140 HU.
    let band_bottom = band_table.2 + hu(140.0);
    // host 저장 줄(문단 첫 글줄 1쪽 위끝 + vpos 5200)이 띠에 걸린다 — 이 검사가 다루는 형상.
    let host_stored_top = line(0) + hu(5200.0);
    assert!(
        host_stored_top < band_bottom && host_stored_top + hu(1300.0) > band_table.1,
        "{sample}: host 저장 줄 {host_stored_top:.2} 이 표 띠 {:.2}..{band_bottom:.2} 에 걸려야 한다",
        band_table.1
    );
    // 밀린 host 줄은 띠 아래에서 줄높이 + 줄간격(1300 + 1300 HU)을 차지하고 「2.」 가 그 뒤에 선다.
    let expected = hu(1300.0 + 1300.0);
    let gap = line(3) - band_bottom;
    assert!(
        (gap - expected).abs() <= 1.0,
        "{sample}: 표 띠 아래끝→문단 0.3 글줄 위끝 {gap:.2} ≠ host 줄 전진 {expected:.2}"
    );
    // 뒤 글자처럼 취급 표(문단 0.4)는 「2.」 줄 뒤에 온다(순서·겹침 없음).
    assert!(
        next_table.1 >= line(3) + hu(1300.0),
        "{sample}: 문단 0.4 표 위끝 {:.2} 이 문단 0.3 글줄 아래끝 {:.2} 보다 위",
        next_table.1,
        line(3) + hu(1300.0)
    );
}
