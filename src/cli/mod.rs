//! rhwp 프로세스 어댑터의 명령 표면.
//!
//! #5511은 handler를 이동하기 전에 실제 dispatch와 외부 자기서술의 관계부터
//! 고정한다. 하위 모듈은 application/service 계층이 아니며 도메인 로직을 소유하지
//! 않는다.

pub(crate) mod catalog;
pub(crate) mod queries;
