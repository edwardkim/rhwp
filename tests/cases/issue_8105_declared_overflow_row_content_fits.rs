//! [#8105] 내용이 쪽 끝 공간에 다 드는 일반 행은 선언 높이가 넘쳐도 그 쪽에 둔다.
//!
//! 기준: 한/글 2020 Print `pdf/1613000-202200037-air-traffic-controller-cbta-2020.pdf`.
//! 188쪽 구역 2 표(78×3) 행 26 «6. 모니터링»: 칸 선언 2614HU(34.85px), 한 줄 내용. 쪽 끝에 남은
//! 29.9px 에 내용이 들어 한/글은 이 행을 188쪽에 두고 아래 괘선을 닫는 선(1040.6)에 긋는다. 189쪽은
//! 행 27 부터다. 수정 전 rhwp 는 선언 높이가 넘친다고 행 26 을 통째로 189쪽에 넘겨 189·190쪽이
//! 한 행씩 밀렸다(Native 49.7·40.1%).
//!
//! 비대상: 병합 칸이 닿는 행(기존 밴드 컷 경로)과 재조판 물리 프레임 행(기존 경로)은 이 경로를 타지
//! 않는다. 남는 선언 몫(4.9px)은 다음 조각이 행 칸 선언으로 다시 재며 1280HU 이하라 버린다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::document_core::DocumentCore;
use rhwp::renderer::render_tree::{RenderNode, RenderNodeType};

const SAMPLE: &str = "samples/issue6764/1613000-202200037-air-traffic-controller-cbta.hwp";

fn core() -> DocumentCore {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    DocumentCore::from_bytes(&bytes).unwrap_or_else(|error| panic!("parse {SAMPLE}: {error:?}"))
}

/// 표 `(구역, 문단, 개체)` 가 쪽 `page`(0-기준)에 그린 칸의 행 번호.
fn table_rows(core: &DocumentCore, page: u32, table: (usize, usize, usize)) -> Vec<u16> {
    fn walk(node: &RenderNode, inside: bool, table: (usize, usize, usize), out: &mut Vec<u16>) {
        let inside = match &node.node_type {
            RenderNodeType::Table(t) => {
                (t.section_index, t.para_index, t.control_index)
                    == (Some(table.0), Some(table.1), Some(table.2))
            }
            RenderNodeType::TableCell(cell) if inside => {
                out.push(cell.row);
                false
            }
            _ => inside,
        };
        for child in &node.children {
            walk(child, inside, table, out);
        }
    }
    let tree = core
        .build_page_render_tree(page)
        .unwrap_or_else(|error| panic!("{}쪽 렌더 트리: {error:?}", page + 1));
    let mut out = Vec::new();
    walk(&tree.root, false, table, &mut out);
    out
}

#[test]
fn row_whose_content_fits_stays_on_the_page_though_its_declared_height_overflows() {
    let core = core();
    let table = (2, 7, 0);
    let on_188 = table_rows(&core, 187, table);
    let on_189 = table_rows(&core, 188, table);
    assert!(
        on_188.contains(&26) && !on_189.contains(&26) && on_189.iter().min() == Some(&27),
        "188쪽이 행 26 «6. 모니터링» 을 담고 189쪽은 행 27 부터(정본): 188쪽 {on_188:?} / 189쪽 {on_189:?}"
    );
}
