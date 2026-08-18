//! Issue #5174: 묶음 빈칸(U+00A0)의 표기가 저장 방향마다 뒤집히는 회귀.
//!
//! 한컴이 만든 문서는 묶음 빈칸을 두 가지로 적는다 — HWPX 는 `<hp:nbSpace/>` 요소 또는
//! `<hp:t>` 안 리터럴 U+00A0, HWP5 는 PARA_TEXT 제어코드 `0x001E` 또는 리터럴 `a0 00`.
//! 한글은 **제어·요소 표기를 본문 텍스트 추출에 싣지 않고 리터럴은 싣는다.** 그래서
//! 저장하면서 표기를 바꾸면 원본에 없던 공백이 생기거나(요소→리터럴) 있던 공백이
//! 사라진다(리터럴→제어).
//!
//! 종전 코드는 방향마다 한쪽으로 강제했다 — HWPX 저장은 늘 리터럴, HWP5 저장은 늘 코드
//! `0x001E`. 한글 2022 오라클 10k 스윕에서 이 서명이 206경로였다(리터럴 강등 185 · 제어
//! 승격 21).
//!
//! 고정폭 빈칸(#4675)처럼 한쪽으로 강제할 수는 없다. 한컴 원본이 두 표기를 다 쓰고 한
//! 문단이 둘을 섞는 경우는 0건이라, 소프트 하이픈(#4895)과 같이 **출처를 보존**해야 한다.
//! 출처 신호는 PARA_HEADER `control_mask` 비트 30 이다.
//!
//! 바이트·XML 축의 단위 계약은 `src/serializer/body_text.rs` 의 단위시험이 잡고,
//! 여기서는 실제 문서가 두 저장 축을 지나도 표기가 유지되는지를 지킨다.

use std::fs;
use std::io::Read;
use std::path::Path;

use rhwp::document_core::DocumentCore;

/// 묶음 빈칸을 `<hp:nbSpace/>` 요소로만 적은 표본(요소 411 · 리터럴 0).
const ELEMENT_ORIGIN: &str = "samples/hwpx/exam_kor.hwpx";

/// 묶음 빈칸을 리터럴 U+00A0 으로만 적은 표본(요소 0 · 리터럴 16).
const LITERAL_ORIGIN: &str =
    "samples/정책연구용역사업 중간진도보고서(살아있는 간장 기증자의 의학적 선별기준 연구).hwpx";

fn load(rel: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("parse")
}

/// 저장된 HWPX 의 `Contents/section*.xml` 을 모두 이어 붙인다.
fn section_xml(bytes: &[u8]) -> String {
    let mut zin = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip 열기");
    let mut out = String::new();
    for i in 0..zin.len() {
        let mut f = zin.by_index(i).expect("zip 항목");
        let name = f.name().to_string();
        if name.starts_with("Contents/section") && name.ends_with(".xml") {
            let mut s = String::new();
            f.read_to_string(&mut s).expect("section xml 읽기");
            out.push_str(&s);
        }
    }
    out
}

/// (요소 표기 수, 리터럴 표기 수)
fn representations(xml: &str) -> (usize, usize) {
    (
        xml.matches("<hp:nbSpace/>").count(),
        xml.matches('\u{00A0}').count(),
    )
}

/// rhwp 자신의 본문 추출 축 — 표기와 무관하게 U+00A0 으로 올라오므로 **글자 수 보존**을 본다.
fn nbsp_char_count(core: &DocumentCore) -> usize {
    (0..core.page_count())
        .filter_map(|p| core.extract_page_text_native(p).ok())
        .map(|t| t.matches('\u{00A0}').count())
        .sum()
}

#[test]
fn hwpx_save_keeps_element_origin_as_element() {
    let core = load(ELEMENT_ORIGIN);
    let before = nbsp_char_count(&core);
    assert!(before > 0, "표본에 묶음 빈칸이 있어야 시험이 성립한다");

    let (el, lit) = representations(&section_xml(
        &core.export_hwpx_native().expect("export hwpx"),
    ));

    assert!(
        el > 0,
        "요소 표기 원본은 `<hp:nbSpace/>` 로 보존되어야 한다 — 리터럴로 내리면 한글 추출 \
         텍스트에 없던 공백이 생긴다"
    );
    assert_eq!(lit, 0, "요소로 낸 뒤 리터럴이 함께 남으면 안 된다");
}

#[test]
fn hwpx_save_keeps_literal_origin_as_literal() {
    let core = load(LITERAL_ORIGIN);
    let before = nbsp_char_count(&core);
    assert!(before > 0, "표본에 묶음 빈칸이 있어야 시험이 성립한다");

    let (el, lit) = representations(&section_xml(
        &core.export_hwpx_native().expect("export hwpx"),
    ));

    assert_eq!(
        el, 0,
        "리터럴 원본을 `<hp:nbSpace/>` 요소로 바꾸면 한글이 그 공백을 텍스트에서 버린다"
    );
    assert!(lit > 0, "리터럴 표기가 `<hp:t>` 안에 그대로 남아야 한다");
}

/// x2h — HWP5 축을 지나도 요소 출처가 살아남는다.
///
/// HWPX 파서가 `<hp:nbSpace/>` 에서 세운 `control_mask` 비트 30 이 PARA_HEADER 로 넘어가
/// PARA_TEXT 가 코드 `0x001E` 로 나가야, 그 HWP5 를 다시 읽었을 때 요소로 되돌아온다.
#[test]
fn hwp5_roundtrip_keeps_element_origin() {
    let mut core = load(ELEMENT_ORIGIN);
    let before = nbsp_char_count(&core);

    let hwp5 = core.export_hwp_with_adapter().expect("export hwp");
    let reloaded = DocumentCore::from_bytes(&hwp5).expect("reparse");
    assert_eq!(
        nbsp_char_count(&reloaded),
        before,
        "HWP5 왕복에서 묶음 빈칸 수가 달라지면 안 된다"
    );

    let (el, lit) = representations(&section_xml(
        &reloaded.export_hwpx_native().expect("export hwpx"),
    ));
    assert!(el > 0, "HWP5 를 지난 뒤에도 요소 표기여야 한다");
    assert_eq!(lit, 0, "HWP5 를 지나며 리터럴로 강등되면 안 된다");
}

/// x2h 반대편 — 리터럴 출처는 HWP5 축에서 제어코드로 승격되면 안 된다.
#[test]
fn hwp5_roundtrip_keeps_literal_origin() {
    let mut core = load(LITERAL_ORIGIN);
    let before = nbsp_char_count(&core);

    let hwp5 = core.export_hwp_with_adapter().expect("export hwp");
    let reloaded = DocumentCore::from_bytes(&hwp5).expect("reparse");
    assert_eq!(
        nbsp_char_count(&reloaded),
        before,
        "HWP5 왕복에서 묶음 빈칸 수가 달라지면 안 된다"
    );

    let (el, lit) = representations(&section_xml(
        &reloaded.export_hwpx_native().expect("export hwpx"),
    ));
    assert_eq!(
        el, 0,
        "리터럴 원본이 HWP5 축에서 제어코드로 승격되면 한글이 그 공백을 버린다"
    );
    assert!(lit > 0, "HWP5 를 지난 뒤에도 리터럴 표기여야 한다");
}
