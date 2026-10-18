//! 현재 프레임에서 이어 붙일 수 있는 저장된 마지막 자동 줄바꿈.

use super::{
    line_breaking, utf16_range_to_text_range, ComposedParagraph, ResolvedStyleSet, SpaceMetric,
};
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::renderer::layout_frame::LayoutFrame;

/// 저장 run과 현재 채움이 서로 다른 공백 전진을 사용하면 줄 분할도 재검토해야 한다.
/// 같은 공백 메트릭의 정상 본문은 더 적은 줄로 들어간다는 이유만으로 무효화하지 않는다.
pub(super) fn regenerated_spaces_change_metrics(
    composed: &ComposedParagraph,
    styles: &ResolvedStyleSet,
    metric: SpaceMetric,
) -> bool {
    composed
        .lines
        .iter()
        .flat_map(|line| &line.runs)
        .any(|run| {
            if !run.text.contains(' ') {
                return false;
            }
            let stored = run.text_style(styles);
            let mut regenerated = stored.clone();
            metric.apply_to_style(&mut regenerated);
            let advance = |style: &crate::renderer::TextStyle| {
                crate::renderer::layout::estimate_text_width_unrounded(" ", style)
            };
            (advance(&stored) - advance(&regenerated)).abs() > f64::EPSILON
        })
}

/// 저장된 마지막 줄이 앞줄의 영숫자·한글 어절 중간에서 시작하는지 확인한다.
/// 명시적 개행·실제 쪽/단 경계·어울림 상자의 경계는 재조판으로 합치지 않는다.
/// 완결된 마지막 어절은 그대로 보존하며, 같은 번호 마커·스타일·가용 폭을 쓰는
/// 프레임이 더 적은 줄로 수용할 때만 낡은 자동 줄 경계를 재검토한다.
pub(super) fn can_rejoin_stored_word_tail(
    para: &Paragraph,
    frame: &LayoutFrame,
    styles: &ResolvedStyleSet,
    dpi: f64,
    supported_inline_picture: bool,
) -> bool {
    if para.line_segs.len() < 2 {
        return false;
    }
    let chars: Vec<_> = para.text.chars().collect();
    let (start, _) = utf16_range_to_text_range(
        &para.char_offsets,
        para.line_seg_text_start(para.line_segs.len() - 1),
        u32::MAX,
        chars.len(),
    );
    // 어절을 자른 경계인지는 앞줄 끝과 마지막 줄 첫 글자로 판단한다.
    // 같은 줄 뒤의 다른 어절·문장부호는 이 경계의 소속을 바꾸지 않는다.
    // 실제 줄 수 감소는 그 뒤 내용까지 포함한 동일 프레임 조판으로 확인한다.
    if start == 0
        || start >= chars.len()
        || !chars[start - 1].is_alphanumeric()
        || !chars[start].is_alphanumeric()
    {
        return false;
    }
    project_stored_soft_wraps(para, frame, styles, dpi, supported_inline_picture).is_some()
}

/// 같은 frame에서 자동 줄을 다시 구성한다. 줄 수 감소는 캐시 무효화 근거가 아니다.
/// 호출자는 현재 마커 기하·공백 메트릭 등 갱신 이유와 후속 축 소유를 먼저 확인한다.
pub(crate) fn project_stored_soft_wraps(
    para: &Paragraph,
    frame: &LayoutFrame,
    styles: &ResolvedStyleSet,
    dpi: f64,
    supported_inline_picture: bool,
) -> Option<(Vec<LineSeg>, Vec<(u32, SpaceMetric)>)> {
    if para.line_segs.len() < 2
        || (!para.controls.is_empty() && !supported_inline_picture)
        || para.text.contains('\n')
        || para.stored_text_partition_is_dirty()
        || super::stored_row_fragments::crosses_physical_frame(para)
        || frame.models_exclusions()
        || para
            .line_segs
            .iter()
            .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
    {
        return None;
    }
    let last = para.line_segs.last()?;
    let previous = &para.line_segs[para.line_segs.len() - 2];
    if last.is_first_line_of_page()
        || last.is_first_line_of_column()
        || last.vertical_pos == previous.vertical_pos
        // 같은 가로 상자의 저장 캐시 되감김은 명시적 쪽 경계와 구분한다.
        || (last.vertical_pos < previous.vertical_pos
            && (last.vertical_pos < 0
                || last.column_start != previous.column_start
                || last.segment_width <= 0
                || last.segment_width != previous.segment_width))
    {
        return None;
    }
    let count = para.text.chars().count();
    let (last_start, _) = utf16_range_to_text_range(
        &para.char_offsets,
        para.line_seg_text_start(para.line_segs.len() - 1),
        u32::MAX,
        count,
    );
    // 빈 문단·마지막 빈 예약 행의 물리 점유를 자동 어절 줄로 바꾸지 않는다.
    if last_start == 0
        || last_start >= count
        || !para
            .text
            .chars()
            .skip(last_start)
            .any(|ch| !ch.is_whitespace())
    {
        return None;
    }
    let mut probe = frame.clone();
    // 채움은 새 줄을 만들지만 원본의 줄별 들여쓰기 기록도 소비한다. 저장 줄을
    // 비워 입력의 근거까지 없애면 명시적으로 꺼진 들여쓰기가 다시 적용된다.
    let rows = line_breaking::layout_paragraph_in_frame(para, &mut probe, styles, dpi)?;
    (rows.len() < para.line_segs.len()).then(|| (rows, probe.project_space_metrics()))
}
