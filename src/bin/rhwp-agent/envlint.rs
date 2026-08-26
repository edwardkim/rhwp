//! rhwp/rhwp-agent JSON 봉투 선검증. 문서를 열지 않는다.

use crate::envelope::{
    envelope, one_file, print_json, read_file, EXIT_OK, EXIT_RUNTIME, EXIT_USAGE,
};
use serde_json::json;

pub fn run_envelope_lint(args: &[String]) -> i32 {
    let usage = "rhwp-agent envelope-lint <봉투.json> [--json]";
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
    let mut missing: Vec<&str> = Vec::new();
    for key in ["schemaVersion", "command"] {
        if value
            .get(key)
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false)
        {
            continue;
        }
        missing.push(key);
    }
    let has_untrusted = value.get("untrustedContent").is_some();
    let ok = missing.is_empty();
    let payload = json!({
        "source": opts.path,
        "ok": ok,
        "missing": missing,
        "hasUntrustedContent": has_untrusted,
        "schemaVersion": value.get("schemaVersion"),
        "command": value.get("command"),
    });
    if opts.json {
        print_json(&envelope("envelope-lint", payload, &[]));
    } else if ok {
        crate::outln!("ok");
    } else {
        crate::outln!("missing {}", missing.join(","));
    }
    if ok {
        EXIT_OK
    } else {
        EXIT_USAGE
    }
}

pub fn run_nextcall(args: &[String]) -> i32 {
    let usage = "rhwp-agent nextcall <봉투.json> [--json]";
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
    let next = value.get("nextCall").cloned().unwrap_or(json!(null));
    let present = !next.is_null();
    let payload = json!({
        "source": opts.path,
        "present": present,
        "nextCall": next,
    });
    if opts.json {
        print_json(&envelope("nextcall", payload, &[]));
    } else if present {
        crate::outln!(
            "{}",
            next.get("name").and_then(|v| v.as_str()).unwrap_or("")
        );
    } else {
        crate::outln!("none");
    }
    EXIT_OK
}
