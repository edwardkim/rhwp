//! Issue #2751 회귀 가드 — HML 직렬화기가 무검증 `row_count` 만큼 `<ROW>` 를
//! 방출하던 문제. `RowCount` 를 셀이 실제로 자리한 행 수보다 훨씬 크게 조작한
//! 입력에서, 수정 전에는 내용이 없는 `<ROW></ROW>` 가 대량으로 찍혔다.

use rhwp::document_core::DocumentCore;

/// `samples/hml/formatting_table.hml` 의 `RowCount="1"` 을 `"65535"` 로만 치환한
/// 최소 재현 입력. `ColCount="1"` 은 그대로 두어 #2731 의 그리드 상한(4,000,000)이
/// 전혀 발동하지 않는 영역만 겨냥한다 (row_count × col_count = 65,535).
fn malicious_row_count_bytes() -> Vec<u8> {
    let original = include_bytes!("../samples/hml/formatting_table.hml");
    let text = String::from_utf8(original.to_vec()).expect("샘플은 UTF-8");
    assert!(
        text.contains(r#"RowCount="1""#),
        "샘플의 RowCount 표기가 바뀌었으면 재현 치환도 갱신해야 함"
    );
    text.replacen(r#"RowCount="1""#, r#"RowCount="65535""#, 1)
        .into_bytes()
}

fn count_occurrences(haystack: &str, needle: &str) -> usize {
    haystack.matches(needle).count()
}

#[test]
fn malicious_row_count_does_not_inflate_output_with_empty_rows() {
    let bytes = malicious_row_count_bytes();
    let core = DocumentCore::from_bytes(&bytes).expect("조작된 RowCount 여도 문서는 열려야 함");

    let exported = core
        .export_hml_native()
        .expect("export 는 성공해야 함(유한 시간 내)");
    let xml = String::from_utf8(exported).expect("직렬화 산출물은 UTF-8");

    let row_open = count_occurrences(&xml, "<ROW>") + count_occurrences(&xml, "<ROW ");
    let empty_row = count_occurrences(&xml, "<ROW></ROW>");

    // 원본 표는 셀 1개가 row=0 에 있을 뿐이므로, RowCount 가 65535 로 조작돼도
    // 실제로 방출돼야 할 <ROW> 는 1개, 빈 <ROW></ROW> 는 0개다.
    assert_eq!(
        row_open, 1,
        "row_count=65535 라도 실제 셀이 자리한 행만큼만 <ROW> 를 방출해야 함 (실제 {xml})"
    );
    assert_eq!(
        empty_row, 0,
        "내용 0인 <ROW></ROW> 를 방출하면 안 됨 (실제 {xml})"
    );
}

#[test]
fn malicious_row_count_reparse_preserves_table_ir() {
    // 6.1의 IR 동등성 확인: 조작된 RowCount 입력을 export 한 산출물을 다시
    // 파싱해도 셀 개수·좌표가 원본 재파싱 결과와 같아야 한다(빈 <ROW> 제거가
    // 무손실임의 증거).
    let bytes = malicious_row_count_bytes();
    let core = DocumentCore::from_bytes(&bytes).expect("조작된 RowCount 여도 문서는 열려야 함");
    let cell_count_before: usize = core
        .document()
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .filter_map(|c| match c {
            rhwp::model::control::Control::Table(t) => Some(t.cells.len()),
            _ => None,
        })
        .sum();
    assert_eq!(cell_count_before, 1, "재현 입력의 표는 셀 1개");

    let exported = core.export_hml_native().expect("export 성공");
    let reparsed = DocumentCore::from_bytes(&exported).expect("재직렬화 산출물도 다시 열려야 함");
    let cell_count_after: usize = reparsed
        .document()
        .sections
        .iter()
        .flat_map(|s| &s.paragraphs)
        .flat_map(|p| &p.controls)
        .filter_map(|c| match c {
            rhwp::model::control::Control::Table(t) => Some(t.cells.len()),
            _ => None,
        })
        .sum();

    assert_eq!(
        cell_count_after, cell_count_before,
        "빈 <ROW> 제거는 표 IR(셀 개수)을 바꾸면 안 됨"
    );
}
