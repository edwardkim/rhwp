//! Issue #1434: 누름틀 안내문(Direction)이 한컴 에디터에서 바인딩되지 않는 결함의
//! 회귀 가드. 누름틀 command 를 한컴 정답지 포맷으로 저장하고, 저장→재파싱 시
//! guide/memo/이름이 보존되며 command 에 Name 키가 끼지 않는지 검증한다.
//!
//! 근본 원인(해소): build_clickhere_command 가 ① Name 키를 command 에 포함
//! ② trailing 공백 1개 ③ set 길이=inner 전체 로 어긋나 한컴이 Direction 범위를
//! 잘못 잘라 안내문 바인딩 실패. → Name 제거 + 공백 2개 + set=inner−1 로 정정.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::FieldType;

fn make_doc_with_clickhere(guide: &str, memo: &str, name: &str) -> DocumentCore {
    let mut core = DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("create blank document");
    core.insert_text_native(0, 0, 0, "ABC")
        .expect("insert base text");
    core.insert_click_here_field_at(0, 0, 1, guide, memo, name, true)
        .expect("insert clickhere field");
    core
}

/// 삽입 직후 command 가 한컴 포맷이며 Name 키가 없어야 한다.
#[test]
fn clickhere_command_is_hancom_format_without_name_key() {
    let core = make_doc_with_clickhere("여기에 입력", "", "회사명");
    let fields = core.collect_all_fields();
    let f = fields
        .iter()
        .find(|f| f.field.field_type == FieldType::ClickHere)
        .expect("clickhere field present");

    // 한컴 정답지 동형: set:48, 공백 2개, Name 키 부재.
    assert_eq!(
        f.field.command, "Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  ",
        "command 가 한컴 포맷이어야 함"
    );
    assert!(
        !f.field.command.contains("Name:"),
        "command 에 Name 키가 끼면 한컴 안내문 바인딩 실패: {}",
        f.field.command
    );
}

/// HWP 저장 → 재파싱 후 안내문/메모/이름이 보존되어야 한다 (round-trip).
#[test]
fn clickhere_guide_memo_name_survive_hwp_roundtrip() {
    let core = make_doc_with_clickhere("여기에 입력", "도움말", "회사명");
    let saved = core.export_hwp_native().expect("export hwp");
    // 한컴 편집기 바인딩 판정용 샘플 산출 (RHWP_ISSUE1434_OUT 지정 시).
    if let Ok(output_path) = std::env::var("RHWP_ISSUE1434_OUT") {
        if let Some(parent) = std::path::Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent).expect("create output parent");
        }
        std::fs::write(&output_path, &saved).expect("write issue #1434 output hwp");
    }
    let reparsed = DocumentCore::from_bytes(&saved).expect("reparse exported hwp");

    let fields = reparsed.collect_all_fields();
    let f = fields
        .iter()
        .find(|f| f.field.field_type == FieldType::ClickHere)
        .expect("clickhere field after roundtrip");

    assert_eq!(
        f.field.guide_text(),
        Some("여기에 입력"),
        "안내문(Direction)이 저장→재파싱 후 보존돼야 함"
    );
    assert_eq!(
        f.field.memo_text(),
        Some("도움말"),
        "메모(HelpState)가 보존돼야 함"
    );
    // 이름은 CTRL_DATA(0x57)에 저장되며 field_name 으로 복원된다.
    assert_eq!(
        f.field.field_name(),
        Some("회사명"),
        "필드 이름이 CTRL_DATA 로 보존돼야 함"
    );
    // 재파싱한 command 에도 Name 키가 없어야 한다.
    assert!(
        !f.field.command.contains("Name:"),
        "재파싱 command 에 Name 키 부재: {}",
        f.field.command
    );
}
