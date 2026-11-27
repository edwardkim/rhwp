//! 관계형·고정형 — #7910 본문 빈 문단 쪽 소속과 꼬리말 삭제 경계. 독립 한컴 Print와 전쪽 대조.
//! 주석 갱신: 2026-11-22 03:31:00 KST (새 파일)
//!
//! 측정: 2026-11-22 KST, 렌더 코드 `d5321686b8`. 한컴 2020 Hancom PDF와
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`로 Native 및 fresh WASM을
//! 비교했다. raw는 `silhouette_raw_match_percent`다. 아래 %는 각 원본의 첫 쪽부터 순서대로 적었다.
//! - `repro_footer_picture.hwp` 2쪽. Native 99.17% / 100.00%, WASM 99.18% / 100.00%.
//! - `control_footer_nopicture.hwp` 2쪽. Native 99.04% / 100.00%, WASM 99.05% / 100.00%.
//! - `hancom-original.hwpx` 2쪽. Native 99.17% / 100.00%, WASM 99.18% / 100.00%.
//! - `no-auto.hwp` 2쪽. Native 99.17% / 100.00%, WASM 99.18% / 100.00%.
//! - `no-auto.hwpx` 2쪽. Native 99.17% / 100.00%, WASM 99.18% / 100.00%.
//! - `no-picture.hwp` 2쪽. Native 99.11% / 100.00%, WASM 99.12% / 100.00%.
//! - `no-picture.hwpx` 2쪽. Native 99.11% / 100.00%, WASM 99.12% / 100.00%.
//! - `no-rect.hwp` 2쪽. Native 99.12% / 100.00%, WASM 99.13% / 100.00%.
//! - `no-rect.hwpx` 2쪽. Native 99.12% / 100.00%, WASM 99.13% / 100.00%.
//! - `no-final-body-paragraph.hwpx` 1쪽. Native 99.17%, WASM 99.18%.
//!
//! 원본별 전체 최저는 각 행의 최저 쪽이다. 전체 입력 최저는 Native 99.04%, fresh WASM 99.05%다.
//! 기준과 증거: `pdf/issue7910/`, `mydocs/pr/assets/issue_7910/measurements.json` 및 같은 폴더의
//! Native·WASM TSV, review·overlay. 페이지 수는 Print, 위치는 표와 빈 줄 및 개체의 관계로 확인한다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};
use std::path::Path;

const ORIGINAL: &str = "tests/fixtures/issue7909/repro_footer_picture.hwp";
const CONTROL: &str = "tests/fixtures/issue7909/control_footer_nopicture.hwp";
const FOOTER_TEXT: &str = "ABCDEF양식Version0.0[0000-00-05]";

fn open(fixture: &str) -> DocumentCore {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join(fixture);
    let bytes = std::fs::read(source).expect("공개 #7910 fixture");
    DocumentCore::from_bytes(&bytes).expect("문서 열기")
}

fn variant(name: &str) -> String {
    format!("tests/fixtures/issue7910/{name}")
}

fn body(root: &RenderNode) -> Vec<&RenderNode> {
    root.children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .flat_map(|node| &node.children)
        .flat_map(|column| &column.children)
        .collect()
}

fn text(node: &RenderNode) -> String {
    match &node.node_type {
        RenderNodeType::TextRun(run) => run
            .display_text
            .as_deref()
            .unwrap_or(&run.text)
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .collect(),
        _ => node.children.iter().map(text).collect(),
    }
}

fn assert_body_pages(fixture: &str, expected_pages: u32) {
    let doc = open(fixture);
    assert_eq!(doc.page_count(), expected_pages, "{fixture}");
    let first_page = doc.build_page_render_tree(0).unwrap();
    let items = body(&first_page.root);
    let tables: Vec<_> = items
        .iter()
        .filter(
            |node| matches!(&node.node_type, RenderNodeType::Table(t) if t.para_index == Some(2)),
        )
        .collect();
    assert_eq!(tables.len(), 1, "{fixture}: 본문 표 소속과 개수");
    let table = tables[0].bbox;
    let host_text: String = items
        .iter()
        .filter(|node| matches!(&node.node_type, RenderNodeType::TextLine(line) if line.para_index == Some(2)))
        .map(|node| text(node))
        .collect();
    assert_eq!(host_text, "가나다라마바사아자차카타파하", "{fixture}");
    let blank_rows: Vec<_> = items
        .iter()
        .filter_map(|node| match &node.node_type {
            RenderNodeType::TextLine(line) => {
                line.para_index.filter(|pi| *pi >= 3).map(|pi| (pi, *node))
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        blank_rows.iter().map(|(pi, _)| *pi).collect::<Vec<_>>(),
        (3..=9).collect::<Vec<_>>(),
        "{fixture}: 표 뒤 빈 줄의 순서와 쪽 소속"
    );
    for pair in blank_rows.windows(2) {
        let a = pair[0].1.bbox;
        let b = pair[1].1.bbox;
        assert!(a.y + a.height <= b.y, "{fixture}: 빈 줄 겹침");
    }
    let beside = blank_rows[5].1.bbox;
    let below = blank_rows[6].1.bbox;
    assert!(beside.y < table.y + table.height, "{fixture}");
    assert!(below.y >= table.y + table.height, "{fixture}");
    assert!(below.x < beside.x, "{fixture}: 표 뒤 문단의 전폭 복귀");
    if expected_pages == 2 {
        let second_page = doc.build_page_render_tree(1).unwrap();
        let items = body(&second_page.root);
        let blank: Vec<_> = items
            .into_iter()
            .filter(|node| {
                matches!(
                    node.node_type,
                    RenderNodeType::Table(_) | RenderNodeType::TextLine(_)
                )
            })
            .collect();
        assert_eq!(blank.len(), 1, "{fixture}: 둘째 쪽의 본문 항목");
        assert!(
            matches!(&blank[0].node_type, RenderNodeType::TextLine(line) if line.para_index == Some(10))
        );
        assert!(text(blank[0]).is_empty(), "{fixture}: 마지막 빈 줄");
    }
}

struct Footer {
    content: String,
    line: BoundingBox,
    band: Option<BoundingBox>,
    image: Option<BoundingBox>,
}

fn footer(root: &RenderNode) -> Footer {
    let area = root
        .children
        .iter()
        .find(|node| matches!(node.node_type, RenderNodeType::Footer))
        .expect("꼬리말 소속");
    let mut lines = area
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::TextLine(_)));
    let line = lines.next().expect("꼬리말 글줄");
    assert!(lines.next().is_none(), "꼬리말 글줄 중복");
    let band = area
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Rectangle(_)));
    let image = area
        .children
        .iter()
        .find(|n| matches!(n.node_type, RenderNodeType::Image(_)));
    Footer {
        content: text(line),
        line: line.bbox,
        band: band.map(|n| n.bbox),
        image: image.map(|n| n.bbox),
    }
}

fn assert_clears_band(area: &Footer) -> BoundingBox {
    let band = area.band.expect("검은 사각형");
    assert!(
        area.line.y + 1e-6 >= band.y + band.height,
        "꼬리말 글줄과 띠 겹침"
    );
    band
}

#[test]
fn square_table_following_empty_rows_keep_their_page() {
    for fixture in [ORIGINAL, CONTROL] {
        assert_body_pages(fixture, 2);
    }
    assert_body_pages(&variant("hancom-original.hwpx"), 2);
    for ext in ["hwp", "hwpx"] {
        for name in ["no-auto", "no-picture", "no-rect"] {
            assert_body_pages(&variant(&format!("{name}.{ext}")), 2);
        }
    }
    assert_body_pages(&variant("no-final-body-paragraph.hwpx"), 1);
}

#[test]
fn removing_one_footer_control_preserves_the_others_and_their_flow() {
    let original = open(ORIGINAL);
    let original_hwpx = open(&variant("hancom-original.hwpx"));
    let public_control = open(CONTROL);
    for page in 0..2 {
        let expected = format!("{}{FOOTER_TEXT}", page + 1);
        for source in [&original, &original_hwpx] {
            let area = footer(&source.build_page_render_tree(page).unwrap().root);
            assert_eq!(area.content, expected);
            let band = assert_clears_band(&area);
            let image = area.image.expect("꼬리말 그림");
            assert!((image.y - band.y).abs() < 1e-6, "그림과 사각형의 공통 원점");
        }
        let control = footer(&public_control.build_page_render_tree(page).unwrap().root);
        assert_eq!(control.content, format!("{FOOTER_TEXT}05]"));
        assert!(control.band.is_none() && control.image.is_none());
        for ext in ["hwp", "hwpx"] {
            let inspect = |name| {
                let candidate = open(&variant(&format!("{name}.{ext}")));
                footer(&candidate.build_page_render_tree(page).unwrap().root)
            };
            let no_auto = inspect("no-auto");
            assert_eq!(no_auto.content, FOOTER_TEXT);
            assert!(no_auto.image.is_some());
            let band = assert_clears_band(&no_auto);
            let no_picture = inspect("no-picture");
            assert_eq!(no_picture.content, expected);
            assert!(no_picture.image.is_none());
            assert_clears_band(&no_picture);
            let no_rect = inspect("no-rect");
            assert_eq!(no_rect.content, expected);
            assert!(no_rect.band.is_none() && no_rect.image.is_some());
            assert!((no_auto.line.y - no_rect.line.y - band.height).abs() < 1e-6);
        }
    }
}
