//! 원본 HWP3의 빈 caption 그림 host가 별도로 점유하는 논리 줄.
//!
//! HWP3의 그림 자리차지 LineSeg를 제거한 뒤에도 실제 글자 줄은 남는다.
//! 이 띠는 그림·캡션의 여백이나 잉크가 아니므로 조판 예약과 배치 흐름이
//! 같은 값을 소비한다. 변환 HWP5/HWPX의 저장 successor 원점은 이 계약이 아니다.

use crate::model::image::Picture;
use crate::model::paragraph::Paragraph;
use crate::model::provenance::LayoutCompatibilityProfile;
use crate::model::shape::{TextWrap, VertRelTo};

pub(crate) fn line_advance_hwp(
    paragraph: &Paragraph,
    picture: &Picture,
    profile: LayoutCompatibilityProfile,
) -> i32 {
    if !profile.hwp3_native_layout()
        || profile.session_edited()
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
    line.line_height.saturating_add(line.line_spacing)
}
