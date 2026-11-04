//! 구역 끝 임의 바탕쪽의 지정 쪽 범위를 원본 목록에서 보존한다.
use super::{parse_master_pages_from_raw, HeaderFooterApply, MasterPage, RawRecord};

pub(super) fn parse_extended_master_pages(records: &[RawRecord]) -> Vec<MasterPage> {
    let mut pages = parse_master_pages_from_raw(records, 0);
    for page in &mut pages {
        // 구역 끝 목록의 바탕쪽은 모두 확장 바탕쪽이다(한컴 12.30: 기본 바탕쪽은 구역 정의 아래에 둔다).
        // 종류 값 3은 구역 마지막 쪽, 4 이상은 지정 쪽 번호 + 3이다. 기본 바탕쪽 배열 순서나
        // 확장/겹침 비트로 읽으면 지정 1·2쪽이 홀짝 쪽에 새어 나온다.
        page.is_extension = true;
        page.apply_to = HeaderFooterApply::Both;
        // 겹치게 하기(byte 22 bit 0)가 아니면 기본 바탕쪽 대신 그린다.
        page.replace_base = !page.overlap;
        if let Some(number) = page.ext_flags.checked_sub(3).filter(|number| *number > 0) {
            page.optional_page_number = Some(number);
        }
    }
    pages
}
