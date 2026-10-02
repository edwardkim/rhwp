//! 확정한 미주 분할/전체 배치와 단 전환 적용.

use crate::renderer::typeset::notes::endnotes::profile::EnSsotLevel;
use crate::renderer::typeset::{
    Control, FormattedParagraph, PageItem, Paragraph, ResolvedStyleSet, TypesetEngine,
    TypesetState, ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX,
};

impl TypesetEngine {
    /// [#2064 추출] P8: split 후보가 확정된 미주 문단의 PartialParagraph 방출 —
    /// 원본 무변경 이동. 반환: split 방출 수행 여부 (caller 의 split_endnote_emitted).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn emit_endnote_split(
        &self,
        st: &mut TypesetState,
        fmt: &FormattedParagraph,
        en_para: &Paragraph,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        split_candidate: Option<usize>,
        en_para_idx: usize,
        en_advance: f64,
        en_col_w: f64,
        available: f64,
        col_count: u16,
        compact_between_notes_gap: bool,
        compact_endnote_separator_profile: bool,
        endnote_has_text_or_equation: bool,
        has_visible_endnote_separator: bool,
        large_separator_block: bool,
        line_advances_sum: f64,
        non_tac_object_height: Option<f64>,
        pre_emit_tail_before_non_tac_object_advance: bool,
        ssot_debug: bool,
        ssot_level: EnSsotLevel,
        tac_picture_rewind_height: Option<f64>,
    ) -> bool {
        let mut emitted = false;
        if let Some(split_line) = split_candidate {
            let first_h = fmt.line_advances_sum(0..split_line);
            st.append_item(PageItem::PartialParagraph {
                para_index: en_para_idx,
                start_line: 0,
                end_line: split_line,
            });
            st.advance_flow_by(first_h);
            st.mark_endnote_flow();
            st.advance_column_or_new_page();
            let rest_h =
                fmt.line_advances_sum(split_line..fmt.line_heights.len()) + fmt.spacing_after;
            st.append_item(PageItem::PartialParagraph {
                para_index: en_para_idx,
                start_line: split_line,
                end_line: fmt.line_heights.len(),
            });
            st.advance_flow_by(rest_h);
            st.mark_endnote_flow();
            emitted = true;
        } else {
            let table_only_endnote_para = en_para.text.is_empty()
                && en_para
                    .controls
                    .iter()
                    .any(|ctrl| matches!(ctrl, Control::Table(_)))
                && !en_para
                    .controls
                    .iter()
                    .any(|ctrl| matches!(ctrl, Control::Equation(_)));
            let pre_emitted_non_tac_object_only_para = pre_emit_tail_before_non_tac_object_advance
                && non_tac_object_height.is_some()
                && !endnote_has_text_or_equation;
            if !table_only_endnote_para && !pre_emitted_non_tac_object_only_para {
                st.append_item(PageItem::FullParagraph {
                    para_index: en_para_idx,
                });
                st.mark_endnote_flow();
            }
            for (ctrl_idx, ctrl) in en_para.controls.iter().enumerate() {
                match ctrl {
                    Control::Table(_) if table_only_endnote_para => {
                        st.append_item(PageItem::Table {
                            para_index: en_para_idx,
                            control_index: ctrl_idx,
                        });
                        st.mark_endnote_flow();
                    }
                    Control::Shape(_) | Control::Picture(_) => {
                        st.append_item(PageItem::Shape {
                            para_index: en_para_idx,
                            control_index: ctrl_idx,
                        });
                        st.mark_endnote_flow();
                    }
                    _ => {}
                }
            }
            // [Task #1363 Divergence C] TAC 그림 미주 para 의 누적 경로.
            // 종전(legacy/A): local_vpos_rewind TAC 그림은 저장 vpos 가
            // 앞 제목 옆을 가리킨다고 보고 `max(rewind_start+adv)` 로 누적
            // (겹침 가정). 그러나 TAC(treat_as_char) 그림은 렌더러가 문단
            // 흐름에 inline 으로 **순차 적층**한다(옆 배치 아님). 겹침 가정은
            // 그림 높이를 과소 계상해 단을 과충전(sep20/20 p22 col0 +58px →
            // 본문 50px 초과). SSOT(B+): 렌더러처럼 순차 적층(`+= adv`).
            let tac_stack_ssot =
                matches!(tac_picture_rewind_height, Some(_)) && ssot_level >= EnSsotLevel::B;
            if let Some(rewind_start) = tac_picture_rewind_height.filter(|_| !tac_stack_ssot) {
                // 단 하단의 TAC 그림은 renderer가 직전 텍스트를 침범하는
                // vpos 되감김을 버리고 순차 y를 유지한다. pagination도
                // 같은 경우에는 그림 높이를 소비해야 뒤 문단이 frame 아래로
                // 밀리지 않는다.
                let rewind_end = rewind_start + en_advance;
                let consume_rewind_picture_height = compact_endnote_separator_profile
                    && st.current_column + 1 >= st.col_count
                    && rewind_end <= st.current_height + ENDNOTE_COLUMN_BOTTOM_BLEED_TOLERANCE_PX
                    && ((compact_between_notes_gap && has_visible_endnote_separator)
                        || (large_separator_block && !has_visible_endnote_separator));
                if ssot_debug {
                    eprintln!(
                        "EN_ACC pi={} path={} ch_before={:.1} rewind_start={:.1} adv={:.1} ch_after={:.1}",
                        en_para_idx,
                        if consume_rewind_picture_height { "TACbottom" } else { "TACmax" },
                        st.current_height,
                        rewind_start,
                        en_advance,
                        if consume_rewind_picture_height {
                            st.current_height + en_advance
                        } else {
                            st.current_height.max(rewind_end)
                        },
                    );
                }
                if consume_rewind_picture_height {
                    st.advance_flow_by(en_advance);
                } else {
                    st.align_flow_to(st.current_height.max(rewind_end));
                }
            } else {
                if ssot_debug {
                    eprintln!(
                        "EN_ACC pi={} path={} ch_before={:.1} adv={:.1} ch_after={:.1}",
                        en_para_idx,
                        if tac_stack_ssot { "TACstack" } else { "add" },
                        st.current_height,
                        en_advance,
                        st.current_height + en_advance,
                    );
                }
                st.advance_flow_by(en_advance);
            }
            // [Task #1363 v2 Stage 2] A2: 누적을 렌더 시뮬 bottom 으로 스냅.
            // compute_en_metrics(saved-delta) 대신 HeightCursor 시뮬레이션이
            // 단 실제 렌더 높이를 산출 → fit 결정(다음 para)이 렌더 정합.
            // [Task #1370 Stage 2 실험] A3 한정: exact 스냅이 rewind/빈 para 를
            // hancom 보다 ~80px/단 높게 누적해 경계 split 을 막고 13건 cascade 유발.
            // 실험으로 A3 에서 스냅 OFF → break-결정 높이를 compact(acc)로 환원.
            if ssot_level == EnSsotLevel::A2 {
                if let Some(sim_bottom) = self.simulate_endnote_column_bottom_y(
                    &st, paragraphs, styles, available, en_col_w, None, false,
                ) {
                    if ssot_debug {
                        eprintln!(
                            "EN_ACC pi={} path=A2sim {:.1} -> {:.1}",
                            en_para_idx, st.current_height, sim_bottom,
                        );
                    }
                    st.align_flow_to(sim_bottom);
                }
            }
        }
        emitted
    }
}
