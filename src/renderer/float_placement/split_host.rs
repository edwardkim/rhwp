//! 재조판 호스트의 글줄 사이에 놓이는 자리차지 표의 소유·물리 상자.
use super::{
    hwpunit_to_px, is_para_topbottom_float, signed_hwpunit, Paragraph, ParagraphFloatFlow,
    ParagraphFloatPlacement, ParagraphHostLine, Table, VertAlign,
};

impl ParagraphFloatPlacement {
    pub(crate) fn split_host_prefix(self) -> Option<usize> {
        match self.flow {
            ParagraphFloatFlow::SplitHost { prefix_lines } => Some(prefix_lines),
            _ => None,
        }
    }

    pub(crate) fn advances_paragraph_flow(self) -> bool {
        matches!(
            self.flow,
            ParagraphFloatFlow::NextLine | ParagraphFloatFlow::SplitHost { .. }
        )
    }

    /// 개체 소속 줄이 표와 충돌하면 앞선 글줄을 보존하고 그 줄부터 표 뒤에 잇는다.
    /// 앞 글줄이 없으면 같은 계획이 표와 호스트 전체의 소유·예산을 처리한다.
    /// 원본 저장 좌표가 아니라 현재 문자 경계·줄 상자만 소비한다.
    pub(crate) fn from_split_computed_host(
        para: &Paragraph,
        table: &Table,
        control_index: usize,
        text_origin: f64,
        lines: &[ParagraphHostLine],
        table_height: f64,
        dpi: f64,
    ) -> Option<Self> {
        let position = Self::text_tail_control_position(para, control_index)
            .or_else(|| Self::text_head_control_position(para, control_index))?;
        let text_len = para.text.chars().count();
        if !dpi.is_finite()
            || dpi <= 0.0
            || !text_origin.is_finite()
            || !table_height.is_finite()
            || table_height <= 0.0
            || !is_para_topbottom_float(&table.common)
            || table.common.vert_align != VertAlign::Top
            || table.common.allow_overlap
            || lines
                .first()
                .is_none_or(|line| line.char_start != 0 || line.top != 0.0)
            || lines.iter().any(|line| {
                !line.top.is_finite()
                    || !line.height.is_finite()
                    || line.height <= 0.0
                    || line.char_start > text_len
            })
            || lines.windows(2).any(|pair| {
                pair[1].char_start <= pair[0].char_start
                    || pair[1].top < pair[0].top + pair[0].height
            })
        {
            return None;
        }
        let prefix_lines = lines.iter().rposition(|line| line.char_start <= position)?;
        let anchor = lines.get(prefix_lines)?;
        let offset = hwpunit_to_px(signed_hwpunit(table.common.vertical_offset), dpi);
        // 첫 줄 앞의 0/음수 앵커에는 보호할 prefix가 없다. 그 표의 기존
        // 흐름·되감김 소유자를 유지하고, 양수 오프셋의 빈 공간만 여기서 판정한다.
        if prefix_lines == 0 && offset <= 0.0 {
            return None;
        }
        // 모든 글줄이 표 앞에 들어가면 기존 소유 경로가 처리한다.
        if offset >= anchor.height {
            return None;
        }
        let outer_top = hwpunit_to_px(i32::from(table.outer_margin_top), dpi);
        let outer_bottom = hwpunit_to_px(i32::from(table.outer_margin_bottom), dpi);
        if outer_top < 0.0 || outer_bottom < 0.0 {
            return None;
        }
        let prefix_end = text_origin + anchor.top;
        let proposed_top = prefix_end + offset;
        let proposed_end = proposed_top + outer_top + table_height + outer_bottom;
        if proposed_end <= text_origin {
            return None;
        }
        // 앞 글줄이 이미 점유한 흐름 띠와 겹치는 상자만 그 띠 다음에 수용한다.
        // 단순 좌표 보정이 아니라 prefix의 실제 예약과 표 외곽 상자를 함께 닫는다.
        let outside_top = if proposed_top < prefix_end {
            prefix_end
        } else {
            proposed_top
        };
        let table_top = outside_top + outer_top;
        let occupied_bottom = table_top + table_height + outer_bottom;
        if !occupied_bottom.is_finite() {
            return None;
        }
        Some(Self {
            flow: ParagraphFloatFlow::SplitHost { prefix_lines },
            anchor_y: prefix_end,
            stored_host_origin: Some(text_origin),
            stored_successor_line_origin: None,
            table_left: None,
            table_top,
            occupied_bottom,
        })
    }
}
