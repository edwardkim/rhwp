//! 실제 텍스트 경계에 속한 인라인 개체의 흐름 메트릭 목록.

use super::{
    flow_inline_baseline_hwp, inline_control_size_hwp, inline_control_spacing_height_hwp, Control,
    FlowInlineControl, LineBreakResult, Paragraph, ResolvedStyleSet,
};
use crate::renderer::{hwpunit_to_px, px_to_hwpunit};

pub(super) fn flow_inline_controls(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
) -> Vec<FlowInlineControl> {
    match flow_inline_controls_with_storage(para, styles, &crate::storage::Unbounded) {
        Ok(value) => value,
        Err(never) => match never {},
    }
}

/// 덧말 한 칸의 흐름 메트릭. 한컴 12.30 저장 줄 기준: 위 덧말은 줄 위에 덧말 높이(본말의
/// 절반)를 더하고 기준선을 그만큼 내린다(10pt 본말 → 높이 1500, 기준선 1350). 아래 덧말은
/// 기준선을 두고 줄을 덧말 높이의 약 0.816배 늘인다(10pt → 1408). 줄 간격도 이 높이를 따른다.
fn ruby_flow_size(
    para: &Paragraph,
    ruby: &crate::model::control::Ruby,
    utf16_position: u32,
    styles: &ResolvedStyleSet,
) -> Option<(i32, i32, i32)> {
    let host = para
        .char_shapes
        .iter()
        .filter(|shape| shape.start_pos <= utf16_position)
        .max_by_key(|shape| shape.start_pos)
        .or_else(|| para.char_shapes.first())
        .map_or(0, |shape| shape.char_shape_id);
    let composition = crate::renderer::char_overlap::Composition::from_ruby(ruby, styles, host)?;
    let dpi = styles.resolved_dpi.unwrap_or(crate::renderer::DEFAULT_DPI);
    let main_px = styles.char_styles.get(host as usize)?.font_size;
    let main = px_to_hwpunit(main_px, dpi);
    let ratio = if ruby.sz_ratio == 0 {
        0.5
    } else {
        f64::from(ruby.sz_ratio) / 100.0
    };
    let ruby_height = px_to_hwpunit(main_px * ratio, dpi);
    let width = px_to_hwpunit(composition.advance_px(), dpi);
    let baseline = px_to_hwpunit(hwpunit_to_px(main, dpi) * 0.85, dpi);
    Some(if ruby.pos_type == 1 {
        (
            width,
            main + (f64::from(ruby_height) * 0.816).round() as i32,
            baseline,
        )
    } else {
        (width, main + ruby_height, baseline + ruby_height)
    })
}

/// 글자 없이 덧말만 든 문단의 줄 메트릭 `(높이, 기준선)`. 글자 흐름(`flow_inline_controls`)은
/// 글이 있는 문단만 다루므로, 빈 문단 줄도 글자 있는 줄처럼 가장 큰 덧말 높이를 따른다.
pub(super) fn empty_text_ruby_row(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
) -> Option<(i32, i32)> {
    para.controls
        .iter()
        .zip(para.control_utf16_positions())
        .filter_map(|(control, utf16_position)| match control {
            Control::Ruby(ruby) => ruby_flow_size(para, ruby, utf16_position, styles)
                .map(|(_, height, baseline)| (height, baseline)),
            _ => None,
        })
        .max()
}

pub(crate) fn flow_inline_controls_with_storage<S: crate::storage::Storage>(
    para: &Paragraph,
    styles: &ResolvedStyleSet,
    storage: &S,
) -> Result<Vec<FlowInlineControl>, S::Error> {
    Ok({
        if para.controls.is_empty() {
            return Ok(Vec::new());
        }
        let text_len = para.text.chars().count();
        storage.collect(
            para.controls
                .iter()
                .zip(para.try_control_text_positions(storage)?)
                .zip(para.control_utf16_positions_with_storage(storage)?)
                .filter_map(|((control, char_position), utf16_position)| {
                    // 글자처럼 취급되는 표는 renderer가 control 위치를 기준으로 별도
                    // TextRun/Table 경계를 만든다. 보이지 않는 PARA_TEXT 위치의 다음
                    // 글자 폭에 표 전체 폭을 더하면 HML의 `abc + table + efg`처럼
                    // 기존 경계를 잃는다. #3211의 HWP oracle은 수식·그림 계열의
                    // 재조판 폭을 대상으로 하므로 표는 기존 control 배치 경로에 둔다.
                    if matches!(control, Control::Table(_)) {
                        return None;
                    }
                    if let Control::Ruby(ruby) = control {
                        let (width_hwp, height_hwp, baseline) =
                            ruby_flow_size(para, ruby, utf16_position, styles)?;
                        return (text_len > 0 && char_position <= text_len).then_some(
                            FlowInlineControl {
                                char_position,
                                utf16_position,
                                width_hwp,
                                height_hwp,
                                baseline_distance_hwp: Some(baseline),
                                spacing_height_hwp: Some(height_hwp),
                            },
                        );
                    }
                    let (width_hwp, height_hwp) = inline_control_size_hwp(control)?;
                    let baseline_distance_hwp = flow_inline_baseline_hwp(control, height_hwp);
                    // 마지막 scalar 뒤 개체도 마지막 물리 줄의 높이·기준선을 소유한다.
                    // 빈 텍스트의 퇴화 수식 배정은 기존 전용 경로에 남긴다.
                    (text_len > 0 && char_position <= text_len).then_some(FlowInlineControl {
                        char_position,
                        utf16_position,
                        width_hwp,
                        height_hwp,
                        baseline_distance_hwp,
                        spacing_height_hwp: inline_control_spacing_height_hwp(control),
                    })
                }),
        )?
    })
}

/// 종단 개체는 명시적 개행 뒤의 마지막 빈 줄이 소유한다.
/// 개행 앞 줄의 end_idx도 text_len과 같으므로 끝 좌표만으로 고르지 않는다.
pub(super) fn owns_line(
    control: &FlowInlineControl,
    line: &LineBreakResult,
    text_len: usize,
) -> bool {
    (line.start_idx..line.end_idx).contains(&control.char_position)
        || (!line.has_line_break && line.end_idx == text_len && control.char_position == text_len)
}
