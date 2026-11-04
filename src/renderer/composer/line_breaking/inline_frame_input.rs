//! 재조판하는 인라인 호스트의 개체 슬롯을 프레임 입력으로 투영한다.
//! 원본 텍스트·제어문은 바꾸지 않고 원시 UTF-16 경계를 그대로 발행한다.

use super::*;

pub(super) fn project(
    source: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<(Paragraph, Vec<FlowInlineControl>)> {
    match project_with_storage(source, styles, dpi, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

pub(crate) fn project_with_storage<S: crate::storage::Storage>(
    source: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
    storage: &S,
) -> Result<Option<(Paragraph, Vec<FlowInlineControl>)>, S::Error> {
    Ok({
        let chars: Vec<char> = storage.collect(source.text.chars())?;
        let mut slots = storage.vec(0)?;
        for ((control, position), raw) in source
            .controls
            .iter()
            .zip(source.try_control_text_positions(storage)?)
            .zip(source.control_utf16_positions_with_storage(storage)?)
        {
            let Some((width, height)) =
                inline_control_occupied_size_hwp_with_storage(control, styles, dpi, storage)?
            else {
                continue;
            };
            let position = if chars.is_empty() && source.char_offsets.is_empty() {
                0
            } else {
                position
            };
            if position <= chars.len() {
                storage.push(&mut slots, (control, position, raw, width, height))?;
            }
        }
        if slots.is_empty() {
            return Ok(None);
        }
        let mut input = storage.clone_paragraph(source)?;
        input.text.clear();
        input.char_offsets.clear();
        input.layout_field_values.clear();
        let mut controls = storage.vec(slots.len())?;
        let mut scalar_index = 0;
        for position in 0..=chars.len() {
            let mut represented = false;
            for &(control, owner, raw, width, height) in &slots {
                if owner != position {
                    continue;
                }
                storage.push_char(&mut input.text, '\u{FFFC}')?;
                storage.push(&mut input.char_offsets, raw)?;
                storage.push(
                    &mut controls,
                    FlowInlineControl {
                        char_position: scalar_index,
                        utf16_position: raw,
                        width_hwp: width,
                        height_hwp: height,
                        baseline_distance_hwp: flow_inline_baseline_hwp(control, height),
                        spacing_height_hwp: inline_control_spacing_height_hwp(control),
                    },
                )?;
                scalar_index += 1;
                represented |= chars.get(position) == Some(&'\u{FFFC}')
                    && source.char_offsets.get(position) == Some(&raw);
            }
            if let Some(&character) = chars.get(position).filter(|_| !represented) {
                if let Some((_, value)) = source
                    .layout_field_values
                    .iter()
                    .find(|(at, _)| *at == position)
                {
                    storage.push(
                        &mut input.layout_field_values,
                        (scalar_index, storage.string(value)?),
                    )?;
                }
                storage.push_char(&mut input.text, character)?;
                storage.push(
                    &mut input.char_offsets,
                    char_index_to_utf16_offset(source, position),
                )?;
                scalar_index += 1;
            }
        }
        Some((input, controls))
    })
}
