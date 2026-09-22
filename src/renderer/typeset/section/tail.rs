//! 구역 문단 처리의 absorb_section_tail 단계. 조건·예약·발행 순서를 유지한다.
use crate::renderer::typeset::{
    hwpunit_to_px, is_para_topbottom_float, para_has_visible_text, paragraph_saved_visible_bounds,
    paragraph_saved_vpos_reset_starts_new_page_after, ColumnBreakType, Control, PageItem,
    Paragraph, ResolvedStyleSet, TypesetEngine, TypesetState,
};
impl TypesetEngine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn absorb_section_tail(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        has_table: bool,
        last_content_para_idx: Option<usize>,
        para_style_break: bool,
        force_page_break: bool,
    ) -> bool {
        // [Task #359] 단독 항목 페이지 차단:
        // 다음 pi 가 vpos-reset 가드를 발동할 예정이고 현재 pi 가 잔여 공간 부족으로
        // 새 페이지를 시작하면 단독 항목 페이지가 발생.
        //   - 현재 pi 가 빈 문단이면: skip (한컴은 표시하지 않음)
        //   - 현재 pi 가 일반 텍스트이면: fit 안전마진 (10px) 1회 비활성화
        //     (kps-ai pi=317 case: 0.x px 차이로 fit 실패하여 단독 페이지 35 발생)
        // 가드 제외 조건:
        //   - 다음 pi 가 force_page_break (column_type==Page/Section) 인 경우 발동 안 함
        //     (정상 쪽나누기 신호 — 단독 페이지 발생 안 함, hwp-multi-001 회귀 차단)
        let next_will_vpos_reset = if (!st.current_items.is_empty() || !st.pages.is_empty())
            && para_idx + 1 < paragraphs.len()
        {
            let next_para = &paragraphs[para_idx + 1];
            let next_force_break = next_para.column_type == ColumnBreakType::Page
                || next_para.column_type == ColumnBreakType::Section;
            if next_force_break {
                false
            } else {
                // [Task #470] 다단 섹션에서는 nv == 0 → nv < cl 로 완화 (컬럼 헤더 오프셋).
                // 단일 단에서는 partial-table split 회귀 (issue #418) 회피 위해 nv == 0 유지.
                paragraph_saved_vpos_reset_starts_new_page_after(
                    para,
                    next_para,
                    st.col_count,
                    st.profile.hwp3_layout(),
                )
            }
        } else {
            false
        };

        // [Task #1725 v2] 현재 일반텍스트 tail 과 새 페이지(vpos-reset) 사이에 빈 문단이 1개
        // 끼어 immediate-next reset 을 놓치는 각주-tail 케이스(국제고속선기준 pi=537/2378).
        // 이 경우도 tail 로 보고 fit 완화 플래그(각주 버퍼/안전마진)만 켠다. 빈 문단 skip
        // 로직(아래 next_will_vpos_reset 분기)에는 영향 없음.
        let tail_before_break_through_empty = !next_will_vpos_reset
            && !st.current_items.is_empty()
            && para_has_visible_text(para)
            && para.controls.is_empty()
            && para_idx + 2 < paragraphs.len()
            && {
                let mid = &paragraphs[para_idx + 1];
                let after = &paragraphs[para_idx + 2];
                mid.text.trim().is_empty()
                    && mid.controls.is_empty()
                    && after.column_type != ColumnBreakType::Page
                    && after.column_type != ColumnBreakType::Section
                    && paragraph_saved_vpos_reset_starts_new_page_after(
                        mid,
                        after,
                        st.col_count,
                        st.profile.hwp3_layout(),
                    )
            };
        if tail_before_break_through_empty {
            st.allow_tail_safety_margin_once();
            st.allow_tail_footnote_margin_once();
            st.reserve_saved_tail_bounds(paragraph_saved_visible_bounds(para, 0, self.dpi));
        }

        // [Task #1733] 페이지 하단 빈 줄이 다음 vpos-reset 흐름 앞에 1개 이상 끼는 경우.
        // 기존 가드는 "현재 빈 문단 바로 다음이 reset" 인 경우만 흡수한다. 국제고속선기준은
        // 빈 줄 2개 뒤 본문이 새 쪽 상단으로 reset 되거나, 빈 줄 뒤 하단 제목 1줄이 있고
        // 그 다음 본문이 reset 되는 형태가 있어 near-empty 페이지가 남는다. 현재 빈 문단이
        // 이미 페이지 하단 vpos 를 가지고 있고, 뒤쪽 저장 flow 가 reset 을 명확히 보일 때만
        // 0-높이로 흡수한다.
        let empty_tail_bridge_to_reset = !next_will_vpos_reset
            && !st.current_items.is_empty()
            && !st
                .current_items
                .iter()
                .any(|item| matches!(item, PageItem::PartialTable { .. }))
            && para.text.is_empty()
            && para.controls.is_empty()
            && para.line_segs.first().is_some_and(|seg| {
                let body_h_hu =
                    crate::renderer::px_to_hwpunit(st.layout.body_area.height, self.dpi);
                seg.vertical_pos > body_h_hu * 70 / 100
            })
            && {
                let mut idx = para_idx + 1;
                let mut prev = para;
                let mut found = false;
                while let Some(next_para) = paragraphs.get(idx) {
                    if next_para.text.is_empty() && next_para.controls.is_empty() {
                        prev = next_para;
                        idx += 1;
                        continue;
                    }

                    let reset_after_empty_run = paragraph_saved_vpos_reset_starts_new_page_after(
                        prev,
                        next_para,
                        st.col_count,
                        st.profile.hwp3_layout(),
                    );
                    let high_tail_heading_then_reset = para_has_visible_text(next_para)
                        && next_para.controls.is_empty()
                        && next_para.line_segs.first().is_some_and(|seg| {
                            let body_h_hu = crate::renderer::px_to_hwpunit(
                                st.layout.body_area.height,
                                self.dpi,
                            );
                            seg.vertical_pos > body_h_hu * 70 / 100
                        })
                        && paragraphs.get(idx + 1).is_some_and(|after| {
                            after.column_type != ColumnBreakType::Page
                                && after.column_type != ColumnBreakType::Section
                                && paragraph_saved_vpos_reset_starts_new_page_after(
                                    next_para,
                                    after,
                                    st.col_count,
                                    st.profile.hwp3_layout(),
                                )
                        });

                    found = reset_after_empty_run || high_tail_heading_then_reset;
                    break;
                }
                found
            };
        if empty_tail_bridge_to_reset {
            st.hide_empty_paragraph(para_idx);
            st.append_item(PageItem::FullParagraph {
                para_index: para_idx,
            });
            return true;
        }

        if next_will_vpos_reset {
            // [Task #362] 빈 paragraph 가 표/도형/그림 컨트롤을 포함하면 skip 안 함
            // (kps-ai pi=778 case: 빈 텍스트 + 3x3 wrap=Square 표를 가진 paragraph 가
            //  잘못 skip 되어 표 누락).
            let is_empty_no_ctrl = para.text.is_empty() && para.controls.is_empty();
            if is_empty_no_ctrl {
                // 페이지 하단 vpos 를 가진 빈 문단이 다음 vpos=0 본문을 잇는 경우.
                // fit 가능으로 emit 하더라도 뒤쪽 overflow 방어에서 빈 문단 단독 페이지로
                // 이동할 수 있으므로, fit 판정 전에 0-높이로 흡수한다.
                let page_bottom_empty_reset_bridge = para.line_segs.first().is_some_and(|seg| {
                    let body_h_hu =
                        crate::renderer::px_to_hwpunit(st.layout.body_area.height, self.dpi);
                    seg.vertical_pos > body_h_hu * 70 / 100
                });
                if page_bottom_empty_reset_bridge {
                    st.hide_empty_paragraph(para_idx);
                    if !st.current_items.is_empty() {
                        st.append_item(PageItem::FullParagraph {
                            para_index: para_idx,
                        });
                    }
                    return true;
                }

                // [#1648] 빈 문단이 현재 페이지에 들어가면 정상 배치한다(한글 동작 —
                //   페이지 하단에 빈 줄 1개). 들어가지 않을 때만 skip 하여 단독 빈 페이지를
                //   차단한다. 종전엔 fit 무검사로 fit 하는 빈 문단까지 드롭하여, 페이지를
                //   채운 TAC 표 직후의 빈 문단이 누락되고 페이지↔PI 가 한글과 어긋났다
                //   (#1643 rhwp_pNone).
                //
                // 1) height fit: 합산 current_height 기준 (종전 #1648 판정).
                let empty_h_px = para
                    .line_segs
                    .first()
                    .map(|s| {
                        hwpunit_to_px(
                            (s.line_height.saturating_add(s.line_spacing)) as i32,
                            self.dpi,
                        )
                    })
                    .unwrap_or(0.0);
                let height_fits = empty_h_px <= st.available_height() - st.current_height;

                // 2) [#1659] vpos fit: 합산 height 는 음수 줄간격 문단에서 실제 vpos 진행을
                //   과소평가 → 페이지 하단 빈 문단을 height fit 으로 오판 emit 하지만 placement
                //   (아래 vpos overflow 가드, ~L2300)는 vpos overflow 로 새 페이지에 단독 배치
                //   → 단독 빈 페이지 +1 회귀(synam-001 35→36). placement(L2333/L2339)와 동일한
                //   page_top_vpos 기준 vpos 판정을 AND 로 더해, height·vpos 둘 다 fit 일 때만
                //   emit. placement 가 height 기반인 다단/wrap 에선 vpos 판정을 생략(true).
                let vpos_fits = if st.col_count == 1 && st.wrap_around_cs < 0 {
                    // 페이지 첫 실 item 의 top vpos. PartialParagraph continuation 은 원
                    //   문단의 첫 줄이 아니라 fragment 시작 줄(start_line)의 vpos 가 페이지
                    //   상단이다 → line_segs[start_line] 사용. 줄 기준 vpos 가 없는 항목
                    //   (PartialTable continuation)은 None 으로 두어 vpos 판정을 보류(height
                    //   fit 에 위임) — 잘못된 baseline 으로 skip/emit 오판 방지(#1659 리뷰).
                    let page_top_vpos =
                        st.current_items
                            .iter()
                            .find(|item| !matches!(item, PageItem::EndnoteSeparator { .. }))
                            .and_then(|item| match item {
                                PageItem::FullParagraph { para_index }
                                | PageItem::Table { para_index, .. }
                                | PageItem::Shape { para_index, .. } => paragraphs
                                    .get(*para_index)
                                    .and_then(|p| p.line_segs.first())
                                    .map(|s| s.vertical_pos),
                                PageItem::PartialParagraph {
                                    para_index,
                                    start_line,
                                    ..
                                } => paragraphs
                                    .get(*para_index)
                                    .and_then(|p| p.line_segs.get(*start_line))
                                    .map(|s| s.vertical_pos),
                                // 줄 기준 vpos 없음 → 판정 보류.
                                PageItem::PartialTable { .. }
                                | PageItem::EndnoteSeparator { .. } => None,
                            });
                    match (para.line_segs.last(), page_top_vpos) {
                        (Some(last_seg), Some(top)) => {
                            let body_h_hu = crate::renderer::px_to_hwpunit(
                                st.layout.body_area.height,
                                self.dpi,
                            );
                            let vpos_end =
                                last_seg.vertical_pos.saturating_add(last_seg.line_height);
                            vpos_end <= top + body_h_hu + 283
                        }
                        // vpos 판정 불가 → 제약 없음(height fit 에 위임).
                        _ => true,
                    }
                } else {
                    true
                };

                if !(height_fits && vpos_fits) {
                    // [#1706] 빈 문단이 현재 페이지에 안 들어감.
                    // 종전엔 통째로 drop(continue) → 문단이 모델에서 사라져 한글 대비
                    // 문단→페이지 매핑이 어긋났다(rhwp_pNone; 대형 TAC 표가 페이지를 채운
                    // 직후의 빈 문단). 한글은 이 빈 문단을 현재 페이지 하단의 빈 줄 1개로 유지.
                    // → drop 대신 현재 페이지에 0-높이로 흡수 기록(hide_empty_line 와 동일
                    //   시멘틱). 페이지를 advance 하지 않으므로 단독 빈 페이지 회귀(synam-001
                    //   등)는 발생하지 않는다.
                    st.hide_empty_paragraph(para_idx);
                    st.append_item(PageItem::FullParagraph {
                        para_index: para_idx,
                    });
                    return true;
                }
                // height·vpos 둘 다 fit → 정상 emit (아래로 진행)
            } else {
                // 일반 텍스트 또는 컨트롤 보유: 안전마진 1회 비활성화 (단독 텍스트 페이지 차단)
                st.allow_tail_safety_margin_once();
                // [Task #1725] 각주 있는 페이지의 tail 문단: 각주 안전마진(40px 버퍼)도 1회
                // 비활성화. 한글은 tail 을 본문에 배치하는데 rhwp 각주 예약 버퍼가 tail 을 수 px
                // 밀어 near-empty 페이지 over-pagination(국제고속선기준 258 vs 242) 을 만든다.
                st.allow_tail_footnote_margin_once();
                // [Task #1725 v2] 각주 없이 페이지가 수 px over-fill 되어 tail 이 밀리는 경우도
                // 한글은 저장 tail의 본문 하단 좌표를 유지한다. 다음 fit에서 실제
                // 저장 bottom까지의 차이만 허용하도록 bounds를 전달한다.
                st.reserve_saved_tail_bounds(paragraph_saved_visible_bounds(para, 0, self.dpi));
            }
        } else if !st.current_items.is_empty() && para_idx + 1 < paragraphs.len() {
            // [Task #967] 빈 paragraph 직후 force page break (쪽나누기) case 가드:
            // 빈 paragraph 가 현재 page 잔여 공간 초과 시 별도 page 분기 →
            // +1 page inflate 회귀 (sample18.hwp 의 pi=27, pi=164).
            // 한컴은 빈 paragraph 를 trailing overflow 로 흡수 + 쪽나누기로 새 page 시작.
            // next_will_vpos_reset 가드는 next_force_break 인 경우 발동 안 함
            // (hwp-multi-001 회귀 차단). 본 추가 가드는 빈 paragraph + 다음 쪽나누기
            // case 중에서 **현재 page 잔여 공간 부족 (overflow) 시에만** skip — 빈
            // paragraph 가 page 에 fit 하면 정상 emit (aift.hwp 의 18 case 회귀 방지).
            let next_para = &paragraphs[para_idx + 1];
            let next_force_break = next_para.column_type == ColumnBreakType::Page
                || next_para.column_type == ColumnBreakType::Section;
            let is_curr_empty = para.text.is_empty() && para.controls.is_empty();
            if next_force_break && is_curr_empty {
                // empty paragraph 의 예상 height = first line_seg 의 lh + ls
                let empty_h_px = para
                    .line_segs
                    .first()
                    .map(|s| {
                        hwpunit_to_px(
                            (s.line_height.saturating_add(s.line_spacing)) as i32,
                            self.dpi,
                        )
                    })
                    .unwrap_or(0.0);
                let avail = st.available_height() - st.current_height;
                if empty_h_px > avail {
                    // [#1706] 빈 paragraph 가 fit 안 됨.
                    // 종전엔 drop(continue) → 문단이 모델에서 사라져 한글 대비 매핑이
                    // 어긋났다(rhwp_pNone). 한컴은 이 빈 문단을 현재 page 하단의 빈 줄로
                    // 흡수(위 주석)하므로, drop 대신 현재 페이지에 0-높이로 흡수 기록한다.
                    // 페이지를 advance 하지 않으므로 단독 page 회귀(sample18 등)는 없다.
                    st.hide_empty_paragraph(para_idx);
                    st.append_item(PageItem::FullParagraph {
                        para_index: para_idx,
                    });
                    return true;
                }
                // fit 가능 — 정상 emit (기존 동작)
            }
        }

        // co-anchored 자리차지 표가 페이지를 가득 채운 뒤(위 orphan 가드로 통째 이월된
        // 표 등) 오는, 문서 말미의 trailing 빈 문단(텍스트·컨트롤 없음, 뒤에 가시 콘텐츠
        // 없음)이 현재 page 잔여 공간을 초과하면 새 page 를 만들지 않고 skip 한다. 한컴은
        // page 를 채운 자리차지 표 뒤의 빈 문단을 trailing overflow 로 흡수하고 빈 page 를
        // 추가하지 않는다(검증점검표: 결재+점검표 표 뒤 빈 문단 2개로 빈 3쪽이 생기던 회귀).
        // 단독 anchored 표·일반 문단 흐름의 trailing 빈 줄은 정상 페이지네이션을 유지해야
        // 하므로, page 마지막 항목이 co-anchored 자리차지 표일 때로 한정한다(orphan 가드와
        // 동일 신호). 명시적 쪽나누기가 걸린 빈 문단은 사용자 의도이므로 제외한다.
        let last_item_is_coanchored_float_table = match st.current_items.last() {
            Some(PageItem::Table {
                para_index: tpi,
                control_index: tci,
            })
            | Some(PageItem::PartialTable {
                para_index: tpi,
                control_index: tci,
                ..
            }) => paragraphs.get(*tpi).is_some_and(|hp| {
                let this_is_float = hp.controls.get(*tci).is_some_and(
                    |c| matches!(c, Control::Table(t) if is_para_topbottom_float(&t.common)),
                );
                let has_preceding_float =
                    hp.controls.iter().take(*tci).any(
                        |c| matches!(c, Control::Table(t) if is_para_topbottom_float(&t.common)),
                    );
                this_is_float && has_preceding_float
            }),
            _ => false,
        };
        let is_trailing_empty = para.text.is_empty()
            && para.controls.is_empty()
            && last_content_para_idx.is_some_and(|lc| para_idx > lc)
            && last_item_is_coanchored_float_table
            && !force_page_break
            && !para_style_break;
        if is_trailing_empty {
            let empty_h_px = para
                .line_segs
                .first()
                .map(|s| {
                    hwpunit_to_px(
                        (s.line_height.saturating_add(s.line_spacing)) as i32,
                        self.dpi,
                    )
                })
                .unwrap_or(0.0);
            let avail = st.available_height() - st.current_height;
            if empty_h_px > avail {
                return true;
            }
        }

        // [#1955] 글뒤로/글앞으로 표 직후의 빈 후행 문단 흡수.
        // 이 wrap 은 본문 플로우를 소비하지 않아야 하나(한글 시멘틱) 현재
        // 페이지네이션은 fragment 로 플로우를 소비하므로, 최소한 빈 후행 문단은
        // anchor 첫 fragment 단에 소급 기록하여 pi-page 정합을 복원한다.
        // (조례 [별표] 서식: 표 끝 쪽으로 밀리던 후행 빈 문단 — pi 9쪽 이탈)
        if let Some(anchor_pi) = st.behind_float_table_para {
            if !has_table {
                let is_empty_para = para
                    .text
                    .chars()
                    .all(|ch| ch.is_whitespace() || ch == '\r' || ch == '\n')
                    && para.controls.is_empty();
                if is_empty_para {
                    // 표 fragment 가 지연 flush 라 지금은 anchor 단을 특정할 수
                    // 없다 — 보류 목록에 넣고 페이지 확정 후 일괄 부착.
                    st.defer_behind_wrap_absorption(crate::renderer::pagination::WrapAroundPara {
                        para_index: para_idx,
                        table_para_index: anchor_pi,
                        has_text: false,
                        start_line: 0,
                        end_line: usize::MAX,
                    });
                    return true;
                }
                st.finish_behind_float_absorption();
            } else {
                st.finish_behind_float_absorption();
            }
        }

        false
    }
}
