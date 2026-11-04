//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-04T06:30:00+09:00 (새 파일, 직전 파일 커밋 없음).
//! 2026-11-04 측정 raw(`silhouette_raw_match_percent`, 96dpi, visual_sweep --silhouette-only, compat 2022,
//! base 7d06d5c706 + 이 수정, Native skia / fresh WASM 07:30+09:00):
//! - 21138757 안건표지 별지(HWP, 한컴 1쪽 = rhwp 1쪽): Native 93.75%, fresh WASM 93.40%;
//!   전1쪽 최저 Native 93.75% / WASM 93.40%.
//! - 수정 전(base 7d06d5c706) 1쪽 Native 49.72% / WASM 49.90%.
//!
//! 근거: mydocs/pr/assets/issue_8560/{head,base}/{native,wasm}/i470_silhouette.tsv.
//!
//! 글자 없는 빈 host 의 문단 기준 자리차지 표 뒤에 저장 host 줄이 있고, 다음 문단도 글자 없는
//! 자리차지 표 host 이면 그 host 줄은 두 표 사이에서 줄높이와 줄간격을 차지한다(#8560).
//! 수정 전 rhwp 의 조판은 다음 문단을 앞 표 띠 아래끝에서 시작해 뒤 표와 그 아래 내용이
//! 23.5 px(1100 + 660 HU) 올라갔다.
//!
//! 기대값은 원본 저장값과 독립 한/글 Print(`pdf/issue8560/*-2020.pdf`)로 정한다.
//! - 21138757 안건표지 별지(docs#470, 1쪽): 문단 0.8 은 빈 host(줄 vpos 35780, 줄높이 1100,
//!   줄간격 660)에 1×1 표(오프셋 0, 바깥 여백 284 HU)를 품는다. 저장 등식은 앞 줄 끝 27704 +
//!   바깥 상자 8076 = host 줄 35780, host 줄 끝 35780 + 1760 = 문단 0.9 줄 37540 이다.
//!   문단 0.9 의 2×2 표(오프셋 1685 HU)는 한/글 Print 에서 37540 + 1685 + 284 HU 에 선다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn hu(v: f64) -> f64 {
    v * 96.0 / 7200.0
}

/// 본문(표 밖) 표의 (문단, 위끝, 아래끝).
fn collect(node: &RenderNode, in_table: bool, tables: &mut Vec<(usize, f64, f64)>) {
    let mut inside = in_table;
    if let RenderNodeType::Table(t) = &node.node_type {
        if let Some(p) = t.para_index {
            if !in_table {
                tables.push((p, node.bbox.y, node.bbox.y + node.bbox.height));
            }
        }
        inside = true;
    }
    for child in &node.children {
        collect(child, inside, tables);
    }
}

#[test]
fn textless_float_host_line_before_next_float_host_keeps_its_advance() {
    let sample = "samples/issue8560/21138757-agenda-cover-form.hwp";
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(core.page_count(), 1, "{sample}: 독립 한/글 Print 전체 쪽수");

    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let mut tables = Vec::new();
    collect(&tree.root, false, &mut tables);
    let table = |para: usize| {
        tables
            .iter()
            .find(|t| t.0 == para)
            .copied()
            .unwrap_or_else(|| panic!("{sample}: 1쪽 문단 {para} 표 없음 {tables:?}"))
    };
    // 표 순서: 0.6 → 0.8 → 0.9 → 0.12 (누락·중복 없음).
    let order: Vec<usize> = tables.iter().map(|t| t.0).collect();
    assert_eq!(order, vec![6, 8, 9, 12], "{sample}: 1쪽 본문 표 소속·순서");

    let (prev, next) = (table(8), table(9));
    // 문단 0.8 띠 아래끝 = 표 아래끝 + 바깥 아래 여백 284 HU. 그 아래 host 줄(1100 + 660 HU)
    // 뒤에서 문단 0.9 가 시작하고, 표는 거기서 오프셋 1685 + 바깥 위 284 HU 아래다.
    let band_bottom = prev.2 + hu(284.0);
    let expected = hu(1100.0 + 660.0 + 1685.0 + 284.0);
    let gap = next.1 - band_bottom;
    assert!(
        (gap - expected).abs() <= 1.0,
        "{sample}: 문단 0.8 띠 아래끝→문단 0.9 표 윗변 {gap:.2} ≠ host 줄 + 오프셋 {expected:.2}"
    );
    // 뒤 표들도 겹치지 않고 아래로 이어진다.
    assert!(
        table(12).1 >= next.2 + hu(284.0),
        "{sample}: 문단 0.12 표 {:.2} 가 문단 0.9 표 띠 {:.2} 와 겹침",
        table(12).1,
        next.2 + hu(284.0)
    );
}
