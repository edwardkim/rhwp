//! 구역 문단 처리의 place_paragraph_controls 단계. 조건·예약·발행 순서를 유지한다.
use crate::renderer::typeset::{
    body_pile_stays_on_anchor_page, estimate_footnote_note_height, flow_noninline_picture,
    has_majority_fullpage_images, hwpunit_to_px, notes, Control, DeferredSquarePictureControl,
    EndnoteRef, FootnoteRef, FootnoteSource, PageItem, Paragraph, ResolvedStyleSet, TypesetEngine,
    TypesetState,
};
impl TypesetEngine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn place_paragraph_controls(
        &self,
        st: &mut TypesetState,
        para_idx: usize,
        para: &Paragraph,
        paragraphs: &[Paragraph],
        styles: &ResolvedStyleSet,
        section_index: usize,
        has_table: bool,
        picture_host_origin: (usize, u16, f64),
        native_hwp5_footnote_break: Option<notes::footnotes::boundary::NativeHwp5FootnoteBreak>,
    ) {
        // [#1995] 한 문단에 근접-전면(near-full-page) non-TAC 그림이 여러 장이면
        // (임베드 매뉴얼을 페이지 이미지로 삽입한 경우 등) 각 이미지는 공존 불가하므로
        // 각각 한 페이지에 단독 배치해야 한다. 미수정 시 96장이 한 앵커에 스택되어
        // 문서가 과소 페이지가 된다(오라클 268 vs rhwp 174). 한 문단에 본문높이의
        // 60% 이상인 non-TAC 그림이 2장 이상일 때만 발동(정상 단일 그림 문단 불변).
        // 글뒤로/글앞으로 그림은 후보·분모 모두에서 제외한다(#6511, `flow_noninline_picture`).
        let fullpage_img_body_h = st.base_available_height();
        let fullpage_img_ctrls: Vec<usize> = if fullpage_img_body_h > 0.0 && !has_table {
            para.controls
                .iter()
                .enumerate()
                .filter_map(|(ci, c)| {
                    let pic = flow_noninline_picture(c)?;
                    let h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                    (h >= fullpage_img_body_h * 0.6).then_some(ci)
                })
                .collect()
        } else {
            Vec::new()
        };
        // [#4654] 낱장 배치는 **전면 크기가 과반**인 문단에만 — 디자인
        // 보드형 pile(체육대회 4510000-202300010: 한 문단 그림 210장 중
        // 전면 ~15장, 한글은 쪽당 60여 장 통 적재에 전면 그림도 포함)에서
        // 소수 전면 그림이 낱장으로 탈출해 +12쪽이 됐다. #1995 원 취지
        // (임베드 매뉴얼: 전량 전면 96장, 오라클 268 vs 174 과소)는 과반
        // 조건으로 그대로 보존된다.
        let noninline_pic_count = para
            .controls
            .iter()
            .filter(|c| flow_noninline_picture(c).is_some())
            .count();
        // [#4770] #2004 본문 정규화와 같은 엄격한 저장 스택 계약을 쓴다. 즉 빈
        // 문단의 비-TAC Square·겹침불허 그림/그림-도형이 같은 세로 band에 있고,
        // 저장 첫 줄이 그림 폭 이상 오른쪽에서 시작하며, 그림 하단이 본문 하단 절대
        // 좌표 안에 있을 때만 #1995 낱장 배치를 억제한다. 일반 TopAndBottom·서로
        // 다른 anchor·본문 텍스트 문단은 기존 분산을 유지한다.
        let fullpage_img_min_height_hu =
            (crate::renderer::px_to_hwpunit(st.layout.body_area.height, self.dpi) / 2).max(1);
        let fullpage_img_body_bottom_hu = crate::renderer::px_to_hwpunit(
            st.layout.body_area.y + st.layout.body_area.height,
            self.dpi,
        );
        let stored_line_beside_pile = body_pile_stays_on_anchor_page(
            para,
            fullpage_img_min_height_hu,
            fullpage_img_body_bottom_hu,
        );
        let is_multi_fullpage_img_para = !stored_line_beside_pile
            && has_majority_fullpage_images(fullpage_img_ctrls.len(), noninline_pic_count);

        // [#2097] 이 문단의 TopAndBottom 자리차지 float pushdown 가로 컬럼
        // (h_left, h_right, 스택_높이) px — 가로 겹침으로 스택/나란히 판별.
        let mut topbottom_cols: Vec<(f64, f64, f64)> = Vec::new();
        // [#2814] 이 문단의 pushdown 대상(비-TAC TopAndBottom vert=Para) 그림/도형 수.
        // 3장 이상이 세로로 스택되면 쪽 용량 기반 분배 대상(아래 overflow 이월).
        let pushdown_topbottom_ctrl_count = para
            .controls
            .iter()
            .filter(|c| match c {
                Control::Picture(pic) => {
                    !pic.common.treat_as_char
                        && matches!(
                            pic.common.text_wrap,
                            crate::model::shape::TextWrap::TopAndBottom
                        )
                        && matches!(pic.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
                }
                Control::Shape(s) => {
                    !s.common().treat_as_char
                        && matches!(
                            s.common().text_wrap,
                            crate::model::shape::TextWrap::TopAndBottom
                        )
                        && matches!(s.common().vert_rel_to, crate::model::shape::VertRelTo::Para)
                }
                _ => false,
            })
            .count();

        // 인라인 컨트롤 처리: 도형/그림/수식/각주 (Paginator engine.rs:509-525 동일)
        for (ctrl_idx, ctrl) in para.controls.iter().enumerate() {
            // [#1995] 다수 전면 이미지: 각 전면 그림을 새 페이지에 단독 배치.
            if is_multi_fullpage_img_para && fullpage_img_ctrls.contains(&ctrl_idx) {
                st.force_new_page();
                st.append_item(PageItem::Shape {
                    para_index: para_idx,
                    control_index: ctrl_idx,
                });
                // 페이지를 채워 후속 그림/문단이 다음 페이지로 밀리도록.
                st.align_flow_to(fullpage_img_body_h);
                continue;
            }
            match ctrl {
                // [#6266] 비-TAC 양식 개체는 자기 배치(기준·정렬·오프셋)를 갖는
                // 개체다. 종전에는 IR 에 배치가 없어 무조건 인라인으로 흘렀고,
                // 쪽 하단 가운데 서식 번호가 제목 줄 안에 그려졌다.
                Control::Form(form) if !form.common.treat_as_char => {
                    st.append_item(PageItem::Shape {
                        para_index: para_idx,
                        control_index: ctrl_idx,
                    });
                }
                Control::Shape(_) | Control::Picture(_) | Control::Equation(_) => {
                    // [#6146] 저장 리셋 경계에서 떠나는 쪽의 흐름 말미에 이미 흘려
                    // 놓은 자리차지 밴드는 다시 배치하지 않는다.
                    if st.page_tail_spilled_floats.contains(&(para_idx, ctrl_idx)) {
                        continue;
                    }
                    if !has_table {
                        // [#3738 Stage 22] page-tail Square picture는 anchor 본문을
                        // 현재 쪽에 남기되 그림만 다음 physical page의 narrow wrap
                        // band에 배치한다. p155 그림 64처럼 현재 PageItem에 넣으면
                        // caption이 기존 FootnoteArea와 겹친다.
                        if let Some((wrap_target_para_indices, wrap_anchor)) = self
                            .native_hwp5_square_picture_next_page_owner(
                                &st, para_idx, para, paragraphs, ctrl, styles,
                            )
                        {
                            st.defer_square_picture(DeferredSquarePictureControl {
                                para_index: para_idx,
                                control_index: ctrl_idx,
                                wrap_target_para_indices,
                                wrap_anchor,
                            });
                            continue;
                        }
                        // [Issue #476/#4092] treat_as_char 그림/도형은 박스가 속한 line 이 라우팅된
                        // 페이지/단에 등록. paragraph 가 페이지 분할되면 이 시점의
                        // st.current_items 는 마지막 페이지 상태이므로, 그대로 push 하면
                        // 박스가 잘못된 페이지에 떠 있게 된다.
                        let routed = if crate::renderer::pagination::is_routable_treat_as_char_picture_or_shape(ctrl) {
                                crate::renderer::pagination::find_inline_control_target_page(
                                    &st.pages,
                                    &st.current_items,
                                    para_idx,
                                    ctrl_idx,
                                    para,
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
                                st.append_routed_item(page_idx, col_idx, item);
                            }
                            None => {
                                st.append_item(item);
                            }
                        }
                        // [Task #1052] 글상자 내 각주 수집 (engine.rs:1376-1398 동등)
                        // NO_LS 호스트의 측정 원점만 전달한다. 저장 vpos 소유자는
                        // 기존 저장 배치 경로에 남긴다.
                        let host_top = (para.line_segs.is_empty()
                            && (st.pages.len(), st.current_column)
                                == (picture_host_origin.0, picture_host_origin.1))
                            .then_some(picture_host_origin.2);
                        st.register_side_wrap_picture(para_idx, ctrl_idx, para, host_top, styles);
                        if self.profile.get().hwp5_stored_pagination_layout()
                                && !self.profile.get().session_edited()
                                && st.current_items.iter().any(|item| {
                                    matches!(item, PageItem::FullParagraph { para_index } if *para_index == para_idx)
                                })
                            {
                                let saved = paragraphs.get(para_idx + 1).and_then(|next| {
                                    // An earlier picture with measured flow leaves
                                    // the column's painted origin outside this
                                    // stored reservation contract. Do not resume
                                    // absolute saved coordinates midway through
                                    // that chain and paint over preceding text.
                                    let unresolved_picture_flow = st.current_items.iter().any(|item| {
                                        let PageItem::Shape { para_index: owner, control_index } = item else { return false; };
                                        if *owner == para_idx { return false; }
                                        let Some(Control::Picture(picture)) = paragraphs.get(*owner).and_then(|p| p.controls.get(*control_index)) else { return false; };
                                        !picture.common.treat_as_char
                                            && picture.common.text_wrap == crate::model::shape::TextWrap::TopAndBottom
                                            && !st.paragraph_float_placements.get(&(*owner, *control_index)).is_some_and(|placement| matches!(placement.flow, crate::renderer::float_placement::ParagraphFloatFlow::StoredPicture { .. }))
                                    });
                                    if unresolved_picture_flow { return None; }
                                    let host_style = styles.para_styles.get(para.para_shape_id as usize)?;
                                    let next_style = styles.para_styles.get(next.para_shape_id as usize)?;
                                    // The first stored line may retain its paragraph's
                                    // spacing-before at column top. That is an inset,
                                    // not the origin of the source coordinate system.
                                    let first_para = st.current_items.iter().find_map(|item| {
                                        match item {
                                            PageItem::FullParagraph { para_index } => paragraphs.get(*para_index),
                                            _ => None,
                                        }
                                    })?;
                                    let first_before = styles.para_styles.get(first_para.para_shape_id as usize)?.spacing_before;
                                    let base = st.vpos_page_base.unwrap_or(0);
                                    let retained_before = first_before.max(0.0).min(hwpunit_to_px(base.max(0), self.dpi));
                                    let frame_vpos = base - crate::renderer::px_to_hwpunit(retained_before, self.dpi);
                                    crate::renderer::float_placement::stored_picture_successor_placement(
                                        para, next, host_style.spacing_before,
                                        next_style.spacing_before, frame_vpos, self.dpi,
                                    )
                                });
                                if let Some(placement) = saved.filter(|p| {
                                    p.occupied_bottom <= st.available_height()
                                        && p.anchor_y <= st.current_height
                                }) {
                                    st.record_paragraph_float_placement((para_idx, ctrl_idx), placement);
                                    st.align_flow_to(placement.paragraph_end(st.current_height, 0.0));
                                    continue;
                                }
                            }
                        // footnote-tbox-01.hwpx 의 글상자 안 각주 본문이 페이지 하단 영역
                        // 에 누락되는 결함 정정. engine.rs (legacy) 는 이미 처리하나
                        // typeset.rs (main, default) 만 누락 — feedback_image_renderer_paths_separate.
                        if let Control::Shape(shape_obj) = ctrl {
                            if let Some(text_box) =
                                shape_obj.drawing().and_then(|d| d.text_box.as_ref())
                            {
                                for (tp_idx, tp) in text_box.paragraphs.iter().enumerate() {
                                    for (tc_idx, tc) in tp.controls.iter().enumerate() {
                                        if let Control::Footnote(fn_ctrl) = tc {
                                            if !st.pages.is_empty() {
                                                st.record_current_footnote(FootnoteRef {
                                                    number: fn_ctrl.number,
                                                    source: FootnoteSource::ShapeTextBox {
                                                        para_index: para_idx,
                                                        shape_control_index: ctrl_idx,
                                                        tb_para_index: tp_idx,
                                                        tb_control_index: tc_idx,
                                                    },
                                                    fragment: None,
                                                });
                                                let fn_height = estimate_footnote_note_height(
                                                    fn_ctrl, self.dpi,
                                                );
                                                st.add_footnote_height(fn_height);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        // Task #409 v2: 비-TAC TopAndBottom + vert=Para Picture/Shape 는
                        // layout 에서 picture_footnote.rs:356 의 `y_offset + total_height`
                        // 패턴으로 후속 콘텐츠를 개체 높이만큼 밀어냄. 하지만 paragraph
                        // line_seg 의 lh 는 텍스트 baseline 만 반영하므로 페이지네이션의
                        // current_height 가 개체 높이만큼 부족하게 누적되어 page packing
                        // 시 layout 실제 y 와 어긋남 (21페이지: pagination used=803px vs
                        // layout y=1275px → pi=192 가 21페이지에 packing 되었다가
                        // overflow 로 잘림). pagination 측에서도 layout 과 동일하게
                        // 개체 높이를 current_height 에 누적.
                        use crate::model::shape::{TextWrap, VertRelTo};
                        // (obj_h, extra=obj_h+margin_bottom, h_left, h_right px)
                        let pushdown_h: Option<(f64, f64, f64, f64)> = match ctrl {
                            Control::Picture(pic)
                                if !pic.common.treat_as_char
                                    && matches!(pic.common.text_wrap, TextWrap::TopAndBottom)
                                    && matches!(pic.common.vert_rel_to, VertRelTo::Para) =>
                            {
                                let h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                let mb = hwpunit_to_px(pic.common.margin.bottom as i32, self.dpi);
                                let hl =
                                    hwpunit_to_px(pic.common.horizontal_offset as i32, self.dpi);
                                let hr = hl + hwpunit_to_px(pic.common.width as i32, self.dpi);
                                Some((h, h + mb, hl, hr))
                            }
                            Control::Shape(s)
                                if !s.common().treat_as_char
                                    && matches!(s.common().text_wrap, TextWrap::TopAndBottom)
                                    && matches!(s.common().vert_rel_to, VertRelTo::Para) =>
                            {
                                let cm = s.common();
                                let h = hwpunit_to_px(cm.height as i32, self.dpi);
                                let mb = hwpunit_to_px(cm.margin.bottom as i32, self.dpi);
                                let hl = hwpunit_to_px(cm.horizontal_offset as i32, self.dpi);
                                let hr = hl + hwpunit_to_px(cm.width as i32, self.dpi);
                                Some((h, h + mb, hl, hr))
                            }
                            _ => None,
                        };
                        if let Some((obj_h, extra, h_left, h_right)) = pushdown_h {
                            // [Task #1079] 파일 vpos 가 이미 그림 공간을 반영(그림 para 줄
                            // 앞 gap ≥ 그림 높이)하면 VPOS_CORR sync 가 그 공간을 따르므로
                            // pushdown 가산은 이중 계상. gap 이 그림 높이 미만(파일 vpos
                            // 미반영, Task #409 계열)일 때만 가산.
                            const PUSHDOWN_GAP_TOL_PX: f64 = 8.0;
                            let already_accounted = para_idx > 0 && {
                                let v_cur = para.line_segs.first().map(|s| s.vertical_pos);
                                let prev_end = paragraphs[para_idx - 1]
                                    .line_segs
                                    .last()
                                    .map(|s| s.vertical_pos.saturating_add(s.line_height));
                                match (v_cur, prev_end) {
                                    (Some(vc), Some(pe)) if vc > pe => {
                                        hwpunit_to_px((vc - pe) as i32, self.dpi)
                                            >= obj_h - PUSHDOWN_GAP_TOL_PX
                                    }
                                    _ => false,
                                }
                            };
                            // [#6888] 자기 앵커보다 아래로 떨어진 개체는 뒤따르는
                            // 문단을 밀지 않는다 — 판별은 공용 헬퍼에 둔다(배치와
                            // 같은 답을 써야 `#409` 가 막으려던 desync 가 안 생긴다).
                            let displaced_below_following_flow = match ctrl {
                                Control::Picture(pic) => {
                                    crate::renderer::topbottom_float_displaced_below_following_flow(
                                        para,
                                        paragraphs.get(para_idx + 1),
                                        &pic.common,
                                        self.dpi,
                                    )
                                }
                                Control::Shape(s) => {
                                    crate::renderer::topbottom_float_displaced_below_following_flow(
                                        para,
                                        paragraphs.get(para_idx + 1),
                                        s.common(),
                                        self.dpi,
                                    )
                                }
                                _ => false,
                            };
                            if !already_accounted && !displaced_below_following_flow {
                                // [#2814] 절반쪽급 그림이 한 문단에 여럿 스택되면 한컴은
                                // 흐름처럼 쪽을 채우며 다음 쪽으로 넘긴다(창조경제 보고서:
                                // 절반쪽 그림 37장 = 쪽당 2장 × ~19쪽; #1995 의 전면 그림
                                // 1장/쪽과 별개 축). 이 그림을 더하면 스택 하단이 본문을
                                // 넘고 현재 쪽에 이미 다른 항목이 있으면, 방금 push 한 이
                                // Shape 항목을 새 쪽으로 이월하고 스택을 리셋한다.
                                // 발동은 3장 이상으로 한정 — 2장 스택은 한컴이 razor-full
                                // 페이지에 압축 유지하는 실측 반례(1051000-201800093 p60,
                                // 158쪽 정답 유지)가 있어 제외한다.
                                if pushdown_topbottom_ctrl_count >= 3 {
                                    let ovl_base = topbottom_cols
                                        .iter()
                                        .filter(|c| c.1 > h_left && c.0 < h_right)
                                        .map(|c| c.2)
                                        .fold(0.0_f64, f64::max);
                                    let before_max =
                                        topbottom_cols.iter().map(|c| c.2).fold(0.0_f64, f64::max);
                                    let delta = (ovl_base + extra - before_max).max(0.0);
                                    let self_is_last = matches!(
                                        st.current_items.last(),
                                        Some(PageItem::Shape { para_index: p, control_index: c })
                                            if *p == para_idx && *c == ctrl_idx
                                    );
                                    if delta > 0.0
                                        && st.current_height + delta > st.available_height() + 0.5
                                        && self_is_last
                                        && st.current_items.len() > 1
                                    {
                                        st.remove_last_item();
                                        st.advance_column_or_new_page();
                                        st.append_item(PageItem::Shape {
                                            para_index: para_idx,
                                            control_index: ctrl_idx,
                                        });
                                        topbottom_cols.clear();
                                    }
                                }
                                // [#2097] 같은 문단의 TopAndBottom float pushdown 을 가로
                                // 컬럼 모델로 예약한다. 판별 축은 세로 offset 이 아니라 가로
                                // 겹침 — 가로로 겹치는 float 은 세로로 스택되어 합산
                                // (1342000 취업정책연구: 큰 그림 4장이 같은 단 off≈0 에 스택,
                                // 세로 offset 은 작아 offset-union 은 오병합), 가로로 분리된
                                // float 은 나란히라 같은 세로 band 를 공유해 max 예약
                                // (17809123 자원봉사증: 좌우 그림 2장 h-range 분리). 예약
                                // 총량 = max(컬럼별 스택 높이). 새 float 이 겹치는 컬럼(들)에
                                // 스택되면 그 컬럼 높이에 extra 가산, 안 겹치면 새 컬럼. 단일
                                // float 은 컬럼 1개=extra 라 동작 불변.
                                let overlapping: Vec<usize> = topbottom_cols
                                    .iter()
                                    .enumerate()
                                    .filter(|(_, c)| c.1 > h_left && c.0 < h_right)
                                    .map(|(i, _)| i)
                                    .collect();
                                let applied_before =
                                    topbottom_cols.iter().map(|c| c.2).fold(0.0_f64, f64::max);
                                if overlapping.is_empty() {
                                    topbottom_cols.push((h_left, h_right, extra));
                                } else {
                                    let base = overlapping
                                        .iter()
                                        .map(|&i| topbottom_cols[i].2)
                                        .fold(0.0_f64, f64::max);
                                    let new_l = overlapping
                                        .iter()
                                        .map(|&i| topbottom_cols[i].0)
                                        .fold(h_left, f64::min);
                                    let new_r = overlapping
                                        .iter()
                                        .map(|&i| topbottom_cols[i].1)
                                        .fold(h_right, f64::max);
                                    for &i in overlapping.iter().rev() {
                                        topbottom_cols.remove(i);
                                    }
                                    topbottom_cols.push((new_l, new_r, base + extra));
                                }
                                let applied_after =
                                    topbottom_cols.iter().map(|c| c.2).fold(0.0_f64, f64::max);
                                st.advance_flow_by((applied_after - applied_before).max(0.0));
                            }
                        }
                    }
                }
                Control::Footnote(fn_ctrl) => {
                    self.register_body_footnote(
                        st,
                        para_idx,
                        para,
                        paragraphs,
                        ctrl_idx,
                        fn_ctrl,
                        has_table,
                        native_hwp5_footnote_break,
                    );
                }
                Control::Endnote(en_ctrl) => {
                    // [Task #836] 미주 수집 — 문서 끝에 모아서 렌더
                    st.collect_endnote(EndnoteRef {
                        number: en_ctrl.number,
                        section_index,
                        para_index: para_idx,
                        control_index: ctrl_idx,
                    });
                }
                _ => {}
            }
        }
        // #6950: text and its tail table may be emitted in paint order rather
        // than logical order. Finish the paragraph after ALL its controls;
        // post-text must not return flow to a position above its own table.
        let spacing_after = styles
            .para_styles
            .get(para.para_shape_id as usize)
            .map_or(0.0, |style| style.spacing_after);
        st.finish_paragraph_float_flow(para_idx, spacing_after);
    }
}
