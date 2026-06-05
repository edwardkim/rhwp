//! 문단 기준(vert=Para) 위아래(TopAndBottom) 비-TAC 표는 흐름 안에서 앵커
//! 문단 위치에 inline 으로 흘러야 한다(한컴 viewer 정합). 이런 표의 앵커
//! LINE_SEG 는 vertical_pos=0 으로 인코딩되는데, typeset.rs 의 cross-paragraph
//! vpos-reset 가드가 이를 페이지 reset 신호로 오인해 표를 통째로 다음 페이지로
//! 밀어내 본문에 큰 공백을 만들던 결함의 회귀 테스트.
//!
//! 영향 샘플:
//! - `samples/basic/service_contract_float_table.hwp` (용역계약서_예시):
//!   14 문단(제목, 제1~11조 + 비용 표 1개). 표는 제4조 직후 흐름에 inline 배치
//!   되어 전체가 1 페이지에 들어가야 한다. 결함 시 2 페이지(표가 page 2 로 밀림).

use std::fs;
use std::path::Path;

fn load_page_count(rel_path: &str) -> u32 {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path = Path::new(repo_root).join(rel_path);
    let bytes = fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", rel_path, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", rel_path, e));
    doc.page_count()
}

#[test]
fn float_topbottom_para_table_flows_inline_single_page() {
    let pages = load_page_count("samples/basic/service_contract_float_table.hwp");
    assert_eq!(
        pages, 1,
        "용역계약서_예시.hwp 는 1 페이지여야 함 (비-TAC TopAndBottom vert=Para 표가 \
         제4조 직후 흐름에 inline 배치). 결함 시 2 페이지: 표 앵커 LINE_SEG vpos=0 이 \
         cross-paragraph vpos-reset 로 오인되어 표가 통째로 page 2 로 밀림"
    );
}
