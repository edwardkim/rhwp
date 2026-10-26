//! #8160: sample16 전용 회귀. 이전 등록: tests/cases/issue_7445_original_font_program_ownership.rs
use rhwp::DocumentCore;
use serde_json::Value;

fn sample16_layer_ops<'a>(value: &'a Value, kind: &str, prefix: &str, out: &mut Vec<&'a Value>) {
    match value {
        Value::Object(object) => {
            if value["type"] == kind
                && value["source"]["stableSourceKey"]
                    .as_str()
                    .is_some_and(|key| key.starts_with(prefix))
            {
                out.push(value);
            }
            for child in object.values() {
                sample16_layer_ops(child, kind, prefix, out);
            }
        }
        Value::Array(array) => {
            for child in array {
                sample16_layer_ops(child, kind, prefix, out);
            }
        }
        _ => {}
    }
}

fn sample16_assert_tab_owner(label: &str) {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/hwp3-sample16-hwp5.hwp")).unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(true);
    let (pi, text) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, p)| p.text.contains(label) && p.text.contains('\t'))
        .map(|(i, p)| (i, p.text.clone()))
        .unwrap();
    let layer: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_options_native(
                1,
                rhwp::paint::RenderProfile::Print,
                rhwp::paint::LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
    )
    .unwrap();
    let prefix = format!("section:0/para:{pi}/char:");
    let mut texts = vec![];
    let mut leaders = vec![];
    sample16_layer_ops(&layer, "textRun", &prefix, &mut texts);
    sample16_layer_ops(&layer, "tabLeader", &prefix, &mut leaders);
    assert_eq!(
        texts
            .iter()
            .map(|op| op["text"].as_str().unwrap())
            .collect::<String>(),
        text,
        "원문 순서·누락·중복"
    );
    assert_eq!(
        leaders.len(),
        text.chars().filter(|&c| c == '\t').count(),
        "탭 하나의 리더가 도형 앞뒤에 중복되지 않음"
    );
    for leader in leaders {
        let owner: Vec<_> = texts
            .iter()
            .filter(|op| op["source"]["stableSourceKey"] == leader["source"]["stableSourceKey"])
            .collect();
        assert_eq!(owner.len(), 1);
        assert!(
            owner[0]["text"].as_str().unwrap().contains('\t'),
            "탭이 없는 앞 조각은 리더를 그리지 않음"
        );
        let source_start: usize = owner[0]["source"]["stableSourceKey"]
            .as_str()
            .unwrap()
            .strip_prefix(&prefix)
            .unwrap()
            .parse()
            .unwrap();
        let tab_local = owner[0]["text"]
            .as_str()
            .unwrap()
            .chars()
            .position(|c| c == '\t')
            .unwrap();
        assert_eq!(text.chars().nth(source_start + tab_local), Some('\t'));
        let number = texts.last().unwrap();
        for span in leader["leaders"].as_array().unwrap() {
            assert!(
                leader["bbox"]["x"].as_f64().unwrap() + span["endX"].as_f64().unwrap()
                    <= number["bbox"]["x"].as_f64().unwrap(),
                "리더는 뒤 쪽번호를 침범하지 않음"
            );
        }
    }
    assert_eq!(core.document().sections[0].paragraphs[pi].text, text);
}

#[test]
fn issue_8160_sample16_inline_object_fragments_keep_each_leader_with_its_source_tab() {
    sample16_assert_tab_owner("추진목적");
}

#[test]
fn issue_8160_sample16_ordinary_tab_keeps_its_single_source_owner() {
    sample16_assert_tab_owner("추진방향");
}

#[test]
fn issue_8160_sample16_hft_bold_heading_preserves_content_and_independent_completed_page_origin() {
    // 독립 Print p19 /T4의 기준선과 뒤 글자. 최종 양쪽 raw98% 자격은 stage5에 기록한다.
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(
        &std::fs::read(root.join("samples/hwp3-sample16-hwp5.hwp")).unwrap(),
    )
    .unwrap();
    core.set_hangul2024_compat(true);
    let (pi, text) = core.document().sections[0]
        .paragraphs
        .iter()
        .enumerate()
        .find(|(_, p)| p.text.starts_with("3. 원격지 재해복구센터") && p.text.contains("Disaster"))
        .map(|(i, p)| (i, p.text.clone()))
        .unwrap();
    let layer: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_options_native(
                18,
                rhwp::paint::RenderProfile::Print,
                rhwp::paint::LayerJsonOptions {
                    omit_image_bytes: true,
                    omit_font_bytes: true,
                },
            )
            .unwrap(),
    )
    .unwrap();
    let prefix = format!("section:0/para:{pi}/char:");
    let mut ops = vec![];
    sample16_layer_ops(&layer, "textRun", &prefix, &mut ops);
    assert_eq!(
        ops.iter()
            .map(|op| op["text"].as_str().unwrap())
            .collect::<String>(),
        text
    );
    let tail = text.chars().position(|c| c == '터').unwrap();
    let mut origins = vec![];
    for op in ops {
        let start: usize = op["source"]["stableSourceKey"]
            .as_str()
            .unwrap()
            .strip_prefix(&prefix)
            .unwrap()
            .parse()
            .unwrap();
        for cluster in op["clusters"].as_array().unwrap() {
            let local = cluster["textRangeUtf16"]["start"].as_u64().unwrap() as usize;
            if start + local == tail {
                assert_eq!(op["text"].as_str().unwrap().chars().nth(local), Some('터'));
                origins.push((
                    op["placement"]["runToPage"]["e"].as_f64().unwrap()
                        + cluster["origin"]["x"].as_f64().unwrap(),
                    op["placement"]["runToPage"]["f"].as_f64().unwrap()
                        + cluster["origin"]["y"].as_f64().unwrap(),
                ));
            }
        }
    }
    assert_eq!(origins.len(), 1);
    assert!(
        (origins[0].0 - 299.6766764322917).abs() <= 2.0,
        "원본 HFT 굵은 전진 뒤 '터': {:?}",
        origins[0]
    );
    assert!((origins[0].1 - 636.1599934895834).abs() <= 2.0);
    assert_eq!(core.document().sections[0].paragraphs[pi].text, text);
}
