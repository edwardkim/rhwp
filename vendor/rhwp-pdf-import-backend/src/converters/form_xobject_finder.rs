//! Locate Form XObject invocations on a PDF page and compute the
//! page-space bbox each one occupies.
//!
//! Existing image extraction in `pdf_to_ir::extract_page_images` only
//! covers raster `Image` XObjects (`/Subtype /Image`). Many cover
//! pages — federal regs (CFR), agency logos, dissertation banners —
//! draw small vector decorations as `/Subtype /Form` XObjects
//! instead. Without surfacing those we leave a visible hole in the
//! office round-trip.
//!
//! This module walks the page content stream for `Do` operators
//! whose name resolves to a Form XObject in the page resources.
//! Each invocation captures the current transformation matrix (CTM)
//! at the point of invocation; combined with the Form's own `/BBox`
//! (and optional `/Matrix`), it gives the page-space rectangle the
//! Form draws into.
//!
//! The caller then renders that rectangle as a bitmap (via
//! `pdf_oxide::rendering::render_page_region`) and embeds the result
//! into the IR as an `Image`. Output: every Form XObject on the page
//! becomes a positioned raster image at the source coordinates.

use crate::content::{parse_content_stream, Operator};
use crate::object::{Object, ObjectRef};
use std::collections::HashMap;

/// One Form-XObject invocation discovered on a page. Bounds are in
/// PDF user space (origin bottom-left, y-up).
#[derive(Debug, Clone)]
pub struct FormInvocation {
    /// Resource name the page used to invoke the Form
    /// (`Do` operand, e.g. `"Fm0"`).
    #[allow(dead_code)]
    pub name: String,
    /// PDF user-space bbox: (x_lower_left, y_lower_left, width, height).
    pub bbox_pt: (f32, f32, f32, f32),
}

/// Scan `page_idx`'s content stream for inline image (`BI…ID…EI`)
/// blocks. Each inline image's bbox is the unit square (0,0)-(1,1)
/// transformed through the current CTM at the BI point, per PDF
/// spec §8.9.7.
///
/// Inline images on cover pages are typically 1-bit raster icons —
/// agency logos, GPO marks, accessibility badges — that don't
/// warrant a full Image XObject. Without surfacing them the office
/// round-trip drops the icon entirely.
pub fn find_inline_image_invocations(
    doc: &crate::document::PdfDocument,
    page_idx: usize,
) -> Vec<FormInvocation> {
    let content_bytes = match doc.get_page_content_data(page_idx) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };
    let ops = match parse_content_stream(&content_bytes) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let mut ctm: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let mut stack: Vec<[f32; 6]> = Vec::new();
    let mut out = Vec::new();
    let mut inline_idx = 0usize;
    for op in ops {
        match op {
            Operator::SaveState => {
                stack.push(ctm);
            },
            Operator::RestoreState => {
                if let Some(prev) = stack.pop() {
                    ctm = prev;
                }
            },
            Operator::Cm { a, b, c, d, e, f } => {
                ctm = matrix_multiply(&[a, b, c, d, e, f], &ctm);
            },
            Operator::InlineImage { .. } => {
                // Inline image source space is the unit square.
                let bbox_pt = transform_bbox(&(0.0, 0.0, 1.0, 1.0), &ctm);
                out.push(FormInvocation {
                    name: format!("__inline_{inline_idx}"),
                    bbox_pt,
                });
                inline_idx += 1;
            },
            _ => {},
        }
    }
    out
}

/// Scan `page_idx`'s content stream for `Do` operators that invoke
/// Form XObjects in the page's `/Resources/XObject` dictionary.
/// Returns the page-space bbox each Form occupies.
pub fn find_form_xobject_invocations(
    doc: &crate::document::PdfDocument,
    page_idx: usize,
) -> Vec<FormInvocation> {
    // Fetch page resources/XObject dict so we can filter Do operands
    // to only Form-type XObjects (skip raster Images already handled
    // by `extract_page_images`).
    let xobject_dict = match get_page_xobjects(doc, page_idx) {
        Some(d) => d,
        None => return Vec::new(),
    };

    // Map name → Form-XObject info (BBox, optional Matrix). Names
    // not present here are either raster Images (skipped here) or
    // unresolvable refs.
    let mut form_info: HashMap<String, FormGeometry> = HashMap::new();
    for (name, value) in &xobject_dict {
        let obj_ref = match value.as_reference() {
            Some(r) => r,
            None => continue,
        };
        if let Some(geom) = load_form_geometry(doc, obj_ref) {
            form_info.insert(name.clone(), geom);
        }
    }
    if form_info.is_empty() {
        return Vec::new();
    }

    // Pull the page's content stream bytes. `get_page_content_data`
    // joins all stream parts in order and decompresses them.
    let content_bytes = match doc.get_page_content_data(page_idx) {
        Ok(b) => b,
        Err(_) => return Vec::new(),
    };
    let ops = match parse_content_stream(&content_bytes) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    // Walk the operator stream tracking the CTM. PDF spec §8.3.4:
    //   - `q` saves graphics state; `Q` restores.
    //   - `cm` concatenates a 6-element matrix to the current CTM.
    //   - `Do` invokes the named XObject, transforming its own
    //     coordinate system through the current CTM (and the form's
    //     own `/Matrix` if present) before drawing.
    let mut ctm: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    let mut stack: Vec<[f32; 6]> = Vec::new();
    let mut out = Vec::new();
    for op in ops {
        match op {
            Operator::SaveState => {
                stack.push(ctm);
            },
            Operator::RestoreState => {
                if let Some(prev) = stack.pop() {
                    ctm = prev;
                }
            },
            Operator::Cm { a, b, c, d, e, f } => {
                ctm = matrix_multiply(&[a, b, c, d, e, f], &ctm);
            },
            Operator::Do { name } => {
                if let Some(geom) = form_info.get(&name) {
                    let mut effective = ctm;
                    if let Some(form_matrix) = geom.matrix {
                        effective = matrix_multiply(&form_matrix, &ctm);
                    }
                    let bbox_pt = transform_bbox(&geom.bbox, &effective);
                    out.push(FormInvocation { name, bbox_pt });
                }
            },
            _ => {},
        }
    }
    out
}

struct FormGeometry {
    /// Form-space BBox: (llx, lly, urx, ury).
    bbox: (f32, f32, f32, f32),
    /// Optional /Matrix mapping form space → user space.
    matrix: Option<[f32; 6]>,
}

fn load_form_geometry(
    doc: &crate::document::PdfDocument,
    obj_ref: ObjectRef,
) -> Option<FormGeometry> {
    let obj = doc.load_object(obj_ref).ok()?;
    let dict = obj.as_dict()?;
    if dict.get("Subtype").and_then(|s| s.as_name()) != Some("Form") {
        return None;
    }
    let bbox_arr = dict.get("BBox").and_then(|b| b.as_array())?;
    if bbox_arr.len() != 4 {
        return None;
    }
    let bbox = (
        as_number(&bbox_arr[0])?,
        as_number(&bbox_arr[1])?,
        as_number(&bbox_arr[2])?,
        as_number(&bbox_arr[3])?,
    );
    let matrix = dict
        .get("Matrix")
        .and_then(|m| m.as_array())
        .and_then(|arr| {
            if arr.len() != 6 {
                return None;
            }
            Some([
                as_number(&arr[0])?,
                as_number(&arr[1])?,
                as_number(&arr[2])?,
                as_number(&arr[3])?,
                as_number(&arr[4])?,
                as_number(&arr[5])?,
            ])
        });
    Some(FormGeometry { bbox, matrix })
}

fn get_page_xobjects(
    doc: &crate::document::PdfDocument,
    page_idx: usize,
) -> Option<HashMap<String, Object>> {
    let page = doc.get_page(page_idx).ok()?;
    let page_dict = page.as_dict()?;
    let resources = {
        let r = page_dict.get("Resources")?;
        match r.as_reference() {
            Some(rref) => doc.load_object(rref).ok()?,
            None => r.clone(),
        }
    };
    let res_dict = resources.as_dict()?;
    let xobjects = {
        let x = res_dict.get("XObject")?;
        match x.as_reference() {
            Some(xref) => doc.load_object(xref).ok()?,
            None => x.clone(),
        }
    };
    let dict = xobjects.as_dict()?;
    Some(dict.clone())
}

fn as_number(obj: &Object) -> Option<f32> {
    match obj {
        Object::Integer(i) => Some(*i as f32),
        Object::Real(f) => Some(*f as f32),
        _ => None,
    }
}

/// PDF matrices: row-major in spec but stored column-major in
/// content streams as [a b c d e f] meaning
///   | a b 0 |
///   | c d 0 |
///   | e f 1 |
/// (concatenating: `cm` postmultiplies onto the current CTM).
fn matrix_multiply(a: &[f32; 6], b: &[f32; 6]) -> [f32; 6] {
    [
        a[0] * b[0] + a[1] * b[2],
        a[0] * b[1] + a[1] * b[3],
        a[2] * b[0] + a[3] * b[2],
        a[2] * b[1] + a[3] * b[3],
        a[4] * b[0] + a[5] * b[2] + b[4],
        a[4] * b[1] + a[5] * b[3] + b[5],
    ]
}

/// Rasterise every Form-XObject + inline-image region on `page_idx`
/// at 200 DPI as PNG bytes, returning each one paired with its
/// page-space bbox in PDF user space (origin bottom-left, y-up).
///
/// Used by both the flow-mode path (`pdf_to_ir::extract_page_images`)
/// and the layout-mode writers (`docx_layout`, `pptx_layout`,
/// `xlsx_layout`). Without it, vector figures embedded as Form
/// XObjects — common in academic papers and government docs — are
/// dropped from the office round-trip entirely.
///
/// `existing_rects_pdf` carries the bboxes of raster Image XObjects
/// already extracted by the caller (PDF y-up coords); any
/// Form/inline region that overlaps one of those by more than 50% of
/// its own area is skipped to avoid double-rasterising (e.g. the
/// NARA seal referenced both as Image and via a Form alias).
#[cfg(feature = "rendering")]
pub fn rasterize_form_and_inline_regions(
    doc: &crate::document::PdfDocument,
    page_idx: usize,
    page_h_pt: f32,
    existing_rects_pdf: &[(f32, f32, f32, f32)],
) -> Vec<((f32, f32, f32, f32), Vec<u8>)> {
    use crate::rendering::{render_page, ImageFormat as RFmt, RenderOptions};

    let mut invs = find_form_xobject_invocations(doc, page_idx);
    invs.extend(find_inline_image_invocations(doc, page_idx));

    // Same heuristics as `pdf_to_ir::extract_page_images`:
    // - drop pixel-tiny boxes (probably parser artefacts);
    // - drop boxes wider than 1.5× page height or taller than the
    //   page itself (extreme outliers, likely CTM noise);
    // - drop regions that significantly overlap an already-extracted
    //   raster Image (avoid rendering the seal twice).
    invs.retain(|inv| {
        let (_, _, w, h) = inv.bbox_pt;
        w >= 4.0 && h >= 4.0 && w < page_h_pt * 1.5 && h < page_h_pt
    });
    let overlaps = |bbox: &(f32, f32, f32, f32)| -> bool {
        let (ix, iy, iw, ih) = *bbox;
        existing_rects_pdf.iter().any(|(rx, ry, rw, rh)| {
            let l = ix.max(*rx);
            let r = (ix + iw).min(rx + rw);
            let b = iy.max(*ry);
            let t = (iy + ih).min(ry + rh);
            let inter = (r - l).max(0.0) * (t - b).max(0.0);
            let area = iw * ih;
            area > 0.0 && inter / area > 0.5
        })
    };
    invs.retain(|inv| !overlaps(&inv.bbox_pt));
    if invs.is_empty() {
        return Vec::new();
    }

    // Render the full page ONCE, then crop in image space for each
    // region. Previously we called `render_page_region` per region;
    // that helper does a full-page render + crop internally, so a
    // page with N Form-XObjects re-rendered the page N times. On a
    // 10-page LaTeX paper with 12 Form-XObjects we saw 263 seconds
    // for PDF→DOCX; with this caching it should drop by ~Nx.
    let bytes = doc.source_bytes.clone();
    if bytes.is_empty() {
        return Vec::new();
    }
    let doc_mut = match crate::document::PdfDocument::from_bytes(bytes) {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };
    let dpi: u32 = 150;
    let opts = RenderOptions {
        dpi,
        format: RFmt::Png,
        ..Default::default()
    };
    let full = match render_page(&doc_mut, page_idx, &opts) {
        Ok(i) => i,
        Err(_) => return Vec::new(),
    };
    let full_img = match image::load_from_memory(&full.data) {
        Ok(i) => i,
        Err(_) => return Vec::new(),
    };
    let scale = dpi as f32 / 72.0;
    let img_w = full_img.width();
    let img_h = full_img.height();

    let mut out = Vec::with_capacity(invs.len());
    for inv in invs {
        let (x_pdf, y_pdf, w, h) = inv.bbox_pt;
        // PDF y-up → image y-down. Image origin is top-left.
        let top_y_pt = page_h_pt - (y_pdf + h);
        let cx = (x_pdf * scale).round().max(0.0) as u32;
        let cy = (top_y_pt * scale).round().max(0.0) as u32;
        let cw = (w * scale).round().max(1.0) as u32;
        let ch = (h * scale).round().max(1.0) as u32;
        let x = cx.min(img_w.saturating_sub(1));
        let y = cy.min(img_h.saturating_sub(1));
        let cw = cw.min(img_w - x);
        let ch = ch.min(img_h - y);
        if cw == 0 || ch == 0 {
            continue;
        }
        let cropped = full_img.crop_imm(x, y, cw, ch);
        let mut buf = Vec::new();
        use image::codecs::png::{CompressionType, FilterType, PngEncoder};
        use image::ImageEncoder;
        if PngEncoder::new_with_quality(&mut buf, CompressionType::Fast, FilterType::Sub)
            .write_image(cropped.as_bytes(), cw, ch, cropped.color().into())
            .is_err()
        {
            continue;
        }
        if buf.is_empty() {
            continue;
        }
        out.push(((x_pdf, y_pdf, w, h), buf));
    }
    out
}

/// Transform a form-space BBox (llx, lly, urx, ury) through the
/// effective matrix → page-space (x, y, width, height). Computes
/// all four corners and returns the axis-aligned bounding rectangle.
fn transform_bbox(bbox: &(f32, f32, f32, f32), m: &[f32; 6]) -> (f32, f32, f32, f32) {
    let corners = [
        (bbox.0, bbox.1),
        (bbox.2, bbox.1),
        (bbox.2, bbox.3),
        (bbox.0, bbox.3),
    ];
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for (x, y) in corners {
        let tx = m[0] * x + m[2] * y + m[4];
        let ty = m[1] * x + m[3] * y + m[5];
        if tx < min_x {
            min_x = tx;
        }
        if tx > max_x {
            max_x = tx;
        }
        if ty < min_y {
            min_y = ty;
        }
        if ty > max_y {
            max_y = ty;
        }
    }
    (min_x, min_y, (max_x - min_x).max(0.0), (max_y - min_y).max(0.0))
}
