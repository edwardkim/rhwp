//! EMR_HEADER 레코드 파싱 래퍼.

use crate::emf::parser::{objects::Header, Cursor};
use crate::emf::Error;

/// 커서가 EMR_HEADER(type=1)의 선두에 있을 때 호출. 성공 시 Header 반환.
pub fn parse(cursor: &mut Cursor<'_>) -> Result<Header, Error> {
    Header::read(cursor)
}
