//! `rhwp run` 계획서 JSON 선검증. 문서를 열지 않는다.

use crate::envelope::{
    envelope, one_file, print_json, read_file, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::json;

pub fn run_plan_lint(args: &[String]) -> i32 {
    let usage = "rhwp-agent plan-lint <계획.json> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let data = match read_file(&opts.path) {
        Ok(d) => d,
        Err(m) => {
            eprintln!("오류: {m}");
            return EXIT_RUNTIME;
        }
    };
    let value: serde_json::Value = match serde_json::from_slice(&data) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: JSON 이 아닙니다 - {e}");
            return EXIT_USAGE;
        }
    };
    let mut invalid: Vec<String> = Vec::new();
    if !value
        .get("planVersion")
        .and_then(|v| v.as_str())
        .map(|s| !s.is_empty())
        .unwrap_or(false)
    {
        invalid.push("planVersion 누락".into());
    }
    match value.get("steps") {
        Some(s) if s.is_array() => {}
        Some(_) => invalid.push("steps 가 배열이 아님".into()),
        None => invalid.push("steps 누락".into()),
    }
    if let Some(pre) = value.get("preconditions") {
        if let Some(obj) = pre.as_object() {
            if let Some(sha) = obj.get("inputSha256").and_then(|v| v.as_str()) {
                if sha.len() != 64 || !sha.chars().all(|c| c.is_ascii_hexdigit()) {
                    invalid.push("preconditions.inputSha256 형식".into());
                }
            }
        } else {
            invalid.push("preconditions 가 객체가 아님".into());
        }
    }
    let ok = invalid.is_empty();
    let step_count = value
        .get("steps")
        .and_then(|s| s.as_array())
        .map(|a| a.len())
        .unwrap_or(0);
    let payload = json!({
        "source": opts.path,
        "ok": ok,
        "stepCount": step_count,
        "invalid": invalid,
    });
    if opts.json {
        print_json(&envelope("plan-lint", payload, &[]));
    } else if ok {
        crate::outln!("ok steps={step_count}");
    } else {
        crate::outln!("invalid");
        for item in payload["invalid"].as_array().cloned().unwrap_or_default() {
            eprintln!("{}", item.as_str().unwrap_or(""));
        }
    }
    if ok {
        EXIT_OK
    } else {
        EXIT_USAGE
    }
}
