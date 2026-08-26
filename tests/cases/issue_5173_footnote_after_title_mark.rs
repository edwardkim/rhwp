//! [Issue #5173] HWP5 → HWPX 저장에서 각주가 문단 맨 앞으로 나가던 문제.
//!
//! 제목 차례 표시(titleMark)가 있는 문단에서, HWPX 직렬화기의 슬롯 루프가 titleMark 가
//! 차지하는 8유닛 갭까지 슬롯 몫으로 보고 각주(슬롯)를 먼저 방출했다. titleMark 는 텍스트
//! run(`render_hp_t_content`) 안 char_idx 위치에 방출되므로, 각주가 제목·본문보다 앞서 나갔다
//! (08435 캄보디아 최종보고서: `1)1) …각주내용` 이 `제1절 지정학적 위치와 자연환경` 앞으로).
//!
//! 수정: titleMark 유닛(각 8)을 슬롯 갭 계산에서 빼, 각주가 자기 실제 갭(제목·본문 뒤)으로
//! 내려가 원본·h2h 와 같은 순서(제목 → 본문 → 각주)를 따르게 한다.
//!
//! 계약: titleMark + 본문 + 각주가 든 문단을 HWPX 로 저장하면 `<hp:titleMark>` 가
//! `<hp:footNote>` 보다 **앞**에 나와야 한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Read;

use rhwp::model::control::Control;
use rhwp::model::document::{Document, Section};
use rhwp::model::footnote::Footnote;
use rhwp::model::paragraph::{Paragraph, TitleMark};
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

#[test]
fn footnote_is_emitted_after_title_mark_and_text() {
    let para = Paragraph {
        text: "가나다".to_string(),
        // titleMark 8유닛 뒤에 3글자.
        char_offsets: vec![8, 9, 10],
        // 표시 8 + 글자 3 + 각주 8 + 문단 끝 마커 1 = 20. 각주 유닛을 세야 축이 충실해져
        // (그렇지 않으면 fallback 경로로 빠져 각주가 그냥 말미로 가 버그가 가려진다) 슬롯
        // 루프가 실행되고 #5173 회귀를 잡는다.
        char_count: 20,
        title_marks: vec![TitleMark {
            char_idx: 0,
            ignore: true,
        }],
        controls: vec![Control::Footnote(Box::new(Footnote {
            number: 1,
            ..Default::default()
        }))],
        ..Default::default()
    };
    let mut doc = Document::default();
    doc.sections.push(Section {
        paragraphs: vec![para],
        ..Default::default()
    });

    let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
    let xml = section0_xml(&hwpx);

    let tm = xml.find("<hp:titleMark").expect("titleMark 가 있어야 한다");
    // 인라인 각주(`<hp:footNote number=…`)만 찾는다 — 구역 속성의 각주 모양
    // (`<hp:footNotePr`)은 뒤에 공백이 없어 제외된다.
    let fnpos = xml
        .find("<hp:footNote ")
        .expect("인라인 footNote 가 있어야 한다");
    assert!(
        tm < fnpos,
        "titleMark 가 footNote 보다 앞에 나와야 한다 — 각주가 문단 맨 앞으로 나가면 안 된다 \
         (#5173 회귀): titleMark@{tm} footNote@{fnpos}"
    );
}
