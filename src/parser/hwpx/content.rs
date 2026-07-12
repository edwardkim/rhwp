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
    /// 또는 외부 파일 절대 경로 (예: `C:\path\to\image.gif`, [Task #873])
    pub href: String,
    /// MIME 유형
    pub media_type: String,
    /// 항목 ID
    pub id: String,
    /// [Task #873] isEmbeded attribute — true = ZIP 내 embedded, false = 외부 file 참조
    pub is_embedded: bool,
}

/// content.hpf 파싱 결과
#[derive(Debug, Default)]
pub struct PackageInfo {
    /// 섹션 XML 파일 경로 목록 (순서 보존)
    pub section_files: Vec<String>,
    /// 섹션별 바탕쪽 XML 파일 경로 목록.
    ///
    /// HWPX manifest는 masterpage 항목을 section 항목 앞에 배치하는 형태가 관찰된다.
    /// 예: masterpage0..2, section0, masterpage3..5, section1 ...
    pub section_master_page_files: Vec<Vec<String>>,
    /// 바탕쪽 XML manifest 항목 목록 (id/href 보존).
    ///
    /// section XML의 `<hp:masterPage idRef="...">`와 manifest item id를 매칭해
    /// 명시적으로 연결하기 위한 원본 정보다.
    pub master_page_items: Vec<PackageItem>,
    /// BinData 항목 목록
    pub bin_data_items: Vec<PackageItem>,
}

/// content.hpf XML을 파싱하여 섹션/BinData 목록을 추출한다.
pub fn parse_content_hpf(xml: &str) -> Result<PackageInfo, HwpxError> {
    let mut reader = Reader::from_str(xml);
    let mut info = PackageInfo::default();
    let mut buf = Vec::new();

    // 임시 저장: 모든 item을 수집 후 섹션은 spine 순서로 정렬
    // [Task #873] is_embedded 추가 — isEmbeded="0" (외부 file 참조) 식별
    let mut all_items: Vec<(String, String, String, bool)> = Vec::new(); // (id, href, media_type, is_embedded)
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
                        // [Task #873] 기본값 true (BinData/ 폴더 내 embedded image 가 일반적 케이스).
                        // isEmbeded="0" 인 경우만 false 로 설정.
                        let mut is_embedded = true;
                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"id" => id = attr_value(&attr),
                                b"href" => href = attr_value(&attr),
                                b"media-type" => media_type = attr_value(&attr),
                                b"isEmbeded" => {
                                    is_embedded = attr_value(&attr) != "0";
                                }
                                _ => {}
                            }
                        }
                        if !id.is_empty() && !href.is_empty() {
                            all_items.push((id, href, media_type, is_embedded));
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
        if let Some((_, href, media_type, _)) = all_items.iter().find(|(id, _, _, _)| id == idref) {
            if media_type == "application/xml" && href.contains("section") {
                info.section_files.push(href.clone());
            }
        }
    }

    // spine에 없는 섹션도 manifest에서 추출 (fallback)
    if info.section_files.is_empty() {
        let mut section_items: Vec<_> = all_items
            .iter()
            .filter(|(_, href, mt, _)| mt == "application/xml" && href.contains("section"))
            .collect();
        section_items.sort_by(|a, b| a.1.cmp(&b.1));
        info.section_files = section_items
            .into_iter()
            .map(|(_, href, _, _)| href.clone())
            .collect();
    }

    info.section_master_page_files = collect_section_master_pages(&all_items, &info.section_files);
    info.master_page_items = collect_master_page_items(&all_items);

    // BinData 항목 추출
    // [Task #873] 기존: BinData/ 폴더 내 항목만 수집 → isEmbeded="0" (외부 file 참조)
    // image 가 누락되어 HWP3 → HWPX 변환본의 image 가 표시되지 않음.
    // 정정: media_type 이 image/* 인 모든 item 도 수집.
    for (id, href, media_type, is_embedded) in &all_items {
        let is_image = media_type.starts_with("image/");
        let is_bin_data_path = href.starts_with("BinData/") || href.contains("/BinData/");
        // [#1891] 외부 참조(isEmbeded="0")는 media-type 과 무관하게 BinData 항목이다.
        // 원본이 "image/" 같은 퇴화 값을 갖거나 직렬화기가 octet-stream 으로 쓰는 등
        // media-type 만으로는 판별할 수 없고, 누락 시 이후 항목의 인덱스(=bin_data_id)
        // 가 밀려 그림 참조 전체가 어긋난다.
        let is_external_link = !*is_embedded && media_type != "application/xml";
        if is_image || is_bin_data_path || is_external_link {
            info.bin_data_items.push(PackageItem {
                href: href.clone(),
                media_type: media_type.clone(),
                id: id.clone(),
                is_embedded: *is_embedded,
            });
        }
    }

    Ok(info)
}

fn collect_master_page_items(all_items: &[(String, String, String, bool)]) -> Vec<PackageItem> {
    all_items
        .iter()
        .filter(|(_, href, media_type, _)| {
            media_type == "application/xml" && href.to_ascii_lowercase().contains("masterpage")
        })
        .map(|(id, href, media_type, is_embedded)| PackageItem {
            href: href.clone(),
            media_type: media_type.clone(),
            id: id.clone(),
            is_embedded: *is_embedded,
        })
        .collect()
}

fn collect_section_master_pages(
    all_items: &[(String, String, String, bool)],
    section_files: &[String],
) -> Vec<Vec<String>> {
    let mut groups = vec![Vec::new(); section_files.len()];
    if section_files.is_empty() {
        return groups;
    }

    let mut pending: Vec<String> = Vec::new();
    for (_, href, media_type, _) in all_items {
        if media_type != "application/xml" {
            continue;
        }
        let lower_href = href.to_ascii_lowercase();
        if lower_href.contains("masterpage") {
            pending.push(href.clone());
            continue;
        }
        if let Some(section_idx) = section_files.iter().position(|section| section == href) {
            if !pending.is_empty() {
                groups[section_idx].append(&mut pending);
            }
        }
    }

    if groups.iter().all(|group| group.is_empty()) {
        let mut master_pages: Vec<String> = all_items
            .iter()
            .filter(|(_, href, media_type, _)| {
                media_type == "application/xml" && href.to_ascii_lowercase().contains("masterpage")
            })
            .map(|(_, href, _, _)| href.clone())
            .collect();
        master_pages.sort();

        if !master_pages.is_empty() {
            let chunk = master_pages.len().div_ceil(section_files.len());
            for (section_idx, group) in groups.iter_mut().enumerate() {
                let start = section_idx * chunk;
                let end = ((section_idx + 1) * chunk).min(master_pages.len());
                if start < end {
                    group.extend_from_slice(&master_pages[start..end]);
                }
            }
        }
    }

    groups
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
        assert_eq!(info.section_master_page_files.len(), 2);
        assert_eq!(info.bin_data_items.len(), 2);
        assert_eq!(info.bin_data_items[0].href, "BinData/image1.png");
        assert_eq!(info.bin_data_items[1].id, "image2");
        // [Task #873] isEmbeded="1" → is_embedded = true
        assert!(info.bin_data_items[0].is_embedded);
        assert!(info.bin_data_items[1].is_embedded);
    }

    #[test]
    fn test_parse_content_hpf_external_image() {
        // [Task #873] HWP3 → HWPX 변환본: isEmbeded="0" + 외부 절대 경로
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<opf:package xmlns:opf="http://www.idpf.org/2007/opf/">
  <opf:manifest>
    <opf:item id="header" href="Contents/header.xml" media-type="application/xml"/>
    <opf:item id="image1" href="D:\Work\images\oracle.gif" media-type="image/gif" isEmbeded="0"/>
    <opf:item id="image2" href="BinData/image2.jpg" media-type="image/jpeg" isEmbeded="1"/>
    <opf:item id="section0" href="Contents/section0.xml" media-type="application/xml"/>
  </opf:manifest>
  <opf:spine><opf:itemref idref="section0"/></opf:spine>
</opf:package>"#;
        let info = parse_content_hpf(xml).unwrap();
        assert_eq!(info.bin_data_items.len(), 2);
        assert_eq!(info.bin_data_items[0].href, "D:\\Work\\images\\oracle.gif");
        assert!(!info.bin_data_items[0].is_embedded);
        assert_eq!(info.bin_data_items[1].href, "BinData/image2.jpg");
        assert!(info.bin_data_items[1].is_embedded);
    }

    #[test]
    fn test_parse_empty_content() {
        let xml = r#"<?xml version="1.0"?><opf:package xmlns:opf="http://www.idpf.org/2007/opf/"><opf:manifest/><opf:spine/></opf:package>"#;
        let info = parse_content_hpf(xml).unwrap();
        assert!(info.section_files.is_empty());
        assert!(info.bin_data_items.is_empty());
    }

    #[test]
    fn test_parse_content_hpf_master_pages_by_manifest_order() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<opf:package xmlns:opf="http://www.idpf.org/2007/opf/">
  <opf:manifest>
    <opf:item id="masterpage0" href="Contents/masterpage0.xml" media-type="application/xml"/>
    <opf:item id="masterpage1" href="Contents/masterpage1.xml" media-type="application/xml"/>
    <opf:item id="section0" href="Contents/section0.xml" media-type="application/xml"/>
    <opf:item id="masterpage2" href="Contents/masterpage2.xml" media-type="application/xml"/>
    <opf:item id="section1" href="Contents/section1.xml" media-type="application/xml"/>
  </opf:manifest>
  <opf:spine>
    <opf:itemref idref="section0"/>
    <opf:itemref idref="section1"/>
  </opf:spine>
</opf:package>"#;

        let info = parse_content_hpf(xml).unwrap();
        assert_eq!(info.master_page_items.len(), 3);
        assert_eq!(info.master_page_items[0].id, "masterpage0");
        assert_eq!(info.master_page_items[0].href, "Contents/masterpage0.xml");
        assert_eq!(info.master_page_items[1].id, "masterpage1");
        assert_eq!(info.master_page_items[1].href, "Contents/masterpage1.xml");
        assert_eq!(info.master_page_items[2].id, "masterpage2");
        assert_eq!(info.master_page_items[2].href, "Contents/masterpage2.xml");
        assert_eq!(
            info.section_master_page_files,
            vec![
                vec![
                    "Contents/masterpage0.xml".to_string(),
                    "Contents/masterpage1.xml".to_string()
                ],
                vec!["Contents/masterpage2.xml".to_string()]
            ]
        );
    }
}
