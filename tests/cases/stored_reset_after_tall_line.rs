//! 직전 문단의 마지막 저장 줄이 쪽 위쪽에서 시작해 쪽 끝까지 내려가는 경우
//! (한 줄짜리 쪽 높이 TAC 표), 다음 문단의 저장 vpos=0 은 쪽 경계다.
//! 종전 단일 단 판정은 직전 줄의 **시작** vpos 만 보고 5000HU 이하면 경계를 놓쳐
//! 다음 문단을 같은 쪽 남은 공간에 겹쳐 놓았다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use std::io::{Cursor, Read, Write};

const BASE: &[u8] = include_bytes!("../../samples/task2169/empty_ladder.hwpx");

const LINE: &str =
    r#"textheight="1000" baseline="850" spacing="600" horzpos="0" horzsize="48190" flags="393216""#;

fn fixture(tail_vpos: i32) -> Vec<u8> {
    let mut source = zip::ZipArchive::new(Cursor::new(BASE)).expect("base ZIP");
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).expect("entry");
        let name = entry.name().expect("name").into_owned();
        if name != "Contents/section0.xml" {
            output.raw_copy_file(entry).expect("copy");
            continue;
        }
        let mut xml = String::new();
        entry.read_to_string(&mut xml).expect("section XML");
        // 구역 정의(secPr·colPr)만 남기고 본문을 새로 쓴다.
        let head_end = xml.find("<hp:bookmark").expect("secPr paragraph");
        let mut section = xml[..head_end].to_string();
        section.push_str(&format!(
            r#"<hp:t>머리</hp:t></hp:run><hp:linesegarray><hp:lineseg textpos="0" vertpos="0" vertsize="1000" {LINE}/></hp:linesegarray></hp:p>
<hp:p id="2" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:tbl id="200" zOrder="0" numberingType="TABLE" textWrap="TOP_AND_BOTTOM" textFlow="BOTH_SIDES" lock="0" dropcapstyle="None" pageBreak="CELL" repeatHeader="0" rowCnt="1" colCnt="1" cellSpacing="0" borderFillIDRef="2" noAdjust="0"><hp:sz width="48000" widthRelTo="ABSOLUTE" height="65000" heightRelTo="ABSOLUTE" protect="0"/><hp:pos treatAsChar="1" affectLSpacing="0" flowWithText="1" allowOverlap="0" holdAnchorAndSO="0" vertRelTo="PARA" horzRelTo="PARA" vertAlign="TOP" horzAlign="LEFT" vertOffset="0" horzOffset="0"/><hp:outMargin left="0" right="0" top="0" bottom="0"/><hp:inMargin left="141" right="141" top="141" bottom="141"/><hp:tr><hp:tc name="" header="0" hasMargin="0" protect="0" editable="0" dirty="0" borderFillIDRef="2"><hp:subList id="" textDirection="HORIZONTAL" lineWrap="BREAK" vertAlign="TOP" linkListIDRef="0" linkListNextIDRef="0" textWidth="0" textHeight="0" hasTextRef="0" hasNumRef="0"><hp:p id="201" paraPrIDRef="1" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>표</hp:t></hp:run></hp:p></hp:subList><hp:cellAddr colAddr="0" rowAddr="0"/><hp:cellSpan colSpan="1" rowSpan="1"/><hp:cellSz width="48000" height="65000"/><hp:cellMargin left="141" right="141" top="141" bottom="141"/></hp:tc></hp:tr></hp:tbl></hp:run><hp:linesegarray><hp:lineseg textpos="0" vertpos="1600" vertsize="65000" textheight="65000" baseline="55250" spacing="600" horzpos="0" horzsize="48190" flags="393216"/></hp:linesegarray></hp:p>
<hp:p id="3" paraPrIDRef="0" styleIDRef="0" pageBreak="0" columnBreak="0" merged="0"><hp:run charPrIDRef="0"><hp:t>다음 쪽</hp:t></hp:run><hp:linesegarray><hp:lineseg textpos="0" vertpos="{tail_vpos}" vertsize="1000" {LINE}/></hp:linesegarray></hp:p>
</hs:sec>"#
        ));
        output
            .start_file(name, zip::write::SimpleFileOptions::default())
            .expect("section entry");
        output.write_all(section.as_bytes()).expect("section");
    }
    output.finish().expect("finish").into_inner()
}

#[test]
fn stored_zero_vpos_after_page_tall_line_starts_new_page() {
    // 본문 높이 70018HU, 표 줄은 1600..66600 이라 마지막 문단 한 줄은 남은
    // 공간에 들어간다. 저장 vpos=0 이 쪽 경계를 알리므로 2쪽이어야 한다.
    let core = DocumentCore::from_bytes(&fixture(0)).expect("fixture");
    assert_eq!(core.page_count(), 2);
}

#[test]
fn stored_continuing_vpos_after_page_tall_line_stays_on_page() {
    // 같은 배치에서 vpos 가 이어지면 쪽 경계가 아니다.
    let core = DocumentCore::from_bytes(&fixture(67200)).expect("fixture");
    assert_eq!(core.page_count(), 1);
}
