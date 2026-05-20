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

use crate::model::bin_data::{BinData, BinDataContent, BinDataType};
use crate::model::document::{Document, FileHeader, HwpVersion, Section};

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
}

impl std::fmt::Display for HwpxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HwpxError::ZipError(e) => write!(f, "ZIP 오류: {}", e),
            HwpxError::XmlError(e) => write!(f, "XML 파싱 오류: {}", e),
            HwpxError::MissingFile(e) => write!(f, "필수 파일 누락: {}", e),
            HwpxError::ConversionError(e) => write!(f, "변환 오류: {}", e),
        }
    }
}

impl std::error::Error for HwpxError {}

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

/// HWPX 파일 바이트 데이터를 파싱하여 Document IR로 변환
pub fn parse_hwpx(data: &[u8]) -> Result<Document, HwpxError> {
    // 1. ZIP 컨테이너 열기
    let mut reader = reader::HwpxReader::open(data)?;

    // 2. content.hpf → 섹션 파일 목록 + BinData 목록
    let content_xml = reader.read_file("Contents/content.hpf")?;
    let package_info = content::parse_content_hpf(&content_xml)?;

    // 3. header.xml → DocInfo, DocProperties
    let header_xml = reader.read_file("Contents/header.xml")?;
    let (mut doc_info, doc_properties) = header::parse_hwpx_header(&header_xml)?;

    // [Task #554] HWP3 → HWPX 변환본 식별: hwpml 스키마 버전 = "1.4"
    // 변환본은 한글97의 "마지막 줄 tolerance" (1600 HU) 가 누락되어 페이지 수가
    // 늘어나므로, 본 시점에 식별하여 page_def.margin_bottom 보정 (post-process)에 사용.
    let hwpml_version = header::parse_hwpx_hwpml_version(&header_xml);
    let is_hwp3_origin = hwpml_version.as_deref() == Some("1.4");

    // BinData 목록을 DocInfo에 등록
    // [Task #873] isEmbeded="0" 인 외부 file 참조 (예: HWP3 → HWPX 변환본 의 절대 경로)
    // 는 BinDataType::Link + abs_path 로 등록. 이후 populate_link_image_paths (parser/mod.rs)
    // 가 Picture.external_path 설정 → Task #741 fallback 로 같은 dir 영역 image load.
    for (i, item) in package_info.bin_data_items.iter().enumerate() {
        let ext = item.href.rsplit('.').next().unwrap_or("dat").to_string();
        let (data_type, abs_path) = if item.is_embedded {
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
    for section_href in &package_info.section_files {
        let section_xml = reader.read_file(section_href)?;
        match section::parse_hwpx_section(&section_xml) {
            Ok(section) => sections.push(section),
            Err(e) => {
                eprintln!("경고: {} 파싱 실패: {}", section_href, e);
                sections.push(Section::default());
            }
        }
    }

    // [Task #554] HWP3 변환본 보정: 한글97의 마지막 줄 tolerance 모방
    // 모든 SectionDef.page_def 의 margin_bottom 을 1600 HU 줄여 한글97 페이지네이션과 정합.
    if is_hwp3_origin {
        for section in sections.iter_mut() {
            section.section_def.page_def.margin_bottom = section
                .section_def
                .page_def
                .margin_bottom
                .saturating_sub(1600);
        }
    }

    // 5. BinData 이미지 로딩
    let mut bin_data_content = Vec::new();
    for (i, item) in package_info.bin_data_items.iter().enumerate() {
        // [Task #873] isEmbeded="0" (외부 file 참조) 는 ZIP 영역 영역 부재. skip.
        // populate_link_image_paths + populate_external_images_from_dir 가 후처리.
        if !item.is_embedded {
            continue;
        }
        match reader.read_file_bytes(&item.href) {
            Ok(data) => {
                let ext = item.href.rsplit('.').next().unwrap_or("dat").to_string();
                bin_data_content.push(BinDataContent {
                    id: (i + 1) as u16,
                    data,
                    extension: ext,
                });
            }
            Err(e) => {
                eprintln!("경고: BinData '{}' 로드 실패: {}", item.href, e);
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
        is_hwp3_variant: false,
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
}
