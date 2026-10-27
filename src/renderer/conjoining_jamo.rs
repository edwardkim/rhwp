//! 한컴의 자모 조합 소속과 독립 중성·종성의 무전진 표시 계약.
//!
//! 빈 문서에서 네 face와 초성/중성/종성·호환 아래아를 교차한 2020 Print가
//! 독립 중성·종성에는 HCRDotum을 선택한다. 원 프로그램의 232개 중성·종성은
//! 모두 hmtx=0이며 호환 U+318D와 초성은 이 계약에 포함하지 않는다.
//! 2024는 원 face의 spacing 자모를 유지하므로 이 모듈의 문자 분류만으로
//! 무전진을 선택하지 않는다. style resolver가 선택한 실제 프로그램을 함께 소비한다.

/// 독립 자모의 표시·전진에 함께 적용할 실제 프로그램 선택이다.
/// 글꼴 이름의 문자열 포인터를 모든 텍스트 런에 복제하지 않는다.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsolatedJamoFont {
    HcrDotum,
}

/// 자모 클러스터 길이 매핑 계산
///
/// 한글 자모 조합(초+중+종)을 1개 클러스터로 묶는다.
/// cluster_len[i] > 0: 클러스터 시작 (길이), 0: 클러스터 내부 (이전 문자와 동일 위치)
pub(crate) fn cluster_lengths(chars: &[char]) -> Vec<u8> {
    let char_count = chars.len();
    let mut cluster_len = vec![0u8; char_count];
    let mut ci = 0;
    while ci < char_count {
        if is_hangul_choseong(chars[ci]) {
            let start = ci;
            ci += 1;
            if ci < char_count && is_hangul_jungseong(chars[ci]) {
                ci += 1;
                if ci < char_count && is_hangul_jongseong(chars[ci]) {
                    ci += 1;
                }
            }
            cluster_len[start] = (ci - start) as u8;
        } else {
            cluster_len[ci] = 1;
            ci += 1;
        }
    }
    cluster_len
}

/// 한글 자모 초성 여부 (옛한글 포함)
pub(crate) fn is_hangul_choseong(c: char) -> bool {
    ('\u{1100}'..='\u{115F}').contains(&c) || ('\u{A960}'..='\u{A97F}').contains(&c)
}

/// 한글 자모 중성 여부 (옛한글 포함, ᆞ U+119E 포함)
pub(crate) fn is_hangul_jungseong(c: char) -> bool {
    ('\u{1160}'..='\u{11A7}').contains(&c) || ('\u{D7B0}'..='\u{D7C6}').contains(&c)
}

/// 한글 자모 종성 여부 (옛한글 포함)
pub(crate) fn is_hangul_jongseong(c: char) -> bool {
    ('\u{11A8}'..='\u{11FF}').contains(&c) || ('\u{D7CB}'..='\u{D7FB}').contains(&c)
}

pub(crate) fn is_mark(c: char) -> bool {
    is_hangul_jungseong(c) || is_hangul_jongseong(c)
}

pub(crate) fn isolated_positions(chars: &[char]) -> Vec<bool> {
    cluster_lengths(chars)
        .iter()
        .zip(chars)
        .map(|(&length, &c)| length == 1 && is_mark(c))
        .collect()
}
