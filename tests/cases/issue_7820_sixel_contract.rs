//! 인코더/CLI 계약. 실물 조판 golden을 추가하거나 한컴 fidelity를 주장하지 않는다.
use rhwp::renderer::layer_renderer::RasterColorSpace;
use rhwp::renderer::raster_pixels::{RasterPixelFormat, RasterPixels};

fn pixels(bytes: Vec<u8>, width: i32, height: i32, stride: usize) -> RasterPixels {
    RasterPixels {
        bytes,
        width,
        height,
        stride,
        format: RasterPixelFormat::Rgba8Premultiplied,
        color_space: RasterColorSpace::Srgb,
        dpi: None,
    }
}

#[test]
fn padded_rows_and_premultiplied_alpha_compose_on_white() {
    // 반투명 빨강과 초록의 원 RGBA는 (255,0,0,128), (0,255,0,64)다.
    // 패딩은 검은 픽셀로 해석하거나 다음 행으로 흘려보내지 않는다.
    let source = pixels(
        vec![
            128, 0, 0, 128, 0, 0, 0, 0, 42, 42, 42, 42, 0, 64, 0, 64, 0, 0, 255, 255, 42, 42, 42,
            42,
        ],
        2,
        2,
        12,
    );
    assert_eq!(
        source.opaque_rgba_on_white().unwrap(),
        [255, 127, 127, 255, 255, 255, 255, 255, 191, 255, 191, 255, 0, 0, 255, 255]
    );
}

#[test]
fn rejects_invalid_stride_buffer_dimensions_and_premultiplication() {
    for invalid in [
        pixels(vec![], 0, 1, 0),
        pixels(vec![], 1, -1, 4),
        pixels(vec![0; 3], 1, 1, 3),
        pixels(vec![0; 7], 1, 2, 4),
        pixels(vec![1, 0, 0, 0], 1, 1, 4),
        pixels(vec![], 1, 2, usize::MAX),
    ] {
        assert!(invalid.opaque_rgba_on_white().is_err());
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
#[test]
fn png_preserves_straight_alpha_and_explicit_rgba_order() {
    let source = pixels(vec![128, 0, 0, 128, 0, 0, 0, 0, 0, 0, 255, 255], 3, 1, 12);
    let output = rhwp::renderer::skia::SkiaLayerRenderer::encode_pixels_png(&source).unwrap();
    let image = image::load_from_memory(&output.bytes).unwrap().to_rgba8();
    assert_eq!(image.dimensions(), (3, 1));
    assert_eq!(
        image.as_raw(),
        &[255, 0, 0, 128, 0, 0, 0, 0, 0, 0, 255, 255]
    );
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
#[test]
fn target_width_preserves_ratio_and_checks_both_dimensions_and_pixel_budget() {
    use rhwp::paint::{LayerNode, PageLayerTree};
    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    use rhwp::renderer::render_tree::BoundingBox;
    use rhwp::renderer::skia::SkiaLayerRenderer;
    // 독립 기하 계약: 3:4 용지를 너비 1픽셀로 읽으면 높이는 ceil(4/3)=2다.
    let tree = PageLayerTree::new(
        3.0,
        4.0,
        LayerNode::leaf(BoundingBox::new(0.0, 0.0, 3.0, 4.0), None, vec![]),
    );
    let renderer = SkiaLayerRenderer::new();
    let raster = renderer
        .render_pixels_at_width(&tree, 1, Default::default())
        .unwrap();
    assert_eq!((raster.width, raster.height), (1, 2));
    assert_eq!(raster.opaque_rgba_on_white().unwrap(), [255; 8]);
    for width in [0, 16385, u32::MAX] {
        assert!(renderer
            .render_pixels_at_width(&tree, width, Default::default())
            .is_err());
    }
    for options in [
        RasterRenderOptions {
            max_pixels: 1,
            ..Default::default()
        },
        RasterRenderOptions {
            max_dimension: 1,
            ..Default::default()
        },
    ] {
        assert!(renderer.render_pixels_at_width(&tree, 1, options).is_err());
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]
#[test]
fn sixel_has_only_one_dcs_stream_with_square_pixels() {
    let source = pixels(
        vec![255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255],
        3,
        1,
        12,
    );
    let stream = rhwp::renderer::sixel::encode_sixel(&source).unwrap();
    assert!(stream.starts_with(b"\x1bP"));
    assert!(stream.ends_with(b"\x1b\\"));
    assert_eq!(stream.iter().filter(|byte| **byte == 27).count(), 2);
    assert!(std::str::from_utf8(&stream).unwrap().contains("\"1;1;3;1"));
    let decoded = icy_sixel::SixelImage::decode(&stream).unwrap();
    // 자체 decoder는 마지막 밴드의 빈 줄도 할당한다. 크기의 정답지는
    // raster header와 독립 libsixel 복원이며, 여기서는 색·내용과 빈 패딩을 검사한다.
    assert_eq!(decoded.width, 3);
    assert!(decoded.aspect_ratio.is_square());
    let (content, padding) = decoded.pixels.split_at(source.bytes.len());
    assert_eq!(content, source.bytes);
    assert!(padding.iter().all(|byte| *byte == 0));
}

#[cfg(not(target_arch = "wasm32"))]
fn run(args: &[&str]) -> std::process::Output {
    let bin = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string());
    std::process::Command::new(bin).args(args).output().unwrap()
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn cli_usage_errors_never_emit_graphics_or_logs_to_stdout() {
    for args in [
        vec!["export-sixel"],
        vec!["export-sixel", "--width", "0"],
        vec!["export-sixel", "--width", "16385"],
        vec!["export-sixel", "--width", "NaN"],
        vec!["export-sixel", "--page", "-1"],
        vec!["export-sixel", "--page"],
        vec!["export-sixel", "--output"],
        vec!["export-sixel", "-o"],
        vec!["export-sixel", "--json"],
        vec!["export-sixel", "a.hwp", "b.hwp"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert!(!output.stderr.is_empty(), "{args:?}");
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn cli_help_and_capabilities_declare_feature_and_options() {
    let output = run(&["export-sixel", "--help"]);
    assert!(output.status.success());
    let help = String::from_utf8(output.stdout).unwrap();
    for expected in ["--width", "--page", "--output", "--font-path", "stdout"] {
        assert!(help.contains(expected), "missing {expected}");
    }
    let output = run(&["capabilities"]);
    let envelope: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let entry = envelope["commands"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| entry["name"] == "export-sixel")
        .unwrap();
    assert_eq!(entry["requiresFeature"], "native-skia");
    assert_eq!(entry["available"], cfg!(feature = "native-skia"));
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "native-skia")))]
#[test]
fn missing_native_feature_is_an_explicit_nonzero_error() {
    let output = run(&["export-sixel", "unused.hwp"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("native-skia"));
}
