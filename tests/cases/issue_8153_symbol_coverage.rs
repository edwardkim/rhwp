//! 관계형: 가시 기호의 실제 공급 범위와 정상적인 빈 glyph의 구분.
//! 주석 갱신: 2026-11-06 04:45:01 KST; 직전 파일 커밋: c417310cab 2026-10-30T19:50:11+09:00.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! 이 검사는 합성 입력을 사용하며 아래 수치는 관련 편람의 별도 시각 검증이다.
//! HWP 200쪽 Native 100.00% / fresh WASM 100.00%.
//! HWP 전 383쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! HWPX 200쪽 Native 100.00% / fresh WASM 100.00%.
//! HWPX 전 383쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.

// 파생 suite 루트에서 한 번 로딩한 동일 helper를 공유한다. 검사 본문은 유지한다.
use super::symbol_font_program;
use rhwp::renderer::font_symbol_coverage::{empty_symbol_codepoints, font_symbol_unicode_range};

#[test]
fn empty_symbols_do_not_claim_coverage_but_blank_semantics_and_real_ink_remain() {
    use rhwp::renderer::font_symbol_coverage::is_visible_symbol;
    let program =
        include_bytes!("../../vendor/rhwp-pdf-import-backend/src/fonts/assets/DejaVuSans.ttf");
    let face = ttf_parser::Face::parse(program, 0).unwrap();
    let empty_object = face
        .glyph_index('\u{fffc}')
        .expect("원본에 등록된 객체 대체 기호");
    assert!(face.glyph_bounding_box(empty_object).is_none());
    let empty = empty_symbol_codepoints(program, 0);
    assert_eq!(empty, [u32::from('\u{fffc}')].into());
    assert_eq!(
        font_symbol_unicode_range(program, 0),
        "U+0-FFFB,U+FFFD-10FFFF",
        "실제 빈 기호만 다음 글꼴로 넘긴다"
    );

    for character in ['⚫', '∑', '€', '˘', '★'] {
        assert!(is_visible_symbol(character));
        let glyph = face.glyph_index(character).unwrap();
        assert!(face.glyph_bounding_box(glyph).is_some());
        assert!(!empty.contains(&u32::from(character)));
    }
    for character in [' ', 'A', 'B', '\u{2800}', '\u{0301}', '\u{200d}'] {
        assert!(!is_visible_symbol(character));
        assert!(face.glyph_index(character).is_some());
        assert!(!empty.contains(&u32::from(character)));
    }
    for character in [' ', '\u{2800}', '\u{200d}'] {
        let glyph = face.glyph_index(character).unwrap();
        assert!(face.glyph_bounding_box(glyph).is_none());
    }
    assert!(font_symbol_unicode_range(program, 1).is_empty());
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
