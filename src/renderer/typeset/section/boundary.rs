//! 구역 boundary 책임. 기존 조건과 호출 순서를 보존한다.
use crate::renderer::typeset::{
    is_synthetic_line_seg, para_has_visible_text, positive_vpos_end_before_negative_wrap, Control,
    Paragraph, ResolvedStyleSet, TypesetEngine,
};
impl TypesetEngine {
    /// [Task #2094] HWP3 변환본 vpos 리셋 쪽나눔 판정 — 소스분기(is_hwp3_variant)는
    /// caller 유지, 본 함수는 판정 본체만 담당한다 (원본 무변경 이동).
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn judge_hwp3_variant_vpos_reset_break(
        &self,
        para: &Paragraph,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        para_idx: usize,
        body_height_hu_for_variant: i32,
        variant_prev_para_idx: Option<usize>,
    ) -> bool {
        let mut variant_vpos_reset_break = false;
        if body_height_hu_for_variant > 0 && !para.text.is_empty() {
            let para_sb = styles
                .para_styles
                .get(para.para_shape_id as usize)
                .map(|ps| ps.spacing_before)
                .unwrap_or(0.0);
            let para_sb_hu = (para_sb * 7200.0 / 96.0) as i32;
            let prev_real_idx_and_ls = variant_prev_para_idx.and_then(|prev_pi| {
                (0..=prev_pi).rev().find_map(|i| {
                    paragraphs
                        .get(i)
                        .and_then(|p| p.line_segs.last())
                        .filter(|ls| !is_synthetic_line_seg(ls))
                        .map(|ls| (i, ls))
                })
            });
            let curr_real = para
                .line_segs
                .first()
                .filter(|ls| !is_synthetic_line_seg(ls));
            if let Some((prev_real_idx, prev_last)) = prev_real_idx_and_ls {
                let prev_end_vpos = prev_last.vertical_pos.saturating_add(prev_last.line_height);
                let prev_positive_wrap_end = paragraphs
                    .get(prev_real_idx)
                    .and_then(positive_vpos_end_before_negative_wrap);
                let prev_prev_end_vpos = if prev_real_idx > 0 {
                    (0..prev_real_idx).rev().find_map(|i| {
                        paragraphs.get(i).and_then(|p| {
                            p.line_segs
                                .last()
                                .filter(|ls| !is_synthetic_line_seg(ls))
                                .map(|ls| ls.vertical_pos.saturating_add(ls.line_height))
                        })
                    })
                } else {
                    None
                };
                let prev_top_content_reset = paragraphs.get(prev_real_idx).is_some_and(|p| {
                    let prev_sb_hu = styles
                        .para_styles
                        .get(p.para_shape_id as usize)
                        .map(|ps| (ps.spacing_before * 7200.0 / 96.0) as i32)
                        .unwrap_or(0);
                    p.line_segs.len() == 1
                        && p.line_segs
                            .first()
                            .is_some_and(|ls| !is_synthetic_line_seg(ls) && ls.vertical_pos == 0)
                        && p.controls.is_empty()
                        && para_has_visible_text(p)
                        && prev_sb_hu < 250
                });
                let next_first_real_vpos = paragraphs
                    .get(para_idx + 1)
                    .and_then(|next_para| next_para.line_segs.first())
                    .filter(|ls| !is_synthetic_line_seg(ls))
                    .map(|ls| ls.vertical_pos);
                let bridge_missing_count = (prev_real_idx + 1..para_idx)
                    .filter(|&i| {
                        paragraphs.get(i).is_some_and(|p| {
                            p.line_segs.is_empty()
                                && p.controls.is_empty()
                                && para_has_visible_text(p)
                        })
                    })
                    .count();
                let high_threshold = body_height_hu_for_variant * 95 / 100;
                let table_heading_reset = prev_real_idx + 1 == para_idx
                    && para.line_segs.is_empty()
                    && para.controls.is_empty()
                    && para_has_visible_text(para)
                    && para_sb_hu >= 500
                    && prev_end_vpos > body_height_hu_for_variant * 85 / 100
                    && paragraphs.get(prev_real_idx).is_some_and(|prev_para| {
                        prev_para
                            .controls
                            .iter()
                            .any(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                    })
                    && paragraphs
                        .get(para_idx + 1)
                        .and_then(|next_para| next_para.line_segs.first())
                        .filter(|ls| !is_synthetic_line_seg(ls))
                        .is_some_and(|ls| ls.vertical_pos <= 4000);
                let empty_bridge_heading_reset = para.line_segs.is_empty()
                    && para.controls.is_empty()
                    && para_has_visible_text(para)
                    && para_sb_hu >= 500
                    && bridge_missing_count == 1
                    && prev_end_vpos > body_height_hu_for_variant * 80 / 100
                    && prev_end_vpos <= body_height_hu_for_variant * 85 / 100;

                let real_heading_or_bridge_reset = curr_real.is_some_and(|curr_first| {
                    let curr_first_vpos = curr_first.vertical_pos;
                    let strict_heading_reset = para_sb_hu >= 500
                        && prev_end_vpos > high_threshold
                        && curr_first_vpos < 1500;
                    let delayed_heading_after_top_content_reset = prev_real_idx + 1 == para_idx
                        && para.line_segs.len() >= 2
                        && para_sb_hu >= 500
                        && para.controls.is_empty()
                        && para_has_visible_text(para)
                        && curr_first_vpos > 0
                        && curr_first_vpos <= 2500
                        && prev_top_content_reset
                        && prev_prev_end_vpos
                            .is_some_and(|end| end > body_height_hu_for_variant * 70 / 100);
                    let bridged_reset = bridge_missing_count >= 2
                        && para.controls.is_empty()
                        && para_has_visible_text(para)
                        && curr_first_vpos <= 1500
                        && prev_end_vpos > body_height_hu_for_variant * 75 / 100;
                    let negative_wrap_heading_reset = prev_real_idx + 1 == para_idx
                        && para.line_segs.len() == 1
                        && para_sb_hu >= 250
                        && para.controls.is_empty()
                        && para_has_visible_text(para)
                        && curr_first_vpos < 0
                        && prev_positive_wrap_end
                            .is_some_and(|end| end > body_height_hu_for_variant * 75 / 100);
                    let bottom_heading_before_next_reset = prev_real_idx + 1 == para_idx
                        && para.line_segs.len() == 1
                        && para_sb_hu >= 250
                        && para.controls.is_empty()
                        && para_has_visible_text(para)
                        && curr_first_vpos > body_height_hu_for_variant * 75 / 100
                        && next_first_real_vpos.is_some_and(|next_vpos| {
                            next_vpos > 0 && next_vpos <= 4000 && curr_first_vpos > next_vpos
                        });
                    strict_heading_reset
                        || delayed_heading_after_top_content_reset
                        || bridged_reset
                        || negative_wrap_heading_reset
                        || bottom_heading_before_next_reset
                });

                if table_heading_reset || empty_bridge_heading_reset || real_heading_or_bridge_reset
                {
                    variant_vpos_reset_break = true;
                }
            }
        }
        variant_vpos_reset_break
    }
}
