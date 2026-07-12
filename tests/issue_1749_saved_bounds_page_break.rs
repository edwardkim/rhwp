//! Issue #1749 v2: 누적좌표 문서라도 다음 문단이 명시적 쪽나누기면 saved bounds 를 신뢰한다.
//!
//! Regression shape (samples/task1749/saved_bounds_cumulative_page_break.hwpx):
//! - 2쪽 말미 pi=26 은 누적높이 검사 탈락(919.2+36.3 > 930.5px)이지만 저장 bounds
//!   (vpos 137484 − 2쪽 기준 69310 → bottom ≈ 930.3px ≤ avail)로 2쪽 배치가 정답.
//! - 이 문서는 누적좌표(쪽 경계에서도 vpos 리셋 없음)인데 다음 문단 pi=27 이 명시적
//!   [쪽나누기](column_type=Page)라 "vpos 리셋" 검사로는 페이지-마지막 증거를 못 본다.
//!   #1749 1차 게이트가 이 증거를 누락해 pi=26 이 3쪽 단독 문단으로 밀렸다(5쪽→6쪽).
//! - 저장 lineseg 근거: pi=25(vpos=134764)와 pi=26(vpos=137484)은 한 줄(2720HU) 간격
//!   연속 배치 = 한글은 pi=26 을 2쪽 마지막 줄로 인코딩.

use std::fs;
use std::path::Path;

use rhwp::model::control::Control;

const HWPX_SAMPLE: &str = "samples/task1749/saved_bounds_cumulative_page_break.hwpx";
const HWP_SAMPLE: &str = "samples/task1749/saved_bounds_cumulative_page_break.hwp";

fn load_sample(sample: &str) -> rhwp::wasm_api::HwpDocument {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(sample);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", sample, e));
    rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", sample, e))
}

fn load_doc() -> rhwp::wasm_api::HwpDocument {
    load_sample(HWPX_SAMPLE)
}

#[test]
fn issue_1749_v2_pi26_stays_on_page_2() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        5,
        "쪽나누기 직전 단일 줄 문단 pi=26 이 밀리면 5쪽 문서가 6쪽이 된다"
    );

    let page2 = doc.dump_page_items(Some(1));
    assert!(
        page2.contains("pi=26"),
        "pi=26 은 2쪽 마지막 문단이어야 한다 (저장 lineseg: pi=25 와 한 줄 간격 연속)\n--- page 2 ---\n{}",
        page2
    );
}

#[test]
fn issue_1811_hwpx_pi52_rowbreak_cut_matches_hwp_reference() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        5,
        "p5 tail drift 보정 후에도 전체 5쪽이어야 한다"
    );

    let page4 = doc.dump_page_items(Some(3));
    let page4_lines: Vec<_> = page4.lines().collect();
    let host_idx = page4_lines
        .iter()
        .position(|line| line.contains("PartialParagraph") && line.contains("pi=52"))
        .unwrap_or_else(|| {
            panic!("4쪽에서 pi=52 host 텍스트를 찾지 못함\n--- page 4 ---\n{page4}")
        });
    let table_idx = page4_lines
        .iter()
        .position(|line| line.contains("PartialTable") && line.contains("pi=52"))
        .unwrap_or_else(|| panic!("4쪽에서 pi=52 분할 표를 찾지 못함\n--- page 4 ---\n{page4}"));
    assert!(
        host_idx < table_idx,
        "HWPX RowBreak mixed 문단은 PDF 기준처럼 host 텍스트를 표 fragment 보다 먼저 소비해야 한다\n--- page 4 ---\n{page4}"
    );
    let pi52_line = page4_lines[table_idx];

    // [#2015] 종전 이 테스트는 HWPX end_cut=[1] 을 기대했으나, 그것은 vert_offset 이중계상
    // (pre-emit 된 host_h 위에 vert_off 를 재차감 → page_avail=0)으로 남는 공간이 0 이라
    // 오판된 값이었다. 이중계상을 보정하면 실제 잔여 공간(≈124px)에 3 유닛이 들어가
    // HWPX end_cut=[3] 이 HWP 저장 LINE_SEG 참조([3] 아래) 및 한컴 PDF 와 일치한다.
    assert!(
        pi52_line.contains("end_cut=[3]"),
        "HWPX mixed host 텍스트를 p4 에 먼저 배치한 뒤, vert_offset 이중계상 보정으로 첫 fragment 는 \
         HWP 참조와 동일하게 3 유닛을 담아야 한다(#2015)\n{pi52_line}"
    );

    let hwp_doc = load_sample(HWP_SAMPLE);
    assert_eq!(
        hwp_doc.page_count(),
        5,
        "HWP 저장 LINE_SEG 경로는 5쪽을 유지해야 한다"
    );
    let hwp_page4 = hwp_doc.dump_page_items(Some(3));
    let hwp_pi52_line = hwp_page4
        .lines()
        .find(|line| line.contains("PartialTable") && line.contains("pi=52"))
        .unwrap_or_else(|| {
            panic!("HWP 4쪽에서 pi=52 분할 표를 찾지 못함\n--- page 4 ---\n{hwp_page4}")
        });
    assert!(
        hwp_pi52_line.contains("end_cut=[3]"),
        "HWP 저장 LINE_SEG 경로는 기존 p4 3유닛 컷을 유지해야 한다\n{hwp_pi52_line}"
    );

    let section = &doc.document().sections[0];
    let table52 = section.paragraphs[52]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("pi=52 table");
    let cell52 = table52
        .cells
        .iter()
        .find(|cell| cell.row == 2 && cell.col == 0)
        .expect("pi=52 row 2 merged cell");
    let line_counts52: Vec<usize> = cell52
        .paragraphs
        .iter()
        .map(|para| para.line_segs.len())
        .collect();
    assert_eq!(
        line_counts52,
        vec![1, 2, 2, 2, 2, 1],
        "pi=52 RowBreak 셀은 명시 셀 높이와 저장 anchor lineSeg 기준으로 p1/p4까지 2줄 합성이 필요하다"
    );
}

#[test]
fn issue_1811_hwpx_pi57_tac_rowbreak_cell_uses_saved_height() {
    let doc = load_doc();
    let section = &doc.document().sections[0];
    let table57 = section.paragraphs[57]
        .controls
        .iter()
        .find_map(|control| match control {
            Control::Table(table) => Some(table.as_ref()),
            _ => None,
        })
        .expect("pi=57 table");
    let cell57 = table57
        .cells
        .iter()
        .find(|cell| cell.row == 2 && cell.col == 0)
        .expect("pi=57 row 2 merged cell");
    let line_counts57: Vec<usize> = cell57
        .paragraphs
        .iter()
        .map(|para| para.line_segs.len())
        .collect();
    assert_eq!(
        line_counts57,
        vec![2, 2],
        "TAC RowBreak 셀은 anchor 문단이 없어도 표/셀 저장 높이를 기준으로 부족한 합성 줄을 보강해야 한다"
    );
}
