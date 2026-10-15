//! 동적 필드는 모델 한 칸을 유지하며 표시 문자열과 줄 나눔 폭을 함께 제공한다.

use super::{ComposedParagraph, ComposedTextRun};
use crate::model::paragraph::Paragraph;
use crate::renderer::style_resolver::{detect_lang_category, ResolvedStyleSet};

impl ComposedTextRun {
    /// 모델 축으로 런을 잘라도 필드·PUA의 표시 문자열을 잃지 않는다.
    pub(crate) fn display_for_model_range(&self, start: usize, end: usize) -> Option<String> {
        let display = self.display_text.as_ref()?;
        let model_len = self.text.chars().count();
        if start == 0 && end == model_len {
            return Some(display.clone());
        }
        if display.chars().count() == model_len {
            return Some(display.chars().skip(start).take(end - start).collect());
        }
        let text: String = self.text.chars().skip(start).take(end - start).collect();
        let expanded = super::expand_pua_display_text(&text);
        (expanded != text).then_some(expanded)
    }
}

pub(super) fn apply(para: &Paragraph, composed: &mut ComposedParagraph) {
    if para.layout_field_values.is_empty() {
        return;
    }
    for line in &mut composed.lines {
        let mut position = line.char_start;
        let mut runs = Vec::new();
        for run in &line.runs {
            if run.inserted_control_text {
                runs.push(run.clone());
                continue;
            }
            let chars: Vec<_> = run.text.chars().collect();
            let mut plain_start = 0;
            for index in 0..chars.len() {
                let Some((_, value)) = para
                    .layout_field_values
                    .iter()
                    .find(|(at, _)| *at == position + index)
                else {
                    continue;
                };
                push_slice(&mut runs, run, &chars, plain_start, index);
                let mut field = run.clone();
                field.text = chars[index].to_string();
                field.display_text = Some(value.clone());
                field.lang_index = value
                    .chars()
                    .next()
                    .map(detect_lang_category)
                    .unwrap_or(run.lang_index);
                runs.push(field);
                plain_start = index + 1;
            }
            push_slice(&mut runs, run, &chars, plain_start, chars.len());
            position += chars.len();
        }
        line.runs = runs;
    }
}

fn push_slice(
    out: &mut Vec<ComposedTextRun>,
    source: &ComposedTextRun,
    chars: &[char],
    start: usize,
    end: usize,
) {
    if start == end {
        return;
    }
    let mut piece = source.clone();
    piece.text = chars[start..end].iter().collect();
    piece.display_text = source.display_for_model_range(start, end);
    out.push(piece);
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
