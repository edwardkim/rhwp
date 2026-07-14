#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HmlError {
    UnsupportedEncoding,
    InvalidXml(String),
    NotHmlDocument,
    UnsupportedVersion(String),
    MissingHead,
    MissingBody,
    InvalidReference(String),
    LimitExceeded(String),
}

impl std::fmt::Display for HmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedEncoding => write!(f, "지원하지 않는 HML 문자 인코딩입니다"),
            Self::InvalidXml(message) => write!(f, "잘못된 HML XML입니다: {message}"),
            Self::NotHmlDocument => write!(f, "HML 문서가 아닙니다"),
            Self::UnsupportedVersion(version) => {
                write!(f, "지원하지 않는 HWPML 버전입니다: {version}")
            }
            Self::MissingHead => write!(f, "HML HEAD 요소가 없습니다"),
            Self::MissingBody => write!(f, "HML BODY 요소가 없습니다"),
            Self::InvalidReference(reference) => write!(f, "잘못된 HML 참조입니다: {reference}"),
            Self::LimitExceeded(limit) => write!(f, "HML XML 제한을 초과했습니다: {limit}"),
        }
    }
}

impl std::error::Error for HmlError {}
