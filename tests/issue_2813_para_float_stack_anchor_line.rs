//! Issue #2813: para-relative TopAndBottom float 스택이 host 앵커 줄 기준
//! 하단 배치되어 상단 공백 + 겹침 + +1 과분할.
//!
//! `samples/issue2813/dangjik_dutylog.hwpx` (당직근무일지 서식): 공백-only host
//! 문단 pi=0 이 자리차지 표 2개(결재표 6×13, 당직근무자 21×12)를 anchor 하고,
//! 저장 앵커 줄(lineseg)은 스택 하단 이후(vpos 50005 HU = 666.7px)를 인코딩한다.
//! 한글은 표들을 본문 상단부터 문서순으로 흘리고 앵커 줄이 그 뒤에 razor-fit
//! (666.7 + 13.3 = 680.0px ≤ 본문 680.3px)으로 1쪽에 담긴다 — 총 2쪽.
//!
//! 회귀 (수정 전 버그):
//! - 셀 실측 팽창으로 naive fit(697.8px)이 실패하자 co-anchored orphan 가드가
//!   둘째 표를 통째 이월 → 3쪽(+1 과분할).
//! - 렌더는 표들을 앵커 줄에 하단정렬로 매달아 상단 2/3 공백 + 표 겹침.
//!
//! 정정: host 의 유일한 저장 앵커 줄이 스택 아래·본문 안을 인코딩하면(스택
//! 형상 한정, 단일 float 는 분할 의미론 유지 — issue #1488 18쪽 가드) 통째
//! 배치하고, 앵커 줄 아이템을 표들 뒤로 이연해 한글 문서순으로 렌더한다.

use std::fs;
use std::path::Path;

#[test]
fn issue_2813_stack_stays_on_first_page_with_trailing_anchor_line() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwpx_path = Path::new(repo_root).join("samples/issue2813/dangjik_dutylog.hwpx");
    let bytes =
        fs::read(&hwpx_path).unwrap_or_else(|e| panic!("read {}: {}", hwpx_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("parse dangjik_dutylog.hwpx");

    // 한글 COM PageCount = 2 (저장 PDF 동일). 수정 전 3쪽.
    assert_eq!(
        doc.page_count(),
        2,
        "para-float 스택 +1 과분할 회귀 — 페이지 수가 2가 아님 (수정 전 3)"
    );

    // 앵커 줄(PartialParagraph) 아이템은 같은 쪽에서 두 표 뒤에 와야 한다
    // (한글 문서순: 표→표→줄). 수정 전에는 줄이 표들 앞에 있어 렌더가 표를
    // 줄-이후 흐름에 배치, 하단정렬 겹침이 났다.
    let dump = doc.dump_page_items(None);
    let first_page: Vec<&str> = dump
        .lines()
        .take_while(|l| !l.contains("페이지 2") && !l.contains("page_num=2"))
        .collect();
    let table_lines: Vec<usize> = first_page
        .iter()
        .enumerate()
        .filter(|(_, l)| l.contains("Table") && l.contains("pi=0"))
        .map(|(i, _)| i)
        .collect();
    let line_item = first_page
        .iter()
        .position(|l| l.contains("PartialParagraph") && l.contains("pi=0"));
    assert_eq!(table_lines.len(), 2, "1쪽에 pi=0 표 2개가 있어야 함");
    let line_idx = line_item.expect("1쪽에 pi=0 앵커 줄 PartialParagraph가 있어야 함");
    assert!(
        table_lines.iter().all(|&t| t < line_idx),
        "앵커 줄 아이템이 표들 뒤(문서순)에 와야 함 — 표 {:?} vs 줄 {}",
        table_lines,
        line_idx
    );
}
