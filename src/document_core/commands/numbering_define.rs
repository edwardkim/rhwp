//! 문단 번호 정의 만들기·제자리 고치기(한/글 '문단 번호/개요 번호 사용자 정의 모양'의 수준별 값).
//!
//! `createNumbering`은 수준별 형식·번호 모양·시작 번호만 받고 문단 머리 정보의 나머지를 기본값으로 둔다.
//! 한/글 사용자 정의 모양은 수준마다 정렬·번호 너비를 실제 문자열 너비에 따름·자동 내어쓰기·너비 조정·
//! 본문과의 간격·번호 글자 모양을 고른다. 한/글 12.30의 개요 번호 모양은 구역이 가리키는 번호 정의를
//! 새로 만들지 않고 그 정의를 고쳐 쓰므로, 같은 번호(ID)를 가리키는 문단 모양·구역 정의는 그대로 둔다.

use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::style::{Numbering, NumberingHead, NUMBERING_LEVELS};
use serde_json::Value;

/// 예전 앱이 남긴 사용자 정의 모양은 7수준만 담는다.
const OLD_LEVELS: usize = 7;
/// 문단 머리 정보 속성(표 42): 0~1 정렬, 2 번호 너비를 실제 문자열 너비에 따름, 3 자동 내어쓰기, 5~8 번호 모양.
const USE_INST_WIDTH: u32 = 1 << 2;
const AUTO_INDENT: u32 = 1 << 3;

fn error(message: impl Into<String>) -> HwpError {
    HwpError::RenderError(message.into())
}

fn number(level: &Value, key: &str, min: i64, max: i64) -> Result<i64, HwpError> {
    level
        .get(key)
        .and_then(Value::as_i64)
        .filter(|n| (min..=max).contains(n))
        .ok_or_else(|| error(format!("{key} 값은 {min}~{max} 사이여야 함")))
}

fn flag(level: &Value, key: &str) -> Result<bool, HwpError> {
    level
        .get(key)
        .and_then(Value::as_bool)
        .ok_or_else(|| error(format!("{key} 값이 없음")))
}

/// 수준 하나: 머리 정보, 번호 서식, 시작 번호, 번호 글자 모양(applyCharFormat 속성, 없으면 본문을 따름).
fn level(value: &Value) -> Result<(NumberingHead, String, u32, Option<String>), HwpError> {
    let format = value
        .get("format")
        .and_then(Value::as_str)
        .filter(|s| {
            s.encode_utf16().count() <= 255
                && !s
                    .chars()
                    .any(|c| c.is_control() || c == '\u{FFFE}' || c == '\u{FFFF}')
        })
        .ok_or_else(|| error("번호 서식은 제어 문자 없이 255자까지 쓸 수 있음"))?
        .to_string();
    let number_format = number(value, "numberFormat", 0, 16)? as u8;
    let mut attr = number(value, "align", 0, 2)? as u32 | (u32::from(number_format) << 5);
    if flag(value, "useInstWidth")? {
        attr |= USE_INST_WIDTH;
    }
    if flag(value, "autoIndent")? {
        attr |= AUTO_INDENT;
    }
    let head = NumberingHead {
        attr,
        width_adjust: number(value, "widthAdjust", i16::MIN.into(), i16::MAX.into())? as i16,
        text_distance: number(value, "textOffset", 0, 100)? as i16,
        char_shape_id: u32::MAX,
        number_format,
    };
    let char_shape = match value.get("charShape") {
        None | Some(Value::Null) => None,
        Some(shape @ Value::Object(_)) => Some(shape.to_string()),
        Some(_) => return Err(error("번호 글자 모양은 객체여야 함")),
    };
    let start = number(value, "start", 1, u32::MAX.into())? as u32;
    Ok((head, format, start, char_shape))
}

impl DocumentCore {
    /// 번호 정의 `numbering_id`(1부터)의 10수준 값을 `json`으로 통째로 바꾼다. 0이면 새로 만든다.
    /// 고친(만든) 정의의 번호를 돌려준다.
    ///
    /// `json`: `{"levels":[{"format":"^1.","numberFormat":0,"start":1,"align":0,"useInstWidth":true,
    /// "autoIndent":true,"widthAdjust":0,"textOffset":50,"charShape":{"fontSize":0,"bold":true}}, …]}`.
    /// 수준은 한/글의 10개(예전 앱이 남긴 모양은 7개)다. 7개면 8~10수준을 한/글의 빈 수준으로 채운다.
    /// 정렬은 0 왼쪽·1 가운데·2 오른쪽, 너비 조정은 HWPUNIT, 본문과의 간격은 글자 크기에 대한 %다.
    /// `charShape`는 applyCharFormat 글자 속성과 바탕 글자 모양 `baseCharShapeId`이고 `fontSize` 0은
    /// '본문 글자 크기 따름'이다. 고친 정의는 HWP 원본 레코드·HWPX 원본 `paraHead` 대신 모델 값으로 저장한다.
    pub fn set_numbering_native(&mut self, numbering_id: u16, json: &str) -> Result<u16, HwpError> {
        let value: Value = serde_json::from_str(json).map_err(|e| error(e.to_string()))?;
        let mut levels = value
            .get("levels")
            .and_then(Value::as_array)
            .filter(|levels| levels.len() == OLD_LEVELS || levels.len() == NUMBERING_LEVELS)
            .ok_or_else(|| error("수준 10개의 번호 모양이 필요함"))?
            .iter()
            .map(level)
            .collect::<Result<Vec<_>, _>>()?;
        // 한/글의 빈 수준: 번호 서식 없음·1,2,3·번호 너비 따름·자동 내어쓰기·본문과의 간격 50%.
        while levels.len() < NUMBERING_LEVELS {
            let head = NumberingHead {
                attr: USE_INST_WIDTH | AUTO_INDENT,
                text_distance: 50,
                char_shape_id: u32::MAX,
                ..Default::default()
            };
            levels.push((head, String::new(), 1, None));
        }
        let count = self.document.doc_info.numberings.len();
        let index = match numbering_id {
            0 if count >= usize::from(u16::MAX) => {
                return Err(error("번호 정의를 더 만들 수 없음"));
            }
            0 => count,
            id if usize::from(id) <= count => usize::from(id) - 1,
            id => return Err(error(format!("번호 정의 {id}가 없음"))),
        };
        let mut numbering = Numbering::default();
        for (i, (head, format, start, char_shape)) in levels.into_iter().enumerate() {
            numbering.heads[i] = head;
            if let Some(shape) = char_shape {
                numbering.heads[i].char_shape_id = self.head_char_shape_id(&shape);
            }
            numbering.level_formats[i] = format;
            numbering.level_start_numbers[i] = start;
        }
        // 한/글은 1수준 시작 번호를 번호 정의의 시작 번호로도 적는다(start 속성).
        numbering.start_number = numbering.level_start_numbers[0].min(u16::MAX.into()) as u16;
        let numberings = &mut self.document.doc_info.numberings;
        if index == count {
            numberings.push(numbering);
        } else {
            numberings[index] = numbering;
        }
        // 원본 DocInfo 스트림을 그대로 쓰지 않도록 표시한다.
        self.document.doc_info.raw_stream_dirty = true;
        let id = index as u16 + 1;
        if self.numbering_in_use(id) {
            // 이 번호를 쓰는 문단은 문서 어디에나 있으므로 스타일부터 쪽 나눔까지 한 번에 다시 만든다.
            self.rebuild_derived_state();
        } else {
            self.rebuild_resolved_styles();
        }
        Ok(id)
    }
}
