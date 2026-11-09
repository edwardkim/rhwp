//! 관계형: 문단마다 하나씩 앵커한 문단 기준 자리차지(TopAndBottom) 그림 셋이 겹치지 않고 앵커 문단 순서대로 위에서 아래로 쌓이는지 검사한다.
//! 주석 갱신: 2026-11-09 12:20 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96` silhouette_raw_match_percent(색상 경계 보정 전 2px 내용 실루엣), Native(native-skia)·fresh WASM 각 101쪽 전쪽, 수정 전 = #8806 head 2bbc3b1773.
//! 원본 `samples/issue8795/1351000-201500141_arrhythmia_research_plan.hwp`(저장 제품 정보 없음), 전체 101쪽 = 한/글 2020 Print 101쪽.
//! 54쪽: Native 99.99 / fresh WASM 99.50(수정 전 27.88 / 29.09). 전쪽 최저: Native 69.61(58쪽) / fresh WASM 70.07(58쪽). 90% 미만 Native·WASM 각 1쪽(58쪽)은 수정 전과 같은 값이며 이 범위 밖이다(58쪽 그림이 아래 캡션 글을 가림, 별도 이슈).
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8807/{base,head}/{native,wasm}/arrhythmia_research_plan_silhouette.tsv`, `png/`.
//! 확대 판독: 54쪽 review·overlay 를 한/글 2020 Print 와 같은 배율로 대조 — 본문 세 줄 아래 원그래프 셋(브루가다·특발성 심실세동·긴 QT)이 PDF 와 같은 자리에 차례로 쌓이고 테두리·범례가 겹치지 않는다.
//! 같은 assertion 이 수정 전 FAIL(54쪽 pi613 그림이 pi612 그림과 겹침) / 수정 후 PASS.
//! 독립 기대값: 한/글 2020 Print `pdf/issue8795/1351000-201500141_arrhythmia_research_plan-2020.pdf` 54쪽 — 원그래프 셋이 차례로 쌓인다.
//! 저장 근거: pi612 줄 4800·오프셋 713·그림 17711 → pi613 줄 23224, pi613 오프셋 1105(줄 높이 1000 이상) → pi614 줄 42040 = 23224 + 1105 + 17711, pi614 오프셋 1360 → pi615 줄 61111.
//! 반례(정식 검사 아님 — 해당 쪽이 수정 전부터 90% 미만): 저장 줄이 띠 아래끝에 놓인 글 없는 host(3780000-202000005 pi1042 줄 30849 = 앞 줄 끝 + 오프셋 3409 + 그림 23700)는 줄이 띠 위에 있다고 보지 않는다 — 공개 문서 corpus A/B 에서 수정 전과 SVG 동일.
//! 독립 절대 px 좌표 대신 그림의 쪽 소속·순서·서로 겹치지 않음을 검사한다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8795/1351000-201500141_arrhythmia_research_plan.hwp";
const PAGE: u32 = 53; // 54쪽(0 기준)

fn collect(node: &RenderNode, pictures: &mut Vec<(usize, BoundingBox)>) {
    match &node.node_type {
        RenderNodeType::Image(image) => {
            if let (Some(0), Some(para)) = (image.section_index, image.para_index) {
                pictures.push((para, node.bbox));
            }
        }
        RenderNodeType::Table(_) => return,
        _ => {}
    }
    for child in &node.children {
        collect(child, pictures);
    }
}

#[test]
fn stacked_paragraph_topbottom_pictures_do_not_overlap() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    let tree = core.build_page_render_tree(PAGE).expect("54쪽 배치");
    let mut pictures = Vec::new();
    collect(&tree.root, &mut pictures);

    let paras: Vec<usize> = pictures.iter().map(|(para, _)| *para).collect();
    assert_eq!(paras, vec![612, 613, 614], "54쪽 그림의 앵커 문단과 순서");
    for pair in pictures.windows(2) {
        let (upper, lower) = (&pair[0], &pair[1]);
        assert!(
            lower.1.y + 0.5 >= upper.1.y + upper.1.height,
            "pi{} 그림은 pi{} 그림 아래에서 시작한다: {pictures:?}",
            lower.0,
            upper.0
        );
    }
}
