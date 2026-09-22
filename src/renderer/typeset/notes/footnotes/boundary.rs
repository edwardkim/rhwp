//! 저장 각주 줄의 소유·분할·본문 충돌 Query. 입력의 기존 유효성 조건을 유지한다.

use crate::renderer::typeset::notes::footnotes::measure::composed_footnote_content_height;
use crate::renderer::typeset::{
    compose_paragraph, hwpunit_to_px, is_synthetic_line_seg, para_has_visible_text,
    ColumnBreakType, Control, Footnote, FootnoteFragment, FootnoteSource, FormattedParagraph,
    PageItem, Paragraph, TypesetState,
};

/// native HWP5의 두 줄짜리 단일 각주를 물리 페이지 경계에서 연속 fragment로 나눈다.
///
/// 한컴은 본문 `LINE_SEG` reset 앞의 첫 줄은 해당 page의 separator 아래에 두고,
/// 둘째 줄만 다음 page의 bottom footnote lane에 둔다. 일반 각주는 원자적으로 유지한다.
pub(in crate::renderer::typeset) fn native_hwp5_two_line_footnote_fragments(
    footnote: &Footnote,
) -> Option<(FootnoteFragment, FootnoteFragment)> {
    if footnote.paragraphs.len() != 1 {
        return None;
    }
    let composed = crate::renderer::composer::compose_paragraph(&footnote.paragraphs[0]);
    (composed.lines.len() == 2).then_some((
        FootnoteFragment {
            start_line: 0,
            end_line: 1,
            draw_separator: true,
            draw_number: true,
        },
        FootnoteFragment {
            start_line: 1,
            end_line: 2,
            draw_separator: false,
            draw_number: false,
        },
    ))
}

pub(in crate::renderer::typeset) fn native_hwp5_footnote_fragment_height(
    footnote: &Footnote,
    fragment: FootnoteFragment,
    dpi: f64,
) -> f64 {
    let Some(paragraph) = footnote.paragraphs.first() else {
        return 0.0;
    };
    let composed = crate::renderer::composer::compose_paragraph(paragraph);
    composed.lines
        [fragment.start_line.min(composed.lines.len())..fragment.end_line.min(composed.lines.len())]
        .iter()
        .map(|line| hwpunit_to_px(line.line_height, dpi))
        .sum()
}

/// native HWP5 각주의 저장 line reset을 physical footnote fragment로 보존한다.
///
/// p728의 note 77처럼 셀 marker는 첫 table fragment에 있고 각주 문단의 세 번째
/// stored line이 `vpos=0`으로 다시 시작할 수 있다. 한컴 PDF는 reset 앞 두 줄을
/// marker page에, 뒤 두 줄을 다음 table fragment page에 둔다. queue가 note 전체를
/// 원자적으로 넘기면 첫 page에서 note number가 사라지고 다음 page body와 각주가
/// 겹친다. 같은 저장 계약은 본문 marker가 물리 page 경계를 넘는 p129의 note 176에도
/// 적용된다. generic wrap/capacity 추측이 아니라 stored line count와 reset이 composer
/// line count에 정확히 대응하는 경우만 인정한다.
pub(in crate::renderer::typeset) fn native_hwp5_footnote_reset_fragments(
    footnote: &Footnote,
    dpi: f64,
) -> Option<NativeHwp5FootnoteFragmentSplit> {
    let mut flat_lines = Vec::new();
    let mut split_line = None;
    let mut force_next_page = false;
    for paragraph in &footnote.paragraphs {
        let composed = crate::renderer::composer::compose_paragraph(paragraph);
        // source LINE_SEG가 composer line과 일대일로 남아 있는 경우만 reset을
        // physical owner 신호로 쓴다. 재flow/합성 line은 capacity heuristic으로
        // 오인하지 않고 기존 원자 queue를 유지한다.
        if composed.lines.is_empty() || composed.lines.len() != paragraph.line_segs.len() {
            return None;
        }
        let base = flat_lines.len();
        for (index, line) in composed.lines.iter().enumerate() {
            flat_lines.push((
                hwpunit_to_px(line.line_height, dpi),
                hwpunit_to_px(line.line_spacing, dpi),
            ));
            if index == 0 {
                continue;
            }
            let previous = &paragraph.line_segs[index - 1];
            let next = &paragraph.line_segs[index];
            let regular_reset = previous.vertical_pos > 0 && next.vertical_pos == 0;
            let repeated_page_top =
                index == 1 && previous.vertical_pos == 0 && next.vertical_pos == 0;
            if !is_synthetic_line_seg(previous)
                && !is_synthetic_line_seg(next)
                && (regular_reset || repeated_page_top)
            {
                if split_line.replace(base + index).is_some() {
                    return None;
                }
                force_next_page = repeated_page_top;
            }
        }
    }
    let line_count = flat_lines.len();
    let split_line = split_line?;
    if split_line == 0 || split_line >= line_count {
        return None;
    }

    let prefix = FootnoteFragment {
        start_line: 0,
        end_line: split_line,
        draw_separator: true,
        draw_number: true,
    };
    let suffix = FootnoteFragment {
        start_line: split_line,
        end_line: line_count,
        draw_separator: false,
        draw_number: false,
    };
    let fragment_height = |fragment: FootnoteFragment| {
        flat_lines[fragment.start_line..fragment.end_line]
            .iter()
            .enumerate()
            .map(|(index, (line_height, line_spacing))| {
                *line_height
                    + if index + 1 < fragment.end_line - fragment.start_line {
                        *line_spacing
                    } else {
                        0.0
                    }
            })
            .sum()
    };
    Some(NativeHwp5FootnoteFragmentSplit {
        prefix_height: fragment_height(prefix),
        suffix_height: fragment_height(suffix),
        prefix,
        suffix,
        force_next_page,
    })
}

/// native HWP5 본문 각주 marker 뒤에서 현재 쪽의 `PartialParagraph`가 시작하는
/// stored reset을 찾는다.
///
/// control 순회는 문단 전체의 pagination이 끝난 뒤 실행된다. 따라서 marker가 이전
/// 완료 page에 있고 같은 문단의 reset tail이 current page에 있으면, 현재 item은
/// `start_line > marker_line`인 PartialParagraph다. 임의의 줄 되감김을 각주 owner로
/// 오인하지 않도록 단일 Footnote control, 정확한 `vpos > 0 -> 0` 경계로 한정한다.
pub(in crate::renderer::typeset) fn native_hwp5_body_footnote_tail_reset(
    st: &TypesetState,
    para_idx: usize,
    para: &Paragraph,
    ctrl_idx: usize,
) -> Option<(usize, usize)> {
    if !st.profile.hwp5_stored_pagination_layout()
        || st.col_count != 1
        || para.controls.len() != 1
        || !matches!(para.controls.get(ctrl_idx), Some(Control::Footnote(_)))
        || !para_has_visible_text(para)
    {
        return None;
    }

    let control_pos = *para.control_text_positions().get(ctrl_idx)?;
    let marker_line = para
        .line_segs
        .iter()
        .enumerate()
        .rev()
        .find(|(_, line)| !is_synthetic_line_seg(line) && line.text_start as usize <= control_pos)?
        .0;
    let reset_line = st.current_items.iter().rev().find_map(|item| match item {
        PageItem::PartialParagraph {
            para_index,
            start_line,
            ..
        } if *para_index == para_idx && *start_line > marker_line => Some(*start_line),
        _ => None,
    })?;
    let previous = para.line_segs.get(reset_line.checked_sub(1)?)?;
    let next = para.line_segs.get(reset_line)?;
    (!is_synthetic_line_seg(previous)
        && !is_synthetic_line_seg(next)
        && previous.vertical_pos > 0
        && next.vertical_pos == 0)
        .then_some((marker_line, reset_line))
}

pub(in crate::renderer::typeset) fn native_hwp5_first_footnote_overlap_break_line(
    st: &TypesetState,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    dpi: f64,
) -> Option<NativeHwp5FootnoteBreak> {
    let line_count = fmt.line_heights.len();
    if !st.profile.hwp5_stored_pagination_layout()
        || !st.is_first_footnote_on_page
        || st.current_footnote_height > 0.0
        || line_count < 2
        || para.line_segs.len() < line_count
        || !para_has_visible_text(para)
    {
        return None;
    }

    let footnotes: Vec<&Footnote> = para
        .controls
        .iter()
        .filter_map(|control| match control {
            Control::Footnote(footnote) => Some(footnote.as_ref()),
            _ => None,
        })
        .collect();
    if footnotes.len() != 1 || footnotes.len() != para.controls.len() {
        return None;
    }
    let footnote_control_index = para
        .controls
        .iter()
        .position(|control| matches!(control, Control::Footnote(_)))?;
    let footnote_control_pos = *para.control_text_positions().get(footnote_control_index)?;

    let two_line_footnote_fragment = native_hwp5_two_line_footnote_fragments(footnotes[0]);
    let mut footnote_height = st.footnote_separator_overhead;
    for (para_idx, note_para) in footnotes[0].paragraphs.iter().enumerate() {
        let composed = crate::renderer::composer::compose_paragraph(note_para);
        if composed.lines.is_empty() {
            footnote_height += hwpunit_to_px(400, dpi);
            continue;
        }
        let note_last_para = para_idx + 1 == footnotes[0].paragraphs.len();
        for (line_idx, line) in composed.lines.iter().enumerate() {
            footnote_height += hwpunit_to_px(line.line_height, dpi);
            if !(note_last_para && line_idx + 1 == composed.lines.len()) {
                footnote_height += hwpunit_to_px(line.line_spacing, dpi);
            }
        }
    }

    let footnote_top = st.layout.body_area.height - footnote_height;
    let projected_reclaim = st.footer_band_reclaim_for_height(footnote_height);
    let projected_available = (st.base_available_height()
        - (footnote_height - projected_reclaim).max(0.0)
        - st.footnote_safety_margin
        - st.current_zone_y_offset
        - st.current_bottom_fixed_exclusion)
        .max(0.0);
    let page_vpos_base = st.vpos_page_base.or(st.vpos_lazy_base).unwrap_or(0);
    para.line_segs[..line_count]
        .windows(2)
        .enumerate()
        .find_map(|(prev_idx, pair)| {
            let (prev, next) = (&pair[0], &pair[1]);
            if is_synthetic_line_seg(prev)
                || is_synthetic_line_seg(next)
                || prev.vertical_pos <= page_vpos_base
                || next.vertical_pos != 0
            {
                return None;
            }

            let visible_bottom =
                hwpunit_to_px(prev.vertical_pos - page_vpos_base + prev.line_height, dpi);
            let trailing_bottom = hwpunit_to_px(
                prev.vertical_pos - page_vpos_base
                    + prev.line_height.saturating_add(prev.line_spacing),
                dpi,
            );
            let trailing_spacing_only_overlap =
                visible_bottom <= footnote_top + 0.5 && trailing_bottom > footnote_top + 0.5;
            // 두 줄 각주는 첫 줄만 현재 page에 남겨야 한다. 저장 본문 줄 자체가 full-note
            // top을 넘고(단순 trailing-spacing 침범과 구별), 각주 marker가 그 reset 직전
            // 본문 줄 안에 있을 때만 두 번째 각주 줄을 다음 page로 보낸다. full note를
            // 원자 배치하면 p31처럼 separator·본문이 겹친다.
            let first_line_footnote_fragment_overlap = two_line_footnote_fragment.is_some()
                && visible_bottom > footnote_top + 0.5
                && trailing_bottom > footnote_top + 0.5
                && (prev.text_start as usize..next.text_start as usize)
                    .contains(&footnote_control_pos);
            // 첫 각주의 marker가 reset보다 앞선 prefix에 있고, renderer와 같은 flow
            // advance에서 reset 직전 줄만 projected FootnoteArea+safety 안에 들어가면
            // 저장 reset은 실제 physical page 경계다. marker를 reset 직전 한 줄로
            // 제한하면 정책연구 p120처럼 marker(line 1)와 reset(line 4) 사이에 본문이
            // 있는 정상 형상을 놓친다.
            let marker_before_reset = footnote_control_pos < next.text_start as usize;
            let flow_prev_top =
                st.current_height + fmt.spacing_before + fmt.line_advances_sum(0..prev_idx);
            let flow_prev_bottom = flow_prev_top + fmt.line_heights[prev_idx];
            let flow_next_bottom =
                flow_prev_top + fmt.line_advance(prev_idx) + fmt.line_heights[prev_idx + 1];
            let projected_boundary_overlap = marker_before_reset
                && flow_prev_bottom <= projected_available + 0.5
                && flow_next_bottom > projected_available + 0.5;
            (trailing_spacing_only_overlap
                || first_line_footnote_fragment_overlap
                || projected_boundary_overlap)
                .then_some(NativeHwp5FootnoteBreak {
                    body_break_line: prev_idx + 1,
                    split_footnote: first_line_footnote_fragment_overlap,
                })
        })
}

/// native HWP5에서 문단 마지막 marker의 단일 각주가 다음 paragraph의 저장 reset을
/// 소유하는 매우 좁은 형상을 판정한다.
///
/// 보통 inline Footnote는 marker가 놓인 현재 physical page에 등록한다. 그러나 이
/// 형상에서는 본문 marker가 현재 page의 마지막 줄에 남고, **다음** paragraph가
/// `vpos=0`으로 새 page를 시작한다. 각주를 현재 page에 뒤늦게 예약하면 이미 배치한
/// 본문 하단이 FootnoteArea를 침범한다. 본문을 되감지 않고 note registration만
/// 다음 page로 넘겨 한컴의 physical owner를 보존한다.
///
/// 같은 paragraph 안 reset, 두 줄 각주 fragment, 기존 각주가 있는 page의 tail은
/// 각각 기존 보정 경로가 담당한다. 따라서 아래 조건을 모두 만족하는 경우에만 쓴다.
pub(in crate::renderer::typeset) fn native_hwp5_final_marker_footnote_uses_next_reset_page(
    st: &TypesetState,
    para_idx: usize,
    para: &Paragraph,
    paragraphs: &[Paragraph],
    ctrl_idx: usize,
    footnote: &Footnote,
    footnote_height: f64,
) -> bool {
    if !st.profile.hwp5_stored_pagination_layout()
        || st.col_count != 1
        || para.controls.len() != 1
        || !matches!(para.controls.get(ctrl_idx), Some(Control::Footnote(_)))
        || !para_has_visible_text(para)
        || footnote.paragraphs.len() != 1
        || crate::renderer::composer::compose_paragraph(&footnote.paragraphs[0])
            .lines
            .len()
            != 1
        || !matches!(
            st.current_items.last(),
            Some(PageItem::FullParagraph { para_index }) if *para_index == para_idx
        )
    {
        return false;
    }

    let control_pos = match para.control_text_positions().get(ctrl_idx) {
        Some(position) => *position,
        None => return false,
    };
    let last_visible_line = match para
        .line_segs
        .iter()
        .rposition(|line| !is_synthetic_line_seg(line))
    {
        Some(index) => index,
        None => return false,
    };
    let marker_line =
        match para.line_segs.iter().rposition(|line| {
            !is_synthetic_line_seg(line) && line.text_start as usize <= control_pos
        }) {
            Some(index) => index,
            None => return false,
        };
    if marker_line != last_visible_line {
        return false;
    }

    let Some(next_para) = paragraphs.get(para_idx + 1) else {
        return false;
    };
    if !para_has_visible_text(next_para)
        || !matches!(next_para.column_type, ColumnBreakType::None)
        || !next_para
            .line_segs
            .iter()
            .find(|line| !is_synthetic_line_seg(line))
            .is_some_and(|line| line.vertical_pos == 0)
    {
        return false;
    }

    // marker 뒤의 다음 문단이 stored `vpos=0`으로 새 physical page를 시작할 때,
    // 현재 page의 마지막 note를 새 page로 소유시킬지 판정한다.
    //
    // 첫 각주뿐 아니라 기존 각주 뒤의 마지막 note도 같은 계약을 가질 수 있다. 예를
    // 들어 이 fixture의 p199에는 257)이 남고 258)의 marker는 p199 tail에 있지만,
    // p200의 첫 본문과 함께 footnote area를 시작한다. 258)을 p199에 더하면 p200의
    // `pi=2310` reset tail 여섯 줄이 footer 아래로 그려진다. 다음 문단의 explicit
    // page-top reset과 projected footnote collision을 둘 다 요구하므로, 일반 multi-note
    // owner를 이동시키지는 않는다.
    let projected_footnote_height = st.projected_footnote_height(footnote_height, 1);
    let projected_reclaim = st.footer_band_reclaim_for_height(projected_footnote_height);
    let projected_available = (st.base_available_height()
        - (projected_footnote_height - projected_reclaim).max(0.0)
        - st.footnote_safety_margin
        - st.current_zone_y_offset
        - st.current_bottom_fixed_exclusion)
        .max(0.0);

    st.current_height > projected_available + 0.5
}

/// 현재 page의 일반 Body 각주를 layout과 같은 composed line metric으로 재측정한다.
///
/// 일반 pagination의 `current_footnote_height`는 빠른 stored-LineSeg 추정이다. 긴 URL이나
/// 여러 각주가 있는 HWP5 page에서는 실제 FootnoteArea보다 작을 수 있다. 그 차이를 전역 예약값으로
/// 바꾸면 과페이지화 회귀가 생기므로, reset tail의 physical collision 판정에만 이 exact metric을 쓴다.
/// table/text-box source 또는 이미 fragment된 각주는 각자 별도 owner 계약이 있으므로 이 좁은 경로에서
/// 재측정하지 않는다.
pub(in crate::renderer::typeset) fn native_hwp5_existing_body_footnote_area_height(
    st: &TypesetState,
    paragraphs: &[Paragraph],
    dpi: f64,
) -> Option<f64> {
    let footnotes = &st.pages.last()?.footnotes;
    if footnotes.is_empty() || footnotes.iter().any(|footnote| footnote.fragment.is_some()) {
        return None;
    }

    let mut total = st.footnote_separator_overhead;
    for (footnote_idx, footnote_ref) in footnotes.iter().enumerate() {
        let FootnoteSource::Body {
            para_index,
            control_index,
        } = &footnote_ref.source
        else {
            return None;
        };
        let Control::Footnote(footnote) = paragraphs
            .get(*para_index)
            .and_then(|para| para.controls.get(*control_index))?
        else {
            return None;
        };

        total += composed_footnote_content_height(footnote, dpi);
        if footnote_idx + 1 < footnotes.len() {
            total += st.footnote_between_notes_margin;
        }
    }
    Some(total)
}

/// 이미 예약된 각주 영역을 침범하는 native HWP5 본문 reset tail을 찾는다.
///
/// HWP5는 한 문단의 뒤쪽 줄을 다음 physical page에 두면서 `vpos=0`으로 저장한다.
/// 기존 각주가 있는 page에서 이 신호를 전역으로 따르면 과분할될 수 있으므로, source 좌표가
/// 현재 flow와 맞고 reset 직전 줄은 FootnoteArea 위에 끝나며 다음 줄만 실제 각주 경계를
/// 침범하는 경우에만 허용한다. 이 조건은 p43의 pi=512처럼 body tail이 separator/첫 각주를
/// 덮는 경우를 고치되, 일반적인 paragraph reset은 건드리지 않는다.
pub(in crate::renderer::typeset) fn native_hwp5_existing_footnote_reset_overlap_break_line(
    st: &TypesetState,
    para: &Paragraph,
    fmt: &FormattedParagraph,
    paragraphs: &[Paragraph],
    dpi: f64,
) -> Option<usize> {
    if !st.profile.hwp5_stored_pagination_layout()
        || st.col_count != 1
        || st.current_footnote_height <= 0.0
        || !para_has_visible_text(para)
        || para.line_segs.len() < fmt.line_heights.len()
    {
        return None;
    }

    let page_vpos_base = st.vpos_page_base.or(st.vpos_lazy_base).unwrap_or(0);
    let actual_footnote_height =
        native_hwp5_existing_body_footnote_area_height(st, paragraphs, dpi)?;
    let control_positions = para.control_text_positions();
    let mut current_footnotes = Vec::with_capacity(para.controls.len());
    for (control_index, control) in para.controls.iter().enumerate() {
        let Control::Footnote(footnote) = control else {
            // 표·그림 등 다른 inline control의 owner 계약과 섞지 않는다.
            return None;
        };
        current_footnotes.push((*control_positions.get(control_index)?, footnote.as_ref()));
    }
    const FLOW_SOURCE_TOLERANCE_PX: f64 = 2.0;
    const FOOTNOTE_BOUNDARY_TOLERANCE_PX: f64 = 0.5;

    para.line_segs[..fmt.line_heights.len()]
        .windows(2)
        .enumerate()
        .find_map(|(prev_idx, pair)| {
            let (prev, next) = (&pair[0], &pair[1]);
            let next_idx = prev_idx + 1;
            if is_synthetic_line_seg(prev)
                || is_synthetic_line_seg(next)
                || prev.vertical_pos <= page_vpos_base
                || next.vertical_pos != 0
            {
                return None;
            }

            let source_prev_top = hwpunit_to_px(prev.vertical_pos - page_vpos_base, dpi);
            let flow_prev_top =
                st.current_height + fmt.spacing_before + fmt.line_advances_sum(0..prev_idx);
            if (source_prev_top - flow_prev_top).abs() > FLOW_SOURCE_TOLERANCE_PX {
                return None;
            }

            // 현재 문단의 각주는 본문을 배치한 뒤 등록된다. marker가 reset 직전 줄에
            // 있으면 이미 존재하는 각주만으로 계산한 top은 늦고, 새 note가 사후에
            // FootnoteArea를 위로 키워 방금 배치한 다음 줄과 겹친다. 이 candidate보다
            // 앞에 marker가 있는 현재 문단 각주만 exact 높이로 투영한다.
            let next_text_start = next.text_start as usize;
            let prev_text_range = prev.text_start as usize..next_text_start;
            let marker_on_prev_line = current_footnotes
                .iter()
                .any(|(position, _)| prev_text_range.contains(position));
            if !current_footnotes.is_empty() && !marker_on_prev_line {
                return None;
            }
            let projected_current_notes: Vec<&Footnote> = current_footnotes
                .iter()
                .filter(|(position, _)| *position < next_text_start)
                .map(|(_, footnote)| *footnote)
                .collect();
            let projected_footnote_height = actual_footnote_height
                + projected_current_notes.len() as f64 * st.footnote_between_notes_margin
                + projected_current_notes
                    .iter()
                    .map(|footnote| composed_footnote_content_height(footnote, dpi))
                    .sum::<f64>();
            let footnote_top = (st.layout.body_area.height - projected_footnote_height).max(0.0);
            let flow_prev_bottom = flow_prev_top + fmt.line_heights[prev_idx];
            let flow_next_bottom =
                flow_prev_top + fmt.line_advance(prev_idx) + fmt.line_heights[next_idx];
            (flow_prev_bottom <= footnote_top + FOOTNOTE_BOUNDARY_TOLERANCE_PX
                && flow_next_bottom > footnote_top + FOOTNOTE_BOUNDARY_TOLERANCE_PX)
                .then_some(next_idx)
        })
}
/// RowBreak 표가 쪽 경계를 넘을 때 순서대로 배치할 셀 각주.
///
/// 일반 표는 표 조판 뒤 한 페이지에 모아 등록하는 기존 계약을 유지한다. 표 본체와
/// 모든 각주가 새 쪽 하나에는 들어가지만 현재 잔여에는 전부 들어가지 않는 작은
/// RowBreak 표만 이 정보를 이용해 fragment별 각주 예약을 늦춘다.
#[derive(Debug, Clone, Copy)]
pub(in crate::renderer::typeset) struct NativeHwp5FootnoteFragmentSplit {
    pub(in crate::renderer::typeset) prefix: FootnoteFragment,
    pub(in crate::renderer::typeset) prefix_height: f64,
    pub(in crate::renderer::typeset) suffix: FootnoteFragment,
    pub(in crate::renderer::typeset) suffix_height: f64,
    /// 첫 두 stored line이 모두 `vpos=0`인 명시적 다음 physical page 시작.
    pub(in crate::renderer::typeset) force_next_page: bool,
}

/// 실제 각주 영역과 겹치는 native HWP5 본문 문단의 저장 reset을 찾는다.
///
/// HWP5는 본문 마지막 줄과 다음 physical page의 첫 줄을 하나의 paragraph
/// `LINE_SEG`에 넣고, 뒤쪽 줄의 `vpos=0`으로 경계를 표시할 수 있다. 보통 reset을
/// 전역으로 따르는 것은 과분할을 일으킨다. 다만 현재 페이지의 **첫** 각주가 이 문단에
/// 있고, reset 직전 줄의 trailing line-spacing이 실제 각주 영역을 침범할 때는 그
/// 경계를 무시하면 renderer가 뒤쪽 줄을 각주 위에 계속 쌓는다.
///
/// 각주 본문은 stored LineSeg가 아니라 renderer와 같은 composer 결과로 재서, 긴 각주의
/// wrap/line-spacing을 포함한다. 이미 다른 각주가 있는 page는 ownership과 누적 높이가
/// 별도 경로이므로 이 좁은 보정의 대상이 아니다.
#[derive(Debug, Clone, Copy)]
pub(in crate::renderer::typeset) struct NativeHwp5FootnoteBreak {
    pub(in crate::renderer::typeset) body_break_line: usize,
    pub(in crate::renderer::typeset) split_footnote: bool,
}
