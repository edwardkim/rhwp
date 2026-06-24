//! Contents/content.hpf — OPF 패키지 매니페스트
//!
//! `parser::hwpx::content`의 역방향. 한컴 호환을 위해 14개 네임스페이스와
//! 기본 metadata를 선언한다.

use std::io::{Cursor, Write};

use quick_xml::Writer;

use super::utils::{empty_tag, end_tag, start_tag_attrs, text, write_xml_decl};
use super::SerializeError;

/// BinData 엔트리 (manifest 등록용)
#[derive(Debug, Clone)]
pub struct BinDataEntry {
    pub id: String,
    pub href: String,
    pub media_type: String,
}

/// 원본 content.hpf 에서 `<opf:metadata> … </opf:metadata>` 블록(태그 포함)을
/// 그대로 추출한다.
///
/// metadata 는 본문(섹션/BinData)과 무관한 저작자·일자·주제 정보라, manifest/spine 을
/// IR 로 재생성하더라도 이 블록만은 원본을 보존해야 손실이 없다. self-closing
/// (`<opf:metadata/>`) 형태도 처리한다. 형태를 인식하지 못하면 `None` 을 돌려
/// 호출자가 하드코딩 기본값으로 폴백하도록 한다.
fn extract_metadata_block(original: &str) -> Option<&str> {
    let open = original.find("<opf:metadata>")?;
    let close = original[open..].find("</opf:metadata>")? + open + "</opf:metadata>".len();
    Some(&original[open..close])
}

/// content.hpf XML 생성
pub fn write_content_hpf(
    section_hrefs: &[String],
    bin_data: &[BinDataEntry],
    master_items: &[(String, String)],
    original_content_hpf: Option<&[u8]>,
) -> Result<Vec<u8>, SerializeError> {
    // 원본 metadata 블록(있으면) — 본문과 무관한 저작자/일자/주제 보존용.
    let original_str = original_content_hpf.and_then(|b| std::str::from_utf8(b).ok());
    let original_metadata = original_str.and_then(extract_metadata_block);

    let buf = Cursor::new(Vec::new());
    let mut w = Writer::new(buf);

    write_xml_decl(&mut w)?;

    // 한컴 HWPX 2011/2016 네임스페이스 + 표준 스키마
    start_tag_attrs(
        &mut w,
        "opf:package",
        &[
            ("xmlns:ha", "http://www.hancom.co.kr/hwpml/2011/app"),
            ("xmlns:hp", "http://www.hancom.co.kr/hwpml/2011/paragraph"),
            ("xmlns:hp10", "http://www.hancom.co.kr/hwpml/2016/paragraph"),
            ("xmlns:hs", "http://www.hancom.co.kr/hwpml/2011/section"),
            ("xmlns:hc", "http://www.hancom.co.kr/hwpml/2011/core"),
            ("xmlns:hh", "http://www.hancom.co.kr/hwpml/2011/head"),
            ("xmlns:hhs", "http://www.hancom.co.kr/hwpml/2011/history"),
            ("xmlns:hm", "http://www.hancom.co.kr/hwpml/2011/master-page"),
            ("xmlns:hpf", "http://www.hancom.co.kr/schema/2011/hpf"),
            ("xmlns:dc", "http://purl.org/dc/elements/1.1/"),
            ("xmlns:opf", "http://www.idpf.org/2007/opf/"),
            (
                "xmlns:ooxmlchart",
                "http://www.hancom.co.kr/hwpml/2016/ooxmlchart",
            ),
            (
                "xmlns:hwpunitchar",
                "http://www.hancom.co.kr/hwpml/2016/HwpUnitChar",
            ),
            ("xmlns:epub", "http://www.idpf.org/2007/ops"),
            (
                "xmlns:config",
                "urn:oasis:names:tc:opendocument:xmlns:config:1.0",
            ),
            ("version", ""),
            ("unique-identifier", ""),
            ("id", ""),
        ],
    )?;

    // <opf:metadata> — 원본 블록이 있으면 그대로 splice(저작자/일자/주제 보존),
    // 없으면(HWP5 등) 하드코딩 기본값으로 폴백.
    if let Some(meta) = original_metadata {
        // quick_xml Writer 가 동기 쓰기이므로 현재 위치에 raw 바이트를 직접 기록한다.
        w.get_mut()
            .write_all(meta.as_bytes())
            .map_err(|e| SerializeError::XmlError(format!("metadata splice: {e}")))?;
    } else {
        start_tag_attrs(&mut w, "opf:metadata", &[])?;
        empty_tag(&mut w, "opf:title", &[])?;
        start_tag_attrs(&mut w, "opf:language", &[])?;
        text(&mut w, "ko")?;
        end_tag(&mut w, "opf:language")?;
        start_tag_attrs(
            &mut w,
            "opf:meta",
            &[("name", "creator"), ("content", "text")],
        )?;
        text(&mut w, "rhwp")?;
        end_tag(&mut w, "opf:meta")?;
        empty_tag(
            &mut w,
            "opf:meta",
            &[("name", "CreatedDate"), ("content", "text")],
        )?;
        empty_tag(
            &mut w,
            "opf:meta",
            &[("name", "ModifiedDate"), ("content", "text")],
        )?;
        end_tag(&mut w, "opf:metadata")?;
    }

    // <opf:manifest>
    start_tag_attrs(&mut w, "opf:manifest", &[])?;

    empty_tag(
        &mut w,
        "opf:item",
        &[
            ("id", "header"),
            ("href", "Contents/header.xml"),
            ("media-type", "application/xml"),
        ],
    )?;

    for (i, href) in section_hrefs.iter().enumerate() {
        let id = format!("section{}", i);
        empty_tag(
            &mut w,
            "opf:item",
            &[
                ("id", id.as_str()),
                ("href", href.as_str()),
                ("media-type", "application/xml"),
            ],
        )?;
    }

    // 바탕쪽(masterpage) 등록 — section XML 의 idRef 와 id 가 일치해야 파서가 바인딩한다.
    for (id, href) in master_items {
        empty_tag(
            &mut w,
            "opf:item",
            &[
                ("id", id.as_str()),
                ("href", href.as_str()),
                ("media-type", "application/xml"),
            ],
        )?;
    }

    // settings.xml 등록
    empty_tag(
        &mut w,
        "opf:item",
        &[
            ("id", "settings"),
            ("href", "settings.xml"),
            ("media-type", "application/xml"),
        ],
    )?;

    for entry in bin_data {
        empty_tag(
            &mut w,
            "opf:item",
            &[
                ("id", entry.id.as_str()),
                ("href", entry.href.as_str()),
                ("media-type", entry.media_type.as_str()),
                ("isEmbeded", "1"),
            ],
        )?;
    }

    end_tag(&mut w, "opf:manifest")?;

    // <opf:spine> — 한컴 원본은 itemref 마다 linear="yes"(OPF 기본값)를 명시한다.
    start_tag_attrs(&mut w, "opf:spine", &[])?;
    empty_tag(
        &mut w,
        "opf:itemref",
        &[("idref", "header"), ("linear", "yes")],
    )?;
    for i in 0..section_hrefs.len() {
        let id = format!("section{}", i);
        empty_tag(
            &mut w,
            "opf:itemref",
            &[("idref", id.as_str()), ("linear", "yes")],
        )?;
    }
    end_tag(&mut w, "opf:spine")?;

    end_tag(&mut w, "opf:package")?;

    Ok(w.into_inner().into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 원본 metadata 블록이 주어지면 저작자/일자/주제가 그대로 보존되고,
    /// spine itemref 에는 linear="yes" 가, package 에는 hwpunitchar 네임스페이스가
    /// 출력되어야 한다.
    #[test]
    fn metadata_block_preserved_verbatim() {
        let original = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes" ?><opf:package xmlns:opf="http://www.idpf.org/2007/opf/"><opf:metadata><opf:title/><opf:language>ko</opf:language><opf:meta name="creator" content="text">손규현</opf:meta><opf:meta name="lastsaveby" content="text">stevek</opf:meta><opf:meta name="CreatedDate" content="text">2012-05-24T02:06:03Z</opf:meta></opf:metadata><opf:manifest/><opf:spine/></opf:package>"#;
        let out = write_content_hpf(
            &["Contents/section0.xml".to_string()],
            &[],
            &[],
            Some(original.as_bytes()),
        )
        .expect("serialize");
        let s = String::from_utf8(out).expect("utf8");

        assert!(
            s.contains(r#"<opf:meta name="creator" content="text">손규현</opf:meta>"#),
            "original creator must be preserved (not overwritten with rhwp): {s}"
        );
        assert!(
            s.contains(r#"<opf:meta name="lastsaveby" content="text">stevek</opf:meta>"#),
            "lastsaveby must be preserved"
        );
        assert!(
            s.contains(
                r#"<opf:meta name="CreatedDate" content="text">2012-05-24T02:06:03Z</opf:meta>"#
            ),
            "CreatedDate value must be preserved"
        );
        assert!(
            !s.contains(">rhwp<"),
            "hardcoded rhwp creator must not appear when original exists"
        );
        assert!(
            s.contains(r#"<opf:itemref idref="section0" linear="yes"/>"#),
            "spine itemref must carry linear=yes"
        );
        assert!(
            s.contains("xmlns:hwpunitchar=\"http://www.hancom.co.kr/hwpml/2016/HwpUnitChar\""),
            "package must declare hwpunitchar namespace"
        );
    }

    /// 원본이 없으면(HWP5 등) 하드코딩 metadata 로 폴백한다.
    #[test]
    fn metadata_falls_back_when_no_original() {
        let out = write_content_hpf(&["Contents/section0.xml".to_string()], &[], &[], None)
            .expect("serialize");
        let s = String::from_utf8(out).expect("utf8");
        assert!(s.contains(r#"<opf:meta name="creator" content="text">rhwp</opf:meta>"#));
        assert!(s.contains(r#"<opf:itemref idref="section0" linear="yes"/>"#));
    }

    /// metadata 블록을 인식하지 못하면(깨진 입력) None → 폴백.
    #[test]
    fn extract_metadata_handles_missing() {
        assert_eq!(extract_metadata_block("<opf:package/>"), None);
        assert_eq!(
            extract_metadata_block("<opf:metadata><opf:title/></opf:metadata>"),
            Some("<opf:metadata><opf:title/></opf:metadata>")
        );
    }
}
