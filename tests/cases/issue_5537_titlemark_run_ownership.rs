//! [Issue #5537] HWPX 저장에서 titleMark의 run 소유를 보존한다.
//!
//! titleMark 직후 문자 위치의 `char_shapes` 경계가 표시 끝 유닛과 일치하면, 원본은
//! 표시까지를 닫히는 run에 넣었다는 뜻이다. 이를 다음 run 머리로 보내면 HWPX 재파싱 시
//! 글자 모양 경계가 표시 폭(8유닛)만큼 무너진다. 반대로 그러한 경계가 없으면 표시는
//! 같은 텍스트 run 안에서 다음 문자 앞에 유지해야 한다.

use std::io::Read;

use rhwp::model::document::{Document, Section};
use rhwp::model::paragraph::{CharShapeRef, Paragraph, TitleMark};
use rhwp::serializer::hwpx::serialize_hwpx;

fn section0_xml(hwpx: &[u8]) -> String {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("HWPX zip 열기");
    let mut entry = zip
        .by_name("Contents/section0.xml")
        .expect("Contents/section0.xml");
    let mut xml = String::new();
    entry.read_to_string(&mut xml).expect("section0.xml 읽기");
    xml
}

fn document_with_title_mark(prev_run_owns_mark: bool) -> Document {
    let mut doc = Document::default();
    // run이 참조하는 글자 모양 ID 0, 1은 HWPX header에도 실재해야 한다.
    doc.doc_info.char_shapes = vec![Default::default(), Default::default()];
    let char_shapes = if prev_run_owns_mark {
        // `다`의 축 위치 10은 `가나`(2유닛) 뒤 titleMark(8유닛)의 끝이다.
        // 이 경계는 titleMark가 닫히는 첫 run 소유라는 신호다.
        vec![
            CharShapeRef {
                start_pos: 0,
                char_shape_id: 0,
            },
            CharShapeRef {
                start_pos: 10,
                char_shape_id: 1,
            },
        ]
    } else {
        vec![CharShapeRef {
            start_pos: 0,
            char_shape_id: 0,
        }]
    };
    let para = Paragraph {
        text: "가나다라".to_string(),
        // `다` 앞의 titleMark가 8유닛을 점유한다.
        char_offsets: vec![0, 1, 10, 11],
        // titleMark 8 + 본문 4 + 문단 끝 마커 1.
        char_count: 13,
        char_shapes,
        title_marks: vec![TitleMark {
            char_idx: 2,
            ignore: true,
        }],
        ..Default::default()
    };
    doc.sections.push(Section {
        paragraphs: vec![para],
        ..Default::default()
    });
    doc
}

#[test]
fn issue_5537_title_mark_stays_in_the_run_that_owns_its_end_boundary() {
    let xml = section0_xml(&serialize_hwpx(&document_with_title_mark(true)).expect("HWPX 직렬화"));
    let mark = xml.find("<hp:titleMark").expect("titleMark가 있어야 한다");
    let next_run = xml[mark..]
        .find("</hp:t></hp:run><hp:run")
        .map(|offset| mark + offset)
        .expect("titleMark 뒤에 다음 char-shape run이 있어야 한다");

    assert!(
        mark < next_run,
        "표시 끝 경계를 소유한 titleMark는 다음 run 머리가 아니라 닫히는 run 말미에 있어야 한다: {xml}"
    );
    assert!(
        xml[next_run..].contains("<hp:t>다라</hp:t>"),
        "다음 run은 titleMark를 다시 쓰지 않고 본문으로 시작해야 한다: {xml}"
    );
}

#[test]
fn issue_5537_title_mark_without_an_ownership_boundary_stays_before_its_text() {
    let xml = section0_xml(&serialize_hwpx(&document_with_title_mark(false)).expect("HWPX 직렬화"));

    assert!(
        xml.contains("<hp:t>가나<hp:titleMark ignore=\"1\"/>다라</hp:t>"),
        "소유 경계가 없는 titleMark는 종전처럼 다음 텍스트 앞에 남아야 한다: {xml}"
    );
}
