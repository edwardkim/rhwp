//! 주소 조회 — 쪽↔문단, 누름틀 이름, 쪽/구역 정의, 차트 데이터. 문서를 고치지 않는다.

use crate::envelope::{
    envelope, one_file, one_file_value, open_core, print_json, EXIT_GATE, EXIT_OK, EXIT_RUNTIME,
    EXIT_USAGE,
};
use serde_json::json;

fn parse_native(raw: &str, what: &str) -> Result<serde_json::Value, i32> {
    serde_json::from_str(raw).map_err(|e| {
        eprintln!("오류: {what} JSON 이 깨졌습니다 - {e}");
        EXIT_RUNTIME
    })
}

pub fn run_doc_info(args: &[String]) -> i32 {
    let usage = "rhwp-agent doc-info <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let info = match parse_native(&core.get_document_info(), "문서 정보") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let payload = json!({
        "source": opts.path,
        "info": info,
    });
    if opts.json {
        print_json(&envelope("doc-info", payload, &["info.fonts[]"]));
    } else {
        crate::outln!("{}", opts.path);
    }
    EXIT_OK
}

pub fn run_page_info(args: &[String]) -> i32 {
    let usage = "rhwp-agent page-info <파일> --page <N> [--json]";
    let (opts, page) = match one_file_value(args, usage, "--page") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(page) = page.and_then(|s| s.parse::<u32>().ok()) else {
        eprintln!("오류: --page 뒤에 쪽 번호(0부터)가 필요합니다.");
        return EXIT_USAGE;
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_page_info_native(page) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: {page}쪽 정보를 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let info = match parse_native(&raw, "쪽 정보") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let payload = json!({
        "source": opts.path,
        "page": page,
        "info": info,
    });
    if opts.json {
        print_json(&envelope("page-info", payload, &[]));
    } else {
        crate::outln!("page={page}");
    }
    EXIT_OK
}

pub fn run_section_def(args: &[String]) -> i32 {
    let usage = "rhwp-agent section-def <파일> --section <N> [--json]";
    let (opts, section) = match one_file_value(args, usage, "--section") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(section) = section.and_then(|s| s.parse::<usize>().ok()) else {
        eprintln!("오류: --section 뒤에 구역 번호(0부터)가 필요합니다.");
        return EXIT_USAGE;
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_section_def_native(section) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 구역 {section} 정의를 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let def = match parse_native(&raw, "구역 정의") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let payload = json!({
        "source": opts.path,
        "section": section,
        "def": def,
    });
    if opts.json {
        print_json(&envelope("section-def", payload, &[]));
    } else {
        crate::outln!("section={section}");
    }
    EXIT_OK
}

pub fn run_field_get(args: &[String]) -> i32 {
    let usage = "rhwp-agent field-get <파일> --name <이름> [--json]";
    let (opts, name) = match one_file_value(args, usage, "--name") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(name) = name.filter(|s| !s.is_empty()) else {
        eprintln!("오류: --name 뒤에 누름틀 이름이 필요합니다.");
        return EXIT_USAGE;
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    match core.get_field_value_by_name(&name) {
        Ok(raw) => {
            let got = match parse_native(&raw, "누름틀 값") {
                Ok(v) => v,
                Err(c) => return c,
            };
            let payload = json!({
                "source": opts.path,
                "name": name,
                "found": true,
                "field": got,
            });
            if opts.json {
                print_json(&envelope("field-get", payload, &["name", "field.value"]));
            } else {
                crate::outln!("{}", got["value"].as_str().unwrap_or(""));
            }
            EXIT_OK
        }
        Err(e) if e.to_string().contains("없음") => {
            let payload = json!({
                "source": opts.path,
                "name": name,
                "found": false,
            });
            if opts.json {
                print_json(&envelope("field-get", payload, &["name"]));
            } else {
                crate::outln!("missing");
            }
            EXIT_GATE
        }
        Err(e) => {
            eprintln!("오류: 누름틀을 읽지 못했습니다 - {e}");
            EXIT_RUNTIME
        }
    }
}

pub fn run_page_pos(args: &[String]) -> i32 {
    let usage = "rhwp-agent page-pos <파일> --page <N> [--json]";
    let (opts, page) = match one_file_value(args, usage, "--page") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(page) = page.and_then(|s| s.parse::<usize>().ok()) else {
        eprintln!("오류: --page 뒤에 쪽 번호(0부터)가 필요합니다.");
        return EXIT_USAGE;
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_position_of_page_native(page) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: {page}쪽 위치를 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let pos = match parse_native(&raw, "쪽 위치") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let payload = json!({
        "source": opts.path,
        "page": page,
        "position": pos,
    });
    if opts.json {
        print_json(&envelope("page-pos", payload, &[]));
    } else {
        crate::outln!("sec={} para={}", pos["sec"], pos["para"]);
    }
    EXIT_OK
}

pub fn run_para_page(args: &[String]) -> i32 {
    let usage = "rhwp-agent para-page <파일> --section <N> --para <N> [--json]";
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut section: Option<usize> = None;
    let mut para: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--section" => {
                let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) else {
                    eprintln!("오류: --section 뒤에 구역 번호가 필요합니다.");
                    return EXIT_USAGE;
                };
                section = Some(v);
                i += 2;
            }
            "--para" => {
                let Some(v) = args.get(i + 1).and_then(|s| s.parse().ok()) else {
                    eprintln!("오류: --para 뒤에 문단 번호가 필요합니다.");
                    return EXIT_USAGE;
                };
                para = Some(v);
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {usage}");
                return EXIT_USAGE;
            }
            other => {
                if path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다.");
                    return EXIT_USAGE;
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }
    let (Some(path), Some(section), Some(para)) = (path, section, para) else {
        eprintln!("오류: 파일과 --section --para 가 필요합니다.");
        eprintln!("사용법: {usage}");
        return EXIT_USAGE;
    };
    let core = match open_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_page_of_position_native(section, para) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 문단 쪽을 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let got = match parse_native(&raw, "문단 쪽") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let payload = json!({
        "source": path,
        "section": section,
        "paragraph": para,
        "page": got.get("page").cloned().unwrap_or(json!(null)),
    });
    if json_mode {
        print_json(&envelope("para-page", payload, &[]));
    } else {
        crate::outln!("{}", payload["page"]);
    }
    EXIT_OK
}

pub fn run_chart_data(args: &[String]) -> i32 {
    let usage = "rhwp-agent chart-data <파일> --chart <N> [--json]";
    let (opts, chart) = match one_file_value(args, usage, "--chart") {
        Ok(v) => v,
        Err(c) => return c,
    };
    let Some(chart) = chart.and_then(|s| s.parse::<usize>().ok()) else {
        eprintln!("오류: --chart 뒤에 차트 번호(0부터)가 필요합니다.");
        return EXIT_USAGE;
    };
    let core = match open_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_chart_data_by_index_native(chart) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 차트 {chart} 데이터를 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let data = match parse_native(&raw, "차트 데이터") {
        Ok(v) => v,
        Err(_) => json!({ "raw": raw }),
    };
    let payload = json!({
        "source": opts.path,
        "chart": chart,
        "data": data,
    });
    if opts.json {
        print_json(&envelope("chart-data", payload, &["data"]));
    } else {
        crate::outln!("chart={chart}");
    }
    EXIT_OK
}
