//! 독립 한컴 인쇄에 근거한 원본 굵기 전진의 family 판정.

/// 선언과 실제 글리프 공급 프로그램 양쪽에서 확인해야 하는 원본 꾸밈 계약.
/// #8946의 변경하지 않은 한컴 Print /Widths·TJ 대조에서 세나루(12/20pt),
/// 강낭콩(20pt), 딸기(18pt)의 별도 굵기 전진을 확인했다.
/// HFT 유형이나 CFF 형식만으로 다른 프로그램에 이 계약을 확장하지 않는다.
pub fn is_verified_family(family: &str) -> bool {
    matches!(
        family.trim(),
        "신명 중고딕"
            | "Sinmyeong Junggothic"
            | "#세나루"
            | "세나루"
            | "Senaru"
            | "강낭콩"
            | "Kidney bean"
            | "딸기"
            | "Strawberry"
    )
}

pub fn has_verified_program_family(names: ttf_parser::name::Names<'_>) -> bool {
    names.into_iter().any(|name| {
        matches!(
            name.name_id,
            ttf_parser::name_id::FAMILY | ttf_parser::name_id::TYPOGRAPHIC_FAMILY
        ) && name
            .to_string()
            .is_some_and(|family| is_verified_family(&family))
    })
}
