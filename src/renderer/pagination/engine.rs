//! 페이지 분할 엔진 (paginate_with_measured)

use crate::model::control::Control;
use crate::model::header_footer::HeaderFooterApply;
use crate::model::paragraph::{Paragraph, ColumnBreakType};
use crate::model::page::{PageDef, ColumnDef};
use crate::model::shape::CaptionDirection;
use crate::renderer::height_measurer::{HeightMeasurer, MeasuredSection};
use crate::renderer::page_layout::PageLayoutInfo;
use super::*;
use super::state::PaginationState;

impl Paginator {
    pub fn paginate_with_measured(
        &self,
        paragraphs: &[Paragraph],
        measured: &MeasuredSection,
        page_def: &PageDef,
        column_def: &ColumnDef,
        section_index: usize,
        para_styles: &[crate::renderer::style_resolver::ResolvedParaStyle],
    ) -> PaginationResult {
        self.paginate_with_measured_opts(paragraphs, measured, page_def, column_def, section_index, para_styles, PaginationOpts::default())
    }

    pub fn paginate_with_measured_opts(
        &self,
        paragraphs: &[Paragraph],
        measured: &MeasuredSection,
        page_def: &PageDef,
        column_def: &ColumnDef,
        section_index: usize,
        para_styles: &[crate::renderer::style_resolver::ResolvedParaStyle],
        opts: PaginationOpts,
    ) -> PaginationResult {
        let hide_empty_line = opts.hide_empty_line;
        let respect_vpos_reset = opts.respect_vpos_reset;
        let layout = PageLayoutInfo::from_page_def(page_def, column_def, self.dpi);
        let measurer = HeightMeasurer::new(self.dpi);

        // 머리말/꼬리말/쪽 번호 위치/새 번호 지정 컨트롤 수집
        let (hf_entries, page_number_pos, page_hides, new_page_numbers) =
            Self::collect_header_footer_controls(paragraphs, section_index);

        let col_count = column_def.column_count.max(1);
        let footnote_separator_overhead = crate::renderer::hwpunit_to_px(400, self.dpi);
        let footnote_safety_margin = crate::renderer::hwpunit_to_px(3000, self.dpi);

        let mut st = PaginationState::new(
            layout, col_count, section_index,
            footnote_separator_overhead, footnote_safety_margin,
        );


        // 비-TAC 표 뒤의 ghost 빈 문단 스킵.
        // HWP에서 비-TAC 표의 LINE_SEG 높이는 실제 표 높이보다 작으며,
        // 그 차이를 빈 문단으로 채워넣음. 이 빈 문단들은 표 영역 안에 숨겨짐.
        // 어울림 배치(비-TAC) 표 오버랩 처리:
        // 어울림 표는 후속 문단들 위에 겹쳐서 렌더링됨.
        // 동일한 column_start(cs) 값을 가진 빈 문단은 표와 나란히 배치되므로
        // pagination에서 높이를 소비하지 않음.
        let mut wrap_around_cs: i32 = -1;  // -1 = 비활성
        let mut wrap_around_sw: i32 = -1;  // wrap zone의 segment_width
        let mut wrap_around_table_para: usize = 0;  // 어울림 표의 문단 인덱스
        let mut wrap_around_any_seg: bool = false;  // true면 any_seg_matches만으로 어울림 판정
        let mut prev_pagination_para: Option<usize> = None;  // vpos 보정용 이전 문단

        // 고정값 줄간격 TAC 표 병행 (Task #9):
        // Percent 전환 시 표 높이 - Fixed 누적 차이분을 current_height에 추가
        let mut fix_table_visual_h: f64 = 0.0;
        let mut fix_vpos_tmp: f64 = 0.0;
        let mut fix_overlay_active = false;

        // 빈 줄 감추기: 페이지 시작 부분에서 감춘 빈 줄 수 (최대 2개)
        let mut hidden_empty_lines: u8 = 0;
        let mut hidden_empty_page: usize = 0; // 현재 감추기 중인 페이지
        let mut hidden_empty_paras: std::collections::HashSet<usize> = std::collections::HashSet::new();

        for (para_idx, para) in paragraphs.iter().enumerate() {
            // 표 컨트롤 여부 사전 감지
            let has_table = measured.paragraph_has_table(para_idx);

            // 사전 측정된 문단 높이
            let mut para_height = measured.get_paragraph_height(para_idx).unwrap_or(0.0);

            // 빈 줄 감추기 (구역 설정 bit 19)
            // 한컴 도움말: "각 쪽의 시작 부분에 빈 줄이 나오면, 두 개의 빈 줄까지는
            // 없는 것처럼 간주하여 본문 내용을 위로 두 줄 당겨서 쪽을 정돈합니다."
            // 구현: 페이지 끝에서 빈 줄이 overflow를 유발하면 높이 0으로 처리 (최대 2개/페이지)
            if hide_empty_line {
                let current_page = st.pages.len();
                if current_page != hidden_empty_page {
                    hidden_empty_lines = 0;
                    hidden_empty_page = current_page;
                }
                let trimmed = para.text.replace(|c: char| c.is_control(), "");
                let is_empty_para = trimmed.trim().is_empty() && para.controls.is_empty();
                if is_empty_para
                    && !st.current_items.is_empty()
                    && st.current_height + para_height > st.available_height()
                    && hidden_empty_lines < 2
                {
                    hidden_empty_lines += 1;
                    para_height = 0.0;
                    hidden_empty_paras.insert(para_idx);
                }
            }

            // 고정값→글자에따라 전환: 표 높이와 Fixed 누적의 차이분 추가 (Task #9)
            if fix_overlay_active && !has_table {
                let is_fixed = para_styles.get(para.para_shape_id as usize)
                    .map(|ps| ps.line_spacing_type == crate::model::style::LineSpacingType::Fixed)
                    .unwrap_or(false);
                if !is_fixed {
                    // 표 높이가 Fixed 누적보다 크면 차이분을 current_height에 추가
                    if fix_table_visual_h > fix_vpos_tmp {
                        st.current_height += fix_table_visual_h - fix_vpos_tmp;
                    }
                    fix_overlay_active = false;
                }
            }

            // 다단 나누기(MultiColumn)
            if para.column_type == ColumnBreakType::MultiColumn {
                self.process_multicolumn_break(&mut st, para_idx, paragraphs, page_def);
            }

            // 단 나누기(Column)
            if para.column_type == ColumnBreakType::Column {
                if !st.current_items.is_empty() {
                    self.process_column_break(&mut st);
                }
            }

            let base_available_height = st.base_available_height();
            let available_height = st.available_height();

            // 쪽/단 나누기 감지
            let force_page_break = para.column_type == ColumnBreakType::Page
                || para.column_type == ColumnBreakType::Section;

            // ParaShape의 "문단 앞에서 항상 쪽 나눔" 속성
            let para_style = para_styles.get(para.para_shape_id as usize);
            let para_style_break = para_style.map(|s| s.page_break_before).unwrap_or(false);


            if (force_page_break || para_style_break) && !st.current_items.is_empty() {
                self.process_page_break(&mut st);
            }

            // tac 표: 표 실측 높이 + 텍스트 줄 높이(th)로 판단 (Task #19)
            let para_height_for_fit = if has_table {
                let has_tac = para.controls.iter().any(|c|
                    matches!(c, Control::Table(t) if t.common.treat_as_char));
                if has_tac {
                    // 표 실측 높이 합산 (outer_top 포함, outer_bottom 제외)
                    // 캡션은 paginate_table_control에서 별도 처리하므로 여기서는 제외
                    // 표 실측 높이 합산 (outer_top + line_spacing 포함, outer_bottom 제외)
                    // 캡션은 paginate_table_control에서 별도 처리하므로 여기서는 제외
                    let mut tac_ci = 0usize;
                    let tac_h: f64 = para.controls.iter().enumerate()
                        .filter_map(|(ci, c)| {
                            if let Control::Table(t) = c {
                                if t.common.treat_as_char {
                                    let mt = measured.get_measured_table(para_idx, ci);
                                    let mt_h = mt.map(|m| {
                                        let cap_h = m.caption_height;
                                        let cap_s = if cap_h > 0.0 {
                                            t.caption.as_ref()
                                                .map(|c| crate::renderer::hwpunit_to_px(c.spacing as i32, self.dpi))
                                                .unwrap_or(0.0)
                                        } else { 0.0 };
                                        m.total_height - cap_h - cap_s
                                    }).unwrap_or(0.0);
                                    let outer_top = crate::renderer::hwpunit_to_px(
                                        t.outer_margin_top as i32, self.dpi);
                                    let ls = para.line_segs.get(tac_ci)
                                        .filter(|seg| seg.line_spacing > 0)
                                        .map(|seg| crate::renderer::hwpunit_to_px(seg.line_spacing, self.dpi))
                                        .unwrap_or(0.0);
                                    tac_ci += 1;
                                    Some(mt_h + outer_top + ls)
                                } else { None }
                            } else { None }
                        })
                        .sum();
                    // 텍스트 줄 높이: th 기반 (lh에 표 높이가 포함되므로 th 사용)
                    let text_h: f64 = para.line_segs.iter()
                        .filter(|seg| seg.text_height > 0 && seg.text_height < seg.line_height / 3)
                        .map(|seg| {
                            crate::renderer::hwpunit_to_px(seg.text_height + seg.line_spacing, self.dpi)
                        })
                        .sum();
                    // host spacing (sb + sa)
                    let mp = measured.get_measured_paragraph(para_idx);
                    let sb = mp.map(|m| m.spacing_before).unwrap_or(0.0);
                    let sa = mp.map(|m| m.spacing_after).unwrap_or(0.0);
                    tac_h + text_h + sb + sa
                } else {
                    para_height
                }
            } else {
                para_height
            };

            // 현재 페이지에 넣을 수 있는지 확인 (표 문단만 플러시)
            // 다중 TAC 표 문단은 개별 표가 paginate_table_control에서 처리되므로 스킵
            let tac_table_count_for_flush = para.controls.iter()
                .filter(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                .count();
            // trailing ls 경계 조건: trailing ls 제거 시 들어가면 flush 안 함
            let has_tac_for_flush = para.controls.iter().any(|c|
                matches!(c, Control::Table(t) if t.common.treat_as_char));
            let trailing_tac_ls = if has_tac_for_flush {
                para.line_segs.last()
                    .filter(|seg| seg.line_spacing > 0)
                    .map(|seg| crate::renderer::hwpunit_to_px(seg.line_spacing, self.dpi))
                    .unwrap_or(0.0)
            } else { 0.0 };
            let fit_without_trail = st.current_height + para_height_for_fit - trailing_tac_ls <= available_height + 0.5;
            let fit_with_trail = st.current_height + para_height_for_fit <= available_height + 0.5;
            if !fit_with_trail && !fit_without_trail
                && !st.current_items.is_empty()
                && has_table
                && tac_table_count_for_flush <= 1
            {
                st.advance_column_or_new_page();
            }

            // 페이지가 아직 없으면 생성
            st.ensure_page();

            // vpos 기준점 설정: 페이지 첫 문단
            if st.page_vpos_base.is_none() {
                if let Some(seg) = para.line_segs.first() {
                    st.page_vpos_base = Some(seg.vertical_pos);
                }
            }

            // vpos 기반 current_height 보정: layout의 vpos 보정과 동기화
            // 현재 페이지에 블록 표(비-TAC)가 존재하면 적용 — 블록 표는 layout의
            // vpos 보정과 pagination의 높이 누적 사이에 누적 drift를 만듦.
            // 핵심: max(current_height, vpos_consumed) — 절대 감소하지 않음
            // 단, TAC 수식/그림 포함 문단은 제외 — LINE_SEG lh에 수식/그림 높이가
            // 포함되어 vpos가 과대하므로 보정하면 current_height가 과대 누적됨
            if let Some(prev_pi) = prev_pagination_para {
                if para_idx != prev_pi && st.page_has_block_table {
                    let prev_has_tac_eq = paragraphs.get(prev_pi).map(|p| {
                        p.controls.iter().any(|c|
                            matches!(c, Control::Equation(_)) ||
                            matches!(c, Control::Picture(pic) if pic.common.treat_as_char) ||
                            matches!(c, Control::Shape(s) if s.common().treat_as_char) ||
                            // 글앞으로/글뒤로 Shape: vpos에 Shape 높이가 포함되어 과대 → bypass
                            matches!(c, Control::Shape(s) if matches!(s.common().text_wrap,
                                crate::model::shape::TextWrap::InFrontOfText | crate::model::shape::TextWrap::BehindText)))
                    }).unwrap_or(false);
                    if !prev_has_tac_eq {
                    if let Some(base) = st.page_vpos_base {
                        if let Some(prev_para) = paragraphs.get(prev_pi) {
                            let col_width_hu = st.layout.column_width_hu();
                            let prev_seg = prev_para.line_segs.iter().rev().find(|ls| {
                                ls.segment_width > 0
                                    && (ls.segment_width - col_width_hu).abs() < 3000
                            });
                            if let Some(seg) = prev_seg {
                                if !(seg.vertical_pos == 0 && prev_pi > 0) {
                                    let vpos_end = seg.vertical_pos
                                        + seg.line_height
                                        + seg.line_spacing;
                                    let vpos_h = crate::renderer::hwpunit_to_px(
                                        vpos_end - base,
                                        self.dpi,
                                    );
                                    if vpos_h > st.current_height && vpos_h > 0.0 {
                                        let avail = st.available_height();
                                        if vpos_h <= avail {
                                            st.current_height = vpos_h;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    }
                }
            }
            prev_pagination_para = Some(para_idx);

            // 어울림 배치 표 오버랩 구간: 동일 cs를 가진 문단은 표 옆에 배치
            if wrap_around_cs >= 0 && !has_table {
                let para_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
                let para_sw = para.line_segs.first().map(|s| s.segment_width as i32).unwrap_or(0);
                let is_empty_para = para.text.chars().all(|ch| ch.is_whitespace() || ch == '\r' || ch == '\n')
                    && para.controls.is_empty();
                // 여러 LINE_SEG 중 하나라도 어울림 cs/sw와 일치하면 어울림 문단
                let any_seg_matches = para.line_segs.iter().any(|s|
                    s.column_start == wrap_around_cs && s.segment_width as i32 == wrap_around_sw
                );
                // sw=0인 어울림 표: 표가 전체 폭을 차지하므로
                // 후속 빈 문단의 sw가 문서 본문 폭보다 현저히 작으면 어울림 문단
                let body_w = (page_def.width as i32) - (page_def.margin_left as i32) - (page_def.margin_right as i32);
                let sw0_match = wrap_around_sw == 0 && is_empty_para && para_sw > 0
                    && para_sw < body_w / 2;
                if para_cs == wrap_around_cs && para_sw == wrap_around_sw
                    || (any_seg_matches && (is_empty_para || wrap_around_any_seg))
                    || sw0_match {
                    // 어울림 문단: 표 옆에 배치 — pagination에서 높이 소비 없이 기록
                    // (표가 이미 이 공간을 차지하고 있음)
                    st.current_column_wrap_around_paras.push(
                        super::WrapAroundPara {
                            para_index: para_idx,
                            table_para_index: wrap_around_table_para,
                            has_text: !is_empty_para,
                        }
                    );
                    continue;
                } else {
                    wrap_around_cs = -1;
                    wrap_around_sw = -1;
                    wrap_around_any_seg = false;
                }
            }

            // 비-표 문단 처리
            if !has_table {
                self.paginate_text_lines(
                    &mut st, para_idx, para, measured, para_height,
                    base_available_height, respect_vpos_reset,
                );
            }

            // 표 문단의 높이 보정용
            let height_before_controls = st.current_height;
            let page_count_before_controls = st.pages.len();

            // 인라인 컨트롤 감지 (표/도형/각주)
            self.process_controls(
                &mut st, para_idx, para, measured, &measurer,
                para_height, para_height_for_fit, base_available_height, page_def,
                height_before_controls,
            );

            let page_changed = st.pages.len() != page_count_before_controls;

            // treat_as_char 표 문단의 높이 보정
            // line_seg.line_height가 실측 표 높이보다 클 수 있으므로
            // 실측 높이를 기준으로 보정하여 레이아웃과 일치시킴
            let has_tac_block_table = para.controls.iter().any(|c| {
                if let Control::Table(t) = c { t.common.treat_as_char } else { false }
            });
            // 비-TAC 어울림(text_wrap=0) 표: 후속 빈 문단의 cs를 기록
            let has_non_tac_table = has_table && !has_tac_block_table;
            // 표 존재 시 플래그 설정 (vpos drift 보정용)
            // TAC/비-TAC 모두 layout의 vpos 보정과 drift를 만들 수 있음
            if has_table && !page_changed {
                st.page_has_block_table = true;
            }
            if has_non_tac_table {
                let is_wrap_around = para.controls.iter().any(|c| {
                    if let Control::Table(t) = c {
                        matches!(t.common.text_wrap, crate::model::shape::TextWrap::Square)
                    } else { false }
                });
                if is_wrap_around {
                    // 어울림 배치: 표의 LINE_SEG (cs, sw) 쌍과 동일한 후속 문단은
                    // 표 옆에 배치되므로 높이를 소비하지 않음
                    wrap_around_cs = para.line_segs.first()
                        .map(|s| s.column_start)
                        .unwrap_or(0);
                    wrap_around_sw = para.line_segs.first()
                        .map(|s| s.segment_width as i32)
                        .unwrap_or(0);
                    wrap_around_table_para = para_idx;
                    wrap_around_any_seg = false;
                }
            }
            // 비-TAC Picture Square wrap (어울림 그림): TABLE wrap과 동일 메커니즘.
            // lineseg가 이미지 존 전후로 분할되어 첫 seg cs=0 일 수 있으므로
            // wrap_around_any_seg=true 로 any_seg_matches만으로 후속 문단 판정 허용.
            let has_non_tac_pic_square = para.controls.iter().any(|c| {
                let cm = match c {
                    Control::Picture(p) => Some(&p.common),
                    Control::Shape(s) => if let crate::model::shape::ShapeObject::Picture(p) = s.as_ref() { Some(&p.common) } else { None },
                    _ => None,
                };
                cm.map(|cm| !cm.treat_as_char && matches!(cm.text_wrap, crate::model::shape::TextWrap::Square)).unwrap_or(false)
            });
            if has_non_tac_pic_square {
                let anchor_cs = para.line_segs.first().map(|s| s.column_start).unwrap_or(0);
                let anchor_sw = para.line_segs.first().map(|s| s.segment_width as i32).unwrap_or(0);
                if anchor_cs > 0 || anchor_sw > 0 {
                    wrap_around_cs = anchor_cs;
                    wrap_around_sw = anchor_sw;
                    wrap_around_table_para = para_idx;
                    wrap_around_any_seg = true;
                }
            }

            if has_tac_block_table && para_height > 0.0 && !page_changed {
                let height_added = st.current_height - height_before_controls;
                // Layout과 동일한 기준으로 TAC 표 높이 계산:
                // layout에서는 max(표 실측 높이, seg.vpos + seg.lh) + ls/2를 사용하므로
                // line_seg의 line_height를 기준으로 계산해야 layout과 일치함
                let tac_count = para.controls.iter()
                    .filter(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                    .count();
                let tac_seg_total: f64 = if tac_count > 0 && !para.line_segs.is_empty() {
                    // 각 TAC 표는 대응하는 line_seg를 사용
                    let mut total = 0.0;
                    let mut tac_idx = 0;
                    for (ci, c) in para.controls.iter().enumerate() {
                        if let Control::Table(t) = c {
                            if t.common.treat_as_char {
                                if let Some(seg) = para.line_segs.get(tac_idx) {
                                    // layout과 동일: max(표 실측, seg.lh) + ls
                                    let seg_lh = crate::renderer::hwpunit_to_px(seg.line_height, self.dpi);
                                    let mt_h = measured.get_table_height(para_idx, ci).unwrap_or(0.0);
                                    let effective_h = seg_lh.max(mt_h);
                                    let ls = if seg.line_spacing > 0 {
                                        crate::renderer::hwpunit_to_px(seg.line_spacing, self.dpi)
                                    } else { 0.0 };
                                    total += effective_h + ls;
                                }
                                tac_idx += 1;
                            }
                        }
                    }
                    total
                } else {
                    0.0
                };
                let cap = if tac_seg_total > 0.0 {
                    let mp = measured.get_measured_paragraph(para_idx);
                    let sb = mp.map(|m| m.spacing_before).unwrap_or(0.0);
                    let sa = mp.map(|m| m.spacing_after).unwrap_or(0.0);
                    let outer_top: f64 = para.controls.iter()
                        .filter_map(|c| match c {
                            Control::Table(t) if t.common.treat_as_char =>
                                Some(crate::renderer::hwpunit_to_px(t.outer_margin_top as i32, self.dpi)),
                            _ => None,
                        })
                        .sum();
                    let is_col_top = height_before_controls < 1.0;
                    let effective_sb = if is_col_top { 0.0 } else { sb };
                    // TAC 블록 표 문단의 post-text 줄 높이 (마지막 LINE_SEG)
                    let post_text_h = if para.line_segs.len() > tac_count {
                        para.line_segs.last()
                            .map(|seg| crate::renderer::hwpunit_to_px(seg.line_height + seg.line_spacing, self.dpi))
                            .unwrap_or(0.0)
                    } else { 0.0 };
                    (effective_sb + outer_top + tac_seg_total + post_text_h + sa).min(para_height)
                } else {
                    para_height
                };
                if height_added > cap {
                    st.current_height = height_before_controls + cap;
                }

                // 표 감지: 시각적 높이 저장 + Fixed 누적 시작 (Task #9)
                // TAC 표의 높이는 이미 paginate_table_control에서 current_height에 반영됨
                // fix_overlay는 고정값→글자에따라 전환이 있는 경우에만 유효
                if let Some(seg) = para.line_segs.first() {
                    if seg.line_spacing < 0 {
                        fix_table_visual_h = crate::renderer::hwpunit_to_px(seg.line_height, self.dpi);
                        fix_vpos_tmp = 0.0;
                        fix_overlay_active = true;
                    } else if has_tac_block_table {
                        // 양수 ls의 TAC 표: fix_overlay 리셋
                        // 이전 표의 fix_table_visual_h를 후속 비-표 문단에 이중 적용 방지
                        fix_overlay_active = false;
                    }
                }
            }

            // Fixed 문단: 높이를 fix_vpos_tmp에 누적 (current_height는 건드리지 않음)
            if fix_overlay_active && !has_table {
                fix_vpos_tmp += para_height;
            }

        }

        // 마지막 남은 항목 처리
        if !st.current_items.is_empty() {
            st.flush_column_always();
        }

        // 빈 문서인 경우 최소 1페이지 보장
        st.ensure_page();

        // 전체 어울림 리턴 문단 수집
        let mut all_wrap_around_paras = Vec::new();
        for page in &mut st.pages {
            for col in &mut page.column_contents {
                all_wrap_around_paras.append(&mut col.wrap_around_paras);
            }
        }
        // 페이지 번호 + 머리말/꼬리말 할당
        Self::finalize_pages(&mut st.pages, &hf_entries, &page_number_pos, &page_hides, &new_page_numbers, section_index);

        PaginationResult { pages: st.pages, wrap_around_paras: all_wrap_around_paras, hidden_empty_paras }
    }

    /// 머리말/꼬리말/쪽 번호 위치/새 번호 컨트롤 수집
    fn collect_header_footer_controls(
        paragraphs: &[Paragraph],
        section_index: usize,
    ) -> (
        Vec<(usize, HeaderFooterRef, bool, HeaderFooterApply)>,
        Option<crate::model::control::PageNumberPos>,
        Vec<(usize, crate::model::control::PageHide)>,
        Vec<(usize, u16)>,
    ) {
        let mut hf_entries: Vec<(usize, HeaderFooterRef, bool, HeaderFooterApply)> = Vec::new();
        let mut page_number_pos: Option<crate::model::control::PageNumberPos> = None;
        // (para_index, PageHide) — 각 PageHide가 속한 문단 인덱스
        let mut page_hides: Vec<(usize, crate::model::control::PageHide)> = Vec::new();
        let mut new_page_numbers: Vec<(usize, u16)> = Vec::new();

        for (pi, para) in paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                match ctrl {
                    Control::Header(h) => {
                        let r = HeaderFooterRef { para_index: pi, control_index: ci, source_section_index: section_index };
                        hf_entries.push((pi, r, true, h.apply_to));
                    }
                    Control::Footer(f) => {
                        let r = HeaderFooterRef { para_index: pi, control_index: ci, source_section_index: section_index };
                        hf_entries.push((pi, r, false, f.apply_to));
                    }
                    Control::PageHide(ph) => {
                        page_hides.push((pi, ph.clone()));
                    }
                    Control::PageNumberPos(pnp) => {
                        page_number_pos = Some(pnp.clone());
                    }
                    Control::NewNumber(nn) => {
                        if nn.number_type == crate::model::control::AutoNumberType::Page {
                            new_page_numbers.push((pi, nn.number));
                        }
                    }
                    Control::Table(table) => {
                        Self::collect_pagehide_in_table(table, pi, &mut page_hides);
                    }
                    _ => {}
                }
            }
        }

        (hf_entries, page_number_pos, page_hides, new_page_numbers)
    }

    /// 표 셀 안 paragraph 의 PageHide 를 재귀 수집.
    /// 외부 paragraph index `pi` 를 그대로 사용해 페이지 매핑 정합성 유지.
    fn collect_pagehide_in_table(
        table: &crate::model::table::Table,
        pi: usize,
        page_hides: &mut Vec<(usize, crate::model::control::PageHide)>,
    ) {
        for cell in &table.cells {
            for cp in &cell.paragraphs {
                for ctrl in &cp.controls {
                    match ctrl {
                        Control::PageHide(ph) => {
                            page_hides.push((pi, ph.clone()));
                        }
                        Control::Table(inner) => {
                            Self::collect_pagehide_in_table(inner, pi, page_hides);
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    /// 다단 나누기 처리
    fn process_multicolumn_break(
        &self,
        st: &mut PaginationState,
        para_idx: usize,
        paragraphs: &[Paragraph],
        page_def: &PageDef,
    ) {
        st.flush_column();

        // 이전 존의 높이를 zone_y_offset에 누적
        let vpos_zone_height = if para_idx > 0 {
            let mut max_vpos_end: i32 = 0;
            for prev_idx in (0..para_idx).rev() {
                if let Some(last_seg) = paragraphs[prev_idx].line_segs.last() {
                    let vpos_end = last_seg.vertical_pos + last_seg.line_height + last_seg.line_spacing;
                    if vpos_end > max_vpos_end {
                        max_vpos_end = vpos_end;
                    }
                    break;
                }
            }
            if max_vpos_end > 0 {
                crate::renderer::hwpunit_to_px(max_vpos_end, self.dpi)
            } else {
                st.current_height
            }
        } else {
            st.current_height
        };
        st.current_zone_y_offset += vpos_zone_height;
        st.current_column = 0;
        st.current_height = 0.0;
        st.on_first_multicolumn_page = true;

        // 새 ColumnDef 찾기
        for ctrl in &paragraphs[para_idx].controls {
            if let Control::ColumnDef(cd) = ctrl {
                st.col_count = cd.column_count.max(1);
                let new_layout = PageLayoutInfo::from_page_def(page_def, cd, self.dpi);
                st.current_zone_layout = Some(new_layout.clone());
                st.layout = new_layout;
                break;
            }
        }
    }

    /// 단 나누기 처리
    fn process_column_break(&self, st: &mut PaginationState) {
        st.advance_column_or_new_page();
    }

    /// 쪽 나누기 처리
    fn process_page_break(&self, st: &mut PaginationState) {
        st.force_new_page();
    }

    /// 비-표 문단의 줄 단위 분할
    fn paginate_text_lines(
        &self,
        st: &mut PaginationState,
        para_idx: usize,
        para: &Paragraph,
        measured: &MeasuredSection,
        para_height: f64,
        base_available_height: f64,
        respect_vpos_reset: bool,
    ) {
        let available_now = st.available_height();

        // LINE_SEG vpos-reset 강제 분리 지점 검출 (line>0 && vertical_pos==0)
        // 옵션 on + multicolumn이 아닌 경우에만 적용. multicolumn은 column-break 메커니즘 우선.
        let forced_breaks: Vec<usize> = if respect_vpos_reset {
            para.line_segs.iter().enumerate()
                .filter(|(i, ls)| *i > 0 && ls.vertical_pos == 0)
                .map(|(i, _)| i)
                .collect()
        } else {
            Vec::new()
        };

        // 다단 레이아웃에서 문단 내 단 경계 감지
        // [Task #459] on_first_multicolumn_page 가드 제거: 다단 구역이 여러 페이지에 걸칠 때
        // 후속 페이지에서도 LINE_SEG vpos-reset 으로 인코딩된 단 경계를 인식해야 함.
        let col_breaks = if st.col_count > 1 && st.current_column == 0 {
            Self::detect_column_breaks_in_paragraph(para)
        } else {
            vec![0]
        };

        if col_breaks.len() > 1 {
            self.paginate_multicolumn_paragraph(st, para_idx, para, measured, para_height, &col_breaks);
        } else if !forced_breaks.is_empty() {
            self.paginate_with_forced_breaks(st, para_idx, para, measured, &forced_breaks, base_available_height);
        } else if {
            // 문단 적합성 검사: trailing line_spacing 제외
            let trailing_ls = para.line_segs.last()
                .map(|seg| crate::renderer::hwpunit_to_px(seg.line_spacing, self.dpi))
                .unwrap_or(0.0);
            // 페이지 하단 여유가 적으면(full para_height 기준 넘침) trailing 제외 비율 축소
            // → 렌더링과 페이지네이션 간 누적 오차로 인한 overflow 방지
            let effective_trailing = if st.current_height + para_height > available_now {
                let margin = available_now - st.current_height;
                // 남은 공간이 para_height의 절반 이하면 trailing 제외 안 함
                if margin < para_height * 0.5 {
                    0.0
                } else {
                    trailing_ls
                }
            } else {
                trailing_ls
            };
            // 부동소수점 누적 오차 허용 (0.5px ≈ 0.13mm)
            st.current_height + (para_height - effective_trailing) <= available_now + 0.5
        } {
            // 문단 전체가 현재 페이지에 들어감
            st.current_items.push(PageItem::FullParagraph {
                para_index: para_idx,
            });
            st.current_height += para_height;
        } else if let Some(mp) = measured.get_measured_paragraph(para_idx) {
            // 문단이 페이지를 초과 → 줄 단위 분할
            let line_count = mp.line_heights.len();
            let sp_before = mp.spacing_before;
            let sp_after = mp.spacing_after;

            if line_count == 0 {
                st.current_items.push(PageItem::FullParagraph {
                    para_index: para_idx,
                });
                st.current_height += para_height;
            } else {
                // 남은 공간이 없거나 첫 줄도 못 넣으면 플러시
                let first_line_h = mp.line_heights.first().copied().unwrap_or(0.0);
                let remaining_for_lines = (available_now - st.current_height).max(0.0);
                if (st.current_height >= available_now || remaining_for_lines < first_line_h)
                    && !st.current_items.is_empty()
                {
                    st.advance_column_or_new_page();
                }

                // 줄 단위 분할 루프
                let mut cursor_line: usize = 0;
                while cursor_line < line_count {
                    let fn_margin = if st.current_footnote_height > 0.0 { st.footnote_safety_margin } else { 0.0 };
                    let page_avail = if cursor_line == 0 {
                        (base_available_height - st.current_footnote_height - fn_margin - st.current_height - st.current_zone_y_offset).max(0.0)
                    } else {
                        base_available_height
                    };

                    let sp_b = if cursor_line == 0 { sp_before } else { 0.0 };
                    let avail_for_lines = (page_avail - sp_b).max(0.0);

                    // 현재 페이지에 들어갈 줄 범위 결정
                    let mut cumulative = 0.0;
                    let mut end_line = cursor_line;
                    for li in cursor_line..line_count {
                        let content_h = mp.line_heights[li];
                        if cumulative + content_h > avail_for_lines && li > cursor_line {
                            break;
                        }
                        cumulative += mp.line_advance(li);
                        end_line = li + 1;
                    }

                    if end_line <= cursor_line {
                        end_line = cursor_line + 1;
                    }

                    let part_line_height: f64 = mp.line_advances_sum(cursor_line..end_line);
                    let part_sp_after = if end_line >= line_count { sp_after } else { 0.0 };
                    let part_height = sp_b + part_line_height + part_sp_after;

                    if cursor_line == 0 && end_line >= line_count {
                        // 전체가 배치되었지만 오버플로 확인
                        let prev_is_table = st.current_items.last().map_or(false, |item| {
                            matches!(item, PageItem::Table { .. } | PageItem::PartialTable { .. })
                        });
                        let overflow_threshold = if prev_is_table {
                            let trailing_ls = mp.line_spacings.get(end_line.saturating_sub(1)).copied().unwrap_or(0.0);
                            cumulative - trailing_ls
                        } else {
                            cumulative
                        };
                        if overflow_threshold > avail_for_lines && !st.current_items.is_empty() {
                            st.advance_column_or_new_page();
                            continue;
                        }
                        st.current_items.push(PageItem::FullParagraph {
                            para_index: para_idx,
                        });
                        // vpos 기준점: 페이지 분할 후 FP으로 배치된 경우
                        if st.page_vpos_base.is_none() {
                            if let Some(seg) = para.line_segs.first() {
                                st.page_vpos_base = Some(seg.vertical_pos);
                            }
                        }
                    } else {
                        st.current_items.push(PageItem::PartialParagraph {
                            para_index: para_idx,
                            start_line: cursor_line,
                            end_line,
                        });
                        // vpos 기준점: 페이지 분할 후 PP로 배치된 경우
                        if st.page_vpos_base.is_none() {
                            if let Some(seg) = para.line_segs.get(cursor_line) {
                                st.page_vpos_base = Some(seg.vertical_pos);
                            }
                        }
                    }
                    st.current_height += part_height;

                    if end_line >= line_count {
                        break;
                    }

                    // 나머지 줄 → 다음 단 또는 새 페이지
                    st.advance_column_or_new_page();
                    cursor_line = end_line;

                    // 새 페이지 시작 시 vpos 기준점 설정 (분할 시작 줄 기준)
                    // layout은 PartialParagraph의 start_line seg vpos를 base로 사용
                    if st.page_vpos_base.is_none() {
                        if let Some(seg) = para.line_segs.get(end_line) {
                            st.page_vpos_base = Some(seg.vertical_pos);
                        }
                    }
                }
            }
        } else {
            // MeasuredParagraph 없음 (fallback)
            st.current_items.push(PageItem::FullParagraph {
                para_index: para_idx,
            });
            st.current_height += para_height;
        }
    }

    /// LINE_SEG vpos-reset에 의한 강제 분리 처리.
    ///
    /// HWP 파일이 LINE_SEG.vertical_pos=0 으로 표시한 단/페이지 경계를 존중하여,
    /// 문단을 forced_breaks 위치에서 PartialParagraph로 분리하고 단/페이지를 진행한다.
    ///
    /// 각 세그먼트가 단일 단/페이지를 초과할 경우 자연 줄 분할로 fallback.
    fn paginate_with_forced_breaks(
        &self,
        st: &mut PaginationState,
        para_idx: usize,
        para: &Paragraph,
        measured: &MeasuredSection,
        forced_breaks: &[usize],
        base_available_height: f64,
    ) {
        let Some(mp) = measured.get_measured_paragraph(para_idx) else {
            // 측정 정보 없음 → fallback FullParagraph
            st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
            return;
        };

        let line_count = mp.line_heights.len();
        if line_count == 0 {
            st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
            return;
        }

        let sp_before = mp.spacing_before;
        let sp_after = mp.spacing_after;

        // 세그먼트 경계: [0, fb1, fb2, ..., line_count]
        let mut boundaries: Vec<usize> = vec![0];
        boundaries.extend(forced_breaks.iter().copied().filter(|&b| b > 0 && b < line_count));
        boundaries.push(line_count);
        boundaries.dedup();

        for win_idx in 0..boundaries.len() - 1 {
            let seg_start = boundaries[win_idx];
            let seg_end = boundaries[win_idx + 1];
            if seg_start >= seg_end { continue; }
            let is_last_segment = win_idx + 2 == boundaries.len();

            // 세그먼트 줄 단위 배치 (자연 분할 + forced break 결합)
            let mut cursor_line = seg_start;
            while cursor_line < seg_end {
                let fn_margin = if st.current_footnote_height > 0.0 { st.footnote_safety_margin } else { 0.0 };
                let page_avail = if cursor_line == seg_start && win_idx == 0 {
                    (base_available_height - st.current_footnote_height - fn_margin - st.current_height - st.current_zone_y_offset).max(0.0)
                } else {
                    base_available_height
                };

                let sp_b = if cursor_line == 0 { sp_before } else { 0.0 };
                let avail_for_lines = (page_avail - sp_b).max(0.0);

                // 세그먼트 안에서만 줄 누적 (seg_end 초과 금지)
                // [Task #643] 마지막 줄은 자체 line_height 만 차지 (트레일링 line_spacing 제외)
                // 트레일링 ls 는 다음 줄/문단으로의 간격이며, 세그먼트 마지막 줄에는 불필요.
                let mut cumulative = 0.0;
                let mut end_line = cursor_line;
                for li in cursor_line..seg_end {
                    let content_h = mp.line_heights[li];
                    if cumulative + content_h > avail_for_lines && li > cursor_line {
                        break;
                    }
                    cumulative += if li + 1 < seg_end {
                        mp.line_advance(li)
                    } else {
                        mp.line_heights[li]
                    };
                    end_line = li + 1;
                }
                if end_line <= cursor_line {
                    end_line = cursor_line + 1;
                }

                // [Task #643] part_line_height 도 동일 산식: 마지막 줄은 lh 만
                let part_line_height: f64 = if end_line > cursor_line {
                    let advances = mp.line_advances_sum(cursor_line..end_line.saturating_sub(1));
                    let last_lh = mp.line_heights.get(end_line - 1).copied().unwrap_or(0.0);
                    advances + last_lh
                } else {
                    0.0
                };
                let part_sp_after = if end_line >= line_count { sp_after } else { 0.0 };
                let part_height = sp_b + part_line_height + part_sp_after;

                // 첫 줄도 안 들어가면 단/페이지 진행 후 재시도
                let first_line_h = mp.line_heights.get(cursor_line).copied().unwrap_or(0.0);
                let remaining_for_lines = (st.available_height() - st.current_height).max(0.0);
                if (st.current_height >= st.available_height() || remaining_for_lines < first_line_h)
                    && !st.current_items.is_empty()
                {
                    st.advance_column_or_new_page();
                    continue;
                }

                // 세그먼트 전체가 한 번에 배치되었고 문단 전체이면 FullParagraph
                if cursor_line == 0 && end_line >= line_count {
                    st.current_items.push(PageItem::FullParagraph { para_index: para_idx });
                } else {
                    st.current_items.push(PageItem::PartialParagraph {
                        para_index: para_idx,
                        start_line: cursor_line,
                        end_line,
                    });
                }

                if st.page_vpos_base.is_none() {
                    if let Some(seg) = para.line_segs.get(cursor_line) {
                        st.page_vpos_base = Some(seg.vertical_pos);
                    }
                }
                st.current_height += part_height;

                cursor_line = end_line;

                if cursor_line < seg_end {
                    // 세그먼트 내부 자연 분할 → 다음 단/페이지
                    st.advance_column_or_new_page();
                }
            }

            // 세그먼트 종료 시점이 마지막이 아니면 강제 분리 (vpos-reset)
            if !is_last_segment {
                st.advance_column_or_new_page();
            }
        }
    }

    /// 다단 문단의 단별 PartialParagraph 분할
    fn paginate_multicolumn_paragraph(
        &self,
        st: &mut PaginationState,
        para_idx: usize,
        para: &Paragraph,
        measured: &MeasuredSection,
        para_height: f64,
        col_breaks: &[usize],
    ) {
        let line_count = para.line_segs.len();
        let measured_line_count = measured.get_measured_paragraph(para_idx)
            .map(|mp| mp.line_heights.len())
            .unwrap_or(line_count);
        for (bi, &break_start) in col_breaks.iter().enumerate() {
            let break_end = if bi + 1 < col_breaks.len() {
                col_breaks[bi + 1]
            } else {
                line_count
            };

            let safe_start = break_start.min(measured_line_count);
            let safe_end = break_end.min(measured_line_count);
            let part_height: f64 = if safe_start < safe_end {
                if let Some(mp) = measured.get_measured_paragraph(para_idx) {
                    mp.line_advances_sum(safe_start..safe_end)
                } else {
                    para_height / col_breaks.len() as f64
                }
            } else {
                para_height / col_breaks.len() as f64
            };

            if break_start == 0 && break_end == line_count {
                st.current_items.push(PageItem::FullParagraph {
                    para_index: para_idx,
                });
            } else {
                st.current_items.push(PageItem::PartialParagraph {
                    para_index: para_idx,
                    start_line: break_start,
                    end_line: break_end,
                });
            }
            st.current_height += part_height;

            // 마지막 부분이 아니면 다음 단으로 이동
            if bi + 1 < col_breaks.len() {
                st.advance_column_or_new_page();
            }
        }
    }

    /// 인라인 컨트롤 처리 (표/도형/각주)
    fn process_controls(
        &self,
        st: &mut PaginationState,
        para_idx: usize,
        para: &Paragraph,
        measured: &MeasuredSection,
        measurer: &HeightMeasurer,
        para_height: f64,
        para_height_for_fit: f64,
        base_available_height: f64,
        page_def: &PageDef,
        para_start_height: f64,
    ) {
        for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
            match ctrl {
                Control::Table(table) => {
                    // 글앞으로 / 글뒤로: Shape처럼 취급 — 공간 차지 없음
                    if matches!(table.common.text_wrap, crate::model::shape::TextWrap::InFrontOfText | crate::model::shape::TextWrap::BehindText) {
                        st.current_items.push(PageItem::Shape {
                            para_index: para_idx,
                            control_index: ctrl_idx,
                        });
                        continue;
                    }
                    // 페이지 하단/중앙 고정 비-TAC 표 (vert=Page/Paper + Bottom/Center):
                    // 본문 흐름 무관 — 현재 페이지에 배치하고 높이 미추가
                    if !table.common.treat_as_char
                        && matches!(table.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                        && matches!(table.common.vert_rel_to,
                            crate::model::shape::VertRelTo::Page | crate::model::shape::VertRelTo::Paper)
                        && matches!(table.common.vert_align,
                            crate::model::shape::VertAlign::Bottom | crate::model::shape::VertAlign::Center)
                    {
                        st.current_items.push(PageItem::Table {
                            para_index: para_idx,
                            control_index: ctrl_idx,
                        });
                        continue;
                    }
                    // treat_as_char 표: 인라인이면 skip
                    if table.common.treat_as_char {
                        let seg_w = para.line_segs.first().map(|s| s.segment_width).unwrap_or(0);
                        if crate::renderer::height_measurer::is_tac_table_inline(table, seg_w, &para.text, &para.controls) {
                            continue;
                        }
                    }
                    self.paginate_table_control(
                        st, para_idx, ctrl_idx, para, measured, measurer,
                        para_height, para_height_for_fit, base_available_height,
                        para_start_height,
                    );
                }
                Control::Shape(shape_obj) => {
                    // [Issue #476] treat_as_char Shape 는 박스가 속한 line 이 라우팅된 페이지/단에 등록.
                    // paragraph 가 페이지 분할되면 process_controls 시점에 st.current_items 는 마지막
                    // 페이지 상태이므로, 그대로 push 하면 박스가 잘못된 페이지에 떠 있게 된다.
                    let routed = if shape_obj.common().treat_as_char {
                        super::find_inline_control_target_page(
                            &st.pages, &st.current_items, para_idx, ctrl_idx, para,
                        )
                    } else {
                        None
                    };
                    let item = PageItem::Shape {
                        para_index: para_idx,
                        control_index: ctrl_idx,
                    };
                    match routed {
                        Some((page_idx, col_idx)) => {
                            // 이전 페이지의 해당 단 items 에 직접 push
                            if let Some(page) = st.pages.get_mut(page_idx) {
                                if let Some(col) = page.column_contents.get_mut(col_idx) {
                                    col.items.push(item);
                                } else {
                                    st.current_items.push(item);
                                }
                            } else {
                                st.current_items.push(item);
                            }
                        }
                        None => {
                            st.current_items.push(item);
                        }
                    }
                    // 글상자 내 각주 수집
                    if let Some(text_box) = shape_obj.drawing().and_then(|d| d.text_box.as_ref()) {
                        for (tp_idx, tp) in text_box.paragraphs.iter().enumerate() {
                            for (tc_idx, tc) in tp.controls.iter().enumerate() {
                                if let Control::Footnote(fn_ctrl) = tc {
                                    if let Some(page) = st.pages.last_mut() {
                                        page.footnotes.push(FootnoteRef {
                                            number: fn_ctrl.number,
                                            source: FootnoteSource::ShapeTextBox {
                                                para_index: para_idx,
                                                shape_control_index: ctrl_idx,
                                                tb_para_index: tp_idx,
                                                tb_control_index: tc_idx,
                                            },
                                        });
                                        let fn_height = measurer.estimate_single_footnote_height(&fn_ctrl);
                                        st.add_footnote_height(fn_height);
                                    }
                                }
                            }
                        }
                    }
                }
                Control::Picture(pic) => {
                    st.current_items.push(PageItem::Shape {
                        para_index: para_idx,
                        control_index: ctrl_idx,
                    });
                    // 비-TAC 그림: 본문 공간을 차지하는 배치이면 높이 추가 (Task #10)
                    if !pic.common.treat_as_char
                        && matches!(pic.common.text_wrap,
                            crate::model::shape::TextWrap::Square
                            | crate::model::shape::TextWrap::TopAndBottom)
                    {
                        let pic_h = crate::renderer::hwpunit_to_px(pic.common.height as i32, self.dpi);
                        let margin_top = crate::renderer::hwpunit_to_px(pic.common.margin.top as i32, self.dpi);
                        let margin_bottom = crate::renderer::hwpunit_to_px(pic.common.margin.bottom as i32, self.dpi);
                        st.current_height += pic_h + margin_top + margin_bottom;
                    }
                }
                Control::Equation(_) => {
                    st.current_items.push(PageItem::Shape {
                        para_index: para_idx,
                        control_index: ctrl_idx,
                    });
                }
                Control::Footnote(fn_ctrl) => {
                    if let Some(page) = st.pages.last_mut() {
                        page.footnotes.push(FootnoteRef {
                            number: fn_ctrl.number,
                            source: FootnoteSource::Body {
                                para_index: para_idx,
                                control_index: ctrl_idx,
                            },
                        });
                        let fn_height = measurer.estimate_single_footnote_height(fn_ctrl);
                        st.add_footnote_height(fn_height);
                    }
                }
                _ => {}
            }
        }
    }

    /// 표 페이지 분할
    fn paginate_table_control(
        &self,
        st: &mut PaginationState,
        para_idx: usize,
        ctrl_idx: usize,
        para: &Paragraph,
        measured: &MeasuredSection,
        measurer: &HeightMeasurer,
        para_height: f64,
        para_height_for_fit: f64,
        base_available_height: f64,
        para_start_height: f64,
    ) {
        let table = if let Control::Table(t) = &para.controls[ctrl_idx] { t } else { return };
        let measured_table = measured.get_measured_table(para_idx, ctrl_idx);
        // 표 본체 높이 (캡션 제외 — 캡션은 host_spacing/caption_overhead에서 별도 처리)
        let effective_height = measured_table
            .map(|mt| {
                let cap_h = mt.caption_height;
                let cap_s = if cap_h > 0.0 {
                    table.caption.as_ref()
                        .map(|c| crate::renderer::hwpunit_to_px(c.spacing as i32, self.dpi))
                        .unwrap_or(0.0)
                } else { 0.0 };
                mt.total_height - cap_h - cap_s
            })
            .unwrap_or_else(|| {
                let row_count = table.row_count as usize;
                let mut row_heights = vec![0.0f64; row_count];
                for cell in &table.cells {
                    if cell.row_span == 1 && (cell.row as usize) < row_count {
                        let h = crate::renderer::hwpunit_to_px(cell.height as i32, self.dpi);
                        if h > row_heights[cell.row as usize] {
                            row_heights[cell.row as usize] = h;
                        }
                    }
                }
                let table_height: f64 = row_heights.iter().sum();
                if table_height > 0.0 { table_height } else { crate::renderer::hwpunit_to_px(1000, self.dpi) }
            });

        // 표 내 각주 높이 사전 계산
        let mut table_footnote_height = 0.0;
        let mut table_has_footnotes = false;
        for cell in &table.cells {
            for cp in &cell.paragraphs {
                for cc in &cp.controls {
                    if let Control::Footnote(fn_ctrl) = cc {
                        let fn_height = measurer.estimate_single_footnote_height(fn_ctrl);
                        if !table_has_footnotes && st.is_first_footnote_on_page {
                            table_footnote_height += st.footnote_separator_overhead;
                        }
                        table_footnote_height += fn_height;
                        table_has_footnotes = true;
                    }
                }
            }
        }

        // 현재 사용 가능한 높이
        let total_footnote = st.current_footnote_height + table_footnote_height;
        let table_margin = if total_footnote > 0.0 { st.footnote_safety_margin } else { 0.0 };
        let table_available_height = (base_available_height - total_footnote - table_margin - st.current_zone_y_offset).max(0.0);

        // 호스트 문단 간격 계산
        let is_tac_table = table.common.treat_as_char;
        let table_text_wrap = table.common.text_wrap;
        let (host_spacing, host_line_spacing, spacing_before_px) = {
            let mp = measured.get_measured_paragraph(para_idx);
            let sb = mp.map(|m| m.spacing_before).unwrap_or(0.0);
            let sa = mp.map(|m| m.spacing_after).unwrap_or(0.0);
            let outer_top = if is_tac_table {
                crate::renderer::hwpunit_to_px(table.outer_margin_top as i32, self.dpi)
            } else {
                0.0
            };
            // layout_table depth=0은 outer_bottom을 반환값에 포함하지 않음
            let outer_bottom = 0.0;
            // 호스트 문단의 line_spacing: 레이아웃에서 표 아래에 추가
            // TAC 표: ctrl_idx 위치의 LINE_SEG line_spacing 사용
            // 비-TAC 표: 마지막 LINE_SEG line_spacing 사용
            let host_line_spacing = if is_tac_table {
                para.line_segs.get(ctrl_idx)
                    .filter(|seg| seg.line_spacing > 0)
                    .map(|seg| crate::renderer::hwpunit_to_px(seg.line_spacing, self.dpi))
                    .unwrap_or(0.0)
            } else {
                para.line_segs.last()
                    .filter(|seg| seg.line_spacing > 0)
                    .map(|seg| crate::renderer::hwpunit_to_px(seg.line_spacing, self.dpi))
                    .unwrap_or(0.0)
            };
            let is_column_top = st.current_height < 1.0;
            // 자리차지(text_wrap=TopAndBottom) 비-TAC 표:
            // - vert=Paper/Page: spacing_before 제외 (shape_reserved가 y_offset 처리)
            // - vert=Para: spacing_before 포함 (레이아웃에서 문단 상대 위치로 spacing_before 반영)
            let before = if !is_tac_table && matches!(table_text_wrap, crate::model::shape::TextWrap::TopAndBottom) {
                let is_para_relative = matches!(table.common.vert_rel_to, crate::model::shape::VertRelTo::Para);
                if is_para_relative {
                    (if !is_column_top { sb } else { 0.0 }) + outer_top
                } else {
                    outer_top // spacing_before 제외
                }
            } else {
                (if !is_column_top { sb } else { 0.0 }) + outer_top
            };
            // spacing_before_px: 레이아웃에서 표 배치 전 y_offset을 전진시키는 양
            // (= before에서 outer_top을 뺀 순수 spacing_before 부분)
            let spacing_before_px = before - outer_top;
            (before + sa + outer_bottom + host_line_spacing, host_line_spacing, spacing_before_px)
        };

        // 문단 내 표 컨트롤 수: 여러 개이면 개별 표 높이 사용
        let tac_table_count = para.controls.iter()
            .filter(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
            .count();
        let table_total_height = if is_tac_table && para_height > 0.0 && tac_table_count <= 1 {
            // TAC 표: 실측 높이 + 호스트 간격
            // trailing ls: 이 표가 페이지 마지막 항목이 될 수 있으면 제외
            // (다음 문단이 없거나, trailing ls 제거 시에만 들어가는 경우)
            let full_h = effective_height + host_spacing;
            let without_trail = full_h - host_line_spacing;
            let remaining = (st.available_height() - st.current_height).max(0.0);
            if without_trail <= remaining + 0.5 && full_h > remaining + 0.5 {
                // trailing ls 제거해야만 들어가는 경계 → 제거 (페이지 마지막)
                without_trail
            } else {
                full_h
            }
        } else if is_tac_table && tac_table_count > 1 {
            // 다중 TAC 표: LINE_SEG 데이터로 개별 표 높이 계산
            // LINE_SEG[k] = k번째 TAC 표의 줄 높이(표 높이 포함) + 줄간격
            let tac_idx = para.controls.iter().take(ctrl_idx)
                .filter(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                .count();
            let is_last_tac = tac_idx + 1 == tac_table_count;
            para.line_segs.get(tac_idx).map(|seg| {
                let line_h = crate::renderer::hwpunit_to_px(seg.line_height, self.dpi);
                if is_last_tac {
                    // 마지막 TAC: line_spacing 제외 (trailing spacing)
                    line_h
                } else {
                    let ls = if seg.line_spacing > 0 {
                        crate::renderer::hwpunit_to_px(seg.line_spacing, self.dpi)
                    } else { 0.0 };
                    line_h + ls
                }
            }).unwrap_or(effective_height + host_spacing)
        } else {
            effective_height + host_spacing
        };

        // 캡션 보정용 높이 (TAC 및 비-TAC 모두 적용):
        // layout_table은 table_bottom = table_y + table_height + caption_extra를 반환하므로
        // current_height에도 Top/Bottom 캡션 높이를 포함해야 레이아웃 y_offset과 일치한다.
        // 피트 판단(effective_table_height)에는 포함하지 않아 기존 배치 로직을 유지한다.
        // Left/Right 캡션은 layout_table에서 caption_extra=0이므로 제외한다.
        let caption_extra_for_current = if let Some(mt) = measured_table {
            if mt.caption_height > 0.0 {
                let is_lr = table.caption.as_ref().map_or(false, |c| {
                    use crate::model::shape::CaptionDirection;
                    matches!(c.direction, CaptionDirection::Left | CaptionDirection::Right)
                });
                if !is_lr {
                    let cap_s = table.caption.as_ref()
                        .map(|c| crate::renderer::hwpunit_to_px(c.spacing as i32, self.dpi))
                        .unwrap_or(0.0);
                    mt.caption_height + cap_s
                } else { 0.0 }
            } else { 0.0 }
        } else { 0.0 };

        // 비-TAC 자리차지 표: vert=Para + vert_offset > 0이면 문단 시작 y 기준으로 피트 판단
        // 같은 문단의 여러 표가 독립적인 vert offset으로 각자 배치되는 경우,
        // current_height(다른 표 처리 후 누적)가 아닌 문단 시작 y 기준으로 절대 하단을 계산한다.
        // 예: ci=2(vert=0mm)와 ci=3(vert=53mm)이 같은 문단에 있을 때,
        //     ci=2 처리 후 current_height가 증가해도 ci=3의 피트는 문단 시작 기준이어야 한다.
        let effective_table_height = if !is_tac_table
            && matches!(table_text_wrap, crate::model::shape::TextWrap::TopAndBottom)
            && matches!(table.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
            && table.common.vertical_offset > 0
        {
            let v_off = crate::renderer::hwpunit_to_px(table.common.vertical_offset as i32, self.dpi);
            // 표의 절대 하단 y = 문단 시작 y + vert_offset + 표 높이
            // 피트 판단식: current_height + effective_table_height <= available
            // 이를 만족하도록 effective_table_height = abs_bottom - current_height
            let abs_bottom = para_start_height + v_off + effective_height + host_spacing;
            if abs_bottom <= base_available_height + 0.5 {
                // 표가 body 범위 내에 완전히 들어옴 → flow height 기여 없음
                0.0
            } else {
                (abs_bottom - st.current_height).max(effective_height + host_spacing)
            }
        } else {
            table_total_height
        };

        // 페이지 하단/중앙 고정 표: 본문 높이에 영향 없음
        // 표가 현재 페이지에 전체 들어가는지 확인
        // 텍스트 문단과 동일한 0.5px 부동소수점 톨러런스 적용
        if st.current_height + effective_table_height <= table_available_height + 0.5 {
            self.place_table_fits(st, para_idx, ctrl_idx, para, measured, table,
                table_total_height, para_height, para_height_for_fit, is_tac_table,
                para_start_height, effective_height, caption_extra_for_current);
        } else if is_tac_table {
            // 글자처럼 취급 표: 페이지에 걸치지 않고 통째로 다음 페이지로 이동
            if !st.current_items.is_empty() {
                st.advance_column_or_new_page();
            }
            self.place_table_fits(st, para_idx, ctrl_idx, para, measured, table,
                table_total_height, para_height, para_height_for_fit, is_tac_table,
                para_start_height, effective_height, caption_extra_for_current);
        } else if let Some(mt) = measured_table {
            // 비-TAC 표: 행 단위 분할
            self.split_table_rows(st, para_idx, ctrl_idx, para, measured, measurer, mt,
                table, table_available_height, base_available_height,
                host_spacing, spacing_before_px, is_tac_table);
        } else {
            // MeasuredTable 없으면 기존 방식 (전체 배치)
            if !st.current_items.is_empty() {
                st.advance_column_or_new_page();
            }
            st.current_items.push(PageItem::Table {
                para_index: para_idx,
                control_index: ctrl_idx,
            });
            st.current_height += effective_height;
        }

        // 표 셀 내 각주 수집
        for (cell_idx, cell) in table.cells.iter().enumerate() {
            for (cp_idx, cp) in cell.paragraphs.iter().enumerate() {
                for (cc_idx, cc) in cp.controls.iter().enumerate() {
                    if let Control::Footnote(fn_ctrl) = cc {
                        if let Some(page) = st.pages.last_mut() {
                            page.footnotes.push(FootnoteRef {
                                number: fn_ctrl.number,
                                source: FootnoteSource::TableCell {
                                    para_index: para_idx,
                                    table_control_index: ctrl_idx,
                                    cell_index: cell_idx,
                                    cell_para_index: cp_idx,
                                    cell_control_index: cc_idx,
                                },
                            });
                            let fn_height = measurer.estimate_single_footnote_height(fn_ctrl);
                            st.add_footnote_height(fn_height);
                        }
                    }
                }
            }
        }
    }

    /// 표가 현재 페이지에 전체 들어가는 경우
    fn place_table_fits(
        &self,
        st: &mut PaginationState,
        para_idx: usize,
        ctrl_idx: usize,
        para: &Paragraph,
        measured: &MeasuredSection,
        table: &crate::model::table::Table,
        table_total_height: f64,
        para_height: f64,
        para_height_for_fit: f64,
        is_tac_table: bool,
        para_start_height: f64,
        effective_height: f64,
        caption_extra_for_current: f64,
    ) {
        let vertical_offset = Self::get_table_vertical_offset(table);
        // 어울림 표(text_wrap=0)는 호스트 텍스트를 wrap 영역에서 처리
        let is_wrap_around_table = !table.common.treat_as_char && matches!(table.common.text_wrap, crate::model::shape::TextWrap::Square);

        if let Some(mp) = measured.get_measured_paragraph(para_idx) {
            let total_lines = mp.line_heights.len();

            // 강제 줄넘김 후 TAC 표: 텍스트가 표 앞에 있음 (Task #19)
            let has_forced_linebreak = is_tac_table && para.text.contains('\n');
            let pre_table_end_line = if vertical_offset > 0 && !para.text.is_empty() {
                total_lines
            } else if has_forced_linebreak && total_lines > 1 {
                // 강제 줄넘김 전 텍스트 줄 수 = \n 개수
                let newline_count = para.text.chars().filter(|&c| c == '\n').count();
                newline_count.min(total_lines - 1)
            } else {
                0
            };

            // 표 앞 텍스트 배치 (첫 번째 표에서만, 중복 방지)
            // 어울림 표는 wrap 영역에서 텍스트 처리하므로 건너뜀
            let is_first_table = !para.controls.iter().take(ctrl_idx)
                .any(|c| matches!(c, Control::Table(_)));
            if pre_table_end_line > 0 && is_first_table && !is_wrap_around_table {
                // 강제 줄넘김+TAC 표: th 기반으로 텍스트 줄 높이 계산 (Task #19)
                let pre_height: f64 = if has_forced_linebreak {
                    para.line_segs.iter().take(pre_table_end_line)
                        .map(|seg| {
                            let th = crate::renderer::hwpunit_to_px(seg.text_height, self.dpi);
                            let ls = crate::renderer::hwpunit_to_px(seg.line_spacing, self.dpi);
                            th + ls
                        })
                        .sum()
                } else {
                    mp.line_advances_sum(0..pre_table_end_line)
                };
                st.current_items.push(PageItem::PartialParagraph {
                    para_index: para_idx,
                    start_line: 0,
                    end_line: pre_table_end_line,
                });
                st.current_height += pre_height;
            }

            // 표 배치
            st.current_items.push(PageItem::Table {
                para_index: para_idx,
                control_index: ctrl_idx,
            });
            // 비-TAC 자리차지 표(wrap=TopAndBottom, vert_offset>0, vert=Para):
            // 피트 판단은 문단 시작 y 기준 독립 배치이지만,
            // 후속 문단은 이 표의 하단 이후에 배치되어야 하므로
            // current_height = max(current_height, para_start_height + v_off + 표높이)
            let is_independent_float = !is_tac_table
                && matches!(table.common.text_wrap, crate::model::shape::TextWrap::TopAndBottom)
                && matches!(table.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
                && table.common.vertical_offset > 0;
            if is_independent_float {
                let v_off = crate::renderer::hwpunit_to_px(table.common.vertical_offset as i32, self.dpi);
                let float_bottom = para_start_height + v_off + effective_height;
                if float_bottom > st.current_height {
                    st.current_height = float_bottom;
                }
            } else {
                // caption_extra_for_current: 비-TAC Top/Bottom 캡션 높이
                // layout_table은 table_bottom에 캡션을 포함해 반환하므로 current_height에도 포함한다.
                // TAC 표 및 Left/Right 캡션 표는 caption_extra_for_current=0.0
                st.current_height += table_total_height + caption_extra_for_current;
            }

            // 표 뒤 텍스트 배치
            // 다중 TAC 표 문단인 경우: 각 LINE_SEG가 개별 표의 높이를 담고 있으므로
            // post-text를 추가하면 뒤 표들의 LINE_SEG 높이가 이중으로 계산됨 → 스킵
            let tac_table_count = para.controls.iter()
                .filter(|c| matches!(c, Control::Table(t) if t.common.treat_as_char))
                .count();
            // 현재 표가 문단 내 마지막 표인지 확인 (중복 텍스트 방지)
            let is_last_table = !para.controls.iter().skip(ctrl_idx + 1)
                .any(|c| matches!(c, Control::Table(_)));
            let post_table_start = if has_forced_linebreak && pre_table_end_line > 0 {
                // 강제 줄넘김 후 TAC 표: 표 이후 post-text 없음 (Task #19)
                total_lines
            } else if table.common.treat_as_char {
                pre_table_end_line.max(1)
            } else if is_last_table && !is_first_table {
                // 다중 표 문단의 마지막 표: pre-table 텍스트는 첫 표에서 처리했으므로
                // 남은 텍스트 줄을 post-table로 배치
                0
            } else {
                pre_table_end_line
            };
            // 중복 방지: 이전 표가 이미 같은 문단의 pre-text(start_line=0)를 추가했으면 건너뜀
            let pre_text_exists = post_table_start == 0 && st.current_items.iter().any(|item| {
                matches!(item, PageItem::PartialParagraph { para_index, start_line, .. }
                    if *para_index == para_idx && *start_line == 0)
            });
            if is_last_table && tac_table_count <= 1 && !para.text.is_empty() && total_lines > post_table_start && !is_wrap_around_table && !pre_text_exists {
                let post_height: f64 = mp.line_advances_sum(post_table_start..total_lines);
                st.current_items.push(PageItem::PartialParagraph {
                    para_index: para_idx,
                    start_line: post_table_start,
                    end_line: total_lines,
                });
                st.current_height += post_height;
            }

            // TAC 표: trailing line_spacing 복원 불필요
            // effective_height + host_spacing 기반 높이를 사용하므로
            // LINE_SEG trailing을 별도 추가하지 않는다.
        } else {
            st.current_items.push(PageItem::Table {
                para_index: para_idx,
                control_index: ctrl_idx,
            });
            st.current_height += table_total_height + caption_extra_for_current;
        }
    }

    /// 표 행 단위 분할
    fn split_table_rows(
        &self,
        st: &mut PaginationState,
        para_idx: usize,
        ctrl_idx: usize,
        para: &Paragraph,
        measured: &MeasuredSection,
        measurer: &HeightMeasurer,
        mt: &crate::renderer::height_measurer::MeasuredTable,
        table: &crate::model::table::Table,
        table_available_height: f64,
        base_available_height: f64,
        host_spacing: f64,
        spacing_before_px: f64,
        _is_tac_table: bool,
    ) {
        let row_count = mt.row_heights.len();
        let cs = mt.cell_spacing;
        let header_row_height = if row_count > 0 { mt.row_heights[0] } else { 0.0 };

        // 호스트 문단 텍스트 높이 계산 (예: <붙임2>)
        // 표의 v_offset으로 호스트 텍스트 공간이 확보되므로,
        // 별도 PageItem이 아닌 가용 높이 차감으로 처리
        // (레이아웃 코드가 PartialTable의 호스트 텍스트를 직접 렌더링함)
        let vertical_offset = Self::get_table_vertical_offset(table);
        let host_text_height = if vertical_offset > 0 && !para.text.is_empty() {
            let is_first_table = !para.controls.iter().take(ctrl_idx)
                .any(|c| matches!(c, Control::Table(_)));
            if is_first_table {
                measured.get_measured_paragraph(para_idx)
                    .map(|mp| mp.line_advances_sum(0..mp.line_heights.len()))
                    .unwrap_or(0.0)
            } else {
                0.0
            }
        } else {
            0.0
        };

        // vertical_offset: 레이아웃에서 표 위에 v_offset만큼 공간을 확보하므로 가용 높이 차감
        let v_offset_px = if vertical_offset > 0 {
            crate::renderer::hwpunit_to_px(vertical_offset as i32, self.dpi)
        } else {
            0.0
        };
        let remaining_on_page = table_available_height - st.current_height - host_text_height - v_offset_px;

        // Task #398 v2: 보호 블록(2~3 rows)만 블록 단위 advance.
        // 큰 rowspan(>3)은 행 단위 분할 허용 (HanCom-compat).
        let (first_block_start, first_block_end, first_block_h) = if row_count > 0 {
            mt.row_block_for(0)
        } else { (0, 0, 0.0) };
        let first_block_size = first_block_end.saturating_sub(first_block_start);
        let first_block_is_single_row = first_block_size == 1;
        // [Task #474] RowBreak 표는 보호 블록 정책 비적용 (HWP 행 경계 분할 정책 정합)
        let first_block_protected = !mt.allows_row_break_split()
            && first_block_size >= 2
            && first_block_size <= crate::renderer::height_measurer::BLOCK_UNIT_MAX_ROWS;
        let can_intra_split_early = !mt.cells.is_empty();
        let split_unit_h = if first_block_protected {
            first_block_h
        } else {
            mt.row_heights.first().copied().unwrap_or(0.0)
        };

        if remaining_on_page < split_unit_h && !st.current_items.is_empty() {
            // 인트라-로우 분할은 단일 행 또는 큰 블록(>3)에서만 시도. 보호 블록은 묶음 단위 advance.
            let first_row_splittable = (first_block_is_single_row || !first_block_protected)
                && can_intra_split_early
                && mt.is_row_splittable(0);
            let min_content = if first_row_splittable {
                mt.min_first_line_height_for_row(0, 0.0) + mt.max_padding_for_row(0)
            } else {
                f64::MAX
            };
            if !first_row_splittable || remaining_on_page < min_content {
                st.advance_column_or_new_page();
            }
        }

        // 캡션 방향
        let caption_is_top = if let Some(Control::Table(t)) = para.controls.get(ctrl_idx) {
            t.caption.as_ref()
                .map(|c| matches!(c.direction, CaptionDirection::Top))
                .unwrap_or(false)
        } else { false };

        // 캡션 높이 계산
        let host_line_spacing_for_caption = para.line_segs.first()
            .map(|seg| crate::renderer::hwpunit_to_px(seg.line_spacing, self.dpi))
            .unwrap_or(0.0);
        let caption_base_overhead = {
            let ch = mt.caption_height;
            if ch > 0.0 {
                let cs_val = if let Some(Control::Table(t)) = para.controls.get(ctrl_idx) {
                    t.caption.as_ref()
                        .map(|c| crate::renderer::hwpunit_to_px(c.spacing as i32, self.dpi))
                        .unwrap_or(0.0)
                } else { 0.0 };
                ch + cs_val
            } else {
                0.0
            }
        };
        let caption_overhead = if caption_base_overhead > 0.0 && !caption_is_top {
            caption_base_overhead + host_line_spacing_for_caption
        } else {
            caption_base_overhead
        };

        // 행 단위 + 행 내부 분할 루프
        let mut cursor_row: usize = 0;
        let mut is_continuation = false;
        let mut content_offset: f64 = 0.0;
        let can_intra_split = !mt.cells.is_empty();

        while cursor_row < row_count {
            // 이전 분할에서 모든 콘텐츠가 소진된 행은 건너뜀
            if content_offset > 0.0 && can_intra_split {
                let rem = mt.remaining_content_for_row(cursor_row, content_offset);
                if rem <= 0.0 {
                    cursor_row += 1;
                    content_offset = 0.0;
                    continue;
                }
            }

            let caption_extra = if !is_continuation && cursor_row == 0 && content_offset == 0.0 && caption_is_top {
                caption_overhead
            } else {
                0.0
            };
            let host_extra = if !is_continuation && cursor_row == 0 && content_offset == 0.0 {
                host_text_height
            } else {
                0.0
            };
            // 첫 분할: v_offset만큼 표가 아래로 밀리므로 가용 높이 차감
            let v_extra = if !is_continuation && cursor_row == 0 && content_offset == 0.0 {
                v_offset_px
            } else {
                0.0
            };
            let page_avail = if is_continuation {
                base_available_height
            } else {
                (table_available_height - st.current_height - caption_extra - host_extra - v_extra).max(0.0)
            };

            let header_overhead = if is_continuation && mt.repeat_header && mt.has_header_cells && row_count > 1 {
                header_row_height + cs
            } else {
                0.0
            };
            // 첫 분할에서 spacing_before만큼 차감:
            // 레이아웃 엔진은 표 배치 전 spacing_before만큼 y_offset을 전진시키지만,
            // page_avail 계산에는 반영되지 않으므로 avail_for_rows에서 보정한다.
            let sb_extra = if !is_continuation && cursor_row == 0 && content_offset == 0.0 {
                spacing_before_px
            } else {
                0.0
            };
            let avail_for_rows = (page_avail - header_overhead - sb_extra).max(0.0);

            let effective_first_row_h = if content_offset > 0.0 && can_intra_split {
                mt.effective_row_height(cursor_row, content_offset)
            } else {
                mt.row_heights[cursor_row]
            };

            // 현재 페이지에 들어갈 행 범위 결정
            let mut end_row = cursor_row;
            let mut split_end_limit: f64 = 0.0;

            {
                const MIN_SPLIT_CONTENT_PX: f64 = 10.0;

                let approx_end_raw = mt.find_break_row(avail_for_rows, cursor_row, effective_first_row_h);
                // Task #398: rowspan 묶음 중간에서 잘리지 않도록 블록 경계로 스냅
                let approx_end = mt.snap_to_block_boundary(approx_end_raw);

                // cursor_row가 속한 블록 정보 (인트라-로우 분할 가드)
                let (cur_b_start, cur_b_end, _) = mt.row_block_for(cursor_row);
                let cur_block_size = cur_b_end.saturating_sub(cur_b_start);
                let cur_block_single = cur_block_size == 1;
                // [Task #474] RowBreak 표는 보호 블록 정책 비적용
                let cur_block_protected = !mt.allows_row_break_split()
                    && cur_block_size >= 2
                    && cur_block_size <= crate::renderer::height_measurer::BLOCK_UNIT_MAX_ROWS;
                // 큰 블록(>3) 또는 단일 행은 분할 가능; 보호 블록(2~3)은 분할 불가
                let cur_can_intra_split = (cur_block_single || !cur_block_protected) && can_intra_split;

                if approx_end <= cursor_row {
                    let r = cursor_row;
                    // 인트라-로우 분할은 보호 블록(2~3)이 아닌 경우 (단일 행 또는 큰 블록>3) 허용
                    let splittable = cur_can_intra_split && mt.is_row_splittable(r);
                    if splittable {
                        let padding = mt.max_padding_for_row(r);
                        let avail_content = (avail_for_rows - padding).max(0.0);
                        let total_content = mt.remaining_content_for_row(r, content_offset);
                        let remaining_content = total_content - avail_content;
                        let min_first_line = mt.min_first_line_height_for_row(r, content_offset);
                        if avail_content >= MIN_SPLIT_CONTENT_PX
                            && avail_content >= min_first_line
                            && remaining_content >= MIN_SPLIT_CONTENT_PX
                        {
                            end_row = r + 1;
                            split_end_limit = avail_content;
                        } else {
                            end_row = r + 1;
                        }
                    } else if cur_can_intra_split && effective_first_row_h > avail_for_rows {
                        // 행이 분할 불가능하지만 페이지보다 클 때: 가용 높이에 맞춰 강제 분할
                        let padding = mt.max_padding_for_row(r);
                        let avail_content = (avail_for_rows - padding).max(0.0);
                        if avail_content >= MIN_SPLIT_CONTENT_PX {
                            end_row = r + 1;
                            split_end_limit = avail_content;
                        } else {
                            end_row = r + 1;
                        }
                    } else if cur_block_protected {
                        // Task #398: 보호 블록(2~3 rows)이 들어가지 않으면 블록 전체 배치.
                        end_row = cur_b_end;
                    } else {
                        end_row = r + 1;
                    }
                } else if approx_end < row_count {
                    end_row = approx_end;
                    let r = approx_end;
                    let delta = if content_offset > 0.0 && can_intra_split {
                        mt.row_heights[cursor_row] - effective_first_row_h
                    } else {
                        0.0
                    };
                    let range_h = mt.range_height(cursor_row, approx_end) - delta;
                    let remaining_avail = avail_for_rows - range_h;
                    // Task #398 v2: 분할 후보 r의 블록 보호 검사 (보호 블록만 분할 차단)
                    let (next_b_start, next_b_end, _) = mt.row_block_for(r);
                    let next_block_size = next_b_end.saturating_sub(next_b_start);
                    let next_block_single = next_block_size == 1;
                    // [Task #474] RowBreak 표는 보호 블록 정책 비적용
                    let next_block_protected = !mt.allows_row_break_split()
                        && next_block_size >= 2
                        && next_block_size <= crate::renderer::height_measurer::BLOCK_UNIT_MAX_ROWS;
                    let next_can_intra_split = (next_block_single || !next_block_protected) && can_intra_split;
                    if next_can_intra_split && mt.is_row_splittable(r) {
                        let row_cs = cs;
                        let padding = mt.max_padding_for_row(r);
                        let avail_content_for_r = (remaining_avail - row_cs - padding).max(0.0);
                        let total_content = mt.remaining_content_for_row(r, 0.0);
                        let remaining_content = total_content - avail_content_for_r;
                        let min_first_line = mt.min_first_line_height_for_row(r, 0.0);
                        if avail_content_for_r >= MIN_SPLIT_CONTENT_PX
                            && avail_content_for_r >= min_first_line
                            && remaining_content >= MIN_SPLIT_CONTENT_PX
                        {
                            end_row = r + 1;
                            split_end_limit = avail_content_for_r;
                        }
                    } else if next_can_intra_split && mt.row_heights[r] > base_available_height {
                        // 행이 splittable=false이지만 전체 페이지 가용높이보다 큰 경우:
                        // 다음 페이지로 넘겨도 들어가지 않으므로 가용 공간에 맞춰 강제 intra-row split.
                        // Task #398: 단일 행 블록에서만 적용 (rowspan 묶음 보호).
                        let row_cs = cs;
                        let padding = mt.max_padding_for_row(r);
                        let avail_content_for_r = (remaining_avail - row_cs - padding).max(0.0);
                        if avail_content_for_r >= MIN_SPLIT_CONTENT_PX {
                            end_row = r + 1;
                            split_end_limit = avail_content_for_r;
                        }
                    }
                } else {
                    end_row = row_count;
                }
            }

            if end_row <= cursor_row {
                end_row = cursor_row + 1;
            }

            // 이 범위의 높이 계산
            let partial_height: f64 = {
                let delta = if content_offset > 0.0 && can_intra_split {
                    mt.row_heights[cursor_row] - effective_first_row_h
                } else {
                    0.0
                };
                if split_end_limit > 0.0 {
                    let complete_range = if end_row > cursor_row + 1 {
                        mt.range_height(cursor_row, end_row - 1) - delta
                    } else {
                        0.0
                    };
                    let split_row = end_row - 1;
                    let split_row_h = split_end_limit + mt.max_padding_for_row(split_row);
                    let split_row_cs = if split_row > cursor_row { cs } else { 0.0 };
                    complete_range + split_row_cs + split_row_h + header_overhead
                } else {
                    mt.range_height(cursor_row, end_row) - delta + header_overhead
                }
            };

            let actual_split_start = content_offset;
            let actual_split_end = split_end_limit;

            // 마지막 파트에 Bottom 캡션 공간 확보
            if end_row >= row_count && split_end_limit == 0.0 && !caption_is_top && caption_overhead > 0.0 {
                let total_with_caption = partial_height + caption_overhead;
                let avail = if is_continuation {
                    (page_avail - header_overhead).max(0.0)
                } else {
                    page_avail
                };
                if total_with_caption > avail {
                    end_row = end_row.saturating_sub(1);
                    if end_row <= cursor_row {
                        end_row = cursor_row + 1;
                    }
                }
            }

            if end_row >= row_count && split_end_limit == 0.0 {
                // 나머지 전부가 현재 페이지에 들어감
                let bottom_caption_extra = if !caption_is_top { caption_overhead } else { 0.0 };
                if cursor_row == 0 && !is_continuation && content_offset == 0.0 {
                    st.current_items.push(PageItem::Table {
                        para_index: para_idx,
                        control_index: ctrl_idx,
                    });
                    st.current_height += partial_height + host_spacing;
                } else {
                    st.current_items.push(PageItem::PartialTable {
                        para_index: para_idx,
                        control_index: ctrl_idx,
                        start_row: cursor_row,
                        end_row,
                        is_continuation,
                        split_start_content_offset: actual_split_start,
                        split_end_content_limit: 0.0,
                    });
                    // 마지막 부분 표: spacing_after도 포함 (레이아웃과 일치)
                    let mp = measured.get_measured_paragraph(para_idx);
                    let sa = mp.map(|m| m.spacing_after).unwrap_or(0.0);
                    st.current_height += partial_height + bottom_caption_extra + sa;
                }
                break;
            }

            // 부분 표 배치
            st.current_items.push(PageItem::PartialTable {
                para_index: para_idx,
                control_index: ctrl_idx,
                start_row: cursor_row,
                end_row,
                is_continuation,
                split_start_content_offset: actual_split_start,
                split_end_content_limit: actual_split_end,
            });
            st.advance_column_or_new_page();

            // 커서 전진
            if split_end_limit > 0.0 {
                let split_row = end_row - 1;
                if split_row == cursor_row {
                    content_offset += split_end_limit;
                } else {
                    content_offset = split_end_limit;
                }
                cursor_row = split_row;
            } else {
                cursor_row = end_row;
                content_offset = 0.0;
            }
            is_continuation = true;
        }
    }

    /// 페이지 번호 재설정 및 머리말/꼬리말 할당
    fn finalize_pages(
        pages: &mut [PageContent],
        hf_entries: &[(usize, HeaderFooterRef, bool, HeaderFooterApply)],
        page_number_pos: &Option<crate::model::control::PageNumberPos>,
        page_hides: &[(usize, crate::model::control::PageHide)],
        new_page_numbers: &[(usize, u16)],
        _section_index: usize,
    ) {
        // 쪽번호: PageNumberAssigner 가 NewNumber 1회 적용 + 단조 증가를 보장 (Issue #353)
        let mut assigner = crate::renderer::page_number::PageNumberAssigner::new(new_page_numbers, 1);
        // 머리말/꼬리말은 한번 설정되면 이후 페이지에도 유지 (누적)
        let mut header_both: Option<HeaderFooterRef> = None;
        let mut header_even: Option<HeaderFooterRef> = None;
        let mut header_odd: Option<HeaderFooterRef> = None;
        let mut footer_both: Option<HeaderFooterRef> = None;
        let mut footer_even: Option<HeaderFooterRef> = None;
        let mut footer_odd: Option<HeaderFooterRef> = None;
        // 머리말/꼬리말은 정의된 문단이 등장하는 페이지부터 적용
        // (전체 스캔 초기 등록 제거 — 각 페이지의 범위 내 머리말만 누적)
        // 각 페이지의 다음 페이지 첫 문단 인덱스 사전 계산 (borrow 충돌 방지)
        let next_page_first_paras: Vec<usize> = (0..pages.len()).map(|i| {
            pages.get(i + 1)
                .and_then(|p| p.column_contents.first())
                .and_then(|cc| cc.items.first())
                .map(|item| match item {
                    PageItem::FullParagraph { para_index } => *para_index,
                    PageItem::PartialParagraph { para_index, .. } => *para_index,
                    PageItem::Table { para_index, .. } => *para_index,
                    PageItem::PartialTable { para_index, .. } => *para_index,
                    PageItem::Shape { para_index, .. } => *para_index,
                })
                .unwrap_or(usize::MAX)
        }).collect();
        for (i, page) in pages.iter_mut().enumerate() {
            page.page_index = i as u32;

            let page_last_para = page.column_contents.iter()
                .flat_map(|col| col.items.iter())
                .filter_map(|item| match item {
                    PageItem::FullParagraph { para_index } => Some(*para_index),
                    PageItem::PartialParagraph { para_index, .. } => Some(*para_index),
                    PageItem::Table { para_index, .. } => Some(*para_index),
                    PageItem::PartialTable { para_index, .. } => Some(*para_index),
                    PageItem::Shape { para_index, .. } => Some(*para_index),
                })
                .max()
                .unwrap_or(0);

            // 현재 페이지까지의 머리말/꼬리말 업데이트
            // 현재 페이지의 마지막 문단까지만 포함 (다음 페이지 첫 문단의 머리말은 다음 페이지에서 등록)
            for (para_idx, hf_ref, is_header, apply_to) in hf_entries.iter() {
                if *para_idx > page_last_para {
                    break;
                }
                if *is_header {
                    match apply_to {
                        HeaderFooterApply::Both => header_both = Some(hf_ref.clone()),
                        HeaderFooterApply::Even => header_even = Some(hf_ref.clone()),
                        HeaderFooterApply::Odd  => header_odd = Some(hf_ref.clone()),
                    }
                } else {
                    match apply_to {
                        HeaderFooterApply::Both => footer_both = Some(hf_ref.clone()),
                        HeaderFooterApply::Even => footer_even = Some(hf_ref.clone()),
                        HeaderFooterApply::Odd  => footer_odd = Some(hf_ref.clone()),
                    }
                }
            }

            let page_num_u32 = assigner.assign(page);
            page.page_number = page_num_u32;

            let page_num = page_num_u32 as usize;
            let is_odd = page_num % 2 == 1;

            page.active_header = if is_odd {
                header_odd.clone().or_else(|| header_both.clone())
            } else {
                header_even.clone().or_else(|| header_both.clone())
            };

            page.active_footer = if is_odd {
                footer_odd.clone().or_else(|| footer_both.clone())
            } else {
                footer_even.clone().or_else(|| footer_both.clone())
            };

            if !assigner.should_hide_page_number() {
                page.page_number_pos = page_number_pos.clone();
            }
            // PageHide: 해당 문단이 이 페이지에서 **처음** 시작하는 경우만 적용
            // (문단이 여러 페이지에 걸치면 첫 페이지에서만 감추기 적용)
            for (ph_para, ph) in page_hides {
                if Self::para_starts_in_page(page, *ph_para) {
                    page.page_hide = Some(ph.clone());
                    break;
                }
            }

            let _ = page_last_para;
        }
    }

    /// 문단이 해당 페이지에서 **처음 시작**하는지 확인
    /// (PartialParagraph의 start_line==0 또는 FullParagraph만 해당)
    fn para_starts_in_page(page: &PageContent, para_idx: usize) -> bool {
        for col in &page.column_contents {
            for item in &col.items {
                match item {
                    PageItem::FullParagraph { para_index } if *para_index == para_idx => return true,
                    PageItem::PartialParagraph { para_index, start_line, .. } if *para_index == para_idx && *start_line == 0 => return true,
                    PageItem::Table { para_index, .. } if *para_index == para_idx => return true,
                    PageItem::Shape { para_index, .. } if *para_index == para_idx => return true,
                    _ => {}
                }
            }
        }
        false
    }

    /// 문단 인덱스가 해당 페이지에 속하는지 확인
    fn para_in_page(page: &PageContent, para_idx: usize) -> bool {
        for col in &page.column_contents {
            for item in &col.items {
                let pi = match item {
                    PageItem::FullParagraph { para_index } => *para_index,
                    PageItem::PartialParagraph { para_index, .. } => *para_index,
                    PageItem::Table { para_index, .. } => *para_index,
                    PageItem::PartialTable { para_index, .. } => *para_index,
                    PageItem::Shape { para_index, .. } => *para_index,
                };
                if pi == para_idx { return true; }
            }
        }
        false
    }

    /// 표의 세로 오프셋 추출
    fn get_table_vertical_offset(table: &crate::model::table::Table) -> u32 {
        table.common.vertical_offset as u32
    }
}
