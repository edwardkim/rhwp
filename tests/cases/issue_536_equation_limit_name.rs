//! #536: 실물 극한 이름의 글꼴·em 크기·내용 순서를 보호한다.
//! 독립 한컴 Print와 Native/fresh WASM 전쪽 증거는 통합 검토 4/5단계에 연결한다.
//! 98% 미만 페이지이므로 절대 배치 좌표를 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use std::path::{Path, PathBuf};
use std::process::Command;

struct OutputDirectory(PathBuf);

impl Drop for OutputDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn temporary_output() -> OutputDirectory {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("rhwp-limit-name-{}-{nonce}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    OutputDirectory(directory)
}

fn origin(node: roxmltree::Node<'_, '_>) -> (f64, f64) {
    if let Some(transform) = node.attribute("transform") {
        let values: Vec<f64> = transform
            .strip_prefix("matrix(")
            .and_then(|value| value.strip_suffix(')'))
            .expect("이름 글리프의 affine 좌표")
            .split(|ch: char| ch.is_whitespace() || ch == ',')
            .filter(|value| !value.is_empty())
            .map(|value| value.parse().unwrap())
            .collect();
        assert_eq!(values.len(), 6);
        (values[4], values[5])
    } else {
        (
            node.attribute("x").unwrap().parse().unwrap(),
            node.attribute("y").unwrap().parse().unwrap(),
        )
    }
}

#[test]
fn hancom_limit_name_keeps_print_face_em_and_scalar_order() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/equation-lim.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(&source).unwrap()).unwrap();
    let equations: Vec<_> = core
        .document()
        .sections
        .iter()
        .flat_map(|section| &section.paragraphs)
        .flat_map(|paragraph| &paragraph.controls)
        .filter_map(|control| match control {
            Control::Equation(equation) => Some(equation),
            _ => None,
        })
        .collect();
    assert_eq!(equations.len(), 1, "검증한 원본의 단일 수식");
    assert!(equations[0].script.contains("lim"));
    assert!(equations[0].font_name.eq_ignore_ascii_case("HYhwpEQ"));
    let body_font_size = f64::from(equations[0].font_size) * 96.0 / 7200.0;

    let output = temporary_output();
    let executable = std::env::var("CARGO_BIN_EXE_rhwp")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_owned());
    let result = Command::new(executable)
        .arg("export-svg")
        .arg(&source)
        .args([
            "--profile",
            "print",
            "--compat",
            "2022",
            "--json",
            "--output",
        ])
        .arg(&output.0)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "원본 SVG 출력 실패: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let svg = std::fs::read_to_string(output.0.join("equation-lim.svg")).unwrap();
    let tree = roxmltree::Document::parse(&svg).unwrap();
    let names: Vec<_> = tree
        .descendants()
        .filter(|node| {
            node.is_element()
                && node.tag_name().name() == "text"
                && matches!(node.text(), Some("l" | "i" | "m" | "lim"))
        })
        .collect();
    let text: String = names.iter().filter_map(|node| node.text()).collect();
    assert_eq!(text, "lim", "이름의 누락·중복 없이 원문 scalar 순서 보존");
    let mut previous: Option<(f64, f64)> = None;
    for node in names {
        assert!(
            node.attribute("font-family")
                .is_some_and(|family| family.split(',').any(|face| face
                    .trim()
                    .trim_matches(['\'', '"'])
                    .eq_ignore_ascii_case("HyhwpEQ"))),
            "독립 Print의 HyhwpEQ ASCII 이름을 일반 수학 글꼴로 대체하면 안 된다"
        );
        let size: f64 = node.attribute("font-size").unwrap().parse().unwrap();
        // 독립 Print의 Tf 110/92. 드라이버 정수 반올림을 포함한 em 비율 계약이다.
        assert!((size / body_font_size - 110.0 / 92.0).abs() < 0.01);
        let position = origin(node);
        if let Some((x, baseline)) = previous {
            assert!(position.0 > x, "이름의 글리프 순서는 앞으로 진행한다");
            assert!((position.1 - baseline).abs() < 0.001, "이름의 기준선 공유");
        }
        previous = Some(position);
    }
}
