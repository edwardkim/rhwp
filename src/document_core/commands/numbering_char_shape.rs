//! 문단 번호 정의의 수준별 번호 글자 모양(한/글 '문단 번호 사용자 정의 모양'의 글자 모양 지정).

use crate::document_core::helpers::{json_i32, parse_char_shape_mods};
use crate::document_core::DocumentCore;
use crate::error::HwpError;

impl DocumentCore {
    /// 번호 정의 `numbering_id`(1부터)의 `level`(0부터) 수준에 번호 글자 모양을 단다.
    ///
    /// `json`은 `applyCharFormat`의 글자 속성에 바탕 글자 모양 `baseCharShapeId`(없으면 0)를 더한 것이다.
    /// 기준 크기 0(`"fontSize":0`)은 한/글 '본문 글자 크기 따름'이라 번호를 문단 끝 글자 크기로 그린다.
    /// `None`이면 글자 모양을 떼어 번호가 본문 글자 모양을 따른다.
    /// 단 글자 모양 ID를, 뗐으면 참조 없음(`u32::MAX`)을 돌려준다.
    pub fn set_numbering_char_shape_native(
        &mut self,
        numbering_id: u16,
        level: usize,
        json: Option<&str>,
    ) -> Result<u32, HwpError> {
        let info = &self.document.doc_info;
        let index = usize::from(numbering_id)
            .checked_sub(1)
            .filter(|index| *index < info.numberings.len())
            .ok_or_else(|| HwpError::RenderError(format!("번호 정의 {numbering_id}가 없음")))?;
        if level >= info.numberings[index].heads.len() {
            return Err(HwpError::RenderError(format!(
                "번호 수준 {level} 범위 초과"
            )));
        }
        let id = match json {
            None => u32::MAX,
            Some(json) => {
                let base = json_i32(json, "baseCharShapeId")
                    .and_then(|id| u32::try_from(id).ok())
                    .filter(|id| (*id as usize) < info.char_shapes.len())
                    .unwrap_or(0);
                let id = self
                    .document
                    .find_or_create_char_shape(base, &parse_char_shape_mods(json));
                if id == 0 {
                    // 이 API의 기존 양수 ID 반환을 보존한다. 문서에서 읽은 0번은
                    // 유효한 모양 참조이며, 참조 없음은 위의 u32::MAX로 구분한다.
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
                } else {
                    id
                }
            }
        };
        let numbering = &mut self.document.doc_info.numberings[index];
        numbering.heads[level].char_shape_id = id;
        // HWPX 원본 수준 구간은 그대로 다시 쓰므로 그 수준의 글자 모양 참조도 고친다.
        if let Some(raw) = &numbering.raw_para_heads {
            numbering.raw_para_heads = set_para_head_char_shape(raw, level + 1, id);
        }
        self.document.doc_info.raw_stream_dirty = true;
        if self.numbering_in_use(numbering_id) {
            // 이 번호를 쓰는 문단은 문서 어디에나 있으므로 스타일부터 쪽 나눔까지 한 번에 다시 만든다.
            self.rebuild_derived_state();
        } else {
            // 아직 아무 문단도 쓰지 않는 번호(막 만든 정의)는 새 글자 모양만 스타일 해소에 넣는다.
            self.rebuild_resolved_styles();
        }
        Ok(id)
    }

    /// 번호 정의를 문단 모양(번호·개요)이나 구역의 개요 번호가 가리키는지.
    fn numbering_in_use(&self, numbering_id: u16) -> bool {
        use crate::model::style::HeadType;
        self.document.doc_info.para_shapes.iter().any(|ps| {
            matches!(ps.head_type, HeadType::Number | HeadType::Outline)
                && ps.numbering_id == numbering_id
        }) || self
            .document
            .sections
            .iter()
            .any(|section| section.section_def.outline_numbering_id == numbering_id)
    }
}

/// HWPX 원본 `paraHead` 구간에서 `level` 수준(1부터)의 `charPrIDRef`를 `id`로 바꾼다.
/// 그 수준이나 속성을 찾지 못하면 `None`이라 저장은 모델의 수준 값으로 다시 쓴다.
fn set_para_head_char_shape(raw: &str, level: usize, id: u32) -> Option<String> {
    let mut found = false;
    let raw = map_para_head_char_refs(raw, |at, old| {
        if at == level {
            found = true;
            Some(id)
        } else {
            Some(old)
        }
    })?;
    found.then_some(raw)
}

/// HWPX 원본 `paraHead` 구간의 `charPrIDRef`를 태그마다 `map(수준, 원래 값)`의 값으로 바꾼다.
/// 수준은 `level` 속성(1부터)이고 없으면 0이다. `map`이 `None`을 돌려주거나 값을 읽지 못하면 `None`이다.
pub(super) fn map_para_head_char_refs(
    raw: &str,
    mut map: impl FnMut(usize, u32) -> Option<u32>,
) -> Option<String> {
    use quick_xml::{events::Event, Reader, XmlVersion};

    let mut reader = Reader::from_str(raw);
    let mut out = String::with_capacity(raw.len());
    let mut copied = 0;
    loop {
        let open = usize::try_from(reader.buffer_position()).ok()?;
        match reader.read_event().ok()? {
            Event::Start(tag) | Event::Empty(tag) if tag.local_name().as_ref() == "paraHead" => {
                let mut level = 0;
                let mut old = None;
                for attribute in tag.attributes() {
                    let attribute = attribute.ok()?;
                    match attribute.key.as_ref() {
                        "level" => {
                            level = attribute
                                .normalized_value(XmlVersion::Implicit1_0)
                                .ok()?
                                .parse()
                                .ok()?;
                        }
                        "charPrIDRef" => {
                            old = Some(
                                attribute
                                    .normalized_value(XmlVersion::Implicit1_0)
                                    .ok()?
                                    .parse()
                                    .ok()?,
                            );
                        }
                        _ => {}
                    }
                }
                let Some(old) = old else {
                    continue;
                };
                let id = map(level, old)?;
                let close = usize::try_from(reader.buffer_position()).ok()?;
                // 태그 경계·속성의 의미는 XML parser로 읽고, 해당 값의 원문 범위만
                // 교체한다. 작은따옴표·공백과 알 수 없는 속성·본문은 그대로 남긴다.
                let value = raw_attribute_value_range(&raw[open..close], "charPrIDRef")?;
                out.push_str(&raw[copied..open + value.start]);
                out.push_str(&id.to_string());
                copied = open + value.end;
            }
            Event::Eof => break,
            _ => {}
        }
    }
    out.push_str(&raw[copied..]);
    Some(out)
}

/// 검증된 시작 태그에서 따옴표 안의 속성 값 범위를 찾는다. 다른 속성 값에 같은
/// 이름이나 `>`가 있어도 그 값 전체를 건너뛰므로 문자열 치환으로 오인하지 않는다.
fn raw_attribute_value_range(tag: &str, wanted: &str) -> Option<std::ops::Range<usize>> {
    let bytes = tag.as_bytes();
    let mut at = tag.find(char::is_whitespace)?;
    loop {
        while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
            at += 1;
        }
        let key_start = at;
        while bytes
            .get(at)
            .is_some_and(|b| !b.is_ascii_whitespace() && *b != b'=')
        {
            at += 1;
        }
        let key = &tag[key_start..at];
        while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
            at += 1;
        }
        if bytes.get(at) != Some(&b'=') {
            return None;
        }
        at += 1;
        while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
            at += 1;
        }
        let quote = *bytes.get(at)?;
        if quote != b'\'' && quote != b'"' {
            return None;
        }
        let value_start = at + 1;
        at = value_start + bytes[value_start..].iter().position(|b| *b == quote)?;
        if key == wanted {
            return Some(value_start..at);
        }
        at += 1;
    }
}
