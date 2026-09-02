//! [#5884] 압축-단조 사다리 셀의 중첩 표 높이 미계상으로 쪽 넘김 판정이 뒤집힌다.
//!
//! `nested_row_first_line_height.hwpx`(3090867 반제품아이스팩 부과 기준 별표 3)는
//! 본문 전체가 1×1 `CELL` 분할 바깥 표 안에 있고, 그 칸의 저장 사다리는 단조지만
//! 중첩 표 높이를 흡수하지 않는다(last_seg_end 896.0px vs 텍스트+중첩 실측 1,108.4px).
//! 측정기의 intact(단조) 검사만으로는 max-합성 경로로 가서 칸이 과소 측정되고,
//! 칸 맨 아래 `예1)/예2)` 2×2 중첩 표의 둘째 행(계산식 2줄 + 수직선 그림)이 1쪽에
//! 우겨넣어져 clip 소실됐다 — 쪽수 1 vs 한글 2022 의 2, 96자·그림 2개 소실.
//!
//! 수정: additive 증거(텍스트+중첩 합이 저장 끝의 1.15배 초과)가 있는 비-native
//! 셀을 additive 측정으로 보낸다. 잔여: 예2) 수식 결과 꼬리 `0.273(kg)` 절단은
//! 수식 분수 확장 축(별개)이다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5884/nested_row_first_line_height.hwpx";

fn load_doc() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&fs::read(&path).expect("read sample")).expect("open")
}

#[test]
fn issue_5884_page_count_matches_hangul_2() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        2,
        "#5884: 한글 2022 는 이 문서를 2쪽으로 조판한다 — 결함 시 1쪽(셀 과소 측정)"
    );
}

#[test]
fn issue_5884_example_row_renders_on_page2() {
    let doc = load_doc();
    let page2 = doc.extract_page_text_native(1).expect("2쪽 텍스트");
    let flat = page2.replace(' ', "");
    for needle in ["직선보간법적용", "0.066", "예1)", "예2)"] {
        assert!(
            flat.contains(needle),
            "#5884: 2쪽에 {needle:?} 가 있어야 한다 (결함 시 1쪽 clip 소실)\n--- 2쪽 ---\n{page2}"
        );
    }
}

#[test]
fn issue_5884_no_overflow_cell_lines() {
    let doc = load_doc();
    let _ = doc.take_overflow_cell_lines();
    let mut total = 0u64;
    for page in 0..doc.page_count() {
        doc.render_page_svg_native(page)
            .unwrap_or_else(|e| panic!("render page {}: {e:?}", page + 1));
        total += u64::from(doc.take_overflow_cell_lines());
    }
    assert_eq!(
        total, 0,
        "#5884: 셀 측정이 내용을 담으면 쪽 밖 소실 줄이 없어야 한다"
    );
}
