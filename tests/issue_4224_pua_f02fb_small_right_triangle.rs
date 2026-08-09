//! #4224 `samples/basic/pau-004.hwp`의 한컴 Supplementary PUA-A 표시 계약.
//!
//! 원문 `U+F02FB`는 IR에 보존하되, 한컴 문자표와 같은 작은 오른쪽 방향
//! 삼각형 `U+25B8`로 paint-time 투영해 공개 글꼴 환경의 tofu를 막는다.

use rhwp::renderer::composer::{expand_pua_render_text, pua_to_display_text, pua_to_text_surface};
use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

fn read_pau_004() -> Vec<u8> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/basic/pau-004.hwp");
    fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

#[test]
fn pau_004_preserves_f02fb_and_projects_small_right_triangle() {
    let bytes = read_pau_004();
    let document = rhwp::parser::parse_hwp(&bytes).expect("parse pau-004.hwp");
    let raw = &document.sections[0].paragraphs[0].text;

    assert_eq!(raw, "\u{F02FB}아름다운", "IR은 한컴 PUA 원문을 보존");
    assert_eq!(
        expand_pua_render_text(raw),
        "▸아름다운",
        "일반 TextRun paint 표면은 작은 오른쪽 방향 삼각형을 사용",
    );
    assert_eq!(
        pua_to_display_text('\u{F02FB}').as_deref(),
        Some("▸"),
        "공통 한컴 PUA 표시표에서도 같은 의미를 반환",
    );
    assert_eq!(
        pua_to_text_surface(raw),
        "▸아름다운",
        "폰트가 없는 텍스트 소비자에게도 raw PUA를 노출하지 않음",
    );
}

#[test]
fn pau_004_svg_never_emits_raw_f02fb() {
    let document = HwpDocument::from_bytes(&read_pau_004()).expect("load pau-004.hwp");
    let svg = document
        .render_page_svg_native(0)
        .expect("render pau-004.hwp page 1");

    assert!(
        svg.contains(">▸</text>"),
        "SVG는 한컴의 작은 오른쪽 방향 삼각형을 출력해야 함",
    );
    for character in ["아", "름", "다", "운"] {
        assert!(
            svg.contains(&format!(">{character}</text>")),
            "삼각형 뒤 본문 글자 `{character}`를 보존해야 함",
        );
    }
    assert!(
        !svg.contains('\u{F02FB}'),
        "SVG에 raw U+F02FB가 남아 공개 글꼴에서 tofu가 되면 안 됨",
    );
}
