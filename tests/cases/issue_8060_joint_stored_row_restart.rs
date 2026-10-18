//! [#8060] 행의 보이는 칸이 모두 같은 저장 높이에서 되감으면 그 자리는 한/글이 저장한 쪽
//! 경계다. 그 경계를 닫는 줄의 줄간격은 이 조각에 예약하지 않는다.
//!
//! 입력 `samples/issue8060/71352_lpg_regulatory_impact.hwp`(규제영향분석서, HWP5 저장 조판)
//! 9쪽 6×3 표의 4행(「사후 평가관리」)은 `셀[12]` 문단 1 과 `셀[14]` 줄 1 이 모두 첫 줄
//! (1200HU) 뒤 `vpos=0` 이다. 정본 `pdf/issue8060/71352_lpg_regulatory_impact-2024.pdf`
//! (한/글 2024 13.0.0.564 Print)는 4행 첫 줄을 9쪽 바닥에 두고 나머지를 10쪽으로 넘긴다.
//!
//! #7870 이 위 중첩 표 위치를 한/글과 맞춘 뒤, 첫 줄 유닛(줄 16.0 + 줄간격 6.4)이 저장
//! 첫 조각 예산을 0.35px 넘어 4행 전체가 10쪽으로 갔다.
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
    // 전체 쪽수(rhwp 49 · 정본 48)는 뒤쪽의 별개 차이라 여기서 보지 않는다. 9·10쪽은
    // 정본과 같은 내용으로 시작한다(Visual Sweep 99.3% / 99.5%).
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
