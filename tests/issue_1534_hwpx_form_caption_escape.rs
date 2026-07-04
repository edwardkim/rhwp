//! Task #1534 회귀 테스트 — HWPX 폼 컨트롤 속성값(caption) 이중 이스케이프 누적.
//!
//! 결함: 파싱 측 `attr_str` 가 속성값을 unescape 하지 않고(원문 그대로) IR 에 저장하는데,
//! 직렬화 측 writer 는 속성값을 다시 escape 한다. 이 비대칭으로 폼 caption 의 XML 특수문자
//! (`&`/`<`/`>`/`"`)가 저장할 때마다 한 겹씩 누적된다:
//!   원본 `R&&D` → 1회 저장 `R&amp;&amp;D` → 2회 저장 `R&amp;amp;&amp;amp;D` …
//!
//! 본문(`<hp:t>`)은 `GeneralRef` 이벤트로 정상 디코딩되어 대칭이라 무손상.
//!
//! `samples/hwpx/form-002.hwpx` 는 체크박스 caption 에 `&`(`IP R&&D연계`,
//! `R&&D 자율성트랙(일반)`)를 포함하는 유일한 샘플이라 픽스처로 사용한다.

use std::io::{Cursor, Read};

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;

const SAMPLE: &str = "samples/hwpx/form-002.hwpx";

/// 문단(표 셀 내부 포함)을 재귀적으로 훑어 폼 caption 을 정렬해 수집한다.
fn collect_captions_in_paragraphs(paras: &[Paragraph], out: &mut Vec<String>) {
    for para in paras {
        for control in &para.controls {
            match control {
                Control::Form(form) => out.push(form.caption.clone()),
                Control::Table(table) => {
                    for cell in &table.cells {
                        collect_captions_in_paragraphs(&cell.paragraphs, out);
                    }
                }
                _ => {}
            }
        }
    }
}

fn collect_captions(core: &DocumentCore) -> Vec<String> {
    let mut out = Vec::new();
    for section in &core.document().sections {
        collect_captions_in_paragraphs(&section.paragraphs, &mut out);
    }
    out.sort();
    out
}

/// 저장본(HWPX 패키지 바이트)에서 `Contents/section*.xml` 원문을 모두 이어 붙여 반환한다.
fn section_xml_concat(pkg: &[u8]) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(pkg)).expect("저장본 ZIP 열기 실패");
    let mut all = String::new();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("ZIP 엔트리 접근 실패");
        let name = file.name().to_string();
        if name.contains("section") && name.ends_with(".xml") {
            let mut s = String::new();
            file.read_to_string(&mut s).expect("section XML 읽기 실패");
            all.push_str(&s);
        }
    }
    all
}

/// 원본에 `&` 포함 caption 이 실제로 존재해야 픽스처가 의미 있다(전제 가드).
#[test]
fn fixture_has_ampersand_caption() {
    let bytes = std::fs::read(SAMPLE).expect("form-002.hwpx 로드 실패");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 파싱 실패");
    let captions = collect_captions(&core);
    assert!(
        captions.iter().any(|c| c.contains('&')),
        "전제 위반: form-002 에 '&' 포함 caption 이 없음: {captions:?}"
    );
}

/// 핵심 회귀: parse→serialize→reparse 후 caption 값이 불변이어야 한다.
/// 이중 이스케이프 버그면 `R&&D` → `R&amp;&amp;D` 로 자라 불일치한다.
#[test]
fn form_caption_survives_roundtrip() {
    let bytes = std::fs::read(SAMPLE).expect("form-002.hwpx 로드 실패");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 파싱 실패");
    let before = collect_captions(&core);

    let saved = core.export_hwpx_native().expect("HWPX 직렬화 실패");
    let reloaded = DocumentCore::from_bytes(&saved).expect("저장본 재파싱 실패");
    let after = collect_captions(&reloaded);

    assert_eq!(
        before, after,
        "라운드트립 후 폼 caption 변형(이중 이스케이프 추정)\n원본:   {before:?}\n저장본: {after:?}"
    );
}

/// 저장본 XML 에 이중 이스케이프(`&amp;amp;`)가 없어야 한다.
#[test]
fn saved_xml_has_no_double_escape() {
    let bytes = std::fs::read(SAMPLE).expect("form-002.hwpx 로드 실패");
    let core = DocumentCore::from_bytes(&bytes).expect("HWPX 파싱 실패");
    let saved = core.export_hwpx_native().expect("HWPX 직렬화 실패");

    let xml = section_xml_concat(&saved);
    assert!(
        !xml.contains("&amp;amp;"),
        "저장본 XML 에 이중 이스케이프 '&amp;amp;' 가 존재함(폼 caption 추정)"
    );
}

/// 누적 회귀: 2회 저장해도 caption 이 자라지 않아야 한다(매 저장 한 겹씩 누적 방지).
#[test]
fn form_caption_stable_across_two_roundtrips() {
    let bytes = std::fs::read(SAMPLE).expect("form-002.hwpx 로드 실패");

    let core1 = DocumentCore::from_bytes(&bytes).expect("1차 파싱 실패");
    let saved1 = core1.export_hwpx_native().expect("1차 직렬화 실패");

    let core2 = DocumentCore::from_bytes(&saved1).expect("2차 파싱 실패");
    let round1 = collect_captions(&core2);
    let saved2 = core2.export_hwpx_native().expect("2차 직렬화 실패");

    let core3 = DocumentCore::from_bytes(&saved2).expect("3차 파싱 실패");
    let round2 = collect_captions(&core3);

    assert_eq!(
        round1, round2,
        "2회 저장 시 caption 누적 변형\n1회: {round1:?}\n2회: {round2:?}"
    );
}
