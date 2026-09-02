//! 서비스 계층 오류 타입 — **자연어가 아니라 타입이 판정한다**.
//!
//! 이 모듈이 존재하는 이유는 하나다. 지금 CLI 는 문서 열기 실패를 이렇게 가른다
//! (`src/main.rs` `classify_hwp_error`).
//!
//! ```text
//! if msg.contains("비밀번호가 일치하지 않") { WrongPassword }
//! else if msg.contains("비밀번호가 필요한 암호 문서") { NeedPassword }
//! ```
//!
//! 한국어 문장 한 글자만 바뀌어도 **exit code 가 조용히 바뀐다**. 그리고 그 판정은
//! CLI 안에만 있어서 MCP 는 아예 갈래를 잃고 모든 실패를 `"{path} 파싱 실패"` 하나로
//! 뭉갠다. 소비자가 실패를 기계로 다루려면 실패에 **이름**이 있어야 한다.
//!
//! [`ServiceError`] 는 그 이름이다. 소비자는 [`ServiceError::code`] 로 안정 토큰을,
//! [`ServiceError::is_usage_error`] 로 "요청을 고치면 되는 실패인가"를 얻는다.
//! `Display` 는 사람에게 보일 한국어 문장이지만 **판정의 근거가 아니다**.

use std::path::PathBuf;

use crate::parser::crypto::CryptoError;
use crate::parser::{FileFormat, ParseError};
use crate::HwpError;

/// 서비스 계층이 돌려주는 실패의 전부.
///
/// "매치 0건", "빈 문서", "표가 없음" 같은 **판정은 여기 없다** — 그것들은 실패가
/// 아니라 `Ok` 안의 값이다([`crate::service::SearchOutcome`] 참고).
///
/// `PartialEq` 를 구현하므로 테스트와 소비자가 값 비교로 갈래를 확인할 수 있다.
/// (`Eq` 는 [`FileFormat`] 이 `Eq` 를 구현하지 않아 파생하지 않는다.)
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceError {
    /// 입력 경로가 존재하지 않는다.
    NotFound {
        /// 열려고 한 경로.
        path: PathBuf,
    },
    /// 경로는 있으나 읽지 못했다(권한 없음·디렉터리·I/O 오류 등).
    ///
    /// `NotFound` 와 갈라 두는 이유: "파일이 없다"는 호출자가 경로를 고치면 되는
    /// 사용법 오류지만, "읽을 수 없다"는 환경 문제라 재시도·권한 조정의 대상이다.
    Io {
        /// 열려고 한 경로.
        path: PathBuf,
        /// 표준 I/O 오류 갈래. `std::io::Error` 는 `Clone`·`PartialEq` 가 아니라
        /// 갈래만 보존한다.
        kind: std::io::ErrorKind,
    },
    /// 매직 바이트로 판별한 형식이 rhwp 가 여는 4형식(HWP5·HWPX·HWP3·HML)이 아니다.
    ///
    /// 파서를 부르기 **전에** 판정하므로, 손상된 대용량 입력이 파서를 통과할 기회를
    /// 얻지 못한다. `detected` 에는 [`FileFormat::Unknown`]·[`FileFormat::Empty`]·
    /// [`FileFormat::DrmProtected`] 중 하나가 담긴다 — DRM 컨테이너를 "알 수 없음"과
    /// 같은 말로 보고하면 사용자가 할 수 있는 조치(DRM 해제 후 저장)를 알 수 없다.
    UnsupportedFormat {
        /// 매직 바이트로 감지한 형식.
        detected: FileFormat,
    },
    /// 암호 문서인데 비밀번호를 받지 못했다.
    PasswordRequired,
    /// 비밀번호가 일치하지 않거나 암호화 데이터가 손상됐다.
    PasswordMismatch,
    /// 입력이 크기 상한을 넘었다. **파싱 전에** 판정한다.
    TooLarge {
        /// 실제 입력 크기.
        size_bytes: usize,
        /// 적용된 상한([`crate::service::OpenOptions::max_bytes`] 또는 서비스 기본값).
        limit_bytes: usize,
    },
    /// 그 밖의 파싱 실패. 원문 메시지는 **사람에게 보여주기 위한 것**이며,
    /// 소비자가 이 문자열을 다시 갈라 읽으면 안 된다.
    Parse(String),
}

impl ServiceError {
    /// 기계 판독용 안정 토큰. JSON 봉투의 `code` 필드와 exit code 매핑의 단일 축이다.
    ///
    /// 이 문자열은 **계약**이다. 값 추가는 허용하되 기존 토큰의 변경·삭제는 소비자를
    /// 깨뜨린다.
    pub fn code(&self) -> &'static str {
        match self {
            ServiceError::NotFound { .. } => "NOT_FOUND",
            ServiceError::Io { .. } => "IO",
            ServiceError::UnsupportedFormat { .. } => "UNSUPPORTED_FORMAT",
            ServiceError::PasswordRequired => "PASSWORD_REQUIRED",
            ServiceError::PasswordMismatch => "PASSWORD_MISMATCH",
            ServiceError::TooLarge { .. } => "TOO_LARGE",
            ServiceError::Parse(_) => "PARSE",
        }
    }

    /// 호출자가 **요청을 고치면 해소되는** 실패인가.
    ///
    /// CLI 는 이 값이 참이면 `EXIT_USAGE`(2), 거짓이면 `EXIT_RUNTIME`(1) 로 매핑하면
    /// 된다. 현행 `main.rs` 의 `LoadError` 매핑(NeedPassword→USAGE,
    /// WrongPassword→RUNTIME)과 일치한다 — 이 축이 없으면 소비자는 다시 문자열을
    /// 들여다볼 수밖에 없다.
    pub fn is_usage_error(&self) -> bool {
        matches!(
            self,
            ServiceError::NotFound { .. }
                | ServiceError::UnsupportedFormat { .. }
                | ServiceError::PasswordRequired
                | ServiceError::TooLarge { .. }
        )
    }

    /// 비밀번호를 주면 다시 시도할 가치가 있는 실패인가.
    ///
    /// MCP 같은 대화형 소비자가 `nextCall` 로 "password 를 붙여 재시도"를 제안할
    /// 조건이다.
    pub fn needs_password(&self) -> bool {
        matches!(
            self,
            ServiceError::PasswordRequired | ServiceError::PasswordMismatch
        )
    }

    /// `DocumentCore` 가 돌려준 [`HwpError`] 를 서비스 오류로 옮긴다.
    ///
    /// # 왜 문자열을 들여다보는가 — 그리고 왜 그래도 안전한가
    ///
    /// `DocumentCore::from_bytes` 는 타입 있는 [`ParseError`] 를
    /// `HwpError::InvalidFile(String)` 으로 **평탄화해서** 돌려준다. 이 PR 은 기존
    /// 파일을 고치지 않으므로 그 평탄화를 되돌릴 수 없고, 타입 복원은 이 경계에서
    /// 한 번만 일어난다.
    ///
    /// 다만 `main.rs` 처럼 한국어 문장을 **상수로 박아 넣지는 않는다**. 대조할
    /// 바늘을 타입에서 그 자리에서 만든다 — `ParseError::EncryptedDocument.to_string()`,
    /// `CryptoError::WrongPassword.to_string()`. 업스트림이 문구를 고치면 바늘도
    /// 같이 따라가므로, 문구 변경이 exit code 를 조용히 뒤집는 사고가 나지 않는다.
    /// HWPX·HWP3 의 비밀번호 불일치도 같은 문장을 감싸 내보내므로(`HWPX 오류: …`)
    /// 부분 일치로 세 형식을 한 번에 덮는다.
    ///
    /// `password_supplied` 는 갈래를 하나 더 바로잡는다. 비밀번호를 **주었는데도**
    /// "암호 문서" 오류가 오면 그건 "비밀번호를 주세요"가 아니라 "그 비밀번호가
    /// 틀렸다"이다. 현행 CLI 는 이 경우에도 `--password 를 전달하세요` 를 출력해,
    /// 이미 준 것을 다시 요구한다.
    pub(crate) fn from_open_failure(error: &HwpError, password_supplied: bool) -> ServiceError {
        let HwpError::InvalidFile(inner) = error else {
            return ServiceError::Parse(error.to_string());
        };
        if inner.contains(&CryptoError::WrongPassword.to_string()) {
            return ServiceError::PasswordMismatch;
        }
        if inner.contains(&ParseError::EncryptedDocument.to_string()) {
            return if password_supplied {
                ServiceError::PasswordMismatch
            } else {
                ServiceError::PasswordRequired
            };
        }
        ServiceError::Parse(inner.clone())
    }
}

impl std::fmt::Display for ServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceError::NotFound { path } => {
                write!(f, "파일을 찾을 수 없습니다: {}", path.display())
            }
            ServiceError::Io { path, kind } => {
                write!(f, "파일을 읽을 수 없습니다: {} ({kind:?})", path.display())
            }
            ServiceError::UnsupportedFormat { detected } => match detected {
                FileFormat::Empty => write!(f, "빈 파일(0 바이트)입니다."),
                FileFormat::DrmProtected => write!(
                    f,
                    "DRM/보안 컨테이너로 보호된 문서입니다. 보호를 해제한 뒤 저장해 열어주세요."
                ),
                other => write!(
                    f,
                    "지원하지 않는 형식입니다: {other:?}. rhwp 는 HWP 5.0·HWPX·일부 HWP 3.0·HWPML 을 엽니다."
                ),
            },
            ServiceError::PasswordRequired => {
                write!(f, "비밀번호가 필요한 암호 문서입니다.")
            }
            ServiceError::PasswordMismatch => write!(
                f,
                "비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다."
            ),
            ServiceError::TooLarge {
                size_bytes,
                limit_bytes,
            } => write!(
                f,
                "입력이 크기 상한을 넘었습니다: {size_bytes}바이트 (상한 {limit_bytes}바이트)"
            ),
            ServiceError::Parse(message) => write!(f, "문서 파싱 실패 - {message}"),
        }
    }
}

impl std::error::Error for ServiceError {}
