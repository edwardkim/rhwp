//! 재조립 HWPX 패키지(ZIP) 구조 검증 (Task #1315).
//!
//! `serialize_hwpx()` 산출 바이트가 한컴/OPF 패키지 규약을 지키는지 IR 대비 검사한다.
//!
//! 검사 항목:
//! 1. ZIP 아카이브로 열림
//! 2. `mimetype` — 최초 엔트리, STORED, 내용 `application/hwp+zip`
//! 3. 필수 엔트리 존재 (version.xml, header.xml, content.hpf, Preview, settings,
//!    META-INF/container.xml·container.rdf·manifest.xml)
//! 4. `Contents/section{N}.xml` 엔트리 수 = IR 섹션 수 (잉여 섹션 엔트리 금지)
//! 5. `Contents/content.hpf` manifest 가 참조하는 href 가 모두 ZIP 에 실재
//! 6. `BinData/` 엔트리 수·확장자 멀티셋 = IR `bin_data_content` 보존
//!
//! 주의: serializer 가 BinData href 를 `BinData/image{N}.{ext}` 로 재명명하므로
//! 원본 ZIP 의 엔트리 **이름**이 아니라 IR 기준 **수·확장자**를 보존 기준으로 삼는다.

use std::collections::HashSet;
use std::io::{Cursor, Read};

use crate::model::document::Document;

/// HWPX mimetype 고정 내용.
const HWPX_MIMETYPE: &[u8] = b"application/hwp+zip";

/// 섹션 수와 무관하게 항상 있어야 하는 엔트리.
const REQUIRED_ENTRIES: [&str; 9] = [
    "version.xml",
    "Contents/header.xml",
    "Contents/content.hpf",
    "Preview/PrvText.txt",
    "Preview/PrvImage.png",
    "settings.xml",
    "META-INF/container.xml",
    "META-INF/container.rdf",
    "META-INF/manifest.xml",
];

/// 패키지 검사 결과 — 발견된 문제 목록.
#[derive(Debug, Default)]
pub struct PackageCheckReport {
    pub problems: Vec<String>,
}

impl PackageCheckReport {
    pub fn is_ok(&self) -> bool {
        self.problems.is_empty()
    }

    pub fn summary(&self) -> String {
        self.problems.join("; ")
    }

    fn push(&mut self, problem: String) {
        self.problems.push(problem);
    }
}

/// 재조립 HWPX 바이트를 IR(`doc`) 기준으로 패키지 구조 검사한다.
///
/// `doc` 은 직렬화에 입력한 Document (원본 파싱 결과)여야 한다.
pub fn check_package(hwpx_bytes: &[u8], doc: &Document) -> PackageCheckReport {
    let mut report = PackageCheckReport::default();

    let mut archive = match zip::ZipArchive::new(Cursor::new(hwpx_bytes)) {
        Ok(a) => a,
        Err(e) => {
            report.push(format!("ZIP 열기 실패: {e}"));
            return report;
        }
    };

    let names: HashSet<String> = archive.file_names().map(String::from).collect();

    // 2. mimetype — 최초 엔트리 + STORED + 내용 일치
    match archive.by_index(0) {
        Ok(mut first) => {
            if first.name() != "mimetype" {
                report.push(format!(
                    "mimetype 이 최초 엔트리가 아님 (첫 엔트리: {})",
                    first.name()
                ));
            } else {
                if first.compression() != zip::CompressionMethod::Stored {
                    report.push(format!(
                        "mimetype 압축 방식이 STORED 가 아님: {:?}",
                        first.compression()
                    ));
                }
                let mut content = Vec::new();
                if first.read_to_end(&mut content).is_ok() {
                    if content != HWPX_MIMETYPE {
                        report.push(format!(
                            "mimetype 내용 불일치: {:?}",
                            String::from_utf8_lossy(&content)
                        ));
                    }
                } else {
                    report.push("mimetype 읽기 실패".to_string());
                }
            }
        }
        Err(e) => report.push(format!("첫 엔트리 접근 실패: {e}")),
    }

    // 3. 필수 엔트리
    for required in REQUIRED_ENTRIES {
        if !names.contains(required) {
            report.push(format!("필수 엔트리 누락: {required}"));
        }
    }

    // 4. 섹션 엔트리 수 = IR 섹션 수
    for i in 0..doc.sections.len() {
        let entry = format!("Contents/section{i}.xml");
        if !names.contains(&entry) {
            report.push(format!("섹션 엔트리 누락: {entry} (IR 섹션 {i})"));
        }
    }
    let section_entry_count = names.iter().filter(|n| is_section_entry_name(n)).count();
    if section_entry_count != doc.sections.len() {
        report.push(format!(
            "섹션 엔트리 수 불일치: zip={} ir={}",
            section_entry_count,
            doc.sections.len()
        ));
    }

    // 5. content.hpf manifest href 실재 확인
    match read_entry_string(&mut archive, "Contents/content.hpf") {
        Ok(hpf) => {
            for href in extract_hrefs(&hpf) {
                if !names.contains(href.as_str()) {
                    report.push(format!("content.hpf 참조 엔트리 누락: {href}"));
                }
            }
        }
        Err(e) => {
            // 필수 엔트리 검사에서 이미 누락 보고됐을 수 있으므로 읽기 실패만 기록
            if names.contains("Contents/content.hpf") {
                report.push(format!("content.hpf 읽기 실패: {e}"));
            }
        }
    }

    // 6. BinData 수·확장자 보존 (IR 기준)
    let zip_bin: Vec<&String> = names.iter().filter(|n| n.starts_with("BinData/")).collect();
    if zip_bin.len() != doc.bin_data_content.len() {
        report.push(format!(
            "BinData 엔트리 수 불일치: zip={} ir={}",
            zip_bin.len(),
            doc.bin_data_content.len()
        ));
    } else {
        let mut zip_exts: Vec<String> = zip_bin.iter().map(|n| extension_lower(n)).collect();
        let mut ir_exts: Vec<String> = doc
            .bin_data_content
            .iter()
            .map(|b| b.extension.to_ascii_lowercase())
            .collect();
        zip_exts.sort();
        ir_exts.sort();
        if zip_exts != ir_exts {
            report.push(format!(
                "BinData 확장자 멀티셋 불일치: zip={:?} ir={:?}",
                zip_exts, ir_exts
            ));
        }
    }

    report
}

/// `Contents/section{숫자}.xml` 형태인지 확인.
fn is_section_entry_name(name: &str) -> bool {
    name.strip_prefix("Contents/section")
        .and_then(|rest| rest.strip_suffix(".xml"))
        .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

/// ZIP 엔트리를 문자열로 읽는다.
fn read_entry_string(
    archive: &mut zip::ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> Result<String, String> {
    let mut entry = archive.by_name(name).map_err(|e| e.to_string())?;
    let mut s = String::new();
    entry.read_to_string(&mut s).map_err(|e| e.to_string())?;
    Ok(s)
}

/// XML 텍스트에서 `href="..."` 값을 단순 스캔으로 추출한다.
///
/// content.hpf 는 자체 writer 산출물이므로 따옴표 이스케이프 변형이 없다.
fn extract_hrefs(xml: &str) -> Vec<String> {
    let mut hrefs = Vec::new();
    let mut rest = xml;
    while let Some(pos) = rest.find("href=\"") {
        rest = &rest[pos + "href=\"".len()..];
        if let Some(end) = rest.find('"') {
            hrefs.push(rest[..end].to_string());
            rest = &rest[end + 1..];
        } else {
            break;
        }
    }
    hrefs
}

/// 파일명에서 소문자 확장자 추출 (없으면 빈 문자열).
fn extension_lower(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(_, ext)| ext.to_ascii_lowercase())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::bin_data::BinDataContent;
    use crate::serializer::hwpx::serialize_hwpx;

    fn doc_with_sections(n: usize) -> Document {
        let mut doc = Document::default();
        for _ in 0..n {
            doc.sections
                .push(crate::model::document::Section::default());
        }
        doc
    }

    #[test]
    fn empty_doc_package_passes() {
        let doc = Document::default();
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let report = check_package(&bytes, &doc);
        assert!(report.is_ok(), "problems: {}", report.summary());
    }

    #[test]
    fn one_section_package_passes() {
        let doc = doc_with_sections(1);
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let report = check_package(&bytes, &doc);
        assert!(report.is_ok(), "problems: {}", report.summary());
    }

    #[test]
    fn detects_missing_section_entry() {
        // 섹션 0개로 직렬화한 ZIP 을 "섹션 1개 IR" 기준으로 검사 → 누락 검출
        let doc0 = Document::default();
        let bytes = serialize_hwpx(&doc0).expect("serialize");
        let doc1 = doc_with_sections(1);
        let report = check_package(&bytes, &doc1);
        assert!(!report.is_ok());
        assert!(
            report.summary().contains("섹션 엔트리 누락"),
            "summary: {}",
            report.summary()
        );
    }

    #[test]
    fn detects_extra_section_entry() {
        // 섹션 1개로 직렬화한 ZIP 을 "섹션 0개 IR" 기준으로 검사 → 잉여 검출
        let doc1 = doc_with_sections(1);
        let bytes = serialize_hwpx(&doc1).expect("serialize");
        let doc0 = Document::default();
        let report = check_package(&bytes, &doc0);
        assert!(!report.is_ok());
        assert!(
            report.summary().contains("섹션 엔트리 수 불일치"),
            "summary: {}",
            report.summary()
        );
    }

    #[test]
    fn detects_bin_data_count_mismatch() {
        // BinData 없는 ZIP 을 "BinData 1개 IR" 기준으로 검사 → 불일치 검출
        let doc0 = Document::default();
        let bytes = serialize_hwpx(&doc0).expect("serialize");
        let mut doc_bin = Document::default();
        doc_bin.bin_data_content.push(BinDataContent {
            id: 1,
            data: vec![1, 2, 3],
            extension: "png".to_string(),
        });
        let report = check_package(&bytes, &doc_bin);
        assert!(!report.is_ok());
        assert!(
            report.summary().contains("BinData 엔트리 수 불일치"),
            "summary: {}",
            report.summary()
        );
    }

    #[test]
    fn rejects_non_zip_bytes() {
        let doc = Document::default();
        let report = check_package(b"not a zip at all", &doc);
        assert!(!report.is_ok());
        assert!(
            report.summary().contains("ZIP 열기 실패"),
            "summary: {}",
            report.summary()
        );
    }

    #[test]
    fn section_entry_name_matcher() {
        assert!(is_section_entry_name("Contents/section0.xml"));
        assert!(is_section_entry_name("Contents/section12.xml"));
        assert!(!is_section_entry_name("Contents/section.xml"));
        assert!(!is_section_entry_name("Contents/sectionA.xml"));
        assert!(!is_section_entry_name("Contents/header.xml"));
    }

    #[test]
    fn extract_hrefs_basic() {
        let xml = r#"<opf:item href="Contents/header.xml"/><opf:item href="settings.xml"/>"#;
        assert_eq!(
            extract_hrefs(xml),
            vec![
                "Contents/header.xml".to_string(),
                "settings.xml".to_string()
            ]
        );
    }
}
