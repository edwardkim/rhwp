//! chart-list — `chart_extract::collect_charts` 조회. 편집 없음.

use rhwp::document_core::queries::chart_extract::collect_charts;
use serde_json::json;

use crate::envelope::{
    envelope, load_core, parse_one_file, print_json, write_stdout, EXIT_RUNTIME,
};

pub fn run(args: &[String]) -> i32 {
    const USAGE: &str = "rhwp-q-kit chart-list <파일> [--json]";
    let opts = match parse_one_file(args, USAGE) {
        Ok(v) => v,
        Err(c) => return c,
    };
    let core = match load_core(&opts.path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let charts = collect_charts(core.document());
    let items = match serde_json::to_value(&charts) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("오류: 차트 목록 JSON 직렬화 실패 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let payload = json!({
        "source": opts.path,
        "chartCount": charts.len(),
        "charts": items,
    });
    if opts.json {
        print_json(&envelope("chart-list", payload, &["charts"]))
    } else {
        write_stdout(&format!("charts={}", charts.len()))
    }
}
