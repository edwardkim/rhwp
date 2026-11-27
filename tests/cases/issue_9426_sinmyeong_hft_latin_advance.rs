//! 고정형(독립 한/글 Print 좌표 ±2px)·관계형(보통/굵게 라틴 런의 시작·끝) — [#9426] HFT 로 저장된 `신명 중명조` 라틴 글자는 원 프로그램 폭으로 잰다.
//! 주석 갱신: 2026-11-27 10:55 KST (새 파일, devel `fef912a6fb` 로 rebase 뒤 재측정).
//!
//! - 입력: `samples/issue8482/germany_law_ch3_resaved2020.hwp`(#8482 fixture, sha256 945183c9…401f) — HWP3
//!   「독일의 법령체계와 입법심사기준」을 rhwp 로 HWP5 변환해 한/글 2020 으로 다시 저장하고 131~141쪽만 남긴
//!   11쪽 문서. 한/글이 다시 저장하면서 `신명 중명조` 를 HFT(유형 2)로 적는다.
//!   기준: `pdf/issue8482/germany_law_ch3-2020.pdf`(한/글 2020 11.0.0.1623 `Hancom PDF` Print, 11쪽).
//! - 원인: HFT 글꼴은 표시 대체 이름(`HY신명조`)으로 바뀐 뒤 측정되는데, `신명 중명조` 는 원 프로그램
//!   폭 표(#7293, 한/글 Type3 `/Widths`)를 가졌어도 HFT 측정 face 목록에 없어 표시 대체 폭으로 쟀다.
//!   라틴 글자가 한/글보다 좁아(`a` 0.540em ↔ 0.654em) 양쪽 정렬이 남는 폭을 낱말 사이로 벌렸다.
//! - 굵게: 한/글 Print 의 굵은 Type3 글꼴은 보통과 같은 `/Widths` 를 쓰고 TJ 로 비공백 글자마다
//!   약 0.05em 더 나아간다(쪽 끝 정렬 없는 마지막 줄 `Schulrecht` 도 같다). 다른 HFT 표와 같은 굵기 간격이다.
//! - 검사(고정형): 3쪽 각주의 굵은 런 `Parlamentsrecht`(시작 293.28 · 뒤 쉼표 421.37) 와
//!   `Information der Parlamente durch die Regierungen`(278.25 · 693.70), 보통 런 `Staff`
//!   (147.12 · 181.18)의 시작과 끝을 한/글 Print 좌표(96 DPI, Type3 글자 원점)에 ±2px 로 맞춘다.
//!   수정 전에는 시작이 14~18px 어긋나고(`Parlamentsrecht` 279.5) 런 폭이 한/글보다 좁다.
//! - 입력 조건: 세 런의 라틴 슬롯 글꼴은 HFT `신명 중명조` 이다(표시 이름만 바뀌는 경로).
//!
//! 측정일 2026-11-27. raw 측정: `scripts/visual_sweep.py --silhouette-only --compat 2022 --dpi 96` 의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣). base = upstream/devel `fef912a6fb`,
//! head = 이 수정.
//!
//! | 쪽 | Native base | Native head | fresh WASM base | fresh WASM head |
//! |---|---|---|---|---|
//! | 3 | 95.14 | 99.89 | 95.29 | 99.81 |
//!
//! 전체 11/11쪽 일치. head 전쪽 최저: Native 99.12(11쪽), fresh WASM 98.86(9쪽). 90% 미만 없음
//! (base 최저 Native 90.58 · fresh WASM 90.81). 11쪽 모두 오르고 내려간 쪽은 없다.
//! 증거: `mydocs/pr/assets/issue_9426/{native,wasm,base-native,base-wasm}/` 의 전쪽 TSV 와 3쪽 review·overlay PNG.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue8482/germany_law_ch3_resaved2020.hwp";
/// 한/글 2020 Print 쪽수.
const HANCOM_PAGES: u32 = 11;
/// 3쪽(0 기준 2).
const PAGE: u32 = 2;
const FACE: &str = "신명 중명조";
/// HWP5 FACE_NAME 유형: 2 = HFT.
const HFT: u8 = 2;
/// 라틴(영문) 글꼴 슬롯.
const LATIN_SLOT: usize = 1;
const TOLERANCE_PX: f64 = 2.0;

/// (런 글자, 한/글 Print 의 시작 x, 다음 글자의 시작 x, 굵게).
const RUNS: [(&str, f64, f64, bool); 3] = [
    ("Parlamentsrecht", 293.28, 421.37, true),
    (
        "Information der Parlamente durch die Regierungen",
        278.25,
        693.70,
        true,
    ),
    ("Staff", 147.12, 181.18, false),
];

fn open() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"));
    assert_eq!(
        core.page_count(),
        HANCOM_PAGES,
        "한/글 2020 Print 와 같은 쪽수"
    );
    core
}

fn collect_runs<'a>(node: &'a RenderNode, text: &str, out: &mut Vec<&'a RenderNode>) {
    if let RenderNodeType::TextRun(run) = &node.node_type {
        if run.text == text {
            out.push(node);
        }
    }
    for child in &node.children {
        collect_runs(child, text, out);
    }
}

/// 각주 영역(굵은 `Parlamentsrecht` 서명 줄 이후)의 첫 런. 본문에도 같은 낱말(`Staff`)이 있다.
fn footnote_run<'a>(root: &'a RenderNode, text: &str) -> &'a RenderNode {
    let mut anchors = Vec::new();
    collect_runs(root, RUNS[0].0, &mut anchors);
    let [anchor] = anchors.as_slice() else {
        panic!("3쪽 각주 서명 {:?} 는 한 번", RUNS[0].0);
    };
    let mut found = Vec::new();
    collect_runs(root, text, &mut found);
    found
        .into_iter()
        .filter(|node| node.bbox.y >= anchor.bbox.y)
        .min_by(|a, b| a.bbox.y.total_cmp(&b.bbox.y))
        .unwrap_or_else(|| panic!("3쪽 각주 런 {text:?}"))
}

#[test]
fn hft_sinmyeong_latin_runs_start_and_end_where_hancom_prints_them() {
    let core = open();
    let tree = core.build_page_render_tree(PAGE).unwrap();
    let doc_info = &core.document().doc_info;
    for (text, start, end, bold) in RUNS {
        let node = footnote_run(&tree.root, text);
        let RenderNodeType::TextRun(run) = &node.node_type else {
            unreachable!()
        };
        // 입력 조건: HFT `신명 중명조` 라틴 슬롯, 굵기.
        let shape = &doc_info.char_shapes[run.char_shape_id.expect("글자 모양") as usize];
        let face = &doc_info.font_faces[LATIN_SLOT][shape.font_ids[LATIN_SLOT] as usize];
        assert_eq!(face.name, FACE, "{text:?} 라틴 글꼴");
        assert_eq!(face.alt_type, HFT, "{text:?} 는 HFT 로 저장된 글꼴");
        assert_eq!(shape.bold, bold, "{text:?} 굵기");

        let got_start = node.bbox.x;
        let got_end = node.bbox.x + node.bbox.width;
        assert!(
            (got_start - start).abs() <= TOLERANCE_PX,
            "{text:?} 시작 {got_start:.2}px 는 한/글 {start:.2}px 에서 ±{TOLERANCE_PX}px 안"
        );
        assert!(
            (got_end - end).abs() <= TOLERANCE_PX,
            "{text:?} 끝 {got_end:.2}px 는 한/글의 다음 글자 {end:.2}px 에서 ±{TOLERANCE_PX}px 안"
        );
    }
}
