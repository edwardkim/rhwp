//! 원본 TAC 저장 프레임의 소유 줄·장식·후행 간격 판정. 측정과 실제 배치가 공유한다.

use super::control_line_seg_index;
use crate::model::{control::Control, paragraph::Paragraph, shape::TextWrap, table::Table};

/// 앞선 글앞 도형은 호스트 줄의 흐름을 점유하지 않는다. 빈 문단에서 이 도형들 뒤의
/// TAC 표가 별도 저장 줄을 갖는 경우, 표의 소유 줄 `vpos`가 실제 페인트 기준이다.
/// Picture, 글자처럼 도형, 글뒤/어울림 도형은 #4622의 이중 가산 반증 범위이므로 제외한다.
pub(super) fn tac_has_only_in_front_decoration_shapes_before(
    para: &Paragraph,
    control_index: usize,
) -> bool {
    let before = &para.controls[..control_index.min(para.controls.len())];
    !before.is_empty()
        && before.iter().all(|control| {
            matches!(control, Control::Shape(shape)
            if !shape.common().treat_as_char
                && matches!(
                    shape.common().text_wrap,
                    crate::model::shape::TextWrap::InFrontOfText
                ))
        })
}

/// TAC 표 앞 컨트롤이 모두 흐름을 전진시키지 않는 장식 개체(글앞·글뒤의 비-TAC
/// 그림·도형)인지 판정한다. 가시성은 기준이 아니다 — 글뒤 그림은 보이지만 줄을
/// 점유하지 않는다. 자리차지·어울림 개체와 글자처럼 개체는 흐름 참여자라 제외한다.
fn tac_has_only_non_flow_decorations_before(para: &Paragraph, control_index: usize) -> bool {
    let before = &para.controls[..control_index.min(para.controls.len())];
    !before.is_empty()
        && before.iter().all(|control| {
            let common = match control {
                Control::Shape(shape) => shape.common(),
                Control::Picture(picture) => &picture.common,
                _ => return false,
            };
            !common.treat_as_char
                && matches!(
                    common.text_wrap,
                    crate::model::shape::TextWrap::InFrontOfText
                        | crate::model::shape::TextWrap::BehindText
                )
        })
}

/// 글앞 장식 뒤의 TAC 표가 물리 표 상자까지 담은 저장 줄과 음수 줄간격을 함께
/// 기록한 경우, 표의 paint 높이에는 줄간격이 없지만 뒤 문단의 flow 예약에는 한 번
/// 포함된다. 이 음수분을 다시 예약하면 표 뒤 본문이 글꼴 높이만큼 아래로 밀린다.
/// 실제 표 소유 줄, 바깥 여백 포함 줄높이, 글앞 장식이라는 세 조건을 모두 확인해
/// 일반 TAC/도형의 저장 줄 계약에는 적용하지 않는다 (#7333 p13·p16~21).
pub(crate) fn tac_in_front_decoration_fixed_line_spacing_deduction_hu(
    para: &Paragraph,
    control_index: usize,
) -> Option<i32> {
    let Control::Table(table) = para.controls.get(control_index)? else {
        return None;
    };
    if !table.common.treat_as_char
        || !matches!(table.common.text_wrap, TextWrap::TopAndBottom)
        || !tac_has_only_in_front_decoration_shapes_before(para, control_index)
        || table.common.height >= 0x8000_0000
    {
        return None;
    }
    let seg = para
        .line_segs
        .get(control_line_seg_index(para, control_index)?)?;
    let outer_box_height = i64::from(table.common.height)
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    (seg.line_spacing < 0 && i64::from(seg.line_height) >= outer_box_height - 10)
        .then_some(-seg.line_spacing)
}

/// 저장 표 소유 줄과 후속 문단이 인증한 원본 끝. fit과 paint가 같은 줄·간격 판정을 쓴다.
pub(crate) fn stored_empty_full_band_tac_successor_vpos_hu(
    para: &Paragraph,
    control_index: usize,
    table: &Table,
    following: &Paragraph,
) -> Option<i32> {
    if !para.text.trim().is_empty()
        || !table.common.treat_as_char
        || !matches!(table.common.text_wrap, TextWrap::TopAndBottom)
        || para.stored_text_partition_dirty
        || !tac_has_only_non_flow_decorations_before(para, control_index)
    {
        return None;
    }

    let synth = crate::model::paragraph::LineSeg::TAG_IMPLEMENTATION_PROPERTY;
    let owner = para
        .line_segs
        .get(control_line_seg_index(para, control_index)?)
        .filter(|line| line.tag & synth == 0)?;
    let following = following
        .line_segs
        .iter()
        .find(|line| line.tag & synth == 0)?;
    let outer_box_height_hu = i64::from(table.common.height)
        + i64::from(table.outer_margin_top)
        + i64::from(table.outer_margin_bottom);
    if outer_box_height_hu <= 0 || (i64::from(owner.text_height) - outer_box_height_hu).abs() > 8 {
        return None;
    }

    let expected_next_vpos = i64::from(owner.vertical_pos)
        + i64::from(owner.text_height)
        + i64::from(owner.line_spacing);
    if following.vertical_pos <= owner.vertical_pos
        || (i64::from(following.vertical_pos) - expected_next_vpos).abs() > 8
    {
        return None;
    }

    Some(following.vertical_pos)
}
