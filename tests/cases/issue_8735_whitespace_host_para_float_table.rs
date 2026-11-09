//! 관계형(밀린 host 줄 = 표 아래끝 + 바깥 아래 여백, ±1px)·고정형(저장 사다리 표 위끝 ±2px·쪽수): 공백만 있는 문단에 앵커한 문단 기준 자리차지 표가 문단 앞 간격만큼 내려가지 않고 「문단 상단 + 바깥 위 여백」에 놓이며, 표가 민 host 줄이 앞 간격·줄간격 없이 표 틀 바로 밑에 오는지 검사한다.
//! 주석 갱신: 2026-11-09T10:10:54+09:00 (Asia/Seoul). 직전 파일 커밋 0b54ab1879(2026-11-09 06:20:19 +0900, #8736 ee0d775f41 위 rebase).
//! 실제 측정: 2026-11-09 KST, base = #8736 head ee0d775f41, head = base + 이 PR 의 src 변경(조판 보완 포함). 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw.
//! 원본 `samples/issue8732/1741000-202200045_second_interim_report.hwp`, 전체 138쪽=138쪽.
//! 7쪽 수정 전 Native 25.20 / fresh WASM 23.57 → 수정 후 Native 99.98 / fresh WASM 99.82(양쪽 98% 이상이라 본문 기준 표 위끝을 ±2px 로 고정한다). 그 밖의 쪽은 수정 전후 같은 값.
//! 수정 후 전쪽 최저 Native 28.91(137쪽)·fresh WASM 26.95(137쪽). 90% 미만 Native·WASM 각 9쪽(15·66·67·68·73·134~137)은 이 변경 전부터 있던 다른 쪽의 미달이다(수정 전후 같은 값).
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8735/{base,head}/{native,wasm}/second_interim_report_silhouette.tsv`, `png/second_interim_report_p007_*`.
//! 확대 판독: 7쪽 표 테두리와 host 다음 본문이 한/글과 같은 높이(overlay 일치). 수정 전에는 문단 앞 간격만큼 18px 아래.
//! 같은 assertion 이 수정 전 FAIL(본문 위에서 94.40px) / 수정 후 PASS.
//! 독립 기대값: 한/글 Print `pdf/issue8732/1741000-202200045_second_interim_report-2020.pdf`(138쪽) 7쪽 표 위끝 208.4px(96 DPI).
//! 저장 근거: 문단 0.130(글 「  」)의 첫 줄 vpos 54151 = 문단 상단 5580 + 표 바깥 위 141 + 높이 48289 + 바깥 아래 141. 표 위끝 = 본문 위 + (54151 − 48289 − 141) HU. 같은 등식이 다른 문서(공백 host 표 쌓임·나란한 표 포함)에서도 성립함을 확인했다.
//! 독립 절대 px 좌표 대신 저장값 등식을 쓴다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::model::control::Control;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8732/1741000-202200045_second_interim_report.hwp";
const PDF_PAGES: u32 = 138;
const HOST_PARA: usize = 130;
const PAGE: u32 = 6;

fn host_table(node: &RenderNode) -> Option<BoundingBox> {
    if let RenderNodeType::Table(table) = &node.node_type {
        if table.section_index == Some(0)
            && table.para_index == Some(HOST_PARA)
            && table.cell_context.is_none()
        {
            return Some(node.bbox);
        }
    }
    node.children.iter().find_map(host_table)
}

fn host_lines(node: &RenderNode, out: &mut Vec<BoundingBox>) {
    if let RenderNodeType::TextLine(line) = &node.node_type {
        if line.section_index == Some(0) && line.para_index == Some(HOST_PARA) {
            out.push(node.bbox);
        }
    }
    for child in &node.children {
        host_lines(child, out);
    }
}

fn body(node: &RenderNode) -> Option<BoundingBox> {
    if matches!(node.node_type, RenderNodeType::Body { .. }) {
        return Some(node.bbox);
    }
    node.children.iter().find_map(body)
}

#[test]
fn whitespace_host_para_float_table_sits_at_paragraph_top() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    assert_eq!(core.page_count(), PDF_PAGES, "독립 Print 와 같은 쪽수");

    let para = &core.document().sections[0].paragraphs[HOST_PARA];
    assert!(
        !para.text.is_empty() && para.text.chars().all(char::is_whitespace),
        "앵커 문단은 공백만 있다: {:?}",
        para.text
    );
    let Some(Control::Table(table)) = para.controls.first() else {
        panic!("앵커 문단의 표");
    };
    let line = para.line_segs.first().expect("저장 줄");
    // 한/글은 표 틀 아래로 첫 줄을 민다: 표 위끝 = 줄 vpos − 높이 − 바깥 아래 여백.
    let top_hu =
        line.vertical_pos - table.common.height as i32 - i32::from(table.outer_margin_bottom);
    let expected_from_body = f64::from(top_hu) * 96.0 / 7200.0;

    let tree = core.build_page_render_tree(PAGE).expect("쪽 배치");
    let frame = host_table(&tree.root).expect("7쪽 표");
    let body = body(&tree.root).expect("본문 영역");
    let actual_from_body = frame.y - body.y;
    assert!(
        (actual_from_body - expected_from_body).abs() <= 2.0,
        "표 위끝은 본문 위에서 {expected_from_body:.2}px(저장 사다리): 실제 {actual_from_body:.2}px, table={frame:?}"
    );

    // 표가 민 host 자기 줄은 표 아래끝 + 바깥 아래 여백에 놓인다(앞 간격·줄간격 없음).
    let mut lines = Vec::new();
    host_lines(&tree.root, &mut lines);
    let pushed = lines
        .iter()
        .find(|l| l.y >= frame.y + frame.height - 0.5)
        .unwrap_or_else(|| panic!("표 아래의 host 줄: {lines:?} table={frame:?}"));
    let expected_gap = f64::from(table.outer_margin_bottom) * 96.0 / 7200.0;
    let gap = pushed.y - (frame.y + frame.height);
    assert!(
        (gap - expected_gap).abs() <= 1.0,
        "host 줄은 표 아래끝에서 바깥 아래 여백 {expected_gap:.2}px 밑: 실제 {gap:.2}px, line={pushed:?} table={frame:?}"
    );
}
