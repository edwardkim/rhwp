//! 주석 제어의 원시 줄 소속과 그 줄에서의 표시 경계를 함께 계산한다.

use super::{control_slot_line_index, stored_line_box_height, ComposedParagraph, Paragraph};
use crate::model::paragraph::LineSeg;

#[derive(Default)]
pub(crate) struct NoteLine {
    pub positions: Vec<(usize, u16, usize)>,
    /// 개행 뒤 제어 슬롯을 같은 저장 줄이 소유하면 개행은 물리 줄의 끝이 아니다.
    pub follows_break: bool,
    pub trailing_slots: usize,
}

pub(crate) struct NoteLineSlots {
    pub lines: Vec<NoteLine>,
}

impl NoteLineSlots {
    pub fn new(composed: &ComposedParagraph, source: Option<&Paragraph>) -> Self {
        let mut lines: Vec<NoteLine> = composed.lines.iter().map(|_| NoteLine::default()).collect();
        if lines.is_empty() || composed.footnote_positions.is_empty() {
            return Self { lines };
        }
        // 줄을 새로 분할했거나 서로 다른 출처의 사다리를 섞었으면 원시 줄을 투영하지 않는다.
        let stored = source.filter(|para| {
            composed.lines.len() == para.line_segs.len()
                && composed
                    .frame_geometry
                    .as_ref()
                    .is_none_or(|frame| frame.stored_partition)
                && composed.lines.iter().zip(&para.line_segs).enumerate().all(
                    |(index, (line, seg))| {
                        seg.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY == 0
                            && line.line_height == stored_line_box_height(seg)
                            && line.segment_width == seg.segment_width
                            && line.char_start
                                == para.char_offsets.partition_point(|&offset| {
                                    offset < para.line_seg_text_start(index)
                                })
                    },
                )
        });
        for &(position, number, control) in &composed.footnote_positions {
            let stored_owner = stored.and_then(|para| control_slot_line_index(para, control));
            let Some(owner) = stored_owner.or_else(|| {
                composed
                    .lines
                    .iter()
                    .rposition(|line| line.char_start <= position)
            }) else {
                continue;
            };
            let line = &composed.lines[owner];
            let text_end = line.char_start
                + line
                    .runs
                    .iter()
                    .map(|run| run.source_scalar_count())
                    .sum::<usize>();
            // 다음 가시 글자의 주소로 압축된 제어는 자기 원시 줄의 텍스트 뒤에 그린다.
            // 원문/논리 주소 자체는 바꾸지 않는다. 측정·배치·간격 배분이 이 경계를 공유한다.
            let owns_boundary = stored_owner.is_some()
                && composed
                    .lines
                    .get(owner + 1)
                    .is_some_and(|next| position >= next.char_start);
            let display_position = if owns_boundary { text_end } else { position };
            let row = &mut lines[owner];
            row.follows_break |= owns_boundary && line.has_line_break && position > text_end;
            row.trailing_slots += usize::from(display_position == text_end);
            row.positions.push((display_position, number, control));
        }
        Self { lines }
    }
}
