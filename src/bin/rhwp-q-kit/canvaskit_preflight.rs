//! canvaskit-preflight — `get_canvaskit_document_preflight_native`. 조회만 한다.

use rhwp::paint::RenderProfile;
use serde_json::json;

use crate::envelope::{
    envelope, load_core, parse_json_string, print_json, write_stdout, EXIT_RUNTIME, EXIT_USAGE,
};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str =
        "rhwp-q-kit canvaskit-preflight <파일> [--mode default|compat] [--profile screen|print|high-quality|fast-preview] [--json]";
    let mut path = None;
    let mut json_mode = false;
    let mut mode = "default".to_string();
    let mut profile = RenderProfile::Screen;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => json_mode = true,
            "--mode" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!("오류: --mode 뒤에 값이 필요합니다 (default|compat).");
                    return EXIT_USAGE;
                };
                match v.trim().to_ascii_lowercase().as_str() {
                    "" | "default" | "compat" | "compatibility" => mode = v.clone(),
                    _ => {
                        eprintln!(
                            "오류: 지원하지 않는 CanvasKit replay mode입니다: {v}. allowed modes: default, compat"
                        );
                        return EXIT_USAGE;
                    }
                }
            }
            "--profile" => {
                i += 1;
                let Some(v) = args.get(i) else {
                    eprintln!(
                        "오류: --profile 뒤에 값이 필요합니다 (screen|print|high-quality|fast-preview)."
                    );
                    return EXIT_USAGE;
                };
                match RenderProfile::parse(v) {
                    Some(p) => profile = p,
                    None => {
                        eprintln!(
                            "오류: --profile 값이 올바르지 않습니다 (screen|print|high-quality|fast-preview)."
                        );
                        return EXIT_USAGE;
                    }
                }
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {USAGE}");
                return EXIT_USAGE;
            }
            other => {
                if path.replace(other.to_string()).is_some() {
                    eprintln!("오류: 입력 파일은 하나만 지정할 수 있습니다: {other}");
                    return EXIT_USAGE;
                }
            }
        }
        i += 1;
    }
    let Some(path) = path else {
        eprintln!("오류: 파일 경로가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let raw = match core.get_canvaskit_document_preflight_native(&mode, profile) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: CanvasKit 사전점검 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let preflight = match parse_json_string(&raw) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let payload = json!({
        "source": path,
        "mode": mode,
        "preflight": preflight,
    });
    if json_mode {
        print_json(&envelope("canvaskit-preflight", payload, &["preflight"]))
    } else {
        let status = payload["preflight"]["status"].as_str().unwrap_or("unknown");
        write_stdout(&format!("status={status}"))
    }
}
