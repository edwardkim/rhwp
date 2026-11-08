//! 독립 한컴 Print의 값축 숫자를 실제 문서 출력에서 보호한다.
//! 전쪽 Native/fresh 선행 증거와 입력·PDF 출처는 자산 README를 참조한다.
#![cfg(not(target_arch = "wasm32"))]

use roxmltree::Node;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const ASSETS: &str = "mydocs/pr/assets/chart_axis_label_occupancy_20261021";
const EXPECTED: &str =
    include_str!("../../mydocs/pr/assets/chart_axis_label_occupancy_20261021/expected.json");
static NEXT_OUTPUT: AtomicUsize = AtomicUsize::new(0);

struct OutputDir(PathBuf);

impl Drop for OutputDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn number(node: Node<'_, '_>, name: &str) -> f64 {
    node.attribute(name)
        .unwrap_or_else(|| panic!("{name}: 속성 누락"))
        .parse::<f64>()
        .expect("유한 좌표")
}

fn check_axis_labels(height_hu: u32) {
    let name = format!("axis-font1800-height{height_hu}");
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    let oracle = &expected[&name];
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(ASSETS);
    let input = root.join(oracle["input"].as_str().expect("입력 경로"));
    assert!(input.is_file(), "{}", input.display());
    assert!(root.join(oracle["pdf"].as_str().unwrap()).is_file());
    let output = OutputDir(std::env::temp_dir().join(format!(
        "rhwp-axis-label-{}-{}-{}",
        std::process::id(),
        height_hu,
        NEXT_OUTPUT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir(&output.0).unwrap();
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let actual = Command::new(binary)
        .arg("export-svg")
        .arg(&input)
        .args([
            "--profile",
            "print",
            "--compat",
            "2022",
            "--backend",
            "layer",
            "-p",
            "0",
            "-o",
        ])
        .arg(&output.0)
        .arg("--json")
        .output()
        .expect("실제 문서 출력 실행");
    assert!(
        actual.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&actual.stderr)
    );
    let manifest: Value = serde_json::from_slice(&actual.stdout).unwrap();
    assert_eq!(manifest["pageCount"], oracle["pdf_pages"], "전체 쪽수");
    let pages = manifest["pages"].as_array().unwrap();
    assert_eq!(pages.len(), 1, "선택한 첫 쪽의 중복·누락");
    let svg = std::fs::read_to_string(pages[0]["path"].as_str().unwrap()).unwrap();
    let doc = roxmltree::Document::parse(&svg).unwrap();
    let charts = doc
        .descendants()
        .filter(|n| n.attribute("class") == Some("hwp-ooxml-chart"))
        .collect::<Vec<_>>();
    assert_eq!(charts.len(), 1, "차트 소유·중복");
    let chart = charts[0];
    let axes = chart
        .descendants()
        .filter(|n| n.attribute("class") == Some("hwp-chart-axis"))
        .collect::<Vec<_>>();
    assert_eq!(axes.len(), 1, "값축 하나");
    let axis = axes[0];
    let x = number(axis, "x1");
    assert_eq!(x, number(axis, "x2"), "세로 값축");
    let texts = chart
        .descendants()
        .filter(|n| n.has_tag_name("text"))
        .collect::<Vec<_>>();
    // 절대 화면 영역으로 고르지 않고 실제 값축 왼쪽의 숫자 소유를 찾는다.
    let labels = texts
        .iter()
        .copied()
        .filter(|n| {
            number(*n, "x") < x
                && n.text()
                    .is_some_and(|t| !t.is_empty() && t.chars().all(|c| c.is_ascii_digit()))
        })
        .collect::<Vec<_>>();
    let values = labels
        .iter()
        .map(|n| n.text().unwrap().parse::<u32>().unwrap())
        .collect::<Vec<_>>();
    let wanted: Vec<u32> = serde_json::from_value(oracle["expected_value_labels"].clone()).unwrap();
    assert_eq!(values, wanted, "{name}: 독립 Print의 표시 숫자·순서");
    assert!(
        labels.windows(2).all(|p| {
            number(p[0], "y") - number(p[1], "y")
                >= number(p[0], "font-size").max(number(p[1], "font-size"))
        }),
        "표시 숫자의 위아래 순서와 글자 크기 대비 비겹침"
    );
    let frame = chart
        .descendants()
        .find(|n| n.has_tag_name("rect"))
        .unwrap();
    for label in &labels {
        assert!(number(*label, "x") > number(frame, "x"));
        assert!(number(*label, "y") > number(frame, "y"));
        assert!(number(*label, "y") < number(frame, "y") + number(frame, "height"));
    }
    let categories = texts
        .iter()
        .copied()
        .filter(|n| n.text().is_some_and(|t| t.starts_with("항목 ")))
        .collect::<Vec<_>>();
    assert_eq!(
        categories
            .iter()
            .map(|n| n.text().unwrap())
            .collect::<Vec<_>>(),
        ["항목 1", "항목 2", "항목 3", "항목 4"],
        "항목 내용·순서·누락·중복"
    );
    assert!(categories
        .windows(2)
        .all(|p| number(p[0], "x") < number(p[1], "x")));
    assert!(categories
        .iter()
        .all(|n| number(*n, "y") > number(axis, "y2")));
    let legends = texts
        .iter()
        .copied()
        .filter(|n| n.text().is_some_and(|t| t.starts_with("계열 ")))
        .collect::<Vec<_>>();
    assert_eq!(
        legends
            .iter()
            .map(|n| n.text().unwrap())
            .collect::<Vec<_>>(),
        ["계열 1", "계열 2", "계열 3"],
        "범례 내용·순서·누락·중복"
    );
    assert!(legends
        .windows(2)
        .all(|p| number(p[0], "x") < number(p[1], "x")));
    assert!(legends.iter().all(|n| {
        categories.iter().all(|c| number(*n, "y") > number(*c, "y"))
            && number(*n, "y") < number(frame, "y") + number(frame, "height")
    }));
    let title = texts
        .iter()
        .filter(|n| n.attribute("class") == Some("hwp-chart-title"))
        .collect::<Vec<_>>();
    assert_eq!(title.len(), 1, "제목 중복·누락");
    assert_eq!(title[0].text(), Some("차트 제목"));
    assert!(number(*title[0], "y") < number(axis, "y1"));
}

#[test]
fn chart_axis_225_preserves_zero_and_six() {
    check_axis_labels(16875);
}

#[test]
fn chart_axis_226_preserves_zero_and_six() {
    check_axis_labels(16950);
}

#[test]
fn chart_axis_227_preserves_zero_and_six() {
    check_axis_labels(17025);
}

#[test]
fn chart_axis_228_preserves_zero_and_six() {
    check_axis_labels(17100);
}

#[test]
fn chart_axis_230_preserves_zero_and_four() {
    check_axis_labels(17250);
}

#[test]
fn chart_axis_240_preserves_zero_and_four() {
    check_axis_labels(18000);
}
