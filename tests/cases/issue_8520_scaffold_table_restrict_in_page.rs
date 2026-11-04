//! 관계형·고정형(표 속성의 형식 간 대응·독립 사양 비트 계약; 실물 px 검사 아님): PR #8535의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T20:32:41+09:00 (`0884eafc7b2c34fae20a765109ff314514175e4b`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8520/scaffold_table_footnote.hwpx`; 독립 Print `pdf/issue8520/scaffold_table_footnote-2024.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 99.09% / fresh WASM 99.15%.
//! 주장 1쪽 raw: Native 99.09% / fresh WASM 99.15%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8535_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-03 16:43 KST (새 파일)
//!
//! 측정: 2026-11-03, `scripts/visual_sweep.py --silhouette-only --compat 2024 --dpi 96`,
//! `silhouette_raw_match_percent`(2px 내용 실루엣), 글꼴 `RHWP_FONT_PATH=ttfs/opensource`.
//! 원본: `samples/issue8520/scaffold_table_footnote.hwpx`(이 수정의 scaffold + 각주 편집, 2쪽),
//! 기준 PDF `pdf/issue8520/scaffold_table_footnote-2024.pdf`(한글 2024 13.0.0.3901 Print, 2쪽).
//! - Native: 1쪽 98.50, 2쪽 100.00 (전쪽 최저 98.50)
//! - fresh WASM: 1쪽 98.54, 2쪽 100.00 (전쪽 최저 98.54)
//!
//! 수정 전 원본 `samples/issue8520/scaffold_table_footnote_base.hwpx`(flowWithText="0")의 한글 PDF
//! `pdf/issue8520/scaffold_table_footnote_base-2024.pdf` 는 1쪽 4·5행을 각주 글자 위에 그렸다
//! (Native 1쪽 95.81, 2쪽 64.94). 증거: `mydocs/pr/assets/issue_8520/`.
//! 이 파일의 assertion 은 저장 속성 값의 관계만 검사하며 화면 좌표를 고정하지 않는다.
//!
//! scaffold 표는 `table.attr`(0x082A_2210) 의 bit 13(세로 기준이 문단일 때 본문 영역으로 제한)을
//! 켜면서 IR `common.flow_with_text` 와 HWP5 raw 개체 공통 속성 flags 에는 bit 13 을 두지 않았다.
//! HWPX 저장기는 IR 을 따라 `flowWithText="0"` 을 써서, 한글 2024 가 쪽 끝 표를 나누지 않고
//! 각주 영역 위에 겹쳐 그렸다. rhwp 미리보기는 같은 표를 나누므로 두 출력의 쪽 배치가 갈렸다.
//!
//! 기대값 근거 — 한글이 만든 공개 샘플 HWPX 의 글자처럼 취급하지 않는 문단 기준 표 1,561개 중
//! 1,531개가 `flowWithText="1"`. HWP5 개체 공통 속성 bit 13 이 같은 값이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;
use rhwp::model::document::Document;
use rhwp::model::table::Table;
use rhwp::scaffold::{build_scaffold, parse_scaffold_str};
use rhwp::serializer::{serialize_document, serialize_hwpx};

const RESTRICT_IN_PAGE: u32 = 1 << 13;

fn spec() -> &'static str {
    r#"{"version":"1","title":"표 배치","blocks":[
        {"type":"paragraph","text":"표 앞 문단"},
        {"type":"table","rows":[["항목","값"],["가","1"]]},
        {"type":"paragraph","text":"표 다음 문단"}
    ]}"#
}

fn only_table(doc: &Document) -> &Table {
    let tables: Vec<&Table> = doc.sections[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.controls.iter())
        .filter_map(|c| match c {
            Control::Table(t) => Some(t.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(tables.len(), 1, "표는 하나여야 한다");
    tables[0]
}

/// HWP5 개체 공통 속성(CTRL_HEADER 본문) 앞 4바이트가 속성 flags 다.
fn raw_flags(table: &Table) -> u32 {
    let bytes = table.raw_ctrl_data.get(0..4).expect("raw 개체 공통 속성");
    u32::from_le_bytes(bytes.try_into().expect("flags 4바이트"))
}

fn hwpx_table_pos(hwpx: &[u8]) -> String {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(hwpx)).expect("zip");
    let mut entry = archive.by_name("Contents/section0.xml").expect("section0");
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut entry, &mut xml).expect("xml");
    let tbl = xml.split("<hp:tbl ").nth(1).expect("hp:tbl");
    tbl.split("<hp:pos ")
        .nth(1)
        .and_then(|rest| rest.split("/>").next())
        .expect("hp:pos")
        .to_string()
}

#[test]
fn scaffold_table_restricts_to_body_area_in_ir_attr_and_raw_flags() {
    let doc = build_scaffold(&parse_scaffold_str(spec()).expect("scaffold 명세"));
    let table = only_table(&doc);
    assert_ne!(
        table.attr & RESTRICT_IN_PAGE,
        0,
        "표 attr 은 본문 영역 제한을 켠다"
    );
    assert!(
        table.common.flow_with_text,
        "IR flow_with_text 가 표 attr bit 13 과 같아야 한다 (attr={:#x})",
        table.attr
    );
    assert_ne!(
        raw_flags(table) & RESTRICT_IN_PAGE,
        0,
        "HWP5 raw 개체 공통 속성 flags 도 같은 값이어야 한다 (flags={:#x})",
        raw_flags(table)
    );
}

#[test]
fn scaffold_table_keeps_restriction_through_hwpx_and_hwp5() {
    let doc = build_scaffold(&parse_scaffold_str(spec()).expect("scaffold 명세"));
    let hwpx = serialize_hwpx(&doc).expect("HWPX 직렬화");
    let pos = hwpx_table_pos(&hwpx);
    assert!(
        pos.contains(r#"flowWithText="1""#) && pos.contains(r#"vertRelTo="PARA""#),
        "HWPX 표는 문단 기준·본문 영역 제한이어야 한다: <hp:pos {pos}/>"
    );

    let from_hwpx = rhwp::parser::parse_document(&hwpx).expect("HWPX 재파싱");
    assert!(
        only_table(&from_hwpx).common.flow_with_text,
        "HWPX 재파싱 뒤에도 제한"
    );

    let hwp5 = serialize_document(&from_hwpx).expect("HWP5 직렬화");
    let from_hwp5 = rhwp::parser::parse_document(&hwp5).expect("HWP5 재파싱");
    let table = only_table(&from_hwp5);
    assert!(table.common.flow_with_text, "HWP5 변환 뒤에도 제한");
    assert_ne!(table.attr & RESTRICT_IN_PAGE, 0, "HWP5 attr bit 13");
}
