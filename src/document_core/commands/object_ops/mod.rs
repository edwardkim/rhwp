//! 그림 속성/삽입/삭제 + 표 생성 + 셀 bbox 관련 native 메서드 — 도메인별 분할 (#1904).
//!
//! 종전 단일 파일(9,845줄, 7개 도메인 응집)을 도메인 모듈로 분할. 함수 이동만 —
//! 로직/외부 인터페이스 무변경 (impl DocumentCore 분산, 메서드 경로 동일).

mod common;
mod connector;
mod equation;
mod note;
mod picture;
mod shape;
mod table;

/// 도형 최소 크기 (HWPUNIT).
/// 0으로 내려가면 Rectangle은 x_coords=[0,0,0,0]이 되고,
/// Group은 current/original 스케일이 0이 되어 자식이 전부 사라진다.
/// table_ops의 MIN_CELL_SIZE와 동일한 기준을 사용한다.
pub(crate) const MIN_SHAPE_SIZE: u32 = 200;
