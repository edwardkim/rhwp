//! HTML 붙여넣기 + HTML 파싱 관련 native 메서드

use super::super::helpers::*;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;
use crate::renderer::style_resolver::resolve_styles;

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

            self.reflow_paragraph(section_idx, para_idx);
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

            let mut insert_idx = if left_empty {
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
        // 셀 접근 검증
        let cell_para_count = {
            let section =
                self.document.sections.get(section_idx).ok_or_else(|| {
                    HwpError::RenderError(format!("구역 {} 범위 초과", section_idx))
                })?;
            let para = section.paragraphs.get(parent_para_idx).ok_or_else(|| {
                HwpError::RenderError(format!("문단 {} 범위 초과", parent_para_idx))
            })?;
            let table = match para.controls.get(control_idx) {
                Some(Control::Table(t)) => t,
                _ => return Err(HwpError::RenderError("표가 아님".to_string())),
            };
            let cell = table
                .cells
                .get(cell_idx)
                .ok_or_else(|| HwpError::RenderError(format!("셀 {} 범위 초과", cell_idx)))?;
            cell.paragraphs.len()
        };
        if cell_para_idx >= cell_para_count {
            return Err(HwpError::RenderError(format!(
                "셀 문단 {} 범위 초과",
                cell_para_idx
            )));
        }

        let parsed_paras = self.parse_html_to_paragraphs(html);
        if parsed_paras.is_empty() {
            return Ok("{\"ok\":false,\"error\":\"empty html\"}".to_string());
        }

        self.document.sections[section_idx].raw_stream = None;

        let clip_count = parsed_paras.len();

        // 셀 내부에는 Table Control 중첩 불가 → 컨트롤 포함 문단은 텍스트만 추출
        let parsed_paras: Vec<Paragraph> = parsed_paras
            .into_iter()
            .map(|mut p| {
                if !p.controls.is_empty() {
                    // 컨트롤 문단은 텍스트로 대체
                    let text = if p.text.is_empty() || p.text == "\u{0002}" {
                        // Table/Picture 등 컨트롤 → 셀 텍스트 추출
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
                    p.char_count = p.text.encode_utf16().count() as u32;
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
            .collect();
        let clip_count = parsed_paras.len();

        let cell_paras = {
            let section = &mut self.document.sections[section_idx];
            let para = &mut section.paragraphs[parent_para_idx];
            let table = match &mut para.controls[control_idx] {
                Control::Table(t) => t,
                _ => unreachable!(),
            };
            &mut table.cells[cell_idx].paragraphs
        };

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

            let _ = cell_paras;
            self.reflow_cell_paragraph(
                section_idx,
                parent_para_idx,
                control_idx,
                cell_idx,
                cell_para_idx,
            );
            // 부모 표 dirty 마킹 + 재페이지네이션 (셀 편집 → composed 불변)
            if let Some(Control::Table(t)) = self.document.sections[section_idx].paragraphs
                [parent_para_idx]
                .controls
                .get_mut(control_idx)
            {
                t.dirty = true;
            }
            self.mark_section_dirty(section_idx);
            self.paginate_if_needed();

            let new_offset = char_offset + new_chars;
            self.event_log.push(DocumentEvent::HtmlImported {
                section: section_idx,
                para: parent_para_idx,
            });
            return Ok(format!(
                "{{\"ok\":true,\"cellParaIdx\":{},\"charOffset\":{}}}",
                cell_para_idx, new_offset
            ));
        }

        // 다중 문단
        let right_half = cell_paras[cell_para_idx].split_at(char_offset);
        cell_paras[cell_para_idx].merge_from(&parsed_paras[0]);

        let mut insert_idx = cell_para_idx + 1;
        for i in 1..clip_count {
            cell_paras.insert(insert_idx, parsed_paras[i].clone());
            insert_idx += 1;
        }

        let last_para_idx = insert_idx - 1;
        let merge_point = cell_paras[last_para_idx].merge_from(&right_half);

        let _ = cell_paras;
        for i in cell_para_idx..=last_para_idx {
            self.reflow_cell_paragraph(section_idx, parent_para_idx, control_idx, cell_idx, i);
        }
        // 부모 표 dirty 마킹 + 재페이지네이션 (셀 편집 → composed 불변)
        if let Some(Control::Table(t)) = self.document.sections[section_idx].paragraphs
            [parent_para_idx]
            .controls
            .get_mut(control_idx)
        {
            t.dirty = true;
        }
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
    pub(crate) fn parse_html_to_paragraphs(&mut self, html: &str) -> Vec<Paragraph> {
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
                        let sub_paras = self.parse_html_to_paragraphs(p_inner);
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

                    let sub_paras = self.parse_html_to_paragraphs(div_inner);
                    paragraphs.extend(sub_paras);
                    pos = div_end;
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
                        // <span>...</span> 인라인 콘텐츠
                        let span_end = find_closing_tag_chars(&chars, pos, "span");
                        let span_full: String =
                            chars[tag_start..span_end.min(len)].iter().collect();
                        let span_full = if let Some(idx) = span_full.rfind("</span>") {
                            &span_full[..idx]
                        } else {
                            &span_full
                        };
                        // span 태그 내부 텍스트 추출
                        if let Some(gt_pos) = span_full.find('>') {
                            pending_text.push_str(&span_full[gt_pos + 1..]);
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

        // 빈 결과 시 최소 처리
        if paragraphs.is_empty() {
            let plain = html_to_plain_text(html);
            if !plain.is_empty() {
                let mut para = Paragraph::default();
                para.text = plain;
                para.char_count = para.text.encode_utf16().count() as u32;
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

        paragraphs
    }

    /// 텍스트를 문단으로 변환하여 추가한다 (줄바꿈 기준 분리).
    pub(crate) fn flush_text_to_paragraphs(&self, paragraphs: &mut Vec<Paragraph>, text: &str) {
        let decoded = decode_html_entities(text);
        for line in decoded.split('\n') {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let mut para = Paragraph::default();
            para.text = trimmed.to_string();
            para.char_count = para.text.encode_utf16().count() as u32;
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
                    let span_end_tag = find_closing_tag_chars(&chars, pos, "span");
                    let inner_start = tag_end + 1;
                    let inner_end = {
                        // char 배열에서 "</span>" 검색 (바이트 인덱스 혼동 방지)
                        let close_chars: Vec<char> = "</span>".chars().collect();
                        let mut found = None;
                        for i in inner_start..len.saturating_sub(close_chars.len() - 1) {
                            let slice: String = chars[i..i + close_chars.len().min(len - i)]
                                .iter()
                                .collect();
                            if slice.to_lowercase() == "</span>" {
                                found = Some(i);
                                break;
                            }
                        }
                        found.unwrap_or(span_end_tag)
                    };
                    let inner: String = chars[inner_start..inner_end.min(len)].iter().collect();
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
        para.char_count = para.text.encode_utf16().count() as u32;
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
            || css_lower.contains("text-decoration-line:underline");
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
        self.styles = resolve_styles(&self.document.doc_info, self.dpi);
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
        self.styles = resolve_styles(&self.document.doc_info, self.dpi);
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
