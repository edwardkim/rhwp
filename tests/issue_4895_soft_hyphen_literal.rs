//! 관계형: 원본 U+00AD와 HWP/HWPX 저장·재열기 뒤의 문자 수를 대조한다.
//! 주석 갱신: 2026-11-03T05:22:26+09:00; 직전 파일 커밋: 2026-10-22T01:37:42+09:00 (7346a928eed0d7ce7460aa4c415cf3c81002f1e8).
//! 기존 저장 왕복 검사의 자료 선택을 보정한 것으로 새 조판·시각 회귀를 등록하지 않는다.
//! Issue #4895: 소프트 하이픈(U+00AD)을 제어 표현으로 저장해 한글이 글자를 버리는 회귀.
//!
//! #4776 은 소프트 하이픈을 HWPX `<hp:hyphen/>` 요소와 HWP5 코드 24(0x18)로 되돌렸다.
//! 그러나 한컴 원본 실측은 반대다 — hwpx 원본은 `<hp:t>` 안에 raw U+00AD 문자를,
//! hwp 원본은 PARA_TEXT 에 `ad 00` 리터럴을 담는다(control_mask 비트 24 도 안 세운다).
//!
//! 한글 2022 대조 실측(10k 코퍼스 01628, 하이픈 표기만 교체):
//!
//! | 검체 | 한글이 뽑은 본문 |
//! |---|---|
//! | 원본 hwpx | 2,478자 |
//! | `<hp:hyphen/>` 로 저장 | 2,477자 (글자 소실) |
//! | raw U+00AD 로 저장 | 2,478자 — 원본과 textSha 일치 |
//!
//! 전수 영향: 한글 2022 오라클 10k 스윕에서 36경로(h2x 22 · x2x 7 · x2h 7)가 깨졌다.
//! 바이트 축 계약은 `src/serializer/body_text.rs`·`src/serializer/hwpx/section.rs` 의
//! 단위 시험이 잡고, 여기서는 실제 문서 왕복을 지킨다.

use std::fs;
use std::io::Read;
use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::model::{control::Control, paragraph::Paragraph};

fn load(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", rel, e));
    DocumentCore::from_bytes(&bytes).expect("parse")
}

/// 저장된 HWPX 의 `Contents/section*.xml` 을 모두 이어 붙인다.
fn section_xml(bytes: &[u8]) -> String {
    let mut zin = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip 열기");
    let mut out = String::new();
    for i in 0..zin.len() {
        let mut f = zin.by_index(i).expect("zip 항목");
        let name = f.name().expect("ZIP 엔트리 이름").to_string();
        if name.starts_with("Contents/section") && name.ends_with(".xml") {
            let mut s = String::new();
            f.read_to_string(&mut s).expect("section xml 읽기");
            out.push_str(&s);
        }
    }
    out
}

/// 저장 원본 IR의 본문과 중첩 표 셀에서 소프트 하이픈을 센다.
/// `export-text`는 렌더링된 표시 문자열을 사용하므로 화면의 하이픈
/// 대체 표기를 원본 문자로 세지 않는다.
fn soft_hyphen_count(core: &DocumentCore) -> usize {
    fn in_paragraph(paragraph: &Paragraph) -> usize {
        let own = paragraph.text.matches('\u{00AD}').count();
        let nested = paragraph
            .controls
            .iter()
            .filter_map(|control| match control {
                Control::Table(table) => Some(
                    table
                        .cells
                        .iter()
                        .flat_map(|cell| &cell.paragraphs)
                        .map(in_paragraph)
                        .sum::<usize>(),
                ),
                _ => None,
            })
            .sum::<usize>();
        own + nested
    }

    core.document()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .map(in_paragraph)
        .sum()
}

#[test]
fn hwpx_save_writes_soft_hyphen_as_literal_char() {
    let core = load("samples/hwpx/hcar-001.hwpx");
    let before = soft_hyphen_count(&core);
    assert!(before > 0, "표본에 소프트 하이픈이 있어야 시험이 성립한다");

    let saved = core.export_hwpx_native().expect("export hwpx");
    let xml = section_xml(&saved);

    assert!(
        !xml.contains("<hp:hyphen/>") && !xml.contains("<hp:hypen/>"),
        "소프트 하이픈을 <hp:hyphen/> 요소로 내리면 한글이 그 글자를 버린다"
    );
    assert!(
        xml.contains('\u{00AD}'),
        "소프트 하이픈이 hp:t 안에 리터럴 문자로 실려야 한다"
    );

    let reloaded = DocumentCore::from_bytes(&saved).expect("reparse");
    assert_eq!(
        soft_hyphen_count(&reloaded),
        before,
        "저장·재로드에서 소프트 하이픈 수가 보존되어야 한다"
    );
}

#[test]
fn hwp5_save_preserves_soft_hyphen_through_roundtrip() {
    let mut core = load("samples/hcar-001.hwp");
    let before = soft_hyphen_count(&core);
    assert!(before > 0, "표본에 소프트 하이픈이 있어야 시험이 성립한다");

    let saved = core.export_hwp_with_adapter().expect("export hwp");
    let reloaded = DocumentCore::from_bytes(&saved).expect("reparse");

    assert_eq!(
        soft_hyphen_count(&reloaded),
        before,
        "HWP5 저장·재로드에서 소프트 하이픈이 '-'(U+002D)로 변질되거나 사라지면 안 된다"
    );
}
