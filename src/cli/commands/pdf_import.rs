//! PDF 선택 쪽 가져오기. 파싱/IR/배치는 공통 Core, 파일 교체는 atomic_file이 소유한다.

use std::path::Path;

use crate::{EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

#[cfg(all(feature = "pdf-import", not(target_arch = "wasm32")))]
mod resources;
#[cfg(all(feature = "pdf-import", not(target_arch = "wasm32")))]
use rhwp::resource_supervisor as supervisor;
#[cfg(all(feature = "pdf-import", not(target_arch = "wasm32")))]
mod worker;

struct ImportArgs<'a> {
    input: &'a str,
    output: &'a str,
    // 범위를 펼치기 전에 문법·순서를 검사하고 frontend 한도로 확장량을 제한한다.
    ranges: Option<Vec<(usize, usize)>>,
    json: bool,
    table_inference: bool,
    layout: Option<&'a str>,
    resource_profile: &'a str,
    resource_limits: Option<&'a str>,
}

fn parse_ranges(value: &str) -> Result<Vec<(usize, usize)>, &'static str> {
    let invalid = "PDF_PAGE_SELECTION: --source-pages는 0부터 시작하는 증가 목록입니다 (예: 0,6-7). 중복·역순·빈 항목은 허용하지 않습니다.";
    let number = |value: &str| {
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(invalid);
        }
        value.parse::<usize>().map_err(|_| invalid)
    };
    let mut ranges = Vec::new();
    for part in value.split(',') {
        let (start, end) = match part.split_once('-') {
            Some((start, end)) => (number(start)?, number(end)?),
            None => {
                let page = number(part)?;
                (page, page)
            }
        };
        if start > end
            || ranges
                .last()
                .is_some_and(|&(_, previous)| start <= previous)
        {
            return Err(invalid);
        }
        ranges.push((start, end));
    }
    Ok(ranges)
}

fn parse_args(args: &[String]) -> Result<ImportArgs<'_>, &'static str> {
    let mut positionals = Vec::new();
    let mut ranges = None;
    let mut json = false;
    let mut table_inference = false;
    let mut layout = None;
    let mut resource_profile = None;
    let mut resource_limits = None;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--source-pages" if ranges.is_none() => {
                ranges = Some(parse_ranges(
                    iter.next().ok_or("--source-pages 값이 필요합니다.")?,
                )?);
            }
            "--json" if !json => json = true,
            "--table-inference" if !table_inference => table_inference = true,
            "--layout" if layout.is_none() => {
                layout = Some(
                    iter.next()
                        .map(String::as_str)
                        .ok_or("--layout 값이 필요합니다 (fidelity|editable).")?,
                );
            }
            "--resource-profile" if resource_profile.is_none() => {
                let profile = iter.next().ok_or("--resource-profile 값이 필요합니다.")?;
                if !matches!(profile.as_str(), "standard" | "constrained") {
                    return Err(
                        "PDF_RESOURCE_POLICY: resource profile은 standard 또는 constrained입니다.",
                    );
                }
                resource_profile = Some(profile.as_str());
            }
            "--resource-limits" if resource_limits.is_none() => {
                resource_limits = Some(
                    iter.next()
                        .ok_or("--resource-limits 파일 경로가 필요합니다.")?
                        .as_str(),
                );
            }
            option if option.starts_with('-') => return Err("알 수 없거나 중복된 옵션입니다."),
            path => positionals.push(path),
        }
    }
    if positionals.len() != 2 {
        return Err("사용법: rhwp import-pdf <입력.pdf> <출력.hwpx|출력.hwp> [--source-pages 0,6-7] [--layout fidelity|editable] [--resource-profile standard|constrained] [--resource-limits limits.json] [--json] [--table-inference]");
    }
    if layout.is_some_and(|value| !matches!(value, "fidelity" | "editable")) {
        return Err("--layout은 fidelity 또는 editable입니다.");
    }
    if table_inference && !json {
        return Err("--table-inference는 --json과 함께 지정하세요. 원본 문자가 포함된 상세 추론 근거를 출력합니다.");
    }
    let input = positionals[0];
    let output = positionals[1];
    if !Path::new(output).extension().is_some_and(|extension| {
        extension.eq_ignore_ascii_case("hwpx") || extension.eq_ignore_ascii_case("hwp")
    }) {
        return Err("출력 확장자는 .hwpx 또는 .hwp여야 합니다.");
    }
    if crate::paths_refer_to_same_file(Path::new(input), Path::new(output)) {
        return Err("입력과 출력이 같은 파일입니다. 원본을 덮어쓰지 않습니다.");
    }
    if crate::cli_output_password().is_some() {
        return Err("import-pdf의 출력 암호는 변환 후 convert/export-hwpx에서 지정하세요.");
    }
    Ok(ImportArgs {
        input,
        output,
        ranges,
        json,
        table_inference,
        layout,
        resource_profile: resource_profile.unwrap_or("standard"),
        resource_limits,
    })
}

pub(crate) fn import_pdf(args: &[String]) -> i32 {
    let options = match parse_args(args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("오류: {error}");
            return EXIT_USAGE;
        }
    };
    match execute(options) {
        Ok(()) => EXIT_OK,
        Err((code, error)) => {
            eprintln!("오류: {error}");
            code
        }
    }
}

#[cfg(any(not(feature = "pdf-import"), target_arch = "wasm32"))]
fn execute(options: ImportArgs<'_>) -> Result<(), (i32, String)> {
    let _ = (
        options.input,
        options.output,
        options.ranges,
        options.json,
        options.table_inference,
        options.layout,
        options.resource_profile,
        options.resource_limits,
    );
    if cfg!(feature = "pdf-import") {
        Err((EXIT_RUNTIME, "PDF_PROCESS_UNSUPPORTED: CLI 작업자는 Native 실행 환경이 필요합니다. WASM에서는 importPdf API를 사용하세요.".into()))
    } else {
        Err((EXIT_RUNTIME, "PDF_IMPORT_DISABLED: 이 빌드는 PDF 가져오기가 제외되었습니다. cargo build --features pdf-import --bin rhwp 로 빌드하세요.".into()))
    }
}

#[cfg(all(feature = "pdf-import", not(target_arch = "wasm32")))]
fn execute(options: ImportArgs<'_>) -> Result<(), (i32, String)> {
    let runtime = |message: &str| (EXIT_RUNTIME, message.to_owned());
    let (profile, policy, execution) =
        resources::resolve(options.resource_profile, options.resource_limits)
            .map_err(|error| (EXIT_USAGE, error))?;
    let max_pages = policy.max_selected_pages;
    let source_pages = options
        .ranges
        .map(|ranges| {
            let mut pages = Vec::new();
            for (start, end) in ranges {
                let count = end.checked_sub(start).and_then(|n| n.checked_add(1));
                if count.is_none_or(|count| count > max_pages - pages.len()) {
                    return Err((
                        EXIT_USAGE,
                        format!("PDF_PAGE_SELECTION: 선택 쪽은 최대 {max_pages}개입니다."),
                    ));
                }
                pages.extend(start..=end);
            }
            Ok(pages)
        })
        .transpose()?;
    let hwpx = Path::new(options.output)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("hwpx"));
    let mut entropy = [0u8; 32];
    getrandom::fill(&mut entropy).map_err(|_| runtime("PDF_PROCESS_START_FAILED"))?;
    let nonce: String = entropy.iter().map(|byte| format!("{byte:02x}")).collect();
    let request = serde_json::to_vec(&worker::Request {
        nonce: nonce.clone(),
        input: options.input.to_owned(),
        hwpx,
        source_pages,
        password: crate::cli_password(),
        table_inference: options.table_inference,
        layout: options.layout.unwrap_or("fidelity").to_owned(),
        policy,
        output_bytes: execution.output_bytes,
        diagnostics_bytes: execution.diagnostics_bytes,
    })
    .map_err(|_| runtime("PDF_PROCESS_PROTOCOL"))?;
    if request.len() > 64 * 1024 {
        return Err((
            EXIT_USAGE,
            "PDF_RESOURCE_POLICY: 요청 설정이 너무 큽니다.".into(),
        ));
    }
    let executable = std::env::current_exe().map_err(|_| runtime("PDF_PROCESS_START_FAILED"))?;
    let mut command = std::process::Command::new(executable);
    command.arg("__pdf-import-worker-v1");
    supervisor::configure_pdf_worker_environment(&mut command);
    command.env("RHWP_PDF_WORKER_NONCE", nonce);
    let mut result = supervisor::run(&mut command, &request, execution).map_err(runtime)?;
    if !result.status.success() {
        let message = String::from_utf8(result.stderr)
            .unwrap_or_else(|_| "PDF_PROCESS_FAILED: 잘못된 작업자 진단입니다.".into());
        let code = if result.status.code() == Some(EXIT_USAGE) {
            EXIT_USAGE
        } else {
            EXIT_RUNTIME
        };
        return Err((
            code,
            if message.is_empty() {
                "PDF_PROCESS_FAILED: PDF 작업자가 완료 전에 종료됐습니다.".into()
            } else {
                message
                    .trim()
                    .strip_prefix("오류: ")
                    .unwrap_or(message.trim())
                    .to_owned()
            },
        ));
    }
    let (metadata, output) = worker::decode_response(&result.stdout, execution).map_err(runtime)?;
    let confined = metadata.isolation.filesystem && metadata.isolation.network;
    #[cfg(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    if !confined || !metadata.isolation.landlock_abi.is_some_and(|abi| abi >= 3) {
        return Err(runtime(
            "PDF_PROCESS_PROTOCOL: worker confinement was not confirmed",
        ));
    }
    result.metrics.filesystem_network_sandbox = confined;
    // Every successful import has nonempty diagnostics.pages. Reuse the
    // provenance catalog while keeping the diagnostic itself as borrowed JSON.
    let untrusted: Vec<_> = rhwp::provenance::entry("import-pdf")
        .ok_or_else(|| runtime("PDF_PROCESS_PROTOCOL"))?
        .untrusted
        .iter()
        .map(|field| field.path)
        .collect();
    let envelope = serde_json::json!({
        "schemaVersion": rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION,
        "source": options.input,
        "output": options.output,
        "format": if hwpx { "hwpx" } else { "hwp5" },
        "bytes": output.len(),
        "pageCount": metadata.page_count,
        "resourceProfile": profile,
        "execution": result.metrics,
        "untrustedContent": !untrusted.is_empty(),
        "untrustedFields": untrusted,
    });
    crate::atomic_file::write_atomically(Path::new(options.output), output)
        .map_err(|error| runtime(&error.to_string()))?;
    if options.json {
        #[derive(serde::Serialize)]
        struct Envelope<'a> {
            #[serde(flatten)]
            fields: serde_json::Value,
            diagnostics: &'a serde_json::value::RawValue,
        }
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        serde_json::to_writer(
            &mut stdout,
            &Envelope {
                fields: envelope,
                diagnostics: metadata.diagnostics,
            },
        )
        .map_err(|_| runtime("PDF_PROCESS_IO_FAILED"))?;
        writeln!(stdout).map_err(|_| runtime("PDF_PROCESS_IO_FAILED"))?;
    } else {
        eprintln!(
            "저장 완료: {} ({} bytes, {}쪽)",
            options.output,
            output.len(),
            metadata.page_count
        );
    }
    Ok(())
}

/// Called only by the internal pipe endpoint in the supervised child.
pub(crate) fn run_worker() -> i32 {
    #[cfg(all(feature = "pdf-import", not(target_arch = "wasm32")))]
    {
        worker::main()
    }
    #[cfg(any(not(feature = "pdf-import"), target_arch = "wasm32"))]
    {
        EXIT_RUNTIME
    }
}
