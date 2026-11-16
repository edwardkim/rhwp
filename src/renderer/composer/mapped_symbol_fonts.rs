//! 표시 투영된 기호만 실제 글꼴 프로그램을 소유하는 별도 런으로 보존한다.

use super::{ComposedParagraph, ComposedTextRun};
use unicode_segmentation::UnicodeSegmentation;

pub(super) const MAPPED_SYMBOL_FONT: &str = "Noto Sans Symbols2";

fn mapped_symbol(source: &str, display: &str) -> bool {
    matches!(
        (source, display),
        ("\u{F06D}", "\u{1F53E}") | ("\u{F0FA}", "\u{1F8AD}")
    )
}

pub(super) fn owns_mapped_symbol_font(run: &ComposedTextRun) -> bool {
    run.display_text
        .as_deref()
        .is_some_and(|display| mapped_symbol(&run.text, display))
        && run.char_overlap.is_none()
        && run.footnote_marker.is_none()
}

/// 원문 한 글자와 대응 표시 한 글자가 같은 경계를 가질 때만 나눈다.
/// 결합 문자·삽입 컨트롤·표시 길이가 다른 옛한글 투영을 기호 런으로 추측하지 않는다.
pub(super) fn try_split_mapped_symbol_font_runs<S: crate::storage::Storage>(
    composed: &mut ComposedParagraph,
    storage: &S,
) -> Result<(), S::Error> {
    for line in &mut composed.lines {
        let mut next = storage.vec(line.runs.len())?;
        for run in std::mem::take(&mut line.runs) {
            if !run.text.contains(['\u{F06D}', '\u{F0FA}'])
                || run.char_overlap.is_some()
                || run.footnote_marker.is_some()
                || run.inserted_control_text
            {
                storage.push(&mut next, run)?;
                continue;
            }
            let Some(display) = run.display_text.as_deref() else {
                storage.push(&mut next, run)?;
                continue;
            };
            let display_chars: Vec<_> = storage.collect(display.chars())?;
            if display_chars.len() != run.text.chars().count() {
                storage.push(&mut next, run)?;
                continue;
            }
            let mut ranges = storage.vec(0)?;
            let mut position = 0;
            let mut start = 0;
            for grapheme in run.text.graphemes(true) {
                let end = position + grapheme.chars().count();
                let mut display_bytes = [0; 4];
                if end == position + 1
                    && mapped_symbol(
                        grapheme,
                        display_chars[position].encode_utf8(&mut display_bytes),
                    )
                {
                    if start < position {
                        storage.push(&mut ranges, (start, position))?;
                    }
                    storage.push(&mut ranges, (position, end))?;
                    start = end;
                }
                position = end;
            }
            if ranges.is_empty() {
                storage.push(&mut next, run)?;
                continue;
            }
            if start < position {
                storage.push(&mut ranges, (start, position))?;
            }
            let raw_chars: Vec<_> = storage.collect(run.text.chars())?;
            for (start, end) in ranges {
                let mut piece = run.try_clone(storage)?;
                piece.text = storage.chars(raw_chars[start..end].iter().copied())?;
                let displayed: String = storage.chars(display_chars[start..end].iter().copied())?;
                piece.display_text = (displayed != piece.text).then_some(displayed);
                storage.push(&mut next, piece)?;
            }
        }
        line.runs = next;
    }
    Ok(())
}
