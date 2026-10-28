//! [#6310] 칸 배경 그림(`hc:imgBrush mode="ZOOM"`)을 칸에 맞춰 그리고, CMYK JPEG 을
//! PDF 에 그대로 싣지 않는다.
//!
//! `samples/issue6310/press_release_cell_logo.hwpx` 는 국가데이터처
//! 「2025년 출생·사망통계(잠정)」(156745900, 6.99MB)의 **구조 보존 슬라이스**다 —
//! 시험 대상인 `BinData/image1.jpg`(로고, 4성분 CMYK JPEG)만 원본 그대로 두고
//! 나머지 BinData 를 8×8 불투명 그림으로 바꿔 0.68MB 로 줄였다.
//!
//! **증상.** 1쪽 `보도자료` 칸 왼쪽의 국가데이터처 로고가 안 보인다.
//!
//! **결함이 사슬로 둘이다.**
//!
//! **① HWPX `mode="ZOOM"` 매핑 부재.** `parser/hwpx/header.rs` 의 `imgBrush` mode
//! 표에 `"ZOOM"` 갈래가 없어 `_ => ImageFillMode::TileAll` 로 떨어졌다. TileAll 은
//! **원본 픽셀 크기**로 타일링하므로 이슈가 관측한 8.33배가 된다
//! (원본 1211×355 → 908.25 × 266.25pt, 칸 clip 안에는 흰 여백 한 조각만 남는다).
//!
//! **② 4성분 CMYK/YCCK JPEG 을 `/DeviceRGB` 로 선언.** ①만 고치면 배치는 맞지만
//! 그림이 여전히 깨진다. 이 로고는 `APP14 Adobe transform=2`, SOF0 **성분 4개**인데
//! PDF `DCTDecode` 는 성분 수를 스트림이 아니라 `/ColorSpace` 선언에서 가져간다.
//! 3성분으로 읽히니 행 보폭이 어긋나 같은 그림이 가로로 반복되고 색이 번진다.
//! 한컴은 같은 그림을 3성분 RGB 로 다시 인코딩해 내보낸다(616KB → 14KB).
//!
//! **오라클 — 한글 2022** (`appVersion 12` = 한글 2022, 설치본 존재,
//! `producer=Hancom PDF 1.3.0.547`).
//!
//! | | 배치 | 종횡비 |
//! |---|---|---|
//! | 종전 rhwp | 908.25 × 266.25pt (8.33배) | 유지 |
//! | **수정 후** | **102.17 × 29.95pt** | 3.411 |
//! | 한글 2022 | 109.02 × 32.01pt | 3.406 |
//!
//! 남는 6% 크기 차와 세로 오프셋은 칸 기하 축이라 이 테스트는 **배율 상한**만 잠근다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

fn document() -> rhwp::model::document::Document {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6310/press_release_cell_logo.hwpx");
    let bytes = std::fs::read(path).expect("재현물이 있다");
    rhwp::parser::parse_document(&bytes).expect("문서를 연다")
}

/// HWPX `mode="ZOOM"` 이 `TileAll` 로 떨어지지 않는다.
///
/// [#6310] ① 축은 `#6473`(kevin9327)이 devel 에 먼저 넣었다. 이 테스트는 그 계약이
/// 유지되는지 함께 지킨다 — 이 PR 이 고치는 것은 ② CMYK JPEG 축이다.
#[test]
fn hwpx_zoom_mode_is_not_parsed_as_tile() {
    use rhwp::model::style::{FillType, ImageFillMode};

    let doc = document();
    let zoom_fills = doc
        .doc_info
        .border_fills
        .iter()
        .filter(|bf| bf.fill.fill_type == FillType::Image)
        .filter_map(|bf| bf.fill.image.as_ref())
        .filter(|img| img.fill_mode == ImageFillMode::Zoom)
        .count();
    assert!(
        zoom_fills > 0,
        "이 문서에는 `mode=\"ZOOM\"` 인 imgBrush 가 있어야 한다 — \
         종전에는 매핑이 없어 전부 TileAll 로 붕괴했다"
    );
}

/// 4성분 CMYK/YCCK JPEG 을 판별한다.
///
/// 이 판별이 거짓이면 CMYK JPEG 이 PDF `/DeviceRGB` 로 그대로 실려 행 보폭이 깨진다.
#[test]
fn four_component_jpeg_is_detected() {
    use rhwp::renderer::image_resolver::{cmyk_jpeg_bytes_to_png_bytes, jpeg_is_four_component};

    let doc = document();
    let blobs: Vec<Vec<u8>> = doc.bin_data_content.iter().map(|b| b.data.load()).collect();
    let logo: &[u8] = blobs
        .iter()
        .map(|d| d.as_slice())
        .find(|d: &&[u8]| d.len() > 100_000 && d.starts_with(&[0xFF, 0xD8]))
        .expect("원본 그대로 남긴 로고 JPEG 을 찾는다");

    assert!(
        jpeg_is_four_component(logo),
        "이 로고는 SOF0 성분 4개(CMYK/YCCK)여야 한다"
    );
    let png = cmyk_jpeg_bytes_to_png_bytes(logo).expect("PNG 로 정규화된다");
    assert!(
        png.starts_with(&[0x89, b'P', b'N', b'G']),
        "정규화 결과는 PNG 여야 한다"
    );

    // 3성분 JPEG 은 건드리지 않는다 — 정규화는 4성분에만 걸린다.
    let three = blobs
        .iter()
        .map(|d| d.as_slice())
        .find(|d: &&[u8]| d.starts_with(&[0xFF, 0xD8]) && !jpeg_is_four_component(d));
    assert!(
        three.is_some(),
        "대조군: 3성분 JPEG 은 4성분으로 판정되지 않는다"
    );
}

/// JPEG marker fill byte가 있어도 4성분 SOF0를 놓치지 않는다.
///
/// [maintainer correction] source-side unit baseline을 늘리지 않고 #6310 fixture contract 안에서
/// 판별기의 유효 JPEG 경계를 고정한다. JPEG은 SOF marker 앞에 0xFF fill byte를 반복할 수 있다.
#[test]
fn four_component_jpeg_detector_accepts_marker_fill_before_sof() {
    use rhwp::renderer::image_resolver::jpeg_is_four_component;

    // 4성분 SOF0: length 20 = 고정 필드 8 + component descriptor 4×3.
    let jpeg = [
        0xFF, 0xD8, // SOI
        0xFF, 0xFF, 0xC0, // marker fill + SOF0
        0x00, 0x14, // segment length
        0x08, 0x00, 0x01, 0x00, 0x01, 0x04, // precision, height, width, components
        0x01, 0x11, 0x00, 0x02, 0x11, 0x00, 0x03, 0x11, 0x00, 0x04, 0x11, 0x00, 0xFF, 0xD9,
    ];

    assert!(jpeg_is_four_component(&jpeg));
}
