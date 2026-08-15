//! WMF/EMF 임베디드 메타파일 파서·컨버터의 DoS 하드닝 회귀 테스트.
//!
//! 손상된 메타파일 바이트(치수·좌표 필드 극단값)가 산술 오버플로 패닉이나
//! 거대 할당(OOM)으로 번지지 않고 graceful 하게 처리되는지 확인한다. 디버그
//! 빌드(overflow-checks on)에서 회귀 시 아래 테스트가 패닉으로 실패한다.

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

/// placeable(22B) + METAHEADER(18B) 프리픽스. 인자로 placeable 경계 사각형을 지정.
fn wmf_header(left: i16, top: i16, right: i16, bottom: i16) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&u32le(0x9AC6_CDD7)); // placeable key
    b.extend_from_slice(&u16le(0)); // hwmf
    b.extend_from_slice(&i16le(left));
    b.extend_from_slice(&i16le(top));
    b.extend_from_slice(&i16le(right));
    b.extend_from_slice(&i16le(bottom));
    b.extend_from_slice(&u16le(0)); // inch
    b.extend_from_slice(&u32le(0)); // reserved
    b.extend_from_slice(&u16le(0)); // checksum
    assert_eq!(b.len(), 22);
    // METAHEADER
    b.extend_from_slice(&u16le(1)); // type
    b.extend_from_slice(&u16le(9)); // header size (words)
    b.extend_from_slice(&u16le(0x0300)); // version
    b.extend_from_slice(&u32le(0)); // size
    b.extend_from_slice(&u16le(0)); // number of objects
    b.extend_from_slice(&u32le(0)); // max record
    b.extend_from_slice(&u16le(0)); // number of members
    assert_eq!(b.len(), 40);
    b
}

fn eof_record() -> Vec<u8> {
    let mut b = Vec::new();
    b.extend_from_slice(&u32le(3)); // record size (words)
    b.extend_from_slice(&u16le(0x0000)); // META_EOF
    b
}

/// META_STRETCHDIB 캐리어 (Info 40B DIB 헤더).
fn stretchdib_info(width: i32, bit_count: u16) -> Vec<u8> {
    let mut b = wmf_header(0, 0, 100, 100);
    b.extend_from_slice(&u32le(0x40));
    b.extend_from_slice(&u16le(0x0F43)); // META_STRETCHDIB
    b.extend_from_slice(&u32le(0x00CC_0020)); // SRCCOPY
    b.extend_from_slice(&u16le(0)); // DIB_RGB_COLORS
    for _ in 0..8 {
        b.extend_from_slice(&i16le(0));
    }
    b.extend_from_slice(&u32le(40)); // header_size
    b.extend_from_slice(&i32le(width));
    b.extend_from_slice(&i32le(1)); // height
    b.extend_from_slice(&u16le(1)); // planes
    b.extend_from_slice(&u16le(bit_count));
    b.extend_from_slice(&u32le(0)); // BI_RGB
    b.extend_from_slice(&u32le(0)); // image_size
    b.extend_from_slice(&i32le(0));
    b.extend_from_slice(&i32le(0));
    b.extend_from_slice(&u32le(0)); // color_used
    b.extend_from_slice(&u32le(0)); // color_important
    b.extend_from_slice(&[0u8; 16]);
    b
}

/// META_STRETCHDIB 캐리어 (Core 12B DIB 헤더).
fn stretchdib_core(width: u16, bit_count: u16) -> Vec<u8> {
    let mut b = wmf_header(0, 0, 100, 100);
    b.extend_from_slice(&u32le(0x40));
    b.extend_from_slice(&u16le(0x0F43));
    b.extend_from_slice(&u32le(0x00CC_0020));
    b.extend_from_slice(&u16le(0));
    for _ in 0..8 {
        b.extend_from_slice(&i16le(0));
    }
    b.extend_from_slice(&u32le(12)); // header_size = 0x0C
    b.extend_from_slice(&u16le(width));
    b.extend_from_slice(&u16le(1)); // height
    b.extend_from_slice(&u16le(1)); // planes
    b.extend_from_slice(&u16le(bit_count));
    b.extend_from_slice(&[0u8; 16]);
    b
}

fn convert_wmf(data: &[u8]) {
    // 반환값은 무시 — 패닉/abort 없이 돌아오기만 하면 통과.
    let _ = WMFConverter::new(data, SVGPlayer::new()).run();
}

/// placeable 경계 좌표(i16)의 `right - left`가 오버플로해도 패닉하지 않는다.
#[test]
fn wmf_placeable_bounds_overflow_is_graceful() {
    let mut data = wmf_header(-32768, -32768, 32767, 32767);
    data.extend_from_slice(&eof_record());
    convert_wmf(&data);
}

/// Info DIB 치수(width*bitcount)가 u32를 넘겨도 패닉/OOM 없이 처리된다.
#[test]
fn wmf_stretchdib_info_huge_dimensions_are_graceful() {
    convert_wmf(&stretchdib_info(0x1000_0000, 0x0020)); // BI_BITCOUNT_6 = 32bpp
    convert_wmf(&stretchdib_info(i32::MAX, 0x0020));
}

/// Core DIB 치수(width*bitcount)가 u16을 넘겨도 패닉하지 않는다.
#[test]
fn wmf_stretchdib_core_huge_dimensions_are_graceful() {
    convert_wmf(&stretchdib_core(0xFFFF, 0x0018)); // BI_BITCOUNT_5 = 24bpp
}

/// 유효한 소형 Info DIB는 여전히 패닉 없이 변환된다(무회귀 sanity).
#[test]
fn wmf_valid_small_dib_still_converts() {
    convert_wmf(&stretchdib_info(4, 0x0020));
    convert_wmf(&stretchdib_core(4, 0x0018));
}

/// EMF EMR_HEADER 경계(i32)의 `right - left`가 오버플로해도 패닉하지 않는다.
#[test]
fn emf_header_bounds_overflow_is_graceful() {
    let mut b = Vec::new();
    b.extend_from_slice(&u32le(1)); // EMR_HEADER
    b.extend_from_slice(&u32le(88)); // size
    b.extend_from_slice(&i32le(-1)); // bounds.left
    b.extend_from_slice(&i32le(-1)); // bounds.top
    b.extend_from_slice(&i32le(i32::MAX)); // bounds.right → right - left 오버플로
    b.extend_from_slice(&i32le(i32::MAX)); // bounds.bottom
    b.extend_from_slice(&i32le(0));
    b.extend_from_slice(&i32le(0));
    b.extend_from_slice(&i32le(10000));
    b.extend_from_slice(&i32le(5000)); // frame
    b.extend_from_slice(&u32le(0x464D_4520)); // " EMF"
    b.extend_from_slice(&u32le(0x0001_0000)); // version
    b.extend_from_slice(&u32le(108)); // bytes
    b.extend_from_slice(&u32le(2)); // records
    b.extend_from_slice(&u16le(1)); // handles
    b.extend_from_slice(&u16le(0)); // reserved
    b.extend_from_slice(&u32le(0));
    b.extend_from_slice(&u32le(0));
    b.extend_from_slice(&u32le(0));
    b.extend_from_slice(&i32le(1920));
    b.extend_from_slice(&i32le(1080));
    b.extend_from_slice(&i32le(508));
    b.extend_from_slice(&i32le(286));
    b.extend_from_slice(&u32le(14)); // EMR_EOF
    b.extend_from_slice(&u32le(20));
    b.extend_from_slice(&u32le(0));
    b.extend_from_slice(&u32le(0));
    b.extend_from_slice(&u32le(20));

    // 파싱 + SVG 변환 모두 패닉 없이 돌아와야 한다.
    let _ = rhwp::emf::parse_emf(&b);
    let _ = rhwp::emf::convert_to_svg(&b, (0.0, 0.0, 100.0, 100.0));
}
