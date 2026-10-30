//! 한컴 2024 Print의 9~48pt 아래 일점쇄선과 원본 글자 모양 구간의 위상을 보존한다.
//! 같은 HWP의 Native·fresh WASM·CanvasKit 전쪽이 시각 검증을 통과한 뒤 추가했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::{paint::RenderProfile, DocumentCore};
use serde_json::Value;

fn number(node: roxmltree::Node<'_, '_>, name: &str) -> f64 {
    node.attribute(name).unwrap().parse().unwrap()
}

fn near(actual: f64, expected: f64, tolerance: f64) {
    assert!(
        (actual - expected).abs() <= tolerance,
        "실제 {actual}, 독립 기대값 {expected}, 허용 오차 {tolerance}"
    );
}

fn bottom_ops<'a>(value: &'a Value, output: &mut Vec<&'a Value>) {
    match value {
        Value::Object(fields) => {
            if value["type"] == "textDecoration" && value["decoration"]["underline"] == "bottom" {
                output.push(value);
            }
            for child in fields.values() {
                bottom_ops(child, output);
            }
        }
        Value::Array(children) => {
            for child in children {
                bottom_ops(child, output);
            }
        }
        _ => {}
    }
}

#[test]
fn printed_dash_dot_size_and_source_phase_reach_svg_and_layer_json() {
    let input =
        include_bytes!("../../mydocs/pr/assets/raised-decoration-hancom/top-probe-shape-3.hwp");
    let mut core = DocumentCore::from_bytes(input).unwrap();
    core.set_hangul2024_compat(true);
    assert_eq!(core.page_count(), 1);
    let rendered = core
        .render_page_svg_layer_with_profile_native(0, RenderProfile::Print)
        .unwrap();
    let svg = roxmltree::Document::parse(&rendered).unwrap();
    let paint: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_profile_native(0, RenderProfile::Print)
            .unwrap(),
    )
    .unwrap();
    let mut ops = Vec::new();
    bottom_ops(&paint, &mut ops);
    assert_eq!(ops.len(), 16, "8개 크기마다 영문과 한글 run을 보존한다");

    // pdf/raised-decoration-hancom/top-probe-shape-3-hancom-print.pdf의 경로 좌표를 96dpi로 환산했다.
    // (pt, 굵기, 긴 획, 간격, 점, 기준선 아래 거리). 긴 획은 600dpi 한 칸 안에서 비교한다.
    let printed = [
        (9.0, 0.48, 6.876, 1.920, 0.6387, 1.9179),
        (10.0, 0.48, 7.676, 2.2387, 0.800, 2.2367),
        (12.0, 0.64, 9.116, 2.8773, 0.960, 2.7171),
        (15.0, 0.80, 11.5147, 3.5173, 1.120, 3.3557),
        (18.0, 0.96, 13.7533, 4.1573, 1.4387, 4.1549),
        (24.0, 1.2787, 18.3907, 5.4373, 1.7587, 5.4335),
        (36.0, 1.9187, 27.3453, 8.316, 2.7187, 8.1508),
        (48.0, 2.5587, 36.620, 10.8747, 3.6787, 10.7079),
    ];
    for (pt, width, long, gap, dot, offset) in printed {
        let size = pt * 4.0 / 3.0;
        let baseline = svg
            .descendants()
            .find(|node| {
                node.has_tag_name("text")
                    && node.text() == Some("B")
                    && (number(*node, "font-size") - size).abs() < 1e-6
            })
            .map(|node| number(node, "y"))
            .expect("각 크기의 원문 B 글리프");
        let mut lines: Vec<_> = svg
            .descendants()
            .filter(|node| {
                node.has_tag_name("line")
                    && node.attribute("stroke") == Some("#0000cc")
                    && (number(*node, "y1") - baseline).abs() < size
            })
            .collect();
        lines.sort_by(|a, b| number(*a, "x1").total_cmp(&number(*b, "x1")));
        assert_eq!(lines.len(), 2, "{pt}pt 원문 언어별 run");
        let origin = number(lines[0], "x1");
        let mut row_ops: Vec<_> = ops
            .iter()
            .copied()
            // 레이어 JSON의 fontSize는 소수 셋째 자리까지 기록한다.
            .filter(|op| (op["decoration"]["fontSize"].as_f64().unwrap() - size).abs() < 0.0005)
            .collect();
        row_ops.sort_by(|a, b| {
            a["bbox"]["x"]
                .as_f64()
                .unwrap()
                .total_cmp(&b["bbox"]["x"].as_f64().unwrap())
        });
        assert_eq!(row_ops.len(), lines.len());
        for (line, op) in lines.into_iter().zip(row_ops) {
            let dash: Vec<f64> = line
                .attribute("stroke-dasharray")
                .unwrap()
                .split_whitespace()
                .map(|value| value.parse().unwrap())
                .collect();
            assert_eq!(dash.len(), 4);
            for (index, (actual, expected)) in dash.iter().zip([long, gap, dot, gap]).enumerate() {
                // 간격의 한 칸 오차도 줄 끝에 불필요한 획을 만들므로 독립 Print의 정밀도를 유지한다.
                near(*actual, expected, if index == 0 { 0.1601 } else { 0.01 });
            }
            near(number(line, "stroke-width"), width, 0.1601);
            near(number(line, "y1") - baseline, offset, size * 0.02);
            let phase = number(line, "stroke-dashoffset");
            near(phase, number(line, "x1") - origin, 1e-6);
            let geometry = &op["decoration"]["topUnderline"];
            near(geometry["phase"].as_f64().unwrap(), phase, 1e-6);
            assert!(geometry["dots"].is_null());
            let strokes = geometry["strokes"].as_array().unwrap();
            assert_eq!(strokes.len(), 1);
            near(
                strokes[0]["width"].as_f64().unwrap(),
                number(line, "stroke-width"),
                1e-6,
            );
            near(
                strokes[0]["offset"].as_f64().unwrap(),
                number(line, "y1") - baseline,
                1e-6,
            );
            let json_dash = strokes[0]["dash"].as_array().unwrap();
            assert_eq!(json_dash.len(), dash.len());
            for (actual, expected) in json_dash.iter().zip(dash) {
                near(actual.as_f64().unwrap(), expected, 1e-6);
            }
        }
    }
}
