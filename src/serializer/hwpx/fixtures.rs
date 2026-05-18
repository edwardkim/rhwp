//! 테스트 대조용 템플릿 상수.
//!
//! Stage 1~5가 동적 생성으로 전환함에 따라, 기존 `templates/` 폴더의 XML 파일들은
//! **테스트 대조용 fixture**로만 재분류된다. 실코드 경로에서는 사용되지 않는다.
//!
//! 용도:
//! - 동적 생성 결과가 한컴 템플릿과 등가인지 비교하는 단위 테스트
//! - Stage 0의 `blank_hwpx.hwpx` 라운드트립 검증

#![allow(dead_code)]

pub const EMPTY_HEADER_XML: &str = include_str!("templates/empty_header.xml");
pub const EMPTY_SECTION0_XML: &str = include_str!("templates/empty_section0.xml");
pub const EMPTY_CONTENT_HPF: &str = include_str!("templates/empty_content.hpf");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn templates_are_not_empty() {
        assert!(!EMPTY_HEADER_XML.is_empty());
        assert!(!EMPTY_SECTION0_XML.is_empty());
        assert!(!EMPTY_CONTENT_HPF.is_empty());
    }

    #[test]
    fn empty_header_contains_hh_head_root() {
        assert!(
            EMPTY_HEADER_XML.contains("<hh:head"),
            "empty_header.xml should contain <hh:head> root"
        );
    }
}
