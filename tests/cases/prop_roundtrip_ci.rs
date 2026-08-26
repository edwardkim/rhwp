//! [#5388] M04-4: 왕복 property CI 배선.
//!
//! `prop_hwpx_roundtrip`(M04-2) · `prop_hwp5_roundtrip`(M04-3) 가 아직 없어도
//! 이 파일은 항상 있다. `.github/workflows/proptest-roundtrip.yml` 이 같은
//! 러너로 셋을 돌리고, 없는 원본은 skip 한다.
//!
//! 문서 IrDiff-0 왕복은 M04-2/3 본체다. 여기는 proptest 가 CI 에서 실제로
//! 돈다는 최소 불변식만 본다.
#![cfg(not(target_arch = "wasm32"))]

use proptest::prelude::*;

/// CI 기본. `PROPTEST_CASES` 가 있으면 proptest 가 덮어쓴다.
const CI_CASES: u32 = 8;

proptest! {
    #![proptest_config(ProptestConfig {
        cases: CI_CASES,
        max_shrink_iters: 16,
        ..ProptestConfig::default()
    })]
    #[test]
    fn prop_roundtrip_ci_utf8_identity(s in "\\PC{0,32}") {
        let bytes = s.as_bytes().to_vec();
        let back = String::from_utf8(bytes).expect("UTF-8");
        prop_assert_eq!(s, back);
    }
}
