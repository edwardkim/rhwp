/// [#4994] `edit insert-row` — 표 행 삽입.
use std::fs;

use super::resolve_top_table;
use crate::cli::commands::edit::runtime::finish_edit_write;
use crate::{load_document, EXIT_RUNTIME, EXIT_USAGE};

pub(in crate::cli::commands::edit) fn edit_insert_row(args: &[String]) -> i32 {
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
pub(in crate::cli::commands::edit) fn edit_insert_col(args: &[String]) -> i32 {
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
pub(in crate::cli::commands::edit) fn edit_delete_row(args: &[String]) -> i32 {
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
pub(in crate::cli::commands::edit) fn edit_merge_cells(args: &[String]) -> i32 {
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
/// [#5009] `edit delete-col` — 표 열 삭제.
pub(in crate::cli::commands::edit) fn edit_delete_col(args: &[String]) -> i32 {
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
pub(in crate::cli::commands::edit) fn edit_split_cell(args: &[String]) -> i32 {
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
pub(in crate::cli::commands::edit) fn edit_split_cell_into(args: &[String]) -> i32 {
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
