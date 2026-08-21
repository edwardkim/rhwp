//! Task #4962 W3 developer-only isolated font metric coverage worker.
//!
//! This example deliberately exposes no product CLI surface.  The Node supervisor
//! starts one copy per document under OS resource limits and accepts stdout only as
//! a machine-readable, de-identified envelope.

use rhwp::document_core::DocumentCore;
use rhwp::parser::{detect_format, FileFormat};
use serde_json::json;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;

const MAX_INPUT_BYTES: u64 = 257 * 1024 * 1024;
const DOCUMENT_FAILURE_EXIT: i32 = 20;
const INVOCATION_FAILURE_EXIT: i32 = 64;

struct Arguments {
    input: PathBuf,
    options_json: String,
}

fn arguments() -> Option<Arguments> {
    let mut values = std::env::args_os().skip(1);
    let mut input = None;
    let mut options_json = None;
    while let Some(option) = values.next() {
        match option.to_str()? {
            "--input" if input.is_none() => input = values.next().map(PathBuf::from),
            "--options-json" if options_json.is_none() => {
                options_json = values.next().and_then(|value| value.into_string().ok())
            }
            _ => return None,
        }
    }
    Some(Arguments {
        input: input?,
        options_json: options_json.unwrap_or_else(|| "{}".to_string()),
    })
}

fn failure(reason: &'static str) -> ! {
    let envelope = json!({
        "schemaVersion": 1,
        "kind": "font-metric-coverage-worker-result",
        "status": "failed",
        "failure": reason,
    });
    println!("{envelope}");
    std::process::exit(DOCUMENT_FAILURE_EXIT);
}

fn read_bounded(input: &PathBuf) -> Result<Vec<u8>, io::Error> {
    let mut bytes = Vec::new();
    File::open(input)?
        .take(MAX_INPUT_BYTES + 1)
        .read_to_end(&mut bytes)?;
    Ok(bytes)
}

fn main() {
    let Some(arguments) = arguments() else {
        std::process::exit(INVOCATION_FAILURE_EXIT);
    };

    let bytes = match read_bounded(&arguments.input) {
        Ok(bytes) if bytes.len() as u64 <= MAX_INPUT_BYTES => bytes,
        Ok(_) => failure("resource-limit"),
        Err(_) => failure("parser"),
    };

    match detect_format(&bytes) {
        FileFormat::Hwp | FileFormat::Hwpx => {}
        FileFormat::Empty => failure("empty"),
        FileFormat::DrmProtected => failure("drm"),
        FileFormat::Hwp3 | FileFormat::Hml | FileFormat::Unknown => failure("unsupported"),
    }

    let core = match DocumentCore::from_bytes(&bytes) {
        Ok(core) => core,
        Err(error) => {
            let message = error.to_string();
            if message.contains("비밀번호가 필요한 암호 문서") || message.contains("암호화된 문서")
            {
                failure("encrypted");
            }
            failure("parser");
        }
    };

    match core.get_font_metric_coverage_analysis_native(&arguments.options_json) {
        Ok(aggregate) => println!("{aggregate}"),
        Err(error) => {
            let message = error.to_string();
            if message.contains("[RESOURCE_LIMIT_EXCEEDED]") {
                failure("resource-limit");
            }
            if message.contains("[ANALYSIS_CANCELLED]") {
                failure("cancelled");
            }
            failure("parser");
        }
    }
}
