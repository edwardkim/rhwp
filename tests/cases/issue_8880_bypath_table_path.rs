//! 고정형·관계형: ByPath 표 조회 API의 경로 입력 계약을 검사한다. 고정형은 만든 표의 행·열·칸 수,
//! 관계형은 마지막 단계 필드를 생략한 경로와 모두 적은 경로의 결과가 같은지다.
//! 주석 갱신: 2026-11-11 21:55 KST (+09:00). 새 파일이라 직전 파일 변경 커밋은 없다.
//! 시각 측정: 비해당 — `path_json` 파싱 계약만 바꾸며 조판·렌더링 결과를 바꾸지 않는다.
//! 입력: 문서 파일 없이 `create_blank_document_native`로 만든 빈 문서에 표를 넣어 재현한다.
//!
//! #8880: 표 자체를 가리키는 `getTableDimensionsByPath`·`getTableCellBboxesByPath`는 마지막
//! 단계의 `cellIndex`·`cellParaIndex`를 쓰지 않는데도 필수로 요구했다. 마지막 단계는
//! `controlIndex`만으로 표를 가리킬 수 있어야 하고, 바깥 칸을 지나는 앞 단계는 여전히 세 필드가
//! 모두 필요하다.

use rhwp::document_core::DocumentCore;
use serde_json::Value;

/// 빈 문서 끝 문단에 2×3 표를 넣고 (표 문단, 표 컨트롤 번호)를 돌려준다.
fn doc_with_table() -> (DocumentCore, usize, usize) {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native().unwrap();
    let last = core.document().sections[0].paragraphs.len() - 1;
    let created: Value =
        serde_json::from_str(&core.create_table_native(0, last, 0, 2, 3).unwrap()).unwrap();
    (
        core,
        created["paraIdx"].as_u64().unwrap() as usize,
        created["controlIdx"].as_u64().unwrap() as usize,
    )
}

fn dims(core: &DocumentCore, para: usize, path_json: &str) -> Value {
    let json = core
        .get_table_dimensions_by_path_native(0, para, path_json)
        .unwrap_or_else(|e| panic!("getTableDimensionsByPath({path_json}) 실패: {e}"));
    serde_json::from_str(&json).unwrap()
}

fn expect_dims(value: &Value, rows: u64, cols: u64) {
    assert_eq!(value["rowCount"].as_u64(), Some(rows), "{value}");
    assert_eq!(value["colCount"].as_u64(), Some(cols), "{value}");
    assert_eq!(value["cellCount"].as_u64(), Some(rows * cols), "{value}");
}

#[test]
fn table_path_last_step_needs_only_control_index() {
    let (core, para, ctrl) = doc_with_table();

    // 이슈 재현 경로: 수정 전에는 앞의 두 경로가 'cellIndex'/'cellParaIndex' 없음으로 실패했다.
    for path in [
        format!(r#"[{{"controlIndex":{ctrl}}}]"#),
        format!(r#"[{{"controlIndex":{ctrl},"cellIndex":0}}]"#),
        format!(r#"[{{"controlIndex":{ctrl},"cellIndex":0,"cellParaIndex":0}}]"#),
        // 마지막 단계의 칸 필드는 표를 고르는 데 쓰이지 않는다(종전 동작 유지).
        format!(r#"[{{"controlIndex":{ctrl},"cellIndex":99,"cellParaIndex":99}}]"#),
    ] {
        expect_dims(&dims(&core, para, &path), 2, 3);
    }
}

#[test]
fn table_path_still_rejects_invalid_last_step() {
    let (core, para, ctrl) = doc_with_table();

    // controlIndex 는 여전히 필수다.
    assert!(core
        .get_table_dimensions_by_path_native(0, para, r#"[{"cellIndex":0,"cellParaIndex":0}]"#)
        .is_err());
    // 생략할 수 있는 필드라도 값이 숫자가 아니면 조용히 0으로 바꾸지 않는다.
    let bad = format!(r#"[{{"controlIndex":{ctrl},"cellIndex":"x"}}]"#);
    assert!(core
        .get_table_dimensions_by_path_native(0, para, &bad)
        .is_err());
    // 마지막 단계는 표여야 한다. 빈 문서 첫 문단의 컨트롤 0은 구역 정의다.
    assert!(core
        .get_table_dimensions_by_path_native(0, 0, r#"[{"controlIndex":0}]"#)
        .is_err());
}

#[test]
fn nested_table_path_needs_full_outer_steps() {
    let (mut core, para, outer) = doc_with_table();

    // 바깥 표 칸 4(2행 2열)의 문단 0에 3×4 안쪽 표를 만든다.
    let created: Value = serde_json::from_str(
        &core
            .create_table_in_cell_by_path_native(
                0,
                para,
                &[(outer, 4, 0)],
                0,
                3,
                4,
                None,
                None,
                true,
            )
            .unwrap(),
    )
    .unwrap();
    let inner_para = created["cellParaIdx"].as_u64().unwrap() as usize;
    let inner = created["controlIdx"].as_u64().unwrap() as usize;

    let outer_step =
        format!(r#"{{"controlIndex":{outer},"cellIndex":4,"cellParaIndex":{inner_para}}}"#);

    // 안쪽 표: 마지막 단계는 controlIndex 만으로 충분하다.
    let short = format!(r#"[{outer_step},{{"controlIndex":{inner}}}]"#);
    let full =
        format!(r#"[{outer_step},{{"controlIndex":{inner},"cellIndex":0,"cellParaIndex":0}}]"#);
    expect_dims(&dims(&core, para, &short), 3, 4);
    assert_eq!(dims(&core, para, &short), dims(&core, para, &full));

    // 바깥 표 자체도 같은 규칙으로 가리킨다.
    expect_dims(
        &dims(&core, para, &format!(r#"[{{"controlIndex":{outer}}}]"#)),
        2,
        3,
    );

    // 앞 단계는 바깥 칸 문단을 지나므로 세 필드가 모두 필요하다.
    for missing in [
        format!(r#"[{{"controlIndex":{outer},"cellIndex":4}},{{"controlIndex":{inner}}}]"#),
        format!(r#"[{{"controlIndex":{outer}}},{{"controlIndex":{inner}}}]"#),
    ] {
        assert!(
            core.get_table_dimensions_by_path_native(0, para, &missing)
                .is_err(),
            "앞 단계 필드가 빠진 경로는 거부해야 한다: {missing}"
        );
    }
}

#[test]
fn cell_bboxes_accept_control_only_last_step() {
    let (core, para, ctrl) = doc_with_table();

    let short = core
        .get_table_cell_bboxes_by_path_native(0, para, &format!(r#"[{{"controlIndex":{ctrl}}}]"#))
        .expect("마지막 단계가 controlIndex 만인 경로");
    let full = core
        .get_table_cell_bboxes_by_path_native(
            0,
            para,
            &format!(r#"[{{"controlIndex":{ctrl},"cellIndex":0,"cellParaIndex":0}}]"#),
        )
        .expect("세 필드를 모두 적은 경로");

    assert_eq!(short, full);
    let cells: Vec<Value> = serde_json::from_str(&short).unwrap();
    assert_eq!(cells.len(), 6, "2×3 표의 칸 6개: {short}");
}
