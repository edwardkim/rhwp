//! 현재 프레임에서 이어 붙일 수 있는 저장된 마지막 자동 줄바꿈.

use super::{
    line_breaking, utf16_range_to_text_range, ComposedParagraph, ResolvedStyleSet, SpaceMetric,
};
use crate::model::paragraph::{LineSeg, Paragraph};
use crate::renderer::layout_frame::{LayoutFrame, ParagraphBox};

/// 원문 문자 밖의 목록 마커는 현재 해소한 기하로 줄 구성을 갱신한다.
/// 본문과 셀이 같은 가로 상자 인증과 마커 소유 판정을 소비한다.
pub(crate) fn projects_current_marker_geometry(
    para: &Paragraph,
    box_: &ParagraphBox,
    styles: &ResolvedStyleSet,
) -> bool {
    stored_horizontal_box_matches(para, box_)
        && crate::renderer::layout::list_marker_geometry(para, styles)
            .is_some_and(|marker| marker.marker_visible)
}

/// 실제 번호·글머리표를 담지 못하는 저장 줄은 현재 공통 채움으로 갱신한다.
/// 줄 수가 그대로여도 마커의 크기는 본문 흐름과 후속 원점에 참여한다.
pub(crate) fn project_current_marker_height_rows(
    para: &Paragraph,
    box_: &ParagraphBox,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<(Vec<LineSeg>, Vec<(u32, SpaceMetric)>)> {
    if !projects_current_marker_geometry(para, box_, styles)
        || !crate::renderer::cell_paragraph_has_only_single_column_metadata(para)
        || para.text.chars().any(char::is_control)
        || para.stored_text_partition_is_dirty()
        || super::stored_row_fragments::crosses_physical_frame(para)
        || para
            .line_segs
            .iter()
            .any(|line| line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0)
    {
        return None;
    }
    let marker = crate::renderer::layout::list_marker_geometry(para, styles)?;
    let minimum_height = (marker.line_font_size_px * 7200.0 / dpi).round() as i32;
    let first = para.line_segs.first()?;
    if first.line_height >= minimum_height {
        return None;
    }
    let mut frame = box_.frame(first.vertical_pos);
    let rows = line_breaking::layout_paragraph_in_frame(para, &mut frame, styles, dpi)?;
    rows.first()
        .is_some_and(|row| row.line_height >= minimum_height)
        .then(|| (rows, frame.project_space_metrics()))
}

fn stored_horizontal_box_matches(para: &Paragraph, box_: &ParagraphBox) -> bool {
    let extent = box_.effective();
    // 셀의 문단 여백 경로는 실제 폭을 유지하고, 저장 줄의 오른쪽 끝은
    // content_width_px와 같은 4 HWPUNIT 격자에 놓인다. 격자에서 접힌
    // 잔여를 다른 상자로 오인하지 않으며 실제 채움 폭은 바꾸지 않는다.
    let stored_right = extent.end
        - extent
            .end
            .rem_euclid(crate::renderer::layout_frame::COLUMN_WIDTH_QUANTUM_HWP);
    box_.is_usable()
        && !para.line_segs.is_empty()
        && para.line_segs.iter().all(|line| {
            line.is_first_segment()
                && line.is_last_segment()
                && line.column_start == extent.start
                && (line.segment_width == extent.end - extent.start
                    || line.segment_width == stored_right - extent.start)
        })
}

/// 본문의 해소 가능한 여백 원점보다 앞에 놓인 저장 줄은 같은 상자가 아니다.
/// 목록의 내어쓰기나 유효한 저장 원점을 버리지 않고, 같은 폭의 모순된
/// 상자에서 현재 채움이 실제 문자 경계를 옮기는 경우만 다시 발행한다.
pub(crate) fn project_current_body_box_rows(
    para: &Paragraph,
    box_: &ParagraphBox,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<(Vec<LineSeg>, Vec<(u32, SpaceMetric)>)> {
    let style = styles.para_styles.get(para.para_shape_id as usize)?;
    let extent = box_.effective();
    if style.head_type != crate::model::style::HeadType::None
        || !box_.is_usable()
        || extent.start <= 0
        || para.line_segs.len() < 2
        || !crate::renderer::cell_paragraph_has_only_single_column_metadata(para)
        || para.text.chars().any(char::is_control)
        || !para.line_segs.iter().all(|line| {
            line.is_first_segment()
                && line.is_last_segment()
                && line.column_start < extent.start
                && line.segment_width == extent.end - extent.start
        })
        || para.line_segs.windows(2).any(|pair| {
            pair[1].vertical_pos <= pair[0].vertical_pos
                || pair[1].is_first_line_of_page()
                || pair[1].is_first_line_of_column()
        })
    {
        return None;
    }
    project_stored_rows(
        para,
        &box_.frame(para.line_segs[0].vertical_pos),
        styles,
        dpi,
        false,
        true,
    )
}

/// 원본 TTF로 잰 같은 상자에서 저장 경계가 본문을 먼저 끊었는지 확인한다.
/// 줄 수 감소나 글꼴 이름만으로 캐시를 버리지 않는다. 대체 선언과 달리 원본
/// 프로그램을 실제 측정에 사용하고, 새 채움이 그 경계의 글자를 앞줄에 보존해야 한다.
pub(crate) fn project_current_source_program_rows(
    para: &Paragraph,
    box_: &ParagraphBox,
    styles: &ResolvedStyleSet,
    dpi: f64,
) -> Option<(Vec<LineSeg>, Vec<(u32, SpaceMetric)>)> {
    if !stored_horizontal_box_matches(para, box_)
        || !crate::renderer::cell_paragraph_has_only_single_column_metadata(para)
        || para.text.chars().any(char::is_control)
    {
        return None;
    }
    let composed = super::compose_paragraph_in_context(para, styles);
    let mut has_text = false;
    let mut fallback_styles = styles.clone();
    fallback_styles.source_program_projection_slots.clear();
    for run in composed.lines.iter().flat_map(|line| &line.runs) {
        if run.text.chars().all(char::is_whitespace) {
            continue;
        }
        has_text = true;
        let Some(fallback) = styles
            .source_program_projection_slots
            .get(&(run.char_style_id, run.lang_index))
        else {
            return None;
        };
        if run.inserted_control_text {
            return None;
        }
        *fallback_styles
            .char_styles
            .get_mut(run.char_style_id as usize)? = fallback.clone();
    }
    if !has_text {
        return None;
    }
    let first = para.line_segs.first()?.vertical_pos;
    let frame = box_.frame(first);
    // 대체 선언이 있다는 사실이나 현재 글자가 한 자 더 들어간다는 사실은
    // 저장 줄이 다른 프로그램의 캐시라는 증거가 아니다. 문서가 지정한 대체
    // 프로그램이 같은 상자에서 저장 줄 전체를 재현하는지 먼저 확인한다.
    let fallback_rows =
        line_breaking::layout_paragraph_in_frame(para, &mut frame.clone(), &fallback_styles, dpi)?;
    if fallback_rows.len() != para.line_segs.len()
        || fallback_rows
            .iter()
            .enumerate()
            .any(|(index, row)| row.text_start != para.line_seg_text_start(index))
    {
        return None;
    }
    project_stored_rows(para, &frame, styles, dpi, false, true)
}

/// 앞선 물리 경계가 없는 마지막 자동 어절의 캐시 원점 되감김인가.
/// 명시적 쪽/단과 다른 가로 상자는 실제 프레임 소유자로 남긴다.
pub(super) fn has_unmarked_final_word_rewind(para: &Paragraph) -> bool {
    let Some((last, prefix)) = para.line_segs.split_last() else {
        return false;
    };
    let Some(previous) = prefix.last() else {
        return false;
    };
    let chars: Vec<_> = para.text.chars().collect();
    let (start, _) = utf16_range_to_text_range(
        &para.char_offsets,
        para.line_seg_text_start(para.line_segs.len() - 1),
        u32::MAX,
        chars.len(),
    );
    start > 0
        && start < chars.len()
        && chars[start - 1].is_alphanumeric()
        && chars[start].is_alphanumeric()
        && !last.is_first_line_of_page()
        && !last.is_first_line_of_column()
        && last.vertical_pos >= 0
        && last.vertical_pos < previous.vertical_pos
        && last.column_start == previous.column_start
        && last.segment_width > 0
        && last.segment_width == previous.segment_width
        && !prefix.windows(2).any(|pair| {
            pair[1].is_first_line_of_page()
                || pair[1].is_first_line_of_column()
                || pair[1].vertical_pos < pair[0].vertical_pos
        })
}

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
    project_stored_rows(para, frame, styles, dpi, supported_inline_picture, false)
}

fn project_stored_rows(
    para: &Paragraph,
    frame: &LayoutFrame,
    styles: &ResolvedStyleSet,
    dpi: f64,
    supported_inline_picture: bool,
    refresh_physical_cache: bool,
) -> Option<(Vec<LineSeg>, Vec<(u32, SpaceMetric)>)> {
    if let Some(projected) = super::stored_marker_frames::project(para, frame, styles, dpi) {
        return Some(projected);
    }
    if para.line_segs.len() < 2
        || (!para.controls.is_empty()
            && !supported_inline_picture
            && !(refresh_physical_cache
                && crate::renderer::cell_paragraph_has_only_single_column_metadata(para)))
        || para.text.contains('\n')
        || para.stored_text_partition_is_dirty()
        || (!refresh_physical_cache
            && super::stored_row_fragments::crosses_physical_frame(para)
            && !has_unmarked_final_word_rewind(para))
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
    if !refresh_physical_cache
        && (last.is_first_line_of_page()
        || last.is_first_line_of_column()
        || last.vertical_pos == previous.vertical_pos
        // 같은 가로 상자의 저장 캐시 되감김은 명시적 쪽 경계와 구분한다.
        || (last.vertical_pos < previous.vertical_pos
            && (last.vertical_pos < 0
                || last.column_start != previous.column_start
                || last.segment_width <= 0
                || last.segment_width != previous.segment_width)))
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
    if refresh_physical_cache {
        let cached = super::compose_paragraph_in_context(para, styles);
        let chars: Vec<_> = para.text.chars().collect();
        let source_index = |position| {
            utf16_range_to_text_range(&para.char_offsets, position, u32::MAX, chars.len()).0
        };
        let fresh_starts: Vec<_> = rows
            .iter()
            .map(|line| source_index(line.text_start))
            .collect();
        // 다른 프로그램으로 저장한 캐시는 어절 안뿐 아니라 어절 사이에서도
        // 너무 일찍 끝날 수 있다. 다음 저장 줄의 실제 첫 글자가 새 채움의
        // 앞줄에 들어가는지 확인한다. 후행 공백만 이동한 차이는 근거가 아니다.
        // 문자 줄을 다시 발행하면 캐시의 쪽/단 플래그도 다시 사용하지 않는다.
        // 명시적 본문 개행은 이 경로에 들어오기 전에 보존한다.
        let cached_content_boundary_moves = (1..para.line_segs.len()).any(|index| {
            let start = source_index(para.line_seg_text_start(index));
            let next_ink = chars
                .iter()
                .enumerate()
                .skip(start)
                .find(|(_, ch)| !ch.is_whitespace())
                .map(|(index, _)| index);
            start > 0
                && next_ink.is_some_and(|next| {
                    fresh_starts.partition_point(|&fresh| fresh <= next) < index + 1
                })
                && !cached
                    .lines
                    .get(index - 1)
                    .is_some_and(|line| line.has_line_break)
        });
        return cached_content_boundary_moves.then(|| (rows, probe.project_space_metrics()));
    }
    // 마지막 자동 어절의 가짜 프레임 원점은 줄 수가 같아도 복원한다.
    // 일반 저장 줄은 같은 줄 수라는 이유만으로 다시 구성하지 않는다.
    (rows.len() < para.line_segs.len()
        || (rows.len() == para.line_segs.len() && has_unmarked_final_word_rewind(para)))
    .then(|| (rows, probe.project_space_metrics()))
}
