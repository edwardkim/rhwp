//! [Issue #5142] 한 section 파일의 다중 `hp:secPr` 을 구역 하나로 몰아 저장해
//! 한글이 개방을 거부한다 (x2h 유일 잔존 암전, 06544).
//!
//! 06544(63 secPr, 9.09MB): 종전 x2h 는 BodyText/Section0 하나에 secd 63개를 몰아
//! 넣어 한글 COM `Open`=false·0자·1쪽. 한글 자신은 같은 HWPX 를 63개 Section
//! 스트림으로 갈라 저장한다(레코드 정답지 실측). 이등분: 스트림 분할만으로는 여전히
//! 거부 — 잔여 결정타는 중간 secd 의 `PAGE_BORDER_FILL` 이 1개뿐인 것
//! (#3676 계약: 구역마다 정확히 3개가 규격, 미달 시 개방 거부).
//!
//! 수정 2단:
//! 1. serializer(cfb_writer): 문단 중간 SectionDef 경계마다 별도 Section 스트림으로
//!    분할, DOCUMENT_PROPERTIES.section_count 를 실제 스트림 수로 갱신.
//! 2. hwpx_to_hwp 어댑터: 문단 중간 SectionDef 컨트롤에도 PBF 3개 패딩.
//!
//! 실물 검증: 06544 x2h 산출을 한글 2022 COM 이 open=true·116쪽(원본과 동일)으로
//! 수용. 본 테스트는 같은 계약을 합성 IR 왕복으로 고정한다.

use rhwp::document_core::converters::hwpx_to_hwp::convert_hwpx_to_hwp_ir;
use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section, SectionDef};
use rhwp::model::page::PageDef;
use rhwp::model::paragraph::Paragraph;

fn secdef() -> SectionDef {
    let mut sd = SectionDef::default();
    sd.page_def = PageDef {
        width: 59528,
        height: 84188,
        ..Default::default()
    };
    sd
}

fn text_para(text: &str) -> Paragraph {
    let mut p = Paragraph::default();
    p.text = text.to_string();
    p.char_count = text.chars().count() as u32;
    p
}

/// HWPX 파서 산출 형상: IR 구역 하나에 secPr 경계 둘 (문단 중간 SectionDef).
fn multi_sec_pr_document() -> Document {
    let mut first = text_para("첫 구역 본문");
    first
        .controls
        .insert(0, Control::SectionDef(Box::new(secdef())));
    let mut second_head = text_para("둘째 구역 시작");
    second_head
        .controls
        .insert(0, Control::SectionDef(Box::new(secdef())));

    let mut section = Section::default();
    section.section_def = secdef();
    section.paragraphs.push(first);
    section.paragraphs.push(text_para("첫 구역 계속"));
    section.paragraphs.push(second_head);
    section.paragraphs.push(text_para("둘째 구역 본문"));

    let mut doc = Document::default();
    doc.doc_properties.section_count = 1;
    doc.sections.push(section);
    // 분할은 순수 HWPX 출처에서만 발동한다 — 네이티브 HWP5 는 단일 스트림 다중
    // secd 를 한글이 수용하므로(#505 계보) 가르지 않는다.
    doc.provenance.format = rhwp::model::provenance::SourceFormat::Hwpx;
    doc
}

#[test]
fn issue_5142_mid_stream_sec_pr_splits_into_own_section_stream() {
    let mut doc = multi_sec_pr_document();
    convert_hwpx_to_hwp_ir(&mut doc);
    let bytes = rhwp::serializer::cfb_writer::serialize_hwp(&doc).expect("serialize");

    let reparsed = rhwp::document_core::DocumentCore::from_bytes(&bytes).expect("reparse");
    let doc2 = reparsed.document();

    // 구역 분할: SectionDef 경계마다 별도 BodyText/SectionN.
    assert_eq!(
        doc2.sections.len(),
        2,
        "문단 중간 SectionDef 는 별도 Section 스트림이어야 한다"
    );
    // DOCUMENT_PROPERTIES.section_count 도 스트림 수와 일치해야 한다.
    assert_eq!(doc2.doc_properties.section_count, 2, "section_count 불일치");
    // 분할 지점의 문단이 새 구역 첫 문단이 된다.
    assert_eq!(doc2.sections[1].paragraphs[0].text, "둘째 구역 시작");
    assert_eq!(doc2.sections[0].paragraphs.len(), 2);

    // [#3676 계약] 두 구역 모두 secd 의 PAGE_BORDER_FILL 이 3개(base + extra 2)
    // 여야 한글이 스트림을 수용한다.
    for (i, sec) in doc2.sections.iter().enumerate() {
        let sd = sec
            .paragraphs
            .first()
            .and_then(|p| {
                p.controls.iter().find_map(|c| match c {
                    Control::SectionDef(sd) => Some(sd),
                    _ => None,
                })
            })
            .unwrap_or_else(|| panic!("구역 {i} 첫 문단에 secd 가 없다"));
        assert_eq!(
            sd.extra_page_border_fills.len(),
            2,
            "구역 {i}: PAGE_BORDER_FILL 3개 규격 미달 — 한글 개방 거부 형상"
        );
    }
}
