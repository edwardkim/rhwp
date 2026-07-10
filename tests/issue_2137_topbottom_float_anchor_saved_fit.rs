//! Issue #2137: TopAndBottom float 전용 앵커의 saved-bounds 신뢰 — 소형 글상자 스필.
//!
//! Regression shape (samples/task2137/156618554_petfood_press.hwp, 공개 보도자료 실문서):
//! - 문서 마지막 pi13 = 빈 앵커 + 소형 부동 글상자(사각형, 자리차지 TopAndBottom,
//!   vert=문단, 19148×3736HU ≈ 49.8px). 앵커 저장 vpos=68600 → 줄 경계 70000 ≤
//!   본문 70018HU — 저장 흐름이 1쪽 하단을 인코딩(문서 끝 = page-last 증거).
//! - 수정 전: `saved_single_line_bottom_fits`(#2093)의 controls.is_empty() 조건에
//!   배제 + 개체 하단 넘침으로 앵커+개체가 2쪽 단독 배치 (한글 1쪽 — 개체를 하단
//!   여백 15mm 로 스필). 10k r12 OVER+SHAPE 계열.
//! - 수정 후: 비-TAC TopAndBottom float 만 가진 단일 줄 앵커를 신뢰 경로에 편입 —
//!   1쪽, 개체는 여백 스필 (visual sweep OK 1=1쪽 88.3% 확인).

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2137/156618554_petfood_press.hwp";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_2137_small_float_anchor_stays_on_page_1() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        1,
        "저장 page-last 증거가 있는 소형 float 앵커는 1쪽 하단 유지 + 개체 여백 스필 (#2137)"
    );

    let page1 = doc.dump_page_items(Some(0));
    assert!(
        page1.contains("pi=13"),
        "앵커 pi=13 은 1쪽 하단이어야 한다\n--- page 1 ---\n{}",
        page1
    );
}
