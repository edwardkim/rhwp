//! 관계형·고정형(쪽수): 공백 앵커 문단에 나란히 앵커한 문단 기준 표 묶음(어울림·자리차지)이 쪽 끝에 들어가지 않으면, 앵커 줄은 앞 쪽에 남고 표 둘만 다음 쪽 맨 위에 같은 높이로 놓이며 뒤 본문이 그 아래에서 시작하는지, 전체 쪽수가 독립 PDF 와 같은지 검사한다.
//! 주석 갱신: 2026-11-08 13:52 KST (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-08, `scripts/visual_sweep.py --silhouette-only` 4열 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 실루엣), 96 DPI, 기준 한/글 2020 Print.
//! 원본 `samples/issue8770/4680000-202100038_research_review_request.hwp`(한/글 2018 저장), 전체 6쪽 = 한/글 2020 Print 6쪽.
//! 2쪽(표 묶음): Native 99.02 / fresh WASM 98.95. 전쪽 최저: Native 99.02(2쪽) / fresh WASM 98.95(2쪽). 수정 전 같은 쪽 27.90 / 27.70, 전쪽 최저 2.52 / 2.78(4쪽, 쪽수 7).
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8770/{base,head}/{native,wasm}/research_review_request_silhouette.tsv`, `png/`.
//! 확대 판독: 2쪽 머리 두 표가 한/글과 같은 높이(본문 위 + 바깥 위 여백)에 나란히 놓이고 뒤 본문이 한/글과 같은 줄에서 시작한다(overlay 일치).
//! 같은 assertion 이 수정 전 FAIL(쪽수 7) / 수정 후 PASS.
//! 독립 기대값: 한/글 2020 Print `pdf/issue8770/4680000-202100038_research_review_request-2020.pdf`(6쪽). 2쪽 두 표 위끝 모두 107px(96 DPI) = 본문 위 105.8 + 바깥 위 여백.
//! 저장 근거: 1쪽 끝 공백 문단 pi22 의 첫 줄 vpos 66019 에서 두 표(높이 8184)는 본문에 들어가지 않는다. 2쪽 첫 문단 pi25 의 첫 줄 vpos 8466 = 바깥 위 141 + 8184 + 바깥 아래 141(넘어간 표는 세로 오프셋 −240 을 버린다).
//! 독립 절대 px 좌표 대신 쪽 소속·두 표의 같은 위끝·표와 뒤 본문의 위아래 관계를 검사한다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8770/4680000-202100038_research_review_request.hwp";
const PDF_PAGES: u32 = 6;
const HOST_PARA: usize = 22;

fn host_tables(node: &RenderNode, out: &mut Vec<BoundingBox>) {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.section_index == Some(0)
            && table.para_index == Some(HOST_PARA)
            && table.cell_context.is_none()
        {
            out.push(node.bbox);
        }
    }
    for child in &node.children {
        host_tables(child, out);
    }
}

fn body_lines_after_host(node: &RenderNode, out: &mut Vec<(usize, BoundingBox)>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.section_index == Some(0) {
            if let Some(para) = line.para_index.filter(|para| *para > HOST_PARA) {
                out.push((para, node.bbox));
            }
        }
    }
    // 표 안의 칸 글줄은 본문 흐름이 아니다.
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return;
    }
    for child in &node.children {
        body_lines_after_host(child, out);
    }
}

#[test]
fn whitespace_host_table_group_moves_to_next_page_without_its_line() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    assert_eq!(core.page_count(), PDF_PAGES, "독립 Print 와 같은 쪽수");

    // 1쪽에는 앵커 문단의 표가 없다(앵커 줄만 남는다).
    let first = core.build_page_render_tree(0).expect("1쪽 배치");
    let mut on_first = Vec::new();
    host_tables(&first.root, &mut on_first);
    assert!(on_first.is_empty(), "1쪽에 표가 남았다: {on_first:?}");

    // 2쪽 머리에 두 표가 같은 높이로 나란히 놓인다.
    let second = core.build_page_render_tree(1).expect("2쪽 배치");
    let mut tables = Vec::new();
    host_tables(&second.root, &mut tables);
    assert_eq!(tables.len(), 2, "2쪽에 앵커 문단의 표 둘: {tables:?}");
    assert!(
        (tables[0].y - tables[1].y).abs() <= 0.5,
        "넘어간 표 둘은 같은 위끝(세로 오프셋을 버린다): {tables:?}"
    );
    assert!(
        tables[0].x + tables[0].width <= tables[1].x
            || tables[1].x + tables[1].width <= tables[0].x,
        "두 표는 가로로 나란하다: {tables:?}"
    );

    // 뒤 본문은 두 표 아래에서 시작한다.
    let bottom = tables
        .iter()
        .map(|table| table.y + table.height)
        .fold(f64::MIN, f64::max);
    let mut lines = Vec::new();
    body_lines_after_host(&second.root, &mut lines);
    assert!(!lines.is_empty(), "2쪽 뒤 본문 글줄");
    for (para, line) in &lines {
        assert!(
            line.y >= bottom - 0.5,
            "pi{para} 글줄은 표 아래: line={line:?} tables={tables:?}"
        );
    }
}
