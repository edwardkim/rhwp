//! 관계형·고정형(원본 속성): PDF 곡선·마스크·회전·색 출처와 저장 내용 보존. 페이지 px 핀 없음.
//! 주석 갱신: 2026-11-01 11:12 KST. 신규 파일.
//! 합성 PDF 13쪽 / 독립 PDFium 원본 출력, 96dpi, 색상 경계 보정 전 2px raw.
//! 최종 Native 전쪽 최저 98.88%, fresh WASM 98.97%; 쪽수 양쪽 13/13.
//! 근거: mydocs/working/task_m100_8413_implementation.md 및 pdf/issue8413/README.md.
//! 곡선/구멍/회전/Type3/색 전달 직접 판독. 증거: mydocs/pr/assets/issue8413/curve-mask-validation-20261101/.
#![cfg(feature = "pdf-import")]
use pdf_oxide::PdfDocument;
use rhwp::{
    model::{control::Control, shape::ShapeObject},
    paint::{LayerNode, LayerNodeKind, PaintOp, RenderProfile},
    DocumentCore,
};
const PDF: &[u8] = include_bytes!("../../pdf/issue8413/curve-mask-controls.pdf");

fn decoded(doc: &PdfDocument, page: usize) -> image::RgbaImage {
    let handles = doc.page_image_handles(page).unwrap();
    assert_eq!(handles.len(), 1, "source paint stays in its owning page");
    handles[0]
        .decode()
        .unwrap()
        .to_dynamic_image()
        .unwrap()
        .to_rgba8()
}

#[test]
fn curved_compound_and_form_clips_preserve_inside_outside_and_winding() {
    let doc = PdfDocument::from_bytes_with_limits(PDF.to_vec(), Default::default()).unwrap();
    for page in [0, 9, 10] {
        let rgba = decoded(&doc, page);
        assert!(
            rgba.pixels().any(|p| p[3] == 0),
            "clip exterior remains transparent"
        );
        assert!(
            rgba.pixels().any(|p| p[3] == 255),
            "interior retains opaque image samples"
        );
    }
    let circle = decoded(&doc, 0);
    assert_eq!(
        circle.get_pixel(circle.width() / 2, circle.height() / 2)[3],
        255
    );
    assert_eq!(
        circle.get_pixel(0, 0)[3],
        0,
        "bounding rectangle is not the curve"
    );
    let even_odd = decoded(&doc, 1);
    let winding = decoded(&doc, 2);
    assert_eq!(
        even_odd, winding,
        "opposite winding and even-odd describe the same hole"
    );
    assert_eq!(
        even_odd.get_pixel(even_odd.width() / 2, even_odd.height() / 2)[3],
        0
    );
    assert!(even_odd.pixels().any(|p| p[3] == 255));
    let outside = doc.page_image_handles(3).unwrap();
    assert_eq!(
        outside.len(),
        2,
        "Q restores the clip for the following image"
    );
    assert!(outside[1]
        .decode()
        .unwrap()
        .to_dynamic_image()
        .unwrap()
        .to_rgba8()
        .pixels()
        .all(|p| p[3] == 255));
    doc.check_import_budget().unwrap();
}

#[test]
fn flate_predictor_and_gray_jpeg_soft_masks_decode_the_same_alpha() {
    let doc = PdfDocument::from_bytes_with_limits(PDF.to_vec(), Default::default()).unwrap();
    let reference = decoded(&doc, 4);
    assert_eq!(reference, decoded(&doc, 5));
    assert_eq!(reference, decoded(&doc, 6));
    let alpha: Vec<_> = reference.pixels().map(|p| p[3]).collect();
    assert!(alpha.contains(&0) && alpha.contains(&255));
    assert!(
        alpha.iter().any(|a| *a > 0 && *a < 255),
        "continuous alpha is not a binary mask"
    );
    doc.check_import_budget().unwrap();
}

fn text_contract(core: &DocumentCore) {
    let doc = core.document();
    let mut rotated = String::new();
    let mut invisible = String::new();
    let mut visible = String::new();
    for (page, section) in doc.sections.iter().enumerate() {
        for control in section.paragraphs.iter().flat_map(|p| &p.controls) {
            let Control::Shape(shape) = control else {
                continue;
            };
            let ShapeObject::Rectangle(rect) = shape.as_ref() else {
                continue;
            };
            let Some(text) = &rect.drawing.text_box else {
                continue;
            };
            for p in &text.paragraphs {
                if page == 7 && p.text.contains("Rotate") {
                    rotated.push_str(&p.text);
                    assert_eq!(
                        rect.drawing.shape_attr.rotation_angle,
                        (-90_i16).rem_euclid(360),
                        "PDF y-up rotation becomes the HWP y-down shape angle"
                    );
                    for run in &p.char_shapes {
                        let cs = &doc.doc_info.char_shapes[run.char_shape_id as usize];
                        assert_eq!(
                            cs.ratios, [80; 7],
                            "source basis 12/15 preserves editable width ratio"
                        );
                    }
                }
                if page == 11 {
                    for (i, ch) in p.text.chars().enumerate() {
                        let cs = &doc.doc_info.char_shapes[p.char_shape_id_at(i).unwrap() as usize];
                        if cs
                            .text_paint
                            .is_some_and(|paint| !paint.fill && paint.width_mpt == 0)
                        {
                            invisible.push(ch);
                        } else {
                            visible.push(ch);
                        }
                    }
                }
            }
        }
    }
    assert_eq!(rotated, "Rotate");
    assert_eq!(
        invisible, "AB",
        "metrics-only Type3 strings remain editable without invented glyphs"
    );
    assert_eq!(
        visible, "Visible",
        "ordinary painted glyphs retain their ink"
    );
}

fn colours(node: &LayerNode, output: &mut Vec<image::RgbaImage>) {
    match &node.kind {
        LayerNodeKind::Group { children, .. } => {
            for child in children {
                colours(child, output);
            }
        }
        LayerNodeKind::ClipRect { child, .. } | LayerNodeKind::ClipPath { child, .. } => {
            colours(child, output)
        }
        LayerNodeKind::Leaf { ops } => {
            for op in ops {
                if let PaintOp::Image {
                    image, resolved, ..
                } = op
                {
                    assert!(
                        !image.hancom2024_print_bitmap,
                        "PDF sample colour is already converted"
                    );
                    let bytes = resolved
                        .as_ref()
                        .map(|p| p.data.as_slice())
                        .or(image.data.as_deref())
                        .unwrap();
                    output.push(image::load_from_memory(bytes).unwrap().to_rgba8());
                }
            }
        }
    }
}

fn colour_contract(core: &mut DocumentCore) {
    core.set_hangul2024_compat(true);
    let tree = core
        .build_page_layer_tree_with_profile(12, RenderProfile::Print)
        .unwrap();
    let mut images = Vec::new();
    colours(&tree.root, &mut images);
    assert_eq!(images.len(), 1);
    let rgb = &images[0];
    // Source PDF has a uniform upper half and >256 colours below. A Hancom
    // RGB555/JPEG transfer would change these samples despite neutral effects.
    assert!(rgb
        .rows()
        .take(rgb.height() as usize / 2)
        .flatten()
        .all(|p| p.0 == [239, 239, 239, 255]));
}

#[test]
fn rotated_text_type3_visibility_and_pdf_colours_survive_both_saves() {
    let mut core = DocumentCore::from_pdf_bytes(PDF, &Default::default()).unwrap();
    text_contract(&core);
    colour_contract(&mut core);
    for bytes in [
        core.export_hwp_native().unwrap(),
        core.export_hwpx_native().unwrap(),
    ] {
        let mut reopened = DocumentCore::from_bytes(&bytes).unwrap();
        assert_eq!(reopened.page_count(), core.page_count());
        text_contract(&reopened);
        colour_contract(&mut reopened);
    }
}

#[test]
fn signed_rectangles_bound_both_endpoints_of_each_painted_subpath() {
    use pdf_oxide::elements::PathOperation;
    let doc = PdfDocument::from_bytes_with_limits(PDF.to_vec(), Default::default()).unwrap();
    let paths = doc.extract_paths(8).unwrap();
    let mut signed = 0;
    for path in paths {
        for operation in &path.operations {
            if let PathOperation::Rectangle(x, y, w, h) = *operation {
                signed += usize::from(w < 0. || h < 0.);
                for (px, py) in [(x, y), (x + w, y + h)] {
                    assert!(px >= path.bbox.x && px <= path.bbox.x + path.bbox.width);
                    assert!(py >= path.bbox.y && py <= path.bbox.y + path.bbox.height);
                }
            }
        }
    }
    assert!(
        signed > 0,
        "counterexample exercises signed source rectangles"
    );
    doc.check_import_budget().unwrap();
}

#[test]
fn type3_visibility_requires_proof_for_every_glyph_program() {
    use pdf_oxide::{fonts::FontInfo, object::Object};
    let doc = PdfDocument::from_bytes_with_limits(PDF.to_vec(), Default::default()).unwrap();
    let page = doc.get_page(11).unwrap();
    let resources = doc
        .resolve_object(page.as_dict().unwrap().get("Resources").unwrap())
        .unwrap();
    let fonts = doc
        .resolve_object(resources.as_dict().unwrap().get("Font").unwrap())
        .unwrap();
    let original = doc
        .resolve_object(fonts.as_dict().unwrap().get("Empty").unwrap())
        .unwrap();
    assert!(
        FontInfo::from_dict(&original, &doc)
            .unwrap()
            .type3_metrics_only
    );
    // These are glyph-program classification contracts, not raster baselines.
    // One painted/unknown/empty procedure cannot prove the whole font invisible.
    for (program, non_painting) in [
        ("0.6 0 d0", true),
        ("0.6 0 0 0 0.6 1 d1 0 0 1 1 re f", false),
        ("0.6 0 d0 /Unknown Do", false),
        ("", false),
    ] {
        let mut changed = original.as_dict().unwrap().clone();
        let mut procedures = doc
            .resolve_object(changed.get("CharProcs").unwrap())
            .unwrap()
            .as_dict()
            .unwrap()
            .clone();
        procedures.insert(
            "A".into(),
            Object::Stream {
                dict: [("Length".into(), Object::Integer(program.len() as i64))].into(),
                data: program.as_bytes().to_vec().into(),
            },
        );
        changed.insert("CharProcs".into(), Object::Dictionary(procedures));
        assert_eq!(
            FontInfo::from_dict(&Object::Dictionary(changed), &doc)
                .unwrap()
                .type3_metrics_only,
            non_painting,
            "program: {program}"
        );
    }
    doc.check_import_budget().unwrap();
}
