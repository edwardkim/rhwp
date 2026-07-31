//! 조합형 텍스트 변환 로직
//!
//! `johab_map.rs`의 테이블을 활용하여 실제 조합형 텍스트를 유니코드(UTF-8) 문자로
//! 디코딩하는 함수(`decode_johab`)를 제공한다.

use crate::parser::hwp3::johab_map;

/// KSSM 조합형의 아래아(중성 인덱스 30)를 Unicode 옛한글 자모로 푼다.
///
/// HWP3은 이 음절을 한 개 hchar로 저장하지만, HWP5/HWPX 변환본은
/// 초성·아래아·종성 자모열로 보존한다. `decode_johab`의 완성형 반환 계약을
/// 바꾸지 않기 위해, 가변 길이 텍스트가 필요한 호출자만 이 함수를 사용한다.
pub fn decode_johab_araea_jamo(ch: u16) -> Option<(char, char, Option<char>)> {
    if ch < 0x8000 {
        return None;
    }

    let cho_idx = ((ch >> 10) & 0x1F) as usize;
    let jung_idx = (ch >> 5) & 0x1F;
    let jong_idx = (ch & 0x1F) as usize;
    if jung_idx != 30 {
        return None;
    }

    let cho_map: [i32; 32] = [
        -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1,
    ];
    let jong_map: [i32; 32] = [
        -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, -1, 17, 18, 19, 20, 21, 22,
        23, 24, 25, 26, 27, -1, -1,
    ];
    let cho = *cho_map.get(cho_idx)?;
    let jong = *jong_map.get(jong_idx)?;
    if cho < 0 || jong < 0 {
        return None;
    }

    let leading = char::from_u32(0x1100 + cho as u32)?;
    let araea = char::from_u32(0x119E)?;
    let trailing = if jong == 0 {
        None
    } else {
        char::from_u32(0x11A7 + jong as u32)
    };
    Some((leading, araea, trailing))
}

pub fn decode_johab(ch: u16) -> char {
    if ch < 0x80 {
        return ch as u8 as char;
    } else if ch >= 0x8000 {
        // 조합형 한글 (상위 비트 1)
        let cho_idx = (ch >> 10) & 0x1F;
        let jung_idx = (ch >> 5) & 0x1F;
        let jong_idx = ch & 0x1F;

        let cho_map: [i32; 32] = [
            -1, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, -1, -1, -1,
            -1, -1, -1, -1, -1, -1, -1, -1,
        ];
        let jung_map: [i32; 32] = [
            -1, -1, -1, 0, 1, 2, 3, 4, -1, -1, 5, 6, 7, 8, 9, 10, -1, -1, 11, 12, 13, 14, 15, 16,
            -1, -1, 17, 18, 19, 20, -1, -1,
        ];
        let jong_map: [i32; 32] = [
            -1, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, -1, 17, 18, 19, 20, 21,
            22, 23, 24, 25, 26, 27, -1, -1,
        ];

        let cho = cho_map[cho_idx as usize];
        let jung = jung_map[jung_idx as usize];
        let jong = jong_map[jong_idx as usize];

        // jong_idx == 0 은 예약된 미사용 값이며, jong_map[1] == 0 이 "받침 없음"을
        // 나타낸다. jong == -1 을 무조건 0(받침 없음)으로 치환하면 예약/무효
        // 조합(jong_idx 0, 18, 30, 31)이 유효한 완성형 음절로 잘못 디코딩된다.
        if cho != -1 && jung != -1 && jong != -1 {
            let uni_val = 0xAC00 + (cho * 21 * 28) + (jung * 28) + jong;
            if let Some(c) = std::char::from_u32(uni_val as u32) {
                return c;
            }
        }

        // 한자 및 기호 영역 (이진 탐색)
        if let Ok(idx) = johab_map::JOHAB_SYMBOLS.binary_search_by_key(&ch, |&(k, _)| k) {
            return johab_map::JOHAB_SYMBOLS[idx].1;
        }
    } else if ch >= 0x0080 {
        // [Task #741 Stage 5] HWP3 사적 graphic char 영역 (0x0080~0x7FFF).
        // 표준 KSSM 조합형 영역 (0x8000+) 외 한컴 사적 인코딩.
        // 매핑은 hwp3-sample10.hwp ↔ hwp3-sample10-hwp5.hwp cross-ref 로 도출.
        // Target: HWP5 변환본 IR 정합 (PUA 보존).
        if let Some(c) = decode_hwp3_extra(ch) {
            return c;
        }
    }

    // 매핑되지 않은 값
    '?'
}

/// HWP3 사적 graphic char (0x0080~0x7FFF 영역) → Unicode 매핑.
///
/// 한컴 변환본 (HWP3 → HWP5) 의 IR 과 정합. PUA (Private Use Area) 영역도
/// 변환본 정합 위해 그대로 보존.
///
/// 매핑 출처: hwp3-sample10.hwp ↔ hwp3-sample10-hwp5.hwp paragraph 별 cross-ref.
fn decode_hwp3_extra(ch: u16) -> Option<char> {
    // [Task #877 Stage 3] 로마숫자 대문자 Ⅰ~Ⅹ: 0x3590~0x3599 → U+2160~U+2169.
    // sample16 (hwp3-sample16.hwp) 의 cross-ref 로 도출. 한컴 HWP5 변환본의
    // paragraph 26/31/36/44 ("Ⅰ. 사업개요", "Ⅱ. 제안 일반사항", "Ⅲ ...", "Ⅳ ...") 정합.
    if (0x3590..=0x3599).contains(&ch) {
        return char::from_u32(0x2160 + (ch - 0x3590) as u32);
    }
    // HWP3 사적 원문자 계열. 연속 코드가 ①~⑩에 대응한다.
    if (0x36E7..=0x36F0).contains(&ch) {
        return char::from_u32(0x2460 + (ch - 0x36E7) as u32);
    }
    // `한글 97 안내문` HWP3 원본의 머리말 회사명 graphic char 여섯 글자.
    // HWP3 암호 fixture의 원시 값 0x37C0..=0x37C5와, 같은 문서의 HWPX
    // 변환본 U+F03EF..=U+F03F4 및 Hancom PDF의 "한글과컴퓨터"를 대조해
    // 확정했다. 렌더러의 공통 한컴 PUA 표가 표시 문자열을 담당하므로 여기서는
    // 해당 PUA를 보존한다.
    if (0x37C0..=0x37C5).contains(&ch) {
        return char::from_u32(0xF03EF + (ch - 0x37C0) as u32);
    }
    let codepoint: u32 = match ch {
        0x0081 => 0x201C,  // 왼쪽 큰따옴표
        0x0082 => 0x201D,  // 오른쪽 큰따옴표
        0x301E => 0xF0811, // 한컴 PUA - 관계도 가지 선문자
        0x301C => 0xF080F, // 한컴 PUA - 굵은 가로선 (94.5% 발생)
        0x3024 => 0xF0817, // 한컴 PUA - 관계도 하단 가지 선문자
        0x3027 => 0xF081A, // 한컴 PUA - 관계도 가로 선문자
        0x3404 => 0x2024,  // 한 점 리더
        0x3446 => 0x2192,  // 오른쪽 화살표
        0x35E1 => 0x2500,  // 상자 그리기 가로선
        0x303D => 0xF0827, // 한컴 PUA
        0x3479 => 0x25B7,  // ▷ WHITE RIGHT-POINTING TRIANGLE
        0x347A => 0x25B6,  // ▶ BLACK RIGHT-POINTING TRIANGLE
        0x3441 => 0x25A0,  // ■ BLACK SQUARE
        // `한글 97 안내문` HWP3 원본의 표 셀 글머리표. HWP5 변환본과
        // 한컴 PDF가 모두 U+25B8(▸)으로 표시하므로, HWP3 사적 코드도 같은
        // 표준 Unicode로 보존한다. 매핑이 없으면 decode_johab가 '?'를 반환하고
        // HWP3 parser가 미지원 사적 코드를 조용히 건너뛰어 글머리표가 사라진다.
        0x2F67 => 0x25B8, // ▸ BLACK RIGHT-POINTING SMALL TRIANGLE
        // [Task #1105] sample16 글머리 prefix.
        // HWP3 0x3366 은 한컴 HWP5 변환본에서 U+F03C5 로 보존되고, 렌더러가
        // 이를 한컴오피스 표시값인 □(U+25A1)로 확장한다. 여기서 ○로 직접
        // 낮추면 HWP3 원본만 정답지와 다른 bullet 로 보이므로 PUA를 보존한다.
        0x3366 => 0xF03C5,
        _ => return None,
    };
    char::from_u32(codepoint)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_johab_rejects_reserved_jong_index() {
        // cho_idx=2(초성 0), jung_idx=3(중성 0), jong_idx=0(예약/무효 값).
        // jong_idx=0 은 종성 매핑에서 -1(무효)이며, "받침 없음"은 jong_idx=1 이
        // 별도로 나타낸다. 무효 jong_idx 를 받침 없음으로 오인하면 존재하지
        // 않아야 할 완성형 음절 '가'(U+AC00)가 잘못 생성된다.
        let jong_idx = 0;
        let ch: u16 = 0x8000 | (2 << 10) | (3 << 5) | jong_idx;
        assert_ne!(decode_johab(ch), '가');
    }

    #[test]
    fn decode_johab_araea_preserves_legacy_jamo_sequence() {
        // HWP3 fixture의 첫 글자. 한컴 HWPX 변환본은 "ᄒᆞᆫ"으로 보존한다.
        assert_eq!(decode_johab_araea_jamo(0xD3C5), Some(('ᄒ', 'ᆞ', Some('ᆫ'))));
    }

    #[test]
    fn decode_hwp3_company_graphics_to_the_common_hancom_pua() {
        let decoded: String = (0x37C0..=0x37C5).map(decode_johab).collect();
        assert_eq!(
            decoded,
            "\u{F03EF}\u{F03F0}\u{F03F1}\u{F03F2}\u{F03F3}\u{F03F4}"
        );
    }

    #[test]
    fn decode_hwp3_table_triangle_bullet() {
        assert_eq!(decode_johab(0x2F67), '▸');
    }
}
