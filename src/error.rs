//! 도메인 에러 타입
//!
//! 파서, 렌더러, 커맨드 등 크레이트 전역에서 사용하는 에러 열거형.

/// 네이티브 에러 타입 (non-WASM 환경에서도 안전하게 사용)
#[derive(Debug)]
pub enum HwpError {
    /// 파일이 유효하지 않음
    InvalidFile(String),
    /// 페이지 범위 초과
    PageOutOfRange(u32),
    /// 렌더링 오류
    RenderError(String),
    /// 필드 관련 오류
    InvalidField(String),
}

impl From<crate::parser::ParseError> for HwpError {
    fn from(e: crate::parser::ParseError) -> Self {
        // 사용자용 Display 메시지 사용 (Debug 아님). Issue #265.
        HwpError::InvalidFile(format!("{e}"))
    }
}

impl From<crate::parser::hwpx::HwpxError> for HwpError {
    fn from(e: crate::parser::hwpx::HwpxError) -> Self {
        HwpError::InvalidFile(format!("{e}"))
    }
}

impl From<crate::serializer::SerializeError> for HwpError {
    fn from(e: crate::serializer::SerializeError) -> Self {
        HwpError::RenderError(format!("{e}"))
    }
}

impl std::fmt::Display for HwpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HwpError::InvalidFile(msg) => write!(f, "유효하지 않은 파일: {}", msg),
            HwpError::PageOutOfRange(n) => write!(f, "페이지 {}을(를) 찾을 수 없습니다", n),
            HwpError::RenderError(msg) => write!(f, "렌더링 오류: {}", msg),
            HwpError::InvalidField(msg) => write!(f, "필드 오류: {}", msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ParseError;

    #[test]
    fn parse_error_to_hwp_error_uses_display_not_debug() {
        // Issue #265: From<ParseError> 가 Debug 대신 Display 를 전파해야 한다.
        // UnsupportedFormat 의 친절한 한국어 힌트가 사용자에게 노출되는 경로.
        let pe = ParseError::UnsupportedFormat {
            format: "HWP 3.0",
            hint: "다시 저장해주세요.",
        };
        let he: HwpError = pe.into();
        let msg = format!("{he}");
        // Display 전파: "유효하지 않은 파일: 지원하지 않는 포맷입니다: HWP 3.0. 다시 저장해주세요."
        assert!(
            msg.contains("HWP 3.0"),
            "HWP 3.0 must appear in display: {msg}"
        );
        assert!(msg.contains("다시 저장해주세요"), "hint must appear: {msg}");
        // Debug 형식 (variant 이름·중괄호) 이 누출되지 않아야 한다.
        assert!(!msg.contains("UnsupportedFormat"), "Debug leaked: {msg}");
        assert!(!msg.contains("{ format"), "Debug leaked: {msg}");
    }
}
