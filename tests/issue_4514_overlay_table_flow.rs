//! 관계형·고정형 — #7808 이어받기 표의 내용 소속·중복·누락과 마지막 글줄 포함.
//! 주석 갱신 2026-11-28T10:42:02+09:00. 직전 파일 커밋 be48607ad9444d8232df4f699c73832d9a34885d 2026-11-28T07:25:36+09:00.
//! Native Skia·fresh WASM Print를 독립 한컴 PDF에 96 DPI·PDFium·2px raw 실루엣으로 대조했다.
//! 측정 코드 be48607ad9444d8232df4f699c73832d9a34885d.
//! sample1-repro · 기준 pdf/issue4514/sample1-repro-hwp-2020.pdf · 전체 46쪽.
//! 전쪽 최저 Native 98.26% / fresh WASM 98.35%.
//! 9쪽 Native 99.97% / fresh WASM 99.97%.
//! 10쪽 Native 99.94% / fresh WASM 99.94%.
//! 11쪽 Native 99.97% / fresh WASM 99.97%.
//! 42쪽 Native 100.00% / fresh WASM 100.00%.
//! 43쪽 Native 100.00% / fresh WASM 100.00%.
//! 전쪽 TSV와 대표 PNG는 mydocs/pr/assets/issues7789_7812/sample1-repro/에 있다.
//!
//! [#4514] overlay 표 필러 흐름·클램프 회귀 가드.
//!
//! sample1-repro 는 글앞으로(overlay) 비-TAC 4×5 표 37개가 빈 문단에 앵커되고 뒤따르는
//! 빈 문단들이 표 높이만큼 흐름 공간을 만드는 문서다. 수정 전에는 (a) #703 Shape
//! 단축(흐름 0) 앵커의 후행 빈 문단까지 #1955 흡수가 제거해 사다리 갭이 소멸하고,
//! (b) 렌더러의 Para-기준 body_bottom 클램프가 쪽 하단을 넘는 표를 위로 끌어올려
//! 6쪽에서 표끼리 최대 555.5px 겹쳤다(판독 불가). 가드 축:
//! 1. 전 페이지 최상위 표 y 구간 무겹침 (임계 2px — #4515 진단과 동일 규칙)
//! 2. ECR-001~005 구간 3쪽 구조 (한컴 실측 구조: N / N+1 / N+2)
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType, TableNode};
use rhwp::DocumentCore;

fn top_level_tables(root: &RenderNode) -> impl Iterator<Item = (&RenderNode, &TableNode)> {
    let body_tables = root
        .children
        .iter()
        .filter(|node| matches!(node.node_type, RenderNodeType::Body { .. }))
        .flat_map(|body| body.children.iter())
        .filter(|node| matches!(node.node_type, RenderNodeType::Column(_)))
        .flat_map(|column| column.children.iter());
    root.children.iter().chain(body_tables).filter_map(|node| {
        let RenderNodeType::Table(table) = &node.node_type else {
            return None;
        };
        Some((node, table))
    })
}

fn append_text_runs<'tree>(
    node: &'tree RenderNode,
    runs: &mut Vec<(&'tree RenderNode, &'tree str)>,
) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if !run.text.trim().is_empty() {
            runs.push((node, &run.text));
        }
    }
    for child in &node.children {
        append_text_runs(child, runs);
    }
}

#[test]
fn issue_4514_overlay_tables_do_not_overlap() {
    let bytes = std::fs::read("samples/issue4514/sample1-repro.hwp")
        .expect("fixture 를 읽을 수 있어야 한다");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture 파싱");
    let total = core.page_count();

    // 수정 전 47쪽(겹침 6쪽), 수정 후 48쪽 (한컴 46 — 잔여 +2 는 overlay 표 쪽 분할
    // 페인트 부재/host 줄 계상의 후속 과제로 남겨 뒀었다). #5918 이 꼬리 조각의
    // 이중 쪽 경계를 제거해 정본(한글 2020, 46쪽)과 수렴 — 예고대로 값을 좁혀
    // 갱신한다.
    assert_eq!(total, 46, "총 페이지 수가 예기치 않게 변했다");

    let mut requirement_table_pages: Vec<u32> = Vec::new();
    let mut ecr004_pages: Vec<u32> = Vec::new();
    let mut ecr004_neighbors: Vec<(u32, Vec<i64>, Vec<i64>)> = Vec::new();
    let mut fragment_text = Vec::new();
    for page in 0..total {
        let tree = core
            .build_page_render_tree(page)
            .expect("render tree 를 얻을 수 있어야 한다");
        let overlaps = core.take_table_overlaps();
        assert!(
            overlaps.is_empty(),
            "page {} 에서 최상위 표 겹침이 재발했다: {:?}",
            page,
            overlaps
                .iter()
                .map(|o| (o.para_a, o.para_b, o.overlap_px))
                .collect::<Vec<_>>()
        );

        let mut spans = Vec::new();
        for (table_node, table) in top_level_tables(&tree.root) {
            let owner = table.para_index.map_or(-1, |index| index as i64);
            let top = table_node.bbox.y;
            let bottom = top + table_node.bbox.height;
            spans.push((owner, top, bottom));

            if matches!(owner, 139 | 158 | 765) {
                let mut runs = Vec::new();
                append_text_runs(table_node, &mut runs);
                let text: String = runs
                    .iter()
                    .flat_map(|(_, text)| text.chars())
                    .filter(|c| !c.is_whitespace())
                    .collect();
                fragment_text.push((owner, page + 1, text));

                if owner == 765 {
                    for (run, text) in runs {
                        assert!(
                            top <= run.bbox.y && run.bbox.y + run.bbox.height <= bottom,
                            "{}쪽 보안 특약 글줄이 해당 표 조각 안에 있어야 한다: {text}",
                            page + 1,
                        );
                    }
                }
            }
        }
        spans.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        for w in spans.windows(2) {
            let overlap = w[0].2 - w[1].1;
            assert!(
                overlap <= 2.0,
                "page {} 표 pi={}·pi={} y 구간 겹침 {:.1}px (임계 2px)",
                page,
                w[0].0,
                w[1].0,
                overlap
            );
        }
        // ECR-001~005 host 문단(102·118·119·139·158)의 표가 있는 페이지 수집
        for (pi, _, _) in &spans {
            if matches!(pi, 102 | 118 | 119 | 139 | 158) && !requirement_table_pages.contains(&page)
            {
                requirement_table_pages.push(page);
            }
        }
        for (index, (pi, _, _)) in spans.iter().enumerate() {
            if *pi == 139 {
                ecr004_pages.push(page);
                ecr004_neighbors.push((
                    page,
                    spans[..index].iter().map(|(owner, _, _)| *owner).collect(),
                    spans[index + 1..]
                        .iter()
                        .map(|(owner, _, _)| *owner)
                        .collect(),
                ));
            }
        }
    }

    // #7808: 고정 base도 미달이었던 원본의 "ECR-001~005는 3쪽" 핀은
    // 독립 Print와 불일치한다. Print 11쪽에도 ECR-005 뒷 본문이 이어지므로
    // 이전 PASS는 이어받기 누락을 허용했다. 이 잘못된 쪽수 핀만 이관하고
    // 전쪽 무겹침·연속 소속·ECR-004 앞뒤 소유의 정확한 기존 관계는 유지한다.
    requirement_table_pages.sort_unstable();
    assert!(
        requirement_table_pages.windows(2).all(|w| w[1] == w[0] + 1),
        "ECR 구간의 모든 조각은 연속된 쪽에 속해야 한다 (실제: {:?})",
        requirement_table_pages
    );

    // [#4568] ECR-004(pi=139)는 앵커 쪽 하단에서 잘린 뒤 다음 쪽 상단에 남은 행이
    // 반드시 이어 그려져야 한다. 표가 3쪽 구간에 존재한다는 판정만으로는 앵커 쪽의
    // 컷만 남기고 continuation을 잃는 회귀를 잡지 못한다.
    assert_eq!(
        ecr004_pages.len(),
        2,
        "ECR-004는 앵커와 다음 쪽 연속 조각으로 정확히 두 번 보여야 한다: {:?}",
        ecr004_pages
    );
    assert_eq!(
        ecr004_pages[1],
        ecr004_pages[0] + 1,
        "ECR-004 연속 조각은 바로 다음 쪽에 있어야 한다: {:?}",
        ecr004_pages
    );
    // 독립 한컴 출력의 소유 순서: 앞 조각은 ECR-003 뒤에 있고, 다음 쪽의
    // 첫 표는 ECR-004의 이어받기이며 ECR-005보다 앞에 있다. 700/100px 같은
    // 화면 영역으로 하단·상단을 고정하지 않는다. 전쪽 무겹침 검사는 위에서 유지한다.
    assert!(
        ecr004_neighbors[0].1.contains(&119),
        "ECR-004 앞 조각은 같은 쪽의 ECR-003 뒤에 있어야 한다: {:?}",
        ecr004_neighbors
    );
    assert!(
        ecr004_neighbors[1].1.is_empty() && ecr004_neighbors[1].2.contains(&158),
        "ECR-004 이어받기는 다음 쪽의 첫 표이며 ECR-005보다 앞에 있어야 한다: {:?}",
        ecr004_neighbors
    );

    // 같은 원본의 독립 한컴 Print 9–11쪽에서 앞 행, 이어받기 끝, 다음
    // 표의 상세 행과 그 이어받기를 골라 페이지와 소유 표를 함께 확인한다.
    for (owner, physical_page, phrase) in [
        (139, 9, "ECR-004"),
        (139, 9, "도입수량:1대"),
        (139, 10, "분리발주예정인백업소프트웨어설치지원"),
        (158, 10, "ECR-005"),
        (158, 10, "스토리지모니터링을위한SW또는유사기능지원"),
        (158, 11, "스토리지컨트롤러기반이기종스토리지가상화기능제공"),
        (765, 42, "도출된취약점에대한개선을"),
        (765, 43, "완료하고그결과를제출해야한다."),
    ] {
        let pages: Vec<_> = fragment_text
            .iter()
            .filter(|(table, _, _)| *table == owner)
            .flat_map(|(_, page, text)| text.match_indices(phrase).map(|_| *page))
            .collect();
        assert_eq!(
            pages,
            [physical_page],
            "{phrase}는 원본 표 {owner}의 해당 쪽에서 한 번만 보여야 한다"
        );
    }
}
