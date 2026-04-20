//! IR 변환 어댑터 모듈
//!
//! 한 IR 형태에서 다른 IR 형태로 의미를 보존하며 변환하는 어댑터를 모은다.
//!
//! 본 모듈의 핵심 정체성: **잘 작동하는 직렬화기 어깨 위에 서자**.
//! 직렬화기 자체를 수정하지 않고, IR 만 직렬화기가 기대하는 모양으로 정렬한다.

pub mod hwpx_to_hwp;
pub mod diagnostics;
pub mod common_obj_attr_writer;
