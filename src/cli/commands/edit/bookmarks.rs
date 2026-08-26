//! 책갈피 구조 편집 명령.

use std::fs;

use super::runtime::finish_edit_write;
use crate::{load_document, EXIT_RUNTIME, EXIT_USAGE};

/// [#5026] `edit add-bookmark` — 책갈피 추가.
pub(super) fn edit_add_bookmark(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit add-bookmark <파일> --name <이름> [--section N] [--para N] [--offset N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut name: Option<String> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--name" => {
                i += 1;
                match args.get(i) {
                    Some(v) => name = Some(v.clone()),
                    None => {
                        eprintln!("오류: --name 뒤에 책갈피 이름이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--para" | "--offset" => {
                let flag = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match flag.as_str() {
                        "--section" => section = n,
                        "--para" => para = n,
                        _ => offset = n,
                    },
                    Err(_) => {
                        eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다: {v}");
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
    let (Some(file_path), Some(name)) = (file_path, name) else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if name.trim().is_empty() {
        eprintln!("오류: --name 은 비어 있을 수 없습니다.");
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
        match doc.add_bookmark_native(section, para, offset, &name) {
            Ok(raw) => {
                let v: serde_json::Value =
                    serde_json::from_str(&raw).unwrap_or(serde_json::json!({}));
                if v["ok"] == false {
                    let err = v["error"].as_str().unwrap_or("책갈피 추가 실패");
                    eprintln!("오류: {err}");
                    return EXIT_RUNTIME;
                }
            }
            Err(e) => {
                eprintln!("오류: 책갈피 추가 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "bookmark",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "offset": offset,
            "name": name
        }),
        &[(section, para)],
        &format!(
            "책갈피 추가 예정: {file_path} 이름 {name} 구역 {section} 문단 {para} 오프셋 {offset}"
        ),
        &format!("책갈피 추가 완료: {file_path}"),
    )
}
/// [#5027] `edit delete-bookmark` — 책갈피 삭제.
pub(super) fn edit_delete_bookmark(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-bookmark <파일> --section N --para N --ctrl N [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
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
    let (Some(file_path), Some(section), Some(para), Some(ctrl)) = (file_path, section, para, ctrl)
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
    if !dry_run {
        match doc.delete_bookmark_native(section, para, ctrl) {
            Ok(raw) => {
                let v: serde_json::Value =
                    serde_json::from_str(&raw).unwrap_or(serde_json::json!({}));
                if v["ok"] == false {
                    let err = v["error"].as_str().unwrap_or("책갈피 삭제 실패");
                    eprintln!("오류: {err}");
                    return EXIT_RUNTIME;
                }
            }
            Err(e) => {
                eprintln!("오류: 책갈피 삭제 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "delbm",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("책갈피 삭제 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("책갈피 삭제 완료: {file_path}"),
    )
}
/// [#5033] `edit rename-bookmark` — 책갈피 이름 변경.
pub(super) fn edit_rename_bookmark(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit rename-bookmark <파일> --section N --para N --ctrl N --name <이름> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut ctrl: Option<usize> = None;
    let mut name: Option<String> = None;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--name" => {
                i += 1;
                match args.get(i) {
                    Some(v) => name = Some(v.clone()),
                    None => {
                        eprintln!("오류: --name 뒤에 책갈피 이름이 필요합니다.");
                        return EXIT_USAGE;
                    }
                }
            }
            "--section" | "--para" | "--ctrl" => {
                let flag = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => match flag.as_str() {
                        "--section" => section = Some(n),
                        "--para" => para = Some(n),
                        _ => ctrl = Some(n),
                    },
                    Err(_) => {
                        eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다: {v}");
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
    let (Some(file_path), Some(section), Some(para), Some(ctrl), Some(name)) =
        (file_path, section, para, ctrl, name)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if name.trim().is_empty() {
        eprintln!("오류: --name 은 비어 있을 수 없습니다.");
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
        match doc.rename_bookmark_native(section, para, ctrl, &name) {
            Ok(raw) => {
                let v: serde_json::Value =
                    serde_json::from_str(&raw).unwrap_or(serde_json::json!({}));
                if v["ok"] == false {
                    let err = v["error"].as_str().unwrap_or("책갈피 이름 변경 실패");
                    eprintln!("오류: {err}");
                    return EXIT_RUNTIME;
                }
            }
            Err(e) => {
                eprintln!("오류: 책갈피 이름 변경 실패 - {e}");
                return EXIT_RUNTIME;
            }
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "renbm",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "ctrl": ctrl,
            "name": name
        }),
        &[(section, para)],
        &format!(
            "책갈피 이름 변경 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl} → {name}"
        ),
        &format!("책갈피 이름 변경 완료: {file_path}"),
    )
}
