//! [#6865] `PATINVERT → DPa → PATINVERT`의 마스크는 바탕과 브러시 색을 선택한다.
//!
//! `PATINVERT → DPa → PATINVERT`의 비트 연산은 `(D XOR G) AND M XOR G`다.
//! M이 검정이면 G, 흰색이면 원래 D가 남는다. 흑백 패턴은 직접 덮어 칠하지 않고
//! 반전한 휘도 mask에서만 소비하며, 실제 브러시 paint는 그 mask를 한 번 참조한다.
//!
//! 과거 PATCOPY 근사에서는 mask를 없앴지만, 원본 mask 선택을 보존하는 SVG가
//! 패턴 정의를 가지는 것은 정상이다. 정의의 문자열 부재로 보이는 칠을 판정하지 않는다.
//! 합성 레코드의 자원 소유·참조·반전 계약과 실제 WMF curve 바이트의 기존 검사를 유지한다.
//! HWP 전체 페이지·글꼴·그림 위치의 한컴 피델리티를 이 단위 계약으로 대신하지 않는다.
//!
//! ```text
//!   36 CreateBrushIndirect   style=SOLID color=#D9D9D9
//!   38 DibBitBlt             rop=0x005A0049  PATINVERT
//!   40 DibCreatePatternBrush 8x8 bpp=1 palette=#000000,#FFFFFF   ← 마스크
//!                            비트 55 AA 55 AA … (정확히 50% 체커보드)
//!   44 DibBitBlt             rop=0x00A000C9  DPa
//!   49 DibBitBlt             rop=0x005A0049  PATINVERT
//! ```
//!
//! **관용구의 참 의미.** 마스크 비트가 1(흰색)이면 바탕이 남고, 0(검정)이면 브러시 색이
//! 남는다.
//!
//! ```text
//!   M=0xFFFFFF :  (D ⊕ G) ∧ 0xFFFFFF ⊕ G = D      바탕 그대로
//!   M=0x000000 :  (D ⊕ G) ∧ 0x000000 ⊕ G = G      브러시 색
//! ```
//!
//! 마스크를 단색으로 대체하거나 삭제하면 흰 비트가 보존해야 할 바탕도 지운다.
//! 아래 합성 검사는 MS-WMF의 비트 연산을 독립 기대값으로 사용한다. 원본 패턴의
//! 팔레트 반전과 중간 연산 영역도 보존해야 하며 SVG 태그·정의 이름은 기대값이 아니다.
//! 실문서의 전쪽 Print 일치 여부는 별도의 Native/fresh WASM TSV로 확인한다.
//!
//! **과거 단색 근사의 실측(3쪽 패널 4점, 96dpi PNG).**
//!
//! | | 수정 전 | 수정 후 | 정본(engine 2020) |
//! |---|---:|---:|---:|
//! | (130,460) | 96 | **217** | 245 |
//! | (150,250) | 98 | **217** | 245 |
//!
//! 당시의 28 계조 차이는 현재 출력의 기대값이 아니다. 현 검사는 패턴 삭제 여부 대신
//! 비트별 바탕 보존과 브러시 선택을 확인하며 실문서 점수와 구분한다.
//!
//! **폭발반경.** 코퍼스 10,000건에서 1비트 패턴 브러시를 가진 문서는 21건뿐이고, 전/후
//! 바이너리 렌더 대조에서 쪽수가 달라진 문서는 0이다.

use rhwp::wmf::converter::{SVGPlayer, WMFConverter};

fn u32le(v: u32) -> [u8; 4] {
    v.to_le_bytes()
}
fn u16le(v: u16) -> [u8; 2] {
    v.to_le_bytes()
}
fn i32le(v: i32) -> [u8; 4] {
    v.to_le_bytes()
}
fn i16le(v: i16) -> [u8; 2] {
    v.to_le_bytes()
}

const PATINVERT: u32 = 0x005A_0049;
const DPA: u32 = 0x00A0_00C9;

/// placeable 헤더(22바이트)와 METAHEADER(18바이트)다.
fn header() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&u32le(0x9AC6_CDD7));
    b.extend_from_slice(&u16le(0));
    for v in [0i16, 0, 100, 100] {
        b.extend_from_slice(&i16le(v));
    }
    b.extend_from_slice(&u16le(96)); // inch
    b.extend_from_slice(&u32le(0));
    b.extend_from_slice(&u16le(0));
    b.extend_from_slice(&u16le(1)); // type = memory metafile
    b.extend_from_slice(&u16le(9)); // header size (words)
    b.extend_from_slice(&u16le(0x0300));
    b.extend_from_slice(&u32le(0));
    b.extend_from_slice(&u16le(4)); // number of objects
    b.extend_from_slice(&u32le(0));
    b.extend_from_slice(&u16le(0));
    assert_eq!(b.len(), 40);
    b
}

fn record(func: u16, params: &[u8]) -> Vec<u8> {
    let words = 3 + params.len() / 2;
    let mut b = Vec::new();
    b.extend_from_slice(&u32le(words as u32));
    b.extend_from_slice(&u16le(func));
    b.extend_from_slice(params);
    b
}

/// `META_CREATEBRUSHINDIRECT` — 단색 브러시.
fn solid_brush(rgb: (u8, u8, u8)) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&u16le(0)); // BS_SOLID
    p.extend_from_slice(&[rgb.0, rgb.1, rgb.2, 0]);
    p.extend_from_slice(&u16le(0)); // hatch
    record(0x02FC, &p)
}

/// `META_DIBCREATEPATTERNBRUSH` — `bpp` 비트 8×8 DIB 패턴 브러시.
///
/// `bpp == 1` 이면 실측과 같은 50% 체커보드(0x55/0xAA), 그 밖이면 24비트 그림이다.
fn dib_pattern_brush(bpp: u16) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&u16le(5)); // BS_DIBPATTERN
    p.extend_from_slice(&u16le(0)); // DIB_RGB_COLORS
    p.extend_from_slice(&u32le(40)); // BITMAPINFOHEADER
    p.extend_from_slice(&i32le(8));
    p.extend_from_slice(&i32le(8));
    p.extend_from_slice(&u16le(1)); // planes
    p.extend_from_slice(&u16le(bpp));
    p.extend_from_slice(&u32le(0)); // BI_RGB
    p.extend_from_slice(&u32le(0));
    p.extend_from_slice(&i32le(0));
    p.extend_from_slice(&i32le(0));
    p.extend_from_slice(&u32le(0));
    p.extend_from_slice(&u32le(0));
    if bpp == 1 {
        p.extend_from_slice(&[0, 0, 0, 0]); // 팔레트 0 = 검정
        p.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0]); // 팔레트 1 = 흰색
        for row in 0..8u8 {
            let bits = if row % 2 == 0 { 0x55 } else { 0xAA };
            p.extend_from_slice(&[bits, 0, 0, 0]); // 8×8 1bpp, 4바이트 정렬
        }
    } else {
        // 8×8 24bpp — 색을 싣는 진짜 그림이라 마스크가 아니다.
        for _ in 0..8 * 8 {
            p.extend_from_slice(&[0x20, 0x40, 0x80]);
        }
    }
    record(0x0142, &p)
}

fn select_object(index: u16) -> Vec<u8> {
    record(0x012D, &u16le(index))
}

/// 소스 없는 `META_DIBBITBLT` — 브러시만 쓰는 ROP.
///
/// 파서는 `record_size == (record_function >> 8) + 3` 일 때만 "소스 없음"으로 읽는다.
/// 0x0940 이면 12워드(24바이트)여야 하므로 파라미터는 `reserved` 를 포함해 18바이트다.
fn brush_blit(rop: u32) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&u32le(rop));
    p.extend_from_slice(&i16le(0)); // YSrc
    p.extend_from_slice(&i16le(0)); // XSrc
    p.extend_from_slice(&u16le(0)); // reserved
    for v in [50i16, 50, 10, 10] {
        p.extend_from_slice(&i16le(v)); // Height, Width, YDest, XDest
    }
    let out = record(0x0940, &p);
    assert_eq!(out.len(), 24, "소스 없는 DIBBITBLT 는 12워드다");
    out
}

fn eof() -> Vec<u8> {
    record(0x0000, &[])
}

/// `PATINVERT(#D9D9D9) → DPa(패턴 브러시) → PATINVERT(#D9D9D9)` 관용구.
fn idiom(pattern_bpp: u16) -> Vec<u8> {
    let mut b = header();
    for part in [
        solid_brush((0xD9, 0xD9, 0xD9)),
        select_object(0),
        brush_blit(PATINVERT),
        dib_pattern_brush(pattern_bpp),
        select_object(1),
        brush_blit(DPA),
        select_object(0),
        brush_blit(PATINVERT),
        eof(),
    ] {
        b.extend_from_slice(&part);
    }
    b
}

fn to_svg(bytes: &[u8]) -> String {
    let bytes = WMFConverter::new(bytes, SVGPlayer::new())
        .run()
        .expect("WMF 변환은 성공해야 한다");
    String::from_utf8(bytes).expect("SVG 는 UTF-8 이다")
}

#[test]
fn issue_6865_complete_mask_selects_brush_or_original_destination() {
    let svg = to_svg(&idiom(1));
    assert!(
        svg.contains("#D9D9D9") || svg.contains("rgb(217,217,217)"),
        "앞 PATINVERT 의 브러시 색은 남아야 한다:\n{svg}"
    );
    assert_xor_pattern_pixels(&svg, [10, 10, 50, 50], false);
    // 동일한 비트 연산이 벡터 윤곽 사이에 저장된 정상 한컴 WMF도 대조한다.
    // 월간 수출입 보도자료 BinData/BIN0003.wmf의 원본 바이트이며, 독립 PDF1쪽은
    // 붉은 사각형 대신 8×8 흑백 패턴을 적용한 꺾은선으로 표시한다.
    let curve = to_svg(include_bytes!("../fixtures/wmf_monthly_trade_curve.wmf"));
    assert!(
        curve.contains("rop_vector_mask"),
        "벡터 윤곽의 마스크를 보존해야 한다"
    );
    assert!(
        curve.contains("fill=\"#C00000\""),
        "원본 선 색을 보존해야 한다"
    );
    assert!(
        !curve.contains("height=\"162\" id=\"elem511\""),
        "첫 XOR을 붉은 면으로 칠하지 않는다"
    );
    assert!(
        !curve.contains("height=\"162\" id=\"elem524\""),
        "마지막 XOR을 붉은 면으로 칠하지 않는다"
    );
}

#[test]
fn issue_6865_multibit_pattern_blit_is_still_painted() {
    // **음성 대조** — `#6469` 가 살려 둔 "그림을 실은 DPa" 다. 1비트가 아니므로 종전처럼
    // 패턴으로 그려져야 한다. 이 갈래까지 지우면 흰 원이 사라진다.
    let svg = to_svg(&idiom(24));
    assert!(
        svg.contains("rop_pat"),
        "1비트가 아닌 패턴은 그대로 그려야 한다:\n{svg}"
    );
}

#[test]
fn issue_6865_lone_monochrome_pattern_blit_is_unaffected() {
    // **음성 대조** — 관용구 밖(앞선 PATINVERT 가 없는) 단독 `DPa` 는 상쇄 대상이 아니다.
    // 좁힘이 "1비트면 무조건 지운다" 로 새지 않는지 잠근다.
    let mut b = header();
    for part in [
        dib_pattern_brush(1),
        select_object(0),
        brush_blit(DPA),
        eof(),
    ] {
        b.extend_from_slice(&part);
    }
    let svg = to_svg(&b);
    assert!(
        svg.contains("rop_pat"),
        "관용구 밖 단독 마스크 blit 은 종전대로 그려야 한다:\n{svg}"
    );
}

fn mask_prefix(pattern: Vec<u8>, middle: Vec<u8>) -> Vec<u8> {
    let mut bytes = header();
    for part in [
        solid_brush((0xD9, 0xD9, 0xD9)),
        select_object(0),
        brush_blit(PATINVERT),
        pattern,
        select_object(1),
        middle,
    ] {
        bytes.extend_from_slice(&part);
    }
    bytes
}

fn finish_sequence(mut bytes: Vec<u8>, tail: &[Vec<u8>]) -> String {
    for part in tail {
        bytes.extend_from_slice(part);
    }
    bytes.extend_from_slice(&eof());
    to_svg(&bytes)
}

fn blit_at(rop: u32, x: i16, y: i16) -> Vec<u8> {
    let mut bytes = brush_blit(rop);
    bytes[20..22].copy_from_slice(&i16le(y));
    bytes[22..24].copy_from_slice(&i16le(x));
    bytes
}

fn blit_rect(rop: u32, x: i16, y: i16, width: i16, height: i16) -> Vec<u8> {
    let mut bytes = blit_at(rop, x, y);
    bytes[16..18].copy_from_slice(&i16le(height));
    bytes[18..20].copy_from_slice(&i16le(width));
    bytes
}

#[test]
fn issue_6865_halftone_padding_keeps_each_operation_region() {
    for rect in [[9, 9, 51, 51], [9, 11, 51, 50]] {
        let [x, y, width, height] = rect;
        let svg = finish_sequence(
            mask_prefix(dib_pattern_brush(1), blit_rect(DPA, x, y, width, height)),
            &[select_object(0), brush_blit(PATINVERT)],
        );
        assert_xor_pattern_pixels(&svg, rect.map(i32::from), false);
    }
}

#[test]
fn issue_6865_halftone_two_unit_edge_difference_is_preserved() {
    let svg = finish_sequence(
        mask_prefix(dib_pattern_brush(1), blit_rect(DPA, 8, 9, 52, 51)),
        &[select_object(0), brush_blit(PATINVERT)],
    );
    assert!(svg.contains("rop_pat0"));
}

#[test]
fn issue_6865_non_halftone_one_unit_difference_is_preserved() {
    let mut pattern = dib_pattern_brush(1);
    // 첫 픽셀 행을 바꾸되 유효한 흑백 1bpp 비트맵은 유지한다.
    pattern[6 + 2 + 2 + 40 + 8] = 0;
    let svg = finish_sequence(
        mask_prefix(pattern, blit_rect(DPA, 9, 9, 51, 51)),
        &[select_object(0), brush_blit(PATINVERT)],
    );
    assert!(svg.contains("rop_pat0"));
}

#[test]
fn issue_6865_one_unit_final_xor_difference_is_preserved() {
    let svg = finish_sequence(
        mask_prefix(dib_pattern_brush(1), blit_rect(DPA, 9, 9, 51, 51)),
        &[select_object(0), blit_at(PATINVERT, 11, 10)],
    );
    assert!(svg.contains("rop_pat0"));
}

#[test]
fn issue_6865_incomplete_idiom_keeps_middle_draw() {
    let svg = finish_sequence(mask_prefix(dib_pattern_brush(1), brush_blit(DPA)), &[]);
    assert!(
        svg.contains("rop_pat0"),
        "EOF must not erase an unconfirmed draw"
    );
    let mut curve = include_bytes!("../fixtures/wmf_monthly_trade_curve.wmf").to_vec();
    // 첫 꺾은선의 마지막 PATINVERT만 DSTINVERT로 바꾼다. 완성되지 않은 쌍은
    // 가운데 윤곽을 임의로 색칠하거나 첫 사각형을 상쇄하면 안 된다.
    let mut offset = 18usize;
    let mut xor_count = 0;
    while offset + 6 <= curve.len() {
        let words = u32::from_le_bytes(curve[offset..offset + 4].try_into().unwrap()) as usize;
        assert!(words >= 3);
        let function = u16::from_le_bytes(curve[offset + 4..offset + 6].try_into().unwrap());
        if function == 0x0940 && curve[offset + 6..offset + 10] == PATINVERT.to_le_bytes() {
            xor_count += 1;
            if xor_count == 2 {
                curve[offset + 6..offset + 10].copy_from_slice(&0x0055_0009u32.to_le_bytes());
                break;
            }
        }
        offset += words * 2;
    }
    assert_eq!(xor_count, 2);
    let svg = to_svg(&curve);
    assert!(
        svg.contains("height=\"162\" id=\"elem511\""),
        "불완전한 벡터 연산의 첫 칠을 보존한다"
    );
}

#[test]
fn issue_6865_different_middle_region_is_not_a_mask_idiom() {
    let svg = finish_sequence(
        mask_prefix(dib_pattern_brush(1), blit_at(DPA, 25, 30)),
        &[select_object(0), brush_blit(PATINVERT)],
    );
    assert!(svg.contains("rop_pat0"));
    assert_eq!(
        svg.matches("fill=\"#D9D9D9\"").count(),
        1,
        "preserving DPA must not disable the existing outer XOR cancellation"
    );
}

#[test]
fn issue_6865_different_final_region_keeps_middle_draw() {
    let svg = finish_sequence(
        mask_prefix(dib_pattern_brush(1), brush_blit(DPA)),
        &[select_object(0), blit_at(PATINVERT, 25, 30)],
    );
    assert!(svg.contains("rop_pat0"));
}

#[test]
fn issue_6865_different_final_brush_keeps_middle_draw() {
    let svg = finish_sequence(
        mask_prefix(dib_pattern_brush(1), brush_blit(DPA)),
        &[
            solid_brush((0x10, 0x20, 0x30)),
            select_object(2),
            brush_blit(PATINVERT),
        ],
    );
    assert!(svg.contains("rop_pat0"));
}

#[test]
fn issue_6865_intervening_draw_keeps_middle_draw() {
    let svg = finish_sequence(
        mask_prefix(dib_pattern_brush(1), brush_blit(DPA)),
        &[
            brush_blit(0x00F0_0021),
            select_object(0),
            brush_blit(PATINVERT),
        ],
    );
    assert!(svg.contains("rop_pat0"));
}

#[test]
fn issue_6865_clip_change_keeps_middle_draw() {
    let mut clip = Vec::new();
    for coordinate in [45i16, 45, 15, 15] {
        clip.extend_from_slice(&i16le(coordinate));
    }
    let svg = finish_sequence(
        mask_prefix(dib_pattern_brush(1), brush_blit(DPA)),
        &[
            record(0x0416, &clip),
            select_object(0),
            brush_blit(PATINVERT),
        ],
    );
    assert!(svg.contains("rop_pat0"));
    assert_eq!(
        svg.matches("fill=\"#D9D9D9\"").count(),
        2,
        "different DC clips must not cancel the outer XOR draws"
    );
}

#[test]
fn issue_6865_colored_one_bit_pattern_is_preserved() {
    let mut pattern = dib_pattern_brush(1);
    let palette = 6 + 2 + 2 + 40;
    pattern[palette..palette + 8].copy_from_slice(&[255, 0, 0, 0, 0, 0, 255, 0]);
    let svg = finish_sequence(
        mask_prefix(pattern, brush_blit(DPA)),
        &[select_object(0), brush_blit(PATINVERT)],
    );
    assert!(svg.contains("rop_pat0"));
    let mut curve = include_bytes!("../fixtures/wmf_monthly_trade_curve.wmf").to_vec();
    // 검은 팔레트를 붉은색으로 바꾸면 흑백 마스크 계약을 적용할 수 없다.
    let palette = [0u8, 0, 0, 0, 255, 255, 255, 0];
    let positions = curve
        .windows(palette.len())
        .enumerate()
        .filter_map(|(i, bytes)| (bytes == palette).then_some(i))
        .collect::<Vec<_>>();
    assert!(!positions.is_empty());
    for position in positions {
        curve[position + 2] = 255;
    }
    let svg = to_svg(&curve);
    assert!(
        !svg.contains("rop_vector_mask"),
        "다색 1비트 패턴을 흑백으로 처리하지 않는다"
    );
}

#[test]
fn issue_6865_reversed_palette_reverses_brush_and_destination_selection() {
    let mut pattern = dib_pattern_brush(1);
    let palette = 6 + 2 + 2 + 40;
    pattern[palette..palette + 8].copy_from_slice(&[255, 255, 255, 0, 0, 0, 0, 0]);
    let svg = finish_sequence(
        mask_prefix(pattern, brush_blit(DPA)),
        &[select_object(0), brush_blit(PATINVERT)],
    );
    assert_xor_pattern_pixels(&svg, [10, 10, 50, 50], true);
}

/// 합성 WMF에 기록한 실제 연산 영역과 DIB 비트로 기대 색을 계산한다.
/// 실물 문서의 화면 좌표 핀이 아니라 GDI 비트 연산의 독립 계약이다.
fn assert_xor_pattern_pixels(svg: &str, middle: [i32; 4], reversed: bool) {
    assert_xor_pattern_pixels_with_source_row(svg, middle, reversed, false);
}

fn assert_xor_pattern_pixels_with_source_row(
    svg: &str,
    middle: [i32; 4],
    reversed: bool,
    zero_bottom_row: bool,
) {
    use resvg::tiny_skia::{Color, Pixmap, Transform};
    let tree = resvg::usvg::Tree::from_str(svg, &resvg::usvg::Options::default())
        .expect("WMF 변환 SVG 해석");
    let size = tree.size().to_int_size();
    let mut bitmap = Pixmap::new(size.width(), size.height()).unwrap();
    let destination = [31u8, 79, 127];
    bitmap.fill(Color::from_rgba8(31, 79, 127, 255));
    // 한 WMF 장치 픽셀을 한 출력 픽셀로 재생한다. 확대 후 소수 위치에서
    // 표본을 고르면 영상 보간 결과가 섞여 이 이산 비트 계약을 검사할 수 없다.
    resvg::render(&tree, Transform::identity(), &mut bitmap.as_mut());
    let [left, top, width, height] = middle;
    for y in 8..63 {
        for x in 8..63 {
            let outer = (10..60).contains(&x) && (10..60).contains(&y);
            let masked = (left..left + width).contains(&x) && (top..top + height).contains(&y);
            // DIB는 bottom-up이다. 첫 저장 행은 0x55, 맨 위 행은 0xAA이므로
            // 기본 팔레트에서 x+y가 짝수인 비트가 흰색이다.
            // 체커가 아닌 대조군은 DIB의 맨 아래 0번 행을 모두 검게 바꾼다.
            // DIB는 아래에서 위로 저장하므로 장치 좌표의 행은 y mod 8 == 7이다.
            let mask = if zero_bottom_row && y % 8 == 7 {
                0
            } else if ((x + y) % 2 == 0) ^ reversed {
                255
            } else {
                0
            };
            let expected = destination.map(|mut value| {
                if outer {
                    value ^= 217;
                }
                if masked {
                    value &= mask;
                }
                if outer {
                    value ^= 217;
                }
                value
            });
            let actual = bitmap.pixel(x as u32, y as u32).unwrap();
            assert_eq!(actual.alpha(), 255, "원래 바탕은 불투명하다");
            for (channel, expected) in [actual.red(), actual.green(), actual.blue()]
                .into_iter()
                .zip(expected)
            {
                assert!(
                    channel.abs_diff(expected) <= 1,
                    "비트 연산 결과 불일치: ({x},{y}), middle={middle:?}, \
                     reversed={reversed}, actual={actual:?}, expected={expected}"
                );
            }
        }
    }
}

/// 마스크 밖은 두 XOR이 상쇄된다. 안쪽 흑백 무늬가 checker일 필요는 없다.
#[test]
fn contained_non_checker_mask_keeps_the_original_brush_as_its_only_paint() {
    let mut pattern = dib_pattern_brush(1);
    pattern[6 + 2 + 2 + 40 + 8] = 0;
    // 완전 안쪽·한쪽 경계 공유·8단위보다 작은 마스크에도 같은 비트 항등식이다.
    for rect in [[14, 15, 30, 20], [11, 10, 49, 50], [31, 25, 2, 3]] {
        let [x, y, width, height] = rect;
        let middle = blit_rect(DPA, x, y, width, height);
        let svg = finish_sequence(
            mask_prefix(pattern.clone(), middle),
            &[select_object(0), brush_blit(PATINVERT)],
        );
        assert_xor_pattern_pixels_with_source_row(&svg, rect.map(i32::from), false, true);
    }
}

/// 원본 1비트의 선택을 끝낸 뒤 장치 표본으로 축소해야 한다.
/// 문서의 좌표 답안이 아니라 흑/백이 같은 수인 마스크의 면적 법칙이다.
#[test]
fn print_device_mask_reduction_preserves_source_bit_coverage() {
    use rhwp::paint::{
        LayerBuilder, LayerNode, LayerNodeKind, PaintOp, RenderProfile, ResolvedImageKind,
    };
    use rhwp::renderer::render_tree::{
        BoundingBox, ImageNode, PageRenderTree, RenderNode, RenderNodeType,
    };

    fn payload(node: &LayerNode) -> &rhwp::paint::ResolvedImagePayload {
        match &node.kind {
            LayerNodeKind::Group { children, .. } => {
                assert_eq!(children.len(), 1, "원본 그림의 단일 소유");
                payload(&children[0])
            }
            LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
                payload(child)
            }
            LayerNodeKind::Leaf { ops } => {
                assert_eq!(ops.len(), 1, "그림의 누락·중복 방지");
                let PaintOp::Image {
                    resolved: Some(resolved),
                    ..
                } = &ops[0]
                else {
                    panic!("공통 Print 장치 그림을 실제로 소비해야 한다")
                };
                assert_eq!(resolved.kind, ResolvedImageKind::OutputDevice);
                assert_eq!(resolved.mime, "image/png");
                resolved
            }
        }
    }

    for pattern_bits in [1, 24] {
        let source = idiom(pattern_bits);
        for size in [16.0, 20.0, 32.0, 100.0, 200.0] {
            let frame = BoundingBox::new(0.0, 0.0, size, size);
            let mut tree = PageRenderTree::new(0, size, size);
            tree.root.children.push(RenderNode::new(
                1,
                RenderNodeType::Image(ImageNode::new(1, Some(source.clone()))),
                frame,
            ));
            let original = serde_json::to_value(&tree.root).unwrap();
            let mut builder = LayerBuilder::new(RenderProfile::Print)
                .with_hancom2024_print_bitmap(true)
                .with_metafile_device_pixel_width(Some(1.0));
            let layer = builder.build(&tree);
            let bytes = &payload(&layer.root).data;
            let image = image::load_from_memory(bytes).unwrap().to_rgba8();
            assert_eq!(image.dimensions(), (size as u32, size as u32));
            let mut alpha = 0u64;
            let mut count = 0u64;
            for (x, y, pixel) in image.enumerate_pixels() {
                // 10..60 원본 ROP 영역의 안쪽만 적분해 바깥 경계의 표본 오차와 구별한다.
                let source_x = (f64::from(x) + 0.5) * 100.0 / size;
                let source_y = (f64::from(y) + 0.5) * 100.0 / size;
                if !(20.0..50.0).contains(&source_x) || !(20.0..50.0).contains(&source_y) {
                    continue;
                }
                alpha += u64::from(pixel[3]);
                count += 1;
                if pattern_bits == 1 && pixel[3] > 64 {
                    assert!(
                        pixel.0[..3].iter().all(|v| {
                            // PNG의 비곱셈 색을 원래 8비트 premultiplied 단위로 되돌린다.
                            // 두 번의 정수 변환에서 생길 수 있는 한 단계만 허용한다.
                            let premultiplied = |channel: u8| {
                                (u16::from(channel) * u16::from(pixel[3]) + 127) / 255
                            };
                            premultiplied(*v).abs_diff(premultiplied(217)) <= 1
                        }),
                        "마스크 선택은 원래 브러시 색을 바꾸지 않는다: {pixel:?}"
                    );
                }
            }
            assert!(count > 0);
            let coverage = alpha as f64 / (255.0 * count as f64);
            if pattern_bits == 1 {
                assert!(
                    (coverage - 0.5).abs() < 0.08,
                    "흑백 원본 비트의 50% 피복률 보존: size={size}, coverage={coverage}"
                );
            } else {
                assert!(
                    (coverage - 1.0).abs() < 0.01,
                    "다색 패턴은 흑백 마스크로 처리하지 않는다: {coverage}"
                );
            }
            assert_eq!(
                serde_json::to_value(&tree.root).unwrap(),
                original,
                "장치 표본 생산은 원본·배치를 바꾸지 않는다"
            );
            assert_eq!(
                payload(&builder.build(&tree).root).data,
                *bytes,
                "반복 조회도 같은 장치 바이트를 전달한다"
            );
        }
    }
}
