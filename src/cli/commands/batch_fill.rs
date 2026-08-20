//! State-changing batch form-fill command adapter.

use std::fs;
use std::path::Path;

use crate::cli::batch::{fail_record, ordered::stream_records};
use crate::cli::commands::edit::{fill_fields_core, runtime::edit_output_format};
use crate::{EXIT_RUNTIME, EXIT_USAGE};

/// 데이터 파일의 한 행. 읽지 못한 행도 **버리지 않고** 들고 간다 — 스트림에서 조용히
/// 사라지면 소비자는 N행을 넣고 N-1건을 받고도 그것을 성공으로 읽는다.
enum FillRow {
    Data(serde_json::Map<String, serde_json::Value>),
    /// 이 행을 읽지 못한 사유. 그대로 `error` 레코드가 된다.
    Broken(String),
}

/// [#3719 §6-6] `batch fill` — 서식 하나에 데이터 N행을 채워 산출 N개를 만든다.
///
/// 다른 batch 축과 **입력 축이 다르다**: stdin 은 읽지 않고, `--data` 파일의 한 행이
/// 산출물 하나가 된다(기존 축의 입력은 '경로'지만 여기서는 '행'이다). 채움 자체는 단건
/// `edit fill-fields` 와 같은 `fill_fields_core` 를 행마다 부를 뿐 — 새 편집 로직은 없다.
pub(crate) fn run(args: &[String]) -> i32 {
    use std::io::Write;

    const USAGE: &str = "사용법: rhwp batch fill --form <서식.hwp|서식.hwpx> --data <행.jsonl|행.csv> --out-dir <폴더> --json [--name-field <필드>] [--verify] [--dry-run] [--threads <N>]\n      데이터는 stdin 이 아니라 --data 파일이다 — 다른 batch 축은 stdin 으로 파일 경로 목록을 받지만 fill 의 입력은 경로가 아니라 '행'이다.";

    let mut form: Option<&str> = None;
    let mut data_path: Option<&str> = None;
    let mut out_dir: Option<std::path::PathBuf> = None;
    let mut name_field: Option<&str> = None;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut dry_run = false;
    let mut threads_opt: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        // 옵션 이름은 리터럴로 고정한다 — 인자에서 온 문자열을 그대로 찍으면 CodeQL
        // cleartext-logging 대상이 된다(batch convert 의 --verify 와 같은 규약).
        let opt: &'static str = match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
                continue;
            }
            "--verify" => {
                verify_mode = true;
                i += 1;
                continue;
            }
            "--dry-run" => {
                dry_run = true;
                i += 1;
                continue;
            }
            "--form" => "--form",
            "--data" => "--data",
            "--out-dir" => "--out-dir",
            "--name-field" => "--name-field",
            "--threads" => "--threads",
            other => {
                eprintln!("알 수 없는 옵션: {}", other);
                eprintln!("{USAGE}");
                return EXIT_USAGE;
            }
        };
        let Some(value) = args.get(i + 1) else {
            eprintln!("오류: {opt} 뒤에 값이 필요합니다.");
            eprintln!("{USAGE}");
            return EXIT_USAGE;
        };
        // 값 자리에 플래그가 오면 삼키지 않는다 — 삼키면 "지정했다고 믿는 옵션이 실제로는
        // 없는" 채로 실행돼 산출물이 엉뚱한 곳에 생긴다.
        if value.is_empty() || value.starts_with('-') {
            eprintln!(
                "오류: {opt} 뒤에 플래그가 아닌 값이 필요합니다 (이름이 - 로 시작하면 ./ 를 붙이세요)."
            );
            return EXIT_USAGE;
        }
        match opt {
            "--form" => form = Some(value),
            "--data" => data_path = Some(value),
            "--out-dir" => out_dir = Some(std::path::PathBuf::from(value)),
            "--name-field" => name_field = Some(value),
            _ => match value.parse::<usize>() {
                Ok(n) if n >= 1 => threads_opt = Some(n),
                _ => {
                    eprintln!("오류: 스레드 수가 올바르지 않습니다 - {}", value);
                    return EXIT_USAGE;
                }
            },
        }
        i += 2;
    }

    if !json_mode {
        eprintln!("오류: batch 는 현재 --json 출력만 지원합니다.");
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    }
    // `--dry-run` 에서도 --out-dir 를 요구한다. 선검증은 **실행과 같은 명령줄에서 --dry-run
    // 하나만 빼면 되는 것**이라야 뜻이 있다 — 인자 모양이 다르면 선검증이 통과한 명령과
    // 실제로 실행하는 명령이 서로 다른 명령이 된다.
    let (Some(form), Some(data_path), Some(out_dir)) = (form, data_path, out_dir.as_deref()) else {
        eprintln!(
            "오류: batch fill 은 --form <서식> --data <행 파일> --out-dir <폴더> 가 모두 필요합니다."
        );
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };

    // 서식은 행마다 다시 열린다. 못 여는 서식이면 N행을 다 돌고 같은 실패를 N번 보고하게
    // 되므로 — 그건 진단이 아니다 — 한 행을 처리하기 전에 여기서 한 번 판정한다.
    let form_bytes = match fs::read(form) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("오류: 서식을 읽을 수 없습니다 - {}: {}", form, e);
            return EXIT_RUNTIME;
        }
    };
    if let Err(e) = rhwp::wasm_api::HwpDocument::from_bytes(&form_bytes) {
        eprintln!("오류: 서식 HWP 파싱 실패 - {}: {}", form, e);
        return EXIT_RUNTIME;
    }
    // [#3383] 산출 형식은 서식 형식을 따른다 — 파일 이름의 확장자도 여기서 정해진다.
    let out_format = edit_output_format(&form_bytes, None);

    let rows = match read_fill_rows(Path::new(data_path)) {
        Ok(r) => r,
        Err((message, code)) => {
            eprintln!("오류: {message}");
            if code == EXIT_USAGE {
                eprintln!("{USAGE}");
            }
            return code;
        }
    };

    // 산출 경로는 **한 행도 쓰기 전에** 전부 정한다 — 병렬 실행에서도 이름이 행 순서만으로
    // 결정되고, 이름 충돌 해소가 실행 순서에 좌우되지 않는다.
    let outputs = batch_fill_output_paths(&rows, out_dir, name_field, out_format.ext());
    if !dry_run {
        if let Err(e) = fs::create_dir_all(out_dir) {
            eprintln!(
                "오류: 출력 폴더를 만들 수 없습니다 - {}: {}",
                out_dir.display(),
                e
            );
            return EXIT_RUNTIME;
        }
    }

    let threads = threads_opt
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
        })
        .max(1);

    let started = std::time::Instant::now();
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());

    let tally = stream_records(
        rows.len(),
        threads,
        |idx| {
            batch_fill_record(
                form,
                idx,
                &rows[idx],
                outputs[idx].as_deref(),
                dry_run,
                verify_mode,
            )
        },
        &mut out,
    );

    if tally.aborted {
        return EXIT_RUNTIME;
    }
    if let Err(e) = out.flush() {
        eprintln!("오류: stdout 쓰기 실패 - {}", e);
        return EXIT_RUNTIME;
    }

    eprintln!(
        "batch fill: {}행 중 {} 성공, {} 실패 ({}ms, threads={}{})",
        tally.emitted,
        tally.emitted - tally.failed,
        tally.failed,
        started.elapsed().as_millis(),
        threads,
        if dry_run { ", dry-run" } else { "" }
    );
    if tally.verify_diff > 0 {
        eprintln!(
            "batch fill: 검증 판정 — verify 차이 {}건 (채움·저장 자체는 성공)",
            tally.verify_diff
        );
    }
    tally.exit_code()
}

/// [#3719 §6-6] 행 하나 → NDJSON 레코드 하나. 실패도 레코드다(스트림은 계속된다).
///
/// 한 행의 파서 panic 이 메일머지 전체를 죽여서는 안 된다 — 기존 `batch_record` 와 같은
/// 격리 규약이다.
fn batch_fill_record(
    form: &str,
    row_index: usize,
    row: &FillRow,
    output: Option<&Path>,
    dry_run: bool,
    verify_mode: bool,
) -> serde_json::Value {
    let mut record = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        batch_fill_record_inner(form, row, output, dry_run, verify_mode)
    })) {
        Ok(record) => record,
        Err(payload) => {
            let message = payload
                .downcast_ref::<&str>()
                .map(|s| (*s).to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "원인 불명".to_string());
            fail_record(form, format!("내부 오류(panic): {}", message))
        }
    };
    // 행 번호는 성공·실패 어느 쪽에도 붙는다. 없으면 어느 행이 빠졌는지 셀 수 없어
    // 스트림 전체가 감사 불가가 된다.
    record["row"] = serde_json::json!(row_index);
    record
}

fn batch_fill_record_inner(
    form: &str,
    row: &FillRow,
    output: Option<&Path>,
    dry_run: bool,
    verify_mode: bool,
) -> serde_json::Value {
    let data = match row {
        FillRow::Data(map) => map,
        FillRow::Broken(reason) => return fail_record(form, reason.clone()),
    };
    let Some(output) = output else {
        return fail_record(form, "산출 경로를 정하지 못했습니다".to_string());
    };
    let output_path = output.to_string_lossy().to_string();
    match fill_fields_core(form, data, Some(output_path.clone()), dry_run, verify_mode) {
        Ok(outcome) => {
            let mut record = outcome.envelope;
            if dry_run {
                // 선검증에도 목적지를 밝힌다. 같은 봉투에 `dryRun: true` 가 함께 있으므로
                // "만들 예정" 경로임이 레코드 안에서 구분된다(디스크에 파일은 없다).
                record["output"] = serde_json::Value::String(output_path);
                record["outputFormat"] =
                    serde_json::Value::String(outcome.output_format.label().to_string());
            }
            record
        }
        Err(message) => fail_record(form, message),
    }
}

/// [#3719 §6-6] 행마다 산출 파일 경로를 정한다.
///
/// 이름은 `--name-field` 값, 없으면 1 기준 순번이다. 파일명에 쓸 수 없는 문자는 치환하고,
/// 서로 다른 행이 같은 이름을 내면 뒤에 순번을 붙인다 — 덮어쓰면 앞 행의 산출물이
/// **조용히** 사라져서 성공 레코드 N건과 실제 파일 수가 어긋난다.
fn batch_fill_output_paths(
    rows: &[FillRow],
    out_dir: &Path,
    name_field: Option<&str>,
    ext: &str,
) -> Vec<Option<std::path::PathBuf>> {
    // 대소문자만 다른 이름도 한 파일이 되는 파일시스템(Windows·macOS 기본)이 있다.
    // batch convert 와 같은 보수적 규약으로 소문자 키 하나로 판정해야, OS 를 바꾼
    // 재실행이 달라지지 않는다.
    let mut taken: std::collections::HashSet<String> = std::collections::HashSet::new();
    let width = rows.len().to_string().len().max(4);
    let mut paths = Vec::with_capacity(rows.len());
    for (idx, row) in rows.iter().enumerate() {
        let FillRow::Data(map) = row else {
            // 읽지 못한 행은 산출물이 없다 — 이름도 잡지 않는다.
            paths.push(None);
            continue;
        };
        let seq = format!("{:0width$}", idx + 1, width = width);
        let base = name_field
            .and_then(|f| map.get(f))
            .map(|v| match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            })
            .map(|s| sanitize_output_stem(&s))
            .filter(|s| !s.is_empty())
            // 이름 필드가 비었거나 치환 후 아무것도 남지 않으면 순번으로 되돌린다.
            .unwrap_or_else(|| seq.clone());

        let mut candidate = base.clone();
        let mut dup = 1usize;
        while !taken.insert(format!("{}.{}", candidate.to_lowercase(), ext)) {
            dup += 1;
            candidate = format!("{base}_{dup}");
        }
        paths.push(Some(out_dir.join(format!("{candidate}.{ext}"))));
    }
    paths
}

/// [#3719 §6-6] 데이터 값에서 파일 이름을 만든다 — 데이터에서 온 문자열이 경로 문법을
/// 타지 못하게 한다.
///
/// 경로 구분자·Windows 금지 문자·제어 문자는 `_` 로 바꾼다. 구분자가 사라지므로
/// `../..` 같은 값도 `--out-dir` 밖으로 나갈 수 없다. Windows 는 이름 끝의 공백·점을
/// 조용히 잘라내므로 미리 없애고, 예약 장치 이름(CON·NUL·COM1…)은 앞에 `_` 를 붙여 피한다.
fn sanitize_output_stem(raw: &str) -> String {
    /// 경로 길이 한도(Windows 260자)에 여유를 두는 이름 길이 상한.
    const MAX_CHARS: usize = 80;
    const RESERVED: [&str; 22] = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    let mut stem = String::new();
    for ch in raw.chars().take(MAX_CHARS) {
        let forbidden =
            matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || ch.is_control();
        stem.push(if forbidden { '_' } else { ch });
    }
    let trimmed = stem.trim().trim_end_matches(['.', ' ']).trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let head = trimmed.split('.').next().unwrap_or("").to_ascii_uppercase();
    if RESERVED.contains(&head.as_str()) {
        format!("_{trimmed}")
    } else {
        trimmed.to_string()
    }
}

/// [#3719 §6-6] `--data` 파일 → 행 목록.
///
/// `Err((사유, 종료 코드))` 는 **한 행도 처리하기 전에** 끝낼 입력 오류다(확장자·헤더·
/// 빈 파일). 개별 행의 결함은 여기서 끝내지 않고 `FillRow::Broken` 으로 스트림에 남긴다 —
/// 한 행이 깨졌다고 나머지 N-1행의 산출물을 포기할 이유가 없다.
fn read_fill_rows(path: &Path) -> Result<Vec<FillRow>, (String, i32)> {
    let text = fs::read_to_string(path).map_err(|e| {
        (
            format!("--data 파일을 읽을 수 없습니다 - {}: {}", path.display(), e),
            EXIT_RUNTIME,
        )
    })?;
    // 엑셀이 저장한 CSV 는 UTF-8 BOM 으로 시작한다. 남겨 두면 첫 헤더 이름이 통째로
    // 어긋나(BOM+이름) 문서의 누름틀과 영영 매칭되지 않는다.
    let text: &str = text.strip_prefix('\u{feff}').unwrap_or(&text);

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    let rows = match ext.as_str() {
        "jsonl" | "ndjson" => parse_jsonl_rows(text),
        "csv" => parse_csv_rows(text)?,
        "" => {
            return Err((
                "--data 파일에 확장자가 없습니다 — .jsonl 또는 .csv 로 지정하세요.".to_string(),
                EXIT_USAGE,
            ));
        }
        other => {
            return Err((
                format!("--data 는 .jsonl 또는 .csv 여야 합니다 - .{other}"),
                EXIT_USAGE,
            ));
        }
    };
    if rows.is_empty() {
        // 0행을 성공(exit 0)으로 끝내면 "전부 처리했다"와 구분되지 않는다.
        return Err((
            format!("--data 에 데이터 행이 없습니다 - {}", path.display()),
            EXIT_USAGE,
        ));
    }
    Ok(rows)
}

/// JSONL: 한 줄 한 객체. 빈 줄은 건너뛴다.
fn parse_jsonl_rows(text: &str) -> Vec<FillRow> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(
            |line| match serde_json::from_str::<serde_json::Value>(line) {
                Ok(serde_json::Value::Object(m)) => FillRow::Data(m),
                Ok(_) => FillRow::Broken(
                    "JSONL 행은 {\"필드이름\":\"값\"} 형식의 JSON 객체여야 합니다".to_string(),
                ),
                Err(e) => FillRow::Broken(format!("JSONL 행 파싱 실패 - {e}")),
            },
        )
        .collect()
}

/// CSV: 첫 줄 헤더가 누름틀 이름이다. 헤더 이름은 **공백까지 그대로** 문서의 이름으로 쓴다.
fn parse_csv_rows(text: &str) -> Result<Vec<FillRow>, (String, i32)> {
    let records = parse_csv_records(text).map_err(|e| (e, EXIT_USAGE))?;
    let mut it = records.into_iter();
    let Some(header) = it.next() else {
        return Err(("--data CSV 에 헤더 줄이 없습니다.".to_string(), EXIT_USAGE));
    };
    for (i, name) in header.iter().enumerate() {
        if name.is_empty() {
            return Err((
                format!("--data CSV 헤더 {}번째 칸의 이름이 비었습니다.", i + 1),
                EXIT_USAGE,
            ));
        }
        // 같은 이름이 두 번이면 뒤 칸이 앞 칸을 덮어 **한 열이 통째로 무시된다**.
        if header[..i].contains(name) {
            return Err((
                format!("--data CSV 헤더에 같은 이름이 두 번 있습니다 - {name}"),
                EXIT_USAGE,
            ));
        }
    }
    Ok(it
        .map(|record| {
            if record.len() != header.len() {
                // 칸 수가 다르면 값이 한 칸씩 밀려 엉뚱한 누름틀로 들어간다. 채우고 나면
                // 아무 오류 없이 잘못된 문서가 나오므로 행 단위로 거부한다.
                return FillRow::Broken(format!(
                    "CSV 칸 수가 헤더와 다릅니다 - 헤더 {}칸, 행 {}칸",
                    header.len(),
                    record.len()
                ));
            }
            FillRow::Data(
                header
                    .iter()
                    .cloned()
                    .zip(record.into_iter().map(serde_json::Value::String))
                    .collect(),
            )
        })
        .collect())
}

/// [#3719 §6-6] RFC 4180 CSV 읽기 — 엑셀 저장본을 그대로 받는다.
///
/// 따옴표 안의 쉼표·줄바꿈·이중 따옴표(`""`)를 보존하고 CRLF/LF 를 모두 줄 끝으로 읽는다.
/// 전용 crate 를 새로 들이지 않는 이유는 여기서 필요한 문법이 RFC 4180 그 자체뿐이라서다.
fn parse_csv_records(text: &str) -> Result<Vec<Vec<String>>, String> {
    let mut records: Vec<Vec<String>> = Vec::new();
    let mut record: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    field.push('"');
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }
        match c {
            // 여는 따옴표는 칸 맨 앞에서만 뜻이 있다. 칸 중간의 따옴표는 값의 일부다.
            '"' if field.is_empty() => in_quotes = true,
            ',' => record.push(std::mem::take(&mut field)),
            '\r' | '\n' => {
                if c == '\r' && chars.peek() == Some(&'\n') {
                    chars.next();
                }
                record.push(std::mem::take(&mut field));
                records.push(std::mem::take(&mut record));
            }
            _ => field.push(c),
        }
    }
    if in_quotes {
        // 여기서 멈추지 않으면 "따옴표 하나 빠뜨린 CSV"가 뒤 행 전체를 한 칸으로 삼킨다.
        return Err("--data CSV 의 따옴표가 닫히지 않았습니다.".to_string());
    }
    if !field.is_empty() || !record.is_empty() {
        record.push(field);
        records.push(record);
    }
    // 마지막 개행이 만든 빈 줄은 행이 아니다(엑셀 저장본은 늘 개행으로 끝난다).
    records.retain(|r| !(r.len() == 1 && r[0].trim().is_empty()));
    Ok(records)
}
