//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-04T14:40:00+09:00 (새 파일, 직전 파일 커밋 없음).
//! 2026-11-04 측정 raw(`silhouette_raw_match_percent`, 96dpi, visual_sweep --silhouette-only, compat 2022,
//! base 1924d371d6 + 이 수정, Native skia / fresh WASM 14:30+09:00):
//! - 20572683 부산 조례안 재의요구안 별지(HWP, 한컴 1쪽 = rhwp 1쪽): Native 99.99%, fresh WASM 99.29%;
//!   전1쪽 최저 Native 99.99% / WASM 99.29%.
//! - 수정 전(base 1924d371d6) 1쪽 Native 54.06% / WASM 54.54%.
//!
//! 근거: mydocs/pr/assets/issue_8567/{head,base}/{native,wasm}/i447_silhouette.tsv.
//!
//! 어울림(Square) 표가 뒤 문단들을 표 옆 띠로 통째로 흡수하면, 띠 끝은 마지막 흡수 문단의
//! 저장 끝(= 흐름상 다음 문단의 저장 줄)이다. 그 문단이 이미 흐름 위치에 있으면 host 줄의
//! 줄간격을 다시 더하지 않는다(#8567). 수정 전 rhwp 는 띠 끝 뒤에 host 줄간격 652 HU(8.7 px)를
//! 한 번 더 더해 본문 전체가 내려갔다.
//!
//! 기대값은 원본 저장값과 독립 한/글 Print(`pdf/issue8567/*-2022.pdf`)로 정한다.
//! - 20572683 부산 조례안 재의요구안 별지(docs#447, 1쪽): 문단 0.4 = 빈 host(줄 vpos 5462,
//!   줄높이 1300, 줄간격 652) + 1×2 어울림 표. 문단 0.5~0.7 이 표 옆 띠(cs 11904)에 놓이고
//!   0.7 끝 10614 + 1000 + 600 = 12214 = 문단 0.8 줄. 이후 0.10 첫 줄 16118 까지 사다리가
//!   연속한다. 한/글 Print 에서 0.10 첫 줄은 0.7 줄보다 (16118 − 10614) HU 아래다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn hu(v: f64) -> f64 {
    v * 96.0 / 7200.0
}

/// 본문(표 밖) 글줄의 (문단, 줄, 위끝, 왼끝).
fn collect(node: &RenderNode, in_table: bool, lines: &mut Vec<(usize, u32, f64, f64)>) {
    let inside = in_table || matches!(node.node_type, RenderNodeType::Table(_));
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if let (false, Some(p), Some(l)) = (inside, line.para_index, line.line_index) {
            lines.push((p, l, node.bbox.y, node.bbox.x));
        }
    }
    for child in &node.children {
        collect(child, inside, lines);
    }
}

#[test]
fn square_wrap_successor_already_on_ladder_skips_host_line_spacing() {
    let sample = "samples/issue8567/20572683-busan-ordinance-reconsideration-form.hwp";
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(core.page_count(), 1, "{sample}: 독립 한/글 Print 전체 쪽수");

    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut lines = Vec::new();
    collect(&tree.root, false, &mut lines);
    let line = |para: usize, idx: u32| {
        lines
            .iter()
            .find(|l| l.0 == para && l.1 == idx)
            .copied()
            .unwrap_or_else(|| panic!("{sample}: 1쪽 문단 {para} 줄 {idx} 없음 {lines:?}"))
    };

    // 표 옆 어울림 문단 0.5~0.7 은 전폭 본문보다 오른쪽 띠에 놓인다.
    let body_left = line(10, 0).3;
    for para in 5..=7 {
        assert!(
            line(para, 0).3 > body_left + hu(11904.0) - 1.0,
            "{sample}: 문단 {para} 가 표 옆 띠(cs 11904)에 없음"
        );
    }
    // 띠 뒤 문단은 저장 순서대로 하나씩 아래로 이어진다(누락·중복 없음).
    let order: Vec<usize> = [7, 8, 9, 10, 14, 17, 19, 25].to_vec();
    for pair in order.windows(2) {
        assert!(
            line(pair[0], 0).2 < line(pair[1], 0).2,
            "{sample}: 문단 {} 가 문단 {} 보다 아래가 아님",
            pair[1],
            pair[0]
        );
    }
    // 띠 마지막 문단 0.7 → 본문 0.10 첫 줄 = 저장 사다리 간격(host 줄간격 이중 가산 없음).
    let gap = line(10, 0).2 - line(7, 0).2;
    let expected = hu(16118.0 - 10614.0);
    assert!(
        (gap - expected).abs() <= 1.0,
        "{sample}: 문단 0.7 → 0.10 글줄 간격 {gap:.2} ≠ 저장 사다리 {expected:.2}"
    );
    // 쪽 끝 「※ 조례안 첨부」(0.25)까지 같은 사다리.
    let tail = line(25, 0).2 - line(7, 0).2;
    let tail_expected = hu(46134.0 - 10614.0);
    assert!(
        (tail - tail_expected).abs() <= 1.0,
        "{sample}: 문단 0.7 → 0.25 글줄 간격 {tail:.2} ≠ 저장 사다리 {tail_expected:.2}"
    );
}
