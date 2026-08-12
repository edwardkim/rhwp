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

use rhwp::DocumentCore;

fn top_level_table_spans(root: &serde_json::Value) -> Vec<(i64, f64, f64)> {
    fn span_of(node: &serde_json::Value) -> Option<(i64, f64, f64)> {
        if node.get("type").and_then(|t| t.as_str()) != Some("Table") {
            return None;
        }
        let b = node.get("bbox")?;
        let y = b.get("y")?.as_f64()?;
        let h = b.get("h")?.as_f64()?;
        Some((
            node.get("pi").and_then(|p| p.as_i64()).unwrap_or(-1),
            y,
            y + h,
        ))
    }
    let empty = Vec::new();
    let children = |n: &serde_json::Value| -> Vec<serde_json::Value> {
        n.get("children")
            .and_then(|c| c.as_array())
            .unwrap_or(&empty)
            .clone()
    };
    let mut out = Vec::new();
    for child in children(root) {
        if let Some(s) = span_of(&child) {
            out.push(s);
        }
        if child.get("type").and_then(|t| t.as_str()) == Some("Body") {
            for col in children(&child) {
                if col.get("type").and_then(|t| t.as_str()) != Some("Column") {
                    continue;
                }
                for item in children(&col) {
                    if let Some(s) = span_of(&item) {
                        out.push(s);
                    }
                }
            }
        }
    }
    out
}

#[test]
fn issue_4514_overlay_tables_do_not_overlap() {
    let bytes = std::fs::read("samples/issue4514/sample1-repro.hwp")
        .expect("fixture 를 읽을 수 있어야 한다");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture 파싱");
    let total = core.page_count();

    // 수정 전 47쪽(겹침 6쪽), 수정 후 48쪽 (한컴 46 — 잔여 +2 는 overlay 표 쪽 분할
    // 페인트 부재/host 줄 계상의 후속 과제. 한컴 수렴 개선 시 이 값을 좁혀 갱신한다.)
    assert_eq!(total, 48, "총 페이지 수가 예기치 않게 변했다");

    let mut requirement_table_pages: Vec<u32> = Vec::new();
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

        let json: serde_json::Value =
            serde_json::from_str(&tree.root.to_json()).expect("render tree JSON");
        let root = json.get("root").unwrap_or(&json);
        let mut spans = top_level_table_spans(root);
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
    }

    // 한컴 구조: ECR-001~005 가 연속 3쪽 (N: 001+002시작 / N+1: 002계속+003+004시작 /
    // N+2: 004계속+005). 수정 전에는 2쪽으로 압축되며 8쪽에 4개가 겹쳐 있었다.
    requirement_table_pages.sort_unstable();
    assert_eq!(
        requirement_table_pages.len(),
        3,
        "ECR-001~005 구간은 한컴처럼 3쪽이어야 한다 (실제: {:?})",
        requirement_table_pages
    );
    assert!(
        requirement_table_pages.windows(2).all(|w| w[1] == w[0] + 1),
        "ECR 구간 3쪽은 연속이어야 한다 (실제: {:?})",
        requirement_table_pages
    );
}
