//! HWPX 바탕쪽 종류를 보존하며 HWP5 저장 순서로 정렬한다.
//!
//! HWP5 SECTION_DEF bit 29/30/31은 Both/Even/Odd의 존재 여부이고,
//! LIST_HEADER는 해당 비트 순서로 저장한다. 희소 선언과 구역 간 상속은
//! 종류 자체로 표현되므로 바탕쪽을 복제하거나 다른 종류로 바꾸지 않는다.

use crate::model::document::Document;
use crate::model::header_footer::HeaderFooterApply;

/// 기본 바탕쪽을 Both/Even/Odd 순서로 두고 확장 바탕쪽의 상대 순서를 보존한다.
/// 반환값은 저장 순서가 바뀐 구역 수다.
///
/// 짝수 바탕쪽을 Both로 바꾸면 맞쪽 제본의 종이 기준 개체에 여백 보정이
/// 적용되고, Both/Even/Odd가 함께 있으면 기존 Both 내용까지 유실된다(#9046).
pub(crate) fn materialize_hwp5_master_page_slots(document: &mut Document) -> u32 {
    fn slot(master: &crate::model::header_footer::MasterPage) -> u8 {
        if master.is_extension {
            return 3;
        }
        match master.apply_to {
            HeaderFooterApply::Both => 0,
            HeaderFooterApply::Even => 1,
            HeaderFooterApply::Odd => 2,
        }
    }

    let mut changed_sections = 0;
    for section in &mut document.sections {
        let master_pages = &mut section.section_def.master_pages;
        if !master_pages
            .windows(2)
            .all(|pair| slot(&pair[0]) <= slot(&pair[1]))
        {
            master_pages.sort_by_key(slot);
            changed_sections += 1;
        }
    }
    changed_sections
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::Section;
    use crate::model::header_footer::MasterPage;
    use crate::model::paragraph::Paragraph;

    fn master_page(apply_to: HeaderFooterApply, text: &str) -> MasterPage {
        MasterPage {
            apply_to,
            paragraphs: vec![Paragraph {
                text: text.to_string(),
                ..Default::default()
            }],
            text_width: 40000,
            text_height: 50000,
            ..Default::default()
        }
    }

    #[test]
    fn sparse_odd_master_keeps_hancom_single_slot_contract() {
        let mut document = Document {
            sections: vec![
                Section {
                    section_def: crate::model::document::SectionDef {
                        master_pages: vec![
                            master_page(HeaderFooterApply::Even, "책 제목"),
                            master_page(HeaderFooterApply::Odd, "제1장"),
                        ],
                        ..Default::default()
                    },
                    ..Default::default()
                },
                Section {
                    section_def: crate::model::document::SectionDef {
                        master_pages: vec![master_page(HeaderFooterApply::Odd, "제2장")],
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        assert_eq!(materialize_hwp5_master_page_slots(&mut document), 0);
        let saved = &document.sections[1].section_def.master_pages;
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].apply_to, HeaderFooterApply::Odd);
        assert_eq!(saved[0].paragraphs[0].text, "제2장");

        assert_eq!(materialize_hwp5_master_page_slots(&mut document), 0);
    }
}
