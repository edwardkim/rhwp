//! 어울림 표 옆 호스트의 실제 글줄 배치와 흐름 끝점을 함께 전달한다.

use super::*;

impl LayoutEngine {
    pub(super) fn layout_wrap_around_paras(
        &self,
        tree: &mut PageLayoutContext,
        col_node: &mut RenderNode,
        paragraphs: &[Paragraph],
        composed: &[ComposedParagraph],
        styles: &ResolvedStyleSet,
        col_area: &LayoutRect,
        section_index: usize,
        table_para_index: usize,
        wrap_around_paras: &[crate::renderer::pagination::WrapAroundPara],
        table_y_start: f64,
        table_y_end: f64,
        wrap_text_x: f64,
        wrap_text_width: f64,
        // [Task #1745] 후속 어울림 문단(WrapAroundPara)이 놓일 wrap 띠. 표 단독 anchor
        // 는 wrap_text_x/width 와 동일. 텍스트 혼합 anchor 는 표 geometry 로 도출한
        // 띠(text_anchor_square_table_strip) — host 영역(전폭)과 분리된다.
        strip_x: f64,
        strip_width: f64,
        // [Task #1745] 호스트 문단 텍스트 렌더 여부. 분할 표의 텍스트 혼합 anchor 는
        // 호스트 텍스트가 이미 일반 경로(분할 표 첫 부분)에서 렌더되므로 false 로
        // 이중 렌더를 막는다.
        render_host_text: bool,
        table_content_offset: f64,
        bin_data_content: &[BinDataContent],
        // Task #463: 인라인 floating 표(예: 인용 따옴표 ｢｣)의 우측 끝 x 좌표.
        // wrap host paragraph 의 외곽선이 이 표 위치까지 둘러싸도록 box 너비를
        // 확장하기 위해 caller 에서 계산하여 전달한다. None 이면 box 미확장.
        tbl_x_right: Option<f64>,
    ) -> Option<f64> {
        // 이 표에 연관된 어울림 문단만 필터링
        let related: Vec<_> = wrap_around_paras
            .iter()
            .filter(|wp| wp.table_para_index == table_para_index)
            .collect();

        let table_para = paragraphs.get(table_para_index)?;
        // 표 문단의 LINE_SEG에서 기준 vertical_pos — 후속 어울림 문단 배치에만 쓴다.
        // 저장 줄이 없는 host 도 자기 글자는 아래에서 그려야 하므로 여기서 끝내지 않는다.
        let table_base_vpos = table_para.line_segs.first().map(|s| s.vertical_pos);

        // 어울림 텍스트 영역
        // Task #463: wrap_text_x 는 LINE_SEG.column_start 기반으로 paragraph
        // margin_left 를 이미 포함하지만, layout_composed_paragraph 가 col_area.x 에
        // margin_left 를 한 번 더 더하기 때문에 wrap host 텍스트가 한 단계 더
        // 들여쓰기 됨 (학생3 wrap host 가 학생1 보다 +margin_left 만큼 우측으로 밀림).
        // wrap_area.x 를 margin_left 만큼 좌측으로 보정하고 width 도 그만큼 확장.
        // (inner_pad 는 외곽선 안쪽 여백으로 wrap_cs 와 무관하므로 보정 대상 아님)
        let host_para_style = composed
            .get(table_para_index)
            .and_then(|c| styles.para_styles.get(c.para_style_id as usize));
        let host_margin_left = host_para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let host_margin_right = host_para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let wrap_area = LayoutRect {
            x: wrap_text_x - host_margin_left,
            y: col_area.y,
            width: wrap_text_width + host_margin_left + host_margin_right,
            height: col_area.height,
        };
        // [Task #1745] 후속 어울림 문단용 띠 영역 (표 단독 anchor 는 wrap_area 와 동일).
        let strip_area = LayoutRect {
            x: strip_x - host_margin_left,
            y: col_area.y,
            width: strip_width + host_margin_left + host_margin_right,
            height: col_area.height,
        };

        let mut host_flow_end = None;
        // 호스트 문단(표 문단) 텍스트를 어울림 영역에 렌더링
        let has_host_text = table_para
            .text
            .chars()
            .any(|c| c > '\u{001F}' && c != '\u{FFFC}');
        if table_content_offset == 0.0 && render_host_text {
            if has_host_text {
                if let Some(comp) = composed.get(table_para_index) {
                    let text_start_line = comp.lines.iter().position(|line| {
                        line.runs
                            .iter()
                            .any(|r| r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}'))
                    });
                    if let Some(start_line) = text_start_line {
                        // 호스트 본문의 모든 텍스트 줄을 wrap 영역에 렌더링
                        // (Task #295: 자가 wrap host의 다중 줄 누락 수정)
                        let text_end_line = comp
                            .lines
                            .iter()
                            .rposition(|line| {
                                line.runs.iter().any(|r| {
                                    r.text.chars().any(|c| c > '\u{001F}' && c != '\u{FFFC}')
                                })
                            })
                            .map(|i| i + 1)
                            .unwrap_or(comp.lines.len());
                        // Task #463: wrap host 의 외곽선은 원래 col_area 너비로 그려야
                        // 인라인 floating 표(인용 따옴표 ｢｣ 등)를 박스가 둘러쌈. tbl_x_right
                        // 가 col_area 우측을 넘으면 그 위치까지 박스 너비를 확장한다.
                        let prev_override = self.border_box_override.get();
                        let extended_width = match tbl_x_right {
                            Some(tx) if tx > col_area.x + col_area.width => tx - col_area.x,
                            _ => col_area.width,
                        };
                        self.border_box_override
                            .set(Some((col_area.x, extended_width)));
                        host_flow_end = Some(self.layout_partial_paragraph(
                            tree,
                            col_node,
                            table_para,
                            Some(comp),
                            styles,
                            styles.hwp3_variant
                                && self.endnote_para_source_for(table_para_index).is_none(),
                            &wrap_area,
                            table_y_start,
                            start_line,
                            text_end_line,
                            section_index,
                            table_para_index,
                            None,
                            Some(bin_data_content),
                            None, // 표 호스트 어울림 문단 — 별도 wrap_anchor 메커니즘
                            Some(comp),
                        ));
                        self.border_box_override.set(prev_override);
                        // 어울림 문단은 항상 ↵ 표시 필요 — 부분 렌더링 시 is_para_end 강제 설정
                        force_para_end_on_last_run(col_node);
                    }
                }
            } else {
                // 호스트 문단에 텍스트 없음 (빈 문단 + 표): ↵ 마크 렌더링
                let seg = table_para.line_segs.first();
                let line_height = seg
                    .map(|s| crate::renderer::hwpunit_to_px(s.line_height, self.dpi))
                    .unwrap_or(13.3);
                let font_size = seg
                    .map(|s| crate::renderer::hwpunit_to_px(s.line_height, self.dpi))
                    .unwrap_or(13.3);
                let baseline = font_size * 0.8;
                let line_id = tree.next_id();
                let line_node = RenderNode::new(
                    line_id,
                    RenderNodeType::TextLine(TextLineNode::new(line_height, font_size)),
                    BoundingBox::new(wrap_text_x, table_y_start, font_size, line_height),
                );
                let run_id = tree.next_id();
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: String::new(),
                        style: TextStyle {
                            font_family: "바탕".to_string(),
                            font_size,
                            color: 0x000000,
                            ..Default::default()
                        },
                        char_shape_id: None,
                        para_shape_id: None,
                        section_index: None,
                        para_index: Some(table_para_index),
                        char_start: None,
                        cell_context: None,
                        is_para_end: true,
                        is_line_break_end: false,
                        rotation: 0.0,
                        is_vertical: false,
                        char_overlap: None,
                        border_fill_id: 0,
                        baseline,
                        field_marker: FieldMarkerType::None,
                        layout_positions: None,
                        display_text: None,
                    }),
                    BoundingBox::new(wrap_text_x, table_y_start, 0.0, line_height),
                );
                let mut line_container = line_node;
                line_container.children.push(run_node);
                col_node.children.push(line_container);
            }
        }

        if related.is_empty() {
            return host_flow_end;
        }
        let Some(table_base_vpos) = table_base_vpos else {
            return host_flow_end;
        };

        // 어울림 텍스트 영역: col_area를 cs/sw 기반으로 조정
        let wrap_area = LayoutRect {
            x: wrap_text_x,
            y: col_area.y,
            width: wrap_text_width,
            height: col_area.height,
        };

        for wp in &related {
            let para = match paragraphs.get(wp.para_index) {
                Some(p) => p,
                None => continue,
            };
            let seg = match para.line_segs.first() {
                Some(s) => s,
                None => continue,
            };
            // 어울림 문단의 표 내 vpos 오프셋 → px
            let vpos_offset = seg.vertical_pos - table_base_vpos;
            let abs_y_in_table = crate::renderer::hwpunit_to_px(vpos_offset, self.dpi);

            // 현재 페이지에서의 y
            let para_y = table_y_start + (abs_y_in_table - table_content_offset);

            // 명시적 prefix 컷은 typeset이 이 단에 소유시킨 텍스트다. 마지막
            // prefix 줄이 실제 표 밴드 바로 아래에 있어도 버리면 suffix가 그 줄을
            // 다시 그리지 않아 내용이 누락된다. 전체/후속 fragment의 범위 필터는
            // 유지하고, 첫 fragment가 소유하는 유한 prefix만 배치한다.
            let owns_prefix =
                wp.has_text && wp.end_line != usize::MAX && table_content_offset == 0.0;
            if para_y < table_y_start - 1.0 || (!owns_prefix && para_y >= table_y_end) {
                continue;
            }

            if wp.has_text {
                // 텍스트 문단: composed paragraph를 사용하여 어울림 영역에 렌더링
                let comp = composed.get(wp.para_index);
                // 표 옆 띠에 기록된 줄 구간만 렌더링한다. 일반 어울림 문단의
                // `end_line=usize::MAX`는 전체 줄이라는 기존 의미를 유지한다.
                let line_count = comp.map(|c| c.lines.len()).unwrap_or(1);
                let start_line = wp.start_line.min(line_count);
                let end_line = wp.end_line.min(line_count).max(start_line);
                if start_line == end_line {
                    continue;
                }
                self.layout_partial_paragraph(
                    tree,
                    col_node,
                    para,
                    comp,
                    styles,
                    styles.hwp3_variant && self.endnote_para_source_for(wp.para_index).is_none(),
                    &strip_area,
                    para_y,
                    start_line,
                    end_line,
                    section_index,
                    wp.para_index,
                    None,
                    Some(bin_data_content),
                    None, // 표 호스트 어울림 문단 — 별도 wrap_anchor 메커니즘
                    None,
                );
                // 어울림 문단은 항상 ↵ 표시 필요
                force_para_end_on_last_run(col_node);
            } else {
                // 빈 리턴 문단: ↵ 마크 렌더링
                let line_height = crate::renderer::hwpunit_to_px(seg.line_height, self.dpi);
                // 문단의 글자 모양에서 실제 폰트 크기 추출
                let font_size = {
                    let cs_id = para
                        .char_shapes
                        .first()
                        .map(|cs| cs.char_shape_id)
                        .unwrap_or(0);
                    styles
                        .char_styles
                        .get(cs_id as usize)
                        .map(|cs| cs.font_size)
                        .filter(|fs| *fs > 0.0)
                        .unwrap_or(13.3)
                };
                let mark_x = strip_x;

                let line_id = tree.next_id();
                let line_node = RenderNode::new(
                    line_id,
                    RenderNodeType::TextLine(TextLineNode::new(line_height, font_size)),
                    BoundingBox::new(mark_x, para_y, font_size, line_height),
                );

                let run_id = tree.next_id();
                let baseline = font_size * 0.8;
                let run_node = RenderNode::new(
                    run_id,
                    RenderNodeType::TextRun(TextRunNode {
                        text: String::new(),
                        style: TextStyle {
                            font_family: "바탕".to_string(),
                            font_size,
                            color: 0x000000,
                            ..Default::default()
                        },
                        char_shape_id: None,
                        para_shape_id: None,
                        section_index: None,
                        para_index: Some(wp.para_index),
                        char_start: None,
                        cell_context: None,
                        is_para_end: true,
                        is_line_break_end: false,
                        rotation: 0.0,
                        is_vertical: false,
                        char_overlap: None,
                        border_fill_id: 0,
                        baseline,
                        field_marker: FieldMarkerType::None,
                        layout_positions: None,
                        display_text: None,
                    }),
                    BoundingBox::new(mark_x, para_y, 0.0, line_height),
                );

                let mut line_container = line_node;
                line_container.children.push(run_node);
                col_node.children.push(line_container);
            }
        }
        host_flow_end
    }
}
