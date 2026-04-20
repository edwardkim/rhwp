//! HWPX vs HWP 직렬화기 가정 차이 추출 CLI (#178 진단 도구)
//!
//! 사용법:
//! ```
//! cargo run --example hwpx_hwp_ir_diff -- samples/hwpx/hwpx-h-01.hwpx
//! cargo run --example hwpx_hwp_ir_diff -- a.hwpx b.hwp     # 두 파일 비교
//! ```

use std::env;
use std::process::ExitCode;

use rhwp::document_core::DocumentCore;
use rhwp::document_core::converters::diagnostics::{
    diff_hwpx_vs_hwp, diff_hwpx_vs_serializer_assumptions,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("usage: {} <hwpx_file> [hwp_file]", args[0]);
        return ExitCode::from(2);
    }

    let hwpx_path = &args[1];
    let hwpx_bytes = match std::fs::read(hwpx_path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("HWPX 로드 실패 {}: {}", hwpx_path, e);
            return ExitCode::from(1);
        }
    };

    let hwpx_core = match DocumentCore::from_bytes(&hwpx_bytes) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("HWPX 파싱 실패 {}: {:?}", hwpx_path, e);
            return ExitCode::from(1);
        }
    };

    if let Some(hwp_path) = args.get(2) {
        let hwp_bytes = match std::fs::read(hwp_path) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("HWP 로드 실패 {}: {}", hwp_path, e);
                return ExitCode::from(1);
            }
        };
        let hwp_core = match DocumentCore::from_bytes(&hwp_bytes) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("HWP 파싱 실패 {}: {:?}", hwp_path, e);
                return ExitCode::from(1);
            }
        };
        let summary = diff_hwpx_vs_hwp(hwpx_core.document(), hwp_core.document());
        println!("=== HWPX vs HWP IR diff: {} vs {} ===", hwpx_path, hwp_path);
        println!("{}", summary.human_report());
    } else {
        let summary = diff_hwpx_vs_serializer_assumptions(hwpx_core.document());
        println!("=== HWPX 단독 검사: {} ===", hwpx_path);
        println!("{}", summary.human_report());
    }

    ExitCode::SUCCESS
}
