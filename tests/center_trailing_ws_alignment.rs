//! 셀 밖 가운데 정렬은 말미 공백을 정렬 폭에서 제외한다 — 30213 의결서 실측.
//!
//! 2쪽 위원 서명 줄 중 마지막(pi52)만 말미 공백 8칸을 품는데, 한글 PDF 는 이 줄도
//! 위 줄들과 같은 x(229.56pt)에서 시작한다. 공백을 포함해 중심을 잡으면 43px
//! 좌측으로 이탈한다. 판정은 같은 쪽 이웃 줄과의 상대 x 비교라 폰트 메트릭
//! 환경에 강건하다(절대좌표 단언 금지 — #3458 의 교훈).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::DocumentCore;

/// pi 문단의 TextRun 최소 x — TextLine bbox 는 단 전체 폭이라 정렬을 못 본다.
fn collect_min_x(node: &serde_json::Value, ty: &str, pi: i64, min_x: &mut f64) {
    if node.get("type").and_then(|t| t.as_str()) == Some(ty)
        && node.get("pi").and_then(|p| p.as_i64()) == Some(pi)
    {
        if let Some(x) = node
            .get("bbox")
            .and_then(|b| b.get("x"))
            .and_then(|v| v.as_f64())
        {
            *min_x = min_x.min(x);
        }
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[])
    {
        collect_min_x(child, ty, pi, min_x);
    }
}

/// pi50 "위  원   이 영 구"(말미 공백 없음)과 pi52 "위  원   유 재 풍        "
/// (말미 공백 8칸 + 글뒤 서명선 앵커)은 같은 Center 문단 모양이므로 시작 x 가
/// 같아야 한다. 말미 공백이 정렬 폭에 포함되면 pi52 만 ~43px 왼쪽으로 밀린다.
#[test]
fn center_alignment_excludes_trailing_spaces_outside_cells() {
    let bytes = std::fs::read("samples/issue4491/30213_1.혼합단지등 제도개선 방안.hwp")
        .expect("fixture 를 읽을 수 있어야 한다");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture 파싱");
    let page = core
        .build_page_render_tree(1)
        .expect("2쪽 render tree 를 얻을 수 있어야 한다");
    let tree: serde_json::Value =
        serde_json::from_str(&page.root.to_json()).expect("render tree JSON");
    let root = tree.get("root").unwrap_or(&tree);

    let mut x_prev = f64::INFINITY;
    collect_min_x(root, "TextRun", 50, &mut x_prev);
    let mut x_last = f64::INFINITY;
    collect_min_x(root, "TextRun", 52, &mut x_last);

    assert!(x_prev.is_finite(), "2쪽에 pi50 줄이 있어야 한다");
    assert!(x_last.is_finite(), "2쪽에 pi52 줄이 있어야 한다");
    assert!(
        (x_last - x_prev).abs() <= 2.0,
        "마지막 위원 줄(pi52, x={:.1})이 이웃 줄(pi50, x={:.1})과 어긋났다 — \
         가운데 정렬이 말미 공백을 폭에 포함했다 (결함 시 Δ≈-43px)",
        x_last,
        x_prev,
    );
}
