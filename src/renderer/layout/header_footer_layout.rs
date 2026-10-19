//! 머리말·꼬리말 문단과 혼합 컨트롤의 배치.

use super::*;

impl LayoutEngine {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_header_footer_paragraphs(
        &self,
        tree: &mut PageLayoutContext,
        area_node: &mut RenderNode,
        hf_paragraphs: &[Paragraph],
        _composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        area: &LayoutRect,
        table_area: Option<&LayoutRect>,
        page_index: u32,
        page_number: u32,
        bin_data_content: &[BinDataContent],
        outer_section_index: Option<usize>,
        outer_hf_ref: Option<crate::renderer::render_tree::HeaderFooterImageRef>,
        is_header: bool,
        list_attr: u32,
        band_height_hu: u32,
    ) {
        // 머리말·꼬리말의 문단 장식은 본문/부모 셀의 수집 큐와 다른 소유다.
        let parent_border_ranges = std::mem::take(&mut *self.para_border_ranges.borrow_mut());
        let parent_border_override = self.border_box_override.replace(None);
        let paragraph_background_index = area_node.children.len();
        // [#6186] 머리말/꼬리말 subList 의 `vertAlign`(HWPX) = LIST_HEADER list_attr
        // 비트 21~22 (0=TOP, 1=CENTER, 2=BOTTOM). 파서는 이미 값을 싣는데
        // 레이아웃이 읽지 않아 늘 밴드 맨 위에 놓였다 — 156755659 는 BOTTOM 이라
        // 쪽번호가 21.8px 위에 붙어, 같은 자리에 겹쳐 놓인 글상자와 두 줄로 갈렸다.
        // 내용 높이를 **줄 높이 합으로 정확히 알 수 있을 때만** 정렬한다 — 표·도형·
        // 그림이 든 꼬리말은 이 합이 과소평가라 과잉 이동한다(exam_kor: 꼬리말에
        // 상자가 들어 있어 17.97px 더 내려가 한글과 멀어졌다 — 한글 A4 실측 쪽 하단
        // 82.1px 위 = A3 환산 116.2, 종전 118.5 가 맞고 변경값 100.6 은 틀림).
        // 쪽번호 같은 인라인 필드는 줄 안에서 자리를 차지하므로 줄 높이에 이미 들어
        // 있다 — 배제 대상은 **자기 높이를 갖는 개체**(표·도형·그림)뿐이다.
        let text_only_footer = !hf_paragraphs.is_empty()
            && hf_paragraphs.iter().all(|para| {
                !para.line_segs.is_empty()
                    && !para.controls.iter().any(|c| {
                        matches!(
                            c,
                            Control::Table(_) | Control::Shape(_) | Control::Picture(_)
                        )
                    })
            });
        let vert_align = if text_only_footer {
            (list_attr >> 21) & 0b11
        } else {
            0
        };
        let content_h: f64 = hf_paragraphs
            .iter()
            .filter_map(|para| para.line_segs.iter().map(|seg| seg.line_height).max())
            .map(|lh| hwpunit_to_px(lh, self.dpi))
            .sum();
        // 정렬 기준은 **문서가 선언한 밴드 높이**(HWPX subList `textHeight`)다.
        // 공유 `layout.footer_area` 는 아래 여백까지 품고 있고(그 rect 는 쪽 계산에도
        // 쓰여 건드리면 쪽수가 흔들린다 — issue_1733 등 8핀 실측), 한글의 세로 정렬은
        // 꼬리말 밴드 안에서만 일어난다. 선언값이 없으면(HWP5 등) 종전대로 area 전체.
        //
        // **선언값이 없으면(HWP5 등) 정렬을 적용하지 않는다.** `area.height` 로 물러서면
        // 밴드가 아래 여백까지 품고 있어 과잉 이동한다 — exam_kor(HWP5, A3 렌더)에서
        // 꼬리말 상자가 17.97px 더 내려가 한글과 멀어졌다(한글 A4 기준 쪽 하단에서
        // 82.1px 위 = A3 환산 116.2px; 종전 118.5px 가 맞고 변경 후 100.6px 는 틀림).
        // HWPX subList 가 `textHeight` 로 밴드를 명시할 때만 그 안에서 정렬한다.
        let slack = if band_height_hu > 0 {
            let band_h = hwpunit_to_px(band_height_hu as i32, self.dpi).min(area.height);
            (band_h - content_h).max(0.0)
        } else {
            0.0
        };
        // HWP5의 그림 혼합 footer는 저장 LINE_SEG가 footer 안 줄 grid를 완전히
        // 표현한다. `BOTTOM` subList에서 첫 줄의 vpos는 0이지만, 한컴은 첫 줄의
        // line_spacing을 footer 원점 뒤의 선행 여백으로 쓴다. 그림을 포함했다는
        // 이유만으로 text-only 정렬을 피하던 기존 경로는 이 1회 선행 여백도 버려
        // rule·로고·AutoNumber를 함께 위로 올렸다(#7333, 452 HU = 6.03px).
        //
        // 임의의 stale vpos를 원점으로 삼지 않는다. 각 문단이 정확히 한 저장 줄을
        // 가지고 vpos가 직전 줄의 height+spacing만큼 진행하며, 전체 grid가 선언한
        // subList 높이 안에 닫힐 때만 적용한다.
        let stored_mixed_footer_leading = if !text_only_footer {
            self.bottom_aligned_footer_grid_leading_px(hf_paragraphs, list_attr, band_height_hu)
        } else {
            0.0
        };
        let mut y_offset = area.y
            + match vert_align {
                1 => slack / 2.0,
                2 => slack,
                _ => 0.0,
            }
            + stored_mixed_footer_leading;
        for (i, para) in hf_paragraphs.iter().enumerate() {
            let paragraph_origin_y = y_offset;
            // 테이블 컨트롤이 있으면 테이블 렌더링
            let has_table = para.controls.iter().any(|c| matches!(c, Control::Table(_)));
            let has_shape = para.controls.iter().any(|c| matches!(c, Control::Shape(_)));
            let has_picture = para
                .controls
                .iter()
                .any(|c| matches!(c, Control::Picture(_)));
            if has_table {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Table(t) = ctrl {
                        let alignment = styles
                            .para_styles
                            .get(para.para_shape_id as usize)
                            .map(|s| s.alignment)
                            .unwrap_or(Alignment::Left);
                        // Task #445: 꼬리말 영역의 wrap=TopAndBottom + vert=Para 표는
                        // 첫 라인의 line_height/2 만큼 아래로 anchor 됨 (HWP 가 line center
                        // 기준으로 표를 배치하는 동작과 일치). 이 보정이 없으면 페이지 번호
                        // 박스가 본문 바닥과 붙어 보이는 문제(Task #445) 발생.
                        // [Issue #924] 머릿말에서는 적용하지 않음 — 표가 header_area 안에 정확히 위치해야 함.
                        // 꼬리말은 Task #445에서 필요하므로 유지.
                        let line_anchor_offset = if !is_header
                            && matches!(
                                t.common.text_wrap,
                                crate::model::shape::TextWrap::TopAndBottom
                            )
                            && matches!(t.common.vert_rel_to, crate::model::shape::VertRelTo::Para)
                            && i == 0
                        {
                            let lh_hu = para
                                .line_segs
                                .first()
                                .map(|ls| ls.line_height as i32)
                                .unwrap_or(0);
                            hwpunit_to_px(lh_hu, self.dpi) / 2.0
                        } else {
                            0.0
                        };
                        let table_y = y_offset + line_anchor_offset;
                        let table_area = table_area.unwrap_or(area);
                        y_offset = self.layout_table(
                            tree,
                            area_node,
                            t.as_ref(),
                            0,
                            styles,
                            0,
                            table_area,
                            table_y,
                            bin_data_content,
                            None,
                            0,
                            Some((i, ci)),
                            alignment,
                            None,
                            0.0,
                            0.0,
                            None,
                            None,
                            None,
                            None,
                            false,
                            is_header,
                            0.0,
                            None,
                            Self::standalone_table_char_border_fill(Some(para), t.as_ref(), styles),
                        );
                    }
                }
            } else if has_picture {
                // Picture 컨트롤이 있는 문단
                let comp = self.compose_header_footer_paragraph(para, page_number, styles);
                if comp.tac_controls.is_empty() {
                    // 글자처럼 취급하지 않는 그림과 쪽번호 AutoNumber가 같은 꼬리말
                    // 문단에 공존할 수 있다. 그림만 개별 배치하고 끝내면 AutoNumber가
                    // 조합 텍스트에 남아 있어도 전혀 그려지지 않는다(#7333).
                    // 그림의 y 누적은 다음 그림 배치용이므로, 인라인 필드는 원래
                    // 문단 기준선(text_y)에서 따로 레이아웃한다.
                    let text_y = y_offset;
                    // 머리말/꼬리말 내 Picture: header/footer area 기준 배치
                    for (ci, ctrl) in para.controls.iter().enumerate() {
                        if let Control::Picture(pic) = ctrl {
                            if pic.common.treat_as_char {
                                let pic_container = LayoutRect {
                                    x: area.x,
                                    y: y_offset,
                                    width: area.width,
                                    height: area.height - (y_offset - area.y),
                                };
                                // [Task #825] inner para_index = i (hf_paragraphs 안 인덱스),
                                // inner control_index = ci. outer 위치는 outer_hf_ref 보존.
                                self.layout_picture_full(
                                    tree,
                                    area_node,
                                    pic,
                                    &pic_container,
                                    bin_data_content,
                                    Alignment::Left,
                                    outer_section_index,
                                    Some(i),
                                    Some(ci),
                                    outer_hf_ref.clone(),
                                    None, // [Task #1151 v4] cell_ctx: 머리말/꼬리말 path
                                    styles,
                                );
                            } else {
                                // `Paper`/`Page` 기준은 header/footer 틀을 원점으로
                                // 다시 해석하므로 para_y의 저장 grid 선행을 읽지 않는다.
                                // 완전한 BOTTOM HWP5 grid로 확인한 경우에만 그 틀 원점도
                                // 첫 줄 leading만큼 이동한다. `Para`/`Column` 그림은 이미
                                // y_offset을 쓰므로 여기서 area를 움직이면 두 번 더해진다.
                                let picture_area = if !is_header
                                    && stored_mixed_footer_leading > 0.0
                                    && matches!(
                                        pic.common.vert_rel_to,
                                        VertRelTo::Paper | VertRelTo::Page
                                    ) {
                                    LayoutRect {
                                        y: area.y + stored_mixed_footer_leading,
                                        ..*area
                                    }
                                } else {
                                    *area
                                };
                                self.layout_header_footer_picture(
                                    tree,
                                    area_node,
                                    pic,
                                    &picture_area,
                                    y_offset,
                                    bin_data_content,
                                    outer_section_index,
                                    i,
                                    ci,
                                    outer_hf_ref.clone(),
                                    styles,
                                );
                            }
                            let pic_h = hwpunit_to_px(pic.common.height as i32, self.dpi);
                            y_offset += pic_h;
                        }
                    }
                    let has_page_auto_number = para.controls.iter().any(|ctrl| {
                        matches!(
                            ctrl,
                            Control::AutoNumber(number)
                                if matches!(
                                    number.number_type,
                                    crate::model::control::AutoNumberType::Page
                                        | crate::model::control::AutoNumberType::TotalPage
                                )
                        )
                    });
                    if has_page_auto_number {
                        let text_end = self.layout_paragraph(
                            tree,
                            area_node,
                            para,
                            Some(&comp),
                            styles,
                            area,
                            text_y,
                            0,
                            usize::MAX - i,
                            None,
                            None,
                            None,
                        );
                        y_offset = y_offset.max(text_end);
                    }
                } else {
                    // TAC Picture: layout_paragraph에서 인라인 배치
                    y_offset = self.layout_paragraph(
                        tree,
                        area_node,
                        para,
                        Some(&comp),
                        styles,
                        area,
                        y_offset,
                        0,
                        usize::MAX - i,
                        None,
                        Some(bin_data_content),
                        None, // 머리말/꼬리말 컨텍스트 — wrap zone 무관
                    );
                }
            } else if has_shape {
                // [#5802] 글자처럼 취급(TAC) 도형은 paragraph_layout 이 inline 좌표를
                // 등록해야 layout_shape 가 그린다 — 미등록 TAC 는 #476 가드가 조용히
                // 스킵한다. 머리말/꼬리말 문단은 종전에 layout_shape 만 불러, 쪽번호
                // 묶음처럼 TAC 도형으로 저장된 내용이 통째로 소실됐다(611쪽 보고서
                // 꼬리말 전량). 본문 등록 키와 충돌하지 않게 구역 인덱스는 HF 전용
                // 센티널(usize::MAX)을 등록·조회 양쪽에 쓴다.
                let has_tac_shape = para
                    .controls
                    .iter()
                    .any(|c| matches!(c, Control::Shape(s) if s.common().treat_as_char));
                let hf_shape_section = usize::MAX;
                if has_tac_shape {
                    let comp = self.compose_header_footer_paragraph(para, page_number, styles);
                    self.layout_paragraph(
                        tree,
                        area_node,
                        para,
                        Some(&comp),
                        styles,
                        area,
                        y_offset,
                        hf_shape_section,
                        i,
                        None,
                        Some(bin_data_content),
                        None,
                    );
                }
                // Shape 컨트롤 렌더링 (머리말/꼬리말 내 글상자 등)
                // 머리말·꼬리말의 Paper/Page 개체도 이 틀 안에서 오프셋을 잰다.
                // 그림의 배치 계약과 같으며, 본문의 물리 용지 원점을 전달하면
                // Paper-Left/Right 글상자가 본문 여백 밖으로 이동한다.
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    if let Control::Shape(_) = ctrl {
                        self.layout_shape(
                            tree,
                            area_node,
                            hf_paragraphs,
                            i,
                            ci,
                            if has_tac_shape { hf_shape_section } else { 0 },
                            styles,
                            area,
                            area,
                            area,
                            y_offset,
                            Alignment::Left,
                            bin_data_content,
                            &std::collections::HashMap::new(),
                            is_header,
                        );
                    }
                }
                // 텍스트도 함께 렌더링 (TAC 도형 경로는 위에서 이미 문단을 레이아웃함)
                if !has_tac_shape && !para.text.is_empty() {
                    let comp = self.compose_header_footer_paragraph(para, page_number, styles);
                    y_offset = self.layout_paragraph(
                        tree,
                        area_node,
                        para,
                        Some(&comp),
                        styles,
                        area,
                        y_offset,
                        0,
                        usize::MAX - i,
                        None,
                        None,
                        None, // 머리말/꼬리말 컨텍스트 — wrap zone 무관
                    );
                }
            } else {
                // 일반 텍스트 문단 레이아웃 (필드 마커 치환 포함)
                let comp = self.compose_header_footer_paragraph(para, page_number, styles);
                y_offset = self.layout_paragraph(
                    tree,
                    area_node,
                    para,
                    Some(&comp),
                    styles,
                    area,
                    y_offset,
                    0,
                    usize::MAX - i,
                    None,
                    None,
                    None, // 머리말/꼬리말 컨텍스트 — wrap zone 무관
                );
            }
            // 표/그림과 함께 있는 부동 도형도 같은 머리말·꼬리말 문단의 컨트롤이다.
            // 표 전진 뒤의 커서를 새 앵커로 쓰지 않고 원래 문단 원점을 소비한다.
            if has_shape && (has_table || has_picture) {
                for (ci, control) in para.controls.iter().enumerate() {
                    if matches!(control, Control::Shape(shape) if !shape.common().treat_as_char) {
                        self.layout_shape(
                            tree,
                            area_node,
                            hf_paragraphs,
                            i,
                            ci,
                            0,
                            styles,
                            area,
                            area,
                            area,
                            paragraph_origin_y,
                            Alignment::Left,
                            bin_data_content,
                            &std::collections::HashMap::new(),
                            is_header,
                        );
                    }
                }
            }
            if y_offset >= area.y + area.height {
                break;
            }
        }
        let local_composed: Vec<_> = hf_paragraphs
            .iter()
            .map(|para| self.compose_header_footer_paragraph(para, page_number, styles))
            .collect();
        // 이미지 편집 주소의 센티널은 유지하고 장식 연결의 문단 번호만 로컬화한다.
        for range in self.para_border_ranges.borrow_mut().iter_mut() {
            if let Some(index) = usize::MAX
                .checked_sub(range.9)
                .filter(|index| *index < hf_paragraphs.len())
            {
                range.9 = index;
            }
        }
        self.render_para_border_groups(
            tree,
            &local_composed,
            area_node,
            styles,
            area,
            bin_data_content,
            paragraph_background_index,
        );
        *self.para_border_ranges.borrow_mut() = parent_border_ranges;
        self.border_box_override.set(parent_border_override);
    }
}
