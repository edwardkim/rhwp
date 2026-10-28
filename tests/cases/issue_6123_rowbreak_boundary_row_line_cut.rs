//! [Issue #6123] RowBreak 표의 쪽 경계 행을 줄 단위로 나누지 않고 통째로 앞 쪽에
//! 얹어 본문 하단을 174px 넘긴다 (3112461 별표 7, 7쪽).
//!
//! 근인: 저장 첫-조각 프레임(`common.height`)을 행 경계로 스냅하는
//! `nearest_saved_rowbreak_frame_row_end` 가 **거리 제한 없이** 최근접 행 끝을
//! 골랐다. 이 문서의 프레임 바닥(388.0px)은 행 1(측정 36.0~573.1)의 한복판인데
//! 행 끝(573.1)으로 스냅돼, "첫 조각은 행 1 까지" 라는 잘못된 신호가 되고
//! `source_first_fragment_overflow_allowance` 가 그 초과분(180.1px)을 통째로
//! 허용했다.
//!
//! 수정: 프레임 바닥이 **저장 좌표계**의 행 누적과 맞을 때만 행 경계로 읽는다.
//! 저장 누적(36 + 472 = 508)과 프레임(388)이 어긋나므로 이 문서는 일반 컷
//! 경로로 내려가 한글과 같은 줄 단위 분할을 얻는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue6123/3112461_railway_emc_criteria.hwpx";
/// 결함이 나타나는 쪽(0-based) — "3) 함체 포트" 표가 시작하는 7쪽.
const PAGE: u32 = 6;
/// 그 표의 문단 인덱스.
const TABLE_PARA: u64 = 118;

#[test]
fn issue_6123_boundary_row_is_cut_by_lines_not_carried_whole() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");

    // 한글 2020 오라클도 14쪽 — 컷이 쪽 수를 늘리지 않는다.
    assert_eq!(core.page_count(), 14, "한글 오라클과 같은 14쪽이어야 한다");

    let pages = core.dump_page_items_json(Some(PAGE));
    let page = pages
        .as_array()
        .and_then(|pages| pages.first())
        .expect("7쪽 항목");
    let body_height = page
        .pointer("/bodyArea/height")
        .and_then(|v| v.as_f64())
        .expect("본문 높이");
    let column = page.pointer("/columns/0").expect("단 0");
    let used = column
        .get("usedHeight")
        .and_then(|v| v.as_f64())
        .expect("단 소비 높이");

    // 결함 시 used=1189.2 > 1009.1 — 표 하단이 용지 밖으로 나갔다.
    assert!(
        used <= body_height + 1.0,
        "7쪽 소비 높이가 본문을 넘었다: used={used:.1}, body={body_height:.1}"
    );

    // 경계 행은 줄 단위로 갈려야 한다 — 결함 시 endCut 이 비어 있다(행 통짜).
    let partial = column
        .get("items")
        .and_then(|items| items.as_array())
        .and_then(|items| {
            items.iter().find(|item| {
                item.get("kind").and_then(|v| v.as_str()) == Some("partialTable")
                    && item.get("paraIndex").and_then(|v| v.as_u64()) == Some(TABLE_PARA)
            })
        })
        .expect("7쪽의 함체 포트 표 조각");
    let end_cut = partial
        .get("endCut")
        .and_then(|v| v.as_array())
        .expect("endCut");
    assert!(
        !end_cut.is_empty(),
        "경계 행이 줄 단위로 갈리지 않았다 — 행을 통째로 앞 쪽에 얹으면 넘친다"
    );

    // 8쪽은 그 행의 나머지부터 이어진다(같은 컷으로 시작).
    let next_pages = core.dump_page_items_json(Some(PAGE + 1));
    let next_partial = next_pages
        .as_array()
        .and_then(|pages| pages.first())
        .and_then(|page| page.pointer("/columns/0/items"))
        .and_then(|items| items.as_array())
        .and_then(|items| {
            items.iter().find(|item| {
                item.get("kind").and_then(|v| v.as_str()) == Some("partialTable")
                    && item.get("paraIndex").and_then(|v| v.as_u64()) == Some(TABLE_PARA)
            })
        })
        .expect("8쪽의 이어지는 표 조각");
    assert_eq!(
        next_partial.get("startCut"),
        partial.get("endCut"),
        "8쪽이 7쪽 컷 지점에서 이어져야 한다 — 어긋나면 그 사이 줄이 보이지 않는 곳에 그려진다"
    );
}
