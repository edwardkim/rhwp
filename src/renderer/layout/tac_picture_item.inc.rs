// 같은 layout 모듈에서 글줄 그림의 표시·캡션·흐름 상태를 처리한다.
impl LayoutEngine {
    #[allow(clippy::too_many_arguments)]
    fn layout_tac_picture_item(
        &self,
        tree: &mut PageLayoutContext,
        col_node: &mut RenderNode,
        para_start_y: &mut std::collections::HashMap<usize, f64>,
        para_inline_state: &mut std::collections::HashMap<usize, paragraph_layout::ParaInlineState>,
        para: &Paragraph,
        pic: &crate::model::image::Picture,
        para_index: usize,
        control_index: usize,
        ctx: &ColumnItemCtx,
        y_offset: f64,
    ) -> f64 {
        let ColumnItemCtx {
            page_content,
            composed,
            styles,
            bin_data_content,
            col_area,
            ..
        } = ctx;
        let mut result_y = y_offset;
        let (pic_w, pic_h) = self.resolve_inline_picture_size(pic, col_area);
        // [#6603] 줄 안 폭·높이는 바깥 여백을 포함한 상자로 세고, 잉크는
        // 상자의 (왼쪽, 위) 여백 안쪽에 그린다 (paragraph_layout 과 같은 계약).
        let (margin_left, margin_right, margin_top, margin_bottom) =
            super::layout::paragraph_layout::tac_picture_outer_margins_px(pic, self.dpi);
        let box_w = pic_w + margin_left + margin_right;
        let box_h = pic_h + margin_top + margin_bottom;
        // 같은 paragraph 의 sibling wrap=TopAndBottom 개체(tac=false)가
        // 차지하는 vertical 영역만큼 picture y 보정.
        let sibling_reserved_hu =
            super::layout::paragraph_layout::calc_sibling_topandbottom_reserved_hu(&para.controls);
        let sibling_reserved_px = hwpunit_to_px(sibling_reserved_hu, self.dpi);

        // [Task #1151 v9 결함 D] sibling TAC picture 시퀀스 위치 판별.
        // 한컴 native 정합: 동일 paragraph 안 sibling tac=true picture 들이
        // 가로로 inline 분배 (inline glyph 처럼).
        let tac_pic_seq: Vec<(usize, f64)> = para
            .controls
            .iter()
            .enumerate()
            .filter_map(|(ci, control)| match control {
                Control::Picture(sibling) if sibling.common.treat_as_char => {
                    let (ml, mr, _, _) =
                        super::layout::paragraph_layout::tac_picture_outer_margins_px(
                            sibling, self.dpi,
                        );
                    Some((
                        ci,
                        self.resolve_inline_picture_size(sibling, col_area).0 + ml + mr,
                    ))
                }
                _ => None,
            })
            .collect();
        let position_in_seq = tac_pic_seq.iter().position(|(ci, _)| *ci == control_index);
        let is_single_pic = tac_pic_seq.len() == 1;
        let is_first_in_seq = position_in_seq == Some(0);
        let is_subsequent_in_seq = position_in_seq.map(|p| p > 0).unwrap_or(false);
        let is_last_in_seq = position_in_seq
            .map(|p| p == tac_pic_seq.len() - 1)
            .unwrap_or(false);

        // pic_y 결정:
        // - 단일 picture / 시퀀스 첫 picture: paragraph 시작 y + sibling_reserved
        //   + 라벨/그림 높이 정합 보정
        // - 시퀀스 후속 picture: state.line_top_y (pic_x wrap 처리 후 결정 — 아래)
        // [Task #1151 v9 결함 D fix] pic_y 의 시퀀스 후속 picture 결정은 pic_x
        // (wrap 처리 포함) 뒤로 옮김. 그 전에는 placeholder 로 default 값 사용.
        let _ = is_single_pic;
        let comp = composed.get(para_index);
        // sibling 자리차지 표 예약은 통짜 배치 가정이다 — 표가 분할
        // 이월된 쪽에서는 "문단 시작 + 전체 표 높이"가 단 하단을 넘어
        // tac 그림이 쪽 밖에 그려진다(재현 문서 A 셀 Enter: 로고만
        // 쪽 하단 밖). 그때는 흐름 y(분할 조각·후행 텍스트 뒤)로
        // 폴백한다.
        let para_y_for_pic = {
            let reserved_based =
                para_start_y.get(&para_index).copied().unwrap_or(y_offset) + sibling_reserved_px;
            if sibling_reserved_px > 0.0 && reserved_based > col_area.y + col_area.height + 60.0 {
                y_offset
            } else {
                reserved_based
            }
        };
        let default_pic_y =
            self.compute_tac_picture_shape_y(para, comp, styles, para_y_for_pic, pic_h);
        let para_style_id = comp
            .map(|c| c.para_style_id as usize)
            .unwrap_or(para.para_shape_id as usize);
        let para_style_ref = styles.para_styles.get(para_style_id);
        let para_alignment = para_style_ref
            .map(|s| s.alignment)
            .unwrap_or(Alignment::Left);
        // Task #347: 첫 줄 effective_margin (hanging indent: indent<0 → first-line은 margin_left만 적용)
        let para_margin_left = para_style_ref.map(|s| s.margin_left).unwrap_or(0.0);
        let para_indent = para_style_ref.map(|s| s.indent).unwrap_or(0.0);
        // [Task #544 v2 정합] paragraph_layout.rs (a30dca73, Task #544 v2 Stage 2)
        // 는 has_visible_stroke / bs_left_px / bs_right_px 를 보고
        // box_margin_left 를 inner padding 으로 한 번 더 가산하던 분기를
        // 이중 inset 부작용으로 판단해 완전히 제거했다 (margin_left =
        // box_margin_left 단일 룰, PDF 정합 확인됨). 본 TAC picture/shape
        // 경로는 그 수정이 반영되지 않아 테두리 있는 문단(has_visible_stroke)
        // + border_spacing[0]=[1]=0 조건에서 그림이 같은 문단의 텍스트보다
        // para_margin_left 만큼 더 오른쪽으로 밀리는 결함이 있었다
        // (exam_kor.hwp p18/pi=46,50,54,56... 실측 확인 — inner_pad_left=11.33px
        // 만큼 이중 가산). paragraph_layout.rs 와 동일하게 가산 없이 통일한다.
        let mut effective_margin_left =
            tac_picture_effective_margin_left(para_margin_left, para_indent);
        // [Task #534 v2] LINE_SEG.column_start 는 Square wrap 인라인 표/그림이
        // 좌측에 floating 시 표 영역 이후 텍스트 시작 위치를 HWP IR 가 인코딩.
        // layout_shape_item 은 col_area.x 그대로 사용 → picture (TAC) 가 표
        // 영역 위에 겹쳐 표시되는 결함 (예: exam_kor p18 pi=50/56 [A]/[B]
        // 표시기 + 그림). cs 가 effective_margin_left 보다 크면 cs 우선.
        let line_seg_cs_px = para
            .line_segs
            .first()
            .map(|s| hwpunit_to_px(s.column_start, self.dpi))
            .unwrap_or(0.0);
        if line_seg_cs_px > effective_margin_left {
            effective_margin_left = line_seg_cs_px;
        }
        let para_margin_right = para_style_ref.map(|s| s.margin_right).unwrap_or(0.0);
        let avail_w = (col_area.width - effective_margin_left - para_margin_right).max(box_w);
        // [Task #1151 v9 결함 D] pic_x 결정:
        // - 단일 picture: 기존 alignment 그대로
        // - 시퀀스 첫 picture: total_tac_width 기반 alignment + state 초기화
        // - 시퀀스 후속 picture: state.cursor_x 사용 (가로 누적)
        let pic_x = if is_subsequent_in_seq {
            let cur = para_inline_state
                .get(&para_index)
                .map(|s| s.cursor_x)
                .unwrap_or(col_area.x + effective_margin_left);
            let line_right = col_area.x + effective_margin_left + avail_w;
            // [Task #1151 v9 Stage 24] line wrap: cursor_x + pic_w > avail 면
            // 다음 line 으로 wrap (cursor_x reset, line_top_y advance).
            if cur + box_w > line_right + 0.5 {
                if let Some(state) = para_inline_state.get_mut(&para_index) {
                    state.cursor_x = col_area.x + effective_margin_left;
                    state.line_top_y += state.line_height;
                    state.line_height = 0.0;
                }
                col_area.x + effective_margin_left
            } else {
                cur
            }
        } else if is_first_in_seq && !is_single_pic {
            // 시퀀스 첫 picture: 전체 시퀀스 폭 기반 alignment.
            let total_tac_width: f64 = tac_pic_seq.iter().map(|(_, w)| w).sum();
            let align_offset = match para_alignment {
                Alignment::Center | Alignment::Distribute => {
                    (avail_w - total_tac_width).max(0.0) / 2.0
                }
                Alignment::Right => (avail_w - total_tac_width).max(0.0),
                _ => 0.0,
            };
            col_area.x + effective_margin_left + align_offset
        } else {
            // 단일 picture (기존 경로): 기존 alignment 그대로.
            match para_alignment {
                Alignment::Center | Alignment::Distribute => {
                    col_area.x + effective_margin_left + (avail_w - box_w).max(0.0) / 2.0
                }
                Alignment::Right => col_area.x + effective_margin_left + (avail_w - box_w).max(0.0),
                _ => col_area.x + effective_margin_left,
            }
        };

        // [Task #1151 v9 결함 D fix] pic_y 의 시퀀스 후속 picture 결정 —
        // pic_x wrap 처리 후 갱신된 state.line_top_y 사용 (wrap 시 진행됨).
        let pic_y = if is_subsequent_in_seq {
            para_inline_state
                .get(&para_index)
                .map(|s| s.line_top_y)
                .unwrap_or(default_pic_y)
        } else {
            default_pic_y
        };

        // Task #347: paragraph_layout이 호출되지 않는 빈 문단(텍스트 없음 +
        // TAC 그림만 있는 경우)에서는 인라인 그림이 누락되어
        // 박스 프레임 시각이 사라지고 후속 InFrontOfText 표가 위로 겹침.
        // 호스트 문단에 실제 텍스트가 없으면 여기서 직접 이미지 노드를 생성하고
        // y_offset을 그림 높이만큼 진행시킨다.
        let has_real_text = para.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}');
        let has_full_para_item = page_content.column_contents.iter().any(|cc| {
            cc.items.iter().any(|it| {
                matches!(
                    it,
                    PageItem::FullParagraph { para_index: pi }
                        if *pi == para_index
                )
            })
        });
        // [Task #418/#376] paragraph_layout 의 빈 문단 + TAC Picture 분기에서
        // 이미 ImageNode 가 emit 되어 inline_shape_position 이 등록된 경우,
        // 여기서 또 push 하면 이중 emit 이 된다. 등록된 경우 push 를 스킵하고
        // result_y 만 갱신한다.
        //
        // [Task #1452 Stage 6] FullParagraph 가 있는 문단은 paragraph_layout 이
        // TAC 그림을 줄 안에 배치한다. 위치 등록이 아직 안 보이는 순서여도
        // Shape fallback 을 그리면 같은 투명 그림이 한 번 더 합성된다.
        let registered_inline_pos = tree.get_inline_shape_position(
            page_content.section_index,
            para_index,
            control_index,
            None,
        );
        let already_registered = registered_inline_pos.is_some();
        // paragraph_layout 은 조합 줄 안에서만 TAC 그림을 그린다. 저장 줄도 글자도 없는
        // 문단은 조합 줄이 비어 높이만 전진하는 폴백을 타므로, FullParagraph 가 있어도
        // 그림은 여기서 그려야 한다.
        let para_path_draws_picture =
            has_full_para_item && comp.is_some_and(|c| !c.lines.is_empty());
        let effective_pic_y = registered_inline_pos
            .map(|(_, registered_y)| registered_y)
            .unwrap_or(pic_y);
        // paragraph_layout 이 이미 emit 한 인라인 그림은 실제 bbox 높이와 같은
        // common.height 기준으로 content bottom 을 판정한다.
        let effective_pic_h = if already_registered {
            hwpunit_to_px(pic.common.height as i32, self.dpi)
        } else {
            pic_h
        };
        // [Task #1151 v9 결함 D] state 갱신 — 가로 분배 cursor 누적.
        // 첫 picture 시 line_top_y / cursor_x 초기화. 후속 picture 마다 cursor_x 가산.
        if !is_single_pic {
            let entry = para_inline_state.entry(para_index).or_insert(
                super::layout::paragraph_layout::ParaInlineState {
                    cursor_x: pic_x + box_w,
                    line_top_y: pic_y,
                    line_height: box_h,
                },
            );
            if is_subsequent_in_seq {
                entry.cursor_x = pic_x + box_w;
                entry.line_height = entry.line_height.max(box_h);
            } else {
                // 첫 picture: 초기화 (기존 값 덮어쓰기)
                entry.cursor_x = pic_x + box_w;
                entry.line_top_y = pic_y;
                entry.line_height = box_h;
            }
        }

        if !already_registered && !para_path_draws_picture {
            let bin_data_id = pic.image_attr.bin_data_id;
            let image_data = find_bin_data_bytes(bin_data_content, bin_data_id);
            let crop = pic.render_crop_rect();
            let original_size_hu = pic.crop_reference_size();
            let img_id = tree.next_id();
            let img_node = RenderNode::new(
                img_id,
                RenderNodeType::Image(ImageNode {
                    section_index: Some(page_content.section_index),
                    para_index: Some(para_index),
                    control_index: Some(control_index),
                    crop,
                    original_size_hu,
                    effect: pic.image_attr.effect,
                    brightness: pic.image_attr.brightness,
                    contrast: pic.image_attr.contrast,
                    opacity: pic.image_attr.opacity(),
                    transform: utils::extract_shape_transform(&pic.shape_attr),
                    // [Issue #1167] wrap 모드 보존 — SVG plane multi-pass z-order
                    // 판별에 사용 (BehindText 워터마크가 본문 뒤로). PaintOp
                    // 경로(skia/canvaskit)는 별도로 image.text_wrap 을 set 하므로 무관.
                    text_wrap: Some(pic.common.text_wrap),
                    external_path: pic.image_attr.external_path.clone(),
                    content_inset: utils::picture_content_inset(pic),
                    ..ImageNode::new(bin_data_id, image_data)
                }),
                BoundingBox::new(pic_x + margin_left, pic_y + margin_top, pic_w, pic_h),
            );
            // Task #347: 같은 문단의 InFrontOfText 표가 이미 렌더되어
            // col_node.children에 들어있으면 그 앞에 끼워넣어 z-order 보존
            // (인라인 TAC 그림은 박스 프레임 시각이고 InFrontOfText 표가
            //  본문 콘텐츠로 그 위에 그려져야 함).
            let border_node =
                self.make_picture_border_node(tree, pic, img_node.bbox);
            let insert_pos = col_node.children.iter().position(|c| {
                matches!(&c.node_type, RenderNodeType::Table(t)
                    if t.para_index == Some(para_index))
            });
            if let Some(pos) = insert_pos {
                col_node.children.insert(pos, img_node);
                if let Some(border_node) = border_node {
                    col_node.children.insert(pos + 1, border_node);
                }
            } else {
                col_node.children.push(img_node);
                if let Some(border_node) = border_node {
                    col_node.children.push(border_node);
                }
            }
            // 후속 InFrontOfText 객체의 para_y 기준이 되도록 위치 등록
            tree.set_inline_shape_position(
                page_content.section_index,
                para_index,
                control_index,
                None,
                pic_x + margin_left,
                pic_y + margin_top,
            );
            if !has_real_text {
                // [Task #462] LINE_SEG 의 lh+ls 를 advance 로 사용 — 이미지 박스
                // 높이만 사용하면 leading + line_spacing 이 누락되어 다음 문단이
                // 그림 바로 아래에 붙음. max(pic_h) 는 LINE_SEG 가 비정상적으로
                // 작은 경우의 안전장치.
                // [Task #1151 v9 결함 D] 가로 분배 시퀀스의 중간 picture 는 result_y
                // 진행 안 함 (y_offset 유지). 시퀀스 마지막 picture 또는 단일 picture
                // 에서만 advance — 다음 paragraph 가 가로 분배 영역 아래로 진행.
                if !has_full_para_item {
                    let line_advance = para
                        .line_segs
                        .first()
                        .map(|ls| hwpunit_to_px(ls.line_height + ls.line_spacing, self.dpi))
                        .unwrap_or(pic_h);
                    if is_single_pic || is_last_in_seq {
                        // 시퀀스 마지막: state 의 line_height (시퀀스 최대 height) 기반 advance
                        let line_top_y = para_inline_state
                            .get(&para_index)
                            .map(|s| s.line_top_y)
                            .unwrap_or(pic_y);
                        let line_height = para_inline_state
                            .get(&para_index)
                            .map(|s| s.line_height)
                            .unwrap_or(pic_h);
                        result_y = line_top_y + line_advance.max(line_height);
                    }
                    // 중간 picture: result_y = y_offset (그대로 유지, line 4527 의 default)
                }
            }
        } else if !has_real_text && !has_full_para_item {
            // [Task #418/#376] paragraph_layout 가 이미 emit 함 — push 스킵, result_y 만 갱신
            // [Task #462] 동일하게 LINE_SEG 기반 advance 사용
            // [Task #1151 v9 결함 D] 가로 분배 시퀀스의 중간 picture 는 result_y
            // 진행 안 함 (y_offset 유지). 시퀀스 마지막 picture 또는 단일 picture
            // 에서만 advance — 다음 paragraph 가 가로 분배 영역 아래로 진행.
            let line_advance = para
                .line_segs
                .first()
                .map(|ls| hwpunit_to_px(ls.line_height + ls.line_spacing, self.dpi))
                .unwrap_or(pic_h);
            if is_single_pic || is_last_in_seq {
                // 시퀀스 마지막: state 의 line_height (시퀀스 최대 height) 기반 advance
                let line_top_y = para_inline_state
                    .get(&para_index)
                    .map(|s| s.line_top_y)
                    .unwrap_or(pic_y);
                let line_height = para_inline_state
                    .get(&para_index)
                    .map(|s| s.line_height)
                    .unwrap_or(pic_h);
                result_y = line_top_y + line_advance.max(line_height);
            }
            // 중간 picture: result_y = y_offset (그대로 유지, line 4527 의 default)
        }

        let mut pic_content_bottom = effective_pic_y + effective_pic_h;
        if let Some(ref caption) = pic.caption {
            use crate::model::shape::CaptionDirection;
            let caption_spacing = hwpunit_to_px(caption.spacing as i32, self.dpi);
            let caption_h = self.calculate_caption_height(&pic.caption, styles);
            // 캡션은 실제로 그려진 그림 상자 바로 아래에 붙는다.
            // `effective_pic_y` 는 emit 된 ImageNode 의 상단이므로 상자 바닥은
            // 그 높이를 더한 값이다. 종전에는 상단에 `max(baseline, 높이)` 를
            // 더했는데, 저장 줄이 그림+캡션을 통째로 예약해 baseline 이 그림보다
            // 큰 줄에서는 (baseline − 그림 높이)만큼 캡션이 내려가 줄을 넘고,
            // 그 아래 내용이 전부 밀렸다 (156489219 5쪽: 캡션 +25.5pt, 뒤따르는
            // 표·그림 +17.9pt). 보통 줄은 baseline < 그림 높이라 두 식이 같다.
            let image_bottom = effective_pic_y + effective_pic_h;
            let cap_y = match caption.direction {
                CaptionDirection::Bottom => image_bottom + caption_spacing,
                CaptionDirection::Top => effective_pic_y,
                _ => image_bottom + caption_spacing,
            };
            if caption.direction == CaptionDirection::Top {
                let dy = caption_h + caption_spacing;
                Self::offset_inline_image_y(col_node, para_index, control_index, dy);
            }
            let cell_ctx = CellContext {
                in_textbox: false,
                parent_para_index: para_index,
                path: vec![CellPathEntry {
                    control_index,
                    cell_index: 0,
                    cell_para_index: 0,
                    text_direction: 0,
                }],
            };
            self.layout_caption(
                tree,
                col_node,
                caption,
                styles,
                col_area,
                pic_x,
                pic_w,
                cap_y,
                &mut self.auto_counter.borrow_mut(),
                bin_data_content,
                Some(cell_ctx),
                CaptionOwner::new(
                    Some(page_content.section_index),
                    Some(para_index),
                    Some(control_index),
                    CaptionControlKind::Image,
                ),
            );
            // [Task #864 Stage F] caption 이 차지한 영역까지 result_y 진행.
            // 미진행 시 다음 paragraph 가 caption 위에 그려져 겹침
            // (HWP3 sample14 page 4 "Visual Block을 이용한 대소문자 변경"
            // 가 본문 "먼저 원하는 구간을..." 와 겹침). Bottom 만 진행 (Top
            // 은 위에서 offset_inline_image_y 로 image 전체를 밀어서 처리).
            //
            // [Task #957] 빈 caption (text 없음 + controls 없음) 은 SVG 에 invisible.
            // pic_y = para_start_y[para_idx] 가 has_prior_tac 로 인해 후속 위치로
            // 갱신되면 image_bottom = pic_y + pic_h 가 페이지 바깥 위치로 계산되어
            // result_y 가 phantom +caption_h 만큼 누적 → 후속 paragraph 가 다음
            // 페이지로 밀림 (sample16 page 18 pi=394 ci=1 "그림" 의 empty caption
            // 으로 +430.6px advance). 빈 caption 은 advance skip.
            let caption_is_empty = caption.paragraphs.iter().all(|p| {
                p.text.chars().all(|c| c <= '\u{001F}' || c == '\u{FFFC}') && p.controls.is_empty()
            });
            if !caption_is_empty && matches!(caption.direction, CaptionDirection::Bottom) {
                let cap_bottom = cap_y + caption_h;
                if cap_bottom > result_y {
                    result_y = cap_bottom;
                }
                pic_content_bottom = pic_content_bottom.max(cap_bottom);
            }
        }
        let prev_bottom = self.last_item_content_bottom.get();
        self.last_item_content_bottom
            .set(if prev_bottom.is_finite() {
                prev_bottom.max(pic_content_bottom)
            } else {
                pic_content_bottom
            });
        result_y
    }
}
