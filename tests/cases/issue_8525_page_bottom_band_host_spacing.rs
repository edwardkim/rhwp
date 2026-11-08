//! 관계형·고정형(쪽수·독립 내용 및 계약 기대값): PR #8531의 정식 회귀.
//! 주석 갱신: 2026-11-05T04:02:25+09:00; 직전 파일 변경: 2026-11-03T20:32:37+09:00 (`c2660b4cf79980b020e48a6f5c4f3707a480fcda`).
//! 현재 통합 측정 근거 시작
//! 측정일 2026-11-04, 제품 코드 `4b149b6cdfc514b9858ed644be2cf8120c731cd4` · `4b149b6cdf`.
//! 실제 전쪽 측정 producer를 기록한다. 이후 충돌 없는 리베이스의 집중 검증은 개별 리뷰에서 구분한다.
//! 96 DPI, Native Skia/fresh WASM Canvas, `visual_sweep.py --silhouette-only`의
//! `silhouette_raw_match_percent`(색상 경계 보정 전 2px 내용 실루엣).
//! 입력 `samples/issue8525/band_host_spacing_22.hwpx`; 독립 Print `pdf/issue8525/band_host_spacing_22-2020.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8525/band_host_spacing_23.hwpx`; 독립 Print `pdf/issue8525/band_host_spacing_23-2020.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8525/band_host_spacing_23.hwpx`; 독립 Print `pdf/issue8525/band_host_spacing_23-2024.pdf`.
//! 전체 1쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 입력 `samples/issue8525/band_host_spacing_24.hwpx`; 독립 Print `pdf/issue8525/band_host_spacing_24-2020.pdf`.
//! 전체 2쪽, 전쪽 최저 Native 100.00% / fresh WASM 100.00%.
//! 주장 1쪽 raw: Native 100.00% / fresh WASM 100.00%.
//! 원본·TSV·대응 쪽 review/overlay: `mydocs/pr/pr_8531_review.md`.
//! 전쪽 통과와 실제 절대 px 좌표 고정은 다른 조건이다. 본문 assertion의 원본 단위·상대 관계를 구분한다.
//! 현재 통합 측정 근거 끝
//!
//! 아래는 선행 작성자의 구현 설명과 당시 관측 기록이며 현재 측정은 위를 따른다.
//!
//! 주석 갱신: 2026-11-03 18:30 KST (새 파일)
//!
//! 측정 근거(측정일 2026-11-03, `scripts/visual_sweep.py --silhouette-only --dpi 96`, 판정값
//! `silhouette_raw_match_percent` — 색상 경계 보정 전 2px 내용 실루엣, 이진화는 스크립트 기본값,
//! 색상 경계 대조 픽셀은 전부 0). 후보 source 는 `task/8525-page-bottom-band-host-spacing` head
//! (#8522 head `98e69aed17` 위, `upstream/devel` `7d06d5c706` 대비 src diff sha256 `8c8b0b7a449fae1c`),
//! Native = native-skia release(sha256 `5321f7bdd3ce3611`), fresh WASM = 같은 source 의
//! `scripts/wasm-pack-locked.sh --target web`(`rhwp_bg.wasm` sha256 `116378d0b9c478c8`).
//! 리베이스 전(Native `356945ebcf43b62b`, WASM `53eb524276c7eb57`)도 같은 점수.
//! 글꼴 `RHWP_FONT_PATH` = 한컴 `ttfs/hwp`·`ttfs/windows`. rhwp 쪽수 = 기준 PDF 쪽수.
//!
//! | 원본 → 기준 Print | 쪽 | Native | fresh WASM |
//! |---|---|---|---|
//! | `band_host_spacing_22.hwpx` → `-22-2020.pdf` | 1 | 100.00 | 100.00 |
//! | `band_host_spacing_23.hwpx` → `-23-2020.pdf` | 1 | 100.00 | 100.00 |
//! | `band_host_spacing_23.hwpx` → `-23-2024.pdf`(`--compat 2024`) | 1 | 100.00 | 100.00 |
//! | `band_host_spacing_24.hwpx` → `-24-2020.pdf` | 2 | 100.00 / 100.00 | 100.00 / 100.00 |
//!
//! 두 경로 모두 전쪽 최저 100.00%. 정식 증거: `mydocs/pr/assets/issue_8525/{native,wasm}/*_silhouette.tsv`,
//! 대표 review·overlay `mydocs/pr/assets/issue_8525/png/`. 기준 PDF 는 `pdf/issue8525/`(한/글 2020
//! 11.0.0.9136, 2024 13.0.0.3901 Print).
//!
//! #8525 쪽 아래 정렬(`vertRelTo=PAGE`, `vertAlign=BOTTOM`) 자리차지 틀의 띠 높이를 조판이
//! `표 높이 + host 문단 간격` 으로 잡아, 마지막 글줄이 한/글보다 먼저 다음 쪽으로 넘어갔다.
//!
//! 입력(공개 합성, `samples/issue8525/`): `gen_band_host_spacing.py` 가 `samples/issue8507/tac_adjacent-saved.hwpx`
//! (한/글 2020 저장본)의 표 A host 문단(글자 없음, 저장 줄 간격 600HU)에서 표 A 앞에 발신명의 틀 꼴의
//! 쪽 아래 틀(1x1, 선언 7593HU, 칸 안에 글자처럼 취급 1x3 표)을 끼우고 문서 끝에 1600HU 글줄을 N 개 더한다.
//! - `band_host_spacing_22.hwpx`·`_23`·`_24`: N = 22·23·24.
//!
//! 독립 기준은 한/글 Print(`pdf/issue8525/`): N=22·23 은 1쪽(23 은 2020·2024 모두), N=24 는 2쪽.
//! 수정 전 rhwp 는 띠를 host 줄 간격 600HU(8.0px)만큼 높게 잡아 N=23 의 「채움 23」을 2쪽으로 넘겼다.
//! 같은 판정은 비공개 결재문서 36443518(한/글 2020·2024 Print 2쪽, 틀 그린 높이 101.27px ≈ 선언)에서
//! 발견했다 — host 간격 100HU 가 더해진 102.57px 띠가 마지막 빈 문단을 3쪽으로 밀었다.
//!
//! 검사: 쪽수는 한/글 Print 와 같고(고정형), 마지막 글줄의 쪽 소속·내용 순서가 보존되며 1쪽에 남은
//! 마지막 글줄의 아래끝이 그린 틀 위끝을 넘지 않는다(관계형). 절대 좌표는 고정하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};
use rhwp::DocumentCore;

fn open(path: &str) -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    DocumentCore::from_bytes(&std::fs::read(path).expect("기준 입력")).expect("기준 문서 열기")
}

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

/// 쪽의 본문 글줄(글자, 위끝, 아래끝)과 쪽 아래 틀(첫 문단 밖 표 중 가장 아래)의 위끝.
struct Page {
    lines: Vec<(String, f64, f64)>,
    frame_top: Option<f64>,
}

fn page(core: &DocumentCore, index: u32) -> Page {
    fn walk(
        node: &RenderNode,
        lines: &mut Vec<(String, f64, f64)>,
        tables: &mut Vec<(f64, f64, String)>,
    ) {
        match &node.node_type {
            RenderNodeType::Table(_) => {
                tables.push((node.bbox.y, node.bbox.y + node.bbox.height, text_of(node)));
                return;
            }
            RenderNodeType::TextLine(_) => {
                lines.push((text_of(node), node.bbox.y, node.bbox.y + node.bbox.height));
                return;
            }
            _ => {}
        }
        for child in &node.children {
            walk(child, lines, tables);
        }
    }
    let tree = core.build_page_render_tree(index).expect("쪽 배치");
    let (mut lines, mut tables) = (Vec::new(), Vec::new());
    walk(&tree.root, &mut lines, &mut tables);
    // 틀은 표 A·B(글자 A0가·B0가) 가 아닌, 쪽에서 가장 아래 놓인 표다.
    let frame_top = tables
        .iter()
        .filter(|(_, _, t)| !t.contains("A0") && !t.contains("B0"))
        .map(|(top, _, _)| *top)
        .fold(None, |acc: Option<f64>, top| {
            Some(acc.map_or(top, |a| a.max(top)))
        });
    Page { lines, frame_top }
}

fn find_line<'a>(p: &'a Page, text: &str) -> Option<&'a (String, f64, f64)> {
    p.lines.iter().find(|(t, _, _)| t.trim() == text)
}

fn check(path: &str, fillers: usize, hancom_pages: u32) {
    let core = open(path);
    assert_eq!(core.page_count(), hancom_pages, "{path}: 한/글 Print 쪽수");
    let first = page(&core, 0);
    let frame_top = first.frame_top.expect("1쪽 쪽 아래 틀");
    let last = format!("채움 {fillers}");
    let prev = format!("채움 {}", fillers - 1);

    // 내용 보존·순서: 「채움 N-1」 은 1쪽, 「채움 N」 은 한/글이 둔 쪽.
    let prev_line = find_line(&first, &prev).unwrap_or_else(|| panic!("{path}: 1쪽 {prev:?}"));
    if hancom_pages == 1 {
        let last_line = find_line(&first, &last).unwrap_or_else(|| panic!("{path}: 1쪽 {last:?}"));
        assert!(prev_line.2 <= last_line.1, "{path}: 순서");
        // 띠에 host 간격이 더해지지 않으면 마지막 글줄 아래끝이 그린 틀 위끝 안에 든다.
        assert!(
            last_line.2 <= frame_top + 0.5,
            "{path}: {last:?} 아래끝 {} > 틀 위끝 {frame_top}",
            last_line.2
        );
    } else {
        assert!(
            find_line(&first, &last).is_none(),
            "{path}: {last:?} 는 2쪽"
        );
        let second = page(&core, 1);
        assert!(find_line(&second, &last).is_some(), "{path}: 2쪽 {last:?}");
    }
}

#[test]
fn page_bottom_band_excludes_host_line_spacing() {
    check("samples/issue8525/band_host_spacing_22.hwpx", 22, 1);
    check("samples/issue8525/band_host_spacing_23.hwpx", 23, 1);
    check("samples/issue8525/band_host_spacing_24.hwpx", 24, 2);
}
