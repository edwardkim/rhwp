//! 저장 줄이 없는 인라인 표 호스트의 개체 슬롯을 프레임 입력으로 투영한다.
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
            let (width, height) = inline_control_occupied_size_hwp(control, styles, dpi)?;
            (position <= chars.len()).then_some((control, position, raw, width, height))
        })
        .collect();
    if slots.is_empty() {
        return None;
    }
    let mut input = source.clone();
    input.text.clear();
    input.char_offsets.clear();
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
            });
            scalar_index += 1;
            represented |= chars.get(position) == Some(&'\u{FFFC}')
                && source.char_offsets.get(position) == Some(&raw);
        }
        if let Some(&character) = chars.get(position).filter(|_| !represented) {
            input.text.push(character);
            input
                .char_offsets
                .push(char_index_to_utf16_offset(source, position));
            scalar_index += 1;
        }
    }
    Some((input, controls))
}
