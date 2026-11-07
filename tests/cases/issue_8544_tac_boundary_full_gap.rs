//! 관계형: 저장 HWP5 TAC 표 줄 뒤 간격을 다음 문단 저장 vpos 가 전량 증명하면 조판도 전량을 남긴다.
//! 주석 갱신: 2026-11-05T22:50:00+09:00 (새 파일, 직전 파일 커밋 없음).
//! 측정(2026-11-05, 제품 코드 head `e8e351687a`·base devel `a95d1d5afa`, `scripts/visual_sweep.py --silhouette-only
//! --compat 2022 --dpi 96`, 판정값 `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣),
//! `RHWP_FONT_PATH=ttfs/hwp:ttfs/windows`, Native Skia / fresh WASM Canvas):
//! - 입력 `samples/issue8543/leading_shape_line-2020.hwp`, 독립 Print `pdf/issue8543/leading_shape_line-2020.pdf`
//!   (한/글 2020, 전체 2쪽 = rhwp 2쪽).
//! - head: Native 100.00/100.00%, fresh WASM 100.00/100.00%(전쪽 최저 두 경로 모두 100.00%).
//! - base: 2쪽 Native 93.14% / fresh WASM 92.99%(표 B 가 2.2px 위).
//! - 10k 코퍼스 11,423건 dump-pages A/B: 출력 변화 55건, 쪽수 변화 0건(5690000-202000006 은 #8486 반영 뒤 185=185).
//!   변화 문서 중 한컴 Print 가 있는 37건 Native: +0.5 이상 개선 15쪽, 하락 1쪽(1500000-200600011 40쪽
//!   32.92→29.36, base 부터 쪽 흐름이 어긋난 기존 미달), 90 경계를 내려간 쪽 0.
//! - 근거: `mydocs/pr/assets/issue_8544/{base,head}/{native,wasm}/`, `corpus/`, `png/`, `handbook/`.
//!
//! #8544 TAC 표 host 마지막 줄 뒤 간격을, 다음 문단 첫 저장 줄이 `vpos + lh + ls` 에서 정확히 시작해
//! 전량을 증명해도 조판 상한이 절반만 세어 뒤 TAC 표가 (ls/2 − 바깥 위 여백)만큼 위에 놓였다.
//!
//! 입력: #8543 의 공개 합성 `samples/issue8543/leading_shape_line-2020.hwp`(한/글 2020 이 HWP 로 저장).
//! 2쪽 저장 사다리: 표 A 줄 vpos 3705 + lh 7812 + ls 600 = 표 B 문단 vpos 12117.
//!
//! 독립 기준은 한/글 2020·2024(13.0.0.3901) Print(`pdf/issue8543/leading_shape_line-2020.pdf`·`-2024.pdf`,
//! 두 판 래스터 동일): 표 A 바닥 → 표 B 위끝 = 표 A 줄 간격 600HU + 바깥 아래·위 여백 141HU 씩.
//! 수정 전 rhwp 는 이 간격이 2.2px 짧았다.
//!
//! `--compat 2024`(#5524 Δ1 리비전 모델) 경로는 그 리비전의 이 경계 동작을 확인하지 못해 바꾸지 않는다 —
//! 같은 입력에서 2024 모드 출력이 수정 전과 같음을 함께 검사한다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn tables(node: &RenderNode, out: &mut Vec<(String, f64, f64)>) {
    if let RenderNodeType::Table(_) = &node.node_type {
        let mut text = String::new();
        let mut stack = vec![node];
        while let Some(n) = stack.pop() {
            if let RenderNodeType::TextRun(run) = &n.node_type {
                text.push_str(&run.text);
            }
            stack.extend(n.children.iter().rev());
        }
        out.push((text, node.bbox.y, node.bbox.y + node.bbox.height));
        return;
    }
    for child in &node.children {
        tables(child, out);
    }
}

/// 2쪽 표 A 바닥 → 표 B 위끝 간격(px).
fn gap(hangul2024: bool) -> f64 {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8543/leading_shape_line-2020.hwp");
    let mut core =
        DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("문서 열기");
    core.set_hangul2024_compat(hangul2024);
    let tree = core.build_page_render_tree(1).expect("2쪽 배치");
    let mut out = Vec::new();
    tables(&tree.root, &mut out);
    let a = out.iter().find(|t| t.0.contains("A0가")).expect("표 A");
    let b = out.iter().find(|t| t.0.contains("B0가")).expect("표 B");
    b.1 - a.2
}

#[test]
fn stored_successor_closing_full_gap_keeps_full_tac_line_spacing() {
    // 저장 표 A 줄 간격 600HU + 바깥 아래·위 여백 141HU 씩 — 한/글 Print 의 표 A→B 간격.
    let expected = rhwp::renderer::hwpunit_to_px(600 + 141 + 141, 96.0);
    let default_gap = gap(false);
    assert!(
        (default_gap - expected).abs() < 0.5,
        "기본 경로 표 A→B 간격 {default_gap} ≠ 저장 간격+여백 {expected}"
    );
    // --compat 2024 경로는 바꾸지 않는다: 그 경로의 표 B 는 기본 경로보다 위에 남는다.
    let compat_gap = gap(true);
    assert!(
        compat_gap < default_gap - 0.5,
        "--compat 2024 경로가 바뀌었다: {compat_gap} vs 기본 {default_gap}"
    );
}
