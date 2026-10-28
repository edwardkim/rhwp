//! Infer a representable soft-wrap box; source rows are not authored hard breaks.
use super::*;
use crate::renderer::{
    composer::{paragraph_flow_end, restamp_indentation, try_reflow_line_segs, ParagraphBox},
    equation_tac_flow::paragraph_line_indent_for_source,
    hwpunit_to_px, px_to_hwpunit,
    style_resolver::ResolvedStyleSet,
};
use pdf_oxide::geometry::Rect;
use std::{cmp::Ordering, collections::VecDeque};

/// The common composer needs only this paragraph's referenced styles. Resolve
/// that projection, rather than rebuilding the growing document for every trial.
/// IDs are local only during composition and restored before publishing the row.
fn projection(
    doc: &Document,
    para: &mut Paragraph,
    allocation: Allocation<'_>,
) -> Result<(ResolvedStyleSet, Vec<u32>), String> {
    let mut ids = allocation
        .collect(para.char_shapes.iter().map(|run| run.char_shape_id))
        .map_err(message)?;
    if let Some(style) = doc.doc_info.styles.first() {
        allocation
            .push(&mut ids, u32::from(style.char_shape_id))
            .map_err(message)?;
    }
    ids.sort_unstable();
    ids.dedup();
    let mut char_styles = allocation.vec(ids.len()).map_err(message)?;
    for &id in &ids {
        let shape = doc
            .doc_info
            .char_shapes
            .get(id as usize)
            .ok_or("missing paragraph character style")?;
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
    let shape = doc
        .doc_info
        .para_shapes
        .get(para.para_shape_id as usize)
        .ok_or("missing paragraph style")?;
    let para_style = crate::renderer::style_resolver::try_resolve_paragraph_style(
        shape,
        &doc.doc_info,
        96.,
        &allocation,
    )
    .map_err(message)?;
    // This private producer resolves normalized PDF styles before Core editing.
    // Its old projection had no sections, native font programs, page-number
    // style, borders or numbering. Keep those empty; borrow all source font
    // tables and share the common character/paragraph rules rather than copying.
    allocation
        .bytes(
            2 * std::mem::size_of::<usize>()
                + std::mem::size_of::<crate::renderer::native_pua_fonts::NativePuaFonts>(),
        )
        .map_err(message)?;
    let styles = ResolvedStyleSet {
        char_styles,
        para_styles: allocation.collect([para_style]).map_err(message)?,
        resolved_dpi: Some(96.),
        ..Default::default()
    };
    for run in &mut para.char_shapes {
        run.char_shape_id = ids.binary_search(&run.char_shape_id).unwrap() as u32;
    }
    para.para_shape_id = 0;
    Ok((styles, ids))
}

fn ends(para: &Paragraph, allocation: Allocation<'_>) -> Result<Vec<u32>, String> {
    allocation
        .collect(
            para.line_segs
                .iter()
                .skip(1)
                .map(|line| line.text_start)
                .chain(std::iter::once(
                    para.text.chars().map(Paragraph::char_stream_len).sum(),
                )),
        )
        .map_err(message)
}

struct Trial<'a> {
    source: &'a Paragraph,
    allocation: Allocation<'a>,
    styles: ResolvedStyleSet,
    owner_width: i32,
    left: i32,
    centered: bool,
}
impl Trial<'_> {
    fn margins(&self, width: i32) -> (i32, i32) {
        let left = if self.centered {
            (self.owner_width - width) / 2
        } else {
            self.left
        };
        (left, self.owner_width - left - width)
    }
    fn run(&mut self, width: i32) -> Result<Paragraph, String> {
        let (left, right) = self.margins(width);
        let style = &mut self.styles.para_styles[self.source.para_shape_id as usize];
        style.margin_left = hwpunit_to_px(left, 96.);
        style.margin_right = hwpunit_to_px(right, 96.);
        let bounds =
            ParagraphBox::content_for_style(hwpunit_to_px(self.owner_width, 96.), Some(style), 96.);
        let mut para = self.allocation.clone_record(self.source).map_err(message)?;
        try_reflow_line_segs(&mut para, bounds, &self.styles, 96., &self.allocation)
            .map_err(message)?;
        Ok(para)
    }

    fn width_for_cuts(
        &mut self,
        lower: i32,
        upper: i32,
        expected: &[u32],
    ) -> Result<(i32, Option<(i32, i32)>), String> {
        let allocation = self.allocation;
        let mut ordered_search = || -> Result<_, String> {
            let (mut lo, mut hi) = (lower, upper + 1);
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if ends(&self.run(mid)?, allocation)?.as_slice().cmp(expected) == Ordering::Less {
                    lo = mid + 1;
                } else {
                    hi = mid;
                }
            }
            let first = lo;
            if first > upper || ends(&self.run(first)?, allocation)? != expected {
                return Err("source cuts have no verified ordinary wrap box".into());
            }
            hi = upper + 1;
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if ends(&self.run(mid)?, allocation)?.as_slice().cmp(expected) == Ordering::Greater
                {
                    hi = mid;
                } else {
                    lo = mid + 1;
                }
            }
            let width = first + (lo - first - 1) / 2;
            if ends(&self.run(width)?, allocation)? != expected {
                return Err("selected box does not reproduce source cuts".into());
            }
            Ok((width, Some((first, lo - 1))))
        };
        let ordered = ordered_search();
        if ordered.is_ok() || !self.source.text.contains('\t') {
            return ordered;
        }
        // A later tab can move a break back before an earlier word separator
        // as the box widens. Cut vectors with tabs are not globally monotone.
        // Search both halves, using only the real common reflow as a witness.
        // This bounded fallback proves one usable width, not an authored box
        // or the absence of other widths when its work budget is exhausted.
        const MAX_TAB_WIDTH_TRIALS: usize = 64;
        allocation.0.check().map_err(|error| error.to_string())?;
        let mut pending = VecDeque::new();
        allocation
            .payload::<(i32, i32)>(MAX_TAB_WIDTH_TRIALS + 1)
            .map_err(message)?;
        pending
            .try_reserve_exact(MAX_TAB_WIDTH_TRIALS + 1)
            .map_err(|_| {
                message(super::super::PdfImportError::backend(
                    allocation.0.reject("wrap search queue allocation"),
                ))
            })?;
        pending.push_back((lower, upper));
        for _ in 0..MAX_TAB_WIDTH_TRIALS {
            let Some((lo, hi)) = pending.pop_front() else {
                break;
            };
            let width = lo + (hi - lo) / 2;
            if ends(&self.run(width)?, allocation)? == expected {
                return Ok((width, Some((width, width))));
            }
            if lo < width {
                pending.push_back((lo, width - 1));
            }
            if width < hi {
                pending.push_back((width + 1, hi));
            }
        }
        Err("tabbed source cuts have no verified wrap witness within search budget".into())
    }
}

pub(super) fn fit(
    page: &ExtractedPage,
    doc: &mut Document,
    proposal: &ParagraphProposal,
    owner: Rect,
    item: &mut FlowItem,
) -> Result<Value, String> {
    let allocation = Allocation(&page.import_budget);
    let attempt = || -> Result<_, String> {
        if proposal.lines.len() < 2 {
            return Ok(None);
        }
        let layout = proposal.layout.as_ref().ok_or("missing paragraph layout")?;
        // Complete recorded advances bound the box from below; an ink bbox or
        // the longest string alone does not prove a line-breaking width.
        let mut minimum = 0_f64;
        for line in &proposal.lines {
            let content =
                reconstruct_line(&page.spans, &line.indices, Allocation(&page.import_budget))?;
            let first = content.characters.first().ok_or("empty source row")?;
            let last = content.characters.last().ok_or("empty source row")?;
            let indent = if layout.alignment == TextAlignment::Center {
                0.
            } else {
                f64::from(line.left - owner.x - layout.margin_left)
            };
            minimum = minimum.max(f64::from(last.origin + last.advance - first.origin) + indent);
        }
        let original = &doc.doc_info.para_shapes[item.paragraph.para_shape_id as usize];
        let owner_width = hu(owner.width)? as i32;
        let centered = layout.alignment == TextAlignment::Center;
        // These are observed PDF rows, not Hancom records denying indentation.
        // Trial the inferred paragraph rule using the same stamp as an explicit
        // indent edit. Publish it only after the physical source origins agree.
        let mut source = allocation.clone_record(&item.paragraph).map_err(message)?;
        restamp_indentation(&mut source.line_segs, 0, original.indent);
        let (styles, char_ids) = projection(doc, &mut source, allocation)?;
        let mut trial = Trial {
            source: &source,
            allocation,
            styles,
            owner_width,
            left: original.margin_left / 2,
            centered,
        };
        let upper = owner_width - trial.left - original.margin_right / 2;
        let lower = (minimum * 100.).ceil() as i32;
        if lower <= 0 || lower > upper {
            return Err("recorded glyph extent exceeds owner box".into());
        }
        let expected = ends(&item.paragraph, allocation)?;
        let full = trial.run(upper)?;
        let (width, interval) = if ends(&full, allocation)? == expected {
            (upper, None)
        } else {
            trial.width_for_cuts(lower, upper, &expected)?
        };
        let mut para = trial.run(width)?;
        if ends(&para, allocation)? != expected
            || para.line_segs.len() != item.paragraph.line_segs.len()
        {
            return Err("selected box does not reproduce source cuts".into());
        }
        let style = &trial.styles.para_styles[para.para_shape_id as usize];
        for (index, (fresh, stored)) in para
            .line_segs
            .iter()
            .zip(&item.paragraph.line_segs)
            .enumerate()
        {
            let indent = paragraph_line_indent_for_source(
                style.indent,
                index,
                Some(&para),
                para.line_segs.len(),
                true,
                hwpunit_to_px(fresh.segment_width, 96.),
                96.,
            );
            let physical_start = fresh.column_start + px_to_hwpunit(indent, 96.);
            if (fresh.vertical_pos + fresh.baseline_distance
                - stored.vertical_pos
                - stored.baseline_distance)
                .abs()
                > 50
                || (!centered && (physical_start - stored.column_start).abs() > 50)
            {
                return Err("ordinary flow does not reproduce source origins".into());
            }
        }
        let (left, right) = trial.margins(width);
        let end = paragraph_flow_end(&para).ok_or("empty common flow result")?;
        for run in &mut para.char_shapes {
            run.char_shape_id = char_ids[run.char_shape_id as usize];
        }
        para.para_shape_id = item.paragraph.para_shape_id;
        Ok(Some((para, left, right, end, interval)))
    };
    match attempt() {
        Ok(Some((para, left, right, end, interval))) => {
            let report = request_json!(allocation, {"status":"ordinary_flow_reproduces_observed_rows","inferredWidthIntervalHu":interval,
                "marginLeftHu":left,"marginRightHu":right,"flowEndHu":end,
                "authoredWidth":"unknown","hardBreaksAdded":false})?;
            let style = &mut doc.doc_info.para_shapes[para.para_shape_id as usize];
            style.margin_left = left * 2;
            style.margin_right = right * 2;
            item.bottom = item.top + end as f32 / 100.;
            item.paragraph = para;
            Ok(report)
        }
        Ok(None) => Ok(request_json!(allocation, {"status":"single_source_row"})?),
        Err(reason) => {
            allocation.0.check().map_err(|error| error.to_string())?;
            Ok(request_json!(allocation, {"status":"source_reflow_unverified","reason":reason})?)
        }
    }
}
