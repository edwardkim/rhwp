//! Stage 2 wasm bridge — `paste_hwpx_fragment_in_document`
//!
//! `DocumentCore` 의 보존된 raw section/header XML 을 이용해 paste fragment 를
//! byte-preserving 으로 적용하고, IR 재파싱·dirty 플래그·이벤트 로그까지 한 번에 처리한다.
//!
//! 5대 원칙 준수:
//! - **Rule 9**: IR→XML 직렬화 회피 (raw 보관본을 사용)
//! - **Rule 10/11/12**: Phase 2 산출물(`paste_fragment_into_section`)에 위임
//! - **Template-based**: source 정의 raw 그대로 보관·재사용

use crate::document_core::commands::fragment_paste::{
    paste_fragment_into_section, FragmentPasteError, IdRemap, SourceDefinitions,
};
use crate::document_core::DocumentCore;
use crate::model::event::DocumentEvent;
use crate::parser::hwpx::{header::parse_hwpx_header, section::parse_hwpx_section};
use crate::renderer::composer::compose_section;

/// `paste_hwpx_fragment_in_document` 결과.
#[derive(Debug, Default)]
pub struct PasteInDocumentResult {
    pub id_remap: IdRemap,
    pub inserted_para_count: usize,
}

/// Stage 2 wasm bridge 에러. Stage 1 의 `FragmentPasteError` 에 IR/IO 관련 케이스를 추가.
#[derive(Debug)]
pub enum PasteInDocumentError {
    /// HWPX raw XML 미보존 (HWP 로드 또는 빈 문서).
    NoSourceXml,
    /// section_idx 가 보관된 raw section 범위를 벗어남.
    SectionOutOfRange { idx: usize, count: usize },
    /// `paste_fragment_into_section` 의 위임 에러.
    Paste(FragmentPasteError),
    /// 결과 section/header XML 의 IR 재파싱 실패.
    Reparse(String),
}

impl std::fmt::Display for PasteInDocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasteInDocumentError::NoSourceXml => {
                write!(
                    f,
                    "document was not loaded from HWPX (no source XML preserved)"
                )
            }
            PasteInDocumentError::SectionOutOfRange { idx, count } => {
                write!(f, "section_idx {idx} out of range (have {count})")
            }
            PasteInDocumentError::Paste(e) => write!(f, "paste failed: {e}"),
            PasteInDocumentError::Reparse(d) => write!(f, "IR reparse failed: {d}"),
        }
    }
}
impl std::error::Error for PasteInDocumentError {}

impl From<FragmentPasteError> for PasteInDocumentError {
    fn from(e: FragmentPasteError) -> Self {
        PasteInDocumentError::Paste(e)
    }
}

impl DocumentCore {
    /// Stage 2 통합 paste 함수.
    ///
    /// 흐름:
    /// 1. 보존된 raw section/header 추출
    /// 2. `paste_fragment_into_section` 호출 (Phase 2)
    /// 3. 결과 raw 를 보존 슬롯에 저장
    /// 4. IR 재파싱 (`parse_hwpx_section` + `parse_hwpx_header`) → `Document.sections[idx]` / `doc_info` 교체
    /// 5. dirty 플래그 + cache 무효화 + `DocumentEvent::FragmentPasted` 추가
    pub fn paste_hwpx_fragment_in_document_native(
        &mut self,
        section_idx: usize,
        after_para_idx: usize,
        fragment_xml: &str,
        source: &SourceDefinitions,
    ) -> Result<PasteInDocumentResult, PasteInDocumentError> {
        if !self.has_source_xmls() {
            return Err(PasteInDocumentError::NoSourceXml);
        }
        let section_count = self.source_section_xmls.len();
        if section_idx >= section_count {
            return Err(PasteInDocumentError::SectionOutOfRange {
                idx: section_idx,
                count: section_count,
            });
        }
        if section_idx >= self.document.sections.len() {
            return Err(PasteInDocumentError::Reparse(format!(
                "source XML section {section_idx} has no matching IR section (have {})",
                self.document.sections.len()
            )));
        }

        // 1. raw 추출 (clone — paste_fragment_into_section 가 &mut String header 를 받기 때문)
        let section_xml = self.source_section_xmls[section_idx].clone();
        let mut header_xml = self.source_header_xml.clone();

        // 2. Phase 2 paste 호출 (byte-preserving)
        let paste_result = paste_fragment_into_section(
            &section_xml,
            &mut header_xml,
            after_para_idx,
            fragment_xml,
            source,
        )?;

        // 3. raw 슬롯 갱신
        let new_section_xml = paste_result.new_section_xml.clone();
        self.source_section_xmls[section_idx] = new_section_xml.clone();
        self.source_header_xml = header_xml.clone();

        // 4. IR 재파싱 (header 먼저, 그 다음 section — id remap 일관성)
        let (new_doc_info, _new_doc_props) = parse_hwpx_header(&header_xml)
            .map_err(|e| PasteInDocumentError::Reparse(format!("header: {e}")))?;
        let new_section = parse_hwpx_section(&new_section_xml)
            .map_err(|e| PasteInDocumentError::Reparse(format!("section: {e}")))?;

        // BinData 목록은 기존을 유지 (재파싱한 doc_info 의 빈 bin_data_list 와 합침)
        let preserved_bin_data = std::mem::take(&mut self.document.doc_info.bin_data_list);
        self.document.doc_info = new_doc_info;
        if self.document.doc_info.bin_data_list.is_empty() {
            self.document.doc_info.bin_data_list = preserved_bin_data;
        }
        self.document.sections[section_idx] = new_section;

        // 4-b. composed 동기화 — 영향받은 섹션만 재컴포즈.
        //      다른 IR 변경 커맨드(document.rs:444 / 466 / 596–597 등)는 모두 동일한
        //      `self.composed = ... compose_section(s) ...` 패턴을 따른다. 이 함수만
        //      누락하면 렌더러가 stale composed 를 참조해 paste 결과가 표시되지 않는다.
        //      (RCA: task_local_yangsik_paste_composed_refresh, 2026-04-28)
        let new_composed = compose_section(&self.document.sections[section_idx]);
        if section_idx < self.composed.len() {
            self.composed[section_idx] = new_composed;
        } else {
            self.composed.resize_with(section_idx + 1, Default::default);
            self.composed[section_idx] = new_composed;
        }

        // 5. dirty 플래그 + cache 무효화
        if section_idx < self.dirty_sections.len() {
            self.dirty_sections[section_idx] = true;
        } else {
            self.dirty_sections.resize(section_idx + 1, true);
            self.dirty_sections[section_idx] = true;
        }
        if section_idx < self.dirty_paragraphs.len() {
            self.dirty_paragraphs[section_idx] = None;
        }
        // 페이지 트리 캐시 전체 무효화
        for slot in self.page_tree_cache.borrow_mut().iter_mut() {
            *slot = None;
        }

        // 6. 이벤트 로그
        self.event_log.push(DocumentEvent::FragmentPasted {
            section: section_idx,
            para: after_para_idx,
        });

        Ok(PasteInDocumentResult {
            id_remap: paste_result.id_remap,
            inserted_para_count: paste_result.inserted_para_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_core_with_minimal_hwpx() -> DocumentCore {
        // 최소 HWPX 구조를 직접 inject (수동 mutation 으로 from_bytes 우회).
        // 단위 테스트용 — 실제 hwpx 로드는 Stage 3 통합 테스트가 검증.
        let mut core = DocumentCore::new_empty();
        core.source_header_xml = String::from(
            "<hh:head>\
<hh:charPropertyList><hh:charPr id=\"0\" height=\"1000\"/></hh:charPropertyList>\
<hh:paraPropertyList><hh:paraPr id=\"0\"/></hh:paraPropertyList>\
<hh:styleList><hh:style id=\"0\" name=\"def\"/></hh:styleList>\
<hh:borderFillList></hh:borderFillList>\
</hh:head>",
        );
        core.source_section_xmls.push(String::from(
            "<hs:sec><hp:p paraPrIDRef=\"0\" styleIDRef=\"0\"><hp:run charPrIDRef=\"0\"><hp:t>x</hp:t></hp:run></hp:p></hs:sec>",
        ));
        // dirty_sections 는 from_bytes 가 채우지만 단위 테스트 픽스처는 직접 채움
        core.dirty_sections = vec![false];
        // IR 도 1 섹션 placeholder 채움
        core.document
            .sections
            .push(crate::model::document::Section::default());
        core
    }

    #[test]
    fn errors_when_no_source_xmls() {
        let mut core = DocumentCore::new_empty();
        let source = SourceDefinitions::default();
        let r =
            core.paste_hwpx_fragment_in_document_native(0, 0, "<hp:p paraPrIDRef=\"0\"/>", &source);
        assert!(matches!(r, Err(PasteInDocumentError::NoSourceXml)));
    }

    #[test]
    fn errors_on_section_out_of_range() {
        let mut core = fixture_core_with_minimal_hwpx();
        let source = SourceDefinitions::default();
        let r =
            core.paste_hwpx_fragment_in_document_native(5, 0, "<hp:p paraPrIDRef=\"0\"/>", &source);
        assert!(matches!(
            r,
            Err(PasteInDocumentError::SectionOutOfRange { idx: 5, count: 1 })
        ));
    }

    #[test]
    fn paragraph_paste_updates_raw_xml_and_dirty() {
        let mut core = fixture_core_with_minimal_hwpx();
        let fragment = r#"<hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>integration</hp:t></hp:run></hp:p>"#;
        let source = SourceDefinitions::default();

        let result = core
            .paste_hwpx_fragment_in_document_native(0, 0, fragment, &source)
            .expect("paste ok");

        assert_eq!(result.inserted_para_count, 1);
        // raw 갱신 확인
        let updated = core.get_source_section_xml(0).expect("section preserved");
        assert!(updated.contains("integration"));
        // dirty 플래그 set
        assert!(core.dirty_sections[0]);
        // 이벤트 로그
        assert!(matches!(
            core.event_log.last(),
            Some(DocumentEvent::FragmentPasted {
                section: 0,
                para: 0
            })
        ));
    }

    #[test]
    fn header_definition_reuse_across_two_pastes() {
        let mut core = fixture_core_with_minimal_hwpx();
        let fragment = r#"<hp:p paraPrIDRef="9" styleIDRef="0"><hp:run charPrIDRef="9"><hp:t>x</hp:t></hp:run></hp:p>"#;
        let source = SourceDefinitions {
            char_prs: "<hh:charPr id=\"9\" height=\"4242\"/>".into(),
            para_prs: "<hh:paraPr id=\"9\" alignTag=\"left\"/>".into(),
            ..Default::default()
        };

        let r1 = core
            .paste_hwpx_fragment_in_document_native(0, 0, fragment, &source)
            .expect("first paste");
        let header_after_first = core.get_source_header_xml().len();

        let r2 = core
            .paste_hwpx_fragment_in_document_native(0, 0, fragment, &source)
            .expect("second paste");
        let header_after_second = core.get_source_header_xml().len();

        // ID 재사용 → header 길이 유지
        assert_eq!(
            header_after_second, header_after_first,
            "header bloated on second paste — ID reuse failed"
        );
        assert_eq!(
            r1.id_remap.char_pr.get(&9),
            r2.id_remap.char_pr.get(&9),
            "ID remap diverged across calls"
        );
        // 두 번 push 됨
        let pasted_events: Vec<_> = core
            .event_log
            .iter()
            .filter(|e| matches!(e, DocumentEvent::FragmentPasted { .. }))
            .collect();
        assert_eq!(pasted_events.len(), 2);
    }

    #[test]
    fn ir_section_actually_grows_after_paste() {
        let mut core = fixture_core_with_minimal_hwpx();
        let fragment = r#"<hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>integration</hp:t></hp:run></hp:p>"#;
        let source = SourceDefinitions::default();

        let para_before = core.document.sections[0].paragraphs.len();
        core.paste_hwpx_fragment_in_document_native(0, 0, fragment, &source)
            .expect("paste ok");
        let para_after = core.document.sections[0].paragraphs.len();
        assert!(
            para_after > para_before,
            "IR section paragraph count did not grow ({} → {})",
            para_before,
            para_after
        );
    }

    #[test]
    fn malformed_fragment_returns_paste_error() {
        let mut core = fixture_core_with_minimal_hwpx();
        let source = SourceDefinitions::default();
        // 닫는 태그 없는 fragment → MalformedFragment
        let r = core.paste_hwpx_fragment_in_document_native(0, 0, "<hp:p>", &source);
        assert!(
            matches!(r, Err(PasteInDocumentError::Paste(_))),
            "expected Paste error, got {r:?}"
        );
    }

    #[test]
    fn page_cache_invalidated_after_paste() {
        let mut core = fixture_core_with_minimal_hwpx();
        // cache 에 더미 항목 채워서 무효화 검증
        core.page_tree_cache.borrow_mut().push(None);
        let fragment = r#"<hp:p paraPrIDRef="0" styleIDRef="0"><hp:run charPrIDRef="0"><hp:t>x</hp:t></hp:run></hp:p>"#;
        let source = SourceDefinitions::default();
        core.paste_hwpx_fragment_in_document_native(0, 0, fragment, &source)
            .expect("paste");
        for slot in core.page_tree_cache.borrow().iter() {
            assert!(slot.is_none(), "cache slot should be None after paste");
        }
    }

    /// 회귀 테스트 — 표 fragment paste 후 `composed[section_idx]` 가 IR 과
    /// 동기화되어야 함. 동기화 누락 시 렌더러가 stale composed 를 사용해 표가 누락된다.
    /// (task_local_yangsik_paste_composed_refresh, RCA: 2026-04-28)
    ///
    /// 시나리오: 표1.fragment.xml(2x2) 1회 paste 후
    /// - IR `Section.paragraphs[*].controls` 에 Table >= 1
    /// - `core.composed[0]` 에 Table inline_control >= 1
    /// 둘 다 존재해야 한다. fixture 시드에 `composed` 가 비어있는 채로 paste 만 진행하면
    /// IR 만 채워지고 composed 는 그대로 비어있어 RED.
    #[test]
    fn table_paste_syncs_composed() {
        let fragment = r#"<hp:p paraPrIDRef="21" styleIDRef="0"><hp:tbl rowCnt="99" colCnt="2"><hp:tr><hp:tc rowAddr="99" colAddr="99" rowSpan="1" colSpan="1"><hp:subList><hp:p paraPrIDRef="22" styleIDRef="0"><hp:run charPrIDRef="9"><hp:t>A</hp:t></hp:run></hp:p></hp:subList></hp:tc><hp:tc rowAddr="99" colAddr="99" rowSpan="1" colSpan="1"><hp:subList><hp:p paraPrIDRef="22" styleIDRef="0"><hp:run charPrIDRef="11"><hp:t>B</hp:t></hp:run></hp:p></hp:subList></hp:tc></hp:tr><hp:tr><hp:tc rowAddr="99" colAddr="99" rowSpan="1" colSpan="1"><hp:subList><hp:p paraPrIDRef="22" styleIDRef="0"><hp:run charPrIDRef="9"><hp:t>C</hp:t></hp:run></hp:p></hp:subList></hp:tc><hp:tc rowAddr="99" colAddr="99" rowSpan="1" colSpan="1"><hp:subList><hp:p paraPrIDRef="22" styleIDRef="0"><hp:run charPrIDRef="11"><hp:t>D</hp:t></hp:run></hp:p></hp:subList></hp:tc></hp:tr></hp:tbl></hp:p>"#;

        let source = SourceDefinitions {
            char_prs: r##"<hh:charPr id="9" height="1200" textColor="#000000" shadeColor="none" useFontSpace="0" useKerning="0" symMark="NONE" borderFillIDRef="2"><hh:fontRef hangul="1" latin="1" hanja="1" japanese="1" other="1" symbol="1" user="1"/><hh:ratio hangul="100" latin="100" hanja="100" japanese="100" other="100" symbol="100" user="100"/><hh:spacing hangul="-10" latin="-10" hanja="-10" japanese="-10" other="-10" symbol="-10" user="-10"/><hh:relSz hangul="100" latin="100" hanja="100" japanese="100" other="100" symbol="100" user="100"/><hh:offset hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/><hh:underline type="NONE" shape="SOLID" color="#000000"/><hh:strikeout shape="NONE" color="#000000"/><hh:outline type="NONE"/><hh:shadow type="NONE" color="#C0C0C0" offsetX="10" offsetY="10"/></hh:charPr>
<hh:charPr id="11" height="1200" textColor="#000000" shadeColor="none" useFontSpace="0" useKerning="0" symMark="NONE" borderFillIDRef="2"><hh:fontRef hangul="2" latin="2" hanja="2" japanese="2" other="2" symbol="2" user="2"/><hh:ratio hangul="100" latin="100" hanja="100" japanese="100" other="100" symbol="100" user="100"/><hh:spacing hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/><hh:relSz hangul="100" latin="100" hanja="100" japanese="100" other="100" symbol="100" user="100"/><hh:offset hangul="0" latin="0" hanja="0" japanese="0" other="0" symbol="0" user="0"/><hh:bold/><hh:underline type="NONE" shape="SOLID" color="#000000"/><hh:strikeout shape="NONE" color="#000000"/><hh:outline type="NONE"/><hh:shadow type="NONE" color="#C0C0C0" offsetX="10" offsetY="10"/></hh:charPr>"##.into(),
            para_prs: r##"<hh:paraPr id="21" tabPrIDRef="0" condense="0" fontLineHeight="0" snapToGrid="1" suppressLineNumbers="0" checked="0"><hh:align horizontal="JUSTIFY" vertical="BASELINE"/><hh:heading type="NONE" idRef="0" level="0"/><hh:breakSetting breakLatinWord="KEEP_WORD" breakNonLatinWord="KEEP_WORD" widowOrphan="0" keepWithNext="0" keepLines="0" pageBreakBefore="0" lineWrap="BREAK"/><hh:autoSpacing eAsianEng="0" eAsianNum="0"/></hh:paraPr>
<hh:paraPr id="22" tabPrIDRef="0" condense="0" fontLineHeight="0" snapToGrid="1" suppressLineNumbers="0" checked="0"><hh:align horizontal="CENTER" vertical="BASELINE"/><hh:heading type="NONE" idRef="0" level="0"/><hh:breakSetting breakLatinWord="KEEP_WORD" breakNonLatinWord="BREAK_WORD" widowOrphan="0" keepWithNext="0" keepLines="0" pageBreakBefore="0" lineWrap="BREAK"/><hh:autoSpacing eAsianEng="0" eAsianNum="0"/></hh:paraPr>"##.into(),
            styles: r##"<hh:style id="0" type="PARA" name="바탕글" engName="Normal" paraPrIDRef="0" charPrIDRef="0" nextStyleIDRef="0" langID="1042" lockForm="0"/>"##.into(),
            border_fills: r##"<hh:borderFill id="3" threeD="0" shadow="0" centerLine="NONE" breakCellSeparateLine="0"><hh:slash type="NONE" Crooked="0" isCounter="0"/><hh:backSlash type="NONE" Crooked="0" isCounter="0"/><hh:leftBorder type="SOLID" width="0.12 mm" color="#000000"/><hh:rightBorder type="SOLID" width="0.12 mm" color="#000000"/><hh:topBorder type="SOLID" width="0.12 mm" color="#000000"/><hh:bottomBorder type="SOLID" width="0.12 mm" color="#000000"/><hh:diagonal type="SOLID" width="0.1 mm" color="#000000"/></hh:borderFill>
<hh:borderFill id="5" threeD="0" shadow="0" centerLine="NONE" breakCellSeparateLine="0"><hh:slash type="NONE" Crooked="0" isCounter="0"/><hh:backSlash type="NONE" Crooked="0" isCounter="0"/><hh:leftBorder type="SOLID" width="0.12 mm" color="#000000"/><hh:rightBorder type="SOLID" width="0.12 mm" color="#000000"/><hh:topBorder type="SOLID" width="0.12 mm" color="#000000"/><hh:bottomBorder type="SOLID" width="0.12 mm" color="#000000"/><hh:diagonal type="SOLID" width="0.1 mm" color="#000000"/><hc:fillBrush><hc:winBrush faceColor="#FBEFD4" hatchColor="#FBEFD4" alpha="0"/></hc:fillBrush></hh:borderFill>"##.into(),
        };

        let mut core = fixture_core_with_minimal_hwpx();
        core.paste_hwpx_fragment_in_document_native(0, 0, &fragment, &source)
            .expect("paste ok");

        // 1) IR 검증 — 표가 들어갔는가
        use crate::model::control::Control;
        let section = &core.document.sections[0];
        let ir_table_count: usize = section
            .paragraphs
            .iter()
            .map(|p| {
                p.controls
                    .iter()
                    .filter(|c| matches!(c, Control::Table(_)))
                    .count()
            })
            .sum();
        assert!(
            ir_table_count >= 1,
            "IR section[0] should have at least 1 Table control after paste, got {ir_table_count}"
        );

        // 2) composed 검증 — 핵심 assertion (현재 코드에선 RED)
        use crate::renderer::composer::InlineControlType;
        let composed_section = core
            .composed
            .get(0)
            .expect("composed[0] should exist after paste — composed sync 누락 시 빈 Vec");
        let composed_table_count: usize = composed_section
            .iter()
            .map(|cp| {
                cp.inline_controls
                    .iter()
                    .filter(|ic| ic.control_type == InlineControlType::Table)
                    .count()
            })
            .sum();
        assert_eq!(
            composed_table_count, ir_table_count,
            "composed[0] table count ({composed_table_count}) must match IR table count ({ir_table_count}) — \
             paste_hwpx_fragment_in_document_native 가 composed 를 갱신하지 않으면 mismatch (RED)"
        );
    }
}
