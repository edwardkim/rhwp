//! 관계형·고정형(쪽수): 맨 앞 글자처럼 취급 도형이 자기 줄을 소유하면 TAC 표 문단 높이 상한이 그 줄을 센다.
//! 주석 갱신: 2026-11-03 21:50 KST (새 파일)
//!
//! 측정 근거(측정일 2026-11-03, `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값
//! `silhouette_raw_match_percent` — 색상 경계 보정 전 2px 내용 실루엣, 이진화는 스크립트 기본값, 색상 경계
//! 대조 픽셀 0). 후보 source 는 `task/8543-leading-tac-shape-line-cap`(#8531 head `2f14656a35` 위,
//! `upstream/devel` `7d06d5c706` 대비 측정 src diff sha256 `4d82a00ba4040d22` — 커밋본 `f1f8330951e3707e` 와는
//! 코드 주석의 이슈 번호만 다르다), Native = native-skia release
//! (sha256 `524a2ed2f9eab54f`), fresh WASM = 같은 source 의 `scripts/wasm-pack-locked.sh --target web`
//! (`rhwp_bg.wasm` sha256 `da39e0252d2d4ce9`). 글꼴 `RHWP_FONT_PATH` = 한컴 `ttfs/hwp`·`ttfs/windows`.
//! `leading_shape_line-2020.hwp` → `pdf/issue8543/leading_shape_line-2020.pdf`(2쪽 = rhwp 2쪽):
//! Native 1쪽 100.00 / 2쪽 93.14, fresh WASM 1쪽 100.00 / 2쪽 92.99(전쪽 최저 Native 93.14, WASM 92.99).
//! 수정 전(#8531 head) Native 2쪽 56.44. 2쪽 남은 차이는 표 B 와 뒤 글줄이 2.2px 위(표 줄 뒤 간격 절반
//! 상한, #8544) — 확대 판독으로 확인. `--compat 2024` 경로(한/글 2024 Print 대조 Native 53.45 / WASM 53.20,
//! 수정 전 56.36)는 #5524 앞 앵커 줄 회수가 도형 줄을 거두는 기존 결함이라 #8544 로 분리했다.
//! 정식 증거: `mydocs/pr/assets/issue_8543/{native,wasm}/*_silhouette.tsv`, 대표 review·overlay
//! `mydocs/pr/assets/issue_8543/png/`.
//!
//! #8543 저장 HWP5 의 TAC 표 host 문단에서 표 앞 줄을 맨 앞 TAC 그림/도형(앞에 다른 TAC 개체 없음)이
//! 소유하면, 조판의 문단 높이 상한(`tac_reconcile`)이 그 줄을 세지 않아 흐름을 되감았다. 글줄은 저장 vpos 로
//! 그려져 맞지만 뒤 TAC 표는 되감긴 조판 좌표에 놓여 그 줄 높이만큼 위로 올라갔다.
//!
//! 입력(공개 합성, `samples/issue8543/`): `gen_leading_shape_line.py` 가 한/글 2020 저장본
//! `samples/issue8507/tac_adjacent-saved.hwpx` 의 표 A host 를 2쪽 머리로 넘기고 표 A 앞에 글뒤로 TAC 사각형
//! (3105HU)을 넣어 `leading_shape_line.hwpx` 를 만든다. `leading_shape_line-2020.hwp` 는 그것을 한/글 2020 이
//! HWP 로 저장한 파일(저장 줄: 줄 0 사각형 vpos 0, 줄 1 ts 8 표 A vpos 3705)이며 검사 입력이다.
//!
//! 독립 기준은 한/글 2020·2024 Print(`pdf/issue8543/leading_shape_line-2020.pdf`·`-2024.pdf`, 2쪽, 두 판 래스터 동일): 2쪽 표 A 바닥 → 표 B 위끝이 표 A 줄 간격
//! 600HU + 바깥 아래·위 여백(141HU 씩)만큼 떨어진다. 수정 전 rhwp 는 표 B 를 사각형 줄 높이만큼 위에 두어
//! 표 A 와 겹쳤다. 비공개 i270(148728029, 3쪽 표 −45.6px)에서 발견했다.
//!
//! 검사: 쪽수(한/글 2쪽), 2쪽 표 A·B·「B 뒤 첫 줄」의 순서와 겹치지 않음, 표 A 바닥 → 표 B 위끝 간격이 두
//! 바깥 여백 이상(관계형). 표 줄 뒤 간격 전량 소비(#8544)는 이 검사 범위가 아니라 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn text_of(node: &RenderNode) -> String {
    let mut text = String::new();
    let mut stack = vec![node];
    while let Some(n) = stack.pop() {
        if let RenderNodeType::TextRun(run) = &n.node_type {
            text.push_str(&run.text);
        }
        stack.extend(n.children.iter().rev());
    }
    text
}

fn flow(node: &RenderNode, out: &mut Vec<(&'static str, String, f64, f64)>) {
    match &node.node_type {
        RenderNodeType::Table(_) => {
            out.push((
                "table",
                text_of(node),
                node.bbox.y,
                node.bbox.y + node.bbox.height,
            ));
            return;
        }
        RenderNodeType::TextLine(_) => {
            out.push((
                "line",
                text_of(node),
                node.bbox.y,
                node.bbox.y + node.bbox.height,
            ));
            return;
        }
        _ => {}
    }
    for child in &node.children {
        flow(child, out);
    }
}

#[test]
fn leading_tac_shape_line_counts_in_tac_paragraph_cap() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8543/leading_shape_line-2020.hwp");
    let core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("문서 열기");
    assert_eq!(core.page_count(), 2, "한/글 Print 쪽수");

    let tree = core.build_page_render_tree(1).expect("2쪽 배치");
    let mut items = Vec::new();
    flow(&tree.root, &mut items);
    let find = |kind: &str, text: &str| {
        items
            .iter()
            .find(|(k, t, _, _)| *k == kind && t.contains(text))
            .unwrap_or_else(|| panic!("{kind} {text:?} 없음: {items:?}"))
            .clone()
    };
    let a = find("table", "A0가");
    let b = find("table", "B0가");
    let after = find("line", "B 뒤 첫 줄");

    // 순서·겹침: 표 A → 표 B → 「B 뒤 첫 줄」.
    assert!(
        a.3 <= b.2,
        "표 B 위끝 {} 가 표 A 바닥 {} 보다 위(겹침)",
        b.2,
        a.3
    );
    assert!(b.3 <= after.2, "「B 뒤 첫 줄」이 표 B 와 겹침: {items:?}");
    // 표 A 바닥 → 표 B 위끝은 적어도 두 바깥 여백(141HU 씩)이다.
    let margins = rhwp::renderer::hwpunit_to_px(141 * 2, 96.0);
    assert!(
        b.2 - a.3 >= margins - 0.5,
        "표 A→B 간격 {} < 바깥 여백 합 {margins}",
        b.2 - a.3
    );
}
