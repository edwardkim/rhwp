//! 앵커 줄 계상 두 축의 회귀 가드 (#4490, #4491).
//!
//! 판정자는 저장 사다리다 — 두 단언 모두 폰트 메트릭이 아니라 저장 lineseg 가 정하는
//! 흐름 관계라 환경에 강건하다(절대좌표 단언 금지 — #3458 의 교훈).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::DocumentCore;

#[derive(Debug, Clone, Copy)]
struct Box2 {
    y: f64,
    h: f64,
}

fn collect(node: &serde_json::Value, ty: &str, pi: i64, out: &mut Vec<Box2>) {
    if node.get("type").and_then(|t| t.as_str()) == Some(ty)
        && node.get("pi").and_then(|p| p.as_i64()) == Some(pi)
    {
        if let Some(b) = node.get("bbox") {
            if let (Some(y), Some(h)) = (
                b.get("y").and_then(|v| v.as_f64()),
                b.get("h").and_then(|v| v.as_f64()),
            ) {
                out.push(Box2 { y, h });
            }
        }
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .map(|v| v.as_slice())
        .unwrap_or(&[])
    {
        collect(child, ty, pi, out);
    }
}

fn page_tree(sample: &str, page_idx: u32) -> serde_json::Value {
    let bytes = std::fs::read(sample).expect("fixture 를 읽을 수 있어야 한다");
    let core = DocumentCore::from_bytes(&bytes).expect("fixture 파싱");
    let page = core
        .build_page_render_tree(page_idx)
        .expect("render tree 를 얻을 수 있어야 한다");
    serde_json::from_str(&page.root.to_json()).expect("render tree JSON")
}

/// #4490 — 글앞으로+글자처럼 표의 앵커 줄(th=7348)을 한글은 흐름에 계상한다
/// (저장 사다리: 표 43940 + 7348 + 400 = 다음 문단 51688). 계상이 빠지면 마지막
/// 문단(pi17)이 일정표(pi15) 위로 91px 겹친다 — 겹침 부재가 곧 회귀 가드다.
#[test]
fn issue_4490_last_paragraph_stays_below_infront_tac_table() {
    let tree = page_tree(
        "samples/issue4490/148720174_111014(인력기획과)민간경력자_5급_일괄채용_필기_합격자_발표.hwp",
        1,
    );
    let root = tree.get("root").unwrap_or(&tree);

    let mut tables = Vec::new();
    collect(root, "Table", 15, &mut tables);
    let mut lines = Vec::new();
    collect(root, "TextLine", 17, &mut lines);

    assert_eq!(tables.len(), 1, "2쪽에 일정표(pi15)가 있어야 한다");
    assert!(
        lines.len() >= 4,
        "pi17 은 여러 줄이어야 한다: {}",
        lines.len()
    );

    let table_bottom = tables[0].y + tables[0].h;
    let first_line = lines.iter().map(|l| l.y).fold(f64::INFINITY, f64::min);
    assert!(
        first_line >= table_bottom - 1.0,
        "pi17 첫 줄(y={:.1})이 일정표 하단(y={:.1}) 위로 올라왔다 — \
         글앞+tac 표의 앵커 줄이 흐름에서 빠졌다 (#4490)",
        first_line,
        table_bottom,
    );
}

/// #4491 — 글앞 Shape 전용 빈 앵커 문단도 이 문서에서는 줄 전체를 예약한다
/// (저장 사다리: pi144 18080 → pi147 25600, Δ=7520 = 100.3px). 예약이 빠지면
/// 앵커 두 개를 지나며 "협의" 라벨이 21.3px 씩 당겨진다(결함 시 Δ=79.0px).
#[test]
fn issue_4491_floating_anchor_paragraphs_reserve_their_stored_lines() {
    let tree = page_tree("samples/issue4491/30213_1.혼합단지등 제도개선 방안.hwp", 8);
    let root = tree.get("root").unwrap_or(&tree);

    let mut imdae = Vec::new(); // pi144 "  ❍ 임대주택"
    collect(root, "TextLine", 144, &mut imdae);
    let mut hyubui = Vec::new(); // pi147 "         협의"
    collect(root, "TextLine", 147, &mut hyubui);

    assert_eq!(imdae.len(), 1, "9쪽 pi144 줄 하나");
    assert_eq!(hyubui.len(), 1, "9쪽 pi147 줄 하나");

    let delta = hyubui[0].y - imdae[0].y;
    let expected = (25600.0 - 18080.0) / 75.0; // 저장 사다리 Δ = 100.3px
    assert!(
        (delta - expected).abs() <= 3.0,
        "임대주택→협의 간격 {:.1}px 이 사다리 {:.1}px 에서 벗어났다 — \
         글앞 Shape 앵커 문단의 줄 예약이 빠졌다 (#4491)",
        delta,
        expected,
    );
}
