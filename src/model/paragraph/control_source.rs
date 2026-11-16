//! 원문 글자를 차지하는 제어와 글자 앞 갭에 있는 제어의 공통 주소.

use super::{Paragraph, CTRL_CHAR_CODE_UNITS};

pub(crate) struct ControlSourcePosition {
    pub char_index: usize,
    pub utf16: u32,
    pub owns_char: bool,
}

impl Paragraph {
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
                        // 이 자리까지의 제목 차례 표시·짝 없는 필드 끝은 CTRL_HEADER 없이 8유닛
                        // 슬롯을 차지하고 같은 자리의 제어보다 먼저 놓인다(직렬화와 같은 순서).
                        // 빼지 않으면 뒤 제어가 한 슬롯씩 앞 줄에 매핑된다(#9016).
                        let leading_markers = self
                            .title_marks
                            .iter()
                            .filter(|mark| mark.char_idx <= at)
                            .count()
                            + self
                                .orphan_field_ends
                                .iter()
                                .filter(|end| end.char_idx <= at)
                                .count();
                        (leading_markers as u32 + group_start as u32) * CTRL_CHAR_CODE_UNITS
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
            for index in group_start..group_end {
                storage.push(
                    &mut result,
                    ControlSourcePosition {
                        char_index: at,
                        utf16: first + (index - group_start) as u32 * CTRL_CHAR_CODE_UNITS,
                        owns_char: index + 1 == group_end && (owns_char || single_unit_marker),
                    },
                )?;
            }
            group_start = group_end;
        }
        Ok(result)
    }
}
