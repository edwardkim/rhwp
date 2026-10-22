//! 재조판하는 인라인 호스트의 개체 슬롯을 프레임 입력으로 투영한다.
//! 원본 텍스트·제어문은 바꾸지 않고 원시 UTF-16 경계를 그대로 발행한다.

use super::*;

pub(super) fn project(
    source: &Paragraph,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<(Paragraph, Vec<FlowInlineControl>)> {
    let chars: Vec<char> = source.text.chars().collect();
    let slots: Vec<_> = source
        .controls
        .iter()
        .zip(source.control_text_positions())
        .zip(source.control_utf16_positions())
        .filter_map(|((control, position), raw)| {
            // 공통 크기는 그림 본체와 바깥여백을 이미 포함한다.
            // 투영 슬롯은 같은 점유를 소비하며 여백을 다시 합산하지 않는다.
            let (width, height) = inline_control_occupied_size_hwp(control, styles, dpi)?;
            // 글자 없는 원시 문단의 제어문은 서로 다른 UTF-16 슬롯을 갖지만
            // 가시 문자 축에서는 모두 유일한 빈 경계에 속한다. 순차 인덱스
            // 폴백을 가시 글자로 해석하면 두 번째 이후 개체가 입력에서 빠진다.
            let position = if chars.is_empty() && source.char_offsets.is_empty() {
                0
            } else {
                position
            };
            (position <= chars.len()).then_some((control, position, raw, width, height))
        })
        .collect();
    if slots.is_empty() {
        return None;
    }
    let mut input = source.clone();
    input.text.clear();
    input.char_offsets.clear();
    input.layout_field_values.clear();
    let mut controls = Vec::with_capacity(slots.len());
    let mut scalar_index = 0;
    for position in 0..=chars.len() {
        let mut represented = false;
        for &(control, owner, raw, width, height) in &slots {
            if owner != position {
                continue;
            }
            input.text.push('\u{FFFC}');
            input.char_offsets.push(raw);
            controls.push(FlowInlineControl {
                char_position: scalar_index,
                utf16_position: raw,
                width_hwp: width,
                height_hwp: height,
                baseline_distance_hwp: flow_inline_baseline_hwp(control, height),
                spacing_height_hwp: inline_control_spacing_height_hwp(control),
            });
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
                input
                    .layout_field_values
                    .push((scalar_index, value.clone()));
            }
            input.text.push(character);
            input
                .char_offsets
                .push(char_index_to_utf16_offset(source, position));
            scalar_index += 1;
        }
    }
    Some((input, controls))
}
