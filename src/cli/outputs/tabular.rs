//! Read-only table and chart output adapters.

use std::fs;
use std::path::Path;

use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use crate::{load_document, tables_json_value, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE};

pub(crate) fn export_tables(args: &[String]) -> i32 {
    use rhwp::document_core::queries::table_extract::extract_tables;

    let mut file_path: Option<&str> = None;
    let mut out_path: Option<String> = None;
    let mut json_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "-o" | "--out" | "--output" => {
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
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다.");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }

    let Some(file_path) = file_path else {
        eprintln!("사용법: rhwp export-tables <파일.hwp|파일.hwpx> [--json] [-o <출력.json>]");
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let doc = match load_document(&data) {
        Ok(d) => d,
        Err(e) => return e.report(),
    };

    let tables = extract_tables(doc.document());
    let envelope = tables_json_value(file_path, &tables);

    if let Some(p) = out_path {
        let json = match serde_json::to_string_pretty(&envelope) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("오류: JSON 직렬화 실패 - {}", e);
                return EXIT_RUNTIME;
            }
        };
        return match fs::write(&p, &json) {
            Ok(_) => {
                println!("표 추출 완료: {}개 → {}", tables.len(), p);
                EXIT_OK
            }
            Err(e) => {
                eprintln!("오류: 출력 쓰기 실패 - {}: {}", p, e);
                EXIT_RUNTIME
            }
        };
    }

    if json_mode {
        println!("{}", envelope);
        return EXIT_OK;
    }

    // 기본 출력은 사람용 요약 — 기계 소비는 --json 이 담당한다.
    println!("문서 로드: {} (표 {}개)", file_path, tables.len());
    for t in &tables {
        let merged = t
            .cells
            .iter()
            .filter(|c| c.row_span > 1 || c.col_span > 1)
            .count();
        let nested = t.cells.iter().filter(|c| !c.nested.is_empty()).count();
        println!(
            "  표{} [구역{}:문단{}]: {}행×{}열, 셀 {}개 (병합 {}개, 중첩 {}개)",
            t.index, t.section, t.paragraph, t.rows, t.cols, t.cell_count, merged, nested
        );
    }
    EXIT_OK
}

/// `export-llm` — HWP/HWPX 를 **LLM-ready RAG 청크**로 내보낸다.
///
/// 세계 문서-AI 도구들(Docling·LlamaParse·MarkItDown 등)이 PDF 에는 해 주지만 HWP 에는
/// 못 해 주는 것 — 구조 인지 청킹·자기완결 표·출처 앵커·untrusted 표지 — 를 rhwp 의
/// **정확한 이진 구조**(픽셀 추측 아님) 위에서 낸다. 재파싱하지 않고 기존 IR
/// (`build_structure`·`extract_tables`)을 소비한다. 설계·한계는 `src/rag/mod.rs`.
///
/// 기본 산출은 NDJSON(한 줄당 청크 하나 — 스트림·grep·재개에 적합). `--format json` 은
/// 단일 봉투. 청크 텍스트는 봉투 출처 계약대로 문서 파생(신뢰 불가)으로 표지한다.
pub(crate) fn table_to_csv(args: &[String]) -> i32 {
    use rhwp::document_core::queries::table_csv::grid_to_csv;
    use rhwp::document_core::queries::table_extract::extract_tables;

    let mut file_path: Option<&str> = None;
    let mut table_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut bom = false;
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--bom" => bom = true,
            "--table" => {
                i += 1;
                match args.get(i).map(|v| v.parse::<usize>()) {
                    Some(Ok(value)) => table_arg = Some(value),
                    _ => {
                        eprintln!("오류: --table 뒤에 0 이상의 정수가 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "-o" | "--out" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(p) => out_path = Some(p.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 경로가 필요합니다.");
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

    let Some(file_path) = file_path else {
        eprintln!(
            "사용법: rhwp table-to-csv <파일.hwp|파일.hwpx> [--table <번호>] [-o <경로>] [--bom] [--json]"
        );
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    // 본문 최상위 표만 다룬다 — `edit set-cell`(resolve_table_cell)과 같은 좌표계라야
    // 내보낸 CSV 의 표 번호를 그대로 되돌려 쓸 수 있다. 중첩 표는 v1 범위 밖이다.
    let grids = extract_tables(doc.document());
    let top_level: Vec<&_> = grids
        .iter()
        .filter(|g| g.container_path.is_empty())
        .collect();
    let selected: Vec<&_> = match table_arg {
        Some(n) => match top_level.iter().find(|g| g.index == n) {
            Some(g) => vec![*g],
            None => {
                eprintln!(
                    "오류: 본문 최상위 표 {} 번이 없습니다 (최상위 표 {}개; 중첩 표는 v1 범위 밖).",
                    n,
                    top_level.len()
                );
                return EXIT_RUNTIME;
            }
        },
        None => top_level.clone(),
    };

    // 표별 CSV 본문. 격자 채움과 인용은 전부 코어(table_csv)가 한다.
    let bodies: Vec<(usize, u16, u16, String)> = selected
        .iter()
        .map(|g| (g.index, g.rows, g.cols, grid_to_csv(g)))
        .collect();

    // -o 의 뜻은 --table 유무로 갈린다: 한 표면 그 경로가 파일, 전부면 표별 파일을
    // 담을 디렉터리다(export-svg 의 -o 규약과 같은 이유 — 산출물이 여러 개다).
    let mut written: Vec<Option<String>> = vec![None; bodies.len()];
    if let Some(dest) = out_path.as_deref() {
        if table_arg.is_some() {
            let body = &bodies[0].3;
            if let Err(e) = write_csv_file(dest, body, bom) {
                eprintln!("오류: 출력 쓰기 실패 - {}: {}", dest, e);
                return EXIT_RUNTIME;
            }
            written[0] = Some(dest.to_string());
        } else {
            if let Err(e) = fs::create_dir_all(dest) {
                eprintln!("오류: 출력 폴더 생성 실패 - {}: {}", dest, e);
                return EXIT_RUNTIME;
            }
            for (slot, (index, _, _, body)) in written.iter_mut().zip(bodies.iter()) {
                let path = Path::new(dest).join(format!("table{index}.csv"));
                let shown = path.to_string_lossy().to_string();
                if let Err(e) = write_csv_file(&shown, body, bom) {
                    eprintln!("오류: 출력 쓰기 실패 - {}: {}", shown, e);
                    return EXIT_RUNTIME;
                }
                *slot = Some(shown);
            }
        }
    }

    if json_mode {
        let tables: Vec<serde_json::Value> = bodies
            .iter()
            .zip(written.iter())
            .map(|((index, rows, cols, body), out)| {
                let mut entry = serde_json::json!({
                    "index": index,
                    "rowCount": rows,
                    "colCount": cols,
                    "csv": body,
                });
                if let Some(p) = out {
                    entry["output"] = serde_json::Value::String(p.clone());
                }
                entry
            })
            .collect();
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "tableCount": tables.len(),
            "tables": tables,
            // BOM 은 **파일 인코딩** 표식이라 봉투의 csv 문자열에는 붙이지 않는다.
            // 붙이면 JSON 을 그대로 파싱하는 소비자가 첫 셀 앞의 U+FEFF 를 값으로 읽는다.
            "bom": bom,
        });
        if let Some(p) = out_path {
            envelope["output"] = serde_json::Value::String(p);
            envelope["outputFormat"] = serde_json::Value::String("csv".to_string());
        }
        println!("{}", envelope);
        return EXIT_OK;
    }

    if out_path.is_some() {
        println!("CSV 내보내기 완료: {} (표 {}개)", file_path, bodies.len());
        for out in written.iter().flatten() {
            println!("  {out}");
        }
        return EXIT_OK;
    }

    // -o 도 --json 도 없으면 CSV 본문을 그대로 stdout 으로 흘린다 — 파이프 사용.
    for (index, rows, cols, body) in &bodies {
        if bodies.len() > 1 {
            println!("# table{index} ({rows}x{cols})");
        }
        print!("{body}");
    }
    EXIT_OK
}

/// 차트 읽기 봉투에서 `(라벨, 계열명, 계열값, 분산형 여부)` 를 꺼낸다.
///
/// 값은 **문자열 그대로** 옮긴다 — 실수로 바꿨다 되쓰면 표기가 달라져 무편집 왕복의
/// 바이트 동일이 깨진다(코어가 문자열만 받는 이유와 같다).
fn chart_matrix_from_envelope(
    read: &serde_json::Value,
) -> (Vec<String>, Vec<String>, Vec<Vec<String>>, bool) {
    let strings = |v: &serde_json::Value| -> Vec<String> {
        v.as_array()
            .map(|a| {
                a.iter()
                    .map(|x| x.as_str().unwrap_or_default().to_string())
                    .collect()
            })
            .unwrap_or_default()
    };
    let labels = strings(&read["labels"]);
    let series = read["series"].as_array().cloned().unwrap_or_default();
    let names: Vec<String> = series
        .iter()
        .map(|s| s["name"].as_str().unwrap_or_default().to_string())
        .collect();
    let values: Vec<Vec<String>> = series.iter().map(|s| strings(&s["values"])).collect();
    let scatter = read["axis"].as_str() == Some("scatter");
    (labels, names, values, scatter)
}

/// `chart-to-csv` — 차트 숫자 데이터를 RFC 4180 CSV 로 내보낸다 (#4100).
///
/// 행 = 카테고리(분산형은 X), 열 = 계열. `table-to-csv` 의 `-o`·`--bom`·`--json` 규약을
/// 그대로 따른다 — 같은 도구로 왕복시킬 수 있어야 한다.
pub(crate) fn chart_to_csv(args: &[String]) -> i32 {
    use rhwp::document_core::queries::chart_csv::to_csv;
    use rhwp::document_core::queries::chart_extract::collect_charts;

    let mut file_path: Option<&str> = None;
    let mut chart_arg: Option<usize> = None;
    let mut out_path: Option<String> = None;
    let mut bom = false;
    let mut json_mode = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--bom" => bom = true,
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
            "-o" | "--out" | "--output" => {
                i += 1;
                match args.get(i) {
                    Some(p) => out_path = Some(p.clone()),
                    None => {
                        eprintln!("오류: -o 뒤에 출력 경로가 필요합니다.");
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

    let Some(file_path) = file_path else {
        eprintln!(
            "사용법: rhwp chart-to-csv <파일.hwp|파일.hwpx> [--chart <번호>] [-o <경로>] [--bom] [--json]"
        );
        return EXIT_USAGE;
    };

    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, e);
            return EXIT_RUNTIME;
        }
    };
    let doc = match rhwp::wasm_api::HwpDocument::from_bytes(&data) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("오류: HWP 파싱 실패 - {}", e);
            return EXIT_RUNTIME;
        }
    };

    let chart_count = collect_charts(doc.document()).len();
    let selected: Vec<usize> = match chart_arg {
        Some(n) if n <= chart_count => vec![n - 1],
        Some(n) => {
            eprintln!("오류: 차트 {} 번이 없습니다 (차트 {}개).", n, chart_count);
            return EXIT_RUNTIME;
        }
        None => (0..chart_count).collect(),
    };

    let mut bodies: Vec<(usize, usize, usize, String)> = Vec::new();
    for index in selected {
        let read: serde_json::Value = match doc.get_chart_data_by_index_native(index).map(|s| {
            serde_json::from_str::<serde_json::Value>(&s).unwrap_or(serde_json::Value::Null)
        }) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("오류: 차트 {} 읽기 실패 - {:?}", index + 1, e);
                return EXIT_RUNTIME;
            }
        };
        if read["ok"] != true {
            eprintln!(
                "오류: 차트 {} 를 읽을 수 없습니다 - {}",
                index + 1,
                read["invalid"][0]["message"]
                    .as_str()
                    .unwrap_or("사유 미상")
            );
            return EXIT_RUNTIME;
        }
        if read["labelsShared"] != true {
            let axis = if read["axis"].as_str() == Some("scatter") {
                "X 값"
            } else {
                "카테고리 라벨"
            };
            eprintln!(
                "오류: 차트 {}는 계열마다 {}이 달라 CSV 한 열로 안전하게 표현할 수 없습니다.",
                index + 1,
                axis
            );
            return EXIT_RUNTIME;
        }
        let (labels, names, values, scatter) = chart_matrix_from_envelope(&read);
        // 행 수는 값이 정한다 — 라벨이 없거나 짧은 차트가 실재한다(chart_csv::to_csv 주석).
        let rows = values
            .iter()
            .map(|s| s.len())
            .chain(std::iter::once(labels.len()))
            .max()
            .unwrap_or(0);
        bodies.push((
            index + 1,
            rows,
            names.len(),
            to_csv(&labels, &names, &values, scatter),
        ));
    }

    // -o 의 뜻은 --chart 유무로 갈린다 — `table-to-csv` 와 같은 규약(산출물이 여러 개다).
    let mut written: Vec<Option<String>> = vec![None; bodies.len()];
    if let Some(dest) = out_path.as_deref() {
        if chart_arg.is_some() {
            if let Err(e) = write_csv_file(dest, &bodies[0].3, bom) {
                eprintln!("오류: 출력 쓰기 실패 - {}: {}", dest, e);
                return EXIT_RUNTIME;
            }
            written[0] = Some(dest.to_string());
        } else {
            if let Err(e) = fs::create_dir_all(dest) {
                eprintln!("오류: 출력 폴더 생성 실패 - {}: {}", dest, e);
                return EXIT_RUNTIME;
            }
            for (slot, (number, _, _, body)) in written.iter_mut().zip(bodies.iter()) {
                let path = Path::new(dest).join(format!("chart{number}.csv"));
                let shown = path.to_string_lossy().to_string();
                if let Err(e) = write_csv_file(&shown, body, bom) {
                    eprintln!("오류: 출력 쓰기 실패 - {}: {}", shown, e);
                    return EXIT_RUNTIME;
                }
                *slot = Some(shown);
            }
        }
    }

    if json_mode {
        let charts: Vec<serde_json::Value> = bodies
            .iter()
            .zip(written.iter())
            .map(|((number, rows, cols, body), out)| {
                let mut entry = serde_json::json!({
                    "chart": number,
                    "rowCount": rows,
                    "colCount": cols,
                    "csv": body,
                });
                if let Some(p) = out {
                    entry["output"] = serde_json::Value::String(p.clone());
                }
                entry
            })
            .collect();
        let mut envelope = serde_json::json!({
            "schemaVersion": ENVELOPE_SCHEMA_VERSION,
            "source": file_path,
            "chartCount": charts.len(),
            "charts": charts,
            // BOM 은 파일 인코딩 표식이라 봉투의 csv 문자열에는 붙이지 않는다
            // (`table-to-csv` 와 같은 이유).
            "bom": bom,
        });
        if let Some(p) = out_path {
            envelope["output"] = serde_json::Value::String(p);
            envelope["outputFormat"] = serde_json::Value::String("csv".to_string());
        }
        println!("{}", envelope);
        return EXIT_OK;
    }

    if out_path.is_some() {
        println!(
            "차트 CSV 내보내기 완료: {} (차트 {}개)",
            file_path,
            bodies.len()
        );
        for out in written.iter().flatten() {
            println!("  {out}");
        }
        return EXIT_OK;
    }

    for (number, rows, cols, body) in &bodies {
        if bodies.len() > 1 {
            println!("# chart{number} ({rows}x{cols})");
        }
        print!("{body}");
    }
    EXIT_OK
}

/// CSV 본문 하나를 파일로 쓴다 (선택적 UTF-8 BOM — 엑셀 한글 깨짐 방지).
fn write_csv_file(path: &str, body: &str, bom: bool) -> std::io::Result<()> {
    use rhwp::document_core::queries::table_csv::UTF8_BOM;
    let mut bytes = Vec::with_capacity(body.len() + 3);
    if bom {
        bytes.extend_from_slice(UTF8_BOM.as_bytes());
    }
    bytes.extend_from_slice(body.as_bytes());
    fs::write(path, bytes)
}
