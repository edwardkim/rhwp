//! Issue #6771 — 셀·글상자 **안**에 있는 표를 지운다.
//!
//! 공공기관 배포 양식은 작성 안내문을 셀 안 1×1 표(점선 상자)에 담고, 본문에 "안내 박스는
//! 반드시 삭제 후 제출"이라고 적는다. 글자는 `delete_text_in_cell_by_path` 로 지울 수 있었지만
//! **그릇을 지울 길이 없었다**: `delete_control_at` 은 본문 리스트만 다루고
//! (`{"ok":false,"reason":"본문 밖 컨트롤은 아직 다루지 않는다"}`),
//! `delete_table_control` 은 `(구역, 문단, 컨트롤)` 셋만 받아 셀 안을 짚지 못한다.
//!
//! 그래서 안내문을 지운 제출본에 **빈 점선 상자**가 남았다. 이 테스트는 그 자리를 고정한다.

use rhwp::document_core::DocumentCore;

/// 셀 안에 표가 든 공개 샘플 — 섹션0 문단 20, 셀 0의 문단 2가 표를 품는다.
const SAMPLE: &str = "samples/2022년 국립국어원 업무계획.hwp";
const PARENT_PARA: usize = 20;

fn load() -> DocumentCore {
    let bytes = std::fs::read(SAMPLE).expect("read sample");
    DocumentCore::from_bytes(&bytes).expect("parse sample")
}

/// 셀 안 표가 든 호스트 문단 경로.
const HOST_PATH: &str = r#"[{"controlIndex":0,"cellIndex":0,"cellParaIndex":2}]"#;

#[test]
fn deletes_table_inside_cell() {
    let mut doc = load();

    let deleted = doc.delete_cell_table_control_by_path_native(0, PARENT_PARA, HOST_PATH, 0);
    assert!(deleted.is_ok(), "셀 안 표 삭제 실패: {:?}", deleted.err());
    assert_eq!(deleted.unwrap(), "{\"ok\":true}");

    // 지운 뒤 같은 자리를 다시 지우려 하면 **없다**고 답해야 한다 — 삭제가 실제로 일어난 증거다.
    let again = doc.delete_cell_table_control_by_path_native(0, PARENT_PARA, HOST_PATH, 0);
    assert!(again.is_err(), "표가 남아 있다: {:?}", again.ok());
}

/// 🔴 종류를 지목해서 지운다 — 그림 자리에 표 삭제를 부르면 거절해야 한다(반대도 같다).
#[test]
fn rejects_when_control_is_not_a_table() {
    let mut doc = load();

    // 같은 자리를 그림으로 지목하면 거절한다(그 컨트롤은 표다).
    let wrong = doc.delete_cell_picture_control_by_path_native(0, PARENT_PARA, HOST_PATH, 0);
    assert!(wrong.is_err(), "그림이 아닌데 지워졌다");
    assert!(
        format!("{:?}", wrong.err()).contains("그림이 아닙니다"),
        "오류 문구가 종류를 말하지 않는다",
    );
}

/// 범위 밖 컨트롤 번호는 조용히 성공하지 않는다.
#[test]
fn rejects_out_of_range_control_index() {
    let mut doc = load();
    assert!(doc
        .delete_cell_table_control_by_path_native(0, PARENT_PARA, HOST_PATH, 99)
        .is_err());
}
