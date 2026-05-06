//! 커서 좌표/히트테스트/셀 커서/경로 기반 조작 관련 native 메서드

use crate::model::control::Control;
use crate::model::paragraph::Paragraph;
use crate::model::path::PathSegment;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use super::super::helpers::{LineInfoResult, utf16_pos_to_char_idx, color_ref_to_css, has_table_control, find_char_at_x, navigable_text_len};
use crate::renderer::render_tree::TextRunNode;

/// PUA 다자리 글자겹침 TextRun의 논리적 char_count (1) 반환, 아니면 실제 글자 수 반환
fn effective_char_count(text_run: &TextRunNode) -> usize {
    if text_run.char_overlap.is_some() {
        let chars: Vec<char> = text_run.text.chars().collect();
        if crate::renderer::composer::decode_pua_overlap_number(&chars).is_some() {
            return 1;
        }
    }
    text_run.text.chars().count()
}

impl DocumentCore {
    pub fn get_cursor_rect_native(
        &self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::renderer::layout::compute_char_positions;

        // 문단이 포함된 페이지 찾기
        let pages = self.find_pages_for_paragraph(section_idx, para_idx)?;

        // 커서 결과를 담을 구조체
        struct CursorHit {
            page_index: u32,
            x: f64,
            y: f64,
            height: f64,
        }

        // 렌더 트리에서 커서 위치를 찾는 재귀 함수
        // exact_only: true이면 정확한 매칭(zero-width 앵커)만 반환
        fn find_cursor_in_node(
            node: &RenderNode,
            sec: usize,
            para: usize,
            offset: usize,
            page_index: u32,
            exact_only: bool,
        ) -> Option<CursorHit> {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                // 번호/글머리표 TextRun (char_start: None)은 건너뛴다
                if let Some(char_start) = text_run.char_start {
                if text_run.section_index == Some(sec)
                    && text_run.para_index == Some(para)
                    && text_run.cell_context.is_none()
                {
                    let char_count = effective_char_count(text_run);

                    // 커서가 이 TextRun 범위 안에 있는지 확인
                    // char_start <= offset <= char_start + char_count
                    if offset >= char_start && offset <= char_start + char_count {
                        // exact_only 모드: zero-width 앵커(bbox.width==0)만 허용
                        if exact_only && !(char_count == 0 && offset == char_start && node.bbox.width == 0.0) {
                            // skip: 이 TextRun은 경계 매칭일 뿐 정확한 앵커가 아님
                        } else {
                        let local_offset = offset - char_start;
                        // PUA 다자리 글자겹침: 커서 위치는 [0.0, bbox.width]
                        let positions = if text_run.char_overlap.is_some() && char_count == 1 {
                            vec![0.0, node.bbox.width]
                        } else {
                            compute_char_positions(&text_run.text, &text_run.style)
                        };
                        let x_in_run = if local_offset < positions.len() {
                            positions[local_offset]
                        } else if !positions.is_empty() {
                            *positions.last().unwrap()
                        } else {
                            0.0
                        };
                        // 베이스라인 기반 캐럿 y 계산:
                        // 같은 줄에 서로 다른 글꼴 크기가 혼재할 때
                        // 각 글자의 ascent 위치에서 캐럿이 시작되어야 함
                        let font_size = text_run.style.font_size;
                        let ascent = font_size * 0.8;
                        let caret_y = node.bbox.y + text_run.baseline - ascent;
                        return Some(CursorHit {
                            page_index,
                            x: node.bbox.x + x_in_run,
                            y: caret_y,
                            height: font_size,
                        });
                        }
                    }
                }
                } // if let Some(char_start)

                // 도형 조판부호 마커 (char_start=None, ShapeMarker(pos))
                if let crate::renderer::render_tree::FieldMarkerType::ShapeMarker(marker_pos) = text_run.field_marker {
                    if text_run.section_index == Some(sec)
                        && text_run.para_index == Some(para)
                        && text_run.cell_context.is_none()
                    {
                        let font_size = text_run.style.font_size;
                        let ascent = font_size * 0.8;
                        let caret_y = node.bbox.y + text_run.baseline - ascent;
                        if marker_pos == offset {
                            // 마커 왼쪽 (마커 앞)
                            return Some(CursorHit {
                                page_index,
                                x: node.bbox.x,
                                y: caret_y,
                                height: font_size.max(10.0),
                            });
                        }
                        if marker_pos + 1 == offset {
                            // 마커 오른쪽 (마커 뒤)
                            return Some(CursorHit {
                                page_index,
                                x: node.bbox.x + node.bbox.width,
                                y: caret_y,
                                height: font_size.max(10.0),
                            });
                        }
                    }
                }
            }
            for child in &node.children {
                if let Some(hit) = find_cursor_in_node(child, sec, para, offset, page_index, exact_only) {
                    return Some(hit);
                }
            }
            None
        }

        // 후보 페이지를 순회하며 커서 위치 탐색
        // 1차: 정확한 앵커(zero-width 노드) 우선 검색, 2차: 일반 검색
        for &page_num in &pages {
            let tree = self.build_page_tree(page_num)?;
            let exact_hit = find_cursor_in_node(&tree.root, section_idx, para_idx, char_offset, page_num, true);
            let hit_result = exact_hit.or_else(|| find_cursor_in_node(&tree.root, section_idx, para_idx, char_offset, page_num, false));
            if let Some(hit) = hit_result {
                return Ok(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    hit.page_index, hit.x, hit.y, hit.height
                ));
            }
        }

        // 조판부호 감추기 모드: 인라인 도형 컨트롤 위치에서 커서 좌표 반환
        // treat_as_char Shape는 inline_shape_positions에서 좌표를 가져와 커서 표시
        if !self.show_control_codes {
            if let Some(section) = self.document.sections.get(section_idx) {
                if let Some(para) = section.paragraphs.get(para_idx) {
                    let text_len = para.text.chars().count();
                    let ctrl_positions = crate::document_core::find_control_text_positions(para);

                    // char_offset 위치에 인라인 컨트롤이 있는지 확인
                    let inline_ctrl = para.controls.iter().enumerate().find(|(ci, ctrl)| {
                        matches!(ctrl, Control::Shape(_) | Control::Picture(_) | Control::Equation(_))
                        && ctrl_positions.get(*ci).copied() == Some(char_offset)
                        && char_offset != text_len
                    });
                    // 텍스트 범위 밖이지만 navigable 범위 내 (도형이 텍스트 뒤에 있을 때)
                    let beyond_ctrl = if char_offset > text_len && char_offset <= navigable_text_len(para) {
                        para.controls.iter().enumerate().find(|(ci, ctrl)| {
                            matches!(ctrl, Control::Shape(_) | Control::Picture(_) | Control::Equation(_))
                            && ctrl_positions.get(*ci).copied() == Some(char_offset)
                        })
                    } else {
                        None
                    };

                    if let Some((ci, _ctrl)) = inline_ctrl.or(beyond_ctrl) {
                        // inline_shape_positions에서 Shape 좌표 조회
                        let first_page = pages[0];
                        let tree = self.build_page_tree(first_page)?;
                        if let Some((sx, sy)) = tree.get_inline_shape_position(section_idx, para_idx, ci, None) {
                            let shape_h = if let Some(Control::Shape(s)) = para.controls.get(ci) {
                                crate::renderer::hwpunit_to_px(s.common().height as i32, crate::renderer::DEFAULT_DPI)
                            } else if let Some(Control::Picture(p)) = para.controls.get(ci) {
                                crate::renderer::hwpunit_to_px(p.common.height as i32, crate::renderer::DEFAULT_DPI)
                            } else {
                                16.0
                            };
                            return Ok(format!(
                                "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                                first_page, sx, sy, shape_h
                            ));
                        }
                    }
                }
            }
        }

        // TextRun에서 찾지 못한 경우 (빈 문단 등): 첫 페이지에서 문단 위치 추정
        let first_page = pages[0];
        let tree = self.build_page_tree(first_page)?;

        // 해당 문단의 첫 TextRun 또는 TextLine 노드를 찾아 y/height 반환
        fn find_para_line(node: &RenderNode, sec: usize, para: usize) -> Option<(f64, f64, f64)> {
            // TextRun 매칭 (일반 문단, 번호/글머리표 TextRun 제외)
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if text_run.section_index == Some(sec)
                    && text_run.para_index == Some(para)
                    && text_run.cell_context.is_none()
                    && text_run.char_start.is_some()
                {
                    return Some((node.bbox.x, node.bbox.y, node.bbox.height));
                }
            }
            // TextLine 매칭 (빈 문단 — TextRun이 없을 때 폴백)
            if let RenderNodeType::TextLine(ref line) = node.node_type {
                if line.section_index == Some(sec) && line.para_index == Some(para) {
                    return Some((node.bbox.x, node.bbox.y, node.bbox.height));
                }
            }
            for child in &node.children {
                if let Some(r) = find_para_line(child, sec, para) {
                    return Some(r);
                }
            }
            None
        }

        if let Some((x, y, h)) = find_para_line(&tree.root, section_idx, para_idx) {
            // 인라인 도형 컨트롤이 있는 경우: char_offset에 따라 x 위치 조정
            let adjusted_x = if char_offset > 0 {
                // 해당 문단의 인라인 Shape/Picture/Table 노드 bbox를 수집
                fn collect_inline_bboxes(
                    node: &RenderNode, sec: usize, para: usize, bboxes: &mut Vec<(f64, f64)>,
                ) {
                    match &node.node_type {
                        RenderNodeType::Line(ln) if ln.section_index == Some(sec) && ln.para_index == Some(para) => {
                            bboxes.push((node.bbox.x, node.bbox.x + node.bbox.width));
                        }
                        RenderNodeType::Rectangle(rn) if rn.section_index == Some(sec) && rn.para_index == Some(para) => {
                            bboxes.push((node.bbox.x, node.bbox.x + node.bbox.width));
                        }
                        RenderNodeType::Ellipse(en) if en.section_index == Some(sec) && en.para_index == Some(para) => {
                            bboxes.push((node.bbox.x, node.bbox.x + node.bbox.width));
                        }
                        RenderNodeType::Table(tn) if tn.section_index == Some(sec) && tn.para_index == Some(para) => {
                            bboxes.push((node.bbox.x, node.bbox.x + node.bbox.width));
                        }
                        RenderNodeType::Image(im) if im.section_index == Some(sec) && im.para_index == Some(para) => {
                            bboxes.push((node.bbox.x, node.bbox.x + node.bbox.width));
                        }
                        _ => {}
                    }
                    for child in &node.children {
                        collect_inline_bboxes(child, sec, para, bboxes);
                    }
                }
                let mut bboxes = Vec::new();
                collect_inline_bboxes(&tree.root, section_idx, para_idx, &mut bboxes);
                bboxes.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

                if char_offset <= bboxes.len() && !bboxes.is_empty() {
                    if char_offset >= bboxes.len() {
                        // 마지막 도형 뒤
                        bboxes.last().map_or(x, |b| b.1)
                    } else {
                        // char_offset번째 도형의 왼쪽
                        bboxes[char_offset].0
                    }
                } else {
                    x
                }
            } else {
                x
            };

            return Ok(format!(
                "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                first_page, adjusted_x, y, h
            ));
        }

        Err(HwpError::RenderError(format!(
            "커서 위치를 찾을 수 없습니다: sec={}, para={}, offset={}",
            section_idx, para_idx, char_offset
        )))
    }

    /// 페이지 좌표에서 문서 위치 찾기 (네이티브)
    pub fn hit_test_native(&self, page_num: u32, x: f64, y: f64) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::renderer::layout::{compute_char_positions, CellContext};

        let tree = self.build_page_tree_cached(page_num)?;

        // 문자 위치를 미리 계산한 TextRun 정보
        struct RunInfo {
            section_index: usize,
            paragraph_index: usize,
            char_start: usize,
            char_count: usize,
            char_positions: Vec<f64>,
            bbox_x: f64,
            bbox_y: f64,
            bbox_w: f64,
            bbox_h: f64,
            // 셀/글상자 컨텍스트 (본문 텍스트는 None)
            cell_context: Option<CellContext>,
            is_textbox: bool,
            // 소속 칼럼 인덱스 (다단 지원)
            column_index: Option<u16>,
        }

        /// 안내문(guide text) TextRun 정보 (char_start: None)
        struct GuideRunInfo {
            section_index: usize,
            paragraph_index: usize,
            bbox_x: f64,
            bbox_y: f64,
            bbox_w: f64,
            bbox_h: f64,
            cell_context: Option<CellContext>,
        }

        /// 셀 bbox 정보
        struct CellBboxInfo {
            section_index: usize,
            parent_para_index: usize,
            control_index: usize,
            cell_index: usize,
            x: f64,
            y: f64,
            w: f64,
            h: f64,
            // Table 노드에서 meta가 채워졌는지 여부 (false이면 TextRun에서만 보완됨)
            has_meta: bool,
        }

        fn collect_runs(
            node: &RenderNode,
            runs: &mut Vec<RunInfo>,
            guide_runs: &mut Vec<GuideRunInfo>,
            cell_bboxes: &mut Vec<CellBboxInfo>,
            current_column: Option<u16>,
            // Table 노드에서 전파되는 (section_index, parent_para_index, control_index)
            current_table_meta: Option<(usize, usize, usize)>,
        ) {
            // Column 노드 진입 시 칼럼 인덱스 전파
            let col = if let RenderNodeType::Column(col_idx) = node.node_type {
                Some(col_idx)
            } else {
                current_column
            };
            // Table 노드 진입 시 section_index / parent_para_index / control_index 전파
            let table_meta = if let RenderNodeType::Table(ref tn) = node.node_type {
                match (tn.section_index, tn.para_index, tn.control_index) {
                    (Some(si), Some(pi), Some(ci)) => Some((si, pi, ci)),
                    _ => current_table_meta,
                }
            } else {
                current_table_meta
            };
            // TableCell 노드의 bbox 수집
            if let RenderNodeType::TableCell(ref tc) = node.node_type {
                if let Some(cell_idx) = tc.model_cell_index {
                    // table_meta가 있으면 즉시 보완, 없으면 자식 TextRun에서 보완
                    let (si, ppi, ci, has_meta) = table_meta
                        .map(|(si, ppi, ci)| (si, ppi, ci, true))
                        .unwrap_or((0, 0, 0, false));
                    cell_bboxes.push(CellBboxInfo {
                        section_index: si,
                        parent_para_index: ppi,
                        control_index: ci,
                        cell_index: cell_idx as usize,
                        x: node.bbox.x,
                        y: node.bbox.y,
                        w: node.bbox.width,
                        h: node.bbox.height,
                        has_meta,
                    });
                }
            }
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if let (Some(si), Some(pi)) = (text_run.section_index, text_run.para_index) {
                    // 머리말/꼬리말·각주 마커 TextRun 건너뛰기
                    if pi >= (usize::MAX - 3000) { /* skip marker runs */ }
                    else if let Some(cs) = text_run.char_start {
                        let ecc = effective_char_count(text_run);
                        let positions = if text_run.char_overlap.is_some() && ecc == 1 {
                            vec![0.0, node.bbox.width]
                        } else {
                            compute_char_positions(&text_run.text, &text_run.style)
                        };
                        runs.push(RunInfo {
                            section_index: si,
                            paragraph_index: pi,
                            char_start: cs,
                            char_count: ecc,
                            char_positions: positions,
                            bbox_x: node.bbox.x,
                            bbox_y: node.bbox.y,
                            bbox_w: node.bbox.width,
                            bbox_h: node.bbox.height,
                            cell_context: text_run.cell_context.clone(),
                            is_textbox: false,
                            column_index: col,
                        });
                    } else {
                        // char_start: None → 안내문 TextRun
                        guide_runs.push(GuideRunInfo {
                            section_index: si,
                            paragraph_index: pi,
                            bbox_x: node.bbox.x,
                            bbox_y: node.bbox.y,
                            bbox_w: node.bbox.width,
                            bbox_h: node.bbox.height,
                            cell_context: text_run.cell_context.clone(),
                        });
                    }
                }
            }
            for child in &node.children {
                collect_runs(child, runs, guide_runs, cell_bboxes, col, table_meta);
            }
        }

        fn format_hit(run: &RunInfo, offset: usize, page_num: u32) -> String {
            let base = format!(
                "\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{}",
                run.section_index, run.paragraph_index, offset
            );
            // 커서 x 좌표: char_positions로 정확한 위치 계산
            let cursor_x = if offset <= run.char_start {
                run.bbox_x
            } else {
                let local_idx = offset - run.char_start;
                if local_idx < run.char_positions.len() {
                    run.bbox_x + run.char_positions[local_idx]
                } else if !run.char_positions.is_empty() {
                    run.bbox_x + run.char_positions.last().copied().unwrap_or(0.0)
                } else {
                    run.bbox_x
                }
            };
            let cursor_rect = format!(
                ",\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                page_num, cursor_x, run.bbox_y, run.bbox_h
            );
            if let Some(ref ctx) = run.cell_context {
                let outer = &ctx.path[0];
                let tb = if run.is_textbox { ",\"isTextBox\":true" } else { "" };
                // cellPath: 전체 중첩 경로 배열
                let path_entries: Vec<String> = ctx.path.iter().map(|e| {
                    format!("{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}",
                        e.control_index, e.cell_index, e.cell_para_index)
                }).collect();
                let cell_path = format!(",\"cellPath\":[{}]", path_entries.join(","));
                format!("{{{},\"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}{}{}{}}}",
                    base, ctx.parent_para_index, outer.control_index, outer.cell_index, outer.cell_para_index,
                    cell_path, tb, cursor_rect)
            } else {
                format!("{{{}{}}}", base, cursor_rect)
            }
        }

        let mut runs: Vec<RunInfo> = Vec::new();
        let mut guide_runs: Vec<GuideRunInfo> = Vec::new();
        let mut cell_bboxes: Vec<CellBboxInfo> = Vec::new();
        collect_runs(&tree.root, &mut runs, &mut guide_runs, &mut cell_bboxes, None, None);

        // cell_bboxes의 section_index/parent_para_index/control_index를 runs로 재확인하여 보완
        // (Table 노드에서 이미 채워진 값이 있어도 runs에서 더 정확한 값을 덮어씀)
        for cb in &mut cell_bboxes {
            if let Some(run) = runs.iter().find(|r| {
                r.cell_context.as_ref().map(|ctx| ctx.path[0].cell_index == cb.cell_index).unwrap_or(false)
            }) {
                if let Some(ref ctx) = run.cell_context {
                    cb.section_index = run.section_index;
                    cb.parent_para_index = ctx.parent_para_index;
                    cb.control_index = ctx.path[0].control_index;
                    cb.has_meta = true;
                }
            }
        }

        // is_textbox 정확 판별: document의 실제 컨트롤 타입으로 재확인
        for run in &mut runs {
            if let Some(ref ctx) = run.cell_context {
                let outer = &ctx.path[0];
                if outer.cell_index == 0 {
                    let is_shape = self.document.sections.get(run.section_index)
                        .and_then(|s| s.paragraphs.get(ctx.parent_para_index))
                        .and_then(|p| p.controls.get(outer.control_index))
                        .map(|c| matches!(c, Control::Shape(_)))
                        .unwrap_or(false);
                    run.is_textbox = is_shape;
                } else {
                    run.is_textbox = false;
                }
            } else {
                run.is_textbox = false;
            }
        }

        // 0. 안내문(guide text) 히트 검사 — 필드 클릭 진입
        // 안내문 위 클릭 시 해당 필드의 시작 위치로 커서를 보낸다.
        for gr in &guide_runs {
            if x >= gr.bbox_x && x <= gr.bbox_x + gr.bbox_w
                && y >= gr.bbox_y && y <= gr.bbox_y + gr.bbox_h
            {
                // 필드 시작 위치 찾기: 해당 문단의 field_ranges에서 검색
                if let Some(field_hit) = self.find_field_hit_for_guide(
                    gr.section_index, gr.paragraph_index, &gr.cell_context, page_num,
                    gr.bbox_x, gr.bbox_y, gr.bbox_h,
                ) {
                    return Ok(field_hit);
                }
            }
        }

        if runs.is_empty() {
            // 텍스트가 없는 페이지: 첫 구역의 첫 문단 시작 반환
            let (page_content, _, _) = self.find_page(page_num)?;
            return Ok(format!(
                "{{\"sectionIndex\":{},\"paragraphIndex\":0,\"charOffset\":0}}",
                page_content.section_index
            ));
        }

        // 0.5. 인라인 Shape 히트 검사 (treat_as_char 도형 클릭)
        // inline_shape_positions에 등록된 Shape의 bbox를 검사하여
        // 클릭 시 해당 Shape의 텍스트 위치(char_offset)를 반환
        for (key, &(sx, sy)) in tree.inline_shape_positions() {
            let (si, pi, ci, ref cell_path) = *key;
            // 셀 내부 inline shape 은 cursor hit-test 에서 별도 처리 — 섹션 단위만 검사
            if !cell_path.is_empty() { continue; }
            if let Some(section) = self.document.sections.get(si) {
                if let Some(para) = section.paragraphs.get(pi) {
                    if let Some(ctrl) = para.controls.get(ci) {
                        let (sw, sh) = match ctrl {
                            Control::Shape(s) => (
                                crate::renderer::hwpunit_to_px(s.common().width as i32, crate::renderer::DEFAULT_DPI),
                                crate::renderer::hwpunit_to_px(s.common().height as i32, crate::renderer::DEFAULT_DPI),
                            ),
                            Control::Picture(p) => (
                                crate::renderer::hwpunit_to_px(p.common.width as i32, crate::renderer::DEFAULT_DPI),
                                crate::renderer::hwpunit_to_px(p.common.height as i32, crate::renderer::DEFAULT_DPI),
                            ),
                            _ => continue,
                        };
                        if x >= sx && x <= sx + sw && y >= sy && y <= sy + sh {
                            let ctrl_positions = crate::document_core::find_control_text_positions(para);
                            let char_offset = ctrl_positions.get(ci).copied().unwrap_or(0);
                            // 클릭이 Shape 오른쪽 절반이면 Shape 뒤(offset+1)
                            let offset = if x > sx + sw / 2.0 { char_offset + 1 } else { char_offset };
                            // 가장 가까운 TextRun을 찾아 format_hit 호출
                            let nearest = runs.iter().enumerate()
                                .filter(|(_, r)| r.section_index == si && r.paragraph_index == pi && r.cell_context.is_none())
                                .min_by_key(|(_, r)| {
                                    
                                    if offset >= r.char_start && offset <= r.char_start + r.char_count {
                                        0i64
                                    } else {
                                        (offset as i64 - r.char_start as i64).abs()
                                    }
                                });
                            if let Some((idx, _)) = nearest {
                                return Ok(format_hit(&runs[idx], offset, page_num));
                            }
                            // TextRun이 없으면 기본 반환
                            return Ok(format!(
                                "{{\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{},\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                                si, pi, offset, page_num, sx, sy, sh
                            ));
                        }
                    }
                }
            }
        }

        // 1. 정확한 bbox 히트 검사
        // 셀/글상자 TextRun을 본문 TextRun보다 우선한다.
        // (본문 TextRun이 컨트롤 높이만큼 큰 bbox를 가져서 글상자 영역을 덮을 수 있음)
        let mut hit_body: Option<(usize, usize)> = None;   // (run_idx, char_offset)
        let mut hit_cell: Option<(usize, usize)> = None;
        for (i, run) in runs.iter().enumerate() {
            if x >= run.bbox_x && x <= run.bbox_x + run.bbox_w
                && y >= run.bbox_y && y <= run.bbox_y + run.bbox_h
            {
                let local_x = x - run.bbox_x;
                let char_offset = find_char_at_x(&run.char_positions, local_x);
                if run.cell_context.is_some() {
                    if hit_cell.is_none() {
                        hit_cell = Some((i, run.char_start + char_offset));
                    }
                } else if hit_body.is_none() {
                    hit_body = Some((i, run.char_start + char_offset));
                }
            }
        }
        // 셀/글상자 히트가 있으면 우선, 없으면 본문 히트
        if let Some((idx, offset)) = hit_cell.or(hit_body) {
            return Ok(format_hit(&runs[idx], offset, page_num));
        }

        // 클릭 좌표가 속한 칼럼 결정 (다단 지원)
        let click_column = self.find_column_at_x(page_num, x);

        // 2. 셀 bbox 기반으로 클릭한 셀 판별
        let clicked_cell: Option<&CellBboxInfo> = cell_bboxes.iter()
            .find(|cb| x >= cb.x && x <= cb.x + cb.w && y >= cb.y && y <= cb.y + cb.h);

        // 셀 내부 클릭이면: 해당 셀의 run만 검색하여 가장 가까운 위치 반환
        if let Some(cb) = clicked_cell {
            let cell_runs: Vec<&RunInfo> = runs.iter()
                .filter(|r| {
                    r.cell_context.as_ref().map(|ctx| {
                        ctx.parent_para_index == cb.parent_para_index
                            && ctx.path[0].control_index == cb.control_index
                            && ctx.path[0].cell_index == cb.cell_index
                    }).unwrap_or(false)
                })
                .collect();

            if !cell_runs.is_empty() {
                // 같은 y 범위의 run 중 x가 가장 가까운 것
                let mut best = cell_runs[0];
                let mut best_offset = best.char_start;
                for r in &cell_runs {
                    if y >= r.bbox_y && y <= r.bbox_y + r.bbox_h {
                        if x < r.bbox_x {
                            // 텍스트 왼쪽 → 해당 run 시작
                            best = r;
                            best_offset = r.char_start;
                            break;
                        } else if x <= r.bbox_x + r.bbox_w {
                            // 텍스트 위 → 정확한 문자 위치
                            let local_x = x - r.bbox_x;
                            best = r;
                            best_offset = r.char_start + find_char_at_x(&r.char_positions, local_x);
                            break;
                        }
                        // 텍스트 오른쪽 → 이 run의 끝 (다음 run이 없으면 여기)
                        best = r;
                        best_offset = r.char_start + r.char_count;
                    }
                }
                // y 범위 매칭이 없으면 가장 가까운 run 사용
                if !cell_runs.iter().any(|r| y >= r.bbox_y && y <= r.bbox_y + r.bbox_h) {
                    let nearest = cell_runs.iter()
                        .min_by_key(|r| {
                            let mid_y = r.bbox_y + r.bbox_h / 2.0;
                            ((y - mid_y).abs() * 1000.0) as i64
                        });
                    if let Some(r) = nearest {
                        best = r;
                        best_offset = if x < r.bbox_x { r.char_start } else { r.char_start + r.char_count };
                    }
                }
                return Ok(format_hit(best, best_offset, page_num));
            }

            // 양식 컨트롤(FormObject)만 있는 셀: TextRun이 없어 cell_runs가 비어있음.
            // table_meta(또는 runs)에서 채워진 meta로 커서 진입.
            if cb.has_meta {
                return Ok(format!(
                    "{{\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":0,\
                     \"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":0,\
                     \"cellPath\":[{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":0}}],\
                     \"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                    cb.section_index, cb.parent_para_index,
                    cb.parent_para_index, cb.control_index, cb.cell_index,
                    cb.control_index, cb.cell_index,
                    page_num,
                    cb.x + 2.0, cb.y + 2.0, cb.h.max(4.0) - 4.0
                ));
            }
        }

        // 같은 줄(y 범위)에서 가장 가까운 본문 TextRun 찾기
        // 다단: 클릭 칼럼의 run만 필터
        let mut same_line_runs: Vec<&RunInfo> = runs.iter()
            .filter(|r| r.cell_context.is_none()) // 본문 run만
            .filter(|r| y >= r.bbox_y && y <= r.bbox_y + r.bbox_h)
            .filter(|r| click_column.is_none() || r.column_index.is_none() || r.column_index == click_column)
            .collect();

        if !same_line_runs.is_empty() {
            same_line_runs.sort_by(|a, b| a.bbox_x.partial_cmp(&b.bbox_x).unwrap());
            // 클릭이 줄의 왼쪽이면 첫 run 시작
            if x < same_line_runs[0].bbox_x {
                let run = same_line_runs[0];
                return Ok(format_hit(run, run.char_start, page_num));
            }
            // 줄의 오른쪽이면 마지막 run 끝
            let last = same_line_runs.last().unwrap();
            return Ok(format_hit(last, last.char_start + last.char_count, page_num));
        }

        // 3. 가장 가까운 줄 찾기 (y 거리 기준)
        // 다단: 클릭 칼럼의 run을 우선 후보로 사용
        let column_runs: Vec<&RunInfo> = runs.iter()
            .filter(|r| click_column.is_none() || r.column_index.is_none() || r.column_index == click_column)
            .collect();
        let candidate_runs = if column_runs.is_empty() { &runs.iter().collect::<Vec<_>>() } else { &column_runs };

        let closest = candidate_runs.iter().min_by(|a, b| {
            let dist_a = (y - (a.bbox_y + a.bbox_h / 2.0)).abs();
            let dist_b = (y - (b.bbox_y + b.bbox_h / 2.0)).abs();
            dist_a.partial_cmp(&dist_b).unwrap()
        }).unwrap();

        let target_y = closest.bbox_y;
        let target_h = closest.bbox_h;
        let mut line_runs: Vec<&&RunInfo> = candidate_runs.iter()
            .filter(|r| (r.bbox_y - target_y).abs() < 1.0 && (r.bbox_h - target_h).abs() < 1.0)
            .collect();
        line_runs.sort_by(|a, b| a.bbox_x.partial_cmp(&b.bbox_x).unwrap());

        if x < line_runs[0].bbox_x {
            let run = line_runs[0];
            return Ok(format_hit(run, run.char_start, page_num));
        }

        // x 좌표로 적합한 run 찾기
        for run in &line_runs {
            if x >= run.bbox_x && x <= run.bbox_x + run.bbox_w {
                let local_x = x - run.bbox_x;
                let char_offset = find_char_at_x(&run.char_positions, local_x);
                return Ok(format_hit(run, run.char_start + char_offset, page_num));
            }
        }

        // 줄의 오른쪽 끝
        let last = line_runs.last().unwrap();
        Ok(format_hit(last, last.char_start + last.char_count, page_num))
    }

    /// 안내문 클릭 시 필드 시작 위치를 찾아 hitTest 결과를 반환한다.
    fn find_field_hit_for_guide(
        &self,
        section_index: usize,
        paragraph_index: usize,
        cell_context: &Option<crate::renderer::layout::CellContext>,
        page_num: u32,
        guide_x: f64,
        guide_y: f64,
        guide_h: f64,
    ) -> Option<String> {
        use crate::model::control::{Control, FieldType};

        // 문단 접근: cell_context가 있으면 전체 경로를 따라가기 (중첩 표 지원)
        let para = if let Some(ctx) = cell_context {
            let path: Vec<(usize, usize, usize)> = ctx.path.iter()
                .map(|e| (e.control_index, e.cell_index, e.cell_para_index))
                .collect();
            self.resolve_paragraph_by_path(section_index, ctx.parent_para_index, &path).ok()?
        } else {
            self.document.sections.get(section_index)?
                .paragraphs.get(paragraph_index)?
        };

        // 이 문단의 ClickHere 필드 범위 검색
        for fr in &para.field_ranges {
            if let Some(Control::Field(field)) = para.controls.get(fr.control_idx) {
                if field.field_type == FieldType::ClickHere {
                    // 필드 시작 위치로 커서를 보낸다
                    let char_offset = fr.start_char_idx;
                    let base = format!(
                        "\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{}",
                        section_index, paragraph_index, char_offset,
                    );
                    let cursor_rect = format!(
                        ",\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                        page_num, guide_x, guide_y, guide_h,
                    );
                    let field_info = format!(
                        ",\"isField\":true,\"fieldId\":{},\"fieldType\":\"{}\"",
                        field.field_id, field.field_type_str(),
                    );
                    if let Some(ctx) = cell_context {
                        let outer = &ctx.path[0];
                        let tb = if matches!(
                            self.document.sections.get(section_index)
                                .and_then(|s| s.paragraphs.get(ctx.parent_para_index))
                                .and_then(|p| p.controls.get(outer.control_index)),
                            Some(Control::Shape(_))
                        ) { ",\"isTextBox\":true" } else { "" };
                        let path_entries: Vec<String> = ctx.path.iter().map(|e| {
                            format!("{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}",
                                e.control_index, e.cell_index, e.cell_para_index)
                        }).collect();
                        let cell_path = format!(",\"cellPath\":[{}]", path_entries.join(","));
                        return Some(format!(
                            "{{{},\"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}{}{}{}{}}}",
                            base, ctx.parent_para_index, outer.control_index,
                            outer.cell_index, outer.cell_para_index,
                            cell_path, tb, field_info, cursor_rect,
                        ));
                    } else {
                        return Some(format!("{{{}{}{}}}", base, field_info, cursor_rect));
                    }
                }
            }
        }

        None
    }

    /// 셀의 (col, row, pad_left_px, pad_top_px, pad_bottom_px)를 모델에서 조회한다.
    pub(crate) fn resolve_cell_position(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Result<(u16, u16, f64, f64, f64), HwpError> {
        use crate::model::control::Control;
        let para = self.document.sections.get(section_idx)
            .and_then(|s| s.paragraphs.get(parent_para_idx))
            .ok_or_else(|| HwpError::RenderError("문단 없음".to_string()))?;
        let ctrl = para.controls.get(control_idx)
            .ok_or_else(|| HwpError::RenderError("컨트롤 없음".to_string()))?;
        match ctrl {
            Control::Table(ref tbl) => {
                let cell = tbl.cells.get(cell_idx)
                    .ok_or_else(|| HwpError::RenderError("셀 없음".to_string()))?;
                let dpi_scale = 96.0 / 7200.0;
                Ok((
                    cell.col,
                    cell.row,
                    cell.padding.left as f64 * dpi_scale,
                    cell.padding.top as f64 * dpi_scale,
                    cell.padding.bottom as f64 * dpi_scale,
                ))
            }
            Control::Shape(_) | Control::Picture(_) => {
                // 글상자/그림 캡션은 패딩 없음
                Ok((0, 0, 0.0, 0.0, 0.0))
            }
            _ => Err(HwpError::RenderError("표 컨트롤이 아닙니다".to_string())),
        }
    }

    /// 표 셀 내부 커서의 픽셀 좌표를 반환한다 (네이티브)
    pub fn get_cursor_rect_in_cell_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::renderer::layout::compute_char_positions;

        // 표 캡션은 cell_index=65534 센티널로 렌더 트리에도 동일하게 저장됨

        // 테이블이 포함된 본문 문단의 페이지 찾기
        let pages = self.find_pages_for_paragraph(section_idx, parent_para_idx)?;

        struct CursorHit {
            page_index: u32,
            x: f64,
            y: f64,
            height: f64,
        }

        fn find_cursor_in_cell(
            node: &RenderNode,
            parent_para: usize,
            ctrl_idx: usize,
            c_idx: usize,
            cp_idx: usize,
            offset: usize,
            page_index: u32,
        ) -> Option<CursorHit> {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                let matches_cell = text_run.cell_context.as_ref().map_or(false, |ctx| {
                    ctx.parent_para_index == parent_para
                        && ctx.path[0].control_index == ctrl_idx
                        && ctx.path[0].cell_index == c_idx
                        && ctx.path[0].cell_para_index == cp_idx
                });
                if matches_cell {
                    let char_start = text_run.char_start.unwrap_or(0);
                    let char_count = effective_char_count(text_run);

                    if offset >= char_start && offset <= char_start + char_count {
                        let local_offset = offset - char_start;
                        let positions = if text_run.char_overlap.is_some() && char_count == 1 {
                            vec![0.0, node.bbox.width]
                        } else {
                            compute_char_positions(&text_run.text, &text_run.style)
                        };
                        let x_in_run = if local_offset < positions.len() {
                            positions[local_offset]
                        } else if !positions.is_empty() {
                            *positions.last().unwrap()
                        } else {
                            0.0
                        };
                        // 베이스라인 기반 캐럿 y 계산 (본문과 동일)
                        let font_size = text_run.style.font_size;
                        let ascent = font_size * 0.8;
                        let caret_y = node.bbox.y + text_run.baseline - ascent;
                        return Some(CursorHit {
                            page_index,
                            x: node.bbox.x + x_in_run,
                            y: caret_y,
                            height: font_size,
                        });
                    }
                }
            }
            for child in &node.children {
                if let Some(hit) = find_cursor_in_cell(child, parent_para, ctrl_idx, c_idx, cp_idx, offset, page_index) {
                    return Some(hit);
                }
            }
            None
        }

        for &page_num in &pages {
            let tree = self.build_page_tree(page_num)?;
            if let Some(hit) = find_cursor_in_cell(
                &tree.root, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset, page_num
            ) {
                return Ok(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    hit.page_index, hit.x, hit.y, hit.height
                ));
            }
        }

        // 빈 셀 fallback: 해당 셀의 아무 TextRun을 찾아 위치 반환
        let first_page = pages[0];
        let tree = self.build_page_tree(first_page)?;

        fn find_cell_run(
            node: &RenderNode,
            parent_para: usize,
            ctrl_idx: usize,
            c_idx: usize,
            cp_idx: usize,
        ) -> Option<(f64, f64, f64)> {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                let matches_cell = text_run.cell_context.as_ref().map_or(false, |ctx| {
                    ctx.parent_para_index == parent_para
                        && ctx.path[0].control_index == ctrl_idx
                        && ctx.path[0].cell_index == c_idx
                        && ctx.path[0].cell_para_index == cp_idx
                });
                if matches_cell {
                    return Some((node.bbox.x, node.bbox.y, node.bbox.height));
                }
            }
            for child in &node.children {
                if let Some(r) = find_cell_run(child, parent_para, ctrl_idx, c_idx, cp_idx) {
                    return Some(r);
                }
            }
            None
        }

        if let Some((x, y, h)) = find_cell_run(&tree.root, parent_para_idx, control_idx, cell_idx, cell_para_idx) {
            return Ok(format!(
                "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                first_page, x, y, h
            ));
        }

        // 빈 셀 최종 fallback: 모델에서 셀의 col/row를 조회한 뒤
        // 렌더 트리의 TableCell 노드 bbox + 패딩으로 커서 위치 산출
        let cell_pos = self.resolve_cell_position(section_idx, parent_para_idx, control_idx, cell_idx)?;

        fn find_table_cell_bbox(
            node: &RenderNode,
            parent_para: usize,
            ctrl_idx: usize,
            target_col: u16,
            target_row: u16,
        ) -> Option<(f64, f64, f64, f64)> {
            if let RenderNodeType::Table(ref tn) = node.node_type {
                let matches_table = tn.para_index == Some(parent_para)
                    && tn.control_index == Some(ctrl_idx);
                if matches_table {
                    for child in &node.children {
                        if let RenderNodeType::TableCell(ref tc) = child.node_type {
                            if tc.col == target_col && tc.row == target_row {
                                return Some((
                                    child.bbox.x, child.bbox.y,
                                    child.bbox.width, child.bbox.height,
                                ));
                            }
                        }
                    }
                }
            }
            for child in &node.children {
                if let Some(r) = find_table_cell_bbox(child, parent_para, ctrl_idx, target_col, target_row) {
                    return Some(r);
                }
            }
            None
        }

        if let Some((cx, cy, _cw, ch)) = find_table_cell_bbox(
            &tree.root, parent_para_idx, control_idx, cell_pos.0, cell_pos.1
        ) {
            // 셀 bbox 좌상단 + 패딩 위치에 커서 배치
            let pad_left = cell_pos.2;
            let pad_top = cell_pos.3;
            let caret_h = (ch - pad_top - cell_pos.4).max(10.0); // 패딩 제외한 높이
            return Ok(format!(
                "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                first_page, cx + pad_left, cy + pad_top, caret_h
            ));
        }

        Err(HwpError::RenderError(format!(
            "셀 커서 위치를 찾을 수 없습니다: sec={}, parentPara={}, ctrl={}, cell={}, cellPara={}, offset={}",
            section_idx, parent_para_idx, control_idx, cell_idx, cell_para_idx, char_offset
        )))
    }

    // ─── 컨테이너 렌더 범위 조회 ──────────────────────────────

    /// 지정된 컨테이너(글상자/표 셀) 내에서 실제로 렌더링된 마지막 문단 인덱스를 반환한다.
    /// 렌더 트리의 TextRun 노드 중 해당 컨테이너에 속한 것의 cell_para_index 최대값을 구한다.
    /// 렌더된 TextRun이 없으면 None을 반환한다.
    pub(crate) fn last_rendered_para_in_container(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
    ) -> Option<usize> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        fn max_para_in_node(
            node: &RenderNode,
            parent_para: usize,
            ctrl_idx: usize,
            c_idx: usize,
        ) -> Option<usize> {
            let mut result: Option<usize> = None;
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                if let Some(ref ctx) = tr.cell_context {
                    if ctx.parent_para_index == parent_para
                        && ctx.path[0].control_index == ctrl_idx
                        && ctx.path[0].cell_index == c_idx
                    {
                        let cp = ctx.path[0].cell_para_index;
                        result = Some(result.map_or(cp, |prev: usize| prev.max(cp)));
                    }
                }
            }
            for child in &node.children {
                if let Some(cp) = max_para_in_node(child, parent_para, ctrl_idx, c_idx) {
                    result = Some(result.map_or(cp, |prev: usize| prev.max(cp)));
                }
            }
            result
        }

        // 해당 문단이 포함된 페이지들에서 검색
        let pages = self.find_pages_for_paragraph(section_idx, parent_para_idx).ok()?;
        let mut max_para: Option<usize> = None;
        for &page_num in &pages {
            let tree = self.build_page_tree(page_num).ok()?;
            if let Some(cp) = max_para_in_node(&tree.root, parent_para_idx, control_idx, cell_idx) {
                max_para = Some(max_para.map_or(cp, |prev: usize| prev.max(cp)));
            }
        }
        max_para
    }

    // ─── 경로 기반 중첩 표 Native API ──────────────────────────

    /// 경로 기반 커서 좌표 조회 (네이티브).
    pub(crate) fn get_cursor_rect_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path_json: &str,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::renderer::layout::compute_char_positions;

        let path = Self::parse_cell_path(path_json)?;
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }

        let last = path.last().unwrap();
        let para = self.resolve_paragraph_by_path(section_idx, parent_para_idx, &path)?;

        // 커서 좌표를 렌더 트리에서 찾기
        let pages = self.find_pages_for_paragraph(section_idx, parent_para_idx)?;

        // 렌더 트리에서 경로가 일치하는 TextRun 찾기
        fn find_cursor_by_path(
            node: &RenderNode,
            parent_para: usize,
            path: &[(usize, usize, usize)],
            offset: usize,
            page: u32,
        ) -> Option<(u32, f64, f64, f64)> {
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                let matches = tr.cell_context.as_ref().map_or(false, |ctx| {
                    ctx.parent_para_index == parent_para
                        && ctx.path.len() == path.len()
                        && ctx.path.iter().zip(path.iter()).all(|(a, b)| {
                            a.control_index == b.0
                                && a.cell_index == b.1
                                && a.cell_para_index == b.2
                        })
                });
                if matches {
                    let cs = tr.char_start.unwrap_or(0);
                    let cc = tr.text.chars().count();
                    if offset >= cs && offset <= cs + cc {
                        let positions = compute_char_positions(&tr.text, &tr.style);
                        let lo = offset - cs;
                        let xr = if lo < positions.len() { positions[lo] }
                                 else if !positions.is_empty() { *positions.last().unwrap() }
                                 else { 0.0 };
                        return Some((page, node.bbox.x + xr, node.bbox.y, node.bbox.height));
                    }
                }
            }
            for child in &node.children {
                if let Some(hit) = find_cursor_by_path(child, parent_para, path, offset, page) {
                    return Some(hit);
                }
            }
            None
        }

        for &page_num in &pages {
            let tree = self.build_page_tree(page_num)?;
            if let Some((pi, x, y, h)) = find_cursor_by_path(&tree.root, parent_para_idx, &path, char_offset, page_num) {
                return Ok(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    pi, x, y, h
                ));
            }
        }

        // fallback: 아무 TextRun이라도 찾기
        for &page_num in &pages {
            let tree = self.build_page_tree(page_num)?;
            fn find_any_run(
                node: &RenderNode,
                parent_para: usize,
                path: &[(usize, usize, usize)],
                page: u32,
            ) -> Option<(u32, f64, f64, f64)> {
                if let RenderNodeType::TextRun(ref tr) = node.node_type {
                    let matches = tr.cell_context.as_ref().map_or(false, |ctx| {
                        ctx.parent_para_index == parent_para
                            && ctx.path.len() == path.len()
                            && ctx.path.iter().zip(path.iter()).all(|(a, b)| {
                                a.control_index == b.0
                                    && a.cell_index == b.1
                                    && a.cell_para_index == b.2
                            })
                    });
                    if matches {
                        return Some((page, node.bbox.x, node.bbox.y, node.bbox.height));
                    }
                }
                for child in &node.children {
                    if let Some(hit) = find_any_run(child, parent_para, path, page) {
                        return Some(hit);
                    }
                }
                None
            }
            if let Some((pi, x, y, h)) = find_any_run(&tree.root, parent_para_idx, &path, page_num) {
                return Ok(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    pi, x, y, h
                ));
            }
        }

        Err(HwpError::RenderError(format!(
            "경로 기반 커서 위치를 찾을 수 없습니다: sec={}, ppi={}, path={}, offset={}",
            section_idx, parent_para_idx, path_json, char_offset
        )))
    }

    /// 경로 기반 셀 정보 조회 (네이티브).
    pub(crate) fn get_cell_info_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path_json: &str,
    ) -> Result<String, HwpError> {
        let path = Self::parse_cell_path(path_json)?;
        let cell = self.resolve_cell_by_path(section_idx, parent_para_idx, &path)?;

        Ok(format!(
            "{{\"row\":{},\"col\":{},\"rowSpan\":{},\"colSpan\":{}}}",
            cell.row, cell.col, cell.row_span, cell.col_span
        ))
    }

    /// 경로 기반 표 차원 조회 (네이티브).
    pub(crate) fn get_table_dimensions_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path_json: &str,
    ) -> Result<String, HwpError> {
        let path = Self::parse_cell_path(path_json)?;
        let table = self.resolve_table_by_path(section_idx, parent_para_idx, &path)?;

        Ok(format!(
            "{{\"rowCount\":{},\"colCount\":{},\"cellCount\":{}}}",
            table.row_count, table.col_count, table.cells.len()
        ))
    }

    /// 경로 기반 표 셀 바운딩박스 조회 (네이티브).
    pub(crate) fn get_table_cell_bboxes_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path_json: &str,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};

        let path = Self::parse_cell_path(path_json)?;
        // 표가 존재하는지 검증
        let _table = self.resolve_table_by_path(section_idx, parent_para_idx, &path)?;

        // 렌더 트리에서 TextRun의 cell_context.path가 일치하는 TableCell의 부모 Table 노드를 찾는다
        fn find_nested_table_cells(
            node: &RenderNode,
            parent_para: usize,
            path: &[(usize, usize, usize)],
            page_idx: usize,
            result: &mut Vec<String>,
        ) -> bool {
            // Table 노드를 발견하면 자식 TableCell 중 TextRun의 cell_context가 경로와 일치하는지 확인
            if let RenderNodeType::Table(_) = node.node_type {
                // 이 테이블의 셀에서 TextRun을 찾아 경로 매칭 여부 확인
                fn check_table_match(
                    node: &RenderNode,
                    parent_para: usize,
                    path: &[(usize, usize, usize)],
                ) -> bool {
                    if let RenderNodeType::TextRun(ref tr) = node.node_type {
                        return tr.cell_context.as_ref().map_or(false, |ctx| {
                            ctx.parent_para_index == parent_para
                                && ctx.path.len() == path.len()
                                && ctx.path.iter().zip(path.iter()).enumerate().all(|(i, (a, b))| {
                                    if i < path.len() - 1 {
                                        // 중간 경로: 전체 매칭 (어떤 셀/문단을 경유하는지)
                                        a.control_index == b.0 && a.cell_index == b.1 && a.cell_para_index == b.2
                                    } else {
                                        // 마지막 경로: control_index만 매칭 (이 표의 모든 셀 포함)
                                        a.control_index == b.0
                                    }
                                })
                        });
                    }
                    for child in &node.children {
                        // 중첩 Table 노드는 건너뛴다 — find_nested_table_cells가 별도로 처리
                        if matches!(child.node_type, RenderNodeType::Table(_)) {
                            continue;
                        }
                        if check_table_match(child, parent_para, path) {
                            return true;
                        }
                    }
                    false
                }

                if check_table_match(node, parent_para, path) {
                    // 이 테이블의 직속 셀 bbox 수집
                    for (cell_idx, child) in node.children.iter().enumerate() {
                        if let RenderNodeType::TableCell(ref cn) = child.node_type {
                            result.push(format!(
                                "{{\"cellIdx\":{},\"row\":{},\"col\":{},\"rowSpan\":{},\"colSpan\":{},\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"w\":{:.1},\"h\":{:.1}}}",
                                cell_idx, cn.row, cn.col, cn.row_span, cn.col_span,
                                page_idx,
                                child.bbox.x, child.bbox.y, child.bbox.width, child.bbox.height
                            ));
                        }
                    }
                    return true;
                }
            }

            for child in &node.children {
                if find_nested_table_cells(child, parent_para, path, page_idx, result) {
                    return true;
                }
            }
            false
        }

        let mut cells = Vec::new();
        let total_pages = self.page_count() as usize;
        let mut found = false;
        for page_num in 0..total_pages {
            let tree = self.build_page_tree(page_num as u32)?;
            if find_nested_table_cells(&tree.root, parent_para_idx, &path, page_num, &mut cells) {
                found = true;
            } else if found {
                // 이전 페이지에서 표를 찾았으나 이 페이지에는 없음 → 표가 끝남
                break;
            }
        }

        if cells.is_empty() {
            return Err(HwpError::RenderError(format!(
                "경로 기반 표 셀 bbox를 찾을 수 없습니다: sec={}, ppi={}, path={}",
                section_idx, parent_para_idx, path_json
            )));
        }

        Ok(format!("[{}]", cells.join(",")))
    }

    /// 경로 기반 수직 커서 이동 (네이티브).
    pub(crate) fn move_vertical_by_path_native(
        &self,
        section_idx: usize,
        parent_para_idx: usize,
        path_json: &str,
        char_offset: usize,
        delta: i32,
        preferred_x: f64,
    ) -> Result<String, HwpError> {
        use crate::renderer::layout::compute_char_positions;

        let path = Self::parse_cell_path(path_json)?;
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }

        let cell = self.resolve_cell_by_path(section_idx, parent_para_idx, &path)?;
        let cell_para_count = cell.paragraphs.len();
        let current_para_idx = path.last().unwrap().2;  // cellParaIndex

        let para = cell.paragraphs.get(current_para_idx)
            .ok_or_else(|| HwpError::RenderError(format!("셀문단 {} 범위 초과", current_para_idx)))?;

        // preferredX 결정
        let actual_px = if preferred_x < 0.0 {
            match self.get_cursor_rect_by_path_native(section_idx, parent_para_idx, path_json, char_offset) {
                Ok(json) => super::super::helpers::json_f64(&json, "x").unwrap_or(0.0),
                Err(_) => 0.0,
            }
        } else {
            preferred_x
        };

        // 줄 정보 계산
        let line_info = Self::compute_line_info_struct(para, char_offset)
            .unwrap_or(LineInfoResult { line_index: 0, line_count: 1, char_start: 0, char_end: navigable_text_len(para) });
        let target_line = line_info.line_index as i32 + delta;

        // 결과: (new_path, new_char_offset)
        let (new_path, new_offset) =
        if target_line >= 0 && (target_line as usize) < line_info.line_count {
            // CASE A: 같은 문단 내 다른 줄 — preferredX 기반 오프셋 찾기
            let target_range = Self::get_line_char_range(para, target_line as usize);
            let best = self.find_best_offset_by_x_in_path(
                section_idx, parent_para_idx, &path, current_para_idx,
                target_range.0, target_range.1, actual_px,
            );
            let mut p = path.clone();
            p.last_mut().unwrap().2 = current_para_idx;
            (p, best)
        } else if delta < 0 && current_para_idx > 0 {
            // CASE B-1: 이전 문단 마지막 줄
            let prev_para = current_para_idx - 1;
            let prev = &cell.paragraphs[prev_para];
            let prev_line_count = Self::compute_line_info_struct(prev, 0)
                .map(|li| li.line_count).unwrap_or(1);
            let last_line = prev_line_count.saturating_sub(1);
            let target_range = Self::get_line_char_range(prev, last_line);
            let best = self.find_best_offset_by_x_in_path(
                section_idx, parent_para_idx, &path, prev_para,
                target_range.0, target_range.1, actual_px,
            );
            let mut p = path.clone();
            p.last_mut().unwrap().2 = prev_para;
            (p, best)
        } else if delta > 0 && current_para_idx + 1 < cell_para_count {
            // CASE B-2: 다음 문단 첫 줄
            let next_para = current_para_idx + 1;
            let next = &cell.paragraphs[next_para];
            let target_range = Self::get_line_char_range(next, 0);
            let best = self.find_best_offset_by_x_in_path(
                section_idx, parent_para_idx, &path, next_para,
                target_range.0, target_range.1, actual_px,
            );
            let mut p = path.clone();
            p.last_mut().unwrap().2 = next_para;
            (p, best)
        } else {
            // CASE C: 셀 경계 — 인접 셀 이동 시도
            let table = self.resolve_table_by_path(section_idx, parent_para_idx, &path)?;
            let last_entry = path.last().unwrap();
            let cell_idx = last_entry.1;
            let current_cell = &table.cells[cell_idx];

            let target_row = if delta > 0 {
                (current_cell.row + current_cell.row_span) as i32
            } else {
                current_cell.row as i32 - 1
            };

            if target_row >= 0 && (target_row as u16) < table.row_count {
                if let Some(target_cell_idx) = table.cell_index_at(target_row as u16, current_cell.col) {
                    // 인접 셀로 이동
                    let target_cell = &table.cells[target_cell_idx];
                    let (target_cpi, target_line_idx) = if delta > 0 {
                        (0usize, 0usize)
                    } else {
                        let last_cpi = target_cell.paragraphs.len().saturating_sub(1);
                        let last_line = target_cell.paragraphs.get(last_cpi)
                            .map(|p| if p.line_segs.is_empty() { 0 } else { p.line_segs.len() - 1 })
                            .unwrap_or(0);
                        (last_cpi, last_line)
                    };
                    let mut new_p = path.clone();
                    let last = new_p.last_mut().unwrap();
                    last.1 = target_cell_idx;   // cellIndex 갱신
                    last.2 = target_cpi;        // cellParaIndex 갱신

                    if let Some(target_para) = target_cell.paragraphs.get(target_cpi) {
                        let target_range = Self::get_line_char_range(target_para, target_line_idx);
                        let best = self.find_best_offset_by_x_in_path(
                            section_idx, parent_para_idx, &new_p, target_cpi,
                            target_range.0, target_range.1, actual_px,
                        );
                        (new_p, best)
                    } else {
                        (new_p, 0)
                    }
                } else {
                    // 해당 행/열에 셀 없음 — 현재 위치 유지
                    (path.clone(), char_offset)
                }
            } else {
                // CASE D: 중첩 표 경계 탈출 — 부모 셀의 다음/이전 문단으로
                if path.len() >= 2 {
                    // 부모 레벨 경로로 올라감
                    let mut parent_path = path[..path.len()-1].to_vec();
                    let parent_last = parent_path.last().unwrap();
                    let parent_cell = self.resolve_cell_by_path(section_idx, parent_para_idx, &parent_path)?;
                    let parent_cpi = parent_last.2;

                    if delta > 0 && parent_cpi + 1 < parent_cell.paragraphs.len() {
                        // 부모 셀의 다음 문단 첫 줄
                        let next_cpi = parent_cpi + 1;
                        let next_para = &parent_cell.paragraphs[next_cpi];
                        let target_range = Self::get_line_char_range(next_para, 0);
                        parent_path.last_mut().unwrap().2 = next_cpi;
                        let best = self.find_best_offset_by_x_in_path(
                            section_idx, parent_para_idx, &parent_path, next_cpi,
                            target_range.0, target_range.1, actual_px,
                        );
                        (parent_path, best)
                    } else if delta < 0 && parent_cpi > 0 {
                        // 부모 셀의 이전 문단 마지막 줄
                        let prev_cpi = parent_cpi - 1;
                        let prev_para = &parent_cell.paragraphs[prev_cpi];
                        let prev_line_count = Self::compute_line_info_struct(prev_para, 0)
                            .map(|li| li.line_count).unwrap_or(1);
                        let last_line = prev_line_count.saturating_sub(1);
                        let target_range = Self::get_line_char_range(prev_para, last_line);
                        parent_path.last_mut().unwrap().2 = prev_cpi;
                        let best = self.find_best_offset_by_x_in_path(
                            section_idx, parent_para_idx, &parent_path, prev_cpi,
                            target_range.0, target_range.1, actual_px,
                        );
                        (parent_path, best)
                    } else {
                        // 부모 셀 경계에서도 더 이상 이동 불가 — 현재 위치 유지
                        (path.clone(), char_offset)
                    }
                } else {
                    // depth=1 표 경계 — 현재 위치 유지 (본문 탈출은 flat API에서 처리)
                    (path.clone(), char_offset)
                }
            }
        };

        let new_para = new_path.last().unwrap().2;
        let path_json_out = Self::format_path_json(&new_path);

        // 커서 좌표 획득
        let (rect_valid, page_idx, fx, fy, fh) = match self.get_cursor_rect_by_path_native(
            section_idx, parent_para_idx, &path_json_out, new_offset,
        ) {
            Ok(json) => (
                true,
                super::super::helpers::json_f64(&json, "pageIndex").unwrap_or(0.0) as usize,
                super::super::helpers::json_f64(&json, "x").unwrap_or(0.0),
                super::super::helpers::json_f64(&json, "y").unwrap_or(0.0),
                super::super::helpers::json_f64(&json, "height").unwrap_or(18.0),
            ),
            Err(_) => (false, 0, 0.0, 0.0, 18.0),
        };

        // MoveVerticalResult 형식 (톱레벨 pageIndex/x/y/height)
        let rect_valid_str = if rect_valid { "" } else { ",\"rectValid\":false" };
        Ok(format!(
            "{{\"sectionIndex\":{},\"paragraphIndex\":{},\"charOffset\":{},\"parentParaIndex\":{},\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{},\"cellPath\":{},\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1},\"preferredX\":{:.1}{}}}",
            section_idx, new_para, new_offset,
            parent_para_idx, new_path[0].0, new_path[0].1, new_path[0].2,
            path_json_out,
            page_idx, fx, fy, fh, actual_px, rect_valid_str
        ))
    }

    /// 경로 기반 문단 내 지정 범위에서 preferredX에 가장 가까운 char offset을 찾는다.
    pub(crate) fn find_best_offset_by_x_in_path(
        &self,
        sec: usize,
        ppi: usize,
        path: &[(usize, usize, usize)],
        para_idx: usize,
        range_start: usize,
        range_end: usize,
        target_x: f64,
    ) -> usize {
        let mut best_offset = range_start;
        let mut best_dist = f64::MAX;

        // 경로에서 para_idx를 사용하는 새 경로 생성
        let mut test_path = path.to_vec();
        if let Some(last) = test_path.last_mut() {
            last.2 = para_idx;
        }
        let path_json = Self::format_path_json(&test_path);

        for offset in range_start..=range_end {
            if let Ok(json) = self.get_cursor_rect_by_path_native(sec, ppi, &path_json, offset) {
                if let Some(x) = super::super::helpers::json_f64(&json, "x") {
                    let dist = (x - target_x).abs();
                    if dist < best_dist {
                        best_dist = dist;
                        best_offset = offset;
                    }
                }
            }
        }
        best_offset
    }

    /// CellPath를 JSON 문자열로 포맷한다.
    pub(crate) fn format_path_json(path: &[(usize, usize, usize)]) -> String {
        let entries: Vec<String> = path.iter().map(|(ci, cei, cpi)| {
            format!("{{\"controlIndex\":{},\"cellIndex\":{},\"cellParaIndex\":{}}}", ci, cei, cpi)
        }).collect();
        format!("[{}]", entries.join(","))
    }

    // ─── Phase 2 Native 끝 ──────────────────────────────────

    /// 클릭 x 좌표가 속한 칼럼 인덱스를 반환한다 (다단 히트 테스트용).
    pub(crate) fn find_column_at_x(&self, page_num: u32, x: f64) -> Option<u16> {
        let (page_content, _, _) = self.find_page(page_num).ok()?;
        let areas = &page_content.layout.column_areas;
        if areas.len() <= 1 {
            return None; // 단일 단 — 칼럼 필터링 불필요
        }
        for (i, area) in areas.iter().enumerate() {
            if x >= area.x && x <= area.x + area.width {
                return Some(i as u16);
            }
        }
        // 칼럼 영역 사이(간격)에 클릭한 경우 가장 가까운 칼럼 반환
        areas.iter().enumerate()
            .min_by(|(_, a), (_, b)| {
                let da = (x - (a.x + a.width / 2.0)).abs();
                let db = (x - (b.x + b.width / 2.0)).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i as u16)
    }

    /// 머리말/꼬리말 내 커서 좌표를 반환한다.
    ///
    /// 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
    pub fn get_cursor_rect_in_header_footer_native(
        &self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
        char_offset: usize,
        preferred_page: i32,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::renderer::layout::compute_char_positions;

        // 머리말/꼬리말 문단의 para_index 마커 값
        // layout_header_footer_paragraphs에서 para_index = usize::MAX - i 로 설정됨
        let marker_para_idx = usize::MAX - hf_para_idx;

        // Header/Footer 노드 타입 판별
        let is_target_node = |nt: &RenderNodeType| -> bool {
            if is_header {
                matches!(nt, RenderNodeType::Header)
            } else {
                matches!(nt, RenderNodeType::Footer)
            }
        };

        struct CursorHit {
            page_index: u32,
            x: f64,
            y: f64,
            height: f64,
        }

        // Header/Footer 서브트리에서 TextRun 찾기
        fn find_cursor_in_hf_subtree(
            node: &RenderNode,
            marker_para: usize,
            offset: usize,
            page_index: u32,
        ) -> Option<CursorHit> {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if let Some(char_start) = text_run.char_start {
                    if text_run.para_index == Some(marker_para)
                        && text_run.cell_context.is_none()
                    {
                        let char_count = effective_char_count(text_run);
                        if offset >= char_start && offset <= char_start + char_count {
                            let local_offset = offset - char_start;
                            let positions = if text_run.char_overlap.is_some() && char_count == 1 {
                                vec![0.0, node.bbox.width]
                            } else {
                                compute_char_positions(&text_run.text, &text_run.style)
                            };
                            let x_in_run = if local_offset < positions.len() {
                                positions[local_offset]
                            } else if !positions.is_empty() {
                                *positions.last().unwrap()
                            } else {
                                0.0
                            };
                            let font_size = text_run.style.font_size;
                            let ascent = font_size * 0.8;
                            let caret_y = node.bbox.y + text_run.baseline - ascent;
                            return Some(CursorHit {
                                page_index,
                                x: node.bbox.x + x_in_run,
                                y: caret_y,
                                height: font_size,
                            });
                        }
                    }
                }
            }
            for child in &node.children {
                if let Some(hit) = find_cursor_in_hf_subtree(child, marker_para, offset, page_index) {
                    return Some(hit);
                }
            }
            None
        }

        // Header/Footer 노드에서 빈 문단 폴백
        fn find_hf_para_line(node: &RenderNode, marker_para: usize) -> Option<(f64, f64, f64)> {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if text_run.para_index == Some(marker_para)
                    && text_run.cell_context.is_none()
                    && text_run.char_start.is_some()
                {
                    return Some((node.bbox.x, node.bbox.y, node.bbox.height));
                }
            }
            if let RenderNodeType::TextLine(ref line) = node.node_type {
                if line.para_index == Some(marker_para) {
                    return Some((node.bbox.x, node.bbox.y, node.bbox.height));
                }
            }
            for child in &node.children {
                if let Some(r) = find_hf_para_line(child, marker_para) {
                    return Some(r);
                }
            }
            None
        }

        // preferred_page가 지정되면 해당 페이지를 먼저 탐색
        let total_pages = self.page_count();
        let page_order: Vec<u32> = if preferred_page >= 0 && (preferred_page as u32) < total_pages {
            let pref = preferred_page as u32;
            std::iter::once(pref)
                .chain((0..total_pages).filter(move |&p| p != pref))
                .collect()
        } else {
            (0..total_pages).collect()
        };
        for page_num in page_order {
            let tree = self.build_page_tree(page_num)?;
            // 루트의 자식에서 Header/Footer 노드 찾기
            for child in &tree.root.children {
                if is_target_node(&child.node_type) {
                    if let Some(hit) = find_cursor_in_hf_subtree(child, marker_para_idx, char_offset, page_num) {
                        return Ok(format!(
                            "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                            hit.page_index, hit.x, hit.y, hit.height
                        ));
                    }
                    // 빈 문단 폴백
                    if let Some((x, y, h)) = find_hf_para_line(child, marker_para_idx) {
                        return Ok(format!(
                            "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                            page_num, x, y, h
                        ));
                    }
                    // Header/Footer 노드는 있지만 TextRun이 없는 경우 — 영역 좌표 반환
                    return Ok(format!(
                        "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                        page_num, child.bbox.x, child.bbox.y,
                        if child.bbox.height > 0.0 { 12.0 } else { 12.0 }
                    ));
                }
            }
        }

        Err(HwpError::RenderError(format!(
            "머리말/꼬리말 커서 위치를 찾을 수 없습니다: sec={}, is_header={}, hf_para={}",
            section_idx, is_header, hf_para_idx
        )))
    }

    /// 머리말/꼬리말 영역의 히트테스트
    ///
    /// 페이지 좌표가 머리말 또는 꼬리말 영역에 해당하는지 판별.
    /// 반환: JSON `{"hit":true,"isHeader":bool,"sectionIndex":N,"applyTo":N}`
    /// 또는 `{"hit":false}`
    pub fn hit_test_header_footer_native(
        &self,
        page_num: u32,
        x: f64,
        y: f64,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::RenderNodeType;

        let tree = self.build_page_tree(page_num)?;

        for child in &tree.root.children {
            let is_header = matches!(child.node_type, RenderNodeType::Header);
            let is_footer = matches!(child.node_type, RenderNodeType::Footer);
            if !is_header && !is_footer { continue; }

            if x >= child.bbox.x && x <= child.bbox.x + child.bbox.width
                && y >= child.bbox.y && y <= child.bbox.y + child.bbox.height
            {
                // active header/footer에서 source_section_index와 apply_to 추출
                // 머리말/꼬리말은 이전 구역에서 상속될 수 있으므로
                // 페이지 소속 구역이 아닌 source_section_index를 반환해야 함
                if let Some((source_sec, apply_to)) = self.get_active_hf_info(page_num, is_header) {
                    return Ok(format!(
                        "{{\"hit\":true,\"isHeader\":{},\"sectionIndex\":{},\"applyTo\":{}}}",
                        is_header, source_sec, apply_to
                    ));
                }
                // active 정보가 없는 경우 fallback
                let (section_idx, _) = self.find_section_for_page(page_num);
                return Ok(format!(
                    "{{\"hit\":true,\"isHeader\":{},\"sectionIndex\":{},\"applyTo\":0}}",
                    is_header, section_idx
                ));
            }
        }

        Ok("{\"hit\":false}".to_string())
    }

    /// 페이지 번호로 구역 인덱스를 찾는다.
    fn find_section_for_page(&self, page_num: u32) -> (usize, usize) {
        let mut offset = 0u32;
        for (si, pr) in self.pagination.iter().enumerate() {
            let count = pr.pages.len() as u32;
            if page_num < offset + count {
                return (si, (page_num - offset) as usize);
            }
            offset += count;
        }
        (0, 0)
    }

    /// 해당 페이지에서 활성화된 머리말/꼬리말의 apply_to 값을 반환한다.
    fn get_active_hf_apply_to(&self, _section_idx: usize, page_num: u32, is_header: bool) -> u8 {
        self.get_active_hf_info(page_num, is_header)
            .map(|(_, apply_to)| apply_to)
            .unwrap_or(0)
    }

    /// 해당 페이지에서 활성화된 머리말/꼬리말의 (source_section_index, apply_to)를 반환한다.
    fn get_active_hf_info(&self, page_num: u32, is_header: bool) -> Option<(usize, u8)> {
        use crate::model::header_footer::HeaderFooterApply;

        let mut offset = 0u32;
        for (_si, pr) in self.pagination.iter().enumerate() {
            let count = pr.pages.len() as u32;
            if page_num < offset + count {
                let local_page = (page_num - offset) as usize;
                let page = &pr.pages[local_page];
                let hf_ref = if is_header { &page.active_header } else { &page.active_footer };
                if let Some(ref r) = hf_ref {
                    let source_sec = r.source_section_index;
                    if let Some(section) = self.document.sections.get(source_sec) {
                        if let Some(para) = section.paragraphs.get(r.para_index) {
                            if let Some(ctrl) = para.controls.get(r.control_index) {
                                let apply_to = match ctrl {
                                    Control::Header(h) => match h.apply_to {
                                        HeaderFooterApply::Both => 0,
                                        HeaderFooterApply::Even => 1,
                                        HeaderFooterApply::Odd => 2,
                                    },
                                    Control::Footer(f) => match f.apply_to {
                                        HeaderFooterApply::Both => 0,
                                        HeaderFooterApply::Even => 1,
                                        HeaderFooterApply::Odd => 2,
                                    },
                                    _ => 0,
                                };
                                return Some((source_sec, apply_to));
                            }
                        }
                    }
                }
                return None;
            }
            offset += count;
        }
        None
    }

    /// 머리말/꼬리말 내부 텍스트 히트테스트
    ///
    /// 편집 모드에서 클릭한 좌표가 어느 문단·문자 위치에 해당하는지 반환.
    /// 반환: JSON `{"hit":true,"paraIndex":N,"charOffset":N,"cursorRect":{...}}`
    /// 또는 `{"hit":false}`
    pub fn hit_test_in_header_footer_native(
        &self,
        page_num: u32,
        is_header: bool,
        x: f64,
        y: f64,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::renderer::layout::compute_char_positions;

        let tree = self.build_page_tree(page_num)?;

        // Header/Footer 서브트리 찾기
        let hf_node = tree.root.children.iter().find(|child| {
            if is_header {
                matches!(child.node_type, RenderNodeType::Header)
            } else {
                matches!(child.node_type, RenderNodeType::Footer)
            }
        });
        let hf_node = match hf_node {
            Some(n) => n,
            None => return Ok("{\"hit\":false}".to_string()),
        };

        // TextRun 정보 수집
        struct HfRunInfo {
            hf_para_idx: usize, // 머리말/꼬리말 내 문단 인덱스 (0, 1, 2, ...)
            char_start: usize,
            char_count: usize,
            char_positions: Vec<f64>,
            bbox_x: f64,
            bbox_y: f64,
            bbox_w: f64,
            bbox_h: f64,
            baseline: f64,
            font_size: f64,
        }

        fn collect_hf_runs(node: &RenderNode, runs: &mut Vec<HfRunInfo>) {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if let (Some(marker_para), Some(cs)) = (text_run.para_index, text_run.char_start) {
                    // marker_para = usize::MAX - hf_para_idx → 복원
                    if marker_para >= (usize::MAX - 1000) {
                        let hf_para_idx = usize::MAX - marker_para;
                        let positions = compute_char_positions(&text_run.text, &text_run.style);
                        runs.push(HfRunInfo {
                            hf_para_idx,
                            char_start: cs,
                            char_count: text_run.text.chars().count(),
                            char_positions: positions,
                            bbox_x: node.bbox.x,
                            bbox_y: node.bbox.y,
                            bbox_w: node.bbox.width,
                            bbox_h: node.bbox.height,
                            baseline: text_run.baseline,
                            font_size: text_run.style.font_size,
                        });
                    }
                }
            }
            for child in &node.children {
                collect_hf_runs(child, runs);
            }
        }

        let mut runs: Vec<HfRunInfo> = Vec::new();
        collect_hf_runs(hf_node, &mut runs);

        if runs.is_empty() {
            // TextRun이 없는 경우 — 빈 머리말/꼬리말
            return Ok(format!(
                "{{\"hit\":true,\"paraIndex\":0,\"charOffset\":0,\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                page_num, hf_node.bbox.x, hf_node.bbox.y, 12.0
            ));
        }

        // hitTest용 헬퍼: char_positions에서 x 좌표로 문자 오프셋 찾기
        fn find_char_at_x_hf(positions: &[f64], local_x: f64) -> usize {
            for (i, &px) in positions.iter().enumerate() {
                if i == 0 {
                    if local_x < px / 2.0 { return 0; }
                } else {
                    let mid = (positions[i - 1] + px) / 2.0;
                    if local_x < mid { return i; }
                }
            }
            positions.len()
        }

        fn format_hf_hit(run: &HfRunInfo, char_offset: usize, page_num: u32) -> String {
            let cursor_x = if char_offset <= run.char_start {
                run.bbox_x
            } else {
                let local_idx = char_offset - run.char_start;
                if local_idx < run.char_positions.len() {
                    run.bbox_x + run.char_positions[local_idx]
                } else if !run.char_positions.is_empty() {
                    run.bbox_x + run.char_positions.last().copied().unwrap_or(0.0)
                } else {
                    run.bbox_x
                }
            };
            let ascent = run.font_size * 0.8;
            let cursor_y = run.bbox_y + run.baseline - ascent;
            format!(
                "{{\"hit\":true,\"paraIndex\":{},\"charOffset\":{},\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                run.hf_para_idx, char_offset, page_num, cursor_x, cursor_y, run.font_size
            )
        }

        // 1단계: 정확한 bbox 히트
        for run in &runs {
            if x >= run.bbox_x && x <= run.bbox_x + run.bbox_w
                && y >= run.bbox_y && y <= run.bbox_y + run.bbox_h
            {
                let local_x = x - run.bbox_x;
                let char_offset = find_char_at_x_hf(&run.char_positions, local_x);
                return Ok(format_hf_hit(run, run.char_start + char_offset, page_num));
            }
        }

        // 2단계: 같은 줄(y 범위)에서 가장 가까운 run
        let same_line: Vec<&HfRunInfo> = runs.iter()
            .filter(|r| y >= r.bbox_y && y <= r.bbox_y + r.bbox_h)
            .collect();
        if !same_line.is_empty() {
            if x < same_line[0].bbox_x {
                let run = same_line[0];
                return Ok(format_hf_hit(run, run.char_start, page_num));
            }
            let last = same_line.last().unwrap();
            return Ok(format_hf_hit(last, last.char_start + last.char_count, page_num));
        }

        // 3단계: 가장 가까운 줄
        let closest = runs.iter().min_by(|a, b| {
            let da = (y - (a.bbox_y + a.bbox_h / 2.0)).abs();
            let db = (y - (b.bbox_y + b.bbox_h / 2.0)).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        }).unwrap();

        let target_y = closest.bbox_y;
        let target_h = closest.bbox_h;
        let mut line_runs: Vec<&HfRunInfo> = runs.iter()
            .filter(|r| (r.bbox_y - target_y).abs() < 1.0 && (r.bbox_h - target_h).abs() < 1.0)
            .collect();
        line_runs.sort_by(|a, b| a.bbox_x.partial_cmp(&b.bbox_x).unwrap());

        if x < line_runs[0].bbox_x {
            let run = line_runs[0];
            return Ok(format_hf_hit(run, run.char_start, page_num));
        }

        for run in &line_runs {
            if x >= run.bbox_x && x <= run.bbox_x + run.bbox_w {
                let local_x = x - run.bbox_x;
                let char_offset = find_char_at_x_hf(&run.char_positions, local_x);
                return Ok(format_hf_hit(run, run.char_start + char_offset, page_num));
            }
        }

        let last = line_runs.last().unwrap();
        Ok(format_hf_hit(last, last.char_start + last.char_count, page_num))
    }

    /// 각주 영역 히트테스트
    ///
    /// 페이지 좌표가 각주 영역에 해당하는지 판별.
    /// 반환: JSON `{"hit":true,"footnoteIndex":N}` 또는 `{"hit":false}`
    pub fn hit_test_footnote_native(
        &self,
        page_num: u32,
        x: f64,
        y: f64,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::RenderNodeType;

        let tree = self.build_page_tree(page_num)?;

        for child in &tree.root.children {
            if !matches!(child.node_type, RenderNodeType::FootnoteArea) { continue; }

            if x >= child.bbox.x && x <= child.bbox.x + child.bbox.width
                && y >= child.bbox.y && y <= child.bbox.y + child.bbox.height
            {
                // FootnoteArea 내에서 가장 가까운 TextRun의 footnote_index 반환
                let mut fn_idx = 0usize;
                fn find_fn_idx(node: &crate::renderer::render_tree::RenderNode, best: &mut usize) {
                    if let RenderNodeType::TextRun(ref tr) = node.node_type {
                        if let Some(pi) = tr.para_index {
                            if pi >= (usize::MAX - 3000) {
                                if let Some(si) = tr.section_index {
                                    *best = si;
                                }
                            }
                        }
                    }
                    for c in &node.children { find_fn_idx(c, best); }
                }
                find_fn_idx(child, &mut fn_idx);
                return Ok(format!("{{\"hit\":true,\"footnoteIndex\":{}}}", fn_idx));
            }
        }

        Ok("{\"hit\":false}".to_string())
    }

    /// 각주 내부 텍스트 히트테스트
    ///
    /// 편집 모드에서 클릭한 좌표의 각주 내 문단·문자 위치를 반환.
    /// 반환: JSON `{"hit":true,"fnParaIndex":N,"charOffset":N,"footnoteIndex":N,"cursorRect":{...}}`
    pub fn hit_test_in_footnote_native(
        &self,
        page_num: u32,
        x: f64,
        y: f64,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::renderer::layout::compute_char_positions;

        let tree = self.build_page_tree(page_num)?;

        let fn_node = tree.root.children.iter().find(|child| {
            matches!(child.node_type, RenderNodeType::FootnoteArea)
        });
        let fn_node = match fn_node {
            Some(n) => n,
            None => return Ok("{\"hit\":false}".to_string()),
        };

        // 각주 TextRun 정보 수집
        struct FnRunInfo {
            footnote_index: usize,
            fn_para_idx: usize,
            char_start: usize,
            char_count: usize,
            char_positions: Vec<f64>,
            bbox_x: f64,
            bbox_y: f64,
            bbox_w: f64,
            bbox_h: f64,
            baseline: f64,
            font_size: f64,
        }

        // 번호 TextRun(char_start: None) 정보 — 빈 각주의 footnoteIndex/위치 결정용
        struct FnNumberInfo {
            footnote_index: usize,
            fn_para_idx: usize,
            bbox_x: f64,
            bbox_y: f64,
            bbox_w: f64,
            bbox_h: f64,
            font_size: f64,
            baseline: f64,
        }

        fn collect_fn_runs(node: &RenderNode, runs: &mut Vec<FnRunInfo>, number_runs: &mut Vec<FnNumberInfo>) {
            if let RenderNodeType::TextRun(ref text_run) = node.node_type {
                if let (Some(marker_para), Some(marker_section)) =
                    (text_run.para_index, text_run.section_index)
                {
                    if marker_para >= (usize::MAX - 3000) && marker_para < (usize::MAX - 1000) {
                        let fn_para_idx = usize::MAX - 2000 - marker_para;
                        if let Some(cs) = text_run.char_start {
                            // 본문 텍스트 TextRun
                            let positions = compute_char_positions(&text_run.text, &text_run.style);
                            runs.push(FnRunInfo {
                                footnote_index: marker_section,
                                fn_para_idx,
                                char_start: cs,
                                char_count: text_run.text.chars().count(),
                                char_positions: positions,
                                bbox_x: node.bbox.x,
                                bbox_y: node.bbox.y,
                                bbox_w: node.bbox.width,
                                bbox_h: node.bbox.height,
                                baseline: text_run.baseline,
                                font_size: text_run.style.font_size,
                            });
                        } else {
                            // 번호 TextRun (char_start: None)
                            number_runs.push(FnNumberInfo {
                                footnote_index: marker_section,
                                fn_para_idx,
                                bbox_x: node.bbox.x,
                                bbox_y: node.bbox.y,
                                bbox_w: node.bbox.width,
                                bbox_h: node.bbox.height,
                                font_size: text_run.style.font_size,
                                baseline: text_run.baseline,
                            });
                        }
                    }
                }
            }
            for child in &node.children {
                collect_fn_runs(child, runs, number_runs);
            }
        }

        let mut runs: Vec<FnRunInfo> = Vec::new();
        let mut number_runs: Vec<FnNumberInfo> = Vec::new();
        collect_fn_runs(fn_node, &mut runs, &mut number_runs);

        // Y 좌표로 가장 가까운 각주의 footnoteIndex 결정 (텍스트 run이 없는 빈 각주 지원)
        if runs.is_empty() || !runs.iter().any(|r| {
            y >= r.bbox_y && y <= r.bbox_y + r.bbox_h
        }) {
            // 번호 run에서 Y 좌표로 가장 가까운 각주 찾기
            let closest_num = number_runs.iter().min_by(|a, b| {
                let da = (y - (a.bbox_y + a.bbox_h / 2.0)).abs();
                let db = (y - (b.bbox_y + b.bbox_h / 2.0)).abs();
                da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
            });
            if let Some(nr) = closest_num {
                let ascent = nr.font_size * 0.8;
                let cursor_y = nr.bbox_y + nr.baseline - ascent;
                return Ok(format!(
                    "{{\"hit\":true,\"fnParaIndex\":{},\"charOffset\":0,\"footnoteIndex\":{},\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                    nr.fn_para_idx, nr.footnote_index, page_num,
                    nr.bbox_x + nr.bbox_w, cursor_y, nr.font_size
                ));
            }
            return Ok(format!(
                "{{\"hit\":true,\"fnParaIndex\":0,\"charOffset\":0,\"footnoteIndex\":0,\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                page_num, fn_node.bbox.x, fn_node.bbox.y, 12.0
            ));
        }

        fn find_char_at_x(positions: &[f64], local_x: f64) -> usize {
            for (i, &px) in positions.iter().enumerate() {
                if i == 0 {
                    if local_x < px / 2.0 { return 0; }
                } else {
                    let mid = (positions[i - 1] + px) / 2.0;
                    if local_x < mid { return i; }
                }
            }
            positions.len()
        }

        fn format_fn_hit(run: &FnRunInfo, char_offset: usize, page_num: u32) -> String {
            let cursor_x = if char_offset <= run.char_start {
                run.bbox_x
            } else {
                let local_idx = char_offset - run.char_start;
                if local_idx < run.char_positions.len() {
                    run.bbox_x + run.char_positions[local_idx]
                } else if !run.char_positions.is_empty() {
                    run.bbox_x + run.char_positions.last().copied().unwrap_or(0.0)
                } else {
                    run.bbox_x
                }
            };
            let ascent = run.font_size * 0.8;
            let cursor_y = run.bbox_y + run.baseline - ascent;
            format!(
                "{{\"hit\":true,\"fnParaIndex\":{},\"charOffset\":{},\"footnoteIndex\":{},\"cursorRect\":{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}}}",
                run.fn_para_idx, char_offset, run.footnote_index, page_num, cursor_x, cursor_y, run.font_size
            )
        }

        // 1단계: 정확한 bbox 히트
        for run in &runs {
            if x >= run.bbox_x && x <= run.bbox_x + run.bbox_w
                && y >= run.bbox_y && y <= run.bbox_y + run.bbox_h
            {
                let local_x = x - run.bbox_x;
                let char_offset = find_char_at_x(&run.char_positions, local_x);
                return Ok(format_fn_hit(run, run.char_start + char_offset, page_num));
            }
        }

        // 2단계: 같은 줄(y 범위)에서 가장 가까운 run
        let same_line: Vec<&FnRunInfo> = runs.iter()
            .filter(|r| y >= r.bbox_y && y <= r.bbox_y + r.bbox_h)
            .collect();
        if !same_line.is_empty() {
            if x < same_line[0].bbox_x {
                let run = same_line[0];
                return Ok(format_fn_hit(run, run.char_start, page_num));
            }
            let last = same_line.last().unwrap();
            return Ok(format_fn_hit(last, last.char_start + last.char_count, page_num));
        }

        // 3단계: 가장 가까운 줄
        let closest = runs.iter().min_by(|a, b| {
            let da = (y - (a.bbox_y + a.bbox_h / 2.0)).abs();
            let db = (y - (b.bbox_y + b.bbox_h / 2.0)).abs();
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        }).unwrap();

        let target_y = closest.bbox_y;
        let target_h = closest.bbox_h;
        let mut line_runs: Vec<&FnRunInfo> = runs.iter()
            .filter(|r| (r.bbox_y - target_y).abs() < 1.0 && (r.bbox_h - target_h).abs() < 1.0)
            .collect();
        line_runs.sort_by(|a, b| a.bbox_x.partial_cmp(&b.bbox_x).unwrap());

        if x < line_runs[0].bbox_x {
            let run = line_runs[0];
            return Ok(format_fn_hit(run, run.char_start, page_num));
        }

        let last = line_runs.last().unwrap();
        Ok(format_fn_hit(last, last.char_start + last.char_count, page_num))
    }

    /// 각주 내 커서 위치 (커서 렉트) 계산
    ///
    /// 반환: JSON `{"pageIndex":N,"x":F,"y":F,"height":F}`
    pub fn get_cursor_rect_in_footnote_native(
        &self,
        page_num: u32,
        footnote_index: usize,
        fn_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::render_tree::{RenderNode, RenderNodeType};
        use crate::renderer::layout::compute_char_positions;

        let tree = self.build_page_tree(page_num)?;

        let fn_node = tree.root.children.iter().find(|child| {
            matches!(child.node_type, RenderNodeType::FootnoteArea)
        });
        let fn_node = match fn_node {
            Some(n) => n,
            None => return Err(HwpError::RenderError("각주 영역을 찾을 수 없습니다".to_string())),
        };

        let marker_para = usize::MAX - 2000 - fn_para_idx;

        // 해당 각주/문단의 TextRun 찾기
        struct FnCursorRun {
            char_start: usize,
            char_count: usize,
            char_positions: Vec<f64>,
            bbox_x: f64,
            bbox_y: f64,
            bbox_h: f64,
            baseline: f64,
            font_size: f64,
        }

        fn collect_cursor_runs(
            node: &RenderNode, target_section: usize, target_para: usize, runs: &mut Vec<FnCursorRun>,
        ) {
            if let RenderNodeType::TextRun(ref tr) = node.node_type {
                if tr.section_index == Some(target_section) && tr.para_index == Some(target_para) {
                    if let Some(cs) = tr.char_start {
                        let positions = compute_char_positions(&tr.text, &tr.style);
                        runs.push(FnCursorRun {
                            char_start: cs,
                            char_count: tr.text.chars().count(),
                            char_positions: positions,
                            bbox_x: node.bbox.x,
                            bbox_y: node.bbox.y,
                            bbox_h: node.bbox.height,
                            baseline: tr.baseline,
                            font_size: tr.style.font_size,
                        });
                    }
                }
            }
            for c in &node.children { collect_cursor_runs(c, target_section, target_para, runs); }
        }

        let mut runs: Vec<FnCursorRun> = Vec::new();
        collect_cursor_runs(fn_node, footnote_index, marker_para, &mut runs);

        if runs.is_empty() {
            // 폴백: 번호 TextRun(char_start=None) 뒤의 위치를 찾기
            // 번호 run은 section_index=footnote_index, para_index=marker_para, char_start=None
            fn find_number_run_end(node: &RenderNode, target_sec: usize, target_para: usize) -> Option<(f64, f64, f64)> {
                if let RenderNodeType::TextRun(ref tr) = node.node_type {
                    if tr.section_index == Some(target_sec) && tr.para_index == Some(target_para) && tr.char_start.is_none() {
                        // 번호 run의 오른쪽 끝
                        return Some((node.bbox.x + node.bbox.width, node.bbox.y + tr.baseline - tr.style.font_size * 0.8, tr.style.font_size));
                    }
                }
                for c in &node.children {
                    if let Some(r) = find_number_run_end(c, target_sec, target_para) { return Some(r); }
                }
                None
            }
            if let Some((x, y, h)) = find_number_run_end(fn_node, footnote_index, marker_para) {
                return Ok(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    page_num, x, y, h
                ));
            }
            return Ok(format!(
                "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                page_num, fn_node.bbox.x, fn_node.bbox.y, 12.0
            ));
        }

        // char_offset에 해당하는 run 찾기
        for run in &runs {
            if char_offset >= run.char_start && char_offset <= run.char_start + run.char_count {
                let local_idx = char_offset - run.char_start;
                let cursor_x = if local_idx < run.char_positions.len() {
                    run.bbox_x + run.char_positions[local_idx]
                } else if !run.char_positions.is_empty() {
                    run.bbox_x + run.char_positions.last().copied().unwrap_or(0.0)
                } else {
                    run.bbox_x
                };
                let ascent = run.font_size * 0.8;
                let cursor_y = run.bbox_y + run.baseline - ascent;
                return Ok(format!(
                    "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
                    page_num, cursor_x, cursor_y, run.font_size
                ));
            }
        }

        // 마지막 run의 끝
        let last = runs.last().unwrap();
        let cursor_x = if !last.char_positions.is_empty() {
            last.bbox_x + last.char_positions.last().copied().unwrap_or(0.0)
        } else {
            last.bbox_x
        };
        let ascent = last.font_size * 0.8;
        let cursor_y = last.bbox_y + last.baseline - ascent;
        Ok(format!(
            "{{\"pageIndex\":{},\"x\":{:.1},\"y\":{:.1},\"height\":{:.1}}}",
            page_num, cursor_x, cursor_y, last.font_size
        ))
    }

    /// 페이지의 각주 참조 정보를 반환한다.
    ///
    /// footnoteIndex에 해당하는 FootnoteRef의 source(para_index, control_index)를 반환.
    /// 반환: JSON `{"ok":true,"sectionIdx":N,"paraIdx":N,"controlIdx":N}`
    pub fn get_page_footnote_info_native(
        &self,
        page_num: u32,
        footnote_index: usize,
    ) -> Result<String, HwpError> {
        use crate::renderer::pagination::FootnoteSource;

        let (section_idx, local_page) = self.find_section_for_page(page_num);
        let pr = self.pagination.get(section_idx)
            .ok_or_else(|| HwpError::RenderError("구역을 찾을 수 없습니다".to_string()))?;
        let page = pr.pages.get(local_page)
            .ok_or_else(|| HwpError::RenderError("페이지를 찾을 수 없습니다".to_string()))?;

        let fn_ref = page.footnotes.get(footnote_index)
            .ok_or_else(|| HwpError::RenderError(format!(
                "각주 인덱스 {} 범위 초과 (총 {}개)", footnote_index, page.footnotes.len()
            )))?;

        let (para_idx, control_idx, source_type) = match &fn_ref.source {
            FootnoteSource::Body { para_index, control_index } => (*para_index, *control_index, "body"),
            FootnoteSource::TableCell { para_index, table_control_index, .. } => (*para_index, *table_control_index, "table"),
            FootnoteSource::ShapeTextBox { para_index, shape_control_index, .. } => (*para_index, *shape_control_index, "shape"),
        };

        Ok(format!(
            "{{\"ok\":true,\"sectionIdx\":{},\"paraIdx\":{},\"controlIdx\":{},\"sourceType\":\"{}\"}}",
            section_idx, para_idx, control_idx, source_type
        ))
    }
}
