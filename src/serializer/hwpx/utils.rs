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
        let value = filter_xml_1_0_chars(v);
        el.push_attribute((*k, value.as_str()));
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
        let value = filter_xml_1_0_chars(v);
        el.push_attribute((*k, value.as_str()));
    }
    w.write_event(Event::Empty(el))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// 텍스트 노드 (자동 이스케이프)
pub fn text<W: Write>(w: &mut Writer<W>, content: &str) -> Result<(), SerializeError> {
    let content = filter_xml_1_0_chars(content);
    w.write_event(Event::Text(BytesText::new(&content)))
        .map_err(|e| SerializeError::XmlError(e.to_string()))?;
    Ok(())
}

/// XML 1.0이 허용하는 문자만 남긴다.
///
/// `quick-xml`의 event writer는 `&` 등의 마크업 문자는 이스케이프하지만 XML 1.0 문자
/// 범위까지 검증하지 않는다. HWPX의 텍스트와 속성 모두 이 helper를 거쳐야 저장한 패키지가
/// 불법 XML이 되지 않는다 (#3382).
pub fn filter_xml_1_0_chars(s: &str) -> String {
    s.chars()
        .filter(|c| {
            matches!(
                c,
                '\u{09}'
                    | '\u{0A}'
                    | '\u{0D}'
                    | '\u{20}'..='\u{D7FF}'
                    | '\u{E000}'..='\u{FFFD}'
                    | '\u{10000}'..='\u{10FFFF}'
            )
        })
        .collect()
}

/// XML 속성·텍스트 이스케이프 (&, <, >, ", ')
///
/// XML 1.0 이 문서에 담을 수 없는 문자(제어문자 등)는 제거한다 — 남겨 두면 저장된
/// HWPX 안의 XML 이 불법이 되어 한컴·뷰어가 파일 자체를 열지 못한다 (#3382 계열).
pub fn xml_escape(s: &str) -> String {
    let filtered = filter_xml_1_0_chars(s);
    let mut out = String::with_capacity(filtered.len());
    for c in filtered.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
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

    #[test]
    fn event_writers_drop_xml_invalid_chars_from_text_and_attributes() {
        let mut bytes = Vec::new();
        let mut writer = Writer::new(&mut bytes);
        start_tag_attrs(&mut writer, "hp:test", &[("name", "a\u{03}b")]).unwrap();
        text(&mut writer, "x\u{03}y & z").unwrap();
        end_tag(&mut writer, "hp:test").unwrap();
        empty_tag(&mut writer, "hp:empty", &[("value", "c\u{03}d")]).unwrap();

        let xml = String::from_utf8(bytes).unwrap();
        assert_eq!(
            xml,
            r#"<hp:test name="ab">xy &amp; z</hp:test><hp:empty value="cd"/>"#
        );
        assert!(!xml.contains('\u{03}'));
    }

    /// HWPX 속성값의 마크업 문자 이스케이프 계약.
    ///
    /// `start_tag_attrs`/`empty_tag` 는 값에 `filter_xml_1_0_chars` 만 적용하고 나머지는
    /// quick-xml 에 맡긴다. 이스케이프는 `push_attribute` 가 아니라 그 인자인
    /// `(&str, &str)` → `Attribute` 변환에서 일어난다 — quick-xml 이 문서화한 동작:
    /// "Key is stored as-is, but the value will be escaped."
    ///
    /// 이 위임 관계는 눈에 보이지 않아서 양쪽으로 조용히 깨질 수 있다.
    /// (a) 값을 `&[u8]` 로 넘기도록 바꾸면 그 `From` 구현은 이스케이프하지 않으므로
    ///     `&`/`<` 가 그대로 새어나가 저장된 `section*.xml` 이 불법 XML 이 되거나,
    ///     `"` 가 속성값을 조기 종료시켜 뒤 내용이 마크업으로 해석된다.
    /// (b) 반대로 여기서 `xml_escape` 를 한 번 더 적용하면 이중 이스케이프가 되어
    ///     `&` 가 `&amp;amp;` 로 저장되고, 다시 읽으면 `&amp;` 가 되어 값이 손상된다.
    ///     (예: Picture href 의 쿼리스트링 `?a=1&b=2`)
    ///
    /// 그래서 형식만 보지 않고 실제 파서로 왕복시켜 값이 그대로 복원되는지까지 확인한다.
    #[test]
    fn event_writers_escape_markup_chars_in_attribute_values() {
        use quick_xml::{Reader, XmlVersion};

        let mut bytes = Vec::new();
        let mut writer = Writer::new(&mut bytes);
        start_tag_attrs(&mut writer, "hp:start", &[("href", r#"a&b<c>d"e"#)]).unwrap();
        end_tag(&mut writer, "hp:start").unwrap();
        empty_tag(&mut writer, "hp:empty", &[("href", r#"x&y"z<w"#)]).unwrap();

        let xml = String::from_utf8(bytes).unwrap();

        // 원문이 그대로 새어나가면 불법 XML / 속성 인젝션이다.
        assert!(
            !xml.contains(r#"href="a&b<c>d"e""#),
            "속성값이 이스케이프되지 않고 그대로 방출됨: {xml}"
        );
        // 마크업 문자는 엔티티로 나타나야 한다.
        assert!(
            xml.contains("&amp;") && xml.contains("&lt;") && xml.contains("&gt;"),
            "{xml}"
        );
        // 이중 이스케이프 방지 — `&amp;amp;` 가 보이면 값이 손상된 것이다.
        assert!(!xml.contains("&amp;amp;"), "이중 이스케이프됨: {xml}");

        // 형식이 아니라 의미 보존 확인: 실제 파서로 왕복시켜 원래 값이 그대로 나와야 한다.
        let mut reader = Reader::from_str(&xml);
        let mut seen = Vec::new();
        loop {
            match reader.read_event().unwrap() {
                Event::Start(e) | Event::Empty(e) => {
                    for a in e.attributes().flatten() {
                        seen.push(
                            a.normalized_value(XmlVersion::Implicit1_0)
                                .unwrap()
                                .into_owned(),
                        );
                    }
                }
                Event::Eof => break,
                _ => {}
            }
        }
        assert_eq!(seen, vec![r#"a&b<c>d"e"#, r#"x&y"z<w"#]);
    }
}
