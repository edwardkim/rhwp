//! HML (XML) 최상위 진입점 퍼징 하네스.
//!
//! 반환값은 무시한다 — 패닉/abort/자원 고갈/타임아웃만 검출 대상이다.
//! 기본 `HmlLimits` 상한이 적용된 경로를 그대로 사용한다 — 상한 기구의
//! 빈틈(#2743류)을 찾는 것이 목적이므로 상한을 풀지 않는다.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = rhwp::parser::hml::parse_hml(data);
});
