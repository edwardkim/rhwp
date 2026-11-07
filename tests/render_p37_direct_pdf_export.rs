//! 관계형·고정형: re-03-latin-only-hancom의 PDF API와 합성 PaintOp의 이미지 픽셀·화살촉 vector/거부 계약.
//! 주석 갱신: 2026-11-06T12:48:21+09:00; 직전 파일 커밋: 2026-11-06T12:42:57+09:00 (0f7251999437118ce95b8b5571e31571a7ad24a2).
//! 실물 페이지의 조판 좌표를 고정하는 검사가 아니므로 이 assertion의 raw 시각 측정값은 비해당이다.
//! 합성 알고리즘 계약과 별도의 Native/fresh WASM 실물 검증을 구분한다.
//! 근거: mydocs/pr/assets/seo_rii_20261030/README.md.
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
    use rhwp::paint::{CacheHint, GroupKind, LayerNode, PageLayerTree, PaintOp, RenderProfile};
    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    use rhwp::renderer::pdf::layer_trees_to_pdf;
    use rhwp::renderer::render_tree::{BoundingBox, GroupNode, LineNode, PathNode};
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
    fn direct_pdf_rejects_head_outlines_collapsed_before_a_parent_translation() {
        let translated = |offset: f64, arrow: ArrowStyle, as_path: bool| {
            let bounds = BoundingBox::new(0.0, 0.0, 256.0, 96.0);
            let source_bounds = BoundingBox::new(offset, 0.0, 256.0, 96.0);
            let shaft_bounds = BoundingBox::new(offset, 40.0, 256.0, 0.0);
            let op = if as_path {
                let mut connector = connector(style(arrow));
                connector.commands = vec![
                    PathCommand::MoveTo(offset, 40.0),
                    PathCommand::LineTo(offset + 256.0, 40.0),
                ];
                connector.connector_endpoints = Some((offset, 40.0, offset + 256.0, 40.0));
                PaintOp::path(shaft_bounds, connector)
            } else {
                PaintOp::line(
                    shaft_bounds,
                    LineNode::new(offset, 40.0, offset + 256.0, 40.0, style(arrow)),
                )
            };
            PageLayerTree::with_profile(
                bounds.width,
                bounds.height,
                LayerNode::group(
                    bounds,
                    None,
                    vec![LayerNode::leaf(source_bounds, None, vec![op])],
                    CacheHint::None,
                    GroupKind::Group(GroupNode {
                        paint_transform: Some([1.0, 0.0, 0.0, 1.0, -offset, 0.0]),
                        section_index: None,
                        para_index: None,
                        control_index: None,
                    }),
                ),
                RenderProfile::Print,
            )
        };
        for as_path in [false, true] {
            let plain = translated(1_073_741_824.0, ArrowStyle::None, as_path);
            let pixels = SkiaLayerRenderer::new()
                .render_pixels_with_options(&plain, RasterRenderOptions::default())
                .unwrap();
            assert!(pixels.bytes[40 * pixels.stride + 128 * 4 + 3] > 0);
            vector_commands(&layer_trees_to_pdf(&[plain]).unwrap());
            for arrow in [ArrowStyle::Arrow, ArrowStyle::OpenArrow, ArrowStyle::Circle] {
                let error = layer_trees_to_pdf(&[translated(1_073_741_824.0, arrow, as_path)])
                    .expect_err("the visible shaft must not hide a collapsed head");
                assert!(error.contains("invalid arrow geometry"), "{error}");
                assert!(error.contains("use the svg backend"));
                vector_commands(&layer_trees_to_pdf(&[translated(0.0, arrow, as_path)]).unwrap());
            }
        }
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

// P80: static admission is a bounded diagnostic API, independent of canvas/font
// preparation. Synthetic invalid fields establish the public reason/location
// contract without a system-font or real-document layout dependency.
mod direct_pdf_preflight_contract {
    use rhwp::paint::{
        CacheHint, ClipKind, GroupKind, LayerNode, PageLayerTree, PaintOp, RenderProfile,
    };
    use rhwp::renderer::pdf::{
        layer_trees_to_pdf, preflight_direct_pdf, DirectPdfExportOptions, DirectPdfPreflightLimit,
        DirectPdfPreflightLimits, DirectPdfPreflightLocation, DirectPdfPreflightReason as Reason,
    };
    use rhwp::renderer::render_tree::{BoundingBox, ImageNode, RectangleNode};
    use rhwp::renderer::{GradientFillInfo, PatternFillInfo, ShadowStyle, ShapeStyle};

    fn bounds() -> BoundingBox {
        BoundingBox::new(0.0, 0.0, 96.0, 64.0)
    }

    fn rectangle(opacity: f64) -> PaintOp {
        PaintOp::rectangle(
            BoundingBox::new(8.0, 8.0, 80.0, 32.0),
            RectangleNode::new(
                0.0,
                ShapeStyle {
                    opacity,
                    ..Default::default()
                },
                None,
            ),
        )
    }

    fn page(root: LayerNode) -> PageLayerTree {
        PageLayerTree::with_profile(96.0, 64.0, root, RenderProfile::Print)
    }

    fn leaf(ops: Vec<PaintOp>) -> LayerNode {
        LayerNode::leaf(bounds(), None, ops)
    }

    #[test]
    fn preflight_collects_independent_failures_on_one_op_and_preserves_export_first_error() {
        let mut op = rectangle(2.0);
        let PaintOp::Rectangle { rect, .. } = &mut op else {
            unreachable!()
        };
        rect.transform.rotation = f64::NAN;
        rect.corner_radius = -1.0;
        rect.gradient = Some(Box::new(GradientFillInfo {
            gradient_type: 3,
            angle: 0,
            center_x: 50,
            center_y: 50,
            colors: vec![0xff],
            positions: vec![-0.1, 1.0],
        }));
        rect.style.pattern = Some(PatternFillInfo {
            pattern_type: 1,
            tile_size: 8.0,
            pattern_color: 0,
            background_color: 0xffffff,
        });
        rect.style.shadow = Some(ShadowStyle {
            shadow_type: 1,
            color: 0,
            offset_x: 1.0,
            offset_y: 1.0,
            alpha: 128,
        });
        rect.style.stroke_width = -1.0;
        let pages = [page(leaf(vec![op]))];
        let report = preflight_direct_pdf(
            &pages,
            &DirectPdfExportOptions::default(),
            DirectPdfPreflightLimits::default(),
        );
        let reasons: Vec<_> = report.issues.iter().map(|issue| issue.reason).collect();
        assert_eq!(
            reasons,
            [
                Reason::InvalidRotation,
                Reason::InvalidCornerRadius,
                Reason::UnsupportedGradientType,
                Reason::GradientColorCountOutOfRange,
                Reason::InvalidGradientStops,
                Reason::UnsupportedPatternFill,
                Reason::UnsupportedShapeShadow,
                Reason::InvalidOpacity,
                Reason::InvalidStrokeWidth,
            ]
        );
        assert!(!report.truncated);
        assert!(!report.is_admitted());
        assert!(report.issues.iter().all(|issue| issue.location
            == DirectPdfPreflightLocation {
                page_index: Some(0),
                node_index: Some(0),
                op_index: Some(0),
                operation: Some("rectangle"),
            }));
        assert_eq!(
            layer_trees_to_pdf(&pages).unwrap_err(),
            report.issues[0].message
        );
        assert_eq!(
            report.issues[0].message,
            "direct PDF page 1 has invalid rectangle rotation"
        );
    }

    #[test]
    fn preflight_orders_layers_clips_ops_and_pages_without_merging_distinct_locations() {
        let clip = LayerNode::clip_rect(
            BoundingBox::new(f64::NAN, 0.0, 96.0, 64.0),
            None,
            BoundingBox::new(0.0, 0.0, -1.0, 16.0),
            leaf(vec![rectangle(2.0)]),
            ClipKind::Generic,
        );
        let root = LayerNode::group(
            bounds(),
            None,
            vec![clip, leaf(vec![rectangle(2.0)])],
            CacheHint::None,
            GroupKind::Generic,
        );
        let mut first = page(root);
        first.page_width = 0.0;
        first.page_height = 20_000.0;
        let pages = [first, page(leaf(vec![rectangle(2.0)]))];
        let options = DirectPdfExportOptions::default();
        let report = preflight_direct_pdf(&pages, &options, DirectPdfPreflightLimits::default());
        assert_eq!(
            report,
            preflight_direct_pdf(&pages, &options, DirectPdfPreflightLimits::default())
        );
        let actual: Vec<_> = report
            .issues
            .iter()
            .map(|issue| {
                (
                    issue.reason,
                    issue.location.page_index,
                    issue.location.node_index,
                    issue.location.op_index,
                )
            })
            .collect();
        assert_eq!(
            actual,
            [
                (Reason::InvalidLayerBounds, Some(0), Some(1), None),
                (Reason::InvalidClipBounds, Some(0), Some(1), None),
                (Reason::InvalidOpacity, Some(0), Some(2), Some(0)),
                (Reason::InvalidOpacity, Some(0), Some(3), Some(1)),
                (Reason::InvalidPageWidth, Some(0), None, None),
                (Reason::PageHeightTooLarge, Some(0), None, None),
                (Reason::InvalidOpacity, Some(1), Some(0), Some(0)),
            ]
        );
        assert_eq!(
            layer_trees_to_pdf(&pages).unwrap_err(),
            report.issues[0].message
        );
    }

    #[test]
    fn preflight_marks_only_omitted_issues_as_truncated_and_clamps_limits() {
        let pages = [page(leaf(vec![
            rectangle(2.0),
            rectangle(2.0),
            rectangle(2.0),
        ]))];
        let options = DirectPdfExportOptions::default();
        let limits = DirectPdfPreflightLimits {
            max_issues: 2,
            max_work_items: 100,
        };
        let report = preflight_direct_pdf(&pages, &options, limits);
        assert_eq!(report.issues.len(), 2);
        assert_eq!(report.limit_reached, Some(DirectPdfPreflightLimit::Issues));
        assert!(report.truncated);
        assert!(!report.is_admitted());
        let exact = preflight_direct_pdf(
            &[page(leaf(vec![rectangle(2.0), rectangle(2.0)]))],
            &options,
            limits,
        );
        assert_eq!(exact.issues.len(), 2);
        assert!(!exact.truncated);
        assert_eq!(exact.limit_reached, None);
        let clamped = preflight_direct_pdf(
            &pages,
            &options,
            DirectPdfPreflightLimits {
                max_issues: 0,
                max_work_items: usize::MAX,
            },
        );
        assert_eq!(clamped.limits.max_issues, 1);
        assert_eq!(clamped.limits.max_work_items, 1_000_000);
        assert_eq!(clamped.issues.len(), 1);
        assert!(clamped.truncated);
        let high = preflight_direct_pdf(
            &[],
            &options,
            DirectPdfPreflightLimits {
                max_issues: usize::MAX,
                max_work_items: 0,
            },
        );
        assert_eq!(high.limits.max_issues, 4096);
        assert_eq!(high.limits.max_work_items, 1);
    }

    #[test]
    fn preflight_work_limit_never_admits_an_uninspected_valid_or_invalid_tail() {
        let options = DirectPdfExportOptions::default();
        let limits = DirectPdfPreflightLimits {
            max_issues: 16,
            max_work_items: 3,
        };
        for tail_opacity in [1.0, 2.0] {
            let pages = [page(leaf(vec![rectangle(1.0), rectangle(tail_opacity)]))];
            let report = preflight_direct_pdf(&pages, &options, limits);
            assert!(report.issues.is_empty());
            assert_eq!(report.work_items_checked, 3); // page + layer + first op
            assert_eq!(
                report.limit_reached,
                Some(DirectPdfPreflightLimit::WorkItems)
            );
            assert!(!report.is_admitted());
            let complete = preflight_direct_pdf(
                &pages,
                &options,
                DirectPdfPreflightLimits {
                    max_work_items: 4,
                    ..limits
                },
            );
            assert!(!complete.truncated);
            assert_eq!(complete.is_admitted(), tail_opacity == 1.0);
        }
    }

    #[test]
    fn preflight_collects_document_and_option_errors_without_a_page_location() {
        let mut options = DirectPdfExportOptions::default();
        options.raster_dpi = f32::NAN;
        let report = preflight_direct_pdf(&[], &options, DirectPdfPreflightLimits::default());
        assert_eq!(
            report
                .issues
                .iter()
                .map(|issue| issue.reason)
                .collect::<Vec<_>>(),
            [Reason::EmptyDocument, Reason::InvalidRasterDpi]
        );
        assert!(report
            .issues
            .iter()
            .all(|issue| issue.location == DirectPdfPreflightLocation::default()));
        assert!(!report.is_admitted());
    }

    #[test]
    fn preflight_and_export_reject_page_extents_that_underflow_skia_scalars() {
        let options = DirectPdfExportOptions::default();
        for tiny in [1e-50, 1e-300, f64::MIN_POSITIVE] {
            assert!(tiny * 0.75 > 0.0);
            assert_eq!((tiny * 0.75) as f32, 0.0);
            for width in [true, false] {
                let mut tree = page(leaf(Vec::new()));
                if width {
                    tree.page_width = tiny;
                } else {
                    tree.page_height = tiny;
                }
                let pages = [tree];
                let report =
                    preflight_direct_pdf(&pages, &options, DirectPdfPreflightLimits::default());
                assert!(!report.is_admitted());
                assert!(!report.truncated);
                assert_eq!(report.issues.len(), 1);
                assert_eq!(
                    report.issues[0].reason,
                    if width {
                        Reason::InvalidPageWidth
                    } else {
                        Reason::InvalidPageHeight
                    }
                );
                assert_eq!(
                    report.issues[0].location,
                    DirectPdfPreflightLocation {
                        page_index: Some(0),
                        ..Default::default()
                    }
                );
                // Exercise the real public export path: this must return Err,
                // rather than pass a zero width/height to Skia's begin_page.
                assert_eq!(
                    layer_trees_to_pdf(&pages).unwrap_err(),
                    report.issues[0].message
                );
            }
        }
        // Positivity is checked after conversion, without inventing a minimum
        // physical page size. Small representable extents remain supported.
        let mut small = page(leaf(Vec::new()));
        small.page_width = 0.0001;
        small.page_height = 0.0002;
        assert!(preflight_direct_pdf(
            std::slice::from_ref(&small),
            &options,
            DirectPdfPreflightLimits::default(),
        )
        .is_admitted());
        assert!(layer_trees_to_pdf(&[small]).unwrap().starts_with(b"%PDF-"));
    }

    #[test]
    fn preflight_static_success_does_not_mask_a_strict_image_decode_failure() {
        // A recognized PNG signature has no decodable image. Static inspection
        // must not allocate a decoded image or claim successful resource replay.
        let image = ImageNode::new(1, Some(b"\x89PNG\r\n\x1a\n".to_vec()));
        let pages = [page(leaf(vec![PaintOp::image(bounds(), image, None)]))];
        let report = preflight_direct_pdf(
            &pages,
            &DirectPdfExportOptions::default(),
            DirectPdfPreflightLimits::default(),
        );
        assert!(report.is_admitted());
        let error = layer_trees_to_pdf(&pages)
            .expect_err("export still decodes the resource and fails closed");
        assert!(error.contains("cannot decode image payload"), "{error}");
    }
}

// P81: the established physical-circle contract is admitted only for bounded
// type-2 gradients. PDF shading dictionaries prove vector recording, while
// analytic color relationships and clipping exercise the common Native paint.
mod direct_pdf_radial_gradient_contract {
    use rhwp::paint::{ClipKind, LayerNode, LayerNodeKind, PageLayerTree, PaintOp, RenderProfile};
    use rhwp::renderer::layer_renderer::RasterRenderOptions;
    use rhwp::renderer::pdf::{
        layer_trees_to_pdf, preflight_direct_pdf, DirectPdfExportOptions, DirectPdfPreflightLimits,
        DirectPdfPreflightReason as Reason,
    };
    use rhwp::renderer::raster_pixels::RasterPixels;
    use rhwp::renderer::render_tree::{
        BoundingBox, EllipseNode, PageBackgroundNode, PathNode, RectangleNode,
    };
    use rhwp::renderer::skia::SkiaLayerRenderer;
    use rhwp::renderer::{GradientFillInfo, PathCommand, ShapeStyle};

    fn bounds() -> BoundingBox {
        BoundingBox::new(8.0, 8.0, 80.0, 48.0)
    }

    fn gradient() -> GradientFillInfo {
        GradientFillInfo {
            gradient_type: 2,
            angle: 0,
            center_x: 50,
            center_y: 50,
            colors: vec![0x0000_00ff, 0x00ff_0000],
            positions: vec![0.0, 1.0],
        }
    }

    fn tree(
        kind: &str,
        gradient: GradientFillInfo,
        bounds: BoundingBox,
        opacity: f64,
    ) -> PageLayerTree {
        let gradient = Some(Box::new(gradient));
        let style = ShapeStyle {
            opacity,
            ..Default::default()
        };
        let op = match kind {
            "background" => PaintOp::page_background(
                bounds,
                PageBackgroundNode {
                    background_color: None,
                    border_color: None,
                    border_width: 0.0,
                    gradient,
                    image: None,
                },
            ),
            "rectangle" => PaintOp::rectangle(bounds, RectangleNode::new(0.0, style, gradient)),
            "ellipse" => PaintOp::ellipse(bounds, EllipseNode::new(style, gradient)),
            "path" => PaintOp::path(
                bounds,
                PathNode::new(
                    vec![
                        PathCommand::MoveTo(bounds.x, bounds.y),
                        PathCommand::LineTo(bounds.x + bounds.width, bounds.y),
                        PathCommand::LineTo(bounds.x + bounds.width, bounds.y + bounds.height),
                        PathCommand::LineTo(bounds.x, bounds.y + bounds.height),
                        PathCommand::ClosePath,
                    ],
                    style,
                    gradient,
                ),
            ),
            _ => unreachable!("known shape fixture"),
        };
        PageLayerTree::with_profile(
            96.0,
            64.0,
            LayerNode::leaf(BoundingBox::new(0.0, 0.0, 96.0, 64.0), None, vec![op]),
            RenderProfile::Print,
        )
    }

    fn report(tree: &PageLayerTree) -> rhwp::renderer::pdf::DirectPdfPreflightReport {
        preflight_direct_pdf(
            std::slice::from_ref(tree),
            &DirectPdfExportOptions::default(),
            DirectPdfPreflightLimits::default(),
        )
    }

    fn assert_radial_vector_pdf(tree: &PageLayerTree) -> Vec<u8> {
        assert!(report(tree).is_admitted());
        let pdf =
            layer_trees_to_pdf(std::slice::from_ref(tree)).expect("bounded radial vector export");
        let dictionaries = String::from_utf8_lossy(&pdf);
        assert!(
            dictionaries.contains("/ShadingType 3"),
            "PDF radial shading must be recorded"
        );
        assert!(
            !dictionaries.contains("/ShadingType 2"),
            "type 2 source is a physical circle"
        );
        assert!(
            !dictionaries.contains("/Subtype /Image"),
            "radial fill must stay vector"
        );
        // Standard PDF radial shading contains start/end circles. Skia may
        // normalize their coordinates into a pattern matrix, so inspect the
        // circle relationship rather than pin its matrix packing.
        let mut circles = 0;
        let mut cursor = dictionaries.as_ref();
        while let Some((_, rest)) = cursor.split_once("/Coords [") {
            let (coords, rest) = rest.split_once(']').expect("closed shading coordinates");
            let values: Vec<f64> = coords
                .split_whitespace()
                .map(|value| value.parse().expect("numeric shading coordinate"))
                .collect();
            if values.len() == 6 {
                assert!(values.iter().all(|value| value.is_finite()));
                assert_eq!(values[0], values[3]);
                assert_eq!(values[1], values[4]);
                assert_eq!(values[2], 0.0);
                assert!(values[5] > 0.0);
                circles += 1;
            }
            cursor = rest;
        }
        assert!(circles > 0, "a radial shader has two concentric circles");
        pdf
    }

    fn pixel(pixels: &RasterPixels, x: usize, y: usize) -> [u8; 4] {
        let offset = y * pixels.stride + x * 4;
        pixels.bytes[offset..offset + 4].try_into().unwrap()
    }

    fn vector_streams(pdf: &[u8]) -> String {
        use flate2::read::ZlibDecoder;
        use std::io::Read;
        let mut output = String::new();
        let mut cursor = 0;
        while let Some(relative) = pdf[cursor..]
            .windows(7)
            .position(|bytes| bytes == b"stream\n")
        {
            let marker = cursor + relative;
            if pdf[..marker].ends_with(b"end") {
                cursor = marker + 7;
                continue;
            }
            let start = pdf[..marker]
                .windows(2)
                .rposition(|bytes| bytes == b"<<")
                .expect("stream dictionary");
            let dictionary = String::from_utf8_lossy(&pdf[start..marker]);
            let length: usize = dictionary
                .split_once("/Length ")
                .expect("stream length")
                .1
                .trim_start()
                .split(|c: char| !c.is_ascii_digit())
                .next()
                .unwrap()
                .parse()
                .unwrap();
            let start = marker + 7;
            let end = start + length;
            let bytes = if dictionary.contains("/FlateDecode") {
                let mut bytes = Vec::new();
                ZlibDecoder::new(&pdf[start..end])
                    .read_to_end(&mut bytes)
                    .unwrap();
                bytes
            } else {
                pdf[start..end].to_vec()
            };
            output.push_str(&String::from_utf8_lossy(&bytes));
            output.push('\n');
            cursor = end;
        }
        output
    }

    #[test]
    fn direct_pdf_admits_radial_background_rectangle_ellipse_and_path_at_source_centers() {
        for kind in ["background", "rectangle", "ellipse", "path"] {
            for (center_x, center_y) in [(0, 0), (50, 50), (100, 100), (0, 100), (100, 0)] {
                let mut source = gradient();
                source.center_x = center_x;
                source.center_y = center_y;
                assert_radial_vector_pdf(&tree(kind, source, bounds(), 1.0));
            }
        }
        for bounds in [
            BoundingBox::new(8.0, 8.0, 48.0, 48.0),
            BoundingBox::new(8.0, 8.0, 24.0, 48.0),
        ] {
            assert_radial_vector_pdf(&tree("rectangle", gradient(), bounds, 1.0));
        }
    }

    #[test]
    fn direct_pdf_radial_preserves_even_spacing_hard_stops_and_color_bound() {
        let mut evenly = gradient();
        evenly.positions.clear();
        assert_radial_vector_pdf(&tree("rectangle", evenly, bounds(), 1.0));
        let mut hard = gradient();
        hard.colors = vec![0xff, 0xff, 0xff0000, 0xff0000];
        hard.positions = vec![0.0, 0.5, 0.5, 1.0];
        assert_radial_vector_pdf(&tree("rectangle", hard, bounds(), 1.0));
        let mut maximum = gradient();
        maximum.colors = (0..4096)
            .map(|index| if index % 2 == 0 { 0xff } else { 0xff0000 })
            .collect();
        maximum.positions.clear();
        assert_radial_vector_pdf(&tree("rectangle", maximum, bounds(), 1.0));
    }

    #[test]
    fn direct_pdf_radial_rejects_invalid_centers_stops_and_f32_circles() {
        for (x, y) in [
            (-1, 50),
            (101, 50),
            (50, -1),
            (50, 101),
            (i16::MIN, i16::MAX),
        ] {
            let mut source = gradient();
            source.center_x = x;
            source.center_y = y;
            let candidate = tree("rectangle", source, bounds(), 1.0);
            assert!(report(&candidate)
                .issues
                .iter()
                .any(|issue| issue.reason == Reason::InvalidRadialGradientCenter));
            assert!(layer_trees_to_pdf(&[candidate])
                .unwrap_err()
                .contains("radial gradient center outside"));
        }
        for bounds in [
            BoundingBox::new(8.0, 8.0, 0.0, 48.0),
            BoundingBox::new(0.0, 0.0, 0.5, 0.5),
            BoundingBox::new(0.0, 0.0, 1e20, 1e20),
            // Nonempty f32 rectangle; its circle radius disappears against
            // the rounded center on the positive x side.
            BoundingBox::new(2_f64.powi(30), 0.0, 96.0, 1.0),
            BoundingBox::new(f64::from(f32::MAX), 0.0, f64::from(f32::MAX), 48.0),
        ] {
            let candidate = tree("rectangle", gradient(), bounds, 1.0);
            assert!(!report(&candidate).is_admitted());
            let error = layer_trees_to_pdf(&[candidate]).unwrap_err();
            assert!(
                error.contains("invalid gradient replay bounds")
                    || error.contains("invalid radial gradient circle"),
                "{error}"
            );
        }
        for positions in [
            vec![0.0],
            vec![1.0, 0.0],
            vec![0.0, f64::NAN],
            vec![-0.1, 1.0],
            vec![0.0, 1.1],
        ] {
            let mut source = gradient();
            source.positions = positions;
            let candidate = tree("rectangle", source, bounds(), 1.0);
            assert!(report(&candidate)
                .issues
                .iter()
                .any(|issue| issue.reason == Reason::InvalidGradientStops));
            assert!(layer_trees_to_pdf(&[candidate]).is_err());
        }
        let mut collapsed = gradient();
        collapsed.colors = vec![0xff, 0x00ff00, 0xff0000];
        collapsed.positions = vec![0.0, 1e-50, 1.0];
        let candidate = tree("rectangle", collapsed, bounds(), 1.0);
        assert!(report(&candidate)
            .issues
            .iter()
            .any(|issue| issue.reason == Reason::InvalidExpandedGradientStops));
        assert!(layer_trees_to_pdf(&[candidate])
            .unwrap_err()
            .contains("collapsed radial gradient stop positions"));
        for count in [0, 1, 4097] {
            let mut source = gradient();
            source.colors.resize(count, 0xff);
            source.positions.clear();
            let candidate = tree("rectangle", source, bounds(), 1.0);
            assert!(report(&candidate)
                .issues
                .iter()
                .any(|issue| issue.reason == Reason::GradientColorCountOutOfRange));
            assert!(layer_trees_to_pdf(&[candidate]).is_err());
        }
    }

    #[test]
    fn native_radial_uses_the_farthest_corner_radius_and_physical_circle_isolines() {
        let candidate = tree("rectangle", gradient(), bounds(), 1.0);
        assert_radial_vector_pdf(&candidate);
        let pixels = SkiaLayerRenderer::new()
            .render_pixels_with_options(&candidate, RasterRenderOptions::default())
            .unwrap();
        // The established corner-distance rule has center (48,32) and radius
        // sqrt(40²+24²). It is neither max(width,height)/2 nor an ellipse.
        let radius = 40_f64.hypot(24.0);
        for (x, y) in [(48, 32), (71, 32), (48, 47)] {
            let distance = (x as f64 + 0.5 - 48.0).hypot(y as f64 + 0.5 - 32.0);
            let blue = (255.0 * distance / radius).round() as u8;
            let actual = pixel(&pixels, x, y);
            assert_eq!(actual[3], 255);
            assert!(
                actual[2].abs_diff(blue) <= 4,
                "{x}/{y}: {actual:?}, blue={blue}"
            );
            assert!(actual[0].abs_diff(255 - blue) <= 4);
        }
        for channel in 0..4 {
            assert!(pixel(&pixels, 63, 32)[channel].abs_diff(pixel(&pixels, 48, 47)[channel]) <= 1);
        }
    }

    #[test]
    fn direct_pdf_radial_records_clip_and_opacity_and_native_preserves_both() {
        let renderer = SkiaLayerRenderer::new();
        for opacity in [0.25, 0.5, 1.0] {
            let mut candidate = tree("rectangle", gradient(), bounds(), opacity);
            candidate.root = LayerNode::clip_rect(
                BoundingBox::new(0.0, 0.0, 96.0, 64.0),
                None,
                BoundingBox::new(32.0, 16.0, 32.0, 32.0),
                candidate.root,
                ClipKind::Generic,
            );
            let pdf = assert_radial_vector_pdf(&candidate);
            let content = vector_streams(&pdf);
            assert!(
                content
                    .split_whitespace()
                    .any(|token| token == "W" || token == "W*"),
                "vector clip must survive recording"
            );
            if opacity < 1.0 {
                let dictionaries = String::from_utf8_lossy(&pdf);
                let has_alpha = dictionaries
                    .split("/ca ")
                    .skip(1)
                    .filter_map(|rest| rest.split_whitespace().next()?.parse::<f64>().ok())
                    .any(|alpha| alpha > 0.0 && alpha < 1.0);
                assert!(
                    has_alpha || dictionaries.contains("/SMask"),
                    "PDF must retain fractional alpha"
                );
            }
            let pixels = renderer
                .render_pixels_with_options(&candidate, RasterRenderOptions::default())
                .unwrap();
            assert_eq!(pixel(&pixels, 16, 24), [0, 0, 0, 0]);
            let inside = pixel(&pixels, 48, 32);
            assert!(inside[3].abs_diff((255.0 * opacity).round() as u8) <= 1);
            assert!(inside[0] > inside[2]);
        }
        for opacity in [-0.1, 1.1, f64::NAN] {
            let candidate = tree("rectangle", gradient(), bounds(), opacity);
            assert!(report(&candidate)
                .issues
                .iter()
                .any(|issue| issue.reason == Reason::InvalidOpacity));
            assert!(layer_trees_to_pdf(&[candidate]).is_err());
        }
        let mut invalid_clip = tree("rectangle", gradient(), bounds(), 1.0);
        invalid_clip.root = LayerNode::clip_rect(
            BoundingBox::new(0.0, 0.0, 96.0, 64.0),
            None,
            BoundingBox::new(0.0, 0.0, -1.0, 16.0),
            invalid_clip.root,
            ClipKind::Generic,
        );
        assert!(report(&invalid_clip)
            .issues
            .iter()
            .any(|issue| issue.reason == Reason::InvalidClipBounds));
        assert!(layer_trees_to_pdf(&[invalid_clip]).is_err());
        let mut missing_clip = tree("rectangle", gradient(), bounds(), 1.0);
        missing_clip.root = LayerNode {
            bounds: missing_clip.root.bounds,
            source_node_id: None,
            layer: None,
            kind: LayerNodeKind::ClipPath {
                paths: Vec::new(),
                child: Box::new(missing_clip.root),
            },
        };
        assert!(report(&missing_clip)
            .issues
            .iter()
            .any(|issue| issue.reason == Reason::InvalidTextClipPath));
        assert!(layer_trees_to_pdf(&[missing_clip]).is_err());
    }

    #[test]
    fn direct_pdf_keeps_radial_approximations_and_sksl_types_closed() {
        for kind in [0, 3, 4, 99] {
            let mut source = gradient();
            source.gradient_type = kind;
            let candidate = tree("rectangle", source, bounds(), 1.0);
            assert!(report(&candidate)
                .issues
                .iter()
                .any(|issue| issue.reason == Reason::UnsupportedGradientType));
            assert!(layer_trees_to_pdf(&[candidate])
                .unwrap_err()
                .contains("non-linear gradient fill"));
        }
    }
}
