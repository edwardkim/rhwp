//! 파일을 열지 않고도 할 수 있는 바이트 검사.

use crate::envelope::{
    envelope, format_token, hex_hash, one_file, print_json, read_file, EXIT_OK, EXIT_RUNTIME,
};
use serde_json::json;

pub fn run_hash(args: &[String]) -> i32 {
    let usage = "rhwp-agent hash <파일> [--json]";
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
    let hash = hex_hash(&data);
    let payload = json!({ "source": opts.path, "bytes": data.len(), "hash": hash });
    if opts.json {
        print_json(&envelope("hash", payload, &[]));
    } else {
        crate::outln!("{hash}");
    }
    EXIT_OK
}

pub fn run_size(args: &[String]) -> i32 {
    let usage = "rhwp-agent size <파일> [--json]";
    let opts = match one_file(args, usage) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let meta = match std::fs::metadata(&opts.path) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("오류: 파일을 읽을 수 없습니다 - {}: {e}", opts.path);
            return EXIT_RUNTIME;
        }
    };
    let payload = json!({ "source": opts.path, "bytes": meta.len() });
    if opts.json {
        print_json(&envelope("size", payload, &[]));
    } else {
        crate::outln!("{}", meta.len());
    }
    EXIT_OK
}

pub fn run_magic(args: &[String]) -> i32 {
    let usage = "rhwp-agent magic <파일> [--json]";
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
    let n = data.len().min(16);
    let hex: String = data[..n]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    let format = format_token(rhwp::parser::detect_format(&data));
    let payload = json!({
        "source": opts.path,
        "bytes": data.len(),
        "headHex": hex,
        "format": format,
    });
    if opts.json {
        print_json(&envelope("magic", payload, &[]));
    } else {
        crate::outln!("{format}\t{hex}");
    }
    EXIT_OK
}
