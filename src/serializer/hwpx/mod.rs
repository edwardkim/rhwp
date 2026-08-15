//! HWPX(ZIP+XML) 직렬화 모듈 — `parser::hwpx`의 역방향.
//!
//! ## 단계 (#182)
//! - Stage 0 (완료): 기반 공사 — SerializeContext, IrDiff 하네스, canonical_defaults
//! - Stage 1: header.xml IR 기반 동적 생성
//! - Stage 2: section.xml 동적화 + charPrIDRef 매핑
//! - Stage 3: 표(Table)
//! - Stage 4: 그림(Picture) + BinData
//! - Stage 5: 도형·필드 + 대형 실문서 스모크

pub mod canonical_defaults;
pub mod content;
pub mod context;
pub mod field;
pub mod fixtures;
pub mod form;
pub mod header;
pub mod master_page;
pub mod package_check;
pub mod picture;
pub mod roundtrip;
pub mod section;
pub mod shape;
pub mod static_assets;
pub mod table;
pub mod utils;
pub mod writer;

use std::collections::HashSet;
use std::fmt::Write as _;

use crate::model::bin_data::MAX_BIN_DATA_BYTES;
use crate::model::document::{Document, HWP5_ORIGIN_HWPX_MARKER_PATH};
use crate::serializer::content_loss::{ContentLoss, ContentLossReason, SerializedDocument};

use super::SerializeError;
use content::BinDataEntry as ContentBinDataEntry;
use context::SerializeContext;
use writer::HwpxZipWriter;

/// Document IR을 HWPX(ZIP+XML) 바이트로 직렬화한다.
///
/// Stage 0 이후: 빈 문서 특수 분기를 제거하고 **항상 동적 경로**를 탄다.
/// `SerializeContext`가 1-pass 스캔으로 ID 풀을 구성하고, 각 writer가 동일 컨텍스트를
/// 참조한다. 직렬화 종료 시 `assert_all_refs_resolved()`가 미등록 참조를 단언한다.
pub fn serialize_hwpx(doc: &Document) -> Result<Vec<u8>, SerializeError> {
    let serialized = serialize_hwpx_with_report(doc)?;
    serialized.content_loss().write_warnings_to_stderr();
    Ok(serialized.into_bytes())
}

/// HWPX 직렬화 바이트와 바로 그 산출물의 내용 손실을 함께 반환한다 (#4430).
pub fn serialize_hwpx_with_report(doc: &Document) -> Result<SerializedDocument, SerializeError> {
    use static_assets::*;

    // 1-pass: ID 풀 구성
    let mut ctx = SerializeContext::collect_from_document(doc);

    let mut z = HwpxZipWriter::new();

    // 1. mimetype (반드시 최초 엔트리, STORED, extra field 없음)
    z.write_stored("mimetype", b"application/hwp+zip")?;

    // 2. version.xml — 원본 보존 우선 (없으면 하드코딩 상수).
    //    하드코딩 상수는 Windows/특정 빌드 고정값이라 한컴 변환본의 실제 플랫폼
    //    버전을 덮어쓴다. 원본 보조 엔트리가 있으면 그대로 출력한다.
    z.write_deflated(
        "version.xml",
        doc.hwpx_aux_entry("version.xml")
            .unwrap_or_else(|| VERSION_XML.as_bytes()),
    )?;

    // 3. Contents/header.xml — Stage 1 동적 생성 (IR 기반)
    let header_xml = header::write_header(doc, &ctx)?;
    z.write_deflated("Contents/header.xml", &header_xml)?;

    // 4. Contents/section{N}.xml — 실제 섹션만큼, 없으면 0개
    let section_hrefs: Vec<String> = (0..doc.sections.len())
        .map(|i| format!("Contents/section{}.xml", i))
        .collect();
    for (i, sec) in doc.sections.iter().enumerate() {
        let xml = section::write_section(sec, doc, i, &mut ctx)?;
        z.write_deflated(&section_hrefs[i], &xml)?;
    }

    // 4b. Contents/masterpage{N}.xml — 바탕쪽 (전 섹션 누적 전역 인덱스).
    //     id/href 의 인덱스는 section.rs 의 idRef 인덱스와 동일 규칙(전역 누적)이라
    //     별도 공유 상태 없이 정합한다.
    let mut master_items: Vec<(String, String)> = Vec::new();
    let mut mp_global = 0usize;
    for sec in &doc.sections {
        for mp in &sec.section_def.master_pages {
            let id = format!("masterpage{}", mp_global);
            let href = format!("Contents/masterpage{}.xml", mp_global);
            let xml = master_page::render_master_page_xml(mp, &id, &mut ctx);
            z.write_deflated(&href, xml.as_bytes())?;
            master_items.push((id, href));
            mp_global += 1;
        }
    }

    // 5. Preview/PrvText.txt + Preview/PrvImage.png — 원본 보존 우선.
    //    하드코딩 상수는 빈/placeholder 미리보기라 원본 썸네일·미리보기 텍스트를
    //    잃는다. 보조 엔트리가 있으면 원본을 그대로 출력한다.
    z.write_deflated(
        "Preview/PrvText.txt",
        doc.hwpx_aux_entry("Preview/PrvText.txt")
            .unwrap_or(PRV_TEXT),
    )?;
    z.write_deflated(
        "Preview/PrvImage.png",
        doc.hwpx_aux_entry("Preview/PrvImage.png")
            .unwrap_or(PRV_IMAGE_PNG),
    )?;

    // 6. settings.xml — 원본 보존 우선.
    //    하드코딩 상수는 PrintInfo(확대/인쇄 설정) 등을 빠뜨린다.
    z.write_deflated(
        "settings.xml",
        doc.hwpx_aux_entry("settings.xml")
            .unwrap_or_else(|| SETTINGS_XML.as_bytes()),
    )?;

    // 7. META-INF/container.rdf — header + every section part.
    // Hancom uses this RDF graph alongside content.hpf; a stale one-section
    // RDF makes multi-section documents fail to open even when the ZIP and
    // content.hpf contain every section.
    let container_rdf = write_container_rdf(&section_hrefs);
    z.write_deflated("META-INF/container.rdf", container_rdf.as_bytes())?;

    // 8. BinData ZIP 엔트리 (Stage 4)
    //    `ctx.bin_data_map` 의 엔트리 순서대로 실제 바이너리를 ZIP에 추가.
    //    3-way 단언(binaryItemIDRef ↔ manifest ↔ ZIP entry) 의 1차 출력 지점.
    let bin_entries = ctx.bin_data_entries();
    let mut zip_bin_entries: HashSet<String> = HashSet::new();
    for entry in &bin_entries {
        // 외부 참조(isEmbeded=0)는 ZIP 엔트리가 없다 — manifest 항목만 방출 (#1891).
        if !entry.is_embedded {
            continue;
        }
        let data = doc
            .bin_data_content
            .iter()
            .find(|b| b.id == entry.bin_data_id)
            .ok_or_else(|| {
                SerializeError::XmlError(format!(
                    "BinDataContent 누락: bin_data_id={}",
                    entry.bin_data_id
                ))
            })?;
        // [Issue #3547] OLE Storage 복원: HWPX 파서(`normalize_ole_bytes`)는 내부 OLE 의
        // 선두 4-byte LE size prefix 를 제거한다. 재부착 없이 쓰면 한컴이 CFB 매직
        // (D0CF11E0)을 OLE 개체 크기(~3.75GB)로 오인하여 "메모리 부족" 오류가 발생한다.
        // HWP5 저장기(`cfb_writer`, #954)와 동형의 복원 — prefix 가 없던(비 CFB) 입력은
        // 매직 검사에서 걸러져 영향이 없다.
        //
        // [#2550] 압축 해제 상한. HWP5 저장과 달리 ZIP 엔트리는 해제된 바이트의
        // 재압축이 필수라 원본 통과 fallback 이 없다. 상한 초과는 [#1917] 이 정한
        // 로드 실패 의미(빈 엔트리 placeholder + pic 컨트롤 보존)를 그대로 따른다 —
        // 여기서 오류로 중단하면 폭탄 문서 하나가 저장 자체를 막는다.
        let bytes = match data.data.load_limited(MAX_BIN_DATA_BYTES) {
            Some(bytes) => bytes,
            None => {
                ctx.content_loss.record(ContentLoss::binary_content_emptied(
                    entry.bin_data_id,
                    entry.href.clone(),
                    ContentLossReason::ResourceReadFailedOrLimitExceeded,
                ));
                Vec::new()
            }
        };
        const CFB_MAGIC: [u8; 8] = [0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1];
        let payload: Vec<u8> = if data.extension.eq_ignore_ascii_case("ole")
            && bytes.len() >= 8
            && bytes[..8] == CFB_MAGIC
        {
            let mut v = Vec::with_capacity(bytes.len() + 4);
            v.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            v.extend_from_slice(&bytes);
            v
        } else {
            bytes
        };
        z.write_deflated(&entry.href, &payload)?;
        zip_bin_entries.insert(entry.href.clone());
    }

    // 8-1. [#3546] OOXML 차트 파트 — Chart/chartN.xml 바이트 원형 방출.
    //      원본 content.hpf 가 Chart 파트를 나열하지 않으므로 manifest·3-way
    //      단언 대상 밖이며(collect_from_document 에서 분리), BinData 로 옮기면
    //      한컴이 차트 XML 을 OLE 복합문서로 해석한다.
    // chart_entries 순회 중에는 ctx를 불변 대여하므로 손실 이벤트를 잠시 분리한다.
    let mut chart_losses = Vec::new();
    for entry in &ctx.chart_entries {
        let data = doc
            .bin_data_content
            .iter()
            .find(|b| b.id == entry.bin_data_id)
            .ok_or_else(|| {
                SerializeError::XmlError(format!(
                    "차트 BinDataContent 누락: bin_data_id={}",
                    entry.bin_data_id
                ))
            })?;
        // [#2550] 차트 파트도 같은 상한 — bin_data_content 를 공유하므로 폭탄 표면이다.
        // 상한 초과·로드 실패는 위 BinData 엔트리와 같은 빈 파트 placeholder 다.
        let chart_bytes = match data.data.load_limited(MAX_BIN_DATA_BYTES) {
            Some(bytes) => bytes,
            None => {
                chart_losses.push(ContentLoss::binary_content_emptied(
                    entry.bin_data_id,
                    entry.href.clone(),
                    ContentLossReason::ResourceReadFailedOrLimitExceeded,
                ));
                Vec::new()
            }
        };
        z.write_deflated(&entry.href, &chart_bytes)?;
    }
    for loss in chart_losses {
        ctx.content_loss.record(loss);
    }

    // 9. Contents/content.hpf — 항상 동적 경로 + BinData 매니페스트 엔트리
    let content_bin_entries: Vec<ContentBinDataEntry> = bin_entries
        .iter()
        .map(|e| ContentBinDataEntry {
            id: e.manifest_id.clone(),
            href: e.href.clone(),
            media_type: e.media_type.clone(),
            is_embedded: e.is_embedded,
        })
        .collect();
    let content_hpf = content::write_content_hpf(
        &section_hrefs,
        &content_bin_entries,
        &master_items,
        doc.hwpx_aux_entry("Contents/content.hpf"),
    )?;
    z.write_deflated("Contents/content.hpf", &content_hpf)?;

    // 10. META-INF/container.xml
    z.write_deflated("META-INF/container.xml", META_INF_CONTAINER_XML.as_bytes())?;

    // 11. META-INF/manifest.xml
    z.write_deflated("META-INF/manifest.xml", META_INF_MANIFEST_XML.as_bytes())?;

    // HWP5-origin HWPX marker — HWP5에서 HWPX로 export한 산출물은 HWPX 컨테이너라도
    // lineSeg 부재/pagination 시멘틱을 HWP5 원본처럼 해석해야 자기정합한다.
    if let Some(marker) = doc.hwpx_aux_entry(HWP5_ORIGIN_HWPX_MARKER_PATH) {
        z.write_deflated(HWP5_ORIGIN_HWPX_MARKER_PATH, marker)?;
    }
    // HWP3-origin 마커 — 재열람 시 hwp3_lineage 복원(왕복 레이아웃 등식).
    if let Some(marker) = doc.hwpx_aux_entry(crate::model::document::HWP3_ORIGIN_HWPX_MARKER_PATH) {
        z.write_deflated(crate::model::document::HWP3_ORIGIN_HWPX_MARKER_PATH, marker)?;
    }

    // 참조 정합성 단언 (Stage 1+)
    ctx.assert_all_refs_resolved()?;

    // 3-way BinData 단언 (Stage 4):
    //   - ctx.bin_data_map 의 manifest_id/href 집합
    //   - content.hpf opf:item (위에서 content_bin_entries 로 생성됨, 집합 동일)
    //   - ZIP entry (위에서 zip_bin_entries 로 기록됨)
    // 세 집합이 동일해야 한컴이 바인딩 오류 없이 그림을 표시함.
    assert_bin_data_3way(&bin_entries, &zip_bin_entries)?;

    let bytes = z.finish()?;
    Ok(SerializedDocument::new(bytes, ctx.content_loss))
}

/// Document IR을 한컴 ODF AES-256-CBC 비밀번호 보호 HWPX로 직렬화한다.
///
/// ZIP/문서 생성은 평문 `serialize_hwpx()`와 동일하게 수행한 뒤 공통 암호 모듈이
/// header, section, settings, preview와 BinData 엔트리만 암호화한다.
pub fn serialize_hwpx_with_password(
    doc: &Document,
    password: &[u8],
) -> Result<Vec<u8>, SerializeError> {
    let serialized = serialize_hwpx_with_report(doc)?;
    let (plain, content_loss) = serialized.into_parts();
    let bytes = encrypt_hwpx_bytes(&plain, password)?;
    content_loss.write_warnings_to_stderr();
    Ok(bytes)
}

/// 비밀번호 HWPX 바이트와 바로 그 산출물의 내용 손실을 함께 반환한다 (#4430).
pub fn serialize_hwpx_with_password_and_report(
    doc: &Document,
    password: &[u8],
) -> Result<SerializedDocument, SerializeError> {
    let serialized = serialize_hwpx_with_report(doc)?;
    let (plain, content_loss) = serialized.into_parts();
    let bytes = encrypt_hwpx_bytes(&plain, password)?;
    Ok(SerializedDocument::new(bytes, content_loss))
}

fn encrypt_hwpx_bytes(plain: &[u8], password: &[u8]) -> Result<Vec<u8>, SerializeError> {
    crate::password_crypto::encrypt_hwpx_package(plain, password)
        .map_err(|error| SerializeError::CryptoError(error.to_string()))
}

fn write_container_rdf(section_hrefs: &[String]) -> String {
    const PKG_NS: &str = "http://www.hancom.co.kr/hwpml/2016/meta/pkg#";

    let mut out = String::new();
    out.push_str(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes" ?>"#);
    out.push_str(r#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">"#);
    out.push_str(r#"<rdf:Description rdf:about="">"#);
    let _ = write!(
        out,
        r#"<ns0:hasPart xmlns:ns0="{PKG_NS}" rdf:resource="Contents/header.xml"/>"#
    );
    out.push_str(r#"</rdf:Description>"#);
    out.push_str(r#"<rdf:Description rdf:about="Contents/header.xml">"#);
    let _ = write!(out, r#"<rdf:type rdf:resource="{PKG_NS}HeaderFile"/>"#);
    out.push_str(r#"</rdf:Description>"#);

    for href in section_hrefs {
        out.push_str(r#"<rdf:Description rdf:about="">"#);
        let _ = write!(
            out,
            r#"<ns0:hasPart xmlns:ns0="{PKG_NS}" rdf:resource="{href}"/>"#
        );
        out.push_str(r#"</rdf:Description>"#);
        let _ = write!(out, r#"<rdf:Description rdf:about="{href}">"#);
        let _ = write!(out, r#"<rdf:type rdf:resource="{PKG_NS}SectionFile"/>"#);
        out.push_str(r#"</rdf:Description>"#);
    }

    out.push_str(r#"<rdf:Description rdf:about="">"#);
    let _ = write!(out, r#"<rdf:type rdf:resource="{PKG_NS}Document"/>"#);
    out.push_str(r#"</rdf:Description>"#);
    out.push_str(r#"</rdf:RDF>"#);
    out
}

/// 3-way BinData 동기화 단언: `ctx.bin_data_entries()`, content.hpf manifest,
/// ZIP entry 의 href 집합이 모두 일치하는지 확인.
/// 외부 참조(isEmbeded=0) 항목은 ZIP 엔트리가 없는 것이 정상이므로 제외한다 (#1891).
fn assert_bin_data_3way(
    bin_entries: &[context::BinDataEntry],
    zip_entries: &HashSet<String>,
) -> Result<(), SerializeError> {
    let ctx_hrefs: HashSet<String> = bin_entries
        .iter()
        .filter(|e| e.is_embedded)
        .map(|e| e.href.clone())
        .collect();
    if ctx_hrefs != *zip_entries {
        let missing_zip: Vec<_> = ctx_hrefs.difference(zip_entries).cloned().collect();
        let orphan_zip: Vec<_> = zip_entries.difference(&ctx_hrefs).cloned().collect();
        return Err(SerializeError::XmlError(format!(
            "3-way BinData 불일치: ctx(href) vs zip_entries — ctx에만 있음: {:?}, zip에만 있음: {:?}",
            missing_zip, orphan_zip
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    use crate::parser::hwpx::parse_hwpx;

    #[test]
    fn serialize_empty_doc_parses_back() {
        let doc = Document::default();
        let bytes = serialize_hwpx(&doc).expect("serialize empty");
        let parsed = parse_hwpx(&bytes).expect("parse back");
        assert_eq!(parsed.sections.len(), 0);
        assert!(parsed.bin_data_content.is_empty());
    }

    /// 보조 엔트리(version.xml/settings.xml/Preview/*)가 보존된 경우,
    /// 직렬화기는 하드코딩 상수가 아니라 원본 바이트를 그대로 출력해야 한다.
    /// (한컴 변환본 라운드트립 무손실 보장)
    #[test]
    fn aux_entries_passthrough_verbatim() {
        let mut doc = Document::default();
        let custom: &[(&str, &[u8])] = &[
            ("version.xml", b"<custom-version platform=\"Mac\"/>"),
            ("settings.xml", b"<custom-settings printInfo=\"on\"/>"),
            ("Preview/PrvText.txt", b"preview body text"),
            ("Preview/PrvImage.png", &[0x89, b'P', b'N', b'G', 1, 2, 3]),
        ];
        doc.hwpx_aux_entries = custom
            .iter()
            .map(|(p, d)| (p.to_string(), d.to_vec()))
            .collect();

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        for (path, expected) in custom {
            let mut entry = archive
                .by_name(path)
                .unwrap_or_else(|_| panic!("missing entry {path}"));
            let mut got = Vec::new();
            std::io::Read::read_to_end(&mut entry, &mut got).expect("read");
            assert_eq!(&got, expected, "{path} not passed through verbatim");
        }
    }

    /// 보조 엔트리가 없으면(HWP5 등) 하드코딩 상수로 폴백한다.
    #[test]
    fn aux_entries_fallback_to_constants() {
        let doc = Document::default();
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut entry = archive.by_name("version.xml").expect("version.xml");
        let mut got = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut got).expect("read");
        assert_eq!(got, static_assets::VERSION_XML.as_bytes());
    }

    /// `version.xml` 의 `xmlVersion` 은 조판을 가른다 — 상수를 되돌리지 못하게 못 박는다.
    ///
    /// 한글은 선언된 OWPML 판본에 따라 옛 조판 규칙을 적용한다. `1.2` 를 선언하면 같은
    /// 내용이 한 쪽 더 늘어난다(01513·01888·01892·01908 실측: 원본 1쪽 / `1.2` 2쪽 /
    /// `1.5` 1쪽). 쪽수가 이미 맞던 문서 12건에는 영향이 없었다(회귀 0).
    ///
    /// 같은 파일의 `appVersion`·`micro`·`buildNumber` 는 바꿔도 쪽수가 그대로다 —
    /// 조판을 가르는 것은 이 속성 하나뿐이다.
    #[test]
    fn version_xml_declares_the_owpml_revision_that_hangul_lays_out_with() {
        assert!(
            static_assets::VERSION_XML.contains(r#"xmlVersion="1.5""#),
            "xmlVersion 은 1.5 여야 한다: {}",
            static_assets::VERSION_XML
        );
    }

    #[test]
    fn serialize_with_one_section_parses_back() {
        let mut doc = Document::default();
        doc.sections
            .push(crate::model::document::Section::default());
        let bytes = serialize_hwpx(&doc).expect("serialize one-section");
        let parsed = parse_hwpx(&bytes).expect("parse back");
        assert_eq!(parsed.sections.len(), 1);
    }

    #[test]
    fn master_pages_are_serialized_as_package_parts() {
        use crate::model::document::Section;
        use crate::model::header_footer::{HeaderFooterApply, MasterPage};
        use crate::model::paragraph::Paragraph;
        use crate::serializer::hwpx::package_check::check_package;

        let mut doc = Document::default();
        let mut section0 = Section::default();
        let mut section1 = Section::default();

        let mut first_master_para = Paragraph::default();
        first_master_para.text = "first master".to_string();
        section0.section_def.master_pages.push(MasterPage {
            apply_to: HeaderFooterApply::Both,
            text_width: 10_000,
            text_height: 10_000,
            text_ref: 1,
            paragraphs: vec![first_master_para],
            ..Default::default()
        });

        let mut second_master_para = Paragraph::default();
        second_master_para.text = "second master".to_string();
        section1.section_def.master_pages.push(MasterPage {
            apply_to: HeaderFooterApply::Odd,
            text_width: 12_000,
            text_height: 8_000,
            text_ref: 1,
            paragraphs: vec![second_master_para],
            ..Default::default()
        });

        doc.sections.push(section0);
        doc.sections.push(section1);

        let bytes = serialize_hwpx(&doc).expect("serialize master pages");
        let report = check_package(&bytes, &doc);
        assert!(report.is_ok(), "problems: {}", report.summary());

        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        for name in ["Contents/masterpage0.xml", "Contents/masterpage1.xml"] {
            archive
                .by_name(name)
                .unwrap_or_else(|_| panic!("missing {name}"));
        }

        let mut content_hpf = String::new();
        archive
            .by_name("Contents/content.hpf")
            .expect("content.hpf")
            .read_to_string(&mut content_hpf)
            .expect("read content.hpf");
        assert!(content_hpf.contains(r#"id="masterpage0""#));
        assert!(content_hpf.contains(r#"href="Contents/masterpage1.xml""#));

        let mut section1_xml = String::new();
        archive
            .by_name("Contents/section1.xml")
            .expect("section1")
            .read_to_string(&mut section1_xml)
            .expect("read section1");
        assert!(section1_xml.contains(r#"masterPageCnt="1""#));
        assert!(section1_xml.contains(r#"idRef="masterpage1""#));

        drop(archive);
        let parsed = parse_hwpx(&bytes).expect("parse back");
        assert_eq!(parsed.sections[0].section_def.master_pages.len(), 1);
        assert_eq!(parsed.sections[1].section_def.master_pages.len(), 1);
        assert_eq!(
            parsed.sections[1].section_def.master_pages[0].apply_to,
            HeaderFooterApply::Odd
        );
    }

    #[test]
    fn container_rdf_lists_every_section() {
        let mut doc = Document::default();
        for _ in 0..3 {
            doc.sections
                .push(crate::model::document::Section::default());
        }

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut container_rdf = String::new();
        archive
            .by_name("META-INF/container.rdf")
            .expect("container.rdf")
            .read_to_string(&mut container_rdf)
            .expect("read container.rdf");

        assert!(container_rdf.contains(r#"rdf:resource="Contents/header.xml""#));
        for i in 0..3 {
            let href = format!("Contents/section{i}.xml");
            assert!(
                container_rdf.contains(&format!(r#"rdf:resource="{href}""#)),
                "missing hasPart for {href}: {container_rdf}"
            );
            assert!(
                container_rdf.contains(&format!(r#"rdf:about="{href}""#)),
                "missing type description for {href}: {container_rdf}"
            );
        }
    }

    #[test]
    fn header_footer_ids_are_preserved() {
        use crate::model::control::Control;
        use crate::model::document::Section;
        use crate::model::header_footer::{Footer, Header, HeaderFooterApply};
        use crate::model::paragraph::Paragraph;

        let mut doc = Document::default();
        let mut section = Section::default();
        let mut para = Paragraph::default();

        para.controls.push(Control::Footer(Box::new(Footer {
            raw_ctrl_extra: 2u32.to_le_bytes().to_vec(),
            apply_to: HeaderFooterApply::Even,
            ..Default::default()
        })));
        para.controls.push(Control::Header(Box::new(Header {
            raw_ctrl_extra: 1u32.to_le_bytes().to_vec(),
            apply_to: HeaderFooterApply::Odd,
            ..Default::default()
        })));
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize header/footer ids");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut section0_xml = String::new();
        archive
            .by_name("Contents/section0.xml")
            .expect("section0")
            .read_to_string(&mut section0_xml)
            .expect("read section0");

        assert!(
            section0_xml.contains(r#"<hp:footer id="2" applyPageType="EVEN">"#),
            "footer id not preserved: {section0_xml}"
        );
        assert!(
            section0_xml.contains(r#"<hp:header id="1" applyPageType="ODD">"#),
            "header id not preserved: {section0_xml}"
        );
    }

    #[test]
    fn serialize_text_paragraph_roundtrip() {
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "안녕 Hello 123".to_string();
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize text");
        // 직렬화된 XML에 텍스트가 그대로 들어갔는지 ZIP에서 추출해 확인
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        assert!(
            xml.contains("<hp:t>안녕 Hello 123</hp:t>"),
            "text not injected into section0.xml"
        );

        // 라운드트립도 확인
        drop(sec0);
        let parsed = parse_hwpx(&bytes).expect("parse back");
        assert_eq!(parsed.sections.len(), 1);
        let p0 = &parsed.sections[0].paragraphs[0];
        assert!(
            p0.text.contains("안녕 Hello 123"),
            "text roundtrip failed: {:?}",
            p0.text
        );
    }

    /// #4387: 불균등 다단(단별 개별 너비/간격, `sameSz="false"`)이 HWPX 왕복에서
    /// 균등 폭으로 저하되지 않아야 한다. `<hp:colSz>` 미방출/미파싱 시
    /// `ColumnDef.widths`/`gaps` 가 비어 렌더러(page_layout.rs
    /// calculate_column_areas) 가 same_width=false 조건을 만족 못 하고 균등
    /// 분할 분기로 폴백한다.
    #[test]
    fn issue4387_unequal_column_widths_roundtrip() {
        use crate::model::control::Control;
        use crate::model::document::Section;
        use crate::model::page::{ColumnDef, ColumnType};
        use crate::model::paragraph::Paragraph;

        let mut cd = ColumnDef::default();
        cd.column_type = ColumnType::Normal;
        cd.column_count = 2;
        cd.same_width = false;
        cd.widths = vec![4000, 6000];
        cd.gaps = vec![500, 0];

        let mut para = Paragraph::default();
        para.text = "x".to_string();
        para.controls.push(Control::ColumnDef(cd));

        let mut section = Section::default();
        section.paragraphs.push(para);
        let mut doc = Document::default();
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize unequal columns");

        // 직렬화된 XML에 단별 colSz 가 실제로 들어갔는지 ZIP에서 추출해 확인.
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        sec0.read_to_string(&mut xml).expect("read");
        assert!(
            xml.contains(r#"sameSz="0""#),
            "sameSz=false 가 방출돼야 함: {}",
            &xml[..900.min(xml.len())]
        );
        assert!(
            xml.contains(r#"<hp:colSz width="4000" gap="500"/>"#)
                && xml.contains(r#"<hp:colSz width="6000" gap="0"/>"#),
            "colSz 요소가 방출돼야 함(#4387): {}",
            &xml[..900.min(xml.len())]
        );
        drop(sec0);

        // 왕복 확인: 다시 파싱했을 때 개별 너비/간격이 그대로 복원돼야 한다.
        let parsed = parse_hwpx(&bytes).expect("parse back");
        let p0 = &parsed.sections[0].paragraphs[0];
        let cd2 = p0
            .controls
            .iter()
            .find_map(|c| match c {
                Control::ColumnDef(cd) => Some(cd),
                _ => None,
            })
            .expect("ColumnDef control roundtrips");
        assert!(!cd2.same_width);
        assert_eq!(
            cd2.widths,
            vec![4000, 6000],
            "단별 너비가 왕복에서 보존돼야 함(#4387)"
        );
        assert_eq!(
            cd2.gaps,
            vec![500, 0],
            "단별 간격이 왕복에서 보존돼야 함(#4387)"
        );
        assert!(
            !cd2.proportional_widths,
            "HWPX colSz 는 절대 HWPUNIT — 비례값 플래그가 켜지면 안 됨"
        );
    }

    #[test]
    fn tab_and_linebreak_emitted_inline() {
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "A\tB\nC".to_string();
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        // Stage 2.3 (ref_mixed 기반): 혼합 콘텐츠 + tab 속성 포함.
        // [#4403] tab_extended 데이터 없는 암묵적 기본 탭은 "데이터 없음" 마커
        // (width=0)로 방출한다 — 고정 상수(옛 4000)를 emit 하면 재적재 시 렌더러가
        // 그 폭을 실제 계산값으로 신뢰해 문단의 진짜 TabDef 를 무시한다.
        assert!(
            xml.contains(
                r#"<hp:t>A<hp:tab width="0" leader="0" type="1"/>B<hp:lineBreak/>C</hp:t>"#
            ),
            "mixed content not rendered: {}",
            xml
        );
    }

    #[test]
    fn equation_control_roundtrip_preserves_script() {
        use crate::model::control::{Control, Equation};
        use crate::model::shape::{
            CommonObjAttr, HorzAlign, HorzRelTo, TextWrap, VertAlign, VertRelTo,
        };
        use crate::model::Padding;

        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "AB".to_string();
        para.char_offsets = vec![0, 9];
        para.char_count = 11;
        para.controls.push(Control::Equation(Box::new(Equation {
            common: CommonObjAttr {
                instance_id: 7,
                z_order: 3,
                width: 2400,
                height: 1200,
                vertical_offset: 80,
                horizontal_offset: 160,
                margin: Padding {
                    left: 10,
                    right: 20,
                    top: 30,
                    bottom: 40,
                },
                treat_as_char: true,
                text_wrap: TextWrap::TopAndBottom,
                vert_rel_to: VertRelTo::Para,
                horz_rel_to: HorzRelTo::Para,
                vert_align: VertAlign::Bottom,
                horz_align: HorzAlign::Center,
                ..Default::default()
            },
            attr: 0,
            script: "x < y & z".to_string(),
            font_size: 1000,
            color: 0x000000FF,
            baseline: 120,
            unknown: 0,
            eqedit: 0,
            font_name: "HYhwpEQ".to_string(),
            version_info: "Equation Version 60".to_string(),
            raw_ctrl_data: Vec::new(),
        })));
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize equation");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        assert!(
            xml.contains("<hp:equation "),
            "equation XML missing: {}",
            xml
        );
        assert!(
            xml.contains("<hp:script>x &lt; y &amp; z</hp:script>"),
            "script XML missing: {}",
            xml
        );
        drop(sec0);

        let parsed = parse_hwpx(&bytes).expect("parse back");
        let parsed_para = &parsed.sections[0].paragraphs[0];
        assert_eq!(parsed_para.text, "AB");
        let parsed_eq = parsed_para.controls.iter().find_map(|ctrl| match ctrl {
            Control::Equation(eq) => Some(eq),
            _ => None,
        });
        match parsed_eq {
            Some(eq) => {
                assert_eq!(eq.script, "x < y & z");
                assert_eq!(eq.font_size, 1000);
                assert_eq!(eq.color, 0x000000FF);
                assert_eq!(eq.baseline, 120);
                assert_eq!(eq.font_name, "HYhwpEQ");
                assert_eq!(eq.version_info, "Equation Version 60");
                assert!(eq.common.treat_as_char);
                assert_eq!(eq.common.width, 2400);
                assert_eq!(eq.common.height, 1200);
                assert_eq!(eq.common.instance_id, 7);
                assert_eq!(eq.common.z_order, 3);
                assert_eq!(eq.common.vertical_offset, 80);
                assert_eq!(eq.common.horizontal_offset, 160);
                assert_eq!(eq.common.margin.left, 10);
                assert_eq!(eq.common.margin.right, 20);
                assert_eq!(eq.common.margin.top, 30);
                assert_eq!(eq.common.margin.bottom, 40);
                assert_eq!(eq.common.text_wrap, TextWrap::TopAndBottom);
                assert_eq!(eq.common.vert_rel_to, VertRelTo::Para);
                assert_eq!(eq.common.horz_rel_to, HorzRelTo::Para);
                assert_eq!(eq.common.vert_align, VertAlign::Bottom);
                assert_eq!(eq.common.horz_align, HorzAlign::Center);
            }
            None => panic!("expected equation control, got {:?}", parsed_para.controls),
        }
    }

    #[test]
    fn task1655_equation_flow_with_text_roundtrips() {
        use crate::model::control::{Control, Equation};
        use crate::model::shape::CommonObjAttr;

        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "A".to_string();
        para.char_offsets = vec![0];
        para.char_count = 9;
        para.controls.push(Control::Equation(Box::new(Equation {
            common: CommonObjAttr {
                width: 1600,
                height: 900,
                treat_as_char: true,
                flow_with_text: false,
                ..Default::default()
            },
            script: "a+b".to_string(),
            font_size: 1000,
            ..Default::default()
        })));
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize equation");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        assert!(
            xml.contains(r#"flowWithText="0""#),
            "수식 flowWithText=false 는 0으로 방출되어야 한다: {}",
            xml
        );
        drop(sec0);

        let parsed = parse_hwpx(&bytes).expect("parse back");
        let parsed_eq = parsed.sections[0].paragraphs[0]
            .controls
            .iter()
            .find_map(|ctrl| match ctrl {
                Control::Equation(eq) => Some(eq),
                _ => None,
            })
            .expect("equation control");
        assert!(
            !parsed_eq.common.flow_with_text,
            "수식 flowWithText=false 는 재파싱 뒤에도 보존되어야 한다"
        );
    }

    #[test]
    fn equation_control_between_text_runs_roundtrips_position() {
        use crate::model::control::{Control, Equation};
        use crate::model::page::ColumnDef;
        use crate::model::shape::CommonObjAttr;
        use crate::model::table::Table;

        let mut doc = Document::default();
        // Table::default() 의 border_fill_id(0) 가 검증을 통과하도록 등록
        doc.doc_info
            .border_fills
            .push(crate::model::style::BorderFill::default());
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "ACB".to_string();
        para.char_offsets = vec![0, 9, 18];
        para.char_count = 20;
        para.controls.push(Control::ColumnDef(ColumnDef::default()));
        para.controls.push(Control::Table(Box::default()));
        para.controls.push(Control::Equation(Box::new(Equation {
            common: CommonObjAttr {
                width: 1000,
                height: 1000,
                treat_as_char: true,
                ..Default::default()
            },
            script: "a+b".to_string(),
            font_size: 1000,
            ..Default::default()
        })));
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize equation");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");

        let a_pos = xml.find("<hp:t>A</hp:t>").expect("A text run");
        let c_pos = xml.find("<hp:t>C</hp:t>").expect("C text run");
        let eq_pos = xml.find("<hp:equation ").expect("equation");
        let b_pos = xml.find("<hp:t>B</hp:t>").expect("B text run");
        assert!(
            a_pos < c_pos && c_pos < eq_pos && eq_pos < b_pos,
            "equation must stay after non-equation inline slots: {}",
            xml
        );
    }

    #[test]
    fn task1587_ruby_control_roundtrips() {
        // Ruby(덧말) 컨트롤은 is_hwpx_inline_slot 에 등록돼 슬롯으로 인식되나
        // render_control_slot 에 방출 arm 이 없어 저장 시 드롭된다(controls=[]).
        // 수정 전: reparse 후 Ruby 소실 → RED. 수정 후: 보존 → GREEN.
        use crate::model::control::{Control, Ruby};

        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "ab".to_string();
        para.char_offsets = vec![0, 9];
        para.char_count = 11; // (11-1-2)/8 = 1 슬롯
        para.controls.push(Control::Ruby(Ruby {
            main_text: "기준글".to_string(),
            ruby_text: "덧말".to_string(),
            pos_type: 1, // BOTTOM
            align: 2,    // CENTER
            sz_ratio: 80,
            option: 3,
            style_id_ref: 5,
        }));
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize ruby");
        let doc2 = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse");
        let rubies: Vec<_> = doc2.sections[0].paragraphs[0]
            .controls
            .iter()
            .filter_map(|c| match c {
                Control::Ruby(r) => Some(r),
                _ => None,
            })
            .collect();
        assert_eq!(
            rubies.len(),
            1,
            "Ruby 컨트롤이 roundtrip 후 보존돼야 한다 (현재 드롭): {:?}",
            doc2.sections[0].paragraphs[0].controls
        );
        let r = rubies[0];
        // 전 필드 무손실 (#1587 — mainText/posType/align/szRatio/option/styleIDRef)
        assert_eq!(r.main_text, "기준글", "mainText 보존");
        assert_eq!(r.ruby_text, "덧말", "subText(덧말) 보존");
        assert_eq!(r.pos_type, 1, "posType(BOTTOM) 보존");
        assert_eq!(r.align, 2, "align(CENTER) 보존");
        assert_eq!(r.sz_ratio, 80, "szRatio 보존");
        assert_eq!(r.option, 3, "option 보존");
        assert_eq!(r.style_id_ref, 5, "styleIDRef 보존");
    }

    #[test]
    fn issue4388_hyperlink_control_drop_is_not_silent() {
        // [#4388] `Control::Hyperlink` 는 HWP3 파서만 생성하는 legacy IR 이며 HWPX 는
        // 대응 표현이 없다(HWPX 는 하이퍼링크를 Field(HYPERLINK) 로 표현). 종전엔
        // `render_control_slot` 의 catch-all `_ => {}` 로 떨어져 **경고도 없이** 사라졌다.
        // 이 테스트는 "드롭 자체"만 확인한다 — 드롭이 실제로 조용하지 않은지(stderr
        // 경고)는 이 코드베이스의 기존 관례상 단위 테스트로 캡처하지 않는다(Table/
        // Picture/Field 직렬화 실패 eprintln! 도 동일). `warn_if_unrepresentable_in_hwpx`
        // (section.rs)가 실제 경고를 담당한다.
        //
        // [#4388 후속] `--verify` 가 이 손실을 검출해야 한다는 조건은 **되돌렸다** —
        // `is_hwpx_inline_slot` 에 Hyperlink 를 등록했더니 그 목록을 공유하는
        // `roundtrip::diff_documents`(포맷 비종속, `convert`(HWP5 대상) `--verify` 도
        // 재사용)가 HWP5 직렬화기의 기존(이슈 범위 밖) Hyperlink 손실까지 새로
        // "검출"해 `tests/issue_hwp3_bookmark_native.rs::bookmarks_survive_saving_to_hwp5`
        // 가 hwp3-sample16.hwp 에서 깨졌다(실측 재현). 그 회귀 가드는
        // `roundtrip.rs::issue4388_diff_documents_hyperlink_not_compared_as_control` 로
        // 옮겼다 — "diff 가 검출해야 함"이 아니라 "diff 가 검출하면 안 됨"으로 반전.
        use crate::model::control::{Control, Hyperlink};
        use crate::model::paragraph::CharShapeRef;
        use crate::model::style::CharShape;

        let mut doc = Document::default();
        // char_shapes[0] 을 등록해 char_shape_id=0 참조가 유효하도록 한다(그렇지
        // 않으면 직렬화가 "미등록 ID 참조" 로 실패).
        doc.doc_info.char_shapes.push(CharShape::default());
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "ab".to_string();
        para.char_offsets = vec![0, 9];
        para.char_count = 11; // (11-1-2)/8 = 1 슬롯 (task1587_ruby_control_roundtrips 와 동일 관례)
        para.char_shapes = vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }];
        para.controls.push(Control::Hyperlink(Hyperlink {
            url: "http://example.com".to_string(),
            text: "example".to_string(),
        }));
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize hyperlink");
        let doc2 = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse");

        let hyperlinks_after: Vec<_> = doc2.sections[0].paragraphs[0]
            .controls
            .iter()
            .filter(|c| matches!(c, Control::Hyperlink(_)))
            .collect();
        assert!(
            hyperlinks_after.is_empty(),
            "Hyperlink 는 HWPX 로 안전하게 변환할 대응 표현이 없어 드롭되는 것이 현재 계약이다: {:?}",
            doc2.sections[0].paragraphs[0].controls
        );

        // [#4388 후속] is_hwpx_inline_slot 미등록 회귀 가드 — 등록하면 mismatch-분기
        // (render_runs) 가 이 컨트롤을 슬롯으로 취급해 위치 축이 바뀐다.
        assert!(
            !crate::serializer::hwpx::section::is_hwpx_inline_slot(&Control::Hyperlink(
                Hyperlink::default()
            )),
            "Hyperlink 는 is_hwpx_inline_slot 에 등록되면 안 됨(#4388 후속 회귀 참조)"
        );
    }

    #[test]
    fn issue4388_unknown_control_drop_is_not_silent() {
        // [#4388] catch-all 전수 census 중 Hyperlink 와 동형인 두 번째 사례. HWP5/HWP3
        // 바이너리 파서가 인식하지 못한 ctrl_id 는 `Control::Unknown` 으로 남는데(HWPX
        // 파서는 이 변형을 절대 만들지 않는다 — grep 확인), HWP5 PARA_TEXT 는 그 자리를
        // Table/Picture 와 같은 0x000B 확장 컨트롤 문자(8유닛)로 표시한다. HWPX 는 "인식
        // 못 한 컨트롤"을 표현할 수단이 없고 `UnknownControl` 은 ctrl_id 하나만 보존해
        // 원본 바이트도 없다 — 변환 불가가 맞다. 종전엔 catch-all 로 떨어져 **경고
        // 없이** 사라졌다 — 이제 `warn_if_unrepresentable_in_hwpx` 가 경고한다(stderr
        // 캡처는 Hyperlink 테스트와 동일한 이유로 이 테스트에서 확인하지 않는다).
        //
        // [#4388 후속] Hyperlink 와 동일하게 --verify 검출 요구는 되돌렸다 — 이유와
        // 회귀 가드는 `roundtrip.rs::issue4388_diff_documents_unknown_not_compared_as_control`.
        use crate::model::control::{Control, UnknownControl};
        use crate::model::paragraph::CharShapeRef;
        use crate::model::style::CharShape;

        let mut doc = Document::default();
        doc.doc_info.char_shapes.push(CharShape::default());
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        // "a" + 오브젝트 마커(U+FFFC) + "b" — Table/Picture 와 동일한 8유닛 슬롯 관례.
        para.text = "a\u{fffc}b".to_string();
        para.char_offsets = vec![0, 1, 9];
        para.char_count = 11; // 1(a) + 8(obj) + 1(b) + 1(종단 관례) = 11
        para.char_shapes = vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }];
        para.controls.push(Control::Unknown(UnknownControl {
            ctrl_id: 0x1234_5678,
        }));
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize unknown");
        let doc2 = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse");

        let unknowns_after: Vec<_> = doc2.sections[0].paragraphs[0]
            .controls
            .iter()
            .filter(|c| matches!(c, Control::Unknown(_)))
            .collect();
        assert!(
            unknowns_after.is_empty(),
            "Unknown 컨트롤은 HWPX 로 변환할 대응 표현·원본 바이트가 없어 드롭되는 것이 \
             현재 계약이다: {:?}",
            doc2.sections[0].paragraphs[0].controls
        );

        assert!(
            !crate::serializer::hwpx::section::is_hwpx_inline_slot(&Control::Unknown(
                UnknownControl::default()
            )),
            "Unknown 은 is_hwpx_inline_slot 에 등록되면 안 됨(#4388 후속 회귀 참조)"
        );
    }

    #[test]
    fn task1591_bookmark_not_hoisted_before_slot() {
        // [#1591] 북마크가 슬롯 컨트롤(표 등) 뒤에 있을 때, 직렬화기가 북마크를 문단
        // 시작으로 hoisting 하면 컨트롤 순서가 뒤바뀐다. [#1591 v2] 슬롯 있는 문단의
        // 북마크 in-order 방출로 수정 — GREEN 전환 (1라운드 RED, 순서 보존 가드).
        use crate::model::control::{Bookmark, Control};
        use crate::model::style::BorderFill;
        use crate::model::table::Table;

        let mut doc = Document::default();
        doc.doc_info.border_fills.push(BorderFill::default());
        let mut section = crate::model::document::Section::default();
        section
            .paragraphs
            .push(crate::model::paragraph::Paragraph::default()); // para0 더미
        let mut p = crate::model::paragraph::Paragraph::default();
        p.text = "AB".to_string();
        p.char_offsets = vec![0, 9]; // A@0, [표 슬롯 8], B@9
        p.char_count = 11;
        p.controls.push(Control::Table(Box::<Table>::default()));
        p.controls.push(Control::Bookmark(Bookmark {
            name: "bm".to_string(),
        }));
        section.paragraphs.push(p);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let doc2 = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse");
        let ctrls: Vec<&str> = doc2.sections[0].paragraphs[1]
            .controls
            .iter()
            .map(|c| match c {
                Control::Table(_) => "tbl",
                Control::Bookmark(_) => "bm",
                _ => "?",
            })
            .collect();
        assert_eq!(
            ctrls,
            vec!["tbl", "bm"],
            "북마크가 표 뒤 위치를 보존해야 한다 (hoisting 시 [bm,tbl] 로 뒤바뀜)"
        );
    }

    #[test]
    fn task1591v2_first_para_hidden_slot_char_shape_position() {
        // [#1591 v2, Class C1 — 36384689 동형] 첫 문단의 hidden 슬롯(secPr/템플릿 흡수
        // colPr)이 cc 축 8유닛을 점유하는 문서: 종전엔 slot_count(4) != slots.len() 로
        // mismatch 폴백이 후위 슬롯(pageNum)을 char-offset 없이 방출해 char_shape 경계가
        // (24,·)→(32,·) +8 시프트. hidden 정합으로 메인 경로에 진입해 경계 24 가 보존된다.
        use crate::model::control::{Bookmark, Control, PageNumberPos};
        use crate::model::document::SectionDef;
        use crate::model::page::ColumnDef;
        use crate::model::paragraph::CharShapeRef;
        use crate::model::style::{BorderFill, CharShape};
        use crate::model::table::Table;

        let mut doc = Document::default();
        doc.doc_info.border_fills.push(BorderFill::default());
        doc.doc_info.char_shapes.push(CharShape::default()); // id 0
        doc.doc_info.char_shapes.push(CharShape::default()); // id 1
        let mut section = crate::model::document::Section::default();
        let mut p = crate::model::paragraph::Paragraph::default();
        p.text = String::new();
        // [secd 0..8][cold 8..16][tbl 16..24][pageNum 24..32][bokm 32..40] + 끝 1 = 41
        //
        // [#4677] 종전 값은 33 이었다 — 책갈피를 zero-width 로 보던 시절의 예산이다. 한컴
        // 원본 HWP5 는 책갈피도 8 유닛 제어문자로 본문에 싣고(`16 00 6d 6b 6f 62 … 16 00`),
        // rhwp 도 이제 두 축(HWP5 제어문자·HWPX 슬롯) 모두에서 자리를 잡는다. 이 테스트가
        // 지키는 축(hidden 슬롯 정합 → char_shape 경계 24 보존)은 그대로다.
        p.char_count = 41;
        p.char_shapes = vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            },
            CharShapeRef {
                start_pos: 24,
                char_shape_id: 1,
            },
        ];
        p.controls
            .push(Control::SectionDef(Box::<SectionDef>::default()));
        p.controls.push(Control::ColumnDef(ColumnDef::default()));
        p.controls.push(Control::Table(Box::<Table>::default()));
        p.controls
            .push(Control::PageNumberPos(PageNumberPos::default()));
        p.controls.push(Control::Bookmark(Bookmark {
            name: "별첨 1".to_string(),
        }));
        section.paragraphs.push(p);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let doc2 = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse");
        let p2 = &doc2.sections[0].paragraphs[0];
        let positions: Vec<u32> = p2.char_shapes.iter().map(|cs| cs.start_pos).collect();
        assert_eq!(
            positions,
            vec![0, 24],
            "char_shape 경계가 24 에 보존돼야 한다 (mismatch 폴백 시 +8 → 32)"
        );
        let ctrls: Vec<&str> = p2
            .controls
            .iter()
            .map(|c| match c {
                Control::SectionDef(_) => "secd",
                Control::ColumnDef(_) => "cold",
                Control::Table(_) => "tbl",
                Control::PageNumberPos(_) => "pagenum",
                Control::Bookmark(_) => "bm",
                _ => "?",
            })
            .collect();
        assert_eq!(
            ctrls,
            vec!["secd", "cold", "tbl", "pagenum", "bm"],
            "컨트롤 순서(표·pageNum 뒤 북마크) 보존"
        );
    }

    #[test]
    fn task1593_first_para_same_para_field_end_preserved() {
        // [#1593, Class C2 — 36388711 동형] 첫 문단(hidden 슬롯 점유)의 same-para 균형
        // 필드: 종전 mismatch 폴백은 fieldBegin(슬롯)만 방출하고 닫는 fieldEnd 를
        // 소실시켜 cc −8 + 후속 char_shape 경계 −8. hidden 정합 후 메인 경로가
        // fieldEnd 를 위치대로 방출해 cc·경계가 보존된다.
        use crate::model::control::{Bookmark, Control, Field};
        use crate::model::document::SectionDef;
        use crate::model::page::ColumnDef;
        use crate::model::paragraph::{CharShapeRef, FieldRange};
        use crate::model::style::CharShape;

        let mut doc = Document::default();
        doc.doc_info.char_shapes.push(CharShape::default()); // id 0
        doc.doc_info.char_shapes.push(CharShape::default()); // id 1
        let mut section = crate::model::document::Section::default();
        let mut p = crate::model::paragraph::Paragraph::default();
        // [secd 0..8][cold 8..16][fieldBegin 16..24][bokm 24..32] A@32 [fieldEnd 33..41] B@41
        // → cc 43
        //
        // [#4677] 종전 예산은 책갈피에 자리를 주지 않아 `A@24 … B@33`, cc 35 였다. 한컴 원본은
        // 책갈피도 확장 제어문자 8 유닛을 쓰므로(`16 00 6d 6b 6f 62 … 16 00`) 자리를 하나
        // 넣고 뒤 위치를 8 씩 민다. 이 테스트가 지키는 축(same-para 균형 필드의 fieldEnd 가
        // 자기 자리에 보존되고 **뒤 char_shape 경계가 밀리지 않는 것**)은 그대로다.
        p.text = "AB".to_string();
        p.char_offsets = vec![32, 41];
        p.char_count = 43;
        p.char_shapes = vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            },
            CharShapeRef {
                start_pos: 41,
                char_shape_id: 1,
            },
        ];
        p.controls
            .push(Control::SectionDef(Box::<SectionDef>::default()));
        p.controls.push(Control::ColumnDef(ColumnDef::default()));
        p.controls.push(Control::Field(Field::default()));
        p.controls.push(Control::Bookmark(Bookmark {
            name: "bm".to_string(),
        }));
        p.field_ranges = vec![FieldRange {
            start_char_idx: 0,
            end_char_idx: 1,
            control_idx: 2,
            ..Default::default()
        }];
        section.paragraphs.push(p);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let doc2 = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse");
        let p2 = &doc2.sections[0].paragraphs[0];
        assert_eq!(
            p2.field_ranges.len(),
            1,
            "same-para 균형 필드(fieldBegin/End 1/1)가 보존돼야 한다 (종전: fieldEnd 드롭)"
        );
        assert_eq!(
            p2.char_count, 43,
            "fieldEnd 8유닛 + 책갈피 8유닛(#4677) 보존"
        );
        let positions: Vec<u32> = p2.char_shapes.iter().map(|cs| cs.start_pos).collect();
        assert_eq!(
            positions,
            vec![0, 41],
            "후속 char_shape 경계 보존 (종전 −8)"
        );
    }

    #[test]
    fn task1592_empty_paragraph_no_spurious_charshape() {
        // [#1592] run 이 없던 완전 빈 문단(char_shapes=[])에 직렬화기가 빈
        // <hp:run charPrIDRef="0"> 를 추가하면 재파싱 시 char_shapes 가 [(0,0)] 으로 생긴다.
        // 비-첫 문단으로 구성(첫 문단 템플릿 회피).
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        // para0: 텍스트 있는 일반 문단
        let mut p0 = crate::model::paragraph::Paragraph::default();
        p0.text = "본문".to_string();
        section.paragraphs.push(p0);
        // para1: 완전 빈 문단 (text="", char_shapes=[], controls=[])
        section
            .paragraphs
            .push(crate::model::paragraph::Paragraph::default());
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let doc2 = crate::parser::hwpx::parse_hwpx(&bytes).expect("parse");
        let cs = &doc2.sections[0].paragraphs[1].char_shapes;
        assert!(
            cs.is_empty(),
            "빈 문단은 char_shapes 가 비어야 한다 (spurious (0,0) 금지): {:?}",
            cs.iter()
                .map(|c| (c.start_pos, c.char_shape_id))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn equation_control_does_not_consume_unmapped_control_gap() {
        use crate::model::control::{Control, Equation};
        use crate::model::shape::CommonObjAttr;

        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "ACB".to_string();
        para.char_offsets = vec![0, 9, 18];
        para.char_count = 20;
        para.controls.push(Control::Equation(Box::new(Equation {
            common: CommonObjAttr {
                width: 1000,
                height: 1000,
                treat_as_char: true,
                ..Default::default()
            },
            script: "a+b".to_string(),
            font_size: 1000,
            ..Default::default()
        })));
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize equation");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");

        let text_pos = xml.find("<hp:t>ACB</hp:t>").expect("text run");
        let eq_pos = xml.find("<hp:equation ").expect("equation");
        assert!(
            text_pos < eq_pos,
            "ambiguous control gap must not move equation before text: {}",
            xml
        );
    }

    /// 한컴 편집기가 만든 hwp 샘플(`samples/equation-lim.hwp`)의 수식 IR이
    /// HWPX 직렬화 → 재파싱 사이클에서 의미를 잃지 않는지 검증한다.
    ///
    /// 자체 IR 생성 패턴(Document::default + 수동 push)을 회피하고,
    /// 한컴 origin 데이터에서 추출한 Equation을 입력으로 사용한다.
    #[test]
    fn equation_roundtrip_from_hancom_origin_hwp_sample() {
        use crate::model::control::{Control, Equation};
        use crate::parser::parse_hwp;

        let bytes = std::fs::read("samples/equation-lim.hwp")
            .expect("samples/equation-lim.hwp must be readable");
        let original = parse_hwp(&bytes).expect("parse hancom origin hwp");

        let collect_equations = |doc: &Document| -> Vec<Equation> {
            doc.sections
                .iter()
                .flat_map(|s| s.paragraphs.iter())
                .flat_map(|p| p.controls.iter())
                .filter_map(|c| match c {
                    Control::Equation(eq) => Some((**eq).clone()),
                    _ => None,
                })
                .collect()
        };

        let original_eqs = collect_equations(&original);
        assert!(
            !original_eqs.is_empty(),
            "한컴 origin 샘플에 수식이 존재해야 회귀 비교가 의미있음"
        );

        let hwpx_bytes = serialize_hwpx(&original).expect("serialize to hwpx");
        let reparsed = parse_hwpx(&hwpx_bytes).expect("parse hwpx back");
        let reparsed_eqs = collect_equations(&reparsed);

        assert_eq!(
            reparsed_eqs.len(),
            original_eqs.len(),
            "수식 컨트롤 개수가 hwpx 라운드트립에서 유지되어야 함"
        );

        for (i, (orig, rep)) in original_eqs.iter().zip(reparsed_eqs.iter()).enumerate() {
            assert_eq!(
                rep.script, orig.script,
                "[#{}] script must roundtrip through hwpx",
                i
            );
            assert_eq!(
                rep.font_size, orig.font_size,
                "[#{}] font_size must roundtrip",
                i
            );
            assert_eq!(
                rep.baseline, orig.baseline,
                "[#{}] baseline must roundtrip",
                i
            );
            assert_eq!(
                rep.font_name, orig.font_name,
                "[#{}] font_name must roundtrip",
                i
            );
            assert_eq!(rep.color, orig.color, "[#{}] color must roundtrip", i);
            assert_eq!(
                rep.common.width, orig.common.width,
                "[#{}] common.width must roundtrip",
                i
            );
            assert_eq!(
                rep.common.height, orig.common.height,
                "[#{}] common.height must roundtrip",
                i
            );
            assert_eq!(
                rep.common.treat_as_char, orig.common.treat_as_char,
                "[#{}] common.treat_as_char must roundtrip",
                i
            );
        }
    }

    #[test]
    fn linesegs_from_ir_emitted_per_linebreak() {
        // IR 에 lineseg 가 있으면 줄 수만큼 그대로 방출. IR 에 없으면 방출 생략 (#1380)
        // — 종전의 텍스트 `\n` 기반 fallback 합성은 제거됐다.
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "A\nB\nC".to_string();
        for (textpos, vertpos) in [(0u32, 0i32), (2, 1600), (4, 3200)] {
            para.line_segs.push(crate::model::paragraph::LineSeg {
                text_start: textpos,
                vertical_pos: vertpos,
                line_height: 1000,
                text_height: 1000,
                baseline_distance: 850,
                line_spacing: 600,
                tag: crate::model::paragraph::LineSeg::TAG_SINGLE_SEGMENT_LINE,
                ..Default::default()
            });
        }
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");

        let count = xml.matches("<hp:lineseg ").count();
        assert_eq!(count, 3, "expected 3 linesegs, got {}: {}", count, xml);
        assert!(xml.contains(r#"textpos="0" vertpos="0""#));
        assert!(xml.contains(r#"textpos="2" vertpos="1600""#));
        assert!(xml.contains(r#"textpos="4" vertpos="3200""#));
    }

    #[test]
    fn task1380_no_linesegarray_when_ir_has_none() {
        // 파서가 zero-default 를 주입하지 않으므로(#1380) IR 에 lineseg 없는 문단은
        // 패키지 산출물에서도 linesegarray 가 없어야 한다 (원본 무 → RT 무 대칭).
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "A\nB\nC".to_string();
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");

        assert!(!xml.contains("<hp:linesegarray"), "got: {}", xml);
    }

    #[test]
    fn multi_paragraph_emits_multiple_hp_p() {
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        for t in ["첫째 줄", "둘째", "끝"] {
            let mut p = crate::model::paragraph::Paragraph::default();
            p.text = t.to_string();
            section.paragraphs.push(p);
        }
        doc.sections.push(section);
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        let p_count = xml.matches("<hp:p ").count();
        assert_eq!(p_count, 3, "expected 3 <hp:p>, got {}", p_count);
        assert!(xml.contains("<hp:t>첫째 줄</hp:t>"));
        assert!(xml.contains("<hp:t>둘째</hp:t>"));
        assert!(xml.contains("<hp:t>끝</hp:t>"));
    }

    #[test]
    fn xml_escape_applied_to_section_text() {
        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "a & b < c".to_string();
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        assert!(xml.contains("a &amp; b &lt; c"), "escape missing: {}", xml);
    }

    #[test]
    fn mimetype_is_first_entry() {
        let doc = Document::default();
        let bytes = serialize_hwpx(&doc).expect("serialize");
        assert_eq!(&bytes[0..4], b"PK\x03\x04", "ZIP signature");
        let name_len = u16::from_le_bytes([bytes[26], bytes[27]]) as usize;
        let name = &bytes[30..30 + name_len];
        assert_eq!(name, b"mimetype");
    }

    #[test]
    fn mimetype_stored_not_deflated() {
        let doc = Document::default();
        let bytes = serialize_hwpx(&doc).expect("serialize");
        let method = u16::from_le_bytes([bytes[8], bytes[9]]);
        assert_eq!(method, 0, "mimetype must be STORED (method=0)");
    }

    #[test]
    fn hancom_required_files_present() {
        let mut doc = Document::default();
        doc.sections
            .push(crate::model::document::Section::default());
        let bytes = serialize_hwpx(&doc).expect("serialize");
        // ZIP 파일 목록에 한컴 필수 11개가 모두 있는지 확인
        let cursor = std::io::Cursor::new(&bytes);
        let archive = zip::ZipArchive::new(cursor).expect("valid zip");
        let names: Vec<String> = archive.file_names().map(String::from).collect();
        let required = [
            "mimetype",
            "version.xml",
            "Contents/header.xml",
            "Contents/section0.xml",
            "Contents/content.hpf",
            "Preview/PrvText.txt",
            "Preview/PrvImage.png",
            "settings.xml",
            "META-INF/container.xml",
            "META-INF/container.rdf",
            "META-INF/manifest.xml",
        ];
        for r in &required {
            assert!(names.iter().any(|n| n == r), "missing required file: {}", r);
        }
    }

    #[test]
    fn picture_bindata_roundtrip() {
        use crate::model::bin_data::BinDataContent;
        use crate::model::control::Control;
        use crate::model::image::{ImageAttr, Picture};
        use crate::model::shape::CommonObjAttr;

        let fake_png = b"\x89PNG\r\n\x1a\nfake_image_data_for_test";

        let mut doc = Document::default();
        doc.bin_data_content.push(BinDataContent {
            id: 1,
            data: fake_png.to_vec().into(),
            extension: "png".to_string(),
        });

        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "A".to_string();
        para.char_offsets = vec![8];
        para.char_count = 10;
        para.controls.push(Control::Picture(Box::new(Picture {
            common: CommonObjAttr {
                width: 5000,
                height: 3000,
                treat_as_char: true,
                ..Default::default()
            },
            image_attr: ImageAttr {
                bin_data_id: 1,
                ..Default::default()
            },
            ..Default::default()
        })));
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize picture");

        // ZIP에 BinData 엔트리 존재 확인
        let cursor = std::io::Cursor::new(&bytes);
        let archive = zip::ZipArchive::new(cursor).expect("zip");
        let names: Vec<String> = archive.file_names().map(String::from).collect();
        assert!(
            names.iter().any(|n| n.starts_with("BinData/")),
            "BinData/ ZIP entry missing: {:?}",
            names
        );
        drop(archive);

        // section XML에 binaryItemIDRef 포함 확인
        let cursor2 = std::io::Cursor::new(&bytes);
        let mut archive2 = zip::ZipArchive::new(cursor2).expect("zip");
        let mut sec0 = archive2.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        assert!(
            xml.contains("binaryItemIDRef"),
            "binaryItemIDRef missing in section XML: {}",
            xml
        );
        drop(sec0);

        // content.hpf manifest에 embedded BinData 계약이 명시되어야 한컴이 이미지를 로드한다.
        let cursor3 = std::io::Cursor::new(&bytes);
        let mut archive3 = zip::ZipArchive::new(cursor3).expect("zip");
        let mut content_hpf = archive3
            .by_name("Contents/content.hpf")
            .expect("content.hpf");
        let mut content_xml = String::new();
        std::io::Read::read_to_string(&mut content_hpf, &mut content_xml).expect("read");
        assert!(
            content_xml.contains(r#"isEmbeded="1""#),
            "embedded BinData manifest item must include isEmbeded=1: {}",
            content_xml
        );
        drop(content_hpf);

        // 라운드트립: BinData 보존 확인
        let parsed = parse_hwpx(&bytes).expect("parse back");
        assert_eq!(parsed.bin_data_content.len(), 1);
        assert_eq!(&parsed.bin_data_content[0].data.load()[..], fake_png);
        assert_eq!(parsed.bin_data_content[0].extension, "png");
    }

    #[test]
    fn issue1917_bindata_load_failure_preserves_pic_control() {
        // [#1917] BinData 엔트리 로드 실패(압축 해제 상한 초과·엔트리 손상 등)
        // 시에도 pic 컨트롤과 binaryItemIDRef 가 보존되어야 한다. 종전에는
        // bin_data_content 미등록 → resolve_bin_id 실패 → <hp:pic> 드롭으로
        // 왕복 구조 손실(IR_DIFF 하드 실패)이 발생했다.
        use crate::model::bin_data::BinDataContent;
        use crate::model::control::Control;
        use crate::model::image::{ImageAttr, Picture};
        use crate::model::shape::CommonObjAttr;

        fn count_pics(doc: &Document) -> usize {
            doc.sections
                .iter()
                .flat_map(|s| s.paragraphs.iter())
                .flat_map(|p| p.controls.iter())
                .filter(|c| matches!(c, Control::Picture(_)))
                .count()
        }

        let mut doc = Document::default();
        // 재직렬화 시 charPrIDRef=0 해소용 기본 글자모양 (parse_hwpx 산출 IR 동형)
        doc.doc_info
            .char_shapes
            .push(crate::model::style::CharShape::default());
        doc.bin_data_content.push(BinDataContent {
            id: 1,
            data: b"BMfake_bmp_data".to_vec().into(),
            extension: "bmp".to_string(),
        });
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "A".to_string();
        para.char_offsets = vec![8];
        para.char_count = 10;
        para.controls.push(Control::Picture(Box::new(Picture {
            common: CommonObjAttr {
                width: 5000,
                height: 3000,
                treat_as_char: true,
                ..Default::default()
            },
            image_attr: ImageAttr {
                bin_data_id: 1,
                ..Default::default()
            },
            ..Default::default()
        })));
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize picture");

        // BinData 엔트리만 제거한 ZIP 재작성 — read_file_bytes 실패(Err 분기)를
        // 유발한다. 상한 초과와 동일 분기를 실 512MB 페이로드 없이 재현.
        let stripped = {
            let mut archive = zip::ZipArchive::new(std::io::Cursor::new(&bytes)).expect("zip open");
            let mut out = std::io::Cursor::new(Vec::<u8>::new());
            let mut zw = zip::ZipWriter::new(&mut out);
            let opts = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated);
            for i in 0..archive.len() {
                let mut f = archive.by_index(i).expect("entry");
                let name = f.name().to_string();
                if name.starts_with("BinData/") {
                    continue;
                }
                let mut data = Vec::new();
                f.read_to_end(&mut data).expect("read entry");
                zw.start_file(name, opts).expect("start_file");
                std::io::Write::write_all(&mut zw, &data).expect("write entry");
            }
            zw.finish().expect("finish");
            out.into_inner()
        };

        // 재파싱: placeholder 등록 + pic 컨트롤 보존
        let doc2 = parse_hwpx(&stripped).expect("parse stripped");
        assert_eq!(
            doc2.bin_data_content.len(),
            1,
            "로드 실패한 BinData 도 placeholder 로 등록되어야 한다"
        );
        assert!(
            doc2.bin_data_content[0].data.is_empty(),
            "placeholder 데이터는 빈 값이어야 한다"
        );
        assert_eq!(
            doc2.bin_data_content[0].extension, "bmp",
            "placeholder 확장자는 manifest 기준으로 보존"
        );
        assert_eq!(count_pics(&doc2), 1, "pic 컨트롤이 보존되어야 한다");

        // 재직렬화 성공(binaryItemIDRef 미등록 에러 없음) + pic 재보존
        let bytes2 = serialize_hwpx(&doc2).expect("re-serialize with placeholder");
        let doc3 = parse_hwpx(&bytes2).expect("parse re-serialized");
        assert_eq!(count_pics(&doc3), 1, "재직렬화 왕복 후에도 pic 보존");
    }

    #[test]
    fn issue1452_hwpx_preserves_alpha_png_bindata_bytes() {
        use crate::model::bin_data::BinDataContent;
        use crate::model::control::Control;
        use crate::model::image::{ImageAttr, Picture};
        use crate::model::shape::CommonObjAttr;

        let alpha_png = vec![
            0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x04, 0x00, 0x00,
            0x00, 0xb5, 0x1c, 0x0c, 0x02, 0x00, 0x00, 0x00, 0x0b, 0x49, 0x44, 0x41, 0x54, 0x78,
            0xda, 0x63, 0x60, 0x60, 0x00, 0x00, 0x00, 0x03, 0x00, 0x01, 0x2b, 0x09, 0x4d, 0x84,
            0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
        ];

        let mut doc = Document::default();
        doc.bin_data_content.push(BinDataContent {
            id: 1,
            data: alpha_png.clone().into(),
            extension: "png".to_string(),
        });

        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "A".to_string();
        para.char_offsets = vec![8];
        para.char_count = 10;
        para.controls.push(Control::Picture(Box::new(Picture {
            common: CommonObjAttr {
                width: 5000,
                height: 5000,
                treat_as_char: true,
                ..Default::default()
            },
            image_attr: ImageAttr {
                bin_data_id: 1,
                ..Default::default()
            },
            ..Default::default()
        })));
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize alpha png picture");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let bin_name = archive
            .file_names()
            .find(|name| name.starts_with("BinData/") && name.ends_with(".png"))
            .map(String::from)
            .expect("png BinData entry");
        let mut bin = archive.by_name(&bin_name).expect("read png BinData");
        let mut actual = Vec::new();
        std::io::Read::read_to_end(&mut bin, &mut actual).expect("read");

        assert_eq!(
            actual, alpha_png,
            "알파 채널이 있는 PNG BinData 바이트는 HWPX ZIP 안에서도 그대로 보존되어야 한다"
        );
    }

    #[test]
    fn issue_1345_picture_effects_shadow_roundtrip() {
        use crate::model::control::Control;
        use crate::model::shape::ShapeObject;

        fn count_shape_picture_shadows(shape: &ShapeObject) -> usize {
            match shape {
                ShapeObject::Picture(pic) => usize::from(pic.effects.shadow.is_some()),
                ShapeObject::Group(group) => {
                    group.children.iter().map(count_shape_picture_shadows).sum()
                }
                _ => 0,
            }
        }

        fn count_picture_shadows(control: &Control) -> usize {
            match control {
                Control::Picture(pic) => usize::from(pic.effects.shadow.is_some()),
                Control::Shape(shape) => count_shape_picture_shadows(shape),
                _ => 0,
            }
        }

        let bytes = std::fs::read("samples/hwpx/aift.hwpx")
            .expect("samples/hwpx/aift.hwpx must be readable");
        let original = parse_hwpx(&bytes).expect("parse original");
        let parsed_shadow_count: usize = original
            .sections
            .iter()
            .flat_map(|section| &section.paragraphs)
            .flat_map(|para| &para.controls)
            .map(count_picture_shadows)
            .sum();
        assert!(
            parsed_shadow_count > 0,
            "parser must preserve at least one picture shadow effect"
        );

        let serialized = serialize_hwpx(&original).expect("serialize roundtrip");

        let mut archive =
            zip::ZipArchive::new(std::io::Cursor::new(serialized)).expect("roundtrip zip");
        let section_names: Vec<String> = archive
            .file_names()
            .filter(|name| name.starts_with("Contents/section") && name.ends_with(".xml"))
            .map(String::from)
            .collect();
        let mut section_xml = String::new();
        for name in section_names {
            let mut section = archive.by_name(&name).expect("roundtrip section");
            std::io::Read::read_to_string(&mut section, &mut section_xml).expect("read section");
        }

        for needle in ["<hp:shadow ", "<hp:scale ", "<hp:effectsColor ", "<hp:rgb "] {
            assert!(
                section_xml.contains(needle),
                "roundtrip section XML must preserve {needle}"
            );
        }
    }

    #[test]
    fn table_control_roundtrip() {
        use crate::model::control::Control;
        use crate::model::table::Table;

        let mut doc = Document::default();
        doc.doc_info
            .border_fills
            .push(crate::model::style::BorderFill::default());

        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "A".to_string();
        para.char_offsets = vec![8];
        para.char_count = 10;
        para.controls.push(Control::Table(Box::default()));
        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize table");

        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        assert!(
            xml.contains("<hp:tbl ") || xml.contains("<hp:tbl>"),
            "table element missing in section XML: {}",
            xml
        );
        drop(sec0);

        let parsed = parse_hwpx(&bytes).expect("parse back");
        let has_table = parsed.sections[0].paragraphs[0]
            .controls
            .iter()
            .any(|c| matches!(c, Control::Table(_)));
        assert!(has_table, "table control missing after roundtrip");
    }

    #[test]
    fn footnote_endnote_roundtrip() {
        use crate::model::control::Control;
        use crate::model::footnote::{Endnote, Footnote};
        use crate::model::paragraph::Paragraph;

        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "본문".to_string();
        para.char_offsets = vec![8, 17];
        para.char_count = 20;

        let mut fn_para = Paragraph::default();
        fn_para.text = "각주 텍스트".to_string();
        para.controls.push(Control::Footnote(Box::new(Footnote {
            number: 1,
            paragraphs: vec![fn_para],
            after_decoration_letter: 0x0029,
            ..Default::default()
        })));

        let mut en_para = Paragraph::default();
        en_para.text = "미주 텍스트".to_string();
        para.controls.push(Control::Endnote(Box::new(Endnote {
            number: 1,
            paragraphs: vec![en_para],
            after_decoration_letter: 0x0029,
            ..Default::default()
        })));

        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize notes");

        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        assert!(xml.contains("<hp:footNote"), "footNote missing: {}", xml);
        assert!(xml.contains("<hp:endNote"), "endNote missing: {}", xml);
        assert!(
            xml.contains("각주 텍스트"),
            "footnote text missing: {}",
            xml
        );
        assert!(xml.contains("미주 텍스트"), "endnote text missing: {}", xml);
        drop(sec0);

        let parsed = parse_hwpx(&bytes).expect("parse back");
        let ctrls = &parsed.sections[0].paragraphs[0].controls;
        let has_fn = ctrls.iter().any(|c| matches!(c, Control::Footnote(_)));
        let has_en = ctrls.iter().any(|c| matches!(c, Control::Endnote(_)));
        assert!(has_fn, "footnote missing after roundtrip");
        assert!(has_en, "endnote missing after roundtrip");

        let fn_ctrl = ctrls.iter().find_map(|c| match c {
            Control::Footnote(f) => Some(f),
            _ => None,
        });
        assert!(
            fn_ctrl.unwrap().paragraphs[0].text.contains("각주 텍스트"),
            "footnote paragraph text not preserved"
        );
    }

    /// [#2716] `<hp:footNote>` / `<hp:endNote>` 의 장식 문자·번호 모양·고유 ID 왕복 보존.
    ///
    /// 종전 `render_note_sublist` 는 `number` 만 방출해 한컴 저장본이 항상 쓰는
    /// `suffixChar`/`instId`(828/828), `prefixChar`(598/828), `flag`(27/828) 가 전량
    /// 유실됐다. 그 결과 미주 마커 「문1）」이 저장 왕복마다 「1)」로 퇴화했다
    /// (samples/3-09월_교육_통합_2022.hwpx 46/46).
    ///
    /// 방출 규칙은 한컴 HWP5/HWPX 쌍(3-09월_교육_통합_2023, note 46개 전수 대조) 실측:
    /// `flag` 는 number_shape != 0 일 때만, `prefixChar` 는 before != 0 일 때만,
    /// `number`/`suffixChar`/`instId` 는 항상.
    #[test]
    fn footnote_endnote_decoration_attrs_roundtrip() {
        use crate::model::control::Control;
        use crate::model::footnote::{Endnote, Footnote};
        use crate::model::paragraph::Paragraph;

        let mut doc = Document::default();
        let mut section = crate::model::document::Section::default();
        let mut para = crate::model::paragraph::Paragraph::default();
        para.text = "본문".to_string();
        para.char_offsets = vec![8, 17];
        para.char_count = 20;

        let mut fn_para = Paragraph::default();
        fn_para.text = "각주 본문".to_string();
        para.controls.push(Control::Footnote(Box::new(Footnote {
            number: 1,
            paragraphs: vec![fn_para],
            before_decoration_letter: 47928, // 0xBB38 '문'
            after_decoration_letter: 65289,  // 0xFF09 '）'
            number_shape: 3211264,           // 0x00310000
            instance_id: 1085612573,
            ..Default::default()
        })));

        // 접두 없음(0) + 닫는 장식 없음(0): prefixChar/flag 는 생략, suffixChar 는 0 방출.
        let mut en_para = Paragraph::default();
        en_para.text = "미주 본문".to_string();
        para.controls.push(Control::Endnote(Box::new(Endnote {
            number: 2,
            paragraphs: vec![en_para],
            before_decoration_letter: 0,
            after_decoration_letter: 0,
            number_shape: 0,
            instance_id: 7,
            ..Default::default()
        })));

        section.paragraphs.push(para);
        doc.sections.push(section);

        let bytes = serialize_hwpx(&doc).expect("serialize notes");
        let cursor = std::io::Cursor::new(&bytes);
        let mut archive = zip::ZipArchive::new(cursor).expect("zip");
        let mut sec0 = archive.by_name("Contents/section0.xml").expect("section0");
        let mut xml = String::new();
        std::io::Read::read_to_string(&mut sec0, &mut xml).expect("read");
        drop(sec0);

        assert!(
            xml.contains(
                r#"<hp:footNote flag="3211264" number="1" prefixChar="47928" suffixChar="65289" instId="1085612573">"#
            ),
            "footNote 속성이 한컴 계약대로 방출되지 않음: {}",
            xml
        );
        assert!(
            xml.contains(r#"<hp:endNote number="2" suffixChar="0" instId="7">"#),
            "endNote 생략 규칙(flag/prefixChar 생략, suffixChar 항상)이 어긋남: {}",
            xml
        );

        let parsed = parse_hwpx(&bytes).expect("parse back");
        let ctrls = &parsed.sections[0].paragraphs[0].controls;

        let fn_ctrl = ctrls
            .iter()
            .find_map(|c| match c {
                Control::Footnote(f) => Some(f),
                _ => None,
            })
            .expect("footnote ctrl");
        assert_eq!(
            fn_ctrl.before_decoration_letter, 47928,
            "footNote prefixChar"
        );
        assert_eq!(
            fn_ctrl.after_decoration_letter, 65289,
            "footNote suffixChar"
        );
        assert_eq!(fn_ctrl.number_shape, 3211264, "footNote flag");
        assert_eq!(fn_ctrl.instance_id, 1085612573, "footNote instId");

        let en_ctrl = ctrls
            .iter()
            .find_map(|c| match c {
                Control::Endnote(e) => Some(e),
                _ => None,
            })
            .expect("endnote ctrl");
        assert_eq!(
            en_ctrl.before_decoration_letter, 0,
            "endNote prefixChar 없음"
        );
        assert_eq!(
            en_ctrl.after_decoration_letter, 0,
            "endNote suffixChar 0 이 ')' 로 오염됨"
        );
        assert_eq!(en_ctrl.number_shape, 0, "endNote flag 없음");
        assert_eq!(en_ctrl.instance_id, 7, "endNote instId");
    }

    /// tac-img-02.hwpx 파싱 후 BinData 가 존재하는지, Picture 컨트롤이
    /// section XML 에 반영되는지 확인하는 스모크 테스트.
    /// 실문서는 borderFillIDRef 가 복잡하여 full roundtrip 대신 직렬화 단계만 검증.
    #[test]
    fn tac_img_sample_has_pictures_and_bindata() {
        use crate::model::control::Control;

        let bytes = std::fs::read("samples/tac-img-02.hwpx")
            .expect("samples/tac-img-02.hwpx must be readable");
        let original = parse_hwpx(&bytes).expect("parse original");

        assert!(
            !original.bin_data_content.is_empty(),
            "sample must contain BinData"
        );

        let pic_count = original
            .sections
            .iter()
            .flat_map(|s| s.paragraphs.iter())
            .flat_map(|p| p.controls.iter())
            .filter(|c| matches!(c, Control::Picture(_)))
            .count();
        assert!(pic_count > 0, "sample must contain Picture controls");
    }
}
