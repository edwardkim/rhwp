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

fn edit_set_chart_data(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-chart-data <파일> --chart N --data <JSON> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut chart_arg: Option<usize> = None;
    let mut data: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--chart" => {
                i += 1;
                match args.get(i).map(|v| v.parse::<usize>()) {
                    Some(Ok(n)) if n >= 1 => chart_arg = Some(n),
                    _ => {
                        eprintln!("오류: --chart 뒤에 1 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--data" => {
                i += 1;
                match args.get(i) {
                    Some(v) => data = Some(v.clone()),
                    None => {
                        eprintln!("오류: --data 뒤에 JSON 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(chart_no), Some(data)) = (file_path, chart_arg, data) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        match doc.set_chart_data_by_index_native(chart_no - 1, &data) {
            Ok(raw) => {
                let parsed: serde_json::Value =
                    serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null);
                if parsed["ok"] != true {
                    if json_mode {
                        let envelope = serde_json::json!({
                            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                            "source": file_path,
                            "count": chart_no,
                            "dryRun": false,
                            "invalid": parsed["invalid"],
                        });
                        println!("{}", provenance::marked(envelope, "edit"));
                    } else {
                        eprintln!("오류: 차트 데이터 변경 실패 - {raw}");
                    }
                    return EXIT_USAGE;
                }
            }
            Err(e) => {
                eprintln!("오류: 차트 데이터 변경 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "chartdata",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "count": chart_no }),
        &[(0, 0)],
        &format!("차트 데이터 예정: {file_path} 차트 {chart_no}"),
        &format!("차트 데이터 기록 완료: {file_path}"),
    )
}

fn edit_insert_number(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-number <파일> [--section N] [--para N] [--offset N] [--count N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut start_num: u16 = 1;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" | "--count" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--offset" => match v.parse::<usize>() {
                        Ok(n) => offset = n,
                        Err(_) => {
                            eprintln!("오류: --offset 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) if n >= 1 => start_num = n,
                        Ok(_) => {
                            eprintln!("오류: --count 는 1 이상 65535 이하여야 합니다.");
                            return EXIT_USAGE;
                        }
                        Err(_) => {
                            eprintln!("오류: --count 뒤에 1 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.insert_new_number_native(section, para, offset, start_num) {
            eprintln!("오류: 새 번호 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "newnum",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "offset": offset,
            "count": start_num,
        }),
        &[(section, para)],
        &format!(
            "새 번호 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset} 번호 {start_num}"
        ),
        &format!("새 번호 삽입 완료: {file_path}"),
    )
}

fn edit_insert_shape(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-shape <파일> --width N --height N [--section N] [--para N] [--offset N] [--x N] [--y N] [--shape rectangle] [--wrap InFrontOfText] [--treat-as-char] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut width_arg: Option<u32> = None;
    let mut height_arg: Option<u32> = None;
    let mut x_hu: u32 = 0;
    let mut y_hu: u32 = 0;
    let mut shape_type = "rectangle".to_string();
    let mut wrap = "InFrontOfText".to_string();
    let mut treat_as_char = false;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        _ => offset = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--width" | "--height" | "--x" | "--y" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT).");
                    return EXIT_USAGE;
                };
                match v.parse::<u32>() {
                    Ok(n) => match name.as_str() {
                        "--width" => width_arg = Some(n),
                        "--height" => height_arg = Some(n),
                        "--x" => x_hu = n,
                        _ => y_hu = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT): {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--shape" => {
                i += 1;
                match args.get(i) {
                    Some(v) => shape_type = v.clone(),
                    None => {
                        eprintln!("오류: --shape 뒤에 도형 종류가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--wrap" => {
                i += 1;
                match args.get(i) {
                    Some(v) => wrap = v.clone(),
                    None => {
                        eprintln!("오류: --wrap 뒤에 감싸기 값이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--treat-as-char" => treat_as_char = true,
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(width), Some(height)) = (file_path, width_arg, height_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if width == 0 && height == 0 {
        eprintln!("오류: --width 와 --height 가 모두 0입니다.");
        return EXIT_USAGE;
    }
    if shape_type.trim().is_empty() {
        eprintln!("오류: --shape 은 비어 있을 수 없습니다.");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.create_shape_control_native(
            section,
            para,
            offset,
            width,
            height,
            x_hu,
            y_hu,
            treat_as_char,
            &wrap,
            &shape_type,
            false,
            false,
            &[],
        ) {
            eprintln!("오류: 도형 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "shape",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "offset": offset,
            "width": width,
            "height": height,
            "x": x_hu,
            "y": y_hu,
        }),
        &[(section, para)],
        &format!("도형 삽입 예정: {file_path} 구역 {section} 문단 {para} {width}x{height}"),
        &format!("도형 삽입 완료: {file_path}"),
    )
}

fn edit_delete_shape(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-shape <파일> --section N --para N --ctrl N [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl)) = (file_path, section, para, ctrl)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.delete_shape_control_native(section, para, ctrl) {
            eprintln!("오류: 도형 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delshape",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("도형 삭제 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("도형 삭제 완료: {file_path}"),
    )
}

fn edit_group_shapes(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit group-shapes <파일> --targets P,C;P,C [--section N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut targets: Vec<(usize, usize)> = Vec::new();
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => section = n,
                    Err(_) => {
                        eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--targets" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --targets 뒤에 para,ctrl;para,ctrl 목록이 필요합니다.");
                    return EXIT_USAGE;
                };
                match parse_shape_targets(v) {
                    Some(list) => targets.extend(list),
                    None => {
                        eprintln!("오류: --targets 형식이 아닙니다 (예: 0,1;0,2): {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--target" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --target 뒤에 para,ctrl 이 필요합니다.");
                    return EXIT_USAGE;
                };
                match parse_shape_target(v) {
                    Some(pair) => targets.push(pair),
                    None => {
                        eprintln!("오류: --target 형식이 아닙니다 (예: 0,1): {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if targets.len() < 2 {
        eprintln!("오류: 묶으려면 --targets 또는 --target 을 2개 이상 지정해야 합니다.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let mut group_para = targets[0].0;
    let mut group_ctrl = targets[0].1;
    if !dry_run {
        match doc.group_shapes_native(section, &targets) {
            Ok(raw) => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                    if let Some(n) = v["paraIdx"].as_u64() {
                        group_para = n as usize;
                    }
                    if let Some(n) = v["controlIdx"].as_u64() {
                        group_ctrl = n as usize;
                    }
                }
            }
            Err(e) => {
                eprintln!("오류: 도형 묶기 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "grpshape",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": group_para,
            "ctrl": group_ctrl,
            "count": targets.len(),
        }),
        &[(section, group_para)],
        &format!(
            "도형 묶기 예정: {file_path} 구역 {section} {}개",
            targets.len()
        ),
        &format!("도형 묶기 완료: {file_path}"),
    )
}

fn edit_set_form_value(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-form-value <파일> --section N --para N --ctrl N --value <JSON> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut value: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--value" => {
                i += 1;
                match args.get(i) {
                    Some(v) => value = Some(v.clone()),
                    None => {
                        eprintln!("오류: --value 뒤에 JSON 이 필요합니다. 예: {{\"value\":1}}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl), Some(value)) =
        (file_path, section, para, ctrl, value)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if value.trim().is_empty() {
        eprintln!("오류: --value 는 비어 있을 수 없습니다.");
        return EXIT_USAGE;
    }
    match serde_json::from_str::<serde_json::Value>(&value) {
        Ok(v) if v.is_object() => {}
        _ => {
            eprintln!("오류: --value 는 JSON 객체여야 합니다. 예: {{\"value\":1}} 또는 {{\"text\":\"입력값\"}}");
            return EXIT_USAGE;
        }
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        match doc.set_form_value_native(section, para, ctrl, &value) {
            Ok(raw) => {
                let v: serde_json::Value =
                    serde_json::from_str(&raw).unwrap_or(serde_json::json!({}));
                if v["ok"] == false {
                    let err = v["error"].as_str().unwrap_or("양식 값 설정 실패");
                    eprintln!("오류: {err}");
                    return EXIT_RUNTIME;
                }
            }
            Err(e) => {
                eprintln!("오류: 양식 값 설정 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "formval",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "ctrl": ctrl,
            "value": value
        }),
        &[(section, para)],
        &format!("양식 값 설정 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("양식 값 설정 완료: {file_path}"),
    )
}

fn edit_set_form_value_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-form-value-in-cell <파일> --section N --table-para N --table-ci N --cell N --cell-para N --ctrl N --value <JSON> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut table_para: Option<usize> = None;
    let mut table_ci: Option<usize> = None;
    let mut cell: Option<usize> = None;
    let mut cell_para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut value: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--table-para" | "--table-ci" | "--cell" | "--cell-para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--table-para" => table_para = Some(n),
                        "--table-ci" => table_ci = Some(n),
                        "--cell" => cell = Some(n),
                        "--cell-para" => cell_para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--value" => {
                i += 1;
                match args.get(i) {
                    Some(v) => value = Some(v.clone()),
                    None => {
                        eprintln!("오류: --value 뒤에 JSON 이 필요합니다. 예: {{\"value\":1}}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (
        Some(file_path),
        Some(section),
        Some(table_para),
        Some(table_ci),
        Some(cell),
        Some(cell_para),
        Some(ctrl),
        Some(value),
    ) = (
        file_path, section, table_para, table_ci, cell, cell_para, ctrl, value,
    )
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if value.trim().is_empty() {
        eprintln!("오류: --value 는 비어 있을 수 없습니다.");
        return EXIT_USAGE;
    }
    match serde_json::from_str::<serde_json::Value>(&value) {
        Ok(v) if v.is_object() => {}
        _ => {
            eprintln!("오류: --value 는 JSON 객체여야 합니다. 예: {{\"value\":1}} 또는 {{\"text\":\"입력값\"}}");
            return EXIT_USAGE;
        }
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        match doc.set_form_value_in_cell_native(
            section, table_para, table_ci, cell, cell_para, ctrl, &value,
        ) {
            Ok(raw) => {
                let v: serde_json::Value =
                    serde_json::from_str(&raw).unwrap_or(serde_json::json!({}));
                if v["ok"] == false {
                    let err = v["error"].as_str().unwrap_or("셀 양식 값 설정 실패");
                    eprintln!("오류: {err}");
                    return EXIT_RUNTIME;
                }
            }
            Err(e) => {
                eprintln!("오류: 셀 양식 값 설정 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "formcell",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "tablePara": table_para,
            "tableCi": table_ci,
            "cell": cell,
            "cellPara": cell_para,
            "ctrl": ctrl,
            "value": value
        }),
        &[(section, table_para)],
        &format!(
            "셀 양식 값 설정 예정: {file_path} 구역 {section} 표문단 {table_para} 표컨트롤 {table_ci} 셀 {cell}"
        ),
        &format!("셀 양식 값 설정 완료: {file_path}"),
    )
}

fn edit_set_page_border_fill(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-page-border-fill <파일> --props <JSON> [--section N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut props: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => section = n,
                    Err(_) => {
                        eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 쪽 테두리/배경 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(props)) = (file_path, props) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if serde_json::from_str::<serde_json::Value>(props).is_err() {
        eprintln!("오류: --props 는 JSON 객체여야 합니다: {props}");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.set_page_border_fill_native(section, props) {
            eprintln!("오류: 쪽 테두리/배경 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "pgborder",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "props": props
        }),
        &[(section, 0)],
        &format!("쪽 테두리/배경 예정: {file_path} 구역 {section}"),
        &format!("쪽 테두리/배경 완료: {file_path}"),
    )
}

fn parse_shape_targets(raw: &str) -> Option<Vec<(usize, usize)>> {
    let mut out = Vec::new();
    for piece in raw.split([';', '|']) {
        let piece = piece.trim();
        if piece.is_empty() {
            continue;
        }
        out.push(parse_shape_target(piece)?);
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn parse_shape_target(raw: &str) -> Option<(usize, usize)> {
    let sep = if raw.contains(',') {
        ','
    } else if raw.contains(':') {
        ':'
    } else {
        return None;
    };
    let mut parts = raw.split(sep);
    let para = parts.next()?.trim().parse::<usize>().ok()?;
    let ctrl = parts.next()?.trim().parse::<usize>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((para, ctrl))
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

// ─── [#3719 §6-11] 공개 전 정리 — edit redact / edit sanitize ───

/// `-o` 도 `--in-place` 도 없이 원본을 덮어쓰려 할 때의 거부 메시지.
///
/// 마스킹은 되돌릴 수 없다. "실수로 원본을 잃는" 경로를 아예 만들지 않기 위해,
/// 산출 경로를 **명시하지 않으면 실행하지 않는다**(다른 edit 명령의 `_replaced` 류
/// 기본 이름조차 만들지 않는다 — 어디에 무엇이 생겼는지 모른 채로 두지 않기 위해).
const REDACT_DESTINATION_REQUIRED: &str = "오류: 마스킹은 되돌릴 수 없습니다. \
     산출 경로를 -o <출력> 으로 지정하거나, 원본을 덮어쓸 의도라면 --in-place 를 \
     명시하세요 (먼저 --dry-run 으로 무엇이 지워질지 확인하기를 권합니다).";

/// `edit redact` — 개인정보를 찾아 자릿수를 유지한 채 마스킹한다.
///
/// 탐지는 [`rhwp::document_core::queries::pii_scan`] 의 읽기 전용 판정을 쓰고, 실제
/// 변경은 검증된 치환 경로(`replace_all_native`)를 재사용한다 — 새 편집 로직이 없다.
/// 되돌릴 수 없는 작업이라 ① `--dry-run` 이 권장 흐름이고 ② 산출 경로를 명시하지
/// 않으면 exit 2 로 거부한다.
fn edit_redact(args: &[String]) -> i32 {
    use rhwp::document_core::queries::pii_scan::PiiKind;

    let mut file_path: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut kinds: Vec<PiiKind> = Vec::new();
    let mut mask_char: char = '*';
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut in_place = false;
    let mut no_raw = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--kind" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("오류: --kind 뒤에 ssn|phone|email|card|all 이 필요합니다.");
                    return EXIT_USAGE;
                };
                for token in value.split(',').map(str::trim).filter(|t| !t.is_empty()) {
                    if token == "all" {
                        kinds.extend(PiiKind::all());
                        continue;
                    }
                    match PiiKind::parse(token) {
                        Some(k) => kinds.push(k),
                        None => {
                            eprintln!(
                                "오류: 알 수 없는 --kind 값 - {token} (ssn|phone|email|card|all)"
                            );
                            return EXIT_USAGE;
                        }
                    }
                }
            }
            "--mask" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("오류: --mask 뒤에 마스킹 문자 한 글자가 필요합니다.");
                    return EXIT_USAGE;
                };
                let mut chars = value.chars();
                match (chars.next(), chars.next()) {
                    // 두 글자 이상이면 자릿수 보존이 깨진다 — 조용히 자르지 않고 거부한다.
                    (Some(c), None) if !c.is_alphanumeric() => mask_char = c,
                    (Some(_), None) => {
                        eprintln!("오류: --mask 는 영숫자가 아닌 문자여야 합니다 (예: * # ●).");
                        return EXIT_USAGE;
                    }
                    _ => {
                        eprintln!("오류: --mask 는 정확히 한 글자여야 합니다 (자릿수 보존).");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--in-place" => in_place = true,
            "--dry-run" => dry_run = true,
            "--verify" => verify_mode = true,
            "--json" => json_mode = true,
            "--no-raw" => no_raw = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let Some(file_path) = file_path else {
        eprintln!(
            "사용법: rhwp edit redact <파일.hwp|파일.hwpx> [--kind ssn|phone|email|card|all] [--mask <문자>] [--dry-run] [--no-raw] [--verify] [-o <출력>|--in-place] [--json]"
        );
        return EXIT_USAGE;
    };
    if kinds.is_empty() {
        kinds.extend(PiiKind::all());
    }
    kinds.sort_unstable();
    kinds.dedup();

    if out_path.is_some() && in_place {
        eprintln!("오류: -o 와 --in-place 는 함께 쓸 수 없습니다 (산출 경로가 모호합니다).");
        return EXIT_USAGE;
    }
    // 원본 보호 — 산출 경로가 없는 실제 실행은 거부한다(--dry-run 은 아무것도 쓰지 않음).
    if !dry_run && out_path.is_none() && !in_place {
        eprintln!("{REDACT_DESTINATION_REQUIRED}");
        return EXIT_USAGE;
    }
    // -o 로 원본을 지목한 경우도 같은 사고다 — 의도를 --in-place 로 말하게 한다.
    if let Some(out) = out_path.as_deref() {
        if !in_place && same_existing_path(file_path, out) {
            eprintln!("{REDACT_DESTINATION_REQUIRED}");
            return EXIT_USAGE;
        }
    }

    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let findings = doc.scan_pii(&kinds, mask_char);
    let changed_paras: Vec<(usize, usize)> = {
        let mut v: Vec<(usize, usize)> =
            findings.iter().map(|f| (f.section, f.paragraph)).collect();
        v.sort_unstable();
        v.dedup();
        v
    };

    // 치환은 값 단위 전량이다. 긴 값을 먼저 바꿔야 짧은 값이 긴 값의 부분열일 때
    // 원문을 깨뜨리지 않는다.
    let mut targets: Vec<(String, String)> = Vec::new();
    for f in &findings {
        if !targets.iter().any(|(raw, _)| *raw == f.raw) {
            targets.push((f.raw.clone(), f.masked.clone()));
        }
    }
    targets.sort_by(|a, b| b.0.chars().count().cmp(&a.0.chars().count()));

    let mut redacted_count = 0usize;
    if !dry_run {
        for (raw, masked) in &targets {
            match doc.replace_all_native(raw, masked, true) {
                Ok(result) => {
                    redacted_count += serde_json::from_str::<serde_json::Value>(&result)
                        .ok()
                        .and_then(|v| v["count"].as_u64())
                        .unwrap_or(0) as usize;
                }
                Err(e) => {
                    // 실패 시 원본 불변 — 출력 파일을 쓰지 않고 즉시 끝낸다.
                    eprintln!("오류: 마스킹 실패 - {:?}", e);
                    return EXIT_RUNTIME;
                }
            }
        }
    }

    let out_format = edit_output_format(&bytes, out_path.as_deref());
    let output_path = match (&out_path, in_place) {
        (Some(p), _) => p.clone(),
        (None, true) => file_path.to_string(),
        // 여기 도달하려면 dry-run 이다 — 산출 경로를 쓰지 않는다.
        (None, false) => String::new(),
    };

    // 탐지 0건이면 무변경이다 — 산출물을 만들지 않는다(원본을 그대로 두는 편이 안전하다).
    let wrote_output = !dry_run && redacted_count > 0;
    let mut verify_report = serde_json::Value::Null;
    let mut verify_failed = false;
    if wrote_output {
        let out_bytes = match edit_serialize(&mut doc, out_format) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "오류: {} 직렬화 실패 - {}",
                    out_format.label().to_uppercase(),
                    e
                );
                return EXIT_RUNTIME;
            }
        };
        if let Err(e) = atomic_file::write_atomically(Path::new(&output_path), &out_bytes) {
            eprintln!("오류: 출력 쓰기 실패 - {}: {}", output_path, e);
            return EXIT_RUNTIME;
        }
        if verify_mode {
            let cross = out_format == EditOutputFormat::Hwp
                && rhwp::parser::detect_format(&bytes) == rhwp::parser::FileFormat::Hwpx;
            let (report, failed) = edit_verify_report(&doc, &out_bytes, cross);
            verify_report = report;
            verify_failed = failed;
        }
    }

    let changed_pages = if wrote_output {
        match doc.pages_covering_paragraphs(&changed_paras) {
            Some(pages) => serde_json::json!(pages),
            None => serde_json::Value::Null,
        }
    } else {
        serde_json::Value::Null
    };

    if json_mode {
        // --no-raw: findings[].raw(원문 개인정보)를 봉투에서 아예 뺀다. `null`로 채우지
        // 않는 이유 — 이 코드베이스는 "선택적으로 없을 수 있는 필드"를 스키마 차원에서
        // 생략으로 표현한다(PiiFinding.page 의 skip_serializing_if 가 같은 관례). raw 를
        // null 로 두면 소비자가 "탐지는 됐지만 값이 비었다"와 "일부러 뺐다"를 구분할
        // 근거가 없어지고, jq 같은 파이프라인에서 null 이 그대로 로그에 찍혀 새 유출
        // 경로가 될 수 있다. 필드 자체가 없으면 그 위험이 구조적으로 사라진다.
        let mut findings_value =
            serde_json::to_value(&findings).unwrap_or(serde_json::Value::Array(Vec::new()));
        if no_raw {
            if let serde_json::Value::Array(items) = &mut findings_value {
                for item in items.iter_mut() {
                    if let serde_json::Value::Object(obj) = item {
                        obj.remove("raw");
                    }
                }
            }
        }
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "kinds": kinds.iter().map(|k| k.as_str()).collect::<Vec<_>>(),
            "mask": mask_char.to_string(),
            "dryRun": dry_run,
            "inPlace": in_place,
            "noRaw": no_raw,
            "findingCount": findings.len(),
            "findings": findings_value,
            "redactedCount": redacted_count,
            "changedPages": changed_pages,
        });
        if wrote_output {
            envelope["output"] = serde_json::Value::String(output_path.clone());
            envelope["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
            envelope["verify"] = verify_report.clone();
        }
        // [#3885] findings[].raw 는 마스킹 전 원문 — 개인정보 그 자체다. 가장 민감한
        // 값을 싣는 봉투가 출처 표지 없이 나가면 S1 계약("표지는 항상 실린다")이
        // 정확히 그 지점에서 무너진다. --no-raw 면 raw 경로가 봉투에 없으므로
        // 표지도 masked 만 선언한다(실재 경로 필터).
        println!("{}", provenance::marked(envelope, "edit"));
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }

    if dry_run {
        println!(
            "마스킹 예정: {} — 탐지 {}건 (원문 {}개). 실제 적용은 -o 또는 --in-place.",
            file_path,
            findings.len(),
            targets.len()
        );
        for f in &findings {
            // --no-raw 는 --json 뿐 아니라 이 사람용 출력에도 적용한다 — 콘솔 로그·
            // 터미널 스크롤백도 유출 경로이므로 절반만 가려서는 목적을 달성하지 못한다.
            let shown_raw: &str = if no_raw {
                "(생략됨, --no-raw)"
            } else {
                &f.raw
            };
            println!(
                "  [{}] {} → {} (구역 {}, 문단 {}, 쪽 {})",
                f.kind,
                shown_raw,
                f.masked,
                f.section,
                f.paragraph,
                f.page
                    .map(|p| (p + 1).to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );
        }
    } else if redacted_count == 0 {
        println!("마스킹 0건: {} — 탐지 없음 (출력 파일 미생성)", file_path);
    } else {
        println!(
            "마스킹 완료: {} → {} — {}건",
            file_path, output_path, redacted_count
        );
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    EXIT_OK
}

/// 두 경로가 **이미 존재하는 같은 파일**을 가리키는지. 판정 불가면 `false`.
///
/// 산출 경로는 대개 존재하지 않으므로 정규화가 실패하는 것이 정상이다. 여기서
/// 잡으려는 것은 `-o` 로 원본 자신을 지목한 경우 하나뿐이다.
fn same_existing_path(a: &str, b: &str) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}

/// `\u{5}HwpSummaryInformation` 에서 지울 속성 — `(PID, 봉투 필드 이름)`.
///
/// PID 는 HWP5 사양의 `HWPPIDSI_*` 다. 본문과 무관한 작성자·이력 메타만 고른다.
const SUMMARY_TARGETS: [(u32, &str); 11] = [
    (0x02, "title"),
    (0x03, "subject"),
    (0x04, "author"),
    (0x05, "keywords"),
    (0x06, "comments"),
    (0x08, "lastSavedBy"),
    (0x09, "revisionNumber"),
    (0x0B, "lastPrintedAt"),
    (0x0C, "createdAt"),
    (0x0D, "lastSavedAt"),
    (0x14, "dateString"),
];

/// FILETIME(1601-01-01 UTC 기준 100ns) → `YYYY-MM-DDTHH:MM:SSZ`.
///
/// 감사 기록용이다 — 무엇을 지웠는지 사람이 읽을 수 있어야 "조용히 지우지 않았다"가
/// 성립한다.
fn filetime_to_iso(ft: u64) -> String {
    const SECS_1601_TO_1970: i64 = 11_644_473_600;
    let secs = (ft / 10_000_000) as i64 - SECS_1601_TO_1970;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Howard Hinnant, civil_from_days (proleptic Gregorian).
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + i64::from(m <= 2);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year,
        m,
        d,
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// `\u{5}HwpSummaryInformation`(OLE 속성 집합)에서 작성자·이력 메타를 지운다.
///
/// **바이트 길이를 바꾸지 않는다** — 속성 오프셋 표가 절대 위치를 담고 있어 크기를
/// 줄이면 나머지 속성이 전부 어긋난다. 문자열은 `cch=1`(NUL 하나)로 만들고 남은
/// 자리를 0으로 덮으며, FILETIME 은 0(미설정)으로 만든다.
///
/// 반환: `(필드 이름, 지우기 전 값)` 목록. 형식을 해석하지 못하면 빈 목록(무변경).
fn sanitize_summary_information(data: &mut [u8]) -> Vec<(String, String)> {
    fn u32_at(d: &[u8], off: usize) -> Option<u32> {
        d.get(off..off + 4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    let mut removed: Vec<(String, String)> = Vec::new();
    if data.len() < 48 || data[0] != 0xFE || data[1] != 0xFF {
        return removed;
    }
    let Some(section_off) = u32_at(data, 44).map(|v| v as usize) else {
        return removed;
    };
    let Some(count) = u32_at(data, section_off + 4).map(|v| v as usize) else {
        return removed;
    };
    // 병적으로 큰 개수는 해석을 포기한다(손상 파일에서 헛돌지 않게).
    if count > 4096 || section_off + 8 + count * 8 > data.len() {
        return removed;
    }

    for idx in 0..count {
        let pair = section_off + 8 + idx * 8;
        let (Some(pid), Some(rel)) = (u32_at(data, pair), u32_at(data, pair + 4)) else {
            continue;
        };
        let Some((_, field)) = SUMMARY_TARGETS.iter().find(|(p, _)| *p == pid) else {
            continue;
        };
        let abs = section_off + rel as usize;
        let Some(vt) = u32_at(data, abs) else {
            continue;
        };
        match vt {
            // VT_LPWSTR — UTF-16LE, cch 는 종단 NUL 을 포함한 문자 수.
            0x1F => {
                let Some(cch) = u32_at(data, abs + 4).map(|v| v as usize) else {
                    continue;
                };
                let start = abs + 8;
                let Some(raw) = data.get(start..start + cch * 2) else {
                    continue;
                };
                let units: Vec<u16> = raw
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .take_while(|u| *u != 0)
                    .collect();
                if units.is_empty() {
                    continue;
                }
                removed.push((field.to_string(), String::from_utf16_lossy(&units)));
                data[start..start + cch * 2].fill(0);
                data[abs + 4..abs + 8].copy_from_slice(&1u32.to_le_bytes());
            }
            // VT_FILETIME.
            0x40 => {
                let Some(raw) = data.get(abs + 4..abs + 12) else {
                    continue;
                };
                let mut bytes = [0u8; 8];
                bytes.copy_from_slice(raw);
                let value = u64::from_le_bytes(bytes);
                if value == 0 {
                    continue;
                }
                removed.push((field.to_string(), filetime_to_iso(value)));
                data[abs + 4..abs + 12].fill(0);
            }
            _ => {}
        }
    }
    removed
}

/// HWPX `Contents/content.hpf` 의 `<opf:metadata>` 블록을 중립 블록으로 바꾼다.
///
/// 이 블록은 직렬화기가 원본에서 그대로 splice 하는 유일한 저작자 정보 경로다
/// (`serializer::hwpx::content::write_content_hpf`). 지우지 않으면 HWPX 산출물에
/// 작성자·작성일이 그대로 남는다. 반환: 지우기 전 블록(있었을 때만).
fn sanitize_hwpx_metadata(entry: &mut Vec<u8>) -> Option<String> {
    const NEUTRAL: &str =
        "<opf:metadata><opf:title/><opf:language>ko</opf:language></opf:metadata>";
    let text = String::from_utf8(entry.clone()).ok()?;
    let open = text.find("<opf:metadata>")?;
    let close = text[open..].find("</opf:metadata>")? + open + "</opf:metadata>".len();
    let before = text[open..close].to_string();
    if before == NEUTRAL {
        return None;
    }
    let mut rebuilt = String::with_capacity(text.len());
    rebuilt.push_str(&text[..open]);
    rebuilt.push_str(NEUTRAL);
    rebuilt.push_str(&text[close..]);
    *entry = rebuilt.into_bytes();
    Some(before)
}

/// 본문 문단 텍스트를 공백·제어문자를 뺀 한 줄로 잇는다 (미리보기 대조용).
///
/// `serializer::cfb_writer::build_preview_text` 와 같은 범위(본문 문단만, 표·글상자 제외).
fn body_text_signature(document: &rhwp::model::document::Document) -> String {
    const MAX: usize = 4000;
    let mut out = String::new();
    for section in &document.sections {
        for para in &section.paragraphs {
            out.extend(
                para.text
                    .chars()
                    .filter(|c| !c.is_whitespace() && !c.is_control()),
            );
            if out.chars().count() >= MAX {
                return out;
            }
        }
    }
    out
}

/// 미리보기 텍스트가 **지금 본문**의 앞부분과 같은지.
///
/// 같으면 유출이 아니라 본문의 파생물이다(저장 시 어차피 같은 값이 다시 만들어진다).
/// 다르면 예전 판의 잔재 — 본문에서 지운 문장이 미리보기에만 남아 있는 전형적 사고다.
fn preview_text_is_current(preview: &str, body_signature: &str) -> bool {
    let stripped: String = preview
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect();
    stripped.is_empty() || body_signature.starts_with(&stripped)
}

/// `edit sanitize` — 문서 메타데이터를 제거한다 (본문은 건드리지 않는다).
///
/// 작성자·회사·최종수정자·작성일과 미리보기(PrvText/PrvImage)를 지운다. 무엇을
/// 지웠는지 `removed[]` 로 남긴다 — 조용히 지우면 감사할 수 없다.
fn edit_sanitize(args: &[String]) -> i32 {
    let mut file_path: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut keep_preview = false;
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--keep-preview" => keep_preview = true,
            "--json" => json_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let Some(file_path) = file_path else {
        eprintln!(
            "사용법: rhwp edit sanitize <파일.hwp|파일.hwpx> [--keep-preview] [-o <출력>] [--json]"
        );
        return EXIT_USAGE;
    };

    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let out_format = edit_output_format(&bytes, out_path.as_deref());
    // HWPX 원본의 `/HwpSummaryInformation` 은 **파일에 없던** 계약 fallback 상수다
    // (`parser::hwpx::contract_streams`). HWPX 로 저장하면 산출물에도 실리지 않으므로
    // 손대지 않는다 — 없던 것을 지웠다고 보고하면 감사 기록이 거짓이 된다. HWP5 로
    // 변환할 때만 실제 산출물에 들어가므로 그때는 지운다.
    let source_is_hwpx = matches!(
        rhwp::parser::detect_format(&bytes),
        rhwp::parser::FileFormat::Hwpx
    );
    let touch_summary = !(source_is_hwpx && out_format == EditOutputFormat::Hwpx);

    let mut removed: Vec<(String, String)> = Vec::new();
    {
        let document = doc.document_mut();

        // ① OLE 요약 정보 (HWP5 원본 · HWPX→HWP5 변환 계약 스트림).
        if touch_summary {
            for (path, data) in document.extra_streams.iter_mut() {
                if !path
                    .trim_start_matches(['/', '\u{5}'])
                    .eq_ignore_ascii_case("HwpSummaryInformation")
                {
                    continue;
                }
                removed.extend(sanitize_summary_information(data));
            }
        }

        // ② HWPX 저작자 메타(content.hpf 의 opf:metadata splice 경로).
        for (path, entry) in document.hwpx_aux_entries.iter_mut() {
            if path != "Contents/content.hpf" {
                continue;
            }
            if let Some(before) = sanitize_hwpx_metadata(entry) {
                removed.push(("hwpx.metadata".to_string(), before));
            }
        }

        // ③ 미리보기 — 예전 판의 잔재가 남는 자리다. 본문에서 이미 지운 문장이
        //    미리보기에만 남아 공개되는 사고가 이 명령의 존재 이유 중 하나다.
        //    지금 본문과 같은 미리보기는 파생물이므로 보고하지 않는다(저장 시 재생성).
        let body_signature = body_text_signature(document);

        if let Some(preview) = document.preview.as_mut() {
            let stale = preview
                .text
                .as_deref()
                .is_some_and(|t| !preview_text_is_current(t, &body_signature));
            if stale {
                if let Some(text) = preview.text.take() {
                    removed.push((
                        "preview.text".to_string(),
                        text.chars().take(60).collect::<String>(),
                    ));
                }
            }
            if !keep_preview {
                if let Some(image) = preview.image.take() {
                    removed.push((
                        "preview.image".to_string(),
                        format!("{:?} {} bytes", image.format, image.data.len()),
                    ));
                }
            }
        }
        if document
            .preview
            .as_ref()
            .is_some_and(|p| p.text.is_none() && p.image.is_none())
        {
            document.preview = None;
        }

        // HWPX 컨테이너의 미리보기 — ZIP 엔트리(HWPX 산출용)와 계약 스트림
        // (HWPX→HWP5 변환용)은 같은 것의 두 표현이므로 함께 지우고 한 번만 보고한다.
        let hwpx_preview_text = document
            .hwpx_aux_entry("Preview/PrvText.txt")
            .and_then(|b| std::str::from_utf8(b).ok())
            .map(str::to_string);
        let drop_hwpx_text = hwpx_preview_text
            .as_deref()
            .is_some_and(|t| !preview_text_is_current(t, &body_signature));
        if drop_hwpx_text {
            if let Some(text) = hwpx_preview_text {
                removed.push((
                    "preview.text".to_string(),
                    text.chars().take(60).collect::<String>(),
                ));
            }
        }
        // 직렬화기는 엔트리가 없으면 빈 자리표시자를 넣는다. 이미 자리표시자면
        // 지울 것이 없다 — 반복 실행이 매번 "지웠다"고 보고하지 않게 한다.
        let drop_hwpx_image = !keep_preview
            && document
                .hwpx_aux_entry("Preview/PrvImage.png")
                .is_some_and(|b| b != rhwp::serializer::hwpx::static_assets::PRV_IMAGE_PNG);
        if drop_hwpx_image {
            if let Some(bytes) = document.hwpx_aux_entry("Preview/PrvImage.png") {
                removed.push((
                    "preview.image".to_string(),
                    format!("Png {} bytes", bytes.len()),
                ));
            }
        }
        document.hwpx_aux_entries.retain(|(path, _)| {
            !(path == "Preview/PrvText.txt" && drop_hwpx_text)
                && !(path == "Preview/PrvImage.png" && drop_hwpx_image)
        });
        document.extra_streams.retain(|(path, _)| {
            !(path == "/PrvText" && drop_hwpx_text) && !(path == "/PrvImage" && !keep_preview)
        });
    }

    let output_path = out_path.unwrap_or_else(|| {
        let stem = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        format!("{}_sanitized.{}", stem, out_format.ext())
    });

    let out_bytes = match edit_serialize(&mut doc, out_format) {
        Ok(b) => b,
        Err(e) => {
            eprintln!(
                "오류: {} 직렬화 실패 - {}",
                out_format.label().to_uppercase(),
                e
            );
            return EXIT_RUNTIME;
        }
    };
    if let Err(e) = atomic_file::write_atomically(Path::new(&output_path), &out_bytes) {
        eprintln!("오류: 출력 쓰기 실패 - {}: {}", output_path, e);
        return EXIT_RUNTIME;
    }

    if json_mode {
        let envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "keepPreview": keep_preview,
            "removedCount": removed.len(),
            "removed": removed
                .iter()
                .map(|(field, before)| serde_json::json!({ "field": field, "before": before }))
                .collect::<Vec<_>>(),
            "output": output_path,
            "outputFormat": out_format.label(),
        });
        // [#3885] removed[].before 는 지워진 문서 속성 원문이다 — 제목·작성자에
        // 더해 preview.text 는 본문 첫 화면 발췌라 문서 문장이 통째로 실린다.
        println!("{}", provenance::marked(envelope, "edit"));
        return EXIT_OK;
    }

    println!(
        "메타 제거 완료: {} → {} — {}건",
        file_path,
        output_path,
        removed.len()
    );
    for (field, before) in &removed {
        println!("  {field}: {before}");
    }
    EXIT_OK
}

/// `edit set-cell` — 표 격자 좌표로 셀 값을 바꾼다 (실물 표 양식 채우기).
///
/// [#3381] 좌표계는 `export-tables` 격자와 동일하다 — 발견과 편집이 같은 주소를 쓴다.
/// 검증된 코어 셀 편집 경로(delete/insert_text_in_cell)를 재사용하므로 새 편집 로직이
/// 없다. v1 범위: 본문 최상위 표, 셀 첫 문단 교체(중첩 표·다문단 셀은 후속).
/// [#3391] 셀 문단 0 의 글자모양을 검정·비이탤릭·비진하게 글자모양 하나로 덮는다.
/// 안내문(파란 이탤릭)을 지우고 실값을 쓰는 set-cell 의 제출 요건(검정 글씨) 대응.
/// 대상 셀의 첫 글자모양을 복제하므로 글꼴·크기·자간은 보존한다. 같은 모양이 이미 있으면
/// 재사용한다.
/// 반환: 적용 성공 여부(좌표 해석 실패 시 false).
fn recolor_cell_text_black(
    document: &mut rhwp::model::document::Document,
    sec: usize,
    para: usize,
    ctrl: usize,
    cell_idx: usize,
) -> bool {
    use rhwp::model::control::Control;
    use rhwp::model::paragraph::CharShapeRef;

    // 대상 셀의 현재 글자모양을 기준으로 해야 한다. 문서 어딘가의 "검정" 모양을 재사용하면
    // 글꼴·크기까지 바뀔 수 있다.
    let source_id = {
        let Some(section) = document.sections.get(sec) else {
            return false;
        };
        let Some(parent) = section.paragraphs.get(para) else {
            return false;
        };
        let Some(Control::Table(table)) = parent.controls.get(ctrl) else {
            return false;
        };
        let Some(cell) = table.cells.get(cell_idx) else {
            return false;
        };
        let Some(paragraph) = cell.paragraphs.first() else {
            return false;
        };
        let Some(shape) = paragraph.char_shapes.first() else {
            return false;
        };
        shape.char_shape_id as usize
    };
    let Some(base) = document
        .doc_info
        .char_shapes
        .get(source_id)
        .or_else(|| document.doc_info.char_shapes.first())
        .cloned()
    else {
        return false;
    };
    let mut black = base;
    black.raw_data = None; // 원본 바이트를 버려 변경된 필드가 직렬화되게 한다.
    black.text_color = 0;
    black.italic = false;
    black.bold = false;
    black.strikethrough = false;
    black.underline_type = rhwp::model::style::UnderlineType::None;
    let black_id = document
        .doc_info
        .char_shapes
        .iter()
        .position(|candidate| candidate == &black)
        .map(|idx| idx as u32)
        .unwrap_or_else(|| {
            let new_id = document.doc_info.char_shapes.len() as u32;
            document.doc_info.char_shapes.push(black);
            new_id
        });

    let Some(section) = document.sections.get_mut(sec) else {
        return false;
    };
    let Some(parent) = section.paragraphs.get_mut(para) else {
        return false;
    };
    let Some(Control::Table(table)) = parent.controls.get_mut(ctrl) else {
        return false;
    };
    let Some(cell) = table.cells.get_mut(cell_idx) else {
        return false;
    };
    let Some(cell_para) = cell.paragraphs.get_mut(0) else {
        return false;
    };
    // 문단 전체를 하나의 검정 글자모양으로 덮는다.
    cell_para.char_shapes = vec![CharShapeRef {
        start_pos: 0,
        char_shape_id: black_id,
    }];
    true
}

/// [#3480] 셀에 넣을 텍스트가 칸 폭을 넘치는지 잰다.
///
/// 넘치면 `(칸 폭 px, 글자 폭 px, 예상 줄 수)` 를 돌려주고, 들어가면 `None`.
/// 폭은 조판 엔진의 글자 폭 추정(`estimate_text_width_px`)과 IR 의 `Cell.width` 를 쓴다.
/// **채우기를 막지는 않는다** — 여러 줄이 정상인 칸도 있으므로 신호만 준다.
fn measure_cell_overflow(
    doc: &rhwp::wasm_api::HwpDocument,
    sec: usize,
    para: usize,
    ctrl: usize,
    cell_idx: usize,
    text: &str,
) -> Option<(f64, f64, usize)> {
    use rhwp::model::control::Control;
    use rhwp::renderer::hwpunit_to_px;

    if text.is_empty() {
        return None;
    }
    let cell = doc
        .document()
        .sections
        .get(sec)?
        .paragraphs
        .get(para)?
        .controls
        .get(ctrl)
        .and_then(|c| match c {
            Control::Table(t) => t.cells.get(cell_idx),
            _ => None,
        })?;

    // 셀 안여백을 뺀 실제 글자 영역 폭.
    let padding = (cell.padding.left + cell.padding.right) as f64;
    let usable = hwpunit_to_px(
        (cell.width as f64 - padding) as i32,
        rhwp::renderer::DEFAULT_DPI,
    );
    if usable <= 0.0 {
        return None;
    }

    let text_w = estimate_text_width_px(doc, sec, para, ctrl, cell_idx, text);
    if text_w <= usable {
        return None;
    }
    let lines = (text_w / usable).ceil() as usize;
    Some((usable, text_w, lines))
}

/// 셀의 첫 문단 글자 모양을 기준으로 텍스트 폭(px)을 추정한다.
///
/// 정밀 조판이 아니라 **넘침 여부 판정용 근사**다 — 한글은 전각, ASCII 는 반각으로 센다.
fn estimate_text_width_px(
    doc: &rhwp::wasm_api::HwpDocument,
    sec: usize,
    para: usize,
    ctrl: usize,
    cell_idx: usize,
    text: &str,
) -> f64 {
    use rhwp::model::control::Control;
    use rhwp::renderer::hwpunit_to_px;

    // 셀 첫 문단의 글자 크기(HWPUNIT, 1pt = 100). 못 찾으면 10pt 로 본다.
    let size_hwpunit = doc
        .document()
        .sections
        .get(sec)
        .and_then(|s| s.paragraphs.get(para))
        .and_then(|p| p.controls.get(ctrl))
        .and_then(|c| match c {
            Control::Table(t) => t.cells.get(cell_idx),
            _ => None,
        })
        .and_then(|cell| cell.paragraphs.first())
        .and_then(|p| p.char_shapes.first())
        .and_then(|cs| {
            doc.document()
                .doc_info
                .char_shapes
                .get(cs.char_shape_id as usize)
        })
        .map(|cs| cs.base_size as f64)
        .unwrap_or(1000.0);

    let em = hwpunit_to_px(size_hwpunit as i32, rhwp::renderer::DEFAULT_DPI);
    text.chars()
        .map(|c| if c.is_ascii() { em * 0.5 } else { em })
        .sum()
}

/// [#3603] `set-cell` 계열이 셀 값으로 거부하는 제어문자 안내문.
///
/// CLI(`edit set-cell`)와 세션 도구(`hwp_doc_set_cell`)가 **같은 문장**으로 거부해야 한다 —
/// 두 경로가 서로 다른 문장(또는 한쪽만 검사)을 내면 에이전트는 같은 제약을 두 번 배워야
/// 하고, 무엇보다 세션 경로만 통과시키면 한 셀 문단 안에 raw 개행이 박힌 문서가 만들어진다.
/// v1 셀 기록 계약은 '한 줄 값'이다.
const SET_CELL_CONTROL_CHAR_MESSAGE: &str =
    "오류: --text 에 줄바꿈·탭은 넣을 수 없습니다 (한 줄 값 기록).";

/// 셀 값에 제어문자가 있으면 공통 안내문을 돌려준다 (없으면 `None`).
///
/// 문장뿐 아니라 **판정식까지** 공유해야 '문장은 같은데 거부 조건이 다른' 어긋남이 안 생긴다.
fn set_cell_control_char_rejection(text: &str) -> Option<&'static str> {
    text.chars()
        .any(|ch| matches!(ch, '\r' | '\n' | '\t'))
        .then_some(SET_CELL_CONTROL_CHAR_MESSAGE)
}

/// [#3603] 격자 주소(export-tables 좌표) → 모델 좌표 해석.
/// CLI(edit set-cell)와 세션 도구(hwp_doc_set_cell)가 공유한다 — 병합으로 덮인 칸은
/// 앵커 좌표를 안내하며 실패한다(보호 동작). 반환: (sec, para, ctrl, cell_idx,
/// 문단별 글자 수, 기존 텍스트).
enum CellResolveError {
    Usage(String),
    Runtime(String),
}

/// 본문 최상위 표 번호 → (section, paragraph, control).
fn resolve_top_table(
    document: &rhwp::model::document::Document,
    table_no: usize,
) -> Result<(usize, usize, usize), String> {
    use rhwp::document_core::queries::table_extract::extract_tables;
    let grids = extract_tables(document);
    match grids
        .iter()
        .find(|g| g.index == table_no && g.container_path.is_empty())
    {
        Some(g) => Ok((g.section, g.paragraph, g.control)),
        None => {
            let n = grids.iter().filter(|g| g.container_path.is_empty()).count();
            Err(format!(
                "오류: 본문 최상위 표 {table_no} 번이 없습니다 (최상위 표 {n}개)."
            ))
        }
    }
}

#[allow(clippy::type_complexity)]
fn resolve_table_cell(
    document: &rhwp::model::document::Document,
    table_no: usize,
    row: u16,
    col: u16,
) -> Result<(usize, usize, usize, usize, Vec<usize>, String), CellResolveError> {
    use rhwp::document_core::queries::table_extract::extract_tables;
    use rhwp::model::control::Control;
    let grids = extract_tables(document);
    let Some(grid) = grids
        .iter()
        .find(|g| g.index == table_no && g.container_path.is_empty())
    else {
        let top_level = grids.iter().filter(|g| g.container_path.is_empty()).count();
        return Err(CellResolveError::Runtime(format!(
            "오류: 본문 최상위 표 {} 번이 없습니다 (최상위 표 {}개; 중첩 표는 v1 범위 밖).",
            table_no, top_level
        )));
    };
    let Some(Control::Table(table)) = document.sections[grid.section].paragraphs[grid.paragraph]
        .controls
        .get(grid.control)
    else {
        return Err(CellResolveError::Runtime(
            "오류: 표 컨트롤 좌표 해석 실패 (내부 불일치).".into(),
        ));
    };
    if row >= table.row_count || col >= table.col_count {
        return Err(CellResolveError::Usage(format!(
            "오류: 좌표가 격자를 벗어났습니다 — 표 {} 는 {}x{} 입니다.",
            table_no, table.row_count, table.col_count
        )));
    }
    match table
        .cells
        .iter()
        .enumerate()
        .find(|(_, c)| c.row == row && c.col == col)
    {
        Some((cell_idx, c)) => {
            let para_lens: Vec<usize> = c
                .paragraphs
                .iter()
                .map(|p| p.text.chars().count())
                .collect();
            let old_text = c
                .paragraphs
                .iter()
                .map(|p| p.text.as_str())
                .collect::<Vec<_>>()
                .join(
                    "
",
                )
                .trim()
                .to_string();
            Ok((
                grid.section,
                grid.paragraph,
                grid.control,
                cell_idx,
                para_lens,
                old_text,
            ))
        }
        None => {
            let anchor = table.cells.iter().find(|c| {
                c.row <= row && row < c.row + c.row_span && c.col <= col && col < c.col + c.col_span
            });
            Err(CellResolveError::Usage(match anchor {
                Some(a) => format!(
                    "오류: ({},{}) 는 병합으로 덮인 칸입니다 — 앵커 ({},{}) 를 지정하세요.",
                    row, col, a.row, a.col
                ),
                None => format!("오류: ({},{}) 위치에 셀이 없습니다.", row, col),
            }))
        }
    }
}

fn edit_set_cell(args: &[String]) -> i32 {
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut text_arg: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    // [#3702] 저장 직후 자기검증 — 판정은 데이터, 차이 시 exit 3.
    let mut verify_mode = false;
    // [#3391] 실물 공고 양식의 기입 칸 안내문은 파란 이탤릭이 흔하다. set-cell 은
    // "안내문을 지우고 실값을 쓰는" 용도이므로 제출 요건(검정 글씨)에 맞춰 기본을
    // 검정·비이탤릭·비진하게로 기록한다. --keep-style 로 셀 스타일 상속을 유지한다.
    let mut keep_style = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--keep-style" => keep_style = true,
            "--table" | "--row" | "--col" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {} 뒤에 0 이상의 정수가 필요합니다.", name);
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(value) => table_arg = Some(value),
                        Err(_) => {
                            eprintln!("오류: {} 뒤에 0 이상의 정수가 필요합니다.", name);
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(value) => row_arg = Some(value),
                        Err(_) => {
                            eprintln!("오류: {} 뒤에 0 이상 65535 이하의 정수가 필요합니다.", name);
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(value) => col_arg = Some(value),
                        Err(_) => {
                            eprintln!("오류: {} 뒤에 0 이상 65535 이하의 정수가 필요합니다.", name);
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--text" => {
                i += 1;
                match args.get(i) {
                    Some(v) => text_arg = Some(v),
                    None => {
                        eprintln!(
                            "오류: --text 뒤에 셀에 넣을 문자열이 필요합니다 (비우기는 \"\")."
                        );
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(new_text)) =
        (file_path, table_arg, row_arg, col_arg, text_arg)
    else {
        eprintln!(
            "사용법: rhwp edit set-cell <파일> --table <번호> --row <행> --col <열> --text <문자열> [-o <출력>] [--keep-style] [--dry-run] [--json]"
        );
        return EXIT_USAGE;
    };
    // 판정과 문장 모두 세션 도구(hwp_doc_set_cell)와 공유한다 — 문서를 읽기 전에 끊는다.
    if let Some(message) = set_cell_control_char_rejection(new_text) {
        eprintln!("{message}");
        return EXIT_USAGE;
    }

    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    // 격자 주소(export-tables 좌표) → 모델 좌표. 병합으로 덮인 칸은 앵커가 아니므로
    // 모델 셀 순회로 (row,col) 앵커를 직접 찾는다 (격자 배열 위치는 손상 방어 필터
    // 때문에 모델 인덱스와 어긋날 수 있어 쓰지 않는다).
    let (sec, para, ctrl, cell_idx, para_lens, old_text) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };

    // [#3480] 값이 그 칸에 들어가는지 재고 넘치면 알린다.
    // 에이전트는 렌더 결과를 보지 않으므로, 신호가 없으면 표 경계를 벗어난 문서를
    // 완성본으로 판단한다. 조판 엔진이 있어야 답할 수 있는 검사다.
    let overflow = measure_cell_overflow(&doc, sec, para, ctrl, cell_idx, &new_text).map(
        |(cell_w, text_w, lines)| {
            serde_json::json!({
                "target": format!("table{}[{},{}]", table_no, row, col),
                "text": new_text,
                "cellWidthPx": (cell_w * 100.0).round() / 100.0,
                "textWidthPx": (text_w * 100.0).round() / 100.0,
                "lines": lines,
            })
        },
    );

    if !dry_run {
        // 셀의 모든 문단 텍스트를 비운다 (다문단 셀 — 빈 문단 골격은 유지된다).
        for (pi, len) in para_lens.iter().enumerate() {
            if *len == 0 {
                continue;
            }
            if let Err(e) = doc.delete_text_in_cell(
                sec as u32,
                para as u32,
                ctrl as u32,
                cell_idx as u32,
                pi as u32,
                0,
                *len as u32,
            ) {
                eprintln!("오류: 셀 비우기 실패(문단 {}) - {:?}", pi, e);
                return EXIT_RUNTIME;
            }
        }
        if !new_text.is_empty() {
            if let Err(e) = doc.insert_text_in_cell(
                sec as u32,
                para as u32,
                ctrl as u32,
                cell_idx as u32,
                0,
                0,
                new_text,
            ) {
                eprintln!("오류: 셀 쓰기 실패 - {:?}", e);
                // 실패 시 원본 불변 — 출력 파일을 쓰지 않고 즉시 끝낸다.
                return EXIT_RUNTIME;
            }
            // [#3391] 기본은 제출 요건(검정 글씨)에 맞춘다 — 셀 문단 0 의 글자모양을
            // 검정·비이탤릭·비진하게 글자모양 하나로 덮는다. --keep-style 이면 생략.
            if !keep_style
                && !recolor_cell_text_black(doc.document_mut(), sec, para, ctrl, cell_idx)
            {
                eprintln!("경고: 셀 글자색을 검정으로 바꾸지 못했습니다 (상속 스타일 유지).");
            }
        }
    }

    // [#3383] 입력 형식을 보존한다 — 기본 확장자도 산출 형식을 따른다.
    let out_format = edit_output_format(&bytes, out_path.as_deref());
    let output_path = out_path.unwrap_or_else(|| {
        let stem = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        format!("{}_cell.{}", stem, out_format.ext())
    });

    let mut verify_report = serde_json::Value::Null;
    let mut verify_failed = false;
    if !dry_run {
        let out_bytes = match edit_serialize(&mut doc, out_format) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "오류: {} 직렬화 실패 - {}",
                    out_format.label().to_uppercase(),
                    e
                );
                return EXIT_RUNTIME;
            }
        };
        if let Err(e) = fs::write(&output_path, &out_bytes) {
            eprintln!("오류: 출력 쓰기 실패 - {}: {}", output_path, e);
            return EXIT_RUNTIME;
        }
        // [#3702] 저장 직후 자기검증 — 편집 후 IR ↔ 저장본 재파싱 IR.
        if verify_mode {
            let cross = out_format == EditOutputFormat::Hwp
                && rhwp::parser::detect_format(&bytes) == rhwp::parser::FileFormat::Hwpx;
            let (report, failed) = edit_verify_report(&doc, &out_bytes, cross);
            verify_report = report;
            verify_failed = failed;
        }
    }

    // [#3712] 눈검증 대상 페이지 — 표 호스트 문단이 걸친 쪽 전부(분할 표 포함).
    let changed_pages = if dry_run {
        serde_json::Value::Null
    } else {
        match doc.pages_covering_paragraphs(&[(sec, para)]) {
            Some(pages) => serde_json::json!(pages),
            None => serde_json::Value::Null,
        }
    };

    if json_mode {
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "table": table_no,
            "row": row,
            "col": col,
            "oldText": old_text,
            "newText": new_text,
            "dryRun": dry_run,
            "changedPages": changed_pages,
            "keepStyle": keep_style,
            "overflow": overflow.clone().map(|o| vec![o]).unwrap_or_default(),
        });
        if !dry_run {
            envelope["output"] = serde_json::Value::String(output_path.clone());
            envelope["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
            envelope["verify"] = verify_report.clone();
        }
        println!("{}", provenance::marked(envelope, "edit"));
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }

    if dry_run {
        println!(
            "변경 예정: {} 표{} ({},{}) {:?} → {:?}",
            file_path, table_no, row, col, old_text, new_text
        );
    } else {
        println!(
            "셀 기록 완료: {} → {} — 표{} ({},{}) {:?} → {:?}",
            file_path, output_path, table_no, row, col, old_text, new_text
        );
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    EXIT_OK
}

/// [#3719 §6-5] `edit insert-image` 가 받는 그림 형식.
///
/// BinData 로 넣을 수 있고 **원본 픽셀 크기를 헤더만 읽어 잴 수 있는** 형식만 담는다.
/// 크기를 못 재면 배율·배치 좌표가 의미를 잃으므로 삽입을 시작하지 않는다.
const INSERT_IMAGE_FORMATS: [&str; 6] = ["png", "jpg", "jpeg", "bmp", "tif", "tiff"];

/// 96dpi 픽셀 1개 = 75 HWPUNIT(7200/96). 코어가 crop 을 `px * 75` 로 잡는 것과 같은 환산비다.
const HWPUNIT_PER_PX: u32 = 75;

/// 그림의 원본 픽셀 크기 — 전체 디코드 없이 헤더만 읽는다.
///
/// 확장자는 거짓말할 수 있으므로 매직 바이트로 형식을 다시 판정한다. 알아보지 못하면
/// `None` — 호출부가 인자 오류(exit 2)로 끊는다.
fn insert_image_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    use image::ImageFormat;

    let format = image::guess_format(bytes).ok()?;
    if !matches!(
        format,
        ImageFormat::Png | ImageFormat::Jpeg | ImageFormat::Bmp | ImageFormat::Tiff
    ) {
        return None;
    }
    let (width, height) = image::ImageReader::with_format(std::io::Cursor::new(bytes), format)
        .into_dimensions()
        .ok()?;
    if width == 0 || height == 0 {
        return None;
    }
    Some((width, height))
}

/// `--page` 가 가리키는 쪽의 **앵커 문단**(구역 인덱스, 문단 인덱스).
///
/// 용지 기준(Paper-relative) floating 그림은 앵커 문단이 놓인 쪽에 그려진다. 그래서
/// "몇 쪽" 을 "어느 문단" 으로 옮겨야 하는데, 그 환산은 이미 조판 결과가 알고 있다 —
/// 기존 진단 질의 `dump_page_items_json` 을 그대로 읽어 그 쪽의 첫 본문 항목을 고른다
/// (새 조판 로직 0). 미주(`isEndnote`)는 구역 뒤에 합성된 문단이라 앵커로 쓰지 않는다.
fn insert_image_page_anchor(
    doc: &rhwp::wasm_api::HwpDocument,
    page: u32,
) -> Option<(usize, usize)> {
    let empty: Vec<serde_json::Value> = Vec::new();
    let pages = doc.dump_page_items_json(Some(page));
    let page_json = pages.as_array()?.first()?;
    let section = page_json["section"].as_u64()? as usize;

    for column in page_json["columns"].as_array().unwrap_or(&empty) {
        for item in column["items"].as_array().unwrap_or(&empty) {
            if item["isEndnote"] == true {
                continue;
            }
            if let Some(para) = item["paraIndex"].as_u64() {
                return Some((section, para as usize));
            }
        }
    }
    // 항목이 하나도 없는 쪽(어울림 문단·감춘 빈 줄만 귀속된 쪽)은 extras 로 온다.
    for extra in page_json["extras"].as_array().unwrap_or(&empty) {
        if let Some(para) = extra["paraIndex"].as_u64() {
            return Some((section, para as usize));
        }
    }
    None
}

/// `edit set-equation-properties` — 본문 수식 속성을 바꾼다.
fn edit_set_equation_properties(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-equation-properties <파일> --section N --para N --ctrl N --props <JSON> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path = None;
    let mut section = None;
    let mut para = None;
    let mut ctrl = None;
    let mut props = None;
    let mut out_path = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let flag = args[i].clone();
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                let Ok(value) = value.parse::<usize>() else {
                    eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다: {value}");
                    return EXIT_USAGE;
                };
                match flag.as_str() {
                    "--section" => section = Some(value),
                    "--para" => para = Some(value),
                    _ => ctrl = Some(value),
                }
            }
            "--props" => {
                i += 1;
                props = args
                    .get(i)
                    .map(String::as_str)
                    .filter(|value| !value.is_empty());
                if props.is_none() {
                    eprintln!("오류: --props 뒤에 수식 속성 JSON 이 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "-o" | "--output" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                    return EXIT_USAGE;
                };
                out_path = Some(value.clone());
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            option if option.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {option}");
                return EXIT_USAGE;
            }
            path => {
                if file_path.replace(path).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {path}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl), Some(props)) =
        (file_path, section, para, ctrl, props)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if serde_json::from_str::<serde_json::Value>(props).is_err() {
        eprintln!("오류: --props 는 JSON 객체여야 합니다: {props}");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, error);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(doc) => doc,
        Err(error) => return error.report(),
    };
    if !dry_run {
        if let Err(error) =
            doc.set_equation_properties_native(section, para, ctrl, None, None, props)
        {
            eprintln!("오류: 수식 속성 설정 실패 - {error}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "eqprop",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl, "text": props }),
        &[(section, para)],
        &format!("수식 속성 설정 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("수식 속성 설정 완료: {file_path}"),
    )
}

/// `edit insert-image` — 도장·서명 같은 그림을 쪽 좌표에 붙인다 (#3719 §6-5).
/// `edit insert-text-in-cell` — 표 셀 문단에 텍스트 삽입. 코어 `insert_text_in_cell_native`.
fn edit_insert_text_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-text-in-cell <파일> --table <번호> --row <행> --col <열> --text <문자열> [--offset N] [--cell-para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut text_arg: Option<&str> = None;
    let mut offset_arg: usize = 0;
    let mut cell_para_arg: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--offset" | "--cell-para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" | "--offset" | "--cell-para" => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--table" => table_arg = Some(n),
                            "--offset" => offset_arg = n,
                            _ => cell_para_arg = n,
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--text" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => text_arg = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --text 뒤에 넣을 문자열이 필요합니다 (빈 문자열 불가).");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(text)) =
        (file_path, table_arg, row_arg, col_arg, text_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl, cell_idx, para_lens, _old) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };
    if cell_para_arg >= para_lens.len() {
        eprintln!(
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 0~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if offset_arg > para_lens[cell_para_arg] {
        eprintln!(
            "오류: --offset 이 문단 길이를 넘습니다 (문단 길이 {}): {offset_arg}",
            para_lens[cell_para_arg]
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.insert_text_in_cell_native(
            sec,
            para,
            ctrl,
            cell_idx,
            cell_para_arg,
            offset_arg,
            text,
        ) {
            eprintln!("오류: 셀 텍스트 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellins",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "cellPara": cell_para_arg,
            "offset": offset_arg,
            "text": text
        }),
        &[(sec, para)],
        &format!(
            "셀 텍스트 삽입 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg} 오프셋 {offset_arg}"
        ),
        &format!("셀 텍스트 삽입 완료: {file_path}"),
    )
}

/// `edit delete-text-in-cell` — 표 셀 문단 텍스트 삭제. 코어 `delete_text_in_cell_native`.
fn edit_delete_text_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-text-in-cell <파일> --table <번호> --row <행> --col <열> --count <글자수> [--offset N] [--cell-para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut count_arg: Option<usize> = None;
    let mut offset_arg: usize = 0;
    let mut cell_para_arg: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--offset" | "--cell-para" | "--count" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" | "--offset" | "--cell-para" | "--count" => {
                        match v.parse::<usize>() {
                            Ok(n) => match name.as_str() {
                                "--table" => table_arg = Some(n),
                                "--offset" => offset_arg = n,
                                "--count" => {
                                    if n == 0 {
                                        eprintln!("오류: --count 는 1 이상이어야 합니다.");
                                        return EXIT_USAGE;
                                    }
                                    count_arg = Some(n);
                                }
                                _ => cell_para_arg = n,
                            },
                            Err(_) => {
                                eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                                return EXIT_USAGE;
                            }
                        }
                    }
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(count)) =
        (file_path, table_arg, row_arg, col_arg, count_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl, cell_idx, para_lens, _old) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };
    if cell_para_arg >= para_lens.len() {
        eprintln!(
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 0~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if offset_arg > para_lens[cell_para_arg] {
        eprintln!(
            "오류: --offset 이 문단 길이를 넘습니다 (문단 길이 {}): {offset_arg}",
            para_lens[cell_para_arg]
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.delete_text_in_cell_native(
            sec,
            para,
            ctrl,
            cell_idx,
            cell_para_arg,
            offset_arg,
            count,
        ) {
            eprintln!("오류: 셀 텍스트 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "celldel",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "cellPara": cell_para_arg,
            "offset": offset_arg,
            "count": count
        }),
        &[(sec, para)],
        &format!(
            "셀 텍스트 삭제 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg} 오프셋 {offset_arg} 글자 {count}"
        ),
        &format!("셀 텍스트 삭제 완료: {file_path}"),
    )
}

/// [#4990 / #3608 M9] `edit insert-text` — 문단 좌표에 새 텍스트를 삽입한다.
///
/// 에이전트는 기존 문자열을 바꿀 수 있었지만(replace-text·fill-fields·set-cell)
/// **없는 자리에 글자를 넣는** 표면이 없었다. 새 편집 로직은 없다 —
/// 검증된 코어 `insert_text_native`(스튜디오·세션이 이미 쓰는 경로)만 배선한다.
/// 주소 어휘는 `search` 와 같다(구역·문단·문자 오프셋, 전부 0 기준).
fn edit_insert_text(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-text <파일> --text <문자열> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";

    let mut file_path: Option<&str> = None;
    let mut text_arg: Option<&str> = None;
    let mut section_arg: u32 = 0;
    let mut para_arg: u32 = 0;
    let mut offset_arg: u32 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--text" => {
                i += 1;
                match args.get(i) {
                    Some(v) => text_arg = Some(v),
                    None => {
                        eprintln!("오류: --text 뒤에 넣을 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터).");
                    return EXIT_USAGE;
                };
                let Ok(value) = v.parse::<u32>() else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터): {v}");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => section_arg = value,
                    "--para" => para_arg = value,
                    _ => offset_arg = value,
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let (Some(file_path), Some(text)) = (file_path, text_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if text.is_empty() {
        eprintln!("오류: --text 는 빈 문자열일 수 없습니다.");
        return EXIT_USAGE;
    }

    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    let sec = section_arg as usize;
    let para = para_arg as usize;
    let offset = offset_arg as usize;
    let section_count = doc.document().sections.len();
    if sec >= section_count {
        eprintln!(
            "오류: --section 이 범위를 벗어났습니다 (0~{}): {section_arg}",
            section_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_count = doc.document().sections[sec].paragraphs.len();
    if para >= para_count {
        eprintln!(
            "오류: --para 이 범위를 벗어났습니다 (구역 {section_arg} 문단 0~{}): {para_arg}",
            para_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_chars = doc.document().sections[sec].paragraphs[para]
        .text
        .chars()
        .count();
    if offset > para_chars {
        eprintln!("오류: --offset 이 문단 길이를 넘습니다 (문단 길이 {para_chars}): {offset_arg}");
        return EXIT_USAGE;
    }

    if !dry_run {
        if let Err(e) = doc.insert_text_native(sec, para, offset, text) {
            eprintln!("오류: 텍스트 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }

    let out_format = edit_output_format(&bytes, out_path.as_deref());
    let output_path = out_path.unwrap_or_else(|| {
        let stem = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        format!("{}_inserted.{}", stem, out_format.ext())
    });

    let mut verify_report = serde_json::Value::Null;
    let mut verify_failed = false;
    if !dry_run {
        let out_bytes = match edit_serialize(&mut doc, out_format) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "오류: {} 직렬화 실패 - {}",
                    out_format.label().to_uppercase(),
                    e
                );
                return EXIT_RUNTIME;
            }
        };
        if let Err(e) = fs::write(&output_path, &out_bytes) {
            eprintln!("오류: 출력 쓰기 실패 - {}: {}", output_path, e);
            return EXIT_RUNTIME;
        }
        if verify_mode {
            let cross = out_format == EditOutputFormat::Hwp
                && rhwp::parser::detect_format(&bytes) == rhwp::parser::FileFormat::Hwpx;
            let (report, failed) = edit_verify_report(&doc, &out_bytes, cross);
            verify_report = report;
            verify_failed = failed;
        }
    }

    let changed_pages = if dry_run {
        serde_json::Value::Null
    } else {
        match doc.pages_covering_paragraphs(&[(sec, para)]) {
            Some(pages) => serde_json::json!(pages),
            None => serde_json::Value::Null,
        }
    };
    let inserted_chars = text.chars().count();

    if json_mode {
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "section": section_arg,
            "paragraph": para_arg,
            "offset": offset_arg,
            "text": text,
            "insertedChars": inserted_chars,
            "dryRun": dry_run,
            "changedPages": changed_pages,
        });
        if !dry_run {
            envelope["output"] = serde_json::Value::String(output_path.clone());
            envelope["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
            envelope["verify"] = verify_report.clone();
        }
        // 삽입 문자열은 호출자 인자이지 문서 유래가 아니다 — 표지는 항상 싣되
        // untrustedFields 는 비운다 (키 부재 = 판정 안 함).
        println!("{}", provenance::marked(envelope, "edit"));
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }

    if dry_run {
        println!(
            "삽입 예정: {} 구역 {section_arg} 문단 {para_arg} 오프셋 {offset_arg} ← {inserted_chars}자",
            file_path
        );
    } else {
        println!(
            "텍스트 삽입 완료: {} → {} — 구역 {section_arg} 문단 {para_arg} 오프셋 {offset_arg} ← {inserted_chars}자",
            file_path, output_path
        );
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    EXIT_OK
}

/// [#4992] `edit insert-paragraph` — 지정 자리에 빈 문단을 끼운다.
fn edit_insert_paragraph(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-paragraph <파일> [--section N] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section_arg: u32 = 0;
    let mut para_arg: u32 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터).");
                    return EXIT_USAGE;
                };
                let Ok(value) = v.parse::<u32>() else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터): {v}");
                    return EXIT_USAGE;
                };
                if name == "--section" {
                    section_arg = value;
                } else {
                    para_arg = value;
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let sec = section_arg as usize;
    let para = para_arg as usize;
    let section_count = doc.document().sections.len();
    if sec >= section_count {
        eprintln!(
            "오류: --section 이 범위를 벗어났습니다 (0~{}): {section_arg}",
            section_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_count = doc.document().sections[sec].paragraphs.len();
    if para > para_count {
        eprintln!(
            "오류: --para 이 범위를 벗어났습니다 (구역 {section_arg} 문단 0~{para_count}): {para_arg}"
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.insert_paragraph_native(sec, para) {
            eprintln!("오류: 문단 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "paragraph",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section_arg, "paragraph": para_arg }),
        &[(sec, para)],
        &format!("문단 삽입 예정: {file_path} 구역 {section_arg} 문단 {para_arg}"),
        &format!("문단 삽입 완료: {file_path}"),
    )
}

/// [#4993] `edit insert-page-break` — 쪽 나눔 삽입.
fn edit_insert_page_break(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-page-break <파일> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section_arg: u32 = 0;
    let mut para_arg: u32 = 0;
    let mut offset_arg: u32 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터).");
                    return EXIT_USAGE;
                };
                let Ok(value) = v.parse::<u32>() else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터): {v}");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => section_arg = value,
                    "--para" => para_arg = value,
                    _ => offset_arg = value,
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let sec = section_arg as usize;
    let para = para_arg as usize;
    let offset = offset_arg as usize;
    let section_count = doc.document().sections.len();
    if sec >= section_count {
        eprintln!(
            "오류: --section 이 범위를 벗어났습니다 (0~{}): {section_arg}",
            section_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_count = doc.document().sections[sec].paragraphs.len();
    if para >= para_count {
        eprintln!(
            "오류: --para 이 범위를 벗어났습니다 (구역 {section_arg} 문단 0~{}): {para_arg}",
            para_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.insert_page_break_native(sec, para, offset) {
            eprintln!("오류: 쪽 나눔 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "pagebreak",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section_arg,
            "paragraph": para_arg,
            "offset": offset_arg
        }),
        &[(sec, para)],
        &format!(
            "쪽 나눔 예정: {file_path} 구역 {section_arg} 문단 {para_arg} 오프셋 {offset_arg}"
        ),
        &format!("쪽 나눔 삽입 완료: {file_path}"),
    )
}

/// [#5019] `edit insert-column-break` — 단 나눔 삽입.
fn edit_insert_column_break(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-column-break <파일> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section_arg: u32 = 0;
    let mut para_arg: u32 = 0;
    let mut offset_arg: u32 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터).");
                    return EXIT_USAGE;
                };
                let Ok(value) = v.parse::<u32>() else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다 (0부터): {v}");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => section_arg = value,
                    "--para" => para_arg = value,
                    _ => offset_arg = value,
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let sec = section_arg as usize;
    let para = para_arg as usize;
    let offset = offset_arg as usize;
    let section_count = doc.document().sections.len();
    if sec >= section_count {
        eprintln!(
            "오류: --section 이 범위를 벗어났습니다 (0~{}): {section_arg}",
            section_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_count = doc.document().sections[sec].paragraphs.len();
    if para >= para_count {
        eprintln!(
            "오류: --para 이 범위를 벗어났습니다 (구역 {section_arg} 문단 0~{}): {para_arg}",
            para_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.insert_column_break_native(sec, para, offset) {
            eprintln!("오류: 단 나눔 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "colbreak",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section_arg,
            "paragraph": para_arg,
            "offset": offset_arg
        }),
        &[(sec, para)],
        &format!(
            "단 나눔 예정: {file_path} 구역 {section_arg} 문단 {para_arg} 오프셋 {offset_arg}"
        ),
        &format!("단 나눔 삽입 완료: {file_path}"),
    )
}

/// `edit insert-table` — 본문 표 생성. 코어 `create_table_native` 배선.
fn edit_insert_table(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-table <파일> --rows N --cols N [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut rows_arg: Option<u16> = None;
    let mut cols_arg: Option<u16> = None;
    let mut section_arg: usize = 0;
    let mut para_arg: usize = 0;
    let mut offset_arg: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--rows" | "--cols" | "--section" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--rows" | "--cols" => match v.parse::<u16>() {
                        Ok(n) if n >= 1 => {
                            if name == "--rows" {
                                rows_arg = Some(n);
                            } else if n > 256 {
                                eprintln!("오류: --cols 는 1~256 이어야 합니다: {v}");
                                return EXIT_USAGE;
                            } else {
                                cols_arg = Some(n);
                            }
                        }
                        _ => {
                            eprintln!("오류: {name} 뒤에 1 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--section" => section_arg = n,
                            "--para" => para_arg = n,
                            _ => offset_arg = n,
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(rows), Some(cols)) = (file_path, rows_arg, cols_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.create_table_native(section_arg, para_arg, offset_arg, rows, cols) {
            eprintln!("오류: 표 생성 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "table",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section_arg,
            "paragraph": para_arg,
            "offset": offset_arg,
            "rows": rows,
            "cols": cols
        }),
        &[(section_arg, para_arg)],
        &format!("표 생성 예정: {file_path} {rows}x{cols} 구역 {section_arg} 문단 {para_arg} 오프셋 {offset_arg}"),
        &format!("표 생성 완료: {file_path}"),
    )
}

/// [#5120] `edit set-numbering-restart` — 문단 번호 다시 시작.
fn edit_set_numbering_restart(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-numbering-restart <파일> --mode N [--count N] [--section N] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut mode_arg: Option<u8> = None;
    let mut start_num: u32 = 1;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--mode" | "--count" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--mode" => match v.parse::<u8>() {
                        Ok(n) => mode_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --mode 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u32>() {
                        Ok(n) => start_num = n,
                        Err(_) => {
                            eprintln!("오류: --count 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(mode)) = (file_path, mode_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.set_numbering_restart_native(section, para, mode, start_num) {
            eprintln!("오류: 번호 다시 시작 설정 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "numrst",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "count": start_num }),
        &[(section, para)],
        &format!("번호 다시 시작 예정: {file_path} 구역 {section} 문단 {para} mode {mode}"),
        &format!("번호 다시 시작 설정 완료: {file_path}"),
    )
}

/// [#4994] `edit insert-row` — 표 행 삽입.
fn edit_insert_row(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-row <파일> --table <번호> --row <행> [--below] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut below = false;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(n) => table_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--below" => below = true,
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row)) = (file_path, table_arg, row_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.insert_table_row_native(sec, para, ctrl, row, below) {
            eprintln!("오류: 행 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "row",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no, "row": row, "below": below }),
        &[(sec, para)],
        &format!("행 삽입 예정: {file_path} 표 {table_no} 행 {row} below={below}"),
        &format!("행 삽입 완료: {file_path}"),
    )
}

/// [#4995] `edit insert-col` — 표 열 삽입.
fn edit_insert_col(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-col <파일> --table <번호> --col <열> [--right] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut col_arg: Option<u16> = None;
    let mut right = false;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--col" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(n) => table_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--right" => right = true,
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(col)) = (file_path, table_arg, col_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.insert_table_column_native(sec, para, ctrl, col, right) {
            eprintln!("오류: 열 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "col",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no, "col": col, "right": right }),
        &[(sec, para)],
        &format!("열 삽입 예정: {file_path} 표 {table_no} 열 {col} right={right}"),
        &format!("열 삽입 완료: {file_path}"),
    )
}

/// [#4996] `edit delete-row` — 표 행 삭제.
fn edit_delete_row(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-row <파일> --table <번호> --row <행> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(n) => table_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row)) = (file_path, table_arg, row_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.delete_table_row_native(sec, para, ctrl, row) {
            eprintln!("오류: 행 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delrow",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no, "row": row }),
        &[(sec, para)],
        &format!("행 삭제 예정: {file_path} 표 {table_no} 행 {row}"),
        &format!("행 삭제 완료: {file_path}"),
    )
}

/// [#4997] `edit merge-cells` — 표 셀 병합.
fn edit_merge_cells(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit merge-cells <파일> --table <번호> --row <행> --col <열> --end-row <행> --end-col <열> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut end_row_arg: Option<u16> = None;
    let mut end_col_arg: Option<u16> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--end-row" | "--end-col" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(n) => table_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--col" => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--end-row" => match v.parse::<u16>() {
                        Ok(n) => end_row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --end-row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => end_col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --end-col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(end_row), Some(end_col)) = (
        file_path,
        table_arg,
        row_arg,
        col_arg,
        end_row_arg,
        end_col_arg,
    ) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.merge_table_cells_native(sec, para, ctrl, row, col, end_row, end_col) {
            eprintln!("오류: 셀 병합 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "merge",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "endRow": end_row,
            "endCol": end_col
        }),
        &[(sec, para)],
        &format!("셀 병합 예정: {file_path} 표 {table_no} ({row},{col})-({end_row},{end_col})"),
        &format!("셀 병합 완료: {file_path}"),
    )
}

/// [#4998] `edit insert-footnote` — 각주 삽입.
fn edit_insert_footnote(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-footnote <파일> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        _ => offset = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.insert_footnote_native(section, para, offset) {
            eprintln!("오류: 각주 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "footnote",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "offset": offset }),
        &[(section, para)],
        &format!("각주 삽입 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset}"),
        &format!("각주 삽입 완료: {file_path}"),
    )
}

/// [#5009] `edit delete-col` — 표 열 삭제.
/// `edit insert-equation` — 본문 수식 삽입. 코어 `insert_equation_native`.
fn edit_insert_equation(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-equation <파일> --script <수식> [--section N] [--para N] [--offset N] [--font-size N] [--color N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut script_arg: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut font_size: u32 = 1000;
    let mut color: u32 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" | "--font-size" | "--color" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--font-size" | "--color" => match v.parse::<u32>() {
                        Ok(n) => {
                            if name == "--font-size" {
                                if n == 0 {
                                    eprintln!("오류: --font-size 는 1 이상이어야 합니다.");
                                    return EXIT_USAGE;
                                }
                                font_size = n;
                            } else {
                                color = n;
                            }
                        }
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--section" => section = n,
                            "--para" => para = n,
                            _ => offset = n,
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--script" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => script_arg = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --script 뒤에 수식 문자열이 필요합니다 (빈 문자열 불가).");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(script)) = (file_path, script_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.insert_equation_native(section, para, offset, script, font_size, color)
        {
            eprintln!("오류: 수식 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "eq",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "offset": offset,
            "script": script,
            "fontSize": font_size,
            "color": color
        }),
        &[(section, para)],
        &format!("수식 삽입 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset}"),
        &format!("수식 삽입 완료: {file_path}"),
    )
}

/// [#5009] `edit delete-col` — 표 열 삭제.
fn edit_delete_col(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-col <파일> --table <번호> --col <열> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut col_arg: Option<u16> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--col" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(n) => table_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(col)) = (file_path, table_arg, col_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.delete_table_column_native(sec, para, ctrl, col) {
            eprintln!("오류: 열 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delcol",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no, "col": col }),
        &[(sec, para)],
        &format!("열 삭제 예정: {file_path} 표 {table_no} 열 {col}"),
        &format!("열 삭제 완료: {file_path}"),
    )
}

/// [#5010] `edit split-cell` — 병합 셀 분할.
fn edit_split_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit split-cell <파일> --table <번호> --row <행> --col <열> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(n) => table_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row), Some(col)) =
        (file_path, table_arg, row_arg, col_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.split_table_cell_native(sec, para, ctrl, row, col) {
            eprintln!("오류: 셀 분할 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "split",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no, "row": row, "col": col }),
        &[(sec, para)],
        &format!("셀 분할 예정: {file_path} 표 {table_no} ({row},{col})"),
        &format!("셀 분할 완료: {file_path}"),
    )
}

/// [#5120] `edit split-cell-into` — 셀을 n행 × m열로 나눈다. 코어 `split_table_cell_into_native`.
fn edit_split_cell_into(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit split-cell-into <파일> --table <번호> --row <행> --col <열> --rows <행수> --cols <열수> [--equal-row-height] [--merge-first] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut rows_arg: Option<u16> = None;
    let mut cols_arg: Option<u16> = None;
    let mut equal_row_height = false;
    let mut merge_first = false;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--equal-row-height" => equal_row_height = true,
            "--merge-first" => merge_first = true,
            "--table" | "--row" | "--col" | "--rows" | "--cols" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(n) => table_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--col" => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--rows" => match v.parse::<u16>() {
                        Ok(n) if n >= 1 => rows_arg = Some(n),
                        Ok(_) => {
                            eprintln!("오류: --rows 는 1 이상이어야 합니다.");
                            return EXIT_USAGE;
                        }
                        Err(_) => {
                            eprintln!("오류: --rows 뒤에 1 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) if n >= 1 => cols_arg = Some(n),
                        Ok(_) => {
                            eprintln!("오류: --cols 는 1 이상이어야 합니다.");
                            return EXIT_USAGE;
                        }
                        Err(_) => {
                            eprintln!("오류: --cols 뒤에 1 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(n_rows), Some(m_cols)) =
        (file_path, table_arg, row_arg, col_arg, rows_arg, cols_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.split_table_cell_into_native(
            sec,
            para,
            ctrl,
            row,
            col,
            n_rows,
            m_cols,
            equal_row_height,
            merge_first,
        ) {
            eprintln!("오류: 셀 n×m 분할 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "splitinto",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "rows": n_rows,
            "cols": m_cols
        }),
        &[(sec, para)],
        &format!("셀 n×m 분할 예정: {file_path} 표 {table_no} ({row},{col}) {n_rows}×{m_cols}"),
        &format!("셀 n×m 분할 완료: {file_path}"),
    )
}

/// `edit split-table` — 표를 지정 행에서 둘로 나눈다. 코어 `split_table_native`.
fn edit_split_table(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit split-table <파일> --table <번호> --row <행> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(n) => table_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => {
                            if n == 0 {
                                eprintln!("오류: --row 는 1 이상이어야 합니다 (첫 행에서는 나눌 수 없음).");
                                return EXIT_USAGE;
                            }
                            row_arg = Some(n);
                        }
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row)) = (file_path, table_arg, row_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.split_table_native(sec, para, ctrl, row) {
            eprintln!("오류: 표 나누기 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "tblsplit",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no, "row": row }),
        &[(sec, para)],
        &format!("표 나누기 예정: {file_path} 표 {table_no} 행 {row}"),
        &format!("표 나누기 완료: {file_path}"),
    )
}

/// `edit fit-table` — 표를 페이지 본문 폭에 맞춘다. 코어 `fit_table_to_page_native`.
fn edit_fit_table(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit fit-table <파일> --table <번호> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => table_arg = Some(n),
                    Err(_) => {
                        eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no)) = (file_path, table_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.fit_table_to_page_native(sec, para, ctrl) {
            eprintln!("오류: 표 폭 맞춤 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "fittbl",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no }),
        &[(sec, para)],
        &format!("표 폭 맞춤 예정: {file_path} 표 {table_no}"),
        &format!("표 폭 맞춤 완료: {file_path}"),
    )
}

/// `edit resize-table` — 표 행/열 크기 조절. 코어 `resize_table_native`.
fn edit_resize_table(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit resize-table <파일> --table <번호> --row <행> --col <열> [--vertical] [--forward] [--line] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut vertical = false;
    let mut forward = false;
    let mut line_mode = false;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--vertical" => vertical = true,
            "--forward" => forward = true,
            "--line" => line_mode = true,
            "--table" | "--row" | "--col" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(n) => table_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row), Some(col)) =
        (file_path, table_arg, row_arg, col_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) =
            doc.resize_table_native(sec, para, ctrl, row, col, vertical, forward, line_mode)
        {
            eprintln!("오류: 표 크기 조절 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "tblrsz",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no, "row": row, "col": col }),
        &[(sec, para)],
        &format!("표 크기 조절 예정: {file_path} 표 {table_no} 행 {row} 열 {col}"),
        &format!("표 크기 조절 완료: {file_path}"),
    )
}

/// `edit merge-table` — 다음 표를 이어 붙인다. 코어 `merge_table_with_next_native`.
fn edit_merge_table(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit merge-table <파일> --table <번호> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => table_arg = Some(n),
                    Err(_) => {
                        eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no)) = (file_path, table_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.merge_table_with_next_native(sec, para, ctrl) {
            eprintln!("오류: 표 붙이기 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "mergetbl",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no }),
        &[(sec, para)],
        &format!("표 붙이기 예정: {file_path} 표 {table_no}"),
        &format!("표 붙이기 완료: {file_path}"),
    )
}

/// `edit set-column-widths` — 열 폭 설정. 코어 `set_table_column_widths_native`.
fn edit_set_column_widths(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-column-widths <파일> --table <번호> --widths <W1,W2,...> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut widths_arg: Option<Vec<u32>> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => table_arg = Some(n),
                    Err(_) => {
                        eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--widths" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --widths 뒤에 HWPUNIT 목록(쉼표 구분)이 필요합니다.");
                    return EXIT_USAGE;
                };
                let mut parsed: Vec<u32> = Vec::new();
                for token in v.split(',').map(str::trim).filter(|t| !t.is_empty()) {
                    match token.parse::<u32>() {
                        Ok(n) if n >= 1 => parsed.push(n),
                        Ok(_) => {
                            eprintln!("오류: --widths 각 값은 1 이상이어야 합니다: {token}");
                            return EXIT_USAGE;
                        }
                        Err(_) => {
                            eprintln!("오류: --widths 뒤에 HWPUNIT 정수가 필요합니다: {token}");
                            return EXIT_USAGE;
                        }
                    }
                }
                if parsed.is_empty() {
                    eprintln!("오류: --widths 뒤에 HWPUNIT 목록(쉼표 구분)이 필요합니다.");
                    return EXIT_USAGE;
                }
                widths_arg = Some(parsed);
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(widths)) = (file_path, table_arg, widths_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.set_table_column_widths_native(sec, para, ctrl, widths.clone()) {
            eprintln!("오류: 열 폭 설정 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "colw",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no, "widths": widths }),
        &[(sec, para)],
        &format!("열 폭 설정 예정: {file_path} 표 {table_no}"),
        &format!("열 폭 설정 완료: {file_path}"),
    )
}

/// [#5011] `edit delete-text` — 문단 좌표 텍스트 삭제.
fn edit_delete_text(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-text <파일> --count <글자수> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut count_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" | "--count" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        "--offset" => offset = n,
                        _ => {
                            if n == 0 {
                                eprintln!("오류: --count 는 1 이상이어야 합니다.");
                                return EXIT_USAGE;
                            }
                            count_arg = Some(n);
                        }
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(count)) = (file_path, count_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.delete_text_native(section, para, offset, count) {
            eprintln!("오류: 텍스트 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "deltext",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "offset": offset,
            "count": count
        }),
        &[(section, para)],
        &format!(
            "텍스트 삭제 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset} 글자 {count}"
        ),
        &format!("텍스트 삭제 완료: {file_path}"),
    )
}

/// `edit delete-text-in-footnote` — 각주/미주 문단 텍스트 삭제.
fn edit_delete_text_in_footnote(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-text-in-footnote <파일> --count <글자수> [--section N] [--para N] [--ctrl N] [--fn-para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut ctrl: usize = 0;
    let mut fn_para: usize = 0;
    let mut offset: usize = 0;
    let mut count_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" | "--fn-para" | "--offset" | "--count" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        "--ctrl" => ctrl = n,
                        "--fn-para" => fn_para = n,
                        "--offset" => offset = n,
                        _ => {
                            if n == 0 {
                                eprintln!("오류: --count 는 1 이상이어야 합니다.");
                                return EXIT_USAGE;
                            }
                            count_arg = Some(n);
                        }
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(count)) = (file_path, count_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) =
            doc.delete_text_in_footnote_native(section, para, ctrl, fn_para, offset, count)
        {
            eprintln!("오류: 각주/미주 텍스트 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "fndeltxt",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "ctrl": ctrl,
            "fnPara": fn_para,
            "offset": offset,
            "count": count
        }),
        &[(section, para)],
        &format!(
            "각주/미주 텍스트 삭제 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl} 각주문단 {fn_para} 오프셋 {offset} 글자 {count}"
        ),
        &format!("각주/미주 텍스트 삭제 완료: {file_path}"),
    )
}
fn edit_set_page_def(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-page-def <파일> --props <JSON> [--section N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut props: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => section = n,
                    Err(_) => {
                        eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 용지 설정 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(props)) = (file_path, props) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if serde_json::from_str::<serde_json::Value>(props).is_err() {
        eprintln!("오류: --props 는 JSON 객체여야 합니다: {props}");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.set_page_def_native(section, props) {
            eprintln!("오류: 용지 설정 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "pagedef",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "props": props }),
        &[(section, 0)],
        &format!("용지 설정 예정: {file_path} 구역 {section}"),
        &format!("용지 설정 완료: {file_path}"),
    )
}

/// `edit set-section-def` — 구역 정의. 코어 `set_section_def_native`.
fn edit_set_section_def(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-section-def <파일> --props <JSON> [--section N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut props: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => section = n,
                    Err(_) => {
                        eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 구역 정의 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(props)) = (file_path, props) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if serde_json::from_str::<serde_json::Value>(props).is_err() {
        eprintln!("오류: --props 는 JSON 객체여야 합니다: {props}");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.set_section_def_native(section, props) {
            eprintln!("오류: 구역 정의 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "secdef",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "props": props
        }),
        &[(section, 0)],
        &format!("구역 정의 예정: {file_path} 구역 {section}"),
        &format!("구역 정의 완료: {file_path}"),
    )
}

/// [#5012] `edit delete-paragraph` — 문단 삭제.
fn edit_delete_paragraph(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-paragraph <파일> [--section N] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => {
                        if name == "--section" {
                            section = n;
                        } else {
                            para = n;
                        }
                    }
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.delete_paragraph_native(section, para) {
            eprintln!("오류: 문단 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delpara",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para }),
        &[(section, para.saturating_sub(1))],
        &format!("문단 삭제 예정: {file_path} 구역 {section} 문단 {para}"),
        &format!("문단 삭제 완료: {file_path}"),
    )
}

/// [#5018] `edit merge-paragraph` — 문단 병합.
fn edit_merge_paragraph(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit merge-paragraph <파일> [--section N] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => {
                        if name == "--section" {
                            section = n;
                        } else {
                            para = n;
                        }
                    }
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.merge_paragraph_native(section, para) {
            eprintln!("오류: 문단 병합 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "mergepara",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para }),
        &[(section, para.saturating_sub(1))],
        &format!("문단 병합 예정: {file_path} 구역 {section} 문단 {para}"),
        &format!("문단 병합 완료: {file_path}"),
    )
}

/// [#5013] `edit insert-endnote` — 미주 삽입.
fn edit_insert_endnote(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-endnote <파일> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        _ => offset = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.insert_endnote_native(section, para, offset) {
            eprintln!("오류: 미주 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "endnote",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "offset": offset }),
        &[(section, para)],
        &format!("미주 삽입 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset}"),
        &format!("미주 삽입 완료: {file_path}"),
    )
}

/// [#5017] `edit delete-footnote` — 각주/미주 삭제.
fn edit_delete_footnote(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-footnote <파일> --section N --para N --ctrl N [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl)) = (file_path, section, para, ctrl)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.delete_footnote_native(section, para, ctrl) {
            eprintln!("오류: 각주 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delfn",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("각주 삭제 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("각주 삭제 완료: {file_path}"),
    )
}

/// `edit resize-table-cell` — 한 칸 크기 조절. 코어 `resize_table_cell_native`.
fn edit_resize_table_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit resize-table-cell <파일> --table <번호> --row <행> --col <열> [--vertical] [--forward] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut vertical = false;
    let mut forward = false;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--vertical" => vertical = true,
            "--forward" => forward = true,
            "--table" | "--row" | "--col" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(n) => table_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row), Some(col)) =
        (file_path, table_arg, row_arg, col_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.resize_table_cell_native(sec, para, ctrl, row, col, vertical, forward) {
            eprintln!("오류: 셀 크기 조절 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellrsz",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "vertical": vertical,
            "forward": forward
        }),
        &[(sec, para)],
        &format!("셀 크기 조절 예정: {file_path} 표 {table_no} ({row},{col})"),
        &format!("셀 크기 조절 완료: {file_path}"),
    )
}

/// `edit set-cell-props` — 표 셀 속성. 코어 `set_cell_properties_native`.
fn edit_set_cell_props(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-cell-props <파일> --table <번호> --row <행> --col <열> --props <JSON> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut props: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" => match v.parse::<usize>() {
                        Ok(n) => table_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) => props = Some(v.clone()),
                    None => {
                        eprintln!("오류: --props 뒤에 JSON 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(props)) =
        (file_path, table_arg, row_arg, col_arg, props)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if !matches!(
        serde_json::from_str::<serde_json::Value>(&props),
        Ok(serde_json::Value::Object(_))
    ) {
        eprintln!("오류: --props 는 JSON 객체여야 합니다.");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl, cell_idx, _para_lens, _old) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };
    if !dry_run {
        if let Err(e) = doc.set_cell_properties_native(sec, para, ctrl, cell_idx, &props) {
            eprintln!("오류: 셀 속성 변경 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellprop",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no, "row": row, "col": col }),
        &[(sec, para)],
        &format!("셀 속성 변경 예정: {file_path} 표 {table_no} ({row},{col})"),
        &format!("셀 속성 변경 완료: {file_path}"),
    )
}

/// `edit set-table-props` — 표 속성. 코어 `set_table_properties_native`.
fn edit_set_table_props(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-table-props <파일> --table <번호> --props <JSON> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut props: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => table_arg = Some(n),
                    Err(_) => {
                        eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) => props = Some(v.clone()),
                    None => {
                        eprintln!("오류: --props 뒤에 JSON 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(props)) = (file_path, table_arg, props) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if !matches!(
        serde_json::from_str::<serde_json::Value>(&props),
        Ok(serde_json::Value::Object(_))
    ) {
        eprintln!("오류: --props 는 JSON 객체여야 합니다.");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.set_table_properties_native(sec, para, ctrl, &props) {
            eprintln!("오류: 표 속성 변경 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "tblprop",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no }),
        &[(sec, para)],
        &format!("표 속성 변경 예정: {file_path} 표 {table_no}"),
        &format!("표 속성 변경 완료: {file_path}"),
    )
}

/// `edit move-table` — 표 위치 오프셋 이동. 코어 `move_table_offset_native`.
fn edit_move_table(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit move-table <파일> --table <번호> --dx <가로> --dy <세로> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut dx_arg: Option<i32> = None;
    let mut dy_arg: Option<i32> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => table_arg = Some(n),
                    Err(_) => {
                        eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dx" | "--dy" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<i32>() {
                    Ok(n) if name == "--dx" => dx_arg = Some(n),
                    Ok(n) => dy_arg = Some(n),
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(dx), Some(dy)) =
        (file_path, table_arg, dx_arg, dy_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.move_table_offset_native(sec, para, ctrl, dx, dy) {
            eprintln!("오류: 표 이동 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "movetbl",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no, "dx": dx, "dy": dy }),
        &[(sec, para)],
        &format!("표 이동 예정: {file_path} 표 {table_no} dx={dx} dy={dy}"),
        &format!("표 이동 완료: {file_path}"),
    )
}

/// [#5120] `edit delete-equation` — 본문 수식 삭제.
fn edit_delete_equation(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-equation <파일> --section N --para N --ctrl N [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl)) = (file_path, section, para, ctrl)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.delete_equation_control_native(section, para, ctrl) {
            eprintln!("오류: 수식 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "deleq",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("수식 삭제 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("수식 삭제 완료: {file_path}"),
    )
}

/// [#5026] `edit add-bookmark` — 책갈피 추가.
fn edit_add_bookmark(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit add-bookmark <파일> --name <이름> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut name: Option<String> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--name" => {
                i += 1;
                match args.get(i) {
                    Some(v) => name = Some(v.clone()),
                    None => {
                        eprintln!("오류: --name 뒤에 책갈피 이름이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--para" | "--offset" => {
                let flag = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match flag.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        _ => offset = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(name)) = (file_path, name) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if name.trim().is_empty() {
        eprintln!("오류: --name 은 비어 있을 수 없습니다.");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        match doc.add_bookmark_native(section, para, offset, &name) {
            Ok(raw) => {
                let v: serde_json::Value =
                    serde_json::from_str(&raw).unwrap_or(serde_json::json!({}));
                if v["ok"] == false {
                    let err = v["error"].as_str().unwrap_or("책갈피 추가 실패");
                    eprintln!("오류: {err}");
                    return EXIT_RUNTIME;
                }
            }
            Err(e) => {
                eprintln!("오류: 책갈피 추가 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "bookmark",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "offset": offset,
            "name": name
        }),
        &[(section, para)],
        &format!(
            "책갈피 추가 예정: {file_path} 이름 {name} 구역 {section} 문단 {para} 오프셋 {offset}"
        ),
        &format!("책갈피 추가 완료: {file_path}"),
    )
}

/// [#5027] `edit delete-bookmark` — 책갈피 삭제.
fn edit_delete_bookmark(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-bookmark <파일> --section N --para N --ctrl N [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl)) = (file_path, section, para, ctrl)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        match doc.delete_bookmark_native(section, para, ctrl) {
            Ok(raw) => {
                let v: serde_json::Value =
                    serde_json::from_str(&raw).unwrap_or(serde_json::json!({}));
                if v["ok"] == false {
                    let err = v["error"].as_str().unwrap_or("책갈피 삭제 실패");
                    eprintln!("오류: {err}");
                    return EXIT_RUNTIME;
                }
            }
            Err(e) => {
                eprintln!("오류: 책갈피 삭제 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delbm",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("책갈피 삭제 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("책갈피 삭제 완료: {file_path}"),
    )
}

/// [#5028] `edit delete-table` — 본문 최상위 표 삭제.
fn edit_delete_table(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-table <파일> --table <번호> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => table_arg = Some(n),
                    Err(_) => {
                        eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no)) = (file_path, table_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(t) => t,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    if !dry_run {
        if let Err(e) = doc.delete_table_control_native(sec, para, ctrl) {
            eprintln!("오류: 표 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "deltable",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "table": table_no }),
        &[(sec, para)],
        &format!("표 삭제 예정: {file_path} 표 {table_no}"),
        &format!("표 삭제 완료: {file_path}"),
    )
}

/// [#5036] `edit insert-header-footer` — 머리말/꼬리말 생성.
fn edit_insert_header_footer(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-header-footer <파일> --header|--footer [--section N] [--apply-to 0|1|2] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--section" | "--apply-to" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                if name == "--section" {
                    match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    }
                } else {
                    match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(is_header)) = (file_path, is_header) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.create_header_footer_native(section, is_header, apply_to) {
            eprintln!("오류: 머리말/꼬리말 생성 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hf",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to
        }),
        &[(section, 0)],
        &format!("머리말/꼬리말 생성 예정: {file_path} 구역 {section} apply-to {apply_to}"),
        &format!("머리말/꼬리말 생성 완료: {file_path}"),
    )
}

/// [#5033] `edit rename-bookmark` — 책갈피 이름 변경.
fn edit_rename_bookmark(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit rename-bookmark <파일> --section N --para N --ctrl N --name <이름> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut name: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--name" => {
                i += 1;
                match args.get(i) {
                    Some(v) => name = Some(v.clone()),
                    None => {
                        eprintln!("오류: --name 뒤에 책갈피 이름이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--para" | "--ctrl" => {
                let flag = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match flag.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl), Some(name)) =
        (file_path, section, para, ctrl, name)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if name.trim().is_empty() {
        eprintln!("오류: --name 은 비어 있을 수 없습니다.");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        match doc.rename_bookmark_native(section, para, ctrl, &name) {
            Ok(raw) => {
                let v: serde_json::Value =
                    serde_json::from_str(&raw).unwrap_or(serde_json::json!({}));
                if v["ok"] == false {
                    let err = v["error"].as_str().unwrap_or("책갈피 이름 변경 실패");
                    eprintln!("오류: {err}");
                    return EXIT_RUNTIME;
                }
            }
            Err(e) => {
                eprintln!("오류: 책갈피 이름 변경 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "renbm",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "ctrl": ctrl,
            "name": name
        }),
        &[(section, para)],
        &format!(
            "책갈피 이름 변경 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl} → {name}"
        ),
        &format!("책갈피 이름 변경 완료: {file_path}"),
    )
}

/// [#5039] `edit delete-header-footer` — 머리말/꼬리말 삭제.
fn edit_delete_header_footer(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-header-footer <파일> --header|--footer [--section N] [--apply-to 0|1|2] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--section" | "--apply-to" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                if name == "--section" {
                    match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    }
                } else {
                    match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(is_header)) = (file_path, is_header) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.delete_header_footer_native(section, is_header, apply_to) {
            eprintln!("오류: 머리말/꼬리말 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delhf",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to
        }),
        &[(section, 0)],
        &format!("{kind} 삭제 예정: {file_path} 구역 {section} apply-to {apply_to}"),
        &format!("{kind} 삭제 완료: {file_path}"),
    )
}

/// `edit insert-header-footer-text` — 기존 머리말/꼬리말 문단에 텍스트 삽입.
fn edit_insert_header_footer_text(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-header-footer-text <파일> --header|--footer --text <문자열> [--section N] [--apply-to 0|1|2] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut text_arg: Option<&str> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--text" => {
                i += 1;
                match args.get(i) {
                    Some(v) => text_arg = Some(v),
                    None => {
                        eprintln!("오류: --text 뒤에 넣을 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--apply-to" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => offset = n,
                        Err(_) => {
                            eprintln!("오류: --offset 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(is_header), Some(text)) = (file_path, is_header, text_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if text.is_empty() {
        eprintln!("오류: --text 는 빈 문자열일 수 없습니다.");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc
            .insert_text_in_header_footer_native(section, is_header, apply_to, para, offset, text)
        {
            eprintln!("오류: 머리말/꼬리말 텍스트 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    let inserted_chars = text.chars().count();
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfins",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para,
            "offset": offset,
            "text": text,
            "insertedChars": inserted_chars
        }),
        &[(section, 0)],
        &format!("{kind} 텍스트 삽입 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset}"),
        &format!("{kind} 텍스트 삽입 완료: {file_path}"),
    )
}

/// `edit set-header-footer-text` — 기존 머리말/꼬리말 문단 텍스트 교체.
fn edit_set_header_footer_text(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-header-footer-text <파일> --header|--footer --text <문자열> [--section N] [--apply-to 0|1|2] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut text_arg: Option<&str> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--text" => {
                i += 1;
                match args.get(i) {
                    Some(v) => text_arg = Some(v),
                    None => {
                        eprintln!("오류: --text 뒤에 바꿀 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--apply-to" | "--para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(is_header), Some(text)) = (file_path, is_header, text_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if text.is_empty() {
        eprintln!("오류: --text 는 빈 문자열일 수 없습니다.");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        let info_raw =
            match doc.get_header_footer_para_info_native(section, is_header, apply_to, para) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("오류: 머리말/꼬리말 문단 조회 실패 - {e}");
                    return EXIT_RUNTIME;
                }
            };
        let info: serde_json::Value = match serde_json::from_str(&info_raw) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("오류: 머리말/꼬리말 문단 JSON 파싱 실패 - {e}");
                return EXIT_RUNTIME;
            }
        };
        let char_count = info["charCount"].as_u64().unwrap_or(0) as usize;
        if char_count > 0 {
            if let Err(e) = doc.delete_text_in_header_footer_native(
                section, is_header, apply_to, para, 0, char_count,
            ) {
                eprintln!("오류: 머리말/꼬리말 기존 텍스트 삭제 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
        if let Err(e) =
            doc.insert_text_in_header_footer_native(section, is_header, apply_to, para, 0, text)
        {
            eprintln!("오류: 머리말/꼬리말 텍스트 교체 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfset",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para,
            "text": text
        }),
        &[(section, 0)],
        &format!("{kind} 텍스트 교체 예정: {file_path} 구역 {section} 문단 {para}"),
        &format!("{kind} 텍스트 교체 완료: {file_path}"),
    )
}

/// `edit delete-hf-text` — 기존 머리말/꼬리말 문단에서 글자를 지운다.
fn edit_delete_hf_text(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-hf-text <파일> --header|--footer --count <글자수> [--section N] [--apply-to 0|1|2] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut count_arg: Option<usize> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--section" | "--apply-to" | "--para" | "--offset" | "--count" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--offset" => match v.parse::<usize>() {
                        Ok(n) => offset = n,
                        Err(_) => {
                            eprintln!("오류: --offset 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) if n >= 1 => count_arg = Some(n),
                        Ok(_) => {
                            eprintln!("오류: --count 는 1 이상이어야 합니다.");
                            return EXIT_USAGE;
                        }
                        Err(_) => {
                            eprintln!("오류: --count 뒤에 1 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(is_header), Some(count)) = (file_path, is_header, count_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc
            .delete_text_in_header_footer_native(section, is_header, apply_to, para, offset, count)
        {
            eprintln!("오류: 머리말/꼬리말 텍스트 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfdeltxt",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para,
            "offset": offset,
            "count": count
        }),
        &[(section, 0)],
        &format!(
            "{kind} 텍스트 삭제 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset} 글자 {count}"
        ),
        &format!("{kind} 텍스트 삭제 완료: {file_path}"),
    )
}

/// `edit split-paragraph-in-hf` — 기존 머리말/꼬리말 문단을 오프셋에서 나눈다.
fn edit_split_paragraph_in_hf(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit split-paragraph-in-hf <파일> --header|--footer [--section N] [--apply-to 0|1|2] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--section" | "--apply-to" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => offset = n,
                        Err(_) => {
                            eprintln!("오류: --offset 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(is_header)) = (file_path, is_header) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.split_paragraph_in_header_footer_native(
            section, is_header, apply_to, para, offset, None,
        ) {
            eprintln!("오류: 머리말/꼬리말 문단 분할 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfsplit",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para,
            "offset": offset
        }),
        &[(section, 0)],
        &format!("{kind} 문단 분할 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset}"),
        &format!("{kind} 문단 분할 완료: {file_path}"),
    )
}

/// `edit merge-paragraph-in-hf` — 머리말/꼬리말 문단을 바로 앞 문단과 합친다.
fn edit_merge_paragraph_in_hf(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit merge-paragraph-in-hf <파일> --header|--footer [--section N] [--apply-to 0|1|2] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 1;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 중 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--section" | "--apply-to" | "--para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(is_header)) = (file_path, is_header) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) =
            doc.merge_paragraph_in_header_footer_native(section, is_header, apply_to, para)
        {
            eprintln!("오류: 머리말/꼬리말 문단 병합 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfmerge",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para
        }),
        &[(section, 0)],
        &format!("{kind} 문단 병합 예정: {file_path} 구역 {section} 문단 {para}"),
        &format!("{kind} 문단 병합 완료: {file_path}"),
    )
}

/// `edit split-paragraph-in-cell` — 표 셀 문단을 오프셋에서 나눈다.
fn edit_split_paragraph_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit split-paragraph-in-cell <파일> --table <번호> --row <행> --col <열> [--cell-para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut cell_para_arg: usize = 0;
    let mut offset_arg: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--offset" | "--cell-para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" | "--offset" | "--cell-para" => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--table" => table_arg = Some(n),
                            "--offset" => offset_arg = n,
                            _ => cell_para_arg = n,
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row), Some(col)) =
        (file_path, table_arg, row_arg, col_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl, cell_idx, para_lens, _old) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };
    if cell_para_arg >= para_lens.len() {
        eprintln!(
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 0~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if offset_arg > para_lens[cell_para_arg] {
        eprintln!(
            "오류: --offset 이 문단 길이를 넘습니다 (문단 길이 {}): {offset_arg}",
            para_lens[cell_para_arg]
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.split_paragraph_in_cell_native(
            sec,
            para,
            ctrl,
            cell_idx,
            cell_para_arg,
            offset_arg,
            None,
        ) {
            eprintln!("오류: 셀 문단 분할 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellsplit",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "paragraph": cell_para_arg,
            "offset": offset_arg
        }),
        &[(sec, para)],
        &format!(
            "셀 문단 분할 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg} 오프셋 {offset_arg}"
        ),
        &format!("셀 문단 분할 완료: {file_path}"),
    )
}

/// `edit merge-paragraph-in-cell` — 표 셀 문단을 바로 앞 문단과 합친다.
fn edit_merge_paragraph_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit merge-paragraph-in-cell <파일> --table <번호> --row <행> --col <열> [--cell-para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut cell_para_arg: usize = 1;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--cell-para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" | "--cell-para" => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--table" => table_arg = Some(n),
                            _ => cell_para_arg = n,
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row), Some(col)) =
        (file_path, table_arg, row_arg, col_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if cell_para_arg == 0 {
        eprintln!("오류: --cell-para 는 1 이상이어야 합니다 (첫 문단은 병합할 수 없습니다).");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl, cell_idx, para_lens, _old) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };
    if cell_para_arg >= para_lens.len() {
        eprintln!(
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 1~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.merge_paragraph_in_cell_native(sec, para, ctrl, cell_idx, cell_para_arg)
        {
            eprintln!("오류: 셀 문단 병합 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellmerge",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "paragraph": cell_para_arg
        }),
        &[(sec, para)],
        &format!("셀 문단 병합 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg}"),
        &format!("셀 문단 병합 완료: {file_path}"),
    )
}

/// `edit apply-char-format` — 본문 문단 글자 범위에 글자 서식을 적용한다.
fn edit_apply_char_format(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-char-format <파일> --props <JSON> [--section N] [--para N] [--offset N] [--count N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut count_arg: Option<usize> = None;
    let mut props_arg: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" | "--count" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        "--offset" => offset = n,
                        _ => count_arg = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props_arg = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 글자 서식 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(props)) = (file_path, props_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if serde_json::from_str::<serde_json::Value>(props).is_err() {
        eprintln!("오류: --props 는 JSON 객체여야 합니다: {props}");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let Some(sec) = doc.document().sections.get(section) else {
        eprintln!("오류: 구역 {section} 이 없습니다.");
        return EXIT_USAGE;
    };
    let Some(paragraph) = sec.paragraphs.get(para) else {
        eprintln!("오류: 문단 {para} 이 없습니다.");
        return EXIT_USAGE;
    };
    let para_len = paragraph.text.chars().count();
    if offset > para_len {
        eprintln!("오류: --offset 이 문단 길이를 넘습니다 (문단 길이 {para_len}): {offset}");
        return EXIT_USAGE;
    }
    let end = match count_arg {
        Some(n) => offset.saturating_add(n).min(para_len),
        None => para_len,
    };
    if !dry_run {
        if let Err(e) = doc.apply_char_format_native(section, para, offset, end, props) {
            eprintln!("오류: 글자 서식 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "chfmt",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "offset": offset,
            "count": end.saturating_sub(offset),
            "text": props
        }),
        &[(section, para)],
        &format!("글자 서식 적용 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset}"),
        &format!("글자 서식 적용 완료: {file_path}"),
    )
}

/// `edit apply-para-format` — 본문 문단에 문단 서식을 적용한다.
fn edit_apply_para_format(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-para-format <파일> --props <JSON> [--section N] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut props_arg: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        _ => para = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props_arg = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 문단 서식 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(props)) = (file_path, props_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if serde_json::from_str::<serde_json::Value>(props).is_err() {
        eprintln!("오류: --props 는 JSON 객체여야 합니다: {props}");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let Some(sec) = doc.document().sections.get(section) else {
        eprintln!("오류: 구역 {section} 이 없습니다.");
        return EXIT_USAGE;
    };
    if sec.paragraphs.get(para).is_none() {
        eprintln!("오류: 문단 {para} 이 없습니다.");
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.apply_para_format_native(section, para, props) {
            eprintln!("오류: 문단 서식 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "pfmt",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "text": props
        }),
        &[(section, para)],
        &format!("문단 서식 적용 예정: {file_path} 구역 {section} 문단 {para}"),
        &format!("문단 서식 적용 완료: {file_path}"),
    )
}

/// `edit apply-char-format-in-cell` — 표 셀 글자 서식. 코어 `apply_char_format_in_cell_native`.
fn edit_apply_char_format_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-char-format-in-cell <파일> (--table N --row N --col N | --section N --para N --ctrl N --cell N) [--cell-para N] [--start N] [--end N] [--offset N] [--count N] [--props JSON] [--bold] [--font-size N] [--color 색] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut section_arg: Option<usize> = None;
    let mut para_arg: Option<usize> = None;
    let mut ctrl_arg: Option<usize> = None;
    let mut cell_arg: Option<usize> = None;
    let mut cell_para_arg: usize = 0;
    let mut start_arg: Option<usize> = None;
    let mut end_arg: Option<usize> = None;
    let mut offset_arg: Option<usize> = None;
    let mut count_arg: Option<usize> = None;
    let mut props_arg: Option<String> = None;
    let mut bold_flag = false;
    let mut font_size_arg: Option<i32> = None;
    let mut color_arg: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--section" | "--para" | "--ctrl" | "--cell"
            | "--cell-para" | "--start" | "--end" | "--offset" | "--count" | "--font-size" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--col" => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--font-size" => match v.parse::<i32>() {
                        Ok(n) if n > 0 => font_size_arg = Some(n),
                        _ => {
                            eprintln!("오류: --font-size 뒤에 1 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--table" => table_arg = Some(n),
                            "--section" => section_arg = Some(n),
                            "--para" => para_arg = Some(n),
                            "--ctrl" => ctrl_arg = Some(n),
                            "--cell" => cell_arg = Some(n),
                            "--cell-para" => cell_para_arg = n,
                            "--start" => start_arg = Some(n),
                            "--end" => end_arg = Some(n),
                            "--offset" => offset_arg = Some(n),
                            _ => count_arg = Some(n),
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props_arg = Some(v.clone()),
                    _ => {
                        eprintln!("오류: --props 뒤에 글자 서식 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--color" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => color_arg = Some(v.clone()),
                    _ => {
                        eprintln!("오류: --color 뒤에 색 값이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--bold" => bold_flag = true,
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let mut props_val = match props_arg.as_deref() {
        Some(raw) => match serde_json::from_str::<serde_json::Value>(raw) {
            Ok(serde_json::Value::Object(map)) => serde_json::Value::Object(map),
            Ok(_) | Err(_) => {
                eprintln!("오류: --props 는 JSON 객체여야 합니다: {raw}");
                return EXIT_USAGE;
            }
        },
        None => serde_json::json!({}),
    };
    if bold_flag {
        props_val["bold"] = serde_json::json!(true);
    }
    if let Some(n) = font_size_arg {
        props_val["fontSize"] = serde_json::json!(n);
    }
    if let Some(ref c) = color_arg {
        props_val["textColor"] = serde_json::json!(c);
    }
    if props_val.as_object().is_none_or(|o| o.is_empty()) {
        eprintln!("오류: --props 또는 --bold/--font-size/--color 가 필요합니다.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }
    let props = props_val.to_string();
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, parent_para, ctrl, cell_idx, para_lens, table_no, row, col) = match (
        table_arg,
        row_arg,
        col_arg,
        section_arg,
        para_arg,
        ctrl_arg,
        cell_arg,
    ) {
        (Some(table_no), Some(row), Some(col), _, _, _, _) => {
            match resolve_table_cell(doc.document(), table_no, row, col) {
                Ok((sec, para, ctrl, cell_idx, para_lens, _)) => {
                    (sec, para, ctrl, cell_idx, para_lens, table_no, row, col)
                }
                Err(CellResolveError::Usage(msg)) => {
                    eprintln!("{msg}");
                    return EXIT_USAGE;
                }
                Err(CellResolveError::Runtime(msg)) => {
                    eprintln!("{msg}");
                    return EXIT_RUNTIME;
                }
            }
        }
        (_, _, _, Some(sec), Some(para), Some(ctrl), Some(cell_idx)) => {
            let para_lens = match cell_para_lens(doc.document(), sec, para, ctrl, cell_idx) {
                Ok(v) => v,
                Err(msg) => {
                    eprintln!("{msg}");
                    return EXIT_USAGE;
                }
            };
            (sec, para, ctrl, cell_idx, para_lens, 0, 0, 0)
        }
        _ => {
            eprintln!("{USAGE}");
            return EXIT_USAGE;
        }
    };
    if cell_para_arg >= para_lens.len() {
        eprintln!(
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 0~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_len = para_lens[cell_para_arg];
    let start = start_arg.or(offset_arg).unwrap_or(0);
    if start > para_len {
        eprintln!("오류: --start/--offset 이 문단 길이를 넘습니다 (문단 길이 {para_len}): {start}");
        return EXIT_USAGE;
    }
    let end = if let Some(e) = end_arg {
        e
    } else if let Some(n) = count_arg {
        start.saturating_add(n)
    } else {
        para_len
    };
    if end < start || end > para_len {
        eprintln!("오류: --end 가 범위를 벗어났습니다 (시작 {start}, 문단 길이 {para_len}): {end}");
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.apply_char_format_in_cell_native(
            sec,
            parent_para,
            ctrl,
            cell_idx,
            cell_para_arg,
            start,
            end,
            &props,
        ) {
            eprintln!("오류: 셀 글자 서식 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "chfmtcell",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "section": sec,
            "paragraph": parent_para,
            "ctrl": ctrl,
            "cellPara": cell_para_arg,
            "innerPara": cell_para_arg,
            "offset": start,
            "count": end.saturating_sub(start),
            "text": props,
            "props": props,
            "bold": bold_flag,
            "fontSize": font_size_arg,
            "color": color_arg,
        }),
        &[(sec, parent_para)],
        &format!(
            "셀 글자 서식 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg} {start}..{end}"
        ),
        &format!("셀 글자 서식 적용 완료: {file_path}"),
    )
}

fn cell_para_lens(
    document: &rhwp::model::document::Document,
    section: usize,
    para: usize,
    ctrl: usize,
    cell_idx: usize,
) -> Result<Vec<usize>, String> {
    use rhwp::model::control::Control;
    let Some(sec) = document.sections.get(section) else {
        return Err(format!("오류: 구역 {section} 이 없습니다."));
    };
    let Some(paragraph) = sec.paragraphs.get(para) else {
        return Err(format!("오류: 문단 {para} 이 없습니다."));
    };
    let Some(Control::Table(table)) = paragraph.controls.get(ctrl) else {
        return Err(format!("오류: 문단 {para} 컨트롤 {ctrl} 은 표가 아닙니다."));
    };
    let Some(cell) = table.cells.get(cell_idx) else {
        return Err(format!(
            "오류: --cell 이 범위를 벗어났습니다 (셀 0~{}): {cell_idx}",
            table.cells.len().saturating_sub(1)
        ));
    };
    Ok(cell
        .paragraphs
        .iter()
        .map(|p| p.text.chars().count())
        .collect())
}

/// `edit apply-style` — 본문 문단에 스타일을 적용한다.
fn edit_apply_style(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-style <파일> --style N [--section N] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut style_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--style" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        _ => style_arg = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(style_id)) = (file_path, style_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if style_id >= doc.document().doc_info.styles.len() {
        eprintln!(
            "오류: --style 이 범위를 벗어났습니다 (스타일 0~{}): {style_id}",
            doc.document().doc_info.styles.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let Some(sec) = doc.document().sections.get(section) else {
        eprintln!("오류: 구역 {section} 이 없습니다.");
        return EXIT_USAGE;
    };
    if sec.paragraphs.get(para).is_none() {
        eprintln!("오류: 문단 {para} 이 없습니다.");
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) = doc.apply_style_native(section, para, style_id) {
            eprintln!("오류: 스타일 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "style",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "ctrl": style_id
        }),
        &[(section, para)],
        &format!("스타일 적용 예정: {file_path} 구역 {section} 문단 {para} 스타일 {style_id}"),
        &format!("스타일 적용 완료: {file_path}"),
    )
}

/// `edit apply-cell-style` — 표 셀 문단에 스타일을 적용한다.
fn edit_apply_cell_style(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-cell-style <파일> --table <번호> --row <행> --col <열> --style N [--cell-para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut cell_para_arg: usize = 0;
    let mut style_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--cell-para" | "--style" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" | "--cell-para" | "--style" => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--table" => table_arg = Some(n),
                            "--style" => style_arg = Some(n),
                            _ => cell_para_arg = n,
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(style_id)) =
        (file_path, table_arg, row_arg, col_arg, style_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if style_id >= doc.document().doc_info.styles.len() {
        eprintln!(
            "오류: --style 이 범위를 벗어났습니다 (스타일 0~{}): {style_id}",
            doc.document().doc_info.styles.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let (sec, para, ctrl, cell_idx, para_lens, _old) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };
    if cell_para_arg >= para_lens.len() {
        eprintln!(
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 0~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) =
            doc.apply_cell_style_native(sec, para, ctrl, cell_idx, cell_para_arg, style_id)
        {
            eprintln!("오류: 셀 스타일 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellstyle",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "paragraph": cell_para_arg,
            "ctrl": style_id
        }),
        &[(sec, para)],
        &format!(
            "셀 스타일 적용 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg} 스타일 {style_id}"
        ),
        &format!("셀 스타일 적용 완료: {file_path}"),
    )
}

/// `edit apply-para-format-in-cell` — 표 셀 문단에 문단 서식을 적용한다.
fn edit_apply_para_format_in_cell(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-para-format-in-cell <파일> --table <번호> --row <행> --col <열> --props <JSON> [--cell-para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut row_arg: Option<u16> = None;
    let mut col_arg: Option<u16> = None;
    let mut cell_para_arg: usize = 0;
    let mut props_arg: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" | "--row" | "--col" | "--cell-para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--table" | "--cell-para" => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--table" => table_arg = Some(n),
                            _ => cell_para_arg = n,
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--row" => match v.parse::<u16>() {
                        Ok(n) => row_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --row 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u16>() {
                        Ok(n) => col_arg = Some(n),
                        Err(_) => {
                            eprintln!("오류: --col 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props_arg = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 문단 서식 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no), Some(row), Some(col), Some(props)) =
        (file_path, table_arg, row_arg, col_arg, props_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if serde_json::from_str::<serde_json::Value>(props).is_err() {
        eprintln!("오류: --props 는 JSON 객체여야 합니다: {props}");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (sec, para, ctrl, cell_idx, para_lens, _old) =
        match resolve_table_cell(doc.document(), table_no, row, col) {
            Ok(v) => v,
            Err(CellResolveError::Usage(msg)) => {
                eprintln!("{msg}");
                return EXIT_USAGE;
            }
            Err(CellResolveError::Runtime(msg)) => {
                eprintln!("{msg}");
                return EXIT_RUNTIME;
            }
        };
    if cell_para_arg >= para_lens.len() {
        eprintln!(
            "오류: --cell-para 가 범위를 벗어났습니다 (셀 문단 0~{}): {cell_para_arg}",
            para_lens.len().saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    if !dry_run {
        if let Err(e) =
            doc.apply_para_format_in_cell_native(sec, para, ctrl, cell_idx, cell_para_arg, props)
        {
            eprintln!("오류: 셀 문단 서식 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "cellpfmt",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "row": row,
            "col": col,
            "paragraph": cell_para_arg,
            "text": props
        }),
        &[(sec, para)],
        &format!(
            "셀 문단 서식 적용 예정: {file_path} 표 {table_no} ({row},{col}) 문단 {cell_para_arg}"
        ),
        &format!("셀 문단 서식 적용 완료: {file_path}"),
    )
}

/// [#5041] `edit delete-control` — 문단 컨트롤 삭제 (갈래 무관).
fn edit_delete_control(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-control <파일> --section N --para N --ctrl N [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl)) = (file_path, section, para, ctrl)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.delete_control_native(section, para, ctrl) {
            eprintln!("오류: 컨트롤 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delctrl",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("컨트롤 삭제 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("컨트롤 삭제 완료: {file_path}"),
    )
}

/// `edit insert-field-in-hf` — 머리말/꼬리말 필드 삽입. 코어 `insert_field_in_hf_native`.
fn edit_insert_field_in_hf(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-field-in-hf <파일> --header|--footer --field-type <1|2|3> [--section N] [--apply-to 0|1|2] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut field_type: Option<u8> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--field-type" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --field-type 뒤에 1·2·3 이 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<u8>() {
                    Ok(n) if (1..=3).contains(&n) => field_type = Some(n),
                    _ => {
                        eprintln!("오류: --field-type 은 1(쪽번호)·2(총쪽수)·3(파일이름) 만 허용합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--apply-to" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => offset = n,
                        Err(_) => {
                            eprintln!("오류: --offset 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(is_header), Some(field_type)) = (file_path, is_header, field_type)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) =
            doc.insert_field_in_hf_native(section, is_header, apply_to, para, offset, field_type)
        {
            eprintln!("오류: 머리말/꼬리말 필드 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hffield",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para,
            "offset": offset,
            "fieldType": field_type
        }),
        &[(section, 0)],
        &format!("{kind} 필드 삽입 예정: {file_path} 구역 {section} type {field_type}"),
        &format!("{kind} 필드 삽입 완료: {file_path}"),
    )
}

/// [#5081] `edit set-column-def` — 구역 단 정의. 코어 `set_column_def_native`.
fn edit_set_column_def(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-column-def <파일> --count N [--section N] [--type 0|1|2] [--same-width|--mixed-width] [--spacing N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut count_arg: Option<u16> = None;
    let mut section: usize = 0;
    let mut column_type: u8 = 0;
    let mut same_width = true;
    let mut spacing: i16 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--count" | "--section" | "--type" | "--spacing" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--count" => match v.parse::<u16>() {
                        Ok(n) if n >= 1 => count_arg = Some(n),
                        _ => {
                            eprintln!("오류: --count 뒤에 1 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--type" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => column_type = n,
                        _ => {
                            eprintln!("오류: --type 은 0(일반)·1(배분)·2(평행) 만 허용합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<i16>() {
                        Ok(n) => spacing = n,
                        Err(_) => {
                            eprintln!("오류: --spacing 뒤에 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--same-width" => same_width = true,
            "--mixed-width" => same_width = false,
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(count)) = (file_path, count_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.set_column_def_native(section, count, column_type, same_width, spacing)
        {
            eprintln!("오류: 단 정의 변경 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "coldef",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "columnCount": count,
            "columnType": column_type,
            "sameWidth": same_width,
            "spacing": spacing
        }),
        &[(section, 0)],
        &format!("단 정의 변경 예정: {file_path} 구역 {section} 단 {count}"),
        &format!("단 정의 변경 완료: {file_path}"),
    )
}

/// [#5082] `edit split-paragraph` — 본문 문단 분할. 코어 `split_paragraph_native`.
fn edit_split_paragraph(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit split-paragraph <파일> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        _ => offset = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.split_paragraph_native(section, para, offset, None) {
            eprintln!("오류: 문단 분할 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "splitpara",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "offset": offset }),
        &[(section, para)],
        &format!("문단 분할 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset}"),
        &format!("문단 분할 완료: {file_path}"),
    )
}

/// [#5083] `edit set-page-hide` — 쪽 감추기. 코어 `set_page_hide_native`.
fn edit_set_page_hide(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-page-hide <파일> [--section N] [--para N] [--hide-header] [--hide-footer] [--hide-master] [--hide-border] [--hide-fill] [--hide-page-num] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut hide_header = false;
    let mut hide_footer = false;
    let mut hide_master = false;
    let mut hide_border = false;
    let mut hide_fill = false;
    let mut hide_page_num = false;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => {
                        if name == "--section" {
                            section = n;
                        } else {
                            para = n;
                        }
                    }
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--hide-header" => hide_header = true,
            "--hide-footer" => hide_footer = true,
            "--hide-master" => hide_master = true,
            "--hide-border" => hide_border = true,
            "--hide-fill" => hide_fill = true,
            "--hide-page-num" => hide_page_num = true,
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.set_page_hide_native(
            section,
            para,
            hide_header,
            hide_footer,
            hide_master,
            hide_border,
            hide_fill,
            hide_page_num,
        ) {
            eprintln!("오류: 쪽 감추기 설정 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "pagehide",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "hideHeader": hide_header,
            "hideFooter": hide_footer,
            "hideMasterPage": hide_master,
            "hideBorder": hide_border,
            "hideFill": hide_fill,
            "hidePageNum": hide_page_num
        }),
        &[(section, para)],
        &format!("쪽 감추기 예정: {file_path} 구역 {section} 문단 {para}"),
        &format!("쪽 감추기 완료: {file_path}"),
    )
}

/// [#5108] `edit transpose-table` — 표 행/열 바꿈. 코어 `transpose_table_cells_in_place_native`.
fn edit_transpose_table(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit transpose-table <파일> --table <번호> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--table" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => table_arg = Some(n),
                    Err(_) => {
                        eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(table_no)) = (file_path, table_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let (section, para, ctrl) = match resolve_top_table(doc.document(), table_no) {
        Ok(v) => v,
        Err(msg) => {
            eprintln!("{msg}");
            return EXIT_USAGE;
        }
    };
    let mut source_rows = 0u32;
    let mut source_cols = 0u32;
    let mut target_rows = 0u32;
    let mut target_cols = 0u32;
    if !dry_run {
        match doc.transpose_table_cells_in_place_native(section, para, ctrl) {
            Ok(raw) => {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&raw) {
                    source_rows = v["sourceRows"].as_u64().unwrap_or(0) as u32;
                    source_cols = v["sourceCols"].as_u64().unwrap_or(0) as u32;
                    target_rows = v["targetRows"].as_u64().unwrap_or(0) as u32;
                    target_cols = v["targetCols"].as_u64().unwrap_or(0) as u32;
                }
            }
            Err(e) => {
                eprintln!("오류: 표 행/열 바꿈 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "transpose",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "table": table_no,
            "section": section,
            "paragraph": para,
            "ctrl": ctrl,
            "sourceRows": source_rows,
            "sourceCols": source_cols,
            "targetRows": target_rows,
            "targetCols": target_cols
        }),
        &[(section, para)],
        &format!("표 행/열 바꿈 예정: {file_path} 표 {table_no}"),
        &format!("표 행/열 바꿈 완료: {file_path}"),
    )
}

/// `edit insert-image` — 도장·서명 같은 그림을 쪽 좌표에 붙인다 (#3719 §6-5).
///
/// 실물 서식 제출의 마지막 조각이다. 채워 넣은 서식에 직인·서명 이미지를 얹지 못하면
/// 사람이 한 번 더 한컴을 열어야 하고, 그 순간 자동화 사슬이 끊긴다.
///
/// 새 삽입 로직을 만들지 않는다 — 검증된 코어 `insert_picture_native` 의 **본문 floating
/// 분기**(용지 기준 offset, `treat_as_char=false`, 한컴 native 기본값)를 그대로 쓴다.
/// 인자 파싱·저장·봉투·`--verify`·`changedPages` 는 `edit set-cell` 과 같은 형태다.
///
/// **길이 단위는 전부 HWPUNIT(1/7200 inch)** 이다 — px 로 오해하면 도장이 점만 하게
/// 찍히거나 아예 안 보인다. A4 세로는 59528 × 84188 HWPUNIT.
fn edit_insert_image(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-image <파일> --image <그림> [--page N] [--x N --y N] [--width N --height N] [-o <출력>] [--dry-run] [--verify] [--json]";

    let mut file_path: Option<&str> = None;
    let mut image_path: Option<&str> = None;
    let mut page_arg: u32 = 0;
    let mut x_hu: u32 = 0;
    let mut y_hu: u32 = 0;
    let mut width_arg: Option<u32> = None;
    let mut height_arg: Option<u32> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    // [#3702] 저장 직후 자기검증 — 판정은 데이터, 차이 시 exit 3.
    let mut verify_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--image" => {
                i += 1;
                match args.get(i) {
                    Some(v) => image_path = Some(v),
                    None => {
                        eprintln!("오류: --image 뒤에 그림 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--page" | "--x" | "--y" | "--width" | "--height" => {
                let name = args[i].clone();
                // 단위를 오류 문구에도 박아 둔다 — px 로 넣으면 도장이 사라진다.
                let unit = if name == "--page" {
                    " (0부터)"
                } else {
                    " (HWPUNIT, 1/7200 inch)"
                };
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다{unit}.");
                    return EXIT_USAGE;
                };
                let Ok(value) = v.parse::<u32>() else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다{unit}: {v}");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--page" => page_arg = value,
                    "--x" => x_hu = value,
                    "--y" => y_hu = value,
                    "--width" => width_arg = Some(value),
                    _ => height_arg = Some(value),
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let (Some(file_path), Some(image_path)) = (file_path, image_path) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    for (name, value) in [("--width", width_arg), ("--height", height_arg)] {
        if value == Some(0) {
            eprintln!("오류: {name} 는 1 이상이어야 합니다 (HWPUNIT, 1/7200 inch).");
            return EXIT_USAGE;
        }
    }

    // ── 그림 선검증 — 문서를 읽기 전에 끊는다 ──
    // 지원하지 않는 형식은 **인자 문제**다(런타임 실패가 아니다) → exit 2.
    let image_ext = Path::new(image_path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if !INSERT_IMAGE_FORMATS.contains(&image_ext.as_str()) {
        eprintln!(
            "오류: 지원하지 않는 그림 형식입니다 - {} (지원: {})",
            if image_ext.is_empty() {
                "확장자 없음".to_string()
            } else {
                image_ext.clone()
            },
            INSERT_IMAGE_FORMATS.join(", ")
        );
        return EXIT_USAGE;
    }
    let image_bytes = match fs::read(image_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 그림 파일을 읽을 수 없습니다 - {}: {}", image_path, e);
            return EXIT_RUNTIME;
        }
    };
    // 확장자만 믿지 않는다 — 내용이 그림이 아니면 원본 픽셀 크기를 못 재고,
    // 크기를 모르면 배치 좌표가 의미를 잃는다.
    let Some((natural_w_px, natural_h_px)) = insert_image_dimensions(&image_bytes) else {
        eprintln!(
            "오류: 그림 형식을 알아볼 수 없습니다 - {} (지원: {})",
            image_path,
            INSERT_IMAGE_FORMATS.join(", ")
        );
        return EXIT_USAGE;
    };

    // 크기 결정: 둘 다 없으면 원본 픽셀(96dpi 환산), 하나만 주면 원본 비율 유지.
    // 어느 쪽이든 최종 값은 봉투에 그대로 실어 "조용한 보정" 이 없게 한다.
    let (width_hu, height_hu) = match (width_arg, height_arg) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => (
            w,
            ((w as u64 * natural_h_px as u64) / natural_w_px as u64).max(1) as u32,
        ),
        (None, Some(h)) => (
            ((h as u64 * natural_w_px as u64) / natural_h_px as u64).max(1) as u32,
            h,
        ),
        (None, None) => (
            natural_w_px.saturating_mul(HWPUNIT_PER_PX),
            natural_h_px.saturating_mul(HWPUNIT_PER_PX),
        ),
    };
    // 코어는 offset·크기를 i32/u32 로 다룬다. 범위를 넘는 값이 조용히 감기면 도장이
    // 엉뚱한 곳에 찍히므로 인자 오류로 끊는다.
    for (name, value) in [
        ("--x", x_hu),
        ("--y", y_hu),
        ("--width", width_hu),
        ("--height", height_hu),
    ] {
        if value > i32::MAX as u32 {
            eprintln!(
                "오류: {name} 값이 너무 큽니다 (HWPUNIT 최대 {}): {value}",
                i32::MAX
            );
            return EXIT_USAGE;
        }
    }

    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match rhwp::wasm_api::HwpDocument::from_bytes(&bytes) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let page_count = doc.page_count();
    if page_arg >= page_count {
        eprintln!(
            "오류: 페이지 번호가 범위를 벗어났습니다 (0~{}): {page_arg}",
            page_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let Some((sec, para)) = insert_image_page_anchor(&doc, page_arg) else {
        eprintln!("오류: {page_arg}쪽(0 기준)에서 그림을 붙일 본문 문단을 찾지 못했습니다.");
        return EXIT_RUNTIME;
    };

    // [#3480 과 같은 취지] 쪽 밖으로 나가면 **조용히 자르지 않는다**. 에이전트는 렌더
    // 결과를 보지 않으므로 신호가 없으면 잘려 나간 도장을 완성본으로 판단한다.
    let page_def = &doc.document().sections[sec].section_def.page_def;
    let (paper_w, paper_h) = if page_def.landscape {
        (page_def.height as i64, page_def.width as i64)
    } else {
        (page_def.width as i64, page_def.height as i64)
    };
    let right = x_hu as i64 + width_hu as i64;
    let bottom = y_hu as i64 + height_hu as i64;
    let overflow = if right > paper_w || bottom > paper_h {
        Some(serde_json::json!({
            "page": page_arg,
            "paperWidthHu": paper_w,
            "paperHeightHu": paper_h,
            "rightHu": right,
            "bottomHu": bottom,
            "overflowXHu": (right - paper_w).max(0),
            "overflowYHu": (bottom - paper_h).max(0),
        }))
    } else {
        None
    };

    let mut bin_data_id = serde_json::Value::Null;
    if !dry_run {
        // 그림 설명(대체 텍스트)은 파일명 — 한컴이 개체 속성에 보여 주는 값이다.
        let description = Path::new(image_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let inserted = match doc.insert_picture_native(
            sec,
            para,
            0,
            &[],
            &image_bytes,
            width_hu,
            height_hu,
            natural_w_px,
            natural_h_px,
            &image_ext,
            &description,
            Some(x_hu as i32),
            Some(y_hu as i32),
        ) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("오류: 그림 삽입 실패 - {}", e);
                // 실패 시 원본 불변 — 출력 파일을 쓰지 않고 즉시 끝낸다.
                return EXIT_RUNTIME;
            }
        };
        // binDataId 는 새 조회 API 없이 방금 삽입한 컨트롤에서 직접 읽는다 —
        // 같은 그림을 다시 참조하거나(도장 재사용) 산출물을 감사할 때 쓰는 주소다.
        let ctrl_idx = serde_json::from_str::<serde_json::Value>(&inserted)
            .ok()
            .and_then(|v| v["controlIdx"].as_u64())
            .unwrap_or_default() as usize;
        if let Some(rhwp::model::control::Control::Picture(picture)) = doc
            .document()
            .sections
            .get(sec)
            .and_then(|s| s.paragraphs.get(para))
            .and_then(|p| p.controls.get(ctrl_idx))
        {
            bin_data_id = serde_json::json!(picture.image_attr.bin_data_id);
        }
    }

    // [#3383] 입력 형식을 보존한다 — 기본 확장자도 산출 형식을 따른다.
    let out_format = edit_output_format(&bytes, out_path.as_deref());
    let output_path = out_path.unwrap_or_else(|| {
        let stem = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        format!("{}_image.{}", stem, out_format.ext())
    });

    let mut verify_report = serde_json::Value::Null;
    let mut verify_failed = false;
    if !dry_run {
        let out_bytes = match edit_serialize(&mut doc, out_format) {
            Ok(b) => b,
            Err(e) => {
                eprintln!(
                    "오류: {} 직렬화 실패 - {}",
                    out_format.label().to_uppercase(),
                    e
                );
                return EXIT_RUNTIME;
            }
        };
        if let Err(e) = fs::write(&output_path, &out_bytes) {
            eprintln!("오류: 출력 쓰기 실패 - {}: {}", output_path, e);
            return EXIT_RUNTIME;
        }
        // [#3702] 저장 직후 자기검증 — 편집 후 IR ↔ 저장본 재파싱 IR.
        if verify_mode {
            let cross = out_format == EditOutputFormat::Hwp
                && rhwp::parser::detect_format(&bytes) == rhwp::parser::FileFormat::Hwpx;
            let (report, failed) = edit_verify_report(&doc, &out_bytes, cross);
            verify_report = report;
            verify_failed = failed;
        }
    }

    // [#3712] 눈검증 대상 페이지 — 앵커 문단이 걸친 쪽 전부.
    let changed_pages = if dry_run {
        serde_json::Value::Null
    } else {
        match doc.pages_covering_paragraphs(&[(sec, para)]) {
            Some(pages) => serde_json::json!(pages),
            None => serde_json::Value::Null,
        }
    };

    if json_mode {
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "image": image_path,
            "page": page_arg,
            "x": x_hu,
            "y": y_hu,
            "width": width_hu,
            "height": height_hu,
            "binDataId": bin_data_id,
            "dryRun": dry_run,
            "changedPages": changed_pages,
            "overflow": overflow.clone().map(|o| vec![o]).unwrap_or_default(),
        });
        if !dry_run {
            envelope["output"] = serde_json::Value::String(output_path.clone());
            envelope["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
            envelope["verify"] = verify_report.clone();
        }
        // [#3885] 이 봉투의 값은 전부 호출자 인자·엔진 판정이라 문서 유래 경로가
        // 없지만, 표지 자체는 항상 싣는다 — 키 부재는 "안전"이 아니라 "판정 안 함"
        // 으로 읽어야 하기 때문이다(S1).
        println!("{}", provenance::marked(envelope, "edit"));
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }

    if dry_run {
        println!(
            "배치 예정: {} {}쪽 ({}, {}) 크기 {}×{} HWPUNIT ← {} (원본 {}×{}px)",
            file_path,
            page_arg,
            x_hu,
            y_hu,
            width_hu,
            height_hu,
            image_path,
            natural_w_px,
            natural_h_px
        );
    } else {
        println!(
            "그림 삽입 완료: {} → {} — {}쪽 ({}, {}) 크기 {}×{} HWPUNIT ← {} (원본 {}×{}px)",
            file_path,
            output_path,
            page_arg,
            x_hu,
            y_hu,
            width_hu,
            height_hu,
            image_path,
            natural_w_px,
            natural_h_px
        );
    }
    if overflow.is_some() {
        eprintln!(
            "경고: 그림이 쪽 밖으로 나갑니다 (용지 {}×{} HWPUNIT, 오른쪽 {} 아래 {}) — 상세는 --json 의 overflow",
            paper_w, paper_h, right, bottom
        );
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    EXIT_OK
}

/// `edit insert-picture` — 문단 좌표에 본문 그림을 끼운다. 코어 `insert_picture_native`.
/// `insert-image`(도장·서명, 쪽 좌표) 와 다르다.
fn edit_insert_picture(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-picture <파일> --image <그림> [--section N] [--para N] [--offset N] [--width N] [--height N] [--x N] [--y N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut image_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut x_hu: u32 = 0;
    let mut y_hu: u32 = 0;
    let mut width_arg: Option<u32> = None;
    let mut height_arg: Option<u32> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--image" => {
                i += 1;
                match args.get(i) {
                    Some(v) => image_path = Some(v),
                    None => {
                        eprintln!("오류: --image 뒤에 그림 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--para" | "--offset" | "--x" | "--y" | "--width" | "--height" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--para" => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--offset" => match v.parse::<usize>() {
                        Ok(n) => offset = n,
                        Err(_) => {
                            eprintln!("오류: --offset 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--x" => match v.parse::<u32>() {
                        Ok(n) => x_hu = n,
                        Err(_) => {
                            eprintln!("오류: --x 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT): {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--y" => match v.parse::<u32>() {
                        Ok(n) => y_hu = n,
                        Err(_) => {
                            eprintln!("오류: --y 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT): {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--width" => match v.parse::<u32>() {
                        Ok(n) => width_arg = Some(n),
                        Err(_) => {
                            eprintln!(
                                "오류: --width 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT): {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<u32>() {
                        Ok(n) => height_arg = Some(n),
                        Err(_) => {
                            eprintln!(
                                "오류: --height 뒤에 0 이상의 정수가 필요합니다 (HWPUNIT): {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(image_path)) = (file_path, image_path) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    for (name, value) in [("--width", width_arg), ("--height", height_arg)] {
        if value == Some(0) {
            eprintln!("오류: {name} 는 1 이상이어야 합니다 (HWPUNIT, 1/7200 inch).");
            return EXIT_USAGE;
        }
    }
    let image_ext = Path::new(image_path)
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if !INSERT_IMAGE_FORMATS.contains(&image_ext.as_str()) {
        eprintln!(
            "오류: 지원하지 않는 그림 형식입니다 - {} (지원: {})",
            if image_ext.is_empty() {
                "확장자 없음".to_string()
            } else {
                image_ext.clone()
            },
            INSERT_IMAGE_FORMATS.join(", ")
        );
        return EXIT_USAGE;
    }
    let image_bytes = match fs::read(image_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 그림 파일을 읽을 수 없습니다 - {}: {}", image_path, e);
            return EXIT_RUNTIME;
        }
    };
    let Some((natural_w_px, natural_h_px)) = insert_image_dimensions(&image_bytes) else {
        eprintln!(
            "오류: 그림 형식을 알아볼 수 없습니다 - {} (지원: {})",
            image_path,
            INSERT_IMAGE_FORMATS.join(", ")
        );
        return EXIT_USAGE;
    };
    let (width_hu, height_hu) = match (width_arg, height_arg) {
        (Some(w), Some(h)) => (w, h),
        (Some(w), None) => (
            w,
            ((w as u64 * natural_h_px as u64) / natural_w_px as u64).max(1) as u32,
        ),
        (None, Some(h)) => (
            ((h as u64 * natural_w_px as u64) / natural_h_px as u64).max(1) as u32,
            h,
        ),
        (None, None) => (
            natural_w_px.saturating_mul(HWPUNIT_PER_PX),
            natural_h_px.saturating_mul(HWPUNIT_PER_PX),
        ),
    };
    for (name, value) in [
        ("--x", x_hu),
        ("--y", y_hu),
        ("--width", width_hu),
        ("--height", height_hu),
    ] {
        if value > i32::MAX as u32 {
            eprintln!(
                "오류: {name} 값이 너무 큽니다 (HWPUNIT 최대 {}): {value}",
                i32::MAX
            );
            return EXIT_USAGE;
        }
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let section_count = doc.document().sections.len();
    if section >= section_count {
        eprintln!(
            "오류: --section 이 범위를 벗어났습니다 (0~{}): {section}",
            section_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let para_count = doc.document().sections[section].paragraphs.len();
    if para >= para_count {
        eprintln!(
            "오류: --para 이 범위를 벗어났습니다 (구역 {section} 문단 0~{}): {para}",
            para_count.saturating_sub(1)
        );
        return EXIT_USAGE;
    }
    let mut bin_data_id = serde_json::Value::Null;
    if !dry_run {
        let description = Path::new(image_path)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        let inserted = match doc.insert_picture_native(
            section,
            para,
            offset,
            &[],
            &image_bytes,
            width_hu,
            height_hu,
            natural_w_px,
            natural_h_px,
            &image_ext,
            &description,
            Some(x_hu as i32),
            Some(y_hu as i32),
        ) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("오류: 그림 삽입 실패 - {e}");
                return EXIT_RUNTIME;
            }
        };
        let ctrl_idx = serde_json::from_str::<serde_json::Value>(&inserted)
            .ok()
            .and_then(|v| v["controlIdx"].as_u64())
            .unwrap_or_default() as usize;
        if let Some(rhwp::model::control::Control::Picture(picture)) = doc
            .document()
            .sections
            .get(section)
            .and_then(|s| s.paragraphs.get(para))
            .and_then(|p| p.controls.get(ctrl_idx))
        {
            bin_data_id = serde_json::json!(picture.image_attr.bin_data_id);
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "picture",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "image": image_path,
            "section": section,
            "paragraph": para,
            "offset": offset,
            "x": x_hu,
            "y": y_hu,
            "width": width_hu,
            "height": height_hu,
            "binDataId": bin_data_id,
        }),
        &[(section, para)],
        &format!(
            "그림 삽입 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset} ← {image_path}"
        ),
        &format!("그림 삽입 완료: {file_path}"),
    )
}

/// `edit delete-picture` — 본문 그림 컨트롤 삭제. 코어 `delete_picture_control_native`.
fn edit_delete_picture(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-picture <파일> --section N --para N --ctrl N [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl)) = (file_path, section, para, ctrl)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.delete_picture_control_native(section, para, ctrl) {
            eprintln!("오류: 그림 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delpic",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("그림 삭제 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("그림 삭제 완료: {file_path}"),
    )
}

/// `edit set-picture` — 본문 그림 속성. 코어 `set_picture_properties_native`.
fn edit_set_picture(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-picture <파일> --section N --para N --ctrl N --props <JSON> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut props: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) => props = Some(v.clone()),
                    None => {
                        eprintln!("오류: --props 뒤에 JSON 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl), Some(props)) =
        (file_path, section, para, ctrl, props)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if props.trim().is_empty() {
        eprintln!("오류: --props 는 비어 있을 수 없습니다.");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.set_picture_properties_native(section, para, ctrl, &props) {
            eprintln!("오류: 그림 속성 설정 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "setpic",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("그림 속성 변경 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("그림 속성 변경 완료: {file_path}"),
    )
}

/// `edit ungroup-shape` — 묶음 풀기. 코어 `ungroup_shape_native`.
fn edit_ungroup_shape(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit ungroup-shape <파일> --section N --para N --ctrl N [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl)) = (file_path, section, para, ctrl)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.ungroup_shape_native(section, para, ctrl) {
            eprintln!("오류: 도형 묶음 풀기 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "ungroup",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("도형 묶음 풀기 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("도형 묶음 풀기 완료: {file_path}"),
    )
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
fn edit_set_hf_picture(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-hf-picture <파일> --section N --para N --ctrl N --inner-para N --inner-ctrl N --props <JSON> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut inner_para: Option<usize> = None;
    let mut inner_ctrl: Option<usize> = None;
    let mut props: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" | "--inner-para" | "--inner-ctrl" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        "--ctrl" => ctrl = Some(n),
                        "--inner-para" => inner_para = Some(n),
                        _ => inner_ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 그림 속성 JSON이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (
        Some(file_path),
        Some(section),
        Some(para),
        Some(ctrl),
        Some(inner_para),
        Some(inner_ctrl),
        Some(props),
    ) = (
        file_path, section, para, ctrl, inner_para, inner_ctrl, props,
    )
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.set_header_footer_picture_properties_native(
            section, para, ctrl, inner_para, inner_ctrl, props,
        ) {
            eprintln!("오류: 머리말/꼬리말 그림 속성 변경 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfpic",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "ctrl": ctrl,
            "innerPara": inner_para,
            "innerCtrl": inner_ctrl,
            "props": props
        }),
        &[(section, para)],
        &format!(
            "머리말/꼬리말 그림 속성 변경 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl} 내부 {inner_para}/{inner_ctrl}"
        ),
        &format!("머리말/꼬리말 그림 속성 변경 완료: {file_path}"),
    )
}

// `edit insert-image` — 도장·서명 같은 그림을 쪽 좌표에 붙인다 (#3719 §6-5).
//
// 실물 서식 제출의 마지막 조각이다. 채워 넣은 서식에 직인·서명 이미지를 얹지 못하면
// 사람이 한 번 더 한컴을 열어야 하고, 그 순간 자동화 사슬이 끊긴다.
//
// 새 삽입 로직을 만들지 않는다 — 검증된 코어 `insert_picture_native` 의 **본문 floating
// 분기**(용지 기준 offset, `treat_as_char=false`, 한컴 native 기본값)를 그대로 쓴다.
// 인자 파싱·저장·봉투·`--verify`·`changedPages` 는 `edit set-cell` 과 같은 형태다.
//
// **길이 단위는 전부 HWPUNIT(1/7200 inch)** 이다 — px 로 오해하면 도장이 점만 하게
// 찍히거나 아예 안 보인다. A4 세로는 59528 × 84188 HWPUNIT.

fn edit_apply_hf_template(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-hf-template <파일> --header|--footer --template <0-10> [--section N] [--apply-to 0|1|2] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut template: Option<u8> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--template" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --template 뒤에 0~10 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<u8>() {
                    Ok(n) if n <= 10 => template = Some(n),
                    _ => {
                        eprintln!("오류: --template 은 0~10 만 허용합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--apply-to" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                if name == "--section" {
                    match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    }
                } else {
                    match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(is_header), Some(template)) = (file_path, is_header, template)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.apply_hf_template_native(section, is_header, apply_to, template) {
            eprintln!("오류: 머리말/꼬리말 마당 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hftpl",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "templateId": template
        }),
        &[(section, 0)],
        &format!("{kind} 마당 적용 예정: {file_path} 구역 {section} template {template}"),
        &format!("{kind} 마당 적용 완료: {file_path}"),
    )
}

// `edit insert-image` — 도장·서명 같은 그림을 쪽 좌표에 붙인다 (#3719 §6-5).
//
// 실물 서식 제출의 마지막 조각이다. 채워 넣은 서식에 직인·서명 이미지를 얹지 못하면
// 사람이 한 번 더 한컴을 열어야 하고, 그 순간 자동화 사슬이 끊긴다.
//
// 새 삽입 로직을 만들지 않는다 — 검증된 코어 `insert_picture_native` 의 **본문 floating
// 분기**(용지 기준 offset, `treat_as_char=false`, 한컴 native 기본값)를 그대로 쓴다.
// 인자 파싱·저장·봉투·`--verify`·`changedPages` 는 `edit set-cell` 과 같은 형태다.
//
// **길이 단위는 전부 HWPUNIT(1/7200 inch)** 이다 — px 로 오해하면 도장이 점만 하게
// 찍히거나 아예 안 보인다. A4 세로는 59528 × 84188 HWPUNIT.

fn edit_toggle_hide_hf(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit toggle-hide-hf <파일> --header|--footer [--page N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut page: u32 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--page" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<u32>() {
                    Ok(n) => page = n,
                    Err(_) => {
                        eprintln!("오류: --page 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(is_header)) = (file_path, is_header) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let mut hidden = false;
    if !dry_run {
        match doc.toggle_hide_header_footer_native(page, is_header) {
            Ok(raw) => {
                hidden = serde_json::from_str::<serde_json::Value>(&raw)
                    .ok()
                    .and_then(|v| v.get("hidden").and_then(|h| h.as_bool()))
                    .unwrap_or(false);
            }
            Err(e) => {
                eprintln!("오류: 머리말/꼬리말 감추기 토글 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfhide",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "page": page,
            "isHeader": is_header,
            "hidden": hidden
        }),
        &[(0, 0)],
        &format!("{kind} 감추기 토글 예정: {file_path} 쪽 {page}"),
        &format!("{kind} 감추기 토글 완료: {file_path}"),
    )
}

// `edit insert-image` — 도장·서명 같은 그림을 쪽 좌표에 붙인다 (#3719 §6-5).
//
// 실물 서식 제출의 마지막 조각이다. 채워 넣은 서식에 직인·서명 이미지를 얹지 못하면
// 사람이 한 번 더 한컴을 열어야 하고, 그 순간 자동화 사슬이 끊긴다.
//
// 새 삽입 로직을 만들지 않는다 — 검증된 코어 `insert_picture_native` 의 **본문 floating
// 분기**(용지 기준 offset, `treat_as_char=false`, 한컴 native 기본값)를 그대로 쓴다.
// 인자 파싱·저장·봉투·`--verify`·`changedPages` 는 `edit set-cell` 과 같은 형태다.
//
// **길이 단위는 전부 HWPUNIT(1/7200 inch)** 이다 — px 로 오해하면 도장이 점만 하게
// 찍히거나 아예 안 보인다. A4 세로는 59528 × 84188 HWPUNIT.

fn edit_apply_para_format_in_hf(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-para-format-in-hf <파일> --header|--footer --props <JSON> [--section N] [--apply-to 0|1|2] [--para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut is_header: Option<bool> = None;
    let mut props: Option<&str> = None;
    let mut section: usize = 0;
    let mut apply_to: u8 = 0;
    let mut para: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--header" => {
                if is_header.replace(true).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--footer" => {
                if is_header.replace(false).is_some() {
                    eprintln!("오류: --header 와 --footer 는 하나만 지정합니다.");
                    return EXIT_USAGE;
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 문단 서식 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--apply-to" | "--para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--apply-to" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => apply_to = n,
                        _ => {
                            eprintln!(
                                "오류: --apply-to 는 0(양쪽)·1(짝수)·2(홀수) 만 허용합니다: {v}"
                            );
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => para = n,
                        Err(_) => {
                            eprintln!("오류: --para 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(is_header), Some(props)) = (file_path, is_header, props) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) =
            doc.apply_para_format_in_hf_native(section, is_header, apply_to, para, props)
        {
            eprintln!("오류: 머리말/꼬리말 문단 서식 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    let kind = if is_header { "머리말" } else { "꼬리말" };
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "hfpfmt",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "isHeader": is_header,
            "applyTo": apply_to,
            "paragraph": para,
            "props": props
        }),
        &[(section, 0)],
        &format!("{kind} 문단 서식 적용 예정: {file_path} 구역 {section}"),
        &format!("{kind} 문단 서식 적용 완료: {file_path}"),
    )
}

// `edit insert-image` — 도장·서명 같은 그림을 쪽 좌표에 붙인다 (#3719 §6-5).
//
// 실물 서식 제출의 마지막 조각이다. 채워 넣은 서식에 직인·서명 이미지를 얹지 못하면
// 사람이 한 번 더 한컴을 열어야 하고, 그 순간 자동화 사슬이 끊긴다.
//
// 새 삽입 로직을 만들지 않는다 — 검증된 코어 `insert_picture_native` 의 **본문 floating
// 분기**(용지 기준 offset, `treat_as_char=false`, 한컴 native 기본값)를 그대로 쓴다.
// 인자 파싱·저장·봉투·`--verify`·`changedPages` 는 `edit set-cell` 과 같은 형태다.
//
// **길이 단위는 전부 HWPUNIT(1/7200 inch)** 이다 — px 로 오해하면 도장이 점만 하게
// 찍히거나 아예 안 보인다. A4 세로는 59528 × 84188 HWPUNIT.

fn edit_apply_endnote_shape(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-endnote-shape <파일> --props <JSON> [--section N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut props: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => section = n,
                    Err(_) => {
                        eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) => props = Some(v.clone()),
                    None => {
                        eprintln!("오류: --props 뒤에 JSON 문자열이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(props)) = (file_path, props) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.apply_endnote_shape_native(section, &props) {
            eprintln!("오류: 미주 모양 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "enshape",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section }),
        &[(section, 0)],
        &format!("미주 모양 예정: {file_path} 구역 {section}"),
        &format!("미주 모양 적용 완료: {file_path}"),
    )
}

// [#5017] `edit delete-footnote` — 각주/미주 삭제.

fn edit_insert_footnote_text(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-footnote-text <파일> --ctrl N --text <문자열> [--section N] [--para N] [--fn-para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut text_arg: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut ctrl_arg: Option<usize> = None;
    let mut fn_para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--text" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => text_arg = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --text 뒤에 넣을 문자열이 필요합니다 (빈 문자열 거부).");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--para" | "--ctrl" | "--fn-para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        "--ctrl" => ctrl_arg = Some(n),
                        "--fn-para" => fn_para = n,
                        _ => offset = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(ctrl), Some(text)) = (file_path, ctrl_arg, text_arg) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) =
            doc.insert_text_in_footnote_native(section, para, ctrl, fn_para, offset, text)
        {
            eprintln!("오류: 각주 텍스트 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "fntext",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "ctrl": ctrl,
            "fnPara": fn_para,
            "offset": offset,
            "text": text
        }),
        &[(section, para)],
        &format!("각주 텍스트 삽입 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("각주 텍스트 삽입 완료: {file_path}"),
    )
}

// `edit insert-image` — 도장·서명 같은 그림을 쪽 좌표에 붙인다 (#3719 §6-5).
//
// 실물 서식 제출의 마지막 조각이다. 채워 넣은 서식에 직인·서명 이미지를 얹지 못하면
// 사람이 한 번 더 한컴을 열어야 하고, 그 순간 자동화 사슬이 끊긴다.
//
// 새 삽입 로직을 만들지 않는다 — 검증된 코어 `insert_picture_native` 의 **본문 floating
// 분기**(용지 기준 offset, `treat_as_char=false`, 한컴 native 기본값)를 그대로 쓴다.
// 인자 파싱·저장·봉투·`--verify`·`changedPages` 는 `edit set-cell` 과 같은 형태다.
//
// **길이 단위는 전부 HWPUNIT(1/7200 inch)** 이다 — px 로 오해하면 도장이 점만 하게
// 찍히거나 아예 안 보인다. A4 세로는 59528 × 84188 HWPUNIT.

fn edit_split_paragraph_in_footnote(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit split-paragraph-in-footnote <파일> [--section N] [--para N] [--ctrl N] [--fn-para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut ctrl: usize = 0;
    let mut fn_para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" | "--fn-para" | "--offset" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        "--ctrl" => ctrl = n,
                        "--fn-para" => fn_para = n,
                        _ => offset = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) =
            doc.split_paragraph_in_footnote_native(section, para, ctrl, fn_para, offset, None)
        {
            eprintln!("오류: 각주/미주 문단 분할 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "fnsplit",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "ctrl": ctrl,
            "fnPara": fn_para,
            "offset": offset
        }),
        &[(section, para)],
        &format!(
            "각주/미주 문단 분할 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl} 각주문단 {fn_para} 오프셋 {offset}"
        ),
        &format!("각주/미주 문단 분할 완료: {file_path}"),
    )
}

// [#5012] `edit delete-paragraph` — 문단 삭제.

fn edit_merge_paragraph_in_footnote(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit merge-paragraph-in-footnote <파일> [--section N] [--para N] [--ctrl N] [--fn-para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut ctrl: usize = 0;
    let mut fn_para: usize = 1;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" | "--fn-para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        "--ctrl" => ctrl = n,
                        _ => {
                            if n == 0 {
                                eprintln!("오류: --fn-para 는 1 이상이어야 합니다.");
                                return EXIT_USAGE;
                            }
                            fn_para = n;
                        }
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(file_path) = file_path else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) = doc.merge_paragraph_in_footnote_native(section, para, ctrl, fn_para) {
            eprintln!("오류: 각주/미주 문단 병합 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "fnmerge",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "ctrl": ctrl,
            "fnPara": fn_para
        }),
        &[(section, para)],
        &format!(
            "각주/미주 문단 병합 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl} 각주문단 {fn_para}"
        ),
        &format!("각주/미주 문단 병합 완료: {file_path}"),
    )
}

// [#5012] `edit delete-paragraph` — 문단 삭제.

fn edit_apply_para_format_in_footnote(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit apply-para-format-in-footnote <파일> --section N --para N --ctrl N --props <JSON> [--fn-para N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut fn_para: usize = 0;
    let mut props_arg: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" | "--fn-para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match name.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        "--ctrl" => ctrl = Some(n),
                        _ => fn_para = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--props" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => props_arg = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --props 뒤에 문단 서식 JSON 이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(v) => out_path = Some(v.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return EXIT_USAGE;
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl), Some(props)) =
        (file_path, section, para, ctrl, props_arg)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if serde_json::from_str::<serde_json::Value>(props).is_err() {
        eprintln!("오류: --props 는 JSON 객체여야 합니다: {props}");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    if !dry_run {
        if let Err(e) =
            doc.apply_para_format_in_footnote_native(section, para, ctrl, fn_para, props)
        {
            eprintln!("오류: 각주 문단 서식 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "fnpfmt",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "ctrl": ctrl,
            "count": fn_para,
            "text": props
        }),
        &[(section, para)],
        &format!("각주 문단 서식 적용 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("각주 문단 서식 적용 완료: {file_path}"),
    )
}

// [#5041] `edit delete-control` — 문단 컨트롤 삭제 (갈래 무관).
