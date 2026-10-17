//! Separation plate renderer.
//!
//! Renders individual ink separation plates as grayscale images where
//! pixel intensity represents the tint percentage of that ink at each point.
//! Used in prepress workflows, ink coverage analysis, and ML pipelines
//! that process packaging/label PDFs.
//!
//! # ICCBased heuristic
//!
//! When a fill color space resolves to an `ICCBased` array, this renderer
//! does **not** parse the embedded ICC profile. Instead it inspects the
//! component count of the current fill/stroke color: a 4-component
//! `ICCBased` space is treated as CMYK (component order C, M, Y, K), a
//! 3-component space is treated as RGB (skipped — no separation routing),
//! and a 1-component space is treated as Gray (skipped). This matches
//! the convention used by Adobe Illustrator and InDesign when exporting
//! to PDF/X-1a and PDF/X-4 with CMYK working spaces. PDFs that rely on
//! lab-CMYK profile interpretation for separation routing are out of
//! scope for this renderer; they are rare in prepress workflows that
//! ship separated artwork.
//!
//! # Images
//!
//! Raster image XObjects (`Do` with `Subtype /Image`) are routed into
//! separation plates per ISO 32000-1 §8.9:
//!
//! - **DeviceCMYK** and **ICCBased N=4** images: per-pixel C / M / Y / K
//!   samples route to the Cyan / Magenta / Yellow / Black plates. JPEG-
//!   encoded streams decode through
//!   `crate::extractors::images::decode_cmyk_jpeg_to_raw_cmyk` which
//!   preserves the Adobe APP14 inversion semantics so plate values are
//!   physical ink coverage (0 = no ink, 255 = full).
//! - **Separation /\<spot-ink\>**: the single sample channel routes to
//!   the named spot plate.
//! - **DeviceN [\<ink1\> \<ink2\> …]**: each sample channel routes to its
//!   named plate; the `tintTransform` function is not consulted —
//!   samples go directly to plates, which is the standard prepress
//!   per-plate routing convention.
//! - **Image masks** (`/ImageMask true`): the 1-bpc samples are a
//!   stencil through which the current non-stroking colour is painted.
//!   Per-plate routing uses the same `tint_for_ink` decision tree as
//!   vector fills, so `/All`, `/None`, and spot/process semantics match
//!   the rest of the renderer.
//! - **DeviceRGB / DeviceGray / ICCBased N∈{1,3}** images: skipped.
//!   RGB/Gray have no declared ink-coverage intent in the subtractive
//!   output model, so they neither paint nor knock out plates. Matches
//!   `tint_for_ink`'s vector handling.
//! - **JPX (JPEG 2000) image XObjects**: logged and skipped. No pure-
//!   Rust JP2 decoder is bundled.
//! - **Indexed images** (`[/Indexed …]`): expanded to RGB upstream and
//!   therefore skipped by separation routing for now. Indexed CMYK
//!   palettes would need a separate `expand_indexed_to_cmyk` path.
//!
//! ICC profiles (per-image and document `/OutputIntents`) and TRC /
//! BG / UCR functions are **not** consulted when routing image samples
//! to plates; samples are written verbatim. The plate is an absolute
//! ink-coverage measurement independent of any colour-management
//! transform.
//!
//! Spot / DeviceN ink *declarations* in nested Form XObject `/Resources`
//! are surfaced as plates via
//! [`crate::document::PdfDocument::get_page_inks_deep`] even when the
//! form's local content stream doesn't paint them.
//!
//! # Limitations
//!
//! The following classes of content are recognised by the operator
//! walker but not actually painted into the plate:
//!
//! - **Shading patterns** (`sh` operator) — gradients used as fills.
//! - **Tiling and shading patterns** invoked via `scn` / `SCN` with a
//!   `/Pattern` colour space.
//! - **Inline images** (`BI` / `ID` / `EI`) — prepress artwork uses
//!   XObjects exclusively.
//! - **Page annotations.** [`render_separations`] renders only the
//!   page's content stream; annotation appearance streams are not
//!   walked, in contrast to [`super::page_renderer`] which composites
//!   annotation appearances on top of the page.
//!
//! These are intentional v1 omissions: the primary use case is
//! vector and image-based prepress artwork (dielines, varnish layers,
//! spot-PMS text and shapes, CMYK photographs, spot-ink-tinted images).
//!
//! # Transparency
//!
//! Plate output is opaque: the renderer treats `fill_alpha` / `stroke_alpha`
//! from ExtGState (`/CA`, `/ca`) and the blend mode (`/BM`) as if both were
//! `1.0` / `Normal`. This is intentional — a separation plate represents ink
//! coverage on the press, not transparent compositing. Callers who need the
//! transparent intent (e.g. a 50%-alpha spot text overlay) should evaluate it
//! against the underlying content with [`super::page_renderer`] first.
//!
//! # Overprint
//!
//! The renderer implements the per-plate overprint model defined in
//! ISO 32000-1 §11.7.4 ("Overprint Control"). The ExtGState entries
//! `/OP` (stroke), `/op` (non-stroke), and `/OPM` (overprint mode) are
//! parsed and applied to the graphics state.
//!
//! - **Default (`OP = false`):** for every plate, the spec rule "areas
//!   of unspecified colorants are erased (painted with a tint value of
//!   0.0)" applies. A DeviceCMYK fill knocks out underlying Cyan,
//!   Magenta, Yellow, Black, *and* any spot inks within its shape; a
//!   Separation `/Pantone-185` fill knocks out underlying process and
//!   other-spot plates within its shape. This is the standard
//!   per-plate prepress convention.
//! - **`OP = true`:** plates outside the source's colorant set are left
//!   untouched. Designers use this to overlay spot inks on process
//!   backgrounds without knocking them out (the typical packaging /
//!   label authoring workflow).
//! - **`OPM = 1` (Adobe nonzero overprint):** when the source colour
//!   space is DeviceCMYK and overprint is enabled, a component value of
//!   exactly `0.0` is treated as "colorant not specified" — the
//!   matching plate is left untouched. Per §11.7.4.3, OPM applies only
//!   to DeviceCMYK sources; Separation and DeviceN content is
//!   unaffected by OPM and routes through OP/op alone.
//!
//! Overprint state participates in `q`/`Q` save/restore via the existing
//! graphics-state stack and propagates into Form XObjects per §8.10.1.
//! The decision happens in `tint_for_ink`, which returns either
//! `PaintAction::Paint(tint)` (write tint into the plate; 0.0 = knockout)
//! or `PaintAction::Skip` (leave the plate untouched). Spot/DeviceN
//! sources route to their named plates regardless of overprint, matching
//! the inherent behavior of real separation devices.
#![allow(
    clippy::field_reassign_with_default,
    clippy::ptr_arg,
    clippy::only_used_in_recursion
)]

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tiny_skia::{FillRule, Mask, PathBuilder, Pixmap, Transform};

use crate::content::graphics_state::{GraphicsState, GraphicsStateStack, Matrix};
use crate::content::operators::{Operator, TextElement};
use crate::content::parser::parse_content_stream;
use crate::document::PdfDocument;
use crate::error::{Error, Result};
use crate::fonts::FontInfo;
use crate::object::Object;

use super::ext_gstate::{parse_ext_g_state_inner, ParsedExtGState};
use super::resolution::{
    InkName, PaintBackend, PaintIntent, PaintKind, PaintSide, ResolutionContext,
    ResolutionPipeline, SeparationBackend, SeparationSurface,
};
use super::text_rasterizer::TextRasterizer;
use super::{device_bounds_rasterizable, guarded_fill_path, guarded_stroke_path};
use crate::rendering::resolution::{DeviceColor, LogicalColor};
use smallvec::SmallVec;

/// A rendered separation plate for a single ink.
///
/// The pixel convention is **ML/QC-friendly**: `value == ink coverage`.
/// 0 means no ink on paper at that pixel, 255 means full tint coverage.
/// To display the plate as black ink on white paper (prepress viewer
/// convention) invert before showing: `display = 255 - value`.
#[derive(Debug, Clone)]
pub struct SeparationPlate {
    /// Ink name (e.g., "Cyan", "PANTONE 185 C", "Dieline").
    pub ink_name: String,
    /// Grayscale pixel data, row-major, top-left origin.
    /// 0 = no ink, 255 = full tint. `data.len() == width * height`.
    pub data: Vec<u8>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
}

/// Render all separation plates for a page.
///
/// Returns one [`SeparationPlate`] per ink. Process inks (Cyan, Magenta,
/// Yellow, Black) are always emitted; if the page uses no CMYK content
/// those plates will be all-zero. Spot inks are emitted only when the
/// page's resource dictionary declares a `Separation` or `DeviceN` colour
/// space that names them.
///
/// Each plate is a grayscale image where pixel intensity equals the
/// tint percentage of that ink (255 = full tint, 0 = no ink).
///
/// # Performance
///
/// The content stream is parsed **once** and the operator walk dispatches
/// paint operations to all referenced plates in parallel. Form XObjects
/// are also recursed into once per page. Unreferenced inks short-circuit
/// to an all-zero plate before any pixmap is allocated.
pub fn render_separations(
    doc: &PdfDocument,
    page_num: usize,
    dpi: u32,
) -> Result<Vec<SeparationPlate>> {
    let inks = collect_page_inks(doc, page_num)?;
    if inks.is_empty() {
        return Ok(Vec::new());
    }

    // Pre-parse the content stream once to detect which inks are actually
    // referenced. Plates for unreferenced inks short-circuit to an empty
    // pixmap and skip the per-plate operator walk entirely (O6).
    let referenced = collect_referenced_inks(doc, page_num)?;

    render_plates_for_inks(doc, page_num, dpi, &inks, &referenced)
}

/// Render a single ink separation plate for a page.
///
/// Returns a grayscale image where pixel intensity = tint percentage
/// of the named ink. If the ink is not present on the page, the plate
/// is all zeros.
///
/// This is a thin wrapper over the multi-ink path; if you need every
/// plate on a page, call [`render_separations`] instead — it walks the
/// content stream once for all inks together.
pub fn render_separation(
    doc: &PdfDocument,
    page_num: usize,
    ink_name: &str,
    dpi: u32,
) -> Result<SeparationPlate> {
    // Always walk operators for the requested ink — the per-page short-circuit
    // in [`render_separations`] is an optimisation that scans the resource
    // declarations to skip inks that are *definitely* unused. For the single-ink
    // entry point the caller has already named the ink they want, and the
    // scanner can miss inks reached via DefaultRGB/DefaultGray remapping
    // through colour operators like `rg`/`g`. Treat the named ink as referenced
    // and let the operator walk produce an honest plate.
    let inks = vec![ink_name.to_string()];
    let referenced = inks.clone();
    let mut plates = render_plates_for_inks(doc, page_num, dpi, &inks, &referenced)?;
    plates
        .pop()
        .ok_or_else(|| Error::InvalidPdf("render_separation: no plate produced".to_string()))
}

/// Core multi-ink rendering: allocate one pixmap per referenced ink,
/// walk the content stream once, and extract grayscale data from each.
///
/// ISO 32000-1 §11.7.3 + §11.7.4.2 mandate composite-then-separate
/// when the page declares any transparency or overprint trigger: the
/// §11.4 composite buffer must be evaluated first (process lanes in
/// the page-group blend space, spot lanes as a §11.7.3 sidecar with
/// the §11.7.4.2 BM split), and only after every blend / SMask /
/// knockout has resolved is the result decomposed into per-plate
/// §10.5 output. The detection-gate dispatch at the top of this
/// function picks that path for any page that fires the round-1
/// detection helper. For detection-OFF pages the per-plate walker
/// stays — it is byte-identical to a "no transparency" render at the
/// pixel level and avoids a needless ICC + spot-sidecar allocation.
fn render_plates_for_inks(
    doc: &PdfDocument,
    page_num: usize,
    dpi: u32,
    inks: &[String],
    referenced: &[String],
) -> Result<Vec<SeparationPlate>> {
    // Detection-gate dispatch: route detection-ON (transparency-only)
    // pages through the page renderer's composite path so the §11.4
    // transparency model produces the per-plate buffer the per-plate
    // walker (which is SMask-blind, /K-blind, and BM-blind by design)
    // cannot. Pure-overprint pages without any transparency trigger
    // stay on the per-plate walker — its `tint_for_ink` already
    // implements §11.7.4 OP / OPM correctly per-plate, and the
    // composite path's overprint handler is RGB-composite-oriented
    // (its OPM=0 rule additively merges plates, which is wrong for
    // per-plate output where OPM=0 means "replace per-plate"). The
    // gate uses the transparency-only helper for that reason.
    let resources = doc.get_page_resources(page_num)?;
    if crate::rendering::sidecar::page_declares_transparency(doc, &resources) {
        return render_plates_via_composite(doc, page_num, dpi, inks, referenced);
    }

    let (width, height, base_transform) = compute_page_extent(doc, page_num, dpi)?;

    // Partition inks into "needs rendering" vs "short-circuit to empty plate".
    // We track the original index so the output order matches `inks`.
    let mut render_indices: Vec<usize> = Vec::new();
    let mut empty_indices: Vec<usize> = Vec::new();
    for (i, ink) in inks.iter().enumerate() {
        if referenced.iter().any(|r| r == ink) {
            render_indices.push(i);
        } else {
            empty_indices.push(i);
        }
    }

    // Build pixmaps and a parallel `target_inks` slice for the inks we
    // actually need to walk operators for.
    let mut pixmaps: Vec<Pixmap> = Vec::with_capacity(render_indices.len());
    for _ in &render_indices {
        let pixmap = Pixmap::new(width, height)
            .ok_or_else(|| Error::InvalidPdf("Failed to create separation pixmap".to_string()))?;
        pixmaps.push(pixmap);
    }
    let target_inks: Vec<&str> = render_indices.iter().map(|&i| inks[i].as_str()).collect();

    if !pixmaps.is_empty() {
        let color_spaces = load_color_spaces(doc, &resources)?;
        let fonts = load_fonts(doc, &resources);
        let text_rasterizer = TextRasterizer::new();

        let content_data = doc.get_page_content_data(page_num)?;
        let operators = parse_content_stream(&content_data)?;

        // Same baseline the composite renderer uses (§8.11.4): the document's
        // own /OCProperties /D BaseState//ON//OFF configuration. Computed here
        // rather than passed in, because the separation entry points take no
        // render options — and the defect this closes is a layer the *document*
        // hides being counted as ink.
        let excluded_layers = crate::optional_content::compute_default_off_ocgs(doc);

        let mut ctx = SeparationContext {
            doc,
            text_rasterizer: &text_rasterizer,
            fonts: &fonts,
            excluded_layers: &excluded_layers,
        };

        execute_separation_operators(
            &mut pixmaps,
            base_transform,
            &operators,
            &mut ctx,
            &resources,
            &color_spaces,
            None,
            &target_inks,
        )?;
    }

    // Re-assemble in original ink order: empty plates for unreferenced
    // inks, extracted R channel for rendered ones.
    let pixel_count = (width as usize) * (height as usize);
    let mut result: Vec<Option<SeparationPlate>> = (0..inks.len()).map(|_| None).collect();

    for (k, &i) in render_indices.iter().enumerate() {
        let mut data = vec![0u8; pixel_count];
        let rgba = pixmaps[k].data();
        for j in 0..pixel_count {
            data[j] = rgba[j * 4];
        }
        result[i] = Some(SeparationPlate {
            ink_name: inks[i].clone(),
            data,
            width,
            height,
        });
    }
    for &i in &empty_indices {
        result[i] = Some(SeparationPlate {
            ink_name: inks[i].clone(),
            data: vec![0u8; pixel_count],
            width,
            height,
        });
    }

    Ok(result
        .into_iter()
        .map(|o| o.expect("plate filled"))
        .collect())
}

/// Composite-then-separate path. Invoked when the page declares any
/// transparency or overprint trigger (round-1 detection helper).
///
/// ISO 32000-1 §11.7.3 — composite the §11.4 transparency model in the
/// process blend space (CMYK), with spot lanes riding alongside per
/// §11.7.3 / §11.7.4.2, then extract per-plate output for the
/// requested ink set per §10.5. Concretely:
///
/// 1. Drive the page renderer's composite path with
///    `force_cmyk_sidecar = true` so the §11.4 buffer survives the
///    render regardless of `OutputIntent` presence.
/// 2. Harvest the populated sidecar
///    ([`PageRenderer::take_cmyk_sidecar`]).
/// 3. For each requested ink: if it matches a process colorant
///    ("Cyan", "Magenta", "Yellow", "Black") read the CMYK channel
///    from `process_plate`; otherwise look up the spot plane from
///    `spot_plate`. Inks neither named in the sidecar's spot set nor
///    matching a process colorant produce an all-zero plate (the
///    §8.6.6.3 "no plate" semantic).
///
/// The `referenced` set is honoured for the same short-circuit the
/// per-plate path uses — an ink that is not referenced on the page
/// produces an all-zero plate without consulting the sidecar.
fn render_plates_via_composite(
    doc: &PdfDocument,
    page_num: usize,
    dpi: u32,
    inks: &[String],
    referenced: &[String],
) -> Result<Vec<SeparationPlate>> {
    use crate::rendering::page_renderer::{PageRenderer, RenderOptions};

    let mut renderer = PageRenderer::new(RenderOptions::with_dpi(dpi).as_raw());
    renderer.force_cmyk_sidecar = true;
    let rendered = renderer.render_page(doc, page_num)?;
    let width = rendered.width;
    let height = rendered.height;
    let pixel_count = (width as usize) * (height as usize);

    // The composite path may decline to allocate a sidecar if the
    // detection trigger flickers off between the separation entry
    // point and the renderer (e.g. a page whose resources hash
    // resolved without ExtGState). Fall back to an all-zero stack —
    // this matches the per-plate walker's behaviour on a page that
    // declares no inks.
    let sidecar = renderer.take_cmyk_sidecar();
    let mut plates: Vec<SeparationPlate> = Vec::with_capacity(inks.len());
    for ink in inks {
        let mut data = vec![0u8; pixel_count];
        // §8.6.6.3 "no plate" branch — unreferenced inks short-circuit.
        if !referenced.iter().any(|r| r == ink) {
            plates.push(SeparationPlate {
                ink_name: ink.clone(),
                data,
                width,
                height,
            });
            continue;
        }
        if let Some(s) = sidecar.as_ref() {
            if matches!(ink.as_str(), "Cyan" | "Magenta" | "Yellow" | "Black") {
                if let Some(plate) = s.process_plate(ink) {
                    data = plate;
                }
            } else if let Some(lane) = s.spot_plate(ink) {
                data = lane.to_vec();
            }
        }
        plates.push(SeparationPlate {
            ink_name: ink.clone(),
            data,
            width,
            height,
        });
    }
    Ok(plates)
}

/// Collect all ink names present on a page.
///
/// CMYK is always returned regardless of whether the page actually uses
/// CMYK content; unused process plates are filtered out by the per-plate
/// short-circuit in [`render_separations`].
///
/// Spot inks come from [`PdfDocument::get_page_inks_deep`], which walks
/// the page's content stream into nested Form XObjects (§8.10) so spots
/// declared in form-local resources are discovered.
fn collect_page_inks(doc: &PdfDocument, page_num: usize) -> Result<Vec<String>> {
    let mut inks = vec![
        "Cyan".to_string(),
        "Magenta".to_string(),
        "Yellow".to_string(),
        "Black".to_string(),
    ];

    let spot_inks = doc.get_page_inks_deep(page_num)?;
    for ink in spot_inks {
        if !inks.contains(&ink) {
            inks.push(ink);
        }
    }

    Ok(inks)
}

/// Walk the content stream (and any Form XObjects it references) and
/// collect every ink name that could possibly appear on the page.
fn collect_referenced_inks(doc: &PdfDocument, page_num: usize) -> Result<Vec<String>> {
    let resources = doc.get_page_resources(page_num)?;
    let color_spaces = load_color_spaces(doc, &resources)?;
    let content_data = doc.get_page_content_data(page_num)?;
    let operators = parse_content_stream(&content_data)?;
    let mut referenced: Vec<String> = Vec::new();
    let mut visited: Vec<String> = Vec::new();
    scan_operators_for_inks(
        &operators,
        doc,
        &resources,
        &color_spaces,
        &mut referenced,
        &mut visited,
    )?;
    Ok(referenced)
}

fn scan_operators_for_inks(
    operators: &[Operator],
    doc: &PdfDocument,
    resources: &Object,
    color_spaces: &HashMap<String, Object>,
    referenced: &mut Vec<String>,
    visited: &mut Vec<String>,
) -> Result<()> {
    let xobjects = match resources {
        Object::Dictionary(rd) => rd.get("XObject").and_then(|o| doc.resolve_object(o).ok()),
        _ => None,
    };

    let push = |list: &mut Vec<String>, name: &str| {
        if !list.iter().any(|s| s == name) {
            list.push(name.to_string());
        }
    };

    for op in operators {
        match op {
            Operator::SetFillCmyk { .. } | Operator::SetStrokeCmyk { .. } => {
                push(referenced, "Cyan");
                push(referenced, "Magenta");
                push(referenced, "Yellow");
                push(referenced, "Black");
            },
            Operator::SetFillColorSpace { name } | Operator::SetStrokeColorSpace { name } => {
                inks_from_space(name, color_spaces, resources, doc, referenced);
            },
            Operator::Do { name } => {
                if visited.iter().any(|s| s == name) {
                    continue;
                }
                visited.push(name.clone());
                if let Some(xobj_dict) = xobjects.as_ref().and_then(|o| o.as_dict()) {
                    if let Some(xobj_ref_obj) = xobj_dict.get(name) {
                        if let Ok(xobj) = doc.resolve_object(xobj_ref_obj) {
                            if let Object::Stream { ref dict, .. } = xobj {
                                let subtype = dict.get("Subtype").and_then(|o| o.as_name());
                                if subtype == Some("Form") {
                                    let stream_data = if let Some(r) = xobj_ref_obj.as_reference() {
                                        doc.decode_stream_with_encryption(&xobj, r)?
                                    } else {
                                        xobj.decode_stream_data()?
                                    };
                                    let form_resources = if let Some(res) = dict.get("Resources") {
                                        doc.resolve_object(res)?
                                    } else {
                                        resources.clone()
                                    };
                                    let form_cs = load_color_spaces(doc, &form_resources)?;
                                    let mut merged_cs = color_spaces.clone();
                                    merged_cs.extend(form_cs);
                                    if let Ok(form_ops) = parse_content_stream(&stream_data) {
                                        scan_operators_for_inks(
                                            &form_ops,
                                            doc,
                                            &form_resources,
                                            &merged_cs,
                                            referenced,
                                            visited,
                                        )?;
                                    }
                                } else if subtype == Some("Image") {
                                    // §8.9: image XObjects carry their own
                                    // /ColorSpace declaration and contribute
                                    // their colorants without needing a
                                    // colour-setting operator in the content
                                    // stream. Surface those inks so the
                                    // per-plate short-circuit doesn't drop
                                    // the image's plates as empty.
                                    let resolved = resolve_image_color_space(
                                        dict,
                                        color_spaces,
                                        resources,
                                        doc,
                                    );
                                    match resolved {
                                        ResolvedSpace::Cmyk | ResolvedSpace::IccCmyk => {
                                            push(referenced, "Cyan");
                                            push(referenced, "Magenta");
                                            push(referenced, "Yellow");
                                            push(referenced, "Black");
                                        },
                                        ResolvedSpace::Separation(ink) => {
                                            if ink != "None" && !ink.is_empty() {
                                                if ink == "All" {
                                                    push(referenced, "Cyan");
                                                    push(referenced, "Magenta");
                                                    push(referenced, "Yellow");
                                                    push(referenced, "Black");
                                                } else {
                                                    push(referenced, &ink);
                                                }
                                            }
                                        },
                                        ResolvedSpace::DeviceN(names) => {
                                            for n in names {
                                                if n != "None" && !n.is_empty() {
                                                    if n == "All" {
                                                        push(referenced, "Cyan");
                                                        push(referenced, "Magenta");
                                                        push(referenced, "Yellow");
                                                        push(referenced, "Black");
                                                    } else {
                                                        push(referenced, &n);
                                                    }
                                                }
                                            }
                                        },
                                        // RGB / Gray / Unknown contribute no
                                        // plates per the renderer's policy.
                                        _ => {},
                                    }
                                }
                            }
                        }
                    }
                }
            },
            _ => {},
        }
    }
    Ok(())
}

fn inks_from_space(
    space_name: &str,
    color_spaces: &HashMap<String, Object>,
    resources: &Object,
    doc: &PdfDocument,
    out: &mut Vec<String>,
) {
    // Honour DefaultCMYK/RGB/Gray remap (RED #2 — see resolve_color_space).
    let space = resolve_color_space(space_name, color_spaces, resources, doc);
    match space {
        ResolvedSpace::Cmyk | ResolvedSpace::IccCmyk => {
            for ink in ["Cyan", "Magenta", "Yellow", "Black"] {
                if !out.iter().any(|s| s == ink) {
                    out.push(ink.to_string());
                }
            }
        },
        ResolvedSpace::Separation(name) => {
            // §8.6.6.4: /All marks every output separation — list CMYK so the
            // per-plate short-circuit in render_separations doesn't skip them.
            // /None paints nothing and never names a plate.
            if name == "All" {
                for ink in ["Cyan", "Magenta", "Yellow", "Black"] {
                    if !out.iter().any(|s| s == ink) {
                        out.push(ink.to_string());
                    }
                }
            } else if name != "None" && !out.iter().any(|s| s == &name) {
                out.push(name);
            }
        },
        ResolvedSpace::DeviceN(names) => {
            for n in names {
                if n == "All" {
                    for ink in ["Cyan", "Magenta", "Yellow", "Black"] {
                        if !out.iter().any(|s| s == ink) {
                            out.push(ink.to_string());
                        }
                    }
                } else if n != "None" && !out.iter().any(|s| s == &n) {
                    out.push(n);
                }
            }
        },
        ResolvedSpace::Rgb
        | ResolvedSpace::Gray
        | ResolvedSpace::IccRgb
        | ResolvedSpace::IccGray
        | ResolvedSpace::Unknown => {},
    }
}

/// Page extent computation (width/height in pixels and the base
/// transform that maps PDF user space into the pixmap).
fn compute_page_extent(
    doc: &PdfDocument,
    page_num: usize,
    dpi: u32,
) -> Result<(u32, u32, Transform)> {
    let page_info = doc.get_page_info(page_num)?;
    // Same page rectangle the composite renderer uses — §14.11.2 crop box
    // intersected with the media box — so plates and composite stay the same
    // size and origin.
    let media_box = super::page_render_box(&page_info.media_box, page_info.crop_box.as_ref());

    // Shared with the composite renderer, so the two cannot disagree about
    // which way a rotated page faces — see `page_base_transform`.
    let rotation = page_info.rotation;
    let (page_w, page_h) = super::rotated_page_extent(&media_box, rotation);
    let scale = dpi as f32 / 72.0;
    let width = (page_w * scale).ceil() as u32;
    let height = (page_h * scale).ceil() as u32;
    let base_transform = super::page_base_transform(&media_box, rotation, scale);

    Ok((width, height, base_transform))
}

/// Resolved colour-space classification used by the separation pipeline.
#[derive(Debug, Clone)]
enum ResolvedSpace {
    Cmyk,
    Rgb,
    Gray,
    Separation(String),
    DeviceN(Vec<String>),
    /// ICCBased with a 4-component profile (treated as CMYK by heuristic).
    IccCmyk,
    /// ICCBased with 3 components (RGB).
    IccRgb,
    /// ICCBased with 1 component (Gray).
    IccGray,
    Unknown,
}

/// Resolve a colour-space name to a known classification.
///
/// Handles ISO 32000-1 §8.6.5.6: when the named space is one of the
/// Device families and the resource dictionary defines a corresponding
/// `Default*` entry, the Default mapping is consulted instead.
fn resolve_color_space(
    space_name: &str,
    color_spaces: &HashMap<String, Object>,
    resources: &Object,
    doc: &PdfDocument,
) -> ResolvedSpace {
    // Direct Device* names — try DefaultCMYK / DefaultRGB / DefaultGray remap first.
    let default_key = match space_name {
        "DeviceCMYK" | "CMYK" => Some("DefaultCMYK"),
        "DeviceRGB" | "RGB" => Some("DefaultRGB"),
        "DeviceGray" | "G" => Some("DefaultGray"),
        _ => None,
    };
    if let Some(key) = default_key {
        if let Some(default) = color_spaces.get(key) {
            // Walk into the default array as a fresh classification.
            return classify_resolved(default, color_spaces, resources, doc);
        }
        return match key {
            "DefaultCMYK" => ResolvedSpace::Cmyk,
            "DefaultRGB" => ResolvedSpace::Rgb,
            _ => ResolvedSpace::Gray,
        };
    }

    if let Some(cs_obj) = color_spaces.get(space_name) {
        classify_resolved(cs_obj, color_spaces, resources, doc)
    } else {
        ResolvedSpace::Unknown
    }
}

/// Classify a colour-space object (either an array or a name) into a
/// [`ResolvedSpace`]. Used both as the entry point from a resource-dict
/// lookup and recursively when an array starts with a name that is
/// itself a device alias.
fn classify_resolved(
    cs_obj: &Object,
    color_spaces: &HashMap<String, Object>,
    resources: &Object,
    doc: &PdfDocument,
) -> ResolvedSpace {
    // Plain name (e.g. /DeviceCMYK as the array's tail target).
    if let Some(name) = cs_obj.as_name() {
        return match name {
            "DeviceCMYK" | "CMYK" => ResolvedSpace::Cmyk,
            "DeviceRGB" | "RGB" => ResolvedSpace::Rgb,
            "DeviceGray" | "G" => ResolvedSpace::Gray,
            _ => resolve_color_space(name, color_spaces, resources, doc),
        };
    }

    let arr = match cs_obj.as_array() {
        Some(a) => a,
        None => return ResolvedSpace::Unknown,
    };
    let type_name = match arr.first().and_then(|o| o.as_name()) {
        Some(n) => n,
        None => return ResolvedSpace::Unknown,
    };
    match type_name {
        "DeviceCMYK" | "CMYK" => ResolvedSpace::Cmyk,
        "DeviceRGB" | "RGB" => ResolvedSpace::Rgb,
        "DeviceGray" | "G" => ResolvedSpace::Gray,
        "Pattern" => {
            // ISO 32000-1 §8.7.3.1: Pattern colour space's optional
            // index-1 element is the underlying colour space
            // (uncoloured Tiling carries the underlying space's
            // tints). For separation-ink scanning, recurse so a
            // Pattern[/Separation /Foo] marks /Foo as referenced.
            // Brings this into parity with the sidecar extractor's
            // Pattern arm.
            //
            // Real-world PDFs commonly share a Pattern's underlying
            // colour space via an indirect reference
            // (`/Pattern [<obj> <gen> R]`). Dereference before
            // recursing so the indirect form classifies identically
            // to an inline-array underlying. The sidecar's analogous
            // arm performs the same deref via its `deref` closure.
            match arr.get(1) {
                Some(underlying) => {
                    let resolved = doc
                        .resolve_object(underlying)
                        .unwrap_or_else(|_| underlying.clone());
                    classify_resolved(&resolved, color_spaces, resources, doc)
                },
                None => ResolvedSpace::Unknown,
            }
        },
        "Separation" => {
            let ink = arr
                .get(1)
                .and_then(|o| o.as_name())
                .map(|s| s.to_string())
                .unwrap_or_default();
            ResolvedSpace::Separation(ink)
        },
        "DeviceN" => {
            if let Some(Object::Array(ink_names)) = arr.get(1) {
                let names = ink_names
                    .iter()
                    .filter_map(|o| o.as_name().map(|s| s.to_string()))
                    .collect();
                ResolvedSpace::DeviceN(names)
            } else {
                ResolvedSpace::Unknown
            }
        },
        "ICCBased" => {
            // ICCBased: read /N from the stream dict to pick the component-count
            // interpretation. Unknown / unreachable / unsupported N → Unknown,
            // since fabricating CMYK plate values from an N=2 or N=5 profile
            // would silently corrupt output. tint_for_ink skips Unknown spaces.
            if let Some(stream_obj) = arr.get(1) {
                if let Ok(resolved) = doc.resolve_object(stream_obj) {
                    if let Object::Stream { ref dict, .. } = resolved {
                        if let Some(n) = dict.get("N").and_then(|o| o.as_integer()) {
                            return match n {
                                4 => ResolvedSpace::IccCmyk,
                                3 => ResolvedSpace::IccRgb,
                                1 => ResolvedSpace::IccGray,
                                _ => ResolvedSpace::Unknown,
                            };
                        }
                    }
                }
            }
            ResolvedSpace::Unknown
        },
        _ => ResolvedSpace::Unknown,
    }
}

/// Load color space definitions from page resources.
fn load_color_spaces(doc: &PdfDocument, resources: &Object) -> Result<HashMap<String, Object>> {
    let mut color_spaces = HashMap::new();
    if let Object::Dictionary(res_dict) = resources {
        if let Some(cs_obj) = res_dict.get("ColorSpace") {
            let cs_dict_obj = doc.resolve_object(cs_obj)?;
            if let Some(cs_dict) = cs_dict_obj.as_dict() {
                for (name, o) in cs_dict {
                    if let Ok(resolved_cs) = doc.resolve_object(o) {
                        color_spaces.insert(name.clone(), resolved_cs);
                    }
                }
            }
        }
    }
    Ok(color_spaces)
}

/// Load font resources for the page. Failures are swallowed (text using
/// unloadable fonts is dropped); this matches the page renderer's
/// best-effort behaviour and keeps separation rendering robust on PDFs
/// with corrupt or missing fonts.
fn load_fonts(doc: &PdfDocument, resources: &Object) -> HashMap<String, Arc<FontInfo>> {
    let mut fonts = HashMap::new();
    if let Object::Dictionary(res_dict) = resources {
        if let Some(font_obj) = res_dict.get("Font") {
            if let Ok(font_dict_obj) = doc.resolve_object(font_obj) {
                if let Some(font_dict) = font_dict_obj.as_dict() {
                    for (name, f_obj) in font_dict {
                        if let Ok(info) = doc.get_or_load_font_for_rendering(f_obj) {
                            fonts.insert(name.clone(), info);
                        }
                    }
                }
            }
        }
    }
    fonts
}

/// Per-plate routing decision for a single paint operation, after applying
/// the overprint rules of ISO 32000-1 §11.7.4.
///
/// - [`PaintAction::Paint`] writes the given tint into the plate. A tint
///   of 0.0 is the spec-default "knockout" — the existing
///   [`fill_separation`] / [`stroke_separation`] use opaque source-over,
///   so writing 0.0 erases any underlying ink at the touched pixels.
/// - [`PaintAction::Skip`] leaves the plate completely untouched. Used
///   when (a) the source colour space doesn't reference this plate and
///   overprint is enabled, or (b) the source is DeviceCMYK with OPM=1
///   and the component is exactly 0.0 (the "Adobe nonzero overprint"
///   rule, §11.7.4).
enum PaintAction {
    Paint(f32),
    Skip,
}

/// Decide how the current paint operation contributes to `target_ink`,
/// honoring ISO 32000-1 §11.7.4 (Overprint Control).
///
/// The decision tree:
///
/// ```text
/// For each plate P, source colour space S with component vector c[]:
///
///   if S = Separation(/All):                              Paint(c[0])
///   if S = Separation(/None) or empty components:         Skip
///   if S = Separation(name) and name == P:                Paint(c[0])
///   if S = Separation(name) and name != P:
///         overprint? Skip : Paint(0.0)                    // §11.7.4 default knockout
///
///   if S = DeviceN(names) and P in names:                 Paint(c[index_of_P])
///   if S = DeviceN(names) and P not in names:
///         overprint? Skip : Paint(0.0)
///
///   if S = DeviceCMYK / IccCmyk:
///         if P in {C, M, Y, K}:
///             overprint && opm == 1 && tint == 0.0 ? Skip : Paint(tint)
///         else:                                            // spot plate
///             overprint? Skip : Paint(0.0)                 // §11.7.4 default knockout
///
///   if S = RGB/Gray/IccRgb/IccGray:                       Skip
/// ```
fn tint_for_ink(
    fill: bool,
    gs: &GraphicsState,
    color_spaces: &HashMap<String, Object>,
    resources: &Object,
    doc: &PdfDocument,
    target_ink: &str,
    fill_components: &[f32],
    stroke_components: &[f32],
) -> PaintAction {
    let space_name = if fill {
        &gs.fill_color_space
    } else {
        &gs.stroke_color_space
    };
    let components = if fill {
        fill_components
    } else {
        stroke_components
    };
    let overprint = if fill {
        gs.fill_overprint
    } else {
        gs.stroke_overprint
    };
    // §11.7.4.3: OPM applies only when the source is DeviceCMYK (or implicit
    // conversion thereto). The match arms below check this where relevant.
    let opm = gs.overprint_mode;

    // Default action when the source colour space doesn't name the
    // target plate: under OP=true, leave it alone; under OP=false (the
    // spec default), erase it to 0.0 ("areas of unspecified colorants
    // are erased" — §11.7.4).
    let other_plate_action = if overprint {
        PaintAction::Skip
    } else {
        PaintAction::Paint(0.0)
    };

    let resolved = resolve_color_space(space_name, color_spaces, resources, doc);
    match resolved {
        ResolvedSpace::Cmyk | ResolvedSpace::IccCmyk => {
            let cmyk_state = if fill {
                gs.fill_color_cmyk
            } else {
                gs.stroke_color_cmyk
            };
            let (c, m, y, k) = if let Some(v) = cmyk_state {
                v
            } else if components.len() >= 4 {
                (components[0], components[1], components[2], components[3])
            } else {
                return PaintAction::Skip;
            };
            let tint = match target_ink {
                "Cyan" => c,
                "Magenta" => m,
                "Yellow" => y,
                "Black" => k,
                // Spot plate — not in DeviceCMYK's colorant set.
                _ => return other_plate_action,
            };
            // §11.7.4 OPM=1 nonzero overprint: zero source components on
            // DeviceCMYK are treated as "not specified" — leave the
            // matching plate untouched. OPM=0 (default) paints zero,
            // which erases (knocks out) the plate.
            if overprint && opm == 1 && tint == 0.0 {
                PaintAction::Skip
            } else {
                PaintAction::Paint(tint)
            }
        },
        ResolvedSpace::Rgb
        | ResolvedSpace::Gray
        | ResolvedSpace::IccRgb
        | ResolvedSpace::IccGray => {
            // §11.7.4: overprint is a separation-space concept. RGB / Gray
            // sources do not route to ink plates at all. Converting them
            // would require a tint transform and is intentionally not done.
            PaintAction::Skip
        },
        ResolvedSpace::Separation(ink) => {
            // §8.6.6.4: /All paints to every plate; /None paints nothing.
            if components.is_empty() || ink == "None" {
                return PaintAction::Skip;
            }
            if ink == "All" {
                return PaintAction::Paint(components[0]);
            }
            if ink == target_ink {
                PaintAction::Paint(components[0])
            } else {
                other_plate_action
            }
        },
        ResolvedSpace::DeviceN(names) => {
            for (i, n) in names.iter().enumerate() {
                if n == "None" {
                    continue;
                }
                if (n == "All" || n == target_ink) && i < components.len() {
                    return PaintAction::Paint(components[i]);
                }
            }
            other_plate_action
        },
        ResolvedSpace::Unknown => PaintAction::Skip,
    }
}

/// Build a [`LogicalColor`] for the per-plate path from the current
/// graphics-state colour space and component values. Mirrors the
/// resolution the composite-side `build_logical_color` does, but
/// keyed on the separation walker's `gs.fill_color_space` /
/// `gs.stroke_color_space` strings and the parallel
/// `SeparationColorState` components vectors.
///
/// Returns `None` when the colour space can't be resolved or is empty.
fn logical_color_for_side<'a>(
    fill: bool,
    gs: &'a GraphicsState,
    cs: &'a SeparationColorState,
    color_spaces: &'a HashMap<String, Object>,
) -> Option<LogicalColor<'a>> {
    let space_name = if fill {
        &gs.fill_color_space
    } else {
        &gs.stroke_color_space
    };
    let components = if fill {
        &cs.fill_components
    } else {
        &cs.stroke_components
    };
    let cmyk_state = if fill {
        gs.fill_color_cmyk
    } else {
        gs.stroke_color_cmyk
    };

    // Device-family aliases: emit the operator-side LogicalColor::Device
    // so the resolver passes straight through to the right channel
    // decomposition.
    match space_name.as_str() {
        "DeviceCMYK" | "CMYK" => {
            let (c, m, y, k) = cmyk_state.or_else(|| {
                if components.len() >= 4 {
                    Some((components[0], components[1], components[2], components[3]))
                } else {
                    None
                }
            })?;
            return Some(LogicalColor::Device(DeviceColor::Cmyk(c, m, y, k)));
        },
        "DeviceRGB" | "RGB" => {
            if components.len() >= 3 {
                return Some(LogicalColor::Device(DeviceColor::Rgb(
                    components[0],
                    components[1],
                    components[2],
                )));
            }
            return None;
        },
        "DeviceGray" | "G" => {
            if !components.is_empty() {
                return Some(LogicalColor::Device(DeviceColor::Gray(components[0])));
            }
            return None;
        },
        _ => {},
    }

    // Spaced: needs a borrow into the page-resource colour-space map.
    let space = color_spaces.get(space_name)?;
    let comps: SmallVec<[f32; 8]> = components.iter().copied().collect();
    Some(LogicalColor::Spaced {
        space,
        components: comps,
    })
}

/// Dispatch a single paint operation through the resolution pipeline
/// and the [`SeparationBackend`]. Used for the spot / DeviceN / ICCBased
/// cases the inline `tint_for_ink` path can't resolve (notably Type-4
/// tint transforms on Separation/DeviceN sources). Returns `true` on a
/// successful pipeline dispatch; `false` if the colour can't be made
/// into a logical colour (caller falls back to the inline path).
#[allow(clippy::too_many_arguments)]
fn paint_through_pipeline(
    fill: bool,
    fill_rule: Option<FillRule>,
    path: &tiny_skia::Path,
    pixmaps: &mut [Pixmap],
    target_inks: &[InkName],
    base_transform: Transform,
    gs: &GraphicsState,
    cs: &SeparationColorState,
    color_spaces: &HashMap<String, Object>,
    resources: &Object,
    doc: &PdfDocument,
    clip: Option<&Mask>,
    pipeline: &ResolutionPipeline,
    backend: &mut SeparationBackend,
) -> Result<()> {
    let _ = resources; // ResolutionContext consumes (doc, color_spaces); kept for future audits.
    let Some(logical) = logical_color_for_side(fill, gs, cs, color_spaces) else {
        return Ok(());
    };
    let side = if fill {
        PaintSide::Fill
    } else {
        PaintSide::Stroke
    };
    let intent = PaintIntent {
        kind: PaintKind::Path {
            path,
            fill_rule: fill_rule.unwrap_or(FillRule::Winding),
        },
        side,
        gs,
        color: logical,
        ctm: gs.ctm,
    };
    // Thread the same colour-policy borrows as the composite path
    // (page_renderer's run_pipeline_for_logical). The per-plate backend
    // consumes ResolvedColor::Cmyk channel-by-channel for plate routing
    // and never projects to RGBA, so the document /OutputIntents CMYK
    // profile carried here is effectively no-op for separations — the
    // plates ARE the press-target ink coverage. Threading it uniformly
    // keeps the resolver call surface symmetric with the composite path
    // so a single ColorResolver change can't silently diverge between
    // the two renderers.
    //
    // HONEST_GAP: the per-page `IccTransformCache` that amortises qcms
    // transform construction across paint operators lives on
    // `PageRenderer`. The separation walker is a free function — it
    // would need a SeparationRendererState struct to hold the cache
    // across paint operators within a page. That's a separate refactor;
    // the per-plate path doesn't actually invoke `cmyk_to_rgb_via_intent`
    // (the per-plate router consumes `ResolvedColor::Cmyk` directly),
    // so the only Transform construction here is on `/ICCBased` N=4
    // paint, and only when the embedded profile has a working CMM —
    // which is the design's expected (cold-path) case.
    let output_intent = doc.output_intent_cmyk_profile();
    let ctx = ResolutionContext::new(doc, color_spaces)
        .with_output_intent(output_intent.as_ref())
        .with_rendering_intent(crate::color::RenderingIntent::from_pdf_name(&gs.rendering_intent))
        .with_defaults(
            color_spaces.get("DefaultGray"),
            color_spaces.get("DefaultRGB"),
            color_spaces.get("DefaultCMYK"),
        );
    let cmd = pipeline.resolve(&intent, &ctx, None)?;
    // Wrap the clip mask back into a borrowed ClipPlan-equivalent via
    // the SeparationSurface's externally-visible state. The
    // SeparationBackend reads cmd.clip; build the cmd with an Arc-wrapped
    // mask only when one is present.
    let surface = SeparationSurface {
        pixmaps,
        inks: target_inks,
        base_transform,
    };
    // The pipeline currently produces ClipPlan::None because we passed
    // None into resolve(); for the separation walker the active clip
    // lives on `clip_stack` and is the same mask for every plate. Hand
    // it through by rebuilding the cmd with a wrapped Arc when present.
    let cmd = if let Some(mask) = clip {
        let mut new = cmd;
        new.clip = crate::rendering::resolution::ClipPlan::Mask(std::sync::Arc::new(mask.clone()));
        new
    } else {
        cmd
    };
    backend.paint(&cmd, surface)?;
    Ok(())
}

/// Decide whether the current paint at `gs.{fill,stroke}_color_space`
/// should route through the [`ResolutionPipeline`] or stay on the
/// inline `tint_for_ink` fast path.
///
/// The pipeline is the only path that handles Type-4 tint transforms,
/// Separation reserved colorant names (`/All`, `/None`), and the OPM=1
/// zero-component rule via [`InkRouter`]. Process colour direct
/// (`DeviceCMYK`, `DeviceGray`) and `DeviceRGB` (which the per-plate
/// path skips entirely) keep the existing inline behaviour — it's
/// cheaper and the inline arms are already correct for those cases.
fn side_uses_pipeline(
    fill: bool,
    gs: &GraphicsState,
    color_spaces: &HashMap<String, Object>,
    resources: &Object,
    doc: &PdfDocument,
) -> bool {
    let space_name = if fill {
        &gs.fill_color_space
    } else {
        &gs.stroke_color_space
    };
    // Plain Device-* names take the inline path.
    if matches!(
        space_name.as_str(),
        "DeviceCMYK" | "CMYK" | "DeviceRGB" | "RGB" | "DeviceGray" | "G"
    ) {
        return false;
    }
    // Anything else: classify, and route compound spaces through the
    // pipeline so Type-4 / DeviceN / ICCBased N=4 evaluations land.
    matches!(
        resolve_color_space(space_name, color_spaces, resources, doc),
        ResolvedSpace::Separation(_)
            | ResolvedSpace::DeviceN(_)
            | ResolvedSpace::IccCmyk
            | ResolvedSpace::IccRgb
            | ResolvedSpace::IccGray
    )
}

/// Per-render shared context (read-only) passed through the operator
/// walk and into recursive Form XObject invocations.
///
/// The set of target inks is **not** stored here; instead it is passed
/// as a separate `target_inks: &[&str]` slice alongside the `&mut [Pixmap]`
/// to [`execute_separation_operators`]. This keeps the borrow checker
/// happy: the pixmaps slice is the only `&mut` in play, while everything
/// in `SeparationContext` is `&`.
struct SeparationContext<'a> {
    doc: &'a PdfDocument,
    text_rasterizer: &'a TextRasterizer,
    fonts: &'a HashMap<String, Arc<FontInfo>>,
    /// OCGs hidden by the document's own `/OCProperties /D` configuration
    /// (ISO 32000-1:2008 §8.11.4), computed exactly as the composite renderer
    /// computes its baseline. Without this the plates counted ink for layers
    /// the page render omits, so the two disagreed about the same page.
    excluded_layers: &'a HashSet<String>,
}

/// Color state tracked alongside the graphics state for separation rendering.
#[derive(Clone, Debug)]
struct SeparationColorState {
    fill_components: Vec<f32>,
    stroke_components: Vec<f32>,
}

impl SeparationColorState {
    fn new() -> Self {
        Self {
            fill_components: Vec::new(),
            stroke_components: Vec::new(),
        }
    }
}

/// Compute the initial colour components for a colour space per
/// ISO 32000-1 §8.6.4.2. `cs`/`CS` resets the current colour to these
/// values when entering the space.
fn initial_components_for_space(
    space_name: &str,
    color_spaces: &HashMap<String, Object>,
    resources: &Object,
    doc: &PdfDocument,
) -> (Vec<f32>, Option<(f32, f32, f32, f32)>) {
    let resolved = resolve_color_space(space_name, color_spaces, resources, doc);
    match resolved {
        ResolvedSpace::Cmyk | ResolvedSpace::IccCmyk => {
            (vec![0.0, 0.0, 0.0, 1.0], Some((0.0, 0.0, 0.0, 1.0)))
        },
        ResolvedSpace::Rgb | ResolvedSpace::IccRgb => (vec![0.0, 0.0, 0.0], None),
        ResolvedSpace::Gray | ResolvedSpace::IccGray => (vec![0.0], None),
        ResolvedSpace::Separation(_) => (vec![1.0], None),
        ResolvedSpace::DeviceN(names) => {
            let n = names.len().max(1);
            (vec![1.0; n], None)
        },
        ResolvedSpace::Unknown => (Vec::new(), None),
    }
}

/// State inherited from a calling context when recursing into a Form
/// XObject (PDF §8.10.1: a Form XObject's initial graphics state is
/// the calling context's graphics state).
struct InheritedState {
    fill_color_space: String,
    stroke_color_space: String,
    fill_color_cmyk: Option<(f32, f32, f32, f32)>,
    stroke_color_cmyk: Option<(f32, f32, f32, f32)>,
    fill_components: Vec<f32>,
    stroke_components: Vec<f32>,
    fill_overprint: bool,
    stroke_overprint: bool,
    overprint_mode: u8,
}

/// Execute operators for separation plate rendering, dispatching paint
/// operations to **all** target inks in parallel.
///
/// `pixmaps` and `target_inks` are parallel slices: `pixmaps[i]` receives
/// paint for ink `target_inks[i]`. The operator stream is walked exactly
/// once; every paint site (fill, stroke, text, Form XObject) iterates the
/// pair list and contributes to each plate whose ink the current colour
/// touches.
#[allow(clippy::too_many_arguments)]
fn execute_separation_operators(
    pixmaps: &mut [Pixmap],
    base_transform: Transform,
    operators: &[Operator],
    ctx: &mut SeparationContext<'_>,
    resources: &Object,
    color_spaces: &HashMap<String, Object>,
    inherited: Option<&InheritedState>,
    target_inks: &[&str],
) -> Result<()> {
    debug_assert_eq!(pixmaps.len(), target_inks.len());
    let mut gs_stack = GraphicsStateStack::new();
    {
        let gs = gs_stack.current_mut();
        if let Some(inh) = inherited {
            gs.fill_color_space = inh.fill_color_space.clone();
            gs.stroke_color_space = inh.stroke_color_space.clone();
            gs.fill_color_cmyk = inh.fill_color_cmyk;
            gs.stroke_color_cmyk = inh.stroke_color_cmyk;
            // §8.10.1: inherit the caller's overprint state too. Without
            // this, an outer `gs` setting OP=true would be silently
            // dropped at the Form XObject boundary and the form's CMYK
            // content would knock out underlying inks against the
            // caller's intent.
            gs.fill_overprint = inh.fill_overprint;
            gs.stroke_overprint = inh.stroke_overprint;
            gs.overprint_mode = inh.overprint_mode;
        } else {
            gs.fill_color_space = "DeviceGray".to_string();
            gs.stroke_color_space = "DeviceGray".to_string();
        }
        gs.fill_color_rgb = (0.0, 0.0, 0.0);
        gs.stroke_color_rgb = (0.0, 0.0, 0.0);
    }

    let initial_cs = if let Some(inh) = inherited {
        SeparationColorState {
            fill_components: inh.fill_components.clone(),
            stroke_components: inh.stroke_components.clone(),
        }
    } else {
        SeparationColorState::new()
    };
    let mut color_state_stack: Vec<SeparationColorState> = vec![initial_cs];
    let mut current_path = PathBuilder::new();
    let mut pending_clip: Option<(tiny_skia::Path, FillRule)> = None;
    let mut clip_stack: Vec<Option<Mask>> = vec![None];
    let mut in_text_object = false;

    // Pre-resolve ExtGState for the gs cache.
    let ext_g_state_resolved: Option<Object> = match resources {
        Object::Dictionary(rd) => rd
            .get("ExtGState")
            .and_then(|o| ctx.doc.resolve_object(o).ok()),
        _ => None,
    };
    let ext_g_states: Option<&HashMap<String, Object>> =
        ext_g_state_resolved.as_ref().and_then(|o| o.as_dict());
    let mut ext_g_state_cache: HashMap<String, ParsedExtGState> = HashMap::new();

    let xobjects_resolved: Option<Object> = match resources {
        Object::Dictionary(rd) => rd
            .get("XObject")
            .and_then(|o| ctx.doc.resolve_object(o).ok()),
        _ => None,
    };

    // Pixmap extent — every plate shares the same dimensions because they
    // all originate from a single allocation in `render_plates_for_inks`.
    // If `pixmaps` is empty (no inks to render), use a zero extent; the
    // operator walk still progresses for graphics-state tracking but
    // paint loops are no-ops because there are no targets.
    let pixmap_width = pixmaps.first().map(|p| p.width()).unwrap_or(0);
    let pixmap_height = pixmaps.first().map(|p| p.height()).unwrap_or(0);

    // Pipeline-driven dispatch state. The pipeline replaces the inline
    // `tint_for_ink` decision tree for Separation / DeviceN / ICCBased
    // sources — it's the only path that evaluates Type-4 tint
    // transforms, honours §8.6.6.3 `/All` and `/None`, and routes via
    // the §11.7.4 / §11.7.4.3 InkRouter rules. Process colour direct
    // (DeviceCMYK / DeviceGray) and DeviceRGB keep the inline fast
    // path because the inline arms are already correct for those.
    let pipeline = ResolutionPipeline::new();
    let mut backend = SeparationBackend::new();
    let target_inks_owned: Vec<InkName> = target_inks.iter().map(|s| InkName::new(*s)).collect();

    // Optional-content exclusion, mirroring the composite renderer.
    //
    // The separation renderer had no marked-content arms at all, so the /OC
    // hidden-layer exclusion the composite render applies never reached the ink
    // plates: a layer excluded from the page was still counted in the
    // separations, and two renderers of one page gave contradictory answers.
    //
    // `excluded_layer_depth` counts nested BDC/OC scopes that resolve to an
    // excluded layer; `marked_content_is_excluded` lets EMC decrement only the
    // entries that incremented.
    let mut excluded_layer_depth: u32 = 0;
    let mut marked_content_is_excluded: Vec<bool> = Vec::new();
    let excluded_layers: &HashSet<String> = ctx.excluded_layers;

    for op in operators {
        // ISO 32000-1:2008 §8.11.3: when optional content is hidden "the
        // content shall not be drawn" while "graphics state operations ...
        // shall still be applied". So suppression replaces each painting
        // operator with the path-clearing `n` rather than skipping the
        // operator loop — the path bookkeeping and every state change still
        // happen, exactly as they would if the content were visible.
        let op = if excluded_layer_depth > 0 {
            match op {
                Operator::Fill
                | Operator::FillEvenOdd
                | Operator::Stroke
                | Operator::FillStroke
                | Operator::CloseFillStroke
                | Operator::FillStrokeEvenOdd
                | Operator::CloseFillStrokeEvenOdd => &Operator::EndPath,
                other => other,
            }
        } else {
            op
        };

        // Drawing an XObject or showing text inside a hidden scope marks the
        // page just as a fill would, and neither has a path-clearing
        // equivalent, so those are skipped outright.
        if excluded_layer_depth > 0
            && matches!(
                op,
                Operator::Do { .. }
                    | Operator::Tj { .. }
                    | Operator::TJ { .. }
                    | Operator::Quote { .. }
                    | Operator::DoubleQuote { .. }
                    | Operator::PaintShading { .. }
                    | Operator::InlineImage { .. }
            )
        {
            continue;
        }

        match op {
            Operator::BeginMarkedContent { .. } => {
                marked_content_is_excluded.push(false);
            },
            Operator::BeginMarkedContentDict { tag, properties } => {
                let mut is_excluded = false;
                if tag == "OC" {
                    is_excluded = crate::optional_content::resolve_and_check_ocg_excluded(
                        properties,
                        Some(resources),
                        Some(ctx.doc),
                        excluded_layers,
                    );
                }
                if is_excluded {
                    excluded_layer_depth += 1;
                }
                marked_content_is_excluded.push(is_excluded);
            },
            Operator::EndMarkedContent => {
                if let Some(was_excluded) = marked_content_is_excluded.pop() {
                    if was_excluded && excluded_layer_depth > 0 {
                        excluded_layer_depth -= 1;
                    }
                }
            },
            Operator::SaveState => {
                gs_stack.save();
                let cs = color_state_stack
                    .last()
                    .cloned()
                    .unwrap_or_else(SeparationColorState::new);
                color_state_stack.push(cs);
                clip_stack.push(clip_stack.last().cloned().unwrap_or(None));
            },
            Operator::RestoreState => {
                gs_stack.restore();
                if color_state_stack.len() > 1 {
                    color_state_stack.pop();
                }
                if clip_stack.len() > 1 {
                    clip_stack.pop();
                }
            },

            Operator::Cm { a, b, c, d, e, f } => {
                let current = gs_stack.current_mut();
                let new_matrix = Matrix {
                    a: *a,
                    b: *b,
                    c: *c,
                    d: *d,
                    e: *e,
                    f: *f,
                };
                current.ctm = new_matrix.multiply(&current.ctm);
            },

            Operator::SetFillRgb { r, g, b } => {
                let gs = gs_stack.current_mut();
                gs.fill_color_rgb = (*r, *g, *b);
                gs.fill_color_space = "DeviceRGB".to_string();
                gs.fill_color_cmyk = None;
                if let Some(cs) = color_state_stack.last_mut() {
                    cs.fill_components = vec![*r, *g, *b];
                }
            },
            Operator::SetStrokeRgb { r, g, b } => {
                let gs = gs_stack.current_mut();
                gs.stroke_color_rgb = (*r, *g, *b);
                gs.stroke_color_space = "DeviceRGB".to_string();
                gs.stroke_color_cmyk = None;
                if let Some(cs) = color_state_stack.last_mut() {
                    cs.stroke_components = vec![*r, *g, *b];
                }
            },
            Operator::SetFillGray { gray } => {
                let g = *gray;
                let gs = gs_stack.current_mut();
                gs.fill_color_rgb = (g, g, g);
                gs.fill_color_space = "DeviceGray".to_string();
                gs.fill_color_cmyk = None;
                if let Some(cs) = color_state_stack.last_mut() {
                    cs.fill_components = vec![g];
                }
            },
            Operator::SetStrokeGray { gray } => {
                let g = *gray;
                let gs = gs_stack.current_mut();
                gs.stroke_color_rgb = (g, g, g);
                gs.stroke_color_space = "DeviceGray".to_string();
                gs.stroke_color_cmyk = None;
                if let Some(cs) = color_state_stack.last_mut() {
                    cs.stroke_components = vec![g];
                }
            },
            Operator::SetFillCmyk { c, m, y, k } => {
                let gs = gs_stack.current_mut();
                gs.fill_color_cmyk = Some((*c, *m, *y, *k));
                gs.fill_color_space = "DeviceCMYK".to_string();
                if let Some(cs) = color_state_stack.last_mut() {
                    cs.fill_components = vec![*c, *m, *y, *k];
                }
            },
            Operator::SetStrokeCmyk { c, m, y, k } => {
                let gs = gs_stack.current_mut();
                gs.stroke_color_cmyk = Some((*c, *m, *y, *k));
                gs.stroke_color_space = "DeviceCMYK".to_string();
                if let Some(cs) = color_state_stack.last_mut() {
                    cs.stroke_components = vec![*c, *m, *y, *k];
                }
            },
            Operator::SetFillColorSpace { name } => {
                let (components, cmyk) =
                    initial_components_for_space(name, color_spaces, resources, ctx.doc);
                let gs = gs_stack.current_mut();
                gs.fill_color_space = name.clone();
                gs.fill_color_cmyk = cmyk;
                if let Some(cs) = color_state_stack.last_mut() {
                    cs.fill_components = components;
                }
            },
            Operator::SetStrokeColorSpace { name } => {
                let (components, cmyk) =
                    initial_components_for_space(name, color_spaces, resources, ctx.doc);
                let gs = gs_stack.current_mut();
                gs.stroke_color_space = name.clone();
                gs.stroke_color_cmyk = cmyk;
                if let Some(cs) = color_state_stack.last_mut() {
                    cs.stroke_components = components;
                }
            },
            Operator::SetFillColor { components } | Operator::SetFillColorN { components, .. } => {
                let gs = gs_stack.current_mut();
                let space = gs.fill_color_space.clone();
                match space.as_str() {
                    "DeviceCMYK" | "CMYK" if components.len() >= 4 => {
                        gs.fill_color_cmyk =
                            Some((components[0], components[1], components[2], components[3]));
                    },
                    _ => {},
                }
                if let Some(cs) = color_state_stack.last_mut() {
                    cs.fill_components = components.clone();
                }
            },
            Operator::SetStrokeColor { components }
            | Operator::SetStrokeColorN { components, .. } => {
                let gs = gs_stack.current_mut();
                let space = gs.stroke_color_space.clone();
                match space.as_str() {
                    "DeviceCMYK" | "CMYK" if components.len() >= 4 => {
                        gs.stroke_color_cmyk =
                            Some((components[0], components[1], components[2], components[3]));
                    },
                    _ => {},
                }
                if let Some(cs) = color_state_stack.last_mut() {
                    cs.stroke_components = components.clone();
                }
            },

            Operator::SetLineWidth { width } => {
                gs_stack.current_mut().line_width = *width;
            },
            Operator::SetLineCap { cap_style } => {
                gs_stack.current_mut().line_cap = *cap_style;
            },
            Operator::SetLineJoin { join_style } => {
                gs_stack.current_mut().line_join = *join_style;
            },
            Operator::SetMiterLimit { limit } => {
                gs_stack.current_mut().miter_limit = *limit;
            },
            Operator::SetDash { array, phase } => {
                gs_stack.current_mut().dash_pattern = (array.clone(), *phase);
            },
            Operator::SetRenderingIntent { intent } => {
                // §10.7.3 — mirror the composite renderer's dispatch.
                // The per-plate path doesn't consult OutputIntent for
                // its CMYK channels (the plates ARE the press target),
                // but `gs.rendering_intent` still flows through the
                // resolver's ICCBased N=4 path, so keeping it current
                // matches the composite path's behaviour.
                gs_stack.current_mut().rendering_intent = intent.clone();
            },

            Operator::MoveTo { x, y } => {
                current_path.move_to(*x, *y);
            },
            Operator::LineTo { x, y } => {
                current_path.line_to(*x, *y);
            },
            Operator::CurveTo {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                current_path.cubic_to(*x1, *y1, *x2, *y2, *x3, *y3);
            },
            Operator::CurveToV { x2, y2, x3, y3 } => {
                if let Some(last) = current_path.last_point() {
                    current_path.cubic_to(last.x, last.y, *x2, *y2, *x3, *y3);
                }
            },
            Operator::CurveToY { x1, y1, x3, y3 } => {
                current_path.cubic_to(*x1, *y1, *x3, *y3, *x3, *y3);
            },
            Operator::Rectangle {
                x,
                y,
                width,
                height,
            } => {
                let (nx, nw) = if *width < 0.0 {
                    (x + width, -width)
                } else {
                    (*x, *width)
                };
                let (ny, nh) = if *height < 0.0 {
                    (y + height, -height)
                } else {
                    (*y, *height)
                };
                if let Some(rect) = tiny_skia::Rect::from_xywh(nx, ny, nw, nh) {
                    current_path.push_rect(rect);
                }
            },
            Operator::ClosePath => {
                current_path.close();
            },

            Operator::Stroke | Operator::CloseAndStroke => {
                // `s` closes the subpath first (Table 60).
                if matches!(op, Operator::CloseAndStroke) {
                    current_path.close();
                }
                apply_separation_clip(
                    &mut pending_clip,
                    &mut clip_stack,
                    pixmap_width,
                    pixmap_height,
                    base_transform,
                    &gs_stack,
                );
                if let Some(path) = current_path.finish() {
                    let gs = gs_stack.current();
                    let empty = SeparationColorState::new();
                    let cs = color_state_stack.last().unwrap_or(&empty);
                    let transform = combine_transforms(base_transform, &gs.ctm);
                    let clip = clip_stack.last().and_then(|c| c.as_ref());
                    if side_uses_pipeline(false, gs, color_spaces, resources, ctx.doc) {
                        paint_through_pipeline(
                            false,
                            None,
                            &path,
                            pixmaps,
                            &target_inks_owned,
                            base_transform,
                            gs,
                            cs,
                            color_spaces,
                            resources,
                            ctx.doc,
                            clip,
                            &pipeline,
                            &mut backend,
                        )?;
                    } else {
                        for (i, &ink) in target_inks.iter().enumerate() {
                            if let PaintAction::Paint(tint) = tint_for_ink(
                                false,
                                gs,
                                color_spaces,
                                resources,
                                ctx.doc,
                                ink,
                                &cs.fill_components,
                                &cs.stroke_components,
                            ) {
                                stroke_separation(
                                    &mut pixmaps[i],
                                    &path,
                                    transform,
                                    gs,
                                    tint,
                                    clip,
                                );
                            }
                        }
                    }
                }
                current_path = PathBuilder::new();
            },
            Operator::Fill => {
                apply_separation_clip(
                    &mut pending_clip,
                    &mut clip_stack,
                    pixmap_width,
                    pixmap_height,
                    base_transform,
                    &gs_stack,
                );
                if let Some(path) = current_path.finish() {
                    let gs = gs_stack.current();
                    let empty = SeparationColorState::new();
                    let cs = color_state_stack.last().unwrap_or(&empty);
                    let transform = combine_transforms(base_transform, &gs.ctm);
                    let clip = clip_stack.last().and_then(|c| c.as_ref());
                    if side_uses_pipeline(true, gs, color_spaces, resources, ctx.doc) {
                        paint_through_pipeline(
                            true,
                            Some(FillRule::Winding),
                            &path,
                            pixmaps,
                            &target_inks_owned,
                            base_transform,
                            gs,
                            cs,
                            color_spaces,
                            resources,
                            ctx.doc,
                            clip,
                            &pipeline,
                            &mut backend,
                        )?;
                    } else {
                        for (i, &ink) in target_inks.iter().enumerate() {
                            if let PaintAction::Paint(tint) = tint_for_ink(
                                true,
                                gs,
                                color_spaces,
                                resources,
                                ctx.doc,
                                ink,
                                &cs.fill_components,
                                &cs.stroke_components,
                            ) {
                                fill_separation(
                                    &mut pixmaps[i],
                                    &path,
                                    transform,
                                    tint,
                                    FillRule::Winding,
                                    clip,
                                );
                            }
                        }
                    }
                }
                current_path = PathBuilder::new();
            },
            Operator::FillEvenOdd => {
                apply_separation_clip(
                    &mut pending_clip,
                    &mut clip_stack,
                    pixmap_width,
                    pixmap_height,
                    base_transform,
                    &gs_stack,
                );
                if let Some(path) = current_path.finish() {
                    let gs = gs_stack.current();
                    let empty = SeparationColorState::new();
                    let cs = color_state_stack.last().unwrap_or(&empty);
                    let transform = combine_transforms(base_transform, &gs.ctm);
                    let clip = clip_stack.last().and_then(|c| c.as_ref());
                    if side_uses_pipeline(true, gs, color_spaces, resources, ctx.doc) {
                        paint_through_pipeline(
                            true,
                            Some(FillRule::EvenOdd),
                            &path,
                            pixmaps,
                            &target_inks_owned,
                            base_transform,
                            gs,
                            cs,
                            color_spaces,
                            resources,
                            ctx.doc,
                            clip,
                            &pipeline,
                            &mut backend,
                        )?;
                    } else {
                        for (i, &ink) in target_inks.iter().enumerate() {
                            if let PaintAction::Paint(tint) = tint_for_ink(
                                true,
                                gs,
                                color_spaces,
                                resources,
                                ctx.doc,
                                ink,
                                &cs.fill_components,
                                &cs.stroke_components,
                            ) {
                                fill_separation(
                                    &mut pixmaps[i],
                                    &path,
                                    transform,
                                    tint,
                                    FillRule::EvenOdd,
                                    clip,
                                );
                            }
                        }
                    }
                }
                current_path = PathBuilder::new();
            },
            Operator::FillStroke | Operator::CloseFillStroke => {
                apply_separation_clip(
                    &mut pending_clip,
                    &mut clip_stack,
                    pixmap_width,
                    pixmap_height,
                    base_transform,
                    &gs_stack,
                );
                if let Some(path) = current_path.finish() {
                    let gs = gs_stack.current();
                    let empty = SeparationColorState::new();
                    let cs = color_state_stack.last().unwrap_or(&empty);
                    let transform = combine_transforms(base_transform, &gs.ctm);
                    let clip = clip_stack.last().and_then(|c| c.as_ref());
                    // Fill side.
                    if side_uses_pipeline(true, gs, color_spaces, resources, ctx.doc) {
                        paint_through_pipeline(
                            true,
                            Some(FillRule::Winding),
                            &path,
                            pixmaps,
                            &target_inks_owned,
                            base_transform,
                            gs,
                            cs,
                            color_spaces,
                            resources,
                            ctx.doc,
                            clip,
                            &pipeline,
                            &mut backend,
                        )?;
                    } else {
                        for (i, &ink) in target_inks.iter().enumerate() {
                            if let PaintAction::Paint(tint) = tint_for_ink(
                                true,
                                gs,
                                color_spaces,
                                resources,
                                ctx.doc,
                                ink,
                                &cs.fill_components,
                                &cs.stroke_components,
                            ) {
                                fill_separation(
                                    &mut pixmaps[i],
                                    &path,
                                    transform,
                                    tint,
                                    FillRule::Winding,
                                    clip,
                                );
                            }
                        }
                    }
                    // Stroke side.
                    if side_uses_pipeline(false, gs, color_spaces, resources, ctx.doc) {
                        paint_through_pipeline(
                            false,
                            None,
                            &path,
                            pixmaps,
                            &target_inks_owned,
                            base_transform,
                            gs,
                            cs,
                            color_spaces,
                            resources,
                            ctx.doc,
                            clip,
                            &pipeline,
                            &mut backend,
                        )?;
                    } else {
                        for (i, &ink) in target_inks.iter().enumerate() {
                            if let PaintAction::Paint(tint) = tint_for_ink(
                                false,
                                gs,
                                color_spaces,
                                resources,
                                ctx.doc,
                                ink,
                                &cs.fill_components,
                                &cs.stroke_components,
                            ) {
                                stroke_separation(
                                    &mut pixmaps[i],
                                    &path,
                                    transform,
                                    gs,
                                    tint,
                                    clip,
                                );
                            }
                        }
                    }
                }
                current_path = PathBuilder::new();
            },
            Operator::FillStrokeEvenOdd | Operator::CloseFillStrokeEvenOdd => {
                apply_separation_clip(
                    &mut pending_clip,
                    &mut clip_stack,
                    pixmap_width,
                    pixmap_height,
                    base_transform,
                    &gs_stack,
                );
                if let Some(path) = current_path.finish() {
                    let gs = gs_stack.current();
                    let empty = SeparationColorState::new();
                    let cs = color_state_stack.last().unwrap_or(&empty);
                    let transform = combine_transforms(base_transform, &gs.ctm);
                    let clip = clip_stack.last().and_then(|c| c.as_ref());
                    // Fill side.
                    if side_uses_pipeline(true, gs, color_spaces, resources, ctx.doc) {
                        paint_through_pipeline(
                            true,
                            Some(FillRule::EvenOdd),
                            &path,
                            pixmaps,
                            &target_inks_owned,
                            base_transform,
                            gs,
                            cs,
                            color_spaces,
                            resources,
                            ctx.doc,
                            clip,
                            &pipeline,
                            &mut backend,
                        )?;
                    } else {
                        for (i, &ink) in target_inks.iter().enumerate() {
                            if let PaintAction::Paint(tint) = tint_for_ink(
                                true,
                                gs,
                                color_spaces,
                                resources,
                                ctx.doc,
                                ink,
                                &cs.fill_components,
                                &cs.stroke_components,
                            ) {
                                fill_separation(
                                    &mut pixmaps[i],
                                    &path,
                                    transform,
                                    tint,
                                    FillRule::EvenOdd,
                                    clip,
                                );
                            }
                        }
                    }
                    // Stroke side.
                    if side_uses_pipeline(false, gs, color_spaces, resources, ctx.doc) {
                        paint_through_pipeline(
                            false,
                            None,
                            &path,
                            pixmaps,
                            &target_inks_owned,
                            base_transform,
                            gs,
                            cs,
                            color_spaces,
                            resources,
                            ctx.doc,
                            clip,
                            &pipeline,
                            &mut backend,
                        )?;
                    } else {
                        for (i, &ink) in target_inks.iter().enumerate() {
                            if let PaintAction::Paint(tint) = tint_for_ink(
                                false,
                                gs,
                                color_spaces,
                                resources,
                                ctx.doc,
                                ink,
                                &cs.fill_components,
                                &cs.stroke_components,
                            ) {
                                stroke_separation(
                                    &mut pixmaps[i],
                                    &path,
                                    transform,
                                    gs,
                                    tint,
                                    clip,
                                );
                            }
                        }
                    }
                }
                current_path = PathBuilder::new();
            },
            Operator::EndPath => {
                apply_separation_clip(
                    &mut pending_clip,
                    &mut clip_stack,
                    pixmap_width,
                    pixmap_height,
                    base_transform,
                    &gs_stack,
                );
                current_path = PathBuilder::new();
            },

            Operator::ClipNonZero => {
                if let Some(path) = current_path.clone().finish() {
                    pending_clip = Some((path, FillRule::Winding));
                }
            },
            Operator::ClipEvenOdd => {
                if let Some(path) = current_path.clone().finish() {
                    pending_clip = Some((path, FillRule::EvenOdd));
                }
            },

            // Text object
            Operator::BeginText => {
                in_text_object = true;
                let gs = gs_stack.current_mut();
                gs.text_matrix = Matrix::identity();
                gs.text_line_matrix = Matrix::identity();
            },
            Operator::EndText => {
                in_text_object = false;
            },

            // Text state
            Operator::Tc { char_space } => {
                gs_stack.current_mut().char_space = *char_space;
            },
            Operator::Tw { word_space } => {
                gs_stack.current_mut().word_space = *word_space;
            },
            Operator::Tz { scale } => {
                gs_stack.current_mut().horizontal_scaling = *scale;
            },
            Operator::TL { leading } => {
                gs_stack.current_mut().leading = *leading;
            },
            Operator::Ts { rise } => {
                gs_stack.current_mut().text_rise = *rise;
            },
            Operator::Tr { render } => {
                gs_stack.current_mut().render_mode = *render;
            },
            Operator::Tf { font, size } => {
                let wmode = ctx.fonts.get(font).map(|f| f.wmode).unwrap_or(0);
                let gs = gs_stack.current_mut();
                gs.font_name = Some(font.clone());
                gs.font_size = *size;
                gs.text_wmode = wmode;
            },

            // Text positioning
            Operator::Td { tx, ty } => {
                if in_text_object {
                    let gs = gs_stack.current_mut();
                    let translation = Matrix::translation(*tx, *ty);
                    gs.text_line_matrix = translation.multiply(&gs.text_line_matrix);
                    gs.text_matrix = gs.text_line_matrix;
                }
            },
            Operator::TD { tx, ty } => {
                if in_text_object {
                    let gs = gs_stack.current_mut();
                    gs.leading = -(*ty);
                    let translation = Matrix::translation(*tx, *ty);
                    gs.text_line_matrix = translation.multiply(&gs.text_line_matrix);
                    gs.text_matrix = gs.text_line_matrix;
                }
            },
            Operator::Tm { a, b, c, d, e, f } => {
                if in_text_object {
                    let gs = gs_stack.current_mut();
                    gs.text_matrix = Matrix {
                        a: *a,
                        b: *b,
                        c: *c,
                        d: *d,
                        e: *e,
                        f: *f,
                    };
                    gs.text_line_matrix = gs.text_matrix;
                }
            },
            Operator::TStar => {
                if in_text_object {
                    let gs = gs_stack.current_mut();
                    let leading = gs.leading;
                    let translation = Matrix::translation(0.0, -leading);
                    gs.text_line_matrix = translation.multiply(&gs.text_line_matrix);
                    gs.text_matrix = gs.text_line_matrix;
                }
            },

            // Text showing. Each `render_*_to_plate` returns a scalar
            // advance along the active writing axis; the caller routes it
            // into the text matrix through `advance_text_matrix`, which
            // performs the H/V axis swap in one place.
            Operator::Tj { text } => {
                if in_text_object {
                    let advance = render_text_to_plate(
                        pixmaps,
                        text,
                        base_transform,
                        &mut gs_stack,
                        &color_state_stack,
                        color_spaces,
                        resources,
                        ctx,
                        clip_stack.last().and_then(|c| c.as_ref()),
                        target_inks,
                    )?;
                    gs_stack.current_mut().advance_text_matrix(advance);
                }
            },
            Operator::TJ { array } => {
                if in_text_object {
                    let advance = render_tj_to_plate(
                        pixmaps,
                        array,
                        base_transform,
                        &mut gs_stack,
                        &color_state_stack,
                        color_spaces,
                        resources,
                        ctx,
                        clip_stack.last().and_then(|c| c.as_ref()),
                        target_inks,
                    )?;
                    gs_stack.current_mut().advance_text_matrix(advance);
                }
            },
            Operator::Quote { text } => {
                if in_text_object {
                    let gs_mut = gs_stack.current_mut();
                    let leading = gs_mut.leading;
                    let translation = Matrix::translation(0.0, -leading);
                    gs_mut.text_line_matrix = translation.multiply(&gs_mut.text_line_matrix);
                    gs_mut.text_matrix = gs_mut.text_line_matrix;

                    let advance = render_text_to_plate(
                        pixmaps,
                        text,
                        base_transform,
                        &mut gs_stack,
                        &color_state_stack,
                        color_spaces,
                        resources,
                        ctx,
                        clip_stack.last().and_then(|c| c.as_ref()),
                        target_inks,
                    )?;
                    gs_stack.current_mut().advance_text_matrix(advance);
                }
            },
            Operator::DoubleQuote {
                word_space,
                char_space,
                text,
            } => {
                if in_text_object {
                    let gs_mut = gs_stack.current_mut();
                    gs_mut.word_space = *word_space;
                    gs_mut.char_space = *char_space;
                    let leading = gs_mut.leading;
                    let translation = Matrix::translation(0.0, -leading);
                    gs_mut.text_line_matrix = translation.multiply(&gs_mut.text_line_matrix);
                    gs_mut.text_matrix = gs_mut.text_line_matrix;

                    let advance = render_text_to_plate(
                        pixmaps,
                        text,
                        base_transform,
                        &mut gs_stack,
                        &color_state_stack,
                        color_spaces,
                        resources,
                        ctx,
                        clip_stack.last().and_then(|c| c.as_ref()),
                        target_inks,
                    )?;
                    gs_stack.current_mut().advance_text_matrix(advance);
                }
            },

            // ExtGState
            Operator::SetExtGState { dict_name } => {
                let entry = ext_g_state_cache
                    .entry(dict_name.clone())
                    .or_insert_with(|| {
                        if let Some(states) = ext_g_states {
                            if let Some(state_obj) = states.get(dict_name) {
                                return parse_ext_g_state_inner(state_obj, ctx.doc)
                                    .unwrap_or_default();
                            }
                        }
                        ParsedExtGState::default()
                    });
                entry.apply(gs_stack.current_mut());
            },

            // XObject — Form XObjects recurse into their content stream;
            // Image XObjects route per-channel samples to the matching ink
            // plates (§8.9, §11.7.4 default routing).
            Operator::Do { name } => {
                if let Some(xobjects) = xobjects_resolved.as_ref().and_then(|o| o.as_dict()) {
                    if let Some(xobj_ref_obj) = xobjects.get(name) {
                        if let Ok(xobj) = ctx.doc.resolve_object(xobj_ref_obj) {
                            if let Object::Stream { ref dict, .. } = xobj {
                                if let Some(subtype) = dict.get("Subtype").and_then(|o| o.as_name())
                                {
                                    if subtype == "Image" {
                                        let xobj_ref = xobj_ref_obj.as_reference();
                                        paint_image_to_plates(
                                            pixmaps,
                                            name,
                                            &xobj,
                                            xobj_ref,
                                            base_transform,
                                            &gs_stack,
                                            color_state_stack.last(),
                                            color_spaces,
                                            resources,
                                            ctx,
                                            clip_stack.last().and_then(|c| c.as_ref()),
                                            target_inks,
                                        )?;
                                    } else if subtype == "Form" {
                                        let xobj_ref = xobj_ref_obj.as_reference();
                                        let stream_data = if let Some(r) = xobj_ref {
                                            ctx.doc.decode_stream_with_encryption(&xobj, r)?
                                        } else {
                                            xobj.decode_stream_data()?
                                        };

                                        let form_resources =
                                            if let Some(res) = dict.get("Resources") {
                                                ctx.doc.resolve_object(res)?
                                            } else {
                                                resources.clone()
                                            };

                                        let form_cs = load_color_spaces(ctx.doc, &form_resources)?;
                                        let mut merged_cs = color_spaces.clone();
                                        merged_cs.extend(form_cs);

                                        let form_matrix = parse_form_matrix(dict);
                                        let gs = gs_stack.current();
                                        let combined = combine_transforms(base_transform, &gs.ctm)
                                            .pre_concat(form_matrix);

                                        // Inherit the calling context's colour state into the
                                        // form's initial graphics state (PDF §8.10.1, O5).
                                        let empty = SeparationColorState::new();
                                        let cs = color_state_stack.last().unwrap_or(&empty);
                                        let inherited = InheritedState {
                                            fill_color_space: gs.fill_color_space.clone(),
                                            stroke_color_space: gs.stroke_color_space.clone(),
                                            fill_color_cmyk: gs.fill_color_cmyk,
                                            stroke_color_cmyk: gs.stroke_color_cmyk,
                                            fill_components: cs.fill_components.clone(),
                                            stroke_components: cs.stroke_components.clone(),
                                            fill_overprint: gs.fill_overprint,
                                            stroke_overprint: gs.stroke_overprint,
                                            overprint_mode: gs.overprint_mode,
                                        };

                                        let form_ops = parse_content_stream(&stream_data)?;
                                        execute_separation_operators(
                                            pixmaps,
                                            combined,
                                            &form_ops,
                                            ctx,
                                            &form_resources,
                                            &merged_cs,
                                            Some(&inherited),
                                            target_inks,
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                }
            },

            _ => {},
        }
    }
    Ok(())
}

/// Render text into every target separation pixmap, routing each glyph
/// through the per-ink tint. The strategy is to clone the GraphicsState,
/// replace its fill colour with a grayscale paint equal to the tint, and
/// reuse the standard [`TextRasterizer`]. This preserves glyph shape,
/// kerning, and anti-aliasing — the same fidelity as the page renderer.
///
/// The returned advance is shared across all plates (the rasteriser is
/// deterministic for a given font/text/state, so each plate's advance
/// agrees) — we use the last computed value, matching the single-plate
/// behaviour. If no plate is touched (every plate's [`PaintAction`] is
/// `Skip`, or render mode 3) the advance is computed from the font
/// metrics so the text matrix still progresses correctly.
#[allow(clippy::too_many_arguments)]
fn render_text_to_plate(
    pixmaps: &mut [Pixmap],
    text: &[u8],
    base_transform: Transform,
    gs_stack: &mut GraphicsStateStack,
    color_state_stack: &[SeparationColorState],
    color_spaces: &HashMap<String, Object>,
    resources: &Object,
    ctx: &mut SeparationContext<'_>,
    clip: Option<&Mask>,
    target_inks: &[&str],
) -> Result<f32> {
    let gs = gs_stack.current();
    let empty = SeparationColorState::new();
    let cs = color_state_stack.last().unwrap_or(&empty);

    // Render mode 3 = invisible text; mode 7 = clip-only, no paint (§9.3.6,
    // WS1.5). Both advance the text matrix but skip painting.
    if gs.render_mode == 3 || gs.render_mode == 7 {
        return measure_text_advance(text, gs, ctx.fonts);
    }

    let transform = combine_transforms(base_transform, &gs.ctm);
    let mut painted_advance: Option<f32> = None;

    for (i, &ink) in target_inks.iter().enumerate() {
        let tint = match tint_for_ink(
            true,
            gs,
            color_spaces,
            resources,
            ctx.doc,
            ink,
            &cs.fill_components,
            &cs.stroke_components,
        ) {
            PaintAction::Paint(t) => t,
            PaintAction::Skip => continue,
        };

        // Build a faked-grayscale GraphicsState so the rasteriser paints in
        // (tint, tint, tint) which becomes the plate value in the R channel.
        let mut faux = gs.clone();
        faux.fill_color_rgb = (tint, tint, tint);
        faux.fill_alpha = 1.0;
        faux.blend_mode = "Normal".to_string();

        let advance = ctx.text_rasterizer.render_text(
            &mut pixmaps[i],
            text,
            transform,
            &faux,
            // The separation backend bakes its own faux grayscale into
            // `faux.fill_color_rgb`; the composite-side resolution pipeline
            // is not in play here, so no colour override is needed.
            None,
            resources,
            ctx.doc,
            clip,
            ctx.fonts,
        )?;
        painted_advance = Some(advance);
    }

    match painted_advance {
        Some(a) => Ok(a),
        // No plate was touched by this text — still advance the matrix so
        // subsequent glyphs land at the correct position.
        None => measure_text_advance(text, gs, ctx.fonts),
    }
}

/// Render a TJ array (sequence of strings + offsets) into all target
/// plates. Walks the array applying offsets between strings, painting
/// each string component via [`render_text_to_plate`].
#[allow(clippy::too_many_arguments)]
fn render_tj_to_plate(
    pixmaps: &mut [Pixmap],
    array: &[TextElement],
    base_transform: Transform,
    gs_stack: &mut GraphicsStateStack,
    color_state_stack: &[SeparationColorState],
    color_spaces: &HashMap<String, Object>,
    resources: &Object,
    ctx: &mut SeparationContext<'_>,
    clip: Option<&Mask>,
    target_inks: &[&str],
) -> Result<f32> {
    let mut total_advance = 0.0;
    for element in array {
        match element {
            TextElement::String(text) => {
                let advance = render_text_to_plate(
                    pixmaps,
                    text,
                    base_transform,
                    gs_stack,
                    color_state_stack,
                    color_spaces,
                    resources,
                    ctx,
                    clip,
                    target_inks,
                )?;
                gs_stack.current_mut().advance_text_matrix(advance);
                total_advance += advance;
            },
            TextElement::Offset(offset) => {
                let shift = (-*offset / 1000.0) * gs_stack.current().font_size;
                gs_stack.current_mut().advance_text_matrix(shift);
                total_advance += shift;
            },
        }
    }
    Ok(total_advance)
}

/// Compute the horizontal advance a [`TextRasterizer`] call would
/// produce, without painting. Used for invisible/skipped text so the
/// text matrix stays consistent with the painted ink plates.
///
/// Best-effort: when an embedded width table is unavailable we fall
/// back to `font_size * len * 0.5` — close enough to keep glyph
/// positions inside the rest of the line.
fn measure_text_advance(
    text: &[u8],
    gs: &GraphicsState,
    fonts: &HashMap<String, Arc<FontInfo>>,
) -> Result<f32> {
    let font_info = gs
        .font_name
        .as_ref()
        .and_then(|n| fonts.get(n))
        .map(Arc::clone);

    // Sum widths from the font's width table (in glyph units / 1000)
    // multiplied by font_size, plus per-char Tc spacing.
    let mut units: f32 = 0.0;
    let mut count: usize = 0;
    if let Some(info) = font_info.as_ref() {
        if info.subtype != "Type0" {
            for &b in text {
                units += info.get_glyph_width(b as u16);
                count += 1;
            }
        } else {
            // Type0: iterate 2-byte codes (approx).
            let mut i = 0;
            while i + 1 < text.len() {
                let code = ((text[i] as u16) << 8) | text[i + 1] as u16;
                units += info.get_glyph_width(code);
                count += 1;
                i += 2;
            }
        }
    } else {
        for _ in text {
            units += 500.0;
            count += 1;
        }
    }
    let advance = units * gs.font_size / 1000.0 + (count as f32) * gs.char_space;
    Ok(advance)
}

/// Fill a path into the separation pixmap with the given tint value.
///
/// `pub(crate)` so the resolution pipeline's [`super::resolution::SeparationBackend`]
/// can take it as a parity reference in its byte-for-byte equivalence test.
/// The shipping per-plate walker calls it directly; production callers
/// outside the renderer should not.
pub(crate) fn fill_separation(
    pixmap: &mut Pixmap,
    path: &tiny_skia::Path,
    transform: Transform,
    tint: f32,
    fill_rule: FillRule,
    clip: Option<&Mask>,
) {
    let gray = (tint.clamp(0.0, 1.0) * 255.0).round() as u8;
    let color = tiny_skia::Color::from_rgba8(gray, gray, gray, 255);
    let mut paint = tiny_skia::Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;
    // SourceOver with opaque (alpha=255) source = replacement; this matches
    // PDF's opaque painting model where each new fill overwrites the pixels
    // under it within the path. Overlapping fills are *not* accumulated —
    // PDF separation semantics dictate last-writer-wins per ink at the
    // overlapping pixels, which SourceOver gives us for free.
    paint.blend_mode = tiny_skia::BlendMode::SourceOver;

    guarded_fill_path(pixmap, path, &paint, fill_rule, transform, clip);
}

/// Stroke a path into the separation pixmap with the given tint value.
fn stroke_separation(
    pixmap: &mut Pixmap,
    path: &tiny_skia::Path,
    transform: Transform,
    gs: &GraphicsState,
    tint: f32,
    clip: Option<&Mask>,
) {
    let gray = (tint.clamp(0.0, 1.0) * 255.0).round() as u8;
    let color = tiny_skia::Color::from_rgba8(gray, gray, gray, 255);
    let mut paint = tiny_skia::Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;

    let mut stroke = tiny_skia::Stroke::default();
    stroke.width = gs.line_width;
    stroke.line_cap = match gs.line_cap {
        1 => tiny_skia::LineCap::Round,
        2 => tiny_skia::LineCap::Square,
        _ => tiny_skia::LineCap::Butt,
    };
    stroke.line_join = match gs.line_join {
        1 => tiny_skia::LineJoin::Round,
        2 => tiny_skia::LineJoin::Bevel,
        _ => tiny_skia::LineJoin::Miter,
    };
    stroke.miter_limit = gs.miter_limit;

    if !gs.dash_pattern.0.is_empty() {
        stroke.dash = tiny_skia::StrokeDash::new(gs.dash_pattern.0.clone(), gs.dash_pattern.1);
    }

    guarded_stroke_path(pixmap, path, &paint, &stroke, transform, clip);
}

/// Apply a pending clip path to the clip stack.
///
/// The clip mask is identical across all plates — it depends only on the
/// path, fill rule, current transform, and pixmap dimensions (which are
/// shared). So we build it once and store it on the shared clip stack.
fn apply_separation_clip(
    pending: &mut Option<(tiny_skia::Path, FillRule)>,
    clip_stack: &mut Vec<Option<Mask>>,
    pixmap_width: u32,
    pixmap_height: u32,
    base_transform: Transform,
    gs_stack: &GraphicsStateStack,
) {
    if let Some((path, fill_rule)) = pending.take() {
        // No pixmaps means no plates to clip — bail out early. Width/height
        // would be zero and Mask::new would refuse them anyway.
        if pixmap_width == 0 || pixmap_height == 0 {
            return;
        }
        let gs = gs_stack.current();
        let transform = combine_transforms(base_transform, &gs.ctm);

        // See `apply_pending_clip` in page_renderer: a clip path beyond f32
        // device precision is dropped, not materialized as an empty mask.
        if !device_bounds_rasterizable(&path, transform) {
            log::debug!("skipping clip beyond f32 device precision: {:?}", path.bounds());
            return;
        }

        if let Some(path_transformed) = path.transform(transform) {
            let mut new_mask = Mask::new(pixmap_width, pixmap_height).unwrap();
            new_mask.fill_path(&path_transformed, fill_rule, true, Transform::identity());

            if let Some(Some(current_mask)) = clip_stack.last() {
                let mut combined = current_mask.clone();
                let combined_data = combined.data_mut();
                let new_data = new_mask.data();
                for i in 0..combined_data.len() {
                    combined_data[i] = ((combined_data[i] as u32 * new_data[i] as u32) / 255) as u8;
                }
                *clip_stack.last_mut().unwrap() = Some(combined);
            } else {
                *clip_stack.last_mut().unwrap() = Some(new_mask);
            }
        }
    }
}

/// Parse a form XObject matrix from its dictionary.
fn parse_form_matrix(dict: &HashMap<String, Object>) -> Transform {
    if let Some(Object::Array(arr)) = dict.get("Matrix") {
        let get_f32 = |i: usize| -> f32 {
            match arr.get(i) {
                Some(Object::Real(v)) => *v as f32,
                Some(Object::Integer(v)) => *v as f32,
                _ => {
                    if i == 0 || i == 3 {
                        1.0
                    } else {
                        0.0
                    }
                },
            }
        };
        Transform::from_row(get_f32(0), get_f32(1), get_f32(2), get_f32(3), get_f32(4), get_f32(5))
    } else {
        Transform::identity()
    }
}

/// Combine two transformations (base + CTM).
fn combine_transforms(base: Transform, ctm: &Matrix) -> Transform {
    base.pre_concat(Transform::from_row(ctm.a, ctm.b, ctm.c, ctm.d, ctm.e, ctm.f))
}

/// Resolve the image XObject's declared `/ColorSpace` to a [`ResolvedSpace`].
///
/// Handles all three syntactic shapes the spec allows:
/// - `/DeviceCMYK` (a name) — direct, honouring `Default*` remap from the
///   page's `/Resources/ColorSpace` per §8.6.5.6.
/// - `[/Separation /InkName /Alt /Tint]` (inline array) — classified directly.
/// - An indirect reference to either of the above — resolved first.
fn resolve_image_color_space(
    image_dict: &HashMap<String, Object>,
    color_spaces: &HashMap<String, Object>,
    resources: &Object,
    doc: &PdfDocument,
) -> ResolvedSpace {
    let cs_obj = match image_dict.get("ColorSpace") {
        Some(o) => o,
        None => return ResolvedSpace::Unknown,
    };
    let resolved_obj = match cs_obj.as_reference() {
        Some(r) => match doc.load_object(r) {
            Ok(o) => o,
            Err(_) => return ResolvedSpace::Unknown,
        },
        None => cs_obj.clone(),
    };
    if let Some(name) = resolved_obj.as_name() {
        return resolve_color_space(name, color_spaces, resources, doc);
    }
    classify_resolved(&resolved_obj, color_spaces, resources, doc)
}

/// For a given source colour space and target ink, return the index of the
/// channel that contributes to that ink, or `None` when the ink is outside
/// the source's colorant set.
///
/// Matching mirrors `tint_for_ink`:
/// - DeviceCMYK / IccCmyk: Cyan/Magenta/Yellow/Black → 0/1/2/3, spots → None
/// - Separation(name): match against the named ink (and `/All`); else None
/// - DeviceN(names): position of the matching name (or `/All`)
/// - RGB / Gray / Unknown: None (no plate intent)
fn image_channel_for_ink(space: &ResolvedSpace, ink: &str) -> Option<usize> {
    match space {
        ResolvedSpace::Cmyk | ResolvedSpace::IccCmyk => match ink {
            "Cyan" => Some(0),
            "Magenta" => Some(1),
            "Yellow" => Some(2),
            "Black" => Some(3),
            _ => None,
        },
        ResolvedSpace::Separation(name) => {
            if name == "None" {
                None
            } else if name == "All" || name == ink {
                Some(0)
            } else {
                None
            }
        },
        ResolvedSpace::DeviceN(names) => names
            .iter()
            .position(|n| n.as_str() != "None" && (n == "All" || n == ink)),
        ResolvedSpace::Rgb
        | ResolvedSpace::Gray
        | ResolvedSpace::IccRgb
        | ResolvedSpace::IccGray
        | ResolvedSpace::Unknown => None,
    }
}

/// Pull samples for `target_channel` out of an interleaved 8-bpc image buffer
/// (`stride` bytes per pixel). Returns a `W*H` byte plane suitable for blitting
/// as the R channel of an opaque grayscale RGBA pixmap.
fn extract_image_channel(
    samples: &[u8],
    pixel_count: usize,
    stride: usize,
    target_channel: usize,
) -> Vec<u8> {
    let mut plane = Vec::with_capacity(pixel_count);
    for p in 0..pixel_count {
        let off = p * stride + target_channel;
        plane.push(samples.get(off).copied().unwrap_or(0));
    }
    plane
}

/// Blit a single-channel ink-coverage plane (`W*H` bytes, 0 = no ink, 255 =
/// full ink) into the destination separation pixmap at the image's
/// CTM-derived transform. The image is treated as occupying the PDF unit
/// square in user space; the formula mirrors `page_renderer.rs:2146-2148`
/// (pre-translate y by 1, pre-scale 1/w by -1/h to flip the row order).
fn blit_image_plane_to_plate(
    dst: &mut Pixmap,
    plane: &[u8],
    src_w: u32,
    src_h: u32,
    transform: Transform,
    clip: Option<&Mask>,
) {
    // Build an opaque RGBA buffer where every channel carries the plane
    // value. `Pixmap::draw_pixmap` then composites with SourceOver at
    // alpha 255 (opaque replacement), so the R channel at the destination
    // ends up equal to the source value — the same convention as
    // `fill_separation`.
    let n = (src_w as usize) * (src_h as usize);
    if plane.len() < n {
        return;
    }
    let mut rgba = Vec::with_capacity(n * 4);
    for &v in &plane[..n] {
        rgba.extend_from_slice(&[v, v, v, 255]);
    }
    let Some(size) = tiny_skia::IntSize::from_wh(src_w, src_h) else {
        return;
    };
    let Some(src) = Pixmap::from_vec(rgba, size) else {
        return;
    };
    let image_transform = transform
        .pre_translate(0.0, 1.0)
        .pre_scale(1.0 / src_w as f32, -1.0 / src_h as f32);
    let mut paint = tiny_skia::PixmapPaint::default();
    paint.blend_mode = tiny_skia::BlendMode::SourceOver;
    paint.quality = tiny_skia::FilterQuality::Bilinear;
    dst.draw_pixmap(0, 0, src.as_ref(), &paint, image_transform, clip);
}

/// Returns true if the image XObject's `/Filter` chain contains a filter we
/// can't decode. `/JPXDecode` (JPEG 2000) is decodable only when the `jpeg2000`
/// feature is built in; without it, JPX images are still skipped here.
fn image_has_unsupported_filter(image_dict: &HashMap<String, Object>) -> bool {
    let filter = match image_dict.get("Filter") {
        Some(f) => f,
        None => return false,
    };
    let names: Vec<&str> = match filter {
        Object::Name(n) => vec![n.as_str()],
        Object::Array(arr) => arr.iter().filter_map(|o| o.as_name()).collect(),
        _ => vec![],
    };
    #[cfg(feature = "jpeg2000")]
    {
        let _ = &names;
        false
    }
    #[cfg(not(feature = "jpeg2000"))]
    {
        names.iter().any(|f| matches!(*f, "JPXDecode" | "J2"))
    }
}

/// Paint an image XObject into the separation plates.
///
/// Per ISO 32000-1 §11.7.4 image samples are routed channel-by-channel to
/// the matching ink plates. §11.7.4.3 explicitly carves images out of the
/// `OPM` rule — this function never consults `gs.overprint_mode`.
///
/// Currently in scope:
/// - DeviceCMYK / ICCBased(N=4) images → C/M/Y/K plates
/// - Separation images → the named spot plate
/// - DeviceN images → per-channel routing by colorant name
/// - Image masks (`/ImageMask true`) → paint the current fill colour through
///   the 1-bit stencil (delegates to `tint_for_ink` for spot/process logic)
/// - JPX-filtered images logged and skipped (no decoder bundled)
///
/// Out of scope, dropped silently for now: RGB/Gray images, indexed images,
/// inline images. See module-level Limitations.
#[allow(clippy::too_many_arguments)]
fn paint_image_to_plates(
    pixmaps: &mut [Pixmap],
    name: &str,
    xobject: &Object,
    obj_ref: Option<crate::object::ObjectRef>,
    base_transform: Transform,
    gs_stack: &GraphicsStateStack,
    color_state: Option<&SeparationColorState>,
    color_spaces: &HashMap<String, Object>,
    resources: &Object,
    ctx: &SeparationContext<'_>,
    clip: Option<&Mask>,
    target_inks: &[&str],
) -> Result<()> {
    use crate::extractors::images::{
        extract_image_from_xobject, ColorSpace as PdfCs, ImageData, PixelFormat,
    };

    let dict = match xobject {
        Object::Stream { dict, .. } => dict,
        _ => return Ok(()),
    };

    // §8.9.6.2: image masks are 1-bpc stencils painted with the current
    // non-stroking colour, not channel-bearing images. Route through the
    // same per-plate logic as a vector fill.
    let is_image_mask = dict
        .get("ImageMask")
        .map(|o| matches!(o, Object::Boolean(true)))
        .unwrap_or(false);
    if is_image_mask {
        return paint_image_mask_to_plates(
            pixmaps,
            name,
            xobject,
            obj_ref,
            base_transform,
            gs_stack,
            color_state,
            color_spaces,
            resources,
            ctx,
            clip,
            target_inks,
        );
    }

    // §D3: JPX images get a debug log and are dropped — no pure-Rust JP2
    // decoder is bundled.
    if image_has_unsupported_filter(dict) {
        log::warn!(
            "Skipping image XObject '{name}' on separation plates: \
             unsupported filter (JPXDecode — JPEG 2000 decoder not bundled)"
        );
        return Ok(());
    }

    // Resolve the image's declared colour space, honouring DefaultCMYK etc.
    let resolved_space = resolve_image_color_space(dict, color_spaces, resources, ctx.doc);

    // For RGB / Gray / Unknown the image carries no ink-coverage intent.
    // Skip entirely; underlying plates are left untouched. pdf_oxide does
    // not synthesise CMYK from RGB because no deterministic UCR/BG
    // strategy is in place. Matches tint_for_ink's vector treatment.
    let needs_4ch = matches!(resolved_space, ResolvedSpace::Cmyk | ResolvedSpace::IccCmyk);
    let needs_separation = matches!(resolved_space, ResolvedSpace::Separation(_));
    let needs_devicen = matches!(resolved_space, ResolvedSpace::DeviceN(_));
    if !(needs_4ch || needs_separation || needs_devicen) {
        log::debug!(
            "Skipping image XObject '{name}' on separation plates: \
             source colour space has no subtractive-ink intent"
        );
        return Ok(());
    }

    let pdf_image =
        match extract_image_from_xobject(Some(ctx.doc), xobject, obj_ref, Some(color_spaces)) {
            Ok(img) => img,
            Err(e) => {
                log::warn!("Skipping image XObject '{name}': {e}");
                return Ok(());
            },
        };
    let w = pdf_image.width() as usize;
    let h = pdf_image.height() as usize;
    let pixel_count = w * h;
    if pixel_count == 0 {
        return Ok(());
    }

    // §8.9.5: BitsPerComponent ∈ {1, 2, 4, 8, 16}. Channel extraction below
    // assumes 8 bits per sample (one byte per channel per pixel) and would
    // mis-read packed sub-byte or 16-bit streams. Until the routing path
    // supports full BPC expansion, skip with a log entry — matching the
    // JPX carve-out.
    let bpc = pdf_image.bits_per_component();
    if bpc != 8 {
        log::warn!(
            "Skipping image XObject '{name}' on separation plates: \
             BitsPerComponent={bpc} not supported (only 8-bpc channel \
             routing is implemented; 1/2/4/16-bpc expansion pending)"
        );
        return Ok(());
    }

    let extractor_cs = pdf_image.color_space();

    // Extract interleaved raw samples in the source colour space. The
    // extractor exposes RGB after Indexed expansion; for separation
    // routing we only consume CMYK / Separation / DeviceN paths (the
    // shapes above), so anything else falls through to skip.
    // `decode_pre_applied`: whether /Decode is ALREADY mapped into these
    // samples (ISO 32000-1 §8.9.5.2), in which case re-applying it here would
    // double-invert. For extractor-provided Raw buffers this is exactly what
    // the extractor recorded — notably it is false when the extractor could
    // not interpret the array (e.g. a DeviceN whose ink count differs from
    // the colour space's declared component count), so this path still owes
    // the image its /Decode. JPEG samples are decoded locally and always do.
    // Deliberately not `!samples_are_raw()`: that is the wider "were these
    // samples rescaled at all" fact, true for every sub-byte and 16-bit
    // image, and reading it here drops the /Decode those images are owed.
    let extractor_decode_applied = pdf_image.decode_folded_in();
    let (samples, stride, decode_pre_applied) =
        match (resolved_space.clone(), extractor_cs, pdf_image.data()) {
            // Raw CMYK pixel buffer (Flate / CCITT / etc. on a DeviceCMYK image).
            (
                ResolvedSpace::Cmyk | ResolvedSpace::IccCmyk,
                PdfCs::DeviceCMYK | PdfCs::ICCBased(4),
                ImageData::Raw {
                    pixels,
                    format: PixelFormat::CMYK,
                },
            ) => (pixels.clone(), 4usize, extractor_decode_applied),
            // JPEG-encoded DeviceCMYK image — decode to raw CMYK preserving APP14 inversion.
            (
                ResolvedSpace::Cmyk | ResolvedSpace::IccCmyk,
                PdfCs::DeviceCMYK | PdfCs::ICCBased(4),
                ImageData::Jpeg(bytes),
            ) => (crate::extractors::images::decode_cmyk_jpeg_to_raw_cmyk(bytes)?, 4, false),
            // Separation: 1 channel.
            (ResolvedSpace::Separation(_), PdfCs::Separation, ImageData::Raw { pixels, .. }) => {
                (pixels.clone(), 1, extractor_decode_applied)
            },
            // DeviceN: N channels (extractor reports DeviceN with N components).
            (ResolvedSpace::DeviceN(ref names), PdfCs::DeviceN, ImageData::Raw { pixels, .. }) => {
                (pixels.clone(), names.len().max(1), extractor_decode_applied)
            },
            // Shape mismatch (e.g. extractor reports a different colour space than
            // the dict declared after our resolver ran). Drop silently — the
            // resolver result wins for routing semantics but we won't fabricate
            // channels we don't have.
            _ => {
                log::debug!(
                    "Image XObject '{name}': shape mismatch between resolved colour space \
                 and extractor sample format; skipping"
                );
                return Ok(());
            },
        };
    let _ = color_state; // currently unused outside the image-mask path

    // §8.9.5.2: /Decode maps raw sample values into the colour space's range.
    // For per-plate routing the colour space is treated as identity, so the
    // only effect that matters is inversion (`/Decode [1 0]` on a Separation
    // image, etc.). Default identity is `[0 1]` per channel. Only consulted
    // for sample sources the extractor has not already mapped.
    let decode = if decode_pre_applied {
        None
    } else {
        read_decode_array(dict, stride)
    };

    let gs = gs_stack.current();
    let transform = combine_transforms(base_transform, &gs.ctm);

    for (i, &ink) in target_inks.iter().enumerate() {
        let Some(channel_idx) = image_channel_for_ink(&resolved_space, ink) else {
            continue;
        };
        if channel_idx >= stride {
            continue;
        }
        let mut plane = extract_image_channel(&samples, pixel_count, stride, channel_idx);
        if let Some(decode_pairs) = decode.as_ref() {
            if let Some(&(dmin, dmax)) = decode_pairs.get(channel_idx) {
                apply_decode_to_plane(&mut plane, dmin, dmax);
            }
        }
        blit_image_plane_to_plate(&mut pixmaps[i], &plane, w as u32, h as u32, transform, clip);
    }
    Ok(())
}

/// Expand a 1-bpc packed bitmap into one byte per pixel (0 or 255).
///
/// Per §8.9.5.1 each row is packed MSB-first into `ceil(width / 8)` bytes;
/// trailing bits in the final byte of each row are padding. Used to
/// normalise `/ImageMask true` stencils before they're blitted onto plates.
fn expand_1bpc_to_8bpc(packed: &[u8], width: u32, height: u32) -> Vec<u8> {
    let row_bytes = width.div_ceil(8) as usize;
    let w = width as usize;
    let h = height as usize;
    let mut out = Vec::with_capacity(w * h);
    for row in 0..h {
        let row_start = row * row_bytes;
        for col in 0..w {
            let byte_idx = row_start + col / 8;
            let bit_idx = 7 - (col % 8);
            let bit = packed
                .get(byte_idx)
                .map(|b| (*b >> bit_idx) & 1)
                .unwrap_or(0);
            out.push(if bit == 1 { 255 } else { 0 });
        }
    }
    out
}

/// Read the image's `/Decode` array as per-channel `(dmin, dmax)` pairs.
/// Returns `None` if the entry is absent or malformed; callers fall back
/// to the identity mapping (no remap).
fn read_decode_array(
    dict: &HashMap<String, Object>,
    num_components: usize,
) -> Option<Vec<(f32, f32)>> {
    let decode = dict.get("Decode")?;
    let arr = decode.as_array()?;
    if arr.len() < num_components * 2 {
        return None;
    }
    let to_f32 = |o: &Object| -> Option<f32> {
        match o {
            Object::Real(r) => Some(*r as f32),
            Object::Integer(i) => Some(*i as f32),
            _ => None,
        }
    };
    let mut out = Vec::with_capacity(num_components);
    for i in 0..num_components {
        let dmin = to_f32(&arr[i * 2])?;
        let dmax = to_f32(&arr[i * 2 + 1])?;
        out.push((dmin, dmax));
    }
    Some(out)
}

/// Apply a single channel's `/Decode` pair to an extracted 8-bpc plane.
///
/// For the identity mapping (`dmin = 0`, `dmax = 1`) this is a no-op. For
/// `[1 0]` this inverts the plane (raw 0 → 1.0 → 255, raw 255 → 0.0 → 0).
fn apply_decode_to_plane(plane: &mut [u8], dmin: f32, dmax: f32) {
    if dmin == 0.0 && dmax == 1.0 {
        return;
    }
    for byte in plane.iter_mut() {
        let raw = *byte as f32 / 255.0;
        let decoded = (dmin + raw * (dmax - dmin)).clamp(0.0, 1.0);
        *byte = (decoded * 255.0).round() as u8;
    }
}

/// Paint an image mask (`/ImageMask true`) into the separation plates.
///
/// §8.9.6.2: the image samples are a 1-bpc stencil. The colour comes from
/// the current non-stroking graphics state, exactly as a vector fill would.
/// Each plate's tint comes from `tint_for_ink` for the current fill colour;
/// the stencil's alpha multiplies the paint into the destination.
#[allow(clippy::too_many_arguments)]
fn paint_image_mask_to_plates(
    pixmaps: &mut [Pixmap],
    name: &str,
    xobject: &Object,
    obj_ref: Option<crate::object::ObjectRef>,
    base_transform: Transform,
    gs_stack: &GraphicsStateStack,
    color_state: Option<&SeparationColorState>,
    color_spaces: &HashMap<String, Object>,
    resources: &Object,
    ctx: &SeparationContext<'_>,
    clip: Option<&Mask>,
    target_inks: &[&str],
) -> Result<()> {
    // §8.9.6.2 ImageMask: the dict has no /ColorSpace, so the standard
    // image-extraction path rejects it. Read width/height/bpc directly from
    // the dict and decode the stream ourselves.
    let dict = match xobject {
        Object::Stream { dict, .. } => dict,
        _ => return Ok(()),
    };
    // Same geometry contract as the stencil path in `page_renderer`, which
    // already rejects non-positive and unbacked dimensions before allocating.
    let layout = crate::rendering::page_renderer::PageRenderer::image_mask_layout(dict);
    let Ok((w32, h32, _row_bytes, packed_len, _rgba_len)) = layout else {
        log::warn!("Skipping image mask '{name}': {}", layout.unwrap_err());
        return Ok(());
    };
    let (w, h) = (w32 as usize, h32 as usize);
    let pixel_count = w * h;
    let bpc = dict
        .get("BitsPerComponent")
        .and_then(|o| o.as_integer())
        .unwrap_or(1) as u8;
    if bpc != 1 {
        log::warn!(
            "Skipping image mask '{name}': BitsPerComponent={bpc} out of spec \
             (§8.9.6.2 mandates 1-bpc)"
        );
        return Ok(());
    }

    let packed = if let Some(r) = obj_ref {
        ctx.doc.decode_stream_with_encryption(xobject, r)?
    } else {
        xobject.decode_stream_data()?
    };
    // Checked before expanding, because `expand_1bpc_to_8bpc` zero-pads and
    // would size the buffer from the declaration: `/Width 2147483648 /Height
    // 2147483648` is representable and asks for 2^62 bytes.
    if packed.len() < packed_len {
        log::warn!(
            "Skipping image mask '{name}': {w}x{h} needs {packed_len} bytes, the stream \
             carries {}",
            packed.len()
        );
        return Ok(());
    }
    let mut stencil = expand_1bpc_to_8bpc(&packed, w32, h32);

    // §8.9.6.2: decoded sample value 0 marks the pixel with the current
    // colour; value 1 leaves it transparent. /Decode defaults to [0 1] —
    // applied here so /Decode [1 0] correctly inverts the stencil — then we
    // map decoded → alpha as `255 - decoded` so the existing SourceOver
    // composite paints where the spec says to paint.
    if let Some(decode_pairs) = read_decode_array(dict, 1) {
        if let Some(&(dmin, dmax)) = decode_pairs.first() {
            apply_decode_to_plane(&mut stencil, dmin, dmax);
        }
    }
    for byte in stencil.iter_mut() {
        *byte = 255 - *byte;
    }

    let gs = gs_stack.current();
    let transform = combine_transforms(base_transform, &gs.ctm);
    let empty = SeparationColorState::new();
    let cs = color_state.unwrap_or(&empty);

    for (i, &ink) in target_inks.iter().enumerate() {
        let PaintAction::Paint(tint) = tint_for_ink(
            true,
            gs,
            color_spaces,
            resources,
            ctx.doc,
            ink,
            &cs.fill_components,
            &cs.stroke_components,
        ) else {
            continue;
        };
        let gray = (tint.clamp(0.0, 1.0) * 255.0).round() as u8;

        // Build an RGBA buffer where R=G=B=gray and A=stencil_byte. SourceOver
        // composites this against the destination so opaque-stencil pixels
        // replace the plate value with `gray`; transparent-stencil pixels
        // leave the plate untouched.
        let mut rgba = Vec::with_capacity(pixel_count * 4);
        for &alpha in &stencil[..pixel_count] {
            rgba.extend_from_slice(&[gray, gray, gray, alpha]);
        }
        let Some(size) = tiny_skia::IntSize::from_wh(w as u32, h as u32) else {
            continue;
        };
        let Some(src) = Pixmap::from_vec(rgba, size) else {
            continue;
        };
        let image_transform = transform
            .pre_translate(0.0, 1.0)
            .pre_scale(1.0 / w as f32, -1.0 / h as f32);
        let mut paint = tiny_skia::PixmapPaint::default();
        paint.blend_mode = tiny_skia::BlendMode::SourceOver;
        paint.quality = tiny_skia::FilterQuality::Bilinear;
        pixmaps[i].draw_pixmap(0, 0, src.as_ref(), &paint, image_transform, clip);
    }
    Ok(())
}
