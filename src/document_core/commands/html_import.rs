//! HTML 붙여넣기 + HTML 파싱 관련 native 메서드

use super::super::helpers::*;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;

impl DocumentCore {
    pub fn paste_html_native(
        &mut self,
        section_idx: usize,
        para_idx: usize,
        char_offset: usize,
        html: &str,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 {} 범위 초과",
                section_idx
            )));
        }
        if para_idx >= self.document.sections[section_idx].paragraphs.len() {
            return Err(HwpError::RenderError(format!(
                "문단 {} 범위 초과",
                para_idx
            )));
        }

        // HTML 파싱 → 문단 목록 생성
        let parsed_paras = self.parse_html_to_paragraphs(html);
        if parsed_paras.is_empty() {
            return Ok("{\"ok\":false,\"error\":\"empty html\"}".to_string());
        }

        self.document.sections[section_idx].raw_stream = None;

        let clip_count = parsed_paras.len();

        if clip_count == 1 && parsed_paras[0].controls.is_empty() {
            // 단일 문단 텍스트 삽입
            let clip_text = parsed_paras[0].text.clone();
            let clip_char_shapes = parsed_paras[0].char_shapes.clone();
            let clip_char_offsets = parsed_paras[0].char_offsets.clone();
            let new_chars = clip_text.chars().count();

            self.document.sections[section_idx].paragraphs[para_idx]
                .insert_text_at(char_offset, &clip_text);

            self.apply_clipboard_char_shapes(
                section_idx,
                para_idx,
                char_offset,
                &clip_char_shapes,
                &clip_char_offsets,
                new_chars,
            );

            // [Task #2299] 리셋 판별용 — reflow 이전 저장 흐름 end 캡처.
            let stored_end_for_reset = crate::renderer::composer::paragraph_flow_end(
                &self.document.sections[section_idx].paragraphs[para_idx],
            );
            self.reflow_paragraph(section_idx, para_idx);
            // [Task #2299] 삽입/변경 문단들의 vpos 를 흐름에 연결한다 — placeholder 를
            // 방치하면 이후 편집의 vpos 재계산이 저장 단/쪽 리셋으로 오인해 고착시킨다.
            let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                para_idx,
                None,
                stored_end_for_reset,
                &self.styles,
                self.dpi,
                doc_hwp3_layout,
            );
            self.recompose_paragraph(section_idx, para_idx);
            self.paginate_if_needed();

            let new_offset = char_offset + new_chars;
            self.event_log.push(DocumentEvent::HtmlImported {
                section: section_idx,
                para: para_idx,
            });
            return Ok(format!(
                "{{\"ok\":true,\"paraIdx\":{},\"charOffset\":{}}}",
                para_idx, new_offset
            ));
        }

        // 컨트롤(표/이미지 등)을 포함하는 문단이 있는지 확인
        let has_controls = parsed_paras.iter().any(|p| !p.controls.is_empty());

        if has_controls {
            // 컨트롤 포함 문단은 merge 불가 → 직접 삽입
            let right_half =
                self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);

            // 현재 문단 (왼쪽 반)이 비어있으면 첫 번째 파싱 문단으로 대체
            let left_empty = self.document.sections[section_idx].paragraphs[para_idx]
                .text
                .is_empty();

            let insert_idx = if left_empty {
                // 빈 왼쪽 문단을 첫 번째 파싱 문단으로 대체
                self.document.sections[section_idx].paragraphs[para_idx] = parsed_paras[0].clone();
                let idx = para_idx + 1;
                for i in 1..clip_count {
                    self.document.sections[section_idx]
                        .paragraphs
                        .insert(idx + i - 1, parsed_paras[i].clone());
                }
                para_idx + clip_count
            } else {
                // 왼쪽 문단에 텍스트 → 파싱 문단들을 그 뒤에 삽입
                let idx = para_idx + 1;
                for i in 0..clip_count {
                    self.document.sections[section_idx]
                        .paragraphs
                        .insert(idx + i, parsed_paras[i].clone());
                }
                para_idx + 1 + clip_count
            };

            // 오른쪽 반이 비어있지 않으면 새 문단으로 추가
            let last_para_idx;
            let merge_point;
            if !right_half.text.is_empty() {
                self.document.sections[section_idx]
                    .paragraphs
                    .insert(insert_idx, right_half);
                last_para_idx = insert_idx;
                merge_point = 0;
            } else {
                last_para_idx = insert_idx - 1;
                // 마지막 문단이 컨트롤 문단이면 그 뒤 위치
                let last = &self.document.sections[section_idx].paragraphs[last_para_idx];
                merge_point = last.text.chars().count();
            }

            for i in para_idx..=last_para_idx {
                self.reflow_paragraph(section_idx, i);
            }
            // [Task #2299] 삽입 문단들의 vpos 를 흐름에 연결한다 — 클론/placeholder
            // 좌표를 방치하면 이후 편집의 vpos 재계산이 저장 단/쪽 리셋으로 오인해
            // 고착시킨다. left_empty 면 host 자체가 클론이라 신규 구간에 포함한다.
            let fresh_start = if left_empty { para_idx } else { para_idx + 1 };
            let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
            crate::renderer::composer::recalculate_section_vpos(
                &mut self.document.sections[section_idx].paragraphs,
                para_idx,
                Some(fresh_start..last_para_idx + 1),
                None,
                &self.styles,
                self.dpi,
                doc_hwp3_layout,
            );

            // 선택적 재구성: 원본 문단 재구성 + 삽입 문단 composed 추가
            self.recompose_paragraph(section_idx, para_idx);
            for i in (para_idx + 1..=last_para_idx).rev() {
                self.insert_composed_paragraph(section_idx, i);
            }
            self.paginate_if_needed();

            self.event_log.push(DocumentEvent::HtmlImported {
                section: section_idx,
                para: para_idx,
            });
            return Ok(format!(
                "{{\"ok\":true,\"paraIdx\":{},\"charOffset\":{}}}",
                last_para_idx, merge_point
            ));
        }

        // 다중 문단 삽입 (컨트롤 없는 텍스트만)
        let right_half =
            self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);

        self.document.sections[section_idx].paragraphs[para_idx].merge_from(&parsed_paras[0]);

        let mut insert_idx = para_idx + 1;
        for i in 1..clip_count {
            self.document.sections[section_idx]
                .paragraphs
                .insert(insert_idx, parsed_paras[i].clone());
            insert_idx += 1;
        }

        let last_para_idx = insert_idx - 1;
        let merge_point =
            self.document.sections[section_idx].paragraphs[last_para_idx].merge_from(&right_half);

        for i in para_idx..=last_para_idx {
            self.reflow_paragraph(section_idx, i);
        }
        // [Task #2299] 삽입/변경 문단들의 vpos 를 흐름에 연결한다 — placeholder 를
        // 방치하면 이후 편집의 vpos 재계산이 저장 단/쪽 리셋으로 오인해 고착시킨다.
        let doc_hwp3_layout = self.document.layout_profile().hwp3_layout();
        crate::renderer::composer::recalculate_section_vpos(
            &mut self.document.sections[section_idx].paragraphs,
            para_idx,
            Some(para_idx + 1..last_para_idx + 1),
            None,
            &self.styles,
            self.dpi,
            doc_hwp3_layout,
        );

        // 선택적 재구성: 원본 문단 재구성 + 삽입 문단 composed 추가
        self.recompose_paragraph(section_idx, para_idx);
        for i in para_idx + 1..=last_para_idx {
            self.insert_composed_paragraph(section_idx, i);
        }
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::HtmlImported {
            section: section_idx,
            para: para_idx,
        });
        Ok(format!(
            "{{\"ok\":true,\"paraIdx\":{},\"charOffset\":{}}}",
            last_para_idx, merge_point
        ))
    }

    fn normalize_html_paragraphs_for_cell_paste(parsed_paras: Vec<Paragraph>) -> Vec<Paragraph> {
        // 셀 내부에는 Table Control 중첩 불가 → 컨트롤 포함 문단은 텍스트만 추출
        parsed_paras
            .into_iter()
            .map(|mut p| {
                if !p.controls.is_empty() {
                    let text = if p.text.is_empty() || p.text == "\u{0002}" {
                        match p.controls.first() {
                            Some(Control::Table(tbl)) => tbl
                                .cells
                                .iter()
                                .map(|c| {
                                    c.paragraphs
                                        .iter()
                                        .map(|cp| cp.text.clone())
                                        .collect::<Vec<_>>()
                                        .join(" ")
                                })
                                .collect::<Vec<_>>()
                                .join("\t"),
                            _ => String::new(),
                        }
                    } else {
                        p.text.clone()
                    };
                    p.controls.clear();
                    p.text = text;
                    // [#3494] char_count 는 문단 종결자를 포함한다 (model/paragraph.rs:1042).
                    p.char_count = p.text.encode_utf16().count() as u32 + 1;
                    p.char_offsets = p
                        .text
                        .chars()
                        .scan(0u32, |acc, c| {
                            let off = *acc;
                            *acc += c.len_utf16() as u32;
                            Some(off)
                        })
                        .collect();
                }
                p
            })
            .collect()
    }

    fn paste_html_paragraphs_into_cell_paragraphs(
        cell_paras: &mut Vec<Paragraph>,
        cell_para_idx: usize,
        char_offset: usize,
        parsed_paras: &[Paragraph],
    ) -> Result<(usize, usize), HwpError> {
        if cell_para_idx >= cell_paras.len() {
            return Err(HwpError::RenderError(format!(
                "셀 문단 {} 범위 초과",
                cell_para_idx
            )));
        }

        let clip_count = parsed_paras.len();
        if clip_count == 1 && parsed_paras[0].controls.is_empty() {
            let clip_text = parsed_paras[0].text.clone();
            let new_chars = clip_text.chars().count();

            cell_paras[cell_para_idx].insert_text_at(char_offset, &clip_text);

            let clip_char_shapes = parsed_paras[0].char_shapes.clone();
            let clip_char_offsets = parsed_paras[0].char_offsets.clone();
            Self::apply_clipboard_char_shapes_to_para(
                &mut cell_paras[cell_para_idx],
                char_offset,
                &clip_char_shapes,
                &clip_char_offsets,
                new_chars,
            );

            return Ok((cell_para_idx, char_offset + new_chars));
        }

        let right_half = cell_paras[cell_para_idx].split_at(char_offset);
        cell_paras[cell_para_idx].merge_from(&parsed_paras[0]);

        let mut insert_idx = cell_para_idx + 1;
        for parsed_para in parsed_paras.iter().skip(1) {
            cell_paras.insert(insert_idx, parsed_para.clone());
            insert_idx += 1;
        }

        let last_para_idx = insert_idx - 1;
        let merge_point = cell_paras[last_para_idx].merge_from(&right_half);
        Ok((last_para_idx, merge_point))
    }

    /// HTML 문자열을 파싱하여 셀 내부 캐럿 위치에 삽입한다.
    pub fn paste_html_in_cell_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        control_idx: usize,
        cell_idx: usize,
        cell_para_idx: usize,
        char_offset: usize,
        html: &str,
    ) -> Result<String, HwpError> {
        let parsed_paras = self.parse_html_to_paragraphs(html);
        if parsed_paras.is_empty() {
            return Ok("{\"ok\":false,\"error\":\"empty html\"}".to_string());
        }
        let parsed_paras = Self::normalize_html_paragraphs_for_cell_paste(parsed_paras);

        let (last_para_idx, merge_point) = {
            let section =
                self.document.sections.get_mut(section_idx).ok_or_else(|| {
                    HwpError::RenderError(format!("구역 {} 범위 초과", section_idx))
                })?;
            section.raw_stream = None;
            let para = section.paragraphs.get_mut(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 {} 범위 초과", parent_para_idx))
            })?;
            let control = para.controls.get_mut(control_idx).ok_or_else(|| {
                HwpError::RenderError(format!("컨트롤 {} 범위 초과", control_idx))
            })?;
            let table = match control {
                Control::Table(t) => t,
                _ => return Err(HwpError::RenderError("표가 아님".to_string())),
            };
            let cell_paras = &mut table
                .cells
                .get_mut(cell_idx)
                .ok_or_else(|| HwpError::RenderError(format!("셀 {} 범위 초과", cell_idx)))?
                .paragraphs;
            Self::paste_html_paragraphs_into_cell_paragraphs(
                cell_paras,
                cell_para_idx,
                char_offset,
                &parsed_paras,
            )?
        };

        for i in cell_para_idx..=last_para_idx {
            self.reflow_cell_paragraph(section_idx, parent_para_idx, control_idx, cell_idx, i);
        }
        self.mark_cell_control_dirty(section_idx, parent_para_idx, control_idx);
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::HtmlImported {
            section: section_idx,
            para: parent_para_idx,
        });
        Ok(format!(
            "{{\"ok\":true,\"cellParaIdx\":{},\"charOffset\":{}}}",
            last_para_idx, merge_point
        ))
    }

    /// HTML 문자열을 파싱하여 cellPath가 가리키는 중첩 표 셀에 삽입한다.
    pub fn paste_html_in_cell_by_path_native(
        &mut self,
        section_idx: usize,
        parent_para_idx: usize,
        path: &[(usize, usize, usize)],
        char_offset: usize,
        html: &str,
    ) -> Result<String, HwpError> {
        if path.is_empty() {
            return Err(HwpError::RenderError("경로가 비어있습니다".to_string()));
        }

        let parsed_paras = self.parse_html_to_paragraphs(html);
        if parsed_paras.is_empty() {
            return Ok("{\"ok\":false,\"error\":\"empty html\"}".to_string());
        }
        let parsed_paras = Self::normalize_html_paragraphs_for_cell_paste(parsed_paras);

        let cell_para_idx = path[path.len() - 1].2;
        let (last_para_idx, merge_point) = {
            let cell_paras =
                self.get_cell_paragraphs_mut_by_path(section_idx, parent_para_idx, path)?;
            Self::paste_html_paragraphs_into_cell_paragraphs(
                cell_paras,
                cell_para_idx,
                char_offset,
                &parsed_paras,
            )?
        };

        let outer_ctrl = path[0].0;
        self.mark_cell_control_dirty(section_idx, parent_para_idx, outer_ctrl);
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::HtmlImported {
            section: section_idx,
            para: parent_para_idx,
        });
        Ok(format!(
            "{{\"ok\":true,\"cellParaIdx\":{},\"charOffset\":{}}}",
            last_para_idx, merge_point
        ))
    }

    // === HTML 파서 ===

    /// HTML 문자열을 파싱하여 Paragraph 목록을 생성한다.
    /// `<div>`/`<p><table>` 재귀 하강 깊이 상한. Gmail 등 웹메일 클립보드는 서명·본문을
    /// 감싸는 wrapper `<div>`가 수십 겹인 경우가 흔하다(예: 실사용 리포트 — 서명 블록 하나에
    /// `</div>` 8개 이상 연속). 이 깊이만큼 매번 `find_closing_tag_chars`로 전체 구간을
    /// 다시 훑고 재귀하므로, 깊이가 무제한이면 붙여넣기 한 번이 브라우저를 "응답 없음"으로
    /// 멈춰 세울 만큼 느려진다(실사용 확인). 이 상한을 넘으면 태그 트리 파싱을 포기하고
    /// 태그만 제거한 평문 문단으로 폴백한다 — 서식은 잃어도 붙여넣기 자체는 항상 끝난다.
    const HTML_PASTE_MAX_RECURSION_DEPTH: u32 = 16;

    /// 파싱을 시도할 최대 HTML 바이트 크기. 이보다 크면 태그 트리 파싱 없이 평문으로
    /// 폴백한다 — 크기 자체가 계산량의 또 다른 축이라 깊이 상한과 별개로 방어한다.
    const HTML_PASTE_MAX_BYTES: usize = 400_000;

    pub(crate) fn parse_html_to_paragraphs(&mut self, html: &str) -> Vec<Paragraph> {
        self.parse_html_to_paragraphs_at_depth(html, 0)
    }

    fn parse_html_to_paragraphs_at_depth(&mut self, html: &str, depth: u32) -> Vec<Paragraph> {
        if depth >= Self::HTML_PASTE_MAX_RECURSION_DEPTH || html.len() > Self::HTML_PASTE_MAX_BYTES
        {
            let mut fallback_paragraphs = Vec::new();
            self.flush_text_to_paragraphs(&mut fallback_paragraphs, &html_strip_tags(html));
            return fallback_paragraphs;
        }

        let mut paragraphs: Vec<Paragraph> = Vec::new();

        // <!--StartFragment-->...<!--EndFragment--> 영역 추출 (없으면 전체 사용)
        let content = if let Some(start) = html.find("<!--StartFragment-->") {
            let after = &html[start + 20..];
            if let Some(end) = after.find("<!--EndFragment-->") {
                &after[..end]
            } else {
                after
            }
        } else {
            // <body>...</body> 영역 추출 시도
            if let Some(start) = html.find("<body") {
                let after_tag = &html[start..];
                if let Some(gt) = after_tag.find('>') {
                    let inner = &after_tag[gt + 1..];
                    if let Some(end) = inner.find("</body>") {
                        &inner[..end]
                    } else {
                        inner
                    }
                } else {
                    html
                }
            } else {
                html
            }
        };

        // 최상위 태그 파싱
        let mut pos = 0;
        let chars: Vec<char> = content.chars().collect();
        let len = chars.len();
        let mut pending_text = String::new();

        while pos < len {
            if chars[pos] == '<' {
                // 태그 시작
                let tag_start = pos;
                let tag_end = find_char(&chars, pos, '>');
                if tag_end >= len {
                    break;
                }

                let tag_str: String = chars[tag_start..=tag_end].iter().collect();
                let tag_lower = tag_str.to_lowercase();

                if tag_lower.starts_with("<table") {
                    // 보류 중인 텍스트 처리
                    if !pending_text.trim().is_empty() {
                        self.flush_text_to_paragraphs(&mut paragraphs, &pending_text);
                    }
                    pending_text.clear();

                    // 표 전체 추출
                    let table_end = find_closing_tag_chars(&chars, pos, "table");
                    let table_html: String = chars[tag_start..table_end.min(len)].iter().collect();
                    self.parse_table_html(&mut paragraphs, &table_html);
                    pos = table_end;
                    continue;
                } else if tag_lower.starts_with("<img") {
                    if !pending_text.trim().is_empty() {
                        self.flush_text_to_paragraphs(&mut paragraphs, &pending_text);
                    }
                    pending_text.clear();

                    self.parse_img_html(&mut paragraphs, &tag_str);
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("<p") {
                    // 보류 중인 텍스트 처리
                    if !pending_text.trim().is_empty() {
                        self.flush_text_to_paragraphs(&mut paragraphs, &pending_text);
                    }
                    pending_text.clear();

                    // <p> 블록 추출
                    let p_content_start = tag_end + 1;
                    let p_end = find_closing_tag_chars(&chars, pos, "p");
                    let p_inner: String = chars[p_content_start..p_end.min(len)].iter().collect();
                    // </p> 태그 제거
                    let p_inner = if let Some(idx) = p_inner.rfind("</p>") {
                        &p_inner[..idx]
                    } else {
                        &p_inner
                    };

                    // <p> 내부에 <table>이 있으면 재귀적으로 처리
                    if p_inner.to_lowercase().contains("<table") {
                        let sub_paras = self.parse_html_to_paragraphs_at_depth(p_inner, depth + 1);
                        paragraphs.extend(sub_paras);
                        pos = p_end;
                        continue;
                    }

                    let para_style = parse_inline_style(&tag_str);
                    let para_shape_id = self.css_to_para_shape_id(&para_style);

                    let mut para = Paragraph::default();
                    para.para_shape_id = para_shape_id;
                    self.parse_inline_content(&mut para, p_inner);
                    paragraphs.push(para);

                    pos = p_end;
                    continue;
                } else if tag_lower.starts_with("<div") {
                    // div 내부의 콘텐츠를 재귀적으로 처리
                    let div_content_start = tag_end + 1;
                    let div_end = find_closing_tag_chars(&chars, pos, "div");
                    let div_inner: String =
                        chars[div_content_start..div_end.min(len)].iter().collect();
                    let div_inner = if let Some(idx) = div_inner.rfind("</div>") {
                        &div_inner[..idx]
                    } else {
                        &div_inner
                    };

                    let sub_paras = self.parse_html_to_paragraphs_at_depth(div_inner, depth + 1);
                    paragraphs.extend(sub_paras);
                    pos = div_end;
                    continue;
                } else if tag_lower.starts_with("<ul") || tag_lower.starts_with("<ol") {
                    // [Gmail 등 웹메일 서명 붙여넣기가 raw 태그로 나오던 결함] 목록 태그
                    // 자체는 컨테이너일 뿐이라 <div>처럼 내부를 재귀 처리한다 — <li> 각각이
                    // 실제 항목 문단이 된다.
                    if !pending_text.trim().is_empty() {
                        self.flush_text_to_paragraphs(&mut paragraphs, &pending_text);
                        pending_text.clear();
                    }
                    let list_tag_name = if tag_lower.starts_with("<ul") {
                        "ul"
                    } else {
                        "ol"
                    };
                    let list_content_start = tag_end + 1;
                    let list_end = find_closing_tag_chars(&chars, pos, list_tag_name);
                    let list_inner: String = chars[list_content_start..list_end.min(len)]
                        .iter()
                        .collect();
                    let close_marker = format!("</{list_tag_name}>");
                    let list_inner = if let Some(idx) = list_inner.rfind(&close_marker) {
                        &list_inner[..idx]
                    } else {
                        &list_inner
                    };
                    let sub_paras = self.parse_html_to_paragraphs(list_inner);
                    paragraphs.extend(sub_paras);
                    pos = list_end;
                    continue;
                } else if tag_lower.starts_with("<li") {
                    // <li> 내부 전체(중첩 span/strong 등 포함)를 한 문단으로 묶어
                    // parse_inline_content 로 서식까지 보존해 파싱하고, 글머리 기호를
                    // 앞에 붙인다. 표 없는 최상위 <p> 처리와 동일한 패턴.
                    if !pending_text.trim().is_empty() {
                        self.flush_text_to_paragraphs(&mut paragraphs, &pending_text);
                        pending_text.clear();
                    }
                    let li_content_start = tag_end + 1;
                    let li_end = find_closing_tag_chars(&chars, pos, "li");
                    let li_inner: String =
                        chars[li_content_start..li_end.min(len)].iter().collect();
                    let li_inner = if let Some(idx) = li_inner.rfind("</li>") {
                        &li_inner[..idx]
                    } else {
                        &li_inner
                    };
                    let mut para = Paragraph::default();
                    self.parse_inline_content(&mut para, li_inner);
                    if !para.text.trim().is_empty() {
                        para.text = format!("• {}", para.text);
                        para.char_offsets = para
                            .text
                            .chars()
                            .scan(0u32, |acc, c| {
                                let off = *acc;
                                *acc += c.len_utf16() as u32;
                                Some(off)
                            })
                            .collect();
                        para.char_count = para.text.encode_utf16().count() as u32 + 1;
                        // 글머리 기호("• ")만큼 스타일 구간을 오른쪽으로 밀어 정렬을 맞춘다.
                        // start_pos 는 UTF-16 코드유닛 단위(위 char_offsets 와 동일 축).
                        let bullet_len = "• ".encode_utf16().count() as u32;
                        for cs in &mut para.char_shapes {
                            cs.start_pos += bullet_len;
                        }
                        paragraphs.push(para);
                    }
                    pos = li_end;
                    continue;
                } else if tag_lower.starts_with("<br") {
                    // <br> → 문단 구분
                    if !pending_text.is_empty() {
                        self.flush_text_to_paragraphs(&mut paragraphs, &pending_text);
                        pending_text.clear();
                    } else {
                        // 빈 문단 추가
                        paragraphs.push(Paragraph::default());
                    }
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("</") {
                    // 닫는 태그 무시
                    pos = tag_end + 1;
                    continue;
                } else {
                    // 기타 태그 무시 (span 등 인라인은 <p> 밖에서 직접 올 수 있음)
                    if tag_lower.starts_with("<span") {
                        // [Gmail 등 웹메일 서명 붙여넣기가 raw 태그로 나오던 결함] 예전
                        // 코드는 span 내부(중첩 <u>/<strong>/주석 포함)를 첫 ">" 뒤부터
                        // 그대로 pending_text 에 밀어 넣어, 태그 자체가 문서에 문자로
                        // 그대로 찍혔다. <p> 처리와 같은 방식으로 parse_inline_content 에
                        // 넘겨 중첩 서식(굵게 등)까지 해석한 문단으로 만든다.
                        if !pending_text.trim().is_empty() {
                            self.flush_text_to_paragraphs(&mut paragraphs, &pending_text);
                            pending_text.clear();
                        }
                        let span_end = find_closing_tag_chars(&chars, pos, "span");
                        let inner_start = tag_end + 1;
                        let inner_end = span_end.saturating_sub(7); // "</span>".len()
                        let span_inner: String = chars
                            [inner_start..inner_end.max(inner_start).min(len)]
                            .iter()
                            .collect();
                        let mut para = Paragraph::default();
                        self.parse_inline_content(&mut para, &span_inner);
                        if !para.text.trim().is_empty() {
                            paragraphs.push(para);
                        }
                        pos = span_end;
                        continue;
                    }
                    pos = tag_end + 1;
                    continue;
                }
            } else {
                // 일반 텍스트
                pending_text.push(chars[pos]);
                pos += 1;
            }
        }

        // 남은 텍스트 처리
        if !pending_text.trim().is_empty() {
            self.flush_text_to_paragraphs(&mut paragraphs, &pending_text);
        }

        // 빈 결과 시 최소 처리 — flush_text_to_paragraphs 재사용으로 줄바꿈 분리와
        // 긴 줄 강제 절단(FLUSH_LINE_CHAR_CAP)을 여기도 동일하게 적용한다.
        // flush_text_to_paragraphs 가 자체적으로 decode_html_entities 를 수행하므로,
        // 여기서는 태그만 벗긴 원문(html_strip_tags)을 넘겨 엔티티 이중 디코딩을 피한다.
        if paragraphs.is_empty() {
            let stripped = html_strip_tags(html);
            if !stripped.trim().is_empty() {
                self.flush_text_to_paragraphs(&mut paragraphs, &stripped);
            }
        }

        paragraphs
    }

    /// 개행이 전혀 없는 한 "줄"을 이 길이(문자 수) 단위로 강제 절단해 별도 문단으로 만든다.
    ///
    /// [붙여넣기 화면 겹침 방지] 웹페이지 렌더 결과가 아니라 원본 소스(view-source 등)를
    /// 통째로 복사하면, 내부 텍스트에 실제 개행 문자가 전혀 없는 경우(예: 한 줄짜리 최소화
    /// JS/JSON 블록)가 있다 — 실사용 확인: Daum 홈페이지 전체 소스(HTML 598KB) 붙여넣기가
    /// 개행 없는 50만자 이상 단일 문단을 만들어 화면이 겹쳐 보이는 결과로 이어졌다. 문단
    /// 하나가 이 정도로 크면 줄바꿈 계산 등 조판 경로가 원래 가정하지 않은 크기라 무너진다.
    const FLUSH_LINE_CHAR_CAP: usize = 4000;

    /// 텍스트를 문단으로 변환하여 추가한다 (줄바꿈 기준 분리, 개행 없는 긴 줄은 추가 절단).
    pub(crate) fn flush_text_to_paragraphs(&self, paragraphs: &mut Vec<Paragraph>, text: &str) {
        let decoded = decode_html_entities(text);
        for line in decoded.split('\n') {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let chars: Vec<char> = trimmed.chars().collect();
            for chunk in chars.chunks(Self::FLUSH_LINE_CHAR_CAP) {
                let mut para = Paragraph::default();
                para.text = chunk.iter().collect();
                // [#3494] char_count 는 문단 종결자를 포함한다 (model/paragraph.rs:1042).
                para.char_count = para.text.encode_utf16().count() as u32 + 1;
                para.char_offsets = para
                    .text
                    .chars()
                    .scan(0u32, |acc, c| {
                        let off = *acc;
                        *acc += c.len_utf16() as u32;
                        Some(off)
                    })
                    .collect();
                paragraphs.push(para);
            }
        }
    }

    /// <p> 태그 내부의 인라인 콘텐츠를 파싱하여 Paragraph에 채운다.
    pub(crate) fn parse_inline_content(&mut self, para: &mut Paragraph, html: &str) {
        let mut full_text = String::new();
        // (char_start, char_end, char_shape_id) 형태의 스타일 범위
        let mut style_runs: Vec<(usize, usize, u32)> = Vec::new();

        let chars: Vec<char> = html.chars().collect();
        let len = chars.len();
        let mut pos = 0;

        // 중첩 볼드/이탤릭/밑줄 추적
        let mut inherited_bold = false;
        let mut inherited_italic = false;
        let mut inherited_underline = false;

        while pos < len {
            if chars[pos] == '<' {
                let tag_end = find_char(&chars, pos, '>');
                if tag_end >= len {
                    break;
                }

                let tag_str: String = chars[pos..=tag_end].iter().collect();
                let tag_lower = tag_str.to_lowercase();

                if tag_lower.starts_with("<span") {
                    // [붙여넣기 무한루프/응답없음 방지] span_end_tag(깊이 인식 탐색)가 이미
                    // 정확한 닫는 위치를 갖고 있는데, 예전 코드는 그 뒤에 또 "</span>" 리터럴을
                    // 처음부터 선형 재탐색했다 — 중첩 span이 많은 Gmail류 클립보드(span 수백
                    // 개)에서 O(n) 재탐색이 span마다 반복돼 실질적으로 O(n²)이 됐고, 게다가
                    // 깊이를 무시한 첫 "</span>" 매치라 중첩 span에서는 내부 span의 닫는
                    // 태그를 잘못 집는 경계 버그이기도 했다. span_end_tag 하나로 통일한다
                    // ("</span>".len() == 7 만큼 빼면 내용 끝 위치).
                    let span_end_tag = find_closing_tag_chars(&chars, pos, "span");
                    let inner_start = tag_end + 1;
                    let inner_end = span_end_tag.saturating_sub(7);
                    let inner: String = chars[inner_start..inner_end.max(inner_start).min(len)]
                        .iter()
                        .collect();
                    let inner_text = decode_html_entities(&html_strip_tags(&inner));

                    if !inner_text.is_empty() {
                        let css = parse_inline_style(&tag_str);
                        let char_shape_id = self.css_to_char_shape_id(
                            &css,
                            inherited_bold,
                            inherited_italic,
                            inherited_underline,
                        );
                        let start = full_text.chars().count();
                        full_text.push_str(&inner_text);
                        let end = full_text.chars().count();
                        style_runs.push((start, end, char_shape_id));
                    }

                    pos = span_end_tag;
                    continue;
                } else if tag_lower.starts_with("<b>") || tag_lower.starts_with("<strong") {
                    inherited_bold = true;
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("</b>") || tag_lower.starts_with("</strong") {
                    inherited_bold = false;
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("<i>") || tag_lower.starts_with("<em") {
                    inherited_italic = true;
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("</i>") || tag_lower.starts_with("</em") {
                    inherited_italic = false;
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("<u>") {
                    inherited_underline = true;
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("</u>") {
                    inherited_underline = false;
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("<br") {
                    full_text.push('\n');
                    pos = tag_end + 1;
                    continue;
                } else {
                    // 기타 태그 무시
                    pos = tag_end + 1;
                    continue;
                }
            } else {
                // 태그 밖의 일반 텍스트
                let text_start = pos;
                while pos < len && chars[pos] != '<' {
                    pos += 1;
                }
                let raw: String = chars[text_start..pos].iter().collect();
                let decoded = decode_html_entities(&raw);
                if !decoded.is_empty() {
                    if inherited_bold || inherited_italic || inherited_underline {
                        let css_parts: Vec<String> = [
                            if inherited_bold {
                                Some("font-weight:bold".to_string())
                            } else {
                                None
                            },
                            if inherited_italic {
                                Some("font-style:italic".to_string())
                            } else {
                                None
                            },
                            if inherited_underline {
                                Some("text-decoration:underline".to_string())
                            } else {
                                None
                            },
                        ]
                        .into_iter()
                        .flatten()
                        .collect();
                        let fake_css = css_parts.join(";");
                        let char_shape_id =
                            self.css_to_char_shape_id(&fake_css, false, false, false);
                        let start = full_text.chars().count();
                        full_text.push_str(&decoded);
                        let end = full_text.chars().count();
                        style_runs.push((start, end, char_shape_id));
                    } else {
                        full_text.push_str(&decoded);
                    }
                }
                continue;
            }
        }

        para.text = full_text;
        // [#3494] char_count 는 문단 종결자를 포함한다 (model/paragraph.rs:1042).
        para.char_count = para.text.encode_utf16().count() as u32 + 1;
        para.char_offsets = para
            .text
            .chars()
            .scan(0u32, |acc, c| {
                let off = *acc;
                *acc += c.len_utf16() as u32;
                Some(off)
            })
            .collect();

        // 스타일 범위를 CharShapeRef로 변환
        for (start, _end, char_shape_id) in &style_runs {
            // char index → UTF-16 위치
            let utf16_pos: u32 = para
                .text
                .chars()
                .take(*start)
                .map(|c| c.len_utf16() as u32)
                .sum();
            para.char_shapes
                .push(crate::model::paragraph::CharShapeRef {
                    start_pos: utf16_pos,
                    char_shape_id: *char_shape_id,
                });
        }
    }

    /// CSS 인라인 스타일 → CharShape ID 변환 (기존에서 검색 또는 신규 생성).
    pub(crate) fn css_to_char_shape_id(
        &mut self,
        css: &str,
        inherited_bold: bool,
        inherited_italic: bool,
        inherited_underline: bool,
    ) -> u32 {
        use crate::model::style::{CharShape, UnderlineType};

        // 기본 CharShape를 기반으로 수정
        let base_id = if !self.document.doc_info.char_shapes.is_empty() {
            0u32
        } else {
            self.document
                .doc_info
                .char_shapes
                .push(CharShape::default());
            0
        };
        let mut cs = self.document.doc_info.char_shapes[base_id as usize].clone();
        // 파싱된 문서의 CharShape 는 원본 CHAR_SHAPE 레코드 바이트를 raw_data 로 들고 있고
        // (parser/doc_info.rs), 직렬화기는 raw_data 가 있으면 필드 대신 그 바이트를 그대로
        // 쓴다(serializer/doc_info.rs). 아래에서 굵기·색·크기를 바꿔도 raw_data 를 비우지
        // 않으면 저장 시 원본 서식 바이트가 나가 붙여넣은 서식이 통째로 사라진다.
        // PartialEq 가 raw_data 를 비교에서 제외하므로 아래 중복 검색도 이를 걸러내지 못한다.
        // CharShapeMods::apply_to(model/style.rs)가 같은 이유로 첫 줄에서 raw_data 를 비운다.
        cs.raw_data = None;

        // CSS 속성 파싱 및 적용
        let css_lower = css.to_lowercase();

        // font-family
        if let Some(font_name) = parse_css_value(&css_lower, "font-family") {
            let clean_name = font_name
                .trim_matches(|c: char| c == '\'' || c == '"')
                .trim()
                .to_string();
            if !clean_name.is_empty() {
                if let Some(font_id) = self.find_font_id(&clean_name) {
                    cs.font_ids = [font_id; 7];
                }
            }
        }

        // font-size
        if let Some(size_str) = parse_css_value(&css_lower, "font-size") {
            if let Some(pt) = parse_pt_value(&size_str) {
                // pt → HWPUNIT: 1pt = 100 HWPUNIT (base_size 단위)
                cs.base_size = (pt * 100.0) as i32;
            }
        }

        // font-weight
        let is_bold = inherited_bold
            || css_lower.contains("font-weight:bold")
            || css_lower.contains("font-weight: bold")
            || css_lower.contains("font-weight:700")
            || css_lower.contains("font-weight: 700");
        cs.bold = is_bold;

        // font-style
        let is_italic = inherited_italic
            || css_lower.contains("font-style:italic")
            || css_lower.contains("font-style: italic");
        cs.italic = is_italic;

        // color
        if let Some(color_str) = parse_css_value(&css_lower, "color") {
            if let Some(bgr) = css_color_to_hwp_bgr(&color_str) {
                cs.text_color = bgr;
            }
        }

        // text-decoration
        let has_underline = inherited_underline
            || css_lower.contains("text-decoration:underline")
            || css_lower.contains("text-decoration: underline")
            || css_lower.contains("text-decoration-line:underline")
            || css_lower.contains("text-decoration-line: underline");
        cs.underline_type = if has_underline {
            UnderlineType::Bottom
        } else {
            UnderlineType::None
        };

        let has_strikethrough = css_lower.contains("text-decoration:line-through")
            || css_lower.contains("text-decoration: line-through")
            || css_lower.contains("line-through");
        cs.strikethrough = has_strikethrough;

        // 동일한 CharShape 검색
        for (i, existing) in self.document.doc_info.char_shapes.iter().enumerate() {
            if *existing == cs {
                return i as u32;
            }
        }

        // 새로 추가
        let new_id = self.document.doc_info.char_shapes.len() as u32;
        self.document.doc_info.char_shapes.push(cs);
        self.document.doc_info.raw_stream_dirty = true;
        // 스타일 세트 갱신
        self.rebuild_resolved_styles();
        new_id
    }

    /// CSS 인라인 스타일 → ParaShape ID 변환.
    pub(crate) fn css_to_para_shape_id(&mut self, css: &str) -> u16 {
        use crate::model::style::{Alignment, LineSpacingType};

        if css.is_empty() && !self.document.doc_info.para_shapes.is_empty() {
            return 0;
        }

        let base_id: u16 = 0;
        let mut ps = self
            .document
            .doc_info
            .para_shapes
            .get(base_id as usize)
            .cloned()
            .unwrap_or_default();
        // CharShape 쪽과 동일 — 원본 PARA_SHAPE 바이트를 비우지 않으면 정렬·줄간격 변경이
        // 저장 시 사라진다(ParaShapeMods::apply_to 와 같은 처리).
        ps.raw_data = None;

        let css_lower = css.to_lowercase();

        // text-align
        if let Some(align) = parse_css_value(&css_lower, "text-align") {
            ps.alignment = match align.trim() {
                "left" => Alignment::Left,
                "right" => Alignment::Right,
                "center" => Alignment::Center,
                "justify" => Alignment::Justify,
                _ => ps.alignment,
            };
        }

        // line-height
        if let Some(lh) = parse_css_value(&css_lower, "line-height") {
            let lh = lh.trim();
            if lh.ends_with('%') {
                if let Ok(pct) = lh.trim_end_matches('%').parse::<i32>() {
                    ps.line_spacing = pct;
                    ps.line_spacing_type = LineSpacingType::Percent;
                }
            } else if lh.ends_with("px") {
                if let Ok(px) = lh.trim_end_matches("px").parse::<f64>() {
                    // px → HWPUNIT (1px ≈ 75 HWPUNIT at 96dpi)
                    ps.line_spacing = (px * 7200.0 / 25.4 / (self.dpi / 25.4)).round() as i32;
                    ps.line_spacing_type = LineSpacingType::Fixed;
                }
            }
        }

        // 동일한 ParaShape 검색
        for (i, existing) in self.document.doc_info.para_shapes.iter().enumerate() {
            if *existing == ps {
                return i as u16;
            }
        }

        let new_id = self.document.doc_info.para_shapes.len() as u16;
        self.document.doc_info.para_shapes.push(ps);
        self.document.doc_info.raw_stream_dirty = true;
        self.rebuild_resolved_styles();
        new_id
    }

    /// 폰트 이름으로 font_faces에서 ID를 찾는다.
    pub(crate) fn find_font_id(&self, name: &str) -> Option<u16> {
        let name_lower = name.to_lowercase();
        // 한글 폰트 (인덱스 0)를 먼저, 영어 폰트 (인덱스 1)를 다음으로 검색
        for lang_idx in 0..self.document.doc_info.font_faces.len() {
            for (font_idx, font) in self.document.doc_info.font_faces[lang_idx]
                .iter()
                .enumerate()
            {
                if font.name.to_lowercase() == name_lower {
                    return Some(font_idx as u16);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::Document;
    use crate::model::style::{CharShape, ParaShape};

    /// 파싱을 거친 문서를 흉내낸다 — CharShape/ParaShape 가 원본 레코드 바이트를
    /// raw_data 로 들고 있는 상태(parser/doc_info.rs 가 하는 일).
    fn core_with_parsed_shapes() -> DocumentCore {
        let mut doc = Document::default();
        let mut cs = CharShape::default();
        cs.raw_data = Some(vec![0xAA; 72]);
        doc.doc_info.char_shapes.push(cs);
        let mut ps = ParaShape::default();
        ps.raw_data = Some(vec![0xBB; 54]);
        doc.doc_info.para_shapes.push(ps);
        let mut core = DocumentCore::new_empty();
        core.document = doc;
        core
    }

    // HTML 붙여넣기가 만드는 CharShape/ParaShape 는 char_shapes[0]/para_shapes[0] 의 clone
    // 이라 원본 raw_data 를 물고 온다. 직렬화기는 raw_data 가 있으면 필드 대신 그 바이트를
    // 그대로 쓰므로(serializer/doc_info.rs), 비우지 않으면 붙여넣은 서식이 저장 시 사라진다.
    // PartialEq 가 raw_data 를 제외하므로 중복 검색도 이를 걸러내지 못한다.

    #[test]
    fn html_paste_char_shape_drops_stale_raw_data() {
        let mut core = core_with_parsed_shapes();
        let id = core.css_to_char_shape_id("font-weight:bold;color:#ff0000", false, false, false);
        let cs = &core.document.doc_info.char_shapes[id as usize];
        assert!(cs.bold, "전제: CSS 가 반영돼야 함");
        assert!(
            cs.raw_data.is_none(),
            "raw_data 가 남으면 저장 시 원본 서식 바이트가 나가 붙여넣은 서식이 사라진다"
        );
    }

    #[test]
    fn html_paste_para_shape_drops_stale_raw_data() {
        let mut core = core_with_parsed_shapes();
        let id = core.css_to_para_shape_id("text-align:center");
        let ps = &core.document.doc_info.para_shapes[id as usize];
        assert!(
            ps.raw_data.is_none(),
            "raw_data 가 남으면 정렬·줄간격 변경이 저장 시 사라진다"
        );
    }
}

#[cfg(test)]
mod textdecoline_tests {
    use super::*;
    use crate::model::document::Document;
    use crate::model::style::{CharShape, UnderlineType};

    #[test]
    fn css_underline_recognizes_text_decoration_line_with_space() {
        let mut doc = Document::default();
        doc.doc_info.char_shapes.push(CharShape::default());
        let mut core = DocumentCore::new_empty();
        core.document = doc;

        let id = core.css_to_char_shape_id("text-decoration-line: underline", false, false, false);
        let cs = &core.document.doc_info.char_shapes[id as usize];
        assert_ne!(
            cs.underline_type,
            UnderlineType::None,
            "콜론 뒤 공백이 있는 text-decoration-line: underline 도 밑줄로 인식돼야 함"
        );
    }
}
