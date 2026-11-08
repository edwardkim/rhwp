//! 독립 한컴 Print의 실제 공급자와 자연 전진을 보존한다.
//! 세 입력 전쪽 Native/fresh raw100%와 같은 좌표의 확대 판독을 확인했다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::wasm_api::HwpDocument;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::path::Path;

const ASSETS: &str = "mydocs/pr/assets/verified_point_suppliers_20261025";
const EXPECTED: &str = include_str!(
    "../../mydocs/pr/assets/verified_point_suppliers_20261025/independent_coordinates.json"
);

fn check(key: &str) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    let expected = &expected[key];
    let input = std::fs::read(root.join(ASSETS).join("inputs").join(format!("{key}.hwpx")))
        .expect("같은 입력의 전체 쪽 Native/fresh raw100% 근거");
    assert_eq!(
        Sha256::digest(&input)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        expected["input_sha256"].as_str().unwrap(),
        "시각 검증 입력을 다른 바이트로 교체하지 않는다"
    );
    let pdf = std::fs::read(root.join(format!("pdf/verified-point-suppliers/{key}-2020.pdf")))
        .expect("독립 한컴 Print PDF");
    assert_eq!(
        Sha256::digest(pdf)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        expected["pdf_sha256"].as_str().unwrap()
    );
    let doc = HwpDocument::from_bytes(&input).unwrap();
    assert_eq!(doc.page_count(), 1, "동일 Print의 전체 쪽수");
    let svg = doc.render_page_svg_native(0).unwrap();
    let xml = roxmltree::Document::parse(&svg).unwrap();
    let visible: Vec<_> = xml
        .descendants()
        .filter(|node| {
            node.is_element()
                && node.tag_name().name() == "text"
                && node.attribute("fill-opacity") != Some("0")
        })
        .collect();
    let start = visible
        .iter()
        .rposition(|node| node.text() == Some("A"))
        .expect("세 입력에 공통인 원문 대조 줄");
    let line = &visible[start..];
    let text: String = line
        .iter()
        .filter_map(|node| node.text())
        .flat_map(str::chars)
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert_eq!(
        text, "A․BC.D‧E",
        "원으로 그리는 U+00B7을 제외한 원문 순서·기호·뒤 글자를 한 번씩 보존"
    );
    for pair in line.windows(2) {
        let previous: f64 = pair[0].attribute("x").unwrap().parse().unwrap();
        let next: f64 = pair[1].attribute("x").unwrap().parse().unwrap();
        assert!(
            previous < next,
            "기호와 뒤 글자의 원문 순서가 뒤집히지 않는다"
        );
        assert_eq!(
            pair[0].attribute("y"),
            pair[1].attribute("y"),
            "독립 Print처럼 같은 줄의 글자들은 기준선을 공유한다"
        );
    }
    for ch in ['․', '‧', 'B', 'C', 'E'] {
        let text = ch.to_string();
        let actual: Vec<_> = visible
            .iter()
            .filter(|node| node.text() == Some(text.as_str()))
            .collect();
        let reference: Vec<_> = expected["glyphs"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|glyph| glyph["text"] == text)
            .collect();
        assert!(!reference.is_empty());
        assert_eq!(
            actual.len(),
            reference.len(),
            "기호와 뒤 글자를 한 번씩 보존: {ch}"
        );
        for (node, glyph) in actual.into_iter().zip(reference) {
            for (attr, field) in [("x", "x_css96"), ("y", "baseline_css96")] {
                let actual: f64 = node.attribute(attr).unwrap().parse().unwrap();
                let independent = glyph[field].as_f64().unwrap();
                assert!(
                    (actual - independent).abs() <= 2.0,
                    "양쪽 raw98% 이상인 1쪽의 독립 96dpi 좌표: {key} {ch} {attr}={actual}, Print={independent}"
                );
            }
            if matches!(ch, '․' | '‧') {
                let family = node.attribute("font-family").unwrap();
                let supplier = expected["glyph_supplier"].as_str().unwrap();
                assert!(
                    family.contains(supplier),
                    "측정과 같은 실제 공급자: {family}, {supplier}"
                );
                if expected["point_program_is_primary"] == true {
                    let primary = family
                        .split(',')
                        .next()
                        .unwrap()
                        .trim()
                        .trim_matches(|ch| ch == '\'' || ch == '"');
                    assert!(
                        !primary.contains("Haansoft"),
                        "원 프로그램의 실제 hit를 대체하지 않는다"
                    );
                }
            }
        }
    }
    assert_eq!(
        xml.descendants()
            .filter(|node| node.is_element() && node.tag_name().name() == "circle")
            .count(),
        1,
        "U+00B7만 기존 벡터 경로이며 두 별개 기호를 무조건 원으로 바꾸지 않는다"
    );
    assert!(
        !svg.contains("<ellipse "),
        "확인된 자연 글리프를 전각 휴리스틱 윤곽으로 바꾸지 않는다"
    );
}

#[test]
fn verified_arial_missing_points_keep_print_supplier_and_following_glyphs() {
    check("arial");
}

#[test]
fn verified_gulim_missing_points_keep_print_supplier_and_following_glyphs() {
    check("gulim");
}

#[test]
fn verified_hcr_owned_points_keep_their_original_font_program() {
    check("hcr-batang");
}
