//! 표시 투영된 기호만 실제 글꼴 프로그램을 소유하는 별도 런으로 보존한다.

use super::{ComposedParagraph, ComposedTextRun};
use unicode_segmentation::UnicodeSegmentation;

pub(super) const SHADOWED_CIRCLE_FONT: &str = "Noto Sans Symbols2";

pub(super) fn owns_shadowed_circle_font(run: &ComposedTextRun) -> bool {
    run.text == "\u{F06D}"
        && run.display_text.as_deref() == Some("\u{1F53E}")
        && run.char_overlap.is_none()
        && run.footnote_marker.is_none()
}

/// 원문 한 글자와 대응 표시 한 글자가 같은 경계를 가질 때만 나눈다.
/// 결합 문자·삽입 컨트롤·표시 길이가 다른 옛한글 투영을 기호 런으로 추측하지 않는다.
pub(super) fn split_mapped_symbol_font_runs(composed: &mut ComposedParagraph) {
    for line in &mut composed.lines {
        let mut next = Vec::with_capacity(line.runs.len());
        for run in std::mem::take(&mut line.runs) {
            if !run.text.contains('\u{F06D}')
                || run.char_overlap.is_some()
                || run.footnote_marker.is_some()
                || run.inserted_control_text
            {
                next.push(run);
                continue;
            }
            let Some(display) = run.display_text.as_deref() else {
                next.push(run);
                continue;
            };
            let display_chars: Vec<_> = display.chars().collect();
            if display_chars.len() != run.text.chars().count() {
                next.push(run);
                continue;
            }
            let mut ranges = Vec::new();
            let mut position = 0;
            let mut start = 0;
            for grapheme in run.text.graphemes(true) {
                let end = position + grapheme.chars().count();
                if grapheme == "\u{F06D}" && display_chars[position] == '\u{1F53E}' {
                    if start < position {
                        ranges.push((start, position));
                    }
                    ranges.push((position, end));
                    start = end;
                }
                position = end;
            }
            if ranges.is_empty() {
                next.push(run);
                continue;
            }
            if start < position {
                ranges.push((start, position));
            }
            let raw_chars: Vec<_> = run.text.chars().collect();
            for (start, end) in ranges {
                let mut piece = run.clone();
                piece.text = raw_chars[start..end].iter().collect();
                let displayed: String = display_chars[start..end].iter().collect();
                piece.display_text = (displayed != piece.text).then_some(displayed);
                next.push(piece);
            }
        }
        line.runs = next;
    }
}
