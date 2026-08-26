//! [#5916] 되살린 HWP3 기호가 한글 조판에서 쪽을 넘기던 근인 — 셀 안여백 사상 —
//! 이 제거되어 실서식 문서의 표 안여백이 원값 140 균일로 보존되는지 검증.
//!
//! 05434(양양군 차량운행일지, 2쪽 서식)는 #5860 이 기호를 되살린 뒤 한글이
//! 저장본을 3쪽으로 조판했다. 레코드 이등분(정답 = 한글 2024 SaveAs HWP5)으로
//! HWPTAG_TABLE 의 기본 셀 안여백 (510,510,141,141)→(140,140,140,140) 복원만이
//! 쪽수를 2쪽으로 되돌림을 실측 확정했다 — #5557 의 한글 2022 사상 규칙이
//! 현행 오라클(한글 2024)과 어긋나 표를 부풀린 것이다. 한글 2024 는 HWP5·HWPX
//! SaveAs 모두 140 균일을 유지한다.

use rhwp::model::control::Control;

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue5916/05434_vehicle_log_form.hwp")
}

fn table_paddings(doc: &rhwp::model::document::Document) -> Vec<(i16, i16, i16, i16)> {
    let mut out = Vec::new();
    for section in &doc.sections {
        for para in &section.paragraphs {
            for control in &para.controls {
                if let Control::Table(table) = control {
                    out.push((
                        table.padding.left,
                        table.padding.right,
                        table.padding.top,
                        table.padding.bottom,
                    ));
                }
            }
        }
    }
    out
}

// HWP3 원본 파싱: 표 기본 안여백이 원값(35 hunit ×4 = 140 균일)으로 보존된다.
#[test]
fn hwp3_real_form_table_padding_preserved_verbatim() {
    let bytes = std::fs::read(sample_path()).expect("샘플 읽기 실패");
    let doc = rhwp::parser::hwp3::parse_hwp3(&bytes).expect("HWP3 파싱 실패");
    let paddings = table_paddings(&doc);
    assert!(!paddings.is_empty(), "표가 없음");
    for p in &paddings {
        assert_eq!(
            *p,
            (140, 140, 140, 140),
            "표 안여백은 원값 140 균일이어야 함"
        );
    }
}

// h2h 왕복: HWP5 로 저장한 뒤 재파싱해도 HWPTAG_TABLE 안여백이 140 균일이다.
// (#5916 의 3쪽 넘침은 이 값이 510 으로 저장돼 한글이 표를 부풀린 것이었다.)
#[test]
fn hwp3_to_hwp5_roundtrip_keeps_table_padding() {
    let bytes = std::fs::read(sample_path()).expect("샘플 읽기 실패");
    let doc = rhwp::parser::hwp3::parse_hwp3(&bytes).expect("HWP3 파싱 실패");
    let saved = rhwp::serializer::serialize_document(&doc).expect("HWP5 직렬화 실패");
    let reparsed = rhwp::parser::parse_document(&saved).expect("재파싱 실패");
    let paddings = table_paddings(&reparsed);
    assert!(!paddings.is_empty(), "왕복 후 표가 없음");
    for p in &paddings {
        assert_eq!(
            *p,
            (140, 140, 140, 140),
            "왕복 후 표 안여백은 140 균일이어야 함"
        );
    }
}
