//! [#5880/#5784] 1×1 RowBreak 칸 조각에 상자 높이보다 많은 내용을 채워 조각 말미
//! 줄·표가 clip 에 소실되던 문서 — 조각 회계와 페인터의 저장 사다리 스냅을 정합시킨다.
//!
//! `rowbreak_fragment_overfill.hwpx`(2737927 재건축부담금 평가지침 별표 1)는 본문
//! 전체가 1×1 RowBreak 칸 안에 있고 쪽마다 조각으로 쪼개진다. 수정 전 rhwp 는
//! 컷 회계가 페인터보다 짧아(빈 문단 spacer 0높이·중첩 host ls 누락·한글 쪽 프레임
//! 리셋 흡수·연속 조각 원점 미보정) 조각마다 1~5줄을 초과 배정했고, 넘친 12줄·표가
//! clip 으로 사라졌다(한컴 PDF 대비 -125~187자). 한글 2022 는 8쪽, rhwp 는 7쪽이었다.
//!
//! 수정 후: 쪽수 8, `LAYOUT_OVERFLOW_CELL`/off-canvas 0, PDF 문자 멀티셋 잔차는
//! 한컴 PDF 측 PUA 인코딩 잡음(U+E0xx 6자)+`×` 1자뿐이다.

use std::fs;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5880/rowbreak_fragment_overfill.hwpx";

fn load_doc() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&fs::read(&path).expect("read sample")).expect("open")
}

#[test]
fn issue_5880_page_count_matches_hangul_8() {
    let doc = load_doc();
    assert_eq!(
        doc.page_count(),
        8,
        "#5880: 한글 2022 는 이 문서를 8쪽으로 조판한다 — 결함 시 7쪽(조각 과적재)"
    );
}

#[test]
fn issue_5880_no_overflow_cell_lines() {
    let doc = load_doc();
    let _ = doc.take_overflow_cell_lines();
    let mut total = 0u64;
    for page in 0..doc.page_count() {
        doc.render_page_svg_native(page)
            .unwrap_or_else(|e| panic!("render page {}: {e:?}", page + 1));
        total += u64::from(doc.take_overflow_cell_lines());
    }
    assert_eq!(
        total, 0,
        "#5880: 조각 회계가 사다리와 정합하면 쪽 밖 소실 줄이 없어야 한다"
    );
}

#[test]
fn issue_5880_lost_lines_render_on_their_pages() {
    let doc = load_doc();
    // 수정 전 clip 소실 대표 4줄 — 수정 후 2·5·6쪽 소속(0-기반 1·4·5).
    let expectations: [(u32, &str); 4] = [
        (1, "녹지, 공원 등 자연친화공간"),
        (4, "위탁 운영"),
        (5, "관리비 지원액"),
        (5, "라. 주택공급질서 확립 노력"),
    ];
    for (page, needle) in expectations {
        let text = doc
            .extract_page_text_native(page)
            .unwrap_or_else(|e| panic!("{}쪽 텍스트: {e}", page + 1));
        assert!(
            text.contains(needle),
            "#5880: {}쪽에 {needle:?} 가 있어야 한다\n--- {}쪽 ---\n{text}",
            page + 1,
            page + 1
        );
    }
}
