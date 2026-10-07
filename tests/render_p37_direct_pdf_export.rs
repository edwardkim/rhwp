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

// These pixel regressions run in the existing Native Skia CI target.
mod image_adjustment_regressions {
    #[cfg(feature = "native-skia")]
    fn issue_4764_png_rgba(width: u32, height: u32, rgba: [u8; 4]) -> Vec<u8> {
        let pixels = image::RgbaImage::from_pixel(width, height, image::Rgba(rgba));
        let mut bytes = std::io::Cursor::new(Vec::new());
        pixels
            .write_to(&mut bytes, image::ImageFormat::Png)
            .expect("encode test PNG");
        bytes.into_inner()
    }

    #[cfg(feature = "native-skia")]
    fn issue_4764_adjusted_rgb(rgb: [u8; 3], brightness: i8, contrast: i8) -> [u8; 3] {
        // Independent sRGB transfer equations from CSS Color 4. SVG filter
        // primitives use linearRGB by default (Filter Effects 1 section 10).
        let brightness = brightness.clamp(-100, 100) as f64 / 100.0;
        let slope = (100.0 + contrast.clamp(-100, 100) as f64) / 100.0;
        let intercept = (0.5 - 0.5 * slope) + brightness;
        rgb.map(|channel| {
            let encoded = channel as f64 / 255.0;
            let linear = if encoded <= 0.04045 {
                encoded / 12.92
            } else {
                ((encoded + 0.055) / 1.055).powf(2.4)
            };
            let adjusted = (linear * slope + intercept).clamp(0.0, 1.0);
            let encoded = if adjusted <= 0.0031308 {
                adjusted * 12.92
            } else {
                1.055 * adjusted.powf(1.0 / 2.4) - 0.055
            };
            (encoded * 255.0).round().clamp(0.0, 255.0) as u8
        })
    }

    #[cfg(feature = "native-skia")]
    struct Issue4764PdfImageStream {
        width: usize,
        height: usize,
        pixels: Vec<u8>,
    }

    #[cfg(feature = "native-skia")]
    fn issue_4764_rfind(haystack: &[u8], needle: &[u8]) -> Option<usize> {
        haystack
            .windows(needle.len())
            .rposition(|window| window == needle)
    }

    #[cfg(feature = "native-skia")]
    fn issue_4764_pdf_dict_usize(dict: &[u8], key: &str) -> Option<usize> {
        let dict = String::from_utf8_lossy(dict);
        let start = dict.find(key)? + key.len();
        dict[start..]
            .split_whitespace()
            .next()
            .and_then(|value| value.parse::<usize>().ok())
    }

    #[cfg(feature = "native-skia")]
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

    #[cfg(feature = "native-skia")]
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

    #[cfg(feature = "native-skia")]
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

    #[cfg(feature = "native-skia")]
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

    #[cfg(feature = "native-skia")]
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

    #[cfg(feature = "native-skia")]
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
                    issue_4764_adjusted_rgb(source, brightness, contrast)
                ),
                "PDF image streams should contain brightness/contrast adjusted pixels"
            );
            assert!(
                !issue_4764_pdf_images_contain_rgb(&images, source),
                "PDF image streams should not keep the unadjusted source pixels"
            );
        }
    }

    #[cfg(feature = "native-skia")]
    #[test]
    fn issue_4764_native_adjustments_preserve_alpha_and_object_opacity() {
        use rhwp::renderer::layer_renderer::RasterRenderOptions;
        use rhwp::renderer::render_tree::ImageNode;
        use rhwp::renderer::skia::SkiaLayerRenderer;

        let source = [80, 100, 120];
        for (brightness, contrast, opacity) in
            [(20, 0, 1.0), (0, 20, 1.0), (0, 8, 0.5), (0, 0, 1.0)]
        {
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
            let expected = issue_4764_adjusted_rgb(source, brightness, contrast);
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

    #[cfg(feature = "native-skia")]
    #[test]
    fn issue_4764_direct_pdf_applies_grayscale_before_linear_contrast() {
        use rhwp::model::image::ImageEffect;
        use rhwp::renderer::pdf::layer_trees_to_pdf;
        use rhwp::renderer::render_tree::ImageNode;

        let mut image = ImageNode::new(1, Some(issue_4764_png_rgba(4, 4, [80, 100, 120, 255])));
        image.effect = ImageEffect::GrayScale;
        image.contrast = 50;
        let pdf = layer_trees_to_pdf(&[issue_4764_direct_pdf_image_tree(image)])
            .expect("grayscale then contrast PDF export");
        // Linear luminance is about 0.1202. Applying contrast afterward gives
        // 1.5 * 0.1202 - 0.25 < 0; reversing the filters leaves blue ink.
        assert!(issue_4764_pdf_images_contain_rgb(
            &issue_4764_pdf_image_streams(&pdf),
            [0, 0, 0]
        ));
    }

    #[cfg(feature = "native-skia")]
    #[test]
    fn issue_4764_direct_pdf_embeds_adjusted_page_background_pixels_in_display_order() {
        use rhwp::model::image::ImageEffect;
        use rhwp::model::style::ImageFillMode;
        use rhwp::renderer::pdf::layer_trees_to_pdf;
        use rhwp::renderer::render_tree::PageBackgroundImage;

        // Keep the displayed linearRGB adjustment away from clipping all channels
        // to black so an incorrectly scaled offset cannot pass this regression.
        let source = [180, 190, 200];
        let image = PageBackgroundImage {
            data: issue_4764_png_rgba(4, 4, [source[0], source[1], source[2], 255]),
            fill_mode: ImageFillMode::FitToSize,
            brightness: 35,
            contrast: -10,
            effect: ImageEffect::RealPic,
        };

        let pdf = layer_trees_to_pdf(&[issue_4764_direct_pdf_page_background_tree(image)])
            .expect("direct PDF export");
        let images = issue_4764_pdf_image_streams(&pdf);

        assert!(
            issue_4764_pdf_images_contain_rgb(&images, issue_4764_adjusted_rgb(source, -10, 35)),
            "page background must apply display brightness/contrast order"
        );
        assert!(
            !issue_4764_pdf_images_contain_rgb(&images, issue_4764_adjusted_rgb(source, 35, -10)),
            "page background must not use the raw stored brightness/contrast order"
        );
    }

    #[cfg(feature = "native-skia")]
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

    #[cfg(feature = "native-skia")]
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

    #[cfg(feature = "native-skia")]
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

    #[cfg(feature = "native-skia")]
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
        assert!(!issue_4764_pdf_images_contain_rgb(
            &images,
            issue_4764_adjusted_rgb(baked, 70, -50)
        ));
    }
}
