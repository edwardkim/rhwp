//! 실제 텍스트 경계에 속한 인라인 개체의 흐름 메트릭 목록.

use super::{
    flow_inline_baseline_hwp, inline_control_size_hwp, Control, FlowInlineControl, LineBreakResult,
    Paragraph,
};

pub(super) fn flow_inline_controls(para: &Paragraph) -> Vec<FlowInlineControl> {
    let text_len = para.text.chars().count();
    para.controls
        .iter()
        .zip(para.control_text_positions())
        .zip(para.control_utf16_positions())
        .filter_map(|((control, char_position), utf16_position)| {
            // 글자처럼 취급되는 표는 renderer가 control 위치를 기준으로 별도
            // TextRun/Table 경계를 만든다. 보이지 않는 PARA_TEXT 위치의 다음
            // 글자 폭에 표 전체 폭을 더하면 HML의 `abc + table + efg`처럼
            // 기존 경계를 잃는다. #3211의 HWP oracle은 수식·그림 계열의
            // 재조판 폭을 대상으로 하므로 표는 기존 control 배치 경로에 둔다.
            if matches!(control, Control::Table(_)) {
                return None;
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
            })
        })
        .collect()
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
