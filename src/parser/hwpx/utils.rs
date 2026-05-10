//! HWPX 파서 공통 유틸리티 함수
//!
//! header.rs, section.rs 등에서 공통으로 사용하는 XML 파싱 헬퍼.

use quick_xml::events::Event;
use quick_xml::Reader;

use super::HwpxError;

/// XML 네임스페이스 접두사를 제거하고 로컬 이름만 반환
/// 예: b"hp:p" → b"p", b"tbl" → b"tbl"
pub fn local_name(name: &[u8]) -> &[u8] {
    if let Some(pos) = name.iter().position(|&b| b == b':') {
        &name[pos + 1..]
    } else {
        name
    }
}

/// 속성 값을 String으로 변환
pub fn attr_str(attr: &quick_xml::events::attributes::Attribute) -> String {
    String::from_utf8_lossy(&attr.value).to_string()
}

/// 속성 값이 특정 문자열과 일치하는지 확인 (비교용)
pub fn attr_eq(attr: &quick_xml::events::attributes::Attribute, val: &str) -> bool {
    attr.value.as_ref() == val.as_bytes()
}

pub fn parse_u8(attr: &quick_xml::events::attributes::Attribute) -> u8 {
    attr_str(attr).parse().unwrap_or(0)
}

pub fn parse_i8(attr: &quick_xml::events::attributes::Attribute) -> i8 {
    attr_str(attr).parse().unwrap_or(0)
}

pub fn parse_u16(attr: &quick_xml::events::attributes::Attribute) -> u16 {
    attr_str(attr).parse().unwrap_or(0)
}

pub fn parse_i16(attr: &quick_xml::events::attributes::Attribute) -> i16 {
    attr_str(attr).parse().unwrap_or(0)
}

pub fn parse_u32(attr: &quick_xml::events::attributes::Attribute) -> u32 {
    attr_str(attr).parse().unwrap_or(0)
}

pub fn parse_i32(attr: &quick_xml::events::attributes::Attribute) -> i32 {
    attr_str(attr).parse().unwrap_or(0)
}

/// "#RRGGBB" → 0x00BBGGRR, "#AARRGGBB" → 0xAABBGGRR (alpha 보존)
pub fn parse_color(attr: &quick_xml::events::attributes::Attribute) -> u32 {
    let s = attr_str(attr);
    parse_color_str(&s)
}

/// 색상 문자열을 HWP ColorRef로 변환
pub fn parse_color_str(s: &str) -> u32 {
    if s == "none" || s.is_empty() {
        return 0xFFFFFFFF; // 투명/없음
    }
    let hex = s.trim_start_matches('#');
    if hex.len() == 6 {
        // RRGGBB → 0x00BBGGRR
        if let Ok(v) = u32::from_str_radix(hex, 16) {
            let r = (v >> 16) & 0xFF;
            let g = (v >> 8) & 0xFF;
            let b = v & 0xFF;
            return b << 16 | g << 8 | r;
        }
    } else if hex.len() == 8 {
        // AARRGGBB → 0xAABBGGRR (alpha 보존)
        if let Ok(v) = u32::from_str_radix(hex, 16) {
            let a = (v >> 24) & 0xFF;
            let r = (v >> 16) & 0xFF;
            let g = (v >> 8) & 0xFF;
            let b = v & 0xFF;
            return a << 24 | b << 16 | g << 8 | r;
        }
    }
    0x00000000 // 검정
}

/// 속성 값을 bool로 파싱 ("true", "1" → true)
pub fn parse_bool(attr: &quick_xml::events::attributes::Attribute) -> bool {
    let s = attr_str(attr);
    s == "true" || s == "1"
}

/// XML 요소를 자식 포함하여 건너뛰기 (깊이 추적)
pub fn skip_element(reader: &mut Reader<&[u8]>, _end_tag: &[u8]) -> Result<(), HwpxError> {
    let mut buf = Vec::new();
    let mut depth = 1u32;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(_)) => {
                depth += 1;
            }
            Ok(Event::End(_)) => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(HwpxError::XmlError(format!("skip: {}", e))),
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_name() {
        assert_eq!(local_name(b"hp:p"), b"p");
        assert_eq!(local_name(b"tbl"), b"tbl");
        assert_eq!(local_name(b"hh:charPr"), b"charPr");
    }

    #[test]
    fn test_parse_color_str() {
        assert_eq!(parse_color_str("#FF0000"), 0x000000FF); // 빨강 → R=FF → BGR=0000FF
        assert_eq!(parse_color_str("#00FF00"), 0x0000FF00); // 초록
        assert_eq!(parse_color_str("#0000FF"), 0x00FF0000); // 파랑
        assert_eq!(parse_color_str("#000000"), 0x00000000); // 검정
        assert_eq!(parse_color_str("none"), 0xFFFFFFFF);    // 투명
    }

    #[test]
    fn test_parse_color_str_with_alpha() {
        // AARRGGBB → 0xAABBGGRR (alpha 보존)
        assert_eq!(parse_color_str("#80FF0000"), 0x800000FF);
        assert_eq!(parse_color_str("#FF000000"), 0xFF000000); // 상위 바이트 비제로 → 채우기 없음
        assert_eq!(parse_color_str("#00FF0000"), 0x000000FF); // alpha=00 → 동일
    }
}
