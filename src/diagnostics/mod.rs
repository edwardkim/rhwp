//! Diagnostic tooling for HWP/HWPX compatibility work.

// CLI 종료 코드 계약(mydocs/manual/cli_commands.md)의 진단 명령용 단일 출처.
// `src/main.rs` 의 EXIT_* 는 바이너리 크레이트 전용이라 라이브러리 쪽 진단 진입점에서
// 참조할 수 없다. 같은 값을 여기 한 곳에 두어 값 표류를 막는다.
/// 성공.
pub const EXIT_OK: i32 = 0;
/// 런타임 실패 — 읽기·파싱·렌더·쓰기.
pub const EXIT_RUNTIME: i32 = 1;
/// 사용법 오류 — 인자 없음, 알 수 없는 옵션.
pub const EXIT_USAGE: i32 = 2;

pub mod bench;
pub mod core_pages_probe;
pub mod hwp5_anchor_trace;
pub mod hwp5_borderfill_diagonal_probe;
pub mod hwp5_cell_header_probe;
pub mod hwp5_char_shape_audit;
pub mod hwp5_contract_analyze;
pub mod hwp5_contract_probe;
pub mod hwp5_ctrl_data_trace;
pub mod hwp5_first_para_control_probe;
pub mod hwp5_inventory;
pub mod hwp5_inventory_diff;
pub mod hwp5_mel_personnel_probe;
pub mod hwp5_roundtrip_batch;
pub mod hwp5_table_probe;
pub mod hwpx_roundtrip_batch;
pub mod ir_field_sweep;
pub mod perf_counters;
pub mod render_geom_diff;
pub mod text_width_probe;
