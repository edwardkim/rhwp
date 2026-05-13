//! HWPX raw XML 보존본 게터 (paste fragment wasm bridge 용).
//!
//! `DocumentCore::from_bytes` 가 HWPX 로 로드한 경우 `source_section_xmls`/`source_header_xml`
//! 에 원본 `Contents/section{N}.xml` 과 `Contents/header.xml` 문자열이 byte-exact 로 보존된다.
//! HWP 로드 시에는 둘 다 비어있다.
//!
//! Stage 2 의 `paste_hwpx_fragment_in_document` 가 이 게터로 raw XML 을 꺼낸 뒤
//! `paste_fragment_into_section` 을 호출하고, 결과를 다시 동일 슬롯에 저장한다.

use crate::document_core::DocumentCore;

impl DocumentCore {
    /// 보존된 섹션 raw XML 을 반환한다.
    /// HWPX 로 로드된 경우 인덱스가 유효하면 원본 문자열, 그 외엔 `None`.
    pub fn get_source_section_xml(&self, section_idx: usize) -> Option<&str> {
        self.source_section_xmls
            .get(section_idx)
            .map(|s| s.as_str())
    }

    /// 보존된 header.xml raw 문자열을 반환한다.
    /// HWPX 로 로드된 경우 원본 문자열, HWP 의 경우 빈 문자열.
    pub fn get_source_header_xml(&self) -> &str {
        self.source_header_xml.as_str()
    }

    /// 보존된 섹션 개수 (`source_section_xmls.len()`).
    pub fn source_section_xml_count(&self) -> usize {
        self.source_section_xmls.len()
    }

    /// HWPX raw XML 보존 여부.
    /// `true` 이면 paste fragment wasm bridge 호출이 가능, `false` 이면 NoSourceXml 류 에러.
    pub fn has_source_xmls(&self) -> bool {
        !self.source_section_xmls.is_empty() && !self.source_header_xml.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 다중 섹션 hwpx 샘플을 hwpx 로 로드 → raw XML byte-exact 일치 검증.
    /// 표준 샘플이 없는 경우 단일 섹션 fallback.
    fn locate_hwpx_sample() -> Option<std::path::PathBuf> {
        let candidates = [
            "samples/standard.hwpx",
            "samples/page_layout.hwpx",
            "samples/lineseg.hwpx",
        ];
        for c in candidates {
            let p = std::path::Path::new(c);
            if p.exists() {
                return Some(p.to_path_buf());
            }
        }
        None
    }

    #[test]
    fn raw_xml_byte_exact_after_hwpx_load() {
        let Some(sample) = locate_hwpx_sample() else {
            eprintln!("HWPX 샘플 없음 — 테스트 스킵");
            return;
        };
        let bytes = std::fs::read(&sample).expect("read sample");
        let core = DocumentCore::from_bytes(&bytes).expect("from_bytes");

        assert!(core.has_source_xmls(), "HWPX 로드 후 source XML 보존 기대");
        let header = core.get_source_header_xml();
        assert!(
            header.contains("<hh:head") || header.contains("<head"),
            "header.xml 본문 확인"
        );

        // ZIP 에서 직접 읽은 raw 와 byte-exact 일치 검증.
        let mut zip = zip::ZipArchive::new(std::io::Cursor::new(&bytes)).expect("zip open");
        let mut zip_header = String::new();
        std::io::Read::read_to_string(
            &mut zip.by_name("Contents/header.xml").expect("header"),
            &mut zip_header,
        )
        .expect("read header");
        assert_eq!(header, zip_header.as_str(), "header.xml byte-exact 일치");

        // 섹션 0 도 동일 검증.
        let section0 = core
            .get_source_section_xml(0)
            .expect("section 0 보존 기대");
        let mut zip_section0 = String::new();
        std::io::Read::read_to_string(
            &mut zip.by_name("Contents/section0.xml").expect("section0"),
            &mut zip_section0,
        )
        .expect("read section0");
        assert_eq!(
            section0,
            zip_section0.as_str(),
            "section0.xml byte-exact 일치"
        );
    }

    #[test]
    fn empty_document_has_no_source_xmls() {
        let core = DocumentCore::new_empty();
        assert!(!core.has_source_xmls());
        assert_eq!(core.source_section_xml_count(), 0);
        assert_eq!(core.get_source_header_xml(), "");
        assert!(core.get_source_section_xml(0).is_none());
    }

    #[test]
    fn hwp_load_leaves_source_xmls_empty() {
        // HWP 샘플이 있으면 로드, 아니면 스킵.
        let candidates = ["samples/blank.hwp", "samples/standard.hwp"];
        let mut found: Option<std::path::PathBuf> = None;
        for c in candidates {
            let p = std::path::Path::new(c);
            if p.exists() {
                found = Some(p.to_path_buf());
                break;
            }
        }
        let Some(sample) = found else {
            eprintln!("HWP 샘플 없음 — 테스트 스킵");
            return;
        };
        let bytes = std::fs::read(&sample).expect("read sample");
        let core = DocumentCore::from_bytes(&bytes).expect("from_bytes");
        assert!(
            !core.has_source_xmls(),
            "HWP 로드는 raw XML 보존 안함 (graceful)"
        );
        assert_eq!(core.source_section_xml_count(), 0);
    }

    #[test]
    fn out_of_range_section_returns_none() {
        let core = DocumentCore::new_empty();
        assert!(core.get_source_section_xml(0).is_none());
        assert!(core.get_source_section_xml(99).is_none());
    }

    #[test]
    fn manual_mutation_reflected_in_getters() {
        // 게터가 in-place 갱신을 그대로 반영하는지 검증 (Stage 2 paste 흐름의 사전 보장).
        let mut core = DocumentCore::new_empty();
        core.source_section_xmls.push("<section/>".into());
        core.source_header_xml = "<header/>".into();
        assert_eq!(core.get_source_section_xml(0), Some("<section/>"));
        assert_eq!(core.get_source_header_xml(), "<header/>");
        assert!(core.has_source_xmls());

        // 갱신
        core.source_section_xmls[0] = "<section><p/></section>".into();
        core.source_header_xml = "<header><charPr/></header>".into();
        assert_eq!(
            core.get_source_section_xml(0),
            Some("<section><p/></section>")
        );
        assert_eq!(core.get_source_header_xml(), "<header><charPr/></header>");
    }

    #[test]
    fn multi_section_indexing_independent() {
        let mut core = DocumentCore::new_empty();
        core.source_section_xmls.push("<a/>".into());
        core.source_section_xmls.push("<b/>".into());
        core.source_section_xmls.push("<c/>".into());
        assert_eq!(core.source_section_xml_count(), 3);
        assert_eq!(core.get_source_section_xml(0), Some("<a/>"));
        assert_eq!(core.get_source_section_xml(1), Some("<b/>"));
        assert_eq!(core.get_source_section_xml(2), Some("<c/>"));
        assert!(core.get_source_section_xml(3).is_none());
    }
}
