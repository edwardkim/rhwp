//! 텍스트 편집이 원본 줄의 수직 프레임을 실제로 바꿨는지 확인한다.

use crate::model::paragraph::{LineSeg, Paragraph};

#[derive(PartialEq, Eq)]
pub(super) struct StoredVerticalFrame {
    lines: Vec<(i32, i32, i32)>,
    controls: usize,
    empty: bool,
    whitespace_only: bool,
}

impl StoredVerticalFrame {
    /// 생성 줄이나 무효화한 원본은 보존 근거가 아니다.
    pub(super) fn capture(paragraph: &Paragraph) -> Option<Self> {
        if paragraph.line_segs.is_empty()
            || paragraph.stored_text_partition_is_dirty()
            || paragraph.cell_format_vpos_dirty
            || paragraph.line_segs.iter().any(|line| {
                line.tag & LineSeg::TAG_IMPLEMENTATION_PROPERTY != 0
                    || line.vertical_pos < 0
                    || line.line_height <= 0
                    || line.line_spacing < 0
            })
        {
            return None;
        }
        Some(Self {
            lines: paragraph
                .line_segs
                .iter()
                .map(|line| (line.vertical_pos, line.line_height, line.line_spacing))
                .collect(),
            controls: paragraph.controls.len(),
            empty: paragraph.text.is_empty(),
            whitespace_only: paragraph.text.trim().is_empty(),
        })
    }

    /// 문자 시작점은 달라도 같은 수직 줄 상자·소유는 저장 컷을 바꾸지 않는다.
    pub(super) fn matches(&self, paragraph: &Paragraph) -> bool {
        Self::capture(paragraph).as_ref() == Some(self)
    }
}
