//! OOXML 차트(`c:chartSpace`) XML 파서 진입점 퍼징 하네스.
//!
//! 반환값은 무시한다 — 패닉/abort/자원 고갈/타임아웃만 검출 대상이다.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = rhwp::ooxml_chart::parser::parse_chart_xml(data);
});
