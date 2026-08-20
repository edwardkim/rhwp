//! Document format conversion command adapters.

use std::fs;
use std::process;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{
    atomic_file, cli_output_password, load_document, load_document_core, paths_refer_to_same_file,
    verification_exit_code, ConversionVerifyOptions, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};

fn parse_conversion_verify_args(
    args: &[String],
    usage: &str,
    min_positionals: usize,
    max_positionals: usize,
    allow_json: bool,
) -> Result<(Vec<String>, ConversionVerifyOptions), String> {
    let mut positionals = Vec::new();
    let mut options = ConversionVerifyOptions::default();

    for arg in args {
        match arg.as_str() {
            "--verify" => options.verify = true,
            "--verify-pages" => options.verify_pages = true,
            // [#3596] 구현 없는 명령이 --json 을 조용히 받으면 소비자가 빈 stdout 을
            // 성공 봉투로 오인한다 — 허용된 명령에서만 받는다.
            "--json" if allow_json => options.json = true,
            value if value.starts_with('-') => {
                return Err(format!("알 수 없는 옵션: {}\n사용법: {}", value, usage));
            }
            value => positionals.push(value.to_string()),
        }
    }

    if positionals.len() < min_positionals || positionals.len() > max_positionals {
        return Err(format!("사용법: {}", usage));
    }

    Ok((positionals, options))
}

fn print_ir_verify_failure(diff: &rhwp::serializer::hwpx::roundtrip::IrDiff, converted: &str) {
    eprintln!(
        "검증 실패(--verify): {} 재파싱 후 IR 차이 {}건",
        converted,
        diff.differences.len()
    );
    for difference in diff.differences.iter().take(20) {
        eprintln!("  [차이] {}", difference);
    }
    if diff.differences.len() > 20 {
        eprintln!(
            "  ... 이하 생략 (총 {}건, 상세 비교는 ir-diff 사용)",
            diff.differences.len()
        );
    }
}

fn verify_reparse_failed_exit_code(options: ConversionVerifyOptions) -> i32 {
    if options.verify {
        3
    } else {
        4
    }
}

/// [#3565] `extract-pages` — 쪽 범위만 남겨 저장한다.
///
/// 대형 문서의 결함을 이분법으로 좁히기 위한 도구다. 384쪽 문서가 저장 후 한컴에서
/// 열리지 않을 때, 절반씩 잘라 재현 여부를 보면 방아쇠를 특정할 수 있다.
///
/// 쪽 단위로 자르되 **문단 단위로** 지운다 — 여러 쪽에 걸친 문단은 한 쪽이라도 범위 안이면
/// 남긴다. 결과 쪽수가 요청 범위와 정확히 같지 않을 수 있다(레이아웃이 다시 흐른다).
pub(crate) fn extract_pages(args: &[String]) -> i32 {
    let mut input: Option<&str> = None;
    let mut output: Option<&str> = None;
    let mut from: Option<u32> = None;
    let mut to: Option<u32> = None;
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--from" | "--to" => {
                // 옵션 이름을 리터럴로 고정하고 인자 값은 에코하지 않는다.
                // 같은 `args` 에 `--password` 가 실릴 수 있어, 인자에서 온 문자열을
                // 그대로 찍으면 비밀번호가 로그에 남는다 (CodeQL: cleartext logging).
                let opt: &'static str = if args[i] == "--from" {
                    "--from"
                } else {
                    "--to"
                };
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {opt} 뒤에 쪽 번호가 필요합니다.");
                    return EXIT_USAGE;
                };
                let Ok(n) = v.parse::<u32>() else {
                    eprintln!("오류: {opt} 값이 숫자가 아닙니다.");
                    return EXIT_USAGE;
                };
                if opt == "--from" {
                    from = Some(n)
                } else {
                    to = Some(n)
                }
            }
            "-o" | "--output" => {
                i += 1;
                output = args.get(i).map(|s| s.as_str());
            }
            "--json" => json_mode = true,
            v if v.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {v}");
                return EXIT_USAGE;
            }
            v => {
                if input.is_none() {
                    input = Some(v)
                } else if output.is_none() {
                    output = Some(v)
                }
            }
        }
        i += 1;
    }

    let (Some(input), Some(output)) = (input, output) else {
        eprintln!("사용법: rhwp extract-pages <입력> <출력.hwp> --from N --to M [--json]");
        return EXIT_USAGE;
    };
    let from = from.unwrap_or(1);
    let Some(to) = to else {
        eprintln!("오류: --to 가 필요합니다.");
        return EXIT_USAGE;
    };

    let data = match fs::read(input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {input}: {e}");
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };
    let report = match doc.extract_page_range(from, to) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("오류: 쪽 추출 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let bytes = match doc.export_hwp_with_adapter() {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: HWP 직렬화 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    if let Err(e) = fs::write(output, &bytes) {
        eprintln!("오류: 출력 쓰기 실패 - {output}: {e}");
        return EXIT_RUNTIME;
    }

    if json_mode {
        println!(
            "{}",
            provenance::marked(
                serde_json::json!({
                    "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                    "source": input,
                    "output": output,
                    "from": from,
                    "to": to,
                    "pagesBefore": report.pages_before,
                    "pagesAfter": report.pages_after,
                    "paragraphsKept": report.kept,
                    "paragraphsRemoved": report.removed,
                }),
                "extract-pages",
            )
        );
    } else {
        println!(
            "추출 완료: {output} ({}~{}쪽) — {}쪽 → {}쪽, 문단 {}개 남기고 {}개 제거",
            from, to, report.pages_before, report.pages_after, report.kept, report.removed
        );
    }
    EXIT_OK
}

pub(crate) fn convert_hwp(args: &[String]) -> i32 {
    let (positionals, verify_options) = match parse_conversion_verify_args(
        args,
        "rhwp convert <입력.hwp|입력.hwpx> <출력.hwp> [--verify] [--verify-pages] [--json]",
        2,
        2,
        true,
    ) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{}", message);
            return EXIT_USAGE;
        }
    };

    let input_path = &positionals[0];
    let output_path = &positionals[1];

    // [#4586] `convert`는 편집 가능한 HWP5를 만드는 명령이다. 출력 이름만
    // `.hwpx`로 주면 HWP5 바이트가 HWPX처럼 보이고, 후속 도구가 확장자만 믿을 때
    // 거짓 양성이 된다. 입력 IO보다 먼저 출력 계약을 판정해 잘못된 산출물을 쓰지 않는다.
    let output_is_hwp = std::path::Path::new(output_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("hwp"));
    if !output_is_hwp {
        eprintln!("오류: convert 출력 경로는 .hwp 확장자여야 합니다: {output_path}");
        eprintln!("HWPX로 변환하려면 `rhwp export-hwpx <입력> <출력.hwpx>`를 사용하세요.");
        return EXIT_USAGE;
    }

    // 입력 파일 읽기
    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", input_path, e);
            return EXIT_RUNTIME;
        }
    };
    // [#3505] --verify 비교 강도를 정하려면 원본 포맷을 알아야 한다 (대상은 항상 HWP5).
    let source_format = rhwp::parser::detect_format(&data);

    // 문서 로드
    let mut doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    let page_count_before = if verify_options.verify_pages {
        Some(doc.page_count())
    } else {
        None
    };
    let json_mode = verify_options.json;
    let output_password = cli_output_password();
    let was_distribution = doc.document().header.distribution;
    if !was_distribution && !json_mode {
        println!("{}: 이미 편집 가능한 문서입니다.", input_path);
    }

    // 변환
    match doc.convert_to_editable_native() {
        Ok(_) => {
            if was_distribution && !json_mode {
                println!("배포용 → 편집 가능 변환 완료");
            }
        }
        Err(e) => {
            eprintln!("오류: 변환 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    }

    // 직렬화
    // [#3605] JSON 봉투 — export-hwpx(#3596)와 같은 "판정은 데이터" 규약.
    let emit_envelope =
        |bytes_len: usize, verify: serde_json::Value, verify_pages: serde_json::Value| {
            println!(
                "{}",
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                        "source": input_path,
                        "output": output_path,
                        "format": "hwp5",
                        "bytes": bytes_len,
                        "wasDistribution": was_distribution,
                        "passwordProtected": output_password.is_some(),
                        "verify": verify,
                        "verifyPages": verify_pages,
                    }),
                    "convert",
                )
            );
        };
    let serialized = match output_password.as_deref() {
        Some(password) => doc.export_hwp_with_adapter_with_password(password.as_bytes()),
        None => doc.export_hwp_with_adapter(),
    };
    match serialized {
        Ok(bytes) => match fs::write(output_path, &bytes) {
            Ok(_) => {
                if !json_mode {
                    println!("저장 완료: {} ({}KB)", output_path, bytes.len() / 1024);
                }
                let mut verify_report = serde_json::Value::Null;
                let mut verify_pages_report = serde_json::Value::Null;
                let mut exit_code = EXIT_OK;
                if verify_options.enabled() {
                    let reloaded = match output_password.as_deref() {
                        Some(password) => rhwp::wasm_api::HwpDocument::from_bytes_with_password(
                            &bytes,
                            password.as_bytes(),
                        ),
                        None => rhwp::wasm_api::HwpDocument::from_bytes(&bytes),
                    };
                    let reloaded = match reloaded {
                        Ok(d) => d,
                        Err(e) => {
                            eprintln!("검증 실패: 저장된 HWP 재파싱 실패 - {}", e);
                            process::exit(verify_reparse_failed_exit_code(verify_options));
                        }
                    };

                    if let Some(before) = page_count_before {
                        let after = reloaded.page_count();
                        if before != after {
                            eprintln!(
                                "검증 실패(--verify-pages): 변환 전 {}쪽, 재파싱 후 {}쪽",
                                before, after
                            );
                            verify_pages_report = serde_json::json!({
                                "before": before, "after": after, "identical": false,
                            });
                            // [#3915] 여기서 곧장 종료하면 `--verify` 를 함께 준 경우 IR
                            // 비교가 아예 돌지 않아 **IR 차이가 있어도 보고되지 않는다.**
                            // 쪽수와 IR 은 서로 다른 결함을 재므로, 한쪽이 실패해도 다른
                            // 쪽을 마저 재고 함께 보고한다. 종료 코드는 종전대로 쪽수
                            // 실패를 우선한다(4) — 계약 무변경.
                            exit_code = verification_exit_code(true, exit_code == 3);
                        } else {
                            if !json_mode {
                                println!("검증 통과(--verify-pages): {}쪽", before);
                            }
                            verify_pages_report = serde_json::json!({
                                "before": before, "after": after, "identical": true,
                            });
                        }
                    }

                    if verify_options.verify {
                        let diff = rhwp::serializer::hwpx::roundtrip::diff_documents(
                            doc.document(),
                            reloaded.document(),
                        );
                        // [#3505, #3930] 출처별로 대상 포맷에 표현 자리가 없는 항목만
                        // 걷어낸다. 같은 포맷(HWP5→HWP5) 왕복은 엄격 비교 그대로다.
                        let diff = match source_format {
                            rhwp::parser::FileFormat::Hwp => diff,
                            rhwp::parser::FileFormat::Hwpx => {
                                rhwp::serializer::hwpx::roundtrip::strip_hwpx_to_hwp_noise(diff)
                            }
                            _ => rhwp::serializer::hwpx::roundtrip::strip_cross_format_noise(diff),
                        };
                        if !diff.is_empty() {
                            print_ir_verify_failure(&diff, output_path);
                            verify_report = serde_json::json!({
                                "identical": false, "diffCount": diff.differences.len(),
                            });
                            // [#3915] 쪽수 실패(4)가 이미 잡혔으면 그 코드를 유지한다 —
                            // 두 축이 함께 실패해도 종전 계약대로 4 로 끝난다.
                            exit_code = verification_exit_code(exit_code == 4, true);
                        } else {
                            if !json_mode {
                                println!("검증 통과(--verify): IR 차이 없음");
                            }
                            verify_report = serde_json::json!({
                                "identical": true, "diffCount": 0,
                            });
                        }
                    }
                }
                if json_mode {
                    emit_envelope(bytes.len(), verify_report, verify_pages_report);
                }
                if exit_code != EXIT_OK {
                    process::exit(exit_code);
                }
                EXIT_OK
            }
            Err(e) => {
                eprintln!("오류: 파일 저장 실패 - {}: {}", output_path, e);
                // [#2707] 출력 파일이 아예 안 만들어졌는데 0으로 끝나던 경로.
                EXIT_RUNTIME
            }
        },
        Err(e) => {
            eprintln!("오류: 직렬화 실패 - {}", e);
            EXIT_RUNTIME
        }
    }
}

/// `rhwp export-hwpx <입력.hwp|입력.hwpx> [출력.hwpx]` — HWP→HWPX 직접 변환 (#1868).
///
/// 파서가 포맷을 자동 감지(HWP5/HWP3/HWPX)해 `Document` IR 로 읽고
/// `export_hwpx_native()` 로 HWPX(ZIP) 직렬화한다. `convert`(배포용 해제 → .hwp 출력)와
/// 별개의 포맷 변환 명령. 출력 생략 시 입력과 같은 폴더에 `<stem>.hwpx`.
pub(crate) fn export_hwpx(args: &[String]) -> i32 {
    let (positionals, verify_options) = match parse_conversion_verify_args(
        args,
        "rhwp export-hwpx <입력.hwp|입력.hwpx> [출력.hwpx] [--verify] [--verify-pages] [--json]",
        1,
        2,
        true,
    ) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("{}", message);
            return EXIT_USAGE;
        }
    };

    let input_path = std::path::Path::new(&positionals[0]);
    let output_path = match positionals.get(1) {
        Some(p) => std::path::PathBuf::from(p),
        None => input_path.with_extension("hwpx"),
    };
    if output_path
        .extension()
        .map(|e| !e.eq_ignore_ascii_case("hwpx"))
        .unwrap_or(true)
    {
        eprintln!(
            "경고: 출력 확장자가 .hwpx 가 아닙니다: {}",
            output_path.display()
        );
    }
    if output_path == input_path {
        eprintln!("오류: 입력과 출력 경로가 같습니다. 원본을 덮어쓰지 않습니다.");
        return EXIT_USAGE;
    }

    let data = match fs::read(input_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!(
                "오류: 파일을 읽을 수 없습니다 - {}: {}",
                input_path.display(),
                e
            );
            return EXIT_RUNTIME;
        }
    };
    let source_format = rhwp::parser::detect_format(&data);

    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    let page_count_before = if verify_options.verify_pages {
        Some(doc.page_count())
    } else {
        None
    };

    // [#3596] JSON 봉투: 판정(verify/verifyPages)까지 채운 뒤 한 번에 낸다.
    // 종료 코드 계약(0/1/3/4)은 무변경 — 차이가 검출되어도 봉투를 stdout 에 내고
    // exit 3/4 로 끝난다(ir-diff --json 과 같은 "판정은 데이터" 규약).
    let json_mode = verify_options.json;
    let output_password = cli_output_password();
    let emit_envelope =
        |bytes_len: usize, verify: serde_json::Value, verify_pages: serde_json::Value| {
            println!(
                "{}",
                provenance::marked(
                    serde_json::json!({
                        "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                        "source": positionals[0],
                        "output": output_path.display().to_string(),
                        "format": "hwpx",
                        "bytes": bytes_len,
                        "passwordProtected": output_password.is_some(),
                        "verify": verify,
                        "verifyPages": verify_pages,
                    }),
                    "export-hwpx",
                )
            );
        };

    let serialized = match output_password.as_deref() {
        Some(password) => doc.export_hwpx_native_with_password(password.as_bytes()),
        None => doc.export_hwpx_native(),
    };
    match serialized {
        Ok(bytes) => match fs::write(&output_path, &bytes) {
            Ok(_) => {
                if !json_mode {
                    println!(
                        "저장 완료: {} ({}KB)",
                        output_path.display(),
                        bytes.len() / 1024
                    );
                }
                let mut verify_report = serde_json::Value::Null;
                let mut verify_pages_report = serde_json::Value::Null;
                let mut exit_code = EXIT_OK;
                if verify_options.enabled() {
                    let reloaded = match output_password.as_deref() {
                        Some(password) => rhwp::wasm_api::HwpDocument::from_bytes_with_password(
                            &bytes,
                            password.as_bytes(),
                        ),
                        None => rhwp::wasm_api::HwpDocument::from_bytes(&bytes),
                    };
                    let reloaded = match reloaded {
                        Ok(d) => d,
                        Err(e) => {
                            // 재파싱 실패는 판정 불가 — JSON 모드에서도 stdout 을 비운다.
                            eprintln!("검증 실패: 저장된 HWPX 재파싱 실패 - {}", e);
                            process::exit(verify_reparse_failed_exit_code(verify_options));
                        }
                    };

                    if let Some(before) = page_count_before {
                        let after = reloaded.page_count();
                        if before != after {
                            eprintln!(
                                "검증 실패(--verify-pages): 변환 전 {}쪽, 재파싱 후 {}쪽",
                                before, after
                            );
                            verify_pages_report = serde_json::json!({
                                "before": before, "after": after, "identical": false,
                            });
                            // [#3915] 여기서 곧장 종료하면 `--verify` 를 함께 준 경우 IR
                            // 비교가 아예 돌지 않아 **IR 차이가 있어도 보고되지 않는다.**
                            // 쪽수와 IR 은 서로 다른 결함을 재므로, 한쪽이 실패해도 다른
                            // 쪽을 마저 재고 함께 보고한다. 종료 코드는 종전대로 쪽수
                            // 실패를 우선한다(4) — 계약 무변경.
                            exit_code = verification_exit_code(true, exit_code == 3);
                        } else {
                            if !json_mode {
                                println!("검증 통과(--verify-pages): {}쪽", before);
                            }
                            verify_pages_report = serde_json::json!({
                                "before": before, "after": after, "identical": true,
                            });
                        }
                    }

                    if verify_options.verify {
                        let diff = rhwp::serializer::hwpx::roundtrip::diff_documents(
                            doc.document(),
                            reloaded.document(),
                        );
                        // HWP 계열은 HWPX와 표현 자리가 다른 필드 메타데이터가 있고,
                        // HWP3에는 하이퍼텍스트·빈 그림 imgRect의 추가 정규화가 있다.
                        // 실제 내용·배치 차이는 계속 검출한다 (#3739).
                        let diff = match source_format {
                            rhwp::parser::FileFormat::Hwp => {
                                rhwp::serializer::hwpx::roundtrip::strip_hwp_to_hwpx_noise(diff)
                            }
                            rhwp::parser::FileFormat::Hwp3 => {
                                rhwp::serializer::hwpx::roundtrip::strip_hwp3_to_hwpx_noise(
                                    doc.document(),
                                    reloaded.document(),
                                    diff,
                                )
                            }
                            _ => diff,
                        };
                        if !diff.is_empty() {
                            print_ir_verify_failure(&diff, &output_path.display().to_string());
                            verify_report = serde_json::json!({
                                "identical": false, "diffCount": diff.differences.len(),
                            });
                            // [#3915] 쪽수 실패(4)가 이미 잡혔으면 그 코드를 유지한다 —
                            // 두 축이 함께 실패해도 종전 계약대로 4 로 끝난다.
                            exit_code = verification_exit_code(exit_code == 4, true);
                        } else {
                            if !json_mode {
                                println!("검증 통과(--verify): IR 차이 없음");
                            }
                            verify_report = serde_json::json!({
                                "identical": true, "diffCount": 0,
                            });
                        }
                    }
                }
                if json_mode {
                    emit_envelope(bytes.len(), verify_report, verify_pages_report);
                }
                if exit_code != EXIT_OK {
                    process::exit(exit_code);
                }
                EXIT_OK
            }
            Err(e) => {
                eprintln!("오류: 파일 저장 실패 - {}: {}", output_path.display(), e);
                // [#2707] 출력 파일이 아예 안 만들어졌는데 0으로 끝나던 경로.
                EXIT_RUNTIME
            }
        },
        Err(e) => {
            eprintln!("오류: HWPX 직렬화 실패 - {}", e);
            EXIT_RUNTIME
        }
    }
}

struct HmlExportArgs {
    input: std::path::PathBuf,
    output: std::path::PathBuf,
    /// [#3616] 봉투를 stdout 순수 JSON 으로.
    json: bool,
}

fn parse_hml_export_args(args: &[String]) -> Result<HmlExportArgs, String> {
    let usage = "rhwp export-hml <입력.hml> -o <출력.hml> [--json]";
    let mut input = None;
    let mut output = None;
    let mut json = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--json" => {
                json = true;
                index += 1;
            }
            "-o" | "--output" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| format!("출력 경로가 필요합니다\n사용법: {usage}"))?;
                if value.starts_with('-') {
                    return Err(format!("출력 경로가 필요합니다\n사용법: {usage}"));
                }
                if output.replace(std::path::PathBuf::from(value)).is_some() {
                    return Err(format!("출력 경로를 한 번만 지정하세요\n사용법: {usage}"));
                }
                index += 2;
            }
            value if value.starts_with('-') => {
                return Err(format!("알 수 없는 옵션: {value}\n사용법: {usage}"));
            }
            value => {
                if input.replace(std::path::PathBuf::from(value)).is_some() {
                    return Err(format!("입력 파일을 하나만 지정하세요\n사용법: {usage}"));
                }
                index += 1;
            }
        }
    }
    Ok(HmlExportArgs {
        json,
        input: input.ok_or_else(|| format!("입력 파일이 필요합니다\n사용법: {usage}"))?,
        output: output.ok_or_else(|| format!("출력 경로가 필요합니다\n사용법: {usage}"))?,
    })
}

fn print_hml_export_error(error: &rhwp::serializer::hml::HmlExportError) {
    eprintln!("오류: {error}");
    for blocker in error.blockers() {
        eprintln!(
            "  [{}] {}: {}",
            blocker.code, blocker.xml_path, blocker.message
        );
    }
}

pub(crate) fn export_hml(args: &[String]) {
    let paths = parse_hml_export_args(args).unwrap_or_else(|message| {
        eprintln!("{message}");
        process::exit(2);
    });
    if paths_refer_to_same_file(&paths.input, &paths.output) {
        eprintln!("오류: 입력과 출력 경로가 같습니다. 원본을 덮어쓰지 않습니다.");
        process::exit(2);
    }
    let data = fs::read(&paths.input).unwrap_or_else(|error| {
        eprintln!(
            "오류: 파일을 읽을 수 없습니다 - {}: {error}",
            paths.input.display()
        );
        process::exit(1);
    });
    let core = match load_document_core(&data) {
        Ok(c) => c,
        Err(e) => process::exit(e.report()),
    };
    let bytes = core.export_hml_native().unwrap_or_else(|error| {
        print_hml_export_error(&error);
        process::exit(1);
    });
    atomic_file::write_atomically(&paths.output, &bytes).unwrap_or_else(|error| {
        eprintln!("오류: 파일 저장 실패 - {}: {error}", paths.output.display());
        process::exit(1);
    });
    if paths.json {
        println!(
            "{}",
            provenance::marked(
                serde_json::json!({
                    "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                    "source": paths.input.display().to_string(),
                    "output": paths.output.display().to_string(),
                    "format": "hml",
                    "bytes": bytes.len(),
                }),
                "export-hml",
            )
        );
    } else {
        println!(
            "저장 완료: {} ({}KB)",
            paths.output.display(),
            bytes.len() / 1024
        );
    }
}
