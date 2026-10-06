//! Issue #6603: 글자처럼(TAC) 그림의 바깥 여백이 줄 안 잉크 위치에 반영되지 않아
//! 여백만큼 좌·상으로 어긋나던 결함의 가드 (#6596 의 줄 안 형제).
//!
//! 한/글은 여백을 포함한 상자를 줄 안에 놓고 잉크를 상자의 (왼쪽 여백, 위 여백)
//! 안쪽에 그린다. 종전 줄 안 배치 경로(`tac_offsets_px` 폭·`make_picture_image_node`
//! 세 자리·`layout_shape_item` fallback)는 여백을 어디에도 넣지 않았다.
//!
//! `samples/hwp3-sample14-hwp5.hwp` (한컴 PDF `pdf/hwp3-sample14-hwp5-2022.pdf` 대조,
//! 96DPI px). 빈 문단의 TAC 그림, 바깥 여백 3.01mm(11.36px) 사방:
//!
//! ```text
//! 2쪽 pi=16  문단 양쪽 정렬   → (124.73, 143.52)   종전 (113.4, 132.3)
//! 3쪽 pi=29  문단 가운데 정렬 → (263.22, 143.52)   종전 (263.3, 132.3)
//! ```
//!
//! 가운데 정렬은 좌우 여백이 같아 x 가 그대로이고 y 만 위 여백만큼 내려간다 —
//! 상자 폭으로 가운데를 잡지 않고 잉크 원점에만 여백을 더하면 x 가 11.36 틀어진다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/hwp3-sample14-hwp5.hwp";
const TOLERANCE_PX: f64 = 1.5;

struct Case {
    page_index: u32,
    para_index: u64,
    expected: (f64, f64),
    defect: (f64, f64),
    note: &'static str,
}

const CASES: &[Case] = &[
    Case {
        page_index: 1,
        para_index: 16,
        expected: (124.73, 143.52),
        defect: (113.4, 132.3),
        note: "양쪽 정렬 빈 문단 — 잉크가 (왼쪽, 위) 여백만큼 안쪽",
    },
    Case {
        page_index: 2,
        para_index: 29,
        expected: (263.22, 143.52),
        defect: (263.3, 132.3),
        note: "가운데 정렬 빈 문단 — 상자를 가운데 놓으므로 x 는 그대로, y 만 위 여백만큼",
    },
];

fn find_image(node: &serde_json::Value, para_index: u64, out: &mut Vec<(f64, f64)>) {
    if node.get("type").and_then(|t| t.as_str()) == Some("Image")
        && node.get("pi").and_then(|v| v.as_u64()) == Some(para_index)
    {
        let bbox = &node["bbox"];
        let get = |k: &str| bbox.get(k).and_then(|v| v.as_f64()).unwrap_or(f64::NAN);
        out.push((get("x"), get("y")));
    }
    for child in node
        .get("children")
        .and_then(|c| c.as_array())
        .into_iter()
        .flatten()
    {
        find_image(child, para_index, out);
    }
}

#[test]
fn tac_picture_ink_sits_inside_its_outer_margin_box() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {SAMPLE}: {e}"));
    for case in CASES {
        let json = document
            .get_page_render_tree(case.page_index)
            .unwrap_or_else(|e| panic!("{}쪽 render tree: {e:?}", case.page_index + 1));
        let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

        let mut images = Vec::new();
        find_image(&tree, case.para_index, &mut images);
        assert_eq!(
            images.len(),
            1,
            "{}쪽 pi={} 그림은 하나여야 한다: {images:?}",
            case.page_index + 1,
            case.para_index
        );
        let (x, y) = images[0];
        let (ex, ey) = case.expected;
        assert!(
            (x - ex).abs() < TOLERANCE_PX && (y - ey).abs() < TOLERANCE_PX,
            "{}쪽 pi={} 그림 ({x:.2}, {y:.2}) — 한컴 PDF 실측 ({ex}, {ey}) 이어야 한다. \
             결함 시 ({:.2}, {:.2}). {}",
            case.page_index + 1,
            case.para_index,
            case.defect.0,
            case.defect.1,
            case.note
        );
    }
}
