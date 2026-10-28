//! 관계형: 가시 기호의 실제 공급 범위와 정상적인 빈 glyph의 구분.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! 이 검사는 합성 입력을 사용하며 아래 수치는 관련 편람의 별도 시각 검증이다.
//! HWP 200쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 200쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.

#[path = "../common/symbol_font_program.rs"]
mod symbol_font_program;
use rhwp::renderer::font_symbol_coverage::{empty_symbol_codepoints, font_symbol_unicode_range};

#[test]
fn empty_symbols_do_not_claim_coverage_but_blank_semantics_and_real_ink_remain() {
    let original = include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf");
    let bytes = symbol_font_program::mapped_program(
        original,
        &[
            (' ', 1),
            ('A', 2),
            ('B', 1),
            ('⚫', 1),
            ('∑', 1),
            ('€', 1),
            ('˘', 1),
            ('\u{2800}', 1),
            ('\u{301}', 1),
            ('\u{200d}', 1),
            ('★', 2),
        ],
    );
    let before = bytes.clone();
    let empty = empty_symbol_codepoints(&bytes, 0);
    assert_eq!(empty, ['˘', '€', '∑', '⚫'].map(u32::from).into());
    let range = font_symbol_unicode_range(&bytes, 0);
    assert!(!range.is_empty());
    for ch in [' ', 'A', 'B', '★', '\u{2800}', '\u{301}', '\u{200d}'] {
        assert!(
            !empty.contains(&u32::from(ch)),
            "정상 의미/잉크 보존: {ch:?}"
        );
    }
    assert_eq!(bytes, before, "원본 cmap/glyph/hmtx/name은 바뀌지 않는다");
    assert!(font_symbol_unicode_range(original, 0).is_empty());
    assert!(
        font_symbol_unicode_range(&bytes, 1).is_empty(),
        "없는 collection face를 추측하지 않는다"
    );
    assert!(font_symbol_unicode_range(b"not a font", 0).is_empty());
}

#[test]
fn bitmap_and_svg_symbols_with_empty_glyf_still_supply_their_ink() {
    let bytes = symbol_font_program::mapped_program(
        include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf"),
        &[('⚫', 1), ('★', 2)],
    );
    let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
    assert!(face
        .glyph_raster_image(face.glyph_index('⚫').unwrap(), u16::MAX)
        .is_some());
    assert!(face
        .glyph_svg_image(face.glyph_index('★').unwrap())
        .is_some());
    assert!(empty_symbol_codepoints(&bytes, 0).is_empty());
    assert!(font_symbol_unicode_range(&bytes, 0).is_empty());
}

#[test]
fn entirely_blank_ebdt_does_not_mask_fallback_but_any_ink_strike_is_preserved() {
    let base = symbol_font_program::mapped_program(
        include_bytes!("../fixtures/fonts/RHWPExactKerningSmoke.ttf"),
        &[('⚫', 1)],
    );
    for ink in [[false, false, false], [false, true, false]] {
        let bytes = symbol_font_program::with_bitmap_strikes(&base, &ink);
        let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
        let glyph = face.glyph_index('⚫').unwrap();
        for (i, has_ink) in ink.iter().enumerate() {
            let bitmap = face.glyph_raster_image(glyph, 12 + i as u16 * 2).unwrap();
            assert_eq!(bitmap.data.iter().any(|byte| *byte != 0), *has_ink);
        }
        // 최대 크기만 보면 두 자료는 모두 비어 있다. 중간 크기를 놓치지 않는다.
        assert!(face
            .glyph_raster_image(glyph, u16::MAX)
            .unwrap()
            .data
            .iter()
            .all(|byte| *byte == 0));
        assert_eq!(
            empty_symbol_codepoints(&bytes, 0).contains(&u32::from('⚫')),
            !ink.iter().any(|value| *value)
        );
    }
}
