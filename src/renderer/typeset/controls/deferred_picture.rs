//! 다음 물리 쪽의 자리차지 그림 소유 후보를 조회한다.
//! 저장 줄·각주 예약·그림/캡션 요구 높이를 조회하며 큐와 페이지 상태는 변경하지 않는다.
//! 가용 높이는 기존 두 분기의 지연 호출로 읽는다. 페이지 전이와 발행은 R5 소유다.

use super::super::{is_synthetic_line_seg, para_has_visible_text};
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

/// 저장 후보만 반환한다. 앵커 본문 배치와 그림 큐 반영은 호출자에 남긴다.
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
    if !(page.profile.hwp5_stored_pagination_layout() || page.profile.hwpx_stored_layout())
        || page.profile.session_edited()
        || para.stored_text_partition_is_dirty()
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

    // 다음 문단의 vpos=0 좁은 띠는 한컴 저장 흐름에서 그림의 다음 물리 쪽 소유를
    // 직접 가리킨다. 같은 문단의 전폭 줄 뒤에서 재개하는 경우와 다음 문단이
    // 좁은 띠로 시작하는 경우를 모두 수용하되, 이 저장 형상이 없는 일반
    // 자리차지 그림을 옮기지 않는다.
    let next_para = paragraphs.get(para_idx + 1)?;
    if next_para.stored_text_partition_is_dirty() {
        return None;
    }
    let (reset_idx, reset_seg) = next_para.line_segs.iter().enumerate().find(|(_, seg)| {
        !is_synthetic_line_seg(seg)
            && seg.vertical_pos == 0
            && seg.column_start == 0
            && seg.segment_width > 0
            && (seg.segment_width as i32 - common.horizontal_offset as i32).abs() <= 200
    })?;
    let has_full_width_before_reset = reset_idx > 0
        && next_para.line_segs[..reset_idx]
            .iter()
            .any(|seg| seg.segment_width > reset_seg.segment_width.saturating_add(1000));
    // 다음 문단이 좁은 띠로 시작하면 그 문단 전체의 저장 진행 높이가 현재 각주
    // 예약 뒤의 가용 높이를 초과해야만 다음 물리 쪽 소유로 확정한다. 이 조건이
    // 없으면 단순히 그림 옆을 흐르는 문단을 이후의 무관한 쪽 나눔에 묶을 수 있다.
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
    // 같은 cs/sw인 문단이라도 이 범위를 넘어가면 다음 일반 본문까지 어울림이
    // 새어 나간다. 반대로 빈 안내 문단은 글자가 없어도 뒤의 보이는 문단에
    // 어울림 계약을 전달하므로 포함한다. #3821의156쪽은 문단1693–1697이
    // 이 띠에 속하고 문단1698은 바로 뒤에서 제외되는 실물 사례다.
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

    // 자리차지 그림은 배치 커서를 전진시키지 않지만 이 저장 계약에서는 그림과
    // 캡션을 담을 물리 공간이 필요하다. 기존 각주를 제외한 현재 쪽 말미에
    // 그림 프레임조차 들어가지 않으면 현재 PageItem을 만들지 않고 다음 쪽
    // 대기열로 보낸다.
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

/// 다음 물리 쪽을 소유한 자리차지 그림의 저장 어울림 띠에 속하는 연속 문단을 찾는다.
///
/// 한 문단에 전폭 꼬리 줄과 `vpos=0` 재개 띠가 공존할 수 있으므로 첫 LineSeg만
/// 보지 않는다. 그림의 실제 세로 범위를 벗어나거나 cs/sw 계약이 달라지는 첫
/// 문단에서 멈춰, 뒤 일반 본문으로 어울림 앵커가 전파되지 않게 한다.
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
