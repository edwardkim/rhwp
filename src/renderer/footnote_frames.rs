//! 각주 소유 단과 통단·오른쪽 단 정책을 예약·측정·배치에서 함께 사용한다.
use super::page_layout::{LayoutRect, PageLayoutInfo};
use crate::model::footnote::FootnotePlacement;

#[derive(Clone, Copy)]
pub(crate) struct FramePolicy {
    pub placement: FootnotePlacement,
    pub columns: u16,
}

impl FramePolicy {
    pub fn target(self, owner: u16) -> u16 {
        match self.placement {
            FootnotePlacement::BelowText => 0,
            FootnotePlacement::RightColumn => self.columns.saturating_sub(1),
            FootnotePlacement::EachColumn => owner.min(self.columns.saturating_sub(1)),
        }
    }

    pub fn spans_page(self) -> bool {
        self.columns <= 1 || self.placement == FootnotePlacement::BelowText
    }

    /// 각 단 아래 각주가 단 영역을 넘치면 같은 쪽 다음 단 각주 영역에 잇는다(한/글 12.30).
    pub fn continues_in_next_column(self) -> bool {
        self.placement == FootnotePlacement::EachColumn && self.columns > 1
    }

    pub fn charges_column(self, column: u16) -> bool {
        self.placement != FootnotePlacement::RightColumn || column + 1 >= self.columns
    }

    pub fn area(self, layout: &PageLayoutInfo, owner: u16) -> LayoutRect {
        if self.spans_page() {
            layout.body_area
        } else {
            layout
                .column_areas
                .get(usize::from(self.target(owner)))
                .copied()
                .unwrap_or(layout.body_area)
        }
    }
}

/// 구분선과 주석 사이 간격을 포함하는 한 각주 프레임의 누적 예약.
#[derive(Clone, Copy, Default)]
pub(crate) struct Reservation {
    pub height: f64,
    pub count: usize,
    pub separator: bool,
}

impl Reservation {
    pub fn projected(
        self,
        content: f64,
        count: usize,
        separator: bool,
        above: f64,
        between: f64,
    ) -> f64 {
        if count == 0 {
            return self.height;
        }
        self.height
            + content
            + if separator && !self.separator {
                above
            } else {
                0.0
            }
            + between
                * if self.count == 0 {
                    count.saturating_sub(1)
                } else {
                    count
                } as f64
    }

    pub fn add(&mut self, content: f64, separator: bool, above: f64, between: f64) {
        self.height = self.projected(content, 1, separator, above, between);
        self.count += 1;
        self.separator |= separator;
    }
}

/// 너비가 다른 단으로 이어받은 각주를 그 단 각주 영역 폭으로 다시 나눈 문단들과, 다시 나눈
/// 첫 줄의 평탄 줄 번호(한/글 12.30: 8000·14000·20000HU 3단에서 1단 각주를 2단 폭으로 다시 나눈다).
///
/// `at`(각주 문단, UTF-16 위치) 앞의 줄은 저장 줄 그대로 두어 앞 조각의 줄 번호를 지키고, 그
/// 글자부터 뒤 문단까지 `width_px` 영역 폭(문단 좌우 여백을 뺀다)으로 나눈다. 조판과 그리기가
/// 같은 입력으로 이 함수를 불러 같은 줄을 쓴다.
pub(crate) fn refold_note_paragraphs(
    paragraphs: &[crate::model::paragraph::Paragraph],
    at: (usize, u32),
    width_px: f64,
    styles: &super::style_resolver::ResolvedStyleSet,
    dpi: f64,
) -> (Vec<crate::model::paragraph::Paragraph>, usize) {
    use super::composer::{reflow_line_segs, reflow_line_segs_from, ParagraphBox};
    let mut start_line = 0;
    let refolded = paragraphs
        .iter()
        .enumerate()
        .map(|(index, para)| {
            let mut para = para.clone();
            if index < at.0 {
                start_line += para.line_segs.len();
                return para;
            }
            let style = styles.para_styles.get(para.para_shape_id as usize);
            let margins = style.map_or(0.0, |style| style.margin_left + style.margin_right);
            let paragraph_box = ParagraphBox::content_width_px((width_px - margins).max(0.0), dpi);
            let kept = if index == at.0 {
                (0..para.line_segs.len())
                    .take_while(|&line| para.line_seg_text_start(line) < at.1)
                    .count()
            } else {
                0
            };
            if index == at.0 {
                start_line += kept;
            }
            if kept == 0 {
                reflow_line_segs(&mut para, paragraph_box, styles, dpi);
            } else {
                reflow_line_segs_from(&mut para, kept, at.1, paragraph_box, styles, dpi);
            }
            para
        })
        .collect();
    (refolded, start_line)
}
