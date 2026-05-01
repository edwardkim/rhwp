//! HWP3 인코딩 및 디코딩 모듈
//!
//! HWP 3.0은 텍스트 처리를 위해 `hchar` (2바이트 문자 코드)를 사용하며,
//! 이 모듈은 원시 바이트 데이터를 UTF-8 형식의 Rust `String`으로 안전하게 변환한다.
//! 조합형, 완성형 등 다양한 문자 코드 처리를 지원한다.

use crate::parser::hwp3::johab::decode_johab;

/// HWP 문자열을 나타내는 바이트 배열을 UTF-8 `String`으로 디코딩합니다.
///
/// HWP 3.0은 문자열 인코딩으로 상용조합형을 사용합니다.
/// 이는 영어/ASCII 문자는 1바이트(< 0x80),
/// 한글/한자/기호는 2바이트(>= 0x80)로 처리하는 MBCS(다바이트 문자셋)입니다.
pub fn decode_hwp3_string(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len());
    let mut i = 0;
    
    while i < bytes.len() {
        let b1 = bytes[i];
        if b1 == 0 {
            break; // 널(Null) 종료 문자
        }
        
        if b1 < 0x80 {
            // 1바이트 ASCII 문자
            result.push(b1 as char);
            i += 1;
        } else {
            // 2바이트 조합형 문자
            if i + 1 < bytes.len() {
                let b2 = bytes[i + 1];
                let ch = ((b1 as u16) << 8) | (b2 as u16);
                result.push(decode_johab(ch));
                i += 2;
            } else {
                // 짝이 없는 후행 바이트
                result.push('?');
                i += 1;
            }
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_hwp3_string() {
        // ASCII 테스트
        let ascii_bytes = b"Hello\0World";
        assert_eq!(decode_hwp3_string(ascii_bytes), "Hello");

        // 조합형 테스트
        // "가"의 조합형 코드는 0x88 0x61
        // 리틀 엔디안 바이트 배열: [0x88, 0x61]
        let johab_bytes = [0x88, 0x61, 0x00];
        assert_eq!(decode_hwp3_string(&johab_bytes), "가");

        // 혼합 텍스트 테스트
        let mixed_bytes = [0x41, 0x88, 0x61, 0x42, 0x00];
        assert_eq!(decode_hwp3_string(&mixed_bytes), "A가B");
    }
}
