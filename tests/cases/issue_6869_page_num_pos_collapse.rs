//! [Issue #6869] HWPX 저장은 쪽번호 위치 컨트롤을 **문단당 하나만** 낸다.
//!
//! HWP5 는 같은 문단에 `pngp`(쪽번호 위치)를 여러 개 담을 수 있고 rhwp 파서는 그것을
//! 그대로 보존한다. 그런데 HWPX 로 그대로 내면 **한글이 그 문서에서 멈춘다** — 정답지
//! (한컴 2024) 실측으로 20분 타임아웃이고, 이슈는 44쪽 문서가 12쪽으로 관측됐다.
//! 한컴 자신이 같은 문서를 HWPX 로 저장하면 문단당 하나로 접는다(문서 전체 23 → 5).
//!
//! 접은 슬롯은 **축에서도 빠져야 한다.** 컨트롤만 지우고 `hp:lineseg/@textpos` 를 그대로
//! 두면 축이 오히려 더 어긋나 여전히 멈춘다(실측). 그래서 접을 때 XML 을 한 글자도 내지
//! 않아 `#5943` 의 "방출하지 않은 슬롯만큼 textpos 를 내린다" 보정이 그대로 걸리게 한다.
//!
//! 이 시험이 고정하는 것은 **접기** 다(문단당 하나). 축 보정은 `#5943` 의 기계가 이어받으며,
//! 합성 픽스처로는 그 경로(슬롯 사다리)를 재현할 수 없어 실문서 실측으로 확인했다 —
//! `156532689` 문단 179 의 `textpos` 가 `0/68/128` 로, 한컴이 같은 문서를 저장한 값과
//! 정확히 같아진다(수정 전 `0/132/192`).

#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;

use rhwp::model::control::{Control, PageNumberPos};
use rhwp::model::document::Document;
use rhwp::model::paragraph::{LineSeg, Paragraph};
use rhwp::serializer::hwpx::serialize_hwpx;

fn section0_xml(doc: &Document) -> String {
    let bytes = serialize_hwpx(doc).expect("HWPX 직렬화");
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip");
    let mut f = zip.by_name("Contents/section0.xml").expect("section0.xml");
    let mut s = String::new();
    f.read_to_string(&mut s).expect("read");
    s
}

/// 쪽번호 위치 컨트롤 `n` 개를 담은 문단 하나짜리 문서.
fn doc_with_page_num_controls(n: usize) -> Document {
    let mut doc = Document::default();
    if doc.sections.is_empty() {
        doc.sections.push(Default::default());
    }
    let mut para = Paragraph::default();
    // 확장 제어 문자 n 개 + 뒤따르는 글자.
    para.text = "\u{2}".repeat(n) + "본문";
    for _ in 0..n {
        para.controls
            .push(Control::PageNumberPos(PageNumberPos::default()));
    }
    // 저장 사다리: 둘째 줄은 슬롯 n 개(8유닛씩) 뒤에서 시작한다.
    para.line_segs = vec![
        LineSeg {
            text_start: 0,
            ..Default::default()
        },
        LineSeg {
            text_start: (n as u32) * 8,
            ..Default::default()
        },
    ];
    doc.sections[0].paragraphs.push(para);
    doc
}

#[test]
fn issue6869_page_num_pos_is_emitted_once_per_paragraph() {
    let xml = section0_xml(&doc_with_page_num_controls(9));
    let emitted = xml.matches("<hp:pageNum").count();
    assert_eq!(
        emitted, 1,
        "문단당 hp:pageNum 은 하나여야 한다(한글이 중복에서 멈춘다): {emitted}개\n{xml:.600}"
    );
}

#[test]
fn issue6869_single_page_num_is_untouched() {
    // 중복이 아닌 정상 문단은 종전 그대로 — 컨트롤도 축도 건드리지 않는다.
    let xml = section0_xml(&doc_with_page_num_controls(1));
    assert_eq!(xml.matches("<hp:pageNum").count(), 1);
}
