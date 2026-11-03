//! 검사 유형: 관계형·고정형.
//! 주석 갱신: 2026-11-04T16:55:00+09:00 (새 파일, 직전 파일 커밋 없음).
//! 2026-11-04 측정 raw(`silhouette_raw_match_percent`, 96dpi, visual_sweep --silhouette-only,
//! base 1924d371d6 + 이 수정, Native skia / fresh WASM 16:30+09:00):
//! - 1051000-202000124 표지(HWP, 한컴 4쪽 = rhwp 4쪽): Native 99.94/100.00/99.04/99.63%, fresh WASM
//!   99.93/100.00/99.86/97.06%; 전4쪽 최저 Native 99.04%(3쪽) / WASM 97.06%(4쪽).
//! - 수정 전(base 1924d371d6) 1쪽 Native 18.60% / WASM 17.95%(다른 쪽은 같은 값).
//! - 원본 209쪽 HWP(10k 코퍼스 prism 기획예산처) 1쪽도 Native 18.76% → 99.92%.
//!
//! 근거: mydocs/pr/assets/issue_8516/{native,wasm}/*_silhouette.tsv.
//!
//! 글자 없는(공백만 있는) 문단에 문단 기준 자리차지 표 둘이 공동 앵커되고, 두 표가 모두 양수
//! 세로 오프셋이며 가로로 겹치지 않으면 각 표는 자기 오프셋에 나란히 놓인다(#8516).
//!
//! 수정 전 rhwp 는 조판(`place_table_with_text` 의 공동 앵커 술어가 첫 표 오프셋 0 만 인정)과
//! 배치(`layout.rs` 표 exclusion 보정이 같은 host 앞 표의 밴드로 끌어내림) 모두에서 오른쪽 표를
//! 왼쪽 표 아래로 쌓았다. 그 결과 오른쪽 표와 뒤 본문 전체가 약 47 px 아래로 밀렸다.
//!
//! 기대값은 원본 저장값과 독립 한/글 2020 Print(`pdf/issue8516/*-2020.pdf`)로 정한다.
//! - 1051000-202000124 「재정성과 향상을 위한 증거기반 정책평가 연구」 표지(원본 209쪽 HWP 의
//!   1쪽을 `rhwp extract-pages` 로 남기고, 남은 구역 3 첫 문단 제목을 `rhwp edit delete-text` 로
//!   「제3절」만 남긴 뒤(원본 제목의 라틴 글자 간격 차이 — 이번 수정과 무관한 base 동일 85% 쪽 — 제외)
//!   한/글 2020 11.0.0.9136 으로 다시 저장, 4쪽. 1쪽 표지 문단은 손대지 않았다).
//!   문단 0.0(공백 47자): 표 ci2(2×1, 세로 1395, 가로 단 0), 표 ci3(1×1, 세로 2666, 가로 단 23823),
//!   두 표 바깥 여백 141. 문단 0.1 저장 vpos 6494 = 2666 + 141 + 3546(ci3 높이) + 141 — 다음 문단은
//!   ci3 바닥(아래 표)에서 시작하고, ci2 바닥에 ci3 를 더 쌓은 높이가 아니다.
//!   문단 0.6 저장 vpos 13254.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn hu(v: f64) -> f64 {
    v * 96.0 / 7200.0
}

/// 본문(표 밖) 글줄의 (문단, 위끝)과 표의 (문단, 컨트롤, bbox).
fn collect(
    node: &RenderNode,
    in_table: bool,
    lines: &mut Vec<(usize, f64)>,
    tables: &mut Vec<(usize, usize, f64, f64, f64, f64)>,
) {
    let mut inside = in_table;
    match &node.node_type {
        RenderNodeType::Table(t) => {
            if let (Some(p), Some(c)) = (t.para_index, t.control_index) {
                if !in_table {
                    let b = &node.bbox;
                    tables.push((p, c, b.x, b.y, b.x + b.width, b.y + b.height));
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
fn textless_host_side_by_side_offset_floats_keep_own_offsets() {
    let sample = "samples/issue8516/1051000-202000124-evidence-policy-cover.hwp";
    let bytes = std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample))
        .expect("시각 검증한 원본");
    let core = DocumentCore::from_bytes(&bytes).expect("원본 배치");
    assert_eq!(
        core.page_count(),
        4,
        "{sample}: 독립 한/글 2020 Print 전체 쪽수"
    );

    let tree = core.build_page_render_tree(0).expect("1쪽 render tree");
    let (mut lines, mut tables) = (Vec::new(), Vec::new());
    collect(&tree.root, false, &mut lines, &mut tables);
    let table = |control: usize| {
        tables
            .iter()
            .find(|t| t.0 == 0 && t.1 == control)
            .copied()
            .unwrap_or_else(|| panic!("{sample}: 1쪽 문단 0 표 {control} 없음 {tables:?}"))
    };
    let (left, right) = (table(2), table(3));

    // 두 표는 가로로 겹치지 않는다(오른쪽 표는 왼쪽 표 오른쪽 끝 뒤).
    assert!(
        right.2 >= left.4,
        "{sample}: 표 ci3 왼쪽 {:.2} 이 표 ci2 오른쪽 {:.2} 앞",
        right.2,
        left.4
    );
    // 오른쪽 표는 왼쪽 표 아래로 쌓이지 않고 자기 세로 오프셋에 놓인다(바깥 위 여백은 같다).
    let expected = hu(2666.0 - 1395.0);
    assert!(
        (right.3 - left.3 - expected).abs() <= 1.0,
        "{sample}: 표 ci2 위끝→표 ci3 위끝 {:.2} ≠ 세로 오프셋 차 {expected:.2}",
        right.3 - left.3
    );
    // 뒤 본문 첫 글줄(문단 0.6)은 저장 사다리 위치 — 표 ci2 위끝에서 vpos 13254 − (1395 + 141).
    let title = lines
        .iter()
        .find(|l| l.0 == 6)
        .map(|l| l.1)
        .unwrap_or_else(|| panic!("{sample}: 1쪽 문단 6 글줄 없음 {lines:?}"));
    let expected = hu(13254.0 - 1395.0 - 141.0);
    assert!(
        (title - left.3 - expected).abs() <= 1.0,
        "{sample}: 표 ci2 위끝→문단 0.6 글줄 위끝 {:.2} ≠ 저장 사다리 {expected:.2}",
        title - left.3
    );
}
