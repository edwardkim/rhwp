//! 자동 번호를 해당 control의 원문 자리표와 같은 표시 런에 투영한다.

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
                    before.display_text = run.display_for_model_range(0, offset);
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
                    after.display_text = run.display_for_model_range(offset + 1, chars.len());
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
    project_where(composed, para, |kind| {
        // 본문 개체 번호도 모델에서 할당한 현재 번호를 자기 자리표에 표시한다.
        // 쪽·총쪽수는 페이지 계획이 완성된 후 실제 쪽을 아는 소비자가 결정한다.
        !matches!(kind, AutoNumberType::Page | AutoNumberType::TotalPage)
    });
}

/// 캡션은 일반 본문 구성의 project를 거치지 않으므로 자체 구성 직후 투영한다.
pub(crate) fn project_caption(composed: &mut ComposedParagraph, para: &Paragraph) {
    project_where(composed, para, |kind| {
        !matches!(kind, AutoNumberType::Footnote | AutoNumberType::Endnote)
    });
}

fn project_where(
    composed: &mut ComposedParagraph,
    para: &Paragraph,
    owns: impl Fn(AutoNumberType) -> bool,
) {
    let sources = para.control_source_positions();
    for (index, control) in para.controls.iter().enumerate() {
        let Control::AutoNumber(number) = control else {
            continue;
        };
        if !owns(number.number_type) {
            continue;
        }
        let Some(source) = sources.get(index) else {
            continue;
        };
        let position = source.char_index;
        let utf16 = source.utf16;
        let style = super::find_active_char_shape(&para.char_shapes, utf16);
        if source.owns_char && project_placeholder(composed, position, number, style) {
            continue;
        }
        // 원문 scalar를 갖지 않는 형식의 control만 별도 표시 런을 삽입한다.
        let run = ComposedTextRun {
            text: label(number),
            char_style_id: style,
            footnote_marker: number.superscript.then_some(number.assigned_number),
            inserted_control_text: true,
            space_metric: SpaceMetric::Stored,
            regenerated_font_metrics: super::text_metric_origin::regenerated_row(para, 0),
            literal_figure_space: false,
            ..Default::default()
        };
        for run in super::split_runs_by_lang(vec![run]) {
            insert_control_text_run(composed, position, run);
        }
    }
}
