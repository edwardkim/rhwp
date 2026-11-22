//! UTF-16 position changes at the HWP5 to HWPX paragraph-text boundary.

use crate::model::{paragraph::Paragraph, provenance::body_symbol};
use crate::serializer::SerializeError;

/// Source UTF-16 positions immediately after characters that grow by one unit.
/// Keep original control-slot gaps and already supplementary characters intact.
#[derive(Default)]
pub(super) struct BodySymbolExpansion {
    ends: Vec<u32>,
}

impl BodySymbolExpansion {
    pub(super) fn for_paragraph<S: crate::storage::Storage>(
        para: &Paragraph,
        legacy: bool,
        storage: &S,
    ) -> Result<Self, SerializeError>
    where
        S::Error: std::fmt::Display,
    {
        if !legacy {
            return Ok(Self::default());
        }
        let mut ends = Vec::new();
        let mut next = 0u32;
        for (i, c) in para.text.chars().enumerate() {
            let start = para.char_offsets.get(i).copied().unwrap_or(next);
            next = start.saturating_add(Paragraph::char_stream_len(c));
            if body_symbol(c, true) != c {
                storage
                    .reserve(&mut ends, 1)
                    .map_err(super::utils::xml_error)?;
                ends.push(start.saturating_add(1));
            }
        }
        ends.sort_unstable();
        Ok(Self { ends })
    }

    pub(super) fn is_empty(&self) -> bool {
        self.ends.is_empty()
    }

    pub(super) fn project(&self, source: u32) -> u32 {
        source.saturating_add(self.ends.partition_point(|&end| end <= source) as u32)
    }

    /// Verification uses the same expansion as the writer, while the general IR
    /// comparator still reports unrelated style or line-position loss. Text is checked against the Hancom oracle.
    pub(super) fn apply_to_expected(&self, para: &mut Paragraph) {
        if self.is_empty() {
            return;
        }
        para.text = para.text.chars().map(|c| body_symbol(c, true)).collect();
        para.char_count = self.project(para.char_count);
        for offset in &mut para.char_offsets {
            *offset = self.project(*offset);
        }
        for shape in &mut para.char_shapes {
            shape.start_pos = self.project(shape.start_pos);
        }
        for tag in &mut para.range_tags {
            tag.start = self.project(tag.start);
            tag.end = self.project(tag.end);
        }
        for seg in &mut para.line_segs {
            let paragraph_pos = seg.text_start.saturating_add(para.hwpx_axis_shift);
            seg.text_start = self
                .project(paragraph_pos)
                .saturating_sub(para.hwpx_axis_shift);
        }
        for mark in &mut para.markpen_marks {
            if let Some(pos) = &mut mark.utf16_pos {
                *pos = self.project(*pos);
            }
        }
    }
}
