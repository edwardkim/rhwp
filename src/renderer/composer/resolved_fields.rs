//! 동적 필드는 모델 한 칸을 유지하며 표시 문자열과 줄 나눔 폭을 함께 제공한다.

use super::{ComposedParagraph, ComposedTextRun};
use crate::model::paragraph::Paragraph;
use crate::renderer::style_resolver::{detect_lang_category, ResolvedStyleSet};

impl ComposedTextRun {
    /// 모델 축으로 런을 잘라도 필드·PUA의 표시 문자열을 잃지 않는다.
    pub(crate) fn display_for_model_range(&self, start: usize, end: usize) -> Option<String> {
        match self.try_display_for_model_range(start, end, &crate::storage::Unbounded) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
    pub(super) fn try_display_for_model_range<S: crate::storage::Storage>(
        &self,
        start: usize,
        end: usize,
        storage: &S,
    ) -> Result<Option<String>, S::Error> {
        let Some(display) = self.display_text.as_ref() else {
            return Ok(None);
        };
        let model_len = self.text.chars().count();
        if start == 0 && end == model_len {
            return storage.string(display).map(Some);
        }
        if display.chars().count() == model_len {
            return storage
                .chars(display.chars().skip(start).take(end - start))
                .map(Some);
        }
        let text = storage.chars(self.text.chars().skip(start).take(end - start))?;
        let expanded = super::try_expand_pua_surface(&text, None, false, storage)?;
        Ok((expanded != text).then_some(expanded))
    }
}

pub(super) fn try_apply<S: crate::storage::Storage>(
    para: &Paragraph,
    composed: &mut ComposedParagraph,
    storage: &S,
) -> Result<(), S::Error> {
    if para.layout_field_values.is_empty() {
        return Ok(());
    }
    for line in &mut composed.lines {
        let mut position = line.char_start;
        let mut runs = storage.vec(0)?;
        for run in &line.runs {
            if run.inserted_control_text {
                storage.push(&mut runs, run.try_clone(storage)?)?;
                continue;
            }
            let chars: Vec<_> = storage.collect(run.text.chars())?;
            let mut plain_start = 0;
            for index in 0..chars.len() {
                let Some((_, value)) = para
                    .layout_field_values
                    .iter()
                    .find(|(at, _)| *at == position + index)
                else {
                    continue;
                };
                try_push_slice(&mut runs, run, &chars, plain_start, index, storage)?;
                let mut field = run.try_clone(storage)?;
                field.text = storage.chars(std::iter::once(chars[index]))?;
                field.display_text = Some(storage.string(value)?);
                field.lang_index = value
                    .chars()
                    .next()
                    .map(detect_lang_category)
                    .unwrap_or(run.lang_index);
                storage.push(&mut runs, field)?;
                plain_start = index + 1;
            }
            try_push_slice(&mut runs, run, &chars, plain_start, chars.len(), storage)?;
            position += chars.len();
        }
        line.runs = runs;
    }
    Ok(())
}

fn try_push_slice<S: crate::storage::Storage>(
    out: &mut Vec<ComposedTextRun>,
    source: &ComposedTextRun,
    chars: &[char],
    start: usize,
    end: usize,
    storage: &S,
) -> Result<(), S::Error> {
    if start == end {
        return Ok(());
    }
    let mut piece = source.try_clone(storage)?;
    piece.text = storage.chars(chars[start..end].iter().copied())?;
    piece.display_text = source.try_display_for_model_range(start, end, storage)?;
    storage.push(out, piece)?;
    Ok(())
}

/// 자동 번호는 줄 수용 시 한 글자 상자를 예약한다. 실제 pen 전진은 표시
/// 숫자의 폭이다. 독립 Print의 36/38.5px에서는 `1경`·`계`, 39.52px에서는 `1경계`다.
/// 20pt 대조군도 같은 em 비례 경계를 따르며, PAGE12도39.52px에서 한 줄이다.
pub(super) fn token_width(
    para: &Paragraph,
    position: usize,
    styles: &ResolvedStyleSet,
) -> Option<(f64, f64)> {
    let _ = para
        .layout_field_values
        .iter()
        .find(|(at, _)| *at == position)?;
    let style_id = para.char_shape_id_at(position).unwrap_or(0);
    let size = styles
        .char_styles
        .get(style_id as usize)
        .map_or(12.0, |s| s.font_size);
    Some((size, size))
}
