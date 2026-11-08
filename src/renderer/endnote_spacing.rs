//! 실제 미주 점유 끝에서 간격을 한 번만 예약하는 공통 원점 계약.

use super::pagination::{EndnoteParaSource, PageItem};

/// 현재 단이 앞 미주의 마지막 줄을 소유하며, 다음 항목이 다른 미주의 시작인가.
/// 저장 원점의 변화나 제목 문자열은 공간 소유의 근거가 아니다.
pub(crate) fn starts_note_after_complete_tail(
    previous_source: Option<&EndnoteParaSource>,
    current_source: &EndnoteParaSource,
    current_item: &PageItem,
    previous_items: &[PageItem],
    previous_para_index: usize,
    previous_line_count: usize,
) -> bool {
    let Some(previous_source) = previous_source else {
        return false;
    };
    if current_source.note_para_index != 0
        || previous_line_count == 0
        || (
            previous_source.section_index,
            previous_source.para_index,
            previous_source.control_index,
        ) == (
            current_source.section_index,
            current_source.para_index,
            current_source.control_index,
        )
        || !matches!(
            current_item,
            PageItem::FullParagraph { .. } | PageItem::PartialParagraph { start_line: 0, .. }
        )
    {
        return false;
    }
    previous_items.iter().any(|item| match item {
        PageItem::FullParagraph { para_index } => *para_index == previous_para_index,
        PageItem::PartialParagraph {
            para_index,
            end_line,
            ..
        } => *para_index == previous_para_index && *end_line == previous_line_count,
        _ => false,
    })
}

/// 보이는 내용과 실제 흐름 줄 상자가 함께 소유하는 끝점이다.
/// 빈 줄도 줄 상자를 소비하며, 후행 간격은 두 입력에 포함하지 않는다.
pub(crate) fn occupied_end(content_end: Option<f64>, line_box_end: Option<f64>) -> Option<f64> {
    content_end
        .into_iter()
        .chain(line_box_end)
        .filter(|end| end.is_finite())
        .reduce(f64::max)
}

pub(crate) fn origin_after_gap(flow_origin: f64, occupied_end: Option<f64>, gap: f64) -> f64 {
    // 실제 점유 끝이 없으면 기존 흐름 기반 추정을 유지한다. 점유 결과가
    // 있으면 이미 소비한 trailing에 같은 미주 간격을 다시 더하지 않는다.
    occupied_end.map_or(flow_origin + gap, |end| (end + gap).max(flow_origin))
}

/// 원점 이동에 따른 부동소수점 반올림만 허용해 간격의 예약 여부를 판정한다.
pub(crate) fn gap_is_reserved(flow_origin: f64, occupied_end: f64, gap: f64) -> bool {
    let end_with_gap = occupied_end + gap;
    let roundoff = (flow_origin.abs() + occupied_end.abs() + gap.abs()) * f64::EPSILON;
    end_with_gap <= flow_origin + roundoff
}

/// 인라인 제어 항목도 같은 호스트가 실제 소비한 글줄 상자의 끝을 이어받는다.
/// 다른 문단·단이나 독립 개체의 점유를 호스트 글줄로 누적하지 않는다.
#[derive(Default)]
pub(crate) struct ParagraphLineBoxEnd {
    owner: Option<usize>,
    bottom: Option<f64>,
}

impl ParagraphLineBoxEnd {
    pub(crate) fn after_item(
        &mut self,
        item: &PageItem,
        is_inline_control: bool,
        line_bottom: f64,
    ) -> Option<f64> {
        let measured = line_bottom.is_finite().then_some(line_bottom);
        match item {
            PageItem::FullParagraph { para_index }
            | PageItem::PartialParagraph { para_index, .. } => {
                self.owner = Some(*para_index);
                self.bottom = measured;
            }
            PageItem::Shape { para_index, .. }
                if is_inline_control && self.owner == Some(*para_index) =>
            {
                // Shape의 방출 순서는 이미 소비한 호스트 빈 줄을 지우지 않는다.
                self.bottom = occupied_end(self.bottom, measured);
            }
            _ => {
                self.owner = None;
                self.bottom = measured;
            }
        }
        self.bottom
    }
}
