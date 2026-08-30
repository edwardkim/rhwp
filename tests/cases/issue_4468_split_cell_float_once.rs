//! [Issue #4468] 쪽 경계에 걸친 표 셀의 TopAndBottom floating 그림이
//! 앞쪽 하단과 다음 쪽 상단에 중복 출력된다.
//!
//! 한글은 그 칸 조각을 소유한 쪽에만 그림을 그린다. rHWP 는 TopAndBottom
//! atomic cell unit 에 control identity 가 없어, 같은 문단의 빈 범위 unit 이
//! 있는 모든 조각에서 그림을 다시 emit 했다.
//!
//! 픽스처 `samples/issue5734/cell_float_stack_stored_vpos.hwpx` 는 #6045 와
//! 같은 156684746 표 8 축소본이다. 오른쪽 칸 서울경제TV 캡처(h=289.2px)가
//! 쪽을 건넌다. 결함 시 같은 control 이 1쪽·2쪽에 모두 나타난다.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;
use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5734/cell_float_stack_stored_vpos.hwpx";
const TV_H: f64 = 289.2;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CellFloatId {
    sec: i64,
    para: i64,
    ctrl: i64,
    cell: i64,
    cell_para: i64,
}

fn cell_tb_images(layout_json: &str) -> Vec<(CellFloatId, f64)> {
    let value: serde_json::Value = serde_json::from_str(layout_json).expect("control layout JSON");
    let items = value
        .as_array()
        .or_else(|| value.get("controls").and_then(|v| v.as_array()))
        .expect("control layout 배열");
    let mut out = Vec::new();
    for item in items {
        if item.get("type").and_then(|v| v.as_str()) != Some("image") {
            continue;
        }
        if item.get("wrap").and_then(|v| v.as_str()) != Some("topAndBottom") {
            continue;
        }
        let Some(cell) = item.get("cellIdx").and_then(|v| v.as_i64()) else {
            continue;
        };
        let Some(cell_para) = item.get("cellParaIdx").and_then(|v| v.as_i64()) else {
            continue;
        };
        let id = CellFloatId {
            sec: item.get("secIdx").and_then(|v| v.as_i64()).unwrap_or(-1),
            para: item.get("paraIdx").and_then(|v| v.as_i64()).unwrap_or(-1),
            ctrl: item
                .get("controlIdx")
                .and_then(|v| v.as_i64())
                .unwrap_or(-1),
            cell,
            cell_para,
        };
        let h = item.get("h").and_then(|v| v.as_f64()).unwrap_or(0.0);
        out.push((id, h));
    }
    out
}

#[test]
fn issue_4468_split_cell_tb_float_paints_once() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page_count = core.page_count();
    assert!(
        page_count >= 2,
        "표 8 축소본은 2쪽이어야 한다: {page_count}"
    );

    let mut pages_by_id: HashMap<CellFloatId, Vec<usize>> = HashMap::new();
    let mut tv_pages = Vec::new();
    for page in 0..page_count {
        let layout = core
            .get_page_control_layout_native(page as u32)
            .unwrap_or_else(|e| panic!("page {} control layout: {e}", page + 1));
        for (id, h) in cell_tb_images(&layout) {
            if (h - TV_H).abs() < 0.6 {
                tv_pages.push(page);
            }
            pages_by_id.entry(id).or_default().push(page);
        }
    }

    assert!(
        !pages_by_id.is_empty(),
        "셀 TopAndBottom 그림이 한 장도 없다"
    );
    for (id, pages) in &pages_by_id {
        let unique: Vec<usize> = {
            let mut p = pages.clone();
            p.sort_unstable();
            p.dedup();
            p
        };
        assert_eq!(
            unique.len(),
            1,
            "셀 TopAndBottom 그림이 여러 쪽에 중복 출력된다: {:?} pages={:?} (결함 시 앞쪽 하단+다음 쪽 상단)",
            id,
            unique.iter().map(|p| p + 1).collect::<Vec<_>>()
        );
    }

    assert!(
        !tv_pages.is_empty(),
        "서울경제TV 캡처(h=289.2)가 어느 쪽에도 없다"
    );
    tv_pages.sort_unstable();
    tv_pages.dedup();
    assert_eq!(
        tv_pages.len(),
        1,
        "서울경제TV 캡처가 여러 쪽에 중복 출력된다: {:?}",
        tv_pages.iter().map(|p| p + 1).collect::<Vec<_>>()
    );
    assert!(
        tv_pages[0] >= 1,
        "캡처는 다음 쪽에 한 번만 있어야 한다: page={}",
        tv_pages[0] + 1
    );
}
