//! HWPX shared XML events. Counting and output use the same borrowed encoder.
use super::SerializeError;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Writer;
use std::io::Write;

pub(super) fn xml_error(error: impl std::fmt::Display) -> SerializeError {
    SerializeError::XmlError(format!("HWPX XML: {error}"))
}
pub(super) fn xml_value<S: crate::storage::Storage>(
    storage: &S,
    value: impl std::fmt::Display,
) -> Result<String, SerializeError>
where
    S::Error: std::fmt::Display,
{
    storage.display(&value).map_err(xml_error)
}

pub fn write_xml_decl<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    write_xml_decl_with_storage(w, &crate::storage::Unbounded)
}
pub(super) fn write_xml_decl_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    storage.bytes(0).map_err(xml_error)?;
    let declaration = BytesDecl::from_start(BytesStart::from_content(
        r#"xml version="1.0" encoding="UTF-8" standalone="yes""#,
        3,
    ));
    w.write_event(Event::Decl(declaration)).map_err(xml_error)
}
pub fn start_tag<W: Write>(w: &mut Writer<W>, name: &str) -> Result<(), SerializeError> {
    w.write_event(Event::Start(BytesStart::new(name)))
        .map_err(xml_error)
}
pub(super) fn start_tag_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    name: &str,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    storage.bytes(0).map_err(xml_error)?;
    start_tag(w, name)
}
pub fn end_tag<W: Write>(w: &mut Writer<W>, name: &str) -> Result<(), SerializeError> {
    w.write_event(Event::End(BytesEnd::new(name)))
        .map_err(xml_error)
}
pub(super) fn end_tag_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    name: &str,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    storage.bytes(0).map_err(xml_error)?;
    end_tag(w, name)
}
pub fn start_tag_attrs<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    attrs: &[(&str, &str)],
) -> Result<(), SerializeError> {
    start_tag_attrs_with_storage(w, name, attrs, &crate::storage::Unbounded)
}
pub(super) fn start_tag_attrs_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    name: &str,
    attrs: &[(&str, &str)],
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let bytes = storage
        .binary(&XmlPayload::Tag(name, attrs))
        .map_err(xml_error)?;
    let content = std::str::from_utf8(&bytes).expect("XML encoder preserves UTF-8");
    w.write_event(Event::Start(BytesStart::from_content(content, name.len())))
        .map_err(xml_error)
}
pub fn empty_tag<W: Write>(
    w: &mut Writer<W>,
    name: &str,
    attrs: &[(&str, &str)],
) -> Result<(), SerializeError> {
    empty_tag_with_storage(w, name, attrs, &crate::storage::Unbounded)
}
pub(super) fn empty_tag_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    name: &str,
    attrs: &[(&str, &str)],
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let bytes = storage
        .binary(&XmlPayload::Tag(name, attrs))
        .map_err(xml_error)?;
    let content = std::str::from_utf8(&bytes).expect("XML encoder preserves UTF-8");
    w.write_event(Event::Empty(BytesStart::from_content(content, name.len())))
        .map_err(xml_error)
}
pub fn text<W: Write>(w: &mut Writer<W>, content: &str) -> Result<(), SerializeError> {
    text_with_storage(w, content, &crate::storage::Unbounded)
}
pub(super) fn text_with_storage<W: Write, S: crate::storage::Storage>(
    w: &mut Writer<W>,
    content: &str,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    let bytes = storage
        .binary(&XmlPayload::Text(content, Escape::Text))
        .map_err(xml_error)?;
    let content = std::str::from_utf8(&bytes).expect("XML encoder preserves UTF-8");
    w.write_event(Event::Text(BytesText::from_escaped(content)))
        .map_err(xml_error)
}
fn valid_xml_char(c: char) -> bool {
    matches!(c,'\u{09}'|'\u{0A}'|'\u{0D}'|'\u{20}'..='\u{D7FF}'|'\u{E000}'..='\u{FFFD}'|'\u{10000}'..='\u{10FFFF}')
}
pub fn filter_xml_1_0_chars(s: &str) -> String {
    s.chars().filter(|c| valid_xml_char(*c)).collect()
}
pub fn xml_escape(s: &str) -> String {
    xml_escape_with_storage(s, &crate::storage::Unbounded).expect("unbounded XML escape")
}
pub(super) fn xml_escape_with_storage<S: crate::storage::Storage>(
    s: &str,
    storage: &S,
) -> Result<String, SerializeError>
where
    S::Error: std::fmt::Display,
{
    let bytes = storage
        .binary(&XmlPayload::Text(s, Escape::Literal))
        .map_err(xml_error)?;
    Ok(String::from_utf8(bytes).expect("XML encoder preserves UTF-8"))
}
#[derive(Clone, Copy, PartialEq)]
enum Escape {
    Literal,
    Text,
    Attribute,
}
enum XmlPayload<'a> {
    Tag(&'a str, &'a [(&'a str, &'a str)]),
    Text(&'a str, Escape),
}
impl crate::serializer::byte_writer::BinaryProducer for XmlPayload<'_> {
    fn write<W: Write>(
        &self,
        w: &mut crate::serializer::byte_writer::ByteWriter<W>,
    ) -> std::io::Result<()> {
        match self {
            Self::Tag(name, attrs) => {
                w.write_bytes(name.as_bytes())?;
                for (key, value) in *attrs {
                    w.write_bytes(b" ")?;
                    w.write_bytes(key.as_bytes())?;
                    w.write_bytes(b"=\"")?;
                    write_escaped(w, value, Escape::Attribute)?;
                    w.write_bytes(b"\"")?;
                }
            }
            Self::Text(value, mode) => write_escaped(w, value, *mode)?,
        }
        Ok(())
    }
}
fn write_escaped<W: Write>(
    w: &mut crate::serializer::byte_writer::ByteWriter<W>,
    s: &str,
    mode: Escape,
) -> std::io::Result<()> {
    let mut start = 0;
    for (at, c) in s.char_indices() {
        let replacement = match c {
            '\t' if mode == Escape::Attribute => Some("&#9;"),
            '\n' if mode == Escape::Attribute => Some("&#10;"),
            '\r' if mode != Escape::Literal => Some("&#13;"),
            '&' => Some("&amp;"),
            '<' => Some("&lt;"),
            '>' => Some("&gt;"),
            '"' => Some("&quot;"),
            '\'' => Some("&apos;"),
            _ if !valid_xml_char(c) => Some(""),
            _ => None,
        };
        if let Some(value) = replacement {
            w.write_bytes(&s.as_bytes()[start..at])?;
            w.write_bytes(value.as_bytes())?;
            start = at + c.len_utf8();
        }
    }
    w.write_bytes(&s.as_bytes()[start..])
}

/// Literal replacement shares the same borrowed spans with the fallible display sink.
/// `match_indices` also preserves the standard empty-pattern UTF-8 boundary behavior.
pub(super) fn xml_replacen<S: crate::storage::Storage>(
    input: impl AsRef<str>,
    needle: &str,
    replacement: &str,
    limit: usize,
    storage: &S,
) -> Result<String, SerializeError>
where
    S::Error: std::fmt::Display,
{
    struct Replacement<'a> {
        input: &'a str,
        needle: &'a str,
        replacement: &'a str,
        limit: usize,
    }
    impl std::fmt::Display for Replacement<'_> {
        fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            let mut start = 0;
            for (at, matched) in self.input.match_indices(self.needle).take(self.limit) {
                out.write_str(&self.input[start..at])?;
                out.write_str(self.replacement)?;
                start = at + matched.len();
            }
            out.write_str(&self.input[start..])
        }
    }
    xml_value(
        storage,
        Replacement {
            input: input.as_ref(),
            needle,
            replacement,
            limit,
        },
    )
}
pub(super) fn xml_insert<S: crate::storage::Storage>(
    input: &mut String,
    at: usize,
    value: &str,
    storage: &S,
) -> Result<(), SerializeError>
where
    S::Error: std::fmt::Display,
{
    storage
        .reserve_string(input, value.len())
        .map_err(xml_error)?;
    input.insert_str(at, value);
    Ok(())
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
