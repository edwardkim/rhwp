//! Issue #6596: 본문 부동 그림의 바깥 여백이 잉크 위치에 반영되지 않아 여백만큼
//! 좌·상으로 어긋나던 결함의 가드.
//!
//! 한/글은 여백을 포함한 상자를 오프셋·정렬 자리에 놓고 잉크를 그 안쪽
//! (왼쪽 여백, 위 여백) 에 그린다. 종전 `layout_body_picture` 는 잉크 크기만으로
//! 자리를 잡고 그 원점에 바로 그려, 사방 3.01mm(11.36px) 여백 문서에서 그림이
//! (−11.3, −11.2)px 어긋났다 (`samples`↔`pdf` 215문서 실측, 여백>0 45건 중 44건).
//!
//! 세 표본이 규칙을 고정한다 (96DPI px, 한컴 PDF 이미지 bbox 대조):
//!
//! ```text
//! hwp3-sample5-hwp5   4쪽 pi=74  Square       Paper/Paper  왼쪽 정렬  → (62.85, 87.43)   종전 (51.6, 76.3)
//! hwp3-sample-hwp5    3쪽 pi=41  TopAndBottom Column/Para  왼쪽 정렬  → (124.73, 143.52) 종전 (113.4, 132.3)
//! [2027] 온새미로     35쪽 pi=7  Square       Column/Para  오른쪽 정렬 → (393.23, 165.9)  종전과 같음
//! ```
//!
//! 용지 기준과 단/문단 기준이 같은 규칙을 따른다. 오른쪽 정렬 표본은 여백이
//! left 2mm·right 0·top 0 이라 잉크가 움직이지 않아야 한다 — 여백을 상자 크기로
//! 정렬에 넣지 않고 원점에만 더하면 이 표본이 깨진다.

#![cfg(not(target_arch = "wasm32"))]

use std::fs;
use std::path::Path;

const TOLERANCE_PX: f64 = 1.5;

struct Case {
    sample: &'static str,
    page_index: u32,
    para_index: u64,
    expected: (f64, f64),
    defect: (f64, f64),
    note: &'static str,
}

const CASES: &[Case] = &[
    Case {
        sample: "samples/hwp3-sample5-hwp5.hwp",
        page_index: 3,
        para_index: 74,
        expected: (62.85, 87.43),
        defect: (51.6, 76.3),
        note: "Square·용지 기준·왼쪽 정렬·여백 3.01mm 사방 — 잉크가 (왼쪽, 위) 여백만큼 안쪽",
    },
    Case {
        sample: "samples/hwp3-sample-hwp5.hwp",
        page_index: 2,
        para_index: 41,
        expected: (124.73, 143.52),
        defect: (113.4, 132.3),
        note: "TopAndBottom·단 기준·문단 기준 세로·왼쪽 정렬·여백 3.01mm 사방 — Paper 기준과 같은 규칙",
    },
    Case {
        sample: "samples/[2027] 온새미로 1 본교재.hwp",
        page_index: 34,
        para_index: 7,
        expected: (393.23, 165.9),
        defect: (393.23, 165.9),
        note: "Square·단 기준·오른쪽 정렬·여백 left 2mm/right 0/top 0 — 상자 기준 정렬이라 잉크 불변",
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
fn float_picture_ink_sits_inside_its_outer_margin_box() {
    for case in CASES {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(case.sample);
        let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", case.sample));
        let document = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("parse {}: {e}", case.sample));
        let json = document
            .get_page_render_tree(case.page_index)
            .unwrap_or_else(|e| {
                panic!(
                    "{} 의 {}쪽 render tree: {e:?}",
                    case.sample,
                    case.page_index + 1
                )
            });
        let tree: serde_json::Value = serde_json::from_str(&json).expect("parse render tree json");

        let mut images = Vec::new();
        find_image(&tree, case.para_index, &mut images);
        assert_eq!(
            images.len(),
            1,
            "{} {}쪽 pi={} 그림은 하나여야 한다: {images:?}",
            case.sample,
            case.page_index + 1,
            case.para_index
        );
        let (x, y) = images[0];
        let (ex, ey) = case.expected;
        assert!(
            (x - ex).abs() < TOLERANCE_PX && (y - ey).abs() < TOLERANCE_PX,
            "{} {}쪽 pi={} 그림 ({x:.2}, {y:.2}) — 한컴 PDF 실측 ({ex}, {ey}) 이어야 한다. \
             결함 시 ({:.2}, {:.2}). {}",
            case.sample,
            case.page_index + 1,
            case.para_index,
            case.defect.0,
            case.defect.1,
            case.note
        );
    }
}
