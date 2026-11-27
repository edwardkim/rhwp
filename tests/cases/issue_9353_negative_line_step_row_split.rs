//! 관계형·고정형 — [#9353] 음수 줄 간격 저장 줄의 정상 다음 줄을 쪽 되감김으로 읽어 표 행을 쪽 안에서 나누지 않는다.
//!
//! 주석 갱신: 2026-11-23 20:11 KST(+09:00), 새 파일.
//!
//! - 입력: 공개 문서 `samples/issue9353/uiwang_lane_min_length.hwp`(10k 공개 코퍼스
//!   `21013395_[별표 5] 변속차로의 최소길이(의왕시 도로와 다른 시설의 연결에 관한 조례).hwp`,
//!   한컴 2024 13.0.0.2986 저장본, 44×12 RowBreak 표).
//! - 기준: `pdf/issue9353/uiwang_lane_min_length-2024.pdf`(한컴 2024 Hwp.exe 13.0.0.564,
//!   `Hancom PDF` Print, `hancom2024_pdf_driver_one_up`, 3쪽, SHA-256 `84249e6a…91dc35e5`).
//!   행 37(`21대 이상 50대 이하`)은 2쪽에 온전히 있고 3쪽은 행 38(`51대 이상`)로 시작한다.
//! - 저장 근거: 셀[156] r=37,c=1 `ls[0] vpos 0 lh 1200 ls −240`, `ls[1] vpos 960` —
//!   0 + 1200 − 240 = 960 이라 둘째 줄은 첫 줄이 저장한 전진에 정확히 놓인 평범한 줄이다.
//!   셀[129] r=32,c=1 `11대 이상 30대 이하` 도 같은 꼴이다.
//! - 수정 전 rhwp 는 960 < 0 + 1200 을 저장 쪽 되감김(hard break)으로 읽어 행 37 을 2·3쪽에
//!   나눠 그렸다(3쪽 머리에 `50대 이하` 조각).
//!
//! 검사: 전체 쪽수는 한컴 3쪽이다. 모든 쪽에서 두 줄짜리 칸의 둘째 줄 `50대 이하` 는 첫 줄
//! `…대 이상` 과 같은 쪽에 있다. 행 37 은 2쪽, 3쪽은 `51대 이상` 행을 담고 `50대 이하` 조각이 없다.
//! 수정 전 두 검사 모두 3쪽의 `50대 이하` 조각으로 실패한다.
//!
//! 측정(2026-11-23, base = devel `17b17c934b`, 후보 = base + 이 수정,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96 --compat 2022`, raw =
//! `silhouette_raw_match_percent`, 글꼴은 스크립트 기본 공급):
//! - Native: 3쪽 / 3쪽, 1쪽 98.97 → 98.97, 2쪽 94.05 → 96.84, 3쪽 43.47 → 97.67, 전쪽 최저 96.84(2쪽)
//! - fresh WASM: 3쪽 / 3쪽, 1쪽 99.25 → 99.25, 2쪽 93.78 → 96.89, 3쪽 44.10 → 97.77, 전쪽 최저 96.89(2쪽)
//! - 2쪽 Native·WASM raw 가 98% 미만이므로 절대 px 좌표는 고정하지 않는다.
//! - 증거: `mydocs/pr/assets/issue_9353/`(native·wasm·base 전쪽 TSV, 2·3쪽 review·overlay PNG, base 3쪽)
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::font_environment::FontEnvironment;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue9353/uiwang_lane_min_length.hwp";
/// 한컴 PDF 의 전체 쪽수.
const HANCOM_PAGES: u32 = 3;
/// 음수 줄 간격으로 저장한 두 줄짜리 칸의 둘째 줄과, 그 칸 전체(첫 줄 끝 + 둘째 줄).
/// 1쪽의 다른 표 행도 같은 칸을 가지므로 쪽마다 개수를 맞춘다.
const TWO_LINE_TAIL: &str = "50대이하";
const TWO_LINE_WHOLE: &str = "대이상50대이하";
/// 행 37 의 칸. 한컴 PDF 에서 2쪽에 온전히 있다.
const ROW_37: &str = "21대이상50대이하";
const ROW_37_TAIL: &str = "50대이하";
/// 3쪽이 시작하는 행 38 의 칸.
const ROW_38: &str = "51대이상";

fn flat(text: &str) -> String {
    text.chars().filter(|c| !c.is_whitespace()).collect()
}

fn page_texts() -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let core = DocumentCore::from_bytes(&bytes).unwrap_or_else(|e| panic!("open {SAMPLE}: {e:?}"));
    assert_eq!(core.page_count(), HANCOM_PAGES, "한컴 PDF 의 전체 쪽수");
    (0..core.page_count())
        .map(|page| flat(&core.extract_page_text_native(page).unwrap_or_default()))
        .collect()
}

#[test]
fn negative_spacing_two_line_cell_tail_never_leaves_its_first_line() {
    let pages = page_texts();
    for (page, text) in pages.iter().enumerate() {
        assert_eq!(
            text.matches(TWO_LINE_TAIL).count(),
            text.matches(TWO_LINE_WHOLE).count(),
            "{}쪽: 둘째 줄 `50대 이하` 가 첫 줄 `…대 이상` 없이 홀로 그려졌다 — 저장 둘째 줄 \
             (vpos = 첫 줄 vpos + lh + ls)을 되감김으로 읽어 칸을 쪽 사이에서 나눴다",
            page + 1
        );
    }
}

#[test]
fn row_37_closes_page_two_and_page_three_starts_at_row_38() {
    let pages = page_texts();
    assert!(
        pages[1].contains(ROW_37),
        "행 37 `21대 이상 50대 이하` 는 한컴 PDF 처럼 2쪽에 온전히 있어야 한다"
    );
    assert!(
        !pages[2].contains(ROW_37_TAIL),
        "3쪽에 행 37 의 `50대 이하` 조각이 다시 그려졌다 — 행을 쪽 안에서 나누면 안 된다"
    );
    assert_eq!(
        pages[2].matches(ROW_38).count(),
        1,
        "3쪽은 행 38 `51대 이상` 한 번을 담는다"
    );
}

/// 관계형. 같은 원본의 한컴 2024 Print 두 건에서 기호 슬롯 프로그램에 따른
/// 문자 경계를 확인한다. 파일에 저장된 쪽수와 원본 바이트는 두 경로에서 보존한다.
#[test]
fn selected_symbol_program_rejoins_text_in_the_existing_note_lines() {
    fn collect_lines(node: &RenderNode, lines: &mut Vec<String>) {
        if matches!(node.node_type, RenderNodeType::TextLine(_)) {
            lines.push(
                node.children
                    .iter()
                    .filter_map(|child| match &child.node_type {
                        RenderNodeType::TextRun(run) => Some(run.text.as_str()),
                        _ => None,
                    })
                    .collect(),
            );
        }
        for child in &node.children {
            collect_lines(child, lines);
        }
    }

    let mut core = DocumentCore::from_bytes(include_bytes!(
        "../../samples/issue9353/uiwang_lane_min_length.hwp"
    ))
    .unwrap();
    core.set_hangul2024_compat(true);
    let note_lines = |core: &DocumentCore| {
        assert_eq!(core.page_count(), HANCOM_PAGES);
        let mut lines = Vec::new();
        collect_lines(&core.build_page_render_tree(2).unwrap().root, &mut lines);
        let note = lines
            .iter()
            .position(|line| line.contains("주차 대수를 산정할 때에는"))
            .expect("3쪽 비고 3번");
        [
            lines[note].trim().to_owned(),
            lines[note + 1].trim().to_owned(),
        ]
    };
    let [top, bottom] = note_lines(&core);
    assert!(top.ends_with("」 별") && bottom.starts_with("표 1의"));
    let environment = FontEnvironment::from_json(include_str!(
        "../../mydocs/pr/assets/planet6897_stored_layout_20261123/pr-9364/hcr-symbol-slot.json"
    ))
    .unwrap();
    core.set_font_environment(Some(environment)).unwrap();
    let [top, bottom] = note_lines(&core);
    assert!(
        top.ends_with("」 별표") && bottom.starts_with("1의"),
        "함초롬바탕 낫표 전진으로 채운 비고: {top:?} / {bottom:?}"
    );
    core.set_font_environment(None).unwrap();
    let [top, bottom] = note_lines(&core);
    assert!(top.ends_with("」 별") && bottom.starts_with("표 1의"));
}
