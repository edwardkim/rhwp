//! Issue #6549: 어울림(Square) `RowBreak` 표가 쪽을 넘겨도 안 쪼개져 본문 하한을 넘는다.
//!
//! 근인은 **저장 프레임 꼬리 확장이 행 예산을 덮어쓰는 것**이다(#5584 ②/#4763 계약).
//!
//! ```text
//! let mut budget = (avail_for_rows - consumed - cs_before - padding).max(0.0);  // 75.8
//! ...
//! if extension > 0.5 && mid_extension_ok {
//!     budget = source_tail_cut.consumed_height;   // 92.8  (확장 25.6)
//! }
//! ```
//!
//! 확장분이 곧 넘침분이다 — `used 1026.2 − avail 1009.1 = 17.1px`. 예산이 커지자 행 6 이
//! `fully=true` 로 통째 수용되고 행 7 까지 들어가, 표가 한 쪽에 뭉친다(한글은 2쪽).
//!
//! 상한 가드는 있었지만 `mid_frame_only` 갈래에만 걸려 있었다.
//!
//! ```text
//! let mid_extension_ok = !mid_frame_only || (extension <= 24.0 && ...);
//! ```
//!
//! 이 표는 `mid_frame_only == false` 라 앞항이 참이 되어 **뒤 조건 전체가 단락**된다.
//! 조건을 재 보면 셋 중 둘을 위반한다 — `extension 25.6 > 24.0`, `frame_tail_rest 0.0`.
//!
//! **배치 종류가 유일한 판별자다.** 이 계약을 지탱하는 382쪽 편람 핀
//! (`issue_3931`/`3930`/`5801`)의 확장은 전부 `wrap=TopAndBottom` 이고, 확장 크기는
//! 15.3~107.4px 로 이 표의 25.6px 을 사이에 두고 흩어져 있다. `extension`·소비 비율·
//! `frame_tail_rest`·예산 초과율 어느 축으로도 갈리지 않는다.
//!
//! ```text
//! 대상        mid=false ext= 25.6 rest=  0.0 budget= 75.8 over=17.0 wrap=Square
//! 편람 핀     mid=false ext= 15.3 rest=  0.0 budget= 33.5 over= 6.4 wrap=TopAndBottom
//!            mid=false ext= 39.8 rest=  0.0 budget=104.9 over=17.8 wrap=TopAndBottom
//!            mid=false ext=107.4 rest= 91.6 budget=334.5 over=52.4 wrap=TopAndBottom
//!            (…13건 전부 TopAndBottom)
//! ```
//!
//! 어울림 표는 옆으로 글이 흐르므로 프레임 회계가 달라, 상한 없는 확장이 그대로 쪽
//! 넘침이 된다. 그래서 어울림 갈래에도 같은 near-miss 상한을 적용한다.
//!
//! 재현: `rhwp dump-pages <문서>` — 수정 전 1쪽, 수정 후 2쪽(한글 2024 도 2쪽).
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue6549/16418295_square_rowbreak_table.hwp";
/// 한글 2024 와 같은 쪽수. 회귀 시에는 1쪽으로 뭉친다.
const EXPECTED_PAGES: u32 = 2;

#[test]
fn square_rowbreak_table_splits_instead_of_overfilling_the_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue6549 sample");

    let pages = document.page_count();
    assert_eq!(
        pages, EXPECTED_PAGES,
        "어울림 RowBreak 표는 쪽 경계에서 쪼개져야 한다 — 실측 {pages}쪽. \
         회귀 시 1쪽에 뭉치고 본문 하한을 17.1px 넘는다"
    );
}
