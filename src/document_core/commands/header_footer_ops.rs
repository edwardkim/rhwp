//! 머리말/꼬리말 생성·조회·텍스트 편집 관련 native 메서드

use crate::document_core::helpers::{
    build_tab_def_from_json, json_has_border_keys, json_has_tab_keys, parse_json_i16_array,
    parse_para_shape_mods,
};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::Control;
use crate::model::event::DocumentEvent;
use crate::model::header_footer::{Footer, Header, HeaderFooterApply};
use crate::model::paragraph::Paragraph;
use crate::renderer::composer::reflow_line_segs;

/// applyTo u8 값 → HeaderFooterApply 변환
fn apply_from_u8(v: u8) -> HeaderFooterApply {
    match v {
        1 => HeaderFooterApply::Even,
        2 => HeaderFooterApply::Odd,
        _ => HeaderFooterApply::Both,
    }
}

/// HeaderFooterApply → u8 변환
fn apply_to_u8(a: HeaderFooterApply) -> u8 {
    match a {
        HeaderFooterApply::Both => 0,
        HeaderFooterApply::Even => 1,
        HeaderFooterApply::Odd => 2,
    }
}

/// HeaderFooterApply → 표시 레이블
fn apply_label(a: HeaderFooterApply) -> &'static str {
    match a {
        HeaderFooterApply::Both => "양 쪽",
        HeaderFooterApply::Even => "짝수 쪽",
        HeaderFooterApply::Odd => "홀수 쪽",
    }
}

impl DocumentCore {
    /// 구역의 문단들에서 특정 apply_to의 머리말 또는 꼬리말 컨트롤 위치를 찾는다.
    /// 반환: (para_index, control_index)
    fn find_header_footer_control(
        &self,
        section_idx: usize,
        is_header: bool,
        apply_to: HeaderFooterApply,
    ) -> Option<(usize, usize)> {
        let section = self.document.sections.get(section_idx)?;
        for (pi, para) in section.paragraphs.iter().enumerate() {
            for (ci, ctrl) in para.controls.iter().enumerate() {
                match ctrl {
                    Control::Header(h) if is_header && h.apply_to == apply_to => {
                        return Some((pi, ci));
                    }
                    Control::Footer(f) if !is_header && f.apply_to == apply_to => {
                        return Some((pi, ci));
                    }
                    _ => {}
                }
            }
        }
        None
    }

    /// 머리말/꼬리말 조회 — JSON 반환
    ///
    /// 존재하면: `{"ok":true,"exists":true,"applyTo":0,"paraCount":N,"text":"..."}`
    /// 없으면: `{"ok":true,"exists":false}`
    pub fn get_header_footer_native(
        &self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let apply = apply_from_u8(apply_to);
        if let Some((pi, ci)) = self.find_header_footer_control(section_idx, is_header, apply) {
            let section = &self.document.sections[section_idx];
            let ctrl = &section.paragraphs[pi].controls[ci];
            let (paragraphs, at) = match ctrl {
                Control::Header(h) => (&h.paragraphs, h.apply_to),
                Control::Footer(f) => (&f.paragraphs, f.apply_to),
                _ => unreachable!(),
            };
            let text: String = paragraphs
                .iter()
                .map(|p| p.text.clone())
                .collect::<Vec<_>>()
                .join("\n");
            let kind = if is_header { "header" } else { "footer" };
            let label = apply_label(at);
            Ok(format!(
                "{{\"ok\":true,\"exists\":true,\"kind\":\"{}\",\"applyTo\":{},\"label\":\"{}\",\"paraIndex\":{},\"controlIndex\":{},\"paraCount\":{},\"text\":\"{}\"}}",
                kind, apply_to_u8(at), label, pi, ci, paragraphs.len(),
                super::super::helpers::json_escape(&text)
            ))
        } else {
            Ok(format!("{{\"ok\":true,\"exists\":false}}"))
        }
    }

    /// 머리말/꼬리말 생성 — 빈 문단 1개 포함
    ///
    /// 이미 같은 apply_to의 머리말/꼬리말이 있으면 에러.
    /// 구역의 첫 번째 문단(SectionDef 컨트롤이 있는 문단)에 컨트롤을 추가한다.
    pub fn create_header_footer_native(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let apply = apply_from_u8(apply_to);
        if self
            .find_header_footer_control(section_idx, is_header, apply)
            .is_some()
        {
            let kind = if is_header { "머리말" } else { "꼬리말" };
            return Err(HwpError::RenderError(format!(
                "이미 {}({})이 존재합니다",
                kind,
                apply_label(apply)
            )));
        }

        // 빈 문단 생성
        let empty_para = Paragraph::default();

        // 컨트롤 생성
        let ctrl = if is_header {
            Control::Header(Box::new(Header {
                apply_to: apply,
                paragraphs: vec![empty_para],
                raw_attr: apply_to as u32,
                raw_ctrl_extra: Vec::new(),
            }))
        } else {
            Control::Footer(Box::new(Footer {
                apply_to: apply,
                paragraphs: vec![empty_para],
                raw_attr: apply_to as u32,
                raw_ctrl_extra: Vec::new(),
            }))
        };

        // 구역의 첫 번째 문단에 컨트롤 추가 (SectionDef 컨트롤이 있는 곳)
        let section = &mut self.document.sections[section_idx];
        if section.paragraphs.is_empty() {
            return Err(HwpError::RenderError("구역에 문단이 없습니다".to_string()));
        }
        section.paragraphs[0].controls.push(ctrl);
        // 컨트롤 1개 = UTF-16 8 code units → char_count 갱신
        section.paragraphs[0].char_count += 8;
        section.raw_stream = None;

        // 재페이지네이션 (머리말/꼬리말이 추가되면 페이지 레이아웃에 영향)
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        // 생성된 컨트롤 위치 반환
        let (pi, ci) = self
            .find_header_footer_control(section_idx, is_header, apply)
            .expect("방금 생성한 컨트롤을 찾을 수 없음");

        let kind = if is_header { "header" } else { "footer" };
        let label = apply_label(apply);
        Ok(format!(
            "{{\"ok\":true,\"kind\":\"{}\",\"applyTo\":{},\"label\":\"{}\",\"paraIndex\":{},\"controlIndex\":{}}}",
            kind, apply_to, label, pi, ci
        ))
    }

    /// 머리말/꼬리말 내부 문단에 대한 가변 참조를 얻는다.
    fn get_hf_paragraph_mut(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
    ) -> Result<&mut Paragraph, HwpError> {
        let apply = apply_from_u8(apply_to);
        let (pi, ci) = self
            .find_header_footer_control(section_idx, is_header, apply)
            .ok_or_else(|| {
                let kind = if is_header { "머리말" } else { "꼬리말" };
                HwpError::RenderError(format!(
                    "{}({})이 존재하지 않습니다",
                    kind,
                    apply_label(apply)
                ))
            })?;

        let ctrl = &mut self.document.sections[section_idx].paragraphs[pi].controls[ci];
        match ctrl {
            Control::Header(h) => {
                if hf_para_idx >= h.paragraphs.len() {
                    return Err(HwpError::RenderError(format!(
                        "머리말 문단 인덱스 {} 범위 초과 (총 {}개)",
                        hf_para_idx,
                        h.paragraphs.len()
                    )));
                }
                Ok(&mut h.paragraphs[hf_para_idx])
            }
            Control::Footer(f) => {
                if hf_para_idx >= f.paragraphs.len() {
                    return Err(HwpError::RenderError(format!(
                        "꼬리말 문단 인덱스 {} 범위 초과 (총 {}개)",
                        hf_para_idx,
                        f.paragraphs.len()
                    )));
                }
                Ok(&mut f.paragraphs[hf_para_idx])
            }
            _ => Err(HwpError::RenderError(
                "컨트롤이 머리말/꼬리말이 아닙니다".to_string(),
            )),
        }
    }

    /// 머리말/꼬리말 내부 문단에 대한 불변 참조를 얻는다.
    fn get_hf_paragraph_ref(
        &self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
    ) -> Option<&Paragraph> {
        let apply = apply_from_u8(apply_to);
        let (pi, ci) = self.find_header_footer_control(section_idx, is_header, apply)?;
        let ctrl = &self.document.sections[section_idx].paragraphs[pi].controls[ci];
        match ctrl {
            Control::Header(h) => h.paragraphs.get(hf_para_idx),
            Control::Footer(f) => f.paragraphs.get(hf_para_idx),
            _ => None,
        }
    }

    /// 머리말/꼬리말 내 텍스트 삽입
    pub fn insert_text_in_header_footer_native(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
        char_offset: usize,
        text: &str,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }

        let hf_para = self.get_hf_paragraph_mut(section_idx, is_header, apply_to, hf_para_idx)?;
        let new_chars_count = text.chars().count();
        hf_para.insert_text_at(char_offset, text);

        // 리플로우 (머리말/꼬리말 영역 폭 기반)
        self.reflow_hf_paragraph(section_idx, is_header, apply_to, hf_para_idx);

        // raw 스트림 무효화, 재페이지네이션
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        let new_offset = char_offset + new_chars_count;
        self.event_log.push(DocumentEvent::TextInserted {
            section: section_idx,
            para: 0,
            offset: char_offset,
            len: new_chars_count,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            new_offset
        )))
    }

    /// 머리말/꼬리말 내 텍스트 삭제
    pub fn delete_text_in_header_footer_native(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
        char_offset: usize,
        count: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }

        let hf_para = self.get_hf_paragraph_mut(section_idx, is_header, apply_to, hf_para_idx)?;
        hf_para.delete_text_at(char_offset, count);

        // 리플로우
        self.reflow_hf_paragraph(section_idx, is_header, apply_to, hf_para_idx);

        // raw 스트림 무효화, 재페이지네이션
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::TextDeleted {
            section: section_idx,
            para: 0,
            offset: char_offset,
            count,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            char_offset
        )))
    }

    /// 머리말/꼬리말 내 문단 분할 (Enter 키)
    pub fn split_paragraph_in_header_footer_native(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
        char_offset: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }

        let apply = apply_from_u8(apply_to);
        let (pi, ci) = self
            .find_header_footer_control(section_idx, is_header, apply)
            .ok_or_else(|| {
                let kind = if is_header { "머리말" } else { "꼬리말" };
                HwpError::RenderError(format!("{}이 존재하지 않습니다", kind))
            })?;

        // 문단 분할
        let new_para = {
            let ctrl = &mut self.document.sections[section_idx].paragraphs[pi].controls[ci];
            let paragraphs = match ctrl {
                Control::Header(h) => &mut h.paragraphs,
                Control::Footer(f) => &mut f.paragraphs,
                _ => return Err(HwpError::RenderError("컨트롤 타입 불일치".to_string())),
            };
            if hf_para_idx >= paragraphs.len() {
                return Err(HwpError::RenderError(format!(
                    "문단 인덱스 {} 범위 초과",
                    hf_para_idx
                )));
            }
            paragraphs[hf_para_idx].split_at(char_offset)
        };

        // 새 문단 삽입
        let new_para_idx = hf_para_idx + 1;
        {
            let ctrl = &mut self.document.sections[section_idx].paragraphs[pi].controls[ci];
            let paragraphs = match ctrl {
                Control::Header(h) => &mut h.paragraphs,
                Control::Footer(f) => &mut f.paragraphs,
                _ => unreachable!(),
            };
            paragraphs.insert(new_para_idx, new_para);
        }

        // 리플로우
        self.reflow_hf_paragraph(section_idx, is_header, apply_to, hf_para_idx);
        self.reflow_hf_paragraph(section_idx, is_header, apply_to, new_para_idx);

        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::ParagraphSplit {
            section: section_idx,
            para: hf_para_idx,
            offset: char_offset,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"hfParaIndex\":{},\"charOffset\":0",
            new_para_idx
        )))
    }

    /// 머리말/꼬리말 내 문단 병합 (Backspace at start)
    pub fn merge_paragraph_in_header_footer_native(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        if hf_para_idx == 0 {
            return Err(HwpError::RenderError(
                "첫 번째 문단은 이전 문단과 병합할 수 없습니다".to_string(),
            ));
        }

        let apply = apply_from_u8(apply_to);
        let (pi, ci) = self
            .find_header_footer_control(section_idx, is_header, apply)
            .ok_or_else(|| {
                let kind = if is_header { "머리말" } else { "꼬리말" };
                HwpError::RenderError(format!("{}이 존재하지 않습니다", kind))
            })?;

        // 병합
        let merge_offset;
        {
            let ctrl = &mut self.document.sections[section_idx].paragraphs[pi].controls[ci];
            let paragraphs = match ctrl {
                Control::Header(h) => &mut h.paragraphs,
                Control::Footer(f) => &mut f.paragraphs,
                _ => return Err(HwpError::RenderError("컨트롤 타입 불일치".to_string())),
            };
            if hf_para_idx >= paragraphs.len() {
                return Err(HwpError::RenderError(format!(
                    "문단 인덱스 {} 범위 초과",
                    hf_para_idx
                )));
            }
            merge_offset = paragraphs[hf_para_idx - 1].text.chars().count();
            let removed = paragraphs.remove(hf_para_idx);
            paragraphs[hf_para_idx - 1].merge_from(&removed);
        }

        let prev_idx = hf_para_idx - 1;
        self.reflow_hf_paragraph(section_idx, is_header, apply_to, prev_idx);

        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        self.event_log.push(DocumentEvent::ParagraphMerged {
            section: section_idx,
            para: hf_para_idx,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"hfParaIndex\":{},\"charOffset\":{}",
            prev_idx, merge_offset
        )))
    }

    /// 머리말/꼬리말 문단의 정보를 반환한다 (문단 수, 현재 문단 텍스트 길이 등).
    pub fn get_header_footer_para_info_native(
        &self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        let apply = apply_from_u8(apply_to);
        let (pi, ci) = self
            .find_header_footer_control(section_idx, is_header, apply)
            .ok_or_else(|| {
                let kind = if is_header { "머리말" } else { "꼬리말" };
                HwpError::RenderError(format!("{}이 존재하지 않습니다", kind))
            })?;

        let ctrl = &self.document.sections[section_idx].paragraphs[pi].controls[ci];
        let paragraphs = match ctrl {
            Control::Header(h) => &h.paragraphs,
            Control::Footer(f) => &f.paragraphs,
            _ => return Err(HwpError::RenderError("컨트롤 타입 불일치".to_string())),
        };

        let para_count = paragraphs.len();
        let char_count = if hf_para_idx < para_count {
            paragraphs[hf_para_idx].text.chars().count()
        } else {
            0
        };

        Ok(format!(
            "{{\"ok\":true,\"paraCount\":{},\"charCount\":{}}}",
            para_count, char_count
        ))
    }

    /// 머리말/꼬리말 삭제 (컨트롤 자체를 제거)
    pub fn delete_header_footer_native(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과 (총 {}개)",
                section_idx,
                self.document.sections.len()
            )));
        }
        let apply = apply_from_u8(apply_to);
        let (pi, ci) = self
            .find_header_footer_control(section_idx, is_header, apply)
            .ok_or_else(|| {
                let kind = if is_header { "머리말" } else { "꼬리말" };
                HwpError::RenderError(format!(
                    "{}({})이 존재하지 않습니다",
                    kind,
                    apply_label(apply)
                ))
            })?;

        self.document.sections[section_idx].paragraphs[pi]
            .controls
            .remove(ci);
        // 컨트롤 1개 = UTF-16 8 code units → char_count 갱신
        self.document.sections[section_idx].paragraphs[pi].char_count =
            self.document.sections[section_idx].paragraphs[pi]
                .char_count
                .saturating_sub(8);
        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        Ok(format!("{{\"ok\":true}}"))
    }

    /// 문서 전체의 머리말/꼬리말 목록을 반환한다.
    ///
    /// JSON: `{"ok":true,"items":[{"sectionIdx":N,"isHeader":bool,"applyTo":N,"label":"..."},...],
    ///         "currentIndex":N}`
    /// currentSectionIdx/currentIsHeader/currentApplyTo로 현재 편집 중인 항목의 인덱스를 반환.
    pub fn get_header_footer_list_native(
        &self,
        current_section_idx: usize,
        current_is_header: bool,
        current_apply_to: u8,
    ) -> Result<String, HwpError> {
        let mut items = Vec::new();
        let mut current_index: i32 = -1;
        let current_apply = apply_from_u8(current_apply_to);

        for (si, section) in self.document.sections.iter().enumerate() {
            for (pi, para) in section.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    let (is_header, apply) = match ctrl {
                        Control::Header(h) => (true, h.apply_to),
                        Control::Footer(f) => (false, f.apply_to),
                        _ => continue,
                    };
                    let kind = if is_header { "머리말" } else { "꼬리말" };
                    let label = apply_label(apply);
                    let at = apply_to_u8(apply);

                    if si == current_section_idx
                        && is_header == current_is_header
                        && apply == current_apply
                    {
                        current_index = items.len() as i32;
                    }

                    items.push(format!(
                        "{{\"sectionIdx\":{},\"isHeader\":{},\"applyTo\":{},\"label\":\"{}({})\"}}",
                        si, is_header, at, kind, label
                    ));
                }
            }
        }

        Ok(format!(
            "{{\"ok\":true,\"items\":[{}],\"currentIndex\":{}}}",
            items.join(","),
            current_index
        ))
    }

    /// 페이지 단위로 이전/다음 머리말·꼬리말로 이동한다.
    ///
    /// 현재 페이지에서 direction 방향으로 탐색하여 머리말/꼬리말이 있는 다음 페이지를 찾는다.
    /// 홀수/짝수 페이지에 따라 다른 컨트롤(apply_to)을 반환할 수 있다.
    ///
    /// 반환: JSON `{"ok":true,"pageIndex":N,"sectionIdx":N,"isHeader":bool,"applyTo":N}`
    /// 또는 더 이상 이동할 페이지가 없으면 `{"ok":false}`
    pub fn navigate_header_footer_by_page_native(
        &self,
        current_page: u32,
        is_header: bool,
        direction: i32, // -1 또는 +1
    ) -> Result<String, HwpError> {
        let total = self.page_count();
        if total == 0 {
            return Ok("{\"ok\":false}".to_string());
        }

        // 현재 페이지의 머리말/꼬리말 참조 (동일 컨트롤 스킵용)
        let current_ref = if let Ok((pc, _, _)) = self.find_page(current_page) {
            if is_header {
                pc.active_header.clone()
            } else {
                pc.active_footer.clone()
            }
        } else {
            None
        };

        let mut page = current_page as i64 + direction as i64;
        while page >= 0 && page < total as i64 {
            let p = page as u32;
            if let Ok((pc, _, _)) = self.find_page(p) {
                let hf_ref = if is_header {
                    &pc.active_header
                } else {
                    &pc.active_footer
                };
                if let Some(hf) = hf_ref {
                    // 다른 컨트롤이거나, 같은 컨트롤이라도 다른 페이지이면 이동 대상
                    let is_different_control = match &current_ref {
                        Some(cr) => {
                            cr.para_index != hf.para_index
                                || cr.control_index != hf.control_index
                                || cr.source_section_index != hf.source_section_index
                        }
                        None => true,
                    };
                    // 같은 컨트롤이라도 페이지가 달라지면 이동
                    let section_idx = hf.source_section_index;
                    let para_idx = hf.para_index;
                    let ctrl_idx = hf.control_index;

                    // apply_to 추출
                    let apply_to = if let Some(section) = self.document.sections.get(section_idx) {
                        if let Some(para) = section.paragraphs.get(para_idx) {
                            if let Some(ctrl) = para.controls.get(ctrl_idx) {
                                match ctrl {
                                    Control::Header(h) => apply_to_u8(h.apply_to),
                                    Control::Footer(f) => apply_to_u8(f.apply_to),
                                    _ => 0,
                                }
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    } else {
                        0
                    };

                    return Ok(format!(
                        "{{\"ok\":true,\"pageIndex\":{},\"sectionIdx\":{},\"isHeader\":{},\"applyTo\":{}}}",
                        p, section_idx, is_header, apply_to
                    ));
                }
            }
            page += direction as i64;
        }

        Ok("{\"ok\":false}".to_string())
    }

    /// 특정 페이지의 머리말/꼬리말 감추기를 토글한다.
    ///
    /// 반환: JSON `{"ok":true,"hidden":bool}`
    pub fn toggle_hide_header_footer_native(
        &mut self,
        page_num: u32,
        is_header: bool,
    ) -> Result<String, HwpError> {
        let total = self.page_count();
        if page_num >= total {
            return Err(HwpError::RenderError(format!(
                "페이지 인덱스 {} 범위 초과 (총 {}개)",
                page_num, total
            )));
        }
        let key = (page_num, is_header);
        let hidden = if self.hidden_header_footer.contains(&key) {
            self.hidden_header_footer.remove(&key);
            false
        } else {
            self.hidden_header_footer.insert(key);
            true
        };
        // 렌더 트리 캐시 무효화
        let mut cache = self.page_tree_cache.borrow_mut();
        if let Some(slot) = cache.get_mut(page_num as usize) {
            *slot = None;
        }
        Ok(format!("{{\"ok\":true,\"hidden\":{}}}", hidden))
    }

    /// 특정 페이지의 머리말/꼬리말이 감추기 상태인지 확인한다.
    pub fn is_header_footer_hidden(&self, page_num: u32, is_header: bool) -> bool {
        self.hidden_header_footer.contains(&(page_num, is_header))
    }

    /// 머리말/꼬리말 문단 리플로우
    fn reflow_hf_paragraph(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
    ) {
        use crate::renderer::hwpunit_to_px;

        // 머리말/꼬리말 영역 폭 = 페이지 텍스트 영역 폭
        let available_width = {
            let section = &self.document.sections[section_idx];
            let page_def = &section.section_def.page_def;
            let text_width =
                page_def.width as i32 - page_def.margin_left as i32 - page_def.margin_right as i32;
            hwpunit_to_px(text_width, self.dpi)
        };

        // 문단 여백 적용
        let para_shape_id =
            match self.get_hf_paragraph_ref(section_idx, is_header, apply_to, hf_para_idx) {
                Some(p) => p.para_shape_id,
                None => return,
            };
        let para_style = self.styles.para_styles.get(para_shape_id as usize);
        let margin_left = para_style.map(|s| s.margin_left).unwrap_or(0.0);
        let margin_right = para_style.map(|s| s.margin_right).unwrap_or(0.0);
        let final_width = (available_width - margin_left - margin_right).max(0.0);

        // 가변 참조로 리플로우 실행
        let apply = apply_from_u8(apply_to);
        if let Some((pi, ci)) = self.find_header_footer_control(section_idx, is_header, apply) {
            let ctrl = &mut self.document.sections[section_idx].paragraphs[pi].controls[ci];
            let paragraphs = match ctrl {
                Control::Header(h) => &mut h.paragraphs,
                Control::Footer(f) => &mut f.paragraphs,
                _ => return,
            };
            if let Some(para) = paragraphs.get_mut(hf_para_idx) {
                reflow_line_segs(para, final_width, &self.styles, self.dpi);
            }
        }
    }

    /// 머리말/꼬리말 문단의 문단 속성을 조회한다.
    pub fn get_para_properties_in_hf_native(
        &self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
    ) -> Result<String, HwpError> {
        let para = self
            .get_hf_paragraph_ref(section_idx, is_header, apply_to, hf_para_idx)
            .ok_or_else(|| {
                HwpError::RenderError("머리말/꼬리말 문단을 찾을 수 없음".to_string())
            })?;
        Ok(self.build_para_properties_json(para.para_shape_id, section_idx))
    }

    /// 머리말/꼬리말 문단에 문단 서식을 적용한다.
    pub fn apply_para_format_in_hf_native(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
        props_json: &str,
    ) -> Result<String, HwpError> {
        // 현재 para_shape_id 조회
        let base_id = {
            let para = self
                .get_hf_paragraph_ref(section_idx, is_header, apply_to, hf_para_idx)
                .ok_or_else(|| {
                    HwpError::RenderError("머리말/꼬리말 문단을 찾을 수 없음".to_string())
                })?;
            para.para_shape_id
        };

        let mut mods = parse_para_shape_mods(props_json);

        // 탭 설정 변경 처리
        if json_has_tab_keys(props_json) {
            let base_tab_def_id = self
                .document
                .doc_info
                .para_shapes
                .get(base_id as usize)
                .map(|ps| ps.tab_def_id)
                .unwrap_or(0);
            let new_td = build_tab_def_from_json(
                props_json,
                base_tab_def_id,
                &self.document.doc_info.tab_defs,
            );
            let new_tab_id = self.document.find_or_create_tab_def(new_td);
            mods.tab_def_id = Some(new_tab_id);
        }

        // 테두리/배경 변경 처리
        if json_has_border_keys(props_json) {
            let bf_id = self.create_border_fill_from_json(props_json);
            mods.border_fill_id = Some(bf_id);
        }
        if let Some(arr) = parse_json_i16_array(props_json, "borderSpacing", 4) {
            mods.border_spacing = Some([arr[0], arr[1], arr[2], arr[3]]);
        }

        let new_id = self.document.find_or_create_para_shape(base_id, &mods);

        // para_shape_id 갱신
        {
            let para = self.get_hf_paragraph_mut(section_idx, is_header, apply_to, hf_para_idx)?;
            para.para_shape_id = new_id;
        }

        // 줄간격 변경 시 LineSeg 재계산
        if mods.line_spacing.is_some() || mods.line_spacing_type.is_some() {
            self.reflow_hf_paragraph(section_idx, is_header, apply_to, hf_para_idx);
        }

        self.document.sections[section_idx].raw_stream = None;
        self.rebuild_section(section_idx);
        self.event_log.push(DocumentEvent::ParaFormatChanged {
            section: section_idx,
            para: 0,
        });
        Ok("{\"ok\":true}".to_string())
    }

    /// 머리말/꼬리말 문단에 필드 마커를 삽입한다.
    /// field_type: 1=쪽번호(\u{0015}), 2=총쪽수(\u{0016}), 3=파일이름(\u{0017})
    pub fn insert_field_in_hf_native(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        hf_para_idx: usize,
        char_offset: usize,
        field_type: u8,
    ) -> Result<String, HwpError> {
        let marker = match field_type {
            1 => "\u{0015}", // 현재 쪽번호
            2 => "\u{0016}", // 총 쪽수
            3 => "\u{0017}", // 파일 이름
            _ => {
                return Err(HwpError::RenderError(format!(
                    "알 수 없는 필드 타입: {}",
                    field_type
                )))
            }
        };

        let hf_para = self.get_hf_paragraph_mut(section_idx, is_header, apply_to, hf_para_idx)?;
        hf_para.insert_text_at(char_offset, marker);

        self.reflow_hf_paragraph(section_idx, is_header, apply_to, hf_para_idx);

        self.document.sections[section_idx].raw_stream = None;
        self.mark_section_dirty(section_idx);
        self.paginate_if_needed();

        let new_offset = char_offset + 1;
        self.event_log.push(DocumentEvent::TextInserted {
            section: section_idx,
            para: 0,
            offset: char_offset,
            len: 1,
        });
        Ok(super::super::helpers::json_ok_with(&format!(
            "\"charOffset\":{}",
            new_offset
        )))
    }

    /// 머리말/꼬리말 마당(템플릿)을 적용한다.
    ///
    /// template_id:
    /// - 0: 빈 머리말/꼬리말
    /// - 1: 왼쪽 쪽번호 (기본)
    /// - 2: 가운데 쪽번호 (기본)
    /// - 3: 오른쪽 쪽번호 (기본)
    /// - 4: 쪽번호(왼)+파일이름(오) (기본)
    /// - 5: 파일이름(왼)+쪽번호(오) (기본)
    /// - 6~10: 위 1~5와 동일 배치, 볼드+밑줄 스타일
    pub fn apply_hf_template_native(
        &mut self,
        section_idx: usize,
        is_header: bool,
        apply_to: u8,
        template_id: u8,
    ) -> Result<String, HwpError> {
        if section_idx >= self.document.sections.len() {
            return Err(HwpError::RenderError(format!(
                "구역 인덱스 {} 범위 초과",
                section_idx
            )));
        }
        if template_id > 10 {
            return Err(HwpError::RenderError(format!(
                "알 수 없는 템플릿 ID: {}",
                template_id
            )));
        }

        let apply = apply_from_u8(apply_to);

        // 1) 기존 HF가 있으면 삭제
        if self
            .find_header_footer_control(section_idx, is_header, apply)
            .is_some()
        {
            self.delete_header_footer_native(section_idx, is_header, apply_to)?;
        }

        // 2) 새 HF 생성 (빈 문단 1개)
        self.create_header_footer_native(section_idx, is_header, apply_to)?;

        // 빈 템플릿이면 여기서 종료
        if template_id == 0 {
            self.mark_section_dirty(section_idx);
            self.paginate_if_needed();
            return Ok("{\"ok\":true}".to_string());
        }

        // 3) 배치/스타일 결정
        let layout = if template_id <= 5 {
            template_id
        } else {
            template_id - 5
        };
        let styled = template_id > 5; // bold + underline

        // 4) 텍스트 내용 결정
        let text = match layout {
            1 => "\u{0015}".to_string(),           // 왼쪽 쪽번호
            2 => "\u{0015}".to_string(),           // 가운데 쪽번호
            3 => "\u{0015}".to_string(),           // 오른쪽 쪽번호
            4 => "\u{0015}\t\u{0017}".to_string(), // 쪽번호(왼) + 탭 + 파일이름(오)
            5 => "\u{0017}\t\u{0015}".to_string(), // 파일이름(왼) + 탭 + 쪽번호(오)
            _ => String::new(),
        };

        // 5) 정렬 결정
        use crate::model::style::{
            Alignment, CharShapeMods, ParaShapeMods, TabDef, TabItem, UnderlineType,
        };

        let alignment = match layout {
            1 => Alignment::Left,
            2 => Alignment::Center,
            3 => Alignment::Right,
            4 | 5 => Alignment::Left, // 탭으로 오른쪽 배치
            _ => Alignment::Left,
        };

        // 6) 텍스트 삽입
        {
            let hf_para = self.get_hf_paragraph_mut(section_idx, is_header, apply_to, 0)?;
            hf_para.text = text;
            // char_offsets 재계산
            hf_para.char_offsets = hf_para
                .text
                .char_indices()
                .map(|(byte_idx, _)| byte_idx as u32)
                .collect();
        }

        // 7) 문단 정렬 적용
        let base_para_id = {
            let para = self
                .get_hf_paragraph_ref(section_idx, is_header, apply_to, 0)
                .unwrap();
            para.para_shape_id
        };
        let mut para_mods = ParaShapeMods::default();
        para_mods.alignment = Some(alignment);

        // 8) 좌+우 배치 템플릿: 오른쪽 정렬 탭 추가
        if layout == 4 || layout == 5 {
            let section = &self.document.sections[section_idx];
            let page_def = &section.section_def.page_def;
            let text_width =
                page_def.width as i32 - page_def.margin_left as i32 - page_def.margin_right as i32;

            let new_td = TabDef {
                raw_data: None,
                attr: 0,
                tabs: vec![TabItem {
                    position: text_width as u32,
                    tab_type: 1, // 오른쪽 정렬 탭
                    fill_type: 0,
                }],
                auto_tab_left: false,
                auto_tab_right: false,
            };
            let tab_id = self.document.find_or_create_tab_def(new_td);
            para_mods.tab_def_id = Some(tab_id);
        }

        let new_para_id = self
            .document
            .find_or_create_para_shape(base_para_id, &para_mods);
        {
            let hf_para = self.get_hf_paragraph_mut(section_idx, is_header, apply_to, 0)?;
            hf_para.para_shape_id = new_para_id;
        }

        // 9) 볼드+밑줄 스타일 적용
        if styled {
            let base_char_id = {
                let para = self
                    .get_hf_paragraph_ref(section_idx, is_header, apply_to, 0)
                    .unwrap();
                para.char_shapes
                    .first()
                    .map(|cs| cs.char_shape_id)
                    .unwrap_or(0)
            };
            let mut char_mods = CharShapeMods::default();
            char_mods.bold = Some(true);
            char_mods.underline_type = Some(UnderlineType::Bottom);
            let new_char_id = self
                .document
                .find_or_create_char_shape(base_char_id, &char_mods);

            let hf_para = self.get_hf_paragraph_mut(section_idx, is_header, apply_to, 0)?;
            // 전체 텍스트에 새 CharShape 적용
            for cs in &mut hf_para.char_shapes {
                cs.char_shape_id = new_char_id;
            }
        }

        // 10) 리플로우 + 스타일 재해소 + 재페이지네이션
        self.reflow_hf_paragraph(section_idx, is_header, apply_to, 0);
        self.document.sections[section_idx].raw_stream = None;
        self.rebuild_section(section_idx);

        Ok("{\"ok\":true}".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_core() -> DocumentCore {
        use crate::model::document::{Document, Section, SectionDef};
        use crate::model::page::PageDef;
        let mut doc = Document::default();
        let mut section = Section {
            section_def: SectionDef {
                page_def: PageDef {
                    width: 59528,  // A4 폭
                    height: 84188, // A4 높이
                    margin_left: 8504,
                    margin_right: 8504,
                    margin_top: 5668,
                    margin_bottom: 4252,
                    margin_header: 4252,
                    margin_footer: 4252,
                    ..Default::default()
                },
                ..Default::default()
            },
            paragraphs: vec![Paragraph::default()],
            raw_stream: None,
        };
        doc.sections.push(section);
        let mut core = DocumentCore::new_empty();
        core.document = doc;
        core
    }

    #[test]
    fn test_create_and_get_header() {
        let mut core = make_test_core();

        // 머리말이 없는지 확인
        let result = core.get_header_footer_native(0, true, 0).unwrap();
        assert!(result.contains("\"exists\":false"));

        // 머리말 생성
        let result = core.create_header_footer_native(0, true, 0).unwrap();
        assert!(result.contains("\"ok\":true"));
        assert!(result.contains("\"kind\":\"header\""));

        // 머리말이 있는지 확인
        let result = core.get_header_footer_native(0, true, 0).unwrap();
        assert!(result.contains("\"exists\":true"));
        assert!(result.contains("\"paraCount\":1"));
    }

    #[test]
    fn test_create_and_get_footer() {
        let mut core = make_test_core();

        let result = core.create_header_footer_native(0, false, 0).unwrap();
        assert!(result.contains("\"kind\":\"footer\""));

        let result = core.get_header_footer_native(0, false, 0).unwrap();
        assert!(result.contains("\"exists\":true"));
    }

    #[test]
    fn test_duplicate_create_fails() {
        let mut core = make_test_core();
        core.create_header_footer_native(0, true, 0).unwrap();
        let result = core.create_header_footer_native(0, true, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_insert_text_in_header() {
        let mut core = make_test_core();
        core.create_header_footer_native(0, true, 0).unwrap();

        let result = core
            .insert_text_in_header_footer_native(0, true, 0, 0, 0, "Hello")
            .unwrap();
        assert!(result.contains("\"charOffset\":5"));

        let result = core.get_header_footer_native(0, true, 0).unwrap();
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_delete_text_in_header() {
        let mut core = make_test_core();
        core.create_header_footer_native(0, true, 0).unwrap();
        core.insert_text_in_header_footer_native(0, true, 0, 0, 0, "Hello World")
            .unwrap();

        let result = core
            .delete_text_in_header_footer_native(0, true, 0, 0, 5, 6)
            .unwrap();
        assert!(result.contains("\"charOffset\":5"));

        let result = core.get_header_footer_native(0, true, 0).unwrap();
        assert!(result.contains("Hello"));
    }

    #[test]
    fn test_split_merge_paragraph_in_header() {
        let mut core = make_test_core();
        core.create_header_footer_native(0, true, 0).unwrap();
        core.insert_text_in_header_footer_native(0, true, 0, 0, 0, "HelloWorld")
            .unwrap();

        // 문단 분할
        let result = core
            .split_paragraph_in_header_footer_native(0, true, 0, 0, 5)
            .unwrap();
        assert!(result.contains("\"hfParaIndex\":1"));
        assert!(result.contains("\"charOffset\":0"));

        // 문단 수 확인
        let result = core
            .get_header_footer_para_info_native(0, true, 0, 0)
            .unwrap();
        assert!(result.contains("\"paraCount\":2"));

        // 문단 병합
        let result = core
            .merge_paragraph_in_header_footer_native(0, true, 0, 1)
            .unwrap();
        assert!(result.contains("\"hfParaIndex\":0"));
        assert!(result.contains("\"charOffset\":5"));

        // 문단 수 확인
        let result = core
            .get_header_footer_para_info_native(0, true, 0, 0)
            .unwrap();
        assert!(result.contains("\"paraCount\":1"));
    }

    #[test]
    fn test_delete_header_footer() {
        let mut core = make_test_core();
        core.create_header_footer_native(0, true, 0).unwrap();
        core.insert_text_in_header_footer_native(0, true, 0, 0, 0, "머리말 텍스트")
            .unwrap();

        let result = core.delete_header_footer_native(0, true, 0).unwrap();
        assert!(result.contains("\"ok\":true"));

        let result = core.get_header_footer_native(0, true, 0).unwrap();
        assert!(result.contains("\"exists\":false"));
    }

    #[test]
    fn test_delete_nonexistent_fails() {
        let mut core = make_test_core();
        let result = core.delete_header_footer_native(0, true, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_header_footer_list() {
        let mut core = make_test_core();
        core.create_header_footer_native(0, true, 0).unwrap();
        core.create_header_footer_native(0, false, 0).unwrap();

        let result = core.get_header_footer_list_native(0, true, 0).unwrap();
        assert!(result.contains("\"currentIndex\":0"));
        assert!(result.contains("머리말(양 쪽)"));
        assert!(result.contains("꼬리말(양 쪽)"));

        let result = core.get_header_footer_list_native(0, false, 0).unwrap();
        assert!(result.contains("\"currentIndex\":1"));
    }

    #[test]
    fn test_odd_even_header() {
        let mut core = make_test_core();

        // 양쪽 머리말 생성
        core.create_header_footer_native(0, true, 0).unwrap();
        // 홀수 머리말 생성
        core.create_header_footer_native(0, true, 2).unwrap();

        // 양쪽만 조회
        let result = core.get_header_footer_native(0, true, 0).unwrap();
        assert!(result.contains("\"exists\":true"));
        assert!(result.contains("양 쪽"));

        // 홀수만 조회
        let result = core.get_header_footer_native(0, true, 2).unwrap();
        assert!(result.contains("\"exists\":true"));
        assert!(result.contains("홀수 쪽"));

        // 짝수는 없음
        let result = core.get_header_footer_native(0, true, 1).unwrap();
        assert!(result.contains("\"exists\":false"));
    }

    #[test]
    #[ignore] // 진단용 테스트 — 로컬 파일 의존
    fn test_p222_header_structure() {
        let data = std::fs::read("samples/p222.hwp").expect("samples/p222.hwp 파일 읽기 실패");
        let mut core = DocumentCore::from_bytes(&data).expect("문서 로드 실패");

        let num_sections = core.document.sections.len();
        eprintln!("=== p222.hwp 머리말 구조 진단 ===");
        eprintln!("구역 수: {}", num_sections);
        eprintln!("전체 페이지 수: {}", core.page_count());

        for si in 0..num_sections {
            let section = &core.document.sections[si];
            eprintln!(
                "\n--- 구역 {} (문단 {}개) ---",
                si,
                section.paragraphs.len()
            );
            for (pi, para) in section.paragraphs.iter().enumerate() {
                for (ci, ctrl) in para.controls.iter().enumerate() {
                    match ctrl {
                        Control::Header(h) => {
                            eprintln!(
                                "  머리말: para[{}].ctrl[{}] apply_to={:?} 문단수={} 텍스트=[{}]",
                                pi,
                                ci,
                                h.apply_to,
                                h.paragraphs.len(),
                                h.paragraphs
                                    .iter()
                                    .map(|p| p.text.clone())
                                    .collect::<Vec<_>>()
                                    .join("|")
                            );
                        }
                        Control::Footer(f) => {
                            eprintln!(
                                "  꼬리말: para[{}].ctrl[{}] apply_to={:?} 문단수={} 텍스트=[{}]",
                                pi,
                                ci,
                                f.apply_to,
                                f.paragraphs.len(),
                                f.paragraphs
                                    .iter()
                                    .map(|p| p.text.clone())
                                    .collect::<Vec<_>>()
                                    .join("|")
                            );
                        }
                        _ => {}
                    }
                }
            }
        }

        // 각 페이지의 active_header/footer 확인
        eprintln!("\n--- 페이지별 active header/footer ---");
        for page_num in 0..core.page_count() {
            if let Ok((pc, _, _)) = core.find_page(page_num) {
                let hdr = if let Some(ref h) = pc.active_header {
                    format!(
                        "sec={} para={} ctrl={}",
                        h.source_section_index, h.para_index, h.control_index
                    )
                } else {
                    "없음".to_string()
                };
                let ftr = if let Some(ref f) = pc.active_footer {
                    format!(
                        "sec={} para={} ctrl={}",
                        f.source_section_index, f.para_index, f.control_index
                    )
                } else {
                    "없음".to_string()
                };
                eprintln!("  페이지 {}: header=[{}] footer=[{}]", page_num, hdr, ftr);
            }
        }

        // 편집 시나리오: 7페이지(인덱스6), 8페이지(인덱스7) 머리말 구조 확인
        for apply_to_val in [0u8, 1, 2] {
            for sec_idx in 0..num_sections {
                let result = core
                    .get_header_footer_native(sec_idx, true, apply_to_val)
                    .unwrap();
                if result.contains("\"exists\":true") {
                    eprintln!(
                        "  get_header_footer(sec={}, is_header=true, apply_to={}) => {}",
                        sec_idx, apply_to_val, result
                    );
                }
            }
        }

        // 페이지 6(7p), 7(8p)의 hitTestHeaderFooter 시뮬레이션
        eprintln!("\n--- hitTestHeaderFooter + 텍스트 삽입 시뮬레이션 ---");
        for page in [6u32, 7] {
            // hitTestHeaderFooter 호출 (x=100, y=20 — 머리말 영역 가정)
            let hf_hit = core.hit_test_header_footer_native(page, 100.0, 20.0);
            eprintln!(
                "  페이지 {} hitTestHeaderFooter(100,20) => {:?}",
                page, hf_hit
            );

            // 더 넓은 영역으로도 시도 (y=50)
            let hf_hit2 = core.hit_test_header_footer_native(page, 200.0, 50.0);
            eprintln!(
                "  페이지 {} hitTestHeaderFooter(200,50) => {:?}",
                page, hf_hit2
            );
        }

        // 실제 페이지 정보에서 active_header의 apply_to 직접 확인
        eprintln!("\n--- active_header apply_to 직접 확인 ---");
        for page in [6u32, 7] {
            if let Ok((pc, _, _)) = core.find_page(page) {
                if let Some(ref hdr) = pc.active_header {
                    let sec = hdr.source_section_index;
                    let pi = hdr.para_index;
                    let ci = hdr.control_index;
                    if let Some(section) = core.document.sections.get(sec) {
                        if let Some(para) = section.paragraphs.get(pi) {
                            if let Some(ctrl) = para.controls.get(ci) {
                                let apply_to = match ctrl {
                                    Control::Header(h) => apply_to_u8(h.apply_to),
                                    Control::Footer(f) => apply_to_u8(f.apply_to),
                                    _ => 255,
                                };
                                eprintln!(
                                    "  페이지 {} → sec={}, para={}, ctrl={}, apply_to={}",
                                    page, sec, pi, ci, apply_to
                                );
                                // 이 apply_to로 텍스트 삽입 가능한지 확인
                                let result = core.insert_text_in_header_footer_native(
                                    sec, true, apply_to, 0, 0, "T",
                                );
                                eprintln!("    insert_text => {:?}", result);
                            }
                        }
                    }
                } else {
                    eprintln!("  페이지 {} → active_header 없음", page);
                }
            }
        }

        // 텍스트 삽입 테스트
        eprintln!("\n--- 텍스트 삽입 테스트 ---");
        for apply_to_val in [0u8, 1, 2] {
            for sec_idx in 0..num_sections {
                let result = core.insert_text_in_header_footer_native(
                    sec_idx,
                    true,
                    apply_to_val,
                    0,
                    0,
                    "X",
                );
                match result {
                    Ok(r) => eprintln!(
                        "  insert(sec={}, apply_to={}) => OK: {}",
                        sec_idx, apply_to_val, r
                    ),
                    Err(e) => eprintln!(
                        "  insert(sec={}, apply_to={}) => ERR: {}",
                        sec_idx, apply_to_val, e
                    ),
                }
            }
        }
    }

    /// 꼬리말 마당 적용 후 본문 텍스트 삽입이 정상 렌더링되는지 확인 (회귀 테스트)
    ///
    /// 머리말/꼬리말 컨트롤 추가 시 char_count가 갱신되지 않으면
    /// compose_lines에서 텍스트가 UTF-16 범위 밖으로 밀려 렌더링 누락됨.
    #[test]
    fn test_body_text_after_hf_template() {
        let mut core = DocumentCore::new_empty();
        core.create_blank_document_native().unwrap();

        let controls_before = core.document.sections[0].paragraphs[0].controls.len();

        // 꼬리말 마당 적용 (가운데 쪽번호)
        core.apply_hf_template_native(0, false, 0, 2).unwrap();

        // 컨트롤 추가 확인 + char_count 갱신 확인
        let para = &core.document.sections[0].paragraphs[0];
        assert_eq!(para.controls.len(), controls_before + 1);
        // char_count는 컨트롤 1개(8 UTF-16 code units) 만큼 증가해야 함
        assert!(
            para.char_count >= (para.controls.len() as u32) * 8,
            "char_count({})가 컨트롤 UTF-16 크기({})보다 작음",
            para.char_count,
            para.controls.len() * 8
        );

        // 본문 문단 0에 텍스트 삽입
        core.insert_text_native(0, 0, 0, "가나다").unwrap();

        // composed 데이터에 삽입된 텍스트가 포함되어야 함
        let all_text: String = core.composed[0][0]
            .lines
            .iter()
            .flat_map(|l| l.runs.iter().map(|r| r.text.as_str()))
            .collect();
        assert!(
            all_text.contains("가나다"),
            "HF 컨트롤 추가 후 본문 텍스트가 composed에 누락됨: {:?}",
            all_text
        );

        // 렌더 트리에도 포함되어야 함
        let tree = core.build_page_tree(0).unwrap();
        assert!(
            format!("{:?}", tree).contains("가나다"),
            "렌더 트리에 본문 텍스트 없음"
        );
    }
}
