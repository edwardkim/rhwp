use super::*;

#[test]
fn doc_anomalies_omit_clean_pages() {
    let clean_body = body_node(BoundingBox::new(0.0, 0.0, 100.0, 200.0), vec![]);
    // 페이지 0: 콘텐츠 있음(첫 쪽이라 애초에 empty 후보도 아님).
    let mut line = text_line(10.0, 10.0, 20.0, 10.0);
    line.children.push(text_run("x"));
    let mut body0 = clean_body.clone();
    body0.children.push(line);
    let pa0 = scan_page(
        0,
        &page_root(100.0, 200.0, body0),
        3,
        &AnomalyOptions::default(),
    );
    // 페이지 1: 비어 있음 (중간 쪽 → possible 신호).
    let pa1 = scan_page(
        1,
        &page_root(100.0, 200.0, clean_body.clone()),
        3,
        &AnomalyOptions::default(),
    );
    // 페이지 2: 콘텐츠 있음, 깨끗.
    let mut body2 = clean_body;
    let mut line2 = text_line(10.0, 10.0, 20.0, 10.0);
    line2.children.push(text_run("y"));
    body2.children.push(line2);
    let pa2 = scan_page(
        2,
        &page_root(100.0, 200.0, body2),
        3,
        &AnomalyOptions::default(),
    );

    let pages: Vec<PageAnomalies> = vec![pa0, pa1, pa2]
        .into_iter()
        .filter(|page| !page.is_empty())
        .collect();
    assert_eq!(pages.len(), 1);
    assert_eq!(pages[0].page, 1);
}
