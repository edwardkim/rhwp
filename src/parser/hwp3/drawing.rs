//! HWP3 그리기 객체 파싱
//!
//! HWP3 파일에 포함된 그리기 객체(선, 사각형, 타원, 그룹 등)를 파싱하여 렌더링 가능한 모델로 변환한다.
//! 그리기 객체의 계층 구조(트리)와 캡션, 속성 정보 등을 추출하는 역할을 한다.

use crate::parser::hwp3::encoding::decode_hwp3_string;
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Read, Seek, SeekFrom};

#[derive(Debug, Default)]
pub struct Hwp3DrawingObjectFrameHeader {
    pub header_length: u32,
    pub z_order: u32,
    pub object_count: u32,
    pub bounds: [i32; 4], // shunit32 (x, y, 너비, 높이)
}

impl Hwp3DrawingObjectFrameHeader {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let header_length = reader.read_u32::<LittleEndian>()?;
        let z_order = reader.read_u32::<LittleEndian>()?;
        let object_count = reader.read_u32::<LittleEndian>()?;
        let bounds = [
            reader.read_i32::<LittleEndian>()?,
            reader.read_i32::<LittleEndian>()?,
            reader.read_i32::<LittleEndian>()?,
            reader.read_i32::<LittleEndian>()?,
        ];

        Ok(Hwp3DrawingObjectFrameHeader {
            header_length,
            z_order,
            object_count,
            bounds,
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingObjectHypertextInfo {
    pub length: u32,
    pub jump_file_name: String, // 256 kchar
    pub jump_bookmark: String,  // 16 hchar = 32 바이트 (스펙 8.3절 표 21, 오프셋 264→296)
    pub macro_data: Vec<u8>,    // 325 바이트
    pub kind: u8,
    pub reserved: [u8; 3],
}

impl Hwp3DrawingObjectHypertextInfo {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let length = reader.read_u32::<LittleEndian>()?;
        let mut jump_file_name_buf = [0u8; 256];
        reader.read_exact(&mut jump_file_name_buf)?;
        let jump_file_name = decode_hwp3_string(&jump_file_name_buf);

        // [Task #2831] 스펙 8.3절 표 21은 "건너뛸 책갈피"를 hchar array[16]로 명시하며
        // (hchar=2바이트), 표의 절대 오프셋(264→296)과 전체 길이 공식(617 =
        // 256+32+325+1+3)이 모두 32바이트를 요구한다. 16바이트만 읽으면 이후
        // 그리기 개체 레코드 전체가 16바이트씩 밀려 파싱된다.
        let mut jump_bookmark_buf = [0u8; 32];
        reader.read_exact(&mut jump_bookmark_buf)?;
        let jump_bookmark = decode_hwp3_string(&jump_bookmark_buf);

        let mut macro_data = vec![0u8; 325];
        reader.read_exact(&mut macro_data)?;

        let kind = reader.read_u8()?;
        let mut reserved = [0u8; 3];
        reader.read_exact(&mut reserved)?;

        Ok(Hwp3DrawingObjectHypertextInfo {
            length,
            jump_file_name,
            jump_bookmark,
            macro_data,
            kind,
            reserved,
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingObjectBasicAttr {
    pub line_style: u32,
    pub arrow_end: u32,
    pub arrow_start: u32,
    pub line_color: u32,
    pub line_width: u32,
    pub fill_color: u32,
    pub pattern_type: u32,
    pub pattern_color: u32,
    pub textbox_margin: [u32; 2],
    pub options: u32,
}

impl Hwp3DrawingObjectBasicAttr {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        Ok(Hwp3DrawingObjectBasicAttr {
            line_style: reader.read_u32::<LittleEndian>()?,
            arrow_end: reader.read_u32::<LittleEndian>()?,
            arrow_start: reader.read_u32::<LittleEndian>()?,
            line_color: reader.read_u32::<LittleEndian>()?,
            line_width: reader.read_u32::<LittleEndian>()?,
            fill_color: reader.read_u32::<LittleEndian>()?,
            pattern_type: reader.read_u32::<LittleEndian>()?,
            pattern_color: reader.read_u32::<LittleEndian>()?,
            textbox_margin: [
                reader.read_u32::<LittleEndian>()?,
                reader.read_u32::<LittleEndian>()?,
            ],
            options: reader.read_u32::<LittleEndian>()?,
        })
    }

    pub fn has_gradient(&self) -> bool {
        (self.options & (1 << 16)) != 0
    }

    pub fn has_rotation(&self) -> bool {
        (self.options & (1 << 17)) != 0
    }

    pub fn has_bitmap_pattern(&self) -> bool {
        (self.options & (1 << 18)) != 0
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingObjectRotationAttr {
    pub center_x: i32,
    pub center_y: i32,
    pub parallelogram: [i32; 6],
}

impl Hwp3DrawingObjectRotationAttr {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        Ok(Hwp3DrawingObjectRotationAttr {
            center_x: reader.read_i32::<LittleEndian>()?,
            center_y: reader.read_i32::<LittleEndian>()?,
            parallelogram: [
                reader.read_i32::<LittleEndian>()?,
                reader.read_i32::<LittleEndian>()?,
                reader.read_i32::<LittleEndian>()?,
                reader.read_i32::<LittleEndian>()?,
                reader.read_i32::<LittleEndian>()?,
                reader.read_i32::<LittleEndian>()?,
            ],
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingObjectGradientAttr {
    pub start_color: u32,
    pub end_color: u32,
    pub kind: u32,
    pub angle: u32,
    pub center_x: u32,
    pub center_y: u32,
    pub step: u32,
}

impl Hwp3DrawingObjectGradientAttr {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        Ok(Hwp3DrawingObjectGradientAttr {
            start_color: reader.read_u32::<LittleEndian>()?,
            end_color: reader.read_u32::<LittleEndian>()?,
            kind: reader.read_u32::<LittleEndian>()?,
            angle: reader.read_u32::<LittleEndian>()?,
            center_x: reader.read_u32::<LittleEndian>()?,
            center_y: reader.read_u32::<LittleEndian>()?,
            step: reader.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingObjectBitmapPatternAttr {
    pub start_pos: [u32; 2],
    pub end_pos: [u32; 2],
    pub file_name: String, // 261 바이트
    pub option: u8,
}

impl Hwp3DrawingObjectBitmapPatternAttr {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let start_pos = [
            reader.read_u32::<LittleEndian>()?,
            reader.read_u32::<LittleEndian>()?,
        ];
        let end_pos = [
            reader.read_u32::<LittleEndian>()?,
            reader.read_u32::<LittleEndian>()?,
        ];
        let mut file_name_buf = [0u8; 261];
        reader.read_exact(&mut file_name_buf)?;
        let file_name = decode_hwp3_string(&file_name_buf);
        let option = reader.read_u8()?;

        Ok(Hwp3DrawingObjectBitmapPatternAttr {
            start_pos,
            end_pos,
            file_name,
            option,
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingObjectCommonHeader {
    pub header_length: u32,
    pub object_type: u16,
    pub connection_info: u16,
    pub relative_pos: [u32; 2],
    pub object_size: [u32; 2],
    pub absolute_pos: [u32; 2],
    pub bounds: [i32; 4],
    pub basic_attr: Hwp3DrawingObjectBasicAttr,
    pub rotation_attr: Option<Hwp3DrawingObjectRotationAttr>,
    pub gradient_attr: Option<Hwp3DrawingObjectGradientAttr>,
    pub bitmap_pattern_attr: Option<Hwp3DrawingObjectBitmapPatternAttr>,
}

impl Hwp3DrawingObjectCommonHeader {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let header_length = reader.read_u32::<LittleEndian>()?;
        let object_type = reader.read_u16::<LittleEndian>()?;
        let connection_info = reader.read_u16::<LittleEndian>()?;
        let relative_pos = [
            reader.read_u32::<LittleEndian>()?,
            reader.read_u32::<LittleEndian>()?,
        ];
        let object_size = [
            reader.read_u32::<LittleEndian>()?,
            reader.read_u32::<LittleEndian>()?,
        ];
        let absolute_pos = [
            reader.read_u32::<LittleEndian>()?,
            reader.read_u32::<LittleEndian>()?,
        ];
        let bounds = [
            reader.read_i32::<LittleEndian>()?,
            reader.read_i32::<LittleEndian>()?,
            reader.read_i32::<LittleEndian>()?,
            reader.read_i32::<LittleEndian>()?,
        ];

        let basic_attr = Hwp3DrawingObjectBasicAttr::read(&mut reader)?;

        let rotation_attr = if basic_attr.has_rotation() {
            Some(Hwp3DrawingObjectRotationAttr::read(&mut reader)?)
        } else {
            None
        };

        let gradient_attr = if basic_attr.has_gradient() {
            Some(Hwp3DrawingObjectGradientAttr::read(&mut reader)?)
        } else {
            None
        };

        let bitmap_pattern_attr = if basic_attr.has_bitmap_pattern() {
            Some(Hwp3DrawingObjectBitmapPatternAttr::read(&mut reader)?)
        } else {
            None
        };

        Ok(Hwp3DrawingObjectCommonHeader {
            header_length,
            object_type,
            connection_info,
            relative_pos,
            object_size,
            absolute_pos,
            bounds,
            basic_attr,
            rotation_attr,
            gradient_attr,
            bitmap_pattern_attr,
        })
    }
}

// 개체별 세부 정보
#[derive(Debug)]
pub enum Hwp3DrawingObject {
    Container(Hwp3DrawingObjectCommonHeader),
    Line(Hwp3DrawingObjectCommonHeader, Hwp3DrawingLine),
    Rectangle(Hwp3DrawingObjectCommonHeader),
    Ellipse(Hwp3DrawingObjectCommonHeader),
    Arc(Hwp3DrawingObjectCommonHeader, Hwp3DrawingArc),
    Polygon(Hwp3DrawingObjectCommonHeader, Hwp3DrawingPolygon),
    TextBox(Hwp3DrawingObjectCommonHeader, Hwp3DrawingTextBox),
    Curve(Hwp3DrawingObjectCommonHeader, Hwp3DrawingCurve),
    ModifiedEllipse(Hwp3DrawingObjectCommonHeader, Hwp3DrawingModifiedEllipse),
    ModifiedArc(Hwp3DrawingObjectCommonHeader), // 공통 헤더 외에 추가적인 세부 정보 없음
    ExtendedCurve(Hwp3DrawingObjectCommonHeader, Hwp3DrawingExtendedPolygon),
    ClosedPolygon(Hwp3DrawingObjectCommonHeader, Hwp3DrawingExtendedPolygon),
    Unknown(Hwp3DrawingObjectCommonHeader, Vec<u8>),
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingLine {
    pub info1_len: u32,
    pub shape_info: u32,
    pub info2_len: u32,
}

impl Hwp3DrawingLine {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        Ok(Hwp3DrawingLine {
            info1_len: reader.read_u32::<LittleEndian>()?,
            shape_info: reader.read_u32::<LittleEndian>()?,
            info2_len: reader.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingArc {
    pub info1_len: u32,
    pub shape_info: u32,
    pub info2_len: u32,
}

impl Hwp3DrawingArc {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        Ok(Hwp3DrawingArc {
            info1_len: reader.read_u32::<LittleEndian>()?,
            shape_info: reader.read_u32::<LittleEndian>()?,
            info2_len: reader.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingPolygon {
    pub info1_len: u32,
    pub point_count: u32,
    pub info2_len: u32,
    pub points: Vec<[i32; 2]>,
}

impl Hwp3DrawingPolygon {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let info1_len = reader.read_u32::<LittleEndian>()?;
        let point_count = reader.read_u32::<LittleEndian>()?;
        let info2_len = reader.read_u32::<LittleEndian>()?;
        super::check_record_count(point_count as usize)?;
        let mut points = Vec::with_capacity(point_count as usize);
        for _ in 0..point_count {
            points.push([
                reader.read_i32::<LittleEndian>()?,
                reader.read_i32::<LittleEndian>()?,
            ]);
        }
        Ok(Hwp3DrawingPolygon {
            info1_len,
            point_count,
            info2_len,
            points,
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingTextBox {
    pub info1_len: u32,
    pub info2_len: u32,
    pub paragraph_list_data: Vec<u8>,
}

impl Hwp3DrawingTextBox {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let info1_len = reader.read_u32::<LittleEndian>()?;
        let info2_len = reader.read_u32::<LittleEndian>()?;
        let mut paragraph_list_data = super::alloc_record_buf(info2_len as usize)?;
        if info2_len > 0 {
            reader.read_exact(&mut paragraph_list_data)?;
        }
        Ok(Hwp3DrawingTextBox {
            info1_len,
            info2_len,
            paragraph_list_data,
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingCurve {
    pub info1_len: u32,
    pub point_count: u32,
    pub info2_len: u32,
    pub points: Vec<[i32; 2]>,
}

impl Hwp3DrawingCurve {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let info1_len = reader.read_u32::<LittleEndian>()?;
        let point_count = reader.read_u32::<LittleEndian>()?;
        let info2_len = reader.read_u32::<LittleEndian>()?;
        super::check_record_count(point_count as usize)?;
        let mut points = Vec::with_capacity(point_count as usize);
        for _ in 0..point_count {
            points.push([
                reader.read_i32::<LittleEndian>()?,
                reader.read_i32::<LittleEndian>()?,
            ]);
        }
        Ok(Hwp3DrawingCurve {
            info1_len,
            point_count,
            info2_len,
            points,
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingModifiedEllipse {
    pub info1_len: u32,
    pub arc_bounds: [i32; 4],
    pub info2_len: u32,
}

impl Hwp3DrawingModifiedEllipse {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        Ok(Hwp3DrawingModifiedEllipse {
            info1_len: reader.read_u32::<LittleEndian>()?,
            arc_bounds: [
                reader.read_i32::<LittleEndian>()?,
                reader.read_i32::<LittleEndian>()?,
                reader.read_i32::<LittleEndian>()?,
                reader.read_i32::<LittleEndian>()?,
            ],
            info2_len: reader.read_u32::<LittleEndian>()?,
        })
    }
}

#[derive(Debug, Default)]
pub struct Hwp3DrawingExtendedPolygon {
    pub info1_len: u32,
    pub point_count: u32,
    pub info2_len: u32,
    pub points: Vec<[i32; 2]>,
    pub line_attrs: Vec<u8>,
}

impl Hwp3DrawingExtendedPolygon {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let info1_len = reader.read_u32::<LittleEndian>()?;
        let point_count = reader.read_u32::<LittleEndian>()?;
        let info2_len = reader.read_u32::<LittleEndian>()?;
        super::check_record_count(point_count as usize)?;
        let mut points = Vec::with_capacity(point_count as usize);
        for _ in 0..point_count {
            points.push([
                reader.read_i32::<LittleEndian>()?,
                reader.read_i32::<LittleEndian>()?,
            ]);
        }
        let mut line_attrs = super::alloc_record_buf(point_count as usize)?;
        if point_count > 0 {
            reader.read_exact(&mut line_attrs)?;
        }
        Ok(Hwp3DrawingExtendedPolygon {
            info1_len,
            point_count,
            info2_len,
            points,
            line_attrs,
        })
    }
}

impl Hwp3DrawingObject {
    pub fn read<R: Read + Seek>(mut reader: R) -> Result<Self, io::Error> {
        let header = Hwp3DrawingObjectCommonHeader::read(&mut reader)?;

        // 글상자(6)인 경우, 공통 헤더 바로 뒤에 글상자 정보가 위치함.
        // 테이블 78 "글상자 세부 정보"에 따라 info1_len, info2_len, 문단 리스트가 존재함.
        // 이는 아래에서 처리됨.

        match header.object_type {
            0 => {
                // 컨테이너: 추가 세부 길이 정보 없음
                Ok(Hwp3DrawingObject::Container(header))
            }
            1 => {
                let details = Hwp3DrawingLine::read(&mut reader)?;
                Ok(Hwp3DrawingObject::Line(header, details))
            }
            2 => {
                // 사각형: 세부 정보가 없으면 0으로 채워진 8바이트. 테이블 73에 info1_len=0, info2_len=0으로 명시됨.
                // 단순한 도형의 경우 8바이트를 읽고 무시함.
                let _info1_len = reader.read_u32::<LittleEndian>()?;
                let _info2_len = reader.read_u32::<LittleEndian>()?;
                Ok(Hwp3DrawingObject::Rectangle(header))
            }
            3 => {
                // 타원: 0으로 채워진 8바이트
                let _info1_len = reader.read_u32::<LittleEndian>()?;
                let _info2_len = reader.read_u32::<LittleEndian>()?;
                Ok(Hwp3DrawingObject::Ellipse(header))
            }
            4 => {
                let details = Hwp3DrawingArc::read(&mut reader)?;
                Ok(Hwp3DrawingObject::Arc(header, details))
            }
            5 => {
                let details = Hwp3DrawingPolygon::read(&mut reader)?;
                Ok(Hwp3DrawingObject::Polygon(header, details))
            }
            6 => {
                let details = Hwp3DrawingTextBox::read(&mut reader)?;
                // 글상자일 경우 공통 헤더 뒤에 글상자 정보가 저장된다...
                // 세부 정보가 존재하지 않을 때는 길이 값들이 0이 되어 8개의 연속된 0으로 표현된다.
                // 테이블 78이 글상자의 세부 정보이므로, 세부 정보를 이미 읽었다고 가정함.
                Ok(Hwp3DrawingObject::TextBox(header, details))
            }
            7 => {
                let details = Hwp3DrawingCurve::read(&mut reader)?;
                Ok(Hwp3DrawingObject::Curve(header, details))
            }
            8 => {
                let details = Hwp3DrawingModifiedEllipse::read(&mut reader)?;
                Ok(Hwp3DrawingObject::ModifiedEllipse(header, details))
            }
            9 => {
                // 수정된 호 (회전을 위해 확장된 호): 스펙 11.3.4절에 따라 추가
                // 세부 정보가 전혀 없다. 공통 헤더의 회전 속성(평행사변형
                // 세 점)만으로 첫 점에서 끝 점 방향의 호를 그리므로, 사각형/
                // 타원(타입 2/3)처럼 8바이트 placeholder를 읽으면 안 된다.
                Ok(Hwp3DrawingObject::ModifiedArc(header))
            }
            10 => {
                let details = Hwp3DrawingExtendedPolygon::read(&mut reader)?;
                Ok(Hwp3DrawingObject::ExtendedCurve(header, details))
            }
            11 => {
                // 닫힌 다각형이 11일 것으로 추정. 명세서에 번호가 명시되지 않음.
                // 실제로 명세서에는 10은 "확장된 곡선"이며, "닫혀진 다각형"은 테이블에 ID가 없음.
                // 확장된 다각형과 비슷하게 처리한다고 가정함.
                let details = Hwp3DrawingExtendedPolygon::read(&mut reader)?;
                Ok(Hwp3DrawingObject::ClosedPolygon(header, details))
            }
            _ => {
                // 알 수 없는 객체
                let info1_len = reader.read_u32::<LittleEndian>()?;
                let mut info1 = super::alloc_record_buf(info1_len as usize)?;
                reader.read_exact(&mut info1)?;
                let info2_len = reader.read_u32::<LittleEndian>()?;
                let mut info2 = super::alloc_record_buf(info2_len as usize)?;
                reader.read_exact(&mut info2)?;

                let mut all_data = Vec::new();
                all_data.extend(info1);
                all_data.extend(info2);
                Ok(Hwp3DrawingObject::Unknown(header, all_data))
            }
        }
    }
}

use crate::model::shape::{
    ArcShape, CommonObjAttr, CurveShape, DrawingObjAttr, EllipseShape, GroupShape, LineShape,
    PolygonShape, RectangleShape, ShapeComponentAttr, ShapeObject, TextBox,
};
use crate::model::style::{Fill, FillType, ShapeBorderLine};
use crate::model::Padding;
use crate::parser::hwp3::Hwp3Error;
use std::collections::HashMap;

const HWP3_UNIT_SCALE: i32 = 4;

/// 신뢰할 수 없는 파일에서 읽은 HWP3 raw margin(u32)을 `* HWP3_UNIT_SCALE` 스케일 후
/// `i16` 필드(`TextBox::margin_*`)에 담는다. 곱셈이 `i32`/`i16` 범위를 넘으면 그대로
/// 캐스팅하는 대신 클램프해 오버플로 panic(malformed/fuzzed 파일에서의 DoS)을 막는다.
fn hwp3_margin_to_i16(raw_margin: u32) -> i16 {
    let scaled = raw_margin as i64 * HWP3_UNIT_SCALE as i64;
    scaled.clamp(i16::MIN as i64, i16::MAX as i64) as i16
}

pub fn parse_drawing_object_tree(
    cursor: &mut std::io::Cursor<&[u8]>,
    doc_char_shapes: &mut Vec<crate::model::style::CharShape>,
    doc_para_shapes: &mut Vec<crate::model::style::ParaShape>,
    doc_border_fills: &mut Vec<crate::model::style::BorderFill>,
    doc_tab_defs: &mut Vec<crate::model::style::TabDef>,
    pic_name_to_id: &mut HashMap<String, u16>,
) -> Result<ShapeObject, Hwp3Error> {
    let frame_header = Hwp3DrawingObjectFrameHeader::read(&mut *cursor)
        .map_err(|e| Hwp3Error::IoError { source: e })?;

    if frame_header.header_length > 24 {
        let _hypertext = Hwp3DrawingObjectHypertextInfo::read(&mut *cursor)
            .map_err(|e| Hwp3Error::IoError { source: e })?;
    }

    if frame_header.object_count == 0 {
        return Err(Hwp3Error::ParseError {
            message: "Drawing object has 0 objects".to_string(),
        });
    }

    let mut root_nodes = parse_shape_list(
        cursor,
        doc_char_shapes,
        doc_para_shapes,
        doc_border_fills,
        doc_tab_defs,
        pic_name_to_id,
        0,
    )?;

    if root_nodes.is_empty() {
        return Err(Hwp3Error::ParseError {
            message: "Failed to parse any root drawing objects".to_string(),
        });
    }

    if root_nodes.len() == 1 {
        Ok(root_nodes.remove(0))
    } else {
        let mut group = GroupShape::default();
        group.children = root_nodes;
        Ok(ShapeObject::Group(group))
    }
}

/// [#4285] `has_child`(connection_info bit 1)는 파일에서 그대로 온 값이라,
/// 재귀 깊이에 상한이 없으면 중첩된 Container 객체 체인 하나로 네이티브
/// 스택을 고갈시켜 프로세스를 죽일 수 있다(패닉과 달리 catch_unwind로 못
/// 잡음). 최소 92바이트짜리 Container 객체를 수만 겹 중첩해도
/// HWP3_MAX_RECORD_SIZE(256MiB) 안에 들어간다. HmlLimits::max_depth와 같은
/// 취지로 상한을 둔다.
const MAX_DRAWING_OBJECT_DEPTH: u32 = 256;

fn parse_shape_list(
    cursor: &mut std::io::Cursor<&[u8]>,
    doc_char_shapes: &mut Vec<crate::model::style::CharShape>,
    doc_para_shapes: &mut Vec<crate::model::style::ParaShape>,
    doc_border_fills: &mut Vec<crate::model::style::BorderFill>,
    doc_tab_defs: &mut Vec<crate::model::style::TabDef>,
    pic_name_to_id: &mut HashMap<String, u16>,
    depth: u32,
) -> Result<Vec<ShapeObject>, Hwp3Error> {
    if depth > MAX_DRAWING_OBJECT_DEPTH {
        return Err(Hwp3Error::ParseError {
            message: format!(
                "Drawing object nesting exceeds {} levels",
                MAX_DRAWING_OBJECT_DEPTH
            ),
        });
    }
    let mut list = Vec::new();
    loop {
        let raw_obj =
            Hwp3DrawingObject::read(&mut *cursor).map_err(|e| Hwp3Error::IoError { source: e })?;

        let (mut node, connection_info) = map_to_shape_object(
            raw_obj,
            doc_char_shapes,
            doc_para_shapes,
            doc_border_fills,
            doc_tab_defs,
            pic_name_to_id,
        )?;

        let has_sibling = (connection_info & 0x01) != 0;
        let has_child = (connection_info & 0x02) != 0;

        if has_child {
            let children = parse_shape_list(
                cursor,
                doc_char_shapes,
                doc_para_shapes,
                doc_border_fills,
                doc_tab_defs,
                pic_name_to_id,
                depth + 1,
            )?;
            if let ShapeObject::Group(ref mut g) = node {
                g.children = children;
            } else {
                eprintln!("HWP3 그리기 객체에서 컨테이너가 아닌 도형이 자식을 가짐");
            }
        }

        list.push(node);

        if !has_sibling {
            break;
        }
    }
    Ok(list)
}

/// HWP3 사각형의 무색·무늬 없음 sentinel을 채우기 없음으로 판정한다.
///
/// `fill_color`의 high byte는 표 69의 RGB 영역이 아니다. 실제 HWP3 원본에서
/// 사각형의 `0x10000000`은 테두리만 남기는 no-fill 표식으로 사용된다. 반면 같은
/// 표식이 글상자 기본 스타일에도 나타나므로 개체 종류와 무늬/그라데이션 부재까지
/// 함께 확인한다.
fn hwp3_rectangle_uses_no_fill_marker(header: &Hwp3DrawingObjectCommonHeader) -> bool {
    header.object_type == 2
        && header.basic_attr.fill_color == 0x1000_0000
        && header.basic_attr.pattern_type == 0
        && header.gradient_attr.is_none()
}

/// HWP3 선 색상의 무색 sentinel을 테두리 없음으로 판정한다.
///
/// `0x10000000`은 검정 RGB 값이 아니다. HWP3 원본의 글자처럼 취급된 사각형과
/// 여백 구분선에서 이 값은 "선 없음"을 뜻한다. 기존의 `line_width > 0` 보정이
/// 이 sentinel도 검정 실선으로 승격해 한컴/HWP5 변환본에는 없는 테두리를 그렸다.
/// 실제 검정은 `0x00000000`이므로 구분할 수 있다.
fn hwp3_uses_no_line_marker(header: &Hwp3DrawingObjectCommonHeader) -> bool {
    header.basic_attr.line_color == 0x1000_0000
}

fn map_to_shape_object(
    raw: Hwp3DrawingObject,
    doc_char_shapes: &mut Vec<crate::model::style::CharShape>,
    doc_para_shapes: &mut Vec<crate::model::style::ParaShape>,
    doc_border_fills: &mut Vec<crate::model::style::BorderFill>,
    doc_tab_defs: &mut Vec<crate::model::style::TabDef>,
    pic_name_to_id: &mut HashMap<String, u16>,
) -> Result<(ShapeObject, u16), Hwp3Error> {
    let mut parsed_paragraphs = Vec::new();

    let (header, shape) = match raw {
        Hwp3DrawingObject::Container(hdr) => (hdr, ShapeObject::Group(GroupShape::default())),
        Hwp3DrawingObject::Line(hdr, _details) => (hdr, ShapeObject::Line(LineShape::default())),
        Hwp3DrawingObject::Rectangle(hdr) => {
            (hdr, ShapeObject::Rectangle(RectangleShape::default()))
        }
        Hwp3DrawingObject::Ellipse(hdr) => (hdr, ShapeObject::Ellipse(EllipseShape::default())),
        Hwp3DrawingObject::Arc(hdr, _details) => (hdr, ShapeObject::Arc(ArcShape::default())),
        Hwp3DrawingObject::Polygon(hdr, _details) => {
            (hdr, ShapeObject::Polygon(PolygonShape::default()))
        }
        Hwp3DrawingObject::TextBox(hdr, details) => {
            if details.info2_len > 0 {
                let mut text_cursor = std::io::Cursor::new(details.paragraph_list_data.as_slice());
                let paras = crate::parser::hwp3::parse_paragraph_list(
                    &mut text_cursor,
                    doc_char_shapes,
                    doc_para_shapes,
                    doc_border_fills,
                    doc_tab_defs,
                    pic_name_to_id,
                    0,            // body_left_hu: 드로잉 내부 텍스트, wrap zone 불필요
                    i32::MAX / 2, // column_width_hu
                    0,            // body_height_hu: 도형 내부 텍스트는 본문 페이지 분할 제외
                    false,        // 복호화 원본의 본문 Square-wrap 계약은 적용하지 않음
                )?;
                parsed_paragraphs = paras;
            }
            (hdr, ShapeObject::Rectangle(RectangleShape::default()))
        }
        Hwp3DrawingObject::Curve(hdr, _details) => (hdr, ShapeObject::Curve(CurveShape::default())),
        Hwp3DrawingObject::ModifiedEllipse(hdr, _details) => {
            (hdr, ShapeObject::Ellipse(EllipseShape::default()))
        }
        Hwp3DrawingObject::ModifiedArc(hdr) => (hdr, ShapeObject::Arc(ArcShape::default())),
        Hwp3DrawingObject::ExtendedCurve(hdr, _details) => {
            (hdr, ShapeObject::Curve(CurveShape::default()))
        }
        Hwp3DrawingObject::ClosedPolygon(hdr, _details) => {
            (hdr, ShapeObject::Polygon(PolygonShape::default()))
        }
        Hwp3DrawingObject::Unknown(hdr, _data) => (hdr, ShapeObject::Group(GroupShape::default())),
    };

    let connection_info = header.connection_info;
    let mut final_shape = shape;

    let common = CommonObjAttr {
        width: header.object_size[0].saturating_mul(HWP3_UNIT_SCALE as u32),
        height: header.object_size[1].saturating_mul(HWP3_UNIT_SCALE as u32),
        ..Default::default()
    };

    let mut rotation_angle = 0i16;
    if let Some(ref rot) = header.rotation_attr {
        let x0 = rot.parallelogram[0] as f64;
        let y0 = rot.parallelogram[1] as f64;
        let x1 = rot.parallelogram[2] as f64;
        let y1 = rot.parallelogram[3] as f64;

        let dx = x1 - x0;
        let dy = y1 - y0;
        if dx != 0.0 || dy != 0.0 {
            let mut angle = dy.atan2(dx) * 180.0 / std::f64::consts::PI;
            if angle < 0.0 {
                angle += 360.0;
            }
            rotation_angle = angle.round() as i16;
        }
    }

    let shape_attr = ShapeComponentAttr {
        offset_x: (header.relative_pos[0] as i64 * HWP3_UNIT_SCALE as i64)
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        offset_y: (header.relative_pos[1] as i64 * HWP3_UNIT_SCALE as i64)
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        original_width: header.object_size[0].saturating_mul(HWP3_UNIT_SCALE as u32),
        original_height: header.object_size[1].saturating_mul(HWP3_UNIT_SCALE as u32),
        current_width: header.object_size[0].saturating_mul(HWP3_UNIT_SCALE as u32),
        current_height: header.object_size[1].saturating_mul(HWP3_UNIT_SCALE as u32),
        rotation_angle,
        ..Default::default()
    };

    let border_line = ShapeBorderLine {
        color: header.basic_attr.line_color,
        width: (header.basic_attr.line_width as i64 * HWP3_UNIT_SCALE as i64)
            .clamp(i32::MIN as i64, i32::MAX as i64) as i32,
        // [Task #877 Stage 3] HWP3 drawing line_style = 0 (= "선 종류 없음") 인데
        // line_width > 0 인 경우 → 실제 한컴 viewer 는 실선으로 표시. (sample16 RFP
        // 박스 외곽선 회귀: raw line_style=0, line_width=84, line_color=0 검정)
        // 렌더러 [renderer/layout/utils.rs:163] 의 `attr & 0x3F == 0` 시 외곽선 미표시
        // 규칙에 맞추기 위해 bit 0..5 = 1 (Solid LineType) 보강.
        //
        // [Task #1008 격차 B] HWP3 raw line_style 의 LineType=2~7 (점선/일점쇄선
        // 등) 도 한컴 viewer 는 실선으로 렌더 (sample16 pi=71 사업개요 박스 raw
        // line_style=2 → 한컴 정답 = 실선). HWP3 native LineType 변형은 spec
        // 상 존재하나 한컴 동작은 일관 solid — 작업지시자 한컴 한글 정답지 시각
        // 정답 단언. HWP3 sample 분포 sweep: line_style=2 는 sample16 한정 (다른
        // fixture: 0/1 만), narrow fix 회귀 risk 0. HWP3 한정 (HWP5/HWPX 무영향).
        attr: if hwp3_uses_no_line_marker(&header) {
            // line_width가 남아 있어도 HWP3 무색 sentinel은 선을 그리지 않는다.
            0
        } else {
            let raw_attr = header.basic_attr.line_style as u32;
            let line_type = raw_attr & 0x3F;
            if line_type == 0 && header.basic_attr.line_width > 0 {
                raw_attr | 0x01
            } else if (2..=7).contains(&line_type) {
                // HWP3 의 LineType 2~7 을 1 (Solid) 로 normalize
                (raw_attr & !0x3F) | 0x01
            } else {
                raw_attr
            }
        },
        outline_style: 0,
    };

    let fill = if hwp3_rectangle_uses_no_fill_marker(&header) {
        // HWP3 사각형의 `0x10000000`은 RGB가 아니라 "채우기 없음" sentinel이다.
        // 이 값을 흰색 단색 채움으로 대체하면, 아이콘 위에 놓인 테두리 사각형이
        // 아이콘을 가려 버린다. 한컴 HWP5 변환본도 같은 사각형을 Fill=None으로
        // 기록한다. 글상자(type 6)는 같은 high byte를 서로 다른 기본 스타일에
        // 사용하므로, 사각형·무늬 없음·비그라데이션 조합으로 한정한다.
        Fill::default()
    } else {
        let raw_fc = header.basic_attr.fill_color;
        let fill_flag = (raw_fc >> 24) & 0xFF;
        let fill_rgb = raw_fc & 0x00FFFFFF;
        // HWP3 글상자 등은 high-byte marker와 RGB=0을 기본 흰색 면으로 사용한다.
        // 사각형의 no-fill marker만 위 분기에서 분리하고, 기존 가시성 보정은 유지한다.
        let effective_rgb = if fill_flag != 0 && fill_rgb == 0 {
            0x00FFFFFF
        } else {
            fill_rgb
        };

        // [Task #1008 격차 A] HWP3 gradient_attr 이 파싱된 경우 IR Fill.gradient 에 매핑.
        // HWP3 raw stream 의 Hwp3DrawingObjectGradientAttr (drawing.rs:149~170) 은 이미
        // basic_attr.has_gradient() 시 파싱되어 header.gradient_attr 에 보존되지만, 종전
        // 코드는 fill_type 을 항상 Solid 로 하드코딩하여 데이터가 무시되었음. HWP5 의
        // doc_info.rs:404 매핑과 동일 contract 로 IR 주입 (step→blur, 2-stop colors,
        // positions=vec![] → renderer 가 균등 분포).
        let (fill_type, gradient) = if let Some(g) = header.gradient_attr.as_ref() {
            let grad = crate::model::style::GradientFill {
                gradient_type: g.kind as i16,
                angle: g.angle as i16,
                center_x: g.center_x as i16,
                center_y: g.center_y as i16,
                blur: g.step as i16,
                step_center: 0,
                colors: vec![g.start_color, g.end_color],
                positions: vec![],
            };
            (crate::model::style::FillType::Gradient, Some(grad))
        } else {
            (crate::model::style::FillType::Solid, None)
        };
        Fill {
            fill_type,
            solid: Some(crate::model::style::SolidFill {
                background_color: effective_rgb,
                pattern_color: header.basic_attr.pattern_color,
                pattern_type: header.basic_attr.pattern_type as i32,
            }),
            gradient,
            image: None,
            // [Task #877 Stage 4] 한컴 호환 alpha convention: 0=불투명, 255=완전 투명.
            // (renderer/layout/utils.rs:199 의 opacity 식: opacity = 1 - alpha/255)
            // 기존 alpha=255 → opacity=0 → SVG <rect opacity="0.000"> 완전 투명 회귀.
            // HWP3 raw 에는 alpha 정보 없음, 한컴 viewer 의 default = 불투명 = alpha 0.
            alpha: 0,
        }
    };

    let text_box = if (header.basic_attr.options & (1 << 19)) != 0 || !parsed_paragraphs.is_empty()
    {
        Some(TextBox {
            margin_left: hwp3_margin_to_i16(header.basic_attr.textbox_margin[0]),
            margin_top: hwp3_margin_to_i16(header.basic_attr.textbox_margin[1]),
            margin_right: hwp3_margin_to_i16(header.basic_attr.textbox_margin[0]),
            margin_bottom: hwp3_margin_to_i16(header.basic_attr.textbox_margin[1]),
            paragraphs: parsed_paragraphs,
            ..Default::default()
        })
    } else {
        None
    };

    let drawing_attr = DrawingObjAttr {
        shape_attr,
        border_line,
        fill,
        text_box,
        ..Default::default()
    };

    match final_shape {
        ShapeObject::Line(ref mut s) => {
            s.common = common;
            s.drawing = drawing_attr;
        }
        ShapeObject::Rectangle(ref mut s) => {
            s.common = common;
            s.drawing = drawing_attr;
        }
        ShapeObject::Ellipse(ref mut s) => {
            s.common = common;
            s.drawing = drawing_attr;
        }
        ShapeObject::Arc(ref mut s) => {
            s.common = common;
            s.drawing = drawing_attr;
        }
        ShapeObject::Polygon(ref mut s) => {
            s.common = common;
            s.drawing = drawing_attr;
        }
        ShapeObject::Curve(ref mut s) => {
            s.common = common;
            s.drawing = drawing_attr;
        }
        ShapeObject::Group(ref mut s) => {
            s.common = common;
            s.shape_attr = drawing_attr.shape_attr;
        }
        _ => {}
    }

    Ok((final_shape, connection_info))
}

#[cfg(test)]
mod modified_arc_overread_tests {
    use super::*;
    use std::io::Cursor;

    // [POC] textbox_margin/line_width/object_size 는 파일에서 그대로 읽은 신뢰
    // 불가 u32 값이다. `* HWP3_UNIT_SCALE(4)` 를 i32/i16 로 계산·캐스팅하는
    // 과정에서 큰 값(예: u32::MAX)이 들어오면 곱셈이 i32 오버플로를 일으켜
    // debug 빌드에서 panic한다(fuzzing/악성 파일 경로에서 서비스 거부).
    #[test]
    fn map_to_shape_object_does_not_panic_on_huge_margins() {
        let header = Hwp3DrawingObjectCommonHeader {
            object_type: 6, // TextBox
            object_size: [u32::MAX, u32::MAX],
            relative_pos: [u32::MAX, u32::MAX],
            basic_attr: Hwp3DrawingObjectBasicAttr {
                line_width: u32::MAX,
                textbox_margin: [u32::MAX, u32::MAX],
                ..Default::default()
            },
            ..Default::default()
        };
        let raw = Hwp3DrawingObject::TextBox(
            header,
            Hwp3DrawingTextBox {
                info1_len: 0,
                info2_len: 0,
                paragraph_list_data: Vec::new(),
            },
        );
        let mut doc_char_shapes = Vec::new();
        let mut doc_para_shapes = Vec::new();
        let mut doc_border_fills = Vec::new();
        let mut doc_tab_defs = Vec::new();
        let mut pic_name_to_id = HashMap::new();
        let result = map_to_shape_object(
            raw,
            &mut doc_char_shapes,
            &mut doc_para_shapes,
            &mut doc_border_fills,
            &mut doc_tab_defs,
            &mut pic_name_to_id,
        );
        assert!(
            result.is_ok(),
            "거대한 margin/width 값에서도 panic 없이 처리되어야 함"
        );
    }

    #[test]
    fn rectangle_no_fill_marker_does_not_apply_to_text_boxes() {
        let rectangle = Hwp3DrawingObjectCommonHeader {
            object_type: 2,
            basic_attr: Hwp3DrawingObjectBasicAttr {
                fill_color: 0x1000_0000,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(hwp3_rectangle_uses_no_fill_marker(&rectangle));

        let text_box = Hwp3DrawingObjectCommonHeader {
            object_type: 6,
            basic_attr: Hwp3DrawingObjectBasicAttr {
                fill_color: 0x1000_0000,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(
            !hwp3_rectangle_uses_no_fill_marker(&text_box),
            "글상자의 동일 marker는 기본 스타일이지 사각형 no-fill이 아님"
        );
    }

    #[test]
    fn no_line_marker_is_distinct_from_black_line() {
        let no_line = Hwp3DrawingObjectCommonHeader {
            basic_attr: Hwp3DrawingObjectBasicAttr {
                line_color: 0x1000_0000,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(hwp3_uses_no_line_marker(&no_line));

        let black_line = Hwp3DrawingObjectCommonHeader {
            basic_attr: Hwp3DrawingObjectBasicAttr {
                line_color: 0x0000_0000,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(
            !hwp3_uses_no_line_marker(&black_line),
            "실제 검정 선은 sentinel이 아니므로 기존 line-style 보정을 유지한다"
        );
    }

    // [Task #2824] 변형된 호(object_type=9)는 스펙 11.3.4절에 따라 공통 헤더
    // 외에 추가 세부 정보가 전혀 없어야 한다. 수정 전 코드는 존재하지 않는
    // 8바이트(info1_len, info2_len)를 읽어 버려서, 뒤따르는 형제 레코드의
    // 선두 바이트를 침범했다. 공통 헤더 크기만큼만 커서가 전진하는지 확인한다.
    #[test]
    fn modified_arc_does_not_overread_past_common_header() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&0u32.to_le_bytes()); // header_length
        buf.extend_from_slice(&9u16.to_le_bytes()); // object_type = 9 (변형된 호)
        buf.extend_from_slice(&0u16.to_le_bytes()); // connection_info
        buf.extend_from_slice(&[0u8; 8]); // relative_pos
        buf.extend_from_slice(&[0u8; 8]); // object_size
        buf.extend_from_slice(&[0u8; 8]); // absolute_pos
        buf.extend_from_slice(&[0u8; 16]); // bounds
        buf.extend_from_slice(&[0u8; 32]); // basic_attr: line_style..pattern_color (8 x u32)
        buf.extend_from_slice(&[0u8; 8]); // basic_attr: textbox_margin
        buf.extend_from_slice(&0u32.to_le_bytes()); // basic_attr: options = 0 (no rotation/gradient/bitmap)
        let common_header_len = buf.len() as u64;

        // 다음 형제 레코드의 선두 바이트라고 가정한 마커. 수정 전 코드는 이
        // 8바이트를 info1_len/info2_len으로 잘못 소비한다.
        buf.extend_from_slice(&0xAAAAAAAAu32.to_le_bytes());
        buf.extend_from_slice(&0xBBBBBBBBu32.to_le_bytes());

        let mut cursor = Cursor::new(buf);
        let obj = Hwp3DrawingObject::read(&mut cursor).expect("parse modified arc");

        assert!(matches!(obj, Hwp3DrawingObject::ModifiedArc(_)));
        assert_eq!(
            cursor.position(),
            common_header_len,
            "ModifiedArc 파싱이 공통 헤더 이후 존재하지 않는 바이트를 소비함"
        );
    }
}

#[cfg(test)]
mod hypertext_bookmark_underread_tests {
    use super::*;
    use std::io::Cursor;

    // [Task #2831] 스펙 8.3절 표 21에 따라 "건너뛸 책갈피"는 hchar array[16] = 32바이트다.
    // 수정 전 코드는 16바이트만 읽어, 뒤따르는 필드(매크로/종류/예약)의 선두를
    // 책갈피의 나머지 절반으로 오인하고 읽어버렸다. 32바이트 전체를 소비한 뒤
    // 정확히 마커 위치에 도달하는지 확인한다.
    #[test]
    fn hypertext_info_consumes_full_32_byte_bookmark() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&621u32.to_le_bytes()); // length
        buf.extend_from_slice(&[0u8; 256]); // jump_file_name
        buf.extend_from_slice(&[0u8; 32]); // jump_bookmark (32바이트 전체)
        buf.extend_from_slice(&[0u8; 325]); // macro_data
        buf.push(0u8); // kind
        buf.extend_from_slice(&[0u8; 3]); // reserved
        let hypertext_len = buf.len() as u64;

        // 다음 필드라고 가정한 마커. 수정 전 코드는 이 마커의 앞부분을
        // 책갈피 뒷부분으로 잘못 소비한다.
        buf.extend_from_slice(&0xCCCCCCCCu32.to_le_bytes());

        let mut cursor = Cursor::new(buf);
        let _info =
            Hwp3DrawingObjectHypertextInfo::read(&mut cursor).expect("parse hypertext info");

        assert_eq!(
            cursor.position(),
            hypertext_len,
            "하이퍼텍스트 정보 파싱이 책갈피 필드를 32바이트로 소비하지 않음"
        );
    }
}

#[cfg(test)]
mod drawing_object_recursion_depth_tests {
    use super::*;
    use std::io::Cursor;

    /// object_type=0(Container)에 connection_info=0x0002(has_child, no
    /// sibling)만 실은 최소 92바이트 공통 헤더를 만든다.
    fn container_block() -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&0u32.to_le_bytes()); // header_length
        buf.extend_from_slice(&0u16.to_le_bytes()); // object_type = 0 (Container)
        buf.extend_from_slice(&0x0002u16.to_le_bytes()); // connection_info: has_child, !has_sibling
        buf.extend_from_slice(&[0u8; 8]); // relative_pos
        buf.extend_from_slice(&[0u8; 8]); // object_size
        buf.extend_from_slice(&[0u8; 8]); // absolute_pos
        buf.extend_from_slice(&[0u8; 16]); // bounds
        buf.extend_from_slice(&[0u8; 32]); // basic_attr: line_style..pattern_color
        buf.extend_from_slice(&[0u8; 8]); // basic_attr: textbox_margin
        buf.extend_from_slice(&0u32.to_le_bytes()); // basic_attr: options
        buf
    }

    // [#4285] has_child 는 파일에서 그대로 온 값이라 재귀 깊이 상한이 없으면
    // Container 객체를 깊이 중첩한 파일 하나로 네이티브 스택을 고갈시켜
    // 프로세스를 죽인다(catch_unwind로 못 잡음). MAX_DRAWING_OBJECT_DEPTH를
    // 넘는 중첩이 패닉/abort 대신 파싱 오류로 거부되는지 확인한다.
    #[test]
    fn deeply_nested_container_chain_is_rejected_not_stack_overflowed() {
        let mut buf = Vec::new();
        buf.extend_from_slice(&24u32.to_le_bytes()); // frame header_length (<=24: 하이퍼텍스트 없음)
        buf.extend_from_slice(&0u32.to_le_bytes()); // z_order
        buf.extend_from_slice(&1u32.to_le_bytes()); // object_count
        buf.extend_from_slice(&[0u8; 16]); // bounds

        for _ in 0..(MAX_DRAWING_OBJECT_DEPTH as usize + 4) {
            buf.extend_from_slice(&container_block());
        }

        let mut doc_char_shapes = Vec::new();
        let mut doc_para_shapes = Vec::new();
        let mut doc_border_fills = Vec::new();
        let mut doc_tab_defs = Vec::new();
        let mut pic_name_to_id = HashMap::new();

        let mut cursor = Cursor::new(buf.as_slice());
        let result = parse_drawing_object_tree(
            &mut cursor,
            &mut doc_char_shapes,
            &mut doc_para_shapes,
            &mut doc_border_fills,
            &mut doc_tab_defs,
            &mut pic_name_to_id,
        );

        assert!(
            result.is_err(),
            "상한을 넘는 중첩은 패닉 대신 오류로 거부되어야 함"
        );
    }
}
