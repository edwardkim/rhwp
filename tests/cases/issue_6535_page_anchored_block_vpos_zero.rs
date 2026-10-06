//! [Issue #6535 잔여] **쪽 기준 앵커**(`vert_rel_to == Page`) 자리차지 블록의 저장
//! `vertical_pos = 0` 을 "한컴이 여기서 쪽을 끊었다"는 리셋 신호로 읽어, 남은 자리에
//! 들어가는 표를 혼자 새 쪽으로 밀어내던 결함의 가드.
//!
//! 그런 블록의 저장 `vpos` 는 흐름 좌표가 아니라 **절대배치의 산물**이라 대개 `0` 이다
//! (`#6535` 가 세운 계약). 그런데 단일 단 리셋 판정
//! (`typeset.rs` 의 `cv == 0 && pv > 5000`)은 그 `0` 을 그대로 경계로 읽었다.
//!
//! 재현 문서 `samples/issue6535/36399617_page_anchored_block_reset.hwpx`
//! (한글 2024 실측 **1쪽** — COM 이 이 버전으로만 해석된다):
//!
//! ```text
//! pi=15 까지  cur_h = 639.0px / 본문 990.2px      → 잔여 351.2px
//! pi=16       text="끝."
//!             Table wrap=TopAndBottom vrel=Page tac=false h=22234HU(296.5px)
//!             line_segs = [(vpos 0, lh 1200)]
//!
//! 수정 전  cv==0 · pv=46278>5000 → 리셋으로 읽혀 flush_column
//!          2쪽이 생기고 그 쪽의 used = 0.0px
//! 수정 후  표 296.5px 이 잔여 351.2px 에 54.7px 여유로 들어가 1쪽
//! ```
//!
//! ⚠ `#6560` 이 고친 `uncertain_anchor_margin`(`flow_underrun`) 갈래는 이 문서에서
//! **발동조차 하지 않는다** — `DIAG_SCAN FOOTER` 가 `cur_h=0.00`(이미 2쪽)으로 한 번만
//! 찍힌다. 같은 코호트의 **다른 결정 지점**이다.
//!
//! 이 수정으로 `#6535` 가 등록한 7건이 전부 한/글과 같은 1쪽이 된다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue6535/36399617_page_anchored_block_reset.hwpx";
/// 한/글 2024 실측. 회귀하면 표가 혼자 2쪽으로 밀려 본문 없는 쪽이 생긴다.
const EXPECTED_PAGES: u32 = 1;

#[test]
fn page_anchored_block_vpos_zero_is_not_a_page_reset() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));

    let pages = document.page_count();
    assert_eq!(
        pages, EXPECTED_PAGES,
        "쪽-앵커 블록(vrel=Page)의 저장 vpos=0 을 쪽 리셋으로 읽으면 안 된다 — 실측 {pages}쪽. \
         회귀 시 296.5px 표가 잔여 351.2px 을 두고 새 쪽으로 밀려 used=0.0px 인 쪽이 생긴다"
    );
}
