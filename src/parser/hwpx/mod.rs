//! HWPX 파일 파서 모듈
//!
//! HWPX(XML 기반 HWP) 파일을 파싱하여 Document 모델로 변환한다.
//! HWPX는 ZIP 패키지 내 XML 파일로 구성된 KS X 6101:2024 표준 포맷이다.
//!
//! ## 파싱 순서
//! 1. ZIP 컨테이너 열기 (reader)
//! 2. content.hpf → 섹션 파일 목록 추출 (content)
//! 3. header.xml → DocInfo 변환 (header)
//! 4. section*.xml → Section 변환 (section)
//! 5. BinData → 이미지 로딩

pub mod content;
mod contract_streams;
pub mod header;
pub mod reader;
pub mod section;
pub mod utils;

use std::collections::{HashMap, HashSet};

use crate::model::bin_data::{BinData, BinDataContent, BinDataType};
use crate::model::document::{Document, FileHeader, HwpVersion, Section};

fn is_internal_bin_data_href(href: &str) -> bool {
    let href = href.to_ascii_lowercase();
    href.starts_with("bindata/") || href.contains("/bindata/")
}

fn is_internal_ole_package_item(item: &content::PackageItem) -> bool {
    let href = item.href.to_ascii_lowercase();
    is_internal_bin_data_href(&href)
        && (item.media_type.eq_ignore_ascii_case("application/ole") || href.ends_with(".ole"))
}

fn hwpx_bin_data_extension(item: &content::PackageItem) -> String {
    if is_internal_ole_package_item(item) {
        "OLE".to_string()
    } else {
        item.href.rsplit('.').next().unwrap_or("dat").to_string()
    }
}

fn normalize_internal_ole_data(item: &content::PackageItem, mut data: Vec<u8>) -> Vec<u8> {
    if !is_internal_ole_package_item(item) || data.len() <= 12 {
        return data;
    }

    const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
    if data[..8] != CFB_MAGIC && data[4..12] == CFB_MAGIC {
        data.drain(..4);
    }
    data
}

/// HWPX 파싱 에러
#[derive(Debug)]
pub enum HwpxError {
    /// ZIP 컨테이너 오류
    ZipError(String),
    /// XML 파싱 오류
    XmlError(String),
    /// 필수 파일 누락
    MissingFile(String),
    /// 데이터 변환 오류
    ConversionError(String),
    /// [Issue #1946] 비밀번호 암호화 HWPX(ODF encryption-data, AES-256-CBC).
    /// 복호화 미지원 — 암호문을 UTF-8 로 오독하는 대신 명확히 분류한다.
    Encrypted(String),
}

impl HwpxError {
    /// 암호화 문서 여부 — 배치 게이트의 ENCRYPTED_SKIP 분류에 사용.
    pub fn is_encrypted(&self) -> bool {
        matches!(self, HwpxError::Encrypted(_))
    }
}

impl std::fmt::Display for HwpxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HwpxError::ZipError(e) => write!(f, "ZIP 오류: {}", e),
            HwpxError::XmlError(e) => write!(f, "XML 파싱 오류: {}", e),
            HwpxError::MissingFile(e) => write!(f, "필수 파일 누락: {}", e),
            HwpxError::ConversionError(e) => write!(f, "변환 오류: {}", e),
            HwpxError::Encrypted(e) => write!(f, "암호화된 문서(복호화 미지원): {}", e),
        }
    }
}

impl std::error::Error for HwpxError {}

/// [Issue #1946] META-INF/manifest.xml 바이트에서 ODF 암호화 표식을 감지한다.
/// 암호화면 알고리즘 요약을, 아니면 None 을 반환한다. manifest 자체는 평문이므로
/// UTF-8 손실 없이 검사 가능하나, 안전을 위해 lossy 로 읽어 부분 손상에도 동작한다.
fn detect_odf_encryption(manifest_bytes: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(manifest_bytes);
    if !text.contains("encryption-data") {
        return None;
    }
    let algo = if text.contains("aes256-cbc") {
        "AES-256-CBC"
    } else if text.contains("aes128-cbc") {
        "AES-128-CBC"
    } else {
        "미상 알고리즘"
    };
    let kdf = if text.contains("pbkdf2") {
        " + PBKDF2"
    } else {
        ""
    };
    Some(format!(
        "ODF encryption-data 감지 ({}{}) — 비밀번호 보호 문서",
        algo, kdf
    ))
}

impl From<zip::result::ZipError> for HwpxError {
    fn from(e: zip::result::ZipError) -> Self {
        HwpxError::ZipError(e.to_string())
    }
}

impl From<quick_xml::Error> for HwpxError {
    fn from(e: quick_xml::Error) -> Self {
        HwpxError::XmlError(e.to_string())
    }
}

fn resolve_master_page_hrefs<'a, 'b>(
    id_refs: &'b [String],
    master_page_items: &'a [content::PackageItem],
) -> (Vec<&'a str>, Vec<&'b str>) {
    let href_by_id: HashMap<&str, &str> = master_page_items
        .iter()
        .map(|item| (item.id.as_str(), item.href.as_str()))
        .collect();
    let mut seen_hrefs = HashSet::new();
    let mut hrefs = Vec::new();
    let mut missing_refs = Vec::new();

    for id_ref in id_refs {
        match href_by_id.get(id_ref.as_str()).copied() {
            Some(href) if seen_hrefs.insert(href) => hrefs.push(href),
            Some(_) => {}
            None => missing_refs.push(id_ref.as_str()),
        }
    }

    (hrefs, missing_refs)
}

fn attach_hwpx_master_page(
    reader: &mut reader::HwpxReader,
    section: &mut Section,
    master_page_href: &str,
) -> bool {
    match reader.read_file(master_page_href) {
        Ok(master_page_xml) => match section::parse_hwpx_master_page(&master_page_xml) {
            Ok(master_page) => {
                section.section_def.master_pages.push(master_page);
                true
            }
            Err(e) => {
                eprintln!("경고: {} 파싱 실패: {}", master_page_href, e);
                false
            }
        },
        Err(e) => {
            eprintln!("경고: {} 읽기 실패: {}", master_page_href, e);
            false
        }
    }
}

/// HWPX 파일 바이트 데이터를 파싱하여 Document IR로 변환
pub fn parse_hwpx(data: &[u8]) -> Result<Document, HwpxError> {
    // 1. ZIP 컨테이너 열기
    let mut reader = reader::HwpxReader::open(data)?;

    // [Issue #1946] 암호화 HWPX 조기 감지. META-INF/manifest.xml 은 암호화 문서에서도
    // 평문이며, 암호화된 엔트리마다 <odf:encryption-data> 블록을 갖는다. 감지하면
    // 암호문(Contents/*.xml)을 UTF-8 로 오독하기 전에 명확한 Encrypted 에러로 반환한다
    // (종전엔 "UTF-8 변환 실패" 오진단). manifest 부재/평문 문서는 종전 경로 유지.
    if let Ok(manifest) = reader.read_file_bytes("META-INF/manifest.xml") {
        if let Some(detail) = detect_odf_encryption(&manifest) {
            return Err(HwpxError::Encrypted(detail));
        }
    }

    // 1-1. 보조 엔트리 원본 보존 (라운드트립 무손실).
    //   IR 로 모델링되지 않는 엔트리(version.xml/settings.xml/Preview/*)는
    //   직렬화기가 하드코딩 상수로 재생성하면서 원본 플랫폼/인쇄설정/미리보기를
    //   잃는다. 여기서 원본 바이트를 그대로 보존해 직렬화 시 passthrough 한다.
    const HWPX_AUX_PATHS: &[&str] = &[
        "version.xml",
        "settings.xml",
        "Preview/PrvText.txt",
        "Preview/PrvImage.png",
        crate::model::document::HWP5_ORIGIN_HWPX_MARKER_PATH,
    ];
    let mut hwpx_aux_entries: Vec<(String, Vec<u8>)> = Vec::new();
    for path in HWPX_AUX_PATHS {
        if let Ok(bytes) = reader.read_file_bytes(path) {
            hwpx_aux_entries.push((path.to_string(), bytes));
        }
    }

    // 2. content.hpf → 섹션 파일 목록 + BinData 목록
    let content_xml = reader.read_file("Contents/content.hpf")?;
    // content.hpf 의 manifest/spine 은 본문 의존(섹션/BinData)이라 재생성하지만,
    // <opf:metadata>(저작자/생성·수정일자/주제 등)는 본문과 무관하므로 직렬화 시
    // 원본 블록을 그대로 splice 하기 위해 원본 바이트를 보존한다.
    hwpx_aux_entries.push((
        "Contents/content.hpf".to_string(),
        content_xml.clone().into_bytes(),
    ));
    let package_info = content::parse_content_hpf(&content_xml)?;

    // 3. header.xml → DocInfo, DocProperties
    let header_xml = reader.read_file("Contents/header.xml")?;
    let (mut doc_info, doc_properties) = header::parse_hwpx_header(&header_xml)?;

    // [Task #1608] head version("1.4")은 HWPML **스키마 버전**일 뿐 HWP3→HWPX 변환 지표가
    // 아니다. 네이티브 한글2022 HWPX(version.xml: major=5 minor=1 "Hancom Office Hangul")도
    // head version 1.4 라, 과거 `is_hwp3_origin = (head version == "1.4")` (Task #554) 판정은
    // 거의 모든 모던 HWPX 를 HWP3-origin 으로 오탐지해 부당한 "마지막 줄" tolerance(1600 HU)를
    // 부여했고, 이것이 경계 문서를 1쪽 적게 렌더하는 −1쪽 갭의 한 요인이었다(Task #1600 요인 A).
    // 메타데이터로 진짜 변환본과 네이티브를 구별할 판별자가 없어(조사 확정), 파싱 시점의 HWP3
    // tolerance 부여를 제거한다.
    let hwpml_version = header::parse_hwpx_hwpml_version(&header_xml);
    // 무손실: 원본 HWPML 버전을 보존해 직렬화 때 그대로 재방출(하드코딩 금지).
    doc_info.hwpml_version = hwpml_version.clone();

    // BinData 목록을 DocInfo에 등록
    // [Task #873] isEmbeded="0" 인 외부 file 참조 (예: HWP3 → HWPX 변환본 의 절대 경로)
    // 는 BinDataType::Link + abs_path 로 등록. 이후 populate_link_image_paths (parser/mod.rs)
    // 가 Picture.external_path 설정 → Task #741 fallback 로 같은 dir 영역 image load.
    for (i, item) in package_info.bin_data_items.iter().enumerate() {
        let ext = hwpx_bin_data_extension(item);
        let (data_type, abs_path) = if is_internal_ole_package_item(item) {
            (BinDataType::Storage, None)
        } else if item.is_embedded {
            (BinDataType::Embedding, None)
        } else {
            (BinDataType::Link, Some(item.href.clone()))
        };
        doc_info.bin_data_list.push(BinData {
            data_type,
            storage_id: (i + 1) as u16,
            extension: Some(ext),
            abs_path,
            ..Default::default()
        });
    }

    // 4. section*.xml → Section 변환
    let mut sections = Vec::new();
    for (section_idx, section_href) in package_info.section_files.iter().enumerate() {
        let section_xml = reader.read_file(section_href)?;
        let master_page_refs = match section::collect_hwpx_section_master_page_refs(&section_xml) {
            Ok(refs) => refs,
            Err(e) => {
                eprintln!("경고: {} masterPage 참조 파싱 실패: {}", section_href, e);
                Vec::new()
            }
        };
        match section::parse_hwpx_section(&section_xml) {
            Ok(mut section) => {
                let (master_page_hrefs, missing_master_page_refs) =
                    resolve_master_page_hrefs(&master_page_refs, &package_info.master_page_items);
                for missing_ref in missing_master_page_refs {
                    eprintln!(
                        "경고: {} masterPage idRef '{}' manifest 항목 없음",
                        section_href, missing_ref
                    );
                }

                let mut attached_master_page_count = 0usize;
                for master_page_href in master_page_hrefs {
                    if attach_hwpx_master_page(&mut reader, &mut section, master_page_href) {
                        attached_master_page_count += 1;
                    }
                }

                if attached_master_page_count == 0 {
                    if let Some(master_page_files) =
                        package_info.section_master_page_files.get(section_idx)
                    {
                        let mut fallback_seen = HashSet::new();
                        for master_page_href in master_page_files {
                            if fallback_seen.insert(master_page_href.as_str()) {
                                attach_hwpx_master_page(
                                    &mut reader,
                                    &mut section,
                                    master_page_href,
                                );
                            }
                        }
                    }
                }
                sections.push(section);
            }
            Err(e) => {
                eprintln!("경고: {} 파싱 실패: {}", section_href, e);
                sections.push(Section::default());
            }
        }
    }

    // [Task #1608] (제거) 과거 Task #554 의 HWP3-origin tolerance 부여는
    // head version == "1.4" 오탐지로 네이티브 HWPX 전반에 부당 적용되어 삭제했다.
    // 상세 사유는 위 hwpml_version 파싱부 주석 참조.

    // 5. BinData 이미지 로딩
    let mut bin_data_content = Vec::new();
    for (i, item) in package_info.bin_data_items.iter().enumerate() {
        // [Task #873] isEmbeded="0" (외부 file 참조) 는 ZIP 영역 영역 부재. skip.
        // populate_link_image_paths + populate_external_images_from_dir 가 후처리.
        //
        // Issue #1283: 일부 HWPX는 ZIP 내부 OLE(`BinData/*.ole`)에도 isEmbeded="0"을
        // 기록한다. 이 경우는 외부 링크가 아니므로 로드해야 기존 OLE `/Contents`
        // 차트 렌더러가 동작한다.
        if !item.is_embedded && !is_internal_ole_package_item(item) {
            continue;
        }
        match reader.read_file_bytes(&item.href) {
            Ok(data) => {
                let data = normalize_internal_ole_data(item, data);
                let ext = hwpx_bin_data_extension(item);
                bin_data_content.push(BinDataContent {
                    id: (i + 1) as u16,
                    data,
                    extension: ext,
                });
            }
            Err(e) => {
                // [#1917] 로드 실패(상한 초과·엔트리 손상 등) 시에도 빈 데이터
                // placeholder를 등록해 manifest·binaryItemIDRef를 보존한다.
                // 미등록 시 직렬화기가 <hp:pic>를 통째로 드롭해 왕복 구조
                // 손실(IR_DIFF 하드 실패)이 발생한다. 이미지 데이터만 손실.
                eprintln!(
                    "경고: BinData '{}' 로드 실패: {} — placeholder 등록(이미지 데이터 소실)",
                    item.href, e
                );
                bin_data_content.push(BinDataContent {
                    id: (i + 1) as u16,
                    data: Vec::new(),
                    extension: hwpx_bin_data_extension(item),
                });
            }
        }
    }

    // 5-1. Chart/*.xml (OOXML 차트) 로딩 — bin_data_id = 60000+N, extension="ooxml_chart"
    // section 파서에서 <hp:chart chartIDRef="Chart/chartN.xml">를 만나면 동일 ID의 OleShape 생성
    for n in 1..=64u16 {
        let path = format!("Chart/chart{}.xml", n);
        match reader.read_file_bytes(&path) {
            Ok(data) => {
                bin_data_content.push(BinDataContent {
                    id: 60000 + n,
                    data,
                    extension: "ooxml_chart".to_string(),
                });
            }
            Err(_) => break,
        }
    }

    // Document 조립
    let model_header = FileHeader {
        version: HwpVersion {
            major: 5,
            minor: 1,
            build: 0,
            revision: 0,
        },
        flags: 0,
        compressed: false,
        encrypted: false,
        distribution: false,
        raw_data: None,
    };

    // [Task #852 Stage 2.1] HWPX ZIP 컨테이너 → HWP OLE contract 스트림 변환.
    // 한컴 HWP 정답지 contract (Preview/PrvText, Preview/PrvImage, Scripts/
    // DefaultJScript) 를 HWPX 컨테이너 동등 파일 (Preview/PrvText.txt,
    // Preview/PrvImage.png, Scripts/sourceScripts) 로부터 변환. HWPX 에
    // 동등 데이터가 없는 contract 스트림 (HwpSummaryInformation, DocOptions/
    // _LinkDoc, Scripts/JScriptVersion) 은 Stage 2.2 의 blank2010.hwp
    // fallback 으로 보강. cfb_writer (`src/serializer/cfb_writer.rs:155`)
    // 가 Document::extra_streams 를 그대로 OLE 스트림으로 작성.
    let contract = contract_streams::extract_contract_streams(&mut reader);

    let mut doc = Document {
        header: model_header,
        doc_properties,
        doc_info,
        sections,
        preview: None,
        bin_data_content,
        extra_streams: contract.streams,
        hwpx_aux_entries,
        is_hwp3_variant: false,
        is_hwpx_variant: false,
    };

    // [Task #873] BinData Link 타입 의 외부 file path 영역 영역 Picture.external_path 영역
    // 전달. 이후 model::document::populate_external_images_from_dir (Task #741) 가 같은
    // dir 영역 basename 매칭 영역 image 영역 자동 load. HWP5 parser 와 동일 처리.
    super::populate_link_image_paths(&mut doc);

    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hwpx_invalid_data() {
        let result = parse_hwpx(&[0u8; 10]);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_hwpx_not_zip() {
        // CFB/HWP 데이터로 시도
        let result = parse_hwpx(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]);
        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_master_page_hrefs_uses_id_ref_order_and_dedups() {
        let items = vec![
            content::PackageItem {
                id: "masterpage1".to_string(),
                href: "Contents/masterpage1.xml".to_string(),
                media_type: "application/xml".to_string(),
                is_embedded: true,
            },
            content::PackageItem {
                id: "masterpage0".to_string(),
                href: "Contents/masterpage0.xml".to_string(),
                media_type: "application/xml".to_string(),
                is_embedded: true,
            },
        ];
        let id_refs = vec![
            "masterpage0".to_string(),
            "missing".to_string(),
            "masterpage1".to_string(),
            "masterpage0".to_string(),
        ];

        let (hrefs, missing_refs) = resolve_master_page_hrefs(&id_refs, &items);

        assert_eq!(
            hrefs,
            vec!["Contents/masterpage0.xml", "Contents/masterpage1.xml"]
        );
        assert_eq!(missing_refs, vec!["missing"]);
    }
}
