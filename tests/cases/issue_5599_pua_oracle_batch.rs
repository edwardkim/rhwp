//! Issue #5599 후속: 한글 2022 오라클(COM PDF 내보내기)로 확정한 한컴 PUA 17항목의
//! 회귀 가드.
//!
//! 확정 방법 — 재현 문서를 한글 2022 로 PDF 내보내고(`tools/hwp_oracle_pdf.ps1`,
//! Open 은 포맷 자동판별 `""` 필수), PyMuPDF 로 문단 앵커 텍스트 ↔ PDF 줄을
//! 짝지어 해당 글리프 bbox 를 절단해 눈으로 판정했다. 한글 PDF 는 평면 15 PUA 를
//! BMP PUA(U+F000 계열, 표 셀 bullet 은 U+FFFF)로 재부호화하므로 텍스트 추출로는
//! 원 코드포인트를 못 찾는다 — 앵커 좌표 대응이 유일한 경로다.
//!
//! 이 테스트는 동봉 재현 문서(`samples/issue5599_oracle/3191107_leave_request_form.hwpx`,
//! 이슈 본문의 00447 = 육아기 근로시간 단축 신청서)를 렌더해 (1) 확정 항목
//! `U+F03FF → □`(표 셀 제목 bullet 4곳)와 `U+F02EC → ◇`(안내문 3곳)가 실제로
//! 그려지고 (2) 두 코드포인트의 raw PUA 가 남지 않는 것을 잠근다.

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/issue5599_oracle/3191107_leave_request_form.hwpx";

#[test]
fn issue_5599_oracle_confirmed_pua_is_substituted() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));
    let svg = doc
        .render_page_svg_native(0)
        .unwrap_or_else(|e| panic!("render page 0: {e}"));

    assert!(
        svg.contains('□'),
        "표 셀 제목 bullet U+F03FF 가 □ 로 그려져야 한다"
    );
    assert!(
        svg.contains('◇'),
        "안내문 bullet U+F02EC 가 ◇ 로 그려져야 한다"
    );
    for code_point in [0xF03FFu32, 0xF02EC] {
        let ch = char::from_u32(code_point).expect("valid code point");
        assert!(
            !svg.contains(ch),
            "매핑된 U+{code_point:05X} 가 raw PUA 로 남았다 — 대체표 투영이 끊겼는지 확인"
        );
    }
}

/// U+F02C5 는 연속 구간 가정(21)이 아니라 **네모 12** 다 — 한글 2022 오라클 실측
/// (`samples/mel-001.hwp` p18 국정과제 bullet). 렌더는 네모 숫자 원문 유지 계약
/// (캡스톤 F-1, issue_3385/3385b)이라 그대로 두고, 텍스트 표면만 ⑫ 로 바꾼다.
#[test]
fn issue_5599_boxed_twelve_text_surface_is_readable() {
    let surfaced = rhwp::renderer::composer::pua_to_text_surface("\u{F02C5} 외국인노동자");
    assert!(
        surfaced.contains('\u{246B}'),
        "U+F02C5 텍스트 표면이 ⑫ 로 바뀌어야 한다: {surfaced:?}"
    );
    assert!(
        !surfaced.contains('\u{F02C5}'),
        "U+F02C5 raw PUA 가 텍스트 표면에 남으면 안 된다"
    );
}
