//! Materialize geometry-confirmed grids as editable rhwp tables. Uncertain grids stay positional.
mod body;
mod composition;
mod native_borders;
mod wrap_width;
use super::{
    allocation::{message, Allocation},
    lowering::hu,
    recognition::{ParagraphProposal, ParagraphState},
    text_analysis::{reconstruct_line, LineJoin, ParagraphLayout, TextAlignment, TextLine},
    value_macro::request_json,
    ExtractedPage,
};
use crate::model::{
    control::Control,
    document::Document,
    paragraph::{CharShapeRef, LineSeg, Paragraph},
    shape::{CommonObjAttr, HorzRelTo, ShapeObject, TextWrap, VertRelTo},
    style::{
        border_width_index, Alignment, BorderLineType, LineSpacingType, TabDef, TabItem,
        PARA_ATTR2_LINE_WRAP_MASK,
    },
    table::{Cell, Table, TablePageBreak, VerticalAlign},
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Source spans refer to document IDs, but glyph fitting only reads this page's
/// source styles. Keep those IDs in the document and resolve a compact view for
/// the common measurer, without repeatedly copying all earlier pages' styles.
struct SourceStyles {
    request: std::sync::Arc<pdf_oxide::import_limits::ImportBudget>,
    ids: BTreeMap<u32, u32>,
    resolved: crate::renderer::style_resolver::ResolvedStyleSet,
    widths: std::cell::RefCell<BTreeMap<GlyphWidthKey, f64>>,
}

type GlyphWidthKey = (u32, usize, char, Option<(u8, i8)>);

type StyleCheckpoint = (usize, usize, usize, usize);
fn checkpoint(doc: &Document, typography: &std::cell::RefCell<Vec<Value>>) -> StyleCheckpoint {
    (
        doc.doc_info.para_shapes.len(),
        doc.doc_info.char_shapes.len(),
        doc.doc_info.tab_defs.len(),
        typography.borrow().len(),
    )
}
fn rollback(doc: &mut Document, typography: &std::cell::RefCell<Vec<Value>>, c: StyleCheckpoint) {
    doc.doc_info.para_shapes.truncate(c.0);
    doc.doc_info.char_shapes.truncate(c.1);
    doc.doc_info.tab_defs.truncate(c.2);
    typography.borrow_mut().truncate(c.3);
}

/// A source style and language slot are immutable for this page. Only the
/// ordinary ratio/spacing fields vary during fitting. Memoize exact calls to
/// the common measurer, including the unfitted source style as a distinct key.
/// No font metric, rounding, search order or candidate tolerance is replaced.
struct GlyphWidths<'a> {
    source: &'a SourceStyles,
    style_id: u32,
    lang: usize,
    character: char,
}
impl GlyphWidths<'_> {
    fn measure(
        &self,
        style: &crate::renderer::TextStyle,
        fit: Option<(u8, i8)>,
    ) -> Result<f64, String> {
        self.source.request.check().map_err(|e| e.to_string())?;
        let key = (self.style_id, self.lang, self.character, fit);
        if let Some(width) = self.source.widths.borrow().get(&key) {
            return Ok(*width);
        }
        let mut utf8 = [0; 4];
        let width = crate::renderer::layout::estimate_text_width_exact(
            self.character.encode_utf8(&mut utf8),
            style,
        );
        // Bound auxiliary memory independently of input glyph count. Reaching
        // the memo limit changes only reuse, never acceptance or measurement.
        let mut widths = self.source.widths.borrow_mut();
        if widths.len() < 16_384 {
            Allocation(&self.source.request)
                .tree_entry::<GlyphWidthKey, f64>()
                .map_err(message)?;
            widths.insert(key, width);
        }
        Ok(width)
    }
}
impl SourceStyles {
    fn new(
        doc: &Document,
        shapes: &[Option<u32>],
        request: &std::sync::Arc<pdf_oxide::import_limits::ImportBudget>,
    ) -> Result<Self, String> {
        let allocation = Allocation(request);
        let mut ids = allocation
            .collect(shapes.iter().flatten().copied())
            .map_err(message)?;
        ids.sort_unstable();
        ids.dedup();
        let mut char_styles = allocation.vec(ids.len()).map_err(message)?;
        for &id in &ids {
            let shape = doc
                .doc_info
                .char_shapes
                .get(id as usize)
                .ok_or("missing PDF source character style")?;
            char_styles.push(
                crate::renderer::style_resolver::try_resolve_single_char_style(
                    shape,
                    &doc.doc_info,
                    96.,
                    &allocation,
                )
                .map_err(message)?,
            );
        }
        // The empty projection owns one Arc node; no input font table or
        // character record is copied just to resolve this page's selected IDs.
        allocation
            .bytes(
                2 * std::mem::size_of::<usize>()
                    + std::mem::size_of::<crate::renderer::native_pua_fonts::NativePuaFonts>(),
            )
            .map_err(message)?;
        let resolved = crate::renderer::style_resolver::ResolvedStyleSet {
            char_styles,
            resolved_dpi: Some(96.),
            ..Default::default()
        };
        let mut mapping = BTreeMap::new();
        for (local, id) in ids.into_iter().enumerate() {
            allocation.tree_entry::<u32, u32>().map_err(message)?;
            mapping.insert(id, local as u32);
        }
        Ok(Self {
            request: std::sync::Arc::clone(request),
            ids: mapping,
            resolved,
            widths: Default::default(),
        })
    }
    fn text_style(&self, id: u32, lang: usize) -> Result<crate::renderer::TextStyle, String> {
        let local = *self
            .ids
            .get(&id)
            .ok_or("missing PDF source style projection")?;
        let allocation = Allocation(&self.request);
        let style = self
            .resolved
            .char_styles
            .get(local as usize)
            .ok_or("missing PDF resolved character style")?;
        allocation
            .bytes(std::mem::size_of::<crate::renderer::TextStyle>())
            .map_err(message)?;
        let reserve_family = |language| -> Result<(), String> {
            allocation
                .bytes(style.font_family_for_lang(language).len())
                .map_err(message)?;
            if let Some(face) = style.metric_face_for_lang(language) {
                allocation.bytes(face.len()).map_err(message)?;
            }
            if let Some(face) = style.original_glyph_metric_face_for_lang(language) {
                allocation
                    .bytes(2 * std::mem::size_of::<usize>() + std::mem::size_of::<String>())
                    .map_err(message)?;
                allocation.bytes(face.len()).map_err(message)?;
            }
            Ok(())
        };
        reserve_family(lang)?;
        if lang != 1 && style.ascii_punct_latin_slot {
            allocation
                .bytes(std::mem::size_of::<crate::renderer::LatinSlotMetrics>())
                .map_err(message)?;
            reserve_family(1)?;
        }
        Ok(crate::renderer::layout::resolved_to_text_style(
            &self.resolved,
            local,
            lang,
        ))
    }
}

fn source_baseline_position(
    page: &ExtractedPage,
    doc: &Document,
    baseline: f32,
    span: usize,
    base_id: u32,
) -> Result<(i8, f64), String> {
    let source_delta = page.media_box.3 - page.spans[span].bbox.y - baseline;
    let size = doc.doc_info.char_shapes[base_id as usize].base_size as f64 / 100.;
    let position = (f64::from(source_delta) / size * 100.).round();
    if !position.is_finite() || !(-100. ..=100.).contains(&position) {
        return Err("source baseline displacement exceeds HWP character position range".into());
    }
    Ok((
        position as i8,
        (size * position / 100. - f64::from(source_delta)).abs(),
    ))
}

/// Source baseline is independent of whether horizontal reconstruction succeeds.
/// Both the legacy tab frame and ordinary paragraphs must retain this property.
fn positioned_span_style(
    page: &ExtractedPage,
    doc: &mut Document,
    line: &TextLine,
    span: usize,
    base_id: u32,
    cache: &mut BTreeMap<(u32, u8, i8, i8), u32>,
) -> Result<u32, String> {
    let allocation = Allocation(&page.import_budget);
    let (position, _) = source_baseline_position(page, doc, line.baseline, span, base_id)?;
    if position == 0 {
        return Ok(base_id);
    }
    let base = &doc.doc_info.char_shapes[base_id as usize];
    let key = (base_id, base.ratios[0], base.spacings[0], position);
    if let Some(id) = cache.get(&key) {
        return Ok(*id);
    }
    if doc.doc_info.char_shapes.len() >= u32::MAX as usize {
        return Err("PDF character style budget exceeded".into());
    }
    let mut style = allocation.clone_record(base).map_err(message)?;
    style.raw_data = None;
    style.char_offsets = [position; 7];
    let id = doc.doc_info.char_shapes.len() as u32;
    allocation
        .push(&mut doc.doc_info.char_shapes, style)
        .map_err(message)?;
    allocation
        .tree_entry::<(u32, u8, i8, i8), u32>()
        .map_err(message)?;
    cache.insert(key, id);
    Ok(id)
}

/// Fit the ordinary spacing field after selecting an ordinary ratio. The
/// common final measurer, including font-space and punctuation rules, remains
/// the authority for the emitted advance.
fn fit_spacing(
    style: &mut crate::renderer::TextStyle,
    widths: &GlyphWidths<'_>,
    ratio: u8,
    desired: f64,
) -> Result<Option<(i8, f64, f64)>, String> {
    style.ratio = f64::from(ratio) / 100.;
    style.letter_spacing = 0.;
    if let Some(latin) = &mut style.ascii_punct_latin {
        latin.ratio = style.ratio;
        latin.letter_spacing = 0.;
    }
    let nominal = widths.measure(style, Some((ratio, 0)))?;
    let guess = (desired / nominal - 1.) * 100.;
    if !nominal.is_finite()
        || nominal <= 0.
        || !guess.is_finite()
        || !(-52. ..=52.).contains(&guess)
    {
        return Ok(None);
    }
    let mut best: Option<(i8, f64, f64)> = None;
    for spacing in (guess.floor() as i32 - 2)..=(guess.ceil() as i32 + 2) {
        if !(-50..=50).contains(&spacing) {
            continue;
        }
        style.letter_spacing = style.font_size * f64::from(spacing) / 100.;
        if let Some(latin) = &mut style.ascii_punct_latin {
            latin.letter_spacing = style.letter_spacing;
        }
        let measured = widths.measure(style, Some((ratio, spacing as i8)))?;
        let error = (measured - desired).abs();
        if best.is_none_or(|(_, previous, _)| error < previous) {
            best = Some((spacing as i8, error, measured));
        }
    }
    Ok(best)
}

/// A blank separator has an observed extent, not an outline whose aspect ratio
/// must be recovered. Reject only when the combined legal ratio/spacing fields
/// cannot represent it, rather than requiring ratio alone to own the full gap.
fn fit_whitespace_advance(
    style: &mut crate::renderer::TextStyle,
    widths: &GlyphWidths<'_>,
    desired: f64,
) -> Result<Option<(u8, i8, f64)>, String> {
    let mut best: Option<(u8, i8, f64, f64)> = None;
    for ratio in 50..=200 {
        let Some((spacing, error, measured)) = fit_spacing(style, widths, ratio, desired)? else {
            continue;
        };
        let preference = (spacing.unsigned_abs(), ratio.abs_diff(100));
        if best.is_none_or(|(r, s, previous, _)| {
            error < previous
                || (error == previous && preference < (s.unsigned_abs(), r.abs_diff(100)))
        }) {
            best = Some((ratio, spacing, error, measured));
        }
    }
    Ok(best.map(|(r, s, _, measured)| (r, s, measured)))
}

/// Translate observed glyph metrics into the ordinary HWP character contract.
/// Source-font loss is explicit: width-compatible fallback does not recover its outline.
struct ObservedLineText {
    text: String,
    runs: Vec<(u32, u32)>,
    tabs: Vec<TabItem>,
    extensions: Vec<[u16; 7]>,
    positioning: Vec<PositioningTab>,
    evidence: Value,
}

struct PositioningTab {
    content_utf16: u32,
    cursor_px: f64,
    stop: u32,
}

fn observed_line_text(
    page: &ExtractedPage,
    doc: &mut Document,
    line: &TextLine,
    shape_ids: &[Option<u32>],
    styles: &SourceStyles,
    cache: &mut BTreeMap<(u32, u8, i8, i8), u32>,
    owner_x: f32,
    paragraph_baseline: f32,
) -> Result<ObservedLineText, String> {
    use crate::renderer::style_resolver::detect_lang_category;
    let allocation = Allocation(&page.import_budget);
    let content = reconstruct_line(&page.spans, &line.indices, allocation)?;
    let mut runs = Vec::new();
    let mut text = String::new();
    let mut tabs = Vec::new();
    let mut extensions = Vec::new();
    let mut positioning = Vec::new();
    let mut positioning_tabs = Vec::new();
    let mut offset = 0;
    let mut maximum_error: f64 = 0.;
    let mut maximum_baseline_error: f64 = 0.;
    let mut maximum_origin_error: f64 = 0.;
    let mut joint_whitespace_fits = 0;
    let mut source_space_movements = Vec::new();
    let origin = content
        .characters
        .first()
        .ok_or("empty source line")?
        .origin;
    let mut emitted_advance = 0.;
    for glyph in &content.characters {
        let base_id = shape_ids[glyph.span].ok_or("missing source style")?;
        let source = &page.spans[glyph.span];
        // PDF font size already includes script scaling. Preserve it and encode
        // only the observed displacement in ordinary HWP CHAROFFSET; setting
        // the superscript flag would shrink a small PDF glyph a second time.
        let (position, baseline_error) =
            source_baseline_position(page, doc, paragraph_baseline, glyph.span, base_id)?;
        maximum_baseline_error = maximum_baseline_error.max(baseline_error);
        let lang = detect_lang_category(glyph.character);
        let mut style = styles.text_style(base_id, lang)?;
        let widths = GlyphWidths {
            source: styles,
            style_id: base_id,
            lang,
            character: glyph.character,
        };
        // Glyph reconstruction and actual run placement must use the same
        // fractional width. Rounding each scalar can invent an unsupported
        // ratio (e.g. a 2.5pt TJ separator measured as 7 instead of 6.667px).
        let base_width = widths.measure(&style, None)?;
        if !base_width.is_finite() || base_width <= 0. {
            return Err("glyph advance needs separate shaping".into());
        }
        let fallback = source.font_name.is_empty() || source.font_name == "Unknown";
        // A missing source font needs metric-compatible fallback; declared fonts
        // retain their Tz scaling. Spaces have no outline and own their measured extent.
        let ratio = if glyph.character.is_whitespace() {
            // Whitespace has no outline to preserve. Its full observed advance
            // includes TJ/Tw and displacement across text-show boundaries; using
            // only the source font's nominal space can falsely exceed spacing's
            // range and discard an otherwise representable line.
            (f64::from(glyph.advance) * 96. / 72. / base_width * 100.).round()
        } else if fallback {
            (f64::from(glyph.nominal_advance) * 96. / 72. / base_width * 100.).round()
        } else {
            f64::from(source.horizontal_scaling).round()
        };
        let source_start = (f64::from(glyph.origin) - f64::from(origin)) * 96. / 72.;
        let source_advance = f64::from(glyph.advance) * 96. / 72.;
        let desired = source_start + source_advance - emitted_advance;
        // Preserve the established representable path. Search the combined
        // field contract only when that path cannot encode a blank separator.
        let ordinary = if (50. ..=200.).contains(&ratio) {
            fit_spacing(&mut style, &widths, ratio as u8, desired)?
                .map(|(s, _, measured)| (ratio as u8, s, measured))
        } else {
            None
        };
        let mut fit = if let Some(fit) = ordinary {
            Some(fit)
        } else if glyph.character.is_whitespace() {
            fit_whitespace_advance(&mut style, &widths, desired)?
        } else {
            None
        };
        // A PDF source space and the displacement following it are distinct.
        // Keep the scalar at the ordinary font size, and infer only the excess
        // forward movement as a standard positioning tab. A small/backtracking
        // advance still needs a representable character metric or is deferred.
        let mut space_movement = false;
        if fit.is_none() && glyph.source_character.is_some() && glyph.character == ' ' {
            let ordinary_space =
                styles.text_style(base_id, detect_lang_category(glyph.character))?;
            let measured = widths.measure(&ordinary_space, None)?;
            if measured.is_finite() && measured > 0. && desired > measured {
                fit = Some((100, 0, measured));
                space_movement = true;
            }
        }
        // A source-less geometric separator is an inference, not an authored
        // space with an out-of-range character style. Preserve actual source
        // whitespace; only the inferred separator may become a positioning tab.
        if fit.is_none() && glyph.character.is_whitespace() && glyph.source_character.is_none() {
            let stop_pt = glyph.origin + glyph.advance - owner_x;
            let stop = u32::try_from(physical_margin(stop_pt)?)
                .map_err(|_| "invalid inferred PDF tab position")?;
            allocation
                .push(
                    &mut tabs,
                    TabItem {
                        position: stop,
                        ..Default::default()
                    },
                )
                .map_err(message)?;
            // The stop is editable; an observed advance is not a fixed-width
            // tab cache. Reuse the standard no-cache marker so insertion before
            // the tab still reaches the same stop in measure, paint and saves.
            allocation
                .push(
                    &mut extensions,
                    crate::model::paragraph::TAB_EXT_PLACEHOLDER,
                )
                .map_err(message)?;
            allocation
                .push(
                    &mut positioning,
                    PositioningTab {
                        content_utf16: offset,
                        cursor_px: f64::from(origin - owner_x) * 96. / 72. + emitted_advance,
                        stop,
                    },
                )
                .map_err(message)?;
            let id = positioned_span_style(page, doc, line, glyph.span, base_id, cache)?;
            if runs.last().is_none_or(|&(_, previous)| previous != id) {
                allocation.push(&mut runs, (offset, id)).map_err(message)?;
            }
            allocation
                .push(
                    &mut positioning_tabs,
                    request_json!(allocation, {"emittedContentUtf16Start":offset,
                "sourceSpan":glyph.span,"sourceCharacter":null,"sourceUtf16":null,
                "inference":"positioning_tab_from_observed_gap","authoredTab":"unknown",
                "observedGapPt":glyph.advance,"stopPt":stop_pt,
                "tabWidthPolicy":"standard_tab_stop_without_cached_advance"})?,
                )
                .map_err(message)?;
            allocation.push_char(&mut text, '\t').map_err(message)?;
            offset += 1;
            let measured = (f64::from(stop) / 200. - f64::from(origin - owner_x)) * 96. / 72.
                - emitted_advance;
            maximum_origin_error = maximum_origin_error.max((measured - desired).abs() * 72. / 96.);
            emitted_advance += measured;
            continue;
        }
        let (ratio, spacing, measured) = if let Some(fit) = fit {
            fit
        } else if glyph.character.is_whitespace() {
            return Err("source whitespace advance exceeds combined HWP character range".into());
        } else if !(50. ..=200.).contains(&ratio) {
            return Err("source glyph width exceeds HWP ratio range".into());
        } else {
            return Err("source displacement exceeds HWP spacing range".into());
        };
        if ordinary.is_none() && glyph.character.is_whitespace() && !space_movement {
            joint_whitespace_fits += 1;
        }
        emitted_advance += measured;
        let key = (base_id, ratio, spacing, position);
        let id = if ratio == 100 && spacing == 0 && position == 0 {
            base_id
        } else if let Some(id) = cache.get(&key) {
            *id
        } else {
            if doc.doc_info.char_shapes.len() >= u32::MAX as usize {
                return Err("PDF character style budget exceeded".into());
            }
            let mut character_style = allocation
                .clone_record(&doc.doc_info.char_shapes[base_id as usize])
                .map_err(message)?;
            character_style.raw_data = None;
            character_style.ratios = [ratio; 7];
            character_style.spacings = [spacing; 7];
            character_style.char_offsets = [position; 7];
            let id = doc.doc_info.char_shapes.len() as u32;
            allocation
                .push(&mut doc.doc_info.char_shapes, character_style)
                .map_err(message)?;
            allocation
                .tree_entry::<(u32, u8, i8, i8), u32>()
                .map_err(message)?;
            cache.insert(key, id);
            id
        };
        if runs.last().is_none_or(|&(_, previous)| previous != id) {
            allocation.push(&mut runs, (offset, id)).map_err(message)?;
        }
        allocation
            .push_char(&mut text, glyph.character)
            .map_err(message)?;
        offset += glyph.character.len_utf16() as u32;
        let mut movement = 0.;
        if space_movement {
            let stop_pt = glyph.origin + glyph.advance - owner_x;
            let stop = u32::try_from(physical_margin(stop_pt)?)
                .map_err(|_| "invalid source-space PDF tab position")?;
            allocation
                .push(
                    &mut tabs,
                    TabItem {
                        position: stop,
                        ..Default::default()
                    },
                )
                .map_err(message)?;
            allocation
                .push(
                    &mut extensions,
                    crate::model::paragraph::TAB_EXT_PLACEHOLDER,
                )
                .map_err(message)?;
            allocation
                .push(
                    &mut positioning,
                    PositioningTab {
                        content_utf16: offset,
                        cursor_px: f64::from(origin - owner_x) * 96. / 72. + emitted_advance,
                        stop,
                    },
                )
                .map_err(message)?;
            allocation.push(&mut source_space_movements, request_json!(allocation, {"sourceSpan":glyph.span,
                "sourceCharacter":glyph.source_character,"sourceUtf16":glyph.source_utf16,
                "emittedSpaceUtf16Start":offset-glyph.character.len_utf16() as u32,
                "emittedTabUtf16Start":offset,"stopPt":stop_pt,
                "inference":"forward_movement_after_preserved_source_space",
                "authoredTab":"unknown","tabWidthPolicy":"standard_tab_stop_without_cached_advance"})?).map_err(message)?;
            allocation.push_char(&mut text, '\t').map_err(message)?;
            offset += 1;
            movement = (f64::from(stop) / 200. - f64::from(origin - owner_x)) * 96. / 72.
                - emitted_advance;
            emitted_advance += movement;
        }
        maximum_origin_error =
            maximum_origin_error.max((measured + movement - desired).abs() * 72. / 96.);
        maximum_error = maximum_error.max((measured + movement - source_advance).abs() * 72. / 96.);
    }
    Ok(ObservedLineText {
        text,
        runs,
        tabs,
        extensions,
        positioning,
        evidence: request_json!(allocation, {
            "characters":content.characters,
            "positioningTabs":positioning_tabs,
            "maximumAdvanceQuantizationErrorPt":maximum_error,
            "jointWhitespaceFits":joint_whitespace_fits,
            "sourceSpaceMovements":source_space_movements,
            "maximumOriginQuantizationErrorPt":maximum_origin_error,
            "maximumBaselineQuantizationErrorPt":maximum_baseline_error,
            "lineBaselinePt":line.baseline,
            "paragraphBaselinePt":paragraph_baseline,
            "baselinePolicy":"observed glyph origins encoded as ordinary character positions on the paragraph grid",
            "status":"observed_origins_with_nominal_widths",
            "hwpQuantization":"integer-percent ratio, spacing and character position; no source-font outline recovery"
        })?,
    })
}

/// Each inferred gap names a source destination, whereas a standard tab visits
/// the next stop in the paragraph-wide definition. Other rows may introduce
/// intervening stops. Traverse them with ordinary tabs, preserving every source
/// scalar and retaining the same standard navigation after editing and saving.
fn materialize_positioning_tabs(
    para: &mut Paragraph,
    tabs: &[TabItem],
    positioning: &[PositioningTab],
    allocation: Allocation<'_>,
) -> Result<Vec<Value>, String> {
    use crate::renderer::{hwpunit_to_px, layout::find_next_tab_stop, TabStop};
    if positioning.is_empty() {
        return Ok(Vec::new());
    }
    let stops: Vec<_> = allocation
        .collect(tabs.iter().map(|tab| TabStop {
            position: hwpunit_to_px(tab.position as i32, 96.) / 2.,
            tab_type: tab.tab_type,
            fill_type: tab.fill_type,
        }))
        .map_err(message)?;
    let mut repeats = BTreeMap::new();
    let mut evidence = Vec::new();
    let mut extra = 0_usize;
    let original_length = para.text.chars().count();
    for tab in positioning {
        // All destinations are explicit and inside their source owner. The
        // final common wrap trial checks its actual available width separately.
        let (next, _, _) = find_next_tab_stop(tab.cursor_px, &stops, 0., false, 0., 0.);
        let first = stops
            .binary_search_by(|stop| stop.position.total_cmp(&next))
            .map_err(|_| "inferred tab has no reachable standard stop")?;
        let target = tabs
            .binary_search_by_key(&tab.stop, |stop| stop.position)
            .map_err(|_| "missing inferred tab destination")?;
        let count = target
            .checked_sub(first)
            .map(|delta| delta + 1)
            .ok_or("inferred tab destination precedes the next standard stop")?;
        extra = extra
            .checked_add(count - 1)
            .ok_or("inferred tab navigation exceeds PDF text budget")?;
        if original_length + extra > super::text_analysis::MAX_GLYPHS {
            return Err("inferred tab navigation exceeds PDF text budget".into());
        }
        allocation.tree_entry::<u32, usize>().map_err(message)?;
        repeats.insert(tab.content_utf16, count);
        allocation
            .push(
                &mut evidence,
                request_json!(allocation, {"paragraphContentUtf16Start":tab.content_utf16,
            "stopHwpUnits":tab.stop,"emittedStandardTabs":count,
            "policy":"ordinary_next_stop_navigation; source_scalars_unchanged"})?,
            )
            .map_err(message)?;
    }
    if extra == 0 {
        return Ok(evidence);
    }
    let mut text = String::new();
    let mut extensions = Vec::new();
    let mut offsets = BTreeMap::new();
    let (mut old, mut new, mut tab_index) = (0, 0, 0);
    for ch in para.text.chars() {
        allocation.tree_entry::<u32, u32>().map_err(message)?;
        offsets.insert(old, new);
        let count = repeats.get(&old).copied().unwrap_or(1);
        if ch == '\t' {
            let extension = *para
                .tab_extended
                .get(tab_index)
                .ok_or("missing PDF tab extension")?;
            if count > 1 && !crate::model::paragraph::tab_ext_is_placeholder(&extension) {
                return Err("only inferred uncached tabs may traverse shared stops".into());
            }
            allocation
                .reserve(&mut extensions, count)
                .map_err(message)?;
            extensions.extend(std::iter::repeat_n(extension, count));
            tab_index += 1;
        }
        for _ in 0..count {
            allocation.push_char(&mut text, ch).map_err(message)?;
        }
        old += ch.len_utf16() as u32;
        new += ch.len_utf16() as u32 * count as u32;
    }
    allocation.tree_entry::<u32, u32>().map_err(message)?;
    offsets.insert(old, new);
    let convert = |position| {
        offsets
            .get(&position)
            .copied()
            .ok_or("PDF positioning metadata is not on a scalar boundary")
    };
    for run in &mut para.char_shapes {
        run.start_pos = convert(run.start_pos)?;
    }
    for line in &mut para.line_segs {
        line.text_start = convert(line.text_start)?;
    }
    para.text = text;
    para.tab_extended = extensions;
    Ok(evidence)
}

fn array(v: &Value) -> Result<&Vec<Value>, String> {
    v.as_array()
        .ok_or_else(|| "invalid recognition array".into())
}
fn number(v: &Value) -> Result<f32, String> {
    v.as_f64()
        .filter(|n| n.is_finite())
        .map(|n| n as f32)
        .ok_or_else(|| "invalid recognition coordinate".into())
}
fn index(v: &Value) -> Result<usize, String> {
    v.as_u64()
        .and_then(|n| usize::try_from(n).ok())
        .ok_or_else(|| "invalid recognition index".into())
}
fn refs(v: &Value, allocation: Allocation<'_>) -> Result<Vec<usize>, String> {
    allocation.try_collect(array(v)?.iter().map(index))
}

fn physical_margin(points: f32) -> Result<i32, String> {
    // Odd IR margin values encode CHAR units, not sub-HWPUNIT PDF precision.
    i32::try_from(hu(points)?)
        .ok()
        .and_then(|value| value.checked_mul(2))
        .ok_or_else(|| "physical paragraph measure exceeds format range".into())
}

// PDF TJ displacements can express word spaces without a U+0020 glyph.
// Positional boxes and reconstructed table paragraphs must recover the same
// text from the same glyph origins, before either path measures the line.
pub(super) fn span_text_with_spaces(
    span: &pdf_oxide::layout::TextSpan,
    allocation: Allocation<'_>,
) -> Result<String, String> {
    let reconstructed = reconstruct_line(std::slice::from_ref(span), &[0], allocation);
    allocation.0.check().map_err(|error| error.to_string())?;
    if let Ok(content) = reconstructed {
        return Ok(content.text);
    }
    // Preserve the pre-existing positional path when complete glyph origins
    // are unavailable. This fallback does not assert recovered word boundaries.
    let mut text = String::new();
    for (j, ch) in span.text.chars().enumerate() {
        if j > 0 {
            if let (Some(x), Some(prev), Some(width)) = (
                span.char_x_offsets.get(j),
                span.char_x_offsets.get(j - 1),
                span.char_widths.get(j - 1),
            ) {
                if x - prev - width > span.font_size * 0.2
                    && !text.ends_with(char::is_whitespace)
                    && !ch.is_whitespace()
                {
                    allocation.push_char(&mut text, ' ').map_err(message)?;
                }
            }
        }
        allocation.push_char(&mut text, ch).map_err(message)?;
    }
    Ok(text)
}

fn text_with_spaces(
    page: &ExtractedPage,
    indices: &[usize],
) -> Result<(String, Vec<(u32, usize)>), String> {
    let allocation = Allocation(&page.import_budget);
    let mut text = String::new();
    let mut runs = Vec::new();
    let mut end = None;
    for &i in indices {
        let span = &page.spans[i];
        if end.is_some_and(|x| span.bbox.x - x > span.font_size * 0.2)
            && !text.ends_with(char::is_whitespace)
            && !span.text.starts_with(char::is_whitespace)
        {
            allocation.push_char(&mut text, ' ').map_err(message)?;
        }
        allocation
            .push(&mut runs, (text.encode_utf16().count() as u32, i))
            .map_err(message)?;
        allocation
            .push_str(&mut text, &span_text_with_spaces(span, allocation)?)
            .map_err(message)?;
        end = Some(span.bbox.x + span.bbox.width);
    }
    Ok((text, runs))
}

type LineText = (String, Vec<(u32, usize)>, Vec<TabItem>, Vec<[u16; 7]>);

fn text_with_tabs(
    page: &ExtractedPage,
    indices: &[usize],
    cell_x: f32,
) -> Result<LineText, String> {
    let allocation = Allocation(&page.import_budget);
    let mut text = String::new();
    let mut runs = Vec::new();
    let mut tabs = Vec::new();
    let mut extensions = Vec::new();
    let mut end = None;
    for &i in indices {
        let span = &page.spans[i];
        if let Some(previous_end) = end.filter(|x| span.bbox.x - x > span.font_size * 0.2) {
            allocation.push_char(&mut text, '\t').map_err(message)?;
            allocation
                .push(
                    &mut tabs,
                    TabItem {
                        position: u32::try_from(physical_margin(span.bbox.x - cell_x)?)
                            .map_err(|_| "invalid PDF tab position")?,
                        ..Default::default()
                    },
                )
                .map_err(message)?;
            let width = u16::try_from(hu(span.bbox.x - previous_end)?)
                .map_err(|_| "PDF span gap exceeds inline tab width")?;
            allocation
                .push(&mut extensions, [width, 0, 0, 0, 0, 0, 9])
                .map_err(message)?;
        }
        allocation
            .push(&mut runs, (text.encode_utf16().count() as u32, i))
            .map_err(message)?;
        allocation
            .push_str(&mut text, &span_text_with_spaces(span, allocation)?)
            .map_err(message)?;
        end = Some(span.bbox.x + span.bbox.width);
    }
    Ok((text, runs, tabs, extensions))
}

/// Recognition uses text-content UTF-16 positions. Publish every IR position
/// on the same HWP paragraph stream axis, where an extended tab owns 8 units.
/// Otherwise save/reopen expands tabs but leaves later styles and line starts
/// pointing into the preceding line. Floating paint slots are attached later.
fn finalize_text_axis(para: &mut Paragraph, allocation: Allocation<'_>) -> Result<(), String> {
    let mut content_to_stream = BTreeMap::new();
    let mut offsets = allocation.vec(para.text.chars().count()).map_err(message)?;
    let (mut content, mut stream) = (0, 0);
    for ch in para.text.chars() {
        allocation.tree_entry::<u32, u32>().map_err(message)?;
        content_to_stream.insert(content, stream);
        offsets.push(stream);
        content += ch.len_utf16() as u32;
        stream += Paragraph::char_stream_len(ch);
    }
    allocation.tree_entry::<u32, u32>().map_err(message)?;
    content_to_stream.insert(content, stream);
    let convert = |pos| {
        content_to_stream
            .get(&pos)
            .copied()
            .ok_or_else(|| "PDF text metadata is not on a scalar boundary".to_string())
    };
    for shape in &mut para.char_shapes {
        shape.start_pos = convert(shape.start_pos)?;
    }
    for line in &mut para.line_segs {
        line.text_start = convert(line.text_start)?;
    }
    para.char_offsets = offsets;
    para.char_count = stream + 1;
    Ok(())
}

struct FlowItem {
    top: f32,
    bottom: f32,
    paragraph: Paragraph,
}

fn paragraph(
    page: &ExtractedPage,
    doc: &mut Document,
    proposal: &ParagraphProposal,
    cell_box: &Value,
    shape_ids: &[Option<u32>],
    fixed_line_height: bool,
    styles: &SourceStyles,
    typography: &std::cell::RefCell<Vec<Value>>,
) -> Result<FlowItem, String> {
    let allocation = Allocation(&page.import_budget);
    let lines = &proposal.lines;
    let first = lines.first().ok_or("paragraph has no lines")?;
    let cell_x = number(&cell_box["x"])?;
    let cell_width = number(&cell_box["width"])?;
    // PDF observations are in points (72 dpi). Publish the same HWP row
    // metrics as edit reflow, and derive the row origin from the observed
    // baseline. A separate 0.95em ascent made the first edit move ink upward.
    let row_metrics = allocation.try_collect(lines.iter().map(|line| {
        let size = f64::from(hu(line.font)?) / 100.;
        Ok::<_, String>(crate::renderer::composer::frame_metrics_for_line(
            size,
            size,
            LineSpacingType::Percent,
            100.,
            72.,
        ))
    }))?;
    let mut row_tops: Vec<_> = allocation
        .collect(
            lines
                .iter()
                .zip(&row_metrics)
                .map(|(line, metrics)| line.baseline - metrics.baseline_distance as f32 / 100.),
        )
        .map_err(message)?;
    let top = row_tops[0];
    // Observed row baselines are not an authored paragraph grid. Infer one
    // ordinary fixed advance, then encode each original glyph's displacement
    // with the existing HWP character-position contract. This preserves PDF
    // glyph origins without manufacturing hard breaks or per-backend offsets.
    let fixed_advance = if fixed_line_height {
        let (distance, weight) = row_tops.iter().enumerate().skip(1).fold(
            (0.0_f64, 0.0_f64),
            |(distance, weight), (index, row_top)| {
                let index = index as f64;
                (
                    distance + index * f64::from(*row_top - top),
                    weight + index * index,
                )
            },
        );
        let advance = if weight > 0.0 {
            (distance / weight) as f32
        } else {
            first.font
        };
        let advance = hu(advance)? as f32 / 100.;
        let mut current = top;
        for (row_top, metrics) in row_tops.iter_mut().zip(&row_metrics) {
            *row_top = current;
            current += advance.max(metrics.line_height as f32 / 100.);
        }
        Some(advance)
    } else {
        None
    };
    let layout = proposal
        .layout
        .as_ref()
        .ok_or("paragraph layout deferred")?;
    let centered = layout.alignment == TextAlignment::Center
        || (!fixed_line_height
            && lines.iter().all(|l| {
                ((l.left + l.right) / 2. - cell_x - cell_width / 2.).abs() <= l.font * 0.75
            }));
    allocation
        .bytes(std::mem::size_of::<LineSeg>())
        .map_err(message)?;
    let mut para = Paragraph::new_empty();
    para.char_count_msb = true;
    para.char_shapes.clear();
    para.line_segs.clear();
    let mut last_end = top;
    let mut utf16 = 0;
    let mut tabs = Vec::new();
    let mut positioning = Vec::new();
    let typography_start = typography.borrow().len();
    let mut glyph_styles = BTreeMap::new();
    for (line_no, line) in lines.iter().enumerate() {
        let mut materialization = None;
        let indices = &line.indices;
        if indices.is_empty() {
            return Err("empty recognized line".into());
        }
        let metrics = &row_metrics[line_no];
        let line_height = metrics.line_height as f32 / 100.;
        let line_top = row_tops[line_no];
        let next_top = row_tops
            .get(line_no + 1)
            .copied()
            .unwrap_or(line_top + line_height);
        let mut line_positioning = Vec::new();
        let (text, runs): (String, Vec<(u32, u32)>) = if fixed_line_height {
            let before_styles = doc.doc_info.char_shapes.len();
            let observed = observed_line_text(
                page,
                doc,
                line,
                shape_ids,
                styles,
                &mut glyph_styles,
                cell_x,
                line_top + metrics.baseline_distance as f32 / 100.,
            );
            page.import_budget
                .check()
                .map_err(|error| error.to_string())?;
            match observed {
                Ok(observed) => {
                    materialization = Some(observed.evidence);
                    allocation
                        .reserve(&mut tabs, observed.tabs.len())
                        .map_err(message)?;
                    tabs.extend(observed.tabs);
                    allocation
                        .reserve(&mut para.tab_extended, observed.extensions.len())
                        .map_err(message)?;
                    para.tab_extended.extend(observed.extensions);
                    line_positioning = observed.positioning;
                    (observed.text, observed.runs)
                }
                Err(reason) => {
                    doc.doc_info.char_shapes.truncate(before_styles);
                    glyph_styles.retain(|_, id| (*id as usize) < before_styles);
                    materialization = Some(
                        request_json!(allocation, {"status":"unverified_legacy_line_frame","reason":reason,
                            "baselinePolicy":"observed_source_position_preserved_independently_of_horizontal_metrics"})?,
                    );
                    let (text, runs, line_tabs, extensions) =
                        text_with_tabs(page, indices, cell_x)?;
                    allocation
                        .reserve(&mut tabs, line_tabs.len())
                        .map_err(message)?;
                    tabs.extend(line_tabs);
                    allocation
                        .reserve(&mut para.tab_extended, extensions.len())
                        .map_err(message)?;
                    para.tab_extended.extend(extensions);
                    (
                        text,
                        allocation.try_collect(runs.into_iter().map(|(offset, i)| {
                            let base = shape_ids[i].ok_or("missing source style")?;
                            positioned_span_style(page, doc, line, i, base, &mut glyph_styles)
                                .map(|id| (offset, id))
                        }))?,
                    )
                }
            }
        } else {
            let (text, runs) = text_with_spaces(page, indices)?;
            (
                text,
                allocation.try_collect(runs.into_iter().map(|(offset, i)| {
                    let base = shape_ids[i].ok_or("missing source style")?;
                    positioned_span_style(page, doc, line, i, base, &mut glyph_styles)
                        .map(|id| (offset, id))
                }))?,
            )
        };
        // A visual wrap does not terminate an editable paragraph or insert a hard break.
        // This inferred word separator is recorded separately from source whitespace.
        if matches!(
            proposal.line_joins.get(line_no),
            Some(LineJoin::InferredWordSpace)
        ) {
            allocation.push_char(&mut para.text, ' ').map_err(message)?;
            utf16 += 1;
        }
        let line_start = utf16;
        allocation
            .reserve(&mut positioning, line_positioning.len())
            .map_err(message)?;
        positioning.extend(line_positioning.into_iter().map(|mut tab| {
            tab.content_utf16 += line_start;
            tab
        }));
        if let Some(mut evidence) = materialization {
            allocation
                .set(
                    &mut evidence,
                    "context",
                    request_json!(allocation, proposal.context)?,
                )
                .map_err(message)?;
            allocation
                .set(&mut evidence, "spans", request_json!(allocation, indices)?)
                .map_err(message)?;
            allocation
                .set(
                    &mut evidence,
                    "lineIndex",
                    request_json!(allocation, line_no)?,
                )
                .map_err(message)?;
            allocation
                .set(
                    &mut evidence,
                    "paragraphUtf16Start",
                    request_json!(allocation, line_start)?,
                )
                .map_err(message)?;
            allocation
                .push(&mut typography.borrow_mut(), evidence)
                .map_err(message)?;
        }
        for (run_offset, style) in runs {
            allocation
                .push(
                    &mut para.char_shapes,
                    CharShapeRef {
                        start_pos: utf16 + run_offset,
                        char_shape_id: style,
                    },
                )
                .map_err(message)?;
        }
        allocation
            .push_str(&mut para.text, &text)
            .map_err(message)?;
        utf16 += text.encode_utf16().count() as u32;
        allocation
            .push(
                &mut para.line_segs,
                LineSeg {
                    text_start: line_start,
                    vertical_pos: hu(line_top - top)? as i32,
                    line_height: metrics.line_height,
                    text_height: metrics.text_height,
                    baseline_distance: metrics.baseline_distance,
                    line_spacing: hu((next_top - line_top - line_height).max(0.))? as i32,
                    column_start: if centered {
                        0
                    } else {
                        hu(line.left - cell_x)? as i32
                    },
                    segment_width: if centered {
                        hu(cell_width)? as i32
                    } else {
                        hu(cell_width - (line.left - cell_x))? as i32
                    },
                    tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                },
            )
            .map_err(message)?;
        last_end = line_top + line_height;
    }
    allocation
        .sort_by(&mut tabs, |a, b| a.position.cmp(&b.position))
        .map_err(message)?;
    tabs.dedup_by_key(|tab| tab.position);
    let navigation = materialize_positioning_tabs(&mut para, &tabs, &positioning, allocation)?;
    if !navigation.is_empty() {
        if let Some(first) = typography.borrow_mut().get_mut(typography_start) {
            allocation
                .set(
                    first,
                    "paragraphPositioningTabs",
                    request_json!(allocation, navigation)?,
                )
                .map_err(message)?;
        }
    }
    finalize_text_axis(&mut para, Allocation(&page.import_budget))?;
    para.has_para_text = true;
    let mut style = allocation
        .clone_record(&doc.doc_info.para_shapes[0])
        .map_err(message)?;
    style.raw_data = None;
    if fixed_line_height {
        style.attr1 &= !(1 << 8);
    }
    // Stored LineSegs preserve observed PDF lines. The editable paragraph must
    // still allow normal wrapping: SQUEEZE makes Hancom compress newly typed
    // multiline text onto one line although native reflow creates several.
    style.attr2 &= !PARA_ATTR2_LINE_WRAP_MASK;
    style.alignment = if centered {
        Alignment::Center
    } else {
        Alignment::Left
    };
    style.margin_left = if centered {
        0
    } else {
        physical_margin(layout.margin_left)?
    };
    style.margin_right = 0;
    style.indent = if centered {
        0
    } else {
        let value = physical_margin(layout.indent.abs())?;
        if layout.indent < 0. {
            -value
        } else {
            value
        }
    };
    style.spacing_before = 0;
    style.spacing_after = 0;
    if !tabs.is_empty() {
        style.tab_def_id = u16::try_from(doc.doc_info.tab_defs.len())
            .map_err(|_| "too many PDF line tab definitions")?;
        allocation
            .push(
                &mut doc.doc_info.tab_defs,
                TabDef {
                    tabs,
                    ..Default::default()
                },
            )
            .map_err(message)?;
    }
    style.line_spacing_type = if fixed_line_height {
        LineSpacingType::Fixed
    } else {
        LineSpacingType::Percent
    };
    style.line_spacing = if let Some(advance) = fixed_advance {
        physical_margin(advance)?
    } else if lines.len() > 1 {
        ((lines[1].baseline - first.baseline) / first.font * 100.).round() as i32
    } else {
        100
    };
    // [#8355] 한/글은 고정 줄간격 문단의 마지막 줄도 고정 피치를 차지한다 — 한/글 저장본의
    // 고정 줄간격 문단은 마지막 줄까지 `줄 높이 + 줄간격 = 고정값 / 2` 다. 마지막 줄 줄간격을
    // 0 으로 두고 문단 끝을 줄 바닥으로 재면, 그 뒤 간격에 같은 몫이 한 번 더 들어가 한/글에서
    // 문단마다 그만큼 내려간다(편집 우선 가져오기 저장본 1쪽 오른쪽 단 −37px).
    //
    // [#8361] 저장 줄도 한/글이 짜는 대로 `첫 줄 + 줄 번호 × 고정 피치` 에 둔다. PDF 관측 줄
    // 위치에는 피치가 짧은 줄(줄간격 707HU 등)이 섞여, 한/글이 그 줄마다 1.5px 씩 더 내려가
    // 단 끝 줄이 다음 단으로 넘어갔다(13쪽 → 14쪽).
    let mut bottom = last_end;
    if fixed_line_height {
        let pitch = style.line_spacing / 2;
        if let Some(origin) = para.line_segs.first().map(|seg| seg.vertical_pos) {
            for (row, seg) in para.line_segs.iter_mut().enumerate() {
                seg.vertical_pos = origin + row as i32 * pitch;
                seg.line_spacing = (pitch - seg.line_height).max(0);
            }
        }
        if let Some(last) = para.line_segs.last() {
            bottom = top
                + (last.vertical_pos - para.line_segs[0].vertical_pos
                    + last.line_height
                    + last.line_spacing) as f32
                    / 100.;
        }
    }
    para.para_shape_id = doc.doc_info.para_shapes.len() as u16;
    allocation
        .push(&mut doc.doc_info.para_shapes, style)
        .map_err(message)?;
    Ok(FlowItem {
        top,
        bottom,
        paragraph: para,
    })
}

/// Body and cell owners publish the same ordinary-flow witness. Recorded PDF
/// rows alone do not prove that concatenating them produces a soft paragraph.
fn verified_paragraph(
    page: &ExtractedPage,
    doc: &mut Document,
    proposal: &ParagraphProposal,
    owner: pdf_oxide::geometry::Rect,
    shape_ids: &[Option<u32>],
    styles: &SourceStyles,
    typography: &std::cell::RefCell<Vec<Value>>,
) -> Result<FlowItem, String> {
    let allocation = Allocation(&page.import_budget);
    let start = typography.borrow().len();
    let mut item = paragraph(
        page,
        doc,
        proposal,
        &allocation.value(&owner).map_err(message)?,
        shape_ids,
        true,
        styles,
        typography,
    )?;
    if let Some(reason) = typography.borrow()[start..]
        .iter()
        .find(|r| r["status"] == "unverified_legacy_line_frame")
        .and_then(|r| r["reason"].as_str())
    {
        return Err(allocation
            .display(&format_args!(
                "glyph metrics require positioned source: {reason}"
            ))
            .map_err(message)?);
    }
    let wrap = wrap_width::fit(page, doc, proposal, owner, &mut item)?;
    page.import_budget.check().map_err(|e| e.to_string())?;
    if wrap["status"] == "source_reflow_unverified" {
        return Err(allocation
            .display(&format_args!(
                "source rows require separate owners: {}",
                wrap["reason"]
                    .as_str()
                    .unwrap_or("ordinary flow unverified")
            ))
            .map_err(message)?);
    }
    if let Some(first) = typography.borrow_mut().get_mut(start) {
        first["paragraphReflow"] = wrap;
    }
    Ok(item)
}

fn is_grid_rule(shape: &ShapeObject, grid: &Value) -> Result<bool, String> {
    let (x1, y1, x2, y2) = match shape {
        ShapeObject::Line(line) => (
            (line.common.horizontal_offset as f32 + line.start.x as f32) / 100.,
            (line.common.vertical_offset as f32 + line.start.y as f32) / 100.,
            (line.common.horizontal_offset as f32 + line.end.x as f32) / 100.,
            (line.common.vertical_offset as f32 + line.end.y as f32) / 100.,
        ),
        ShapeObject::Rectangle(r)
            if r.drawing.text_box.is_none() && r.drawing.fill.solid.is_some() =>
        {
            let c = &r.common;
            let (x, y, w, h) = (
                c.horizontal_offset as f32 / 100.,
                c.vertical_offset as f32 / 100.,
                c.width as f32 / 100.,
                c.height as f32 / 100.,
            );
            if w < h {
                (x + w / 2., y, x + w / 2., y + h)
            } else {
                (x, y + h / 2., x + w, y + h / 2.)
            }
        }
        _ => return Ok(false),
    };
    // Only consume axes within the reconstructed grid, never arbitrary interior artwork.
    let b = &grid["bbox"];
    let inside = x1.min(x2) >= number(&b["x"])? - 0.5
        && x1.max(x2) <= number(&b["x"])? + number(&b["width"])? + 0.5
        && y1.min(y2) >= number(&b["y"])? - 0.5
        && y1.max(y2) <= number(&b["y"])? + number(&b["height"])? + 0.5;
    if !inside {
        return Ok(false);
    }
    let on_axis = |values: &Value, at: f32| -> Result<bool, String> {
        Ok(array(values)?
            .iter()
            .any(|v| number(v).is_ok_and(|n| (n - at).abs() <= 0.5)))
    };
    Ok(
        ((y1 - y2).abs() <= 0.5 && on_axis(&grid["yBoundaries"], y1)?)
            || ((x1 - x2).abs() <= 0.5 && on_axis(&grid["xBoundaries"], x1)?),
    )
}

struct TableBuilder<'a> {
    table_id_base: u32,
    page: &'a ExtractedPage,
    controls: &'a [Control],
    grids: &'a [Value],
    recognized: &'a [Value],
    span_controls: &'a [Option<usize>],
    shape_ids: &'a [Option<u32>],
    image_controls: &'a [usize],
    rule_controls: &'a [Option<usize>],
    styles: &'a SourceStyles,
    typography: &'a std::cell::RefCell<Vec<Value>>,
}

fn cell_border(
    doc: &mut Document,
    controls: &[Control],
    bounds: &Value,
    allocation: Allocation<'_>,
) -> Result<u16, String> {
    let (x, y, w, h) = (
        number(&bounds["x"])?,
        number(&bounds["y"])?,
        number(&bounds["width"])?,
        number(&bounds["height"])?,
    );
    let mut border = allocation
        .clone_record(&doc.doc_info.border_fills[1])
        .map_err(message)?;
    border.raw_data = None;
    for (side, (horizontal, at, from, to)) in [
        (false, x, y, y + h),
        (false, x + w, y, y + h),
        (true, y, x, x + w),
        (true, y + h, x, x + w),
    ]
    .into_iter()
    .enumerate()
    {
        let mut segments = Vec::new();
        let mut styles = BTreeSet::new();
        for control in controls {
            let Control::Shape(shape) = control else {
                continue;
            };
            let ShapeObject::Rectangle(r) = shape.as_ref() else {
                continue;
            };
            if r.drawing.text_box.is_some() {
                continue;
            }
            let Some(fill) = &r.drawing.fill.solid else {
                continue;
            };
            let c = &r.common;
            let (xx, yy, ww, hh) = (
                c.horizontal_offset as f32 / 100.,
                c.vertical_offset as f32 / 100.,
                c.width as f32 / 100.,
                c.height as f32 / 100.,
            );
            let (rule_h, rule_at, a, b) = if ww >= hh {
                (true, yy + hh / 2., xx, xx + ww)
            } else {
                (false, xx + ww / 2., yy, yy + hh)
            };
            let (a, b) = (a.max(from), b.min(to));
            if rule_h == horizontal && (rule_at - at).abs() <= 0.5 && b - a > 0.5 {
                allocation.push(&mut segments, (a, b)).map_err(message)?;
                allocation
                    .insert_set(
                        &mut styles,
                        (
                            fill.background_color,
                            border_width_index(ww.min(hh) as f64 * 25.4 / 72.),
                        ),
                    )
                    .map_err(message)?;
            }
        }
        allocation
            .sort_by(&mut segments, |a, b| a.0.total_cmp(&b.0))
            .map_err(message)?;
        let mut end = from;
        for (a, b) in segments {
            if a > end + 0.5 {
                return Err("cell border coverage gap".into());
            }
            end = end.max(b);
        }
        if end < to - 0.5 || styles.len() != 1 {
            return Err("nonuniform or missing cell side style".into());
        }
        let &(color, width) = styles.first().unwrap();
        border.borders[side].line_type = BorderLineType::Solid;
        border.borders[side].color = color;
        border.borders[side].width = width;
    }
    if let Some(i) = doc.doc_info.border_fills.iter().position(|b| b == &border) {
        return u16::try_from(i + 1).map_err(|e| e.to_string());
    }
    allocation
        .push(&mut doc.doc_info.border_fills, border)
        .map_err(message)?;
    u16::try_from(doc.doc_info.border_fills.len()).map_err(|e| e.to_string())
}

impl TableBuilder<'_> {
    fn build(
        &self,
        doc: &mut Document,
        candidate: usize,
        removed: &mut BTreeSet<usize>,
        stack: &mut BTreeSet<usize>,
        family: bool,
        source_paint: bool,
    ) -> Result<Table, String> {
        let allocation = Allocation(&self.page.import_budget);
        if stack.len() >= 8 || !allocation.insert_set(stack, candidate).map_err(message)? {
            return Err("invalid or too-deep nested table composition".into());
        }
        let result = self.build_inner(doc, candidate, removed, stack, family, source_paint);
        stack.remove(&candidate);
        result
    }

    fn build_inner(
        &self,
        doc: &mut Document,
        candidate: usize,
        removed: &mut BTreeSet<usize>,
        stack: &mut BTreeSet<usize>,
        family: bool,
        source_paint: bool,
    ) -> Result<Table, String> {
        let allocation = Allocation(&self.page.import_budget);
        let Self {
            page,
            controls,
            grids,
            recognized,
            span_controls,
            shape_ids,
            image_controls,
            rule_controls,
            ..
        } = self;
        let grid = grids.get(candidate).ok_or("missing recognized grid")?;
        let recognized_table = recognized
            .iter()
            .find(|r| r["candidate"].as_u64() == Some(candidate as u64))
            .ok_or("missing table recognition")?;
        if recognized_table["status"] != "GeometryConfirmed" {
            return Err("recognition_deferred".into());
        }
        let preserve_paint = source_paint;
        // Every PDF table consumes the same observed line metrics. Paint style
        // and whether this is a nested table cannot change source text placement.
        if page
            .recognition
            .cells
            .iter()
            .filter(|((table, _), _)| *table == candidate)
            .flat_map(|(_, region)| &region.paragraphs)
            .flat_map(|p| &p.span_indices)
            .any(|&i| super::text_clip::has_nonrectangular(page.span_clips[i].as_deref()))
        {
            return Err("source text clip requires positional editable owner".into());
        }
        let fixed_lines = true;
        let mut stroke_styles = BTreeSet::new();
        for c in *controls {
            if let Control::Shape(s) = c {
                if is_grid_rule(s, grid)? {
                    match s.as_ref() {
                        ShapeObject::Line(l) => {
                            allocation
                                .insert_set(
                                    &mut stroke_styles,
                                    (
                                        l.drawing.border_line.color,
                                        l.drawing.border_line.width as u32,
                                    ),
                                )
                                .map_err(message)?;
                        }
                        ShapeObject::Rectangle(r) => {
                            allocation
                                .insert_set(
                                    &mut stroke_styles,
                                    (
                                        r.drawing
                                            .fill
                                            .solid
                                            .as_ref()
                                            .ok_or("missing rule fill")?
                                            .background_color,
                                        r.common.width.min(r.common.height),
                                    ),
                                )
                                .map_err(message)?;
                        }
                        _ => {}
                    }
                }
            }
        }
        if !family && !preserve_paint && stroke_styles.len() != 1 {
            return Err("mixed or missing rule styles require further lowering".into());
        }
        let (color, width) = if preserve_paint {
            (0, 0)
        } else {
            *stroke_styles.first().ok_or("missing rule style")?
        };
        let mut border = allocation
            .clone_record(
                doc.doc_info
                    .border_fills
                    .get(1)
                    .ok_or("missing scaffold solid border")?,
            )
            .map_err(message)?;
        border.raw_data = None;
        for side in &mut border.borders {
            if family || preserve_paint {
                side.line_type = BorderLineType::None;
            } else {
                side.color = color;
                side.width = border_width_index(width as f64 / 100. * 25.4 / 72.);
            }
        }
        allocation
            .push(&mut doc.doc_info.border_fills, border)
            .map_err(message)?;
        let border_id =
            u16::try_from(doc.doc_info.border_fills.len()).map_err(|e| e.to_string())?;
        let b = &grid["bbox"];
        let extent = if preserve_paint {
            Some(super::table_paint::dimensions(grid, None)?)
        } else {
            None
        };
        let mut table = Table {
            row_count: u16::try_from(index(&grid["rows"])?).map_err(|e| e.to_string())?,
            col_count: u16::try_from(index(&grid["columns"])?).map_err(|e| e.to_string())?,
            border_fill_id: border_id,
            page_break: TablePageBreak::None,
            common: CommonObjAttr {
                width: extent.map_or_else(|| hu(number(&b["width"])?), |(w, _)| Ok(w))?,
                height: extent.map_or_else(|| hu(number(&b["height"])?), |(_, h)| Ok(h))?,
                horizontal_offset: hu(number(&b["x"])?)?,
                vertical_offset: hu(number(&b["y"])?)?,
                instance_id: self
                    .table_id_base
                    .checked_add(u32::try_from(candidate).map_err(|e| e.to_string())?)
                    .ok_or("table instance ID overflow")?,
                text_wrap: TextWrap::InFrontOfText,
                allow_overlap: true,
                ..Default::default()
            },
            ..Default::default()
        };
        table.row_sizes = allocation
            .filled(table.row_count as usize, table.col_count as i16)
            .map_err(message)?;
        for recognized_cell in array(&recognized_table["cells"])? {
            let cell_idx = index(&recognized_cell["cell"])?;
            let source = array(&grid["cells"])?
                .get(cell_idx)
                .ok_or("missing cell geometry")?;
            let cb = &source["bbox"];
            let cell_border_id = if family && !preserve_paint {
                cell_border(doc, controls, cb, allocation)?
            } else {
                border_id
            };
            let extent = if preserve_paint {
                Some(super::table_paint::dimensions(grid, Some(source))?)
            } else {
                None
            };
            allocation
                .bytes(std::mem::size_of::<Paragraph>() + std::mem::size_of::<LineSeg>())
                .map_err(message)?;
            let mut cell = Cell::new_empty(
                index(&source["column"])? as u16,
                index(&source["row"])? as u16,
                extent.map_or_else(|| hu(number(&cb["width"])?), |(w, _)| Ok(w))?,
                extent.map_or_else(|| hu(number(&cb["height"])?), |(_, h)| Ok(h))?,
                cell_border_id,
            );
            cell.col_span = index(&source["colspan"])? as u16;
            cell.row_span = index(&source["rowspan"])? as u16;
            cell.vertical_align = VerticalAlign::Top;
            cell.apply_inner_margin = true;
            cell.paragraphs[0].char_shapes = allocation
                .collect([CharShapeRef {
                    start_pos: 0,
                    char_shape_id: 0,
                }])
                .map_err(message)?;
            let image_ids = allocation.try_collect(
                refs(&source["imageIndices"], allocation)?
                    .into_iter()
                    .map(|i| {
                        image_controls
                            .get(i)
                            .copied()
                            .ok_or_else(|| "missing cell image owner".to_string())
                    }),
            )?;
            let content = allocation
                .set_from(
                    refs(&source["spanIndices"], allocation)?
                        .into_iter()
                        .filter_map(|i| span_controls[i])
                        .chain(image_ids.iter().copied()),
                )
                .map_err(message)?;
            let mixed = image_ids.iter().any(|image| {
                let a = paint_frame(&controls[*image]).unwrap();
                content.iter().any(|other| {
                    let b = paint_frame(&controls[*other]).unwrap();
                    other != image
                        && a.vertical_offset < b.vertical_offset + b.height
                        && b.vertical_offset < a.vertical_offset + a.height
                })
            });
            if mixed && array(&recognized_cell["childTableIndices"])?.is_empty() {
                // A PDF can interleave text paint and image glyphs within one
                // row. The cell is known; an inline paragraph is not. Keep the
                // original editable owners and paint order locally, rather
                // than rejecting the whole table or rasterizing its text.
                let mut original = allocation.try_collect(
                    content
                        .iter()
                        .map(|&i| allocation.clone_record(&controls[i]).map_err(message)),
                )?;
                allocation
                    .sort_by(&mut original, |a, b| {
                        paint_frame(a)
                            .unwrap()
                            .z_order
                            .cmp(&paint_frame(b).unwrap().z_order)
                    })
                    .map_err(message)?;
                let (x, y) = (hu(number(&cb["x"])?)? as i32, hu(number(&cb["y"])?)? as i32);
                for control in &mut original {
                    let a = match control {
                        Control::Shape(s) => s.common_mut(),
                        Control::Picture(p) => &mut p.common,
                        _ => unreachable!(),
                    };
                    a.horizontal_offset = (a.horizontal_offset as i32 - x) as u32;
                    a.vertical_offset = (a.vertical_offset as i32 - y) as u32;
                    a.horz_rel_to = HorzRelTo::Column;
                    a.vert_rel_to = VertRelTo::Para;
                    a.flow_with_text = true;
                    a.treat_as_char = false;
                    a.text_wrap = TextWrap::InFrontOfText;
                }
                let p = &mut cell.paragraphs[0];
                p.char_count_msb = true;
                p.control_mask |= 0x800;
                p.controls = original;
                p.ctrl_data_records = allocation
                    .collect((0..p.controls.len()).map(|_| None))
                    .map_err(message)?;
                allocation
                    .bytes(std::mem::size_of::<i64>())
                    .map_err(message)?;
                p.rebuild_char_offsets(
                    &allocation
                        .filled(p.controls.len(), 0usize)
                        .map_err(message)?,
                );
                allocation
                    .push(
                        &mut self.typography.borrow_mut(),
                        request_json!(allocation, {"status":"retained_interleaved_cell_content",
                    "candidate":candidate,"cell":cell_idx,"sourceControlCount":content.len(),
                    "paragraphGrouping":"unverified; original editable text owners preserved"})?,
                    )
                    .map_err(message)?;
                allocation.extend_set(removed, content).map_err(message)?;
                allocation.push(&mut table.cells, cell).map_err(message)?;
                continue;
            }
            let mut flow = Vec::new();
            let recognized_text = self
                .page
                .recognition
                .cells
                .get(&(candidate, cell_idx))
                .ok_or("missing typed cell text recognition")?;
            for proposal in &recognized_text.paragraphs {
                if proposal.status == ParagraphState::Confirmed {
                    if !fixed_lines {
                        allocation
                            .push(
                                &mut flow,
                                paragraph(
                                    page,
                                    doc,
                                    proposal,
                                    cb,
                                    shape_ids,
                                    false,
                                    self.styles,
                                    self.typography,
                                )?,
                            )
                            .map_err(message)?;
                        continue;
                    }
                    let before = checkpoint(doc, self.typography);
                    let owner = pdf_oxide::geometry::Rect::new(
                        number(&cb["x"])?,
                        number(&cb["y"])?,
                        number(&cb["width"])?,
                        number(&cb["height"])?,
                    );
                    match verified_paragraph(
                        page,
                        doc,
                        proposal,
                        owner,
                        shape_ids,
                        self.styles,
                        self.typography,
                    ) {
                        Ok(item) => {
                            allocation.push(&mut flow, item).map_err(message)?;
                            continue;
                        }
                        Err(reason) => {
                            rollback(doc, self.typography, before);
                            allocation
                                .push(
                                    &mut self.typography.borrow_mut(),
                                    request_json!(allocation, {
                                        "status":"retained_separate_cell_rows",
                                        "candidate":candidate,"cell":cell_idx,
                                        "spans":proposal.span_indices,"reason":reason,
                                        "authoredParagraphBoundaries":"unknown",
                                    })?,
                                )
                                .map_err(message)?;
                        }
                    }
                }
                if fixed_lines {
                    // Preserve unresolved appearance without claiming recovered paragraph grouping.
                    for line in &proposal.lines {
                        let mut unresolved_line =
                            allocation.clone_record(proposal).map_err(message)?;
                        unresolved_line.lines = allocation
                            .collect([allocation.clone_record(line).map_err(message)?])
                            .map_err(message)?;
                        unresolved_line.line_joins =
                            allocation.collect([LineJoin::Start]).map_err(message)?;
                        unresolved_line.layout = Some(ParagraphLayout {
                            alignment: TextAlignment::Left,
                            margin_left: line.left - number(&cb["x"])?,
                            indent: 0.,
                        });
                        unresolved_line.span_indices =
                            allocation.copy_slice(&line.indices).map_err(message)?;
                        allocation
                            .push(
                                &mut flow,
                                paragraph(
                                    page,
                                    doc,
                                    &unresolved_line,
                                    cb,
                                    shape_ids,
                                    true,
                                    self.styles,
                                    self.typography,
                                )?,
                            )
                            .map_err(message)?;
                    }
                } else {
                    return Err("recognition_deferred".into());
                }
            }
            let children = recognized_cell["childTableIndices"]
                .as_array()
                .map(Vec::as_slice)
                .unwrap_or_default();
            for child_ref in children {
                let child_id = index(&child_ref)?;
                let child_grid = grids.get(child_id).ok_or("missing child grid")?;
                if child_grid["hierarchy"]["parent"]
                    != request_json!(allocation, {"table":candidate,"cell":cell_idx})?
                {
                    return Err("nested cell-slot ownership mismatch".into());
                }
                let mut child = self.build(doc, child_id, removed, stack, true, source_paint)?;
                let child_box = &child_grid["bbox"];
                let top = number(&child_box["y"])?;
                let bottom = top + number(&child_box["height"])?;
                let left = hu(number(&child_box["x"])? - number(&cb["x"])?)?;
                child.common.treat_as_char = true;
                child.common.horz_rel_to = HorzRelTo::Para;
                child.common.vert_rel_to = VertRelTo::Para;
                child.common.horizontal_offset = 0;
                child.common.vertical_offset = 0;
                child.common.text_wrap = TextWrap::TopAndBottom;
                child.common.allow_overlap = false;
                child.common.flow_with_text = true;
                child.outer_margin_left =
                    i16::try_from(left).map_err(|_| "nested inset exceeds format range")?;
                let height = child.common.height;
                allocation
                    .bytes(std::mem::size_of::<LineSeg>())
                    .map_err(message)?;
                let mut host = Paragraph::new_empty();
                host.char_count_msb = true;
                host.char_count = 9;
                host.control_mask = 0x800;
                host.char_shapes = allocation
                    .collect([CharShapeRef {
                        start_pos: 0,
                        char_shape_id: 0,
                    }])
                    .map_err(message)?;
                let mut ps = allocation
                    .clone_record(&doc.doc_info.para_shapes[0])
                    .map_err(message)?;
                ps.raw_data = None;
                ps.attr1 &= !(1 << 8);
                ps.margin_left = 0;
                ps.margin_right = 0;
                ps.indent = 0;
                ps.spacing_before = 0;
                ps.spacing_after = 0;
                ps.line_spacing_type = LineSpacingType::SpaceOnly;
                ps.line_spacing = 0;
                host.para_shape_id =
                    u16::try_from(doc.doc_info.para_shapes.len()).map_err(|e| e.to_string())?;
                allocation
                    .push(&mut doc.doc_info.para_shapes, ps)
                    .map_err(message)?;
                host.line_segs = allocation
                    .collect([LineSeg {
                        line_height: height as i32,
                        text_height: height as i32,
                        baseline_distance: (height as f32 * 0.85).round() as i32,
                        segment_width: cell.width as i32,
                        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                        ..Default::default()
                    }])
                    .map_err(message)?;
                allocation
                    .push(
                        &mut host.controls,
                        Control::Table(allocation.boxed(child).map_err(message)?),
                    )
                    .map_err(message)?;
                allocation
                    .push(
                        &mut flow,
                        FlowItem {
                            top,
                            bottom,
                            paragraph: host,
                        },
                    )
                    .map_err(message)?;
            }
            for image_idx in refs(&source["imageIndices"], allocation)? {
                let control_idx = *image_controls
                    .get(image_idx)
                    .ok_or("image/control map mismatch")?;
                let Control::Picture(original) = &controls[control_idx] else {
                    unreachable!()
                };
                let mut picture = allocation.clone_record(original).map_err(message)?;
                let top = picture.common.vertical_offset as f32 / 100.;
                picture.common.treat_as_char = true;
                picture.common.text_wrap = TextWrap::TopAndBottom;
                picture.common.allow_overlap = false;
                if fixed_lines {
                    picture.common.flow_with_text = true;
                }
                picture.common.vert_rel_to = VertRelTo::Para;
                picture.common.horz_rel_to = HorzRelTo::Para;
                picture.common.horizontal_offset = 0;
                picture.common.vertical_offset = 0;
                allocation
                    .bytes(std::mem::size_of::<LineSeg>())
                    .map_err(message)?;
                let mut para = Paragraph::new_empty();
                para.char_count_msb = true;
                para.char_count = 9;
                para.control_mask = 0x800;
                para.char_shapes = allocation
                    .collect([CharShapeRef {
                        start_pos: 0,
                        char_shape_id: 0,
                    }])
                    .map_err(message)?;
                let mut ps = allocation
                    .clone_record(&doc.doc_info.para_shapes[0])
                    .map_err(message)?;
                ps.raw_data = None;
                ps.margin_left =
                    2 * (original.common.horizontal_offset as i32 - hu(number(&cb["x"])?)? as i32);
                ps.line_spacing = 100;
                if fixed_lines {
                    ps.line_spacing_type = LineSpacingType::Fixed;
                    ps.line_spacing = i32::try_from(picture.common.height)
                        .ok()
                        .and_then(|height| height.checked_mul(2))
                        .ok_or("picture line height exceeds format range")?;
                }
                ps.spacing_before = 0;
                ps.spacing_after = 0;
                para.para_shape_id = doc.doc_info.para_shapes.len() as u16;
                allocation
                    .push(&mut doc.doc_info.para_shapes, ps)
                    .map_err(message)?;
                para.line_segs = allocation
                    .collect([LineSeg {
                        line_height: picture.common.height as i32,
                        text_height: picture.common.height as i32,
                        baseline_distance: picture.common.height as i32,
                        segment_width: cell.width as i32,
                        tag: LineSeg::TAG_SINGLE_SEGMENT_LINE,
                        ..Default::default()
                    }])
                    .map_err(message)?;
                let bottom = top + picture.common.height as f32 / 100.;
                allocation
                    .push(&mut para.controls, Control::Picture(picture))
                    .map_err(message)?;
                allocation
                    .push(
                        &mut flow,
                        FlowItem {
                            top,
                            bottom,
                            paragraph: para,
                        },
                    )
                    .map_err(message)?;
                allocation
                    .insert_set(removed, control_idx)
                    .map_err(message)?;
            }
            allocation
                .sort_by(&mut flow, |a, b| a.top.total_cmp(&b.top))
                .map_err(message)?;
            let mut cursor = number(&cb["y"])?;
            let mut previous_style: Option<u16> = None;
            for (flow_index, item) in flow.iter_mut().enumerate() {
                if item.top < cursor - 0.5 {
                    return Err("overlapping cell flow requires additional lowering".into());
                }
                let gap = (item.top - cursor).max(0.);
                let picture = item.paragraph.controls.first().is_some_and(|c| {
                    matches!(
                        c,
                        Control::Picture(_) | Control::Table(_) | Control::Shape(_)
                    )
                });
                if picture {
                    // Place the inter-item gap before the inline image's flow origin.
                    if let Some(style) = previous_style {
                        doc.doc_info.para_shapes[style as usize].spacing_after =
                            physical_margin(gap)?;
                    } else {
                        cell.padding.top = i16::try_from(hu(gap)?)
                            .map_err(|_| "cell padding exceeds format range")?;
                    }
                } else {
                    doc.doc_info.para_shapes[item.paragraph.para_shape_id as usize]
                        .spacing_before = physical_margin(gap)?;
                }
                if flow_index == 0 && item.paragraph.controls.is_empty() {
                    // Stored cell starts own their lead in the frame, not flow_before.
                    let lead = hu((item.top - cursor).max(0.))? as i32;
                    for segment in &mut item.paragraph.line_segs {
                        segment.vertical_pos += lead;
                    }
                }
                cursor = item.bottom;
                previous_style = Some(item.paragraph.para_shape_id);
            }
            if !flow.is_empty() {
                cell.paragraphs = allocation
                    .collect(flow.into_iter().map(|i| i.paragraph))
                    .map_err(message)?;
            }
            for i in refs(&source["spanIndices"], allocation)? {
                if let Some(c) = span_controls[i] {
                    allocation.insert_set(removed, c).map_err(message)?;
                }
            }
            allocation.push(&mut table.cells, cell).map_err(message)?;
        }
        let grid_work = usize::from(table.row_count)
            .checked_mul(usize::from(table.col_count))
            .and_then(|n| n.checked_mul(std::mem::size_of::<Option<usize>>()))
            .ok_or("PDF table grid work overflow")?;
        allocation.bytes(grid_work).map_err(message)?;
        table.rebuild_grid();
        table.row_sizes = allocation
            .collect(
                (0..table.row_count)
                    .map(|row| table.cells.iter().filter(|c| c.row == row).count() as i16),
            )
            .map_err(message)?;
        if preserve_paint {
            // Page-wide partition attaches paint only after all accepted cell owners are known.
        } else if family {
            for path in refs(&grid["sourcePathIndices"], allocation)? {
                let control = rule_controls
                    .get(path)
                    .ok_or("invalid source rule reference")?
                    .ok_or("source boundary has no lowering representation")?;
                allocation.insert_set(removed, control).map_err(message)?;
            }
        } else {
            for (i, c) in controls.iter().enumerate() {
                if let Control::Shape(s) = c {
                    if is_grid_rule(s, grid)? {
                        allocation.insert_set(removed, i).map_err(message)?;
                    }
                }
            }
        }
        Ok(table)
    }
}

fn table_count(controls: &[Control]) -> usize {
    controls
        .iter()
        .map(|c| match c {
            Control::Table(t) => {
                1 + t
                    .cells
                    .iter()
                    .flat_map(|c| &c.paragraphs)
                    .map(|p| table_count(&p.controls))
                    .sum::<usize>()
            }
            _ => 0,
        })
        .sum()
}

fn paint_frame(control: &Control) -> Option<&CommonObjAttr> {
    match control {
        Control::Shape(shape) => Some(shape.common()),
        Control::Picture(picture) => Some(&picture.common),
        Control::Table(table) => Some(&table.common),
        _ => None,
    }
}

/// Body table paint can use the flow plane, whereas positional source controls
/// paint in front of text. A retained backdrop can consequently cover table
/// labels even with a larger table z-order. Promote only when no earlier
/// overlapping positional paint would cross that plane boundary.
fn table_paint_layer(
    controls: &[Control],
    consumed: &BTreeSet<usize>,
    bounds: &Value,
) -> Result<i32, String> {
    let last = consumed
        .iter()
        .filter_map(|&i| controls.get(i).and_then(paint_frame))
        .map(|c| c.z_order)
        .max()
        .ok_or("table has no source paint")?;
    let (x, y, width, height) = (
        number(&bounds["x"])?,
        number(&bounds["y"])?,
        number(&bounds["width"])?,
        number(&bounds["height"])?,
    );
    for (i, control) in controls.iter().enumerate() {
        if consumed.contains(&i) {
            continue;
        }
        let Some(frame) = paint_frame(control) else {
            continue;
        };
        if frame.z_order > last {
            continue;
        }
        let cx = (frame.horizontal_offset as i32) as f32 / 100.;
        let cy = (frame.vertical_offset as i32) as f32 / 100.;
        if cx < x + width
            && cx + frame.width as f32 / 100. > x
            && cy < y + height
            && cy + frame.height as f32 / 100. > y
        {
            return Err("positional backdrop cannot share inferred table flow plane".into());
        }
    }
    Ok(last)
}

/// Reserve the temporary numeric arrays used by the common paragraph slot
/// producer. The existing helper still owns all position/paint decisions.
fn prepend_cell_paint(
    paragraph: &mut Paragraph,
    paint: Vec<Control>,
    allocation: Allocation<'_>,
) -> Result<(), String> {
    if paint.is_empty() {
        return Ok(());
    }
    let chars = paragraph.text.chars().count();
    let controls = paragraph.controls.len();
    let total = controls
        .checked_add(paint.len())
        .ok_or("PDF cell control count overflow")?;
    let maximum = chars.max(4);
    // logical_control_positions: two control indices and a char projection.
    // rebuild_char_offsets: chars, old/new offsets, gap counts and shifts.
    // Four times the final length covers geometrically grown primitive buffers;
    // these are cumulative work reservations, not allocator/RSS measurements.
    allocation
        .payload::<char>(maximum.checked_mul(8).ok_or("PDF slot work overflow")?)
        .map_err(message)?;
    allocation
        .payload::<u32>(
            maximum
                .checked_mul(8)
                .and_then(|n| n.checked_add(paragraph.line_segs.len()))
                .ok_or("PDF slot work overflow")?,
        )
        .map_err(message)?;
    allocation
        .payload::<i64>(
            chars
                .checked_mul(2)
                .and_then(|n| n.checked_add(1))
                .ok_or("PDF slot work overflow")?,
        )
        .map_err(message)?;
    allocation
        .payload::<usize>(
            total
                .checked_mul(4)
                .and_then(|n| controls.checked_mul(2).and_then(|c| n.checked_add(c)))
                .ok_or("PDF slot work overflow")?,
        )
        .map_err(message)?;
    let auxiliary = paragraph
        .field_ranges
        .len()
        .checked_add(paragraph.orphan_field_ends.len())
        .and_then(|n| n.checked_add(paragraph.title_marks.len()))
        .ok_or("PDF slot work overflow")?;
    allocation
        .bytes(auxiliary.checked_mul(128).ok_or("PDF slot work overflow")?)
        .map_err(message)?;
    allocation
        .reserve(&mut paragraph.controls, paint.len())
        .map_err(message)?;
    let missing = total.saturating_sub(paragraph.ctrl_data_records.len());
    allocation
        .reserve(&mut paragraph.ctrl_data_records, missing)
        .map_err(message)?;
    crate::model::border_paint::prepend_cell_paint(paragraph, paint);
    Ok(())
}

pub(super) fn lower(
    page: &ExtractedPage,
    doc: &mut Document,
    next_id: &mut u32,
) -> Result<Value, String> {
    let allocation = Allocation(&page.import_budget);
    let mut report = super::lowering::lower(page, doc, next_id)?;
    let section = doc.sections.len() - 1;
    let mut controls = std::mem::take(&mut doc.sections[section].paragraphs[0].controls);
    let mut span_controls = allocation.filled(page.spans.len(), None).map_err(message)?;
    let mut shape_ids = allocation.filled(page.spans.len(), None).map_err(message)?;
    let mut image_controls = allocation.filled(page.images.len(), 0).map_err(message)?;
    for source in array(&report["imageSourceControls"])? {
        let image = index(&source["image"])?;
        let control = index(&source["control"])?;
        *image_controls
            .get_mut(image)
            .ok_or("invalid image/control mapping")? = control;
    }
    let mut rule_controls = allocation.filled(page.paths.len(), None).map_err(message)?;
    for source in array(&report["ruleSourceControls"])? {
        let path = index(&source["path"])?;
        let control = index(&source["control"])?;
        *rule_controls
            .get_mut(path)
            .ok_or("invalid rule/control mapping")? = Some(control);
    }
    let nonempty: Vec<_> = allocation
        .collect(
            page.spans
                .iter()
                .enumerate()
                .filter_map(|(i, s)| (!s.text.is_empty()).then_some(i)),
        )
        .map_err(message)?;
    let mut span_cursor = 0;
    for (control_idx, c) in controls.iter().enumerate() {
        match c {
            Control::Picture(_) => {}
            Control::Shape(s) => {
                if let Some(b) = s.drawing().and_then(|d| d.text_box.as_ref()) {
                    let source = *nonempty
                        .get(span_cursor)
                        .ok_or("span/control map mismatch")?;
                    span_controls[source] = Some(control_idx);
                    shape_ids[source] = Some(b.paragraphs[0].char_shapes[0].char_shape_id);
                    span_cursor += 1;
                }
            }
            _ => {}
        }
    }
    let grids = array(&page.snapshot["tableDetection"]["tables"])?;
    let recognized = array(&page.recognition.report["tables"])?;
    let fragment_ids = std::cell::Cell::new(
        next_id
            .checked_add(u32::try_from(grids.len()).map_err(|e| e.to_string())?)
            .ok_or("table instance ID overflow")?,
    );
    let styles = SourceStyles::new(doc, &shape_ids, &page.import_budget)?;
    let typography = std::cell::RefCell::new(Vec::new());
    let builder = TableBuilder {
        table_id_base: *next_id,
        page,
        controls: &controls,
        grids,
        recognized,
        span_controls: &span_controls,
        shape_ids: &shape_ids,
        image_controls: &image_controls,
        rule_controls: &rule_controls,
        styles: &styles,
        typography: &typography,
    };
    *next_id = next_id
        .checked_add(u32::try_from(grids.len()).map_err(|e| e.to_string())?)
        .ok_or("table instance ID overflow")?;
    let mut removed = BTreeSet::new();
    let mut tables = Vec::new();
    let mut table_candidates = Vec::new();
    let mut table_removed = Vec::new();
    let mut held = Vec::new();
    let mut preserve_indices = Vec::new();
    for (candidate, grid) in grids.iter().enumerate() {
        if !grid["hierarchy"]["parent"].is_null() {
            continue;
        }
        let family = array(&grid["cells"])?.iter().any(|c| {
            c["childTableIndices"]
                .as_array()
                .is_some_and(|a| !a.is_empty())
        });
        let style_start = doc.doc_info.para_shapes.len();
        let char_style_start = doc.doc_info.char_shapes.len();
        let typography_start = typography.borrow().len();
        let border_start = doc.doc_info.border_fills.len();
        let tab_start = doc.doc_info.tab_defs.len();
        let mut candidate_removed = BTreeSet::new();
        // All source tables share the same physical cell extents and paint
        // ownership. A solid/linear root is not proof that a separately rebuilt
        // uniform border, rounded width or page-owned backdrop is equivalent.
        let source_paint = true;
        let built = builder
            .build(
                doc,
                candidate,
                &mut candidate_removed,
                &mut BTreeSet::new(),
                family,
                source_paint,
            )
            .and_then(|mut table| {
                if !source_paint {
                    table.common.z_order =
                        table_paint_layer(&controls, &candidate_removed, &grid["bbox"])?;
                }
                Ok(table)
            });
        page.import_budget.check().map_err(|e| e.to_string())?;
        match built {
            Ok(table) => {
                if source_paint {
                    allocation
                        .push(&mut preserve_indices, tables.len())
                        .map_err(message)?;
                }
                allocation
                    .push(&mut table_removed, candidate_removed)
                    .map_err(message)?;
                allocation
                    .push(&mut table_candidates, candidate)
                    .map_err(message)?;
                allocation
                    .push(
                        &mut tables,
                        Control::Table(allocation.boxed(table).map_err(message)?),
                    )
                    .map_err(message)?;
            }
            Err(reason) => {
                doc.doc_info.para_shapes.truncate(style_start);
                doc.doc_info.char_shapes.truncate(char_style_start);
                typography.borrow_mut().truncate(typography_start);
                doc.doc_info.border_fills.truncate(border_start);
                doc.doc_info.tab_defs.truncate(tab_start);
                allocation
                    .push(
                        &mut held,
                        request_json!(allocation, {"candidate":candidate,"reason":reason})?,
                    )
                    .map_err(message)?;
            }
        }
    }
    let mut paint_owners = Vec::new();
    let mut native_borders = Vec::new();
    let mut paint_result = None;
    let mut paint_work = 0;
    while !preserve_indices.is_empty() {
        paint_owners = composition::owners(
            &tables,
            &preserve_indices,
            grids,
            builder.table_id_base,
            allocation,
        )?;
        native_borders = allocation
            .collect(paint_owners.iter().map(|owner| {
                native_borders::plan(page, &controls, &rule_controls, &grids[owner.candidate])
            }))
            .map_err(message)?;
        page.import_budget.check().map_err(|e| e.to_string())?;
        let targets = allocation.collect(paint_owners
            .iter()
            .zip(&native_borders)
            .map(|(owner, native)| super::table_paint::Target {
                grid: &grids[owner.candidate],
                native_border_controls: native.as_ref().map(|n| n.controls.as_slice()),
                root: matches!(&tables[owner.root], Control::Table(t) if t.common.instance_id == owner.instance_id),
            })
            ).map_err(message)?;
        let plan = super::table_paint::partition(
            &controls,
            &rule_controls,
            super::table_paint::SourceContent {
                spans: &span_controls,
                images: &image_controls,
                resources: &doc.bin_data_content,
            },
            &targets,
            &page.snapshot["tableDetection"],
            &page.paths,
            &mut paint_work,
            allocation,
        )
        .and_then(|plan| {
            let consumed = allocation
                .set_from(
                    table_removed
                        .iter()
                        .flatten()
                        .copied()
                        .chain(plan.removed.iter().copied()),
                )
                .map_err(message)?;
            for &ti in &preserve_indices {
                table_paint_layer(&controls, &consumed, &grids[table_candidates[ti]]["bbox"])?;
            }
            Ok(plan)
        });
        page.import_budget.check().map_err(|e| e.to_string())?;
        if let Err(error) = &plan {
            if !error.unconsumed_targets.is_empty() {
                let rejected = allocation
                    .set_from(
                        error
                            .unconsumed_targets
                            .iter()
                            .map(|&i| paint_owners[i].root),
                    )
                    .map_err(message)?;
                composition::discard_roots(
                    &mut tables,
                    &mut table_candidates,
                    &mut table_removed,
                    &mut preserve_indices,
                    &mut held,
                    &rejected,
                    &error.reason,
                    allocation,
                )?;
                continue;
            }
        }
        paint_result = Some(plan);
        break;
    }
    let mut extra_paint = Vec::new();
    let mut paint_fragments = 0;
    let mut native_border_cells = 0;
    let mut native_border_fragments = 0;
    let mut external_boundary_paths = Vec::new();
    let mut cell_fill_sources = Vec::new();
    if let Some(plan) = paint_result {
        match plan {
            Ok(mut plan) => {
                let reserve_paint = |bytes| allocation.bytes(bytes).ok();
                let mut paint_resources = crate::model::border_paint::Resources::with_reservation(
                    &doc.bin_data_content,
                    &reserve_paint,
                );
                external_boundary_paths = allocation
                    .collect(
                        rule_controls
                            .iter()
                            .enumerate()
                            .filter_map(|(path, control)| {
                                control
                                    .is_some_and(|c| plan.external_boundary_controls.contains(&c))
                                    .then_some(path)
                            }),
                    )
                    .map_err(message)?;
                native_border_fragments = plan.native_border_fragments;
                cell_fill_sources = allocation.try_collect(plan.cell_fill_sources.iter().map(|&(path,target,cell)| {
                    request_json!(allocation,{"path":path,"candidate":paint_owners[target].candidate,"cell":cell}).map_err(String::from)
                }))?;
                allocation
                    .extend_set(&mut removed, plan.removed)
                    .map_err(message)?;
                let consumed = allocation
                    .set_from(
                        table_removed
                            .iter()
                            .flatten()
                            .copied()
                            .chain(removed.iter().copied()),
                    )
                    .map_err(message)?;
                for &ti in &preserve_indices {
                    let Control::Table(table) = &mut tables[ti] else {
                        unreachable!()
                    };
                    table.common.z_order = table_paint_layer(
                        &controls,
                        &consumed,
                        &grids[table_candidates[ti]]["bbox"],
                    )?;
                }
                for (target, owner) in paint_owners.iter().enumerate() {
                    let table = composition::table_mut(&mut tables[owner.root], owner.instance_id)
                        .ok_or("source paint owner missing from actual table hierarchy")?;
                    let grid = &grids[owner.candidate];
                    for (ci, cell) in table.cells.iter_mut().enumerate() {
                        if let Some(native) = &native_borders[target] {
                            let border = &native.borders[ci];
                            let id = if let Some(i) =
                                doc.doc_info.border_fills.iter().position(|b| b == border)
                            {
                                i + 1
                            } else {
                                allocation
                                    .push(
                                        &mut doc.doc_info.border_fills,
                                        allocation.clone_record(border).map_err(message)?,
                                    )
                                    .map_err(message)?;
                                doc.doc_info.border_fills.len()
                            };
                            cell.border_fill_id = u16::try_from(id).map_err(|e| e.to_string())?;
                            native_border_cells += usize::from(
                                border
                                    .borders
                                    .iter()
                                    .any(|b| b.line_type != BorderLineType::None),
                            );
                        }
                        if let Some(fill) = plan.cell_fills[target][ci].take() {
                            let mut border = allocation
                                .clone_record(
                                    doc.doc_info
                                        .border_fills
                                        .get(
                                            usize::from(cell.border_fill_id)
                                                .checked_sub(1)
                                                .ok_or("missing cell style")?,
                                        )
                                        .ok_or("missing cell style")?,
                                )
                                .map_err(message)?;
                            border.fill = fill;
                            border.raw_data = None;
                            let id = if let Some(i) =
                                doc.doc_info.border_fills.iter().position(|b| b == &border)
                            {
                                i + 1
                            } else {
                                allocation
                                    .push(&mut doc.doc_info.border_fills, border)
                                    .map_err(message)?;
                                doc.doc_info.border_fills.len()
                            };
                            cell.border_fill_id = u16::try_from(id).map_err(|e| e.to_string())?;
                        }
                        let mut cell_paint = std::mem::take(&mut plan.cells[target][ci]);
                        for paint in &mut cell_paint {
                            let id = fragment_ids.get();
                            fragment_ids
                                .set(id.checked_add(1).ok_or("cell paint instance ID overflow")?);
                            super::table_paint::anchor(
                                paint,
                                cell,
                                &grid["cells"][ci]["bbox"],
                                id,
                            )?;
                            if matches!(&paint, Control::Picture(p) if p.common.cell_border_paint.is_some())
                            {
                                crate::model::border_paint::seal_picture(
                                    paint,
                                    &mut paint_resources,
                                )
                                .ok_or("source border picture seal failed")?;
                            }
                            paint_fragments += 1;
                        }
                        prepend_cell_paint(&mut cell.paragraphs[0], cell_paint, allocation)?;
                    }
                }
                for (ri, mut paint) in plan.retained.into_iter().enumerate() {
                    let id = fragment_ids.get();
                    fragment_ids.set(id.checked_add(1).ok_or("page paint instance ID overflow")?);
                    super::table_paint::assign_id(&mut paint, id);
                    if let Some(&(target, cell, tag)) = plan.outside_owners.get(&ri) {
                        let owner = &paint_owners[target];
                        let Control::Table(table) = &tables[owner.root] else {
                            unreachable!()
                        };
                        crate::model::border_paint::bind_outside(
                            &mut paint,
                            table,
                            cell,
                            tag,
                            &mut paint_resources,
                        )
                        .ok_or("outside border binding no longer matches source frame")?;
                    }
                    allocation.push(&mut extra_paint, paint).map_err(message)?;
                }
            }
            Err(reason) => {
                let mut kept = Vec::new();
                let mut kept_removed = Vec::new();
                for (ti, (table, consumed)) in tables.into_iter().zip(table_removed).enumerate() {
                    if preserve_indices.contains(&ti) {
                        allocation.push(&mut held,
                            request_json!(allocation, {"candidate":table_candidates[ti],"reason":allocation.display(&reason)?})?,
                        ).map_err(message)?;
                    } else {
                        allocation.push(&mut kept, table).map_err(message)?;
                        allocation
                            .push(&mut kept_removed, consumed)
                            .map_err(message)?;
                    }
                }
                tables = kept;
                table_removed = kept_removed;
            }
        }
    }
    allocation
        .extend_set(&mut removed, table_removed.into_iter().flatten())
        .map_err(message)?;
    let root_count = tables.len();
    *next_id = fragment_ids.get();
    let table_count = table_count(&tables);
    let (body_frames, body_mapping, beneath) = body::lower(
        page,
        doc,
        &controls,
        &span_controls,
        &shape_ids,
        &styles,
        &typography,
        &mut removed,
        next_id,
    )?;
    // [#8362] 본문 글자보다 먼저 그린 가리는 그림은 다시 그린 본문 아래에 둔다.
    for &i in &beneath {
        match &mut controls[i] {
            Control::Shape(shape) => shape.common_mut().text_wrap = TextWrap::BehindText,
            Control::Picture(picture) => picture.common.text_wrap = TextWrap::BehindText,
            _ => {}
        }
    }
    let mut retained: Vec<_> = allocation
        .collect(
            controls
                .into_iter()
                .enumerate()
                .filter_map(|(i, c)| (!removed.contains(&i)).then_some(c)),
        )
        .map_err(message)?;
    let mut outside = Vec::new();
    let mut ordinary = Vec::new();
    for c in extra_paint {
        let is_outside = crate::model::border_paint::PaintRef::from_control(&c)
            .is_some_and(|p| p.common().outside_cell_border_paint.is_some());
        allocation
            .push(
                if is_outside {
                    &mut outside
                } else {
                    &mut ordinary
                },
                c,
            )
            .map_err(message)?;
    }
    allocation
        .reserve(&mut retained, ordinary.len())
        .map_err(message)?;
    retained.extend(ordinary);
    allocation
        .reserve(&mut retained, tables.len())
        .map_err(message)?;
    retained.extend(tables);
    allocation
        .reserve(&mut retained, body_frames.len())
        .map_err(message)?;
    retained.extend(body_frames);
    // Logical root table indices stay stable when a trailing page fragment is
    // removed by a cell-border command. Original z-order/coordinates are intact.
    allocation
        .reserve(&mut retained, outside.len())
        .map_err(message)?;
    retained.extend(outside);
    let owner = &mut doc.sections[section].paragraphs[0];
    owner.controls = retained;
    owner.char_count = 1 + owner.controls.len() as u32 * 8;
    allocation
        .set(
            &mut report,
            "mode",
            request_json!(allocation, "structured-tables-with-positional-fallback")?,
        )
        .map_err(message)?;
    allocation
        .set(
            &mut report,
            "semanticTables",
            request_json!(allocation, table_count)?,
        )
        .map_err(message)?;
    allocation
        .set(
            &mut report,
            "semanticRootTables",
            request_json!(allocation, root_count)?,
        )
        .map_err(message)?;
    allocation
        .set(
            &mut report,
            "semanticNestedTables",
            request_json!(allocation, table_count - root_count)?,
        )
        .map_err(message)?;
    allocation.set(&mut report, "nestedParagraphPolicy", request_json!(allocation, "confirmed proposals share Paragraph IR; deferred line frames do not assert authored grouping")?).map_err(message)?;
    allocation.set(&mut report, "paragraphMappingPolicy", request_json!(allocation, {
        "confirmed": "one proposal becomes one Paragraph with multiple source LineSegs",
        "deferred": "line frames retained; paragraph grouping unverified",
        "lineJoins": "inferred wrap connections; original whitespace retained",
        "bodyRegions": "confirmed owner-local paragraphs share the cell paragraph adapter; ambiguous source spans retained"
    })?).map_err(message)?;
    allocation
        .set(&mut report, "bodyParagraphMapping", body_mapping)
        .map_err(message)?;
    let mut source_typography = request_json!(allocation, {
        "policy": "recorded glyph origins; inferred spaces independent of span boundaries; HWP integer-percent character metrics",
        "fontRecovery": "unidentified source fonts use explicit metric-compatible fallback; original outlines remain unverified"
    })?;
    allocation
        .set(
            &mut source_typography,
            "lines",
            Value::Array(typography.into_inner()),
        )
        .map_err(message)?;
    allocation
        .set(&mut report, "sourceTypography", source_typography)
        .map_err(message)?;
    allocation.set(&mut report, "boundaryPaintPolicy", request_json!(allocation, "standard borders or source paint partitioned into cell-owned drawings; no page-level duplicate")?).map_err(message)?;
    allocation
        .set(
            &mut report,
            "cellOwnedPaintFragments",
            request_json!(allocation, paint_fragments)?,
        )
        .map_err(message)?;
    allocation
        .set(
            &mut report,
            "nativeBorderCells",
            request_json!(allocation, native_border_cells)?,
        )
        .map_err(message)?;
    allocation
        .set(
            &mut report,
            "nativeBorderPaintFragments",
            request_json!(allocation, native_border_fragments)?,
        )
        .map_err(message)?;
    allocation.set(&mut report, "nativeCellBackgrounds", request_json!(allocation, {"sources":cell_fill_sources,
        "policy":"single opaque rectangle behind all owned content becomes standard full-cell fill; source inset is normalized; ambiguous or partial highlights retain source paint"})?).map_err(message)?;
    allocation
        .set(
            &mut report,
            "externalBoundaryPaint",
            request_json!(allocation, {"sourcePathIndices":external_boundary_paths,
        "owner":"unchanged page source control outside logical table frame"})?,
        )
        .map_err(message)?;
    allocation
        .set(
            &mut report,
            "ruledTableCandidates",
            request_json!(allocation, grids.len())?,
        )
        .map_err(message)?;
    allocation
        .set(
            &mut report,
            "tableSemantics",
            request_json!(allocation, "inferred-from-geometry-not-authored-metadata")?,
        )
        .map_err(message)?;
    allocation
        .set(
            &mut report,
            "textBoxes",
            request_json!(
                allocation,
                owner
                    .controls
                    .iter()
                    .filter(|c| matches!(c,
        Control::Shape(s) if s.drawing().is_some_and(|d| d.text_box.is_some())))
                    .count()
            )?,
        )
        .map_err(message)?;
    allocation
        .set(
            &mut report,
            "straightRules",
            request_json!(
                allocation,
                owner
                    .controls
                    .iter()
                    .filter(|c| matches!(c,
        Control::Shape(s) if matches!(s.as_ref(), ShapeObject::Line(_))))
                    .count()
            )?,
        )
        .map_err(message)?;
    allocation.set(&mut report, "paintedRuleRectangles", request_json!(allocation, owner.controls.iter().filter(|c|matches!(c,
        Control::Shape(s) if matches!(s.as_ref(),ShapeObject::Rectangle(r) if r.drawing.text_box.is_none() && r.drawing.fill.solid.is_some()))).count())?).map_err(message)?;
    allocation
        .set(&mut report, "deferredTables", Value::Array(held))
        .map_err(message)?;
    allocation
        .set(
            &mut report,
            "hancomCompatibility",
            request_json!(allocation, "unverified-awaiting-editor-test")?,
        )
        .map_err(message)?;
    allocation
        .set(
            &mut report,
            "warnings",
            request_json!(allocation, [
        "PDF table/paragraph semantics are inferred, not recovered authored metadata",
        "PDF fonts, exact glyph advances and paragraph formatting are not fully reconstructed",
        "Hancom editor interoperability must be checked independently"
    ])?,
        )
        .map_err(message)?;
    Ok(report)
}
