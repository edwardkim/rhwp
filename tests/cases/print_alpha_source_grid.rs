//! 연속 알파 그림의 원본 줄 소유와 독립 한컴 Print 운반체를 함께 보호한다.
//! Native/fresh WASM 양쪽 98% 이상과 직접 판독을 마친 두 입력만 사용한다.
//! 증거: mydocs/pr/assets/pr_8032_print_alpha/README.md

#![cfg(not(target_arch = "wasm32"))]

use base64::Engine;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command, time::SystemTime};

fn sha(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn export(root: &Path, input: &Path, output: &Path, profile: &str) -> String {
    fs::create_dir_all(output).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .current_dir(root)
        .arg("export-svg")
        .arg(input)
        .args(["--profile", profile, "--compat", "2022", "-o"])
        .arg(output)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "실제 paint SVG 출력 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let files: Vec<_> = fs::read_dir(output)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|value| value == "svg"))
        .collect();
    assert_eq!(files.len(), 1, "독립 Print와 같은 한 쪽");
    fs::read_to_string(&files[0]).unwrap()
}

fn image<'a>(doc: &'a roxmltree::Document<'a>) -> roxmltree::Node<'a, 'a> {
    let images: Vec<_> = doc
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "image")
        .collect();
    assert_eq!(images.len(), 1, "그림 누락·중복 없음");
    images[0]
}

fn png(node: roxmltree::Node<'_, '_>) -> image::RgbaImage {
    let href = node
        .attributes()
        .find(|attribute| attribute.name() == "href")
        .unwrap()
        .value();
    let data = href.strip_prefix("data:image/png;base64,").unwrap();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data)
        .unwrap();
    image::load_from_memory(&bytes).unwrap().to_rgba8()
}

#[test]
fn continuous_alpha_print_keeps_text_order_and_independent_carrier_frame() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let oracle: Value = serde_json::from_slice(
        &fs::read(root.join("mydocs/pr/assets/pr_8032_print_alpha/independent-alpha-oracle.json"))
            .unwrap(),
    )
    .unwrap();
    assert_eq!(oracle["dpi"], 96);
    let output = root.join("output/test-runs").join(format!(
        "print-alpha-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    for case in oracle["cases"].as_array().unwrap() {
        let label = case["label"].as_str().unwrap();
        let input = root.join(case["input"].as_str().unwrap());
        for kind in ["input", "pdf"] {
            assert_eq!(
                sha(&fs::read(root.join(case[kind].as_str().unwrap())).unwrap()),
                case[format!("{kind}_sha256")].as_str().unwrap(),
                "독립 기준과 같은 파일"
            );
        }
        let svg = export(root, &input, &output.join(label).join("print"), "print");
        let doc = roxmltree::Document::parse(&svg).unwrap();
        let painted = image(&doc);
        // 기대값은 PDF의 XObject CTM에서 직접 읽었으며 구현 계산식은 재인용하지 않는다.
        for (index, attribute) in ["x", "y", "width", "height"].iter().enumerate() {
            let actual: f64 = painted.attribute(*attribute).unwrap().parse().unwrap();
            let expected = case["pdf_frame_pt"][index].as_f64().unwrap() * 96.0 / 72.0;
            assert!(
                (actual - expected).abs() <= 2.0,
                "{label} 독립 Print {attribute}: 실제 {actual}, 기준 {expected}; 허용 ±2px"
            );
        }
        let texts: Vec<_> = doc
            .descendants()
            .filter(|node| node.is_element() && node.tag_name().name() == "text")
            .collect();
        let content: String = texts
            .iter()
            .flat_map(|node| {
                node.descendants()
                    .filter_map(|part| part.text().filter(|_| part.is_text()))
            })
            .collect();
        assert_eq!(
            content, "가나다라마바사아자차카타파하",
            "원문 순서·내용 보존"
        );
        assert!(texts
            .iter()
            .all(|node| node.range().end < painted.range().start));
        let xs: Vec<f64> = texts
            .iter()
            .map(|node| node.attribute("x").unwrap().parse().unwrap())
            .collect();
        assert!(
            xs.windows(2).all(|pair| pair[0] < pair[1]),
            "앞 문장 읽기 순서"
        );
        if label == "wide" {
            let top: f64 = painted.attribute("y").unwrap().parse().unwrap();
            assert!(texts
                .iter()
                .all(|node| node.attribute("y").unwrap().parse::<f64>().unwrap() < top));
        }
        let pixels = png(painted);
        assert_eq!(
            pixels.width(),
            case["mask_pixels"][0].as_u64().unwrap() as u32
        );
        assert_eq!(
            pixels.height(),
            case["mask_pixels"][1].as_u64().unwrap() as u32
        );
        assert!(pixels.pixels().all(|pixel| matches!(pixel[3], 0 | 255)));
        assert!(pixels.pixels().any(|pixel| pixel[3] == 0));
        assert!(pixels.pixels().any(|pixel| pixel[3] == 255));
        let screen = export(root, &input, &output.join(label).join("screen"), "screen");
        let screen = roxmltree::Document::parse(&screen).unwrap();
        assert_eq!(
            png(image(&screen)).dimensions(),
            (16, 16),
            "화면 프로필의 원본 표본 보존"
        );
        assert_eq!(
            sha(&fs::read(&input).unwrap()),
            case["input_sha256"].as_str().unwrap()
        );
    }
}
