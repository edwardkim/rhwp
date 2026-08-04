//! WMF(Windows Metafile) 최상위 변환 진입점 퍼징 하네스.
//!
//! 반환값은 무시한다 — 패닉/abort/자원 고갈/타임아웃만 검출 대상이다.
//! `convert_wmf_to_svg`(renderer/svg.rs)가 `pub(crate)` 라 fuzz 크레이트에서
//! 접근할 수 없으므로, 그 내부 조합을 그대로 편다.

#![no_main]

use libfuzzer_sys::fuzz_target;
use rhwp::wmf::converter::{SVGPlayer, WMFConverter};

fuzz_target!(|data: &[u8]| {
    let _ = WMFConverter::new(data, SVGPlayer::new()).run();
});
