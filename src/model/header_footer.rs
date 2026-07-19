//! 머리말/꼬리말/바탕쪽 (Header, Footer, MasterPage)

use super::paragraph::Paragraph;

/// 머리말 ('head' 컨트롤)
#[derive(Debug, Default, Clone)]
pub struct Header {
    /// 적용 범위
    pub apply_to: HeaderFooterApply,
    /// 문단 리스트
    pub paragraphs: Vec<Paragraph>,
    /// 원본 attr u32 전체 (라운드트립 보존용)
    pub raw_attr: u32,
    /// CTRL_HEADER ctrl_data의 4바이트(attr) 이후 추가 바이트 (라운드트립 보존용)
    pub raw_ctrl_extra: Vec<u8>,
    /// HWPX subList list_attr materialization 값
    pub list_attr: u32,
    /// HWPX subList 텍스트 영역 폭
    pub text_width: u32,
    /// HWPX subList 텍스트 영역 높이
    pub text_height: u32,
    /// HWPX subList 텍스트 참조 비트맵
    pub text_ref: u8,
    /// HWPX subList 번호 참조 비트맵
    pub num_ref: u8,
}

/// 꼬리말 ('foot' 컨트롤)
#[derive(Debug, Default, Clone)]
pub struct Footer {
    /// 적용 범위
    pub apply_to: HeaderFooterApply,
    /// 문단 리스트
    pub paragraphs: Vec<Paragraph>,
    /// 원본 attr u32 전체 (라운드트립 보존용)
    pub raw_attr: u32,
    /// CTRL_HEADER ctrl_data의 4바이트(attr) 이후 추가 바이트 (라운드트립 보존용)
    pub raw_ctrl_extra: Vec<u8>,
    /// HWPX subList list_attr materialization 값
    pub list_attr: u32,
    /// HWPX subList 텍스트 영역 폭
    pub text_width: u32,
    /// HWPX subList 텍스트 영역 높이
    pub text_height: u32,
    /// HWPX subList 텍스트 참조 비트맵
    pub text_ref: u8,
    /// HWPX subList 번호 참조 비트맵
    pub num_ref: u8,
}

/// 머리말/꼬리말 적용 범위
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum HeaderFooterApply {
    #[default]
    /// 양쪽 (모든 페이지)
    Both,
    /// 짝수 쪽
    Even,
    /// 홀수 쪽
    Odd,
}

/// 바탕쪽 (Master Page)
///
/// 구역 단위 페이지 템플릿. 양쪽/홀수/짝수 3종류 설정 가능.
/// SectionDef의 자식 LIST_HEADER 레코드에서 파싱.
#[derive(Debug, Default, Clone)]
pub struct MasterPage {
    /// 적용 범위 (양쪽/홀수/짝수)
    pub apply_to: HeaderFooterApply,
    /// 확장 바탕쪽 여부 (마지막 쪽/임의 쪽 — 두 번째 이후 Both)
    pub is_extension: bool,
    /// 겹치게 하기 (확장 바탕쪽이 기존 바탕쪽 위에 겹쳐 표시)
    pub overlap: bool,
    /// 렌더링 시 기존 기본 바탕쪽을 대체한다.
    ///
    /// HWPX `LAST_PAGE pageDuplicate="0"`는 HWP5 저장 contract상 overlap bit가 켜질 수 있지만,
    /// 렌더링 의미는 마지막쪽 전용 바탕쪽으로 기본 홀/짝 바탕쪽을 대체하는 쪽에 가깝다.
    pub replace_base: bool,
    /// 확장 플래그 raw 값 (LIST_HEADER byte 18-19)
    pub ext_flags: u16,
    /// HWPX `masterPage@pageFront` — 첫 쪽(표지) 전용 바탕쪽 여부.
    /// HWP5에서 읽은 바탕쪽은 해당 XML 속성이 없으므로 기본 false.
    pub page_front: bool,
    /// 문단 리스트
    pub paragraphs: Vec<Paragraph>,
    /// 텍스트 영역 폭 (HWPUNIT)
    pub text_width: u32,
    /// 텍스트 영역 높이 (HWPUNIT)
    pub text_height: u32,
    /// 텍스트 참조 비트맵
    pub text_ref: u8,
    /// 번호 참조 비트맵
    pub num_ref: u8,
    /// HWPX masterPage@pageNumber.
    ///
    /// HWPX에서 이 값은 바탕쪽 내부 PAGE 필드의 치환 가능 여부를 판단하는 힌트로 쓰인다.
    /// HWP5에서 읽은 바탕쪽은 해당 XML 속성이 없으므로 None으로 둔다.
    pub hwpx_page_number: Option<u16>,
    /// LIST_HEADER raw data (라운드트립 보존용)
    pub raw_list_header: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_default() {
        let header = Header::default();
        assert_eq!(header.apply_to, HeaderFooterApply::Both);
        assert!(header.paragraphs.is_empty());
    }

    #[test]
    fn test_footer_default() {
        let footer = Footer::default();
        assert_eq!(footer.apply_to, HeaderFooterApply::Both);
    }
}
