//! [#6865] `PATINVERT → DPa → PATINVERT` 관용구의 **1비트 마스크**는 보이는 칠이 아니다.
//!
//! **증상.** 156627451 3쪽 WMF 도해의 옅은 회색 패널이 흑백 체커보드(50% 회색)로 칠해져
//! 그 위 글자가 배경에 묻힌다. 7쪽 배너는 그러데이션 자리에 디더 띠 220개가 깔린다.
//!
//! **레코드 실측** — `BinData/image4.wmf`:
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
//! 곧 마스크는 **브러시 색이 어디에 남을지**를 정할 뿐 그 자신이 칠이 아니다. `#6469` 가
//! 이 관용구를 `PATCOPY` 로 근사하기로 했으므로 색은 이미 앞 `PATINVERT` 가 칠했고,
//! 가운데 마스크를 또 칠하면 흑백 디더가 그 위를 덮는다.
//!
//! **수정.** 상쇄기가 마지막 `PATINVERT` 만 지우던 것을, 가운데 `DPa` 의 브러시가 1비트일
//! 때 그 blit 도 함께 지우도록 했다. `#6469` 가 살려 둔 "**그림**을 실은 `DPa`"(흰 원)는
//! 1비트가 아니므로 종전대로 그려진다 — 아래 음성 대조가 그것을 잠근다.
//!
//! **실측(3쪽 패널 4점, 96dpi PNG).**
//!
//! | | 수정 전 | 수정 후 | 정본(engine 2020) |
//! |---|---:|---:|---:|
//! | (130,460) | 96 | **217** | 245 |
//! | (150,250) | 98 | **217** | 245 |
//!
//! 남는 28 계조는 `#6469` 의 `PATCOPY` 근사가 50% 하프톤을 단색으로 평탄화하기 때문이고
//! 별개 축이다 — 이 시험은 **디더가 칠로 나가지 않는다**만 잠근다.
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

/// placeable(22B) + METAHEADER(18B).
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
fn issue_6865_monochrome_mask_blit_is_not_painted() {
    let svg = to_svg(&idiom(1));
    assert!(
        svg.contains("#D9D9D9") || svg.contains("rgb(217,217,217)"),
        "앞 PATINVERT 의 브러시 색은 남아야 한다:\n{svg}"
    );
    assert!(
        !svg.contains("rop_pat"),
        "1비트 마스크는 보이는 칠로 나가면 안 된다:\n{svg}"
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
