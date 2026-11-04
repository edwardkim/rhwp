//! 덧말·글자 겹침 넣기와 고치기 (한컴 입력 > 덧말 넣기·글자 겹치기).
//!
//! 두 컨트롤은 본문의 확장 제어 문자(0x17) 한 칸을 차지한다. 넣는 자리는 원문 축의
//! 글자 자리이고, 같은 자리의 다른 컨트롤 뒤에 놓는다(각주 넣기와 같다).

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::control::{CharOverlap, Control, Ruby};

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

impl DocumentCore {
    fn compose_paragraph_check(&self, sec: usize, para: usize) -> Result<(), HwpError> {
        let section = self
            .document
            .sections
            .get(sec)
            .ok_or_else(|| invalid("구역 범위 초과"))?;
        section
            .paragraphs
            .get(para)
            .map(|_| ())
            .ok_or_else(|| invalid("문단 범위 초과"))
    }

    fn insert_compose_control(
        &mut self,
        sec: usize,
        para: usize,
        char_offset: usize,
        control: Control,
    ) -> Result<String, HwpError> {
        self.document.sections[sec].raw_stream = None;
        let paragraph = &mut self.document.sections[sec].paragraphs[para];
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
        self.shift_active_field_for_control_insert(sec, para, insert_idx);
        self.after_compose_edit(sec, para);
        Ok(format!("{{\"ok\":true,\"controlIdx\":{insert_idx}}}"))
    }

    fn after_compose_edit(&mut self, sec: usize, para: usize) {
        self.reflow_paragraph(sec, para);
        self.recompose_section(sec);
        self.paginate_if_needed();
        self.invalidate_page_tree_cache();
    }

    /// 본문 문단 글자 자리(원문 축)에 덧말을 넣는다.
    ///
    /// `json`: `{"mainText","rubyText","below":bool,"styleId":number|null}`.
    /// 반환: `{"ok":true,"controlIdx":n}`.
    pub fn insert_ruby_native(
        &mut self,
        sec: usize,
        para: usize,
        char_offset: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        self.compose_paragraph_check(sec, para)?;
        let input: RubyInput =
            serde_json::from_str(json).map_err(|e| invalid(&format!("덧말 값: {e}")))?;
        let ruby = ruby_from(input, None)?;
        self.insert_compose_control(sec, para, char_offset, Control::Ruby(ruby))
    }

    /// 본문 문단 글자 자리(원문 축)에 글자 겹침을 넣는다. 글자는 그 자리의 글자 모양을 따른다.
    ///
    /// `json`: `{"text","shape":0~13,"innerSize":-8~2,"overlap":bool}`.
    /// 반환: `{"ok":true,"controlIdx":n}`.
    pub fn insert_char_overlap_native(
        &mut self,
        sec: usize,
        para: usize,
        char_offset: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        self.compose_paragraph_check(sec, para)?;
        let input: OverlapInput =
            serde_json::from_str(json).map_err(|e| invalid(&format!("글자 겹치기 값: {e}")))?;
        let paragraph = &self.document.sections[sec].paragraphs[para];
        let host = paragraph.char_shape_id_at(char_offset).unwrap_or(0);
        let overlap = overlap_from(input, host)?;
        self.insert_compose_control(sec, para, char_offset, Control::CharOverlap(overlap))
    }

    /// 덧말·글자 겹침 하나의 값. 겹칠 글자는 한컴 창처럼 친 글로 돌려준다.
    ///
    /// 덧말: `{"kind":"ruby","mainText","rubyText","below","styleId"}`,
    /// 겹침: `{"kind":"overlap","text","shape","innerSize","overlap"}`.
    pub fn get_compose_control_native(
        &self,
        sec: usize,
        para: usize,
        ctrl_idx: usize,
    ) -> Result<String, HwpError> {
        self.compose_paragraph_check(sec, para)?;
        let value = match self.document.sections[sec].paragraphs[para]
            .controls
            .get(ctrl_idx)
        {
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
    pub fn set_compose_control_native(
        &mut self,
        sec: usize,
        para: usize,
        ctrl_idx: usize,
        json: &str,
    ) -> Result<String, HwpError> {
        self.compose_paragraph_check(sec, para)?;
        let paragraph = &self.document.sections[sec].paragraphs[para];
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
                let host = overlap.char_shape_ids.first().copied().unwrap_or_else(|| {
                    paragraph
                        .char_shape_id_at(positions.get(ctrl_idx).copied().unwrap_or(0))
                        .unwrap_or(0)
                });
                Control::CharOverlap(overlap_from(input, host)?)
            }
            _ => return Err(invalid("덧말이나 글자 겹치기가 아닙니다.")),
        };
        self.document.sections[sec].raw_stream = None;
        let paragraph = &mut self.document.sections[sec].paragraphs[para];
        paragraph.controls[ctrl_idx] = replacement;
        if let Some(record) = paragraph.ctrl_data_records.get_mut(ctrl_idx) {
            *record = None;
        }
        paragraph.stored_text_partition_dirty = true;
        self.after_compose_edit(sec, para);
        Ok(r#"{"ok":true}"#.to_string())
    }

    /// 덧말·글자 겹침 하나를 지운다. `keep_main_text`면 한컴 덧말 편집의 [덧말 지움]처럼
    /// 덧말만 지우고 본말을 그 자리의 글로 남긴다.
    pub fn delete_compose_control_native(
        &mut self,
        sec: usize,
        para: usize,
        ctrl_idx: usize,
        keep_main_text: bool,
    ) -> Result<String, HwpError> {
        self.compose_paragraph_check(sec, para)?;
        let paragraph = &self.document.sections[sec].paragraphs[para];
        let main_text = match paragraph.controls.get(ctrl_idx) {
            Some(Control::Ruby(ruby)) => keep_main_text.then(|| ruby.main_text.clone()),
            Some(Control::CharOverlap(_)) if !keep_main_text => None,
            _ => return Err(invalid("덧말이나 글자 겹치기가 아닙니다.")),
        };
        let text_offset = crate::document_core::helpers::find_control_text_positions(paragraph)
            .get(ctrl_idx)
            .copied()
            .unwrap_or(0);
        self.document.sections[sec].raw_stream = None;
        let paragraph = &mut self.document.sections[sec].paragraphs[para];
        paragraph.remove_control(ctrl_idx);
        paragraph.stored_text_partition_dirty = true;
        Self::remap_active_field_root_control(
            &mut self.active_field,
            sec,
            para,
            |index| match index.cmp(&ctrl_idx) {
                std::cmp::Ordering::Less => Some(index),
                std::cmp::Ordering::Equal => None,
                std::cmp::Ordering::Greater => Some(index - 1),
            },
        );
        match main_text {
            Some(text) if !text.is_empty() => {
                self.insert_text_native(sec, para, text_offset, &text)?;
            }
            _ => self.after_compose_edit(sec, para),
        }
        Ok(r#"{"ok":true}"#.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(text: &str, shape: u8, inner: i8) -> OverlapInput {
        OverlapInput {
            text: text.into(),
            shape,
            inner_size: inner,
            overlap: false,
        }
    }

    #[test]
    fn overlap_input_drops_spaces_and_keeps_hancom_limits() {
        let overlap = overlap_from(input("1 2", 1, -3), 7).unwrap();
        assert_eq!(overlap.chars, vec!['1', '2']);
        assert_eq!(overlap.char_shape_ids, vec![7, 7, 7]);
        assert!(!overlap.hwp_wire_chars);
        assert!(overlap_from(input("   ", 1, -3), 0).is_err());
        assert!(overlap_from(input("가나다라마바사아자차", 1, -3), 0).is_err());
        assert!(overlap_from(input("가", 14, -3), 0).is_err());
        assert!(overlap_from(input("가", 1, 3), 0).is_err());
    }

    #[test]
    fn ruby_style_sets_option_bit_and_clears_it() {
        let styled = ruby_from(
            RubyInput {
                main_text: "漢字".into(),
                ruby_text: "한자".into(),
                below: true,
                style_id: Some(17),
            },
            None,
        )
        .unwrap();
        assert_eq!(
            (
                styled.pos_type,
                styled.option,
                styled.style_id_ref,
                styled.align
            ),
            (1, 4, 17, 3)
        );
        let plain = ruby_from(
            RubyInput {
                main_text: "漢字".into(),
                ruby_text: "한자".into(),
                below: false,
                style_id: None,
            },
            Some(&styled),
        )
        .unwrap();
        assert_eq!(
            (plain.pos_type, plain.option, plain.style_id_ref),
            (0, 0, 0)
        );
        assert!(ruby_from(
            RubyInput {
                main_text: " ".into(),
                ruby_text: "x".into(),
                below: false,
                style_id: None,
            },
            None
        )
        .is_err());
    }
}
