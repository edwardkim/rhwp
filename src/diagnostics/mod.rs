//! Diagnostic tooling for HWP/HWPX compatibility work.

use crate::parser::cfb_reader::{CfbError, CfbReader};

// CLI 종료 코드 계약(mydocs/manual/cli_commands.md)의 진단 명령용 단일 출처.
// `src/main.rs` 의 EXIT_* 는 바이너리 크레이트 전용이라 라이브러리 쪽 진단 진입점에서
// 참조할 수 없다. 같은 값을 여기 한 곳에 두어 값 표류를 막는다.
/// 성공.
pub const EXIT_OK: i32 = 0;
/// 런타임 실패 — 읽기·파싱·렌더·쓰기.
pub const EXIT_RUNTIME: i32 = 1;
/// 사용법 오류 — 인자 없음, 알 수 없는 옵션.
pub const EXIT_USAGE: i32 = 2;

/// 등록된 HWP5 진단 명령이 직접 소비하는 단일 스트림 출력 상한.
///
/// 이는 `parse_document*`의 문서 열기 예산이 아니라 raw record 진단 consumer의
/// 독립 정책이다. 각 명령은 이 값을 명시적으로 limited CFB API에 전달한다.
pub(crate) const MAX_HWP5_DIAGNOSTIC_STREAM_OUTPUT_BYTES: usize = 256 * 1024 * 1024;

/// HWP5 진단 consumer가 DocInfo를 명시적 출력 상한으로 읽는 공통 배선.
pub(crate) fn read_hwp5_doc_info_limited(
    cfb: &mut CfbReader,
    compressed: bool,
    max_bytes: usize,
) -> Result<Vec<u8>, CfbError> {
    cfb.read_doc_info_limited(compressed, max_bytes)
}

/// HWP5 진단 consumer가 본문 또는 ViewText 원본을 명시적 상한으로 읽는 공통 배선.
///
/// 배포용 문서는 복호화하지 않는 raw-record 진단 계약을 보존해 제한된 ViewText
/// 암호문만 반환한다. ViewText가 없으면 BodyText로 fallback하지 않는다.
pub(crate) fn read_hwp5_body_text_section_limited(
    cfb: &mut CfbReader,
    index: u32,
    compressed: bool,
    distribution: bool,
    max_bytes: usize,
) -> Result<Vec<u8>, CfbError> {
    if distribution {
        cfb.read_viewtext_section_raw_limited(index, max_bytes)
    } else {
        cfb.read_body_text_section_limited(index, compressed, max_bytes)
    }
}

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
