use std::env;
use std::fs;
use std::io::Read as _;
use std::path::Path;
use std::process;

mod agent_profiles;
mod anchor_log;
mod atomic_file;
mod audit_standard;
mod capsule_sign;
mod cli;
mod disclose;
mod lineage_bundle;
mod mcp_serve;
mod policy_gate;
mod settle;
use cli::commands::edit::runtime::{
    check_expect_sha256, edit_output_format, edit_serialize, edit_verify_report, finish_edit_write,
    EditOutputFormat,
};
pub(crate) use cli::commands::edit::{
    measure_cell_overflow, recolor_cell_text_black, resolve_table_cell,
    set_cell_control_char_rejection, CellResolveError,
};
use cli::integrity::{
    cas_test_mark_checked_and_wait, cas_test_synchronize_before_lock, sha256_hex_of, CasPathLock,
};
use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

/// [#2707] CLI 종료 코드 계약 — 성공.
const EXIT_OK: i32 = 0;
/// [#2707] CLI 종료 코드 계약 — 런타임 실패(읽기·파싱·렌더·쓰기).
const EXIT_RUNTIME: i32 = 1;
/// [#2707] CLI 종료 코드 계약 — 사용법 오류(인자 없음, 알 수 없는 옵션/명령, 페이지 범위 초과).
///
/// 3(`--verify` IR 차이)·4(`--verify-pages` 페이지 수 불일치)는
/// `mydocs/manual/cli_commands.md` 에 이미 문서화된 계약이므로 상수화 대상에서 제외하고
/// 기존 `process::exit(3)`/`process::exit(4)` 호출부를 그대로 둔다.
const EXIT_USAGE: i32 = 2;

/// [#2707] 명령 함수가 돌려준 종료 코드를 프로세스 종료 코드로 전파한다.
///
/// 0이면 아무것도 하지 않아 `main` 이 정상 종료하고, 그 외에는 즉시 그 코드로 종료한다.
fn exit_with(exit_code: i32) {
    if exit_code != EXIT_OK {
        process::exit(exit_code);
    }
}

// ============================================================================
// 전역 비밀번호 (--password / --password-stdin, --output-password / --output-password-stdin)
//
// main() 의 pre-scan 이 설정하고 load_document/load_document_core 가 읽는다.
// CLI는 단일 스레드이므로 thread_local 로 전역 상태를 안전하게 전달한다.
// 명령 함수 시그니처를 일일이 바꾸지 않아도 일반 문서 로드 명령에
// 비밀번호를 적용할 수 있다.
// ============================================================================

thread_local! {
    static CLI_PASSWORD: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
    static CLI_OUTPUT_PASSWORD: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

fn set_cli_password(pw: Option<String>) {
    CLI_PASSWORD.with(|c| *c.borrow_mut() = pw);
}

fn cli_password() -> Option<String> {
    CLI_PASSWORD.with(|c| c.borrow().clone())
}

fn set_cli_output_password(pw: Option<String>) {
    CLI_OUTPUT_PASSWORD.with(|c| *c.borrow_mut() = pw);
}

fn cli_output_password() -> Option<String> {
    CLI_OUTPUT_PASSWORD.with(|c| c.borrow().clone())
}

/// 문서 로드 에러 — 비밀번호 필요/불일치/기타를 구분해 종료 코드를 다르게 매핑.
enum LoadError {
    /// 암호 문서인데 비밀번호가 제공되지 않음 (EXIT_USAGE)
    NeedPassword,
    /// 비밀번호 불일치 (EXIT_RUNTIME)
    WrongPassword,
    /// 그 외 파싱 오류 (EXIT_RUNTIME)
    Other(String),
}

impl LoadError {
    /// stderr 에 메시지를 출력하고 매핑된 종료 코드를 반환한다.
    fn report(self) -> i32 {
        match self {
            LoadError::NeedPassword => {
                eprintln!("오류: 비밀번호가 필요한 암호 문서입니다 (--password <pw> 로 전달).");
                EXIT_USAGE
            }
            LoadError::WrongPassword => {
                eprintln!("오류: 비밀번호가 일치하지 않거나 암호화 데이터가 손상되었습니다.");
                EXIT_RUNTIME
            }
            LoadError::Other(msg) => {
                eprintln!("오류: 문서 파싱 실패 - {}", msg);
                EXIT_RUNTIME
            }
        }
    }
}

/// HwpError Display 메시지에서 비밀번호 관련 에러를 분류한다.
/// CryptoError::WrongPassword → "...비밀번호가 일치하지 않...",
/// ParseError::EncryptedDocument → "...비밀번호가 필요한 암호 문서..." 가
/// HwpError::InvalidFile 로 래핑돼 전해지므로 부분문자열로 판별한다.
fn classify_hwp_error(msg: &str) -> LoadError {
    if msg.contains("비밀번호가 일치하지 않") {
        LoadError::WrongPassword
    } else if msg.contains("비밀번호가 필요한 암호 문서") {
        LoadError::NeedPassword
    } else {
        LoadError::Other(msg.to_string())
    }
}

/// HwpDocument 로드. 전역 비밀번호가 설정돼 있으면 비밀번호 경로로 연다.
fn load_document(data: &[u8]) -> Result<rhwp::wasm_api::HwpDocument, LoadError> {
    let result = match cli_password() {
        Some(pw) => rhwp::wasm_api::HwpDocument::from_bytes_with_password(data, pw.as_bytes()),
        None => rhwp::wasm_api::HwpDocument::from_bytes(data),
    };
    result.map_err(|e| classify_hwp_error(&e.to_string()))
}

/// DocumentCore 로드 (export-pdf/export-hml 등). 동일 분기.
fn load_document_core(data: &[u8]) -> Result<rhwp::document_core::DocumentCore, LoadError> {
    let result = match cli_password() {
        Some(pw) => {
            rhwp::document_core::DocumentCore::from_bytes_with_password(data, pw.as_bytes())
        }
        None => rhwp::document_core::DocumentCore::from_bytes(data),
    };
    result.map_err(|e| classify_hwp_error(&e.to_string()))
}

/// `batch` 는 stdin 전체를 파일 경로 목록으로 소비한다. 전역 인증 옵션 중 stdin
/// 변형은 그 목록과 같은 바이트 스트림을 두 번 읽으려 하고, 리터럴 변형도 worker
/// thread-local 인증 상태로 전달되지 않는다. 따라서 암호화 batch 를 정식으로 설계하기
/// 전에는 네 옵션을 모두 호출 경계에서 거부한다.
///
/// 명령 위치 앞의 전역 인증 옵션만 건너뛰어 `batch` 여부를 판정한다. 단순히 모든 인자에서
/// `batch` 문자열을 찾으면 `search --query batch` 같은 정상 호출을 잘못 막게 된다.
fn is_batch_invocation(args: &[String]) -> bool {
    let mut i = 1; // args[0] 은 프로그램 경로
    while let Some(arg) = args.get(i) {
        match arg.as_str() {
            "--password" | "--output-password" => i += 2,
            "--password-stdin" | "--output-password-stdin" => i += 1,
            _ => return arg == "batch",
        }
    }
    false
}

/// `batch` 명령이 실제로 보이면, 그 뒤·앞 어느 위치의 전역 인증 옵션도 거부한다.
fn has_global_auth_option(args: &[String]) -> bool {
    args.iter().skip(1).any(|arg| {
        matches!(
            arg.as_str(),
            "--password" | "--password-stdin" | "--output-password" | "--output-password-stdin"
        )
    })
}

/// Windows PowerShell/.NET이 UTF-8 표준입력의 첫 바이트에 붙일 수 있는 BOM은
/// 비밀번호 본문이 아니라 인코딩 표식이다. 첫 줄 암호에 섞이면 정상 비밀번호도
/// 오입력으로 판정되므로, stdin 전체의 맨 앞에서만 제거한다.
fn strip_utf8_bom(input: &str) -> &str {
    input.strip_prefix('\u{feff}').unwrap_or(input)
}

/// args 전체를 스캔해 입력·출력 인증 옵션을 떼어낸다.
///
/// 뽑아낸 입력 암호와 출력 암호는 이 함수 안에서 thread-local 상태로 소비하고,
/// 반환값에는 해당 토큰이 제거된 args 만 담는다. 두 stdin 옵션을 같이 사용하면
/// stdin 첫 줄은 입력, 둘째 줄은 출력 암호로 고정한다.
///
/// 이름과 반환 형태가 "정제된 args" 인 것은 의도적이다. 비밀번호를 반환값(과거의
/// `(args, password)` 튜플)에 싣거나 함수 이름에 `password` 를 두면 CodeQL
/// `rust/cleartext-logging` 이 이 호출의 결과 전체를 민감 데이터로 보고, 비밀번호
/// 토큰이 이미 제거된 args 를 쓰는 오류·진단 출력까지 sink 로 분류한다
/// (PR #3405 검토에서 41건 과탐지로 확인, PR #3644 에서 alert #119 로 재발).
/// 반환 경로에 비밀번호가 남지 않으므로 이 분류는 실제 유출 경로가 아니다.
fn strip_global_auth_options(mut args: Vec<String>) -> Result<Vec<String>, i32> {
    let mut password: Option<String> = None;
    let mut output_password: Option<String> = None;
    let mut password_stdin = false;
    let mut output_password_stdin = false;
    let mut i = 1; // args[0] 은 프로그램 경로
    while i < args.len() {
        match args[i].as_str() {
            "--password" => {
                if password.is_some() {
                    eprintln!("오류: 비밀번호 옵션은 한 번만 지정할 수 있습니다.");
                    return Err(EXIT_USAGE);
                }
                if i + 1 >= args.len() {
                    eprintln!("오류: --password 뒤에 비밀번호가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
                password = Some(args[i + 1].clone());
                args.drain(i..=i + 1);
            }
            "--password-stdin" => {
                if password.is_some() || password_stdin {
                    eprintln!("오류: 비밀번호 옵션은 한 번만 지정할 수 있습니다.");
                    return Err(EXIT_USAGE);
                }
                password_stdin = true;
                args.remove(i);
            }
            "--output-password" => {
                if output_password.is_some() || output_password_stdin {
                    eprintln!("오류: 출력 비밀번호 옵션은 한 번만 지정할 수 있습니다.");
                    return Err(EXIT_USAGE);
                }
                if i + 1 >= args.len() {
                    eprintln!("오류: --output-password 뒤에 비밀번호가 필요합니다.");
                    return Err(EXIT_USAGE);
                }
                output_password = Some(args[i + 1].clone());
                args.drain(i..=i + 1);
            }
            "--output-password-stdin" => {
                if output_password.is_some() || output_password_stdin {
                    eprintln!("오류: 출력 비밀번호 옵션은 한 번만 지정할 수 있습니다.");
                    return Err(EXIT_USAGE);
                }
                output_password_stdin = true;
                args.remove(i);
            }
            _ => i += 1,
        }
    }

    if password_stdin || output_password_stdin {
        let mut stdin = String::new();
        if let Err(error) = std::io::stdin().read_to_string(&mut stdin) {
            eprintln!("오류: 표준 입력에서 비밀번호 읽기 실패 - {}", error);
            return Err(EXIT_RUNTIME);
        }
        let stdin = strip_utf8_bom(&stdin);
        let mut lines = stdin.lines();
        if password_stdin {
            password = Some(lines.next().unwrap_or_default().to_string());
        }
        if output_password_stdin {
            output_password = Some(lines.next().unwrap_or_default().to_string());
        }
    }
    if let Some(value) = output_password.as_deref() {
        if value.is_empty() || value.len() > 4096 || value.contains(['\r', '\n']) {
            eprintln!("오류: 출력 비밀번호는 빈 값·줄바꿈 없이 UTF-8 4096바이트 이하여야 합니다.");
            return Err(EXIT_USAGE);
        }
    }
    set_cli_password(password);
    set_cli_output_password(output_password);
    Ok(args)
}

/// 쪽수와 IR 검증은 모두 수행하되, 종료 코드는 쪽수 실패를 우선한다.
fn verification_exit_code(page_failed: bool, ir_failed: bool) -> i32 {
    if page_failed {
        4
    } else if ir_failed {
        3
    } else {
        EXIT_OK
    }
}

#[cfg(test)]
mod verification_exit_code_tests {
    use super::verification_exit_code;

    #[test]
    fn page_failure_keeps_precedence_when_ir_also_fails() {
        assert_eq!(verification_exit_code(false, false), 0);
        assert_eq!(verification_exit_code(false, true), 3);
        assert_eq!(verification_exit_code(true, false), 4);
        assert_eq!(verification_exit_code(true, true), 4);
    }
}

fn main() {
    let raw_args: Vec<String> = env::args().collect();
    if is_batch_invocation(&raw_args) && has_global_auth_option(&raw_args) {
        eprintln!(
            "오류: batch 는 --password·--password-stdin·--output-password·--output-password-stdin 을 지원하지 않습니다. stdin 은 파일 경로 목록 전용입니다."
        );
        process::exit(EXIT_USAGE);
    }
    // 전역 인증 pre-scan: 어느 위치든 입력/출력 비밀번호 옵션을 뽑아낸다.
    // 비밀번호는 pre-scan 안에서 thread-local 상태로 들어가고 여기로는 돌아오지 않는다.
    let args = match strip_global_auth_options(raw_args) {
        Ok(v) => v,
        Err(code) => process::exit(code),
    };

    // [#5791] `rhwp <명령> [<하위명령>] --help` — 그 명령 절만 내고 exit 0.
    // 디스패치보다 앞이다: 하위 명령 판정이나 옵션 파서에 닿기 전에 답해야
    // "알 수 없는 옵션"·"파일을 읽을 수 없습니다"로 떨어지지 않는다.
    if let Some(code) = cli::metadata::help::scoped_help(&args[1..]) {
        process::exit(code);
    }

    match args.get(1).map(|s| s.as_str()) {
        Some("--help") | Some("-h") => cli::metadata::help::print_help(),
        Some("--version") | Some("-V") => println!("rhwp v{}", rhwp::version()),
        Some("export-svg") => exit_with(cli::outputs::vector::export_svg(&args[2..])),
        Some("export-render-tree") => {
            exit_with(cli::outputs::vector::export_render_tree(&args[2..]))
        }
        Some("export-structure") => exit_with(cli::outputs::vector::export_structure(&args[2..])),
        Some("export-png") => exit_with(cli::outputs::raster::export_png(&args[2..])),
        // [gym_gpu_raster] GPU 가속 PNG 래스터화 (feature = "gpu"). export-png(native-skia)과
        // 같은 방식으로 feature 게이팅 — 미빌드 바이너리는 사용법 오류(exit 2)로 안내한다.
        Some("export-png-gpu") => exit_with(cli::outputs::raster::export_png_gpu(&args[2..])),
        Some("gpu-info") => exit_with(cli::outputs::raster::gpu_info(&args[2..])),
        Some("export-pdf") => exit_with(cli::outputs::pdf::export_pdf(&args[2..])),
        Some("export-text") => exit_with(cli::outputs::text::export_text(&args[2..])),
        Some("export-markdown") => exit_with(cli::outputs::text::export_markdown(&args[2..])),
        Some("export-tables") => exit_with(cli::outputs::tabular::export_tables(&args[2..])),
        Some("export-llm") => exit_with(cli::outputs::text::export_llm(&args[2..])),
        Some("table-to-csv") => exit_with(cli::outputs::tabular::table_to_csv(&args[2..])),
        Some("csv-to-table") => exit_with(cli::commands::tabular_import::csv_to_table(&args[2..])),
        Some("chart-to-csv") => exit_with(cli::outputs::tabular::chart_to_csv(&args[2..])),
        Some("csv-to-chart") => exit_with(cli::commands::tabular_import::csv_to_chart(&args[2..])),
        Some("export-hwpx") => exit_with(cli::commands::conversion::export_hwpx(&args[2..])),
        Some("export-hml") => cli::commands::conversion::export_hml(&args[2..]),
        Some("export-doclang") => exit_with(cli::outputs::doclang::export_doclang(&args[2..])),
        Some("export-ir-schema") => exit_with(cmd_export_ir_schema(&args[2..])),
        Some("export-capabilities-schema") => exit_with(cmd_export_capabilities_schema(&args[2..])),
        Some("export-ontology") => exit_with(cmd_export_ontology(&args[2..])),
        Some("capabilities") => {
            exit_with(cli::metadata::capabilities::show_capabilities(&args[2..]))
        }
        Some("export-provenance-map") => exit_with(
            cli::metadata::capabilities::export_provenance_map(&args[2..]),
        ),
        Some("export-agent-manifest") => exit_with(cmd_export_agent_manifest(&args[2..])),
        Some("mcp-serve") => exit_with(mcp_serve::run(&args[2..])),
        Some("batch") => exit_with(cli::batch::run(&args[2..])),
        Some("scan") => exit_with(cli::queries::scan::run(&args[2..])),
        Some("threat-scan") => {
            exit_with(cli::queries::security_inspection::threat_scan(&args[2..]))
        }
        Some("info") => exit_with(cli::queries::info::run(&args[2..])),
        Some("word-count") => exit_with(cli::queries::document_inventory::word_count(&args[2..])),
        Some("bookmarks") => exit_with(cli::queries::document_inventory::bookmarks(&args[2..])),
        Some("charts") => exit_with(cli::queries::document_inventory::charts(&args[2..])),
        Some("form-value") => exit_with(cli::queries::structured_objects::form_value(&args[2..])),
        Some("header-footer") => {
            exit_with(cli::queries::structured_objects::header_footer(&args[2..]))
        }
        Some("headers-footers") => exit_with(cli::queries::structured_objects::headers_footers(
            &args[2..],
        )),
        Some("digest") => exit_with(cli::queries::digest::digest_document(&args[2..])),
        Some("dump") => exit_with(cli::queries::control_dump::run(&args[2..])),
        Some("dump-note-shape") => {
            exit_with(cli::queries::diagnostics::dump_note_shape(&args[2..]))
        }
        Some("dump-endnote-lines") => {
            exit_with(cli::queries::diagnostics::dump_endnote_lines(&args[2..]))
        }
        Some("dump-pages") => exit_with(cli::queries::page_dump::run(&args[2..])),
        Some("dump-extents") => exit_with(cli::queries::diagnostics::dump_extents(&args[2..])),
        Some("diag") => exit_with(cli::queries::diagnostics::diag_document(&args[2..])),
        Some("search") => exit_with(cli::queries::search::search_document(&args[2..])),
        Some("inspect") => exit_with(inspect_command(&args[2..])),
        Some("armor") => exit_with(cli::queries::security_inspection::armor_command(&args[2..])),
        Some("extract-data") => exit_with(cli::queries::data_extraction::extract_data_command(
            &args[2..],
        )),
        Some("convert") => exit_with(cli::commands::conversion::convert_hwp(&args[2..])),
        Some("extract-pages") => exit_with(cli::commands::conversion::extract_pages(&args[2..])),
        Some("build-from-ingest") => {
            exit_with(cli::commands::generation::build_from_ingest(&args[2..]))
        }
        Some("scaffold") => exit_with(cli::commands::generation::run_scaffold(&args[2..])),
        Some("hwp5-inventory") => exit_with(rhwp::diagnostics::hwp5_inventory::run(&args[2..])),
        Some("hwp5-inventory-diff") => {
            exit_with(rhwp::diagnostics::hwp5_inventory_diff::run(&args[2..]))
        }
        Some("hwp5-contract-analyze") => {
            exit_with(rhwp::diagnostics::hwp5_contract_analyze::run(&args[2..]))
        }
        Some("hwp5-ctrl-data-trace") => {
            exit_with(rhwp::diagnostics::hwp5_ctrl_data_trace::run(&args[2..]))
        }
        Some("hwp5-contract-probe") => {
            exit_with(rhwp::diagnostics::hwp5_contract_probe::run(&args[2..]))
        }
        Some("hwp5-table-probe") => exit_with(rhwp::diagnostics::hwp5_table_probe::run(&args[2..])),
        Some("hwp5-mel-personnel-probe") => {
            exit_with(rhwp::diagnostics::hwp5_mel_personnel_probe::run(&args[2..]))
        }
        Some("hwp5-borderfill-diagonal-probe") => exit_with(
            rhwp::diagnostics::hwp5_borderfill_diagonal_probe::run(&args[2..]),
        ),
        Some("hwp5-first-para-control-probe") => exit_with(
            rhwp::diagnostics::hwp5_first_para_control_probe::run(&args[2..]),
        ),
        Some("hwp5-anchor-trace") => {
            exit_with(rhwp::diagnostics::hwp5_anchor_trace::run(&args[2..]))
        }
        Some("hwp5-char-shape-audit") => {
            exit_with(rhwp::diagnostics::hwp5_char_shape_audit::run(&args[2..]))
        }
        Some("hwp5-cell-header-probe") => {
            exit_with(rhwp::diagnostics::hwp5_cell_header_probe::run(&args[2..]))
        }
        Some("dump-records") => exit_with(cli::queries::diagnostics::dump_raw_records(&args[2..])),
        Some("test-shape") => exit_with(test_shape_roundtrip(&args[2..])),
        Some("test-caption") => exit_with(test_caption(&args[2..])),
        Some("gen-table") => exit_with(gen_table(&args[2..])),
        Some("gen-pua") => exit_with(gen_pua_test(&args[2..])),
        Some("test-field") => exit_with(cli::commands::internal_validation::run(&args[2..])),
        Some("ir-diff") => exit_with(cli::queries::ir_comparison::ir_diff(&args[2..])),
        Some("ir-sweep") => exit_with(cli::queries::ir_comparison::ir_sweep(&args[2..])),
        Some("dump-anchors") => {
            exit_with(cli::queries::position_diagnostics::dump_anchors(&args[2..]))
        }
        Some("dump-carets") => {
            exit_with(cli::queries::position_diagnostics::dump_carets(&args[2..]))
        }
        Some("verify") => exit_with(cli::queries::verification::run(&args[2..])),
        Some("hwpx-roundtrip") => rhwp::diagnostics::hwpx_roundtrip_batch::run(&args[2..]),
        Some("hwp5-roundtrip") => rhwp::diagnostics::hwp5_roundtrip_batch::run(&args[2..]),
        Some("render-diff") => rhwp::diagnostics::render_geom_diff::run(&args[2..]),
        Some("layout-anomaly") => exit_with(rhwp::diagnostics::layout_anomaly::run(&args[2..])),
        Some("measure-width") => exit_with(rhwp::diagnostics::text_width_probe::run(&args[2..])),
        Some("core-pages") => exit_with(rhwp::diagnostics::core_pages_probe::run(&args[2..])),
        Some("bench") => exit_with(rhwp::diagnostics::bench::run(&args[2..])),
        Some("thumbnail") => exit_with(cli::outputs::preview::extract_thumbnail(&args[2..])),
        Some("fields") => exit_with(cli::queries::document_inventory::show_fields(&args[2..])),
        Some("explain") => exit_with(cli::queries::explain::explain_document(&args[2..])),
        Some("explore") => exit_with(cli::queries::explore::explore_document(&args[2..])),
        Some("edit") => exit_with(cli::commands::edit::run(&args[2..])),
        Some("run") => exit_with(cli::protocol::cmd_run_plan(&args[2..])),
        Some("replay") => exit_with(cli::protocol::cmd_replay(&args[2..])),
        Some("audit") => exit_with(cli::protocol::cmd_audit(&args[2..])),
        Some("lineage") => exit_with(cli::protocol::cmd_lineage(&args[2..])),
        Some("keygen") => exit_with(cli::protocol::cmd_keygen(&args[2..])),
        Some("verify-signature") => exit_with(cli::protocol::cmd_verify_signature(&args[2..])),
        Some("harness") => exit_with(cli::protocol::cmd_harness(&args[2..])),
        // [#4537] 통합 판정은 **읽기 전용**이라 쓰기 명령(harness)과 표면을 나눈다 —
        // capabilities 의 category 가 도구 주석(readOnlyHint)의 교차 검증 원천이므로,
        // 한 명령이 쓰기·읽기를 겸하면 MCP 주석 계약이 성립하지 않는다.
        Some("harness-status") => exit_with(cli::protocol::cmd_harness_status(&args[2..])),
        Some("anchor") => exit_with(cli::protocol::cmd_anchor(&args[2..])),
        Some("gate") => exit_with(cli::protocol::cmd_gate(&args[2..])),
        Some("bundle") => exit_with(cli::protocol::cmd_bundle(&args[2..])),
        Some("disclose") => exit_with(cli::protocol::cmd_disclose(&args[2..])),
        Some("settle") => exit_with(cli::protocol::cmd_settle(&args[2..])),
        Some("audit-report") => exit_with(cli::protocol::cmd_audit_report(&args[2..])),
        Some("recall-scope") => exit_with(cli::protocol::cmd_recall_scope(&args[2..])),
        Some("conformance") => exit_with(cli::protocol::cmd_conformance(&args[2..])),
        // [#3719 §6-4] 계획을 *만드는* 쪽의 정답지 — `run` 바로 옆에 둔다.
        Some("export-plan-schema") => exit_with(cmd_export_plan_schema(&args[2..])),
        // [#2707] 알 수 없는 명령·명령 누락은 사용법 오류다. 표준 CLI 관례대로 stderr 로 안내하고
        // 종료 코드 2로 끝낸다(기존에는 stdout + 0이라 오타 낸 명령이 스크립트에서 성공으로 보였다).
        other => {
            // [#4220 T4] 수복 한 줄은 stderr 마지막 줄이어야 하므로(소비자는 마지막
            // `수복: ` 줄 하나만 파싱한다) 산문을 모두 낸 뒤에 방출한다. 두 부류만
            // 결정론적이다: 확신 교정(임계 내 오타)과 명령 누락(발견 경로는 언제나
            // capabilities). 임계 밖 오타는 수복 줄도 침묵한다 — 오제안 0.
            let recovery: Option<(String, &str)> = match other {
                Some(command) => {
                    eprintln!("오류: 알 수 없는 명령입니다 - {}", command);
                    // [#3694] did-you-mean — 후보는 capabilities 단일 출처. 이름 환각을
                    // 교정 단서 없이 돌려보내면 경량 에이전트는 맹목 재시도 루프에 빠진다.
                    let names = cli::metadata::capabilities::capabilities_command_names();
                    let hint = cli::metadata::capabilities::closest_name(
                        command,
                        names.iter().map(String::as_str),
                    );
                    if let Some(hint) = &hint {
                        eprintln!("힌트: 가장 가까운 명령은 '{hint}' 입니다");
                    }
                    hint.map(|h| (h, "요청한 이름이 없음 — 가장 가까운 실존 명령으로 교정"))
                }
                None => {
                    eprintln!("오류: 명령을 지정해주세요.");
                    Some((
                        "capabilities".to_string(),
                        "명령이 지정되지 않음 — 실행 가능한 명령 목록·계약은 capabilities 가 자기서술",
                    ))
                }
            };
            eprintln!("rhwp v{}", rhwp::version());
            eprintln!("사용법: rhwp <명령> [옵션]");
            eprintln!("'rhwp --help'로 자세한 사용법을 확인하세요.");
            if let Some((name, why)) = recovery {
                cli::metadata::capabilities::eprint_usage_recovery(&name, None, why);
            }
            process::exit(EXIT_USAGE);
        }
    }
}

// [#5511] 최상위 dispatch 끝 — 소유 모듈 이동과 무관한 characterization 경계다.

/// [#3261] `export-structure --json`·`batch export-structure --json` 이 공유하는
/// 구조 봉투 레코드. `mode`/`nodeCount` 를 톱레벨로 올려 스윕 선별(jq select)이 싸다.
fn structure_json_value(
    file_path: &str,
    st: &rhwp::document_core::queries::structure::StructureDoc,
) -> serde_json::Value {
    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "mode": st.mode,
            "nodeCount": st.node_count,
            "structure": st,
        }),
        "export-structure",
    )
}

/// [#3346] `export-tables --json` 과 `batch export-tables` 가 공유하는 봉투.
fn tables_json_value(
    file_path: &str,
    tables: &[rhwp::document_core::queries::table_extract::TableGrid],
) -> serde_json::Value {
    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "tableCount": tables.len(),
            "tables": tables,
        }),
        "export-tables",
    )
}

/// [#3346] `fields --json` 과 `batch fields` 가 공유하는 봉투.
fn fields_json_value(file_path: &str, fields: &[serde_json::Value]) -> serde_json::Value {
    let names: Vec<String> = fields
        .iter()
        .filter_map(|f| f["name"].as_str().map(String::from))
        .collect();
    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "fieldCount": fields.len(),
            "fields": fields,
            "textSecurity": text_security_value(&names),
        }),
        "fields",
    )
}

/// 누름틀 이름 축의 유니코드 기만 판정 봉투.
///
/// 봉투에 담기는 이름은 **공격자가 내용을 정할 수 있는 문서**에서 온다. 에이전트는
/// 그 이름으로 "이 칸을 채워라"를 지목하므로, 화면상 같지만 바이트가 다른 이름 쌍이
/// 있으면 엉뚱한 칸이 채워지고도 `filledCount` 는 성공을 보고한다(#3707).
///
/// 판정만 하고 이름을 고치지 않는다 — 문서 엔진이 사용자 문자열을 조용히 바꾸는 것은
/// 어떤 보안 이득으로도 정당화되지 않는다. `status` 는 `clean`/`warning` 2단이고,
/// 항상 실려 나간다: 필드가 없으면 `clean`, 옛 바이너리면 키 자체가 없다 —
/// 소비자가 "검사했는데 깨끗함"과 "검사하지 않음"을 구별할 수 있어야 한다.
fn text_security_value(names: &[String]) -> serde_json::Value {
    use rhwp::document_core::text_security as ts;

    let mut findings: Vec<serde_json::Value> = Vec::new();

    // ① 화면상 같은 이름 쌍 — 실제 공격 서명이다.
    for (_, group) in ts::confusable_collisions(names) {
        findings.push(serde_json::json!({
            "kind": "confusableFieldName",
            "scope": "fieldName",
            "names": group,
            "note": "이름이 화면상 구별되지 않는 누름틀이 둘 이상입니다 — 이름으로 지목해 채우면 의도와 다른 칸이 채워질 수 있습니다. occurrence 대신 hwp_fields 가 돌려준 바이트를 그대로 쓰거나, 사람 확인을 거치세요.",
        }));
    }

    // ② 이름 하나하나의 혼합 스크립트·보이지 않는 문자.
    for name in names {
        for risk in ts::scan_identifier(name) {
            findings.push(serde_json::json!({
                "kind": risk.kind.label(),
                "scope": "fieldName",
                "names": [name],
                "codepoints": risk.codepoints.iter().map(|c| ts::format_codepoint(*c))
                    .collect::<Vec<_>>(),
                "note": risk.kind.describe(),
            }));
        }
    }

    if findings.is_empty() {
        return serde_json::json!({ "status": "clean" });
    }
    serde_json::json!({
        "status": "warning",
        "findingCount": findings.len(),
        "findings": findings,
    })
}

/// [#3346] `search --json` 과 `batch search` 가 공유하는 봉투.
fn search_json_value(
    file_path: &str,
    query: &str,
    case_sensitive: bool,
    matches: &[rhwp::document_core::queries::grep::GrepMatch],
    total_match_count: usize,
) -> serde_json::Value {
    provenance::marked(
        serde_json::json!({
        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
        "source": file_path,
        "query": query,
        "caseSensitive": case_sensitive,
        "matchCount": matches.len(),
        "totalMatchCount": total_match_count,
        "truncated": matches.len() < total_match_count,
        // [#3787 S7] 절단 축의 어휘를 텍스트 축(`export-text --max-chars`)과 맞춘다.
        // `totalMatchCount - matchCount` 로 유도할 수 있는 값이지만, 유도를 요구하면
        // "전부 봤다"는 오독이 그대로 남는다 — 생략량은 명시가 계약이다.
        "omittedCount": total_match_count.saturating_sub(matches.len()),
        "matches": matches,
        }),
        "search",
    )
}

/// [#3787 S7] 페이지 텍스트 산출의 문자 예산 절단 — CLI `export-text --json` 과
/// MCP `hwp_doc_text` 가 같은 규칙을 공유한다.
///
/// **조용히 자르지 않는다.** 거대 문서가 에이전트 컨텍스트를 밀어내는 것을 막는 게
/// 목적이지만, 잘랐다는 사실을 숨기면 그 절단이 "전부 읽었다"는 거짓말이 된다.
/// 그래서 두 가지를 지킨다.
///
/// 1. **쪽 주소를 보존한다** — 예산이 떨어져도 `pages[]` 에서 항목을 빼지 않는다.
///    빼면 `pageCount` 가 줄어 문서가 실제보다 짧아 보인다.
/// 2. **생략량을 남긴다** — 잘린 페이지마다 `truncated:true`·`omittedCount`(생략된
///    문자 수)를 싣고, 봉투 최상위에 합계를 싣는다. 최상위 `truncated` 는 절단이
///    없어도 항상 나가고(false), 페이지 항목의 두 필드는 잘린 페이지에만 붙는다.
///
/// `max_chars` 가 `None` 이면 무제한이다(기본값 — 종전 동작 무변경).
fn truncate_page_texts(
    pages: &[(u32, String)],
    max_chars: Option<usize>,
) -> (Vec<serde_json::Value>, usize) {
    let mut objs = Vec::with_capacity(pages.len());
    let mut budget = max_chars;
    let mut omitted_total = 0usize;
    for (page, text) in pages {
        let total = text.chars().count();
        let keep = match budget {
            Some(remaining) => remaining.min(total),
            None => total,
        };
        if let Some(remaining) = budget.as_mut() {
            *remaining -= keep;
        }
        let omitted = total - keep;
        omitted_total += omitted;
        let kept: String = if omitted == 0 {
            text.clone()
        } else {
            text.chars().take(keep).collect()
        };
        let mut obj = serde_json::json!({ "page": page, "text": kept });
        if omitted > 0 {
            obj["truncated"] = serde_json::json!(true);
            obj["omittedCount"] = serde_json::json!(omitted);
        }
        objs.push(obj);
    }
    (objs, omitted_total)
}

/// [#3407] `title` 이 훑는 앞쪽 페이지 수 상한 — 표지가 이미지·빈 쪽인 문서의
/// fallback 범위. digest 발췌(`DIGEST_EXCERPT_PAGES`)와 같은 "앞 3쪽" 어휘를 쓴다.
const TITLE_SCAN_PAGES: u32 = 3;

/// [#3407] 문서 제목 best-effort 추출 — 대량 아카이브 1-pass 대장화용.
///
/// 렌더된 페이지 텍스트(`extract_page_text_native`, `export-text --json` 과 같은
/// 원천)의 첫 의미 줄(trim 후 비어있지 않은 첫 줄)을 돌려준다. 종전 2-pass
/// 대장화(`batch info` + 문서별 `export-text` 첫 줄 파싱)가 소비자 쪽에서 하던
/// 규칙을 엔진이 한 번만 정의한다. 표지가 이미지라 첫 쪽 텍스트가 비면 다음
/// 쪽으로 내려가며(앞 `TITLE_SCAN_PAGES` 쪽까지), 그래도 없으면 `None`(JSON
/// null)이다. 값 자체는 계약이 아닌 best-effort 필드이고, 추출 실패도 문서
/// 메타 조회를 막지 않도록 조용히 다음 쪽으로 넘어간다.
fn document_title(doc: &rhwp::wasm_api::HwpDocument) -> Option<String> {
    for page in 0..doc.page_count().min(TITLE_SCAN_PAGES) {
        let Ok(text) = doc.extract_page_text_native(page) else {
            continue;
        };
        if let Some(line) = text.lines().map(str::trim).find(|l| !l.is_empty()) {
            return Some(line.to_string());
        }
    }
    None
}

/// [#3237] `info --json`·`batch info --json` 이 공유하는 문서 메타 JSON 레코드.
/// `schemaVersion` 이 계약이며 필드 추가는 허용, 변경·삭제는 계약 테스트가 잡는다.
fn info_json_value(
    file_path: &str,
    file_size: usize,
    detected_format: rhwp::parser::FileFormat,
    doc: &rhwp::wasm_api::HwpDocument,
) -> serde_json::Value {
    let document = doc.document();
    let format_str = match detected_format {
        rhwp::parser::FileFormat::Hwp => "hwp5",
        rhwp::parser::FileFormat::Hwpx => "hwpx",
        rhwp::parser::FileFormat::Hwp3 => "hwp3",
        rhwp::parser::FileFormat::Hml => "hml",
        // 파싱이 성공한 뒤에는 도달하지 않지만, 계약상 문자열은 고정해 둔다.
        rhwp::parser::FileFormat::DrmProtected => "drm-protected",
        rhwp::parser::FileFormat::Empty => "empty",
        rhwp::parser::FileFormat::Unknown => "unknown",
    };
    let version = if detected_format == rhwp::parser::FileFormat::Hml {
        serde_json::Value::Null
    } else {
        serde_json::Value::String(format!(
            "{}.{}.{}.{}",
            document.header.version.major,
            document.header.version.minor,
            document.header.version.build,
            document.header.version.revision,
        ))
    };
    // DOCINFO는 한글·영어·한자·일어·기타·기호·사용자 글꼴군을 따로 보관한다.
    // `info --json`은 문서 인벤토리 용도이므로 첫 번째(한글) 군만이 아니라 선언된
    // 모든 글꼴군을 문서 순서대로 평탄화해 내보낸다. 같은 이름이 여러 군에 있으면
    // 소비자가 출처별 필요에 따라 중복을 보존하거나 제거할 수 있게 그대로 남긴다.
    let fonts: Vec<String> = document
        .doc_info
        .font_faces
        .iter()
        .flatten()
        .map(|face| face.name.clone())
        .collect();
    let para_count: usize = document.sections.iter().map(|s| s.paragraphs.len()).sum();
    let last_saved_with = match detected_format {
        rhwp::parser::FileFormat::Hwp => {
            rhwp::parser::hwp_summary::last_saved_with(&document.extra_streams)
        }
        rhwp::parser::FileFormat::Hwpx => {
            rhwp::parser::hwp_summary::hwpx_last_saved_with(&document.hwpx_aux_entries)
        }
        _ => None,
    }
    .map(|save_version| {
        serde_json::json!({
            "product": save_version.product,
            "version": save_version.version,
            "confidence": "metadata",
        })
    });
    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "format": format_str,
            "sizeBytes": file_size,
            "version": version,
            "sections": document.sections.len(),
            "pageCount": doc.page_count(),
            "paraCount": para_count,
            "fonts": fonts,
            // [#3407] best-effort 문서 제목 — 없으면 null. batch info 로 자동 전파.
            "title": document_title(doc),
            // HWP5 summary 또는 HWPX version.xml 메타데이터다. 원 작성 제품이 아니라
            // 마지막 저장 제품을 가리키며 없거나 수정될 수 있다.
            "lastSavedWith": last_saved_with,
            // [#6208] 문서에 실린 인쇄 방식(모아 찍기 등). rhwp 는 이 값을 **출력에
            // 반영하지 않으므로**, 한글 오라클 PDF 와 대조할 때 `impliesNup` 이 true
            // 면 한글 쪽 장 수·용지 방향이 달라 좌표를 그대로 견주면 오판한다.
            // 값이 문서에 없으면 `printMethod: null`.
            "printMethod": doc.document().doc_info.print_method,
            "printMethodImpliesNup": rhwp::model::document::print_method_implies_nup(
                doc.document().doc_info.print_method,
            ),
            // [#3880 T1] 파싱 중 건너뛴 것을 봉투가 스스로 밝힌다.
            //
            // 인간 출력은 `warnings: N` 과 상세를 stderr 로 내는데 JSON 분기는 그
            // 앞에서 `return EXIT_OK` 로 끝나 도달하지 못했다. 그래서 리소스가 조용히
            // 잘린 문서가 **exit 0 + 완전해 보이는 봉투**를 냈다 — `fonts` 가 부분
            // 목록인데 봉투는 그렇다고 말하지 않았다(#3719 "부분 목록 금지" 위반).
            //
            // 경고가 없으면 빈 배열이다. 키를 빼면 소비자가 "경고 없음"과 "이 빌드는
            // 경고를 모름"을 구별할 수 없다.
            "warnings": info_warnings_value(doc),
        }),
        "info",
    )
}

/// [#3880 T1] `info --json` 의 `warnings[]` — 파싱이 건너뛴 것의 기계 판정용.
///
/// 현재 원천은 HML 파서의 `hml_metadata().warnings` 하나다. 다른 포맷이 같은 기구를
/// 갖추면 여기에 합류시킨다 — 그때까지 이 배열이 비어 있다고 해서 "문서가 온전하다"는
/// 뜻은 아니며, 그 한계는 `mydocs/manual/cli_commands.md` 에 적는다.
fn info_warnings_value(doc: &rhwp::wasm_api::HwpDocument) -> serde_json::Value {
    let Some(metadata) = doc.hml_metadata() else {
        return serde_json::Value::Array(Vec::new());
    };
    serde_json::Value::Array(
        metadata
            .warnings
            .iter()
            .map(|w| {
                serde_json::json!({
                    "code": format!("{:?}", w.code),
                    "xmlPath": w.xml_path,
                    "message": w.message,
                })
            })
            .collect(),
    )
}

/// HWPUNIT(u32)을 mm로 변환
fn hu_to_mm(hu: u32) -> f64 {
    hu as f64 * 25.4 / 7200.0
}

/// HWPUNIT(i32)을 mm로 변환
fn hu_to_mm_i(hu: i32) -> f64 {
    hu as f64 * 25.4 / 7200.0
}

/// [#3719 §6-10] `extract-data --json` 봉투.
///
/// `counts` 는 **요청한 종류에 대한 문서 전체 건수**다(`--limit` 절단 전). 요청하지 않은
/// 종류의 키는 아예 넣지 않는다 — `--kind date` 인데 `"amount": 0` 이 보이면 "금액이 없다"로
/// 오독되기 때문이다. `itemCount` 는 실제 반환된 건수이고, `totalItemCount`·`truncated` 가
/// 절단 사실을 드러낸다(#3353 의 `search` 와 같은 어휘).
fn extract_data_json_value(
    file_path: &str,
    kind: &str,
    items: &[rhwp::document_core::queries::extract_data::DataItem],
    total_item_count: usize,
    counts: &serde_json::Value,
) -> serde_json::Value {
    provenance::marked(
        serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "kind": kind,
            "itemCount": items.len(),
            "totalItemCount": total_item_count,
            "truncated": items.len() < total_item_count,
            "counts": counts,
            "items": items,
        }),
        "extract-data",
    )
}

#[derive(Debug, Default, Clone, Copy)]
struct ConversionVerifyOptions {
    verify: bool,
    verify_pages: bool,
    /// [#3596] 봉투를 stdout 순수 JSON 으로. export-hwpx 만 허용한다(`allow_json`).
    json: bool,
}

impl ConversionVerifyOptions {
    fn enabled(self) -> bool {
        self.verify || self.verify_pages
    }
}

fn paths_refer_to_same_file(input: &Path, output: &Path) -> bool {
    input == output
        || paths_have_same_file_identity(input, output)
        || match (input.canonicalize(), output.canonicalize()) {
            (Ok(input), Ok(output)) => input == output,
            _ => false,
        }
}

#[cfg(unix)]
fn paths_have_same_file_identity(input: &Path, output: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;

    match (input.metadata(), output.metadata()) {
        (Ok(input), Ok(output)) => input.dev() == output.dev() && input.ino() == output.ino(),
        _ => false,
    }
}

#[cfg(not(unix))]
fn paths_have_same_file_identity(_input: &Path, _output: &Path) -> bool {
    false
}

/// 옵션을 받지 않는 내부 개발 명령의 위치 인자를 엄격히 검증한다.
///
/// 이 명령들은 capabilities 에도 노출되어 있다. 플래그처럼 보이는 값을 위치 인자로
/// 삼키거나 여분 인자를 무시하면, 호출자는 오타 난 자동화를 성공으로 오인한다.
fn validate_internal_positionals(command: &str, args: &[String], max: usize) -> Result<(), i32> {
    if let Some(flag) = args.iter().find(|arg| arg.starts_with('-')) {
        eprintln!("오류: {command} 은 알 수 없는 옵션을 받지 않습니다 - {flag}");
        return Err(EXIT_USAGE);
    }
    if args.len() > max {
        eprintln!("오류: {command} 은 위치 인자를 최대 {max}개만 받습니다.");
        return Err(EXIT_USAGE);
    }
    Ok(())
}

fn test_shape_roundtrip(args: &[String]) -> i32 {
    if let Err(code) = validate_internal_positionals("test-shape", args, 2) {
        return code;
    }
    let input = if args.is_empty() {
        "saved/g555-s.hwp"
    } else {
        &args[0]
    };
    let output = if args.len() > 1 {
        &args[1]
    } else {
        "/tmp/test-shape-out.hwp"
    };

    let data = match fs::read(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("입력 파일 읽기 오류: {}", e);
            return EXIT_RUNTIME;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("HWP 파싱 오류: {:?}", e);
            return EXIT_RUNTIME;
        }
    };

    let _ = doc.convert_to_editable_native();

    // 글상자 생성 (9000 x 6750 HWPUNIT)
    let result = doc.create_shape_control_native(
        0,
        0,
        0,
        9000,
        6750,
        0,
        0,
        false,
        "InFrontOfText",
        "rectangle",
        false,
        false,
        &[],
    );
    match &result {
        Ok(r) => eprintln!("글상자 생성 성공: {}", r),
        Err(e) => {
            eprintln!("글상자 생성 실패: {:?}", e);
            return EXIT_RUNTIME;
        }
    }

    match doc.export_hwp_native() {
        Ok(bytes) => {
            if let Err(e) = fs::write(output, &bytes) {
                eprintln!("파일 저장 오류: {}", e);
                return EXIT_RUNTIME;
            }
            eprintln!("저장 완료: {} ({}KB)", output, bytes.len() / 1024);
            EXIT_OK
        }
        Err(e) => {
            eprintln!("직렬화 오류: {:?}", e);
            EXIT_RUNTIME
        }
    }
}

/// 캡션 방향별 테스트: 4개 이미지에 각각 Bottom/Top/Left/Right 캡션을 설정하고 SVG 출력
fn test_caption(args: &[String]) -> i32 {
    if args.is_empty() {
        eprintln!("사용법: rhwp test-caption <파일.hwp> [-o <출력 폴더>]");
        return EXIT_USAGE;
    }
    if args[0].starts_with('-') {
        eprintln!(
            "오류: test-caption 입력 파일 자리에 옵션을 쓸 수 없습니다 - {}",
            args[0]
        );
        return EXIT_USAGE;
    }

    let input = &args[0];
    let mut output_dir = Path::new("output/caption-test");
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                let Some(value) = args.get(i + 1) else {
                    eprintln!("오류: {} 뒤에 출력 폴더 경로가 필요합니다.", args[i]);
                    return EXIT_USAGE;
                };
                if value.starts_with('-') {
                    eprintln!("오류: {} 뒤에 출력 폴더 경로가 필요합니다.", args[i]);
                    return EXIT_USAGE;
                }
                output_dir = Path::new(value);
                i += 2;
            }
            option => {
                eprintln!("오류: 알 수 없는 test-caption 옵션입니다 - {option}");
                return EXIT_USAGE;
            }
        }
    }

    let data = match fs::read(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("파일 읽기 오류: {}", e);
            return EXIT_RUNTIME;
        }
    };

    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("파싱 오류: {}", e);
            return EXIT_RUNTIME;
        }
    };

    if doc.document().sections.is_empty() {
        eprintln!("문서 오류: 캡션을 검사할 section이 없습니다.");
        return EXIT_RUNTIME;
    }

    // 문단 0: 컨트롤 2,3 / 문단 1: 컨트롤 0,1
    let pic_refs: [(usize, usize); 4] = [(0, 2), (0, 3), (1, 0), (1, 1)];

    // 4개 이미지에 각각 다른 캡션 방향 설정
    let directions = [
        ("Bottom", "Top"),
        ("Top", "Top"),
        ("Left", "Center"),
        ("Right", "Center"),
    ];

    for (i, ((para, ci), (dir, va))) in pic_refs.iter().zip(directions.iter()).enumerate() {
        let json = format!(
            r#"{{"hasCaption":true,"captionDirection":"{}","captionVertAlign":"{}","captionWidth":8504,"captionSpacing":850}}"#,
            dir, va
        );
        println!("[{}] para={}, ci={}, dir={}, va={}", i, para, ci, dir, va);
        match doc.set_picture_properties_native(0, *para, *ci, &json) {
            Ok(r) => println!("  결과: {}", r),
            Err(e) => println!("  오류: {:?}", e),
        }
    }

    // 캡션 상태 확인
    // [CLI 계약 정합] capabilities 가 "internal" 카테고리로도 <파일.hwp> 를 받는
    // 일반 명령처럼 자기서술한다 — 에이전트가 임의 문서로 호출할 수 있다는 뜻이다.
    // 이 도구는 원래 para=0/1·control 2/3/0/1 을 가진 고정 fixture 전용이었는데,
    // 그 인덱스를 경계검사 없이 바로 인덱싱해 다른 문서를 주면 패닉(exit 101)했다.
    // "안 죽는다"는 CLI 자기서술 계약을 어기므로, 범위를 벗어나면 패닉 대신
    // 제어된 오류를 출력하고 다음 항목으로 넘어간다.
    for (i, (para, ci)) in pic_refs.iter().enumerate() {
        let Some(section) = doc.document().sections.first() else {
            eprintln!("문서 오류: 캡션을 검사할 section이 없습니다.");
            return EXIT_RUNTIME;
        };
        let Some(p) = section.paragraphs.get(*para) else {
            println!(
                "[{}] 건너뜀: para={} 가 문서 범위를 벗어남(문단 {}개)",
                i,
                para,
                section.paragraphs.len()
            );
            continue;
        };
        let Some(ctrl) = p.controls.get(*ci) else {
            println!(
                "[{}] 건너뜀: para={} ci={} 가 범위를 벗어남(컨트롤 {}개)",
                i,
                para,
                ci,
                p.controls.len()
            );
            continue;
        };
        if let rhwp::model::control::Control::Picture(pic) = ctrl {
            println!(
                "[{}] caption={:?}",
                i,
                pic.caption.as_ref().map(|c| {
                    format!(
                        "dir={:?}, paras={}, text={:?}",
                        c.direction,
                        c.paragraphs.len(),
                        c.paragraphs.first().map(|p| &p.text)
                    )
                })
            );
        }
    }

    // SVG 출력
    if let Err(e) = fs::create_dir_all(output_dir) {
        eprintln!("출력 폴더 생성 오류: {}: {}", output_dir.display(), e);
        return EXIT_RUNTIME;
    }
    let page_count = doc.page_count();
    println!("페이지 수: {}", page_count);
    for p in 0..page_count {
        let svg = match doc.render_page_svg(p) {
            Ok(svg) => svg,
            Err(e) => {
                eprintln!("SVG 렌더링 오류(page {}): {:?}", p, e);
                return EXIT_RUNTIME;
            }
        };
        let path = output_dir.join(format!("caption-test-p{}.svg", p));
        if let Err(e) = fs::write(&path, &svg) {
            eprintln!("SVG 저장 오류: {}: {}", path.display(), e);
            return EXIT_RUNTIME;
        }
        println!("  → {}", path.display());
    }
    println!("완료");
    EXIT_OK
}

fn gen_table(args: &[String]) -> i32 {
    if let Err(code) = validate_internal_positionals("gen-table", args, 3) {
        return code;
    }
    let rows = match args.first() {
        Some(value) => match value.parse::<u16>() {
            Ok(value) => value,
            Err(_) => {
                eprintln!("오류: gen-table 행 수는 0~65535 정수여야 합니다 - {value}");
                return EXIT_USAGE;
            }
        },
        None => 1000,
    };
    let cols = match args.get(1) {
        Some(value) => match value.parse::<u16>() {
            Ok(value) => value,
            Err(_) => {
                eprintln!("오류: gen-table 열 수는 0~65535 정수여야 합니다 - {value}");
                return EXIT_USAGE;
            }
        },
        None => 6,
    };
    let output = args
        .get(2)
        .map(|s| s.as_str())
        .unwrap_or("output/gen_table.hwp");

    println!("{}행 × {}열 표 생성 중...", rows, cols);

    let mut core = rhwp::document_core::DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("빈 문서 생성 실패");

    // 표 생성
    let result = core
        .create_table_native(0, 0, 0, rows, cols)
        .expect("표 생성 실패");
    println!("  표 생성: {}", result);

    // 결과에서 paraIdx 파싱
    let table_para_idx: usize = result
        .split("\"paraIdx\":")
        .nth(1)
        .and_then(|s| s.split(&[',', '}'][..]).next())
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(1);
    println!("  표 문단 인덱스: {}", table_para_idx);

    // 배치 모드로 셀 내용 채우기
    core.begin_batch_native().expect("배치 시작 실패");

    let headers = ["번호", "이름", "부서", "직급", "연락처", "비고"];
    // 헤더 행
    for (ci, header) in headers.iter().enumerate().take(cols as usize) {
        let _ = core.insert_text_in_cell_native(0, table_para_idx, 0, ci, 0, 0, header);
    }

    // 데이터 행
    let departments = ["개발팀", "기획팀", "디자인팀", "영업팀", "인사팀", "재무팀"];
    let positions = ["사원", "대리", "과장", "차장", "부장"];
    for row in 1..rows as usize {
        for col in 0..cols as usize {
            let cell_idx = row * cols as usize + col;
            let text = match col {
                0 => format!("{}", row),
                1 => format!("홍길동{}", row),
                2 => departments[row % departments.len()].to_string(),
                3 => positions[row % positions.len()].to_string(),
                4 => format!(
                    "010-{:04}-{:04}",
                    1000 + row % 9000,
                    1000 + (row * 7) % 9000
                ),
                5 => {
                    if row % 3 == 0 {
                        "특이사항 없음".to_string()
                    } else {
                        String::new()
                    }
                }
                _ => format!("R{}C{}", row, col),
            };
            if !text.is_empty() {
                let _ =
                    core.insert_text_in_cell_native(0, table_para_idx, 0, cell_idx, 0, 0, &text);
            }
        }
        if row % 100 == 0 {
            println!("  {} / {} 행 완료", row, rows);
        }
    }

    core.end_batch_native().expect("배치 종료 실패");
    println!("  셀 내용 입력 완료");

    // 저장
    let bytes = core.export_hwp_native().expect("HWP 내보내기 실패");
    let out_path = Path::new(output);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    if let Err(e) = fs::write(out_path, bytes) {
        // 종료 코드 계약: 쓰기 실패는 런타임 오류(1)다. 종전에는 .expect() 로 패닉해
        // 계약에 없는 101 로 끝났다.
        eprintln!("오류: 파일 저장 실패 - {}: {}", output, e);
        return EXIT_RUNTIME;
    }
    println!("저장 완료: {} ({}행 × {}열)", output, rows, cols);
    EXIT_OK
}

/// PUA (Private Use Area) 문자 셋트를 입력한 HWP 테스트 문서 생성.
///
/// Task #509 (PUA 회귀 정정) 의 한컴 정답지 확보용. 본 라이브러리가 발견한
/// 14 샘플 광범위 PUA 코드포인트 18 종을 한 문서에 입력 → 한컴 편집기로 PDF
/// 출력 + rhwp SVG 출력 시각 비교.
///
/// 사용:
///   rhwp gen-pua [output_path]
///   기본 출력: output/pua-test.hwp
fn gen_pua_test(args: &[String]) -> i32 {
    if let Err(code) = validate_internal_positionals("gen-pua", args, 1) {
        return code;
    }
    // gen-pua 의 positional 은 입력이 아니라 **출력** 경로다. capabilities 가 다른
    // 진단 명령과 나란히 노출하는 탓에 `rhwp gen-pua 문서.hwp` 를 "이 파일을 조사"로
    // 읽은 호출이 실제로 원본을 말없이 덮어썼다(#3691 조사 중 발생). 사용자가 명시한
    // 경로가 이미 있으면 거부한다 — 기본 경로는 재생성 대상이라 검사에서 제외한다.
    let explicit = args.first().map(|s| s.as_str());
    if let Some(path) = explicit {
        if Path::new(path).exists() {
            eprintln!("오류: gen-pua 의 인자는 생성할 **출력** 경로입니다 (입력 파일이 아닙니다).");
            eprintln!("      이미 존재하는 파일을 덮어쓰지 않습니다: {}", path);
            eprintln!("사용법: rhwp gen-pua [출력경로]   # 기본 output/pua-test.hwp");
            return EXIT_USAGE;
        }
    }
    let output = explicit.unwrap_or("output/pua-test.hwp");

    println!("PUA 문자 셋트 입력 HWP 문서 생성 중...");

    let mut core = rhwp::document_core::DocumentCore::new_empty();
    core.create_blank_document_native()
        .expect("빈 문서 생성 실패");

    // PUA 코드포인트 셋트 (Task #509 Stage 1 의 14 샘플 광범위 통계 정합)
    // (codepoint, 영역 분류, 사용 샘플, 본 라이브러리 현재 매핑)
    let pua_set: &[(u32, &str, &str, &str)] = &[
        // ── Basic PUA (0xF020~0xF0FF) — 매핑 표 적용 영역 ──
        (0x0F076, "Basic", "mel-001", "❖ U+2756"),
        (0x0F09F, "Basic", "biz_plan", "• U+2022"),
        (0x0F0A0, "Basic", "synam-001", "▪ U+25AA"),
        (0x0F0A7, "Basic", "kps-ai", "▪ U+25AA"),
        (0x0F0E8, "Basic", "kps-ai", "(미정의)"),
        (0x0F0F2, "Basic", "KTX", "⇩ U+21E9 (의도 정정 후보)"),
        (0x0F0FE, "Basic", "k-water-rfp", "☑ U+2611"),
        // ── Basic PUA — 매핑 표 외 영역 ──
        (0x0F53A, "Basic-out", "hwpspec", "(매핑 표 외)"),
        // ── Supplementary PUA-A (0xF0000~0xFFFFD) — 매핑 표 미지원 영역 ──
        (0xF02B1, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B2, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B3, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B4, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B5, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B6, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B7, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B8, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02B9, "Suppl-A", "mel-001", "(매핑 표 외)"),
        (0xF02EF, "Suppl-A", "KTX (회귀)", "(매핑 표 외) ★"),
    ];

    println!("  PUA 코드포인트 {} 종 입력", pua_set.len());

    core.begin_batch_native().expect("배치 시작 실패");

    // 첫 paragraph (0번) 에 제목 입력
    let title = "[PUA 회귀 검증 — Task #509]";
    core.insert_text_native(0, 0, 0, title)
        .expect("제목 입력 실패");

    // 각 PUA 글자별로 paragraph 추가:
    // "U+0F0F2 (Basic, KTX): {char}    ← 한컴 정답지 / rhwp 비교"
    // 빈 paragraph 추가 + 텍스트 입력 패턴
    for (i, &(cp, area, sample, mapping)) in pua_set.iter().enumerate() {
        let pi = i + 1; // 0번은 제목, 1번부터 PUA paragraphs

        // 새 paragraph 추가 (pi 위치에 새 문단 삽입)
        core.insert_paragraph_native(0, pi)
            .unwrap_or_else(|e| panic!("paragraph 추가 실패 (pi={}): {:?}", pi, e));

        // PUA 글자 char 변환 (i32 unsafe 회피)
        let pua_char =
            char::from_u32(cp).unwrap_or_else(|| panic!("invalid codepoint U+{:05X}", cp));

        // 텍스트: "U+0F0F2 (Basic, KTX, ⇩ U+21E9 매핑): " + PUA + "  ← 한컴 PDF 글리프 정답지"
        let text = format!(
            "U+{:05X} ({}, {}, {}): {}  ← 한컴 PDF 정답지",
            cp, area, sample, mapping, pua_char
        );

        core.insert_text_native(0, pi, 0, &text)
            .unwrap_or_else(|e| panic!("텍스트 입력 실패 (pi={}): {:?}", pi, e));
    }

    core.end_batch_native().expect("배치 종료 실패");

    // 저장
    let bytes = core.export_hwp_native().expect("HWP 내보내기 실패");
    let out_path = Path::new(output);
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).ok();
    }
    if let Err(e) = fs::write(out_path, bytes) {
        // 종료 코드 계약: 쓰기 실패는 런타임 오류(1)다. 종전에는 .expect() 로 패닉해
        // 계약에 없는 101 로 끝났다.
        eprintln!("오류: 파일 저장 실패 - {}: {}", output, e);
        return EXIT_RUNTIME;
    }
    println!("저장 완료: {} ({} 종 PUA)", output, pua_set.len());
    println!();
    println!("다음 단계:");
    println!("  1. 한컴 2022 편집기에서 본 파일 열기 → PDF 출력 (정답지)");
    println!("  2. rhwp export-svg {} → SVG 출력 비교", output);
    println!("  3. 시각 비교로 매핑 정합 확정");
    EXIT_OK
}

/// [#3346] `fields --json` 과 `batch fields` 가 공유하는 필드 레코드 수집.
///
/// 단건/배치가 같은 스키마를 내도록 한 곳에서 만든다.
pub(crate) fn collect_field_records(doc: &rhwp::wasm_api::HwpDocument) -> Vec<serde_json::Value> {
    use rhwp::document_core::queries::field_query::NestedEntry;

    doc.collect_all_fields()
        .iter()
        .map(|fi| {
            // 중첩 경로: 표 셀·글상자 안의 필드가 어디에 있는지 — 후속 편집의 좌표다.
            let nested: Vec<serde_json::Value> = fi
                .location
                .nested_path
                .iter()
                .map(|e| match e {
                    NestedEntry::TableCell {
                        control_index,
                        cell_index,
                        para_index,
                    } => serde_json::json!({
                        "kind": "tableCell",
                        "control": control_index,
                        "cell": cell_index,
                        "paragraph": para_index,
                    }),
                    NestedEntry::TextBox {
                        control_index,
                        para_index,
                    } => serde_json::json!({
                        "kind": "textBox",
                        "control": control_index,
                        "paragraph": para_index,
                    }),
                })
                .collect();

            serde_json::json!({
                "fieldId": fi.field.field_id,
                "fieldType": format!("{:?}", fi.field.field_type),
                "name": fi.field.field_name().unwrap_or(""),
                "guide": fi.field.guide_text().unwrap_or(""),
                "memo": fi.field.memo_text().unwrap_or_default(),
                "command": fi.field.command,
                "value": fi.value,
                "editableInForm": fi.field.is_editable_in_form(),
                "location": {
                    "section": fi.location.section_index,
                    "paragraph": fi.location.para_index,
                    "nested": nested,
                },
            })
        })
        .collect()
}

/// [#3762] `export-ir-schema` — 공개 IR 의 JSON Schema 를 낸다 (M18 바인딩 착수 조건).
///
/// 문서를 입력으로 받지 않는다 — 스키마는 **타입의 자기서술**이지 특정 문서의
/// 속성이 아니다. capabilities 가 명령 표면을 설명하듯, 이 명령은 문서 모델을
/// 설명한다. 외부 바인딩 세대가 코드 생성의 단일 출처로 쓴다.
fn cmd_export_ir_schema(args: &[String]) -> i32 {
    let mut out_path: Option<&str> = None;
    let mut json_mode = false;
    let mut bare = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            // 봉투 없이 스키마 본문만 — JSON Schema 도구에 바로 먹이려는 용도.
            "--bare" => bare = true,
            "-o" | "--out" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.as_str()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {}", other);
                return EXIT_USAGE;
            }
        }
        i += 1;
    }

    let payload = if bare {
        // --bare 는 JSON Schema 검증기에 그대로 먹이는 본문이다 — 봉투 표지를 섞지 않는다.
        rhwp::ir_schema::ir_schema()
    } else {
        // [#3885] "표지는 항상 실린다" — 문서를 열지 않는 명령의 봉투도
        // untrustedContent:false 를 명시한다. 키 부재는 "안전"이 아니라
        // "이 빌드는 표지를 모른다"로 읽히기 때문이다.
        provenance::marked(rhwp::ir_schema::envelope(), "export-ir-schema")
    };
    let text = match serde_json::to_string_pretty(&payload) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("오류: 스키마 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    if let Some(path) = out_path {
        if let Err(e) = fs::write(path, text.as_bytes()) {
            eprintln!("오류: 스키마를 쓸 수 없습니다 - {}: {}", path, e);
            return EXIT_RUNTIME;
        }
        if json_mode {
            // 파일로 뺐어도 stdout 은 기계 계약을 유지한다 — 어디에 썼는지 알려준다.
            println!(
                "{}",
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                        "irSchemaVersion": rhwp::ir_schema::IR_SCHEMA_VERSION,
                        "output": path,
                        "bytes": text.len(),
                    }),
                    "export-ir-schema"
                )
            );
        } else {
            println!("IR 스키마 저장: {} ({} bytes)", path, text.len());
        }
        return EXIT_OK;
    }

    println!("{text}");
    EXIT_OK
}

/// [#3719 §6-4] `export-plan-schema` — `run` 계획서 문법의 JSON Schema 를 낸다.
///
/// 문서를 입력으로 받지 않는다 — 스키마는 **계획서 문법의 자기서술**이지 특정 문서의
/// 속성이 아니다. `run --json` 이 이미 쓴 계획을 검사한다면, 이 명령은 계획을 **쓰기
/// 전에** 읽는 정답지다. 필드명을 지어내고 `invalid[]` 로 되돌아오는 왕복이 계획 생성
/// 실패의 대부분이라, 그 왕복을 없애는 것이 목적이다.
fn cmd_export_plan_schema(args: &[String]) -> i32 {
    let mut out_path: Option<&str> = None;
    let mut json_mode = false;
    let mut bare = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            // 봉투 없이 스키마 본문만 — JSON Schema 검증기에 바로 먹이려는 용도.
            "--bare" => bare = true,
            "-o" | "--out" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.as_str()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {}", other);
                return EXIT_USAGE;
            }
        }
        i += 1;
    }

    let payload = if bare {
        // --bare 는 JSON Schema 검증기에 그대로 먹이는 본문이다 — 봉투 표지를 섞지 않는다.
        rhwp::plan_schema::plan_schema()
    } else {
        // [#3787 S1] "표지는 항상 실린다" — 문서를 열지 않는 명령의 봉투도
        // untrustedContent:false 를 명시한다는 것이 capabilities 의 선언이다.
        provenance::marked(rhwp::plan_schema::envelope(), "export-plan-schema")
    };
    let text = match serde_json::to_string_pretty(&payload) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("오류: 스키마 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    if let Some(path) = out_path {
        if let Err(e) = fs::write(path, text.as_bytes()) {
            eprintln!("오류: 스키마를 쓸 수 없습니다 - {}: {}", path, e);
            return EXIT_RUNTIME;
        }
        if json_mode {
            // 파일로 뺐어도 stdout 은 기계 계약을 유지한다 — 어디에 썼는지 알려준다.
            println!(
                "{}",
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                        "planSchemaVersion": rhwp::plan_schema::PLAN_SCHEMA_VERSION,
                        "output": path,
                        "bytes": text.len(),
                    }),
                    "export-plan-schema"
                )
            );
        } else {
            println!("계획 스키마 저장: {} ({} bytes)", path, text.len());
        }
        return EXIT_OK;
    }

    println!("{text}");
    EXIT_OK
}

/// [#3776] `export-capabilities-schema` — capabilities 자체의 JSON Schema 를 낸다.
fn cmd_export_capabilities_schema(args: &[String]) -> i32 {
    let mut out_path: Option<&str> = None;
    let mut json_mode = false;
    let mut bare = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--bare" => bare = true,
            "-o" | "--out" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.as_str()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {}", other);
                return EXIT_USAGE;
            }
        }
        i += 1;
    }

    let payload = if bare {
        // --bare 는 JSON Schema 검증기에 그대로 먹이는 본문이다 — 봉투 표지를 섞지 않는다.
        rhwp::capabilities_schema::capabilities_schema()
    } else {
        // [#3885] export-ir-schema 와 같은 사유 — 문서를 열지 않아도 표지는 싣는다.
        provenance::marked(
            rhwp::capabilities_schema::envelope(),
            "export-capabilities-schema",
        )
    };
    let text = match serde_json::to_string_pretty(&payload) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("오류: 스키마 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    if let Some(path) = out_path {
        if let Err(e) = fs::write(path, text.as_bytes()) {
            eprintln!("오류: 스키마를 쓸 수 없습니다 - {}: {}", path, e);
            return EXIT_RUNTIME;
        }
        if json_mode {
            println!(
                "{}",
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                        "capabilitiesSchemaVersion":
                            rhwp::capabilities_schema::CAPABILITIES_SCHEMA_VERSION,
                        "output": path,
                        "bytes": text.len(),
                    }),
                    "export-capabilities-schema"
                )
            );
        } else {
            println!("capabilities 스키마 저장: {} ({} bytes)", path, text.len());
        }
        return EXIT_OK;
    }

    println!("{text}");
    EXIT_OK
}

/// [#3907 O1] `export-ontology` — 자기서술에서 JSON-LD 온톨로지를 기계 유도한다.
///
/// 문서를 입력으로 받지 않는다 — 온톨로지는 rhwp 라는 **도구 자신**(IR 타입·명령
/// 표면·신뢰 경계)의 서술이지 특정 문서의 속성이 아니다. 유도 원천은 전부 같은
/// 크레이트의 단일 출처 함수다: `ir_schema()`·`cli::metadata::capabilities::capabilities_value()`·
/// `cli::metadata::mcp::mcp_tool_definitions()`·`provenance::MAP`. 손 나열 상수가 없으므로 원천이
/// 바뀌면 온톨로지가 함께 바뀐다 — 드리프트 구조적 불가능이 이 명령의 논지다.
/// 문서 인스턴스 모드(O2)는 후속이다.
fn cmd_export_ontology(args: &[String]) -> i32 {
    let mut out_path: Option<&str> = None;
    let mut json_mode = false;
    let mut bare = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            // 봉투 없이 JSON-LD 본문만 — RDF/JSON-LD 도구에 바로 먹이려는 용도.
            "--bare" => bare = true,
            "-o" | "--out" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.as_str()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            other => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {}", other);
                return EXIT_USAGE;
            }
        }
        i += 1;
    }

    let caps = cli::metadata::capabilities::capabilities_value();
    let tools = cli::metadata::mcp::mcp_tool_definitions();
    let payload = if bare {
        // --bare 는 JSON-LD 처리기에 그대로 먹이는 본문이다 — 봉투 표지를 섞지 않는다.
        rhwp::ontology::ontology(&caps, &tools)
    } else {
        // [#3885] "표지는 항상 실린다" — 문서를 열지 않는 명령의 봉투도
        // untrustedContent:false 를 명시한다.
        provenance::marked(rhwp::ontology::envelope(&caps, &tools), "export-ontology")
    };
    let text = match serde_json::to_string_pretty(&payload) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("오류: 온톨로지 직렬화 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    if let Some(path) = out_path {
        if let Err(e) = fs::write(path, text.as_bytes()) {
            eprintln!("오류: 온톨로지를 쓸 수 없습니다 - {}: {}", path, e);
            return EXIT_RUNTIME;
        }
        if json_mode {
            // 파일로 뺐어도 stdout 은 기계 계약을 유지한다 — 어디에 썼는지 알려준다.
            println!(
                "{}",
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                        "ontologyVersion": rhwp::ontology::ONTOLOGY_VERSION,
                        "output": path,
                        "bytes": text.len(),
                    }),
                    "export-ontology"
                )
            );
        } else {
            println!("온톨로지 저장: {} ({} bytes)", path, text.len());
        }
        return EXIT_OK;
    }

    println!("{text}");
    EXIT_OK
}

/// [#3828 B2] `export-agent-manifest` 조립 코어 — capabilities·irSchema·provenanceMap·
/// planSchema 를 왕복 1회로 묶는다.
///
/// 각 서브필드는 해당 명령의 기존 산출 함수를 그대로 불러 조립만 한다 — 스키마·지도
/// 로직을 여기서 다시 만들지 않는다. `missingAxes` 는 네 축이 모두 실린 지금 빈
/// 배열이지만 필드 자체는 남긴다 — 앞으로 축이 늘 때 "아직 없는 축"을 이 배열로
/// 알리는 것이 B2 의 계약이고, null 로 채우면 "값이 비었다"와 "명령이 아직 없다"를
/// 소비자가 구분할 수 없다.
fn agent_manifest_value(bare: bool) -> serde_json::Value {
    let mut fields = serde_json::Map::new();
    fields.insert(
        "capabilities".to_string(),
        provenance::marked(
            cli::metadata::capabilities::capabilities_value(),
            "capabilities",
        ),
    );
    fields.insert("irSchema".to_string(), rhwp::ir_schema::ir_schema());
    fields.insert(
        "provenanceMap".to_string(),
        provenance::marked(
            provenance::map_json(&rhwp::version()),
            "export-provenance-map",
        ),
    );
    // [#3808] planSchema 축 — irSchema 처럼 bare 본문을 싣는다. 본문이 `$id`·
    // `planSchemaVersion` 을 자체 내장하므로 봉투 메타를 중복하지 않는다.
    fields.insert("planSchema".to_string(), rhwp::plan_schema::plan_schema());
    fields.insert("missingAxes".to_string(), serde_json::json!([]));

    if bare {
        return serde_json::Value::Object(fields);
    }
    let mut envelope = serde_json::Map::new();
    envelope.insert(
        "schemaVersion".to_string(),
        serde_json::json!(ENVELOPE_SCHEMA_VERSION),
    );
    envelope.extend(fields);
    serde_json::Value::Object(envelope)
}

/// [#3828 B2] `export-agent-manifest` — 처음 붙는 에이전트가 capabilities →
/// export-ir-schema → export-provenance-map → export-plan-schema 를 각각 따로
/// 호출하던 왕복 4회를 1회로 줄인다.
fn cmd_export_agent_manifest(args: &[String]) -> i32 {
    let mut json_mode = false;
    let mut bare = false;
    for arg in args {
        match arg.as_str() {
            "--json" => json_mode = true,
            "--bare" => bare = true,
            other => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
        }
    }

    let manifest = provenance::marked(agent_manifest_value(bare), "export-agent-manifest");

    if json_mode {
        let text = match serde_json::to_string_pretty(&manifest) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("오류: 매니페스트 직렬화 실패 - {}", e);
                return EXIT_RUNTIME;
            }
        };
        println!("{text}");
        return EXIT_OK;
    }

    println!("rhwp 에이전트 매니페스트 (capabilities + irSchema + provenanceMap 조립)");
    println!();
    println!("  capabilities     포함");
    println!("  irSchema         포함");
    println!("  provenanceMap    포함");
    println!("  planSchema       포함");
    println!();
    println!("기계 계약은 --json 을 쓰세요 (--bare 로 최상위 표지 없이).");
    EXIT_OK
}

/// [#3787 S2] `tool_directive` 판정에 쓰는 **도구 이름 등록부**.
///
/// 이름을 탐지 모듈에 하드코딩하지 않는다. 도구가 늘어도 목록이 따라오지 않으면
/// 새 도구를 부르는 주입문이 조용히 통과하기 때문이다. 원천은 이 저장소가 이미
/// 가진 두 등록부다 — 무상태 도구는 `cli::metadata::mcp::mcp_tool_definitions()`(= `capabilities --mcp`
/// 의 stdout), 세션 도구는 `agent_profiles::ALL_SESSION_TOOLS`(= `mcp-serve` 가 여는
/// 집합). 둘 중 어디에 도구를 더해도 탐지가 함께 자란다.
fn mcp_tool_name_registry() -> Vec<String> {
    let mut names: Vec<String> = cli::metadata::mcp::mcp_tool_definitions()
        .iter()
        .filter_map(|t| t["name"].as_str().map(String::from))
        .collect();
    names.extend(
        agent_profiles::ALL_SESSION_TOOLS
            .iter()
            .map(|s| s.to_string()),
    );
    names.sort();
    names.dedup();
    names
}

/// `inspect` — 문서를 **읽기만** 하는 보안 검사 명령군.
///
/// `hidden-text`·`injection`·`unicode`는 각각 조판 은닉, 문장형 지시, 화면과 바이트의
/// 불일치를 판정한다. 어느 축도 문서를 고치지 않는다.
fn inspect_command(args: &[String]) -> i32 {
    const USAGE: &str =
        "사용법: rhwp inspect <hidden-text|injection|unicode|watermark> <파일.hwp|파일.hwpx> [각 축 옵션]";

    match args.first().map(|s| s.as_str()) {
        Some("hidden-text") => cli::queries::security_inspection::inspect_hidden_text(&args[1..]),
        Some("injection") => cli::queries::security_inspection::inspect_injection(&args[1..]),
        Some("unicode") => cli::queries::security_inspection::inspect_unicode(&args[1..]),
        Some("watermark") => cli::queries::security_inspection::inspect_watermark(&args[1..]),
        Some(other) => {
            eprintln!("오류: 알 수 없는 inspect 하위 명령입니다 - {other}");
            let hint = cli::metadata::capabilities::closest_name(
                other,
                ["hidden-text", "injection", "unicode", "watermark"],
            );
            if let Some(hint) = &hint {
                eprintln!("혹시 이것인가요? inspect {hint}");
            }
            eprintln!("{USAGE}");
            // [#4220 T4] 확신 교정(#3694 임계 내)일 때만 정형 수복 줄 — 임계 밖은 침묵.
            if let Some(hint) = hint {
                cli::metadata::capabilities::eprint_usage_recovery(
                    "inspect",
                    Some(&hint),
                    "요청한 이름이 없음 — 가장 가까운 실존 하위 명령으로 교정",
                );
            }
            EXIT_USAGE
        }
        None => {
            // [#4220 T4] 하위 명령 누락은 어느 축을 원했는지 결정론적으로 알 수 없다 —
            // 수복 줄을 지어내지 않는다(오제안 0).
            eprintln!(
                "오류: inspect 하위 명령을 지정해주세요 (hidden-text|injection|unicode|watermark)."
            );
            eprintln!("{USAGE}");
            EXIT_USAGE
        }
    }
}

/// 현재 스캔이 실제로 훑는 영역 이름 — 봉투와 사람 출력이 같은 목록을 쓴다.
fn injection_scan_scopes(include_fields: bool) -> Vec<&'static str> {
    let mut scopes = vec![
        "body",
        "tableCell",
        "textBox",
        "equation",
        "footnote",
        "endnote",
        "header",
        "footer",
        "caption",
    ];
    if include_fields {
        scopes.extend([
            "fieldName",
            "fieldGuide",
            "fieldCommand",
            "hiddenComment",
            "fieldMemo",
        ]);
    }
    scopes
}

/// 터미널로 나가는 발췌의 제어문자를 보이는 기호로 바꾼다.
///
/// 문서 텍스트는 고치지 않는다 — 여기서 바뀌는 것은 **화면 표시**뿐이다(`--json` 봉투는
/// serde 가 `\u001b` 로 이스케이프하므로 손대지 않는다). 주입 문서가 ANSI 이스케이프를
/// 함께 심으면 경고 줄 자체를 지우거나 색으로 덮어 사람을 속일 수 있다.
fn display_safe(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\u{1b}' => '␛',
            '\n' | '\r' => '⏎',
            '\t' => '⇥',
            c if (c as u32) < 0x20 => '␀',
            c => c,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::cli::outputs::allows_implicit_sibling_resources;
    use crate::cli::protocol::{
        collect_audit_capsules, replay_scratch_dir, with_replay_input_snapshot,
    };

    use super::{
        cli_output_password, cli_password, set_cli_output_password, set_cli_password,
        strip_global_auth_options, strip_utf8_bom, EXIT_USAGE,
    };
    use rhwp::parser::FileFormat;

    #[test]
    fn hml_does_not_implicitly_load_sibling_resources() {
        assert!(!allows_implicit_sibling_resources(FileFormat::Hml));
        assert!(allows_implicit_sibling_resources(FileFormat::Hwp));
        assert!(allows_implicit_sibling_resources(FileFormat::Hwpx));
    }

    #[test]
    fn replay_engine_receives_the_hashed_input_snapshot() {
        let original =
            std::env::temp_dir().join(format!("rhwp-replay-original-{}.hwp", std::process::id()));
        std::fs::write(&original, b"original bytes").expect("원본 작성");
        let mut plan = serde_json::json!({ "input": original.to_string_lossy() });
        let scratch = replay_scratch_dir("unit").expect("전용 임시 폴더");
        let scratch_path = scratch.0.clone();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&scratch_path)
                    .expect("전용 임시 폴더 metadata")
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
        }
        let seen = with_replay_input_snapshot(
            &mut plan,
            b"hashed snapshot",
            &scratch.0,
            |snapshot_plan| {
                std::fs::write(&original, b"changed after hashing").expect("원본 교체");
                let snapshot_path = snapshot_plan["input"].as_str().expect("스냅샷 경로");
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    assert_eq!(
                        std::fs::metadata(snapshot_path)
                            .expect("입력 스냅샷 metadata")
                            .permissions()
                            .mode()
                            & 0o777,
                        0o600
                    );
                }
                std::fs::read(snapshot_path).expect("스냅샷 읽기")
            },
        )
        .expect("스냅샷 실행");
        assert_eq!(seen, b"hashed snapshot");
        assert_eq!(plan["input"], original.to_string_lossy().as_ref());
        drop(scratch);
        assert!(!scratch_path.exists(), "전용 임시 폴더는 RAII 정리");
        let _ = std::fs::remove_file(original);
    }

    #[test]
    fn audit_directory_entry_errors_are_not_silently_dropped() {
        let entries: [std::io::Result<std::path::PathBuf>; 1] = [Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "denied",
        ))];
        let error = collect_audit_capsules(entries).expect_err("항목 오류는 fail-closed");
        assert!(error.contains("폴더 항목 읽기 실패"));
    }

    #[test]
    fn global_password_option_is_removed_from_any_position() {
        let args = vec![
            "rhwp".to_string(),
            "info".to_string(),
            "sample.hwp".to_string(),
            "--password".to_string(),
            "secret".to_string(),
        ];
        set_cli_password(None);
        let clean = strip_global_auth_options(args).unwrap();
        assert_eq!(clean, ["rhwp", "info", "sample.hwp"]);
        // 비밀번호는 반환값이 아니라 CLI_PASSWORD(thread_local)로 전달된다.
        assert_eq!(cli_password().as_deref(), Some("secret"));
        set_cli_password(None);
    }

    #[test]
    fn password_stdin_ignores_only_a_leading_utf8_bom() {
        assert_eq!(strip_utf8_bom("\u{feff}123456\n"), "123456\n");
        assert_eq!(strip_utf8_bom("123456\n"), "123456\n");
        assert_eq!(
            strip_utf8_bom("123456\n\u{feff}next"),
            "123456\n\u{feff}next"
        );
    }

    #[test]
    fn duplicate_global_password_options_are_rejected() {
        let args = vec![
            "rhwp".to_string(),
            "--password".to_string(),
            "first".to_string(),
            "info".to_string(),
            "sample.hwp".to_string(),
            "--password".to_string(),
            "second".to_string(),
        ];
        assert!(matches!(
            strip_global_auth_options(args),
            Err(code) if code == EXIT_USAGE
        ));
    }

    #[test]
    fn global_output_password_is_removed_without_leaking_into_command_args() {
        let args = vec![
            "rhwp".to_string(),
            "convert".to_string(),
            "source.hwp".to_string(),
            "output.hwp".to_string(),
            "--output-password".to_string(),
            "protected".to_string(),
        ];
        set_cli_password(None);
        set_cli_output_password(None);
        let clean = strip_global_auth_options(args).unwrap();
        assert_eq!(clean, ["rhwp", "convert", "source.hwp", "output.hwp"]);
        assert_eq!(cli_output_password().as_deref(), Some("protected"));
        set_cli_output_password(None);
    }

    #[test]
    fn duplicate_global_output_password_options_are_rejected() {
        let args = vec![
            "rhwp".to_string(),
            "--output-password".to_string(),
            "first".to_string(),
            "convert".to_string(),
            "source.hwp".to_string(),
            "output.hwp".to_string(),
            "--output-password".to_string(),
            "second".to_string(),
        ];
        assert!(matches!(
            strip_global_auth_options(args),
            Err(code) if code == EXIT_USAGE
        ));
    }
}
