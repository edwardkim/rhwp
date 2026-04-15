//! content.hpf 파싱 — 패키지 매니페스트에서 섹션 파일 목록과 BinData 목록 추출
//!
//! content.hpf는 OPF(Open Packaging Format) 형식의 XML로,
//! `<opf:manifest>` 내의 `<opf:item>` 요소에서 섹션/이미지 파일을 식별한다.

use quick_xml::events::Event;
use quick_xml::Reader;

use super::HwpxError;

/// 패키지 내 파일 항목
#[derive(Debug, Clone)]
pub struct PackageItem {
    /// 파일 경로 (ZIP 내 상대 경로, 예: "BinData/image1.png")
    pub href: String,
    /// MIME 유형
    pub media_type: String,
    /// 항목 ID
    pub id: String,
}

/// content.hpf 파싱 결과
#[derive(Debug, Default)]
pub struct PackageInfo {
    /// 섹션 XML 파일 경로 목록 (순서 보존)
    pub section_files: Vec<String>,
    /// BinData 항목 목록
    pub bin_data_items: Vec<PackageItem>,
}

/// content.hpf XML을 파싱하여 섹션/BinData 목록을 추출한다.
pub fn parse_content_hpf(xml: &str) -> Result<PackageInfo, HwpxError> {
    let mut reader = Reader::from_str(xml);
    let mut info = PackageInfo::default();
    let mut buf = Vec::new();

    // 임시 저장: 모든 item을 수집 후 섹션은 spine 순서로 정렬
    let mut all_items: Vec<(String, String, String)> = Vec::new(); // (id, href, media_type)
    let mut spine_order: Vec<String> = Vec::new(); // idref 순서

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Empty(ref e)) | Ok(Event::Start(ref e)) => {
                let ename = e.name();
                let local_name = local_tag_name(ename.as_ref());
                match local_name {
                    b"item" => {
                        let mut id = String::new();
                        let mut href = String::new();
                        let mut media_type = String::new();
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"id" => id = attr_value(&attr),
                                b"href" => href = attr_value(&attr),
                                b"media-type" => media_type = attr_value(&attr),
                                _ => {}
                            }
                        }
                        if !id.is_empty() && !href.is_empty() {
                            all_items.push((id, href, media_type));
                        }
                    }
                    b"itemref" => {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"idref" {
                                spine_order.push(attr_value(&attr));
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("content.hpf: {}", e))),
            _ => {}
        }
        buf.clear();
    }

    // spine 순서대로 섹션 파일 추출
    for idref in &spine_order {
        if let Some((_, href, media_type)) = all_items.iter().find(|(id, _, _)| id == idref) {
            if media_type == "application/xml" && href.contains("section") {
                info.section_files.push(href.clone());
            }
        }
    }

    // spine에 없는 섹션도 manifest에서 추출 (fallback)
    if info.section_files.is_empty() {
        let mut section_items: Vec<_> = all_items
            .iter()
            .filter(|(_, href, mt)| mt == "application/xml" && href.contains("section"))
            .collect();
        section_items.sort_by(|a, b| a.1.cmp(&b.1));
        info.section_files = section_items
            .into_iter()
            .map(|(_, href, _)| href.clone())
            .collect();
    }

    // BinData 항목 추출
    for (id, href, media_type) in &all_items {
        if href.starts_with("BinData/") || href.contains("/BinData/") {
            info.bin_data_items.push(PackageItem {
                href: href.clone(),
                media_type: media_type.clone(),
                id: id.clone(),
            });
        }
    }

    Ok(info)
}

/// XML 어트리뷰트 값을 String으로 변환
fn attr_value(attr: &quick_xml::events::attributes::Attribute) -> String {
    String::from_utf8_lossy(&attr.value).to_string()
}

/// 네임스페이스 접두사를 제거하고 로컬 태그 이름을 반환
fn local_tag_name(name: &[u8]) -> &[u8] {
    if let Some(pos) = name.iter().position(|&b| b == b':') {
        &name[pos + 1..]
    } else {
        name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content_hpf() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<opf:package xmlns:opf="http://www.idpf.org/2007/opf/" version="" unique-identifier="" id="">
  <opf:manifest>
    <opf:item id="header" href="Contents/header.xml" media-type="application/xml"/>
    <opf:item id="image1" href="BinData/image1.png" media-type="image/png" isEmbeded="1"/>
    <opf:item id="image2" href="BinData/image2.jpg" media-type="image/jpeg" isEmbeded="1"/>
    <opf:item id="section0" href="Contents/section0.xml" media-type="application/xml"/>
    <opf:item id="section1" href="Contents/section1.xml" media-type="application/xml"/>
  </opf:manifest>
  <opf:spine>
    <opf:itemref idref="header" linear="yes"/>
    <opf:itemref idref="section0" linear="yes"/>
    <opf:itemref idref="section1" linear="yes"/>
  </opf:spine>
</opf:package>"#;

        let info = parse_content_hpf(xml).unwrap();
        assert_eq!(info.section_files.len(), 2);
        assert_eq!(info.section_files[0], "Contents/section0.xml");
        assert_eq!(info.section_files[1], "Contents/section1.xml");
        assert_eq!(info.bin_data_items.len(), 2);
        assert_eq!(info.bin_data_items[0].href, "BinData/image1.png");
        assert_eq!(info.bin_data_items[1].id, "image2");
    }

    #[test]
    fn test_parse_empty_content() {
        let xml = r#"<?xml version="1.0"?><opf:package xmlns:opf="http://www.idpf.org/2007/opf/"><opf:manifest/><opf:spine/></opf:package>"#;
        let info = parse_content_hpf(xml).unwrap();
        assert!(info.section_files.is_empty());
        assert!(info.bin_data_items.is_empty());
    }
}
