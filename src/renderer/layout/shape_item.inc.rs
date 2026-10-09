// 같은 layout 모듈에서 개체 소유와 최종 표시 원점을 처리한다.
impl LayoutEngine {
    /// Shape PageItem 레이아웃 (layout_column_item에서 분리)
    #[allow(clippy::too_many_arguments)]
    fn layout_shape_item(
        &self,
        tree: &mut PageLayoutContext,
        col_node: &mut RenderNode,
        paper_images: &mut Vec<RenderNode>,
        para_start_y: &mut std::collections::HashMap<usize, f64>,
        // [Task #1151 v9 결함 D] sibling TAC picture 가로 분배 cursor state.
        para_inline_state: &mut std::collections::HashMap<
            usize,
            super::layout::paragraph_layout::ParaInlineState,
        >,
        para_index: usize,
        control_index: usize,
        ctx: &ColumnItemCtx,
        y_offset: f64,
    ) -> f64 {
        let ColumnItemCtx {
            page_content,
            paragraphs,
            composed,
            styles,
            bin_data_content,
            layout,
            col_area,
            wrap_anchors,
            ..
        } = ctx;
        // Task #402: 같은 paragraph 안에 TAC 컨트롤(표/그림/도형) 2개 이상이 서로 다른 line에
        // 배치된 경우, 두 번째 이후의 그림은 paragraph 시작 y가 아니라 진행된 y_offset
        // (선행 TAC 후속 위치)에 그려져야 표와 겹치지 않는다. control_index 이전에 같은
        // paragraph의 TAC 컨트롤이 있고 y_offset이 기존 등록값보다 진행됐으면 갱신한다.
        //
        // [Task #1151 v9 결함 D] sibling TAC picture 만 있는 경우 (= 가로 분배 시나리오) 는
        // para_start_y 갱신 X — picture 의 y 는 line_top_y 로 동일 유지. has_prior_tac 의
        // 종류를 Table/Shape vs Picture 로 분리하여 picture-only 시퀀스에서 y 진행을 차단.
        let has_prior_non_picture_tac = paragraphs
            .get(para_index)
            .map(|p| {
                p.controls.iter().take(control_index).any(|c| match c {
                    Control::Table(t) => t.common.treat_as_char,
                    Control::Shape(s) => s.common().treat_as_char,
                    _ => false,
                })
            })
            .unwrap_or(false);
        let has_prior_tac_picture = paragraphs
            .get(para_index)
            .map(|p| {
                p.controls.iter().take(control_index).any(|c| match c {
                    Control::Picture(p) => p.common.treat_as_char,
                    _ => false,
                })
            })
            .unwrap_or(false);
        let current_is_tac = paragraphs
            .get(para_index)
            .and_then(|para| para.controls.get(control_index))
            .is_some_and(|control| match control {
                Control::Table(table) => table.common.treat_as_char,
                Control::Picture(picture) => picture.common.treat_as_char,
                Control::Shape(shape) => shape.common().treat_as_char,
                _ => false,
            });
        if has_prior_non_picture_tac && current_is_tac {
            // 현재 개체와 선행 Table/Shape가 모두 TAC인 경우에만 진행된 흐름을 쓴다.
            // 떠 있는 후속 개체는 이 흐름을 소비하지 않는다. 문단 원점을 뒤 줄 끝으로
            // 덮으면 공유한 저장 앵커 delta를 올바르지 않은 기준에 더하게 된다.
            let needs_update = para_start_y
                .get(&para_index)
                .map(|&existing| y_offset > existing + 1.0)
                .unwrap_or(true);
            if needs_update {
                para_start_y.insert(para_index, y_offset);
            }
        } else if !has_prior_tac_picture {
            // 첫 picture (선행 TAC picture 도 Table/Shape 도 없음): paragraph 시작 y 등록.
            para_start_y.entry(para_index).or_insert(y_offset);
        }
        // 선행 picture 만 있는 경우 (has_prior_tac_picture && !has_prior_non_picture_tac):
        // para_start_y 의 기존 값 유지 — 가로 분배의 첫 picture y 와 동일.
        let accepted_shape_frame = ctx
            .paragraph_float_placements
            .get(&(para_index, control_index))
            .filter(|frame| {
                frame.flow == super::float_placement::ParagraphFloatFlow::NextLine
                    && paragraphs
                        .get(para_index)
                        .and_then(|para| para.controls.get(control_index))
                        .is_some_and(|control| matches!(control, Control::Shape(_)))
            });
        let y_offset = accepted_shape_frame
            .map(|frame| col_area.y + frame.anchor_y)
            .unwrap_or(y_offset);
        let mut result_y = y_offset;
        if let Some(para) = paragraphs.get(para_index) {
            if let Some(ctrl) = para.controls.get(control_index) {
                if let Control::Picture(pic) = ctrl {
                    if pic.common.treat_as_char {
                        result_y = self.layout_tac_picture_item(
                            tree,
                            col_node,
                            para_start_y,
                            para_inline_state,
                            para,
                            pic,
                            para_index,
                            control_index,
                            ctx,
                            y_offset,
                        );
                    } else {
                        let is_paper_based = (pic.common.vert_rel_to == VertRelTo::Paper
                            || pic.common.vert_rel_to == VertRelTo::Page)
                            && (pic.common.horz_rel_to == HorzRelTo::Paper
                                || pic.common.horz_rel_to == HorzRelTo::Page);
                        if is_paper_based {
                            let mut temp_parent = RenderNode::new(
                                tree.next_id(),
                                RenderNodeType::Column(0),
                                BoundingBox::new(0.0, 0.0, layout.page_width, layout.page_height),
                            );
                            let paper_area = LayoutRect {
                                x: 0.0,
                                y: 0.0,
                                width: layout.page_width,
                                height: layout.page_height,
                            };
                            let _ = self.layout_body_picture(
                                tree,
                                &mut temp_parent,
                                pic,
                                &paper_area,
                                col_area,
                                &layout.body_area,
                                &paper_area,
                                bin_data_content,
                                styles,
                                Alignment::Left,
                                0.0,
                                page_content.section_index,
                                para_index,
                                control_index,
                                false,
                            );
                            let layer = Self::render_layer_from_common(
                                &pic.common,
                                para_index,
                                control_index,
                            );
                            Self::push_layered_paper_children(
                                paper_images,
                                &mut temp_parent,
                                layer,
                            );
                        } else {
                            let comp = composed.get(para_index);
                            let para_style_id = comp
                                .map(|c| c.para_style_id as usize)
                                .unwrap_or(para.para_shape_id as usize);
                            let para_style = styles.para_styles.get(para_style_id);
                            let alignment =
                                para_style.map(|s| s.alignment).unwrap_or(Alignment::Left);
                            let deferred_page_start_square = wrap_anchors
                                .values()
                                .any(|anchor| anchor.anchor_para_index == para_index)
                                && page_content.column_contents.iter().any(|column| {
                                    matches!(
                                        column.items.first(),
                                        Some(PageItem::Shape {
                                            para_index: item_pi,
                                            control_index: item_ci,
                                        }) if *item_pi == para_index && *item_ci == control_index
                                    ) && deferred_page_start_square_picture_uses_body_top(
                                        column,
                                        0,
                                        paragraphs,
                                        para_index,
                                        control_index,
                                    )
                                });
                            // [#6704] 앵커 문단이 앞 쪽/단에서 이어진 조각이면
                            // `para_start_y` 는 그 문단의 시작이 아니라 이 쪽에서 흐름이
                            // 도달한 자리다. Para 기준 개체는 흐름에서 빠져 있으므로 그
                            // 자리를 쓰면 앞 항목이 흘린 만큼 그대로 내려간다 —
                            // hwp3-sample 7쪽 그림이 +85.6px. 한/글은 넘어온 쪽의 본문
                            // 맨 위를 기준으로 삼는다.
                            //
                            // 본문 흐름은 같은 규칙을 이미 지킨다(layout.rs 의
                            // `PartialParagraph { start_line > 0 }` 분기: "이어지는 partial
                            // paragraph 는 이전 쪽/단에서 시작한 문단의 나머지다"). 여기서는
                            // 그 규칙을 개체 앵커에 연결한다.
                            let anchor_starts_on_earlier_page = ctx
                                .page_content
                                .column_contents
                                .iter()
                                .flat_map(|cc| cc.items.iter())
                                .any(|it| {
                                    matches!(it,
                                        PageItem::PartialParagraph { para_index: q, start_line, .. }
                                            if *q == para_index && *start_line > 0)
                                });
                            let para_base_y = if anchor_starts_on_earlier_page {
                                ctx.col_area.y
                            } else {
                                para_start_y.get(&para_index).copied().unwrap_or(y_offset)
                            };
                            if std::env::var("RHWP_5715_DBG").is_ok() {
                                eprintln!(
                                    "[5715] pi={para_index} ci={control_index} base={para_base_y:.1} from_map={} y_off={y_offset:.1}",
                                    para_start_y.contains_key(&para_index)
                                );
                            }
                            let para_base_y = if deferred_page_start_square {
                                // next-page owner의 `vpos=0` narrow band는 새 physical
                                // page의 body top을 가리킨다. layout_body_picture가
                                // para-relative offset을 한 번 적용하므로 source offset을
                                // 여기서 상쇄한다 (#3820 p127 Figure 56).
                                para_base_y
                                    - hwpunit_to_px(pic.common.vertical_offset as i32, self.dpi)
                            } else {
                                para_base_y
                            };
                            // 첫 좁은 줄 대신 control의 실제 소유 줄을 측정과 공유한다.
                            // layout_body_picture는 이 앵커에서 원본 offset을 한 번 적용한다.
                            let pic_y = para_base_y
                                + crate::renderer::float_placement::stored_picture_anchor_delta(
                                    para,
                                    control_index,
                                    col_area.width,
                                    styles,
                                    self.dpi,
                                )
                                .unwrap_or(0.0);
                            let saved_y_offset = y_offset;
                            // [Task #1079] 파일 vpos 가 이미 그림 공간을 반영(그림 para 줄 앞
                            // gap ≥ 그림 높이)하면 그림 높이 추가 진행 생략(typeset pushdown
                            // 게이트와 동일 조건). #409 계열(gap 작음)은 현행 유지.
                            // [편집 세션] 이 게이트의 근거(파일 vpos gap)는 저장
                            // 시점 형상이다 — 셀 편집으로 앞 문단(표 host)이 자라면
                            // gap 은 이미 소비된 공간이라, 그림을 gap 안(base 위)으로
                            // 올리면 커진 표 하단에 겹친다(셀 Enter 재현: 그림이
                            // 표 하단 위에 그려짐).
                            let vpos_accounts_for_height =
                                !self.profile.get().session_edited() && para_index > 0 && {
                                    const PUSHDOWN_GAP_TOL_PX: f64 = 8.0;
                                    let obj_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                                    let v_cur = paragraphs[para_index]
                                        .line_segs
                                        .first()
                                        .map(|s| s.vertical_pos);
                                    let prev_end = paragraphs[para_index - 1]
                                        .line_segs
                                        .last()
                                        .map(|s| s.vertical_pos + s.line_height);
                                    match (v_cur, prev_end) {
                                        (Some(vc), Some(pe)) if vc > pe => {
                                            hwpunit_to_px((vc - pe) as i32, self.dpi)
                                                >= obj_h - PUSHDOWN_GAP_TOL_PX
                                        }
                                        _ => false,
                                    }
                                };
                            // [#5715] #1079 gap 휴리스틱은 그 gap 이 **현재 쪽에 물리적으로
                            // 실재**할 때만 유효하다. 쪽 리셋 뒤 유령 사다리(앞 lineage 의
                            // vpos 65410 이 리셋 0-기저 쪽에 섞임)가 만든 가짜 gap 이면
                            // 바닥-정렬이 그림을 단 상단 위로 밀어 지면 밖 소실이 된다
                            // (베트남노동시장1125: 124쪽 중 8쪽의 차트가 본문 위/지면 밖).
                            // 바닥-정렬 목표가 단 상단을 넘으면 기각하고, 같은 문단의 선행
                            // float(표)이 이미 진행시킨 흐름(y_offset) 뒤로 배치한다.
                            let (vpos_accounts_for_height, pic_y) = if vpos_accounts_for_height
                                && pic_y - hwpunit_to_px(pic.common.height as i32, self.dpi)
                                    < col_area.y - 0.5
                            {
                                (false, pic_y.max(y_offset))
                            } else {
                                (vpos_accounts_for_height, pic_y)
                            };
                            // 조판이 확정한 저장 그림 프레임은 실제 배치에서도 같은 앵커를 쓴다.
                            // 뒤 호스트 줄의 원점이나 그림 높이로 이 앵커를 다시 추정하지 않는다.
                            let stored_picture = ctx.paragraph_float_placements
                                .get(&(para_index, control_index))
                                .filter(|placement| matches!(placement.flow,
                                    super::float_placement::ParagraphFloatFlow::StoredPicture { .. }
                                        | super::float_placement::ParagraphFloatFlow::StoredPictureGroup { .. })
                                    || (placement.flow == super::float_placement::ParagraphFloatFlow::Exclusion
                                        && placement.stored_host_origin.is_some()));
                            let (vpos_accounts_for_height, pic_y) = stored_picture
                                .map(|placement| (false, col_area.y + placement.anchor_y))
                                .unwrap_or((vpos_accounts_for_height, pic_y));
                            // typeset의 ObjectPlacementFrame과 같은 문단 좌우 여백을
                            // Para 기준 그림에도 준다. paint만 단 전체를 기준으로
                            // 두면 TIFF 그림과 캡션이 저장 1000HU만큼 왼쪽으로 간다
                            // (#7406 p61). 다른 상대 기준은 별도 column/body/paper다.
                            let para_margin_left = para_style.map_or(0.0, |s| s.margin_left);
                            let para_margin_right = para_style.map_or(0.0, |s| s.margin_right);
                            let pic_container = LayoutRect {
                                x: col_area.x + para_margin_left,
                                y: pic_y,
                                width: (col_area.width - para_margin_left - para_margin_right)
                                    .max(0.0),
                                height: col_area.height - (pic_y - col_area.y),
                            };
                            result_y = self.layout_body_picture(
                                tree,
                                col_node,
                                pic,
                                &pic_container,
                                col_area,
                                &layout.body_area,
                                &LayoutRect {
                                    x: 0.0,
                                    y: 0.0,
                                    width: layout.page_width,
                                    height: layout.page_height,
                                },
                                bin_data_content,
                                styles,
                                alignment,
                                pic_y,
                                page_content.section_index,
                                para_index,
                                control_index,
                                vpos_accounts_for_height,
                            );
                            if let Some(placement) = stored_picture {
                                result_y = col_area.y
                                    + placement.paragraph_end(saved_y_offset - col_area.y, 0.0);
                            }
                            // layout_body_picture needs the host paragraph y for Para-relative
                            // positioning, but InFront pictures must not rewind the
                            // already-advanced text flow cursor back to that paragraph y.
                            //
                            // Keep BehindText on the legacy non-advancing path. HWP5 files such
                            // as samples/복학원서.hwp use an empty first paragraph with a
                            // BehindText logo; preserving the advanced cursor there inserts an
                            // extra line-height before the following table.
                            if matches!(
                                pic.common.text_wrap,
                                crate::model::shape::TextWrap::InFrontOfText
                            ) {
                                result_y = saved_y_offset;
                            }
                            // A co-anchored fixed title needs the otherwise
                            // empty host's content line. Keep ordinary BehindText
                            // logo hosts on the legacy path described above.
                            if self.profile.get().hwp5_stored_pagination_layout() {
                                if let Some(floor) = anchor_box_flow::backdrop_title_host_floor(
                                    para,
                                    &pic.common,
                                    pic_y,
                                    self.dpi,
                                ) {
                                    result_y = result_y.max(floor.min(saved_y_offset));
                                }
                            }
                            // [Task #959] horz_rel_to=Column 의 picture 가 col_area 우측을
                            // 초과하는 위치에 emit 되면 한컴 viewer 는 column flow 에
                            // reservation 하지 않음. rhwp 는 cursor 를 picture height 만큼
                            // advance → 후속 paragraph 처짐.
                            // (3-11월_실전_통합_2022 page 1 우측 단 pi=69 picture
                            //  pic_emit_x=767 > col_right=759 → +274px advance → 문9 처짐)
                            // Picture 의 좌측 edge (x) 가 col_area 우측을 초과하면 advance skip.
                            let mut picture_outside_column = false;
                            if matches!(pic.common.horz_rel_to, HorzRelTo::Column) {
                                // [Issue #1230] emit 폭 판정은 layout_body_picture 와 동일한
                                // 프레임 크기를 사용해야 skip 판정이 실제 배치와 일치한다.
                                let (pic_width_hu, _) = picture_flow_frame_size_hu(pic);
                                let pic_width_px = hwpunit_to_px(pic_width_hu, self.dpi);
                                let h_offset_px =
                                    hwpunit_to_px(pic.common.horizontal_offset as i32, self.dpi);
                                let pic_emit_x = match pic.common.horz_align {
                                    crate::model::shape::HorzAlign::Left
                                    | crate::model::shape::HorzAlign::Inside => {
                                        col_area.x + h_offset_px
                                    }
                                    crate::model::shape::HorzAlign::Center => {
                                        col_area.x
                                            + (col_area.width - pic_width_px) / 2.0
                                            + h_offset_px
                                    }
                                    crate::model::shape::HorzAlign::Right
                                    | crate::model::shape::HorzAlign::Outside => {
                                        col_area.x + col_area.width - pic_width_px - h_offset_px
                                    }
                                };
                                if pic_emit_x >= col_area.x + col_area.width {
                                    picture_outside_column = true;
                                    result_y = saved_y_offset;
                                }
                            }
                            if !picture_outside_column && !vpos_accounts_for_height {
                                let host_line_advance = hwpunit_to_px(
                                    crate::renderer::hwp3_caption_host::line_advance_hwp(
                                        para,
                                        pic,
                                        self.profile.get(),
                                    ),
                                    self.dpi,
                                );
                                if host_line_advance > 0.0 {
                                    // 뒤의 빈 호스트 줄은 흐름을 전진시키지만 그림·캡션의
                                    // 콘텐츠 하단은 아니다. 기존 초과 판정에 그 점유 끝을
                                    // 공급하고 조판과 공유하는 다음 흐름 원점은 유지한다.
                                    let content_bottom = self.last_item_content_bottom.get();
                                    self.last_item_content_bottom.set(
                                        if content_bottom.is_finite() {
                                            content_bottom.max(result_y)
                                        } else {
                                            result_y
                                        },
                                    );
                                    result_y += host_line_advance;
                                }
                            }
                            // [Task #683] 빈 paragraph (텍스트 없음) + Para-relative TopAndBottom
                            // 그림 (caption 없음) 의 layout 진행량 보정. 한컴 한글 2022 PDF 는
                            // 그림 다음에 paragraph 의 line baseline 1줄(line_height + line_spacing)
                            // 을 추가 진행하나 rhwp 기본 layout 은 image_height 만 진행하여
                            // cluster 거리가 1 line 부족 (pr-149.hwp 18864 HU vs 17280 HU 결함).
                            // 단 밖 그림은 물리 띠를 예약하지 않는다. 호스트가 이미
                            // 예약한 줄을 그림 높이의 뒤 간격으로 다시 더하지 않는다.
                            if !picture_outside_column && matches!(
                                pic.common.text_wrap,
                                crate::model::shape::TextWrap::TopAndBottom
                            ) && matches!(
                                pic.common.vert_rel_to,
                                crate::model::shape::VertRelTo::Para
                            ) && pic.caption.is_none()
                                // 확정된 뒤 줄 원점은 호스트 점유를 이미 포함한다.
                                && stored_picture
                                    .and_then(|placement| placement.stored_successor_line_origin)
                                    .is_none()
                                // 복수 그림의 합집합 계획은 빈 호스트 줄도 이미 소유한다.
                                && !stored_picture.is_some_and(|placement| matches!(placement.flow,
                                    super::float_placement::ParagraphFloatFlow::StoredPictureGroup { .. }))
                            {
                                let has_visible_text =
                                    para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                                // [#6574] 미주 문단은 같은 미주의 다음 문단 저장 vpos 가 한컴이 이
                                // 그림 뒤에 둔 자리다. 사다리가 그림 바닥까지만 진행하면 호스트
                                // 빈 줄을 따로 더하지 않는다(그림 바닥에 바로 이어진다).
                                let endnote_stored_successor_y = (para_index
                                    >= self.endnote_para_base.get()
                                    && self.endnote_para_has_same_endnote_successor(para_index))
                                .then(|| {
                                    let first = para.line_segs.first()?.vertical_pos;
                                    let next_first = paragraphs
                                        .get(para_index + 1)?
                                        .line_segs
                                        .first()?
                                        .vertical_pos;
                                    let stored_y = saved_y_offset
                                        + hwpunit_to_px(next_first - first, self.dpi);
                                    (next_first > first && stored_y + 0.5 >= result_y)
                                        .then_some(stored_y)
                                })
                                .flatten();
                                if let Some(stored_y) = endnote_stored_successor_y {
                                    result_y = stored_y;
                                } else if !has_visible_text
                                    // [#7470] host 줄이 그림 띠에 흡수된 빈 host(`sw=0`)는 더하지 않는다.
                                    && (self.column_is_endnote_flow.get()
                                        || !crate::renderer::empty_host_line_absorbed_by_topbottom_float(
                                            para,
                                            &pic.common,
                                        ))
                                {
                                    let line_advance = para
                                        .line_segs
                                        .first()
                                        .map(|ls| {
                                            hwpunit_to_px(
                                                ls.line_height + ls.line_spacing,
                                                self.dpi,
                                            )
                                        })
                                        .unwrap_or(0.0);
                                    result_y += line_advance;
                                }
                            }
                            // Square wrap + Para-relative: 그림 높이로 column y를 밀지 않는다.
                            // 텍스트는 그림 옆에 segment_width로 제어되어 흐르므로
                            // 후속 문단은 앵커 단락 직후(shape item y_offset)부터 시작해야 한다.
                            // layout_body_picture의 y_offset은 pic_y(=단락 시작 y)이므로
                            // 반환값이 para_start_y로 거슬러 올라감 — 이를 shape item y로 복원.
                            if matches!(pic.common.text_wrap, crate::model::shape::TextWrap::Square)
                                && matches!(
                                    pic.common.vert_rel_to,
                                    crate::model::shape::VertRelTo::Para
                                )
                            {
                                result_y = y_offset;
                            }
                            // [Task #525] Picture Square wrap 의 호스트 paragraph 텍스트는
                            // 정상 PageItem::FullParagraph 경로 (layout_composed_paragraph 의
                            // has_picture_shape_square_wrap 분기, paragraph_layout.rs:822/973)
                            // 가 LINE_SEG.cs/sw 기반으로 그림 옆 (좁은) + 그림 아래 (넓은)
                            // 모두 처리. Task #604 Stage 2 의 wrap_anchors 메타데이터 채널
                            // 로 FullParagraph path 가 cs offset 을 정확히 적용하므로 별도 호출 불필요.
                        }
                    }
                } else if let Control::Shape(shape) = ctrl {
                    let common = shape.common();
                    if common.treat_as_char {
                        let has_real_text =
                            para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                        let registered_inline_pos = tree.get_inline_shape_position(
                            page_content.section_index,
                            para_index,
                            control_index,
                            None,
                        );
                        let already_registered = registered_inline_pos.is_some();

                        if !has_real_text {
                            let shape_w = hwpunit_to_px(common.width as i32, self.dpi);
                            let shape_h = hwpunit_to_px(common.height as i32, self.dpi);
                            // [Task #990] 해당 문단에 PageItem::FullParagraph 가
                            // 발행되었으면(빈 문단이 호스트인 RFP 형) layout_paragraph
                            // 가 이미 LINE_SEG advance 를 마쳤으므로, Shape 항목은
                            // 글상자를 호스트 문단 시작(para_start)에 배치하고 재진행
                            // 하지 않는다 — 이중 가산 방지(Task #974 c3e32151 회귀).
                            // FullParagraph 항목이 없으면(선행 표 등에 이어 붙은
                            // Shape, 예: hy-001 pi=27) Task #974 동작을 유지한다.
                            let has_full_para_item =
                                page_content.column_contents.iter().any(|cc| {
                                    cc.items.iter().any(|it| {
                                        matches!(
                                            it,
                                            PageItem::FullParagraph { para_index: pi }
                                                if *pi == para_index
                                        )
                                    })
                                });
                            let para_start =
                                para_start_y.get(&para_index).copied().unwrap_or(y_offset);
                            let shape_y = if let Some((_, registered_y)) = registered_inline_pos {
                                registered_y
                            } else if has_full_para_item {
                                para_start
                            } else {
                                y_offset
                            };

                            if !already_registered {
                                let comp = composed.get(para_index);
                                let para_style_id = comp
                                    .map(|c| c.para_style_id as usize)
                                    .unwrap_or(para.para_shape_id as usize);
                                let para_style_ref = styles.para_styles.get(para_style_id);
                                let para_alignment = para_style_ref
                                    .map(|s| s.alignment)
                                    .unwrap_or(Alignment::Left);
                                let para_margin_left =
                                    para_style_ref.map(|s| s.margin_left).unwrap_or(0.0);
                                let para_indent = para_style_ref.map(|s| s.indent).unwrap_or(0.0);
                                let para_margin_right =
                                    para_style_ref.map(|s| s.margin_right).unwrap_or(0.0);
                                let effective_margin_left = if para_indent > 0.0 {
                                    para_margin_left + para_indent
                                } else {
                                    para_margin_left
                                };
                                let avail_w =
                                    (col_area.width - effective_margin_left - para_margin_right)
                                        .max(shape_w);
                                let shape_x = match para_alignment {
                                    Alignment::Center | Alignment::Distribute => {
                                        col_area.x
                                            + effective_margin_left
                                            + (avail_w - shape_w).max(0.0) / 2.0
                                    }
                                    Alignment::Right => {
                                        col_area.x
                                            + effective_margin_left
                                            + (avail_w - shape_w).max(0.0)
                                    }
                                    _ => col_area.x + effective_margin_left,
                                };

                                tree.set_inline_shape_position(
                                    page_content.section_index,
                                    para_index,
                                    control_index,
                                    None,
                                    shape_x,
                                    shape_y,
                                );
                            }

                            // [Task #990] FullParagraph 항목이 없는 경우에만
                            // LINE_SEG 1회분을 진행한다. FullParagraph 가 있으면
                            // 이미 진행되었으므로 result_y(=y_offset)를 유지한다.
                            if !has_full_para_item {
                                // 텍스트 없는 호스트 문단에서 자리차지 개체는 하나가
                                // 한 줄을 차지한다. 그러니 진행분은 **이 개체의** 줄
                                // 이어야 하는데 first() 는 언제나 첫 줄이다. 같은
                                // 문단에 선행 자리차지 개체가 있으면(표+글상자 등)
                                // 앞 개체의 줄 높이로 전진해 그만큼 과전진한다
                                // (1351000 정책연구용역 중간보고서 pi=1920:
                                // ls[0] 표 449.9px 로 전진, 실제 도형 줄은 222.1px
                                // — 227.8px 과전진이 뒤따르는 문단 10개를 쪽 밖으로
                                // 밀어냈다). 선행 자리차지 개체 수를 줄 인덱스로 쓴다.
                                let line_idx = para
                                    .controls
                                    .iter()
                                    .take(control_index)
                                    .filter(|c| match c {
                                        Control::Table(t) => t.common.treat_as_char,
                                        Control::Picture(p) => p.common.treat_as_char,
                                        Control::Shape(s) => s.common().treat_as_char,
                                        _ => false,
                                    })
                                    .count();
                                let line_advance = para
                                    .line_segs
                                    .get(line_idx)
                                    .or_else(|| para.line_segs.first())
                                    .map(|ls| {
                                        hwpunit_to_px(ls.line_height + ls.line_spacing, self.dpi)
                                    })
                                    .unwrap_or(shape_h);
                                result_y = shape_y + line_advance.max(shape_h);
                            }
                            let prev_bottom = self.last_item_content_bottom.get();
                            let shape_bottom = shape_y + shape_h;
                            self.last_item_content_bottom
                                .set(if prev_bottom.is_finite() {
                                    prev_bottom.max(shape_bottom)
                                } else {
                                    shape_bottom
                                });
                        }
                    } else if !common.treat_as_char
                        && matches!(shape.as_ref(), ShapeObject::Ole(_))
                        && matches!(common.text_wrap, TextWrap::Square)
                        && matches!(common.vert_rel_to, VertRelTo::Para)
                    {
                        let has_visible_text =
                            para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
                        if !has_visible_text {
                            let line_advance = para
                                .line_segs
                                .first()
                                .map(|ls| hwpunit_to_px(ls.line_height + ls.line_spacing, self.dpi))
                                .unwrap_or(0.0);
                            // FullParagraph와 OLE는 같은 호스트 줄의 점유 끝을 소비한다.
                            // 이미 전진한 커서에 피치를 다시 더하면 Enter 줄이 두 줄 아래로 간다.
                            let host_y = para_start_y.get(&para_index).copied().unwrap_or(y_offset);
                            result_y = result_y.max(host_y + line_advance);
                        }
                    } else if !common.treat_as_char
                        && matches!(
                            common.text_wrap,
                            crate::model::shape::TextWrap::TopAndBottom
                        )
                        && matches!(common.vert_rel_to, crate::model::shape::VertRelTo::Para)
                        && !ctx
                            .paragraph_float_placements
                            .get(&(para_index, control_index))
                            .is_some_and(|placement| {
                                matches!(
                                    placement.flow,
                                    crate::renderer::float_placement::ParagraphFloatFlow::StoredPicture { .. }
                                )
                            })
                    {
                        // [Issue #1156] 비-TAC 자리차지(TopAndBottom) 객체(차트 OLE 등):
                        // 자리차지 객체는 본문 텍스트를 위/아래로 밀어내므로, 후속 콘텐츠
                        // 시작 y(result_y)를 객체 높이 + 위·아래 여백만큼 진행시켜 텍스트가
                        // 객체 영역과 겹치지 않게 한다. (typeset.rs Stage 2 의 current_height
                        // 가산과 layout 정합 — 단 이동 후 단 시작 y_offset 기준.)
                        let shape_h = super::composer::captioned_shape_height_px(
                            hwpunit_to_px(common.height as i32, self.dpi),
                            &super::composer::shape_caption_for_layout(shape),
                            self.dpi,
                        );
                        let margin_top = hwpunit_to_px(
                            super::float_placement::topbottom_shape_top_margin_hu(common),
                            self.dpi,
                        );
                        let margin_bottom = hwpunit_to_px(common.margin.bottom as i32, self.dpi);
                        let occupied_bottom =
                            crate::renderer::float_placement::column_ole_placement(
                                shape, y_offset, self.dpi,
                            )
                            .map_or(y_offset + shape_h + margin_bottom, |placement| {
                                placement.occupied_bottom
                            });
                        result_y = result_y.max(occupied_bottom);
                    }
                }
            }
        }
        accepted_shape_frame
            .map(|frame| col_area.y + frame.occupied_bottom)
            .unwrap_or(result_y)
    }
}
