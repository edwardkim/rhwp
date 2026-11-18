//! 관계형·고정형(쪽수): 쪽·종이 기준 자리차지 그림의 밴드가 표 문단의 시작과 자기 앵커 문단의 첫 줄을 밀어, 표와 뒤 본문이 그림 아래에 오고 부록의 쪽을 거의 채우는 그림이 연속한 쪽에 하나씩 놓이며 전체 쪽수가 독립 PDF 와 같은지 검사한다.
//! 주석 갱신: 2026-11-08 23:47 KST (Asia/Seoul). 새 파일이며 직전 파일 커밋은 없다.
//! 실제 측정: 2026-11-08, `scripts/visual_sweep.py --silhouette-only` 4열 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 실루엣), 96 DPI, 기준 한/글 2020 Print.
//! 원본 `samples/issue8759/6500000-201700118_jeju_tourism_education_plan.hwp`(한/글 2010 저장), 전체 313쪽 = 한/글 2020 Print 313쪽.
//! 55쪽(그림 아래 표): Native 99.98 / fresh WASM 99.99. 271쪽(부록 회전 표): 99.98 / 99.98. 272쪽: 99.99 / 99.99. 전쪽 최저: Native 45.19(49쪽) / fresh WASM 45.57(49쪽). 90% 미만 Native 10쪽·WASM 12쪽은 모두 수정 전에도 미달이다(29·47~51쪽은 수정 전과 같거나 0.2 이내, 56·63·67·103·110·138쪽은 수정 전 31~60 → 80~91). 수정 전 55쪽 29.59 / 29.13, 271쪽 48.04 / 48.26, 쪽수 311, 90% 미만 264쪽.
//! 정식 TSV·review·overlay: `mydocs/pr/assets/issue_8759/{base,head}/{native,wasm}/jeju_tourism_education_plan_silhouette.tsv`, `png/`.
//! 확대 판독: 55쪽 표가 「그래프 3-8」 그림 아래, 뒤 본문이 한/글과 같은 높이(overlay 일치). 수정 전에는 표·본문이 그림 위에 겹쳤다.
//! 같은 assertion 이 수정 전 FAIL(쪽수 311) / 수정 후 PASS.
//! 독립 기대값: 한/글 2020 Print `pdf/issue8759/6500000-201700118_jeju_tourism_education_plan-2020.pdf`(313쪽).
//! 저장 근거: 55쪽 표 host pi720 첫 줄 vpos 31482 = 그림 pi719 밴드 아래끝(종이 오프셋 14808 + 높이 26594 − 본문 위 9920). 부록 앵커 pi3244 첫 줄 vpos 57964 = 10684 + 57200 − 9920, pi3247 58100 = 10620 + 57400 − 9920(각자 쪽 맨 위에서 vpos 가 되감긴다).
//! 독립 절대 px 좌표 대신 그림·표·앵커 줄의 위아래 관계와 쪽 소속을 검사한다. 설치 글꼴을 가정하지 않는다.

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{BoundingBox, RenderNode, RenderNodeType};

const SOURCE: &str = "samples/issue8759/6500000-201700118_jeju_tourism_education_plan.hwp";
const PDF_PAGES: u32 = 313;

#[derive(Default)]
struct Found {
    images: Vec<(usize, BoundingBox)>,
    tables: Vec<(usize, BoundingBox)>,
    lines: Vec<(usize, BoundingBox)>,
}

fn collect(node: &RenderNode, out: &mut Found) {
    match &node.node_type {
        RenderNodeType::Image(image) if image.section_index == Some(0) => {
            if let Some(para) = image.para_index {
                out.images.push((para, node.bbox));
            }
        }
        RenderNodeType::Table(table)
            if table.section_index == Some(0) && table.cell_context.is_none() =>
        {
            if let Some(para) = table.para_index {
                out.tables.push((para, node.bbox));
            }
        }
        RenderNodeType::TextLine(line) if line.section_index == Some(0) => {
            if let Some(para) = line.para_index {
                out.lines.push((para, node.bbox));
            }
        }
        _ => {}
    }
    for child in &node.children {
        collect(child, out);
    }
}

fn page(core: &DocumentCore, index: u32) -> Found {
    let tree = core.build_page_render_tree(index).expect("쪽 배치");
    let mut found = Found::default();
    collect(&tree.root, &mut found);
    found
}

fn image_of(found: &Found, para: usize) -> Option<BoundingBox> {
    found
        .images
        .iter()
        .find(|(p, _)| *p == para)
        .map(|(_, b)| *b)
}

/// 그림을 품은 쪽(0 기준). 쪽마다 그림 하나를 기대하므로 처음 만난 쪽을 돌려준다.
fn page_with_image(core: &DocumentCore, para: usize) -> Option<u32> {
    (0..core.page_count()).find(|&index| image_of(&page(core, index), para).is_some())
}

#[test]
fn page_band_pushes_table_paragraph_and_own_anchor_line() {
    let core = DocumentCore::from_bytes(&std::fs::read(SOURCE).expect("기준 입력"))
        .expect("기준 문서 열기");
    assert_eq!(core.page_count(), PDF_PAGES, "독립 Print 와 같은 쪽수");

    // 55쪽: 종이 기준 자리차지 그림(pi719) 아래에서 표 문단(pi720)이 시작한다.
    let p55 = page(&core, 54);
    let picture = image_of(&p55, 719).expect("55쪽 그래프 3-8 그림");
    let (_, table) = *p55.tables.iter().find(|(p, _)| *p == 720).expect("55쪽 표");
    assert!(
        table.y >= picture.y + picture.height - 0.5,
        "표는 그림 아래에서 시작한다: table={table:?} picture={picture:?}"
    );
    for (para, line) in p55.lines.iter().filter(|(p, _)| (721..=723).contains(p)) {
        assert!(
            line.y >= table.y + table.height - 0.5,
            "표 뒤 본문(pi{para})은 표 아래: line={line:?} table={table:?}"
        );
    }

    // 부록: 쪽을 거의 채우는 종이 기준 그림은 자기 앵커 줄을 자기 아래로 밀어(저장 사다리) 다음
    // 그림의 앵커가 다음 쪽으로 간다 — 그림이 연속한 쪽에 하나씩 놓인다. 빈 앵커 문단은 글줄
    // 노드가 없어 그림의 쪽 소속으로 검사한다.
    let appendix = [3244usize, 3247, 3250];
    let pages: Vec<u32> = appendix
        .iter()
        .map(|&para| page_with_image(&core, para).expect("부록 그림"))
        .collect();
    assert!(
        pages.windows(2).all(|w| w[1] == w[0] + 1),
        "부록 그림은 연속한 쪽에 하나씩: {pages:?}"
    );
    for (&para, &index) in appendix.iter().zip(&pages) {
        let found = page(&core, index);
        let others: Vec<usize> = found
            .images
            .iter()
            .map(|(p, _)| *p)
            .filter(|p| *p != para && appendix.contains(p))
            .collect();
        assert!(
            others.is_empty(),
            "{}쪽에 다른 부록 그림 {others:?}",
            index + 1
        );
    }
}
