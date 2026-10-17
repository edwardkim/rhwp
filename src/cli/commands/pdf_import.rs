//! PDF 선택 쪽 가져오기. 파싱/IR/배치는 공통 Core, 파일 교체는 atomic_file이 소유한다.

use std::path::Path;

use crate::{EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

struct ImportArgs<'a> {
    input: &'a str,
    output: &'a str,
    // 범위를 펼치기 전에 문법·순서를 검사하고 frontend 한도로 확장량을 제한한다.
    ranges: Option<Vec<(usize, usize)>>,
    json: bool,
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
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--source-pages" if ranges.is_none() => {
                ranges = Some(parse_ranges(
                    iter.next().ok_or("--source-pages 값이 필요합니다.")?,
                )?);
            }
            "--json" if !json => json = true,
            option if option.starts_with('-') => return Err("알 수 없거나 중복된 옵션입니다."),
            path => positionals.push(path),
        }
    }
    if positionals.len() != 2 {
        return Err("사용법: rhwp import-pdf <입력.pdf> <출력.hwpx|출력.hwp> [--source-pages 0,6-7] [--json]");
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
    if crate::cli_password().is_some() || crate::cli_output_password().is_some() {
        return Err("import-pdf는 암호 옵션을 지원하지 않습니다. 암호화 PDF는 가져올 수 없으며, 출력 암호는 변환 후 convert/export-hwpx에서 지정하세요.");
    }
    Ok(ImportArgs {
        input,
        output,
        ranges,
        json,
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

#[cfg(not(feature = "pdf-import"))]
fn execute(options: ImportArgs<'_>) -> Result<(), (i32, String)> {
    let _ = (options.input, options.output, options.ranges, options.json);
    Err((EXIT_RUNTIME, "PDF_IMPORT_DISABLED: 이 빌드는 PDF 가져오기가 제외되었습니다. cargo build --features pdf-import --bin rhwp 로 빌드하세요.".into()))
}

#[cfg(feature = "pdf-import")]
fn execute(options: ImportArgs<'_>) -> Result<(), (i32, String)> {
    use std::io::Read;

    use rhwp::document_core::DocumentCore;
    use rhwp::parser::pdf::{PdfImportOptions, MAX_INPUT_BYTES, MAX_PAGES};

    let runtime = |error: String| (EXIT_RUNTIME, error);
    let source_pages = options
        .ranges
        .map(|ranges| {
            let mut pages = Vec::new();
            for (start, end) in ranges {
                let count = end.checked_sub(start).and_then(|n| n.checked_add(1));
                if count.is_none_or(|count| count > MAX_PAGES - pages.len()) {
                    return Err((
                        EXIT_USAGE,
                        format!("PDF_PAGE_SELECTION: 선택 쪽은 최대 {MAX_PAGES}개입니다."),
                    ));
                }
                pages.extend(start..=end);
            }
            Ok(pages)
        })
        .transpose()?;

    // frontend의 입력 한도보다 한 바이트만 더 읽어 과대 입력 오류를 동일 경로로 낸다.
    let input = std::fs::File::open(options.input).map_err(|e| runtime(e.to_string()))?;
    let mut bytes = Vec::new();
    input
        .take(MAX_INPUT_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| runtime(e.to_string()))?;
    let document = DocumentCore::from_pdf_bytes(&bytes, &PdfImportOptions { source_pages })
        .map_err(|error| {
            // Core의 기존 InvalidFile 래핑에서 안정 PDF code만 분류한다. 문서 문자열이나
            // 번역된 설명의 부분 일치로 모든 parse 실패를 사용법 오류로 바꾸지 않는다.
            let selection = matches!(&error, rhwp::error::HwpError::InvalidFile(message)
                if message.starts_with("PDF 오류: PDF_PAGE_SELECTION:"));
            (
                if selection { EXIT_USAGE } else { EXIT_RUNTIME },
                error.to_string(),
            )
        })?;
    let hwpx = Path::new(options.output)
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("hwpx"));
    let output = if hwpx {
        document.export_hwpx_native()
    } else {
        document.export_hwp_native()
    }
    .map_err(|e| runtime(e.to_string()))?;
    let envelope = rhwp::provenance::marked(
        serde_json::json!({
            "schemaVersion": rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION,
            "source": options.input,
            "output": options.output,
            "format": if hwpx { "hwpx" } else { "hwp5" },
            "bytes": output.len(),
            "pageCount": document.page_count(),
            "diagnostics": document.pdf_metadata(),
        }),
        "import-pdf",
    );
    crate::atomic_file::write_atomically(Path::new(options.output), &output)
        .map_err(|e| runtime(e.to_string()))?;
    if options.json {
        println!("{envelope}");
    } else {
        eprintln!(
            "저장 완료: {} ({} bytes, {}쪽)",
            options.output,
            output.len(),
            document.page_count()
        );
    }
    Ok(())
}
