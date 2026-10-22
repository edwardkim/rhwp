//! 저장 줄 사다리의 간격·프레임 소유 유효성 판정.
//!
//! 문단·스타일과 실제 수용한 앞뒤 프레임을 읽어 판정을 반환한다.
//! 실제 spacing 트림과 dirty 전환·페이지 전진은 호출자가 수행한다.

use crate::model::paragraph::Paragraph;

/// 현재 실제 페이지가 저장 프레임의 중간부터 시작하는지 실제 앞뒤 소유로 확인한다.
/// 저장 prefix가 이전 실제 페이지에서 소비됐으면 다음 자동 reset도 재배분 대상이다.
pub(in crate::renderer::typeset) fn stored_frame_prefix_consumed_before_page(
    st: &super::super::TypesetState,
    paragraphs: &[Paragraph],
    para_idx: usize,
    styles: &crate::renderer::style_resolver::ResolvedStyleSet,
) -> bool {
    use crate::renderer::pagination::PageItem;
    if st.col_count != 1
        || st.profile.session_edited()
        || st.profile.hwp3_layout()
        || !(st.profile.hwpx_stored_layout() || st.profile.hwp5_stored_pagination_layout())
        || paragraphs.get(para_idx).is_none_or(|para| {
            para.column_type != crate::model::paragraph::ColumnBreakType::None
                || para
                    .line_segs
                    .first()
                    .is_some_and(|line| line.is_first_line_of_page())
        })
    {
        return false;
    }
    // 완전한 TAC 줄이 새 단의 원점에서 수용되고 뒤 저장 줄까지 같은
    // 전진 끝으로 이어지면 새 저장 프레임이 이미 확립됐다. 옛 host의
    // 양수 vpos만 보고 그 뒤 모든 쪽 경계를 무효화하지 않는다.
    // 배치와 post_flow가 확인한 동일한 placement·줄 전진 결과를 읽는다.
    if st.current_start_height == 0.0
        && !st.current_endnote_flow
        && crate::renderer::float_placement::restarted_tac::accepted_column_origin(
            &st.current_items,
            &st.inline_placements,
            paragraphs,
            styles,
            st.layout.dpi,
        )
        .is_some_and(|(_, origin)| st.vpos_page_base == Some(origin))
    {
        return false;
    }
    let Some(previous_page) = st
        .pages
        .iter()
        .rev()
        .nth(1)
        .filter(|page| page.section_index == st.section_index)
    else {
        return false;
    };
    let Some(previous) = previous_page
        .column_contents
        .last()
        .and_then(|col| col.items.last())
    else {
        return false;
    };
    let Some(first) = st.current_items.first() else {
        return false;
    };
    let complete_tac_source = |para: &Paragraph| {
        matches!(para.controls.as_slice(), [crate::model::control::Control::Table(table)]
            if table.common.treat_as_char)
    };
    let (owner, start) = match first {
        PageItem::FullParagraph { para_index } => (*para_index, 0),
        PageItem::PartialParagraph {
            para_index,
            start_line,
            ..
        } => (*para_index, *start_line),
        PageItem::Table {
            para_index,
            control_index: 0,
        } if paragraphs.get(*para_index).is_some_and(complete_tac_source) => (*para_index, 0),
        _ => return false,
    };
    let Some(para) = paragraphs.get(owner) else {
        return false;
    };
    let valid_source = |para: &Paragraph| {
        !para.stored_text_partition_is_dirty()
            && !para.cell_format_vpos_dirty
            && (para.controls.is_empty() || complete_tac_source(para))
            && para.column_type == crate::model::paragraph::ColumnBreakType::None
            && !para.line_segs.is_empty()
            && para
                .line_segs
                .iter()
                .all(|line| !super::is_synthetic_line_seg(line))
            && para
                .source_line_seg_vertical_pos
                .as_ref()
                .is_none_or(|source| source.len() == para.line_segs.len())
    };
    let position = |para: &Paragraph, index: usize| {
        para.source_line_seg_vertical_pos
            .as_ref()
            .map_or(para.line_segs[index].vertical_pos, |source| source[index])
    };
    if !valid_source(para)
        || start >= para.line_segs.len()
        || position(para, start) <= 0
        || para.line_segs[start].is_first_line_of_page()
    {
        return false;
    }
    if start > 0 {
        // 현재 조각 바로 앞 줄을 이전 실제 페이지가 소유했고, 저장 축은 이어진다.
        return matches!(previous,
            PageItem::PartialParagraph { para_index, end_line, .. }
            if *para_index == owner && *end_line == start)
            && position(para, start) > position(para, start - 1)
            && (0..start).any(|line| position(para, line) == 0);
    }
    let Some(before_index) = owner.checked_sub(1) else {
        return false;
    };
    let Some(before) = paragraphs
        .get(before_index)
        .filter(|para| valid_source(para))
    else {
        return false;
    };
    let complete = match previous {
        PageItem::FullParagraph { para_index } => *para_index == before_index,
        PageItem::PartialParagraph {
            para_index,
            end_line,
            ..
        } => *para_index == before_index && *end_line == before.line_segs.len(),
        PageItem::Table {
            para_index,
            control_index: 0,
        } => *para_index == before_index && complete_tac_source(before),
        _ => false,
    };
    let last = before.line_segs.len() - 1;
    // 실제 쪽이 같은 저장 프레임의 중간에서 시작하면 다음 자동 reset은
    // 이미 소비한 prefix를 다시 쪽 나눌 근거가 아니다.
    // 완전한 TAC 표도 저장 줄 소유자이므로 텍스트 항목과 함께 확인한다.
    let continued_source_frame = position(before, 0) > 0
        && position(para, 0)
            >= position(before, last).saturating_add(before.line_segs[last].line_height);
    complete
        && (continued_source_frame
            || ((1..=last).any(|line| {
                position(before, line) == 0
                    && position(before, line - 1) > 0
                    && !before.line_segs[line].is_first_line_of_page()
            }) && position(para, 0)
                == position(before, last)
                    .saturating_add(before.line_segs[last].line_height)
                    .saturating_add(before.line_segs[last].line_spacing)))
}

/// 위쪽에서 시작한 큰 개체 줄도 끝까지 소비했다면 다음 저장 프레임의 경계를 따른다.
/// 저장 시작 위치의 수치 문턱으로 그 줄의 물리 점유를 대신하지 않는다.
pub(super) fn saved_object_frame_reset_at_current_flow(
    paragraphs: &[Paragraph],
    index: usize,
    current_items: &[crate::renderer::pagination::PageItem],
    body_height: f64,
    dpi: f64,
) -> bool {
    let Some(previous) = index.checked_sub(1).and_then(|i| paragraphs.get(i)) else {
        return false;
    };
    let Some(current) = paragraphs.get(index) else {
        return false;
    };
    // 실제 쪽이 앞 개체 전체를 소유해야 그 뒤의 저장 경계를 사용할 수 있다.
    // 저장 줄 높이와 표 측정 높이는 바깥여백·줄 끝 간격의 계상 범위가 달라,
    // 두 끝 좌표의 정확한 수치 일치를 소유 근거로 사용하지 않는다.
    let previous_is_complete = current_items.last().is_some_and(|item| {
        matches!(item,
            crate::renderer::pagination::PageItem::Table { para_index, .. }
            | crate::renderer::pagination::PageItem::FullParagraph { para_index }
            if Some(*para_index) == index.checked_sub(1))
    });
    if !previous_is_complete {
        return false;
    }
    if !crate::renderer::stored_frame_boundary::owns_saved_object_line(previous)
        || current.stored_text_partition_is_dirty()
        || current.cell_format_vpos_dirty
        || !current
            .controls
            .iter()
            .all(crate::renderer::control_is_body_flow_neutral_marker)
    {
        return false;
    }
    let Some(first) = current.line_segs.first().filter(|line| {
        line.tag & crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
    }) else {
        return false;
    };
    // 사다리 재구성 전의 source 축은 직렬화와 쪽 소속의 독립 근거다.
    // 편집된 줄은 위에서 제외하며 snapshot과 현재 줄의 대응도 확인한다.
    let source_position = |para: &Paragraph, line: usize| match &para.source_line_seg_vertical_pos {
        Some(source) if source.len() == para.line_segs.len() => source.get(line).copied(),
        Some(_) => None,
        None => para.line_segs.get(line).map(|line| line.vertical_pos),
    };
    let Some(previous_top) = source_position(previous, 0) else {
        return false;
    };
    let Some(current_top) = source_position(current, 0) else {
        return false;
    };
    let previous_line = &previous.line_segs[0];
    let painted_end = previous_top.saturating_add(previous_line.line_height);
    let previous_end = painted_end.saturating_add(previous_line.line_spacing);
    let mut source_first = first.clone();
    source_first.vertical_pos = current_top;
    crate::renderer::stored_frame_boundary::saved_positive_frame_reset(
        Some((previous_top, previous_end)),
        &source_first,
        crate::renderer::px_to_hwpunit(body_height, dpi),
    )
}
use crate::renderer::hwpunit_to_px;
use crate::renderer::style_resolver::ResolvedStyleSet;

/// 재조판으로 재기준한 좌표를 저장 쪽 경계의 증거로 다시 사용하지 않는다.
/// 직렬화용으로 보존한 원본 좌표와 해당 저장 줄의 진행량을 같은 축에서 읽는다.
pub(in crate::renderer::typeset) fn stored_tail_end_and_advance_hu(
    para: &Paragraph,
) -> Option<(i32, i32)> {
    if para.stored_text_partition_is_dirty() {
        return None;
    }
    let (index, line) = para
        .line_segs
        .iter()
        .enumerate()
        .rev()
        .find(|(_, line)| !crate::renderer::typeset::is_synthetic_line_seg(line))?;
    let source_vpos = para
        .source_line_seg_vertical_pos
        .as_ref()
        .and_then(|positions| positions.get(index).copied())
        .unwrap_or(line.vertical_pos);
    let advance = line.line_height.saturating_add(line.line_spacing);
    Some((source_vpos.saturating_add(advance), advance))
}

/// 현재 단에서 실제 소비한 재구성 본문 앞의 원본 프레임을 찾는다.
/// 합성 줄의 누적 vpos는 원본 쪽 좌표의 근거가 아니다. 개체나 이어받기
/// 조각을 건너뛰지 않고, 연속 본문만 같은 저장 프레임에 연결한다.
pub(in crate::renderer::typeset) fn stored_vpos_before_reconstructed_text(
    current_items: &[crate::renderer::pagination::PageItem],
    paragraphs: &[Paragraph],
    next_para_index: usize,
) -> Option<i32> {
    let mut expected = next_para_index.checked_sub(1)?;
    let mut reconstructed = false;
    for item in current_items.iter().rev() {
        let crate::renderer::pagination::PageItem::FullParagraph { para_index } = item else {
            return None;
        };
        if *para_index != expected {
            return None;
        }
        let para = paragraphs.get(*para_index)?;
        if !para.controls.is_empty()
            || para.stored_text_partition_is_dirty()
            || para.cell_format_vpos_dirty
            || para.column_type != crate::model::paragraph::ColumnBreakType::None
        {
            return None;
        }
        if para.source_line_seg_vertical_pos.is_none()
            && (para.line_segs.is_empty()
                || para
                    .line_segs
                    .iter()
                    .all(crate::renderer::typeset::is_synthetic_line_seg))
        {
            reconstructed = true;
            expected = expected.checked_sub(1)?;
            continue;
        }
        if !reconstructed
            || para.line_segs.iter().any(|line| {
                crate::renderer::typeset::is_synthetic_line_seg(line) || line.line_height <= 0
            })
        {
            return None;
        }
        let last = para.line_segs.last()?;
        return match &para.source_line_seg_vertical_pos {
            Some(source) if source.len() == para.line_segs.len() => source.last().copied(),
            Some(_) => None,
            None => Some(last.vertical_pos),
        };
    }
    None
}

/// 재조판 뒤 저장 빈 줄만 이어지면 옛 자동 쪽 경계를 복원하지 않는다.
/// 다만 실제 소비한 흐름 끝이 저장 경계와 일치하면 재조판 이력만으로
/// 그 경계를 버리지 않는다. 같은 쪽 소유는 기존 저장 끝/실제 흐름 대조로 확인한다.
/// 저장 본문/개체를 다시 소비한 경계는 그 저장 흐름의 소유다. 따라서 쪽에
/// 재조판 항목이 한 번 있었다는 사실을 쪽 전체의 무효 상태로 사용하지 않는다.
/// 명시적 쪽/단 나누기는 이 검사 이전의 entry 경로에서 처리한다.
pub(in crate::renderer::typeset) fn stored_page_boundary_invalidated_by_reflow(
    profile: crate::model::provenance::LayoutCompatibilityProfile,
    col_count: u16,
    current_items: &[crate::renderer::pagination::PageItem],
    paragraphs: &[Paragraph],
    para_idx: usize,
    current_height_px: f64,
    dpi: f64,
) -> bool {
    if col_count != 1 || !(profile.hwpx_stored_layout() || profile.hwp5_stored_pagination_layout())
    {
        return false;
    }
    // 빈 저장 줄도 실제 점유를 소유한다. 그 줄을 전량 소비한 끝이 저장
    // 흐름과 맞으면, 앞 재조판의 존재만으로 뒤 쪽 경계를 무효화하지 않는다.
    // 미편집 전체 문단만 확인하며 합성 줄이나 개체/이어받기를 기준으로 삼지 않는다.
    if !profile.session_edited()
        && current_items.last().is_some_and(|item| {
            let crate::renderer::pagination::PageItem::FullParagraph { para_index } = item else {
                return false;
            };
            paragraphs.get(*para_index).is_some_and(|para| {
                para.text.trim().is_empty()
                    && para.controls.is_empty()
                    && !para.stored_text_partition_is_dirty()
                    && !para.cell_format_vpos_dirty
                    && !para.line_segs.is_empty()
                    && para.line_segs.iter().all(|line| {
                        !crate::renderer::typeset::is_synthetic_line_seg(line)
                            && line.line_height > 0
                    })
                    && crate::renderer::typeset::stored_rewind_boundary_matches_current_flow(
                        paragraphs,
                        para_index + 1,
                        current_items,
                        current_height_px,
                        dpi,
                    )
            })
        })
    {
        return false;
    }
    for item in current_items.iter().rev() {
        use crate::renderer::pagination::PageItem;
        let para_index = match item {
            PageItem::FullParagraph { para_index }
            | PageItem::PartialParagraph { para_index, .. } => *para_index,
            // 개체 항목은 독립 배치 owner다. host의 합성 줄을 본문 재조판으로
            // 오인하지 않고, 이 owner 뒤의 저장 경계는 기존 개체 경로가 맡는다.
            _ => return false,
        };
        let Some(para) = paragraphs.get(para_index) else {
            return false;
        };
        if crate::renderer::para_has_no_stored_line_segs(para) {
            let boundary_matches_actual_flow = paragraphs.get(para_idx).is_some_and(|current| {
                !current.stored_text_partition_is_dirty()
                    && crate::renderer::typeset::stored_vpos_restarts_near_body_top(current)
                    && crate::renderer::typeset::stored_rewind_boundary_matches_current_flow(
                        paragraphs,
                        para_idx,
                        current_items,
                        current_height_px,
                        dpi,
                    )
            });
            return !boundary_matches_actual_flow;
        }
        if !para.text.trim().is_empty() {
            return false;
        }
    }
    false
}

/// [#5801] 저장 사다리가 이 문단의 **문단 위 간격을 실제로 담고 있는가**.
///
/// `#2279 ①` 의 spacing 트림은 "저장 ladder 가 spacing 을 이미 반영한다"는 전제 위에 선다.
/// 그 게이트(`has_authoritative_seg`)는 lineseg 가 합성인지만 본다. 그런데 **합성이 아닌데도
/// 문단 위 간격을 안 담은** 사다리가 있다 — `156677324` 는 문단 간 delta 와 문단 내 줄
/// advance 가 똑같이 2240 HU 인데, 한글은 그 자리에 50.23px(≈ 줄간격 29.86 + 문단간격 20.4)
/// 를 그린다(한글 PDF 실측, rhwp 렌더 49.87px 과 일치).
///
/// 그런 사다리에 트림을 적용하면 typeset 이 쪽 채움을 문단마다 `sb` 만큼 짧게 세어, 1쪽 끝에서
/// 99px 짜리 착시가 생긴다(트림 합계 109.9px). 쪽이 다 찼는데 남았다고 보고 다음 문단을
/// 현재 쪽에 얹는다(#5755).
///
/// 판별은 데이터 안에 있다 — 앞 문단 마지막 줄 아래에서 이 문단 첫 줄까지의 **저장 간격**이
/// `줄 간격 + 문단 위 간격` 을 담고 있으면 권위 사다리, 줄 간격뿐이면 아니다.
pub(in crate::renderer::typeset) fn stored_ladder_encodes_spacing_before(
    paragraphs: &[Paragraph],
    para_idx: usize,
    spacing_before_px: f64,
    dpi: f64,
) -> bool {
    if spacing_before_px <= 0.5 {
        return true; // 담을 간격이 없다 — 판별 대상 아님.
    }
    let Some(para) = paragraphs.get(para_idx) else {
        return true;
    };
    let Some(first) = para.line_segs.first() else {
        return true;
    };
    let Some(prev) = para_idx
        .checked_sub(1)
        .and_then(|idx| paragraphs.get(idx))
        .filter(|prev| prev.controls.is_empty())
    else {
        return true; // 앞이 없거나 컨트롤 문단 — 사다리 비교가 성립하지 않는다.
    };
    let Some(prev_last) = prev.line_segs.last() else {
        return true;
    };
    // 합성(vpos 전부 0) 사다리는 이 판별의 대상이 아니다 — 기존 게이트가 처리한다.
    if first.vertical_pos == 0 || prev_last.vertical_pos == 0 {
        return true;
    }
    let stored_gap = first
        .vertical_pos
        .saturating_sub(prev_last.vertical_pos.saturating_add(prev_last.line_height));
    if stored_gap <= 0 {
        return true; // 쪽·단 경계 되감김 — 판별 불가.
    }
    // 비교 기준은 **같은 사다리 안의 줄 간 실제 delta** 다. 저장 `line_spacing` 필드를 쓰면
    // 문단 경계에서 줄 간격을 흡수한 사다리를 오탐한다(2990099·3249937 에서 +3·+1쪽 회귀).
    let intra_gap = stored_intra_line_gap(para).or_else(|| stored_intra_line_gap(prev));
    let Some(intra_gap) = intra_gap else {
        // [#6031] 한 줄짜리 문단 연속 구간 — 줄 간 delta 표본이 없다. 직전 문단
        // 마지막 줄의 저장 trailing ls 와 경계 gap 이 **정확히 일치**하면 사다리가
        // 경계를 줄바꿈처럼 적은 것(sb 미인코딩)이다 (3249937 p6: gap 1400 == ls
        // 1400, sb 1000 미반영 → 쪽-말미 2줄이 본문 밖 +40pt). ls 필드를 1차
        // 기준으로 쓰면 경계 흡수형 사다리를 오탐하지만(아래 주석, 2990099·
        // 3249937 +3·+1쪽 회귀), 표본 부재 시의 등가-일치 판별은 흡수형
        // (gap < ls)과 정상형(gap ≥ ls+sb) 어느 쪽에도 걸리지 않는다.
        // sb 하한 5px: 아주 작은 sb(예: hwp3-sample16 계보 285HU=3.8px)는 한글이
        // 저장 흐름을 신뢰하는 문서군과 겹친다(#2158 핀 64쪽 실측 — 누락 판정 시
        // 트림 철회 누적 +3.8px×282 경계로 65쪽 회귀). 여백 관통을 만드는 굵은
        // sb(6.7px+)만 누락 판정한다.
        // 추가 지문: 이 생성기 계열은 줄 피치를 lh·ls 에 양분해 적는다
        // (ls == lh, paraPr 160% 와도 모순 — 3249937 전 문단 1400/1400).
        // 정상 저장 사다리(ls < lh, hwp3-sample16 660/2000)는 한글이 저장
        // 흐름을 신뢰하는 문서군과 겹치므로 등가-일치만으로 누락 판정하지 않는다.
        if spacing_before_px > 5.0
            && prev_last.line_spacing == prev_last.line_height
            && prev_last.line_spacing > 0
            && (stored_gap - prev_last.line_spacing).abs() <= 2
            && hwpunit_to_px(prev_last.line_spacing, dpi) + spacing_before_px
                > hwpunit_to_px(stored_gap, dpi) + 0.5
        {
            return false;
        }
        return true; // 판별 불가 — 종전 보수 유지.
    };
    // 문단 경계 간격이 줄 간격과 같으면 사다리가 경계를 줄바꿈처럼 적은 것이다.
    hwpunit_to_px(stored_gap, dpi) + 0.5 >= hwpunit_to_px(intra_gap, dpi) + spacing_before_px
}

/// [#5801] 같은 문단 안에서 저장 사다리가 적은 줄 사이 실제 간격(HWPUNIT).
fn stored_intra_line_gap(para: &Paragraph) -> Option<i32> {
    para.line_segs.windows(2).find_map(|w| {
        let gap = w[1]
            .vertical_pos
            .saturating_sub(w[0].vertical_pos.saturating_add(w[0].line_height));
        (gap >= 0 && w[0].vertical_pos != 0).then_some(gap)
    })
}

/// [#2279 ①-3] spacing 트림의 복원 가능성 전방 판정.
///
/// 트림은 다음 authoritative(비합성 lineseg) anchor 에서 vpos-snap 이 좌표를
/// 복원한다는 전제다. 현 문단과 다음 anchor 사이에 합성/NO_LS 텍스트 문단이
/// 끼어 있으면 dirty 규칙(#2243 전방-스냅만)으로 복원이 차단되므로 트림하면
/// 그 spacing 이 흐름에서 소실된다 (36398700 pi6: 다음 anchor pi10 앞에
/// 합성 pi7~9 → −35.7px 소실 실측). 빈 호스트의 자리차지 표는 현재 흐름에서
/// 점유 상자를 확정하므로 저장 위치 복원을 보장하지 않는다. 다른 컨트롤은
/// 기존 재앵커 계약을 따른다. 탐색은 32문단 한도(초과 시 보수적으로 복원 불가).
pub(in crate::renderer::typeset) fn spacing_trim_restorable(
    paragraphs: &[Paragraph],
    para_idx: usize,
) -> bool {
    use crate::model::paragraph::LineSeg;
    for para in paragraphs.iter().skip(para_idx + 1).take(32) {
        if !para.controls.is_empty() {
            // 빈 호스트의 자리차지 표는 하나여도 현재 흐름에서 lane을 예약한다.
            // 저장 vpos의 높이 증거만으로 현재 쪽 원점 복원을 가정하면 안 된다.
            // 앞 문단의 줄간격을 보존해 예약과 실제 출력에 같은 원점을 전달한다.
            let flow_lane_tables = !super::super::para_has_visible_text(para)
                && para.controls.iter().any(|control| {
                    matches!(control, crate::model::control::Control::Table(table)
                        if crate::renderer::float_placement::is_para_topbottom_float(&table.common))
                });
            return !flow_lane_tables;
        }
        match para.line_segs.first() {
            Some(seg) if seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0 => {
                return true; // authoritative anchor 도달 — 복원 가능
            }
            _ => {
                if !para.text.is_empty() {
                    return false; // 합성/NO_LS 텍스트 문단이 먼저 — 복원 불가
                }
                // 빈 문단은 계속 탐색
            }
        }
    }
    false
}

/// [#7196] 다음 문단 경계가 `#6031` 철회(dirty)에 걸려 트림 복원이 막히는가.
///
/// `#2279 ①` 트림은 다음 저장 anchor 의 vpos-snap 이 좌표를 되돌린다는 전제인데,
/// HWPX 저장 레이아웃에서 다음 문단이 굵은 문단 위 간격(> 5px)을 가졌고 그 경계의 저장
/// 사다리가 그 간격을 담지 않았으면(`stored_ladder_encodes_spacing_before` 거짓) `#6031` 이
/// 그 스냅을 되돌리고 사다리를 dirty 로 만든다. 그러면 이 문단에서 깎은 문단 위 간격과
/// 끝 줄 간격이 영영 복원되지 않아 조판이 렌더보다 짧게 센다 — 156760012 10쪽 첫 문단
/// pi=66: 트림 52.3px(sb 26.7 + ls 25.6) 미복원, 쪽 말미 표가 본문 바닥 +18.3px 넘침.
/// 판정식·게이트(`hwpx_stored_layout`, `!hwp3_layout`, sb > 5px)는 `#6031` 과 같게 둔다.
pub(in crate::renderer::typeset) fn next_boundary_reverts_spacing_trim(
    hwpx_stored_non_hwp3: bool,
    paragraphs: &[Paragraph],
    styles: &ResolvedStyleSet,
    para_idx: usize,
    dpi: f64,
) -> bool {
    let next_idx = para_idx + 1;
    let Some(next) = paragraphs.get(next_idx) else {
        return false;
    };
    // `format_paragraph` 의 spacing_before 산출과 같은 축 — 저장 줄이 없는 텍스트 문단은 0.
    let spacing_before_px = if next.line_segs.is_empty() && !next.text.is_empty() {
        0.0
    } else {
        styles
            .para_styles
            .get(next.para_shape_id as usize)
            .map(|style| style.spacing_before)
            .unwrap_or(0.0)
    };
    if paragraphs.get(para_idx).is_some_and(|previous| {
        crate::renderer::height_cursor::paragraph_origin_rewinds(previous, next)
            || crate::renderer::height_cursor::rebuilt_ladder_omits_spacing_before(
                previous,
                next,
                spacing_before_px,
                dpi,
            )
            // [#7874] 저장 위치가 이 문단의 아래 간격을 뺐으면 다음 스냅은 끝 줄간격만
            // 되돌리고 아래 간격은 되돌리지 않는다 — 배치처럼 전체 높이로 전진한다.
            || crate::renderer::height_cursor::stored_ladder_omits_spacing_after(
                previous,
                next,
                styles
                    .para_styles
                    .get(previous.para_shape_id as usize)
                    .map_or(0.0, |style| style.spacing_after),
            )
    }) {
        return true;
    }
    if !hwpx_stored_non_hwp3 {
        return false;
    }
    spacing_before_px > 5.0
        && !stored_ladder_encodes_spacing_before(paragraphs, next_idx, spacing_before_px, dpi)
}
