//! HTML 붙여넣기 + HTML 파싱 관련 native 메서드

use super::super::helpers::*;
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::paragraph::Paragraph;

/// parse_inline_content 가 서식으로 읽는 span·b·strong·i·em·u 태그면
/// (태그 이름, 닫는 태그인지)를 돌려준다.
fn inline_format_tag(tag_lower: &str) -> Option<(&str, bool)> {
    let tag = tag_lower.strip_prefix('<')?;
    let (closing, tag) = match tag.strip_prefix('/') {
        Some(rest) => (true, rest),
        None => (false, tag),
    };
    let name = tag.split(|c: char| !c.is_ascii_alphanumeric()).next()?;
    matches!(name, "span" | "b" | "strong" | "i" | "em" | "u").then_some((name, closing))
}

/// 인라인 구간 끝까지 닫히지 않은 서식 여는 태그들을 순서대로 이어 돌려준다.
fn open_format_tags(run: &str) -> String {
    let mut open: Vec<(String, &str)> = Vec::new();
    for (start, _) in run.match_indices('<') {
        let Some(end) = run[start..].find('>') else {
            break;
        };
        let tag = &run[start..=start + end];
        match inline_format_tag(&tag.to_lowercase()) {
            Some((name, true)) => {
                if let Some(i) = open.iter().rposition(|(open, _)| open == name) {
                    open.truncate(i);
                }
            }
            Some((name, false)) => open.push((name.to_string(), tag)),
            None => {}
        }
    }
    open.into_iter().map(|(_, tag)| tag).collect()
}

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

        // 글 없이 개체만 든 문단(표·단독 그림)이 있는지 확인한다.
        // 글 사이에 든 그림은 아래 다중 문단 경로가 글과 함께 캐럿 문단에 병합한다.
        let has_controls = parsed_paras
            .iter()
            .any(|p| p.text.is_empty() && !p.controls.is_empty());

        if has_controls {
            // 컨트롤 포함 문단은 merge 불가 → 직접 삽입
            let right_half =
                self.document.sections[section_idx].paragraphs[para_idx].split_at(char_offset);

            // 현재 문단 (왼쪽 반)이 비어있으면 첫 번째 파싱 문단으로 대체
            let left_empty = self.document.sections[section_idx].paragraphs[para_idx]
                .text
                .is_empty();

            let insert_idx = if left_empty {
                let host = &mut self.document.sections[section_idx].paragraphs[para_idx];
                if host.controls.is_empty() {
                    // 빈 왼쪽 문단을 첫 번째 파싱 문단으로 대체
                    *host = parsed_paras[0].clone();
                } else {
                    // 구역 첫 문단처럼 구역·단 정의가 든 빈 문단을 갈아 끼우면 그 정의가 사라진다.
                    // 첫 파싱 문단을 병합하고 문단 모양만 그 문단을 따른다.
                    host.merge_from(&parsed_paras[0]);
                    host.para_shape_id = parsed_paras[0].para_shape_id;
                }
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

            // 오른쪽 반에 글이나 개체가 있으면 새 문단으로 추가한다.
            // 글 없이 캐럿 뒤 그림·표만 든 오른쪽 반도 버리면 그 개체가 지워진다.
            let last_para_idx;
            let merge_point;
            if !right_half.text.is_empty() || !right_half.controls.is_empty() {
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
            // 역순(.rev()) 삽입은 composed 길이가 paragraphs 보다 짧은 시점에
            // index > len 으로 panic(문단+표 혼합 붙여넣기) 하고, panic 이 안 나도 기존/신규 항목이
            // 교차돼 composed[i] != paragraphs[i] 가 된다. 텍스트 분기(아래)와 같이 정방향으로 append.
            for i in para_idx + 1..=last_para_idx {
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
                    let keep_text = !(p.text.is_empty() || p.text == "\u{0002}");
                    let text = if !keep_text {
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
                    let gapped_offsets = std::mem::replace(
                        &mut p.char_offsets,
                        p.text
                            .chars()
                            .scan(0u32, |acc, c| {
                                let off = *acc;
                                *acc += c.len_utf16() as u32;
                                Some(off)
                            })
                            .collect(),
                    );
                    if keep_text {
                        // 글 사이 그림을 빼면 그 자리(8칸 갭)도 사라진다.
                        // 글자 모양 시작을 같은 글자의 새 위치로 옮긴다.
                        for cs in &mut p.char_shapes {
                            let idx = gapped_offsets.partition_point(|&off| off < cs.start_pos);
                            cs.start_pos =
                                p.char_offsets.get(idx).copied().unwrap_or(p.char_count - 1);
                        }
                    }
                }
                p
            })
            .collect()
    }

    /// 셀에 붙일 문단. 셀에는 개체를 넣지 않으므로 파싱하며 등록한 그림 데이터도 되돌린다.
    fn parse_html_to_cell_paragraphs(&mut self, html: &str) -> Vec<Paragraph> {
        let bin_content_len = self.document.bin_data_content.len();
        let bin_list_len = self.document.doc_info.bin_data_list.len();
        let parsed_paras = self.parse_html_to_paragraphs(html);
        self.document.bin_data_content.truncate(bin_content_len);
        self.document.doc_info.bin_data_list.truncate(bin_list_len);
        Self::normalize_html_paragraphs_for_cell_paste(parsed_paras)
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
        let parsed_paras = self.parse_html_to_cell_paragraphs(html);
        if parsed_paras.is_empty() {
            return Ok("{\"ok\":false,\"error\":\"empty html\"}".to_string());
        }

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

        let parsed_paras = self.parse_html_to_cell_paragraphs(html);
        if parsed_paras.is_empty() {
            return Ok("{\"ok\":false,\"error\":\"empty html\"}".to_string());
        }

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
    /// 400,000 은 실사용 한글 문서에 너무 빡빡하다 — 실측한 6쪽짜리 문서 하나가
    /// 정리 후 399,057바이트로 상한 바로 아래였고, 그림을 넣거나 문단이 조금만 늘면 넘겨
    /// 표·문단·그림이 통째로 평문이 됐다(실사용 신고). 실측 파싱 시간은 이 크기에서 1초 남짓이라
    /// 2MB(= 한글 본문 약 70만 자)로 올린다. data: URI 페이로드는 아래에서 따로 제외한다.
    const HTML_PASTE_MAX_BYTES: usize = 2_000_000;

    pub(crate) fn parse_html_to_paragraphs(&mut self, html: &str) -> Vec<Paragraph> {
        self.parse_html_to_paragraphs_at_depth(html, 0)
    }

    /// 크기 상한은 **태그 트리 복잡도**를 막으려는 것이므로 `data:` URI 로 실린
    /// 그림 바이트는 빼고 잰다. 안 그러면 그림이 있는 문서는 상한을 넘겨 평문으로 폴백해
    /// 표·문단·그림이 통째로 사라진다(한글 붙여넣기 실사용 신고 2026-09-03, 602KB 중 242KB 가 그림).
    fn html_markup_len(html: &str) -> usize {
        let mut payload = 0usize;
        let mut rest = html;
        while let Some(idx) = rest.find("data:") {
            let after = &rest[idx..];
            let end = after.find(['"', '\'', ' ', '>']).unwrap_or(after.len());
            payload += end;
            rest = &after[end..];
            if rest.is_empty() {
                break;
            }
        }
        html.len().saturating_sub(payload)
    }

    fn parse_html_to_paragraphs_at_depth(&mut self, html: &str, depth: u32) -> Vec<Paragraph> {
        if depth >= Self::HTML_PASTE_MAX_RECURSION_DEPTH
            || Self::html_markup_len(html) > Self::HTML_PASTE_MAX_BYTES
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
        // 블록 밖 인라인 구간: 글과 <span>·<b> 등 서식 태그 원문
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
                    self.flush_inline_run(&mut paragraphs, &mut pending_text);

                    // 표 전체 추출
                    let table_end = find_closing_tag_chars(&chars, pos, "table");
                    let table_html: String = chars[tag_start..table_end.min(len)].iter().collect();
                    self.parse_table_html(&mut paragraphs, &table_html);
                    pos = table_end;
                    continue;
                } else if tag_lower.starts_with("<img") {
                    self.flush_inline_run(&mut paragraphs, &mut pending_text);

                    self.parse_img_html(&mut paragraphs, &tag_str);
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("<p") {
                    self.flush_inline_run(&mut paragraphs, &mut pending_text);

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
                    self.flush_inline_run(&mut paragraphs, &mut pending_text);
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
                    self.flush_inline_run(&mut paragraphs, &mut pending_text);
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
                    self.flush_inline_run(&mut paragraphs, &mut pending_text);
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
                    // 그림만 든 항목도 버리지 않는다 — 버리면 등록한 그림 데이터만 문서에 남는다.
                    if !para.text.trim().is_empty() || !para.controls.is_empty() {
                        para.text = format!("• {}", para.text);
                        // 글머리 기호("• ")만큼 글자 위치와 스타일 구간을 오른쪽으로 민다.
                        // 다시 세지 않고 밀어야 글 사이 그림의 자리(8칸 갭)가 남는다.
                        // start_pos 는 UTF-16 코드유닛 단위(char_offsets 와 동일 축).
                        let bullet_len = "• ".encode_utf16().count() as u32;
                        para.char_offsets = (0..bullet_len)
                            .chain(para.char_offsets.iter().map(|off| off + bullet_len))
                            .collect();
                        para.char_count += bullet_len;
                        for cs in &mut para.char_shapes {
                            cs.start_pos += bullet_len;
                        }
                        paragraphs.push(para);
                    }
                    pos = li_end;
                    continue;
                } else if tag_lower.starts_with("<br") {
                    // <br> → 문단 구분. 서식 태그만 든 구간(<b><br></b>)도 빈 줄이다.
                    let blank = pending_text.is_empty() || pending_text.contains('<');
                    let reopen = open_format_tags(&pending_text);
                    let before = paragraphs.len();
                    self.flush_inline_run(&mut paragraphs, &mut pending_text);
                    if blank && paragraphs.len() == before {
                        paragraphs.push(Paragraph::default());
                    }
                    // <b>가<br>나</b> 의 "나" 도 굵게 남도록 열린 서식을 다음 줄에 다시 연다.
                    pending_text = reopen;
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("<span") {
                    // Chrome 은 문단 중간부터 고른 글을 <p> 없이 <span style>·<b style> 로 쓴다.
                    // 서식 태그를 앞뒤 글과 한 인라인 구간에 모아 한 문단으로 읽는다.
                    // span 은 안쪽 그림까지 통째로 넣는다.
                    let span_end = find_closing_tag_chars(&chars, pos, "span");
                    pending_text.extend(&chars[pos..span_end.min(len)]);
                    pos = span_end;
                    continue;
                } else if inline_format_tag(&tag_lower).is_some() {
                    pending_text.push_str(&tag_str);
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("</") {
                    // 닫는 태그 무시
                    pos = tag_end + 1;
                    continue;
                } else {
                    // 기타 태그 무시
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
        self.flush_inline_run(&mut paragraphs, &mut pending_text);

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

    /// 블록 밖 인라인 구간을 문단으로 만들고 비운다.
    /// 서식 태그가 있으면 `<p>` 안처럼 한 문단으로 읽어 서식을 살린다.
    /// 글뿐이면 종전처럼 줄마다 문단을 나눈다.
    fn flush_inline_run(&mut self, paragraphs: &mut Vec<Paragraph>, run: &mut String) {
        let run = std::mem::take(run);
        // 글 속 '<' 는 엔티티로 남으므로 '<' 가 있으면 서식 태그가 든 구간이다.
        if !run.contains('<') {
            self.flush_text_to_paragraphs(paragraphs, &run);
            return;
        }
        let mut para = Paragraph::default();
        self.parse_inline_content(&mut para, run.trim());
        if !para.text.trim().is_empty() || !para.controls.is_empty() {
            paragraphs.push(para);
        }
    }

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
        // (그림 앞 글자 수, 그림) — 글 사이에 든 그림
        let mut pictures: Vec<(usize, crate::model::image::Picture)> = Vec::new();

        let chars: Vec<char> = html.chars().collect();
        let len = chars.len();
        let mut pos = 0;

        // 열린 서식 요소(span·b·i·u 등)의 이름과 style. 서식은 닫힐 때까지 안쪽 글에 이어진다.
        let mut open_styles: Vec<(String, String)> = Vec::new();

        while pos < len {
            if chars[pos] == '<' {
                let tag_end = find_char(&chars, pos, '>');
                if tag_end >= len {
                    break;
                }

                let tag_str: String = chars[pos..=tag_end].iter().collect();
                let tag_lower = tag_str.to_lowercase();

                if let Some((name, closing)) = inline_format_tag(&tag_lower) {
                    // span 도 건너뛰지 않고 안쪽을 계속 읽는다.
                    // 그래야 span 자기 style 과 안쪽 <strong>·<u>·<br>·그림이 함께 남는다.
                    if closing {
                        if let Some(i) = open_styles.iter().rposition(|(open, _)| open == name) {
                            open_styles.truncate(i);
                        }
                    } else {
                        let mut css = parse_inline_style(&tag_str);
                        // Chrome 은 <b style="…"> 처럼 style 을 붙여 쓴다.
                        // style 이 그 속성을 정하면 따른다(Google 문서의 <b style="font-weight:normal">).
                        let implied = match name {
                            "b" | "strong" => Some(("font-weight", "bold")),
                            "i" | "em" => Some(("font-style", "italic")),
                            "u" => Some(("text-decoration", "underline")),
                            _ => None,
                        };
                        if let Some((property, value)) = implied {
                            if parse_css_value(&css.to_lowercase(), property).is_none() {
                                css = format!("{css};{property}:{value}");
                            }
                        }
                        open_styles.push((name.to_string(), css));
                    }
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("<br") {
                    full_text.push('\n');
                    pos = tag_end + 1;
                    continue;
                } else if tag_lower.starts_with("<img") {
                    // 글 사이 그림은 그 자리에 글자처럼 취급하는 그림으로 넣는다.
                    // 종전에는 기타 태그로 버려 `<p>앞<img>뒤</p>` 의 그림이 사라졌다.
                    if let Some(pic) =
                        crate::document_core::html_table_import::html_img_src(&tag_str)
                            .filter(|src| src.starts_with("data:"))
                            .and_then(|src| self.html_data_img_picture(&tag_str, src))
                    {
                        pictures.push((full_text.chars().count(), pic));
                    }
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
                    // 서식 구간 뒤의 평문도 구간을 새로 연다.
                    // 안 열면 앞 구간의 굵게 등이 문단 끝까지 이어진다.
                    if !open_styles.is_empty() || !style_runs.is_empty() {
                        // parse_css_value 는 처음 찾은 값을 쓰므로 안쪽 요소를 앞에 둔다.
                        let css: Vec<&str> = open_styles
                            .iter()
                            .rev()
                            .map(|(_, css)| css.as_str())
                            .collect();
                        let char_shape_id = self.css_to_char_shape_id(&css.join(";"));
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

        // 그림은 확장 제어문자 8칸을 차지한다. 그 자리를 char_offsets 의 갭으로 남겨야
        // control_text_positions 가 그림을 글 사이에 되짚는다.
        for (char_idx, pic) in pictures {
            para.shift_for_inline_control_insert(char_idx);
            para.char_count += 8;
            para.controls.push(Control::Picture(Box::new(pic)));
            para.ctrl_data_records.push(None);
            para.control_mask |= 0x0000_0800;
            para.has_para_text = true;
        }
    }

    /// CSS 인라인 스타일 → CharShape ID 변환 (기존에서 검색 또는 신규 생성).
    pub(crate) fn css_to_char_shape_id(&mut self, css: &str) -> u32 {
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
        //
        // 문서에 없는 글꼴이면 **새로 등록**한다.
        // 종전에는 `find_font_id` 가 못 찾으면 조용히 기본 글꼴로 떨어졌다 — 표본 문서의
        // `바탕` 이 양식의 `맑은 고딕` 으로 바뀌면서 줄 높이가 커져 **표 셀 안 둘째 줄이
        // 행 경계에 잘렸고**, 글자 폭이 넓어져 쪽수도 어긋났다.
        // 값은 대소문자를 보존해야 하므로 `css_lower` 가 아니라 원본 `css` 에서 읽는다
        // (속성 이름은 `parse_css_value` 가 완전일치로 비교하므로
        // `mso-fareast-font-family` 는 걸리지 않는다).
        //
        // 🔴 한글은 **동아시아 글꼴을 `mso-fareast-font-family` 에** 적는다. `font-family` 는
        // 라틴 글꼴이다(실측: `<span style="font-family:바탕;mso-fareast-font-family:바탕">` 도 있고
        // `mso-fareast-font-family` 만 있는 span 도 많다). 라틴만 읽으면 **본문 한글이 통째로
        // 대상 문서 기본 글꼴로 떨어진다** — 표 셀 글자가 고딕으로 바뀌어 줄 높이가 커지고
        // 셀에서 잘렸다.
        // 슬롯 순서는 `serializer/hwpx/canonical_defaults.rs` FONTFACE_LANG_NAMES 와 같다:
        // 0 HANGUL · 1 LATIN · 2 HANJA · 3 JAPANESE · 4 OTHER · 5 SYMBOL · 6 USER.
        let first_family = |value: String| -> String {
            value
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches(|c: char| c == '\'' || c == '"')
                .trim()
                .to_string()
        };
        let latin_name = parse_css_value(css, "font-family")
            .map(first_family)
            .filter(|n| !n.is_empty());
        let east_name = parse_css_value(css, "mso-fareast-font-family")
            .map(first_family)
            .filter(|n| !n.is_empty());
        let east_ids = east_name
            .as_deref()
            .or(latin_name.as_deref())
            .and_then(|n| self.find_or_register_font_ids(n));
        let latin_ids = latin_name
            .as_deref()
            .or(east_name.as_deref())
            .and_then(|n| self.find_or_register_font_ids(n));
        if let Some(ids) = east_ids {
            for lang_idx in [0usize, 2, 3] {
                cs.font_ids[lang_idx] = ids[lang_idx];
            }
        }
        if let Some(ids) = latin_ids {
            for lang_idx in [1usize, 4, 5, 6] {
                cs.font_ids[lang_idx] = ids[lang_idx];
            }
        }

        // font-size
        if let Some(size_str) = parse_css_value(&css_lower, "font-size") {
            if let Some(pt) = parse_pt_value(&size_str) {
                // pt → HWPUNIT: 1pt = 100 HWPUNIT (base_size 단위)
                cs.base_size = (pt * 100.0) as i32;
            }
        }

        // letter-spacing(자간)
        //
        // 한글은 줄을 맞추려고 글자마다 `letter-spacing:-0.2pt` 같은 음수 자간을 넣어 내보낸다.
        // 이를 버리면 같은 글이 원본보다 넓어져 **줄바꿈과 쪽수가 어긋난다**
        // (실측: 원본 11쪽짜리 요약이 붙여넣기 뒤 12쪽, 문단마다 줄 하나씩 늘어남).
        // `CharShape.spacings` 는 글꼴 크기 대비 **퍼센트**다
        // (`renderer/style_resolver.rs`: letter_spacing_px = font_size × spacing / 100).
        // 한글 자간 입력 범위(±50%)로 자른다.
        if let Some(ls_str) = parse_css_value(&css_lower, "letter-spacing") {
            if let Some(pt) = parse_pt_value(&ls_str) {
                let base_pt = (f64::from(cs.base_size) / 100.0).max(1.0);
                let pct = (pt / base_pt * 100.0).round().clamp(-50.0, 50.0) as i8;
                cs.spacings = [pct; 7];
            }
        }

        // font-weight — 안쪽 요소의 값이 앞에 오므로 처음 찾은 값을 따른다.
        // GitHub 등은 <strong> 을 600 으로 쓰므로 600 이상을 굵게 본다.
        cs.bold = parse_css_value(&css_lower, "font-weight")
            .is_some_and(|w| w.starts_with("bold") || w.parse::<u16>().is_ok_and(|n| n >= 600));

        // font-style
        let is_italic =
            css_lower.contains("font-style:italic") || css_lower.contains("font-style: italic");
        cs.italic = is_italic;

        // color
        if let Some(color_str) = parse_css_value(&css_lower, "color") {
            if let Some(bgr) = css_color_to_hwp_bgr(&color_str) {
                cs.text_color = bgr;
            }
        }

        // text-decoration
        let has_underline = css_lower.contains("text-decoration:underline")
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

        // 여백·들여쓰기 — 한글은 개조식 문단을 `margin-left:15pt;text-indent:-15pt`
        // 로 내보낸다. 종전에는 이 둘을 무시해 **둘째 줄부터 왼쪽으로 튀어나왔다**.
        // CSS 길이(pt·px·cm·mm) → ParaShape 여백 단위(= HWPUNIT × 2).
        //
        // 🔴 ParaShape 의 margin_left/right·indent·spacing_* 는 **HWPUNIT 의 2배 스케일**로
        // 저장한다(`renderer/style_resolver.rs` 가 `/2` 로 되돌린다. HWPX 파서도
        // `hp:case`(HwpUnitChar) 값을 2배로 올려 읽는다 — `parser/hwpx/header.rs`).
        // 1배로 넣으면 방향은 맞고 **크기가 정확히 절반**이 되어, 개조식 문단의 둘째 줄이
        // 글머리표 아래로 덜 들어가 "왼쪽으로 튀어나온" 것처럼 보인다.
        // 실측(2026-09-04): CSS `margin-left:30.3pt;text-indent:-30.3pt`
        // → 1배 −3030 이면 내어쓰기 20.2px, 한글 원본(`hp:default` −6060)은 40.4px.
        let css_len_to_hwpunit = |value: &str| -> Option<i32> {
            let v = value.trim();
            let (num, unit): (&str, &str) = if let Some(rest) = v.strip_suffix("pt") {
                (rest, "pt")
            } else if let Some(rest) = v.strip_suffix("px") {
                (rest, "px")
            } else if let Some(rest) = v.strip_suffix("cm") {
                (rest, "cm")
            } else if let Some(rest) = v.strip_suffix("mm") {
                (rest, "mm")
            } else {
                (v, "pt")
            };
            let n: f64 = num.trim().parse().ok()?;
            // 마지막 ×2 가 위에서 말한 ParaShape IR 스케일이다.
            Some(match unit {
                "px" => (n * 72.0 / 96.0 * 200.0).round() as i32,
                "cm" => (n * 72.0 / 2.54 * 200.0).round() as i32,
                "mm" => (n * 72.0 / 25.4 * 200.0).round() as i32,
                _ => (n * 200.0).round() as i32,
            })
        };
        let css_margin_left =
            parse_css_value(&css_lower, "margin-left").and_then(|v| css_len_to_hwpunit(&v));
        let css_text_indent =
            parse_css_value(&css_lower, "text-indent").and_then(|v| css_len_to_hwpunit(&v));
        if let Some(v) = parse_css_value(&css_lower, "margin-right") {
            if let Some(hu) = css_len_to_hwpunit(&v) {
                ps.margin_right = hu.max(0);
            }
        }
        if css_margin_left.is_some() || css_text_indent.is_some() {
            // CSS: 첫 줄 x = margin-left + text-indent, 나머지 = margin-left.
            // HWP: 첫 줄 x = left(+indent 가 양수면 더함), 내어쓰기(indent<0)면 나머지가 left+|indent|.
            // 따라서 left = margin-left + text-indent(첫 줄 위치), indent = text-indent 로 옮긴다.
            let ml = css_margin_left.unwrap_or(0);
            let ti = css_text_indent.unwrap_or(0);
            ps.margin_left = (ml + ti).max(0);
            ps.indent = ti;
            // 평문 표기를 유지하되, 여백은 직렬화기가 공통 IR에서 물리 단위로 되돌린다.
            ps.hwpx_plain_para_margin = true;
        }
        if let Some(v) = parse_css_value(&css_lower, "margin-top") {
            if let Some(hu) = css_len_to_hwpunit(&v) {
                ps.spacing_before = hu.max(0);
                ps.hwpx_plain_para_margin = true;
            }
        }
        if let Some(v) = parse_css_value(&css_lower, "margin-bottom") {
            if let Some(hu) = css_len_to_hwpunit(&v) {
                ps.spacing_after = hu.max(0);
                ps.hwpx_plain_para_margin = true;
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

    /// 글꼴 이름으로 ID 를 찾고, 문서에 없으면 **새로 등록**해 그 ID 를 돌려준다.
    ///
    /// 붙여넣기 원본의 글꼴이 대상 문서에 없을 때 조용히 기본 글꼴로
    /// 떨어지면 줄 높이·글자 폭이 달라져 **표 셀 글자가 잘리고 쪽수가 어긋난다**
    /// (표본: 본문 글꼴 `바탕` 이 대상 문서 기본 글꼴 `맑은 고딕` 으로 바뀌었다).
    /// `Font` 는 `raw_data` 없이도 HWPX(`serializer/hwpx/header.rs`)·
    /// HWP5(`serializer/hwp5/doc_info.rs` `serialize_face_name`) 양쪽이 모델 필드로 써낸다.
    ///
    /// 🔴 언어 슬롯마다 **글꼴 목록이 다르다**(실측한 문서: HANGUL·LATIN 8개, OTHER·USER 7개.
    /// 같은 index 가 슬롯마다 다른 글꼴을 가리킨다).
    /// 그래서 하나의 ID 를 7칸에 복사하면 안 되고, **슬롯마다 찾거나 넣어서 각자의 index** 를 쓴다.
    fn find_or_register_font_ids(&mut self, name: &str) -> Option<[u16; 7]> {
        let trimmed = name.trim();
        // 총칭 글꼴은 등록하지 않는다 — 이름이 아니라 분류다.
        const GENERIC: [&str; 8] = [
            "serif",
            "sans-serif",
            "monospace",
            "cursive",
            "fantasy",
            "system-ui",
            "-apple-system",
            "inherit",
        ];
        if trimmed.is_empty()
            || trimmed.chars().count() > 64
            || GENERIC.contains(&trimmed.to_lowercase().as_str())
        {
            return None;
        }
        let name_lower = trimmed.to_lowercase();
        let faces = &mut self.document.doc_info.font_faces;
        if faces.is_empty() {
            return None;
        }
        let slot_count = faces.len().min(7);
        let mut ids = [0u16; 7];
        for (lang_idx, slot) in faces.iter_mut().take(7).enumerate() {
            let idx = match slot
                .iter()
                .position(|font| font.name.to_lowercase() == name_lower)
            {
                Some(found) => found,
                None => {
                    // 한 문서에 무한정 늘리지 않는다(중복·악성 CSS 방어).
                    if slot.len() >= 256 {
                        return None;
                    }
                    slot.push(crate::model::style::Font {
                        name: trimmed.to_string(),
                        alt_type: 1, // TTF
                        ..Default::default()
                    });
                    slot.len() - 1
                }
            };
            ids[lang_idx] = u16::try_from(idx).ok()?;
        }
        // 슬롯이 7개보다 적은 문서는 남은 칸을 첫 슬롯 값으로 채운다.
        let first = ids[0];
        for id in ids.iter_mut().skip(slot_count) {
            *id = first;
        }
        Some(ids)
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
        let id = core.css_to_char_shape_id("font-weight:bold;color:#ff0000");
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

        let id = core.css_to_char_shape_id("text-decoration-line: underline");
        let cs = &core.document.doc_info.char_shapes[id as usize];
        assert_ne!(
            cs.underline_type,
            UnderlineType::None,
            "콜론 뒤 공백이 있는 text-decoration-line: underline 도 밑줄로 인식돼야 함"
        );
    }
}
