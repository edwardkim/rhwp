//! Issue #1733: 국제고속선기준 본문·각주 재시작 경계의 쪽 소유 회귀 방지.
//!
//! HWP 2020 MCP PDF의 242쪽과 실제 본문 분할을 두 형식에서 유지한다.
//! 같은 쪽수가 추가 빈 쪽과 누락된 뒤쪽 내용을 상쇄한 결과인지도 검사한다.
//! 기대 컷은 기준 PDF의 56/57·109/110·200/201·205/206·219/220쪽 텍스트와
//! 원본 저장 줄의 대응으로 정했다. 문서 전체의 Native/fresh WASM
//! 시각 검증으로 확인하고 기존 두 검사에서 실제 소유와 예약 경계를 검사한다.

use rhwp::wasm_api::HwpDocument;
use std::fs;
use std::path::Path;

const HANCOM_PDF_PAGE_COUNT: u32 = 242;

fn load_doc(sample: &str) -> HwpDocument {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(repo_root).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|err| panic!("parse {}: {err:?}", path.display()))
}

fn assert_hancom_pdf_page_count(sample: &str) {
    let doc = load_doc(sample);
    // 한 경계의 실패가 뒤쪽의 실제 소유 검사까지 건너뛰지 않게 모두 수집한다.
    let mut failures = Vec::new();
    if doc.page_count() != HANCOM_PDF_PAGE_COUNT {
        failures.push(format!(
            "{sample}: 한컴 PDF {HANCOM_PDF_PAGE_COUNT}쪽과 실제 {}쪽이 다르다",
            doc.page_count()
        ));
    }
    // 0부터 세는 API 쪽 번호와 실제 PDF 쪽 번호를 명시적으로 구분한다.
    let expected: &[(u32, &[&str])] = &[
        (56, &["PartialParagraph  pi=1217  lines=0..3"]),
        (
            57,
            &[
                "PartialParagraph  pi=1217  lines=3..5",
                "FullParagraph  pi=1218",
            ],
        ),
        (109, &["PartialParagraph  pi=2226  lines=0..3"]),
        (110, &["PartialParagraph  pi=2226  lines=3..4"]),
        (200, &["PartialParagraph  pi=4348  lines=0..2"]),
        (
            201,
            &[
                "Table          pi=4342 ci=0",
                "PartialParagraph  pi=4348  lines=2..3",
                "Table          pi=4348 ci=0",
            ],
        ),
        (205, &["PartialParagraph  pi=4447  lines=0..6"]),
        (206, &["PartialParagraph  pi=4447  lines=6..7"]),
        (219, &["PartialParagraph  pi=4726  lines=0..1"]),
        (
            220,
            &[
                "PartialParagraph  pi=4726  lines=1..2",
                "FullParagraph  pi=4731",
            ],
        ),
    ];
    for &(pdf_page, items) in expected {
        let actual = doc.dump_page_items(Some(pdf_page - 1));
        for &item in items {
            if actual.matches(item).count() != 1 {
                failures.push(format!(
                    "{sample}: PDF {pdf_page}쪽의 본문·개체 소유가 일치해야 한다: {item}\n{actual}"
                ));
            }
        }
    }
    // 전체 문단으로 잘못 합쳐져도 쪽수만 우연히 맞는 거짓 양성을 막는다.
    for (pdf_page, para) in [
        (56, 1217),
        (57, 1217),
        (109, 2226),
        (110, 2226),
        (200, 4348),
        (201, 4348),
        (205, 4447),
        (206, 4447),
        (219, 4726),
        (220, 4726),
    ] {
        let actual = doc.dump_page_items(Some(pdf_page - 1));
        if actual.contains(&format!("FullParagraph  pi={para} ")) {
            failures.push(format!(
                "{sample}: PDF {pdf_page}쪽의 분할 문단 {para}를 전체 문단으로 중복 소유하면 안 된다\n{actual}"
            ));
        }
    }
    // 소수1자리 JSON 좌표를 재합산하지 않고 실제 f64 경계로 예약과 배치를 비교한다.
    let tree = doc
        .build_page_render_tree(195)
        .expect("PDF196쪽의 실제 렌더 트리");
    if let Some(area) = tree.root.children.iter().find(|node| {
        matches!(
            node.node_type,
            rhwp::renderer::render_tree::RenderNodeType::FootnoteArea
        )
    }) {
        let area_end = area.bbox.y + area.bbox.height;
        for line in &area.children {
            if matches!(
                line.node_type,
                rhwp::renderer::render_tree::RenderNodeType::TextLine(_)
            ) && line.bbox.y + line.bbox.height > area_end + 1e-8
            {
                failures.push(format!(
                    "{sample}: PDF196쪽의 각주 실제 끝 {}가 예약 끝 {area_end}를 넘는다",
                    line.bbox.y + line.bbox.height
                ));
            }
        }
    } else {
        failures.push(format!("{sample}: PDF196쪽의 각주 영역이 있어야 한다"));
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn issue_1733_hwpx_matches_hancom_pdf_page_count() {
    assert_hancom_pdf_page_count("samples/task1725/text_footnote_tail_overpagination.hwpx");
}

#[test]
fn issue_1733_hwp_matches_hancom_pdf_page_count() {
    assert_hancom_pdf_page_count("samples/task1725/text_footnote_tail_overpagination.hwp");
}
