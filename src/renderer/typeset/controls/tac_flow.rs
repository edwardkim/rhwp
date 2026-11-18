//! TAC 흐름 참여와 저장 줄 소속을 조회하는 읽기 전용 경계.
//! 기존 판별을 공유하며 표 배치·페이지 상태 쓰기는 수행하지 않는다.

use super::super::paragraph::metrics::FormattedParagraph;
use crate::model::{paragraph::Paragraph, provenance::LayoutCompatibilityProfile};
use crate::renderer::hwpunit_to_px;
use std::cell::Cell;

pub(in crate::renderer::typeset) struct TacFlowQuery<'a> {
    dpi: f64,
    profile: &'a Cell<LayoutCompatibilityProfile>,
}

impl<'a> TacFlowQuery<'a> {
    pub(in crate::renderer::typeset) fn new(
        dpi: f64,
        profile: &'a Cell<LayoutCompatibilityProfile>,
    ) -> Self {
        Self { dpi, profile }
    }

    pub(super) fn dpi(&self) -> f64 {
        self.dpi
    }

    // 생성 시 profile을 미리 읽지 않고 원래 조건의 평가 지점에서 조회한다.
    pub(super) fn session_edited(&self) -> bool {
        self.profile.get().session_edited()
    }

    /// 저장 줄이 없거나 측정이 새 줄 소속을 발행한 셀은 현재 구성으로 높이를 정한다.
    /// host의 옛 저장 줄이 그 새 높이를 제한하는 근거가 될 수는 없다.
    pub(in crate::renderer::typeset) fn single_tac_line_has_current_cell_rows(
        &self,
        para: &Paragraph,
        table: &crate::model::table::Table,
        fmt: &FormattedParagraph,
        tac_count: usize,
        measured_cells: &[crate::renderer::height_measurer::MeasuredCell],
    ) -> bool {
        if tac_count != 1
            || fmt.line_heights.len() != 1
            || super::super::para_has_non_whitespace_text(para)
        {
            return false;
        }
        // 측정에서 실제 발행한 줄 수와 원본 저장 줄 소속이 다르면 옛 host는
        // 현재 점유의 상한이 아니다. 원본 개체 크기나 이름으로 재조판을 추측하지 않는다.
        let rebuilt_stored_rows = measured_cells.iter().any(|current| {
            table
                .cells
                .iter()
                .find(|source| {
                    usize::from(source.row) == current.row && usize::from(source.col) == current.col
                })
                .is_some_and(|source| {
                    current.para_line_counts.len() == source.paragraphs.len()
                        && current.para_line_counts.iter().zip(&source.paragraphs).any(
                            |(count, paragraph)| {
                                !crate::renderer::para_has_no_stored_line_segs(paragraph)
                                    && *count != paragraph.line_segs.len()
                            },
                        )
                })
        });
        rebuilt_stored_rows
            || table
                .cells
                .iter()
                .flat_map(|cell| &cell.paragraphs)
                .any(super::super::para_has_non_whitespace_text)
                && table
                    .cells
                    .iter()
                    .flat_map(|cell| &cell.paragraphs)
                    .all(crate::renderer::para_has_no_stored_line_segs)
    }

    pub(in crate::renderer::typeset) fn tac_table_line_index(
        &self,
        para: &Paragraph,
        table: &crate::model::table::Table,
        fmt: &FormattedParagraph,
    ) -> Option<usize> {
        if !table.common.treat_as_char || fmt.line_heights.is_empty() {
            return None;
        }
        if fmt.line_heights.len() == 1 && !crate::renderer::para_has_no_stored_line_segs(para) {
            // 저장 줄이 하나뿐인 문단은 흐름 속성(#3930)이 경로를 정한다. 다만 그 줄이
            // 이 표 하나의 줄 상자임을 문서가 증명하면(줄 높이 = 표 높이 + 바깥 여백,
            // 보이는 글과 다른 표 없음) 한/글이 표를 글줄에 놓은 것이므로 그 줄을 따른다.
            return sole_stored_table_line(para, table).then_some(0);
        }

        // 저장 사다리가 없는 현재 단일 줄도 같은 줄 구성에서 실제 표 높이를
        // 소유할 수 있다. 아래 높이 대조를 통과한 줄만 사용하며, 별도 쪽 제한
        // 속성(flowWithText)을 줄 소속의 대용으로 사용하지 않는다. 저장 줄의
        // #3930 판정은 기존 경로에 남긴다.

        // 같은 사다리를 측정하는 경로에서는 실제 제어 슬롯의 소유 줄을 쓴다.
        // 셀 편집으로 표가 커지면 선언 높이와의 매칭은 더 이상 소속 증거가 아니다.
        if fmt.computed_host_lines.is_none() && fmt.line_heights.len() == para.line_segs.len() {
            if let Some(control_index) = para.controls.iter().position(|control| {
                matches!(control, crate::model::control::Control::Table(candidate)
                    if std::ptr::eq(candidate.as_ref(), table))
            }) {
                if let Some(line) =
                    crate::renderer::composer::control_slot_line_index(para, control_index)
                {
                    return Some(line);
                }
            }
        }

        // 저장 줄이 없는 문단은 현재 줄 구성의 제어 소속을 따른다.
        // 셀 내용으로 실제 표 높이가 커져도 선언 높이는 그대로일 수 있다.
        if crate::renderer::para_has_no_stored_line_segs(para) {
            if let (Some(comp), Some(control_index)) = (
                fmt.host_composition.as_deref(),
                para.controls.iter().position(|control| {
                    matches!(control, crate::model::control::Control::Table(candidate)
                        if std::ptr::eq(candidate.as_ref(), table))
                }),
            ) {
                if let Some(owner) = comp.inline_controls.iter().find(|owner| {
                    owner.control_index == control_index && owner.line_index < comp.lines.len()
                }) {
                    return Some(owner.line_index);
                }
            }
        }

        let om_top = hwpunit_to_px(table.outer_margin_top as i32, self.dpi);
        let om_bot = hwpunit_to_px(table.outer_margin_bottom as i32, self.dpi);
        let table_body_h = hwpunit_to_px(table.common.height as i32, self.dpi);
        let table_line_h = table_body_h + om_top + om_bot;
        // 재구성한 개체 줄은 본체 높이를 갖고 바깥 여백은 배치에서 소비한다.
        // 실제 저장 줄의 소유 판정에는 기존 외곽 높이 계약을 유지한다.
        let matches_height = |height: f64| {
            (height - table_line_h).abs() < 1.0
                || (crate::renderer::para_has_no_stored_line_segs(para)
                    && (height - table_body_h).abs() < 1.0)
        };

        // [#2287 후속/1.hwpx p58] text_height(th) 매칭 우선 — 한컴은 문단의
        // 모든 줄에 최대 줄높이를 lh 로 저장하는 관례가 있어(1.hwpx pi=322:
        // 텍스트 줄 ls[0] lh=69085/th=1300, 표 줄 ls[1] lh=th=69085), lh 만으로
        // 는 텍스트 줄이 먼저 오매칭되어 917px TAC 표의 소비가 17.3px 로
        // 붕괴(fmt.line_heights[0] 채택)했다. th 가 표 높이와 일치하는 줄이
        // 있으면 그 줄이 표 줄의 확정 증거이고, 없으면 종전 lh 매칭 유지.
        let th_match = para.line_segs.iter().enumerate().find_map(|(idx, seg)| {
            let th = hwpunit_to_px(seg.text_height, self.dpi);
            matches_height(th).then_some(idx)
        });
        if th_match.is_some() {
            return th_match;
        }

        para.line_segs
            .iter()
            .enumerate()
            .find_map(|(idx, seg)| {
                let line_h = hwpunit_to_px(seg.line_height, self.dpi);
                if matches_height(line_h) {
                    Some(idx)
                } else {
                    None
                }
            })
            .or_else(|| {
                para.line_segs
                    .is_empty()
                    .then(|| {
                        fmt.line_heights.iter().position(|height| {
                            (height - table_line_h).abs() < 1.0
                                || (height - table_body_h).abs() < 1.0
                        })
                    })
                    .flatten()
            })
    }

    pub(in crate::renderer::typeset) fn is_effective_tac_table(
        &self,
        para: &Paragraph,
        table: &crate::model::table::Table,
        fmt: &FormattedParagraph,
    ) -> bool {
        self.uses_tac_table_flow(table)
            || self.tac_table_line_index(para, table, fmt) == Some(0)
            || owns_later_stored_line(para, table)
    }

    /// HWPX 계보 HWP는 HWP5 CTRL_HEADER를 다시 읽으면서 `table.attr` bit 0을
    /// `treatAsChar`로 채운다. 하지만 HWPX의 inline 의미는 `treatAsChar`와
    /// `flowWithText`가 모두 참일 때만 성립한다. 후자가 거짓인 표를 TAC으로
    /// 오인하면 큰 표가 통째로 배치되어 저장 직후 쪽 경계가 압축된다 (#3930).
    pub(in crate::renderer::typeset) fn uses_tac_table_flow(
        &self,
        table: &crate::model::table::Table,
    ) -> bool {
        if self.profile.get().hwpx_stored_layout() {
            table.common.treat_as_char && table.common.flow_with_text
        } else {
            table.attr & 0x01 != 0
        }
    }
}

/// 글자 취급 표가 첫 줄이 아닌 저장 줄을 혼자 소유하는가.
///
/// 저장 줄 높이가 표 높이와 위·아래 바깥 여백의 합과 같으면 한/글이 그 표를 글줄 안에
/// 놓은 증거다. 줄 0 의 같은 증거는 이미 TAC 흐름으로 받으므로, 여러 표가 각자의 줄을
/// 가진 문단(쪽을 넘겨 vpos 를 되감은 둘째 표 포함)의 뒤 표도 같은 흐름으로 받는다.
/// 그렇지 않으면 typeset 은 블록 표로 표 높이만 전진하고 layout 은 저장 줄 전체를
/// 전진해 같은 쪽 뒤 TAC 표의 원점이 갈린다(#8969, 36400485 4쪽).
fn owns_later_stored_line(para: &Paragraph, table: &crate::model::table::Table) -> bool {
    para.controls
        .iter()
        .position(|control| {
            matches!(control, crate::model::control::Control::Table(candidate)
                if std::ptr::eq(candidate.as_ref(), table))
        })
        .and_then(|index| crate::renderer::composer::stored_owned_tac_line(para, index))
        .is_some_and(|(owner, _)| owner > 0)
}

/// 글자처럼 취급 표 하나가 문단의 유일한 저장 줄을 혼자 소유하는가.
///
/// 저장 줄 높이가 표 높이와 위·아래 바깥 여백의 합과 정확히 같고 문단에 보이는 글이
/// 없으면 그 줄은 이 표의 줄 상자다. 표가 둘 이상이면 줄 하나가 누구의 것인지 이 증거로
/// 정할 수 없으므로 제외한다.
fn sole_stored_table_line(para: &Paragraph, table: &crate::model::table::Table) -> bool {
    let mut tables = para.controls.iter().filter_map(|control| match control {
        crate::model::control::Control::Table(candidate) => Some(candidate.as_ref()),
        _ => None,
    });
    tables.next().is_some_and(|only| std::ptr::eq(only, table))
        && tables.next().is_none()
        && para.line_segs.len() == 1
        && crate::renderer::composer::stored_first_tac_line(para).is_some()
}
