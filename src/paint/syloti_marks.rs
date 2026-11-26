//! Paint a defective Syloti Nagri hasanta cluster on a dotted circle. Hancom
//! keeps the original layout advance for the following script run. An explicit
//! circle, in contrast, is a source character and already occupies its space.
use crate::paint::PaintOp;
use crate::renderer::layout::compute_char_positions;
use crate::renderer::render_tree::{BoundingBox, TextRunNode};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};

const HASANTA: char = '\u{a806}';
const DOTTED_CIRCLE: char = '\u{25cc}';

fn is_syloti(ch: char) -> bool {
    ('\u{a800}'..='\u{a82c}').contains(&ch)
}

fn accepts_hasanta(ch: char) -> bool {
    // Hancom keeps these USE generic bases in the Syloti item. Its Print
    // starts a new item after whitespace, multiplication and short hyphens.
    (is_syloti(ch) && ch.general_category() == GeneralCategory::OtherLetter)
        || matches!(ch, '\u{2012}'..='\u{2015}' | '\u{2022}' | DOTTED_CIRCLE | '\u{25fb}'..='\u{25fe}')
}

fn preceding_scalar(run: &TextRunNode, previous: Option<&TextRunNode>) -> Option<char> {
    let prior = previous?;
    if prior.section_index != run.section_index
        || prior.para_index != run.para_index
        || prior.cell_context != run.cell_context
        || prior.char_start?.checked_add(prior.logical_char_count())? != run.char_start?
    {
        return None;
    }
    prior.display_or_text().chars().next_back()
}

pub(super) fn push_text(
    ops: &mut Vec<PaintOp>,
    bbox: BoundingBox,
    run: &TextRunNode,
    previous: Option<&TextRunNode>,
) {
    let text = &run.text;
    if !text.contains(HASANTA) || run.char_overlap.is_some() || run.display_or_text() != text {
        ops.push(PaintOp::text_run(bbox, run.clone()));
        return;
    }

    let mut prior = preceding_scalar(run, previous);
    let mut offsets = Vec::new();
    let mut broken = Vec::new();
    for (index, (byte, ch)) in text.char_indices().enumerate() {
        offsets.push(byte);
        if ch == HASANTA && !prior.is_some_and(accepts_hasanta) {
            broken.push(index);
        }
        if ch != '\u{200d}' {
            prior = Some(ch);
        }
    }
    if broken.is_empty() {
        ops.push(PaintOp::text_run(bbox, run.clone()));
        return;
    }
    offsets.push(text.len());

    let positions = run.replay_positions_for(text);
    let circle_advance = compute_char_positions("◌", &run.style)[1];
    let mut template = run.clone();
    template.text.clear();
    template.display_text = None;
    template.layout_positions = None;

    let mut breaks = broken.into_iter().peekable();
    let mut start = 0;
    let count = offsets.len() - 1;
    while start < count {
        let belongs_to_script = |index: usize| {
            let ch = text[offsets[index]..]
                .chars()
                .next()
                .expect("scalar boundary");
            is_syloti(ch) || matches!(ch, '\u{200c}' | '\u{200d}')
        };
        let script = belongs_to_script(start);
        let mut end = start + 1;
        while end < count && belongs_to_script(end) == script {
            end += 1;
        }
        let mut piece = template.clone();
        piece.text.push_str(&text[offsets[start]..offsets[end]]);
        piece.char_start = run.char_start.and_then(|n| n.checked_add(start));
        piece.is_para_end = run.is_para_end && end == count;
        piece.is_line_break_end = run.is_line_break_end && end == count;
        let origin = positions[start];
        let mut pos = Vec::with_capacity(end - start + 1);
        let mut added = 0.0;
        let has_mark = breaks.peek().is_some_and(|i| *i < end);
        let mut glyphs = String::new();
        for (index, ch) in (start..end).zip(piece.text.chars()) {
            if breaks.peek() == Some(&index) {
                pos.push(positions[index] - origin + added);
                glyphs.push(DOTTED_CIRCLE);
                added += circle_advance;
                breaks.next();
            }
            pos.push(positions[index] - origin + added);
            if has_mark {
                glyphs.push(ch);
            }
        }
        let width = positions[end] - origin + added;
        pos.push(width);
        if has_mark {
            piece.display_text = Some(glyphs);
        }
        piece.layout_positions = Some(pos);

        // The original and the slice rotate about their respective centers.
        let shift = origin + (width - bbox.width) * 0.5;
        let (sin, cos) = run.rotation.to_radians().sin_cos();
        let frame = BoundingBox::new(
            bbox.x + origin + (cos - 1.0) * shift,
            bbox.y + sin * shift,
            width,
            bbox.height,
        );
        ops.push(PaintOp::text_run(frame, piece));
        start = end;
    }
}
