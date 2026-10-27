//! 같은 논리 앵커의 표·글줄 소유와 원문 바깥여백의 일회 소비를 보호한다.
//! 독립 Print·전쪽 Native/fresh WASM 100% 선행 증거:
//! mydocs/pr/assets/issue1510/coanchored-host-flow-20261025/README.md

#![cfg(not(target_arch = "wasm32"))]
use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::collections::BTreeMap;

fn text(node: &RenderNode) -> String {
    let mut result = match &node.node_type {
        RenderNodeType::TextRun(run) => run.display_or_text().to_owned(),
        _ => String::new(),
    };
    for child in &node.children {
        result.push_str(&text(child));
    }
    result
}
fn normalized(value: &str) -> String {
    value
        .chars()
        .filter(|c| !c.is_control() && !c.is_whitespace())
        .collect()
}
struct PaintedTable {
    page: u32,
    control: usize,
    bbox: BoundingBox,
    text: String,
}
struct BodyRun {
    page: u32,
    paragraph: usize,
    bbox: BoundingBox,
    text: String,
    baseline: f64,
}
fn collect(node: &RenderNode, page: u32, tables: &mut Vec<PaintedTable>, runs: &mut Vec<BodyRun>) {
    if let RenderNodeType::Table(info) = &node.node_type {
        assert_eq!(info.para_index, Some(0), "같은 호스트의 실제 표 소유");
        tables.push(PaintedTable {
            page,
            control: info.control_index.unwrap(),
            bbox: node.bbox,
            text: text(node),
        });
        return;
    }
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if let Some(paragraph) = run.para_index {
            runs.push(BodyRun {
                page,
                paragraph,
                bbox: node.bbox,
                baseline: node.bbox.y + run.baseline,
                text: run.display_or_text().to_owned(),
            });
        }
    }
    for child in &node.children {
        collect(child, page, tables, runs);
    }
}
fn verify(path: &str, preceding: usize, following: usize) {
    let core = DocumentCore::from_bytes(
        &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap(),
    )
    .unwrap();
    assert_eq!(core.page_count(), 2, "독립 한컴 Print 전쪽 수");
    let source = &core.document().sections[0].paragraphs;
    let mut tables = Vec::new();
    let mut runs = Vec::new();
    for page in 0..core.page_count() {
        collect(
            &core.build_page_render_tree(page).unwrap().root,
            page,
            &mut tables,
            &mut runs,
        );
    }
    let source_tables = source[0]
        .controls
        .iter()
        .enumerate()
        .filter_map(|(ci, c)| match c {
            Control::Table(t) => Some((ci, t)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        tables.len(),
        source_tables.len(),
        "표 누락·중복·불필요한 분할"
    );
    for (painted, (ci, original)) in tables.iter().zip(&source_tables) {
        assert_eq!(painted.page, 0, "세 표의 독립 Print 쪽 소속");
        assert_eq!(painted.control, *ci, "원문 제어문자 순서·소유");
        let expected = original
            .cells
            .iter()
            .flat_map(|c| &c.paragraphs)
            .map(|p| p.text.as_str())
            .collect::<String>();
        assert_eq!(
            normalized(&painted.text),
            normalized(&expected),
            "각 셀 내용의 순서·보존"
        );
    }
    let before = tables.iter().find(|t| t.control == preceding).unwrap();
    let after = tables.iter().find(|t| t.control == following).unwrap();
    let original_before = source_tables
        .iter()
        .find(|(ci, _)| *ci == preceding)
        .unwrap()
        .1;
    let original_after = source_tables
        .iter()
        .find(|(ci, _)| *ci == following)
        .unwrap()
        .1;
    // 고정 화면 좌표가 아닌 원본 여백의 관계를 HWPUNIT으로 대조한다.
    let gap_hu = (after.bbox.y - before.bbox.y - before.bbox.height) * 7200.0 / 96.0;
    let source_hu =
        f64::from(original_before.outer_margin_bottom) + f64::from(original_after.outer_margin_top);
    assert!(
        (gap_hu - source_hu).abs() <= 1.0,
        "원문 위·아래 여백을 한 번씩만 소비: 실제 {gap_hu}HU / 원본 {source_hu}HU"
    );
    let mut paragraphs = BTreeMap::<usize, String>::new();
    let mut previous_page = 0;
    for run in &runs {
        assert!(run.page >= previous_page, "본문 쪽 순서");
        previous_page = run.page;
        paragraphs
            .entry(run.paragraph)
            .or_default()
            .push_str(&run.text);
        for table in tables.iter().filter(|t| t.page == run.page) {
            let overlap_x = run.bbox.x < table.bbox.x + table.bbox.width
                && table.bbox.x < run.bbox.x + run.bbox.width;
            let overlap_y = run.bbox.y < table.bbox.y + table.bbox.height
                && table.bbox.y < run.bbox.y + run.bbox.height;
            assert!(
                !overlap_x || !overlap_y,
                "본문 {}이 표 {}의 본체와 겹치지 않는다",
                run.paragraph,
                table.control
            );
        }
    }
    assert_eq!(paragraphs.len(), source.len(), "본문 문단 누락·추가");
    for (pi, p) in source.iter().enumerate() {
        assert_eq!(
            normalized(&paragraphs[&pi]),
            normalized(&p.text),
            "문단 {pi} 내용의 누락·중복"
        );
    }
    let title = runs.iter().find(|r| r.paragraph == 0).unwrap();
    let last = tables.iter().find(|t| t.control == 4).unwrap();
    assert!(
        title.bbox.y >= last.bbox.y + last.bbox.height,
        "제목은 앞선 표의 점유 끝 뒤에서 재개한다"
    );
    assert!(
        runs.iter()
            .any(|r| r.page == 1 && r.text.contains("filler paragraph 32")),
        "독립 Print 끝 문단 보존"
    );
}
#[test]
fn negative_and_zero_offset_siblings_consume_only_their_outer_frames() {
    verify("samples/issue1510_coanchored_float_tables.hwpx", 3, 4);
}
#[test]
fn positive_siblings_clear_each_other_and_preserve_the_host_text() {
    verify(
        "samples/hwpx/issue1535_coanchored_float_exclusion.hwpx",
        2,
        3,
    );
}

// 100% 완료 페이지의 독립 PDF 좌표다. 일반 줄·개체 관계는 위 검사에 남긴다.
fn assert_pdf_origin(runs: &[BodyRun], page: u32, needle: &str, x: f64, baseline: f64) {
    let found: Vec<_> = runs
        .iter()
        .filter(|run| run.page == page && run.text == needle)
        .collect();
    assert_eq!(
        found.len(),
        1,
        "완료 페이지의 핵심 run은 한 번만 존재: {needle}"
    );
    assert!((found[0].bbox.x - x).abs() <= 2.0, "독립 PDF x: {needle}");
    assert!(
        (found[0].baseline - baseline).abs() <= 2.0,
        "독립 PDF 기준선: {needle}"
    );
}

#[test]
fn verified_continuation_pages_preserve_the_first_and_last_body_origins() {
    for (path, first, last_baseline) in [
        (
            "samples/issue1510_coanchored_float_tables.hwpx",
            "filler paragraph 30",
            186.19651,
        ),
        (
            "samples/hwpx/issue1535_coanchored_float_exclusion.hwpx",
            "filler paragraph 31",
            164.77992,
        ),
    ] {
        let core = DocumentCore::from_bytes(
            &std::fs::read(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap(),
        )
        .unwrap();
        assert_eq!(core.page_count(), 2, "독립 Print 전쪽 수");
        let mut tables = Vec::new();
        let mut runs = Vec::new();
        for page in 0..core.page_count() {
            collect(
                &core.build_page_render_tree(page).unwrap().root,
                page,
                &mut tables,
                &mut runs,
            );
        }
        assert_pdf_origin(&runs, 1, first, 113.37855, 143.52315);
        assert_pdf_origin(&runs, 1, "filler paragraph 32", 113.37855, last_baseline);
        let first_run = runs.iter().find(|r| r.text == first).unwrap();
        let final_run = runs
            .iter()
            .find(|r| r.text == "filler paragraph 32")
            .unwrap();
        assert!(
            first_run.paragraph < final_run.paragraph && first_run.bbox.y < final_run.bbox.y,
            "원문 순서와 실제 이어받기 순서"
        );
    }
}

#[test]
fn verified_hwp_host_metrics_keep_the_source_cache_and_clear_the_positive_frame() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue1510_coanchored_float_tables.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
    let original = format!("{:?}", core.document());
    assert_eq!(core.page_count(), 1, "새 한컴 2020 Print 1쪽");
    let mut tables = Vec::new();
    let mut runs = Vec::new();
    collect(
        &core.build_page_render_tree(0).unwrap().root,
        0,
        &mut tables,
        &mut runs,
    );
    let frame = tables.iter().find(|table| table.control == 2).unwrap();
    let before = runs
        .iter()
        .find(|run| run.text == "filler paragraph 07")
        .unwrap();
    let after = runs
        .iter()
        .find(|run| run.text == "filler paragraph 08")
        .unwrap();
    assert!(
        before.bbox.y + before.bbox.height <= frame.bbox.y,
        "일곱째 문단은 양수 오프셋 표 위의 남은 공간에 들어간다"
    );
    assert!(
        after.bbox.y >= frame.bbox.y + frame.bbox.height,
        "여덟째 문단은 표 본체 뒤에서 재개한다"
    );
    assert_pdf_origin(&runs, 0, "ISSUE 1510 CENTER TITLE", 306.87368, 204.25677);
    assert_pdf_origin(&runs, 0, "filler paragraph 32", 113.37855, 963.42708);
    assert_eq!(
        format!("{:?}", core.document()),
        original,
        "렌더 전용 메트릭 투영은 원본 저장 IR을 바꾸지 않는다"
    );
}
