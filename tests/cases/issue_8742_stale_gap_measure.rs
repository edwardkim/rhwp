//! 관계형·고정형 회귀 — 자리차지 표 위 틈의 줄은 측정과 배치가 같은 판정으로 틈에 둔다(#8742).
//! 주석 갱신: 2026-11-07 18:23 KST(+09:00).
//! 측정일: 2026-11-07. raw = `silhouette_raw_match_percent`, 96 DPI, 한/글 2022(12.0.0.4547) Print PDF,
//! `--compat 2022`. Native 는 문서 글꼴(굴림·HY 계열)만 공급했다(큰 글꼴 경로는 Native 64MiB 예산을 넘긴다).
//! 고정형은 PDF 와 맞춘 쪽 수와 쪽 경계 문단이고, 나머지는 사다리 변형 전후의 관계다.
//!
//! ```text
//! 원본 (전체 쪽수)                          대상 쪽  수정 전 N/W           수정 후 N/W      전쪽 최저 N/W
//! samples/issue8742/36345427 (1쪽)          1        2쪽으로 측정 불가     100.00/100.00    100.00/100.00
//! samples/issue8742/36453591 (5쪽)          4·5      95.69·23.18/95.68·23.17  99.47·83.91/99.46·83.31  83.91/83.31
//! samples/issue8742/36483048 (6쪽)          4·5·6    52.85·26.68·5.04/53.42·26.51·5.11  73.02·78.02·99.86/72.68·77.45·99.85  73.02/72.68
//! samples/task1789/exclusion_probe_line_spacing (2쪽, 한/글 2022 PDF)  1·2  99.87·95.70/99.94·95.73  같음(정상 사다리)  95.70/95.73
//! ```
//!
//! 새 회귀 검사는 전쪽 최저 90% 이상인 36345427 만 쓴다. 36453591(5쪽 83.91)·36483048(4쪽 73.02)은
//! 쪽 소속이 PDF 와 같아졌지만 이 수정과 다른 원인으로 90% 미만이 남아 회귀 입력으로 등록하지 않는다.
//! 두 문서는 검증 입력으로만 둔다. 정식 증거는 `mydocs/pr/assets/issue_8742/` 에 있다.
//!
//! 36345427 은 구세대 한/글이 표 위 틈의 줄을 표 아래 자리로 저장했다. 한/글 Print 는 그 줄을 틈에
//! 두고 쪽 아래 결재란(pi=11)까지 1쪽에 담는다. 배치는 이미 그 저장 위치를 버렸는데 측정은 따라가서,
//! 그리지 않는 높이로 결재란을 2쪽에 넘겼다. 기대값은 같은 원본의 한/글 Print PDF 쪽 수다.
//!
//! 끝으로 `samples/task1789/exclusion_probe_line_spacing.hwpx` 의 정상 사다리를 메모리에서 낡은
//! 사다리로 바꿔도 쪽 구성이 같은지 본다. 합성 입력의 계약 검사다.

#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use serde_json::Value;
use std::path::Path;

fn load(relative: &str) -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("입력 파일")).expect("열기")
}

/// 쪽마다 (구역, 첫 문단, 끝 문단).
fn page_spans(core: &DocumentCore) -> Vec<(u64, u64, u64)> {
    let dump: Value = core.dump_page_items_json(None);
    dump.as_array()
        .expect("쪽 목록")
        .iter()
        .map(|page| {
            let section = page["section"].as_u64().expect("구역");
            let paras: Vec<u64> = page["columns"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|column| column["items"].as_array().unwrap())
                .filter_map(|item| item["paraIndex"].as_u64())
                .collect();
            (section, paras[0], *paras.last().unwrap())
        })
        .collect()
}

#[test]
fn page_bottom_sign_off_fits_once_gap_lines_stay_above_the_table() {
    let spans = page_spans(&load("samples/issue8742/36345427_approval_body.hwpx"));
    assert_eq!(spans, vec![(0, 0, 11)], "한/글 Print 는 결재란까지 1쪽이다");
}

#[test]
fn stale_ladder_does_not_change_pages_of_a_saved_gap_flow() {
    const SAMPLE: &str = "samples/task1789/exclusion_probe_line_spacing.hwpx";
    /// 표 host(pi=5) 바로 뒤 문단. 원본에서 표 위 틈에 있다.
    const FIRST_GAP_PARAGRAPH: usize = 6;
    /// 36345427 pi=3→4 의 낡은 전방 이동(저장 차이 13819 - 정상 1920).
    const STALE_FORWARD_HU: i32 = 11899;

    let saved = load(SAMPLE);
    let expected = page_spans(&saved);
    let mut stale = load(SAMPLE);
    let mut document = stale.document().clone();
    for paragraph in &mut document.sections[0].paragraphs[FIRST_GAP_PARAGRAPH..] {
        for line in &mut paragraph.line_segs {
            line.vertical_pos += STALE_FORWARD_HU;
        }
    }
    stale.set_document(document);
    stale.repaginate_if_needed();
    assert_eq!(
        page_spans(&stale),
        expected,
        "낡은 사다리가 쪽 구성을 바꾸면 안 된다"
    );
}
