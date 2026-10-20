//! 독립 한컴 Print 전쪽 Native/fresh WASM 100%에서 확인한 테두리 관계.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{document_core::DocumentCore, model::page::PageBorderBasis};
use serde_json::Value;
use std::{path::Path, process::Command};

fn number(tag: &str, name: &str) -> f64 {
    let key = format!("{name}=\"");
    let start = tag.find(&key).unwrap() + key.len();
    tag[start..].split('"').next().unwrap().parse().unwrap()
}

fn check(basis: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let binary = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    for kind in ["solid", "double"] {
        // 독립 Print의 실제 stroke 값. renderer 분해 helper나 폭 표를 호출하지 않는다.
        for (index, single_pt, double_pt) in [(1, 0.36, 0.12), (6, 1.079, 0.24), (10, 2.878, 0.72)]
        {
            let path = root.join(format!(
                "tests/fixtures/issue_7642/page-border/{basis}-{kind}-w{index}-hancom-2020.hwp"
            ));
            let bytes = std::fs::read(&path).unwrap();
            let source = rhwp::parser::parse_document(&bytes).unwrap();
            let section = &source.sections[0].section_def;
            let page = &section.page_def;
            let pbf = &section.page_border_fill;
            let temporary = std::env::temp_dir().join(format!(
                "rhwp-border-{}-{basis}-{kind}-{index}",
                std::process::id()
            ));
            std::fs::create_dir_all(&temporary).unwrap();
            let result = Command::new(&binary)
                .args([
                    "export-svg",
                    path.to_str().unwrap(),
                    "--profile",
                    "print",
                    "-o",
                    temporary.to_str().unwrap(),
                ])
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
            let files: Vec<_> = std::fs::read_dir(&temporary)
                .unwrap()
                .map(|e| e.unwrap().path())
                .filter(|p| p.extension().is_some_and(|s| s == "svg"))
                .collect();
            assert_eq!(files.len(), 1, "독립 Print의 전체 한 쪽 보존");
            let svg = std::fs::read_to_string(&files[0]).unwrap();
            std::fs::remove_dir_all(&temporary).unwrap();
            let root_tag = svg.split('>').next().unwrap();
            // 선과 Studio 정보는 저장 용지의 논리 좌표다. Print viewport만 정수 pt로
            // 자르므로 그 물리 크기로 나눠 원문의 경계 비율을 바꾸지 않는다.
            let width = f64::from(page.width) * 96.0 / 7200.0;
            let height = f64::from(page.height) * 96.0 / 7200.0;
            for (name, source_hu) in [("width", page.width), ("height", page.height)] {
                let physical_pt = number(root_tag, name) * 72.0 / 96.0;
                let stored_pt = f64::from(source_hu) / 100.0;
                assert!(
                    (physical_pt - stored_pt.floor()).abs() < 1e-8,
                    "독립 Print의 물리 종이는 정수 point이고 조판의 논리 용지는 보존한다"
                );
            }
            let lines: Vec<_> = svg.split('<').filter(|s| s.starts_with("line ")).collect();
            let count = if kind == "double" { 2 } else { 1 };
            assert_eq!(lines.len(), count * 4, "네 변의 선 묶음 누락·중복");
            let mut horizontal: Vec<_> = lines
                .iter()
                .filter(|s| number(s, "y1") == number(s, "y2"))
                .copied()
                .collect();
            let mut vertical: Vec<_> = lines
                .iter()
                .filter(|s| number(s, "x1") == number(s, "x2"))
                .copied()
                .collect();
            assert_eq!(horizontal.len(), count * 2);
            assert_eq!(vertical.len(), count * 2);
            horizontal.sort_by(|a, b| number(a, "y1").total_cmp(&number(b, "y1")));
            vertical.sort_by(|a, b| number(a, "x1").total_cmp(&number(b, "x1")));
            let mean = |items: &[&str], key: &str| {
                items.iter().map(|s| number(s, key)).sum::<f64>() / items.len() as f64
            };
            let actual = [
                mean(&vertical[..count], "x1") / width,
                mean(&vertical[count..], "x1") / width,
                mean(&horizontal[..count], "y1") / height,
                mean(&horizontal[count..], "y1") / height,
            ];
            let paper = matches!(pbf.basis, PageBorderBasis::PaperBased);
            // Print가 증언한 관계: 종이 경계 안쪽 또는 본문 경계 바깥쪽의 저장 spacing.
            let expected = if paper {
                [
                    f64::from(pbf.spacing_left) / f64::from(page.width),
                    1.0 - f64::from(pbf.spacing_right) / f64::from(page.width),
                    f64::from(pbf.spacing_top) / f64::from(page.height),
                    1.0 - f64::from(pbf.spacing_bottom) / f64::from(page.height),
                ]
            } else {
                [
                    (f64::from(page.margin_left) - f64::from(pbf.spacing_left))
                        / f64::from(page.width),
                    1.0 - (f64::from(page.margin_right) - f64::from(pbf.spacing_right))
                        / f64::from(page.width),
                    (f64::from(page.margin_top) + f64::from(page.margin_header)
                        - f64::from(pbf.spacing_top))
                        / f64::from(page.height),
                    1.0 - (f64::from(page.margin_bottom) + f64::from(page.margin_footer)
                        - f64::from(pbf.spacing_bottom))
                        / f64::from(page.height),
                ]
            };
            for (have, want) in actual.into_iter().zip(expected) {
                assert!(
                    (have - want).abs() < 0.00001,
                    "{basis}/{kind}/{index}: 저장 spacing의 경계 소속 {have} != {want}"
                );
            }
            let oracle_pt = if kind == "double" {
                double_pt
            } else {
                single_pt
            };
            let page_pt = f64::from(page.width) / 100.0;
            for line in &lines {
                // 독립 인쇄 격자 반 칸의 차이를 종이 너비 비율로 비교한다.
                let actual_width = number(line, "stroke-width") / width;
                assert!(
                    (actual_width - oracle_pt / page_pt).abs() < 0.06 / page_pt,
                    "독립 Print의 선 굵기 비율"
                );
            }
            if kind == "double" {
                for (items, key) in [(&horizontal[..count], "y1"), (&vertical[..count], "x1")] {
                    let stroke = number(items[0], "stroke-width");
                    let separation = (number(items[1], key) - number(items[0], key)).abs() / stroke;
                    assert!(
                        (separation - 3.0).abs() < 0.02,
                        "독립 Print: 빈 간격은 선 굵기의 두 배"
                    );
                }
            }
            let core = DocumentCore::from_bytes(&bytes).unwrap();
            let info: Value = serde_json::from_str(&core.get_page_info_native(0).unwrap()).unwrap();
            let reported = [
                info["pageBorderLeft"].as_f64().unwrap() / width,
                1.0 - info["pageBorderRight"].as_f64().unwrap() / width,
                info["pageBorderTop"].as_f64().unwrap() / height,
                1.0 - info["pageBorderBottom"].as_f64().unwrap() / height,
            ];
            for (have, want) in reported.into_iter().zip(actual) {
                assert!(
                    (have - want).abs() < 0.0002,
                    "Studio 정보와 실제 선 중심의 일관성"
                );
            }
        }
    }
}

#[test]
fn body_border_keeps_spacing_ownership_and_print_stroke_relations() {
    check("body");
}
#[test]
fn paper_border_keeps_spacing_ownership_and_print_stroke_relations() {
    check("paper");
}
