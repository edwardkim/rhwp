//! 기존 인라인 이동 전후 네 차트의 실제 SVG를 독립 한컴 Print와 대조한다.
//! 전쪽 Native/fresh WASM 98% 자격·96dpi 좌표·직접 판독은 ASSETS에 기록한다.

#![cfg(not(target_arch = "wasm32"))]

use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command, time::SystemTime};

const ASSETS: &str = "mydocs/pr/assets/ooxml_chart_print_frame/independent-oracle.json";

struct Output(std::path::PathBuf);

impl Output {
    fn new() -> Self {
        Self(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("output/test-runs")
                .join(format!(
                    "ooxml-chart-frame-{}-{}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                )),
        )
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn svg(root: &Path, input: &Path, output: &Path) -> String {
    fs::create_dir_all(output).unwrap();
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(binary)
        .arg("export-svg")
        .arg(root.join(input))
        .args(["--compat", "2022", "--profile", "print", "-o"])
        .arg(output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "공개 CLI 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let pages: Vec<_> = fs::read_dir(output)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "svg"))
        .collect();
    assert_eq!(pages.len(), 1, "독립 Print와 같은 한 쪽");
    fs::read_to_string(&pages[0]).unwrap()
}

fn number(node: roxmltree::Node<'_, '_>, attribute: &str) -> f64 {
    node.attribute(attribute).unwrap().parse().unwrap()
}

fn near(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() <= 2.0,
        "독립 Print {expected}, 실제 {actual}; 허용 ±2 CSSpx"
    );
}

fn text<'a>(chart: roxmltree::Node<'a, 'a>, expected: &str) -> roxmltree::Node<'a, 'a> {
    let found: Vec<_> = chart
        .descendants()
        .filter(|node| node.has_tag_name("text"))
        .filter(|node| {
            node.descendants()
                .filter(|child| child.is_text())
                .filter_map(|child| child.text())
                .collect::<String>()
                == expected
        })
        .collect();
    assert_eq!(found.len(), 1, "차트 원문 {expected:?}의 누락·중복 없음");
    found[0]
}

fn assert_frame(source: &str, case: &Value) {
    let document = roxmltree::Document::parse(source).unwrap();
    let charts: Vec<_> = document
        .descendants()
        .filter(|node| node.attribute("class") == Some("hwp-ooxml-chart"))
        .collect();
    assert_eq!(charts.len(), 1, "이동 전후 차트 하나의 보존");
    let chart = charts[0];
    let rectangles: Vec<_> = chart
        .descendants()
        .filter(|node| node.has_tag_name("rect"))
        .collect();
    let outer = rectangles[0];
    let plot = rectangles[1];
    let (left, top) = (number(outer, "x"), number(outer, "y"));
    let (right, bottom) = (left + number(outer, "width"), top + number(outer, "height"));
    let expected = case["outer_edges_csspx"].as_array().unwrap();
    for (actual, reference) in [left, top, right, bottom].into_iter().zip(expected) {
        near(actual, reference.as_f64().unwrap());
    }
    let (x, y) = (number(plot, "x"), number(plot, "y"));
    let (end, baseline) = (x + number(plot, "width"), y + number(plot, "height"));
    near(x, case["plot_left_csspx"].as_f64().unwrap());
    near(y, case["plot_top_csspx"].as_f64().unwrap());
    near(baseline, case["plot_baseline_csspx"].as_f64().unwrap());
    assert!(
        left < x && x < end && end < right && top < y && y < baseline && baseline < bottom,
        "플롯은 실제 차트 프레임 내부"
    );
    let title = text(chart, "차트 제목");
    assert!(
        top < number(title, "y") && number(title, "y") < y,
        "제목은 외곽과 플롯 위쪽 사이"
    );
    near(number(title, "x"), (left + right) / 2.0);
    let mut categories = Vec::new();
    for index in 1..=4 {
        let label = text(chart, &format!("항목 {index}"));
        let label_x = number(label, "x");
        assert!(
            x < label_x
                && label_x < end
                && baseline < number(label, "y")
                && number(label, "y") < bottom
        );
        categories.push(label_x);
    }
    assert!(
        categories.windows(2).all(|pair| pair[0] < pair[1]),
        "항목의 왼쪽→오른쪽 순서"
    );
    let legend: Vec<_> = chart
        .descendants()
        .filter(|node| node.attribute("class") == Some("hwp-chart-legend-key"))
        .collect();
    assert_eq!(legend.len(), 3, "세 계열의 범례 보존");
    let mut previous = top;
    for (index, marker) in legend.iter().enumerate() {
        let label = text(chart, &format!("계열 {}", index + 1));
        assert!(
            end < number(*marker, "x")
                && number(*marker, "x") < number(label, "x")
                && number(label, "x") < right
                && previous < number(*marker, "y")
                && number(*marker, "y") < bottom,
            "범례는 플롯 오른쪽·차트 내부에서 원문 순서"
        );
        previous = number(*marker, "y");
    }
    let bars: Vec<_> = rectangles
        .iter()
        .copied()
        .skip(2)
        .filter(|node| node.attribute("class").is_none())
        .collect();
    assert_eq!(bars.len(), 12, "네 항목 × 세 계열의 누락·중복 없음");
    for bar in bars {
        let (bx, by) = (number(bar, "x"), number(bar, "y"));
        assert!(
            x <= bx
                && bx + number(bar, "width") <= end
                && y <= by
                && by < baseline
                && number(bar, "width") > 0.0,
            "막대는 플롯 내부에서 양의 폭·높이"
        );
        near(by + number(bar, "height"), baseline);
    }
}

#[test]
fn hwp_and_hwpx_moved_charts_keep_print_frame_axes_and_content_relations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let oracle: Value = serde_json::from_slice(&fs::read(root.join(ASSETS)).unwrap()).unwrap();
    assert_eq!(oracle["dpi"], 96);
    let output = Output::new();
    let cases = oracle["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 4);
    for case in cases {
        for kind in ["input", "pdf"] {
            let bytes = fs::read(root.join(case[kind].as_str().unwrap())).unwrap();
            let digest: String = Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect();
            assert_eq!(
                digest,
                case[format!("{kind}_sha256")].as_str().unwrap(),
                "검증한 원본 바이트 유지"
            );
        }
        let source = svg(
            root,
            Path::new(case["input"].as_str().unwrap()),
            &output.0.join(case["key"].as_str().unwrap()),
        );
        assert_frame(&source, case);
    }
}
