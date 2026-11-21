//! 관계형·고정형 — [#9285] WMF DIB 의 색상표 길이는 담는 레코드의 크기로 정한다. 색상표 없이 저장된 1비트 DIB 를 색상표가 있는 것으로 읽지 않는다.
//! 주석 갱신: 2026-11-21 20:28 KST
//!
//! 입력: 산업통상자원부 보도자료 공개 문서(「제4차 어린이제품 안전관리 기본계획 발표」, HWPX,
//! `appVersion="11, 0, 0, 8658"`, 9쪽) `samples/issue9285/child-product-safety-plan-wmf-mask-dib.hwpx`.
//! 기준: Windows 데스크톱 한컴 2020 `11.0.0.9136` Hancom PDF Print
//! `pdf/issue9285/child-product-safety-plan-wmf-mask-dib-2020.pdf`(9쪽).
//!
//! 4쪽 1×4 표 셋째 칸의 그림 `BinData/image7.wmf` 는 그림 조각 넷을 `META_DIBSTRETCHBLT` 쌍(마스크
//! `SRCPAINT` + 그림 `SRCAND`)으로 싣는다. 마스크는 512×512 1비트 DIB 이고 레코드의 DIB 영역은
//! `BITMAPINFOHEADER` 40바이트 + 픽셀 32,768바이트뿐이다 — 색상표가 없다. [MS-WMF] 2.2.2.9 는
//! `Colors` 의 크기를 담는 레코드에서 정한다고 적는다. 수정 전 파서는 `BitCount` 만으로 색상표
//! 2항목(8바이트)을 읽어 다음 레코드 경계가 어긋났고, WMF 변환 전체가 실패해 그림 자리가 비었다.
//! - 고정형: 전체 쪽수 9(= PDF).
//! - 관계형(합성 WMF): 색상표 없는 1비트 `META_DIBSTRETCHBLT` 뒤의 레코드까지 변환되고, 결과가
//!   같은 비트에 흑백 색상표(0 = 검정, 1 = 흰색)를 명시한 WMF 의 결과와 같다.
//! - 관계형(실문서): 모든 쪽의 SVG 에 변환하지 못한 WMF 원본(`image/x-wmf`)이 남지 않는다.
//!
//! 원 PR의 한컴 2022 비교값은 참고 측정으로 보존한다. 4쪽은 Native 99.15%, fresh WASM 99.51%였다.
//! 통합 검증의 기준과 출처는 `mydocs/pr/assets/planet6897_9240_9282_20261121/wmf-9285/print-verification.json`에 기록한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::wmf::converter::{SVGPlayer, WMFConverter};
use std::path::Path;

const SAMPLE: &str = "samples/issue9285/child-product-safety-plan-wmf-mask-dib.hwpx";
/// 한컴 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 9;
const SRCPAINT: u32 = 0x00EE_0086;
/// 마스크 뒤 사각형의 채움색. 마스크 레코드 뒤까지 변환됐는지 이 색으로 확인한다.
const TAIL_FILL: (u8, u8, u8) = (0x12, 0x80, 0xC4);

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
    b.extend_from_slice(&u16le(1)); // number of objects
    b.extend_from_slice(&u32le(0));
    b.extend_from_slice(&u16le(0));
    assert_eq!(b.len(), 40);
    b
}

fn record(func: u16, params: &[u8]) -> Vec<u8> {
    assert_eq!(params.len() % 2, 0, "레코드 매개변수는 워드 단위다");
    let words = 3 + params.len() / 2;
    let mut b = Vec::new();
    b.extend_from_slice(&u32le(words as u32));
    b.extend_from_slice(&u16le(func));
    b.extend_from_slice(params);
    b
}

/// `META_DIBSTRETCHBLT`(0x0B41) — 8×8 1비트 DIB 를 `SRCPAINT` 로 옮긴다.
///
/// `with_color_table` 이 거짓이면 실문서의 마스크처럼 `BITMAPINFOHEADER` 바로 뒤에 픽셀이 온다.
/// 참이면 같은 비트 앞에 흑백 색상표 2항목을 둔다.
fn mask_stretch_blt(with_color_table: bool) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&u32le(SRCPAINT));
    for v in [8i16, 8, 0, 0] {
        p.extend_from_slice(&i16le(v)); // SrcHeight, SrcWidth, YSrc, XSrc
    }
    for v in [40i16, 40, 10, 10] {
        p.extend_from_slice(&i16le(v)); // DestHeight, DestWidth, YDest, XDest
    }
    p.extend_from_slice(&u32le(40)); // BITMAPINFOHEADER
    p.extend_from_slice(&i32le(8));
    p.extend_from_slice(&i32le(8));
    p.extend_from_slice(&u16le(1)); // planes
    p.extend_from_slice(&u16le(1)); // 1비트
    p.extend_from_slice(&u32le(0)); // BI_RGB
    p.extend_from_slice(&u32le(32)); // ImageSize
    p.extend_from_slice(&i32le(0));
    p.extend_from_slice(&i32le(0));
    p.extend_from_slice(&u32le(0)); // ColorUsed
    p.extend_from_slice(&u32le(0));
    if with_color_table {
        p.extend_from_slice(&[0, 0, 0, 0]); // 0 = 검정
        p.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0]); // 1 = 흰색
    }
    for row in 0..8u8 {
        let bits = if row < 4 { 0xF0 } else { 0x0F };
        p.extend_from_slice(&[bits, 0, 0, 0]); // 8×8 1비트, 4바이트 정렬
    }
    record(0x0B41, &p)
}

/// `META_CREATEBRUSHINDIRECT` — 단색 브러시.
fn solid_brush(rgb: (u8, u8, u8)) -> Vec<u8> {
    let mut p = Vec::new();
    p.extend_from_slice(&u16le(0)); // BS_SOLID
    p.extend_from_slice(&[rgb.0, rgb.1, rgb.2, 0]);
    p.extend_from_slice(&u16le(0)); // hatch
    record(0x02FC, &p)
}

fn select_object(index: u16) -> Vec<u8> {
    record(0x012D, &u16le(index))
}

/// `META_RECTANGLE` — Bottom, Right, Top, Left.
fn rectangle() -> Vec<u8> {
    let mut p = Vec::new();
    for v in [90i16, 90, 60, 60] {
        p.extend_from_slice(&i16le(v));
    }
    record(0x041B, &p)
}

fn eof() -> Vec<u8> {
    record(0x0000, &[])
}

/// 마스크 `META_DIBSTRETCHBLT` 뒤에 단색 사각형이 오는 WMF.
fn mask_then_rectangle(with_color_table: bool) -> Vec<u8> {
    let mut b = header();
    for part in [
        mask_stretch_blt(with_color_table),
        solid_brush(TAIL_FILL),
        select_object(0),
        rectangle(),
        eof(),
    ] {
        b.extend_from_slice(&part);
    }
    b
}

fn to_svg(bytes: &[u8], what: &str) -> String {
    let bytes = WMFConverter::new(bytes, SVGPlayer::new())
        .run()
        .unwrap_or_else(|e| panic!("{what}: WMF 변환은 성공해야 한다: {e:?}"));
    String::from_utf8(bytes).expect("SVG 는 UTF-8 이다")
}

fn has_tail_fill(svg: &str) -> bool {
    let (r, g, b) = TAIL_FILL;
    let lower = svg.to_ascii_lowercase();
    lower.contains(&format!("#{r:02x}{g:02x}{b:02x}"))
        || lower
            .replace(' ', "")
            .contains(&format!("rgb({r},{g},{b})"))
}

#[test]
fn one_bit_dib_without_color_table_keeps_the_record_boundary() {
    // 대조군: 색상표를 명시한 같은 비트. 기존 해석이 그대로여야 한다.
    let explicit = to_svg(&mask_then_rectangle(true), "색상표 있는 1비트 DIB");
    assert!(
        has_tail_fill(&explicit),
        "색상표 있는 마스크 뒤의 사각형이 그려져야 한다:\n{explicit}"
    );

    // 색상표 없는 마스크: 레코드에 남은 크기가 헤더 + 픽셀뿐이므로 색상표는 0항목이다.
    let bare = to_svg(&mask_then_rectangle(false), "색상표 없는 1비트 DIB");
    assert!(
        has_tail_fill(&bare),
        "색상표 없는 마스크 뒤의 레코드도 변환돼야 한다:\n{bare}"
    );
    assert_eq!(
        bare, explicit,
        "색상표 없는 1비트 DIB 는 흑백 색상표(0 = 검정, 1 = 흰색)를 명시한 것과 같게 그려야 한다"
    );
}

#[test]
fn document_wmf_with_bare_mask_dibs_is_converted_on_every_page() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("fixture 누락 {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "전체 쪽수가 한컴 PDF 와 다르다"
    );
    let mut wmf_pages = 0;
    for page in 0..PDF_PAGES {
        let svg = core
            .render_page_svg_native(page)
            .unwrap_or_else(|e| panic!("{}쪽 SVG 렌더 실패: {e:?}", page + 1));
        assert!(
            !svg.contains("image/x-wmf"),
            "{}쪽에 변환하지 못한 WMF 원본이 남았다",
            page + 1
        );
        if svg.contains("image/svg+xml") {
            wmf_pages += 1;
        }
    }
    assert!(
        wmf_pages > 0,
        "변환된 WMF 그림(SVG 이미지)이 한 쪽에도 없으면 이 검사는 아무것도 확인하지 못한다"
    );
}
