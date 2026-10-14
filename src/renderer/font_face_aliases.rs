//! 실제 Bold 프로그램을 가진 한컴·MS face의 공통 별칭.
//!
//! Regular만 있는 legacy family에 CSS bold를 적용하면 브라우저가 굵기를
//! 합성한다. SVG의 local() 공급과 Canvas의 family 선택이 같은 실제 Bold
//! 후보를 사용하며, 후보가 없으면 각 출력 경로의 기존 fallback을 유지한다.

pub(crate) fn local_bold_aliases(font_family: &str) -> &'static [&'static str] {
    match font_family {
        "함초롬바탕" | "함초롱바탕" | "한컴바탕" | "휴먼명조" => {
            &["HCR Batang Bold", "함초롬바탕 Bold"]
        }
        "함초롬돋움" | "함초롱돋움" | "한컴돋움" | "한양중고딕" | "HY중고딕" => {
            &["HCR Dotum Bold", "함초롬돋움 Bold"]
        }
        "한컴 고딕" | "Hancom Gothic" => &["Hancom Gothic Bold", "한컴 고딕 Bold"],
        "맑은 고딕" | "Malgun Gothic" => &["Malgun Gothic Bold", "맑은 고딕 Bold"],
        _ => &[],
    }
}
