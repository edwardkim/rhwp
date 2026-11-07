//! 구두점의 측정과 표시가 같은 영문 프로그램을 사용하도록 슬롯을 정한다.

fn primary(chain: &str) -> &str {
    chain
        .split(',')
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches(|ch| ch == '\'' || ch == '"')
}

pub(super) fn uses_latin_slot(families: &[String]) -> bool {
    // 독립 한컴2020 Print의 같은 크기 대조군은 궁서/굴림/바탕,
    // 궁서/궁서/바탕, 맑은고딕/Arial/바탕 모두 괄호를 영문 슬롯으로 그린다.
    // 한글·영문 face가 같아도 앞 한자에 상속시키면 다른 프로그램과 폭이 된다.
    // 프로그램의 한글 수록 여부·HFT 대체 종류는 문자의 소유 슬롯을 결정하지 않는다.
    // 전진·paint가 이미 공유하는 슬롯 경로를 사용하며 공백의 상속 언어는 유지한다.
    families
        .get(1)
        .is_some_and(|latin| !primary(latin).is_empty())
}
