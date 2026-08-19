//! State-changing table and chart import command adapters.

use std::fs;
use std::path::Path;
use std::process;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{
    edit_output_format, edit_serialize, edit_verify_report, resolve_table_cell,
    set_cell_control_char_rejection, CellResolveError, EditOutputFormat, EXIT_OK, EXIT_RUNTIME,
    EXIT_USAGE,
};

/// `csv-to-table` — CSV 내용으로 기존 표 N 의 셀을 덮어쓴다 (#3719 §7).
///
/// 표 **크기는 바꾸지 않는다**. CSV 의 행·열 수가 표와 다르면 한 칸도 쓰지 않고
/// `invalid[]` 로 보고하며 exit 2 다 — 조용히 잘라내면 "표는 그럴듯한데 뒤쪽 데이터가
/// 통째로 사라진" 보고서가 나오고, 에이전트는 렌더를 보지 않으므로 알아채지 못한다.
/// 선검증 → 인메모리 적용 → 단 한 번 저장은 `run`(#3703)의 원자 실행과 같은 규약이다.
struct CsvToTableArgs<'a> {
    file_path: &'a str,
    csv_path: String,
    table_no: usize,
    out_path: Option<String>,
    dry_run: bool,
    verify_mode: bool,
    json_mode: bool,
}

fn parse_csv_to_table_args(args: &[String]) -> Result<CsvToTableArgs<'_>, i32> {
    let mut file_path: Option<&str> = None;
    let mut csv_path: Option<String> = None;
    let mut table_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut verify_mode = false;
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--dry-run" => dry_run = true,
            "--verify" => verify_mode = true,
            "--csv" => {
                i += 1;
                match args.get(i) {
                    Some(p) => csv_path = Some(p.clone()),
                    None => {
                        eprintln!("오류: --csv 뒤에 CSV 파일 경로가 필요합니다.");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            "--table" => {
                i += 1;
                match args.get(i).map(|v| v.parse::<usize>()) {
                    Some(Ok(value)) => table_arg = Some(value),
                    _ => {
                        eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다.");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(p) => out_path = Some(p.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return Err(EXIT_USAGE);
                    }
                }
            }
            other if other.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {other}");
                return Err(EXIT_USAGE);
            }
            other => {
                if file_path.replace(other).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return Err(EXIT_USAGE);
                }
            }
        }
        i += 1;
    }

    let (Some(file_path), Some(csv_path), Some(table_no)) = (file_path, csv_path, table_arg) else {
        eprintln!(
            "사용법: rhwp csv-to-table <파일.hwp|파일.hwpx> --csv <경로.csv> --table <번호> [-o <출력>] [--dry-run] [--verify] [--json]"
        );
        return Err(EXIT_USAGE);
    };

    Ok(CsvToTableArgs {
        file_path,
        csv_path,
        table_no,
        out_path,
        dry_run,
        verify_mode,
        json_mode,
    })
}

fn validate_csv_table_records(
    records: &[Vec<String>],
    rows: u16,
    cols: u16,
    anchors: &[(u16, u16, String)],
    table_no: usize,
) -> Vec<serde_json::Value> {
    let mut invalid = Vec::new();
    if records.len() != rows as usize {
        invalid.push(serde_json::json!({
            "reason": "rowCountMismatch",
            "expected": rows,
            "actual": records.len(),
            "message": format!(
                "CSV 행 수 {} 가 표 {} 의 행 수 {} 와 다릅니다 — 표 크기는 바꾸지 않습니다.",
                records.len(), table_no, rows
            ),
        }));
    }
    for (r, record) in records.iter().enumerate() {
        if record.len() != cols as usize {
            invalid.push(serde_json::json!({
                "reason": "colCountMismatch",
                "row": r,
                "expected": cols,
                "actual": record.len(),
                "message": format!(
                    "CSV {}행의 열 수 {} 가 표의 열 수 {} 와 다릅니다.",
                    r, record.len(), cols
                ),
            }));
        }
    }

    if !invalid.is_empty() {
        return invalid;
    }

    for (r, record) in records.iter().enumerate() {
        for (c, value) in record.iter().enumerate() {
            let (row, col) = (r as u16, c as u16);
            let is_anchor = anchors.iter().any(|(ar, ac, _)| *ar == row && *ac == col);
            if !is_anchor {
                // 병합으로 덮인 칸에는 쓸 수 없다. 값이 있으면 조용히 버리지 않고
                // 거부한다 — 버리면 "썼다고 보고했는데 문서엔 없는" 데이터가 된다.
                if !value.is_empty() {
                    invalid.push(serde_json::json!({
                        "reason": "coveredCellNotEmpty",
                        "row": r,
                        "col": c,
                        "message": format!(
                            "({},{}) 는 병합으로 덮인 칸이라 쓸 수 없습니다 — 값은 앵커 칸에 두고 이 칸은 비우세요.",
                            r, c
                        ),
                    }));
                }
                continue;
            }
            // 셀 안 줄바꿈·탭은 set-cell 과 같은 판정으로 거부한다 (문단 골격을
            // 바꾸는 쓰기는 v1 범위 밖). 내보내기 방향은 인용해서 그대로 낸다.
            if let Some(message) = set_cell_control_char_rejection(value) {
                invalid.push(serde_json::json!({
                    "reason": "controlCharacter",
                    "row": r,
                    "col": c,
                    "message": message,
                }));
            }
        }
    }
    invalid
}

pub(crate) fn csv_to_table(args: &[String]) -> i32 {
    use rhwp::document_core::queries::table_csv::parse_csv;
    use rhwp::document_core::queries::table_extract::extract_tables;

    let CsvToTableArgs {
        file_path,
        csv_path,
        table_no,
        out_path,
        dry_run,
        verify_mode,
        json_mode,
    } = match parse_csv_to_table_args(args) {
        Ok(parsed) => parsed,
        Err(code) => return code,
    };

    let csv_bytes = match fs::read(&csv_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: CSV 파일을 읽을 수 없습니다 - {}: {}", csv_path, e);
            return EXIT_RUNTIME;
        }
    };
    let csv_text = match String::from_utf8(csv_bytes) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("오류: CSV 가 UTF-8 이 아닙니다 - {}: {}", csv_path, e);
            return EXIT_RUNTIME;
        }
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

    // 표 좌표는 export-tables/set-cell 과 같은 격자 — 여기서 한 번만 뽑아 쓴다
    // (칸마다 재추출하면 표 53개짜리 문서에서 O(칸수) 순회가 된다).
    let (host_section, host_paragraph, rows, cols, anchors) = {
        let grids = extract_tables(doc.document());
        let Some(grid) = grids
            .iter()
            .find(|g| g.index == table_no && g.container_path.is_empty())
        else {
            let top_level = grids.iter().filter(|g| g.container_path.is_empty()).count();
            eprintln!(
                "오류: 본문 최상위 표 {} 번이 없습니다 (최상위 표 {}개; 중첩 표는 v1 범위 밖).",
                table_no, top_level
            );
            return EXIT_RUNTIME;
        };
        let anchors: Vec<(u16, u16, String)> = grid
            .cells
            .iter()
            .map(|c| (c.row, c.col, c.text.clone()))
            .collect();
        (grid.section, grid.paragraph, grid.rows, grid.cols, anchors)
    };

    // ── 1) 선검증: 한 칸도 쓰기 전에 전부 판정한다 ──
    let mut invalid: Vec<serde_json::Value> = Vec::new();
    let records = match parse_csv(&csv_text) {
        Ok(r) => r,
        Err(e) => {
            invalid.push(serde_json::json!({
                "reason": "csvParse",
                "row": e.record,
                "col": e.field,
                "message": e.to_string(),
            }));
            Vec::new()
        }
    };

    if invalid.is_empty() {
        invalid = validate_csv_table_records(&records, rows, cols, &anchors, table_no);
    }

    if !invalid.is_empty() {
        if json_mode {
            let envelope = serde_json::json!({
                "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                "source": file_path,
                "csv": csv_path,
                "table": table_no,
                "rowCount": rows,
                "colCount": cols,
                "changedCount": 0,
                "changed": [],
                "invalid": invalid,
                "dryRun": dry_run,
                "changedPages": serde_json::Value::Null,
            });
            println!("{}", provenance::marked(envelope, "csv-to-table"));
        } else {
            for item in &invalid {
                eprintln!(
                    "오류: {}",
                    item["message"]
                        .as_str()
                        .unwrap_or("CSV 가 표와 맞지 않습니다.")
                );
            }
        }
        return EXIT_USAGE;
    }

    // ── 2) 적용: 값이 실제로 달라지는 앵커 칸만 다시 쓴다 ──
    let mut changed: Vec<serde_json::Value> = Vec::new();
    for (row, col, old_text) in &anchors {
        let Some(new_text) = records
            .get(*row as usize)
            .and_then(|r| r.get(*col as usize))
        else {
            continue;
        };
        if new_text == old_text {
            continue;
        }
        // 좌표 해석은 set-cell 과 같은 경로(resolve_table_cell)를 쓴다 — 격자 배열
        // 위치와 모델 셀 인덱스가 어긋날 수 있어(손상 방어 필터) 직접 세지 않는다.
        let (sec, para, ctrl, cell_idx, para_lens, old) =
            match resolve_table_cell(doc.document(), table_no, *row, *col) {
                Ok(v) => v,
                Err(CellResolveError::Usage(msg)) | Err(CellResolveError::Runtime(msg)) => {
                    eprintln!("{msg}");
                    return EXIT_RUNTIME;
                }
            };
        if !dry_run {
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
                    eprintln!(
                        "오류: 셀 비우기 실패({},{} 문단 {}) - {:?}",
                        row, col, pi, e
                    );
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
                    eprintln!("오류: 셀 쓰기 실패({},{}) - {:?}", row, col, e);
                    return EXIT_RUNTIME;
                }
            }
        }
        changed.push(serde_json::json!({
            "row": row, "col": col, "oldText": old, "newText": new_text,
        }));
    }

    // ── 3) 저장 ──
    // set-cell 과 달리 글자색을 검정으로 덮지 않는다. csv-to-table 은 빈 서식을 채우는
    // 것이 아니라 **이미 서식이 잡힌 보고서의 값을 갱신**하는 축이라, 표 머리·강조
    // 스타일을 일괄로 지우면 눈에 보이는 회귀가 된다.
    let out_format = edit_output_format(&bytes, out_path.as_deref());
    let output_path = out_path.unwrap_or_else(|| {
        let stem = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        format!("{}_csv.{}", stem, out_format.ext())
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

    // 눈검증 대상 쪽 — 표 호스트 문단이 걸친 쪽 전부(분할 표 포함, #3712).
    let changed_pages = if dry_run {
        serde_json::Value::Null
    } else {
        match doc.pages_covering_paragraphs(&[(host_section, host_paragraph)]) {
            Some(pages) => serde_json::json!(pages),
            None => serde_json::Value::Null,
        }
    };

    if json_mode {
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "csv": csv_path,
            "table": table_no,
            "rowCount": rows,
            "colCount": cols,
            "changedCount": changed.len(),
            "changed": changed,
            "invalid": [],
            "dryRun": dry_run,
            "changedPages": changed_pages,
        });
        if !dry_run {
            envelope["output"] = serde_json::Value::String(output_path.clone());
            envelope["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
            envelope["verify"] = verify_report.clone();
        }
        println!("{}", provenance::marked(envelope, "csv-to-table"));
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }

    if dry_run {
        println!(
            "변경 예정: {} 표{} — {}행×{}열 중 {}칸",
            file_path,
            table_no,
            rows,
            cols,
            changed.len()
        );
    } else {
        println!(
            "표 기록 완료: {} → {} — 표{} {}행×{}열 중 {}칸",
            file_path,
            output_path,
            table_no,
            rows,
            cols,
            changed.len()
        );
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    EXIT_OK
}

/// `csv-to-chart` — CSV 내용으로 기존 차트 N 의 숫자 값을 덮어쓴다 (#4100).
///
/// **크기는 바꾸지 않는다.** 계열 수·값 개수·계열명·카테고리 라벨이 다르면 한 칸도 쓰지
/// 않고 `invalid[]` 로 보고하며 exit 2 다 — `csv-to-table` 과 같은 규약이다.
///
/// 검증은 전부 코어(`set_chart_data_by_index_native`)가 한다. 여기서 하는 것은 CSV 를
/// 행렬로 읽어 넘기고 봉투를 실어 나르는 일뿐이다 — 검증기가 코어와 CLI 로 갈리면
/// 둘이 서로 다른 것을 허용하기 시작한다.
pub(crate) fn csv_to_chart(args: &[String]) -> i32 {
    use rhwp::document_core::queries::chart_csv::from_csv;
    use rhwp::document_core::queries::chart_extract::collect_charts;

    let mut file_path: Option<&str> = None;
    let mut csv_path: Option<String> = None;
    let mut chart_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut verify_mode = false;
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--dry-run" => dry_run = true,
            "--verify" => verify_mode = true,
            "--csv" => {
                i += 1;
                match args.get(i) {
                    Some(p) => csv_path = Some(p.clone()),
                    None => {
                        eprintln!("오류: --csv 뒤에 CSV 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--chart" => {
                i += 1;
                match args.get(i).map(|v| v.parse::<usize>()) {
                    Some(Ok(value)) if value >= 1 => chart_arg = Some(value),
                    _ => {
                        eprintln!("오류: --chart 뒤에 1 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(p) => out_path = Some(p.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
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

    let (Some(file_path), Some(csv_path), Some(chart_no)) = (file_path, csv_path, chart_arg) else {
        eprintln!(
            "사용법: rhwp csv-to-chart <파일.hwp|파일.hwpx> --csv <경로.csv> --chart <번호> [-o <출력>] [--dry-run] [--verify] [--json]"
        );
        return EXIT_USAGE;
    };

    let csv_bytes = match fs::read(&csv_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: CSV 파일을 읽을 수 없습니다 - {}: {}", csv_path, e);
            return EXIT_RUNTIME;
        }
    };
    let csv_text = match String::from_utf8(csv_bytes) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("오류: CSV 가 UTF-8 이 아닙니다 - {}: {}", csv_path, e);
            return EXIT_RUNTIME;
        }
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

    let chart_count = collect_charts(doc.document()).len();
    if chart_no > chart_count {
        eprintln!(
            "오류: 차트 {} 번이 없습니다 (차트 {}개).",
            chart_no, chart_count
        );
        return EXIT_RUNTIME;
    }
    let index = chart_no - 1;

    // ── 1) CSV 구조 — 차트와의 대조는 코어가 한다 ──
    let parsed = match from_csv(&csv_text) {
        Ok(v) => v,
        Err(e) => {
            let invalid = vec![serde_json::json!({
                "reason": "csvParse", "message": e.to_string(),
            })];
            if json_mode {
                let envelope = serde_json::json!({
                    "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                    "source": file_path, "csv": csv_path, "chart": chart_no,
                    "changedCount": 0, "changed": [], "invalid": invalid,
                    "wrote": [], "dryRun": dry_run,
                    "changedPages": serde_json::Value::Null,
                });
                println!("{}", provenance::marked(envelope, "csv-to-chart"));
            } else {
                eprintln!("오류: {e}");
            }
            return EXIT_USAGE;
        }
    };

    // ── 2) 코어에 행렬을 그대로 넘긴다 ──
    let edits = serde_json::json!({
        "labels": parsed.labels,
        "series": parsed
            .names
            .iter()
            .zip(parsed.values.iter())
            .map(|(name, values)| serde_json::json!({"name": name, "values": values}))
            .collect::<Vec<_>>(),
        "dryRun": dry_run,
    });
    let result: serde_json::Value = match doc
        .set_chart_data_by_index_native(index, &edits.to_string())
        .map(|s| serde_json::from_str(&s).unwrap_or(serde_json::Value::Null))
    {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 차트 {} 편집 실패 - {:?}", chart_no, e);
            return EXIT_RUNTIME;
        }
    };

    if result["ok"] != true {
        let invalid = result["invalid"].clone();
        if json_mode {
            let envelope = serde_json::json!({
                "schemaVersion": ENVELOPE_SCHEMA_VERSION,
                "source": file_path, "csv": csv_path, "chart": chart_no,
                "changedCount": 0, "changed": [], "invalid": invalid,
                "wrote": [], "dryRun": dry_run,
                "changedPages": serde_json::Value::Null,
            });
            println!("{}", provenance::marked(envelope, "csv-to-chart"));
        } else {
            for item in invalid.as_array().unwrap_or(&Vec::new()) {
                eprintln!(
                    "오류: {}",
                    item["message"]
                        .as_str()
                        .unwrap_or("CSV 가 차트와 맞지 않습니다.")
                );
            }
        }
        return EXIT_USAGE;
    }

    let changed = result["changed"].clone();
    let changed_count = result["changedCount"].as_u64().unwrap_or(0);
    let wrote = result["wrote"].clone();

    // ── 3) 저장 ──
    let out_format = edit_output_format(&bytes, out_path.as_deref());
    let output_path = out_path.unwrap_or_else(|| {
        let stem = Path::new(file_path)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "output".to_string());
        format!("{}_chart.{}", stem, out_format.ext())
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

    if json_mode {
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "csv": csv_path,
            "chart": chart_no,
            "changedCount": changed_count,
            "changed": changed,
            "invalid": [],
            "wrote": wrote,
            "dryRun": dry_run,
            "changedPages": serde_json::Value::Null,
        });
        if !dry_run {
            envelope["output"] = serde_json::Value::String(output_path.clone());
            envelope["outputFormat"] = serde_json::Value::String(out_format.label().to_string());
            envelope["verify"] = verify_report.clone();
        }
        println!("{}", provenance::marked(envelope, "csv-to-chart"));
        if verify_failed {
            process::exit(3);
        }
        return EXIT_OK;
    }

    if dry_run {
        println!(
            "변경 예정: {} 차트{} — {}칸",
            file_path, chart_no, changed_count
        );
    } else {
        println!(
            "차트 기록 완료: {} → {} — 차트{} {}칸 ({})",
            file_path,
            output_path,
            chart_no,
            changed_count,
            wrote
                .as_array()
                .map(|a| a
                    .iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join("+"))
                .unwrap_or_default()
        );
    }
    if verify_failed {
        eprintln!("검증 실패(--verify): 저장본 재파싱 IR 차이 — 상세는 --json 또는 ir-diff");
        process::exit(3);
    }
    EXIT_OK
}
