//! Versioned HWP5 paragraph-text conversion at the HWPX boundary.
//!
//! Hancom 12.30.0.6544 saves the paired 5.0.1.5 / 5.0.1.6 fixtures differently:
//! only the older input expands measured BMP symbols to supplementary PUA.
//! This is not a font-cmap substitution or a rule for every A000–A8FF value.

use crate::model::{document::Document, paragraph::Paragraph};
use crate::serializer::SerializeError;

/// Values independently observed in the version-boundary samples in addition to
/// the existing #5140/#5861 lookup. Do not extend the compose lookup with these.
const BODY_SYMBOLS: &[u16] = &[
    0xA806, 0xA808, 0xA80C, 0xA811, 0xA813, 0xA817, 0xA81C, 0xA827, 0xA848,
];

pub(super) fn uses_legacy_body_symbols(doc: &Document) -> bool {
    let v = &doc.header.version;
    doc.layout_profile().hwp5_container() && (v.major, v.minor, v.build, v.revision) < (5, 0, 1, 6)
}

pub(super) fn body_symbol(c: char, legacy: bool) -> char {
    if !legacy {
        return c;
    }
    let Ok(unit) = u16::try_from(u32::from(c)) else {
        return c;
    };
    let cp = crate::parser::tags::hancom_symbol_to_plane15(unit).or_else(|| {
        BODY_SYMBOLS
            .contains(&unit)
            .then(|| 0xF0000 | u32::from(unit & 0x0FFF))
    });
    cp.and_then(char::from_u32).unwrap_or(c)
}

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
