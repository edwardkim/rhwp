//! 구역/문서 끝 미주의 원본 참조 순서와 준비→문단 처리 조정.

use crate::renderer::typeset::notes::endnotes::content::resolve_endnote_content;
use crate::renderer::typeset::notes::endnotes::profile::EndnoteFlowProfile;
use crate::renderer::typeset::notes::endnotes::types::{EndnoteFlowState, EndnotePrepCarry};
use crate::renderer::typeset::{
    ComposedParagraph, EndnoteDeferral, EndnoteRef, FootnoteShape, MeasuredTable, PageDef,
    Paragraph, ResolvedStyleSet, TypesetEngine, TypesetState,
};
pub(in crate::renderer::typeset) mod content;
pub(in crate::renderer::typeset) mod debug;
pub(in crate::renderer::typeset) mod emit;
pub(in crate::renderer::typeset) mod fit;
pub(in crate::renderer::typeset) mod format;
pub(in crate::renderer::typeset) mod measure;
pub(in crate::renderer::typeset) mod metrics;
pub(in crate::renderer::typeset) mod paragraph;
pub(in crate::renderer::typeset) mod prepare;
pub(in crate::renderer::typeset) mod profile;
pub(in crate::renderer::typeset) mod types;

impl TypesetEngine {
    /// [Task #1904] 미주(endnote) 가상 삽입·배치 — `typeset_section_with_variant` 에서
    /// 동작 불변 추출 (1차 리팩토링 라운드 1). 미주 deferral 을 정리한 뒤 `st.endnotes` 를
    /// 본문 흐름 뒤에 배치한다 (#836 한컴 정합: 미주는 섹션 끝 2단 플로우).
    /// 소스-포맷(is_hwp3/is_hwpx) 분기 비접촉 블록.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::renderer::typeset) fn typeset_section_endnotes(
        &self,
        st: &mut TypesetState,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        section_index: usize,
        page_def: &PageDef,
        measured_tables: &[MeasuredTable],
        endnote_shape: Option<&FootnoteShape>,
        endnote_deferral: &EndnoteDeferral<'_>,
    ) {
        // 기본(None)/단일 구역: 이 구역 미주를 구역 끝에 렌더 (구역 끝 ≡ 문서 끝).
        // Suppress(END_OF_DOCUMENT 비-마지막 구역): 참조 표시는 인라인 유지, 본문은
        //   문서 끝으로 미루므로 여기서 비운다.
        // RenderAll(END_OF_DOCUMENT 마지막 구역): 앞선 구역 미주(문서 순서) → 이 구역
        //   미주 순으로 endnote_refs 앞에 이어 붙여 모두 문서 끝에 렌더한다.
        match endnote_deferral {
            EndnoteDeferral::Suppress => st.suppress_endnotes(),
            EndnoteDeferral::RenderAll(deferred) => {
                let merged: Vec<EndnoteRef> = deferred.iter().map(|d| d.reff.clone()).collect();
                st.prepend_endnotes(merged);
            }
            EndnoteDeferral::None => {}
        }

        // [Task #836] 미주 paragraphs를 본문 흐름에 가상 삽입
        // 한컴 정합: 미주는 섹션 마지막에 일반 본문처럼 2단 레이아웃 플로우를 따름
        // 미주 paragraphs를 endnote_paragraphs Vec에 모으고, ENDNOTE_PARA_BASE 이상 인덱스로 마킹
        if !st.endnotes.is_empty() {
            let endnote_refs: Vec<EndnoteRef> = st.endnotes.clone();
            // 본문 마지막 paragraph의 vpos 끝 위치 계산
            let mut vpos_offset: i32 = paragraphs
                .last()
                .and_then(|p| p.line_segs.last())
                .map(|ls| {
                    ls.vertical_pos
                        .saturating_add(ls.line_height)
                        .saturating_add(ls.line_spacing)
                })
                .unwrap_or(0);
            // [Task #1082] 다단 미주 vpos-delta 누적용 prev tracker.
            // 시드 = 현재 단의 본문 last bottom vpos(body→endnote 전환 정합); 없으면 None
            // (단의 첫 미주 → 자체 높이 사용). 단 advance 시 flush_column 에서 prev_body 리셋.
            let mut prev_en_bottom_vpos: Option<i32> = st.prev_body_bottom_vpos;
            let mut prev_en_content_bottom_vpos: Option<i32> = st.prev_body_bottom_vpos;
            let mut prev_endnote_had_vpos_rewind = false;
            let mut prev_endnote_had_inline_object_vpos_overestimate = false;
            let mut cleared_single_line_internal_rewind_split = false;
            let mut emitted_endnote_separator = false;
            let mut current_endnote_had_inline_object_vpos_overestimate = false;
            let mut emitted_endnote_count = 0usize;
            let mut last_render_endnote_para_local_idx: Option<usize> = None;
            // 이 플래그는 "시험지 미주 흐름"의 split/rewind 보정 사용 여부다.
            // 구분선 아래 여백이 20mm처럼 커도 문항 미주 흐름 자체는 같은
            // 정책을 타야 하므로 separator 크기와 분리한다.
            let endnote_flow_profile = endnote_shape.map(EndnoteFlowProfile::from_shape);
            let compact_endnote_separator_profile = endnote_flow_profile.is_some();
            if let Some(profile) = endnote_flow_profile {
                st.record_endnote_separator_above(profile.separator_above_hu);
                st.record_endnote_separator_below(profile.separator_below_hu);
                st.record_endnote_between_margin(profile.between_notes_hu);
            }

            for (en_ref_idx, en_ref) in endnote_refs.iter().enumerate() {
                // [미주 배치] 현재 구역 미주는 본문 paragraphs 에서, 문서 끝으로 미뤄진
                // 앞 구역 미주(RenderAll)는 deferral 목록에서 본문(en_ctrl)을 해석한다.
                if let Some(en_ctrl) =
                    resolve_endnote_content(en_ref, section_index, paragraphs, &endnote_deferral)
                {
                    {
                        let (emit_vars, prep) = self.prepare_endnote_emit(
                            st,
                            paragraphs,
                            styles,
                            section_index,
                            endnote_shape,
                            endnote_flow_profile,
                            compact_endnote_separator_profile,
                            emitted_endnote_count,
                            last_render_endnote_para_local_idx,
                            cleared_single_line_internal_rewind_split,
                            prev_endnote_had_inline_object_vpos_overestimate,
                            en_ref,
                            en_ctrl,
                            EndnotePrepCarry {
                                vpos_offset,
                                prev_en_bottom_vpos,
                                prev_en_content_bottom_vpos,
                                prev_endnote_had_vpos_rewind,
                                emitted_endnote_separator,
                                current_endnote_had_inline_object_vpos_overestimate,
                            },
                        );
                        vpos_offset = prep.vpos_offset;
                        prev_en_bottom_vpos = prep.prev_en_bottom_vpos;
                        prev_en_content_bottom_vpos = prep.prev_en_content_bottom_vpos;
                        prev_endnote_had_vpos_rewind = prep.prev_endnote_had_vpos_rewind;
                        emitted_endnote_separator = prep.emitted_endnote_separator;
                        current_endnote_had_inline_object_vpos_overestimate =
                            prep.current_endnote_had_inline_object_vpos_overestimate;
                        let endnote_flow = self.typeset_endnote_paragraphs(
                            st,
                            paragraphs,
                            composed,
                            styles,
                            section_index,
                            page_def,
                            measured_tables,
                            endnote_shape,
                            endnote_flow_profile,
                            &endnote_refs,
                            en_ref_idx,
                            en_ref,
                            en_ctrl,
                            emit_vars,
                            EndnoteFlowState {
                                vpos_offset,
                                prev_en_bottom_vpos,
                                prev_en_content_bottom_vpos,
                                emitted_endnote_count,
                                last_render_endnote_para_local_idx,
                                cleared_single_line_internal_rewind_split,
                                current_endnote_had_inline_object_vpos_overestimate,
                            },
                        );
                        vpos_offset = endnote_flow.vpos_offset;
                        prev_en_bottom_vpos = endnote_flow.prev_en_bottom_vpos;
                        prev_en_content_bottom_vpos = endnote_flow.prev_en_content_bottom_vpos;
                        emitted_endnote_count = endnote_flow.emitted_endnote_count;
                        last_render_endnote_para_local_idx =
                            endnote_flow.last_render_endnote_para_local_idx;
                        cleared_single_line_internal_rewind_split =
                            endnote_flow.cleared_single_line_internal_rewind_split;
                        current_endnote_had_inline_object_vpos_overestimate =
                            endnote_flow.current_endnote_had_inline_object_vpos_overestimate;
                        prev_endnote_had_inline_object_vpos_overestimate =
                            current_endnote_had_inline_object_vpos_overestimate;
                        emitted_endnote_count += 1;
                    }
                }
            }
        }
    }
}
