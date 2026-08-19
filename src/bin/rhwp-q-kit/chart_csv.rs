//! 차트 데이터 CSV — `chart_extract::collect_charts` + `chart_csv::to_csv`.
//!
//! `--chart N` 은 1-based (`ChartRef::index + 1`).

use crate::envelope::{
    envelope, load_core, parse_json_string, parse_usize, print_json, write_stdout, EXIT_RUNTIME,
    EXIT_USAGE,
};
use rhwp::document_core::queries::chart_csv::to_csv;
use rhwp::document_core::queries::chart_extract::collect_charts;
use serde_json::{json, Value};

const USAGE: &str = "rhwp-q-kit chart-csv <파일> --chart <N> [--json]";

fn strings(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|a| {
            a.iter()
                .map(|x| x.as_str().unwrap_or_default().to_string())
                .collect()
        })
        .unwrap_or_default()
}

pub fn run(args: &[String]) -> i32 {
    let mut json_mode = false;
    let mut path: Option<String> = None;
    let mut chart: Option<usize> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--json" => {
                json_mode = true;
                i += 1;
            }
            "--chart" => {
                let Some(raw) = args.get(i + 1) else {
                    eprintln!("오류: --chart 뒤에 1 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {USAGE}");
                    return EXIT_USAGE;
                };
                let n = match parse_usize("--chart", raw) {
                    Ok(v) => v,
                    Err(c) => return c,
                };
                if n < 1 {
                    eprintln!("오류: --chart 뒤에 1 이상의 정수가 필요합니다.");
                    eprintln!("사용법: {USAGE}");
                    return EXIT_USAGE;
                }
                chart = Some(n);
                i += 2;
            }
            other if other.starts_with('-') => {
                eprintln!("오류: 알 수 없는 옵션입니다 - {other}");
                eprintln!("사용법: {USAGE}");
                return EXIT_USAGE;
            }
            other => {
                if path.is_some() {
                    eprintln!("오류: 파일이 너무 많습니다 - {other}");
                    eprintln!("사용법: {USAGE}");
                    return EXIT_USAGE;
                }
                path = Some(other.to_string());
                i += 1;
            }
        }
    }
    let (Some(path), Some(chart)) = (path, chart) else {
        eprintln!("오류: 파일과 --chart 가 필요합니다.");
        eprintln!("사용법: {USAGE}");
        return EXIT_USAGE;
    };
    let core = match load_core(&path) {
        Ok(c) => c,
        Err(c) => return c,
    };
    let charts = collect_charts(core.document());
    let index = chart - 1;
    if index >= charts.len() {
        eprintln!(
            "오류: 차트 {} 번이 없습니다 (차트 {}개).",
            chart,
            charts.len()
        );
        return EXIT_RUNTIME;
    }
    let raw = match core.get_chart_data_by_index_native(index) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("오류: 차트 {chart} 데이터를 읽지 못했습니다 - {e}");
            return EXIT_RUNTIME;
        }
    };
    let read = match parse_json_string(&raw) {
        Ok(v) => v,
        Err(c) => return c,
    };
    if read["ok"] != true {
        eprintln!(
            "오류: 차트 {chart} 를 읽을 수 없습니다 - {}",
            read["invalid"][0]["message"]
                .as_str()
                .unwrap_or("사유 미상")
        );
        return EXIT_RUNTIME;
    }
    if read["labelsShared"] != true {
        let axis = if read["axis"].as_str() == Some("scatter") {
            "X 값"
        } else {
            "카테고리 라벨"
        };
        eprintln!(
            "오류: 차트 {chart} 는 계열마다 {axis}이 달라 CSV 한 열로 안전하게 표현할 수 없습니다."
        );
        return EXIT_RUNTIME;
    }
    let labels = strings(&read["labels"]);
    let series = read["series"].as_array().cloned().unwrap_or_default();
    let names: Vec<String> = series
        .iter()
        .map(|s| s["name"].as_str().unwrap_or_default().to_string())
        .collect();
    let values: Vec<Vec<String>> = series.iter().map(|s| strings(&s["values"])).collect();
    let scatter = read["axis"].as_str() == Some("scatter");
    let csv = to_csv(&labels, &names, &values, scatter);
    let rows = values
        .iter()
        .map(|s| s.len())
        .chain(std::iter::once(labels.len()))
        .max()
        .unwrap_or(0);
    if json_mode {
        print_json(&envelope(
            "chart-csv",
            json!({
                "source": path,
                "chart": chart,
                "rowCount": rows,
                "colCount": names.len(),
                "scatter": scatter,
                "csv": csv,
            }),
            &["csv"],
        ))
    } else {
        write_stdout(&csv)
    }
}
