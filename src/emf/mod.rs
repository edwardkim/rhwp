//! EMF (Enhanced Metafile) 파서 + SVG 컨버터.
//!
//! Task #195 단계 10~14에서 단계적으로 구현한다. 본 단계(10)는 모듈 골격과
//! `EMR_HEADER` 파서까지만 포함한다. WMF 모듈(`crate::wmf`)과 완전히 독립적으로
//! 유지한다 — 레코드 enum·좌표 크기·헤더 구조가 모두 다르므로 코드 공유 금지.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::too_many_lines,
    clippy::unreadable_literal,
    clippy::upper_case_acronyms,
    dead_code,
    unused_imports,
    unused_variables
)]

pub mod converter;
pub mod parser;

#[cfg(test)]
mod tests;

pub use parser::records::Record;
pub use parser::Header;

/// EMF 파서/컨버터 공용 오류.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// 파일 선두 EMR_HEADER의 Signature(" EMF", offset 40) 불일치.
    InvalidSignature { got: u32 },
    /// 파일 선두 레코드 Type이 1(EMR_HEADER)이 아님.
    InvalidFirstRecord { got: u32 },
    /// 스트림 길이 부족.
    UnexpectedEof { at: usize, need: usize },
    /// 레코드 Size 필드가 4의 배수가 아님.
    MisalignedRecord { offset: usize, size: u32 },
    /// 레코드 Size 필드가 최소 헤더(8) 미만.
    RecordTooSmall { offset: usize, size: u32 },
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidSignature { got } => write!(
                f,
                "invalid EMF signature: 0x{got:08X} (expected 0x464D4520 ' EMF')"
            ),
            Self::InvalidFirstRecord { got } => write!(
                f,
                "first record must be EMR_HEADER (type=1), got type={got}"
            ),
            Self::UnexpectedEof { at, need } => {
                write!(f, "unexpected EOF at offset {at}, needed {need} bytes")
            }
            Self::MisalignedRecord { offset, size } => write!(
                f,
                "misaligned record at offset {offset}: size={size} is not multiple of 4"
            ),
            Self::RecordTooSmall { offset, size } => {
                write!(f, "record at offset {offset} too small: size={size} < 8")
            }
        }
    }
}

impl std::error::Error for Error {}

/// EMF 바이트 스트림을 레코드 시퀀스로 파싱한다.
pub fn parse_emf(bytes: &[u8]) -> Result<Vec<Record>, Error> {
    parser::parse(bytes)
}

/// EMF 바이트를 파싱 후 SVG fragment 문자열로 변환한다.
///
/// `render_rect = (x, y, w, h)` (pt 단위)는 SVG 상 배치 영역을 지정한다. Player는
/// EMF Bounds → render_rect 매핑 행렬을 자동 계산하여 `<g transform="...">`으로 감싼다.
///
/// 반환값은 viewBox/xmlns가 없는 **fragment**로, rhwp 렌더 트리의 RawSvg로 삽입된다.
pub fn convert_to_svg(bytes: &[u8], render_rect: (f32, f32, f32, f32)) -> Result<String, Error> {
    let records = parse_emf(bytes)?;
    let mut player = converter::Player::new(render_rect);
    player.play(&records)?;
    Ok(player.svg.into_string())
}
