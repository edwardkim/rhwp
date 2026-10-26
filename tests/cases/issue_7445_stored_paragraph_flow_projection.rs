//! #7445: 저장 문단의 로컬 전진과 확정된 출력 축의 끝점을 구분한다.
//! 독립 Print 원점과 원문 소유·간격 관계를 함께 검증한다.
#![cfg(not(target_arch = "wasm32"))]

mod prep {
    use rhwp::{
        paint::{LayerJsonOptions, RenderProfile},
        DocumentCore,
    };
    use serde_json::Value;
    fn collect<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
        match v {
            Value::Object(m) => {
                if m.get("type").and_then(Value::as_str) == Some("textRun") {
                    out.push(v)
                }
                for v in m.values() {
                    collect(v, out)
                }
            }
            Value::Array(a) => {
                for v in a {
                    collect(v, out)
                }
            }
            _ => {}
        }
    }
    #[test]
    fn prep_post_table_body_preserves_independent_print_when_prior_source_axis_is_invalid() {
        let mut c = DocumentCore::from_bytes(
            &std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/samples/issue2006/1790387_prep_final_report.hwpx"
            ))
            .unwrap(),
        )
        .unwrap();
        c.set_hangul2024_compat(true);
        assert_eq!(c.page_count(), 140);
        let value: Value = serde_json::from_str(
            &c.get_page_layer_tree_with_options_native(
                69,
                RenderProfile::Print,
                LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
        )
        .unwrap();
        let mut runs = vec![];
        collect(&value["root"], &mut runs);
        let owned: Vec<_> = runs
            .iter()
            .filter(|r| {
                r["source"]["stableSourceKey"] == "section:2/para:38/char:1"
                    && r["text"]
                        .as_str()
                        .is_some_and(|t| t.starts_with("트랜스젠더군이"))
            })
            .collect();
        assert_eq!(owned.len(), 1, "원문 첫 어절은 한 번만 배치한다");
        let source = &c.document().sections[2].paragraphs[38].text;
        let rendered: String = runs
            .iter()
            .filter(|r| {
                r["source"]["stableSourceKey"]
                    .as_str()
                    .is_some_and(|s| s.starts_with("section:2/para:38/char:"))
            })
            .map(|r| r["text"].as_str().unwrap())
            .collect();
        let norm = |s: &str| {
            s.chars()
                .filter(|c| !c.is_whitespace() && !c.is_control())
                .collect::<String>()
        };
        assert_eq!(norm(&rendered), norm(source), "원문 순서·누락·중복 보존");
        // 독립 PDF70쪽 KoPubDotumLight, cm0.12, tm(659,5118), MediaBox.top841pt.
        for (k, expected) in [("e", 105.44), ("f", 818.88)] {
            let actual = owned[0]["placement"]["runToPage"][k].as_f64().unwrap();
            assert!(
                (actual - expected).abs() <= 2.0,
                "독립 Print 본문 {k}: actual={actual}, expected={expected}, ±2CSSpx"
            );
        }
    }
}

mod rowbreak {
    use rhwp::{
        paint::{LayerJsonOptions, RenderProfile},
        DocumentCore,
    };
    use serde_json::Value;
    fn collect<'a>(v: &'a Value, out: &mut Vec<&'a Value>) {
        match v {
            Value::Object(m) => {
                if m.get("type").and_then(Value::as_str) == Some("textRun") {
                    out.push(v)
                }
                for v in m.values() {
                    collect(v, out)
                }
            }
            Value::Array(a) => {
                for v in a {
                    collect(v, out)
                }
            }
            _ => {}
        }
    }
    #[test]
    fn rowbreak_local_advance_does_not_publish_an_absolute_paint_endpoint() {
        let mut c = DocumentCore::from_bytes(
            &std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/samples/rowbreak-problem-pages.hwp"
            ))
            .unwrap(),
        )
        .unwrap();
        c.set_hangul2024_compat(false);
        assert_eq!(c.page_count(), 18);
        let value: Value = serde_json::from_str(
            &c.get_page_layer_tree_with_options_native(
                13,
                RenderProfile::Print,
                LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
        )
        .unwrap();
        let mut runs = vec![];
        collect(&value["root"], &mut runs);
        let owned: Vec<_> = runs
            .iter()
            .filter(|r| {
                r["source"]["stableSourceKey"] == "section:1/para:0/char:23/cell:16:0:0:0:0"
                    && r["text"]
                        .as_str()
                        .is_some_and(|t| t.starts_with("공공데이터 품질진단 및 개선("))
            })
            .collect();
        assert_eq!(owned.len(), 1, "원문 첫 어절은 한 번만 배치한다");
        let table = c.document().sections[1].paragraphs[16]
            .controls
            .iter()
            .find_map(|ctrl| {
                if let rhwp::model::control::Control::Table(table) = ctrl {
                    Some(table)
                } else {
                    None
                }
            })
            .expect("원본 표 소유");
        let source = &table.cells[0].paragraphs[0].text;
        fn source_lines<'a>(v: &'a Value, found: &mut Vec<&'a Value>) {
            if v["groupKind"]["kind"] == "textLine" {
                let mut runs = vec![];
                collect(v, &mut runs);
                if runs.iter().any(|r| {
                    r["source"]["stableSourceKey"] == "section:1/para:0/char:23/cell:16:0:0:0:0"
                        && r["text"]
                            .as_str()
                            .is_some_and(|s| s.starts_with("공공데이터 품질진단 및 개선("))
                }) {
                    found.push(v);
                }
            }
            match v {
                Value::Object(m) => {
                    for child in m.values() {
                        source_lines(child, found);
                    }
                }
                Value::Array(a) => {
                    for child in a {
                        source_lines(child, found);
                    }
                }
                _ => {}
            }
        }
        let mut lines = vec![];
        source_lines(&value["root"], &mut lines);
        assert_eq!(
            lines.len(),
            1,
            "원문 표의 인용 문단은 하나의 실제 Print 줄에 속한다"
        );
        let mut line_runs = vec![];
        collect(lines[0], &mut line_runs);
        let rendered: String = line_runs
            .iter()
            .map(|r| r["text"].as_str().unwrap())
            .collect();
        let norm = |s: &str| {
            s.chars()
                .filter(|c| !c.is_whitespace() && !c.is_control())
                .collect::<String>()
        };
        assert_eq!(norm(&rendered), norm(source), "원문 순서·누락·중복 보존");
        // 독립 PDF14쪽 MalgunGothic, cm0.12, tm(2346,3317), MediaBox.top841pt.
        for (k, expected) in [("e", 375.36), ("f", 530.72)] {
            let actual = owned[0]["placement"]["runToPage"][k].as_f64().unwrap();
            assert!(
                (actual - expected).abs() <= 2.0,
                "독립 Print 본문 {k}: actual={actual}, expected={expected}, ±2CSSpx"
            );
        }
    }
}
