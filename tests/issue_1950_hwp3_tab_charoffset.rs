//! Issue #1950: HWP3→HWP5 변환 시 탭 문단 char 위치 단위 불일치 회귀 가드.
//!
//! 재현 문서 (tracked 공개 샘플): `samples/issue1950_hwp3_tab_charoffset.hwp`
//! (법원 예규 별지 서식 "미제사건보고서", HWP3 v3.0.0.0, 탭 다수).
//!
//! 결함 본질: HWP5 파서는 `char_offsets`/char_shape `start_pos` 를 UTF-16 code-unit
//! 위치로 산출하고 탭은 PARA_TEXT 에서 8 code-unit(0x0009 + 확장 7)을 차지한다.
//! 그러나 HWP3 파서는 탭을 1 code-unit 으로만 세어(char_offsets/char_count) IR 단위가
//! 어긋났다. 직렬화기는 탭을 8-unit 으로 확장하므로, HWP3-origin IR 을 HWP5 로 저장하면
//! char_shape[1](자간 0%)이 code-unit 중간(탭 확장 지점)부터 적용돼 탭 폭이 바뀌고 탭 run
//! 이 쪼개져 렌더가 최대 376px 이탈했다(TextRun±1).
//!
//! 정정: HWP3 파서가 탭을 8 code-unit 으로 세어(`parser/hwp3/mod.rs` `utf16_len += 8`)
//! char_offsets/char_count/char_shape start_pos 를 HWP5 시멘틱과 통일. 논리 char↔shape
//! 매핑은 불변이므로 원본 HWP3 렌더는 그대로다(golden svg_snapshot 불변 확인).

use rhwp::document_core::DocumentCore;
use rhwp::model::document::Document;

fn load_bytes() -> Vec<u8> {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/samples/issue1950_hwp3_tab_charoffset.hwp"
    );
    std::fs::read(path).unwrap_or_else(|e| panic!("샘플 읽기 실패: {e}"))
}

/// 탭을 포함한 첫 (section, para) 인덱스를 찾는다.
fn find_tab_para(doc: &Document) -> (usize, usize) {
    for (si, sec) in doc.sections.iter().enumerate() {
        for (pi, para) in sec.paragraphs.iter().enumerate() {
            if para.text.contains('\t') {
                return (si, pi);
            }
        }
    }
    panic!("탭 문단을 찾지 못함");
}

#[test]
fn hwp3_tab_is_eight_code_units_and_roundtrip_consistent() {
    let bytes = load_bytes();

    // 1) HWP3 파서 출력: 탭이 char_offsets 에서 8 code-unit 을 차지해야 한다.
    let doc3 = rhwp::parser::parse_document(&bytes).expect("HWP3 파싱 실패");
    let (si, pi) = find_tab_para(&doc3);
    let p3 = &doc3.sections[si].paragraphs[pi];
    let chars: Vec<char> = p3.text.chars().collect();
    let mut checked = false;
    for (i, &ch) in chars.iter().enumerate() {
        if ch == '\t' && i + 1 < p3.char_offsets.len() {
            let delta = p3.char_offsets[i + 1] - p3.char_offsets[i];
            assert_eq!(
                delta, 8,
                "HWP3 탭 char_offset 증가폭이 {delta} (기대 8 code-unit). \
                 수정 전 회귀(1-unit) — HWP5 탭 확장과 불일치",
            );
            checked = true;
        }
    }
    assert!(checked, "탭 char_offset 을 검증하지 못함");

    // 2) HWP5 왕복 후 char_count 정합: HWP3-origin IR(8-unit)을 HWP5 로 저장·재파스해도
    //    탭 문단 char_count 가 유지되어야 한다(수정 전엔 31→88 로 팽창해 char_shape 정렬 붕괴).
    let core = DocumentCore::from_bytes(&bytes).expect("DocumentCore 로드 실패");
    let hwp5 = core.export_hwp_native().expect("HWP5 직렬화 실패");
    let doc5 = rhwp::parser::parse_document(&hwp5).expect("HWP5 재파싱 실패");
    // 직렬화기가 첫 문단에 SectionDef(#1915)를 보강할 수 있어 para 인덱스는 동일 유지.
    let p5 = &doc5.sections[si].paragraphs[pi];
    // 수정 전 회귀는 탭당 +7 씩 팽창(예: cc 31→88, +57)한다. secd 보강(#1915)·후행 처리
    // 등 탭 무관 소차(±수)는 허용하되, 탭 단위 불일치가 재발하면 문단당 최소 +7 이상 벌어져
    // 이 허용치를 넘는다.
    let diff = (p3.char_count as i64 - p5.char_count as i64).abs();
    assert!(
        diff < 7,
        "HWP3↔HWP5 왕복 탭 문단 char_count 팽창 {diff} (기대 <7). \
         탭 code-unit 단위 불일치 회귀 의심 ({} vs {})",
        p3.char_count,
        p5.char_count,
    );
}
