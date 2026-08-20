//! Chart, numbering, form, and page-object command adapters.

use std::fs;

use rhwp::provenance;
use rhwp::schema_registry::ENVELOPE_SCHEMA_VERSION;

use super::runtime::finish_edit_write;
use crate::{load_document, EXIT_RUNTIME, EXIT_USAGE};

pub(super) fn edit_set_chart_data(args: &[String]) -> i32 {
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

pub(super) fn edit_insert_number(args: &[String]) -> i32 {
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

pub(super) fn edit_set_form_value(args: &[String]) -> i32 {
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

pub(super) fn edit_set_form_value_in_cell(args: &[String]) -> i32 {
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

pub(super) fn edit_set_page_border_fill(args: &[String]) -> i32 {
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
