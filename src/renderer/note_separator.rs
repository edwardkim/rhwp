//! 미주 구분선의 중심 경로와 위·아래 여백에서 정해지는 공통 흐름 높이.

use super::{hwpunit_to_px, layout::border_width_to_px};

/// 색 없음은 잉크만 없앤다. 선언된 구분선 여백과 주석 흐름은 유지한다.
pub(crate) fn has_ink(color: crate::model::ColorRef) -> bool {
    color != crate::model::color::NONE
}

/// 경계 바로 앞에서 실제로 마지막 줄까지 소비한 본문의 줄간격.
/// 다른 단의 꼬리나 표·그림 뒤의 이전 글줄 간격을 거슬러 회수하지 않는다.
pub(crate) fn completed_body_tail_spacing_hu(
    items: &[super::pagination::PageItem],
    paragraphs: &[crate::model::paragraph::Paragraph],
) -> i32 {
    use super::pagination::PageItem;
    let (index, end_line) = match items.last() {
        Some(PageItem::FullParagraph { para_index }) => (*para_index, None),
        Some(PageItem::PartialParagraph {
            para_index,
            end_line,
            ..
        }) => (*para_index, Some(*end_line)),
        _ => return 0,
    };
    let Some(paragraph) = paragraphs.get(index) else {
        return 0;
    };
    if end_line.is_some_and(|end| end != paragraph.line_segs.len()) {
        return 0;
    }
    paragraph
        .line_segs
        .last()
        .map_or(0, |line| line.line_spacing.max(0))
}

pub(crate) struct NoteSeparatorGeometry {
    pub(crate) center_offset: f64,
    pub(crate) width: f64,
    pub(crate) advance: f64,
}

impl NoteSeparatorGeometry {
    pub(crate) fn new(above_hu: i32, below_hu: i32, visible: bool, width: u8, dpi: f64) -> Self {
        let width = if visible {
            border_width_to_px(width).max(0.5)
        } else {
            0.0
        };
        let center_offset = hwpunit_to_px(above_hu, dpi);
        // upstream의 독립 한컴 출력 계약: 선 굵기는 흐름 높이에 더하지 않는다.
        // 측정과 실제 배치는 같은 위·아래 여백의 합을 소비한다.
        let advance = center_offset + hwpunit_to_px(below_hu, dpi);
        Self {
            center_offset,
            width,
            advance,
        }
    }
}
