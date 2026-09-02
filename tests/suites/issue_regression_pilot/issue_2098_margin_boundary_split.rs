//! Issue #2098/#2138: 불확실 앵커(vpos≤0) footer fit 마진의 **적용 조건** — 경계 회귀.
//!
//! Regression shape (samples/task2098/page_bottom_fixed_anchor_margin_split.hwpx, 합성):
//! - 본문 흐름 끝(저장) 56600HU(754.7px), 쪽-하단 고정 표 133.3px →
//!   배타 잔여 800.2px, 슬랙 45.6px — **마진(50px) 이내의 경계 케이스**.
//! - 이 문서의 한글 2020 정본(pdf/task2098/page_bottom_fixed_anchor_margin_split-2020.pdf)
//!   은 **1쪽**이고 틀은 1쪽 하단에 있다. 종전 이 테스트는 코호트(결재문서 60건) 대역을
//!   대리한다는 전제로 2쪽을 잠갔으나, 같은 저장소가 들고 있는 이 문서 자신의 정본과
//!   tests/fixtures/render_page_samples.tsv(hangul_pages=1) 가 모두 1쪽을 가리킨다.
//! - 마진의 불확실성 원천은 "앵커 vpos≤0 이라 본문 끝을 저장 vpos 에서 **복원**했다"는
//!   점이다. 이 문서는 복원값과 흐름 cur_h 가 754.67px 로 **일치**해 복원이 판정을
//!   끌어올리지 않았다 — 남은 불확실성이 없으므로 마진이 걸리지 않아야 한다.
//! - 코호트 분할 정답군(36387725: cur_h 578 → 복원 640.7 로 상향)은 복원이 실제로
//!   상향하므로 마진이 그대로 유지된다 — 이 테스트는 그 대역을 대리하지 않는다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task2098/page_bottom_fixed_anchor_margin_split.hwpx";

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e))
}

#[test]
fn issue_2098_corroborated_anchor_footer_absorbs_into_page_1() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        1,
        "복원값이 흐름과 일치(sync_h == cur_h)하는 경계 footer 는 마진 없이 흡수(1쪽)여야 \
         한다 — 한글 2020 정본 1쪽 (#2098/#2138)"
    );
    assert!(
        doc.dump_page_items(Some(0)).contains("Table"),
        "고정 틀 표는 1쪽 하단에 있어야 한다"
    );
}
