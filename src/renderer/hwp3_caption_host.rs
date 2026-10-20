//! 원본 HWP3의 빈 caption 그림 host가 별도로 점유하는 논리 줄.
//!
//! HWP3의 그림 자리차지 LineSeg를 제거한 뒤에도 실제 글자 줄은 남는다.
//! 이 띠는 그림·캡션의 여백이나 잉크가 아니므로 조판 예약과 배치 흐름이
//! 같은 값을 소비한다. 변환 HWP5/HWPX는 다음 독립 문단의 저장 사다리가
//! 같은 host 줄을 증명하는 경우에만 이 점유를 이어받는다.

use crate::model::image::Picture;
use crate::model::paragraph::Paragraph;
use crate::model::provenance::LayoutCompatibilityProfile;
use crate::model::shape::{CaptionDirection, TextWrap, VertRelTo};

pub(crate) fn line_advance_hwp(
    paragraph: &Paragraph,
    picture: &Picture,
    profile: LayoutCompatibilityProfile,
    next_paragraph: Option<&Paragraph>,
) -> i32 {
    if profile.session_edited()
        || picture.common.treat_as_char
        || picture.caption.is_none()
        || picture.common.text_wrap != TextWrap::TopAndBottom
        || picture.common.vert_rel_to != VertRelTo::Para
        || paragraph.controls.len() != 1
        || paragraph.line_segs.len() != 1
        || paragraph
            .text
            .chars()
            .any(|c| c > '\u{001f}' && c != '\u{fffc}')
    {
        return 0;
    }
    let line = &paragraph.line_segs[0];
    let advance = line.line_height.saturating_add(line.line_spacing);
    if profile.hwp3_native_layout() {
        return advance;
    }
    // 저장 HWP5의 그림 뒤 문단은 자체 줄과 별도로 host의 한 줄도 소유할 수 있다.
    // 컨테이너/변환 계보만으로 원본 HWP3 규칙을 켜지 않는다. 아래 캡션·문단 고정
    // 그림과 독립 successor가 같은 저장 줄 전진을 증명할 때만 공유한다.
    let Some(next) = next_paragraph else {
        return 0;
    };
    if !profile.hwp5_stored_pagination_layout()
        || !picture.common.flow_with_text
        || !picture.common.allow_overlap
        || !picture
            .caption
            .as_ref()
            .is_some_and(|caption| caption.direction == CaptionDirection::Bottom)
        || paragraph.stored_text_partition_is_dirty()
        || next.stored_text_partition_is_dirty()
        || !next.controls.is_empty()
        || next.line_segs.len() != 1
        || advance <= 0
    {
        return 0;
    }
    let successor = &next.line_segs[0];
    let synthetic = crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY;
    if line.tag & synthetic != 0
        || successor.tag & synthetic != 0
        || line.segment_width == 0
        || successor.line_height <= 0
    {
        return 0;
    }
    let origin = |para: &Paragraph| {
        para.source_line_seg_vertical_pos
            .as_ref()
            .and_then(|source| source.first().copied())
            .unwrap_or(para.line_segs[0].vertical_pos)
    };
    if i64::from(origin(next)) - i64::from(origin(paragraph)) == i64::from(advance) {
        advance
    } else {
        0
    }
}

/// 저장 사다리가 증명한 caption host의 물리 흐름 끝.
/// 그림은 이미 전진한 빈 글줄 커서가 아니라 원본 문단 앵커에서 배치된다.
/// 예약과 paint가 같은 앵커·그림/캡션 상자·바깥여백·host 줄을 한 번씩 소비한다.
pub(crate) fn stored_flow_end_px(
    paragraph: &Paragraph,
    picture: &Picture,
    profile: LayoutCompatibilityProfile,
    next_paragraph: Option<&Paragraph>,
    paragraph_origin: f64,
    dpi: f64,
) -> Option<f64> {
    if !profile.hwp5_stored_pagination_layout() {
        return None;
    }
    let host_line = line_advance_hwp(paragraph, picture, profile, next_paragraph);
    if host_line <= 0 {
        return None;
    }
    let offset = crate::renderer::float_placement::signed_hwpunit(picture.common.vertical_offset);
    Some(
        paragraph_origin
            + crate::renderer::hwpunit_to_px(offset, dpi)
            + crate::renderer::hwpunit_to_px(i32::from(picture.common.margin.top), dpi)
            + crate::renderer::composer::captioned_picture_height_px(picture, dpi)
            + crate::renderer::hwpunit_to_px(i32::from(picture.common.margin.bottom), dpi)
            + crate::renderer::hwpunit_to_px(host_line, dpi),
    )
}
