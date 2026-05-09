//! HWP3 그리기 객체 파싱
//! 
//! HWP3 파일에 포함된 그리기 객체(선, 사각형, 타원, 그룹 등)를 파싱하여 렌더링 가능한 모델로 변환한다.
//! 그리기 객체의 계층 구조(트리)와 캡션, 속성 정보 등을 추출하는 역할을 한다.

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Read, Seek, SeekFrom};
use crate::parser::hwp3::encoding::decode_hwp3_string;

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
    pub jump_bookmark: String, // 16 hchar (보통 32 바이트지만 문서에 따라 16 바이트로 처리)
    pub macro_data: Vec<u8>, // 325 바이트
    pub kind: u8,
    pub reserved: [u8; 3],
}

impl Hwp3DrawingObjectHypertextInfo {
    pub fn read<R: Read>(mut reader: R) -> Result<Self, io::Error> {
        let length = reader.read_u32::<LittleEndian>()?;
        let mut jump_file_name_buf = [0u8; 256];
        reader.read_exact(&mut jump_file_name_buf)?;
        let jump_file_name = decode_hwp3_string(&jump_file_name_buf);

        let mut jump_bookmark_buf = [0u8; 16]; // 문서에는 16 hchar(32바이트)로 명시되어 있으나, 오프셋 계산상 16바이트로 처리함
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
        let mut paragraph_list_data = vec![0u8; info2_len as usize];
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
        let mut points = Vec::with_capacity(point_count as usize);
        for _ in 0..point_count {
            points.push([
                reader.read_i32::<LittleEndian>()?,
                reader.read_i32::<LittleEndian>()?,
            ]);
        }
        let mut line_attrs = vec![0u8; point_count as usize];
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
                // 수정된 호
                let _info1_len = reader.read_u32::<LittleEndian>()?;
                let _info2_len = reader.read_u32::<LittleEndian>()?;
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
                let mut info1 = vec![0u8; info1_len as usize];
                reader.read_exact(&mut info1)?;
                let info2_len = reader.read_u32::<LittleEndian>()?;
                let mut info2 = vec![0u8; info2_len as usize];
                reader.read_exact(&mut info2)?;
                
                let mut all_data = Vec::new();
                all_data.extend(info1);
                all_data.extend(info2);
                Ok(Hwp3DrawingObject::Unknown(header, all_data))
            }
        }
    }
}

use crate::model::shape::{ShapeObject, GroupShape, LineShape, RectangleShape, EllipseShape, ArcShape, PolygonShape, CurveShape, CommonObjAttr, DrawingObjAttr, ShapeComponentAttr, TextBox};
use crate::model::Padding;
use crate::model::style::{Fill, ShapeBorderLine, FillType};
use std::collections::HashMap;
use crate::parser::hwp3::Hwp3Error;

const HWP3_UNIT_SCALE: i32 = 4;

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

    let mut root_nodes = parse_shape_list(cursor, doc_char_shapes, doc_para_shapes, doc_border_fills, doc_tab_defs, pic_name_to_id)?;

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

fn parse_shape_list(
    cursor: &mut std::io::Cursor<&[u8]>,
    doc_char_shapes: &mut Vec<crate::model::style::CharShape>,
    doc_para_shapes: &mut Vec<crate::model::style::ParaShape>,
    doc_border_fills: &mut Vec<crate::model::style::BorderFill>,
    doc_tab_defs: &mut Vec<crate::model::style::TabDef>,
    pic_name_to_id: &mut HashMap<String, u16>,
) -> Result<Vec<ShapeObject>, Hwp3Error> {
    let mut list = Vec::new();
    loop {
        let raw_obj = Hwp3DrawingObject::read(&mut *cursor)
            .map_err(|e| Hwp3Error::IoError { source: e })?;

        let (mut node, connection_info) = map_to_shape_object(raw_obj, doc_char_shapes, doc_para_shapes, doc_border_fills, doc_tab_defs, pic_name_to_id)?;

        let has_sibling = (connection_info & 0x01) != 0;
        let has_child = (connection_info & 0x02) != 0;

        if has_child {
            let children = parse_shape_list(cursor, doc_char_shapes, doc_para_shapes, doc_border_fills, doc_tab_defs, pic_name_to_id)?;
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
        Hwp3DrawingObject::Container(hdr) => {
            (hdr, ShapeObject::Group(GroupShape::default()))
        }
        Hwp3DrawingObject::Line(hdr, _details) => {
            (hdr, ShapeObject::Line(LineShape::default()))
        }
        Hwp3DrawingObject::Rectangle(hdr) => {
            (hdr, ShapeObject::Rectangle(RectangleShape::default()))
        }
        Hwp3DrawingObject::Ellipse(hdr) => {
            (hdr, ShapeObject::Ellipse(EllipseShape::default()))
        }
        Hwp3DrawingObject::Arc(hdr, _details) => {
            (hdr, ShapeObject::Arc(ArcShape::default()))
        }
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
                    0,          // body_left_hu: 드로잉 내부 텍스트, wrap zone 불필요
                    i32::MAX / 2, // column_width_hu
                )?;
                parsed_paragraphs = paras;
            }
            (hdr, ShapeObject::Rectangle(RectangleShape::default()))
        }
        Hwp3DrawingObject::Curve(hdr, _details) => {
            (hdr, ShapeObject::Curve(CurveShape::default()))
        }
        Hwp3DrawingObject::ModifiedEllipse(hdr, _details) => {
            (hdr, ShapeObject::Ellipse(EllipseShape::default()))
        }
        Hwp3DrawingObject::ModifiedArc(hdr) => {
            (hdr, ShapeObject::Arc(ArcShape::default()))
        }
        Hwp3DrawingObject::ExtendedCurve(hdr, _details) => {
            (hdr, ShapeObject::Curve(CurveShape::default()))
        }
        Hwp3DrawingObject::ClosedPolygon(hdr, _details) => {
            (hdr, ShapeObject::Polygon(PolygonShape::default()))
        }
        Hwp3DrawingObject::Unknown(hdr, _data) => {
            (hdr, ShapeObject::Group(GroupShape::default()))
        }
    };

    let connection_info = header.connection_info;
    let mut final_shape = shape;

    let common = CommonObjAttr {
        width: (header.object_size[0] as u32 * HWP3_UNIT_SCALE as u32),
        height: (header.object_size[1] as u32 * HWP3_UNIT_SCALE as u32),
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
        offset_x: header.relative_pos[0] as i32 * HWP3_UNIT_SCALE,
        offset_y: header.relative_pos[1] as i32 * HWP3_UNIT_SCALE,
        original_width: (header.object_size[0] as u32 * HWP3_UNIT_SCALE as u32),
        original_height: (header.object_size[1] as u32 * HWP3_UNIT_SCALE as u32),
        current_width: (header.object_size[0] as u32 * HWP3_UNIT_SCALE as u32),
        current_height: (header.object_size[1] as u32 * HWP3_UNIT_SCALE as u32),
        rotation_angle,
        ..Default::default()
    };

    let border_line = ShapeBorderLine {
        color: header.basic_attr.line_color,
        width: header.basic_attr.line_width as i32 * HWP3_UNIT_SCALE,
        attr: header.basic_attr.line_style as u32,
        outline_style: 0,
    };

    let fill = Fill {
        fill_type: crate::model::style::FillType::Solid,
        solid: Some(crate::model::style::SolidFill {
            background_color: header.basic_attr.fill_color,
            pattern_color: header.basic_attr.pattern_color,
            pattern_type: header.basic_attr.pattern_type as i32,
        }),
        gradient: None,
        image: None,
        alpha: 255,
    };
    
    let text_box = if (header.basic_attr.options & (1 << 19)) != 0 || !parsed_paragraphs.is_empty() {
        Some(TextBox {
            margin_left: (header.basic_attr.textbox_margin[0] as i32 * HWP3_UNIT_SCALE) as i16,
            margin_top: (header.basic_attr.textbox_margin[1] as i32 * HWP3_UNIT_SCALE) as i16,
            margin_right: (header.basic_attr.textbox_margin[0] as i32 * HWP3_UNIT_SCALE) as i16,
            margin_bottom: (header.basic_attr.textbox_margin[1] as i32 * HWP3_UNIT_SCALE) as i16,
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
        ShapeObject::Line(ref mut s) => { s.common = common; s.drawing = drawing_attr; },
        ShapeObject::Rectangle(ref mut s) => { s.common = common; s.drawing = drawing_attr; },
        ShapeObject::Ellipse(ref mut s) => { s.common = common; s.drawing = drawing_attr; },
        ShapeObject::Arc(ref mut s) => { s.common = common; s.drawing = drawing_attr; },
        ShapeObject::Polygon(ref mut s) => { s.common = common; s.drawing = drawing_attr; },
        ShapeObject::Curve(ref mut s) => { s.common = common; s.drawing = drawing_attr; },
        ShapeObject::Group(ref mut s) => { s.common = common; s.shape_attr = drawing_attr.shape_attr; },
        _ => {}
    }

    Ok((final_shape, connection_info))
}
