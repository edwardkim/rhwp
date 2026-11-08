use super::*;

fn sample_tree() -> ParameterList {
    ParameterList {
        name: Some(String::new()),
        items: vec![
            Parameter::Integer {
                name: Some("Prop".to_string()),
                value: 9,
            },
            Parameter::String {
                name: Some("Command".to_string()),
                value: "Clickhere:set:66:Direction:wstring:23:이곳을 마우스로 누르고 <입력>하세요 & \"확인\""
                    .to_string(),
                preserve_space: true,
            },
            Parameter::String {
                name: Some("Direction".to_string()),
                value: "이곳을 마우스로 누르고 내용을 입력하세요.".to_string(),
                preserve_space: false,
            },
            // [#4437] 실물 코퍼스 표기 그대로 — `1` 과 `false` 가 섞여 있다.
            Parameter::Boolean {
                name: Some("Fiexde".to_string()),
                value: true,
                lexical: Some("1".to_string()),
            },
            Parameter::Boolean {
                name: Some("RefHyperLink".to_string()),
                value: false,
                lexical: Some("false".to_string()),
            },
            Parameter::Float {
                name: Some("Ratio".to_string()),
                value: 1.5,
            },
            Parameter::List(ParameterList {
                name: Some("623".to_string()),
                items: vec![Parameter::Integer {
                    name: Some("16384".to_string()),
                    value: 42,
                }],
            }),
        ],
    }
}

#[test]
fn render_then_parse_round_trips_the_tree() {
    let original = sample_tree();
    let xml = original.render_xml("parameters");
    assert!(xml.starts_with("<hp:parameters cnt=\"7\" name=\"\">"));
    assert!(xml.ends_with("</hp:parameters>"));
    let parsed = ParameterList::parse_xml(&xml).expect("parse_xml 실패");
    assert_eq!(parsed, original);
}

/// [#4437] 원본 lexical(`false`/`true`) 이 정규화(`0`/`1`)되지 않고 바이트
/// 그대로 왕복해야 한다.
#[test]
fn issue4437_boolean_lexical_round_trips_verbatim() {
    let xml = sample_tree().render_xml("parameters");
    assert!(
        xml.contains(r#"<hp:booleanParam name="RefHyperLink">false</hp:booleanParam>"#),
        "원본 `false` 표기가 `0` 으로 정규화되면 안 된다: {xml}"
    );
    assert!(
        xml.contains(r#"<hp:booleanParam name="Fiexde">1</hp:booleanParam>"#),
        "원본 `1` 표기는 그대로: {xml}"
    );
    // parse → render 재왕복도 바이트 동일.
    let reparsed = ParameterList::parse_xml(&xml).expect("parse_xml");
    assert_eq!(reparsed.render_xml("parameters"), xml);

    // lexical 이 없는(프로그램 생성) Boolean 은 종전대로 0/1 정규화.
    let synth = ParameterList {
        name: Some(String::new()),
        items: vec![Parameter::Boolean {
            name: Some("New".to_string()),
            value: false,
            lexical: None,
        }],
    };
    assert!(synth
        .render_xml("parameters")
        .contains(r#"<hp:booleanParam name="New">0</hp:booleanParam>"#));
}

#[test]
fn empty_list_round_trips() {
    let original = ParameterList {
        name: Some(String::new()),
        items: vec![],
    };
    let xml = original.render_xml("parameters");
    let parsed = ParameterList::parse_xml(&xml).expect("parse_xml 실패");
    assert_eq!(parsed, original);
}

#[test]
fn is_empty_reflects_items() {
    assert!(ParameterList::default().is_empty());
    assert!(!sample_tree().is_empty());
}
