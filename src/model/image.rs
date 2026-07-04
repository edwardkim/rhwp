//! 그림 개체 (Picture, ImageData, CropInfo)

use super::shape::{CommonObjAttr, ShapeComponentAttr};
use super::style::ShapeBorderLine;
use super::*;

/// 그림 개체 (HWPTAG_SHAPE_COMPONENT_PICTURE)
#[derive(Debug, Default, Clone)]
pub struct Picture {
    /// 개체 공통 속성
    pub common: CommonObjAttr,
    /// 개체 요소 속성
    pub shape_attr: ShapeComponentAttr,
    /// 테두리 색
    pub border_color: ColorRef,
    /// 테두리 두께
    pub border_width: i32,
    /// 테두리 속성
    pub border_attr: ShapeBorderLine,
    /// 이미지 테두리 좌표 X (4개)
    pub border_x: [i32; 4],
    /// 이미지 테두리 좌표 Y (4개)
    pub border_y: [i32; 4],
    /// 자르기 정보
    pub crop: CropInfo,
    /// 안쪽 여백
    pub padding: Padding,
    /// 그림 속성
    pub image_attr: ImageAttr,
    /// HWPX `<hp:pic href="...">` 값.
    ///
    /// 한컴 HWP 저장 결과에서는 이 값이 그림 컨트롤 뒤의 CTRL_DATA ParameterSet
    /// (`ps_id=0x021b -> ps_id=0x026f -> id=0x0265 string`) 으로 materialize된다.
    pub href: Option<String>,
    /// 테두리 투명도
    pub border_opacity: u8,
    /// 인스턴스 ID
    pub instance_id: u32,
    /// SHAPE_PICTURE 레코드의 파싱된 필드 이후 추가 바이트 (라운드트립 보존용)
    pub raw_picture_extra: Vec<u8>,
    /// HWPX `<hp:effects>` 그림 효과 정보.
    pub effects: PictureEffects,
    /// 캡션
    pub caption: Option<super::shape::Caption>,
    /// HWPX `<hp:imgDim>` 원본 이미지 픽셀 크기 (#1389).
    ///
    /// imgClip extent 와 독립(전수 측정: 불일치 24/170) — 원본 이미지 픽셀 크기를
    /// verbatim 보존한다. (dimwidth, dimheight). HWPX 파서만 적재.
    pub img_dim: (u32, u32),
}

/// 자르기 정보
#[derive(Debug, Clone, Copy, Default)]
pub struct CropInfo {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

/// 이미지 속성
#[derive(Debug, Clone, Default)]
pub struct ImageAttr {
    /// 밝기
    pub brightness: i8,
    /// 명암
    pub contrast: i8,
    /// 그림 효과
    pub effect: ImageEffect,
    /// BinData ID 참조
    pub bin_data_id: u16,
    /// 그림 개체 전체 투명도. 한컴 UI 기준으로 0=불투명, 100=완전 투명.
    pub transparency: u8,
    /// [Task #741] 외부 file path 그림 (HWP3 spec offset 74 그림 종류 0=외부 파일,
    /// 1=OLE, 2=Embedded Image / offset 83~339 그림 파일 이름).
    /// HWP3 외부 link 그림이고 binary 데이터 부재 시 placeholder 표시용.
    /// `None` = 내부 임베드 그림 (binary 데이터 사용).
    pub external_path: Option<String>,
}

/// HWPX 그림 효과 (`hp:effects`).
#[derive(Debug, Clone, Default)]
pub struct PictureEffects {
    pub shadow: Option<PictureShadow>,
}

/// HWPX 그림 그림자 효과 (`hp:shadow`).
#[derive(Debug, Clone, Default)]
pub struct PictureShadow {
    pub style: Option<String>,
    pub alpha: Option<String>,
    pub radius: Option<String>,
    pub direction: Option<String>,
    pub distance: Option<String>,
    pub align_style: Option<String>,
    pub rotation_style: Option<String>,
    pub skew: Option<EffectPoint>,
    pub scale: Option<EffectPoint>,
    pub color: Option<EffectColor>,
}

/// HWPX 효과의 x/y 좌표성 값 (`hp:skew`, `hp:scale`).
#[derive(Debug, Clone, Default)]
pub struct EffectPoint {
    pub x: Option<String>,
    pub y: Option<String>,
}

/// HWPX 효과 색상 (`hp:effectsColor`).
#[derive(Debug, Clone, Default)]
pub struct EffectColor {
    pub color_type: Option<String>,
    pub scheme_idx: Option<String>,
    pub system_idx: Option<String>,
    pub preset_idx: Option<String>,
    pub rgb: Option<EffectRgb>,
}

/// HWPX 효과 RGB 색상 (`hp:rgb`).
#[derive(Debug, Clone, Default)]
pub struct EffectRgb {
    pub r: Option<String>,
    pub g: Option<String>,
    pub b: Option<String>,
}

impl ImageAttr {
    pub fn clamped_transparency(&self) -> u8 {
        self.transparency.min(100)
    }

    pub fn transparency_alpha_byte(&self) -> u8 {
        transparency_percent_to_alpha_byte(self.clamped_transparency())
    }

    pub fn opacity(&self) -> f64 {
        1.0 - (self.clamped_transparency() as f64 / 100.0)
    }

    /// 워터마크 효과가 적용되어 있는지 식별 (Task #516, Issue #1156 정정).
    ///
    /// HWP/HWPX 에는 워터마크 적용을 나타내는 별도 비트/속성이 **존재하지 않는다**
    /// (한컴 공식 파일구조 3.0/5.0 + water-mark.hwp/.hwpx 두 그림 비교로 확정).
    /// 한컴 편집기는 "워터마크 효과" 체크를 해제하면 밝기·대비를 모두 0 으로
    /// 되돌리고, 체크하면 0 이 아닌 밝기·대비 값을 부여한다 (기본 70/-50, 사용자
    /// 변경 가능). 따라서 워터마크 여부는 **밝기·대비가 둘 다 0 이 아닌 경우**
    /// 로 판정한다 (effect 종류 무관, 한쪽이라도 0 이면 워터마크 아님).
    pub fn is_watermark(&self) -> bool {
        self.brightness != 0 && self.contrast != 0
    }

    /// 워터마크 preset 분류 (Task #516, AI 메타정보).
    pub fn watermark_preset(&self) -> Option<&'static str> {
        if self.is_watermark() {
            Some("custom")
        } else {
            None
        }
    }
}

/// 한컴 UI 투명도(0~100%)를 HWP/HWPX 저장용 alpha byte(0~255)로 변환한다.
pub fn transparency_percent_to_alpha_byte(transparency: u8) -> u8 {
    ((transparency.min(100) as u16 * 255) / 100) as u8
}

/// HWP/HWPX 저장용 alpha byte(0~255)를 한컴 UI 투명도(0~100%)로 변환한다.
pub fn alpha_byte_to_transparency_percent(alpha: u8) -> u8 {
    ((alpha as f64 * 100.0) / 255.0).round().clamp(0.0, 100.0) as u8
}

/// 이미지 효과
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize)]
pub enum ImageEffect {
    #[default]
    RealPic,
    GrayScale,
    BlackWhite,
    Pattern8x8,
}

/// 이미지 데이터 (실제 바이너리 데이터 보관)
#[derive(Debug, Clone)]
pub struct ImageData {
    /// 이미지 형식
    pub format: ImageFormat,
    /// 바이너리 데이터
    pub data: Vec<u8>,
}

/// 이미지 형식
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImageFormat {
    Bmp,
    Jpg,
    Png,
    Gif,
    Tiff,
    Wmf,
    Emf,
    Unknown,
}

impl Default for ImageFormat {
    fn default() -> Self {
        ImageFormat::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_picture_default() {
        let pic = Picture::default();
        assert_eq!(pic.image_attr.effect, ImageEffect::RealPic);
        assert_eq!(pic.border_width, 0);
    }

    #[test]
    fn test_crop_info() {
        let crop = CropInfo {
            left: 100,
            top: 200,
            right: 300,
            bottom: 400,
        };
        assert_eq!(crop.left, 100);
    }

    #[test]
    fn test_image_format_default() {
        assert_eq!(ImageFormat::default(), ImageFormat::Unknown);
    }
}
