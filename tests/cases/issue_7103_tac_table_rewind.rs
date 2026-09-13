//! [#7103] 같은 문단에 연속으로 붙은 글자처럼 취급(TAC) 표가 저장 LINE_SEG 간격을
//! 음수로 되감아 앞 표 위로 겹치던 회귀를 막는다.
//!
//! 문제 문서는 첫 문단에 TAC 표 둘을 연속으로 둔다. 첫 표는 신청서 제목 블록이고,
//! 둘째 표는 개인정보 수집/이용 동의 블록이다. 저장된 다음 LINE_SEG 의 `vertical_pos` 가
//! 이전 줄의 끝보다 작아 보일 때 그 차이를 그대로 더하면 y_offset 이 위로 되감겨 둘째 표가
//! 제목 위에 겹친다. 이 테스트는 둘째 표가 첫 표의 아래에서 시작하는지만 좁게 확인한다.

#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "tests/fixtures/issue_7103/ari-tutoring-application.hwp";
const PAGE: u32 = 0;
const HOST_PARA: usize = 0;
const TITLE_TABLE_CONTROL: usize = 3;
const CONSENT_TABLE_CONTROL: usize = 5;

#[derive(Clone, Copy, Debug)]
struct TableBox {
    control_index: usize,
    y: f64,
    bottom: f64,
}

fn load() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    DocumentCore::from_bytes(&std::fs::read(&path).expect("read #7103 fixture"))
        .unwrap_or_else(|error| panic!("open {}: {error}", path.display()))
}

fn collect_host_tables(node: &RenderNode, in_column: bool, out: &mut Vec<TableBox>) {
    if in_column {
        if let RenderNodeType::Table(table) = &node.node_type {
            if table.para_index == Some(HOST_PARA) {
                if let Some(control_index) = table.control_index {
                    if [TITLE_TABLE_CONTROL, CONSENT_TABLE_CONTROL].contains(&control_index) {
                        out.push(TableBox {
                            control_index,
                            y: node.bbox.y,
                            bottom: node.bbox.y + node.bbox.height,
                        });
                    }
                }
            }
            return;
        }
    }

    let in_column = in_column || matches!(node.node_type, RenderNodeType::Column(_));
    for child in &node.children {
        collect_host_tables(child, in_column, out);
    }
}

#[test]
fn consecutive_tac_tables_do_not_rewind_to_the_previous_line_segment() {
    let core = load();
    let page = core
        .build_page_render_tree(PAGE)
        .expect("render #7103 page 1");
    let mut tables = Vec::new();
    collect_host_tables(&page.root, false, &mut tables);
    tables.sort_by(|a, b| a.control_index.cmp(&b.control_index));

    let title = tables
        .iter()
        .find(|table| table.control_index == TITLE_TABLE_CONTROL)
        .unwrap_or_else(|| {
            panic!("제목 TAC 표(ci={TITLE_TABLE_CONTROL})를 찾을 수 없음: {tables:?}")
        });
    let consent = tables
        .iter()
        .find(|table| table.control_index == CONSENT_TABLE_CONTROL)
        .unwrap_or_else(|| {
            panic!("개인정보 TAC 표(ci={CONSENT_TABLE_CONTROL})를 찾을 수 없음: {tables:?}")
        });

    assert!(
        consent.y + 0.5 >= title.bottom,
        "#7103: 둘째 TAC 표는 제목 표 아래에서 시작해야 한다. \
         title={title:?}, consent={consent:?}"
    );
    assert!(
        consent.y - title.bottom <= 2.0,
        "#7103: 둘째 TAC 표는 제목 표 직후로 이어져야 한다. \
         title={title:?}, consent={consent:?}"
    );
}
