//! [Issue #4397] HWPX Ruby(덧말) 컨트롤이 HWPX→HWP→HWPX 왕복에서 통째로
//! 사라지던 계약을 고정한다.
//!
//! 종전: HWP5 저장은 최소 CTRL_HEADER(짝 맞춤, #4677)만 내고 파서에는 'tdut'
//! arm 이 없어 `Control::Unknown` 으로 떨어졌다 — 본문/덧말 텍스트와 스타일이
//! 경고 없이 소실. 이 PR 은 스펙 표 151 payload(mainText·subText·위치·크기비율·
//! 옵션·스타일·정렬)를 양방향으로 잇는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::parse_document;

const SAMPLE: &str = "samples/hwpx/opengov/36389301_결재문서본문_직장훈련계획_덧말.hwpx";

fn read_sample() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"))
}

fn first_ruby(doc: &rhwp::model::document::Document) -> Option<&rhwp::model::control::Ruby> {
    doc.sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .find_map(|p| {
            p.controls.iter().find_map(|c| match c {
                Control::Ruby(r) => Some(r),
                _ => None,
            })
        })
}

#[test]
fn issue4397_ruby_survives_hwp5_roundtrip() {
    let original = parse_document(&read_sample()).expect("parse hwpx");
    let ruby0 = first_ruby(&original)
        .expect("샘플 전제: 덧말 컨트롤")
        .clone();
    assert!(!ruby0.main_text.is_empty(), "샘플 전제: 본문 텍스트");
    assert!(!ruby0.ruby_text.is_empty(), "샘플 전제: 덧말 텍스트");

    // HWPX → HWP5
    let mut core = DocumentCore::from_bytes(&read_sample()).expect("open hwpx");
    let hwp = core.export_hwp_with_adapter().expect("convert to hwp");
    let mid = parse_document(&hwp).expect("parse hwp");
    let ruby1 = first_ruby(&mid).expect("HWP5 재파싱이 덧말을 복원해야 한다 (#4397)");
    assert_eq!(ruby1.main_text, ruby0.main_text, "mainText 보존");
    assert_eq!(ruby1.ruby_text, ruby0.ruby_text, "subText 보존");
    assert_eq!(
        (
            ruby1.pos_type,
            ruby1.align,
            ruby1.sz_ratio,
            ruby1.option,
            ruby1.style_id_ref
        ),
        (
            ruby0.pos_type,
            ruby0.align,
            ruby0.sz_ratio,
            ruby0.option,
            ruby0.style_id_ref
        ),
        "속성 보존"
    );

    // HWP5 → HWPX (왕복 완주)
    let core2 = DocumentCore::from_bytes(&hwp).expect("open hwp");
    let hwpx2 = core2.export_hwpx_native().expect("export hwpx");
    let fin = parse_document(&hwpx2).expect("parse final hwpx");
    let ruby2 = first_ruby(&fin).expect("왕복 후에도 덧말이 있어야 한다 (#4397)");
    assert_eq!(ruby2.main_text, ruby0.main_text);
    assert_eq!(ruby2.ruby_text, ruby0.ruby_text);
}
