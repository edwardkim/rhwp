//! [Issue #4398] 다단락 필드의 고아 fieldEnd(짝 fieldBegin 이 앞 문단)가 HWP5
//! 를 거치는 왕복에서 소실돼 char_count 9→1 로 무너지던 계약을 고정한다.
//!
//! 근인: `serialize_section` 의 `has_content` 판정과 `compute_control_mask` 가
//! `orphan_field_ends` 를 몰라, 고아 종료 마커만 있는 문단이 "빈 문단" 으로
//! 접혔다 — PARA_TEXT 없이 헤더만 char_count 를 주장하는 자기모순 레코드
//! (한글 2022 는 그런 문단을 만나면 본문 전체를 버린다) 또는 #4677 가드의
//! char_count=1 붕괴. 방출 자체(`serialize_para_text` 의 mid-text·trailing 고아
//! 방출)는 이미 있었으므로 게이트만 맞추면 왕복이 닫힌다.

use rhwp::document_core::DocumentCore;
use rhwp::parse_document;

const SAMPLE: &str = "samples/task2279/36378481_gyeoljae.hwpx";

fn read_sample() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

/// 고아 fieldEnd 만 가진(텍스트·컨트롤 없는) 문단의 인덱스와 char_count.
fn orphan_only_para(doc: &rhwp::model::document::Document) -> (usize, u32) {
    for (pi, p) in doc.sections[0].paragraphs.iter().enumerate() {
        if !p.orphan_field_ends.is_empty() && p.text.is_empty() && p.controls.is_empty() {
            return (pi, p.char_count);
        }
    }
    panic!("샘플 전제: 고아 fieldEnd 전용 문단이 있어야 한다");
}

#[test]
fn issue4398_orphan_fieldend_survives_hwp5_roundtrip() {
    let original = parse_document(&read_sample()).expect("parse hwpx");
    let (pi, cc) = orphan_only_para(&original);
    assert_eq!(cc, 9, "샘플 전제: 고아 8유닛 + 종결 1 = 9");
    assert_ne!(
        original.sections[0].paragraphs[pi].orphan_field_ends[0].begin_ctrl_id, 0,
        "샘플 전제: 짝 fieldBegin 의 ctrl_id 가 연결돼 있어야 한다"
    );

    // HWPX → HWP5
    let mut core = DocumentCore::from_bytes(&read_sample()).expect("open hwpx");
    let hwp = core.export_hwp_with_adapter().expect("convert to hwp");
    let mid = parse_document(&hwp).expect("parse hwp");
    let mid_para = &mid.sections[0].paragraphs[pi];
    assert_eq!(
        mid_para.char_count, 9,
        "HWP5 저장본에서 고아 fieldEnd 8유닛이 실체(PARA_TEXT)로 남아야 한다 (#4398)"
    );
    assert_eq!(
        mid_para.orphan_field_ends.len(),
        1,
        "HWP5 재파싱이 고아 종료 마커를 복원해야 한다"
    );

    // HWP5 → HWPX (왕복 완주)
    let core2 = DocumentCore::from_bytes(&hwp).expect("open hwp");
    let hwpx2 = core2.export_hwpx_native().expect("export hwpx");
    let fin = parse_document(&hwpx2).expect("parse final hwpx");
    let fin_para = &fin.sections[0].paragraphs[pi];
    assert_eq!(
        fin_para.char_count, 9,
        "왕복 후에도 char_count 가 무너지면 안 된다 (종전: 9→1)"
    );
    assert_eq!(fin_para.orphan_field_ends.len(), 1, "hp:fieldEnd 재방출");
}
