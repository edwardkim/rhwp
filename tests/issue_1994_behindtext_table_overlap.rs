//! Issue #1994: 절대위치 글뒤로(BehindText) 용지-앵커 표가 본문 위에 겹쳐 렌더.
//!
//! `samples/basic/issue1994_behindtext_table_20200830.hwp` (가로 2단 교회 주보) 3페이지는
//! 글뒤로(wrap=BehindText) + 용지(Paper)-앵커 1×1 표 2개로 구성:
//! - pi=33 (용지 13.5mm, 쪽나눔 None): 교역자 명단
//! - pi=34 (용지 134.1mm, 쪽나눔 RowBreak): 예배 스케줄
//!
//! 회귀 (수정 전 버그):
//! - pi=34 가 RowBreak 흐름 분할 경로로 빠져 절대 Y(134mm)를 무시하고 흐름 상단에 2단 분할
//!   배치 → pi=33(교역자 명단) 위에 겹쳐 판독 불가. 페이지 수도 rhwp 5쪽 vs 한글 4쪽.
//!
//! 정정: 용지-앵커 글뒤로/글앞으로 표는 RowBreak 여부와 무관하게 절대 좌표에 통째 배치
//! (typeset.rs is_paper_floating_block). 한글 2022 PDF(issue_1994.pdf) = 4페이지 정합.

use std::fs;
use std::path::Path;

#[test]
fn issue_1994_behindtext_paper_table_not_overlapped() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue1994_behindtext_table_20200830.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue1994_behindtext_table_20200830.hwp");

    // 한글 2022 = 4페이지. 수정 전 rhwp = 5페이지(pi=34 흐름 분할로 페이지 팽창).
    assert_eq!(
        doc.page_count(),
        4,
        "글뒤로 용지앵커 표 흐름분할 회귀 — 페이지 수가 4가 아님 (수정 전 5)"
    );

    // pi=34 (글뒤로 용지앵커 RowBreak 표)는 흐름 분할(PartialTable)이 아니라 통째 Table 로
    // 절대 배치되어야 한다. PartialTable 로 분할되면 pi=33 위에 겹치는 회귀.
    let dump = doc.dump_page_items(None);
    let pi34_partial = dump
        .lines()
        .filter(|l| l.contains("PartialTable") && l.contains("pi=34"))
        .count();
    assert_eq!(
        pi34_partial, 0,
        "pi=34 글뒤로 용지앵커 표가 PartialTable 로 흐름 분할됨 — 절대배치 회귀"
    );
}
