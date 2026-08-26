//! 한컴 수식 스크립트 파서 진입점 퍼징 하네스.
//!
//! 렌더러 수식 파서와 EqEdit→LaTeX 변환기를 같은 UTF-8 입력으로 친다.
//! #4865 깊이 상한은 그대로 둔다 — 타깃만 추가한다.
//! 반환값은 무시한다 — 패닉/abort/자원 고갈/타임아웃만 검출 대상이다.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let Ok(script) = std::str::from_utf8(data) else {
        return;
    };
    let _ = rhwp::renderer::equation::parser::parse(script);
    let _ = rhwp::doclang::eqedit::convert(script);
});
