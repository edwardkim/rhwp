use super::*;
use crate::agent::dsel::parse;
use crate::model::control::{Bookmark, Field, FieldType};
use crate::model::document::Section;
use crate::model::paragraph::CharShapeRef;
use crate::model::table::{Cell, Table};

fn para(text: &str) -> Paragraph {
    let mut p = Paragraph {
        text: text.to_string(),
        ..Default::default()
    };
    let mut utf16 = 0u32;
    for ch in text.chars() {
        p.char_offsets.push(utf16);
        utf16 += ch.len_utf16() as u32;
    }
    p
}

fn para_styled(text: &str, style_id: u8) -> Paragraph {
    let mut p = para(text);
    p.style_id = style_id;
    p
}

fn cell(row: u16, col: u16, text: &str) -> Cell {
    Cell {
        row,
        col,
        row_span: 1,
        col_span: 1,
        paragraphs: vec![para(text)],
        ..Default::default()
    }
}

/// 2×2 표 하나를 담은 문단.
fn table_para(rows: u16, cols: u16, texts: &[&str]) -> Paragraph {
    let mut t = Table {
        row_count: rows,
        col_count: cols,
        ..Default::default()
    };
    for (i, text) in texts.iter().enumerate() {
        let r = (i as u16) / cols;
        let c = (i as u16) % cols;
        t.cells.push(cell(r, c, text));
    }
    let mut p = para("");
    p.controls.push(Control::Table(Box::new(t)));
    p
}

fn doc_with(paragraphs: Vec<Paragraph>) -> Document {
    Document {
        sections: vec![Section {
            paragraphs,
            ..Default::default()
        }],
        ..Default::default()
    }
}

fn sel(src: &str, doc: &Document) -> Vec<String> {
    let s = parse(src).unwrap_or_else(|e| panic!("{}", e.render(src)));
    select(&s, doc)
        .unwrap_or_else(|e| panic!("{e}"))
        .into_iter()
        .map(|n| n.id.to_string())
        .collect()
}

fn texts(src: &str, doc: &Document) -> Vec<String> {
    let s = parse(src).unwrap();
    select(&s, doc)
        .unwrap()
        .into_iter()
        .filter_map(|n| n.node.text())
        .collect()
}

#[test]
fn selects_paragraphs_in_document_order() {
    let doc = doc_with(vec![para("가"), para("나"), para("다")]);
    assert_eq!(
        sel("para", &doc),
        vec![
            "/section[0]/para[0]",
            "/section[0]/para[1]",
            "/section[0]/para[2]"
        ]
    );
}

#[test]
fn first_step_reaches_any_depth() {
    // 셀 안 문단도 `para` 로 걸린다 — CSS 의 타입 선택자와 같은 규칙.
    let doc = doc_with(vec![para("바깥"), table_para(1, 2, &["안1", "안2"])]);
    let all = texts("para", &doc);
    assert!(all.contains(&"바깥".to_string()));
    assert!(all.contains(&"안1".to_string()));
}

#[test]
fn child_combinator_does_not_cross_levels() {
    let doc = doc_with(vec![para("바깥"), table_para(1, 2, &["안1", "안2"])]);
    // 구역의 직계 문단만 — 셀 안 문단은 제외.
    let direct = texts("section > para", &doc);
    assert!(direct.contains(&"바깥".to_string()));
    assert!(!direct.contains(&"안1".to_string()));
}

#[test]
fn descendant_combinator_crosses_levels() {
    let doc = doc_with(vec![table_para(1, 2, &["안1", "안2"])]);
    let inner = texts("table para", &doc);
    assert_eq!(inner, vec!["안1".to_string(), "안2".to_string()]);
}

#[test]
fn cell_addressing_by_row_and_col() {
    let doc = doc_with(vec![table_para(2, 2, &["A", "B", "C", "D"])]);
    assert_eq!(texts("cell[row=1][col=0]", &doc), vec!["C".to_string()]);
}

#[test]
fn sibling_combinators_respect_node_kind() {
    let doc = doc_with(vec![para("가"), para("나"), para("다")]);
    // `+` 는 바로 다음 형제 하나.
    assert_eq!(texts("para[index=0] + para", &doc), vec!["나".to_string()]);
    // `~` 는 이후 형제 전부.
    assert_eq!(
        texts("para[index=0] ~ para", &doc),
        vec!["나".to_string(), "다".to_string()]
    );
}

#[test]
fn positional_pseudos_count_over_the_result_set() {
    let doc = doc_with(vec![
        table_para(1, 1, &["첫"]),
        table_para(1, 1, &["둘"]),
        table_para(1, 1, &["셋"]),
    ]);
    // 표 셋은 각각 다른 문단에 있으므로 형제 기준이라면 전부 `index=0` 이다.
    // 결과 집합 기준이므로 `:last` 는 세 번째 표를 고른다.
    assert_eq!(texts("table:last cell", &doc), vec!["셋".to_string()]);
    assert_eq!(texts("table:nth(1) cell", &doc), vec!["둘".to_string()]);
    assert_eq!(texts("table:first cell", &doc), vec!["첫".to_string()]);
}

#[test]
fn negative_nth_counts_from_the_end() {
    let doc = doc_with(vec![para("가"), para("나"), para("다")]);
    assert_eq!(texts("para:nth(-1)", &doc), vec!["다".to_string()]);
    assert_eq!(texts("para:nth(-3)", &doc), vec!["가".to_string()]);
    // 범위를 벗어나면 빈 결과 — 패닉이 아니다.
    assert!(texts("para:nth(-9)", &doc).is_empty());
    assert!(texts("para:nth(9)", &doc).is_empty());
}

#[test]
fn range_clamps_instead_of_failing() {
    let doc = doc_with(vec![para("가"), para("나"), para("다")]);
    assert_eq!(
        texts("para:range(1..99)", &doc),
        vec!["나".to_string(), "다".to_string()]
    );
    assert_eq!(texts("para:range(0..1)", &doc), vec!["가".to_string()]);
    assert!(texts("para:range(5..9)", &doc).is_empty());
}

#[test]
fn value_predicates_run_before_positional_ones() {
    let doc = doc_with(vec![
        para_styled("가", 1),
        para_styled("나", 2),
        para_styled("다", 1),
    ]);
    // 스타일 1 인 문단들 중 마지막 = "다". 위치를 먼저 적용했다면 "다"를
    // 고른 뒤 스타일을 봐서 결과가 같겠지만, 스타일 2 로 물으면 갈린다.
    assert_eq!(texts("para[styleId=2]:last", &doc), vec!["나".to_string()]);
}

#[test]
fn contains_and_matches_use_visible_text() {
    // 제어문자가 섞여 있어도 사람이 보는 글자로 걸려야 한다.
    let doc = doc_with(vec![para("합\u{0003}계")]);
    assert_eq!(texts(r#"para:contains("합계")"#, &doc).len(), 1);
    assert_eq!(texts(r#"para:matches("합*")"#, &doc).len(), 1);
    assert_eq!(texts(r#"para[text="합계"]"#, &doc).len(), 1);
}

#[test]
fn empty_only_matches_nodes_that_have_text() {
    let mut p = para("");
    p.controls.push(Control::Bookmark(Bookmark {
        name: "표시".into(),
    }));
    let doc = doc_with(vec![p]);
    // 빈 문단은 걸린다.
    assert_eq!(sel("para:empty", &doc).len(), 1);
    // 텍스트 개념이 없는 책갈피는 걸리지 않는다.
    assert!(sel("bookmark:empty", &doc).is_empty());
}

#[test]
fn not_excludes_the_inner_result_set() {
    let doc = doc_with(vec![para("가"), para(""), para("다")]);
    assert_eq!(
        texts("para:not(:empty)", &doc),
        vec!["가".to_string(), "다".to_string()]
    );
}

#[test]
fn has_matches_ancestors_of_the_inner_result() {
    let doc = doc_with(vec![
        table_para(1, 1, &["합계"]),
        table_para(1, 1, &["기타"]),
    ]);
    let hits = sel(r#"table:has(cell:contains("합계"))"#, &doc);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].ends_with("/control[0]"));
    assert!(hits[0].starts_with("/section[0]/para[0]"));
}

#[test]
fn union_paths_are_merged_in_document_order_without_duplicates() {
    let doc = doc_with(vec![para("가"), para("나")]);
    // 두 가지가 겹쳐도 한 번만 나온다.
    assert_eq!(sel("para, para[index=0]", &doc).len(), 2);
}

#[test]
fn missing_attribute_never_matches_even_with_ne() {
    let doc = doc_with(vec![table_para(1, 1, &["A"])]);
    // 셀에 field_name 이 없다. `!=` 로도 걸리면 안 된다.
    assert!(sel(r#"cell[name!="합계"]"#, &doc).is_empty());
    assert!(sel("cell[name]", &doc).is_empty());
}

#[test]
fn bare_boolean_attribute_means_true() {
    let mut t = Table {
        row_count: 1,
        col_count: 1,
        ..Default::default()
    };
    let mut c = cell(0, 0, "제목");
    c.is_header = true;
    t.cells.push(c);
    t.cells.push(cell(0, 1, "값"));
    let mut p = para("");
    p.controls.push(Control::Table(Box::new(t)));
    let doc = doc_with(vec![p]);
    assert_eq!(texts("cell[header]", &doc), vec!["제목".to_string()]);
    assert_eq!(texts("cell[header=false]", &doc), vec!["값".to_string()]);
}

#[test]
fn field_name_prefers_ctrl_data_and_ignores_empty() {
    let mut p = para("");
    p.controls.push(Control::Field(Field {
        field_type: FieldType::ClickHere,
        command: "안내문".into(),
        ctrl_data_name: Some("수급자성명".into()),
        ..Default::default()
    }));
    p.controls.push(Control::Field(Field {
        field_type: FieldType::ClickHere,
        command: String::new(),
        ctrl_data_name: None,
        ..Default::default()
    }));
    let doc = doc_with(vec![p]);
    assert_eq!(sel(r#"field[name="수급자성명"]"#, &doc).len(), 1);
    // 이름이 빈 필드는 `[name]` 에 걸리지 않는다.
    assert_eq!(sel("field[name]", &doc).len(), 1);
    assert_eq!(sel(r#"field[type=clickHere]"#, &doc).len(), 2);
}

#[test]
fn table_axis_and_control_kind_select_the_same_thing() {
    let doc = doc_with(vec![table_para(1, 1, &["A"])]);
    assert_eq!(sel("table", &doc), sel("control[kind=table]", &doc));
}

#[test]
fn runs_are_selectable_by_char_shape() {
    let mut p = para("가나다라");
    p.char_shapes = vec![
        CharShapeRef {
            start_pos: 0,
            char_shape_id: 7,
        },
        CharShapeRef {
            start_pos: 2,
            char_shape_id: 9,
        },
    ];
    let doc = doc_with(vec![p]);
    assert_eq!(texts("run[charShapeId=9]", &doc), vec!["다라".to_string()]);
}

#[test]
fn node_limit_is_enforced_as_an_error_not_an_oom() {
    let doc = doc_with((0..50).map(|i| para(&format!("문단{i}"))).collect());
    let s = parse("para").unwrap();
    let limits = EvalLimits {
        max_nodes: 10,
        ..Default::default()
    };
    let err = select_with(&s, &doc, limits).unwrap_err();
    assert!(err.message.contains("노드가 너무 많다"));
}

#[test]
fn result_limit_is_enforced() {
    let doc = doc_with((0..20).map(|i| para(&format!("문단{i}"))).collect());
    let s = parse("para").unwrap();
    let limits = EvalLimits {
        max_results: 5,
        ..Default::default()
    };
    let err = select_with(&s, &doc, limits).unwrap_err();
    assert!(err.message.contains("결과가 너무 많다"));
    // 세는 것은 상한에 막히지 않는다.
    assert_eq!(count(&s, &doc, limits).unwrap(), 20);
}

#[test]
fn empty_document_selects_nothing_without_error() {
    let doc = Document::default();
    assert!(sel("para", &doc).is_empty());
    assert!(sel("table cell", &doc).is_empty());
}

#[test]
fn index_attribute_stays_sibling_relative() {
    // 위치 의사 선택자는 결과 집합 기준이지만 `index` 는 형제 기준이다.
    // 두 기준이 같은 문법을 쓰면 어느 쪽인지 알 수 없게 된다.
    let doc = doc_with(vec![table_para(1, 1, &["첫"]), table_para(1, 1, &["둘"])]);
    // 표 둘 다 자기 문단의 첫 컨트롤이므로 index=0 이 둘 다 걸린다.
    assert_eq!(sel("table[index=0]", &doc).len(), 2);
    // 결과 집합 기준인 :first 는 하나만 고른다.
    assert_eq!(sel("table:first", &doc).len(), 1);
}
