//! 관계형 회귀. 같은 HWP5 본문의 파일 버전과 HWPX 재열기 전후의 실제 문자·괘선 소속을 검사한다.
//! 주석 갱신 2026-11-22 21:04 KST. 직전 변경 eb83e873566cf6dd079c4d8be430b1fa386e6af1 (2026-11-22 19:51:19 KST).
//! 2026-11-22 Windows 한컴 2020 독립 Print와 96 dpi 2px 실루엣으로 각각 전 1쪽을 비교했다.
//! 5.1.0.1 Native/fresh WASM 100.00%/100.00%, 5.0.1.5 99.95%/100.00%.
//! 기호 84개는 같은 본문에 저장되어 있다. 현대 입력에서는 Unicode이고 구형 입력에서는 이중 괘선이다.
//! 근거는 `mydocs/pr/assets/pr9280/`와 `pdf/issue5140-version-boundary/`에 보존한다.

use rhwp::renderer::composer::{expand_pua_render_text, pua_to_display_text, pua_to_text_surface};
use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

#[test]
fn legacy_display_helpers_accept_the_original_raw_symbol_key() {
    // 검증된 한컴 표시표(`renderer::hancom_pua`) 쪽 값.
    assert_eq!(
        expand_pua_render_text("\u{A832}"),
        "═",
        "0xA832 는 U+F0832 와 같은 글자 — 이중 가로 괘선으로 그려야 함",
    );
    // [#8201] U+F012B 는 렌더러 합성 기호라 표시 문자열 대신 정규화한 원 키를 유지한다.
    assert_eq!(
        expand_pua_render_text("도장란 \u{A12B}"),
        "도장란 \u{F012B}",
        "0xA12B 는 U+F012B 와 같은 글자 — 평면 15 키로 정규화돼 서명·날인 합성 기호가 됨",
    );
    assert_eq!(
        expand_pua_render_text("\u{A289}\u{A28A}"),
        "①②",
        "0xA289/0xA28A 는 U+F0289/U+F028A 와 같은 글자 — 원문자",
    );
    // 글머리표 표시표(`renderer::layout::map_pua_bullet_char`) 쪽 값도 같이 탄다.
    assert_eq!(
        expand_pua_render_text("\u{A80F}"),
        "─",
        "0xA80F 는 U+F080F 와 같은 원본 가는 괘선 — C144 독립 Print 전쪽 수용 근거",
    );

    // 렌더러 단건 조회와 텍스트 표면도 같은 답을 낸다.
    assert_eq!(pua_to_display_text('\u{A832}').as_deref(), Some("═"));
    assert_eq!(pua_to_text_surface("\u{A832}"), "═");
}

#[test]
fn normalization_only_applies_to_measured_symbols_with_a_table_value() {
    // (1) 한글 실측 값 집합 밖 — `08103` 의 0xA813 은 한글도 평면 15 로 옮기지 않는다.
    assert_eq!(
        expand_pua_render_text("\u{A813}"),
        "\u{A813}",
        "실측 값 집합 밖의 U+A813 을 기호로 오인하면 안 됨",
    );
    // (2) 값 집합 안이지만 평면-15 대응값이 표에 없는 경우 — 원문 유지(#5599 축).
    assert_eq!(
        expand_pua_render_text("\u{A80A}"),
        "\u{A80A}",
        "표에 값이 없으면 의미를 지어내지 말고 원문을 유지해야 함",
    );
    // (3) HWPX 경로(평면 15 원문)는 종전과 동일하다.
    assert_eq!(expand_pua_render_text("\u{F0832}"), "═");
}

#[test]
fn identical_hwp5_units_follow_the_file_version_through_save_and_display() {
    let cases = [
        ("samples/issue5800-hancom-symbol.hwp", '꠲', '═'),
        (
            "samples/issue5140-version-boundary/issue5800-v5.0.1.5.hwp",
            '═',
            '꠲',
        ),
    ];
    for (name, expected, other) in cases {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(name);
        let source = fs::read(&path).expect(name);
        let parsed = rhwp::parser::parse_hwp(&source).expect("parse HWP5");
        let stored = parsed.sections[0]
            .paragraphs
            .iter()
            .map(|paragraph| paragraph.text.matches('꠲').count())
            .sum::<usize>();
        assert_eq!(stored, 84, "{name}: original HWP5 units");

        let opened = HwpDocument::from_bytes(&source).unwrap();
        let hwpx = opened.export_hwpx_native().unwrap();
        for doc in [opened, HwpDocument::from_bytes(&hwpx).unwrap()] {
            let text = doc.extract_page_text_native(0).unwrap();
            assert_eq!(text.matches(expected).count(), stored, "{name}: text");
            assert!(!text.contains(other), "{name}: {other}");
            let svg = doc.render_page_svg_native(0).unwrap();
            assert!(
                svg.contains(expected) && !svg.contains(other),
                "{name}: SVG"
            );
            for yi in ['ꄫ', 'ꊉ', 'ꊊ'] {
                assert_eq!(svg.contains(yi), expected == '꠲', "{name}: {yi}");
            }
        }
    }
}
