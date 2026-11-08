//! 관계형·고정형: PDF clip의 소유·alpha·저장 보존 및 합성 입력 쪽수/예산 계약.
//! 주석 갱신: 2026-11-01 10:43 KST. 직전 변경 f125c735c7, 2026-10-29T15:34:34+09:00.
//! 이번 보정 대상 consecutive 5쪽·compound 1쪽·curve 1쪽, 독립 PDFium, 96dpi raw 2px.
//! Native/fresh WASM 각 대상 전쪽 최저 모두 100.00%; 쪽수 동일, 곡선/구멍 직접 판독.
//! 증거: mydocs/working/task_m100_8413_implementation.md. 예전 unsupported 기대만 지원 계약으로 교체.
//! Independent PDF path syntax and request-budget contracts; no pixel baselines.
#![cfg(feature = "pdf-import")]
use base64::Engine;
use pdf_oxide::PdfDocument;
use rhwp::model::control::Control;
use rhwp::parser::pdf::{
    parse_pdf_with_options, parse_pdf_with_policy, PdfImportOptions, PdfImportPolicy,
    PdfImportProfile,
};
const CLIPS: &[u8] = include_bytes!("../../pdf/issue8174/clip/consecutive-moves.pdf");
const BATCH: &[u8] = include_bytes!("../../pdf/issue8174/clip/batch-operator-control.pdf");

fn assert_rectangle_crop(core: &rhwp::DocumentCore) {
    assert_eq!(core.page_count(), 5);
    for section in &core.document().sections[..3] {
        let pictures: Vec<_> = section
            .paragraphs
            .iter()
            .flat_map(|p| &p.controls)
            .filter_map(|c| {
                if let Control::Picture(p) = c {
                    Some(p)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(pictures.len(), 1, "one image stays in its owning section");
        let p = pictures[0];
        // Source image spans 10..110, source clip 20..100: retain the middle 80%.
        assert_eq!(p.crop.left * 10, p.img_dim.0 as i32);
        assert_eq!(p.crop.top * 10, p.img_dim.1 as i32);
        assert_eq!((p.crop.right - p.crop.left) * 5, p.img_dim.0 as i32 * 4);
        assert_eq!((p.crop.bottom - p.crop.top) * 5, p.img_dim.1 as i32 * 4);
    }
}

fn painted_images(core: &rhwp::DocumentCore) -> Vec<image::RgbaImage> {
    (0..5)
        .map(|page| {
            let svg = core.render_page_svg_native(page).unwrap();
            let urls: Vec<_> = svg.split("data:image/png;base64,").skip(1).collect();
            assert_eq!(
                urls.len(),
                1,
                "one source paint cannot be duplicated or omitted"
            );
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(urls[0].split('"').next().unwrap())
                .unwrap();
            image::load_from_memory(&bytes).unwrap().to_rgba8()
        })
        .collect()
}

#[test]
fn consecutive_moves_keep_the_same_rectangle_crop_and_saved_picture_ownership() {
    let core = rhwp::DocumentCore::from_pdf_bytes(CLIPS, &Default::default()).unwrap();
    assert_rectangle_crop(&core);
    let images = painted_images(&core);
    assert_eq!(images[0], images[1]);
    assert_eq!(images[0], images[2]);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let reopened = rhwp::DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(
            painted_images(&reopened),
            images,
            "pixels, alpha and page ownership survive both saves"
        );
        // HWP5 may omit the HWPX-only imgDim field; crop remains the same authored selection.
        for (original, saved) in core
            .document()
            .sections
            .iter()
            .zip(&reopened.document().sections)
        {
            let crops = |section: &rhwp::model::document::Section| {
                section
                    .paragraphs
                    .iter()
                    .flat_map(|p| &p.controls)
                    .filter_map(|c| {
                        if let Control::Picture(p) = c {
                            Some((p.crop.left, p.crop.top, p.crop.right, p.crop.bottom))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            };
            assert_eq!(crops(original), crops(saved));
        }
    }
}

#[test]
fn consecutive_moves_preserve_convex_alpha_and_affine_form_geometry() {
    let doc = PdfDocument::from_bytes_with_limits(CLIPS.to_vec(), Default::default()).unwrap();
    for page in [3, 4] {
        let handles = doc.page_image_handles(page).unwrap();
        assert_eq!(handles.len(), 1);
        let image = handles[0].decode().unwrap();
        let b = image.bbox().unwrap();
        let rgba = image.to_dynamic_image().unwrap().to_rgba8();
        let (mut inside, mut outside) = (0, 0);
        for (x, y, pixel) in rgba.enumerate_pixels() {
            let px = b.x as f64 + b.width as f64 * (x as f64 + 0.5) / rgba.width() as f64;
            let py = b.y as f64 + b.height as f64 * (1. - (y as f64 + 0.5) / rgba.height() as f64);
            let (u, v) = if page == 4 {
                let dx = px - 30.;
                (
                    (0.75 * dx + 0.25 * py) / 0.625,
                    (-0.25 * dx + 0.75 * py) / 0.625,
                )
            } else {
                (px, py)
            };
            let distance = if page == 3 {
                (u - 20.).min(v - 20.).min((120. - u - v) / 2f64.sqrt())
            } else {
                (u - 20.).min(v - 20.).min(100. - u).min(100. - v)
            };
            if distance > 1.5 {
                assert_eq!(pixel.0, [255, 0, 0, 255]);
                inside += 1;
            } else if distance < -1.5 {
                assert_eq!(pixel[3], 0, "clip must not paint its bounding rectangle");
                outside += 1;
            }
        }
        assert!(inside > 0 && outside > 0);
    }
    doc.check_import_budget().unwrap();
}

#[test]
fn real_compound_and_curve_clips_preserve_alpha_and_allow_a_followup_request() {
    for pdf in [
        include_bytes!("../../pdf/issue8174/clip/compound-rejected.pdf").as_slice(),
        include_bytes!("../../pdf/issue8174/clip/curve-rejected.pdf").as_slice(),
    ] {
        let core = rhwp::DocumentCore::from_pdf_bytes(pdf, &Default::default()).unwrap();
        let original = core.render_page_svg_native(0).unwrap();
        let pixels = |svg: &str| {
            let urls: Vec<_> = svg.split("data:image/png;base64,").skip(1).collect();
            assert_eq!(urls.len(), 1, "source image ownership survives clipping");
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(urls[0].split('"').next().unwrap())
                .unwrap();
            image::load_from_memory(&bytes).unwrap().to_rgba8()
        };
        let reference = pixels(&original);
        assert!(reference.pixels().any(|p| p[3] == 0));
        assert!(reference.pixels().any(|p| p[3] == 255));
        for bytes in [
            core.export_hwp_native().unwrap(),
            core.export_hwpx_native().unwrap(),
        ] {
            let saved = rhwp::DocumentCore::from_bytes(&bytes).unwrap();
            assert_eq!(saved.page_count(), core.page_count());
            assert_eq!(pixels(&saved.render_page_svg_native(0).unwrap()), reference);
        }
    }
    assert_eq!(
        parse_pdf_with_options(CLIPS, &Default::default())
            .unwrap()
            .metadata
            .pages
            .len(),
        5
    );
}

#[test]
fn batch_import_keeps_single_page_operator_allowance_and_lower_host_caps() {
    for page in 0..2 {
        let x = parse_pdf_with_options(
            BATCH,
            &PdfImportOptions {
                source_pages: Some(vec![page]),
                ..Default::default()
            },
        )
        .unwrap();
        assert!(x.metadata.resource_usage.operators < 250_000);
    }
    let normal = parse_pdf_with_options(BATCH, &Default::default()).unwrap();
    assert_eq!(normal.metadata.pages.len(), 2);
    assert!(normal.metadata.resource_usage.operators > 250_000);
    assert!(
        normal.metadata.resource_usage.operators
            <= normal.metadata.resource_policy.backend.operators
    );
    assert!(
        normal.metadata.resource_policy.backend.operators
            <= PdfImportPolicy::default().backend.operators
    );
    for policy in [
        PdfImportPolicy::for_profile(PdfImportProfile::Constrained),
        PdfImportPolicy::default()
            .with_limits_json("{\"backend\":{\"operators\":250000}}")
            .unwrap(),
    ] {
        let error = parse_pdf_with_policy(BATCH, &Default::default(), policy)
            .err()
            .unwrap();
        assert_eq!(error.code, "PDF_RESOURCE_LIMIT");
        let failure = error
            .diagnostics
            .unwrap()
            .resource_usage
            .unwrap()
            .failure
            .unwrap();
        assert_eq!(failure.reason, "content operators");
        assert_eq!(failure.limit, Some(250_000));
    }
}

#[test]
fn empty_moves_do_not_turn_rectangular_text_into_private_nonrectangular_metadata() {
    let input = include_bytes!("../../pdf/issue8174/clip/text-consecutive-moves.pdf");
    let core = rhwp::DocumentCore::from_pdf_bytes(input, &Default::default()).unwrap();
    fn check(core: &rhwp::DocumentCore) {
        assert_eq!(core.page_count(), 3);
        assert!(
            rhwp::model::text_clip::encode(core.document())
                .unwrap()
                .is_none(),
            "all three authored clip paths are rectangles, including q/Q and re after empty m"
        );
        fn content(paragraphs: &[rhwp::model::paragraph::Paragraph]) -> String {
            let mut result = String::new();
            for p in paragraphs {
                result.extend(p.text.chars().filter(|c| !c.is_whitespace()));
                for control in &p.controls {
                    match control {
                        Control::Shape(shape) => {
                            if let Some(tb) = shape.drawing().and_then(|d| d.text_box.as_ref()) {
                                result += &content(&tb.paragraphs);
                            }
                        }
                        Control::Table(table) => {
                            for cell in &table.cells {
                                result += &content(&cell.paragraphs);
                            }
                        }
                        _ => {}
                    }
                }
            }
            result
        }
        for section in &core.document().sections {
            assert_eq!(
                content(&section.paragraphs),
                "RECTANGLEABCDEF",
                "original scalar order and page ownership survive rectangular clip normalization"
            );
        }
    }
    check(&core);
    for saved in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        check(&rhwp::DocumentCore::from_bytes(&saved).unwrap());
    }
}
