//! [#8060] 행의 보이는 칸 둘 이상이 각자 첫 되감김을 가지면 그 자리는 한/글이 저장한 쪽
//! 경계다. 그 경계를 닫는 줄의 줄간격은 이 조각에 예약하지 않는다.
//!
//! 입력 `samples/issue8060/71352_lpg_regulatory_impact.hwp`(규제영향분석서, HWP5 저장 조판)
//! 9쪽 6×3 표의 4행(「사후 평가관리」)은 `셀[12]` 문단 1 과 `셀[14]` 줄 1 이 모두 첫 줄
//! (1200HU) 뒤 `vpos=0` 이다. 정본 `pdf/issue8060/71352_lpg_regulatory_impact-2024.pdf`
//! (한/글 2024 13.0.0.564 Print)는 4행 첫 줄을 9쪽 바닥에 두고 나머지를 10쪽으로 넘긴다.
//!
//! #7870 이 위 중첩 표 위치를 한/글과 맞춘 뒤, 첫 줄 유닛(줄 16.0 + 줄간격 6.4)이 저장
//! 첫 조각 예산을 0.35px 넘어 4행 전체가 10쪽으로 갔다.
//!
//! 42쪽 7×4 표의 6행(「⑥ 차등화적용 여부」)은 `셀[20]` 1200HU · `셀[21]` 1100HU 뒤에 각각
//! `vpos=0` 으로 되감는다 — 쪽 경계는 하나지만 닫는 줄 크기가 칸마다 다르다. 한/글은 첫 줄을
//! 42쪽 바닥에 두는데, rhwp 는 첫 줄 유닛(16.0 + 줄간격 9.6)이 남은 16.3px 를 넘는다고 보고
//! 6행 전체를 43쪽으로 보냈다. 그 한 줄 만큼 43쪽 흐름이 저장 앵커보다 20px 아래로 밀려
//! 다음 6×3 표가 44쪽으로 이월됐고 문서가 49쪽이 됐다(정본 48쪽).
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;

fn compact_page_text(core: &DocumentCore, page: u32) -> String {
    core.extract_page_text_native(page)
        .unwrap_or_else(|e| panic!("{}쪽 텍스트 추출: {e}", page + 1))
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect()
}

#[test]
fn joint_stored_row_restart_keeps_the_first_line_on_the_closing_page() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8060/71352_lpg_regulatory_impact.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read")).expect("parse");
    let page9 = compact_page_text(&core, 8);
    let page10 = compact_page_text(&core, 9);
    // 4행 오른쪽 칸의 첫 줄(되감김 앞)과 둘째 줄(되감김 뒤).
    let first = "행정절차를규정한사항으로";
    let second = "신고시잔류가스에의한가스시설";
    assert!(
        page9.contains(first),
        "4행 첫 줄은 한/글처럼 9쪽 바닥에 남아야 한다\n--- 9쪽 ---\n{page9}"
    );
    assert!(
        !page10.contains(first),
        "4행 첫 줄이 10쪽에 중복되면 안 된다\n--- 10쪽 ---\n{page10}"
    );
    assert!(
        page10.contains(second) && !page9.contains(second),
        "되감김 뒤 줄은 10쪽에서 시작한다"
    );
}

#[test]
fn row_restarts_at_different_heights_still_close_the_page() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue8060/71352_lpg_regulatory_impact.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read")).expect("parse");
    assert_eq!(core.page_count(), 48, "한/글 2024 정본 쪽수");
    let page42 = compact_page_text(&core, 41);
    let page43 = compact_page_text(&core, 42);
    // 6행 오른쪽 칸 첫 줄(되감김 앞)과 그 뒤 줄, 그리고 다음 6×3 표의 첫 내용 줄.
    let first = "차등화대상결정";
    let second = "가스공급자의공급자의무미이행";
    let next_table = "가스공급자에대한보험금액할증";
    assert!(
        page42.contains(first) && !page43.contains(first),
        "6행 첫 줄은 한/글처럼 42쪽 바닥에 남아야 한다\n--- 42쪽 ---\n{page42}"
    );
    assert!(
        page43.contains(second) && !page42.contains(second),
        "되감김 뒤 줄은 43쪽에서 시작한다"
    );
    assert!(
        page43.contains(next_table),
        "다음 6×3 표는 한/글처럼 43쪽에 함께 놓인다\n--- 43쪽 ---\n{page43}"
    );
}
