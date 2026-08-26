//! 문서 SVG 내보내기 진입점 퍼징 하네스.
//!
//! `export-svg` CLI 와 같은 로드→1페이지 렌더 경로다.
//! 파싱 실패는 정상 동작이며, 퍼저가 잡는 것은 패닉/abort/자원 고갈/타임아웃뿐이다.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(core) = rhwp::DocumentCore::from_bytes(data) else {
        return;
    };
    let _ = core.render_page_svg_native(0);
});
