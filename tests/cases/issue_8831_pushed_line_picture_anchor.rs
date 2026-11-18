//! 관계형: 오프셋 0 문단 기준 자리차지(TopAndBottom) 그림 아래로 밀린 줄을 가진 문단에서 그림이 앞 그림 아래·자기 줄 위에 놓이는지 검사한다.
//! 주석 갱신: 2026-11-09 15:13 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96` silhouette_raw_match_percent(색상 경계 보정 전 2px 내용 실루엣), Native(native-skia)·fresh WASM 각 101쪽 전쪽, 수정 전 = #8807 head b94ffbe3a1.
//! 원본 `samples/issue8795/1351000-201500141_arrhythmia_research_plan.hwp`(저장 제품 정보 없음), 전체 101쪽 = 한/글 2020 Print 101쪽.
//! 58쪽: Native 100.00 / fresh WASM 99.75(수정 전 69.61 / 70.07). 전쪽 최저: Native 97.76(99쪽) / fresh WASM 95.38(55쪽). 90% 미만 0쪽.
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8831/{base,head}/{native,wasm}/arrhythmia_research_plan_silhouette.tsv`, `png/`.
//! 확대 판독: 58쪽 review·overlay 를 한/글 2020 Print 와 같은 배율로 대조 — pi695 논문 표지 그림이 pi694 그림 바로 아래에서 시작하고, 캡션 「미국 브라운 대학교 Dr. Dudley 와 연구논문」이 그 아래 PDF 와 같은 자리에 있다.
//! 반례: 같은 문단의 앞 표까지 gap 에 든 그림(1490000-200800034 pi1090 표 뒤 원그래프)과 #5715 가드가 걸리지 않는 그림(`samples/issue6879/156767332-broadcast-revenue-attachment.hwp` pi46, `text_overlap_baseline`)은 종전 경로 — 공개 문서 corpus A/B 에서 바뀐 문서는 이 원본 58쪽 하나.
//! 같은 assertion 이 수정 전 FAIL(58쪽 pi695 그림이 자기 캡션 줄을 덮음) / 수정 후 PASS.
//! 독립 기대값: 한/글 2020 Print `pdf/issue8795/1351000-201500141_arrhythmia_research_plan-2020.pdf` 58쪽 — 두 그림이 차례로 쌓이고 캡션 줄이 아래 그림 밑에 있다.
//! 저장 근거: pi694 그림 띠 끝 27278 = pi695 문단 시작, pi695 그림 오프셋 0·높이 36661 → pi695 줄 63939 = 27278 + 36661.
//! 독립 절대 px 좌표 대신 그림·줄의 쪽 소속과 위아래 순서를 검사한다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8795/1351000-201500141_arrhythmia_research_plan.hwp";
const PAGE: u32 = 57; // 58쪽(0 기준)

#[derive(Default)]
struct Found {
    pictures: Vec<(usize, BoundingBox)>,
    caption_text_top: Option<f64>,
}

fn collect(node: &RenderNode, found: &mut Found) {
    match &node.node_type {
        RenderNodeType::Image(image) => {
            if let (Some(0), Some(para)) = (image.section_index, image.para_index) {
                found.pictures.push((para, node.bbox));
            }
        }
        RenderNodeType::TextRun(run)
            if run.section_index == Some(0)
                && run.para_index == Some(695)
                && !run.text.trim().is_empty() =>
        {
            let top = found.caption_text_top.get_or_insert(node.bbox.y);
            *top = top.min(node.bbox.y);
        }
        RenderNodeType::Table(_) => return,
        _ => {}
    }
    for child in &node.children {
        collect(child, found);
    }
}

#[test]
fn picture_above_its_pushed_line_stays_at_paragraph_anchor() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    let tree = core.build_page_render_tree(PAGE).expect("58쪽 배치");
    let mut found = Found::default();
    collect(&tree.root, &mut found);

    let paras: Vec<usize> = found.pictures.iter().map(|(para, _)| *para).collect();
    assert_eq!(paras, vec![694, 695], "58쪽 그림의 앵커 문단과 순서");
    let (upper, lower) = (&found.pictures[0].1, &found.pictures[1].1);
    assert!(
        lower.y + 0.5 >= upper.y + upper.height,
        "pi695 그림은 pi694 그림 아래에서 시작한다: {:?}",
        found.pictures
    );
    let caption_top = found.caption_text_top.expect("pi695 캡션 줄");
    assert!(
        caption_top + 0.5 >= lower.y + lower.height,
        "pi695 캡션 줄({caption_top:.1})은 자기 그림 아래에 있다: {lower:?}"
    );
}
