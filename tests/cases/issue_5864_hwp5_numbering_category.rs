//! [#5864] HWP5 개체 번호 범주(캡션 번호·상호참조 계열)를 파서가 읽는다.
//!
//! HWP5 CTRL_HEADER 공통속성 attr bits 26-28 은 3비트 열거다 — 07276 실측
//! (한글 2024 SaveAs HWPX 정답과 대조): 표 155/155(0=NONE 23·2=TABLE 126·
//! 1=PICTURE 6), 수식 8/8(=3), gso 분포 정합(PICTURE 28·TABLE 1 정확 일치).
//! 종전엔 안 읽어 h2x 산출의 `numberingType` 이 전량 NONE 이 됐고, 한글이
//! 상호참조 순번을 전부 1 로 재계산했다(07276 상호참조 153중 101 오염,
//! 07378 `<표 24>`→`<표 1>`).
//!
//! 왕복 고정: HWPX(numberingType 보유) → HWP5 직렬화(비트 기록) → HWP5 재파싱
//! (이 수정이 읽음) 에서 범주가 살아남는다. 수정 전에는 재파싱 단계에서 전부
//! NONE 으로 무너졌다.

use rhwp::model::control::Control;
use rhwp::model::shape::ObjectNumberingType;

const SAMPLE: &str = "samples/hwpx/143E433F503322BD33.hwpx";

fn numbering_types(doc: &rhwp::model::document::Document) -> Vec<(String, ObjectNumberingType)> {
    let mut out = Vec::new();
    for section in &doc.sections {
        for para in &section.paragraphs {
            for ctrl in &para.controls {
                match ctrl {
                    Control::Table(t) => out.push(("tbl".into(), t.common.numbering_type)),
                    Control::Picture(p) => out.push(("pic".into(), p.common.numbering_type)),
                    _ => {}
                }
            }
        }
    }
    out
}

#[test]
fn issue_5864_numbering_category_survives_hwp5_roundtrip() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).expect("read sample");
    let original = rhwp::parse_document(&bytes).expect("parse hwpx");
    let expected = numbering_types(&original);
    assert!(
        expected
            .iter()
            .any(|(kind, nt)| kind == "tbl" && *nt == ObjectNumberingType::Table),
        "전제: 표본에 numberingType=TABLE 표가 있어야 한다: {expected:?}"
    );

    let hwp5 = rhwp::serializer::serialize_document(&original).expect("serialize hwp5");
    let reparsed = rhwp::parse_document(&hwp5).expect("reparse hwp5");
    let actual = numbering_types(&reparsed);

    let expected_tables: Vec<_> = expected.iter().filter(|(k, _)| k == "tbl").collect();
    let actual_tables: Vec<_> = actual.iter().filter(|(k, _)| k == "tbl").collect();
    assert_eq!(
        expected_tables, actual_tables,
        "#5864: 표 번호 범주가 HWP5 왕복에서 살아남아야 한다 — 수정 전엔 재파싱이 전량 NONE"
    );
}
