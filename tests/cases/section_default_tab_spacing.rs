//! 기본 탭 자리를 구역 정의의 기본 탭 간격으로 정한다.
//!
//! 시험 문서는 구역 정의의 기본 탭 간격이 16000(한/글 대화상자 80pt)이고, 둘째 문단이
//! `가\t나\t다\t라\t마`다. 한컴 한글 12.30(Mac) 인쇄 PDF(`pdf/section_default_tab_spacing/`)에서
//! 글자 왼끝은 85.08, 165.12, 245.04, 325.08, 405.12pt로 80pt(106.67px)씩 떨어진다.
//! 같은 문서를 기본값 8000으로 저장하면 한/글은 40pt(53.33px)씩 둔다. 구역 값을 읽지 않으면
//! 두 문서 모두 40pt 간격이 된다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "tests/fixtures/section_default_tab_spacing/tab-spacing-16000.hwp";
const PARA: usize = 1;

fn caret_x(core: &DocumentCore, offset: usize) -> f64 {
    let json = core
        .get_cursor_rect_native(0, PARA, offset)
        .expect("cursor rect");
    let start = json.find("\"x\":").expect("x 필드") + "\"x\":".len();
    json[start..]
        .split([',', '}'])
        .next()
        .and_then(|value| value.trim().parse().ok())
        .unwrap_or_else(|| panic!("x 파싱 실패: {json}"))
}

/// 탭 뒤 글자(나·다·라·마)의 왼끝이 첫 글자에서 `step` px 배수만큼 떨어지는지 본다.
fn assert_tab_step(core: &DocumentCore, step: f64) {
    let origin = caret_x(core, 0);
    for k in 1..=4 {
        let x = caret_x(core, 2 * k) - origin;
        let expected = step * k as f64;
        assert!(
            (x - expected).abs() < 1.0,
            "{k}번째 탭 뒤 글자: {x:.2}px, 기대 {expected:.2}px"
        );
    }
}

#[test]
fn default_tab_stops_follow_section_tab_spacing() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    assert_tab_step(&core, 80.0 * 96.0 / 72.0);

    core.set_section_def_native(0, r#"{"defaultTabSpacing":8000}"#)
        .expect("set section def");
    assert_tab_step(&core, 40.0 * 96.0 / 72.0);
}
