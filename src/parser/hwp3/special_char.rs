//! HWP3 특수 문자 처리
//! 
//! HWP3 문서 내의 제어 문자(표, 그림, 수식, 주석 등) 및 특수 문자를 매핑하고 변환한다.
//! 바이트 코드에 따라 적절한 제어 코드로 해석하여 렌더링에 필요한 정보를 제공한다.

use snafu::Snafu;
use std::io::{self, Cursor, Read};

#[derive(Debug, Snafu)]
pub enum Hwp3SpecialCharError {
    #[snafu(display("잘못된 특수 문자 코드입니다: {code}"))]
    InvalidCode { code: u16 },
    #[snafu(display("입출력 오류가 발생했습니다: {source}"))]
    IoError { source: io::Error },
}

impl From<io::Error> for Hwp3SpecialCharError {
    fn from(error: io::Error) -> Self {
        Hwp3SpecialCharError::IoError { source: error }
    }
}

#[derive(Debug)]
pub enum Hwp3SpecialChar {
    FieldCode { length: u32, data: Vec<u8> }, // 5
    Bookmark { length: u32, data: Vec<u8> }, // 6
    DateFormat { length: u32, data: Vec<u8> }, // 7
    DateCode { length: u32, data: Vec<u8> }, // 8
    Tab { length: u32, data: Vec<u8> }, // 9
    TableBoxEqButtonHypertext { length: u32, data: Vec<u8> }, // 10
    Picture {
        length: u32,
        picture_type: u8,
        frame_header: Option<crate::parser::hwp3::drawing::Hwp3DrawingObjectFrameHeader>,
        drawing_objects: Vec<crate::parser::hwp3::drawing::Hwp3DrawingObject>,
        ole_info: Option<crate::parser::hwp3::ole::Hwp3OleInfo>,
        raw_data: Vec<u8>,
    }, // 11
    Line { length: u32, data: Vec<u8> }, // 14
    HiddenComment { length: u32, data: Vec<u8> }, // 15
    HeaderFooter { length: u32, data: Vec<u8> }, // 16
    FootnoteEndnote { length: u32, data: Vec<u8> }, // 17
    NumberCode { length: u32, data: Vec<u8> }, // 18
    NewNumber { length: u32, data: Vec<u8> }, // 19
    PageNumber { length: u32, data: Vec<u8> }, // 20
    OddEvenPage { length: u32, data: Vec<u8> }, // 21
    MailMerge { length: u32, data: Vec<u8> }, // 22
    CharOverlap { length: u32, data: Vec<u8> }, // 23
    Hyphen { length: u32, data: Vec<u8> }, // 24
    IndexMark { length: u32, data: Vec<u8> }, // 25
    FindMark { length: u32, data: Vec<u8> }, // 26
    OutlineShapeNumber { length: u32, data: Vec<u8> }, // 28
    CrossReference { length: u32, data: Vec<u8> }, // 29
    BundleBlank { length: u32, data: Vec<u8> }, // 30
    FixedBlank { length: u32, data: Vec<u8> }, // 31
    Unknown { code: u16, length: u32, data: Vec<u8> },
}

impl Hwp3SpecialChar {
    pub fn parse(code: u16, length: u32, data: Vec<u8>) -> Result<Self, Hwp3SpecialCharError> {
        match code {
            5 => Ok(Hwp3SpecialChar::FieldCode { length, data }),
            6 => Ok(Hwp3SpecialChar::Bookmark { length, data }),
            7 => Ok(Hwp3SpecialChar::DateFormat { length, data }),
            8 => Ok(Hwp3SpecialChar::DateCode { length, data }),
            9 => Ok(Hwp3SpecialChar::Tab { length, data }),
            10 => Ok(Hwp3SpecialChar::TableBoxEqButtonHypertext { length, data }),
            11 => {
                let mut picture_type = 0;
                let mut frame_header = None;
                let mut drawing_objects = Vec::new();
                let mut ole_info = None;
                
                if data.len() >= 348 {
                    picture_type = data[74];
                    if picture_type == 3 {
                        // 그리기 개체 (Drawing Object)
                        let mut cursor = Cursor::new(&data[348..]);
                        if let Ok(frame) = crate::parser::hwp3::drawing::Hwp3DrawingObjectFrameHeader::read(&mut cursor) {
                            for _ in 0..frame.object_count {
                                if let Ok(obj) = crate::parser::hwp3::drawing::Hwp3DrawingObject::read(&mut cursor) {
                                    drawing_objects.push(obj);
                                } else {
                                    break;
                                }
                            }
                            frame_header = Some(frame);
                        }
                    } else if picture_type == 1 {
                        // OLE 개체 (OLE Object)
                        let mut cursor = Cursor::new(&data[348..]);
                        let ext_len = (data.len() - 348) as u32;
                        if let Ok(ole) = crate::parser::hwp3::ole::Hwp3OleInfo::read(&mut cursor, ext_len) {
                            ole_info = Some(ole);
                        }
                    }
                }
                Ok(Hwp3SpecialChar::Picture {
                    length,
                    picture_type,
                    frame_header,
                    drawing_objects,
                    ole_info,
                    raw_data: data,
                })
            },
            14 => Ok(Hwp3SpecialChar::Line { length, data }),
            15 => Ok(Hwp3SpecialChar::HiddenComment { length, data }),
            16 => Ok(Hwp3SpecialChar::HeaderFooter { length, data }),
            17 => Ok(Hwp3SpecialChar::FootnoteEndnote { length, data }),
            18 => Ok(Hwp3SpecialChar::NumberCode { length, data }),
            19 => Ok(Hwp3SpecialChar::NewNumber { length, data }),
            20 => Ok(Hwp3SpecialChar::PageNumber { length, data }),
            21 => Ok(Hwp3SpecialChar::OddEvenPage { length, data }),
            22 => Ok(Hwp3SpecialChar::MailMerge { length, data }),
            23 => Ok(Hwp3SpecialChar::CharOverlap { length, data }),
            24 => Ok(Hwp3SpecialChar::Hyphen { length, data }),
            25 => Ok(Hwp3SpecialChar::IndexMark { length, data }),
            26 => Ok(Hwp3SpecialChar::FindMark { length, data }),
            28 => Ok(Hwp3SpecialChar::OutlineShapeNumber { length, data }),
            29 => Ok(Hwp3SpecialChar::CrossReference { length, data }),
            30 => Ok(Hwp3SpecialChar::BundleBlank { length, data }),
            31 => Ok(Hwp3SpecialChar::FixedBlank { length, data }),
            _ => Ok(Hwp3SpecialChar::Unknown { code, length, data }),
        }
    }
}
