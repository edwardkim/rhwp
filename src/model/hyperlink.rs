//! HWP/HWPX 하이퍼링크 Command의 공통 주소 인코딩 (#6963).

use crate::error::HwpError;

/// 첫 번째 비이스케이프 세미콜론 앞의 주소를 읽는다.
/// 기존 문서의 mail/file/bookmark도 조회할 수 있도록 읽기에는 scheme 제한을 두지 않는다.
pub fn command_uri(command: &str) -> String {
    let mut out = String::new();
    let mut chars = command.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            }
            ';' => break,
            _ => out.push(c),
        }
    }
    out.trim().to_string()
}

/// 웹 주소를 한컴의 URL 형식 Command로 만든다. 입력을 자동 정규화하지 않는다.
/// 이 검사는 scheme/authority/공백 계약이며 DNS 조회나 접속 가능성 검사가 아니다.
pub fn web_command(uri: &str) -> Result<String, HwpError> {
    let invalid = || HwpError::InvalidField("공백 없는 HTTP/HTTPS 절대 주소가 필요합니다".into());
    let (scheme, rest) = uri.split_once("://").ok_or_else(invalid)?;
    if !(scheme.eq_ignore_ascii_case("http") || scheme.eq_ignore_ascii_case("https"))
        || uri
            .chars()
            .any(|c| c.is_whitespace() || c.is_control() || c == '\\')
    {
        return Err(invalid());
    }
    let authority = rest.split(['/', '?', '#']).next().unwrap_or("");
    if authority.is_empty() || authority.starts_with(':') || authority.contains('@') {
        return Err(invalid());
    }
    let mut command = String::with_capacity(uri.len() + 16);
    for c in uri.chars() {
        if matches!(c, '\\' | ':' | ';' | '#') {
            command.push('\\');
        }
        command.push(c);
    }
    command.push_str(";1;0;0;");
    checked_length(command)
}

/// 문서 내부 책갈피 링크의 한컴 Command(`?<이름>;0;0;0;`)를 만든다.
/// 한컴 실측(HWP `hwpctl_Action_Table__v1.1.hwp`, HWPX `TargetType=BOOKMARK`)처럼 `\ : ;`만 escape한다.
pub fn bookmark_command(name: &str) -> Result<String, HwpError> {
    if name.trim().is_empty() || name.chars().any(char::is_control) {
        return Err(HwpError::InvalidField(
            "책갈피 이름이 비어 있거나 제어 문자를 포함합니다".into(),
        ));
    }
    let mut command = String::with_capacity(name.len() + 8);
    command.push('?');
    for c in name.chars() {
        if matches!(c, '\\' | ':' | ';') {
            command.push('\\');
        }
        command.push(c);
    }
    command.push_str(";0;0;0;");
    checked_length(command)
}

/// Command가 문서 내부 책갈피 링크이면 책갈피 이름을 돌려준다.
pub fn command_bookmark(command: &str) -> Option<String> {
    command_uri(command)
        .strip_prefix('?')
        .filter(|name| !name.is_empty())
        .map(str::to_string)
}

fn checked_length(command: String) -> Result<String, HwpError> {
    // HWP5 CTRL_HEADER의 command 길이는 u16 UTF-16 code unit 수다.
    if command.encode_utf16().count() > u16::MAX as usize {
        return Err(HwpError::InvalidField(
            "하이퍼링크 주소가 너무 깁니다".into(),
        ));
    }
    Ok(command)
}
