//! [#5177] 문서 탐색 메뉴는 공개 facts API에서 결정된다.

use rhwp::document_core::queries::explore::{build_menu, DocFacts};

fn facts() -> DocFacts {
    DocFacts {
        format_label: "HWP5".to_string(),
        page_count: 3,
        para_count: 40,
        ..Default::default()
    }
}

#[test]
fn plain_document_offers_overview() {
    let menu = build_menu(&facts());
    assert_eq!(menu.len(), 1, "{menu:?}");
    assert_eq!(menu[0].affordance, "triage-overview");
    assert_eq!(menu[0].command, "rhwp digest <file> --json");
}

#[test]
fn document_facts_surface_matching_affordances() {
    let mut f = facts();
    f.table_count = 3;
    f.merged_table_count = 1;
    f.field_count = 8;
    f.chart_count = 2;
    let menu = build_menu(&f);

    let table = menu
        .iter()
        .find(|item| item.affordance == "table-extract")
        .expect("표 어포던스");
    assert!(table.why.contains('3'), "{}", table.why);
    assert!(table.why.contains("병합"), "{}", table.why);
    assert_eq!(table.skill, "rhwp-table-exchange");
    assert!(menu.iter().any(|item| {
        item.affordance == "form-fill" && item.command == "rhwp fields <file> --json"
    }));
    assert!(menu.iter().any(|item| item.affordance == "chart-extract"));
}

#[test]
fn security_signal_has_highest_priority() {
    let mut f = facts();
    f.table_count = 2;
    f.field_count = 2;
    f.injection_signal_count = 1;
    let menu = build_menu(&f);
    assert_eq!(menu[0].affordance, "security-sweep", "{menu:?}");
    assert_eq!(menu[0].confidence, "high");
    assert!(menu[0].command.contains("inspect injection"));
}

#[test]
fn hidden_text_uses_medium_confidence_security_command() {
    let mut f = facts();
    f.hidden_text_count = 2;
    let menu = build_menu(&f);
    assert_eq!(menu[0].affordance, "security-sweep");
    assert_eq!(menu[0].confidence, "medium");
    assert!(
        menu[0].command.contains("hidden-text"),
        "{}",
        menu[0].command
    );
}

#[test]
fn long_document_offers_sectioned_digest() {
    let mut f = facts();
    f.page_count = 40;
    let menu = build_menu(&f);
    let long = menu
        .iter()
        .find(|item| item.affordance == "long-doc-digest")
        .expect("장문 어포던스");
    assert_eq!(long.confidence, "high");
    assert!(long.command.contains("--sections"));
    assert!(build_menu(&facts())
        .iter()
        .all(|item| item.affordance != "long-doc-digest"));
}

#[test]
fn different_documents_yield_different_sorted_menus() {
    let mut form = facts();
    form.field_count = 5;
    let mut report = facts();
    report.table_count = 4;
    report.chart_count = 2;
    report.structure_node_count = 6;

    let form_ids: Vec<&str> = build_menu(&form)
        .iter()
        .map(|item| item.affordance)
        .collect();
    let report_ids: Vec<&str> = build_menu(&report)
        .iter()
        .map(|item| item.affordance)
        .collect();
    assert!(form_ids.contains(&"form-fill"));
    assert!(!form_ids.contains(&"chart-extract"));
    assert!(report_ids.contains(&"chart-extract"));
    assert!(!report_ids.contains(&"form-fill"));
    assert_ne!(form_ids, report_ids);

    let mut prioritized = facts();
    prioritized.field_count = 1;
    prioritized.table_count = 1;
    prioritized.chart_count = 1;
    prioritized.injection_signal_count = 1;
    let ids: Vec<&str> = build_menu(&prioritized)
        .iter()
        .map(|item| item.affordance)
        .collect();
    assert_eq!(
        ids,
        vec![
            "security-sweep",
            "form-fill",
            "table-extract",
            "chart-extract",
            "triage-overview"
        ]
    );
}
