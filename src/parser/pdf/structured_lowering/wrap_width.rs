//! Infer a representable soft-wrap box; source rows are not authored hard breaks.
use super::*;
use crate::renderer::{
    composer::{paragraph_flow_end, reflow_line_segs, restamp_indentation, ParagraphBox},
    equation_tac_flow::paragraph_line_indent_for_source,
    hwpunit_to_px, px_to_hwpunit,
    style_resolver::{resolve_styles_for_document, ResolvedStyleSet},
};
use pdf_oxide::geometry::Rect;
use std::cmp::Ordering;

fn ends(para: &Paragraph) -> Vec<u32> {
    para.line_segs
        .iter()
        .skip(1)
        .map(|line| line.text_start)
        .chain(std::iter::once(
            para.text.chars().map(Paragraph::char_stream_len).sum(),
        ))
        .collect()
}

struct Trial<'a> {
    source: &'a Paragraph,
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
    fn run(&mut self, width: i32) -> Paragraph {
        let (left, right) = self.margins(width);
        let style = &mut self.styles.para_styles[self.source.para_shape_id as usize];
        style.margin_left = hwpunit_to_px(left, 96.);
        style.margin_right = hwpunit_to_px(right, 96.);
        let bounds =
            ParagraphBox::content_for_style(hwpunit_to_px(self.owner_width, 96.), Some(style), 96.);
        let mut para = self.source.clone();
        reflow_line_segs(&mut para, bounds, &self.styles, 96.);
        para
    }
}

pub(super) fn fit(
    page: &ExtractedPage,
    doc: &mut Document,
    proposal: &ParagraphProposal,
    owner: Rect,
    item: &mut FlowItem,
) -> Value {
    let attempt = || -> Result<_, String> {
        if proposal.lines.len() < 2 {
            return Ok(None);
        }
        let layout = proposal.layout.as_ref().ok_or("missing paragraph layout")?;
        // Complete recorded advances bound the box from below; an ink bbox or
        // the longest string alone does not prove a line-breaking width.
        let mut minimum = 0_f64;
        for line in &proposal.lines {
            let content = reconstruct_line(&page.spans, &line.indices)?;
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
        let mut source = item.paragraph.clone();
        restamp_indentation(&mut source.line_segs, 0, original.indent);
        let mut trial = Trial {
            source: &source,
            styles: resolve_styles_for_document(doc, 96.),
            owner_width,
            left: original.margin_left / 2,
            centered,
        };
        let upper = owner_width - trial.left - original.margin_right / 2;
        let lower = (minimum * 100.).ceil() as i32;
        if lower <= 0 || lower > upper {
            return Err("recorded glyph extent exceeds owner box".into());
        }
        let expected = ends(&item.paragraph);
        let full = trial.run(upper);
        let (width, interval) = if ends(&full) == expected {
            (upper, None)
        } else {
            // Row-end vectors are compared lexicographically. Widening a
            // soft-wrap box advances its first changed cut. At most two integer
            // bisections are needed; no per-unit scan or per-document constant.
            let (mut lo, mut hi) = (lower, upper + 1);
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if ends(&trial.run(mid)).cmp(&expected) == Ordering::Less {
                    lo = mid + 1;
                } else {
                    hi = mid;
                }
            }
            let first = lo;
            if first > upper || ends(&trial.run(first)) != expected {
                return Err("source cuts have no verified ordinary wrap box".into());
            }
            hi = upper + 1;
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if ends(&trial.run(mid)).cmp(&expected) == Ordering::Greater {
                    hi = mid;
                } else {
                    lo = mid + 1;
                }
            }
            // The original authored width is unknown. The midpoint of the
            // verified interval avoids a quantization-sensitive word threshold.
            (first + (lo - first - 1) / 2, Some((first, lo - 1)))
        };
        let para = trial.run(width);
        if ends(&para) != expected || para.line_segs.len() != item.paragraph.line_segs.len() {
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
        Ok(Some((para, left, right, end, interval)))
    };
    match attempt() {
        Ok(Some((para, left, right, end, interval))) => {
            let style = &mut doc.doc_info.para_shapes[para.para_shape_id as usize];
            style.margin_left = left * 2;
            style.margin_right = right * 2;
            item.bottom = item.top + end as f32 / 100.;
            item.paragraph = para;
            json!({"status":"ordinary_flow_reproduces_observed_rows","inferredWidthIntervalHu":interval,
                "marginLeftHu":left,"marginRightHu":right,"flowEndHu":end,
                "authoredWidth":"unknown","hardBreaksAdded":false})
        }
        Ok(None) => json!({"status":"single_source_row"}),
        Err(reason) => json!({"status":"source_reflow_unverified","reason":reason}),
    }
}
