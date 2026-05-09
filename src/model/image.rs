//! 그림 개체 (Picture, ImageData, CropInfo)

use super::*;
use super::shape::{CommonObjAttr, ShapeComponentAttr};
use super::style::ShapeBorderLine;

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
    /// 테두리 투명도
    pub border_opacity: u8,
    /// 인스턴스 ID
    pub instance_id: u32,
    /// SHAPE_PICTURE 레코드의 파싱된 필드 이후 추가 바이트 (라운드트립 보존용)
    pub raw_picture_extra: Vec<u8>,
    /// 캡션
    pub caption: Option<super::shape::Caption>,
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
    /// [Task #741] 외부 file path 그림 (HWP3 spec offset 74 그림 종류 0=외부 파일,
    /// 1=OLE, 2=Embedded Image / offset 83~339 그림 파일 이름).
    /// HWP3 외부 link 그림이고 binary 데이터 부재 시 placeholder 표시용.
    /// `None` = 내부 임베드 그림 (binary 데이터 사용).
    pub external_path: Option<String>,
}

impl ImageAttr {
    /// 워터마크 효과가 적용되어 있는지 식별 (Task #516).
    /// effect 가 RealPic 이 아니고 brightness/contrast 중 하나라도 변경된 경우.
    pub fn is_watermark(&self) -> bool {
        !matches!(self.effect, ImageEffect::RealPic)
            && (self.brightness != 0 || self.contrast != 0)
    }

    /// 한컴 자동 워터마크 프리셋 정합 여부 (Task #516).
    /// 한컴 도구의 "이미지 → 회색조 → 워터마크 효과" 체크 시 자동 적용:
    /// effect=GrayScale, brightness=70, contrast=-50.
    pub fn is_hancom_watermark_preset(&self) -> bool {
        matches!(self.effect, ImageEffect::GrayScale)
            && self.brightness == 70
            && self.contrast == -50
    }

    /// 워터마크 preset 분류 (Task #516, AI 메타정보).
    pub fn watermark_preset(&self) -> Option<&'static str> {
        if self.is_hancom_watermark_preset() {
            Some("hancom-watermark")
        } else if self.is_watermark() {
            Some("custom")
        } else {
            None
        }
    }
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
        let crop = CropInfo { left: 100, top: 200, right: 300, bottom: 400 };
        assert_eq!(crop.left, 100);
    }

    #[test]
    fn test_image_format_default() {
        assert_eq!(ImageFormat::default(), ImageFormat::Unknown);
    }
}
