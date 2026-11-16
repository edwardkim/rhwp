//! 원문 글자를 차지하는 제어와 글자 앞 갭에 있는 제어의 공통 주소.

use super::{Paragraph, CTRL_CHAR_CODE_UNITS};

pub(crate) struct ControlSourcePosition {
    pub char_index: usize,
    pub utf16: u32,
    pub owns_char: bool,
}

impl Paragraph {
    /// 마지막 글자 뒤(글자 없는 문단이면 문단 전체)에서 `control_index` 번째 제어보다 앞에
    /// 놓이는 `controls[]` 밖 8유닛 슬롯의 수.
    ///
    /// 누름틀 끝·다단락 누름틀 끝·제목 차례 표시는 CTRL_HEADER 없이 원시 스트림의 8유닛을
    /// 차지한다. 놓이는 순서는 직렬화와 같다(`serializer/body_text.rs`): 다단락 누름틀 끝과
    /// 제목 차례 표시가 제어들보다 먼저 오고, 같은 문단에서 연 누름틀의 끝은 그 누름틀이 감싼
    /// 마지막 제어(`control_idx + inner_slot_count`) 바로 뒤에 온다. 한/글이 저장한 줄 시작
    /// (`LineSeg::text_start`)도 이 슬롯을 센 값이다.
    pub(crate) fn trailing_auxiliary_slots_before(&self, control_index: usize) -> u32 {
        self.trailing_auxiliary_slots(control_index, self.text.chars().count())
    }

    fn trailing_auxiliary_slots(&self, control_index: usize, text_len: usize) -> u32 {
        let leading = self
            .title_marks
            .iter()
            .filter(|mark| mark.char_idx >= text_len)
            .count()
            + self
                .orphan_field_ends
                .iter()
                .filter(|end| end.char_idx >= text_len)
                .count();
        let closed = self
            .field_ranges
            .iter()
            .filter(|range| {
                range.end_char_idx >= text_len
                    && range.control_idx.saturating_add(range.inner_slot_count) < control_index
            })
            .count();
        (leading + closed) as u32
    }

    pub(crate) fn control_source_positions(&self) -> Vec<ControlSourcePosition> {
        match self.control_source_positions_with_storage(&crate::storage::Unbounded) {
            Ok(value) => value,
            Err(never) => match never {},
        }
    }
    pub(crate) fn control_source_positions_with_storage<S: crate::storage::Storage>(
        &self,
        storage: &S,
    ) -> Result<Vec<ControlSourcePosition>, S::Error> {
        let mut positions = self.try_control_text_positions(storage)?;
        let chars = storage.collect(self.text.chars())?;
        // 끝 자동 번호의 자리표에는 다음 글자 오프셋이 없다. 편집·저장도
        // 같은 판정으로 마지막 공백이 차지하는 8유닛을 보존한다.
        let trailing_number = self.trailing_auto_number(&positions);
        if let Some(index) = trailing_number {
            positions[index] = chars.len() - 1;
        }
        let text_end = self
            .char_offsets
            .last()
            .zip(chars.last())
            .map(|(offset, ch)| {
                offset
                    + if trailing_number.is_some() || *ch == '\t' {
                        CTRL_CHAR_CODE_UNITS
                    } else {
                        ch.len_utf16() as u32
                    }
            })
            .unwrap_or_else(|| chars.iter().map(|ch| Self::char_stream_len(*ch)).sum());
        let complete_empty_stream = chars.is_empty()
            && self.char_offsets.is_empty()
            && self.char_count >= (self.controls.len() as u32).saturating_mul(CTRL_CHAR_CODE_UNITS);
        let mut result = storage.vec(positions.len())?;
        let mut group_start = 0;
        while group_start < positions.len() {
            let at = positions[group_start];
            let mut group_end = group_start + 1;
            while positions.get(group_end) == Some(&at) {
                group_end += 1;
            }
            let count = (group_end - group_start) as u32;
            // 한 원문 글자에는 제어 하나만 들어간다. 앞선 같은 자리의
            // 제어들은 그 글자보다 앞선 갭을 차례대로 점유한다.
            let owns_char = self.control_owns_char(group_end - 1, at, &chars, &self.char_offsets);
            let single_unit_marker = count == 1
                && chars.get(at) == Some(&'\u{FFFC}')
                && self.char_offsets.get(at).is_some_and(|offset| {
                    let previous_end = at
                        .checked_sub(1)
                        .and_then(|i| self.char_offsets.get(i).zip(chars.get(i)))
                        .map_or(0, |(start, ch)| *start + ch.len_utf16() as u32);
                    offset.saturating_sub(previous_end) < CTRL_CHAR_CODE_UNITS
                });
            let first = self.char_offsets.get(at).map_or_else(
                || {
                    if complete_empty_stream {
                        group_start as u32 * CTRL_CHAR_CODE_UNITS
                    } else {
                        text_end
                    }
                },
                |offset| {
                    if single_unit_marker {
                        *offset
                    } else {
                        offset.saturating_sub((count - u32::from(owns_char)) * CTRL_CHAR_CODE_UNITS)
                    }
                },
            );
            // 마지막 글자 뒤(글자 없는 완전 스트림이면 문단 전체)의 제어는 그 앞에 놓인
            // `controls[]` 밖 슬롯만큼 뒤에 있다. 세지 않으면 뒤 제어가 한 슬롯씩 앞 줄에
            // 매핑된다(#9016, #9058).
            let follows_text = complete_empty_stream
                || (!chars.is_empty()
                    && at >= chars.len()
                    && self.char_offsets.len() == chars.len());
            for index in group_start..group_end {
                let auxiliary = if follows_text {
                    self.trailing_auxiliary_slots(index, chars.len())
                } else {
                    0
                };
                storage.push(
                    &mut result,
                    ControlSourcePosition {
                        char_index: at,
                        utf16: first
                            + ((index - group_start) as u32 + auxiliary) * CTRL_CHAR_CODE_UNITS,
                        owns_char: index + 1 == group_end && (owns_char || single_unit_marker),
                    },
                )?;
            }
            group_start = group_end;
        }
        Ok(result)
    }
}
