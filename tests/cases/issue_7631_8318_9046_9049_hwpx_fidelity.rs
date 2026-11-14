//! [#7631, #8318, #9046, #9049] HWPX/HWP 속성 읽기·쓰기 충실도 통합 회귀 검사.
//!
//! src 쪽 단위 테스트는 unit-test-tier 정책(source-side 총량 동결)에 따라
//! 공개 API 기반 통합 테스트로 이관했다.

use std::io::{Cursor, Read};

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section, SectionDef};
use rhwp::model::header_footer::{HeaderFooterApply, MasterPage};
use rhwp::model::paragraph::Paragraph;
use rhwp::model::style::CharShape;
use rhwp::parser::hwpx::section::parse_hwpx_section;
use rhwp::serializer::hwpx::serialize_hwpx;

fn find_first_table(core: &DocumentCore) -> (usize, usize) {
    for (pi, para) in core.document().sections[0].paragraphs.iter().enumerate() {
        for (ci, ctrl) in para.controls.iter().enumerate() {
            if matches!(ctrl, Control::Table(_)) {
                return (pi, ci);
            }
        }
    }
    panic!("문서에 표가 없음");
}

fn master_page(kind: HeaderFooterApply, text: &str) -> MasterPage {
    MasterPage {
        apply_to: kind,
        text_width: 40000,
        text_height: 50000,
        paragraphs: vec![Paragraph {
            text: text.to_owned(),
            ..Default::default()
        }],
        ..Default::default()
    }
}

#[test]
fn issue_7631_position_properties_survive_both_save_formats() {
    let bytes = include_bytes!("../../samples/tac-case-001.hwp");
    for flow in [false, true] {
        for overlap in [false, true] {
            let mut core = DocumentCore::from_bytes(bytes).unwrap();
            let (pi, ci) = find_first_table(&core);
            let properties = format!(r#"{{"restrictInPage":{flow},"allowOverlap":{overlap}}}"#);
            core.set_table_properties_native(0, pi, ci, &properties)
                .unwrap();
            let hwpx = core.export_hwpx_native().unwrap();
            let mut hwpx_core = DocumentCore::from_bytes(&hwpx).unwrap();
            for saved in [hwpx.clone(), hwpx_core.export_hwp_native().unwrap()] {
                let reloaded = DocumentCore::from_bytes(&saved).unwrap();
                let (pi, ci) = find_first_table(&reloaded);
                let Control::Table(table) =
                    &reloaded.document().sections[0].paragraphs[pi].controls[ci]
                else {
                    panic!("표가 없음");
                };
                assert_eq!(table.common.flow_with_text, flow);
                assert_eq!(table.common.allow_overlap, overlap);
            }
            let (pi, ci) = find_first_table(&hwpx_core);
            let changed = format!(
                r#"{{"restrictInPage":{},"allowOverlap":{}}}"#,
                !flow, !overlap
            );
            hwpx_core
                .set_table_properties_native(0, pi, ci, &changed)
                .unwrap();
            let reloaded =
                DocumentCore::from_bytes(&hwpx_core.export_hwpx_native().unwrap()).unwrap();
            let (pi, ci) = find_first_table(&reloaded);
            let Control::Table(table) =
                &reloaded.document().sections[0].paragraphs[pi].controls[ci]
            else {
                panic!("표가 없음");
            };
            assert_eq!(table.common.flow_with_text, !flow);
            assert_eq!(table.common.allow_overlap, !overlap);
        }
    }
}

#[test]
fn issue_7631_table_position_flags_match_common_without_changing_inline_contract() {
    for flow in [false, true] {
        for overlap in [false, true] {
            for inline in [false, true] {
                let flow_num = u8::from(flow);
                let overlap_num = u8::from(overlap);
                let inline_num = u8::from(inline);
                let xml = format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
<hs:sec xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph" xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section">
  <hp:p><hp:run><hp:tbl id="1" rowCnt="1" colCnt="1" textWrap="SQUARE" textFlow="BOTH_SIDES" numberingType="TABLE">
    <hp:sz width="5000" widthRelTo="ABSOLUTE" height="1000" heightRelTo="ABSOLUTE" protect="0"/>
    <hp:pos treatAsChar="{inline_num}" affectLSpacing="0" flowWithText="{flow_num}" allowOverlap="{overlap_num}" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="COLUMN" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/>
    <hp:outMargin left="0" right="0" top="0" bottom="0"/>
    <hp:inMargin left="0" right="0" top="0" bottom="0"/>
    <hp:tr><hp:tc><hp:subList><hp:p><hp:run><hp:t>A</hp:t></hp:run></hp:p></hp:subList><hp:cellAddr colAddr="0" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="5000" height="1000"/><hp:cellMargin left="0" right="0" top="0" bottom="0"/></hp:tc></hp:tr>
  </hp:tbl></hp:run></hp:p>
</hs:sec>"#
                );
                let section = parse_hwpx_section(&xml).unwrap();
                let Control::Table(table) = &section.paragraphs[0].controls[0] else {
                    panic!("표가 파싱되지 않음");
                };
                assert_eq!(table.common.flow_with_text, flow);
                assert_eq!(table.common.allow_overlap, overlap);
                assert_eq!(table.attr & (1 << 13) != 0, flow);
                assert_eq!(table.attr & (1 << 14) != 0, overlap);
                assert_eq!(table.common.attr & (1 << 13) != 0, flow);
                assert_eq!(table.common.attr & (1 << 14) != 0, overlap);
                assert_eq!(
                    table.attr & 1 != 0,
                    table.common.treat_as_char && table.common.flow_with_text
                );
            }
        }
    }
}

#[test]
fn issue_9046_all_master_page_kinds_and_contents_survive_hwp_storage() {
    let kinds = [
        HeaderFooterApply::Both,
        HeaderFooterApply::Even,
        HeaderFooterApply::Odd,
    ];
    let flags = [0x2000_0000, 0x4000_0000, 0x8000_0000];
    for mask in 0..8 {
        for reverse in [false, true] {
            let mut pages: Vec<_> = kinds
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, &kind)| master_page(kind, &format!("MP-{kind:?}")))
                .collect();
            if reverse {
                pages.reverse();
            }
            let mut section = Section {
                section_def: SectionDef {
                    master_pages: pages,
                    ..Default::default()
                },
                ..Default::default()
            };
            section.paragraphs = vec![Paragraph {
                controls: vec![Control::SectionDef(Box::new(section.section_def.clone()))],
                ..Default::default()
            }];
            let mut blank = DocumentCore::new_empty();
            blank.create_blank_document_native().unwrap();
            let mut document = blank.document().clone();
            document.sections = vec![section];
            let core = DocumentCore::from_bytes(&serialize_hwpx(&document).unwrap()).unwrap();
            let reopened = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
            let expected: Vec<_> = kinds
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, &kind)| kind)
                .collect();
            let saved_sd = &reopened.document().sections[0].section_def;
            let expected_flags = flags
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .fold(0, |acc, (_, &flag)| acc | flag);
            assert_eq!(saved_sd.flags & 0xe000_0000, expected_flags);
            assert_eq!(saved_sd.master_pages.len(), expected.len());
            for (mp, kind) in saved_sd.master_pages.iter().zip(expected) {
                assert_eq!(mp.apply_to, kind);
                assert_eq!(mp.paragraphs[0].text, format!("MP-{kind:?}"));
            }
        }
    }
}

#[test]
fn issue_9046_extended_master_pages_keep_their_order_and_contents() {
    let mut last = master_page(HeaderFooterApply::Both, "마지막 쪽");
    last.is_extension = true;
    last.ext_flags = 3;
    last.overlap = true;
    let mut fifth = master_page(HeaderFooterApply::Both, "다섯째 쪽");
    fifth.is_extension = true;
    fifth.ext_flags = 7;
    fifth.optional_page_number = Some(5);
    fifth.page_front = true;
    let mut section = Section {
        section_def: SectionDef {
            master_pages: vec![
                last,
                master_page(HeaderFooterApply::Odd, "홀수"),
                fifth,
                master_page(HeaderFooterApply::Even, "짝수"),
            ],
            ..Default::default()
        },
        ..Default::default()
    };
    section.paragraphs = vec![Paragraph {
        controls: vec![Control::SectionDef(Box::new(section.section_def.clone()))],
        ..Default::default()
    }];
    let mut blank = DocumentCore::new_empty();
    blank.create_blank_document_native().unwrap();
    let mut document = blank.document().clone();
    document.sections = vec![section];
    let core = DocumentCore::from_bytes(&serialize_hwpx(&document).unwrap()).unwrap();
    let reopened = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
    let pages = &reopened.document().sections[0].section_def.master_pages;
    assert_eq!(pages.len(), 4);
    assert_eq!(pages[0].apply_to, HeaderFooterApply::Even);
    assert_eq!(pages[1].apply_to, HeaderFooterApply::Odd);
    assert_eq!(pages[2].paragraphs[0].text, "마지막 쪽");
    assert_eq!(pages[2].ext_flags, 3);
    assert!(pages[2].overlap);
    assert_eq!(pages[3].paragraphs[0].text, "다섯째 쪽");
    assert_eq!(pages[3].ext_flags, 8);
    assert!(pages[3].page_front);
}

#[test]
fn issue_9049_hwpx_note_lengths_and_flags_survive_hwp_records() {
    for length in [-1, 0, 12280, 14173, 14692344] {
        for note_tag in ["footNotePr", "endNotePr"] {
            let placement = if note_tag == "footNotePr" {
                "MERGED_COLUMN"
            } else {
                "END_OF_SECTION"
            };
            let xml = format!(
                r##"<hs:sec xmlns:hs="http://www.hancom.co.kr/hwpml/2011/section" xmlns:hp="http://www.hancom.co.kr/hwpml/2011/paragraph"><hp:p><hp:run><hp:secPr><hp:{note_tag}><hp:autoNumFormat type="DIGIT" prefixChar="(" suffixChar=")" userChar="" supscript="1"/><hp:noteLine length="{length}" type="SOLID" width="0.12 mm" color="#112233"/><hp:noteSpacing aboveLine="850" belowLine="567" betweenNotes="283"/><hp:numbering type="ON_SECTION" newNum="3"/><hp:placement place="{placement}" beneathText="1"/></hp:{note_tag}></hp:secPr></hp:run></hp:p></hs:sec>"##
            );
            let mut section = parse_hwpx_section(&xml).unwrap();
            section.paragraphs[0].controls =
                vec![Control::SectionDef(Box::new(section.section_def.clone()))];
            let mut blank = DocumentCore::new_empty();
            blank.create_blank_document_native().unwrap();
            let mut document = blank.document().clone();
            document.sections = vec![section];
            let hwpx = serialize_hwpx(&document).unwrap();
            let core = DocumentCore::from_bytes(&hwpx).unwrap();
            let reopened = DocumentCore::from_bytes(&core.export_hwp_native().unwrap()).unwrap();
            let sd = &reopened.document().sections[0].section_def;
            let exported = if note_tag == "footNotePr" {
                &sd.footnote_shape
            } else {
                &sd.endnote_shape
            };
            assert_eq!(exported.separator_length, length);
            assert_eq!(exported.attr, 0x3500);
            assert_eq!(exported.separator_above_margin_hu(), 850);
            assert_eq!(exported.note_spacing, 567);
            assert_eq!(exported.raw_unknown, 283);
            assert_eq!(exported.separator_color, 0x0033_2211);
        }
    }
}

#[test]
fn issue_8318_colors_use_hancom_2018_hex_sentinel() {
    for (color, expected) in [
        (0xffff_ffff, "#FFFFFFFF"),
        (0, "#000000"),
        (0x0033_2211, "#112233"),
        (0x8033_2211, "#80112233"),
    ] {
        let mut blank = DocumentCore::new_empty();
        blank.create_blank_document_native().unwrap();
        let mut document = blank.document().clone();
        document.doc_info.char_shapes.push(CharShape {
            text_color: color,
            shade_color: color,
            ..Default::default()
        });
        let hwpx = serialize_hwpx(&document).unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(&hwpx)).unwrap();
        let mut header_xml = String::new();
        archive
            .by_name("Contents/header.xml")
            .unwrap()
            .read_to_string(&mut header_xml)
            .unwrap();
        assert!(
            header_xml.contains(&format!(r#"textColor="{expected}""#)),
            "{header_xml}"
        );
        let reloaded = DocumentCore::from_bytes(&hwpx).unwrap();
        assert_eq!(
            reloaded
                .document()
                .doc_info
                .char_shapes
                .last()
                .unwrap()
                .text_color,
            color
        );
    }
}
