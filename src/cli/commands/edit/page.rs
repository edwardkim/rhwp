//! 쪽, 구역, 단 구조 편집 명령.

use std::fs;

use super::runtime::finish_edit_write;
use crate::{load_document, EXIT_RUNTIME, EXIT_USAGE};

pub(super) fn edit_set_page_def(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-page-def <파일> --props <JSON> [--section N] [-o <출력>] [--dry-run] [--verify] [--json]";
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
                        eprintln!("오류: --props 뒤에 용지 설정 JSON 이 필요합니다.");
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
        if let Err(e) = doc.set_page_def_native(section, props) {
            eprintln!("오류: 용지 설정 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "pagedef",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({ "section": section, "props": props }),
        &[(section, 0)],
        &format!("용지 설정 예정: {file_path} 구역 {section}"),
        &format!("용지 설정 완료: {file_path}"),
    )
}
/// `edit set-section-def` — 구역 정의. 코어 `set_section_def_native`.
pub(super) fn edit_set_section_def(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-section-def <파일> --props <JSON> [--section N] [-o <출력>] [--dry-run] [--verify] [--json]";
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
                        eprintln!("오류: --props 뒤에 구역 정의 JSON 이 필요합니다.");
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
        if let Err(e) = doc.set_section_def_native(section, props) {
            eprintln!("오류: 구역 정의 적용 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "secdef",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "props": props
        }),
        &[(section, 0)],
        &format!("구역 정의 예정: {file_path} 구역 {section}"),
        &format!("구역 정의 완료: {file_path}"),
    )
}
/// [#5081] `edit set-column-def` — 구역 단 정의. 코어 `set_column_def_native`.
pub(super) fn edit_set_column_def(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-column-def <파일> --count N [--section N] [--type 0|1|2] [--same-width|--mixed-width] [--spacing N] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut count_arg: Option<u16> = None;
    let mut section: usize = 0;
    let mut column_type: u8 = 0;
    let mut same_width = true;
    let mut spacing: i16 = 0;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--count" | "--section" | "--type" | "--spacing" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match name.as_str() {
                    "--count" => match v.parse::<u16>() {
                        Ok(n) if n >= 1 => count_arg = Some(n),
                        _ => {
                            eprintln!("오류: --count 뒤에 1 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--section" => match v.parse::<usize>() {
                        Ok(n) => section = n,
                        Err(_) => {
                            eprintln!("오류: --section 뒤에 0 이상의 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    "--type" => match v.parse::<u8>() {
                        Ok(n) if n <= 2 => column_type = n,
                        _ => {
                            eprintln!("오류: --type 은 0(일반)·1(배분)·2(평행) 만 허용합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                    _ => match v.parse::<i16>() {
                        Ok(n) => spacing = n,
                        Err(_) => {
                            eprintln!("오류: --spacing 뒤에 정수가 필요합니다: {v}");
                            return EXIT_USAGE;
                        }
                    },
                }
            }
            "--same-width" => same_width = true,
            "--mixed-width" => same_width = false,
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
    let (Some(file_path), Some(count)) = (file_path, count_arg) else {
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
        if let Err(e) = doc.set_column_def_native(section, count, column_type, same_width, spacing)
        {
            eprintln!("오류: 단 정의 변경 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "coldef",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "columnCount": count,
            "columnType": column_type,
            "sameWidth": same_width,
            "spacing": spacing
        }),
        &[(section, 0)],
        &format!("단 정의 변경 예정: {file_path} 구역 {section} 단 {count}"),
        &format!("단 정의 변경 완료: {file_path}"),
    )
}
/// [#5083] `edit set-page-hide` — 쪽 감추기. 코어 `set_page_hide_native`.
pub(super) fn edit_set_page_hide(args: &[String]) -> i32 {
    const USAGE: &str = "사용법: rhwp edit set-page-hide <파일> [--section N] [--para N] [--hide-header] [--hide-footer] [--hide-master] [--hide-border] [--hide-fill] [--hide-page-num] [-o <출력>] [--dry-run] [--verify] [--json]";
    let mut file_path: Option<&str> = None;
    let mut section: usize = 0;
    let mut para: usize = 0;
    let mut hide_header = false;
    let mut hide_footer = false;
    let mut hide_master = false;
    let mut hide_border = false;
    let mut hide_fill = false;
    let mut hide_page_num = false;
    let mut out_path: Option<String> = None;
    let mut dry_run = false;
    let mut json_mode = false;
    let mut verify_mode = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--section" | "--para" => {
                let name = args[i].clone();
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다.");
                    return EXIT_USAGE;
                };
                match v.parse::<usize>() {
                    Ok(n) => {
                        if name == "--section" {
                            section = n;
                        } else {
                            para = n;
                        }
                    }
                    Err(_) => {
                        eprintln!("오류: {name} 뒤에 0 이상의 정수가 필요합니다: {v}");
                        return EXIT_USAGE;
                    }
                }
            }
            "--hide-header" => hide_header = true,
            "--hide-footer" => hide_footer = true,
            "--hide-master" => hide_master = true,
            "--hide-border" => hide_border = true,
            "--hide-fill" => hide_fill = true,
            "--hide-page-num" => hide_page_num = true,
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
        if let Err(e) = doc.set_page_hide_native(
            section,
            para,
            hide_header,
            hide_footer,
            hide_master,
            hide_border,
            hide_fill,
            hide_page_num,
        ) {
            eprintln!("오류: 쪽 감추기 설정 실패 - {e}");
            return EXIT_RUNTIME;
        }
    }
    finish_edit_write(
        &mut doc,
        &bytes,
        file_path,
        out_path,
        "pagehide",
        dry_run,
        json_mode,
        verify_mode,
        serde_json::json!({
            "section": section,
            "paragraph": para,
            "hideHeader": hide_header,
            "hideFooter": hide_footer,
            "hideMasterPage": hide_master,
            "hideBorder": hide_border,
            "hideFill": hide_fill,
            "hidePageNum": hide_page_num
        }),
        &[(section, para)],
        &format!("쪽 감추기 예정: {file_path} 구역 {section} 문단 {para}"),
        &format!("쪽 감추기 완료: {file_path}"),
    )
}
