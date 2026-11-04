//! HWP3 사용자 정의 개요 번호의 공통 번호 정의 변환.

/// 표 62의 사용자 정의 번호를 공통 Numbering으로 옮긴다.
/// 한컴 저장본의 다른 단계 기본값 0과 Print로 확인한 5(한글 음절)를 옮긴다.
/// 실제 사용 단계가 5가 아니거나 다른 값이 있으면 호출자가 기존 처리를 유지한다.
pub(super) fn custom_numbering(custom: &str) -> Option<crate::model::style::Numbering> {
    use crate::model::style::{Numbering, NumberingHead};

    if custom.len() != 84 || !custom.is_ascii() {
        return None;
    }
    let bytes = (0..42)
        .map(|i| u8::from_str_radix(&custom[i * 2..i * 2 + 2], 16).ok())
        .collect::<Option<Vec<_>>>()?;
    let mut numbering = Numbering {
        start_number: 0,
        level_start_numbers: [1; 7],
        ..Default::default()
    };
    let mut has_verified_hangul_form = false;
    for level in 0..7 {
        let code = u16::from_le_bytes([bytes[level * 2], bytes[level * 2 + 1]]);
        let number_format = match code {
            0 => 0,
            5 => {
                has_verified_hangul_form = true;
                8
            }
            _ => return None,
        };
        let offset = 14 + level * 4;
        let before = crate::parser::hwp3::encoding::decode_hchar_array(&bytes[offset..offset + 2]);
        let after =
            crate::parser::hwp3::encoding::decode_hchar_array(&bytes[offset + 2..offset + 4]);
        // 공통 번호 문자열에서 ^는 참조 문법이므로 장식으로 추측해 넣지 않는다.
        if before.contains('^') || after.contains('^') {
            return None;
        }
        numbering.heads[level] = NumberingHead {
            number_format,
            ..Default::default()
        };
        numbering.level_formats[level] = format!("{before}^{}{after}", level + 1);
    }
    has_verified_hangul_form.then_some(numbering)
}
