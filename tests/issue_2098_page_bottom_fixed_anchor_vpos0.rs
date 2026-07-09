//! Issue #2098: 쪽-하단 고정 틀 앵커의 저장 vpos=0 을 새 쪽 신호로 오독 — 여분 페이지.
//!
//! Regression shape (samples/task2098/page_bottom_fixed_anchor_vpos0.hwpx, 합성):
//! - 본문 2문단(마지막 저장 vpos=6000 > 리셋 임계 5000) 뒤 **빈 앵커 문단(vpos=0)** 이
//!   쪽-하단 고정 표(vert=쪽, valign=Bottom, wrap=자리차지, 133px)를 안는다.
//! - 수정 전: vpos-reset 가드(cv==0 && pv>5000)가 앵커의 쪽 기준 절대배치 vpos=0 을
//!   흐름 리셋으로 오독 → 표 진입 전에 새 쪽을 열어 틀이 2쪽 단독 배치.
//!   (실문서: opengov 결재문서 36358528/36376848 발신명의 틀 — p1 잔여 391px ≥
//!   틀 351px 인데 2쪽, 백로그 OVER+ORPHAN_PAGE 15/106건 계열.)
//! - 수정 후: 앵커는 리셋 신호에서 제외, page-bottom footer 경로가 배타영역 fit 으로
//!   p1 하단 배치. 전체 1쪽.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2098/page_bottom_fixed_anchor_vpos0.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_2098_page_bottom_fixed_anchor_stays_on_page_1() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        1,
        "쪽-하단 고정 틀 앵커(vpos=0)는 리셋 신호가 아니다 — 전체 1쪽 (#2098 회귀)"
    );

    let page1 = doc.dump_page_items(Some(0));
    assert!(
        page1.contains("Table"),
        "고정 틀 표는 1쪽 하단에 배치되어야 한다\n--- page 1 ---\n{}",
        page1
    );
}
