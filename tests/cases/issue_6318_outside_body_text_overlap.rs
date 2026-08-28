//! [#6318] `layout-anomaly` 글자 겹침 후보가 본문 밖(바탕쪽·머리말·꼬리말·각주)까지
//! 닿는지 고정한다.
//!
//! 종전 `scan_page` 는 페이지 트리에서 `Body` 하나만 순회했다. 본문 글자가 바탕쪽
//! 사이드바를 덮어도 짝이 애초에 후보가 아니라 신호가 0 이었고, 사람이 렌더 이미지를
//! 봐야만 알 수 있었다(#5952 의 "사이드바와 겹친다", PR #6083 검토 실사례).
//!
//! 두 축을 함께 고정한다.
//!
//! 1. **잡아야 하는 것** — 편람 69쪽에서 본문 줄이 바탕쪽 "제1절" 라벨 위에 얹힌다.
//! 2. **건드리지 말아야 하는 것** — 같은 쪽의 컨테이너 판정(overflow·off-canvas·
//!    overlap)은 종전 수치 그대로다.
//!
//! 합성 렌더 트리를 만들지 않고 실제 샘플로 판정한다. `TextRunNode` 는 필드가 21 개라
//! 손으로 지으면 시험이 구조 변경마다 깨지고, 무엇보다 이 결함은 **실제 문서의
//! 조판에서** 나온 것이라 실물로 고정하는 편이 회귀를 정확히 막는다.
//!
//! 이 시험이 다루지 않는 두 성질은 코퍼스 래칫이 더 넓게 지킨다
//! (`tests/cases/text_overlap_baseline.rs`, samples 945 건).
//!
//! - **다른 단(column) 짝짓기 제외**: 규칙이 풀리면 다단 문서 전반에서 겹침이
//!   폭증한다. 래칫의 `Body x Body` 합계가 4,371 로 이 변경 전후 동일하다.
//! - **바탕쪽 전면 배경의 컨테이너 오탐**: 편람 바탕쪽은 종이 전체를 덮는
//!   `Image x=0..740.8 y=0..1014.4` 를 갖는다. 아래 컨테이너 시험이 같은 쪽에서
//!   `overlap == 0` 을 고정하므로 배경이 흐름 후보로 새면 바로 실패한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::diagnostics::layout_anomaly::{scan_page, AnomalyOptions, PageAnomalies};
use rhwp::document_core::DocumentCore;

const HANDBOOK: &str = "samples/2025 행정업무운영 편람(최종).hwp";
/// `-p` 는 0 기준. 한글 인쇄 쪽번호 61 은 rhwp 69쪽(-p 68)이다.
const HANDBOOK_PAGE: u32 = 68;

fn scan_handbook_page() -> PageAnomalies {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(HANDBOOK);
    let bytes = std::fs::read(&path).expect("편람 샘플 읽기");
    let doc = DocumentCore::from_bytes(&bytes).expect("편람 파싱");
    let tree = doc
        .build_page_render_tree(HANDBOOK_PAGE)
        .expect("편람 69쪽 렌더 트리");
    scan_page(
        HANDBOOK_PAGE,
        &tree.root,
        doc.page_count(),
        &AnomalyOptions::default(),
    )
}

/// 본문 글자가 바탕쪽 글자를 덮으면 잡는다 — 이 변경 전에는 0 건이었다.
#[test]
fn body_text_overlapping_master_page_is_detected() {
    let pa = scan_handbook_page();

    let cross: Vec<_> = pa
        .text_overlap
        .iter()
        .filter(|t| t.path_a.contains("/MasterPage") != t.path_b.contains("/MasterPage"))
        .collect();

    assert!(
        !cross.is_empty(),
        "본문 x 바탕쪽 글자 겹침이 잡혀야 한다. 전체 text_overlap={:?}",
        pa.text_overlap
    );
    // 실측(devel b1485e0a14 + 이 변경): 3 건, 겹침 폭 7~8px.
    // 0 으로 되돌아가는 회귀만 막고 상한은 두지 않는다 — 조판이 바뀌면 건수는
    // 정당하게 움직일 수 있다.
    assert!(
        cross.len() >= 3,
        "본문 x 바탕쪽 겹침이 3 건 미만이다({}). 후보 수집 범위가 좁아졌는지 확인할 것: {cross:?}",
        cross.len()
    );
    for t in &cross {
        assert!(
            t.overlap_w > 0.0 && t.overlap_h > 0.0,
            "겹침 폭·높이는 양수여야 한다: {t:?}"
        );
    }
}

/// 같은 쪽의 컨테이너 판정은 이 변경의 영향을 받지 않는다.
///
/// 넓힌 것은 **글자 겹침 후보 수집 하나**다. overflow 는 본문 여백, off-canvas 는
/// 페이지 상자라는 기준이 그대로여야 하고, 컨테이너 overlap 은 종전 단(column)
/// 짝짓기 규칙을 유지해야 한다. 편람 바탕쪽의 전면 배경 이미지가 흐름 후보로
/// 새어 들어가면 `overlap` 이 즉시 0 을 넘는다.
#[test]
fn container_verdicts_are_unchanged_on_the_same_page() {
    let pa = scan_handbook_page();
    // devel 실측: overflow 2(Table 2 건), overlap 0, off_canvas 0.
    assert_eq!(
        pa.overlap.len(),
        0,
        "컨테이너 겹침이 새로 생겼다 — 본문 밖 노드가 flow 후보로 새어 들어갔는지 확인: {:?}",
        pa.overlap
    );
    assert_eq!(
        pa.off_canvas.len(),
        0,
        "off-canvas 가 새로 생겼다: {:?}",
        pa.off_canvas
    );
    assert_eq!(
        pa.overflow.len(),
        2,
        "overflow 건수가 devel 실측(2)과 다르다: {:?}",
        pa.overflow
    );
}
