//! 저장된 두 문단의 줄 경계가 연속인지 확인한다.
//!
//! 새로 만든 줄 좌표와 편집으로 무효가 된 저장 분할은 독립적인
//! 배치 근거가 아니다. 유효한 저장 경계만 실제 소비한 흐름에 연결한다.

use crate::model::control::Control;
use crate::model::paragraph::{LineSeg, Paragraph};

pub(super) fn continuous(previous: &Paragraph, current: &Paragraph) -> bool {
    if previous.stored_text_partition_is_dirty() || current.stored_text_partition_is_dirty() {
        return false;
    }
    let Some((last, first)) = previous.line_segs.last().zip(current.line_segs.first()) else {
        return false;
    };
    if last.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
        || first.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
    {
        return false;
    }
    first.vertical_pos > last.vertical_pos
        && first.vertical_pos
            == last
                .vertical_pos
                .saturating_add(last.line_height)
                .saturating_add(last.line_spacing)
}
/// 이전 저장 축보다 아래에서 새로 시작한 일반 본문의 권위 있는 첫 줄.
/// 축 안의 되감김과 편집/합성 정보는 새 원점의 근거로 사용하지 않는다.
pub(super) fn restart_below_origin(
    previous: &Paragraph,
    current: &Paragraph,
    base: i32,
) -> Option<i32> {
    let valid = |para: &Paragraph| {
        !para.stored_text_partition_is_dirty()
            // 각주·미주 표식은 저장 글줄 안에 포함된다. 별도 본문 개체
            // 프레임을 소유하지 않으므로 정상 저장 축의 재시작을 막지 않는다.
            && para
                .controls
                .iter()
                .all(|control| matches!(control, Control::Footnote(_) | Control::Endnote(_)))
            && !para.line_segs.is_empty()
            && para.line_segs.iter().all(|line| {
                line.vertical_pos >= 0 && line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
            })
            && para
                .line_segs
                .windows(2)
                .all(|pair| pair[1].vertical_pos >= pair[0].vertical_pos)
    };
    if !valid(previous) || !valid(current) {
        return None;
    }
    let first = current.line_segs.first()?.vertical_pos;
    (first < base && previous.line_segs.last()?.vertical_pos >= base).then_some(first)
}
