//! 관계형: 뒤 구역 편집 후 머리말 상속과 수정하지 않은 본문·중첩 셀 프레임을 보존한다.
//! 측정: 2026-10-29, canonical Visual Sweep raw2px/96DPI, source f8f31d0166.
//! 원본 samples/exam_social.hwp; 삽입/삽입·삭제 저장본과 독립 Hancom2020 Print는
//! samples/pr8040/ 및 pdf/pr8040/exam-social-{insert,insert-delete}-2020.pdf.
//! 삽입·삭제 전4쪽 Native: 98.37/98.49/98.58/98.49%, fresh WASM: 99.83/98.74/99.09/99.07%.
//! 삽입 전4쪽 Native: 98.37/98.49/98.58/98.49%, fresh WASM: 99.83/98.75/99.09/99.07%.
//! 각 경로/독립 PDF 4쪽 일치, 최저 Native98.37% / fresh WASM98.74%, 미달·누락0.
//! 2·4쪽 머리말/본문/표/후속 간격과 4쪽 중첩 셀·분수를 확대 판독했다.
//! 정식 증거: mydocs/pr/assets/issue_8040/README.md 및 전쪽 TSV·대표 review/overlay PNG.
//! 독립 PDF가 보존한 중립 편집의 배치 불변 관계를 검사하며 절대 px 기준을 만들지 않는다.
//! 원 머리말 결함 및 본문만 미보정한 1a7c5ad250에서 FAIL, 보정 source에서 PASS.
//! 설치 글꼴을 전제하지 않으며 같은 실행 환경에서 원문·편집 후 소유/순서/상자를 비교한다.
#![cfg(not(target_arch = "wasm32"))]
use rhwp::{
    renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType},
    DocumentCore,
};
#[derive(Debug)]
struct Placement {
    owner: String,
    bounds: BoundingBox,
}
fn placements(node: &RenderNode, path: &str, out: &mut Vec<Placement>) {
    let owner = match &node.node_type {
        RenderNodeType::TextRun(r) => Some(format!(
            "text:{}:{:?}:{:?}:{:?}:{}",
            path,
            r.section_index,
            r.para_index,
            r.cell_context,
            r.display_or_text()
        )),
        RenderNodeType::Table(t) => Some(format!(
            "table:{}:{:?}:{:?}:{:?}",
            path, t.section_index, t.para_index, t.cell_context
        )),
        RenderNodeType::TableCell(c) => Some(format!(
            "cell:{}:{}:{}:{}:{}",
            path, c.row, c.col, c.row_span, c.col_span
        )),
        _ => None,
    };
    if let Some(owner) = owner {
        out.push(Placement {
            owner,
            bounds: node.bbox,
        });
    }
    for (i, c) in node.children.iter().enumerate() {
        placements(c, &format!("{path}/{i}"), out);
    }
}
fn snapshot(d: &DocumentCore) -> Vec<Vec<Placement>> {
    (0..d.page_count())
        .map(|p| {
            let mut out = vec![];
            placements(&d.build_page_render_tree(p).unwrap().root, "", &mut out);
            out
        })
        .collect()
}
#[test]
fn neutral_later_section_edit_preserves_body_and_nested_cell_frames() {
    let mut d = DocumentCore::from_bytes(include_bytes!("../../samples/exam_social.hwp")).unwrap();
    let before = snapshot(&d);
    assert_eq!(before.len(), 4);
    d.insert_text_native(1, 0, 1, "가").unwrap();
    d.delete_text_native(1, 0, 1, 1).unwrap();
    let after = snapshot(&d);
    assert_eq!(after.len(), before.len());
    for (p, (old, new)) in before.iter().zip(&after).enumerate() {
        assert_eq!(
            old.len(),
            new.len(),
            "page {} content/objects preserved",
            p + 1
        );
        for (a, b) in old.iter().zip(new) {
            assert_eq!(a.owner, b.owner, "page {} ownership/order/text", p + 1);
            for (x, y) in [
                (a.bounds.x, b.bounds.x),
                (a.bounds.y, b.bounds.y),
                (a.bounds.width, b.bounds.width),
                (a.bounds.height, b.bounds.height),
            ] {
                assert!(
                    (x - y).abs() < 1e-7,
                    "page {} {:?}: unchanged input must preserve its frame: {:?} -> {:?}",
                    p + 1,
                    a.owner,
                    a.bounds,
                    b.bounds
                );
            }
        }
    }
}

fn header_text(node: &RenderNode, in_header: bool, output: &mut String) {
    let in_header = in_header || matches!(node.node_type, RenderNodeType::Header);
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if in_header {
            output.push_str(run.display_or_text());
        }
    }
    for child in &node.children {
        header_text(child, in_header, output);
    }
}
fn headers(core: &DocumentCore) -> Vec<String> {
    (0..core.page_count())
        .map(|page| {
            let mut output = String::new();
            header_text(
                &core.build_page_render_tree(page).unwrap().root,
                false,
                &mut output,
            );
            output
        })
        .collect()
}
#[test]
fn inserted_text_keeps_inherited_even_headers_in_render_tree() {
    let mut core =
        DocumentCore::from_bytes(include_bytes!("../../samples/exam_social.hwp")).unwrap();
    let before = headers(&core);
    assert_eq!(before.len(), 4, "독립 Print 전체 쪽수");
    for page in [1, 3] {
        assert!(before[page].contains("사회·문화"));
    }
    let original = core.document().sections[1].paragraphs[0].text.clone();
    core.insert_text_native(1, 0, 1, "가").unwrap();
    assert_ne!(core.document().sections[1].paragraphs[0].text, original);
    assert_eq!(
        headers(&core),
        before,
        "실제 삽입 직후 홀짝 머리말 소유 보존"
    );
    core.delete_text_native(1, 0, 1, 1).unwrap();
    assert_eq!(core.document().sections[1].paragraphs[0].text, original);
    assert_eq!(headers(&core), before, "삭제 직후에도 상속 슬롯 보존");
}
