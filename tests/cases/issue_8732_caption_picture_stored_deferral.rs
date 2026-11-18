//! 관계형·고정형(쪽수): 쪽 끝 문단에 앵커한 아래 캡션 자리차지 그림을 한/글처럼 다음 쪽 맨 위로 미루는지(그림·캡션·뒤 본문의 쪽 소속과 앞뒤 순서), 전체 쪽수가 독립 PDF 와 같은지 검사한다.
//! 주석 갱신: 2026-11-07T15:10:27+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-07 KST, base = upstream/devel b8dd67c500, head = base + 이 PR 의 src 변경. 96 DPI·Print profile·색상 경계 보정 전 2px 내용 실루엣 raw.
//! 원본 `samples/issue8732/1741000-202200045_second_interim_report.hwp`, 전체 138쪽=138쪽.
//! 16·17쪽 수정 전 Native 71.09·51.88 / fresh WASM 72.88·53.72 → 수정 후 Native 100.00·100.00 / fresh WASM 99.98·99.83. 다른 쪽 값은 바뀌지 않았다.
//! 수정 후 전쪽 최저 Native 23.70·fresh WASM 23.34 — 90% 미만 Native 10쪽(7·15·66~68·73·134~137)·WASM 9쪽은 수정 전과 같은 값이며 #8735 에서 다룬다.
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8732/{base,head}/{native,wasm}/second_interim_report_silhouette.tsv`, `png/`.
//! 확대 판독: 17쪽 그림·캡션 「[그림 2-3] 의회 중심형」·「3. 문재인 정부 자치분권위원회(안)」·본문 첫 줄이 한/글과 같은 자리(수정 전 16쪽에서 그림이 본문을 덮음).
//! 같은 assertion 이 수정 전 FAIL(그림이 16쪽, 0 기준 15) / 수정 후 PASS. 되감긴 첫 문단 0.265 는 빈 문단이라 빈 문단의 다음 쪽 소유 경로도 함께 검사한다.
//! 독립 기대값: 한/글 2022 Print `pdf/issue8732/1741000-202200045_second_interim_report-2020.pdf`(138쪽). 17쪽(쪽 번호 13) 맨 위에 「그림 2-3」 과 캡션, 그 아래 본문.
//! 저장 근거: 앵커 문단 0.262 다음의 저장 줄이 그림 틀 + 캡션 간격 + 캡션 줄 끝에서 되감긴다.
//! 독립 절대 px 좌표를 고정하지 않는다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8732/1741000-202200045_second_interim_report.hwp";
const PDF_PAGES: u32 = 138;
const HOST_PARA: usize = 262;
/// 저장 줄이 그림 틀 + 캡션 아래에서 되감기는 첫 문단(vpos 16285 = 14335 + 850 + 1100).
/// 그 앞의 빈 문단과 「자료」 줄(0.263·0.264)은 한/글도 앞 쪽 끝에 둔다.
const RESET_PARA: usize = 265;

fn picture(node: &RenderNode) -> Option<BoundingBox> {
    if let RenderNodeType::Image(image) = &node.node_type {
        if image.section_index == Some(0) && image.para_index == Some(HOST_PARA) {
            return Some(node.bbox);
        }
    }
    node.children.iter().find_map(picture)
}

/// 본문 글줄 (문단 번호, 줄 상자). 표 셀·캡션 안 글은 문단 번호가 없거나 셀 문맥을 가진다.
fn body_runs(node: &RenderNode, out: &mut Vec<(usize, String, BoundingBox)>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if let (Some(para), None) = (run.para_index, run.cell_context.as_ref()) {
            out.push((para, run.text.clone(), node.bbox));
        }
    }
    for child in &node.children {
        body_runs(child, out);
    }
}

fn all_text(node: &RenderNode, out: &mut String) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        out.push_str(&run.text);
    }
    for child in &node.children {
        all_text(child, out);
    }
}

#[test]
fn bottom_caption_picture_moves_to_next_page_top_with_its_stored_successor() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    assert_eq!(core.page_count(), PDF_PAGES, "독립 Print 와 같은 쪽수");

    let pages: Vec<u32> = (0..core.page_count())
        .filter(|&page| {
            let tree = core.build_page_render_tree(page).expect("쪽 배치");
            picture(&tree.root).is_some()
        })
        .collect();
    assert_eq!(pages, vec![16], "그림 2-3 은 17쪽(0 기준 16)에만 있다");

    // 앞 쪽: 앵커 문단과 되감기 전 본문(「자료」 줄 포함)이 남고, 되감긴 본문은 없다.
    let before = core.build_page_render_tree(15).expect("쪽 배치");
    let mut runs = Vec::new();
    body_runs(&before.root, &mut runs);
    assert!(
        runs.iter()
            .any(|(para, text, _)| *para == RESET_PARA - 1 && text.contains("자료")),
        "되감기 전 「자료」 줄은 앞 쪽 끝에 남는다"
    );
    assert!(
        runs.iter().all(|(para, _, _)| *para < RESET_PARA),
        "되감긴 본문은 앞 쪽에 없다"
    );

    // 다음 쪽: 그림이 맨 위, 캡션과 저장 재시작 본문은 그림 아래.
    let after = core.build_page_render_tree(16).expect("쪽 배치");
    let frame = picture(&after.root).expect("그림");
    let mut runs = Vec::new();
    body_runs(&after.root, &mut runs);
    let successors: Vec<_> = runs
        .iter()
        .filter(|(para, _, _)| *para > HOST_PARA)
        .collect();
    assert!(
        !successors.is_empty(),
        "되감긴 본문이 그림과 같은 쪽에 온다"
    );
    for (para, text, bbox) in successors {
        assert!(
            bbox.y + 0.5 >= frame.y + frame.height,
            "문단 {para} 「{text}」가 그림 아래에서 시작한다: run={bbox:?} 그림={frame:?}"
        );
    }
    assert!(
        runs.iter()
            .all(|(para, _, bbox)| *para > HOST_PARA || bbox.y + 0.5 >= frame.y + frame.height),
        "그림 위에 다른 본문이 없다"
    );
    let mut text = String::new();
    all_text(&after.root, &mut text);
    assert!(
        text.replace(' ', "").contains("의회중심형"),
        "캡션(「[그림 2-3] 의회 중심형」)이 그림과 같은 쪽에 있다"
    );
}
