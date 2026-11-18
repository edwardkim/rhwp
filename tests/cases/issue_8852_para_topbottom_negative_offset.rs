//! 관계형: 쪽 영역 안으로 제한인 문단 기준 자리차지 그림의 음수 세로 오프셋이 그림을 앞 본문·쪽 위로 올리지 않고, 그림이 앞 문단 글 아래·쪽 안에 놓이는지 검사한다.
//! 주석 갱신: 2026-11-09 22:53 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96` silhouette_raw_match_percent(색상 경계 보정 전 2px 내용 실루엣), Native(native-skia)·fresh WASM 각 235쪽 전쪽, 수정 전 = upstream/devel b768bee1c4(그래서 211~234쪽은 #8842 수정 전 값), 수정 후 = #8842 위 이 수정.
//! 원본 `samples/issue8842/1351000-201800239_hpv_cohort_final_report.hwp`(저장 제품 한/글 2010), 전체 235쪽 = 한/글 2020 Print 235쪽.
//! 56쪽: Native 99.99 / fresh WASM 100.00(수정 전 5.50 / 5.57). 전쪽 최저: Native 0.35(123쪽) / fresh WASM 0.41(123쪽). 90% 미만 Native 101쪽·WASM 99쪽은 56쪽과 211~234쪽(#8842) 밖이며 수정 전과 같은 값이다(76~172쪽 표 쪽 나눔 어긋남 등).
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8852/{base,head}/{native,wasm}/hpv_cohort_final_report_silhouette.tsv`, `png/`.
//! 확대 판독: 56쪽 review·overlay 를 한/글 2020 Print 와 같은 배율로 대조 — 「● 혈액의 시료제작 과정」 아래 「시료 제작의 업무 흐름도 – Blood」 그림이 PDF 와 같은 자리에서 시작하고 상자·화살표·글자가 겹친다(overlay 회색 일치).
//! 같은 assertion 이 수정 전 FAIL(56쪽 pi785 그림이 쪽 위 밖에서 시작해 앞 글 「● 혈액의 시료제작 과정」을 덮음) / 수정 후 PASS.
//! 독립 기대값: 한/글 2020 Print `pdf/issue8842/1351000-201800239_hpv_cohort_final_report-2020.pdf` 56쪽 — 그림 위끝 226.5px = 앵커 줄(vpos 9500), 앞 글은 그 위.
//! 저장 근거: 구역 0 pi785 그림 오프셋 −27507(쪽 영역 안으로 제한·자리차지·문단 기준·위쪽 정렬), 앵커 줄 9500, 다음 문단 줄 45769 = 9500 + 그림 36269. 오프셋을 −12000·−9000 으로 바꾼 한/글 2020 Print 탐침도 같은 앵커 줄에 둔다.
//! 독립 절대 px 좌표 대신 그림·앞 글의 쪽 소속과 위아래 순서를 검사한다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8842/1351000-201800239_hpv_cohort_final_report.hwp";
const PAGE: u32 = 55; // 56쪽(0 기준)

#[derive(Default)]
struct Found {
    picture: Option<BoundingBox>,
    heading_bottom: Option<f64>,
}

fn collect(node: &RenderNode, found: &mut Found) {
    match &node.node_type {
        RenderNodeType::Image(image)
            if image.section_index == Some(0) && image.para_index == Some(785) =>
        {
            found.picture = Some(node.bbox);
        }
        RenderNodeType::TextRun(run)
            if run.section_index == Some(0)
                && run.para_index == Some(783)
                && !run.text.trim().is_empty() =>
        {
            let bottom = node.bbox.y + node.bbox.height;
            let value = found.heading_bottom.get_or_insert(bottom);
            *value = value.max(bottom);
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, found);
    }
}

#[test]
fn restricted_para_topbottom_picture_ignores_negative_offset() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    let tree = core.build_page_render_tree(PAGE).expect("56쪽 배치");
    let mut found = Found::default();
    collect(&tree.root, &mut found);
    let picture = found.picture.expect("56쪽 pi785 그림");
    let heading_bottom = found.heading_bottom.expect("56쪽 pi783 글");
    assert!(
        picture.y + 0.5 >= heading_bottom,
        "pi785 그림({:.1})은 앞 글 아래({heading_bottom:.1})에서 시작한다",
        picture.y
    );
    assert!(
        picture.y >= 0.0 && picture.y + picture.height <= tree.root.bbox.height,
        "pi785 그림은 쪽 안에 있다: {picture:?}"
    );
}
