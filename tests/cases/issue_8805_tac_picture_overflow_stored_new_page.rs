//! 관계형: 쪽 위에서 시작해 아래로 길게 내려오는 글자처럼 취급한 그림 뒤에 저장 vpos 0 으로 시작하는 그림 문단이 통째로 들어가지 않으면, 하단 여백으로 흘리지 않고 다음 쪽 첫머리로 넘기는지를 그림의 쪽 소속으로 검사한다.
//! 주석 갱신: 2026-11-09 08:03 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96` silhouette_raw_match_percent(색상 경계 보정 전 2px 내용 실루엣), Native(native-skia)·fresh WASM 각 101쪽 전쪽, 수정 전 = #8803 head f5d4a22ef2.
//! 원본 `samples/issue8795/1351000-201500141_arrhythmia_research_plan.hwp`(저장 제품 정보 없음), 전체 101쪽 = 한/글 2020 Print 101쪽.
//! 34쪽: Native 99.91 / fresh WASM 99.92. 35쪽: 99.57 / 99.60. 45쪽: 99.91 / 99.92. 46쪽: 99.57 / 99.60(수정 전 37.62/38.90, 1.54/1.67). 전쪽 최저: Native 27.88(54쪽) / fresh WASM 29.09(54쪽). 90% 미만 Native·WASM 각 2쪽(54·58)은 수정 전과 같은 값이며 이 범위 밖이다(54쪽 자리차지 그림 셋 겹침, 58쪽 그림 아래 줄 누락).
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8805/{base,head}/{native,wasm}/arrhythmia_research_plan_silhouette.tsv`, `png/`.
//! 확대 판독: 34·35쪽 review·overlay 를 한/글 2020 Print 와 같은 배율로 대조 — 34쪽은 pi454 서식 그림 하나만 PDF 와 같은 자리에 남고, 35쪽은 pi455 그림이 맨 위·본문 두 줄이 그 아래로 PDF 와 위아래 경계가 겹친다(overlay 주황은 색상 차이).
//! 같은 assertion 이 수정 전 FAIL(34쪽에 pi454·pi455 그림 둘, 본문 아래로 53.6px) / 수정 후 PASS.
//! 독립 기대값: 한/글 2020 Print `pdf/issue8795/1351000-201500141_arrhythmia_research_plan-2020.pdf` — 34·45쪽은 그림 하나, 35·46쪽은 다음 그림이 맨 위.
//! 저장 근거: pi454 시작 1700·아래끝 34496(줄 간격 포함 35196), pi455 vpos 0·높이 34589. 본문 65760HU 에 35196 + 34589 = 69785 로 들어가지 않는다. 45쪽 pi539·pi540 도 같은 형상.
//! 반례: 통째로 들어가는 vpos 0 문단은 같은 쪽에 남는다(`tests/fixtures/pr8036_stored_tall_line/zero.hwpx`, `verified_stored_tall_line_print_flow`).
//! 독립 절대 px 좌표 대신 그림의 쪽 소속과 쪽마다 그림 수를 검사한다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8795/1351000-201500141_arrhythmia_research_plan.hwp";

/// 본문 그림의 앵커 문단 번호. 표 안 그림은 셀 문맥을 가지므로 세지 않는다.
fn pictures(node: &RenderNode, out: &mut Vec<usize>) {
    if let RenderNodeType::Image(image) = &node.node_type {
        if let (Some(0), Some(para)) = (image.section_index, image.para_index) {
            out.push(para);
        }
    }
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return;
    }
    for child in &node.children {
        pictures(child, out);
    }
}

fn page_pictures(core: &DocumentCore, page: u32) -> Vec<usize> {
    let tree = core.build_page_render_tree(page).expect("쪽 배치");
    let mut out = Vec::new();
    pictures(&tree.root, &mut out);
    out
}

#[test]
fn stored_new_page_tac_picture_moves_instead_of_spilling_below_body() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    // (앞 그림이 남는 쪽, 다음 쪽, 앞 그림 문단, 넘어가는 그림 문단) — 0 기준 쪽 번호.
    for (page, next_page, upper, moved) in [(33, 34, 454, 455), (44, 45, 539, 540)] {
        assert_eq!(
            page_pictures(&core, page),
            vec![upper],
            "{}쪽에는 pi{upper} 그림만 남는다",
            page + 1
        );
        let next = page_pictures(&core, next_page);
        assert_eq!(
            next.first(),
            Some(&moved),
            "{}쪽 첫 그림은 pi{moved}: {next:?}",
            next_page + 1
        );
    }
}
