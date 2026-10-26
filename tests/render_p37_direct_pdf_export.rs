#![cfg(all(not(target_arch = "wasm32"), feature = "native-skia"))]

use rhwp::error::HwpError;
use rhwp::renderer::pdf::DirectPdfExportOptions;
use rhwp::wasm_api::HwpDocument;
use std::path::{Path, PathBuf};
use std::process::Command;

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/re-03-latin-only-hancom.hwp")
}

fn sample_doc() -> HwpDocument {
    let path = sample_path();
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    HwpDocument::from_bytes(&bytes).expect("load direct PDF fixture")
}

fn assert_complete_pdf(bytes: &[u8]) {
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map(|index| index + 1)
        .unwrap_or(0);
    assert!(bytes.starts_with(b"%PDF-"), "missing PDF header");
    assert!(bytes[..end].ends_with(b"%%EOF"), "missing PDF trailer");
}

fn unique_pdf_path() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "rhwp-render-p37-{}-{nonce}.pdf",
        std::process::id()
    ))
}

#[test]
fn document_core_direct_pdf_exports_page_selection_and_document() {
    let doc = sample_doc();

    let page = doc
        .render_page_pdf_direct_native(0)
        .expect("direct page PDF");
    let mut options = DirectPdfExportOptions::default();
    options.raster_dpi = 96.0;
    options.title = Some("rhwp direct PDF integration".to_string());
    let selected = doc
        .render_pages_pdf_direct_native_with_options(&[0], &options)
        .expect("direct selected-page PDF");
    let document = doc
        .render_document_pdf_direct_native()
        .expect("direct document PDF");

    assert_complete_pdf(&page);
    assert_complete_pdf(&selected);
    assert_complete_pdf(&document);
    assert!(selected
        .windows(b"rhwp direct PDF integration".len())
        .any(|window| window == b"rhwp direct PDF integration"));
}

#[test]
fn document_core_direct_pdf_preserves_selection_errors() {
    let doc = sample_doc();

    let empty = doc
        .render_pages_pdf_direct_native(&[])
        .expect_err("empty page selection must fail");
    assert!(
        matches!(empty, HwpError::RenderError(ref message) if message.contains("at least one page")),
        "unexpected error: {empty:?}"
    );

    let out_of_range = doc
        .render_pages_pdf_direct_native(&[0, 99])
        .expect_err("out-of-range page must fail");
    assert!(
        matches!(out_of_range, HwpError::PageOutOfRange(99)),
        "unexpected error: {out_of_range:?}"
    );
}

#[test]
fn export_pdf_cli_selects_direct_backend_explicitly() {
    let output_path = unique_pdf_path();
    let output = Command::new(rhwp_bin())
        .arg("export-pdf")
        .arg(sample_path())
        .args(["--backend", "direct"])
        .args(["--raster-dpi", "96", "--output"])
        .arg(&output_path)
        .output()
        .expect("run direct PDF CLI");

    assert!(
        output.status.success(),
        "direct PDF CLI failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("PDF backend: direct"),
        "CLI did not report direct backend"
    );
    let pdf = std::fs::read(&output_path).expect("read direct CLI PDF");
    let _ = std::fs::remove_file(&output_path);
    assert_complete_pdf(&pdf);

    let explicit_print_path = unique_pdf_path();
    let explicit_print = Command::new(rhwp_bin())
        .arg("export-pdf")
        .arg(sample_path())
        .args(["--backend", "direct", "--profile", "print"])
        .args(["--raster-dpi", "96", "--output"])
        .arg(&explicit_print_path)
        .output()
        .expect("run explicit print direct PDF CLI");
    assert!(explicit_print.status.success());
    let explicit_print_pdf =
        std::fs::read(&explicit_print_path).expect("read explicit print direct PDF");
    let _ = std::fs::remove_file(&explicit_print_path);
    assert_eq!(pdf, explicit_print_pdf);
}

#[test]
fn export_pdf_cli_rejects_backend_specific_option_mismatches() {
    let direct_output_path = unique_pdf_path();
    let direct_with_svg_option = Command::new(rhwp_bin())
        .arg("export-pdf")
        .arg(sample_path())
        .args(["--backend", "direct", "--fallback-serif", "serif"])
        .arg("--output")
        .arg(&direct_output_path)
        .output()
        .expect("run direct PDF CLI with compatibility option");
    assert_eq!(direct_with_svg_option.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&direct_with_svg_option.stderr)
        .contains("SVG 호환 옵션을 지원하지 않습니다"));
    assert!(!direct_output_path.exists());

    let compatibility_output_path = unique_pdf_path();
    let svg_with_direct_option = Command::new(rhwp_bin())
        .arg("export-pdf")
        .arg(sample_path())
        .args(["--backend", "svg", "--raster-dpi", "96"])
        .arg("--output")
        .arg(&compatibility_output_path)
        .output()
        .expect("run compatibility PDF CLI with direct option");
    assert_eq!(svg_with_direct_option.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&svg_with_direct_option.stderr)
        .contains("direct PDF backend에서만"));
    assert!(!compatibility_output_path.exists());
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

// #7494·#7751: 독립 한컴 Print의 색상·기존 미지원 경계를 검사한다.
#[cfg(feature = "native-skia")]
mod image_adjustment_regressions {
    fn issue_4764_png_rgba(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
        let pixels = image::RgbaImage::from_pixel(width, height, image::Rgba(rgba));
        let mut bytes = std::io::Cursor::new(Vec::new());
        pixels
            .write_to(&mut bytes, image::ImageFormat::Png)
            .expect("encode test PNG");
        bytes.into_inner()
    }

    fn issue_4764_hancom_rgb(brightness: i8, contrast: i8) -> [u8; 3] {
        // RGB(80,100,120)을 독립 한컴 Print에서 관측한 값이다.
        // 출처·입력/PDF SHA·job은 pdf/issue7751/README.md에 고정한다.
        match (brightness, contrast) {
            (0, 0) => [80, 100, 120],
            (8, 0) => [100, 120, 140],
            (0, 8) => [76, 97, 119],
            (20, 0) => [131, 151, 171],
            (0, 20) => [68, 93, 118],
            (20, 20) => [125, 150, 175],
            _ => panic!("한컴 Print 관측표에 없는 조건: {brightness}/{contrast}"),
        }
    }

    struct Issue4764PdfImageStream {
        width: usize,
        height: usize,
        pixels: Vec<u8>,
    }

    fn issue_4764_rfind(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .rposition(|window| window == needle)
    }

    fn issue_4764_pdf_dict_usize(dict: &[u8], key: &str) -> Option<usize> {
        let dict = String::from_utf8_lossy(dict);
        let start = dict.find(key)? + key.len();
        dict[start..]
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<usize>().ok())
    }

    fn issue_4764_pdf_rgb_color_space(pdf: &[u8], dict: &[u8]) -> bool {
        let dict = String::from_utf8_lossy(dict);
        if dict.contains("/ColorSpace /DeviceRGB") {
            return true;
        }
        // Skia preserves an sRGB ICC profile on unfiltered PNG images.
        let Some((_, reference)) = dict.split_once("/ColorSpace [/ICCBased ") else {
            return false;
        };
        let Some(id) = reference
            .split_whitespace()
            .next()
            .and_then(|id| id.parse::<usize>().ok())
        else {
            return false;
        };
        let marker = format!("\n{id} 0 obj");
        let Some(start) = pdf
            .windows(marker.len())
            .position(|bytes| bytes == marker.as_bytes())
        else {
            return false;
        };
        let object = &pdf[start + marker.len()..];
        let Some(end) = object.windows(2).position(|bytes| bytes == b">>") else {
            return false;
        };
        issue_4764_pdf_dict_usize(&object[..end], "/N") == Some(3)
    }

    fn issue_4764_pdf_image_streams(pdf: &[u8]) -> Vec<Issue4764PdfImageStream> {
        use flate2::read::ZlibDecoder;
        use std::io::Read;
        let mut images = Vec::new();
        let mut cursor = 0;
        while let Some(relative_start) = pdf[cursor..]
            .windows(b"stream".len())
            .position(|window| window == b"stream")
        {
            let stream_marker = cursor + relative_start;
            let Some(dict_start) = issue_4764_rfind(&pdf[..stream_marker], b"<<") else {
                cursor = stream_marker + b"stream".len();
                continue;
            };
            let Some(dict_end) = issue_4764_rfind(&pdf[..stream_marker], b">>") else {
                cursor = stream_marker + b"stream".len();
                continue;
            };
            let dict = &pdf[dict_start..dict_end + 2];
            if !dict
                .windows(b"/Subtype /Image".len())
                .any(|window| window == b"/Subtype /Image")
                || !issue_4764_pdf_rgb_color_space(pdf, dict)
                || !dict
                    .windows(b"/BitsPerComponent 8".len())
                    .any(|window| window == b"/BitsPerComponent 8")
            {
                cursor = stream_marker + b"stream".len();
                continue;
            }
            let Some(width) = issue_4764_pdf_dict_usize(dict, "/Width") else {
                cursor = stream_marker + b"stream".len();
                continue;
            };
            let Some(height) = issue_4764_pdf_dict_usize(dict, "/Height") else {
                cursor = stream_marker + b"stream".len();
                continue;
            };
            let mut start = stream_marker + b"stream".len();
            if pdf.get(start..start + 2) == Some(b"\r\n") {
                start += 2;
            } else if pdf.get(start) == Some(&b'\n') {
                start += 1;
            }
            let Some(relative_end) = pdf[start..]
                .windows(b"endstream".len())
                .position(|window| window == b"endstream")
            else {
                break;
            };
            let end = start + relative_end;
            let raw = &pdf[start..end];
            let pixels = if dict
                .windows(b"/Filter /FlateDecode".len())
                .any(|window| window == b"/Filter /FlateDecode")
            {
                let mut decoded = Vec::new();
                if ZlibDecoder::new(raw).read_to_end(&mut decoded).is_err() {
                    cursor = end + b"endstream".len();
                    continue;
                }
                decoded
            } else {
                raw.to_vec()
            };
            if pixels.len() == width.saturating_mul(height).saturating_mul(3) {
                images.push(Issue4764PdfImageStream {
                    width,
                    height,
                    pixels,
                });
            }
            cursor = end + b"endstream".len();
        }
        images
    }

    fn issue_4764_pdf_images_contain_rgb(
        images: &[Issue4764PdfImageStream],
        expected: [u8; 3],
    ) -> bool {
        images.iter().any(|image| {
            image.width > 0
                && image.height > 0
                && image.pixels.chunks_exact(3).any(|rgb| {
                    rgb.iter()
                        .zip(expected)
                        .all(|(actual, expected)| actual.abs_diff(expected) <= 2)
                })
        })
    }

    fn issue_4764_direct_pdf_image_tree(
        image: rhwp::renderer::render_tree::ImageNode,
    ) -> rhwp::paint::PageLayerTree {
        use rhwp::paint::{LayerNode, PaintOp, RenderProfile};
        use rhwp::renderer::render_tree::BoundingBox;
        let page = BoundingBox::new(0.0, 0.0, 32.0, 32.0);
        rhwp::paint::PageLayerTree::with_profile(
            32.0,
            32.0,
            LayerNode::leaf(
                page,
                None,
                vec![PaintOp::image(
                    BoundingBox::new(8.0, 8.0, 16.0, 16.0),
                    image,
                    None,
                )],
            ),
            RenderProfile::Print,
        )
    }

    fn issue_4764_direct_pdf_page_background_tree(
        image: rhwp::renderer::render_tree::PageBackgroundImage,
    ) -> rhwp::paint::PageLayerTree {
        use rhwp::paint::{LayerNode, PaintOp, RenderProfile};
        use rhwp::renderer::render_tree::{BoundingBox, PageBackgroundNode};
        let page = BoundingBox::new(0.0, 0.0, 32.0, 32.0);
        rhwp::paint::PageLayerTree::with_profile(
            32.0,
            32.0,
            LayerNode::leaf(
                page,
                None,
                vec![PaintOp::page_background(
                    page,
                    PageBackgroundNode {
                        background_color: None,
                        border_color: None,
                        border_width: 0.0,
                        gradient: None,
                        image: Some(image),
                    },
                )],
            ),
            RenderProfile::Print,
        )
    }

    #[test]
    fn issue_4764_direct_pdf_embeds_adjusted_normal_image_pixels() {
        use rhwp::model::style::ImageFillMode;
        use rhwp::renderer::pdf::layer_trees_to_pdf;
        use rhwp::renderer::render_tree::ImageNode;
        let source = [80, 100, 120];
        for (brightness, contrast) in [(20, 0), (0, 20), (0, 8)] {
            let mut image = ImageNode::new(
                1,
                Some(issue_4764_png_rgba(
                    4,
                    4,
                    [source[0], source[1], source[2], 255],
                )),
            );
            image.fill_mode = Some(ImageFillMode::FitToSize);
            image.brightness = brightness;
            image.contrast = contrast;
            let pdf = layer_trees_to_pdf(&[issue_4764_direct_pdf_image_tree(image)])
                .expect("direct PDF export");
            let images = issue_4764_pdf_image_streams(&pdf);
            assert!(
                issue_4764_pdf_images_contain_rgb(
                    &images,
                    issue_4764_hancom_rgb(brightness, contrast)
                ),
                "PDF image streams should contain brightness/contrast adjusted pixels"
            );
            assert!(
                !issue_4764_pdf_images_contain_rgb(&images, source),
                "PDF image streams should not keep the unadjusted source pixels"
            );
        }
    }

    #[test]
    fn issue_4764_native_adjustments_preserve_alpha_and_object_opacity() {
        use rhwp::renderer::layer_renderer::RasterRenderOptions;
        use rhwp::renderer::render_tree::ImageNode;
        use rhwp::renderer::skia::SkiaLayerRenderer;
        for (brightness, contrast, opacity) in [
            (20, 0, 1.0),
            (0, 20, 1.0),
            (20, 20, 1.0),
            (0, 8, 0.5),
            (0, 0, 1.0),
        ] {
            let mut image = ImageNode::new(1, Some(issue_4764_png_rgba(4, 4, [80, 100, 120, 128])));
            image.brightness = brightness;
            image.contrast = contrast;
            image.opacity = opacity;
            let output = SkiaLayerRenderer::new()
                .render_raster_with_options(
                    &issue_4764_direct_pdf_image_tree(image),
                    RasterRenderOptions {
                        transparent: true,
                        ..Default::default()
                    },
                )
                .expect("transparent raster export");
            let rgba = image::load_from_memory(&output.bytes)
                .expect("decode PNG")
                .to_rgba8();
            let pixel = rgba.get_pixel(16, 16).0;
            let expected = issue_4764_hancom_rgb(brightness, contrast);
            assert!(
                pixel[..3]
                    .iter()
                    .zip(expected)
                    .all(|(actual, expected)| actual.abs_diff(expected) <= 3),
                "unexpected adjusted pixel: {pixel:?}, expected RGB: {expected:?}"
            );
            assert!(pixel[3].abs_diff((128.0 * opacity).round() as u8) <= 1);
            assert_eq!(rgba.get_pixel(0, 0).0[3], 0);
        }
    }

    #[test]
    fn issue_7751_native_contrast_endpoints_match_independent_print_pixels() {
        use rhwp::renderer::layer_renderer::RasterRenderOptions;
        use rhwp::renderer::render_tree::ImageNode;
        use rhwp::renderer::skia::SkiaLayerRenderer;
        // boundary Print의 실측값. 중간 범위의 식으로 기대값을 다시 계산하지 않는다.
        let cases = [
            ([80, 100, 120], -100, -100, [0, 0, 0]),
            ([80, 100, 120], -20, -100, [0, 0, 0]),
            ([80, 100, 120], 0, -100, [1, 1, 1]),
            ([80, 100, 120], 20, -100, [52, 52, 52]),
            ([80, 100, 120], 100, -100, [255, 255, 255]),
            ([80, 100, 120], -100, 100, [0, 0, 0]),
            ([80, 100, 120], -20, 100, [0, 0, 0]),
            ([80, 100, 120], 0, 100, [0, 0, 0]),
            ([80, 100, 120], 20, 100, [51, 51, 51]),
            ([80, 100, 120], 100, 100, [255, 255, 255]),
            ([128, 128, 128], 0, 100, [0, 0, 0]),
            ([170, 170, 170], 0, 100, [255, 255, 255]),
        ];
        for (source, brightness, contrast, expected) in cases {
            let mut image = ImageNode::new(
                1,
                Some(issue_4764_png_rgba(
                    4,
                    4,
                    [source[0], source[1], source[2], 255],
                )),
            );
            image.brightness = brightness;
            image.contrast = contrast;
            let output = SkiaLayerRenderer::new()
                .render_raster_with_options(
                    &issue_4764_direct_pdf_image_tree(image),
                    RasterRenderOptions {
                        transparent: true,
                        ..Default::default()
                    },
                )
                .expect("대비 끝값 Native PNG");
            let rgba = image::load_from_memory(&output.bytes)
                .expect("PNG")
                .to_rgba8();
            let pixel = rgba.get_pixel(16, 16).0;
            assert_eq!(
                &pixel[..3],
                &expected,
                "독립 Print 대비 끝값: 원본{source:?}, B{brightness}/C{contrast}"
            );
            assert_eq!(pixel[3], 255, "원본 alpha 보존");
        }
    }

    #[test]
    fn issue_4764_direct_pdf_applies_grayscale_before_hancom_encoded_contrast() {
        use rhwp::model::image::ImageEffect;
        use rhwp::renderer::pdf::layer_trees_to_pdf;
        use rhwp::renderer::render_tree::ImageNode;
        let mut image = ImageNode::new(1, Some(issue_4764_png_rgba(4, 4, [80, 100, 120, 255])));
        image.effect = ImageEffect::GrayScale;
        image.contrast = 50;
        let pdf = layer_trees_to_pdf(&[issue_4764_direct_pdf_image_tree(image)])
            .expect("grayscale then contrast PDF export");
        // 실제 한컴 Print의 회색조→대비50 출력은64다. 기존 SVG의
        // linearRGB·바깥 회색조 중첩 결과12를 한컴의 기대값으로 쓰지 않는다.
        assert!(issue_4764_pdf_images_contain_rgb(
            &issue_4764_pdf_image_streams(&pdf),
            [64, 64, 64]
        ));
    }

    #[test]
    fn issue_4764_direct_pdf_preserves_discrete_blackwhite_or_fails_closed() {
        use rhwp::model::image::ImageEffect;
        use rhwp::model::style::ImageFillMode;
        use rhwp::renderer::pdf::layer_trees_to_pdf;
        use rhwp::renderer::render_tree::{ImageNode, PageBackgroundImage};
        let mut image = ImageNode::new(1, Some(issue_4764_png_rgba(4, 4, [170, 170, 170, 255])));
        image.effect = ImageEffect::BlackWhite;
        image.brightness = 10;
        let background = PageBackgroundImage {
            data: issue_4764_png_rgba(4, 4, [170, 170, 170, 255]),
            fill_mode: ImageFillMode::FitToSize,
            // 쪽 배경은 저장 순서이므로 화면 밝기10은 contrast에 들어간다.
            brightness: 0,
            contrast: 10,
            effect: ImageEffect::BlackWhite,
        };
        for tree in [
            issue_4764_direct_pdf_image_tree(image),
            issue_4764_direct_pdf_page_background_tree(background),
        ] {
            match layer_trees_to_pdf(&[tree]) {
                Ok(pdf) => {
                    // SVG의 회색조 다음 discrete(0,1)는 밝기 적용 후 흰색이 된다.
                    // 고대비 행렬만 사용해 중간 회색을 내보내면 성공으로 처리하지 않는다.
                    assert!(issue_4764_pdf_images_contain_rgb(
                        &issue_4764_pdf_image_streams(&pdf),
                        [255, 255, 255]
                    ));
                }
                Err(error) => assert!(
                    error.contains("BlackWhite") && error.contains("use the svg backend"),
                    "unexpected error: {error}"
                ),
            }
        }
    }

    #[test]
    fn issue_4764_direct_pdf_embeds_adjusted_page_background_pixels_in_display_order() {
        use rhwp::model::image::ImageEffect;
        use rhwp::model::style::ImageFillMode;
        use rhwp::renderer::pdf::layer_trees_to_pdf;
        use rhwp::renderer::render_tree::PageBackgroundImage;
        // 배경의 기존 저장 IR은 뒤집혀 있으며 의미 투영 뒤 밝기8·대비0이다.
        // 같은 입력의 밝기0·대비8은 독립 Print에서도 다른 RGB가 나온다.
        let source = [80, 100, 120];
        let image = PageBackgroundImage {
            data: issue_4764_png_rgba(4, 4, [source[0], source[1], source[2], 255]),
            fill_mode: ImageFillMode::FitToSize,
            brightness: 0,
            contrast: 8,
            effect: ImageEffect::RealPic,
        };
        let pdf = layer_trees_to_pdf(&[issue_4764_direct_pdf_page_background_tree(image)])
            .expect("direct PDF export");
        let images = issue_4764_pdf_image_streams(&pdf);
        assert!(
            issue_4764_pdf_images_contain_rgb(&images, issue_4764_hancom_rgb(8, 0)),
            "page background must apply display brightness/contrast order"
        );
        assert!(
            !issue_4764_pdf_images_contain_rgb(&images, issue_4764_hancom_rgb(0, 8)),
            "page background must not use the raw stored brightness/contrast order"
        );
    }

    #[test]
    fn issue_4764_direct_pdf_still_rejects_pattern8x8_images() {
        use rhwp::model::image::ImageEffect;
        use rhwp::renderer::pdf::layer_trees_to_pdf;
        use rhwp::renderer::render_tree::ImageNode;
        let mut image = ImageNode::new(1, Some(issue_4764_png_rgba(4, 4, [80, 100, 120, 255])));
        image.effect = ImageEffect::Pattern8x8;
        image.brightness = 0;
        image.contrast = 20;
        let error = layer_trees_to_pdf(&[issue_4764_direct_pdf_image_tree(image)]).unwrap_err();
        assert!(
            error.contains("Pattern8x8 effect"),
            "unexpected error: {error}"
        );
        assert!(
            error.contains("use the svg backend"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn issue_4764_direct_pdf_still_rejects_unbaked_image_watermark_tone_or_opacity() {
        use rhwp::renderer::pdf::layer_trees_to_pdf;
        use rhwp::renderer::render_tree::ImageNode;
        for (brightness, contrast) in [(70, -50), (20, 20)] {
            let mut image = ImageNode::new(1, Some(issue_4764_png_rgba(4, 4, [80, 100, 120, 255])));
            image.brightness = brightness;
            image.contrast = contrast;
            assert!(image.is_watermark());
            let error = layer_trees_to_pdf(&[issue_4764_direct_pdf_image_tree(image)]).unwrap_err();
            assert!(
                error.contains("unbaked image watermark tone or opacity"),
                "unexpected error: {error}"
            );
        }
    }

    #[test]
    fn issue_4764_direct_pdf_still_rejects_unbaked_background_watermark_tone() {
        use rhwp::model::image::ImageEffect;
        use rhwp::model::style::ImageFillMode;
        use rhwp::renderer::pdf::layer_trees_to_pdf;
        use rhwp::renderer::render_tree::PageBackgroundImage;
        let image = PageBackgroundImage {
            data: issue_4764_png_rgba(4, 4, [80, 100, 120, 255]),
            fill_mode: ImageFillMode::FitToSize,
            brightness: -50,
            contrast: 70,
            effect: ImageEffect::RealPic,
        };
        assert!(image.is_real_picture_watermark_tone_preset());
        let error =
            layer_trees_to_pdf(&[issue_4764_direct_pdf_page_background_tree(image)]).unwrap_err();
        assert!(
            error.contains("unbaked RealPic watermark tone"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn issue_4764_direct_pdf_does_not_adjust_baked_watermark_pixels_twice() {
        use rhwp::paint::{LayerNodeKind, PaintOp, ResolvedImageKind, ResolvedImagePayload};
        use rhwp::renderer::pdf::layer_trees_to_pdf;
        use rhwp::renderer::render_tree::ImageNode;
        let baked = [80, 100, 120];
        let mut image = ImageNode::new(1, Some(issue_4764_png_rgba(4, 4, [20, 30, 40, 255])));
        image.brightness = 70;
        image.contrast = -50;
        let mut tree = issue_4764_direct_pdf_image_tree(image);
        let LayerNodeKind::Leaf { ops } = &mut tree.root.kind else {
            panic!("image leaf")
        };
        let PaintOp::Image { resolved, .. } = &mut ops[0] else {
            panic!("image paint op")
        };
        *resolved = Some(Box::new(ResolvedImagePayload {
            data: issue_4764_png_rgba(4, 4, [baked[0], baked[1], baked[2], 255]),
            mime: "image/png",
            kind: ResolvedImageKind::BakedWatermark,
            suppress_effects: true,
        }));
        let pdf = layer_trees_to_pdf(&[tree]).expect("baked watermark PDF export");
        let images = issue_4764_pdf_image_streams(&pdf);
        assert!(issue_4764_pdf_images_contain_rgb(&images, baked));
        assert!(!issue_4764_pdf_images_contain_rgb(&images, [20, 30, 40]));
    }
}

// #7501: the shared #7688 arrow outlines are replayed into the PDF canvas too.
// These tests check the PDF's vector coordinates, not the same helper's output.
mod connector_arrow_regressions {
    use rhwp::paint::{LayerNode, PageLayerTree, PaintOp, RenderProfile};
    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    use rhwp::renderer::pdf::layer_trees_to_pdf;
    use rhwp::renderer::render_tree::{BoundingBox, LineNode, PathNode};
    use rhwp::renderer::skia::SkiaLayerRenderer;
    use rhwp::renderer::{
        ArrowStyle, LineRenderType, LineStyle, PathCommand, ShadowStyle, ShapeStyle,
    };

    fn style(arrow: ArrowStyle) -> LineStyle {
        LineStyle {
            width: 1.0,
            start_arrow: arrow,
            end_arrow: arrow,
            start_arrow_size: 4,
            end_arrow_size: 4,
            ..Default::default()
        }
    }

    fn connector(style: LineStyle) -> PathNode {
        let mut path = PathNode::new(
            vec![
                PathCommand::MoveTo(20.0, 40.0),
                PathCommand::LineTo(100.0, 40.0),
            ],
            ShapeStyle {
                stroke_color: Some(0),
                stroke_width: 1.0,
                ..Default::default()
            },
            None,
        );
        path.connector_endpoints = Some((20.0, 40.0, 100.0, 40.0));
        path.line_style = Some(style);
        path
    }

    fn tree(op: PaintOp) -> PageLayerTree {
        let bounds = BoundingBox::new(0.0, 0.0, 128.0, 96.0);
        PageLayerTree::with_profile(
            bounds.width,
            bounds.height,
            LayerNode::leaf(bounds, None, vec![op]),
            RenderProfile::Print,
        )
    }

    fn line(style: LineStyle) -> PageLayerTree {
        tree(PaintOp::line(
            BoundingBox::new(20.0, 40.0, 80.0, 0.0),
            LineNode::new(20.0, 40.0, 100.0, 40.0, style),
        ))
    }

    fn path(path: PathNode) -> PageLayerTree {
        tree(PaintOp::path(
            BoundingBox::new(20.0, 10.0, 80.0, 60.0),
            path,
        ))
    }

    fn vector_commands(pdf: &[u8]) -> String {
        use flate2::read::ZlibDecoder;
        use std::io::Read;

        assert!(pdf.starts_with(b"%PDF-"));
        assert!(pdf.windows(5).any(|bytes| bytes == b"%%EOF"));
        assert!(
            !String::from_utf8_lossy(pdf).contains("/Subtype /Image"),
            "simple arrow outlines must remain vectors"
        );
        let mut content = String::new();
        let mut cursor = 0;
        while let Some(relative) = pdf[cursor..]
            .windows(7)
            .position(|bytes| bytes == b"stream\n")
        {
            let marker = cursor + relative;
            // Skia writes the stream token directly after the dictionary's
            // closing >>. The closing endstream token is not a new stream.
            if pdf[..marker].ends_with(b"end") {
                cursor = marker + 7;
                continue;
            }
            let dictionary_start = pdf[..marker]
                .windows(2)
                .rposition(|bytes| bytes == b"<<")
                .expect("stream dictionary");
            let dictionary = String::from_utf8_lossy(&pdf[dictionary_start..marker]);
            let length: usize = dictionary
                .split_once("/Length ")
                .expect("direct stream length")
                .1
                .trim_start()
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .unwrap()
                .parse()
                .unwrap();
            let start = marker + 7;
            let end = start + length;
            let data = if dictionary.contains("/FlateDecode") {
                let mut decoded = Vec::new();
                ZlibDecoder::new(&pdf[start..end])
                    .read_to_end(&mut decoded)
                    .unwrap();
                decoded
            } else {
                pdf[start..end].to_vec()
            };
            content.push_str(std::str::from_utf8(&data).expect("vector-only content stream"));
            content.push('\n');
            cursor = end;
        }
        assert!(!content.is_empty());
        content
    }

    fn assert_vertices(commands: &str, expected: &[(f64, f64)]) {
        let mut vertices = Vec::new();
        let mut operands = Vec::new();
        let mut matrix = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
        let mut stack = Vec::new();
        for token in commands.split_whitespace() {
            if let Ok(number) = token.parse::<f64>() {
                operands.push(number);
                continue;
            }
            match token {
                "q" => stack.push(matrix),
                "Q" => matrix = stack.pop().expect("balanced PDF graphics state"),
                "cm" if operands.len() == 6 => {
                    let m = &operands;
                    let c = matrix;
                    matrix = [
                        c[0] * m[0] + c[2] * m[1],
                        c[1] * m[0] + c[3] * m[1],
                        c[0] * m[2] + c[2] * m[3],
                        c[1] * m[2] + c[3] * m[3],
                        c[0] * m[4] + c[2] * m[5] + c[4],
                        c[1] * m[4] + c[3] * m[5] + c[5],
                    ];
                }
                "m" | "l" if operands.len() == 2 => {
                    vertices.push((
                        matrix[0] * operands[0] + matrix[2] * operands[1] + matrix[4],
                        matrix[1] * operands[0] + matrix[3] * operands[1] + matrix[5],
                    ));
                }
                _ => {}
            }
            operands.clear();
        }
        for (x, y) in expected {
            // Standard PDF points have a bottom-left origin. Account for the
            // recorded CTM rather than pinning Skia's choice of matrix packing.
            let point = (x * 0.75, (96.0 - y) * 0.75);
            assert!(
                vertices
                    .iter()
                    .any(|(a, b)| (a - point.0).abs() < 0.02 && (b - point.1).abs() < 0.02),
                "missing vector point {point:?} in {vertices:?}"
            );
        }
    }

    #[test]
    fn direct_pdf_records_both_triangle_heads_at_the_original_line_and_connector_tips() {
        for tree in [
            line(style(ArrowStyle::Arrow)),
            path(connector(style(ArrowStyle::Arrow))),
        ] {
            let pdf = layer_trees_to_pdf(std::slice::from_ref(&tree)).unwrap();
            let commands = vector_commands(&pdf);
            // Independent medium-size contract from #7688 Hancom Print: the
            // head length and width are 15.36 CSS px, with its tip at the source.
            assert_vertices(
                &commands,
                &[
                    (20.0, 40.0),
                    (35.36, 32.32),
                    (35.36, 47.68),
                    (100.0, 40.0),
                    (84.64, 32.32),
                    (84.64, 47.68),
                ],
            );
            let pixels = SkiaLayerRenderer::new()
                .render_pixels_with_options(&tree, RasterRenderOptions::default())
                .unwrap();
            // Stay inside the independently specified triangles, away from
            // their AA edges and outside the one-pixel horizontal shaft.
            for (x, y) in [(30, 42), (90, 42)] {
                let offset = y * pixels.stride + x * 4;
                assert_eq!(pixels.bytes[offset + 3], 255);
                assert!(pixels.bytes[offset..offset + 3]
                    .iter()
                    .all(|value| *value < 32));
            }
        }
    }

    #[test]
    fn direct_pdf_connector_heads_follow_curve_tangents_instead_of_the_chord() {
        let mut curved = connector(style(ArrowStyle::Arrow));
        curved.commands[1] = PathCommand::CurveTo(20.0, 10.0, 100.0, 10.0, 100.0, 40.0);
        let commands = vector_commands(&layer_trees_to_pdf(&[path(curved)]).unwrap());
        assert_vertices(
            &commands,
            &[
                (20.0, 40.0),
                (12.32, 24.64),
                (27.68, 24.64),
                (100.0, 40.0),
                (92.32, 24.64),
                (107.68, 24.64),
            ],
        );
        assert!(commands.split_whitespace().any(|token| token == "c"));
    }

    #[test]
    fn direct_pdf_accepts_supported_open_filled_and_asymmetric_head_sizes() {
        let count_subpaths = |commands: &str| {
            commands
                .split_whitespace()
                // PDF `re` starts a rectangle subpath directly; Skia can
                // encode square heads without a separate moveto command.
                .filter(|token| matches!(*token, "m" | "re"))
                .count()
        };
        let plain = vector_commands(&layer_trees_to_pdf(&[line(style(ArrowStyle::None))]).unwrap());
        let plain_subpaths = count_subpaths(&plain);
        for arrow in [
            ArrowStyle::None,
            ArrowStyle::Arrow,
            ArrowStyle::ConcaveArrow,
            ArrowStyle::Diamond,
            ArrowStyle::OpenDiamond,
            ArrowStyle::Circle,
            ArrowStyle::OpenCircle,
            ArrowStyle::Square,
            ArrowStyle::OpenSquare,
            ArrowStyle::OpenArrow,
        ] {
            let mut style = style(arrow);
            style.start_arrow_size = 0;
            style.end_arrow_size = 8;
            for tree in [line(style.clone()), path(connector(style))] {
                let commands = vector_commands(&layer_trees_to_pdf(&[tree]).unwrap());
                if arrow != ArrowStyle::None {
                    assert!(
                        count_subpaths(&commands) > plain_subpaths,
                        "{arrow:?}: no head paths"
                    );
                }
            }
        }
        let mut invisible_metadata = connector(style(ArrowStyle::None));
        invisible_metadata.line_style = None;
        vector_commands(&layer_trees_to_pdf(&[path(invisible_metadata)]).unwrap());
        let zero_line_without_heads = tree(PaintOp::line(
            BoundingBox::new(20.0, 40.0, 0.0, 0.0),
            LineNode::new(20.0, 40.0, 20.0, 40.0, style(ArrowStyle::None)),
        ));
        vector_commands(&layer_trees_to_pdf(&[zero_line_without_heads]).unwrap());
    }

    #[test]
    fn direct_pdf_rejects_missing_degenerate_and_nonfinite_arrow_geometry() {
        let valid = line(style(ArrowStyle::Arrow));
        let mut missing = connector(style(ArrowStyle::Arrow));
        missing.connector_endpoints = None;
        let mut nonfinite = connector(style(ArrowStyle::None));
        nonfinite.connector_endpoints = Some((20.0, 40.0, f64::NAN, 40.0));
        let mut degenerate = connector(style(ArrowStyle::Arrow));
        degenerate.connector_endpoints = Some((20.0, 40.0, 20.0, 40.0));
        degenerate.commands = vec![PathCommand::MoveTo(20.0, 40.0)];
        let zero_line = tree(PaintOp::line(
            BoundingBox::new(20.0, 40.0, 0.0, 0.0),
            LineNode::new(20.0, 40.0, 20.0, 40.0, style(ArrowStyle::Arrow)),
        ));
        let collapsed_line = tree(PaintOp::line(
            BoundingBox::new(20.0, 40.0, 1e-9, 0.0),
            LineNode::new(20.0, 40.0, 20.0 + 1e-9, 40.0, style(ArrowStyle::Arrow)),
        ));
        let nonfinite_line = tree(PaintOp::line(
            BoundingBox::new(20.0, 40.0, 80.0, 0.0),
            LineNode::new(20.0, 40.0, f64::NAN, 40.0, style(ArrowStyle::Arrow)),
        ));
        let mut negative_width = style(ArrowStyle::Arrow);
        negative_width.width = -1.0;
        let mut invalid_size = style(ArrowStyle::Arrow);
        invalid_size.start_arrow_size = 9;
        let mut overflow = style(ArrowStyle::Arrow);
        overflow.width = f64::from(f32::MAX);
        for invalid in [
            path(missing),
            path(nonfinite),
            path(degenerate),
            zero_line,
            collapsed_line,
            nonfinite_line,
            line(negative_width.clone()),
            path(connector(negative_width)),
            line(invalid_size.clone()),
            path(connector(invalid_size)),
            line(overflow.clone()),
            path(connector(overflow)),
        ] {
            let error = layer_trees_to_pdf(&[valid.clone(), invalid]).unwrap_err();
            assert!(error.contains("direct PDF page 2"), "{error}");
            assert!(error.contains("invalid"), "{error}");
        }
    }

    #[test]
    fn direct_pdf_requires_arrow_metadata_to_match_one_continuous_shaft() {
        for commands in [
            vec![],
            vec![PathCommand::MoveTo(20.0, 40.0)],
            vec![PathCommand::LineTo(100.0, 40.0)],
            vec![
                PathCommand::MoveTo(20.0, 40.0),
                PathCommand::LineTo(60.0, 40.0),
                PathCommand::MoveTo(80.0, 40.0),
                PathCommand::LineTo(100.0, 40.0),
            ],
            vec![
                PathCommand::MoveTo(21.0, 40.0),
                PathCommand::LineTo(100.0, 40.0),
            ],
            vec![
                PathCommand::MoveTo(20.0, 40.0),
                PathCommand::LineTo(99.0, 40.0),
            ],
        ] {
            let mut connector = connector(style(ArrowStyle::Arrow));
            connector.commands = commands;
            let error = layer_trees_to_pdf(&[path(connector)]).unwrap_err();
            assert!(error.contains("invalid connector shaft"), "{error}");
            assert!(error.contains("use the svg backend"));
        }
    }

    #[test]
    fn direct_pdf_rejects_unproved_endpoint_controls_only_where_a_head_is_requested() {
        // For B'(0)=0, B''(0) points up to (20,10); a chord fallback would
        // incorrectly point the head left. The end case is its counterpart.
        for (segments, requested, unrequested, expected) in [
            (
                vec![PathCommand::CurveTo(20.0, 40.0, 20.0, 10.0, 100.0, 40.0)],
                (ArrowStyle::Arrow, ArrowStyle::None),
                (ArrowStyle::None, ArrowStyle::Arrow),
                None,
            ),
            (
                vec![PathCommand::CurveTo(20.0, 10.0, 100.0, 40.0, 100.0, 40.0)],
                (ArrowStyle::None, ArrowStyle::Arrow),
                (ArrowStyle::Arrow, ArrowStyle::None),
                Some([(20.0, 40.0), (12.32, 24.64), (27.68, 24.64)]),
            ),
            (
                vec![PathCommand::CurveTo(
                    20.0 + 1e-9,
                    40.0,
                    20.0,
                    10.0,
                    100.0,
                    40.0,
                )],
                (ArrowStyle::Arrow, ArrowStyle::None),
                (ArrowStyle::None, ArrowStyle::Arrow),
                None,
            ),
            (
                vec![PathCommand::CurveTo(
                    20.0,
                    10.0,
                    100.0 - 1e-9,
                    40.0,
                    100.0,
                    40.0,
                )],
                (ArrowStyle::None, ArrowStyle::Arrow),
                (ArrowStyle::Arrow, ArrowStyle::None),
                Some([(20.0, 40.0), (12.32, 24.64), (27.68, 24.64)]),
            ),
            (
                vec![
                    PathCommand::LineTo(20.0 + 1e-9, 40.0 + 1e-9),
                    PathCommand::LineTo(100.0, 40.0),
                ],
                (ArrowStyle::Arrow, ArrowStyle::None),
                (ArrowStyle::None, ArrowStyle::Arrow),
                None,
            ),
            (
                vec![
                    PathCommand::LineTo(100.0 - 1e-9, 40.0 - 1e-9),
                    PathCommand::LineTo(100.0, 40.0),
                ],
                (ArrowStyle::None, ArrowStyle::Arrow),
                (ArrowStyle::Arrow, ArrowStyle::None),
                None,
            ),
        ] {
            let mut connector = connector(style(ArrowStyle::None));
            connector.commands.truncate(1);
            connector.commands.extend(segments);
            let line = connector.line_style.as_mut().unwrap();
            line.start_arrow = requested.0;
            line.end_arrow = requested.1;
            let error = layer_trees_to_pdf(&[path(connector.clone())]).unwrap_err();
            assert!(error.contains("unsupported arrow tangent"), "{error}");
            let line = connector.line_style.as_mut().unwrap();
            line.start_arrow = unrequested.0;
            line.end_arrow = unrequested.1;
            let commands = vector_commands(&layer_trees_to_pdf(&[path(connector)]).unwrap());
            if let Some(expected) = expected {
                assert_vertices(&commands, &expected);
            }
        }
    }

    #[test]
    fn direct_pdf_keeps_unproved_line_styles_and_connector_tangents_closed() {
        let mut multiline = style(ArrowStyle::Arrow);
        multiline.line_type = LineRenderType::Double;
        let mut shadow = style(ArrowStyle::Arrow);
        shadow.shadow = Some(ShadowStyle {
            shadow_type: 1,
            color: 0,
            offset_x: 2.0,
            offset_y: 2.0,
            alpha: 128,
        });
        for style in [multiline, shadow] {
            for invalid in [line(style.clone()), path(connector(style))] {
                let error = layer_trees_to_pdf(&[invalid]).unwrap_err();
                assert!(error.contains("multi-line or shadow style"), "{error}");
                assert!(error.contains("use the svg backend"));
            }
        }
        for unsupported in [
            PathCommand::ArcTo(40.0, 20.0, 0.0, false, true, 100.0, 40.0),
            PathCommand::ClosePath,
        ] {
            let mut curved = connector(style(ArrowStyle::Arrow));
            curved.commands.push(unsupported);
            let error = layer_trees_to_pdf(&[path(curved)]).unwrap_err();
            assert!(error.contains("unsupported arrow tangent"), "{error}");
            assert!(error.contains("use the svg backend"));
        }
    }
}
