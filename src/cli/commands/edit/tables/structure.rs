/// `edit insert-table` — 본문 표 생성. 코어 `create_table_native` 배선.
use std::fs;

use super::resolve_top_table;
use crate::cli::commands::edit::runtime::finish_edit_write;
use crate::{load_document, EXIT_RUNTIME, EXIT_USAGE};

pub(in crate::cli::commands::edit) fn edit_insert_table(args: &[String]) -> i32 {
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
/// `edit split-table` — 표를 지정 행에서 둘로 나눈다. 코어 `split_table_native`.
pub(in crate::cli::commands::edit) fn edit_split_table(args: &[String]) -> i32 {
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
/// `edit merge-table` — 다음 표를 이어 붙인다. 코어 `merge_table_with_next_native`.
pub(in crate::cli::commands::edit) fn edit_merge_table(args: &[String]) -> i32 {
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
/// [#5028] `edit delete-table` — 본문 최상위 표 삭제.
pub(in crate::cli::commands::edit) fn edit_delete_table(args: &[String]) -> i32 {
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
/// [#5108] `edit transpose-table` — 표 행/열 바꿈. 코어 `transpose_table_cells_in_place_native`.
pub(in crate::cli::commands::edit) fn edit_transpose_table(args: &[String]) -> i32 {
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
