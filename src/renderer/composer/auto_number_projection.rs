//! note 본문 밖의 자동 번호를 해당 control의 원문 위치에 표시한다.

use super::{insert_control_text_run, ComposedParagraph, ComposedTextRun, SpaceMetric};
use crate::model::{
    control::{AutoNumber, AutoNumberType, Control},
    paragraph::Paragraph,
};

fn label(number: &AutoNumber) -> String {
    let mut text = String::new();
    if number.prefix_char != '\0' {
        text.push(number.prefix_char);
    }
    text.push_str(&crate::renderer::format_number(
        number.assigned_number,
        crate::renderer::NumberFormat::from_hwp_format(number.format),
    ));
    if number.suffix_char != '\0' {
        text.push(number.suffix_char);
    }
    text
}

// Extended control의 원문 placeholder 한 scalar를 표시 문자열로 투영한다.
// 원문 길이와 뒤 글자·TAC의 source 위치는 바뀌지 않는다.
fn project_placeholder(
    composed: &mut ComposedParagraph,
    position: usize,
    number: &AutoNumber,
    style: u32,
) -> bool {
    for line in &mut composed.lines {
        let mut start = line.char_start;
        for index in 0..line.runs.len() {
            let count = line.runs[index].source_scalar_count();
            if position >= start && position < start + count {
                let run = &line.runs[index];
                let offset = position - start;
                let chars: Vec<_> = run.text.chars().collect();
                if !matches!(chars.get(offset), Some(' ' | '\u{0012}' | '\u{FFFC}')) {
                    return false;
                }
                let mut parts = Vec::new();
                if offset > 0 {
                    let mut before = run.clone();
                    before.text = chars[..offset].iter().collect();
                    parts.push(before);
                }
                let mut marker = run.clone();
                marker.text = chars[offset].to_string();
                marker.display_text = Some(label(number));
                marker.char_style_id = style;
                marker.footnote_marker = number.superscript.then_some(number.assigned_number);
                parts.push(marker);
                if offset + 1 < chars.len() {
                    let mut after = run.clone();
                    after.text = chars[offset + 1..].iter().collect();
                    parts.push(after);
                }
                line.runs.splice(index..=index, parts);
                return true;
            }
            start += count;
        }
    }
    false
}

pub(super) fn project(composed: &mut ComposedParagraph, para: &Paragraph) {
    let positions = para.control_text_positions();
    let utf16_positions = para.control_utf16_positions();
    for (index, control) in para.controls.iter().enumerate() {
        let Control::AutoNumber(number) = control else {
            continue;
        };
        if !matches!(
            number.number_type,
            AutoNumberType::Footnote | AutoNumberType::Endnote
        ) {
            continue;
        }
        let Some(&position) = positions.get(index) else {
            continue;
        };
        let utf16 = utf16_positions.get(index).copied().unwrap_or(0);
        let style = super::find_active_char_shape(&para.char_shapes, utf16);
        if para.char_offsets.get(position).copied() == Some(utf16)
            && project_placeholder(composed, position, number, style)
        {
            continue;
        }
        // 원문 scalar를 갖지 않는 형식의 control만 별도 표시 런을 삽입한다.
        let run = ComposedTextRun {
            text: label(number),
            char_style_id: style,
            footnote_marker: number.superscript.then_some(number.assigned_number),
            inserted_control_text: true,
            space_metric: SpaceMetric::Stored,
            ..Default::default()
        };
        for run in super::split_runs_by_lang(vec![run]) {
            insert_control_text_run(composed, position, run);
        }
    }
}
