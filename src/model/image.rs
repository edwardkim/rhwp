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
