//! 덧말·글자 겹침 넣기와 고치기 (한컴 입력 > 덧말 넣기·글자 겹치기).
//!
//! 두 컨트롤은 문단의 확장 제어 문자(0x17) 한 칸을 차지한다. 넣는 자리는 원문 축의
//! 글자 자리이고, 같은 자리의 다른 컨트롤 뒤에 놓는다(각주 넣기와 같다). 본문 문단과 함께
//! 표 셀·글상자·머리말/꼬리말(셀 경로)과 각주·미주 문단에서도 넣고 고친다.

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::{CharOverlap, Control, Ruby};
use crate::model::paragraph::Paragraph;

/// 한컴 12.30 글자 겹치기 창이 받는 글자 수.
const MAX_OVERLAP_CHARS: usize = 9;

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct RubyInput {
    main_text: String,
    ruby_text: String,
    #[serde(default)]
    below: bool,
    #[serde(default)]
    style_id: Option<u16>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OverlapInput {
    text: String,
    shape: u8,
    /// HWP 내부 글자 크기(charSz). 한컴 창의 100%가 -3이고 10%마다 1씩 바뀐다.
    #[serde(default = "default_inner_size")]
    inner_size: i8,
    /// 모양 안에 글자 겹치기(포개기). 끄면 펼친다.
    #[serde(default)]
    overlap: bool,
}

fn default_inner_size() -> i8 {
    -3
}

fn invalid(message: &str) -> HwpError {
    HwpError::RenderError(message.to_string())
}

fn ruby_from(input: RubyInput, base: Option<&Ruby>) -> Result<Ruby, HwpError> {
    if input.main_text.trim().is_empty() || input.ruby_text.trim().is_empty() {
        return Err(invalid("본말과 덧말을 입력하세요."));
    }
    let mut ruby = base.cloned().unwrap_or(Ruby {
        // 한컴 12.30 창이 넣는 값: 가운데 정렬, 크기 비율 0(본말의 절반).
        align: 3,
        ..Default::default()
    });
    ruby.main_text = input.main_text;
    ruby.ruby_text = input.ruby_text;
    ruby.pos_type = u8::from(input.below);
    match input.style_id {
        Some(style) => {
            ruby.option |= 4;
            ruby.style_id_ref = style;
        }
        None => {
            ruby.option &= !4;
            ruby.style_id_ref = 0;
        }
    }
    Ok(ruby)
}

fn overlap_from(input: OverlapInput, host: u32) -> Result<CharOverlap, HwpError> {
    // 한컴 도움말: 공백은 겹쳐 쓸 수 없고, 아홉 글자까지 쓴다.
    let chars: Vec<char> = input.text.chars().filter(|c| !c.is_whitespace()).collect();
    if chars.is_empty() || chars.len() > MAX_OVERLAP_CHARS {
        return Err(invalid("겹쳐 쓸 글자는 한 글자에서 아홉 글자까지입니다."));
    }
    if input.shape > 13 || !(-8..=2).contains(&input.inner_size) {
        return Err(invalid("글자 겹치기 모양이나 크기가 범위를 벗어났습니다."));
    }
    Ok(CharOverlap {
        // 글자모양 배열의 첫 칸은 테두리, 나머지는 글자마다 놓인 자리의 모양이다.
        char_shape_ids: vec![host; chars.len() + 1],
        chars,
        border_type: input.shape,
        inner_char_size: input.inner_size,
        expansion: u8::from(input.overlap),
        hwp_wire_chars: false,
    })
}

/// 덧말·글자 겹침이 든 문단. 셀 경로는 표 셀(중첩 포함)·글상자·그림 캡션과 머리말·꼬리말 뿌리를
/// 지나는 `...ByPath` API의 경로이고, 주석은 각주·미주(`...InFootnote` API의 주소)다.
#[derive(Clone, Copy, Debug)]
pub enum ComposeHost<'a> {
    Body {
        sec: usize,
        para: usize,
    },
    Cell {
        sec: usize,
        parent_para: usize,
        path: &'a [(usize, usize, usize)],
    },
    Note {
        sec: usize,
        para: usize,
        control: usize,
        note_para: usize,
    },
}

impl ComposeHost<'_> {
    fn section(&self) -> usize {
        match *self {
            Self::Body { sec, .. } | Self::Cell { sec, .. } | Self::Note { sec, .. } => sec,
        }
    }
}

impl DocumentCore {
    fn compose_paragraph(&self, host: ComposeHost) -> Result<&Paragraph, HwpError> {
        match host {
            ComposeHost::Body { sec, para } => self
                .document
                .sections
                .get(sec)
                .ok_or_else(|| invalid("구역 범위 초과"))?
                .paragraphs
                .get(para)
                .ok_or_else(|| invalid("문단 범위 초과")),
            ComposeHost::Cell {
                sec,
                parent_para,
                path,
            } => self.resolve_paragraph_by_path(sec, parent_para, path),
            ComposeHost::Note {
                sec,
                para,
                control,
                note_para,
            } => self
                .get_footnote_paragraph_ref(sec, para, control, note_para)
                .ok_or_else(|| invalid("각주·미주 문단을 찾을 수 없습니다.")),
        }
    }

    fn compose_paragraph_mut(&mut self, host: ComposeHost) -> Result<&mut Paragraph, HwpError> {
        match host {
            ComposeHost::Body { sec, para } => self
                .document
                .sections
                .get_mut(sec)
                .ok_or_else(|| invalid("구역 범위 초과"))?
                .paragraphs
                .get_mut(para)
                .ok_or_else(|| invalid("문단 범위 초과")),
            ComposeHost::Cell {
                sec,
                parent_para,
                path,
            } => self.get_cell_paragraph_mut_by_path(sec, parent_para, path),
            ComposeHost::Note {
                sec,
                para,
                control,
                note_para,
            } => self.get_footnote_paragraph_mut(sec, para, control, note_para),
        }
    }

    fn insert_compose_control(
        &mut self,
        host: ComposeHost,
        char_offset: usize,
        control: Control,
    ) -> Result<String, HwpError> {
        let paragraph = self.compose_paragraph_mut(host)?;
        let positions = crate::document_core::helpers::find_control_text_positions(paragraph);
        let insert_idx = positions
            .iter()
            .position(|&pos| pos > char_offset)
            .unwrap_or(paragraph.controls.len());
        paragraph.align_ctrl_data_records();
        paragraph.controls.insert(insert_idx, control);
        paragraph.ctrl_data_records.insert(insert_idx, None);
        paragraph.shift_for_inline_control_insert(insert_idx, char_offset);
        paragraph.char_count += 8;
        paragraph.control_mask |= 1u32 << 0x0017;
        paragraph.has_para_text = true;
        paragraph.stored_text_partition_dirty = true;
        if let ComposeHost::Body { sec, para } = host {
            self.shift_active_field_for_control_insert(sec, para, insert_idx);
        }
        self.after_compose_edit(host);
        Ok(format!("{{\"ok\":true,\"controlIdx\":{insert_idx}}}"))
    }

    /// 그 문단을 다시 짜고 쪽을 다시 나눈다. 셀은 경로의 가장 안쪽 셀 폭, 주석은 주석 영역 폭으로 짠다.
    fn after_compose_edit(&mut self, host: ComposeHost) {
        let sec = host.section();
        self.document.sections[sec].raw_stream = None;
        match host {
            ComposeHost::Body { para, .. } => {
                self.reflow_paragraph(sec, para);
                self.recompose_section(sec);
            }
            ComposeHost::Cell {
                parent_para, path, ..
            } => {
                let inner_para = path.last().map_or(0, |entry| entry.2);
                self.reflow_cell_paragraph_by_path(sec, parent_para, path, inner_para);
                self.recalculate_cell_paragraph_vpos_by_path(
                    sec,
                    parent_para,
                    path,
                    inner_para,
                    None,
                );
                self.mark_cell_control_dirty(sec, parent_para, path[0].0);
                self.mark_section_dirty(sec);
            }
            ComposeHost::Note {
                para,
                control,
                note_para,
                ..
            } => {
                self.reflow_footnote_paragraph(sec, para, control, note_para);
                self.mark_section_dirty(sec);
            }
        }
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
    }

    /// 그 문단 글자 자리(원문 축)에 덧말을 넣는다.
    ///
    /// `json`: `{"mainText","rubyText","below":bool,"styleId":number|null}`.
    /// 반환: `{"ok":true,"controlIdx":n}`.
    pub fn insert_ruby_at(
        &mut self,
        host: ComposeHost,
        char_offset: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        self.compose_paragraph(host)?;
        let input: RubyInput =
            serde_json::from_str(json).map_err(|e| invalid(&format!("덧말 값: {e}")))?;
        let ruby = ruby_from(input, None)?;
        self.insert_compose_control(host, char_offset, Control::Ruby(ruby))
    }

    /// 그 문단 글자 자리(원문 축)에 글자 겹침을 넣는다. 글자는 그 자리의 글자 모양을 따른다.
    ///
    /// `json`: `{"text","shape":0~13,"innerSize":-8~2,"overlap":bool}`.
    /// 반환: `{"ok":true,"controlIdx":n}`.
    pub fn insert_char_overlap_at(
        &mut self,
        host: ComposeHost,
        char_offset: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        let host_shape = self
            .compose_paragraph(host)?
            .char_shape_id_at(char_offset)
            .unwrap_or(0);
        let input: OverlapInput =
            serde_json::from_str(json).map_err(|e| invalid(&format!("글자 겹치기 값: {e}")))?;
        let overlap = overlap_from(input, host_shape)?;
        self.insert_compose_control(host, char_offset, Control::CharOverlap(overlap))
    }

    /// 덧말·글자 겹침 하나의 값. 겹칠 글자는 한컴 창처럼 친 글로 돌려준다.
    ///
    /// 덧말: `{"kind":"ruby","mainText","rubyText","below","styleId"}`,
    /// 겹침: `{"kind":"overlap","text","shape","innerSize","overlap"}`.
    pub fn get_compose_control_at(
        &self,
        host: ComposeHost,
        ctrl_idx: usize,
    ) -> Result<String, HwpError> {
        let value = match self.compose_paragraph(host)?.controls.get(ctrl_idx) {
            Some(Control::Ruby(ruby)) => serde_json::json!({
                "kind": "ruby",
                "mainText": ruby.main_text,
                "rubyText": ruby.ruby_text,
                "below": ruby.pos_type == 1,
                "styleId": (ruby.option & 4 != 0).then_some(ruby.style_id_ref),
            }),
            Some(Control::CharOverlap(overlap)) => serde_json::json!({
                "kind": "overlap",
                "text": crate::renderer::composer::char_overlap_plain_text(overlap),
                "shape": overlap.border_type,
                "innerSize": overlap.inner_char_size,
                "overlap": overlap.expansion == 1,
            }),
            _ => return Err(invalid("덧말이나 글자 겹치기가 아닙니다.")),
        };
        Ok(value.to_string())
    }

    /// 덧말·글자 겹침 하나를 고친다. 입력은 넣을 때와 같다. 자리와 제어 순서는 그대로다.
    pub fn set_compose_control_at(
        &mut self,
        host: ComposeHost,
        ctrl_idx: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        let paragraph = self.compose_paragraph(host)?;
        let replacement = match paragraph.controls.get(ctrl_idx) {
            Some(Control::Ruby(ruby)) => {
                let input: RubyInput =
                    serde_json::from_str(json).map_err(|e| invalid(&format!("덧말 값: {e}")))?;
                Control::Ruby(ruby_from(input, Some(ruby))?)
            }
            Some(Control::CharOverlap(overlap)) => {
                let input: OverlapInput = serde_json::from_str(json)
                    .map_err(|e| invalid(&format!("글자 겹치기 값: {e}")))?;
                // 고쳐도 겹침이 놓인 자리의 글자 모양(첫 칸)은 그대로 둔다.
                let positions =
                    crate::document_core::helpers::find_control_text_positions(paragraph);
                let host_shape = overlap.char_shape_ids.first().copied().unwrap_or_else(|| {
                    paragraph
                        .char_shape_id_at(positions.get(ctrl_idx).copied().unwrap_or(0))
                        .unwrap_or(0)
                });
                Control::CharOverlap(overlap_from(input, host_shape)?)
            }
            _ => return Err(invalid("덧말이나 글자 겹치기가 아닙니다.")),
        };
        let paragraph = self.compose_paragraph_mut(host)?;
        paragraph.controls[ctrl_idx] = replacement;
        if let Some(record) = paragraph.ctrl_data_records.get_mut(ctrl_idx) {
            *record = None;
        }
        paragraph.stored_text_partition_dirty = true;
        self.after_compose_edit(host);
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 덧말·글자 겹침 하나를 지운다. `keep_main_text`면 한컴 덧말 편집의 [덧말 지움]처럼
    /// 덧말만 지우고 본말을 그 자리의 글로 남긴다.
    pub fn delete_compose_control_at(
        &mut self,
        host: ComposeHost,
        ctrl_idx: usize,
        keep_main_text: bool,
    ) -> Result<String, HwpError> {
        let paragraph = self.compose_paragraph(host)?;
        let main_text = match paragraph.controls.get(ctrl_idx) {
            Some(Control::Ruby(ruby)) => keep_main_text.then(|| ruby.main_text.clone()),
            Some(Control::CharOverlap(_)) if !keep_main_text => None,
            _ => return Err(invalid("덧말이나 글자 겹치기가 아닙니다.")),
        };
        let text_offset = crate::document_core::helpers::find_control_text_positions(paragraph)
            .get(ctrl_idx)
            .copied()
            .unwrap_or(0);
        let paragraph = self.compose_paragraph_mut(host)?;
        paragraph.remove_control(ctrl_idx);
        paragraph.stored_text_partition_dirty = true;
        if let ComposeHost::Body { sec, para } = host {
            Self::remap_active_field_root_control(&mut self.active_field, sec, para, |index| {
                match index.cmp(&ctrl_idx) {
                    std::cmp::Ordering::Less => Some(index),
                    std::cmp::Ordering::Equal => None,
                    std::cmp::Ordering::Greater => Some(index - 1),
                }
            });
        }
        match (host, main_text.filter(|text| !text.is_empty())) {
            // 본문은 본문 글 넣기 경로(필드 소속·줄 다시 짜기)로 본말을 남긴다.
            (ComposeHost::Body { sec, para }, Some(text)) => {
                self.document.sections[sec].raw_stream = None;
                self.insert_text_native(sec, para, text_offset, &text)?;
            }
            // 셀·주석은 머리말·꼬리말·각주 글 넣기처럼 문단에 넣고 그 영역 폭으로 다시 짠다.
            (_, Some(text)) => {
                self.compose_paragraph_mut(host)?
                    .insert_text_at(text_offset, &text);
                self.after_compose_edit(host);
            }
            (_, None) => self.after_compose_edit(host),
        }
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 본문 문단 글자 자리(원문 축)에 덧말을 넣는다. [`Self::insert_ruby_at`]의 본문 주소다.
    pub fn insert_ruby_native(
        &mut self,
        sec: usize,
        para: usize,
        char_offset: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        self.insert_ruby_at(ComposeHost::Body { sec, para }, char_offset, json)
    }

    /// 본문 문단 글자 자리(원문 축)에 글자 겹침을 넣는다.
    pub fn insert_char_overlap_native(
        &mut self,
        sec: usize,
        para: usize,
        char_offset: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        self.insert_char_overlap_at(ComposeHost::Body { sec, para }, char_offset, json)
    }

    /// 본문 문단의 덧말·글자 겹침 하나의 값.
    pub fn get_compose_control_native(
        &self,
        sec: usize,
        para: usize,
        ctrl_idx: usize,
    ) -> Result<String, HwpError> {
        self.get_compose_control_at(ComposeHost::Body { sec, para }, ctrl_idx)
    }

    /// 본문 문단의 덧말·글자 겹침 하나를 고친다.
    pub fn set_compose_control_native(
        &mut self,
        sec: usize,
        para: usize,
        ctrl_idx: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        self.set_compose_control_at(ComposeHost::Body { sec, para }, ctrl_idx, json)
    }

    /// 본문 문단의 덧말·글자 겹침 하나를 지운다.
    pub fn delete_compose_control_native(
        &mut self,
        sec: usize,
        para: usize,
        ctrl_idx: usize,
        keep_main_text: bool,
    ) -> Result<String, HwpError> {
        self.delete_compose_control_at(ComposeHost::Body { sec, para }, ctrl_idx, keep_main_text)
    }
}
