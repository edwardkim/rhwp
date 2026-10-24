//! #7445: 저장 문단의 로컬 전진과 확정된 출력 축의 끝점을 구분한다.
//! 독립 Print 원점과 원문 소유·간격 관계를 함께 검증한다.
#![cfg(not(target_arch = "wasm32"))]

mod handbook {
    use rhwp::{
        model::control::Control,
        renderer::render_tree::{RenderNode, RenderNodeType},
        DocumentCore,
    };
    fn owner(n: &RenderNode, pi: usize, table: bool) -> Option<&RenderNode> {
        if match &n.node_type {
            RenderNodeType::Table(t) => {
                table && t.section_index == Some(7) && t.para_index == Some(pi)
            }
            RenderNodeType::TextRun(t) => {
                !table
                    && t.section_index == Some(7)
                    && t.para_index == Some(pi)
                    && t.cell_context.is_none()
                    && t.char_start == Some(0)
            }
            _ => false,
        } {
            return Some(n);
        }
        n.children.iter().find_map(|c| owner(c, pi, table))
    }
    #[test]
    fn stored_empty_line_and_tac_host_keep_the_same_closed_source_interval() {
        let mut c = DocumentCore::from_bytes(
            &std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/samples/2025 행정업무운영 편람(최종).hwp"
            ))
            .unwrap(),
        )
        .unwrap();
        c.set_hangul2024_compat(true);
        assert_eq!(c.page_count(), 383);
        let p = &c.document().sections[7].paragraphs;
        assert!(p[413].text.is_empty() && p[413].controls.is_empty());
        let Control::Table(t) = &p[414].controls[0] else {
            panic!("원본 표")
        };
        assert!(t.common.treat_as_char);
        let a = &p[413].line_segs[0];
        let b = &p[414].line_segs[0];
        assert_eq!(
            a.vertical_pos + a.line_height + a.line_spacing,
            b.vertical_pos
        );
        let expected = f64::from(b.vertical_pos - a.vertical_pos) / 75.0;
        let tree = c.build_page_render_tree(241).unwrap();
        let blank = owner(&tree.root, 413, false).unwrap();
        let table = owner(&tree.root, 414, true).unwrap();
        let next = owner(&tree.root, 415, false).unwrap();
        assert!(
            (next.bbox.y
                - table.bbox.y
                - f64::from(p[415].line_segs[0].vertical_pos - b.vertical_pos) / 75.0)
                .abs()
                < 1.0 / 75.0,
            "표 이후 원문 간격과 소유도 보존"
        );
        assert!(
            (table.bbox.y - blank.bbox.y - expected).abs() < 1.0 / 75.0,
            "빈 줄과 TAC host의 닫힌 원문 간격: actual={} expected={expected}",
            table.bbox.y - blank.bbox.y
        );
    }

    #[test]
    fn actual_native_print_keeps_the_independent_title_baseline_and_source_owners() {
        use rhwp::paint::{LayerJsonOptions, RenderProfile};
        use serde_json::Value;
        let mut c = DocumentCore::from_bytes(
            &std::fs::read(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/samples/2025 행정업무운영 편람(최종).hwp"
            ))
            .unwrap(),
        )
        .unwrap();
        c.set_hangul2024_compat(true);
        assert_eq!(c.page_count(), 383);
        let page: Value = serde_json::from_str(
            &c.get_page_layer_tree_with_options_native(
                241,
                RenderProfile::Print,
                LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
        )
        .unwrap();
        fn collect<'a>(node: &'a Value, out: &mut Vec<&'a Value>) {
            match node {
                Value::Object(m) => {
                    if m.get("type").and_then(Value::as_str) == Some("textRun") {
                        out.push(node)
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
        let mut runs = vec![];
        collect(&page["root"], &mut runs);
        // 독립 PDF242쪽/F4 KoPubDotumBold: cm0.12, tm(812,3301)/(1085,3301).
        // CSS96은72pt의4/3이다. 후보bbox에서 기대값을 역산하지 않는다.
        for (cell, prefix, x) in [(5, "10", 129.92), (6, "정책연구관리시스템(", 173.6)] {
            let key = format!("section:7/para:0/char:0/cell:414:0:{cell}:0:0");
            let owned: Vec<_> = runs
                .iter()
                .filter(|r| {
                    r["source"]["stableSourceKey"] == key
                        && r["text"].as_str().is_some_and(|t| t.starts_with(prefix))
                })
                .collect();
            assert_eq!(
                owned.len(),
                1,
                "원본 제목의 첫 글자는 Native Print에도 유일한 소유자가 있다"
            );
            let matrix = &owned[0]["placement"]["runToPage"];
            for (name, expected) in [("e", x), ("f", 528.16)] {
                let actual = matrix[name].as_f64().unwrap();
                assert!(
                    (actual - expected).abs() <= 2.0,
                    "독립Print 제목 {prefix} {name}: actual={actual}, expected={expected}, ±2CSSpx"
                );
            }
        }
    }
}

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
