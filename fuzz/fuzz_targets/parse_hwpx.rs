//! HWPX (ZIP 컨테이너) 최상위 진입점 퍼징 하네스.
//!
//! 반환값은 무시한다 — 패닉/abort/자원 고갈/타임아웃만 검출 대상이다.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = rhwp::parser::hwpx::parse_hwpx(data);
});
