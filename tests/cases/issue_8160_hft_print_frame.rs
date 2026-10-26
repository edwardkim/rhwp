//! #8160: 원본 프로그램 선택과 자동 번호의 인쇄 프레임 소유.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{paint::RenderProfile, DocumentCore};
use serde_json::Value;

fn load(path: &str) -> DocumentCore {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut core = DocumentCore::from_bytes(&std::fs::read(root.join(path)).unwrap()).unwrap();
    core.set_hangul2024_compat(false);
    core
}

fn texts<'a>(value: &'a Value, out: &mut Vec<&'a Value>) {
    if value.get("type").and_then(Value::as_str) == Some("textRun") {
        out.push(value);
    }
    match value {
        Value::Array(values) => values.iter().for_each(|value| texts(value, out)),
        Value::Object(values) => values.values().for_each(|value| texts(value, out)),
        _ => {}
    }
}

fn selected(value: &Value, text: &str, family: &str) -> Value {
    let mut runs = vec![];
    texts(value, &mut runs);
    let selected: Vec<_> = runs
        .into_iter()
        .filter(|run| {
            run["text"].as_str().is_some_and(|s| s.contains(text))
                && run["style"]["fontFamily"]
                    .as_str()
                    .is_some_and(|s| s.contains(family))
        })
        .collect();
    assert_eq!(
        selected.len(),
        1,
        "원본 줄과 실제 표시 글꼴을 한 번 선택: {text}/{family}"
    );
    selected[0].clone()
}

fn origin(run: &Value) -> (f64, f64) {
    let placement = &run["placement"]["runToPage"];
    let x = placement["e"].as_f64().unwrap();
    let y = placement["f"].as_f64().unwrap();
    let transform = &run["paintTransform"];
    if transform.is_null() {
        return (x, y);
    }
    (
        transform["a"].as_f64().unwrap() * x
            + transform["c"].as_f64().unwrap() * y
            + transform["e"].as_f64().unwrap(),
        transform["b"].as_f64().unwrap() * x
            + transform["d"].as_f64().unwrap() * y
            + transform["f"].as_f64().unwrap(),
    )
}

fn pair(core: &DocumentCore, page: u32, text: &str, family: &str) -> (Value, Value) {
    let screen: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_profile_native(page, RenderProfile::Screen)
            .unwrap(),
    )
    .unwrap();
    let print: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_profile_native(page, RenderProfile::Print)
            .unwrap(),
    )
    .unwrap();
    let (source, paint) = (
        selected(&screen, text, family),
        selected(&print, text, family),
    );
    assert_eq!(source["text"], paint["text"], "내용 순서·누락·중복 보존");
    assert_eq!(
        source["source"]["stableSourceKey"], paint["source"]["stableSourceKey"],
        "원본 소유 보존"
    );
    assert_eq!(
        source["bbox"], paint["bbox"],
        "paint 프레임으로 논리 상자를 덮어쓰지 않는다"
    );
    (source, paint)
}

#[test]
fn issue_8160_selected_font_and_page_number_preserve_print_frame_ownership() {
    // #8160 새 2024 Print 5쪽: 본문과 달리 자동 번호는 독립 배치 원점을
    // 사용한다. 본문용 종이 CTM을 중복 적용하면 숫자 아래에 미대응 1px가 남는다.
    let mut core = load("samples/hwp3-sample16-hwp5.hwpx");
    core.set_hangul2024_compat(true);
    core.set_font_environment(Some(
        rhwp::renderer::font_environment::FontEnvironment::from_json(
            r#"{"id":"original-page-number-program","substitutions":{"한양신명조":"한양신명조"}}"#,
        )
        .unwrap(),
    ))
    .unwrap();
    let (source, paint) = pair(&core, 4, "- 3 -", "한양신명조");
    assert_eq!(
        origin(&source),
        origin(&paint),
        "자동 번호의 원본 전진·배치에 본문 HFT 프레임을 중복 적용하지 않는다"
    );

    // 명시한 원 프로그램은 이전 슬롯의 HFT 프레임을 상속하지 않는다.
    // 기본/자동 공급은 독립 Print로 입증한 기존 HFT 프레임을 유지한다.
    for preserves_source in [true, false] {
        let mut program = load("samples/hwp3-sample16-hwp5.hwpx");
        program.set_hangul2024_compat(true);
        program
            .set_font_environment(Some(
                rhwp::renderer::font_environment::FontEnvironment::from_json(
                    &serde_json::json!({
                        "id":"selected-original-program-print-frame",
                        "substitutions":{"한양신명조":"한양신명조"},
                        "preserveSourceMetrics":preserves_source
                    })
                    .to_string(),
                )
                .unwrap(),
            ))
            .unwrap();
        let owners: Vec<_> = program
            .document()
            .sections
            .iter()
            .enumerate()
            .flat_map(|(si, s)| {
                s.paragraphs.iter().enumerate().filter_map(move |(pi, p)| {
                    let compact: String = p.text.chars().filter(|c| !c.is_whitespace()).collect();
                    (compact.starts_with("【별첨2】") && compact.contains("기술제안서작성양식"))
                        .then_some((si, pi))
                })
            })
            .collect();
        assert_eq!(owners.len(), 1, "독립 Print의 별첨 제목 원본 소유");
        let owner_key = format!("section:{}/para:{}/", owners[0].0, owners[0].1);
        let owned_bracket = |value: &Value| {
            let mut runs = vec![];
            texts(value, &mut runs);
            let runs: Vec<_> = runs
                .into_iter()
                .filter(|v| {
                    v["text"].as_str().is_some_and(|s| s.contains('】'))
                        && v["source"]["stableSourceKey"]
                            .as_str()
                            .is_some_and(|s| s.starts_with(&owner_key))
                })
                .collect();
            assert_eq!(runs.len(), 1, "제목의 닫는 괄호 내용·소유·중복");
            runs[0].clone()
        };
        let source: Value = serde_json::from_str(
            &program
                .get_page_layer_tree_with_profile_native(1, RenderProfile::Screen)
                .unwrap(),
        )
        .unwrap();
        let print: Value = serde_json::from_str(
            &program
                .get_page_layer_tree_with_profile_native(1, RenderProfile::Print)
                .unwrap(),
        )
        .unwrap();
        let (source, print) = (owned_bracket(&source), owned_bracket(&print));
        assert_eq!(source["text"], print["text"]);
        assert_eq!(
            source["source"]["stableSourceKey"],
            print["source"]["stableSourceKey"]
        );
        assert_eq!(source["bbox"], print["bbox"]);
        if preserves_source {
            assert!(
                origin(&print).1 < origin(&source).1,
                "자동 공급의 원본 HFT 프레임 보존"
            );
        } else {
            assert_eq!(
                origin(&print),
                origin(&source),
                "선택한 원 프로그램에 과거 슬롯의 HFT 프레임을 중첩하지 않음"
            );
        }
    }
}
