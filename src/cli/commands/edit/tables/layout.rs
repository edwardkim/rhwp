/// `edit fit-table` — 표를 페이지 본문 폭에 맞춘다. 코어 `fit_table_to_page_native`.
use std::fs;

use super::resolve_top_table;
use crate::cli::commands::edit::runtime::finish_edit_write;
use crate::{load_document, EXIT_RUNTIME, EXIT_USAGE};

pub(in crate::cli::commands::edit) fn edit_fit_table(args: &[String]) -> i32 {
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
pub(in crate::cli::commands::edit) fn edit_resize_table(args: &[String]) -> i32 {
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
/// `edit set-column-widths` — 열 폭 설정. 코어 `set_table_column_widths_native`.
pub(in crate::cli::commands::edit) fn edit_set_column_widths(args: &[String]) -> i32 {
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
/// `edit resize-table-cell` — 한 칸 크기 조절. 코어 `resize_table_cell_native`.
pub(in crate::cli::commands::edit) fn edit_resize_table_cell(args: &[String]) -> i32 {
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
/// `edit set-table-props` — 표 속성. 코어 `set_table_properties_native`.
pub(in crate::cli::commands::edit) fn edit_set_table_props(args: &[String]) -> i32 {
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
pub(in crate::cli::commands::edit) fn edit_move_table(args: &[String]) -> i32 {
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
