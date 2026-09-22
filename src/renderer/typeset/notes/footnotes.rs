//! 각주 내용 측정·소유 경계와 실제 등록을 분리한다.
mod body;
pub(in crate::renderer::typeset) mod boundary;
pub(in crate::renderer::typeset) mod measure;
