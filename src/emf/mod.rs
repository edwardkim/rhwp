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
pub(crate) mod emfplus;
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
    convert_to_svg_with_device(bytes, render_rect, None)
}

/// 출력 장치의 픽셀 크기를 명시해 재생한다. 참조 장치와 출력 장치를 구분한다.
pub fn convert_to_svg_with_device(
    bytes: &[u8],
    render_rect: (f32, f32, f32, f32),
    device_pixel_width: Option<f64>,
) -> Result<String, Error> {
    let records = parse_emf(bytes)?;
    let mut player =
        converter::Player::new(render_rect).with_device_pixel_width(device_pixel_width);
    player.play(&records)?;
    Ok(player.svg.into_string())
}

/// EMF 바이트를 **단독 SVG 문서**로 변환한다 — data URI·`<img>` 임베드용.
///
/// `convert_to_svg` 는 배치 영역을 아는 호출자(렌더 트리 RawSvg)를 위한
/// viewBox/xmlns 없는 fragment 를 준다. data URI 소비자는 배치 영역이 그림
/// 바깥(SVG `<image>`·DOM `<img>` 의 bbox)에서 정해지므로, 치수를 EMF 헤더에서
/// 세워 문서로 감싼다: frame(0.01mm)을 pt 로 환산하고, frame 이 비어 있으면
/// bounds(장치 좌표)를 96dpi 픽셀로 보고 환산한다.
pub fn convert_to_standalone_svg(bytes: &[u8]) -> Option<Vec<u8>> {
    let records = parse_emf(bytes).ok()?;
    let header = records.iter().find_map(|record| match record {
        Record::Header(header) => Some(header),
        _ => None,
    })?;
    let (w, h) = header_size(header)?;
    let mut player = converter::Player::new((0.0, 0.0, w as f32, h as f32));
    player.play(&records).ok()?;
    let fragment = player.svg.into_string();
    Some(
        format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w:.2}\" height=\"{h:.2}\" viewBox=\"0 0 {w:.2} {h:.2}\">{fragment}</svg>"
        )
        .into_bytes(),
    )
}

/// 그림 배치 크기로 만든 SVG. 장치 펜은 외부 그림 배율을 적용하기 전에 확정한다.
pub fn convert_to_sized_svg(
    bytes: &[u8],
    width: f64,
    height: f64,
    device_pixel_width: f64,
) -> Option<Vec<u8>> {
    if ![width, height, device_pixel_width]
        .iter()
        .all(|v| v.is_finite() && *v > 0.0)
        || width > f32::MAX as f64
        || height > f32::MAX as f64
    {
        return None;
    }
    let fragment = convert_to_svg_with_device(
        bytes,
        (0.0, 0.0, width as f32, height as f32),
        Some(device_pixel_width),
    )
    .ok()?;
    Some(format!("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width:.9}\" height=\"{height:.9}\" viewBox=\"0 0 {width:.9} {height:.9}\">{fragment}</svg>").into_bytes())
}

/// 배치 후에도 profile에 맞는 장치로 다시 재생할 수 있는 원본이다.
#[derive(Debug, Clone)]
pub struct EmfReplaySource {
    pub data: std::sync::Arc<[u8]>,
    pub render_rect: (f32, f32, f32, f32),
}

/// 단독 SVG의 고유 크기. 채움·자르기 재생도 같은 크기를 소비한다.
pub(crate) fn standalone_size(bytes: &[u8]) -> Option<(f64, f64)> {
    let records = parse_emf(bytes).ok()?;
    let header = records.iter().find_map(|r| match r {
        Record::Header(h) => Some(h),
        _ => None,
    })?;
    header_size(header)
}

fn header_size(header: &Header) -> Option<(f64, f64)> {
    let frame_pt = (
        f64::from(header.frame.width()) / 100.0 * 72.0 / 25.4,
        f64::from(header.frame.height()) / 100.0 * 72.0 / 25.4,
    );
    let bounds_pt = (
        f64::from(header.bounds.width()) * 72.0 / 96.0,
        f64::from(header.bounds.height()) * 72.0 / 96.0,
    );
    let (w, h) = if frame_pt.0 > 0.0 && frame_pt.1 > 0.0 {
        frame_pt
    } else {
        bounds_pt
    };
    (w.is_finite() && h.is_finite() && w > 0.0 && h > 0.0).then_some((w, h))
}
