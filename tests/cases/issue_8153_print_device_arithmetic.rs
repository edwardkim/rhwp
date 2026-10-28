//! 관계형: 독립 Print 용지 관측과 공개 표본 계산을 실제 제품 함수에서 검사한다.
//! 편람 실측 Native 2026-10-28 / fresh WASM 2026-10-29: 96dpi, 한컴 2024 Print PDFium, 2px raw 실루엣.
//! 이 검사는 합성 입력을 사용하며 아래 수치는 관련 편람의 별도 시각 검증이다.
//! HWP 163쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWP 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! HWPX 163쪽 Native 100.00000% / fresh WASM 100.00000%.
//! HWPX 전 383쪽 최저 Native 99.96463% / fresh WASM 99.96402%.
//! 근거: mydocs/pr/assets/issue_8153/handbook-validation-20261028/README.md.
//! 제품 소스의 cfg(test) 증설 없이 정식 suite 자동 수집 경로를 사용한다.
#[allow(dead_code)]
#[path = "../../src/paint/print_paper.rs"]
mod print_paper;
#[path = "../../src/renderer/print_picture_pen.rs"]
mod print_picture_pen;
#[allow(dead_code)]
#[path = "../../src/renderer/print_image_sampling_core.rs"]
mod print_sampling;

use print_paper::{reference_geometry, PrintPaper};

#[test]
fn independent_picture_band_keeps_physical_width_and_raster_outline_separate() {
    // 독립 Print 163쪽: 채운 띠 0.36pt와 외곽 pen 0.12pt.
    // 96/192dpi에서는 외곽 pen만 출력 격자의 한 픽셀로 넓어진다.
    // 600dpi에서는 원래 0.12pt를 유지한다. 띠 자체는 확대하지 않는다.
    for (dpi, expected_css) in [(96.0, 1.48), (192.0, 0.98), (600.0, 0.64)] {
        let actual = print_picture_pen::raster_width([0.48, 0.16], Some(96.0 / dpi));
        assert!((actual.unwrap() - expected_css).abs() < 1e-12);
    }
}

#[test]
fn picture_pen_rejects_unusable_device_transform_and_invalid_source_widths() {
    assert_eq!(print_picture_pen::raster_width([0.48, 0.16], None), None);
    for invalid in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            print_picture_pen::raster_width([invalid, 0.16], Some(1.0)),
            None
        );
        assert_eq!(
            print_picture_pen::raster_width([0.48, invalid], Some(1.0)),
            None
        );
        assert_eq!(
            print_picture_pen::raster_width([0.48, 0.16], Some(invalid)),
            None
        );
    }
    assert_eq!(
        print_picture_pen::raster_width([f64::MAX, f64::MAX], Some(1.0)),
        None
    );
}

#[test]
fn independent_print_paper_and_integer_device_origins() {
    // 별도 한컴 Print 4개: 용지 높이와 원점 6000/15000/30000 HU.
    // PDF 연산자의 0.001pt 정밀도로 비교하며 엔진 출력에서 기대값을 얻지 않는다.
    for (hu, points, ys) in [
        (75400, 754, [60.003, 150.008, 300.016]),
        (75408, 754, [59.994, 149.985, 299.971]),
        (75420, 753, [59.906, 149.766, 299.531]),
        (75500, 754, [59.918, 149.795, 299.589]),
    ] {
        let paper =
            PrintPaper::from_css(55559.0 * 96.0 / 7200.0, hu as f64 * 96.0 / 7200.0).unwrap();
        assert_eq!(paper.size.1 * 0.75, points as f64);
        for (device_y, expected_pt) in [500.0, 1250.0, 2500.0].into_iter().zip(ys) {
            assert!((device_y * paper.device_units.1 * 0.75 - expected_pt).abs() < 0.001);
        }
    }
}

#[test]
fn reference_dc_half_millimeter_boundaries() {
    // 표준 GDI GetDeviceCaps, Hancom PDF, 메모리상의 DEVMODE에서 확인했다.
    // 장치 픽셀에서 mm를 역산하면 반 mm 경계에서 이 독립 관측과 다르다.
    for (dm, pixels, mm) in [
        (1954, 4616, 195),
        (1955, 4618, 196),
        (1956, 4620, 196),
        (1964, 4639, 196),
        (1965, 4642, 197),
        (1966, 4644, 197),
        (2654, 6269, 265),
        (2655, 6272, 266),
        (2656, 6274, 266),
        (2664, 6293, 266),
        (2665, 6295, 267),
        (2666, 6298, 267),
    ] {
        assert_eq!(reference_geometry(dm), Some((pixels, mm)));
    }
}

#[test]
fn only_valid_source_hu_paper_is_projected() {
    let paper = PrintPaper::from_css(55559.0 * 96.0 / 7200.0, 75402.0 * 96.0 / 7200.0).unwrap();
    assert_eq!(paper.size.0 * 0.75, 555.0);
    assert_eq!(paper.size.1 * 0.75, 754.0);
    assert!((paper.device_units.1 * 0.75 - 0.11998832302863824).abs() < 1e-15);
    for css in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::MAX, 1.23456789] {
        assert!(PrintPaper::from_css(css, 100.0).is_none());
    }
}

use print_sampling::resample_opaque_rgba;

fn gray(values: &[u8]) -> Vec<u8> {
    values.iter().flat_map(|v| [*v, *v, *v, 255]).collect()
}

#[test]
fn print_upscale_uses_pixel_centers_and_clips_edge_neighbors() {
    // 2→4의 표본 중심은 0.25·0.75·1.25·1.75다. 가장자리 이웃을
    // 자른 뒤 내부 두 표본은 각각 3/4:1/4와 1/4:3/4로 결합한다.
    let source = gray(&[0, 100, 0, 100]);
    assert_eq!(
        resample_opaque_rgba(&source, (2, 2), (4, 2)),
        Some(gray(&[0, 25, 75, 100, 0, 25, 75, 100]))
    );
}

#[test]
fn print_mixed_axes_keep_area_downscale_and_center_upscale() {
    let source = gray(&[0, 100, 40, 140, 80, 180, 120, 220]);
    // 세로 4→2는 각 두 행의 면적 평균, 가로 2→4는 위 중심 보간이다.
    assert_eq!(
        resample_opaque_rgba(&source, (2, 4), (4, 2)),
        Some(gray(&[20, 45, 95, 120, 100, 125, 175, 200]))
    );
}

#[test]
fn print_large_upscale_keeps_public_nearest_sample_policy() {
    let source = gray(&[0, 80, 160, 240]);
    let actual = resample_opaque_rgba(&source, (2, 2), (16, 16)).unwrap();
    for y in 0..16 {
        for x in 0..16 {
            assert_eq!(
                actual[(y * 16 + x) * 4],
                [0, 80, 160, 240][y / 8 * 2 + x / 8]
            );
        }
    }
}

#[test]
fn print_resampling_rejects_unsupported_alpha_and_oversized_output() {
    assert!(resample_opaque_rgba(&[0, 0, 0, 128], (1, 1), (2, 2)).is_none());
    assert!(resample_opaque_rgba(&gray(&[0]), (1, 1), (8193, 1)).is_none());
    assert!(resample_opaque_rgba(&gray(&[0]), (1, 1), (1, 0)).is_none());
}
