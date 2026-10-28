//! CLI 전역 인증 옵션과 문서 로딩 경계.
//!
//! `export-pdf`와 `export-png`를 포함한 CLI 출력 어댑터가 이 모듈의 로더를
//! 공유한다. 따라서 이 파일은 Render Diff와 Native Skia가 직접 추적하는
//! 문서 입력 경계다.

use std::io::Read as _;

use crate::{EXIT_RUNTIME, EXIT_USAGE};

thread_local! {
    static CLI_PASSWORD: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    static CLI_OUTPUT_PASSWORD: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

pub(crate) fn set_cli_password(pw: Option<String>) {
    CLI_PASSWORD.with(|c| *c.borrow_mut() = pw);
}

pub(crate) fn cli_password() -> Option<String> {
    CLI_PASSWORD.with(|c| c.borrow().clone())
}

pub(crate) fn set_cli_output_password(pw: Option<String>) {
    CLI_OUTPUT_PASSWORD.with(|c| *c.borrow_mut() = pw);
}

pub(crate) fn cli_output_password() -> Option<String> {
    CLI_OUTPUT_PASSWORD.with(|c| c.borrow().clone())
}

/// 문서 로드 에러 — 비밀번호 필요/불일치/기타를 구분해 종료 코드를 다르게 매핑.
pub(crate) enum LoadError {
    /// 암호 문서인데 비밀번호가 제공되지 않음 (EXIT_USAGE)
    NeedPassword,
    /// 비밀번호 불일치 (EXIT_RUNTIME)
    WrongPassword,
    /// 그 외 파싱 오류 (EXIT_RUNTIME)
    Other(String),
}

impl LoadError {
    /// stderr 에 메시지를 출력하고 매핑된 종료 코드를 반환한다.
    pub(crate) fn report(self) -> i32 {
        match self {
            LoadError::NeedPassword => {
                eprintln!("오류: 비밀번호가 필요한 암호 문서입니다 (--password <pw> 로 전달).");
                EXIT_USAGE
            }
            LoadError::WrongPassword => {
                eprintln!("오류: 비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다.");
                EXIT_RUNTIME
            }
            LoadError::Other(msg) => {
                eprintln!("오류: 문서 파싱 실패 - {}", msg);
                EXIT_RUNTIME
            }
        }
    }
}

/// HwpError Display 메시지에서 비밀번호 관련 에러를 분류한다.
/// CryptoError::WrongPassword → "...비밀번호가 일치하지 않...",
/// ParseError::EncryptedDocument → "...비밀번호가 필요한 암호 문서..." 가
/// HwpError::InvalidFile 로 래핑돼 전해지므로 부분문자열로 판별한다.
pub(crate) fn classify_hwp_error(msg: &str) -> LoadError {
    if msg.contains("비밀번호가 일치하지 않") {
        LoadError::WrongPassword
    } else if msg.contains("비밀번호가 필요한 암호 문서") {
        LoadError::NeedPassword
    } else {
        LoadError::Other(msg.to_string())
    }
}

/// HwpDocument 로드. 전역 비밀번호가 설정돼 있으면 비밀번호 경로로 연다.
pub(crate) fn load_document(data: &[u8]) -> Result<rhwp::wasm_api::HwpDocument, LoadError> {
    let result = match cli_password() {
        Some(pw) => rhwp::wasm_api::HwpDocument::from_bytes_with_password(data, pw.as_bytes()),
        None => rhwp::wasm_api::HwpDocument::from_bytes(data),
    };
    result.map_err(|e| classify_hwp_error(&e.to_string()))
}

/// DocumentCore 로드 (export-pdf/export-hml 등). 동일 분기.
pub(crate) fn load_document_core(
    data: &[u8],
) -> Result<rhwp::document_core::DocumentCore, LoadError> {
    let result = match cli_password() {
        Some(pw) => {
            rhwp::document_core::DocumentCore::from_bytes_with_password(data, pw.as_bytes())
        }
        None => rhwp::document_core::DocumentCore::from_bytes(data),
    };
    result.map_err(|e| classify_hwp_error(&e.to_string()))
}

/// `batch` 는 stdin 전체를 파일 경로 목록으로 소비한다. 전역 인증 옵션 중 stdin
/// 변형은 그 목록과 같은 바이트 스트림을 두 번 읽으려 하고, 리터럴 변형도 worker
/// thread-local 인증 상태로 전달되지 않는다. 따라서 암호화 batch 를 정식으로 설계하기
/// 전에는 네 옵션을 모두 호출 경계에서 거부한다.
///
/// 명령 위치 앞의 전역 인증 옵션만 건너뛰어 `batch` 여부를 판정한다. 단순히 모든 인자에서
/// `batch` 문자열을 찾으면 `search --query batch` 같은 정상 호출을 잘못 막게 된다.
pub(crate) fn is_batch_invocation(args: &[String]) -> bool {
    let mut i = 1; // args[0] 은 프로그램 경로
    while let Some(arg) = args.get(i) {
        match arg.as_str() {
            "--password" | "--output-password" => i += 2,
            "--password-stdin" | "--output-password-stdin" => i += 1,
            _ => return arg == "batch",
        }
    }
    false
}

/// `batch` 명령이 실제로 보이면, 그 뒤·앞 어느 위치의 전역 인증 옵션도 거부한다.
pub(crate) fn has_global_auth_option(args: &[String]) -> bool {
    args.iter().skip(1).any(|arg| {
        matches!(
            arg.as_str(),
            "--password" | "--password-stdin" | "--output-password" | "--output-password-stdin"
        )
    })
}

/// Windows PowerShell/.NET이 UTF-8 표준입력의 첫 바이트에 붙일 수 있는 BOM은
/// 비밀번호 본문이 아니라 인코딩 표식이다. 첫 줄 암호에 섞이면 정상 비밀번호도
/// 오입력으로 판정되므로, stdin 전체의 맨 앞에서만 제거한다.
pub(crate) fn strip_utf8_bom(input: &str) -> &str {
    input.strip_prefix('\u{feff}').unwrap_or(input)
}

/// args 전체를 스캔해 입력·출력 인증 옵션을 떼어낸다.
///
/// 뽑아낸 입력 암호와 출력 암호는 이 함수 안에서 thread-local 상태로 소비하고,
/// 반환값에는 해당 토큰이 제거된 args 만 담는다. 두 stdin 옵션을 같이 사용하면
/// stdin 첫 줄은 입력, 둘째 줄은 출력 암호로 고정한다.
///
/// 이름과 반환 형태가 "정제된 args" 인 것은 의도적이다. 비밀번호를 반환값(과거의
/// `(args, password)` 튜플)에 싣거나 함수 이름에 `password` 를 두면 CodeQL
/// `rust/cleartext-logging` 이 이 호출의 결과 전체를 민감 데이터로 보고, 비밀번호
/// 토큰이 이미 제거된 args 를 쓰는 오류·진단 출력까지 sink 로 분류한다
/// (PR #3405 검토에서 41건 과탐지로 확인, PR #3644 에서 alert #119 로 재발).
/// 반환 경로에 비밀번호가 남지 않으므로 이 분류는 실제 유출 경로가 아니다.
pub(crate) fn strip_global_auth_options(mut args: Vec<String>) -> Result<Vec<String>, i32> {
    let mut password: Option<String> = None;
    let mut output_password: Option<String> = None;
    let mut password_stdin = false;
    let mut output_password_stdin = false;
    let mut i = 1; // args[0] 은 프로그램 경로
    while i < args.len() {
        match args[i].as_str() {
            "--password" => {
                if password.is_some() {
                    eprintln!("오류: 비밀번호 옵션은 한 번만 지정할 수 있습니다.");
                    return Err(EXIT_USAGE);
                }
                if i + 1 >= args.len() {
                    eprintln!("오류: --password 뒤에 비밀번호가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
                password = Some(args[i + 1].clone());
                args.drain(i..=i + 1);
            }
            "--password-stdin" => {
                if password.is_some() || password_stdin {
                    eprintln!("오류: 비밀번호 옵션은 한 번만 지정할 수 있습니다.");
                    return Err(EXIT_USAGE);
                }
                password_stdin = true;
                args.remove(i);
            }
            "--output-password" => {
                if output_password.is_some() || output_password_stdin {
                    eprintln!("오류: 출력 비밀번호 옵션은 한 번만 지정할 수 있습니다.");
                    return Err(EXIT_USAGE);
                }
                if i + 1 >= args.len() {
                    eprintln!("오류: --output-password 뒤에 비밀번호가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
                output_password = Some(args[i + 1].clone());
                args.drain(i..=i + 1);
            }
            "--output-password-stdin" => {
                if output_password.is_some() || output_password_stdin {
                    eprintln!("오류: 출력 비밀번호 옵션은 한 번만 지정할 수 있습니다.");
                    return Err(EXIT_USAGE);
                }
                output_password_stdin = true;
                args.remove(i);
            }
            _ => i += 1,
        }
    }

    if password_stdin || output_password_stdin {
        let mut stdin = String::new();
        if let Err(error) = std::io::stdin().read_to_string(&mut stdin) {
            eprintln!("오류: 표준 입력에서 비밀번호 읽기 실패 - {}", error);
            return Err(EXIT_RUNTIME);
        }
        let stdin = strip_utf8_bom(&stdin);
        let mut lines = stdin.lines();
        if password_stdin {
            password = Some(lines.next().unwrap_or_default().to_string());
        }
        if output_password_stdin {
            output_password = Some(lines.next().unwrap_or_default().to_string());
        }
    }
    if let Some(value) = output_password.as_deref() {
        if value.is_empty() || value.len() > 4096 || value.contains(['\r', '\n']) {
            eprintln!("오류: 출력 비밀번호는 빈 값·줄바꿈 없이 UTF-8 4096바이트 이하여야 합니다.");
            return Err(EXIT_USAGE);
        }
    }
    set_cli_password(password);
    set_cli_output_password(output_password);
    Ok(args)
}
