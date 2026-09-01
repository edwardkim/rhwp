//! [Issue #6535 잔여] 앵커 `vpos <= 0` 인 자리차지 블록의 **저슬랙 흡수** 코호트.
//!
//! `#2098` 이 넣은 `uncertain_anchor_margin` 은 "본문 끝을 저장 vpos 에서 **복원**했다"는
//! 불확실성을 슬랙 스칼라로 보정한다. 그런데 그 스칼라로는 두 코호트가 갈리지 않는다는
//! 것이 `#2098`·`#2138`·`#2279` 를 거치며 남은 기지 한계였다 — 주석에도
//! "슬랙 스칼라로는 완전 분리 불가(진짜 판별 신호는 후속 조사)"라고 적혀 있었다.
//!
//! 경합 구간을 전수 재 보면 갈리는 것은 슬랙이 아니라 **`flow_underrun`** 이다.
//! 이 단의 문단 place 가 트림한 `(total_height − advance)` 누계로, 0 보다 크면 `cur_h` 가
//! 실제 내용 하단을 **과소**하게 들고 있다는 뜻이다 — 마진이 보정하려던 불확실성이
//! 바로 그것이다.
//!
//! ```text
//!                             slack   underrun   한/글   종전 rhwp
//! 흡수 정답  36358528         25.01     0.00      1쪽     2쪽 ✗   ← #2098 이후 기지 한계
//!            36477251         25.75     0.00      1쪽     2쪽 ✗   ← #2098 이후 기지 한계
//!            36339092         31.60     0.00      1쪽     2쪽 ✗
//!            36357897         35.69     0.00      1쪽     2쪽 ✗
//!            36348992         37.77     0.00      1쪽     2쪽 ✗
//!            36376848         56.40     0.00      1쪽     1쪽 ✔
//! 분할 정답  36395825         42.48   **37.60**   2쪽     2쪽 ✔
//!            36387725         -2.00     0.00      2쪽     2쪽 ✔
//!            36394733        -18.71     0.00      2쪽     2쪽 ✔
//! ```
//!
//! 슬랙은 25.0~42.5 로 완전히 겹치는데 `underrun` 은 **0 vs 37.60** 으로 갈린다. 마진을
//! `flow_underrun > 0.5` 일 때만 걸면 아홉 건 전부 한/글과 맞는다 — `#2098` 이 남긴
//! 저슬랙 흡수 2건까지 함께 닫힌다.
//!
//! 이 시험은 그중 `36339092`(slack 31.60)를 고정한다. 한글 2024 실측 1쪽.
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
        "저슬랙(31.60) · underrun 0 인 앵커 블록은 같은 쪽에 흡수돼야 한다 — 실측 {pages}쪽. \
         회귀 시 마진 50px 이 흐름 좌표 기준 fit 을 깎아 블록을 다음 쪽으로 민다"
    );
}
