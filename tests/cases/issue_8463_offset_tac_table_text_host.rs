//! 관계형·고정형 회귀 검사. #8463 원본과 메인테이너 경계 사례.
//! 주석 갱신 2026-11-26 11:26 KST. 직전 파일 변경 24c34515e8 2026-11-26T00:27:27+09:00.
//! 독립 한컴 Print와 96 DPI의 2px raw 실루엣. 전쪽 TSV · 확대 판독: `mydocs/pr/assets/hyunmyungjanelee-20261126/final/README.md`.
//! `tac_visible_prefix` 전 1쪽. Native 99.82%, fresh WASM 99.77% (쪽 순서).
//! `tac_visible_suffix` 전 1쪽. Native 99.80%, fresh WASM 99.75% (쪽 순서).
//!
//! 관계형·고정형: [#8463] 저장 줄 없는 host 의 양수 세로 오프셋 TAC 표는 표 높이를 한 번만 계상한다.
//!
//! 입력: `samples/issue8463/tac-offset-text-host.hwp`. 한컴오피스 2020 저장 공개 샘플
//! `samples/issue6861/3194097-performance-rating-roster.hwp` 를 `gen_tac_offset_text_host.py` 로
//! 레코드 단위로 고친 HWP5 다(본문 문단의 PARA_LINE_SEG 만 삭제하고 칸 안 저장 줄은 유지, 문단 2
//! TAC 표 세로 오프셋 3000HU, 표 뒤 공백 1칸). 기준: 한컴 2020 Print `pdf/issue8463/tac-offset-text-host-2020.pdf`(1쪽).
//!
//! 조판의 표 앞 글 경로(양수 세로 오프셋 + 글자 있는 host)는 host 의 줄을 모두 표 앞 글로 쌓은
//! 뒤 표 높이를 따로 전진한다. 그런데 이 host 의 줄은 표 줄이다. `b1ee63475d` 이후 줄 재조판이
//! 표 실측 높이를 프레임 줄의 점유로 발행했고, 그 프레임이 없어도 표 뒤 공백만 있는 host 는 표 줄
//! 하나로 조판한다(#8574). 그래서 표 높이가 두 번 계상되어 수정 전(upstream/devel `50324d194d`)에는
//! 1쪽 단 사용 높이 1744.8px, 전체 2쪽이었고 표가 1쪽 본문 아래 경계 밖에 그려졌다.
//! - 고정형: 전체 쪽수 1(= PDF).
//! - 관계형: 문단 2 의 표는 1쪽 본문 영역 안에 있다.
//! - 관계형 대조: 세로 오프셋을 0 으로 되돌린 같은 문서(표 앞 글 경로 밖)도 1쪽, 표는 본문 안이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use std::path::Path;

const SAMPLE: &str = "samples/issue8463/tac-offset-text-host.hwp";
/// 한컴 2020 Print PDF 의 쪽수.
const PDF_PAGES: u32 = 1;
const HOST: usize = 2;

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("fixture 를 읽을 수 없다 ({}): {error}", path.display()));
    DocumentCore::from_bytes(&bytes).expect("문서 로드")
}

fn find<'a>(node: &'a RenderNode, pred: &dyn Fn(&RenderNode) -> bool) -> Option<&'a RenderNode> {
    if pred(node) {
        return Some(node);
    }
    node.children.iter().find_map(|child| find(child, pred))
}

/// 1쪽 (본문 윗변, 본문 아랫변, 표 윗변, 표 아랫변).
fn first_page_table(core: &DocumentCore) -> (f64, f64, f64, f64) {
    let tree = core.build_page_render_tree(0).expect("1쪽 렌더 트리");
    let body = find(&tree.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Body { .. })
    })
    .expect("본문 영역");
    let table = find(&tree.root, &|node| {
        matches!(&node.node_type, RenderNodeType::Table(table)
            if table.para_index == Some(HOST) && table.control_index == Some(0))
    })
    .expect("1쪽에 문단 2 의 표가 있어야 한다");
    (
        body.bbox.y,
        body.bbox.y + body.bbox.height,
        table.bbox.y,
        table.bbox.y + table.bbox.height,
    )
}

fn assert_table_inside_first_page(core: &DocumentCore, case: &str) {
    assert_eq!(
        core.page_count(),
        PDF_PAGES,
        "{case}: 한/글 Print 와 같은 쪽수"
    );
    let (body_top, body_bottom, table_top, table_bottom) = first_page_table(core);
    assert!(
        table_top >= body_top - 0.5 && table_bottom <= body_bottom + 0.5,
        "{case}: 1쪽 표({table_top:.1}..{table_bottom:.1})는 본문({body_top:.1}..{body_bottom:.1}) 안에 있어야 한다"
    );
}

#[test]
fn offset_tac_table_in_whitespace_host_is_charged_once() {
    let core = load();
    let host = &core.document().sections[0].paragraphs[HOST];
    assert!(host.line_segs.is_empty(), "문단 2 는 저장 줄이 없다");
    assert!(
        !host.text.is_empty() && host.text.chars().all(char::is_whitespace),
        "문단 2 의 글은 공백뿐"
    );
    let Some(Control::Table(table)) = host.controls.first() else {
        panic!("문단 2 의 첫 컨트롤은 표");
    };
    assert!(table.common.treat_as_char, "글자처럼 취급 표");
    assert!(table.common.vertical_offset as i32 > 0, "양수 세로 오프셋");
    assert_table_inside_first_page(&core, "오프셋 3000HU");
}

#[test]
fn tac_table_without_offset_stays_on_the_first_page() {
    let mut core = load();
    {
        let host = &mut core.document_mut().sections[0].paragraphs[HOST];
        let Some(Control::Table(table)) = host.controls.first_mut() else {
            panic!("문단 2 의 첫 컨트롤은 표");
        };
        table.common.vertical_offset = 0;
    }
    let document = core.document().clone();
    core.set_document(document);
    assert_table_inside_first_page(&core, "오프셋 0");
}

#[test]
fn visible_text_before_and_after_the_tac_table_keeps_its_printed_order() {
    for (case, prefix) in [("tac_visible_prefix", true), ("tac_visible_suffix", false)] {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("samples/issue8463/{case}.hwp"));
        let mut core = DocumentCore::from_bytes(&std::fs::read(path).unwrap()).unwrap();
        for zero_offset in [false, true] {
            if zero_offset {
                let mut document = core.document().clone();
                let Control::Table(table) = &mut document.sections[0].paragraphs[HOST].controls[0]
                else {
                    panic!("문단 2의 인라인 표");
                };
                table.common.vertical_offset = 0;
                core.set_document(document);
            }
            assert_table_inside_first_page(&core, case);
            let page = core.build_page_render_tree(0).unwrap();
            let mark = find(&page.root, &|node| matches!(&node.node_type, RenderNodeType::TextRun(run)
                if run.para_index == Some(HOST) && run.cell_context.is_none() && run.text.trim() == "X"))
                .expect("본문의 X와 표가 같은 쪽에 있어야 한다");
            let (body_top, body_bottom, table_top, table_bottom) = first_page_table(&core);
            assert!(mark.bbox.y >= body_top && mark.bbox.y + mark.bbox.height <= body_bottom + 0.5);
            if prefix {
                assert!(
                    mark.bbox.y + mark.bbox.height <= table_top + 0.5,
                    "{case}: 선행 글자와 표가 겹친다"
                );
            } else {
                assert!(
                    mark.bbox.y >= table_bottom - 0.5,
                    "{case}: 뒤 글자와 표가 겹친다"
                );
            }
        }
    }
}
