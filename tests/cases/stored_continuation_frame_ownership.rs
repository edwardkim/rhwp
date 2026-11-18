//! 관계형: 표 이어받기의 마지막 행·그림의 쪽 소속과 중복 방지. 고정형: 독립 Print의 쪽 소속 및 98% 이상 페이지의 개체 위끝(±2px).
//! 주석 갱신: 2026-11-11T15:32:36+09:00 (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 부모 016caef695 + 메인터너 보정 R7, PDFium·96 DPI·Print·색상 경계 보정 전 2px raw.
//! HPV 원본 samples/issue8842/1351000-201800239_hpv_cohort_final_report.hwp, 독립 한/글 2020 Print pdf/issue8842/1351000-201800239_hpv_cohort_final_report-2020.pdf(235쪽).
//! 집중 측정 Native/fresh WASM: 79쪽 99.83/99.83, 159쪽 99.76/99.76, 160~163쪽 각각 99.82/99.82·99.84/99.84·99.79/99.79·99.80/99.80%. 전쪽 완료 주장이 아니다.
//! 결재문서 samples/issue8742/36483048_approval_body.hwpx, 독립 한/글 2022 Print pdf/issue8742/36483048_approval_body-2022.pdf(6쪽), 5쪽 Native 98.12/fresh WASM 98.13%.
//! 독립 PDFium 기하: HPV 159쪽 그림 위끝 113.156px, 결재문서 5쪽 제목 표 윗선 중심 678.937px. 양쪽 98% 이상인 이 두 좌표만 ±2px로 고정한다.
//! 수정 전 통합 코드 e6261fd4: 표 잔여 행은 80쪽으로 밀리고 두 그림이 겹치며, 결재문서 표 위끝은 약 686.3px다. 원본의 쪽수·내용을 바꾸지 않는다.
//! 측정 TSV·PDF 기하: mydocs/pr/assets/planet6897_remaining39_20261111/maintainer-repair/의 focus/{native,wasm} 및 *-geometry.json.
//! 실물 Print 측정의 원본 글꼴 조건과 이 검사의 최소 글꼴 환경은 별도다. 설치 글꼴이나 절대 폰트 경로에 의존하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const HPV: &str = "samples/issue8842/1351000-201800239_hpv_cohort_final_report.hwp";
const APPROVAL: &str = "samples/issue8742/36483048_approval_body.hwpx";

fn load(path: &str) -> DocumentCore {
    DocumentCore::from_bytes(&std::fs::read(path).expect("검증 원본")).expect("원본 열기")
}

fn table(node: &RenderNode, section: usize, para: usize) -> Option<&RenderNode> {
    if matches!(&node.node_type, RenderNodeType::Table(t)
        if t.section_index == Some(section) && t.para_index == Some(para) && t.cell_context.is_none())
    {
        return Some(node);
    }
    node.children.iter().find_map(|n| table(n, section, para))
}

fn pictures(node: &RenderNode, out: &mut Vec<(usize, BoundingBox)>) {
    if let RenderNodeType::Image(image) = &node.node_type {
        if let (Some(2), Some(para)) = (image.section_index, image.para_index) {
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

#[test]
fn capacity_continuation_finishes_the_table_without_a_short_extra_page() {
    let core = load(HPV);
    let p79 = core.build_page_render_tree(78).expect("79쪽");
    let fragment = table(&p79.root, 2, 32).expect("79쪽 표 10의 이어받기");
    assert!(
        fragment.children.iter().any(|n| matches!(&n.node_type,
        RenderNodeType::TableCell(c) if c.row == 31)),
        "마지막 HPV 백신 접종여부 행까지 같은 조각이 소유한다"
    );
    let p80 = core.build_page_render_tree(79).expect("80쪽");
    assert!(
        table(&p80.root, 2, 32).is_none(),
        "이미 끝낸 표를 다음 쪽에서 다시 잇지 않는다"
    );
}

#[test]
fn consecutive_deferred_pictures_keep_their_own_restart_pages() {
    let core = load(HPV);
    let owners = [
        (678, 158),
        (680, 159),
        (686, 160),
        (688, 161),
        (690, 162),
        (694, 163),
    ];
    let mut observed = Vec::new();
    for page in 0..core.page_count() {
        let tree = core.build_page_render_tree(page).expect("그림 소속 검사");
        let mut images = Vec::new();
        pictures(&tree.root, &mut images);
        for (para, bbox) in images {
            if owners.iter().any(|(owner, _)| *owner == para) {
                observed.push((para, page + 1));
                if para == 680 {
                    assert!(
                        (bbox.y - 113.156).abs() <= 2.0,
                        "독립 Print 159쪽 그림 위끝(±2px): {bbox:?}"
                    );
                }
            }
        }
    }
    assert_eq!(
        observed, owners,
        "각 그림은 자기 쪽에 한 번만 있고 순서를 유지한다"
    );
}

#[test]
fn accepted_stored_inline_table_origin_is_not_pushed_by_an_empty_guide() {
    let core = load(APPROVAL);
    let p5 = core.build_page_render_tree(4).expect("결재문서 5쪽");
    let heading = table(&p5.root, 0, 69).expect("4 추진일정 제목 표");
    assert!(
        (heading.bbox.y - 678.937).abs() <= 2.0,
        "독립 Print 윗선 중심(±2px), 빈 안내 줄의 간격을 두 번 더하지 않는다: {:?}",
        heading.bbox
    );
}
