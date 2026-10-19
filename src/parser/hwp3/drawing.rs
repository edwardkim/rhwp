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
    /// [#5558] 공통 헤더 뒤 글상자 정보(스펙 11.3 optional 블록, 표 78 형식)로 실려 온
    /// 내부 문단 리스트. 사각형 등 비-글상자 개체도 "글상자로 만들기"면 이 블록을 갖고,
    /// 이 빈티지 코퍼스는 그 전체 길이를 header_length 로 선언한다.
    pub textbox_paragraph_list: Option<Vec<u8>>,
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
            textbox_paragraph_list: None,
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

/// 공통 헤더의 선언 길이(header_length)와 플래그 유도 소비량이 어긋날 때 전진을
/// 허용하는 상한. 정상 헤더는 커야 400여 바이트이므로 이를 훨씬 넘는 선언은
/// 손상된 값으로 보고 종전대로 플래그 유도 위치를 유지한다.
const MAX_HEADER_SKIP: u64 = 64 * 1024;

impl Hwp3DrawingObject {
    pub fn read<R: Read + Seek>(mut reader: R) -> Result<Self, io::Error> {
        let header_start = reader.stream_position()?;
        let mut header = Hwp3DrawingObjectCommonHeader::read(&mut reader)?;

        // [#5141] 빈티지 파일의 공통 헤더에는 이 파서가 모르는 확장 필드가 붙을 수
        // 있고, 전체 길이는 header_length(자기 4바이트 제외)로 선언된다. 플래그
        // 유도로 읽은 소비량이 선언에 못 미치면 선언 끝까지 전진해야 뒤따르는
        // 세부 정보·형제·자식 스트림이 어긋나지 않는다. 선언이 스트림 밖이거나
        // 상한을 넘으면 손상 값으로 보고 종전 위치를 유지한다.
        //
        // [#5558] 그 잉여 구간의 실체는 스펙 11.3 의 optional 글상자 정보인 경우가
        // 대부분이다(표 78 형식: [정보1 길이][정보2 길이][문단 리스트]). 잉여 길이가
        // 표 78 과 정확히 맞아떨어지면 내부 문단 리스트를 회수하고, 아니면 종전대로
        // 건너뛴다(07615 실측: 잉여 보유 137개 중 136개 정합·1개는 3바이트 슬랙).
        // 글상자(type 6)는 세부 정보 경로가 같은 구조를 읽으므로 제외한다.
        let consumed_end = reader.stream_position()?;
        let declared_end = header_start + 4 + header.header_length as u64;
        if declared_end > consumed_end && declared_end - consumed_end <= MAX_HEADER_SKIP {
            let stream_len = reader.seek(SeekFrom::End(0))?;
            reader.seek(SeekFrom::Start(consumed_end))?;
            if declared_end <= stream_len {
                let surplus = declared_end - consumed_end;
                let mut recovered = false;
                if header.object_type != 6 && surplus >= 8 {
                    let info1_len = reader.read_u32::<LittleEndian>()?;
                    let info2_len = reader.read_u32::<LittleEndian>()?;
                    if u64::from(info1_len) + u64::from(info2_len) == surplus - 8 && info2_len > 0 {
                        if info1_len > 0 {
                            reader.seek(SeekFrom::Current(i64::from(info1_len)))?;
                        }
                        let mut list = super::alloc_record_buf(info2_len as usize)?;
                        reader.read_exact(&mut list)?;
                        header.textbox_paragraph_list = Some(list);
                        recovered = true;
                    }
                }
                if !recovered {
                    reader.seek(SeekFrom::Start(declared_end))?;
                }
            }
        }

        // 글상자(6)인 경우, 공통 헤더 바로 뒤에 글상자 정보가 위치함.
        // 테이블 78 "글상자 세부 정보"에 따라 info1_len, info2_len, 문단 리스트가 존재함.
        // 이는 아래에서 처리됨.

        match header.object_type {
            0 => {
                // [#5141] 컨테이너도 사각형/타원처럼 세부 정보 길이 8바이트
                // (info1_len=0, info2_len=0)를 가진다. 이를 읽지 않으면 자식 파싱이
                // 8바이트 어긋나 첫 자식이 hdr_len=0·conn=0 인 가짜 컨테이너로 읽히고
                // 묶음 자식 전체가 소실된다.
                let _info1_len = reader.read_u32::<LittleEndian>()?;
                let _info2_len = reader.read_u32::<LittleEndian>()?;
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
use crate::model::style::{Fill, ShapeBorderLine};
use crate::parser::hwp3::Hwp3Error;
use std::collections::HashMap;

const HWP3_UNIT_SCALE: i32 = 4;
mod group_coordinates;
mod shape_mapping;
use shape_mapping::map_to_shape_object;

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

    let mut root = if root_nodes.len() == 1 {
        root_nodes.remove(0)
    } else {
        let mut group = GroupShape::default();
        group.children = root_nodes;
        ShapeObject::Group(group)
    };
    group_coordinates::assign_group_coordinates(&mut root, 0, (0.0, 0.0));
    Ok(root)
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
        assert!(super::shape_mapping::hwp3_rectangle_uses_no_fill_marker(
            &rectangle
        ));

        let text_box = Hwp3DrawingObjectCommonHeader {
            object_type: 6,
            basic_attr: Hwp3DrawingObjectBasicAttr {
                fill_color: 0x1000_0000,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(
            !super::shape_mapping::hwp3_rectangle_uses_no_fill_marker(&text_box),
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
        assert!(super::shape_mapping::hwp3_uses_no_line_marker(&no_line));

        let black_line = Hwp3DrawingObjectCommonHeader {
            basic_attr: Hwp3DrawingObjectBasicAttr {
                line_color: 0x0000_0000,
                ..Default::default()
            },
            ..Default::default()
        };
        assert!(
            !super::shape_mapping::hwp3_uses_no_line_marker(&black_line),
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
    /// sibling)만 실은 최소 공통 헤더(92바이트, header_length=88) + 세부 정보
    /// 길이 8바이트(#5141 계약)를 만든다.
    fn container_block() -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(&88u32.to_le_bytes()); // header_length (자기 4바이트 제외)
        buf.extend_from_slice(&0u16.to_le_bytes()); // object_type = 0 (Container)
        buf.extend_from_slice(&0x0002u16.to_le_bytes()); // connection_info: has_child, !has_sibling
        buf.extend_from_slice(&[0u8; 8]); // relative_pos
        buf.extend_from_slice(&[0u8; 8]); // object_size
        buf.extend_from_slice(&[0u8; 8]); // absolute_pos
        buf.extend_from_slice(&[0u8; 16]); // bounds
        buf.extend_from_slice(&[0u8; 32]); // basic_attr: line_style..pattern_color
        buf.extend_from_slice(&[0u8; 8]); // basic_attr: textbox_margin
        buf.extend_from_slice(&0u32.to_le_bytes()); // basic_attr: options
        buf.extend_from_slice(&[0u8; 8]); // 세부 정보 길이 (info1_len=0, info2_len=0)
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
