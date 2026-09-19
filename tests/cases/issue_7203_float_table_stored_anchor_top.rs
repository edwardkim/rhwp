//! [Issue #7203] 자리차지 표의 윗변이 경로마다 다른 원점을 쓴다.
//!
//! ## 갈려 있던 두 산식
//!
//! 같은 이름의 게이트가 `layout.rs`(렌더)와 `typeset.rs`(조판)에 각각 있었고 값이 달랐다.
//!
//! ```text
//!   layout.rs   need = 높이 + 아래여백 − 위여백 ,  윗변 = vpos − 위여백
//!   typeset.rs  need = 높이 + 위여백 + 아래여백 ,  윗변 = vpos
//! ```
//!
//! 대칭 여백(283/283)이면 `need` 가 `2 × 위여백` = 566 HU 갈린다. `hwpctl_API_v2.4.hwp` 의
//! 어긋난 자리차지 표는 **예외 없이** 사다리 간격이 `높이 + 66 HU` 였다 — 렌더는 저장
//! 앵커를 수용하고 조판은 500 HU 모자라 거부하는 구간이다. 그래서 흐름이 표를 host 줄
//! 높이(1000 HU = 13.33px)만큼 위에 놓아 `−13.33 + 3.77 = −9.56px` 가 남았다.
//!
//! ## 기대값의 출처 — 구현과 독립
//!
//! 저장소 안 정본 `pdf/hwpctl_API_v2.4-hwp-2020.pdf` 의 가로 괘선과 원본 저장 사다리를
//! 대조하면 한 규칙이 나온다(9건 전부 잔차 +0.26 .. +0.74px).
//!
//! ```text
//!   정본 윗변 = 본문 상단 + (앵커 vpos − 위 바깥여백)
//! ```
//!
//! 이 검사는 그 규칙을 **문서 자신의 저장값**(`rhwp dump` 의 `ls[0] vpos` 와
//! `[outer_margin] top`)으로 세우고 실제 배치와 대조한다. rhwp 가 계산한 좌표를 다시
//! 인용하지 않는다.
//!
//! ## 수정
//!
//! `renderer::stored_float_anchor` 가 판정·원점의 정본이고 조판·렌더가 같은 값을 쓴다.
//! 종전의 "본문 하단 절반의 앵커만" 위치 게이트는 걷었다 — 정본이 상단 절반에서도 같은
//! 규칙을 쓰기 때문이다. 대신 **산식이 성립하는 조건**으로 좁혔다: 문단 기준
//! `vertical_offset` 이 걸린 개체는 윗변이 `앵커 + 오프셋` 이라 흐름 배치에 맡긴다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::wasm_api::HwpDocument;

const SAMPLE: &str = "samples/hwpctl_API_v2.4.hwp";
/// `vertical_offset` 이 걸린 자리차지 표가 있는 문서 — 저장 앵커를 쓰면 안 되는 갈래.
const OFFSET_SAMPLE: &str = "samples/rowbreak-problem-pages.hwp";

const HWPUNIT_PER_PX: f64 = 7200.0 / 96.0;

/// `rhwp dump samples/hwpctl_API_v2.4.hwp` 의 실측값.
///
/// `(0-based 쪽, 문단, 앵커 ls[0] vpos, 위 바깥여백, 정본 PDF 윗변)`
/// 정본 값은 `pdf/hwpctl_API_v2.4-hwp-2020.pdf` 의 가로 괘선(폭 427.7px)이다.
const STORED_ANCHOR_CASES: &[(u32, usize, i64, i64, f64)] = &[
    (11, 156, 20193, 283, 398.28),
    (24, 473, 9460, 283, 255.24),
    (24, 483, 26453, 283, 481.55),
    (31, 694, 12980, 283, 302.23),
    (40, 951, 30893, 283, 540.69),
    (41, 970, 17675, 283, 364.72),
    (48, 1172, 5960, 283, 208.73),
];

fn page_root(sample: &str, page: u32) -> RenderNode {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("fixture 를 읽을 수 없다 ({}): {error}", path.display()));
    let document = HwpDocument::from_bytes(&bytes).expect("문서 로드");
    document
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("쪽 idx {page} render tree: {error:?}"))
        .root
}

fn find<'a>(
    node: &'a RenderNode,
    want: &mut impl FnMut(&RenderNode) -> bool,
) -> Vec<&'a RenderNode> {
    let mut out = Vec::new();
    fn walk<'a>(
        node: &'a RenderNode,
        want: &mut impl FnMut(&RenderNode) -> bool,
        out: &mut Vec<&'a RenderNode>,
    ) {
        if want(node) {
            out.push(node);
        }
        for child in &node.children {
            walk(child, want, out);
        }
    }
    walk(node, want, &mut out);
    out
}

fn body_top(root: &RenderNode) -> f64 {
    find(root, &mut |node| {
        matches!(node.node_type, RenderNodeType::Body { .. })
    })
    .first()
    .map(|node| node.bbox.y)
    .expect("Body 노드")
}

fn table_top(root: &RenderNode, para_index: usize) -> f64 {
    let tables = find(root, &mut |node| match &node.node_type {
        RenderNodeType::Table(table) => table.para_index == Some(para_index),
        _ => false,
    });
    assert!(!tables.is_empty(), "문단 {para_index} 의 표가 이 쪽에 없다");
    // 한 문단이 여러 조각을 낼 수 있다 — 가장 위 조각이 윗변이다.
    tables
        .iter()
        .map(|node| node.bbox.y)
        .fold(f64::INFINITY, f64::min)
}

/// 저장 앵커를 쓰는 자리차지 표의 윗변은 `본문 상단 + (앵커 vpos − 위 바깥여백)` 이다.
///
/// 수정 전에는 조판이 이 앵커를 거부해 표가 host 줄 높이만큼(9.5~10.3px) 위에 놓였고,
/// 그 자리가 앞 문단 글자를 지났다.
#[test]
fn a_float_table_top_sits_on_its_stored_anchor() {
    for &(page, para_index, vpos, outer_margin_top, oracle_top) in STORED_ANCHOR_CASES {
        let root = page_root(SAMPLE, page);
        let expected = body_top(&root) + (vpos - outer_margin_top) as f64 / HWPUNIT_PER_PX;
        let actual = table_top(&root, para_index);

        // 저장값으로 세운 기대값과 맞는가.
        assert!(
            (actual - expected).abs() <= 1.0,
            "쪽 {page} 문단 {para_index}: 표 윗변 {actual:.2} 가 저장 앵커 기대값 \
             {expected:.2}(= 본문 상단 + ({vpos} − {outer_margin_top})HU)에서 벗어났다"
        );
        // 그 기대값이 곧 한/글 정본이다.
        assert!(
            (actual - oracle_top).abs() <= 1.5,
            "쪽 {page} 문단 {para_index}: 표 윗변 {actual:.2} 가 정본 {oracle_top:.2} 에서 \
             1.5px 넘게 벗어났다"
        );
    }
}

/// 어울림(TAC) 표는 이 수정의 비적용 경로다 — 같은 문서의 대조군.
///
/// 이 표들은 수정 전에도 정본과 맞았다(54건 중 38건이 1.5px 이내, 중앙값 +0.33px).
/// 자리차지 원점을 고치면서 이쪽이 흔들리면 수정 범위를 넘은 것이다.
#[test]
fn treat_as_char_tables_are_untouched() {
    // `(쪽, 문단, 정본 PDF 윗변)` — 같은 방식으로 뽑은 어울림 표.
    const TAC_CASES: &[(u32, usize, f64)] = &[
        (12, 176, 136.01),
        (21, 412, 136.01),
        (23, 440, 136.01),
        (35, 797, 136.01),
    ];
    for &(page, para_index, oracle_top) in TAC_CASES {
        let root = page_root(SAMPLE, page);
        let actual = table_top(&root, para_index);
        assert!(
            (actual - oracle_top).abs() <= 1.5,
            "쪽 {page} 문단 {para_index}: 어울림 표 윗변 {actual:.2} 가 정본 \
             {oracle_top:.2} 에서 벗어났다 — 자리차지 수정이 범위를 넘었다"
        );
    }
}

/// 문단 기준 `vertical_offset` 이 걸린 자리차지 표는 저장 앵커를 쓰지 않는다.
///
/// 윗변이 `앵커 + 오프셋` 이라 `vpos − 위여백` 산식이 성립하지 않는다.
/// `rowbreak-problem-pages.hwp` 구역1 의 `vertical_offset` 152·3019 HU 표가 이 갈래다 —
/// 앵커를 그대로 믿으면 12쪽 본문이 쪽 밖 480px 까지 밀리고 글자 11쌍이 겹친다.
#[test]
fn an_offset_anchored_float_table_keeps_flow_placement() {
    const PAGE: u32 = 11;
    let root = page_root(OFFSET_SAMPLE, PAGE);
    // 쪽 번호·꼬리말은 Body 밖에 산다 — 본문 글줄만 센다.
    let body = find(&root, &mut |node| {
        matches!(node.node_type, RenderNodeType::Body { .. })
    })
    .first()
    .copied()
    .expect("Body 노드");
    let body_bottom = body.bbox.y + body.bbox.height;

    let escaped: Vec<f64> = find(body, &mut |node| {
        matches!(node.node_type, RenderNodeType::TextLine(_))
    })
    .iter()
    .map(|node| node.bbox.y + node.bbox.height)
    .filter(|bottom| *bottom > body_bottom + 2.0)
    .collect();

    assert!(
        escaped.is_empty(),
        "본문 바닥 {body_bottom:.1} 아래로 빠져나간 본문 글줄 {}개(최대 {:.1}) — 오프셋이 \
         걸린 자리차지 표까지 저장 앵커로 배치하면 이 쪽이 무너진다",
        escaped.len(),
        escaped.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
    );
}
