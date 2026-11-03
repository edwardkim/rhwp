//! 글머리표·그림 글머리표 정의 만들기(한/글 '글머리표/그림 글머리표 사용자 정의 모양').
//!
//! `ensureDefaultBullet`은 문자만 받고 머리 모양을 기본값(왼쪽·자동 내어쓰기·본문과의 간격 50%)으로
//! 둔다. 한/글 사용자 정의 모양은 문자와 함께 글자 모양·너비 조정·정렬·본문과의 간격·자동 내어쓰기를,
//! 그림 글머리표는 그림 파일을 고른다. 여기서는 그 값을 담은 정의를 새로 만든다.

use crate::document_core::helpers::{json_i32, parse_char_shape_mods};
use crate::document_core::DocumentCore;
use crate::error::HwpError;
use crate::model::style::Bullet;
use serde_json::Value;

/// 문단 머리 정보 속성(표 42): 0~1 정렬, 2 너비를 실제 문자열 너비에 따름, 3 자동 내어쓰기.
const USE_INST_WIDTH: u32 = 1 << 2;
const AUTO_INDENT: u32 = 1 << 3;

fn error(message: &str) -> HwpError {
    HwpError::RenderError(message.to_string())
}

fn number(json: &Value, key: &str, default: i64, min: i64, max: i64) -> Result<i64, HwpError> {
    match json.get(key) {
        None => Ok(default),
        Some(value) => value
            .as_i64()
            .filter(|n| (min..=max).contains(n))
            .ok_or_else(|| HwpError::RenderError(format!("{key} 값은 {min}~{max} 사이여야 함"))),
    }
}

fn flag(json: &Value, key: &str, default: bool) -> Result<bool, HwpError> {
    match json.get(key) {
        None => Ok(default),
        Some(value) => value
            .as_bool()
            .ok_or_else(|| HwpError::RenderError(format!("{key} 값은 참·거짓이어야 함"))),
    }
}

impl DocumentCore {
    /// 글머리표 정의를 새로 만들고 그 번호(1부터)를 돌려준다.
    ///
    /// `json`: `{"char":"❏","align":1,"useInstWidth":false,"autoIndent":false,"widthAdjust":1000,
    /// "textOffset":70,"charShape":{"fontSize":0,"bold":true,"baseCharShapeId":0}}`.
    /// 정렬은 0 왼쪽·1 가운데·2 오른쪽, 너비 조정은 HWPUNIT, 본문과의 간격은 글자 크기에 대한 %다.
    /// 빠진 값은 한/글 기본 머리 모양(왼쪽·자동 내어쓰기·간격 50%)을 따른다. `charShape`는
    /// applyCharFormat 글자 속성과 바탕 글자 모양이고 `fontSize` 0은 '본문 글자 크기 따름'이다.
    /// 없거나 null이면 글머리표가 본문 글자 모양을 따른다.
    /// `image`(바이트, 확장자)가 있으면 그 그림을 문서에 넣어 그림 글머리표로 만든다. 문자는 그림을
    /// 그리지 못할 때 쓴다. HWP는 그림 번호를 한 바이트에 담으므로 255번째 그림까지 쓸 수 있다.
    pub fn create_bullet_native(
        &mut self,
        json: &str,
        image: Option<(&[u8], &str)>,
    ) -> Result<u16, HwpError> {
        let value: Value =
            serde_json::from_str(json).map_err(|e| HwpError::RenderError(e.to_string()))?;
        let mut chars = value
            .get("char")
            .and_then(Value::as_str)
            .unwrap_or("")
            .chars();
        let bullet_char = match (chars.next(), chars.next()) {
            (Some(c), None)
                if !c.is_control()
                    && u32::from(c) <= 0xFFFF
                    && c != '\u{FFFE}'
                    && c != '\u{FFFF}' =>
            {
                c
            }
            _ => return Err(error("글머리표 문자는 제어 문자가 아닌 한 글자여야 함")),
        };
        let mut attr = number(&value, "align", 0, 0, 2)? as u32;
        if flag(&value, "useInstWidth", false)? {
            attr |= USE_INST_WIDTH;
        }
        if flag(&value, "autoIndent", true)? {
            attr |= AUTO_INDENT;
        }
        let width_adjust =
            number(&value, "widthAdjust", 0, i16::MIN.into(), i16::MAX.into())? as i16;
        let text_distance = number(&value, "textOffset", 50, 0, 100)? as i16;
        let char_shape = match value.get("charShape") {
            None | Some(Value::Null) => None,
            Some(shape @ Value::Object(_)) => Some(shape.to_string()),
            Some(_) => return Err(error("글머리표 글자 모양은 객체여야 함")),
        };
        if let Some((bytes, extension)) = image {
            if bytes.is_empty()
                || extension.is_empty()
                || !extension.chars().all(|c| c.is_ascii_alphanumeric())
            {
                return Err(error("그림 글머리표의 그림이 비었거나 확장자가 잘못됨"));
            }
            if self.document.bin_data_content.len() >= usize::from(u8::MAX) {
                return Err(error("그림 글머리표는 문서의 255번째 그림까지 쓸 수 있음"));
            }
        }
        if self.document.doc_info.bullets.len() >= usize::from(u16::MAX) {
            return Err(error("글머리표 정의를 더 만들 수 없음"));
        }
        let char_shape_id = char_shape.map_or(u32::MAX, |json| self.head_char_shape_id(&json));
        let image_bullet = image.map_or(0, |(bytes, extension)| {
            self.register_embedded_bin_data(bytes, &extension.to_ascii_lowercase())
        });
        let info = &mut self.document.doc_info;
        info.bullets.push(Bullet {
            attr,
            width_adjust,
            text_distance,
            char_shape_id,
            bullet_char,
            // 한/글 HWP처럼 그림 사용 여부 1과 그림 데이터 넷째 바이트의 그림 번호로 둔다.
            image_bullet: i32::from(image_bullet > 0),
            // 대비·밝기 0, 효과 실제 그림, 그림 번호.
            image_data: [0, 0, 0, image_bullet as u8],
            ..Default::default()
        });
        // 원본 DocInfo 스트림을 그대로 쓰지 않도록 표시한다.
        info.raw_stream_dirty = true;
        let id = info.bullets.len() as u16;
        // 아직 아무 문단도 쓰지 않는 정의라 스타일 해소에만 넣는다.
        self.rebuild_resolved_styles();
        Ok(id)
    }

    /// 문단 머리(번호 수준·글머리표)가 가리킬 글자 모양 ID. `json`은 applyCharFormat의 글자 속성과
    /// 바탕 글자 모양 `baseCharShapeId`(없으면 0)다. 머리 모양의 0은 '참조 없음'으로 읽히므로
    /// 0번과 같은 모양은 0이 아닌 번호로 둔다.
    pub(super) fn head_char_shape_id(&mut self, json: &str) -> u32 {
        let base = json_i32(json, "baseCharShapeId")
            .and_then(|id| u32::try_from(id).ok())
            .filter(|id| (*id as usize) < self.document.doc_info.char_shapes.len())
            .unwrap_or(0);
        let id = self
            .document
            .find_or_create_char_shape(base, &parse_char_shape_mods(json));
        if id != 0 {
            return id;
        }
        let shapes = &mut self.document.doc_info.char_shapes;
        match shapes[1..].iter().position(|shape| *shape == shapes[0]) {
            Some(index) => index as u32 + 1,
            None => {
                let mut shape = shapes[0].clone();
                shape.raw_data = None;
                shapes.push(shape);
                (shapes.len() - 1) as u32
            }
        }
    }
}
