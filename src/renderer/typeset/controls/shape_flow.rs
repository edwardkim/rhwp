//! 일반 표 문단에 동반된 그림·도형·수식의 기존 흐름 높이와 이월 조건 조회.
//! 원본 LineSeg 선택과 두 높이의 우선순위를 보존하며 상태를 변경하지 않는다.

use crate::model::{control::Control, paragraph::Paragraph};
use crate::renderer::hwpunit_to_px;

pub(in crate::renderer::typeset) struct TableHostShapePage {
    pub has_items: bool,
    pub current_height: f64,
}

pub(in crate::renderer::typeset) struct TableHostShapeFlow {
    pub tac_separate_line_h: Option<f64>,
    pub non_tac_pushdown_h: Option<f64>,
}

pub(super) fn prepare(
    ctrl: &Control,
    para: &Paragraph,
    ctrl_idx: usize,
    dpi: f64,
) -> TableHostShapeFlow {
    // Task #402: 같은 paragraph의 선행 TAC 컨트롤이 있는 TAC 그림은
    // 자기 line_seg에 위치하므로 그 line의 높이를 페이지 누적에 반영해야 함.
    // 누락 시 후속 항목이 페이지 끝을 넘어 그려져 겹침/오버플로 발생 (#402).
    let tac_separate_line_h: Option<f64> = match ctrl {
        Control::Picture(p) if p.common.treat_as_char => Some(()),
        Control::Shape(s) if s.common().treat_as_char => Some(()),
        _ => None,
    }
    .and_then(|_| {
        let prior_tac_count = para
            .controls
            .iter()
            .take(ctrl_idx)
            .filter(|c| match c {
                Control::Table(t) => t.common.treat_as_char,
                Control::Picture(p) => p.common.treat_as_char,
                Control::Shape(s) => s.common().treat_as_char,
                _ => false,
            })
            .count();
        if prior_tac_count == 0 {
            return None;
        }
        para.line_segs.get(prior_tac_count).map(|seg| {
            let lh = hwpunit_to_px(seg.line_height, dpi);
            let ls_extra = if seg.line_spacing > 0 {
                hwpunit_to_px(seg.line_spacing, dpi)
            } else {
                0.0
            };
            lh + ls_extra
        })
    });
    // [Issue #1156] 비-TAC 자리차지(TopAndBottom) 객체(차트 OLE 등):
    // 표와 같은 문단에 있으면 종전에는 높이/단 이동 없이 push 만 되어,
    // 한컴처럼 단 끝을 넘는 객체가 다음 단으로 이동하지 못했다.
    // 한컴: 객체가 현재 단 잔여 영역을 넘으면 다음 단 상단으로 이동.
    // (객체 점유 크기 = common 높이 80mm, spec/한컴/HWPX hp:sz 3중 일치)
    use crate::model::shape::{TextWrap, VertRelTo};
    let non_tac_pushdown_h: Option<f64> = if tac_separate_line_h.is_none() {
        match ctrl {
            Control::Picture(p)
                if !p.common.treat_as_char
                    && matches!(p.common.text_wrap, TextWrap::TopAndBottom)
                    && matches!(p.common.vert_rel_to, VertRelTo::Para) =>
            {
                let h = hwpunit_to_px(p.common.height as i32, dpi);
                let mb = hwpunit_to_px(p.common.margin.bottom as i32, dpi);
                Some(h + mb)
            }
            Control::Shape(s)
                if !s.common().treat_as_char
                    && matches!(s.common().text_wrap, TextWrap::TopAndBottom)
                    && matches!(s.common().vert_rel_to, VertRelTo::Para) =>
            {
                let cm = s.common();
                let h = hwpunit_to_px(cm.height as i32, dpi);
                let mb = hwpunit_to_px(cm.margin.bottom as i32, dpi);
                Some(h + mb)
            }
            _ => None,
        }
    } else {
        None
    };

    TableHostShapeFlow {
        tac_separate_line_h,
        non_tac_pushdown_h,
    }
}

impl TableHostShapeFlow {
    /// TAC의 항목 존재 조건과 float의 단 상단 조건을 구별한다. 예산 조회도 단락 평가한다.
    pub(super) fn needs_advance(
        &self,
        page: TableHostShapePage,
        available_height: impl FnOnce() -> f64,
    ) -> bool {
        if let Some(line_h) = self.tac_separate_line_h {
            // 자기 line이 현재 페이지에 들어가지 않으면 다음 페이지로 분할
            page.has_items && page.current_height + line_h > available_height() + 0.5
        } else if let Some(extra) = self.non_tac_pushdown_h {
            // 비-TAC 자리차지 객체: 현재 단 잔여 부족 + 단 상단 아니면 다음 단/페이지 이동
            let is_column_top = page.current_height < 1.0;
            !is_column_top && page.current_height + extra > available_height() + 0.5
        } else {
            false
        }
    }
}
