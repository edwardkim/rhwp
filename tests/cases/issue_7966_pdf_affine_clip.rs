//! Independent convex PDF clip geometry, winding, transform and saved-image ownership.
#![cfg(feature = "pdf-import")]
use base64::Engine;
use pdf_oxide::PdfDocument;
const PDF: &[u8] = include_bytes!("../../pdf/issue7966/affine-clip-controls.pdf");

#[test]
fn affine_clip_keeps_only_the_intersection_and_preserves_image_alpha() {
    let doc = PdfDocument::from_bytes_with_limits(PDF.to_vec(), Default::default()).unwrap();
    for page in 0..3 {
        let handles = doc.page_image_handles(page).unwrap();
        assert_eq!(handles.len(), 1, "one source paint remains one image");
        let image = handles[0].decode().unwrap();
        let frame = image.bbox().unwrap();
        let rgba = image.to_dynamic_image().unwrap().to_rgba8();
        let mut visible = 0;
        let mut empty = 0;
        for (x, y, pixel) in rgba.enumerate_pixels() {
            // PDF bottom-up geometry from the independent fixture, not the clip helper.
            let px = frame.x as f64 + frame.width as f64 * (x as f64 + 0.5) / rgba.width() as f64;
            let py = frame.y as f64
                + frame.height as f64 * (1.0 - (y as f64 + 0.5) / rgba.height() as f64);
            let distances: Vec<f64> = match page {
                0 => vec![
                    px - 20.,
                    py - 20.,
                    100. - px,
                    100. - py,
                    (120. - px - py) / 2_f64.sqrt(),
                ],
                1 => {
                    let (dx, dy) = (px - 55., py - 20.);
                    let (u, v) = (0.8660254 * dx + 0.5 * dy, -0.5 * dx + 0.8660254 * dy);
                    vec![
                        u,
                        v,
                        60. - u,
                        60. - v,
                        px - 15.,
                        py - 15.,
                        115. - px,
                        115. - py,
                    ]
                }
                _ => vec![
                    px - 20.,
                    py - 20.,
                    100. - px,
                    100. - py,
                    (px + py - 120.) / 2_f64.sqrt(),
                ],
            };
            // Ignore only the subpixel edge band; interior/exterior must follow the polygon.
            let distance = distances.into_iter().fold(f64::INFINITY, f64::min);
            if distance > 1.0 {
                assert_eq!(&pixel.0[..3], &[255, 0, 0]);
                assert_eq!(pixel[3], if page == 2 { 128 } else { 255 });
                visible += 1;
            } else if distance < -1.0 {
                assert_eq!(
                    pixel[3], 0,
                    "paint leaked beyond the actual convex clip on page {page}"
                );
                empty += 1;
            }
        }
        assert!(
            visible > 0 && empty > 0,
            "neither a blank image nor the unclipped bounding rectangle is valid"
        );
    }
    doc.check_import_budget().unwrap();
}

fn images(core: &rhwp::DocumentCore) -> Vec<image::RgbaImage> {
    assert_eq!(core.page_count(), 3);
    (0..3)
        .map(|page| {
            let svg = core.render_page_svg_native(page).unwrap();
            let embedded: Vec<_> = svg.split("data:image/png;base64,").skip(1).collect();
            assert_eq!(
                embedded.len(),
                1,
                "clip fragments must not duplicate the source image"
            );
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(embedded[0].split('"').next().unwrap())
                .unwrap();
            image::load_from_memory(&bytes).unwrap().to_rgba8()
        })
        .collect()
}
#[test]
fn affine_clipped_paint_survives_lowering_and_both_saved_formats() {
    let core = rhwp::DocumentCore::from_pdf_bytes(PDF, &Default::default()).unwrap();
    let expected = images(&core);
    for rgba in &expected {
        assert!(rgba.pixels().any(|p| p[3] == 0));
        assert!(rgba
            .pixels()
            .any(|p| p[3] > 0 && p[0] > 240 && p[1] < 10 && p[2] < 10));
    }
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        assert_eq!(
            images(&rhwp::DocumentCore::from_bytes(&bytes).unwrap()),
            expected,
            "image samples, alpha and single-paint ownership survive save/reopen"
        );
    }
}
