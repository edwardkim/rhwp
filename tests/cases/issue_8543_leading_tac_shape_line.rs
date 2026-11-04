//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8546의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-04T01:10:57+09:00 (`fd97ae040f45d4aa7c681bc6b28def4236c61d09`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8543/leading_shape_line-2020.hwp`; 독립 Print `pdf/issue8543/leading_shape_line-2020.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 93.14% / fresh WASM 92.99%.
//! 주장 2쪽 raw: Native 93.14% / fresh WASM 92.99%.
//! 입력 `samples/issue8543/leading_shape_line-2020.hwp`; 독립 Print `pdf/issue8543/leading_shape_line-2024.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 93.44% / fresh WASM 93.09%.
//! 주장 2쪽 raw: Native 93.44% / fresh WASM 93.09%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8546_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-04 01:06:53 KST
//!
//! 측정일 2026-11-04, 제품 코드 `dc87f4fa2e`, 96 DPI,
//! `scripts/visual_sweep.py --silhouette-only`의 `silhouette_raw_match_percent`
//! (색상 경계 보정 전 2px 내용 실루엣). 입력은 `samples/issue8543/leading_shape_line-2020.hwp`.
//! 독립 한컴 Print PDF와 Native/fresh WASM 모두 전체 2쪽으로 일치한다.
//! - 2020 Print: Native 1쪽 100.00%, 2쪽 93.14%(전쪽 최저 93.14%);
//!   fresh WASM 1쪽 100.00%, 2쪽 92.99%(전쪽 최저 92.99%).
//! - 2024 Print: Native 1쪽 100.00%, 2쪽 93.44%(전쪽 최저 93.44%);
//!   fresh WASM 1쪽 100.00%, 2쪽 93.09%(전쪽 최저 93.09%).
//!
//! 2024 경로의 보정 전 Native 2쪽 53.45%는 도형 소유 줄까지 앵커 회수하던 결함이다.
//! `ad0d4b1277`에서 공통 회수 조건을 보정했으며 이 검사는 두 호환 모드의 순서·겹침을 보호한다.
//! 핵심 쪽의 양쪽 raw가 98% 미만이므로 실제 절대 px 좌표는 고정하지 않는다.
//! Native 실행 파일 SHA-256 `24c7e929b04c14adc46fbcc80b384e74fffd5524bdd2484068ebe6e974952337`,
//! fresh WASM SHA-256 `c7e166e1b087641e727d326dd5ff504653f96134808242f9d1fd83228bbae903`.
//! 현재 측정의 전쪽 TSV·review·overlay: `mydocs/pr/assets/planet6897_20261103/pr_8546/`.
//! 입력 생성·원본 저장값·독립 Print 절차: `mydocs/pr/pr_8546_review.md`.
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
    let bytes = std::fs::read(path).expect("기준 입력");
    for compat_2024 in [false, true] {
        let mut core = DocumentCore::from_bytes(&bytes).expect("문서 열기");
        core.set_hangul2024_compat(compat_2024);
        assert_eq!(core.page_count(), 2, "한/글 Print 쪽수; 2024={compat_2024}");

        let tree = core.build_page_render_tree(1).expect("2쪽 배치");
        let mut items = Vec::new();
        flow(&tree.root, &mut items);
        let find = |kind: &str, text: &str| {
            items
                .iter()
                .find(|(k, t, _, _)| *k == kind && t.contains(text))
                .unwrap_or_else(|| panic!("{kind} {text:?} 없음; 2024={compat_2024}: {items:?}"))
                .clone()
        };
        let a = find("table", "A0가");
        let b = find("table", "B0가");
        let after = find("line", "B 뒤 첫 줄");

        // 순서·겹침: 표 A → 표 B → 「B 뒤 첫 줄」.
        assert!(
            a.3 <= b.2,
            "표 B 위끝 {} 가 표 A 바닥 {} 보다 위; 2024={compat_2024}",
            b.2,
            a.3
        );
        assert!(
            b.3 <= after.2,
            "「B 뒤 첫 줄」이 표 B 와 겹침; 2024={compat_2024}: {items:?}"
        );
        // 표 A 바닥 → 표 B 위끝은 적어도 두 바깥 여백(141HU 씩)이다.
        let margins = rhwp::renderer::hwpunit_to_px(141 * 2, 96.0);
        assert!(
            b.2 - a.3 >= margins - 0.5,
            "표 A→B 간격 {} < 바깥 여백 합 {margins}; 2024={compat_2024}",
            b.2 - a.3
        );
    }
}
