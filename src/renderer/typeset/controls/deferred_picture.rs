//! 다음 physical page의 Square 그림 소유 후보 Query.
//! 저장 줄·각주 예약·그림/캡션 요구 높이를 조회하며 큐와 페이지 상태는 변경하지 않는다.
//! 가용 높이는 기존 두 분기의 지연 호출로 읽는다. 페이지 전이와 발행은 R5 소유다.

use super::super::para_has_visible_text;
use crate::model::{
    control::Control, paragraph::Paragraph, provenance::LayoutCompatibilityProfile,
    shape::CaptionDirection,
};
use crate::renderer::{hwpunit_to_px, pagination::PageItem, style_resolver::ResolvedStyleSet};

pub(in crate::renderer::typeset) struct DeferredPicturePage<'a> {
    pub profile: LayoutCompatibilityProfile,
    pub col_count: u16,
    pub current_items: &'a [PageItem],
    pub current_footnote_height: f64,
    pub current_height: f64,
}

/// 저장 후보만 반환한다. anchor 본문 배치와 그림 큐 반영은 호출자에 남긴다.
#[allow(clippy::too_many_arguments)]
pub(in crate::renderer::typeset) fn next_page_owner(
    page: DeferredPicturePage<'_>,
    available_height: impl Fn() -> f64,
    dpi: f64,
    para_idx: usize,
    para: &Paragraph,
    paragraphs: &[Paragraph],
    ctrl: &Control,
    styles: &ResolvedStyleSet,
) -> Option<(Vec<usize>, crate::renderer::pagination::WrapAnchorRef)> {
    use crate::model::shape::{HorzAlign, HorzRelTo, TextWrap, VertAlign, VertRelTo};

    let Control::Picture(picture) = ctrl else {
        return None;
    };
    let common = &picture.common;
    let has_bottom_caption = picture
        .caption
        .as_ref()
        .is_some_and(|caption| matches!(caption.direction, CaptionDirection::Bottom));
    if !page.profile.hwp5_stored_pagination_layout()
        || page.col_count != 1
        || page.current_items.is_empty()
        || page.current_footnote_height <= 0.0
        || !para_has_visible_text(para)
        || common.treat_as_char
        || !common.flow_with_text
        || common.allow_overlap
        || !matches!(common.text_wrap, TextWrap::Square)
        || !matches!(common.vert_rel_to, VertRelTo::Para)
        || !matches!(common.vert_align, VertAlign::Top)
        || !matches!(common.horz_rel_to, HorzRelTo::Column)
        || !matches!(common.horz_align, HorzAlign::Left)
        || common.horizontal_offset == 0
        || !has_bottom_caption
    {
        return None;
    }

    // 다음 문단의 vpos=0 narrow band는 한컴 저장 흐름에서 그림의 다음
    // physical-page owner를 직접 가리킨다. 같은 문단의 full-width lines 뒤에
    // reset하는 경우(p1693)와 다음 문단이 narrow band로 곧바로 시작하는 경우
    // (p1356)를 모두 수용하되, 이 형상 없이 generic Square float을 옮기지 않는다.
    let next_para = paragraphs.get(para_idx + 1)?;
    let (reset_idx, reset_seg) = next_para.line_segs.iter().enumerate().find(|(_, seg)| {
        seg.vertical_pos == 0
            && seg.column_start == 0
            && seg.segment_width > 0
            && (seg.segment_width as i32 - common.horizontal_offset as i32).abs() <= 200
    })?;
    let has_full_width_before_reset = reset_idx > 0
        && next_para.line_segs[..reset_idx]
            .iter()
            .any(|seg| seg.segment_width > reset_seg.segment_width.saturating_add(1000));
    // p1356처럼 다음 문단이 narrow band로 시작하면, 그 문단 전체의 저장 advance가
    // 현 각주 예약 뒤에 남은 높이를 초과해야만 next physical page owner라고
    // 확정한다. 이 guard가 없으면 단순 side-wrap 문단을 나중의 무관한 page break에
    // 잘못 매달 수 있다.
    let next_para_starts_on_next_page = reset_idx == 0 && {
        let stored_flow_height = next_para
            .line_segs
            .iter()
            .enumerate()
            .map(|(idx, seg)| {
                let trailing_spacing = if idx + 1 < next_para.line_segs.len() {
                    seg.line_spacing
                } else {
                    0
                };
                hwpunit_to_px(seg.line_height + trailing_spacing, dpi)
            })
            .sum::<f64>();
        page.current_height + stored_flow_height > available_height() + 0.5
    };
    if !has_full_width_before_reset && !next_para_starts_on_next_page {
        return None;
    }

    // `vertical_offset + height`는 그림 자체가 차지하는 저장 vpos 구간의 끝이다.
    // 같은 cs/sw인 문단만이라도 이 범위를 넘어가면 다음 일반 본문까지 wrap이
    // 새어 나간다. 반대로 blank guide 문단은 실제 글자가 없어도 다음 visible
    // 문단에 wrap contract를 전달하므로 포함해야 한다. #3821의 p156은
    // p1693..p1697이 이 band에 속하고 p1698은 바로 뒤에서 제외되는 실물 사례다.
    let image_wrap_bottom_vpos = common
        .vertical_offset
        .saturating_add(common.height)
        .min(i32::MAX as u32) as i32;
    let wrap_target_para_indices = square_picture_wrap_band_target_paragraphs(
        paragraphs,
        para_idx + 1,
        reset_seg.vertical_pos,
        image_wrap_bottom_vpos,
        reset_seg.column_start,
        reset_seg.segment_width,
    );
    if wrap_target_para_indices.is_empty() {
        return None;
    }

    // Square는 layout cursor를 전진시키지 않지만, 이 저장 contract에서는 다음
    // page top의 그림+caption을 위해 현재 page tail에 적어도 그림 frame만큼의
    // 여유가 있어야 한다. image frame만으로도 기존 각주가 있는 p155에 들어가지
    // 않으므로 current PageItem을 만들지 않고 next-page queue로 보낸다.
    let image_frame_height = hwpunit_to_px(
        common.height as i32 + common.margin.top as i32 + common.margin.bottom as i32,
        dpi,
    );
    let caption_height = crate::renderer::layout::LayoutEngine::new(dpi)
        .calculate_caption_height(&picture.caption, styles);
    let caption_spacing = picture
        .caption
        .as_ref()
        .map(|caption| hwpunit_to_px(caption.spacing as i32, dpi))
        .unwrap_or(0.0);
    let frame_height = image_frame_height + caption_height + caption_spacing;
    (page.current_height + frame_height > available_height() + 0.5).then_some((
        wrap_target_para_indices,
        crate::renderer::pagination::WrapAnchorRef {
            anchor_para_index: para_idx,
            anchor_cs: reset_seg.column_start,
            anchor_sw: reset_seg.segment_width as i32,
            anchor_image_margin_right: common.margin.right as i32,
            band_y_range: None,
        },
    ))
}

/// 다음 physical page를 소유한 Square 그림의 저장 wrap band에 속하는 연속 문단을 찾는다.
///
/// 한 문단에 full-width tail과 `vpos=0` reset band가 공존할 수 있으므로 문단의 첫
/// LineSeg만 보지 않는다. 그림의 실제 세로 범위를 벗어나거나 cs/sw 계약이 달라지는
/// 첫 문단에서 즉시 멈춰, 뒤 일반 본문으로 wrap anchor가 전파되지 않게 한다.
pub(in crate::renderer::typeset) fn square_picture_wrap_band_target_paragraphs(
    paragraphs: &[Paragraph],
    first_para_index: usize,
    band_start_vpos: i32,
    band_end_vpos: i32,
    expected_column_start: i32,
    expected_segment_width: i32,
) -> Vec<usize> {
    if band_end_vpos <= band_start_vpos {
        return Vec::new();
    }

    let mut targets = Vec::new();
    for (para_index, para) in paragraphs.iter().enumerate().skip(first_para_index) {
        let belongs_to_band = para.line_segs.iter().any(|seg| {
            band_start_vpos <= seg.vertical_pos
                && seg.vertical_pos < band_end_vpos
                && seg.column_start == expected_column_start
                && (i64::from(seg.segment_width) - i64::from(expected_segment_width)).abs() <= 200
        });
        if !belongs_to_band {
            break;
        }
        targets.push(para_index);
    }
    targets
}
