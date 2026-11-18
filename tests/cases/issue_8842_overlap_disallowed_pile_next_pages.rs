//! 관계형: 겹침 허용 안 함 어울림 그림 무리(한 문단 그림 24장)가 앵커 쪽부터 쪽마다 한 장씩 차례로 놓이고 각 그림이 자기 쪽 안에 있는지 검사한다.
//! 주석 갱신: 2026-11-09 18:36 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-09, `scripts/visual_sweep.py --silhouette-only --dpi 96` silhouette_raw_match_percent(색상 경계 보정 전 2px 내용 실루엣), Native(native-skia)·fresh WASM 각 235쪽 전쪽, 수정 전 = upstream/devel 024d610633.
//! 원본 `samples/issue8842/1351000-201800239_hpv_cohort_final_report.hwp`(저장 제품 한/글 2010), 전체 235쪽 = 한/글 2020 Print 235쪽.
//! 211~234쪽: Native 최저 95.16(234쪽) / fresh WASM 최저 95.12(234쪽), 그 밖 23쪽은 Native·WASM 모두 98.19 이상(수정 전 0.17~5.69 / 0.24~5.96). 211쪽 99.19/99.31, 212쪽 99.91/99.90. 전쪽 최저: Native 0.35(123쪽) / fresh WASM 0.41(123쪽). 90% 미만 Native 102쪽·WASM 100쪽은 모두 수정 전과 같은 값이며 이 범위 밖이다(76~172쪽 표 쪽 나눔 어긋남 등, 수정 전 126·124쪽).
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8842/{base,head}/{native,wasm}/hpv_cohort_final_report_silhouette.tsv`, `png/`.
//! 확대 판독: 211·212·234쪽 review·overlay 를 한/글 2020 Print 와 같은 배율로 대조 — 211쪽은 표지·목차 슬라이드, 212쪽은 연구 참여·탈락 기준 슬라이드, 234쪽은 「감사합니다」 슬라이드가 PDF 와 같은 자리에 한 장씩 있고 테두리·글자 위치가 겹친다(overlay 주황은 글리프 경계).
//! 같은 assertion 이 수정 전 FAIL(211쪽에 24장이 겹치고 212~234쪽에 그림 없음) / 수정 후 PASS.
//! 독립 기대값: 한/글 2020 Print `pdf/issue8842/1351000-201800239_hpv_cohort_final_report-2020.pdf`(engine 2020, 11.0.0.9136) 211~234쪽 — 슬라이드 스캔 그림이 쪽마다 한 장.
//! 저장 근거: 구역 2 pi1007 은 글 없는 한 줄 문단(vpos 5040, cs 42520 · sw 3480)에 어울림·겹침 불허·문단 기준 오프셋 0 그림 24장(42520×60110HU). 뒤 빈 문단 저장 줄도 212~234쪽마다 vpos 0 에서 다시 시작하며 그림 옆(cs 42520)에 있다.
//! 반례: 뒤 흐름이 쪽을 만들지 않는 무리(`renderer::typeset` 단위 시험 #4770, 그림 3장 문단 하나)는 종전대로 앵커 쪽 1쪽에 남는다.
//! 독립 절대 px 좌표 대신 그림의 쪽 소속·순서와 쪽 안 포함을 검사한다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8842/1351000-201800239_hpv_cohort_final_report.hwp";
const ANCHOR_PAGE: u32 = 210; // 211쪽(0 기준)
const PILE: usize = 24;

fn pile_pictures(node: &RenderNode, out: &mut Vec<(usize, BoundingBox)>) {
    if let RenderNodeType::Image(image) = &node.node_type {
        if let (Some(2), Some(1007), Some(control)) =
            (image.section_index, image.para_index, image.control_index)
        {
            out.push((control, node.bbox));
        }
    }
    for child in &node.children {
        pile_pictures(child, out);
    }
}

#[test]
fn overlap_disallowed_pile_places_one_picture_per_following_page() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    assert_eq!(core.page_count(), 235, "한/글 2020 Print 와 같은 전체 쪽수");
    for k in 0..PILE {
        let page = ANCHOR_PAGE + k as u32;
        let tree = core.build_page_render_tree(page).expect("쪽 배치");
        let mut pictures = Vec::new();
        pile_pictures(&tree.root, &mut pictures);
        let controls: Vec<usize> = pictures.iter().map(|(c, _)| *c).collect();
        assert_eq!(
            controls,
            vec![k],
            "{}쪽에는 pi1007 그림 {k} 한 장만 있다",
            page + 1
        );
        let (_, bbox) = pictures[0];
        let page_height = tree.root.bbox.height;
        assert!(
            bbox.y >= 0.0 && bbox.y + bbox.height <= page_height,
            "{}쪽 그림은 쪽 안에 있다: {bbox:?}",
            page + 1
        );
    }
}
