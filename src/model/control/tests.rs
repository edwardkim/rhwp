use super::*;

#[test]
fn test_control_variants() {
    let ctrl = Control::Bookmark(Bookmark {
        name: "test".to_string(),
    });
    match ctrl {
        Control::Bookmark(bm) => assert_eq!(bm.name, "test"),
        _ => panic!("Expected Bookmark"),
    }
}

#[test]
fn test_field_type_default() {
    assert_eq!(FieldType::default(), FieldType::Unknown);
}

// ---------- #1434: 누름틀 command 한컴 포맷 정합 ----------

/// 한컴 정답지(field-01/form-01)의 command 문자열과 바이트 동형이어야 한다.
#[test]
fn task1434_clickhere_command_hancom_format() {
    // 한컴 원본: `Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  `
    assert_eq!(
        Field::build_clickhere_command("여기에 입력", ""),
        "Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  "
    );
    // 한컴 원본: `Clickhere:set:47:Direction:wstring:5:제목 입력 HelpState:wstring:0:  `
    assert_eq!(
        Field::build_clickhere_command("제목 입력", ""),
        "Clickhere:set:47:Direction:wstring:5:제목 입력 HelpState:wstring:0:  "
    );
}

/// command 에 Name 키가 들어가면 안 된다 (이름은 CTRL_DATA 전담). 한컴이 Name 키를
/// 만나면 Direction 범위를 잘못 잘라 안내문 바인딩 실패 (#1434 회귀 가드).
#[test]
fn task1434_command_has_no_name_key() {
    let cmd = Field::build_clickhere_command("여기에 입력", "");
    assert!(
        !cmd.contains("Name:"),
        "command 에 Name 키가 있으면 한컴 안내문 바인딩 실패: {cmd}"
    );
}

/// 생성한 command 에서 guide_text/memo_text 가 정확히 재추출되어야 한다 (왕복 정합).
#[test]
fn task1434_command_guide_memo_roundtrip() {
    let field = Field {
        field_type: FieldType::ClickHere,
        command: Field::build_clickhere_command("여기에 입력", "도움말"),
        ..Default::default()
    };
    assert_eq!(field.guide_text(), Some("여기에 입력"));
    assert_eq!(field.memo_text(), Some("도움말"));
}

/// set 길이는 inner 글자수 − 1 (마지막 trailing 공백 제외) 규칙을 따른다.
#[test]
fn task1434_set_length_excludes_trailing_space() {
    let cmd = Field::build_clickhere_command("a", "");
    // inner = "Direction:wstring:1:a HelpState:wstring:0:  " (trailing 2개)
    let inner = cmd.strip_prefix("Clickhere:set:").unwrap();
    let (set_str, body) = inner.split_once(':').unwrap();
    let set_len: usize = set_str.parse().unwrap();
    assert_eq!(
        set_len,
        body.chars().count() - 1,
        "set 길이 = inner 글자수 − 1 (trailing 공백 제외): {cmd}"
    );
}

/// §4.5: wstring {n}·set {N} 은 UTF-16 code unit(WCHAR) 수여야 한다.
/// BMP 밖 문자(U+20000)는 2 WCHAR 이므로 chars().count() 로는 1개 적게 세어진다.
#[test]
fn clickhere_wstring_len_counts_utf16_wchars() {
    // guide "a𠀀b": a(1) + U+20000(2 WCHAR) + b(1) = 4 WCHAR (chars().count()=3)
    let cmd = Field::build_clickhere_command("a\u{20000}b", "");
    assert!(
        cmd.contains("Direction:wstring:4:a\u{20000}b "),
        "guide {{n}} 은 UTF-16 code unit 수(4)여야 한다(chars().count()=3 이면 RED): {cmd}"
    );
    // set 길이도 WCHAR 기준 (inner 의 UTF-16 수 − 1).
    let body = cmd.strip_prefix("Clickhere:set:").unwrap();
    let (set_str, inner) = body.split_once(':').unwrap();
    let set_len: usize = set_str.parse().unwrap();
    assert_eq!(
        set_len,
        inner.encode_utf16().count() - 1,
        "set {{N}} 은 WCHAR 수: {cmd}"
    );
}

#[test]
fn test_hyperlink() {
    let link = Hyperlink {
        url: "https://example.com".to_string(),
        text: "Example".to_string(),
    };
    assert!(!link.url.is_empty());
}
