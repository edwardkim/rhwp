//! 검사 유형: 관계형. 합성 표본의 mip 선택·알파·경계 계약을 Native Skia와 대조한다.
//! 주석 갱신: 2026-11-08T23:28:54+09:00 (Asia/Seoul).
//! 합성 표본에는 실물 PDF 일치율·절대 문서 px 위치가 적용되지 않는다.
//! 실물 후속 검증: sample16 HWP2024/HWPX 18·35쪽을 같은 독립 Print로 재출력한다.
//! 근거: mydocs/pr/assets/planet6897_20261106/native-weight-r46g-analysis.md.
#![cfg(feature = "native-skia")]

// 생산 모듈을 직접 실행하되 테스트 때문에 private API를 공개하지 않는다.
#[path = "../../src/renderer/mipmap_sampling.rs"]
mod mipmap_sampling;

use skia_safe::{
    images, surfaces, AlphaType, Color, ColorType, Data, FilterMode, ImageInfo, MipmapMode, Paint,
    Rect, SamplingOptions,
};

fn checkerboard(width: u32, height: u32) -> Vec<u8> {
    (0..height)
        .flat_map(|y| {
            (0..width).flat_map(move |x| {
                let value = if (x + y).is_multiple_of(2) { 0 } else { 255 };
                [value, value, value, 255]
            })
        })
        .collect()
}

fn host_image(pixels: &[u8], width: u32, height: u32) -> skia_safe::Image {
    let info = ImageInfo::new(
        (width as i32, height as i32),
        ColorType::RGBA8888,
        AlphaType::Unpremul,
        None,
    );
    images::raster_from_data(&info, Data::new_copy(pixels), width as usize * 4).unwrap()
}

fn reference(image: &skia_safe::Image, rectangle: (f64, f64, f64, f64), mips: bool) -> Vec<u8> {
    let mut surface = surfaces::raster_n32_premul((64, 64)).unwrap();
    surface.canvas().clear(Color::WHITE);
    let with_mips = mips.then(|| image.with_default_mipmaps().unwrap());
    let (x, y, width, height) = rectangle;
    surface.canvas().draw_image_rect_with_sampling_options(
        with_mips.as_ref().unwrap_or(image),
        None,
        Rect::from_xywh(x as f32, y as f32, width as f32, height as f32),
        SamplingOptions::new(
            FilterMode::Linear,
            if mips {
                MipmapMode::Linear
            } else {
                MipmapMode::None
            },
        ),
        &Paint::default(),
    );
    pixels(&mut surface)
}

fn pixels(surface: &mut skia_safe::Surface) -> Vec<u8> {
    let info = ImageInfo::new((64, 64), ColorType::RGBA8888, AlphaType::Unpremul, None);
    let mut rgba = vec![0; 64 * 64 * 4];
    assert!(surface.read_pixels(&info, &mut rgba, 64 * 4, (0, 0)));
    rgba
}

#[test]
fn odd_and_even_pyramids_match_host_mipmap_instead_of_aliasing_single_level() {
    for size in [(128, 64), (127, 65), (129, 129)] {
        let source = checkerboard(size.0, size.1);
        let rectangle = (5.25, 7.125, 23.5, 19.75);
        let image = host_image(&source, size.0, size.1);
        let expected = reference(&image, rectangle, true);
        let aliased = reference(&image, rectangle, false);
        let output = mipmap_sampling::rasterize(&source, size, rectangle).unwrap();
        let image = host_image(&output.rgba, output.width, output.height);
        let mut surface = surfaces::raster_n32_premul((64, 64)).unwrap();
        surface.canvas().clear(Color::WHITE);
        surface
            .canvas()
            .draw_image(&image, (output.left as f32, output.top as f32), None);
        let actual = pixels(&mut surface);
        let mut portable_error = 0u64;
        let mut single_level_error = 0u64;
        // 소수 원점의 외곽 coverage와 별도로 내부 축소 표본의 소유를 비교한다.
        for y in 10..24 {
            for x in 8..26 {
                for c in 0..3 {
                    let index = (y * 64 + x) * 4 + c;
                    portable_error += (i32::from(actual[index]) - i32::from(expected[index]))
                        .unsigned_abs() as u64;
                    single_level_error += (i32::from(aliased[index]) - i32::from(expected[index]))
                        .unsigned_abs() as u64;
                }
            }
        }
        assert!(portable_error * 20 < single_level_error,
            "{size:?}: 실제 mip 기준과 단일 레벨을 구별해야 한다: {portable_error} / {single_level_error}");
    }
}

#[test]
fn transparent_rgb_cannot_leak_into_visible_mipmap_samples() {
    let source = (0..64 * 64)
        .flat_map(|i| {
            if i % 2 == 0 {
                [255, 0, 255, 0]
            } else {
                [0, 192, 0, 255]
            }
        })
        .collect::<Vec<_>>();
    let output = mipmap_sampling::rasterize(&source, (64, 64), (0.0, 0.0, 8.0, 8.0)).unwrap();
    assert!(output
        .rgba
        .chunks_exact(4)
        .all(|pixel| pixel[0] == 0 && pixel[2] == 0 && pixel[1] >= 128 && pixel[3] > 0));
}

#[test]
fn source_and_device_limits_preserve_the_existing_fallback_boundary() {
    let source = checkerboard(8, 8);
    for rectangle in [
        (0.0, 0.0, 8.0, 8.0),
        (0.0, 0.0, 16.0, 16.0),
        (f64::NAN, 0.0, 1.0, 1.0),
        (1e30, 0.0, 1.0, 1.0),
        (0.0, 0.0, 0.0, 1.0),
    ] {
        assert!(mipmap_sampling::rasterize(&source, (8, 8), rectangle).is_none());
    }
    assert!(mipmap_sampling::rasterize(&source[..3], (8, 8), (0.0, 0.0, 1.0, 1.0)).is_none());
    assert!(mipmap_sampling::rasterize(&[], (u32::MAX, u32::MAX), (0.0, 0.0, 1.0, 1.0)).is_none());
}
