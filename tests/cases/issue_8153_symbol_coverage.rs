//! 관계형: 가시 기호의 실제 공급 범위와 정상적인 빈 glyph의 구분.
//! 주석 갱신: 2026-11-06 04:45:01 KST; 직전 파일 커밋: c417310cab 2026-10-30T19:50:11+09:00.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! 아래 수치는 관련 편람의 별도 시각 검증이다.
//! HWP 200쪽 Native 100.00% / fresh WASM 100.00%.
//! HWP 전 383쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! HWPX 200쪽 Native 100.00% / fresh WASM 100.00%.
//! HWPX 전 383쪽 최저 Native 99.96% / fresh WASM 99.96%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.

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

use super::font_symbol_bitmap as bitmap;

use ttf_parser::{cbdt, cblc, GlyphId};
use ttf_parser::{Face, RawFaceTables, Tag};

fn strike_tables(ink: [bool; 3]) -> (Vec<u8>, Vec<u8>) {
    let mut ebdt = 0x00020000u32.to_be_bytes().to_vec();
    let mut eblc = 0x00020000u32.to_be_bytes().to_vec();
    eblc.extend((ink.len() as u32).to_be_bytes());
    let array_start = 8 + ink.len() * 48;
    let mut arrays = Vec::new();
    for (index, has_ink) in ink.iter().enumerate() {
        let mut record = vec![0; 48];
        record[..4].copy_from_slice(&((array_start + arrays.len()) as u32).to_be_bytes());
        record[4..8].copy_from_slice(&24u32.to_be_bytes());
        record[8..12].copy_from_slice(&1u32.to_be_bytes());
        record[40..42].copy_from_slice(&1u16.to_be_bytes());
        record[42..44].copy_from_slice(&1u16.to_be_bytes());
        record[44..48].copy_from_slice(&[12 + index as u8 * 2, 12 + index as u8 * 2, 1, 1]);
        eblc.extend(record);
        for value in [1u16, 1] {
            arrays.extend(value.to_be_bytes());
        }
        arrays.extend(8u32.to_be_bytes());
        for value in [1u16, 1] {
            arrays.extend(value.to_be_bytes());
        }
        arrays.extend((ebdt.len() as u32).to_be_bytes());
        arrays.extend(0u32.to_be_bytes());
        arrays.extend(29u32.to_be_bytes());
        ebdt.extend([12, 12, 0, 10, 12]);
        let mut pixels = [0; 24];
        if *has_ink {
            pixels[0] = 0x80;
        }
        ebdt.extend(pixels);
    }
    eblc.extend(arrays);
    (eblc, ebdt)
}

#[test]
fn intermediate_ink_strike_is_kept_when_largest_strike_is_blank() {
    let glyph = GlyphId(1);
    for expected in [[false, false, false], [false, true, false]] {
        // 위치 레코드와 비트맵 표만 조립한다. SFNT 컨테이너나 글꼴은 생성하지 않는다.
        let (locations, pixels) = strike_tables(expected);
        let table = cbdt::Table::parse(cblc::Table::parse(&locations).unwrap(), &pixels).unwrap();
        for (size, expect_ink) in [12, 14, 16].into_iter().zip(expected) {
            let image = table.get(glyph, size).unwrap();
            assert_eq!(image.pixels_per_em, size);
            assert_eq!(image.data.iter().any(|&byte| byte != 0), expect_ink);
        }
        assert!(!table
            .get(glyph, u16::MAX)
            .unwrap()
            .data
            .iter()
            .any(|&byte| byte != 0));
        assert_eq!(
            bitmap::strike_tables_may_supply_ink(&locations, &table, glyph),
            expected[1]
        );
    }
}

#[test]
fn original_bitmap_and_svg_keep_their_ink_when_the_outline_is_empty() {
    let data = include_bytes!("../fixtures/fonts/RHWPBitmapSvgGlyphSmoke.ttf");
    let original = Face::parse(data, 0).unwrap();
    let raster = original.glyph_index('\u{e100}').unwrap();
    let svg = original.glyph_index('\u{e101}').unwrap();
    assert_eq!(original.glyph_bounding_box(raster), None);
    assert_eq!(original.glyph_bounding_box(svg), None);
    assert!(original.glyph_raster_image(raster, u16::MAX).is_some());
    assert!(original.glyph_svg_image(raster).is_none());
    assert!(bitmap::has_color_or_bitmap_ink(&original, raster));

    // 같은 원본의 파싱된 SVG 표만 노출해 독립 공급을 확인한다.
    let read = |tag| original.raw_face().table(Tag::from_bytes(tag));
    let svg_only = Face::from_raw_tables(RawFaceTables {
        head: read(b"head").unwrap(),
        hhea: read(b"hhea").unwrap(),
        maxp: read(b"maxp").unwrap(),
        svg: read(b"SVG "),
        ..Default::default()
    })
    .unwrap();
    assert!(svg_only.glyph_raster_image(svg, u16::MAX).is_none());
    assert!(svg_only.glyph_svg_image(svg).is_some());
    assert!(bitmap::has_color_or_bitmap_ink(&svg_only, svg));
    assert!(!bitmap::has_color_or_bitmap_ink(&svg_only, raster));
}
