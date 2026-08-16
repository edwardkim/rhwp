//! rhwp의 기계 판독 계약을 제공하는 내부 crate.
//!
//! 스키마 버전, IR 스키마, 출처 표지, 온톨로지는 서로만 참조하는 닫힌
//! 계층이다. 이 경계를 별도 crate로 유지하면 공개 API를 보존하면서 해당
//! 단위 테스트를 루트 `rhwp` 테스트 바이너리와 독립적으로 컴파일할 수 있다.

pub mod ir_schema;
pub mod ontology;
pub mod provenance;
pub mod schema_registry;
