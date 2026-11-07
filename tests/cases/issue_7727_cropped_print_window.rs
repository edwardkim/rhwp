//! 관계형 4개: 창의 면적 기여·평행 이동·캐시 신원·범위 검증. 실물 절대 좌표 핀은 없다.
//! 주석 갱신: 2026-11-07 11:10:33 KST.
//! 선행 동일 연산 출력: CBTA48쪽 Native 99.92121% / fresh WASM 99.92877%.
//! 원본 전204쪽 최저는 Native 96.23471% / WASM 96.23239%; 문서98% 완료 아님.
//! 생성 RGB의 원본 면적을 비교한다. runtime leaf를 직접 포함하며 복제 구현이나 원문 fixture를 만들지 않는다.
//! 근거: mydocs/pr/assets/issue7727/cropped-print-window-20261107/README.md.
#![cfg(not(target_arch = "wasm32"))]
// 절대 저장소 경로로 실제 연산을 포함하므로 자동 suite 재배정에도 상대 경로가 깨지지 않는다.
mod shared_crop {
    include!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/renderer/bitmap_raster/cropped_window.rs"
    ));
}
use shared_crop::{opaque_window_downsample, prepare};
use std::io::Cursor;
#[test]
fn issue_7727_fractional_source_window_keeps_the_independent_area_contribution() {
    let mut source = image::RgbImage::from_pixel(3, 2, image::Rgb([255; 3]));
    for y in 0..2 {
        source.put_pixel(0, y, image::Rgb([0; 3]));
    }
    let result = opaque_window_downsample(&source, 1, 1, [0.5, 0.0, 1.0, 2.0]);
    let expected = ((f64::from(source.get_pixel(0, 0)[0]) + f64::from(source.get_pixel(1, 0)[0]))
        / 2.0)
        .round() as u8;
    assert_eq!(result.get_pixel(0, 0).0, [expected; 3]);
}
#[test]
fn issue_7727_moving_the_same_source_window_preserves_its_samples() {
    let source = image::RgbImage::from_fn(12, 12, |x, y| {
        image::Rgb([(x * 17) as u8, (y * 19) as u8, ((x + y) * 9) as u8])
    });
    let mut shifted = image::RgbImage::from_pixel(15, 14, image::Rgb([255, 0, 255]));
    image::imageops::replace(&mut shifted, &source, 3, 2);
    assert_eq!(
        opaque_window_downsample(&source, 2, 2, [2.25, 1.5, 4.0, 4.0]),
        opaque_window_downsample(&shifted, 2, 2, [5.25, 3.5, 4.0, 4.0])
    );
}
fn encoded_halves() -> Vec<u8> {
    let source = image::RgbImage::from_fn(16, 16, |x, _| {
        image::Rgb(if x < 8 { [0; 3] } else { [255; 3] })
    });
    let mut output = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(source)
        .write_to(&mut output, image::ImageFormat::Png)
        .unwrap();
    output.into_inner()
}
#[test]
fn issue_7727_cache_keeps_different_windows_of_one_bitmap_distinct() {
    let bytes = encoded_halves();
    let left = prepare(&bytes, 2, 2, [0.2, 0.4, 1.0, 1.0], [2.0, 2.0, 4.0, 4.0]).unwrap();
    let right = prepare(&bytes, 2, 2, [0.2, 0.4, 1.0, 1.0], [10.0, 2.0, 4.0, 4.0]).unwrap();
    assert_ne!(left, right);
    assert!(image::load_from_memory(left.as_slice())
        .unwrap()
        .to_rgb8()
        .pixels()
        .all(|p| p.0 == [0; 3]));
    assert!(image::load_from_memory(right.as_slice())
        .unwrap()
        .to_rgb8()
        .pixels()
        .all(|p| p.0 == [255; 3]));
}
#[test]
fn issue_7727_invalid_source_windows_are_not_prepared() {
    let bytes = encoded_halves();
    for window in [
        [-1.0, 2.0, 4.0, 4.0],
        [14.0, 2.0, 4.0, 4.0],
        [f64::NAN, 2.0, 4.0, 4.0],
    ] {
        assert!(prepare(&bytes, 2, 2, [0.0, 0.0, 1.0, 1.0], window).is_none());
    }
    assert!(prepare(&bytes, 0, 2, [0.0, 0.0, 1.0, 1.0], [2.0, 2.0, 4.0, 4.0]).is_none());
}
