//! HWPX 직렬화 공용 헬퍼 — XML escape / 공통 이벤트 쓰기

use std::io::Write;

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;

use super::SerializeError;

/// `<?xml version="1.0" encoding="UTF-8" standalone="yes"?>` 선언을 쓴다.
pub fn write_xml_decl<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    w.write_event(Event::Decl(BytesDecl::new(
        "1.0",
        Some("UTF-8"),
        Some("yes"),
    )))
    .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// 속성 없는 시작 태그
pub fn start_tag<W: Write>(w: &mut Writer<W>, name: &str) -> Result<(), SerializeError> {
    w.write_event(Event::Start(BytesStart::new(name)))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// 속성 있는 시작 태그
pub fn start_tag_attrs<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    attrs: &[(&str, &str)],
) -> Result<(), SerializeError> {
    let mut el = BytesStart::new(name);
    for (k, v) in attrs {
        el.push_attribute((*k, *v));
    }
    w.write_event(Event::Start(el))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// 종료 태그
pub fn end_tag<W: Write>(w: &mut Writer<W>, name: &str) -> Result<(), SerializeError> {
    w.write_event(Event::End(BytesEnd::new(name)))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// 자기 닫힘 태그 (`<name a="..."/>`)
pub fn empty_tag<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    attrs: &[(&str, &str)],
) -> Result<(), SerializeError> {
    let mut el = BytesStart::new(name);
    for (k, v) in attrs {
        el.push_attribute((*k, *v));
    }
    w.write_event(Event::Empty(el))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// 텍스트 노드 (자동 이스케이프)
pub fn text<W: Write>(w: &mut Writer<W>, content: &str) -> Result<(), SerializeError> {
    w.write_event(Event::Text(BytesText::new(content)))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// XML 속성·텍스트 이스케이프 (&, <, >, ", ')
///
/// XML 1.0 이 문서에 담을 수 없는 문자(제어문자 등)는 제거한다 — 남겨 두면 저장된
/// HWPX 안의 XML 이 불법이 되어 한컴·뷰어가 파일 자체를 열지 못한다 (#3382 계열).
pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            // XML 1.0 허용 문자: #x9 | #xA | #xD | [#x20-#xD7FF] | [#xE000-#xFFFD] | [#x10000-#x10FFFF]
            '\u{09}' | '\u{0A}' | '\u{0D}' => out.push(c),
            '\u{20}'..='\u{D7FF}' | '\u{E000}'..='\u{FFFD}' | '\u{10000}'..='\u{10FFFF}' => {
                out.push(c)
            }
            _ => {} // XML 무효 문자 제거
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xml_escape_drops_xml_invalid_control_chars() {
        // #3382 계열: 저장 경로에서 제어문자를 그대로 흘리면 section0.xml 이 불법 XML 이 되어
        // 한컴·뷰어가 파일 자체를 열지 못한다. (HWPX→HWP5 변환 등으로 IR 에 0x03 이 유입된 실측 사례)
        assert_eq!(xml_escape("a\u{03}b"), "ab");
        for c in ['\u{00}', '\u{08}', '\u{0B}', '\u{0C}', '\u{0E}', '\u{1F}'] {
            assert_eq!(
                xml_escape(&format!("x{c}y")),
                "xy",
                "control {:#04x}",
                c as u32
            );
        }
        assert_eq!(xml_escape("a\u{FFFE}\u{FFFF}b"), "ab");
        // 탭·개행·복귀는 XML 1.0 허용 문자이므로 유지
        assert_eq!(xml_escape("a\tb\nc\rd"), "a\tb\nc\rd");
        // 기존 마크업 이스케이프·한글·non-BMP 는 무회귀
        assert_eq!(xml_escape("<a & b>\"'"), "&lt;a &amp; b&gt;&quot;&apos;");
        assert_eq!(xml_escape("한글 A\u{1F600}"), "한글 A\u{1F600}");
    }
}
