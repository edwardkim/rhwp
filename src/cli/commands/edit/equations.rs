/// `edit set-equation-properties` — 본문 수식 속성을 바꾼다.
use std::fs;

use super::runtime::finish_edit_write;
use crate::{load_document, EXIT_RUNTIME, EXIT_USAGE};

pub(super) fn edit_set_equation_properties(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-equation-properties <파일> --section N --para N --ctrl N --props <JSON> [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path = None;
    let mut section = None;
    let mut para = None;
    let mut ctrl = None;
    let mut props = None;
    let mut out_path = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--ctrl" => {
                let flag = args[i].clone();
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                let Ok(value) = value.parse::<usize>() else {
                    eprintln!("오류: {flag} 뒤에 0 이상의 정수가 필요합니다: {value}");
                    return EXIT_USAGE;
                };
                match flag.as_str() {
                    "--section" => section = Some(value),
                    "--para" => para = Some(value),
                    _ => ctrl = Some(value),
                }
            }
            "--props" => {
                i += 1;
                props = args
                    .get(i)
                    .map(String::as_str)
                    .filter(|value| !value.is_empty());
                if props.is_none() {
                    eprintln!("오류: --props 뒤에 수식 속성 JSON 이 필요합니다.");
                    return EXIT_USAGE;
                }
            }
            "-o" | "--output" => {
                i += 1;
                let Some(value) = args.get(i) else {
                    eprintln!("오류: -o 뒤에 출력 파일 경로가 필요합니다.");
                    return EXIT_USAGE;
                };
                out_path = Some(value.clone());
            }
            "--dry-run" => dry_run = true,
            "--json" => json_mode = true,
            "--verify" => verify_mode = true,
            option if option.starts_with('-') => {
                eprintln!("알 수 없는 옵션: {option}");
                return EXIT_USAGE;
            }
            path => {
                if file_path.replace(path).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {path}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let (Some(file_path), Some(section), Some(para), Some(ctrl), Some(props)) =
        (file_path, section, para, ctrl, props)
    else {
        eprintln!("{USAGE}");
        return EXIT_USAGE;
    };
    if serde_json::from_str::<serde_json::Value>(props).is_err() {
        eprintln!("오류: --props 는 JSON 객체여야 합니다: {props}");
        return EXIT_USAGE;
    }
    let bytes = match fs::read(file_path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {}", file_path, error);
            return EXIT_RUNTIME;
        }
    };
    let mut doc = match load_document(&bytes) {
        Ok(doc) => doc,
        Err(error) => return error.report(),
    };
    if !dry_run {
        if let Err(error) =
            doc.set_equation_properties_native(section, para, ctrl, None, None, props)
        {
            eprintln!("오류: 수식 속성 설정 실패 - {error}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "eqprop",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl, "text": props }),
        &[(section, para)],
        &format!("수식 속성 설정 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("수식 속성 설정 완료: {file_path}"),
    )
}
/// [#5009] `edit delete-col` — 표 열 삭제.
/// `edit insert-equation` — 본문 수식 삽입. 코어 `insert_equation_native`.
pub(super) fn edit_insert_equation(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit insert-equation <파일> --script <수식> [--section N] [--para N] [--offset N] [--font-size N] [--color N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut script_arg: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut offset: usize = 0;
    let mut font_size: u32 = 1000;
    let mut color: u32 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" | "--offset" | "--font-size" | "--color" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--font-size" | "--color" => match v.parse::<u32>() {
                        Ok(n) => {
                            if name == "--font-size" {
                                if n == 0 {
                                    eprintln!("오류: --font-size 는 1 이상이어야 합니다.");
                                    return EXIT_USAGE;
                                }
                                font_size = n;
                            } else {
                                color = n;
                            }
                        }
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<usize>() {
                        Ok(n) => match name.as_str() {
                            "--section" => section = n,
                            "--para" => para = n,
                            _ => offset = n,
                        },
                        Err(_) => {
                            eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--script" => {
                i += 1;
                match args.get(i) {
                    Some(v) if !v.is_empty() => script_arg = Some(v.as_str()),
                    _ => {
                        eprintln!("오류: --script 뒤에 수식 문자열이 필요합니다 (빈 문자열 불가).");
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
    let (Some(file_path), Some(script)) = (file_path, script_arg) else {
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
        if let Err(e) = doc.insert_equation_native(section, para, offset, script, font_size, color)
        {
            eprintln!("오류: 수식 삽입 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "eq",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "offset": offset,
            "script": script,
            "fontSize": font_size,
            "color": color
        }),
        &[(section, para)],
        &format!("수식 삽입 예정: {file_path} 구역 {section} 문단 {para} 오프셋 {offset}"),
        &format!("수식 삽입 완료: {file_path}"),
    )
}
/// [#5120] `edit delete-equation` — 본문 수식 삭제.
pub(super) fn edit_delete_equation(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit delete-equation <파일> --section N --para N --ctrl N [-o <출력>] [--dry-run] [--verify] [--json]";
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
        if let Err(e) = doc.delete_equation_control_native(section, para, ctrl) {
            eprintln!("오류: 수식 삭제 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "deleq",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "paragraph": para, "ctrl": ctrl }),
        &[(section, para)],
        &format!("수식 삭제 예정: {file_path} 구역 {section} 문단 {para} 컨트롤 {ctrl}"),
        &format!("수식 삭제 완료: {file_path}"),
    )
}
