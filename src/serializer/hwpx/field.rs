//! 필드 컨트롤 직렬화 — Bookmark, Hyperlink, Field (fieldBegin/End) 뼈대.
//!
//! Stage 5 (#182): 인라인 필드 컨트롤의 `<hp:fieldBegin>` / `<hp:fieldEnd>` 및 `<hp:bookmark>`
//! 의 XML 뼈대를 제공한다. 각주(`<hp:fn>`) / 미주(`<hp:en>`) 는 향후 이슈에서 확장.
//!
//! ## 범위 한정
//!
//! - Stage 5 에서는 **필드 뼈대 출력** 기능만 제공 (section.rs dispatcher 연결은 #186).
//! - 누름틀(ClickHere), 날짜, 메일머지 등 복잡한 필드는 `<hp:fieldBegin type="...">` 의
//!   type 속성만 구분하고 내부 command 직렬화는 #186 에서 확장.

use std::io::Write;

use quick_xml::Writer;

use crate::model::control::{Bookmark, Field, FieldType, Hyperlink};

use super::utils::{empty_tag, end_tag, start_tag};
use super::SerializeError;

// =====================================================================
// <hp:bookmark>
// =====================================================================

pub fn write_bookmark<W: Write>(w: &mut Writer<W>, bm: &Bookmark) -> Result<(), SerializeError> {
    empty_tag(w, "hp:bookmark", &[("name", &bm.name)])
}

// =====================================================================
// <hp:fieldBegin> / <hp:fieldEnd>
// =====================================================================

/// `<hp:fieldBegin>` 여는 태그 + 속성 문자열을 만든다 (자기닫힘 `/>` 없이).
///
/// [#1391] parameters / memo subList 가 있으면 호출부가 자식을 채우고 `</hp:fieldBegin>`
/// 로 닫는다. 없으면 호출부가 `/>` 로 자기닫힘 처리.
pub fn field_begin_open_tag(field: &Field) -> String {
    let id_str = field.field_id.to_string();
    let ft = field_type_str(field.field_type);
    let name = xml_escape_attr(field.ctrl_data_name.as_deref().unwrap_or(""));
    format!(
        r#"<hp:fieldBegin id="{}" type="{}" name="{}" editable="{}""#,
        id_str,
        ft,
        name,
        bool01(field.is_editable_in_form()),
    )
}

fn xml_escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// `<hp:fieldBegin>` — 필드 시작 마커 (자식 없는 경우 자기닫힘).
///
/// HWPX 필드는 텍스트 흐름 안에서 `<hp:fieldBegin>` ~ 텍스트 ~ `<hp:fieldEnd>` 쌍으로 표현된다.
pub fn write_field_begin<W: Write>(w: &mut Writer<W>, field: &Field) -> Result<(), SerializeError> {
    let id_str = field.field_id.to_string();
    let ft = field_type_str(field.field_type);
    empty_tag(
        w,
        "hp:fieldBegin",
        &[
            ("id", &id_str),
            ("type", ft),
            ("name", field.ctrl_data_name.as_deref().unwrap_or("")),
            ("editable", bool01(field.is_editable_in_form())),
        ],
    )
}

/// `<hp:fieldEnd>` — 필드 끝 마커.
pub fn write_field_end<W: Write>(w: &mut Writer<W>, field_id: u32) -> Result<(), SerializeError> {
    let id_str = field_id.to_string();
    empty_tag(w, "hp:fieldEnd", &[("beginIDRef", &id_str)])
}

// =====================================================================
// 하이퍼링크 (필드의 특수형) — <hp:fieldBegin type="HYPERLINK"> 변형
// =====================================================================

#[allow(dead_code)]
pub fn write_hyperlink_begin<W: Write>(
    w: &mut Writer<W>,
    link: &Hyperlink,
    field_id: u32,
) -> Result<(), SerializeError> {
    // command 에 URL 이 들어감. 실제 한컴은 별도 command 파싱 필요.
    let id_str = field_id.to_string();
    let url = &link.url;
    empty_tag(
        w,
        "hp:fieldBegin",
        &[
            ("id", &id_str),
            ("type", "HYPERLINK"),
            ("name", ""),
            ("editable", "0"),
            ("command", url),
        ],
    )
}

// =====================================================================
// 각주 / 미주 뼈대 — <hp:fn> / <hp:en>
// =====================================================================

/// `<hp:fn>` 각주 뼈대 (내부 문단 직렬화는 #186 에서 연결).
#[allow(dead_code)]
pub fn write_footnote_open<W: Write>(w: &mut Writer<W>, number: u16) -> Result<(), SerializeError> {
    let n = number.to_string();
    start_tag(w, "hp:fn")?;
    empty_tag(w, "hp:autoNum", &[("num", &n)])?;
    Ok(())
}

#[allow(dead_code)]
pub fn write_footnote_close<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    end_tag(w, "hp:fn")
}

#[allow(dead_code)]
pub fn write_endnote_open<W: Write>(w: &mut Writer<W>, number: u16) -> Result<(), SerializeError> {
    let n = number.to_string();
    start_tag(w, "hp:en")?;
    empty_tag(w, "hp:autoNum", &[("num", &n)])?;
    Ok(())
}

#[allow(dead_code)]
pub fn write_endnote_close<W: Write>(w: &mut Writer<W>) -> Result<(), SerializeError> {
    end_tag(w, "hp:en")
}

// =====================================================================
// 헬퍼
// =====================================================================

fn bool01(b: bool) -> &'static str {
    if b {
        "1"
    } else {
        "0"
    }
}

fn field_type_str(t: FieldType) -> &'static str {
    use FieldType::*;
    match t {
        Unknown => "UNKNOWN",
        Date => "DATE",
        DocDate => "DOCDATE",
        Path => "PATH",
        Bookmark => "BOOKMARK",
        MailMerge => "MAILMERGE",
        CrossRef => "CROSSREF",
        Formula => "FORMULA",
        ClickHere => "CLICKHERE",
        Summary => "SUMMARY",
        UserInfo => "USERINFO",
        Hyperlink => "HYPERLINK",
        Memo => "MEMO",
        PrivateInfoSecurity => "PRIVATE_INFO",
        TableOfContents => "TOC",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::control::{Bookmark, Field, FieldType, Hyperlink};

    fn to_string<F: FnOnce(&mut Writer<Vec<u8>>) -> Result<(), SerializeError>>(f: F) -> String {
        let mut w: Writer<Vec<u8>> = Writer::new(Vec::new());
        f(&mut w).expect("write");
        String::from_utf8(w.into_inner()).unwrap()
    }

    #[test]
    fn bookmark_emits_name() {
        let bm = Bookmark {
            name: "chapter1".to_string(),
        };
        let xml = to_string(|w| write_bookmark(w, &bm));
        assert!(xml.contains(r#"<hp:bookmark name="chapter1"/>"#), "{}", xml);
    }

    #[test]
    fn field_begin_emits_type_attr() {
        let mut f = Field::default();
        f.field_type = FieldType::ClickHere;
        f.field_id = 42;
        let xml = to_string(|w| write_field_begin(w, &f));
        assert!(xml.contains(r#"id="42""#));
        assert!(xml.contains(r#"type="CLICKHERE""#));
    }

    #[test]
    fn field_end_references_begin_id() {
        let xml = to_string(|w| write_field_end(w, 42));
        assert!(xml.contains(r#"<hp:fieldEnd beginIDRef="42"/>"#));
    }

    #[test]
    fn hyperlink_begin_uses_url_command() {
        let link = Hyperlink {
            url: "https://example.com".to_string(),
            text: "".to_string(),
        };
        let xml = to_string(|w| write_hyperlink_begin(w, &link, 7));
        assert!(xml.contains(r#"type="HYPERLINK""#));
        assert!(xml.contains(r#"command="https://example.com""#));
    }

    #[test]
    fn footnote_emits_auto_num() {
        let xml = to_string(|w| {
            write_footnote_open(w, 3)?;
            write_footnote_close(w)
        });
        assert!(xml.contains("<hp:fn>"));
        assert!(xml.contains(r#"<hp:autoNum num="3"/>"#));
        assert!(xml.contains("</hp:fn>"));
    }

    #[test]
    fn field_type_str_covers_main_variants() {
        assert_eq!(field_type_str(FieldType::Hyperlink), "HYPERLINK");
        assert_eq!(field_type_str(FieldType::Bookmark), "BOOKMARK");
        assert_eq!(field_type_str(FieldType::Date), "DATE");
        assert_eq!(field_type_str(FieldType::TableOfContents), "TOC");
    }
}
