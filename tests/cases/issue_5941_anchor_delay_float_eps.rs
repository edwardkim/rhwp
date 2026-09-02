//! [Issue #5941] 저장 앵커 지연 판정이 **부동소수점 1 ULP** 로 뒤집혀, 같은 쪽에 들어가는
//! 자리차지 표가 제 쪽으로 밀려 쪽수가 하나 늘었다.
//!
//! `saved_object_bottom_fits_current` 는 `anchor_delay <= measured_declared_excess` 를 요구한다.
//! 앵커 지연도 measured excess 도 **정확히 0 인 정상 형상**에서 두 값이 각각 HWPUNIT→px
//! 나눗셈으로 만들어지므로 비트 일치가 보장되지 않는다.
//!
//! `1130000-200900012` 실측 (`pi=1`, 9×5 자리차지 표):
//!
//! ```text
//! anchor = 42.93333333333333      cur_h  = 42.93333333333334   ← 1 ULP
//! delay  = 7.105427357601002e-15  excess = 0.0                 → 판정 false
//!
//! 저장 하단 881.386667px  ≤  본문 895.733333px                 ← 들어간다
//! 그런데 표가 제 쪽으로 밀려 2쪽 문서가 3쪽이 됐다(한/글 2쪽, r37 2쪽).
//! ```
//!
//! 이 문서는 `git bisect` 로 `f8c784235`(#4763) 를 재확인할 때 쓴 판정자이고, 후속 수정
//! (#6033·#6038·#6058)으로도 남아 있던 잔여다.
#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/issue5941/1130000-200900012_anchor_delay_ulp.hwp";

#[test]
fn issue_5941_zero_anchor_delay_is_not_flipped_by_one_ulp() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("parse issue5941 sample");

    let pages = document.page_count();
    assert_eq!(
        pages, 2,
        "제목(42.93px) + 9×5 자리차지 표(저장 하단 881.39px)는 본문 895.73px 안에 들어간다 — \
         실측 {pages}쪽 (한/글 2쪽, 회귀 시 3쪽: 앵커 지연이 7.1e-15px 로 판정을 뒤집었다)"
    );
}
