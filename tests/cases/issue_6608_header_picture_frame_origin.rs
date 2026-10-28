//! Issue #6608: 머리말에 직접 놓인 용지 기준(`PAPER`) 부동 그림이 물리 용지 (0, 0) 에서
//! 오프셋을 재 쪽 여백만큼 좌·상으로 어긋나던 결함의 가드.
//!
//! 한/글은 머리말·꼬리말 안 개체의 오프셋을 그 틀(머리말 영역)의 원점에서 잰다.
//! `samples/pic-in-head-02.hwp` (여백 좌 20mm·상 10mm) 머리말 그림
//! `horzRelTo=PAPER vertRelTo=PAPER offset=(245, 1066)HU`:
//!
//! ```text
//! 한컴 PDF (pdf/pic-in-head-02-2022.pdf) 6쪽 전부   (78.68, 51.94)   = 여백 (75.6, 37.8) + 오프셋
//! 종전 rhwp                                       (3.3, 14.2)      = 용지 (0, 0) + 오프셋
//! ```

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/pic-in-head-02.hwp";
const TOLERANCE_PX: f64 = 1.5;
/// 한컴 PDF 실측 (96DPI px). 결함 시 (3.3, 14.2).
const EXPECTED: (f64, f64) = (78.68, 51.94);
const HEADER_PICTURE_WIDTH: f64 = 635.2;

fn collect_header_images(
    node: &serde_json::Value,
    in_header: bool,
    out: &mut Vec<(f64, f64, f64)>,
) {
    let ty = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    if ty == "Image" && in_header {
        let bbox = &node["bbox"];
        let get = |k: &str| bbox.get(k).and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
        out.push((get("x"), get("y"), get("w")));
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        collect_header_images(child, in_header || ty == "Header", out);
    }
}

#[test]
fn header_paper_relative_picture_measures_its_offset_from_the_header_frame() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));
    for page_index in [0u32, 1, 2] {
        let json = document
            .get_page_render_tree(page_index)
            .unwrap_or_else(|e| panic!("{}쪽 render tree: {e:?}", page_index + 1));
        let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

        let mut images = Vec::new();
        collect_header_images(&tree, false, &mut images);
        let (x, y, _) = images
            .iter()
            .copied()
            .find(|(_, _, w)| (w - HEADER_PICTURE_WIDTH).abs() < 1.0)
            .unwrap_or_else(|| {
                panic!(
                    "{}쪽 머리말에 폭 {HEADER_PICTURE_WIDTH}px 그림: {images:?}",
                    page_index + 1
                )
            });
        assert!(
            (x - EXPECTED.0).abs() < TOLERANCE_PX && (y - EXPECTED.1).abs() < TOLERANCE_PX,
            "{}쪽 머리말 그림 ({x:.2}, {y:.2}) — 한컴 PDF 실측 {EXPECTED:?} 이어야 한다. \
             결함 시 (3.3, 14.2): 용지 (0, 0) 기준 오프셋",
            page_index + 1
        );
    }
}
