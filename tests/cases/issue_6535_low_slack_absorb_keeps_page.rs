//! #6535: 하단 쪽 앵커의 본문 끝은 활성 단의 저장 좌표 기준으로 복원한다.
//!
//! 원본과 독립 한컴 PDF는 같은 문서의 1쪽이다. 이전 검사 이관 시 본문은
//! 첫 쪽에 맞았지만 하단 틀만 둘째 쪽으로 밀렸다. 활성 lazy 기준을 누락하여
//! 저장 본문 끝을 원시 구역 좌표로 사용한 것이 원인이었다.
//! Native/fresh WASM 전1쪽99.54444% 확인 후 기존 쪽 소속 검사를 복원한다.
//! 내용의 누락·중복과 소속 하단 표는 maintainer_hwpx_overlay_table_extent의
//! 같은 원본 검사에서 확인하며 실측 픽셀 위치는 기대값으로 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6535/36339092_low_slack_absorb_block.hwpx";
/// 한/글과 같은 쪽수. 회귀 시에는 블록이 2쪽으로 밀려 본문 없는 쪽이 생긴다.
const EXPECTED_PAGES: u32 = 1;

#[test]
fn low_slack_anchor_block_is_absorbed_into_the_same_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue6535 low-slack sample");

    let pages = document.page_count();
    assert_eq!(
        pages, EXPECTED_PAGES,
        "독립 PDF와 달리 하단 쪽 앵커가 별도 쪽으로 밀렸다 — 실측 {pages}쪽. \
         저장 본문 끝과 현재 단의 흐름을 같은 좌표 기준으로 판정해야 한다"
    );
}
