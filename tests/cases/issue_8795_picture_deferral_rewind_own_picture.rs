//! 관계형: 쪽 끝 빈 문단에 앵커한 자리차지 그림이 들어가지 않으면 앵커 줄과 뒤 빈 문단은 앞 쪽에 남고 그림만 다음 쪽 맨 위로 가는지, 다음 쪽을 여는 문단이 자기 그림을 가져도 그 아래에 놓이는지, 정확히 본문 끝에 닿는 그림이 제 쪽에 남는지를 그림의 쪽 소속·순서로 검사한다.
//! 주석 갱신: 2026-11-09 05:30 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96` silhouette_raw_match_percent(색상 경계 보정 전 2px 내용 실루엣), Native(native-skia)·fresh WASM 각 101쪽 전쪽, 수정 전 = #8736 head ee0d775f41.
//! 원본 `samples/issue8795/1351000-201500141_arrhythmia_research_plan.hwp`(저장 제품 정보 없음), 전체 101쪽 = 한/글 2020 Print 101쪽.
//! 77쪽: Native 100.00 / fresh WASM 100.00. 78쪽: 99.89 / 99.69. 80쪽: 99.91 / 99.80. 78~88쪽 전부 99.0 이상(수정 전 20~59). 전쪽 최저: Native 1.54(35쪽) / fresh WASM 1.67(35쪽). 90% 미만 Native·WASM 각 6쪽(34·35·45·46·54·58)은 수정 전과 같은 값이며 이 그림 이월 범위 밖이다(34·45쪽은 저장 쪽 시작 문단의 글자처럼 취급한 그림이 앞 쪽 바닥 아래로 넘치는 기존 형상). 수정 전 102쪽, 90% 미만 30쪽.
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8795/{base,head}/{native,wasm}/arrhythmia_research_plan_silhouette.tsv`, `png/`.
//! 확대 판독: 77·78·80쪽 review·overlay 를 한/글 2020 Print 와 같은 배율로 대조 — 77쪽은 앵커 빈 줄만 남고 본문 줄 위치가 같다, 78쪽은 pi969 그림이 맨 위·pi972 그림이 그 아래로 PDF 와 위아래 경계가 겹친다, 80쪽 pi976 그림 아래끝이 본문 끝과 같다.
//! 같은 assertion 이 수정 전 FAIL(77쪽에 pi969 그림이 남음, 쪽수 102) / 수정 후 PASS. 전체 쪽 최저가 90 미만(아래)이라 쪽수는 고정하지 않는다.
//! 독립 기대값: 한/글 2020 Print `pdf/issue8795/1351000-201500141_arrhythmia_research_plan-2020.pdf`(101쪽). 77쪽에 그림 없음, 78~86쪽 그림 둘, 87쪽 그림 하나.
//! 저장 근거: 앵커 pi969 첫 줄 vpos 59600 + 그림 31878 > 본문 65763. pi970·971(61740·63880)은 앞 쪽에 이어지고, pi972 가 vpos 31878 = 그림 높이에서 되감긴다(pi972 는 자기 그림을 가진 빈 문단). pi976 그림 아래끝 32185 + 1700 + 31878 = 65763 = 본문 높이. pi975 다음 줄 32185 = 오프셋 307 + 그림 31878(빈 앵커의 아래 간격 1000 은 띠 안에 든다).
//! 독립 절대 px 좌표 대신 그림의 쪽 소속·쪽마다 그림 수·그림끼리의 위아래 관계를 검사한다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8795/1351000-201500141_arrhythmia_research_plan.hwp";

/// 본문 그림 (문단 번호, 상자). 표 안 그림은 셀 문맥을 가지므로 세지 않는다.
fn pictures(node: &RenderNode, out: &mut Vec<(usize, BoundingBox)>) {
    if let RenderNodeType::Image(image) = &node.node_type {
        if let (Some(0), Some(para)) = (image.section_index, image.para_index) {
            out.push((para, node.bbox));
        }
    }
    if matches!(node.node_type, RenderNodeType::Table(_)) {
        return;
    }
    for child in &node.children {
        pictures(child, out);
    }
}

fn page_pictures(core: &DocumentCore, page: u32) -> Vec<(usize, BoundingBox)> {
    let tree = core.build_page_render_tree(page).expect("쪽 배치");
    let mut out = Vec::new();
    pictures(&tree.root, &mut out);
    out
}

fn below(upper: &BoundingBox, lower: &BoundingBox) -> bool {
    lower.y + 0.5 >= upper.y + upper.height
}

#[test]
fn deferred_picture_opens_next_page_above_rewinding_paragraph_picture() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    // 77쪽(0 기준 76): 앵커 pi969 의 그림은 남지 않는다.
    let p77 = page_pictures(&core, 76);
    assert!(p77.is_empty(), "77쪽에 그림이 없다: {p77:?}");

    // 78쪽: pi969 그림이 맨 위, 되감기는 pi972 의 자기 그림이 그 아래.
    let p78 = page_pictures(&core, 77);
    let paras: Vec<usize> = p78.iter().map(|(para, _)| *para).collect();
    assert_eq!(paras, vec![969, 972], "78쪽 그림의 앵커 문단과 순서");
    assert!(
        below(&p78[0].1, &p78[1].1),
        "pi972 그림은 pi969 그림 아래: {p78:?}"
    );

    // 80쪽: 본문 끝에 정확히 닿는 pi976 그림이 pi975 그림 아래에 남는다.
    let p80 = page_pictures(&core, 79);
    let paras: Vec<usize> = p80.iter().map(|(para, _)| *para).collect();
    assert_eq!(paras, vec![975, 976], "80쪽 그림의 앵커 문단과 순서");
    assert!(
        below(&p80[0].1, &p80[1].1),
        "pi976 그림은 pi975 그림 아래: {p80:?}"
    );

    // 78~86쪽은 그림 둘, 87쪽은 하나(한/글 2020 Print 와 같다).
    for page in 77..86 {
        assert_eq!(
            page_pictures(&core, page).len(),
            2,
            "{}쪽 그림 수",
            page + 1
        );
    }
    assert_eq!(page_pictures(&core, 86).len(), 1, "87쪽 그림 수");
}
