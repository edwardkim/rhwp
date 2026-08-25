//! [Issue #6045] 쪽 분할 표의 TopAndBottom 셀 그림이 잔여 쪽에 강제 소비되어
//! 지면 밖으로 잘리고, 다음 쪽 칸은 빈 칸이 된다.
//!
//! `samples/issue5734/cell_float_stack_stored_vpos.hwpx` 는 156684746 9·10쪽
//! 〈표 8〉 축소본이다. 오른쪽 마지막 칸 그림은 한글이 다음 쪽에 두는
//! 서울경제TV 캡처(h=289.2px). 결함 시 1쪽 `y=991 bottom=1280` 으로 지면
//! (1122.5) 밖에 잘리고 2쪽 오른쪽 칸에는 `<image>` 가 없다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;

const SAMPLE: &str = "samples/issue5734/cell_float_stack_stored_vpos.hwpx";
const PAGE_H: f64 = 1122.5;
const TV_H: f64 = 289.2;

fn image_rects(svg: &str) -> Vec<(f64, f64, f64, f64)> {
    let mut out = Vec::new();
    for cap in svg.split("<image ").skip(1) {
        let head = &cap[..cap.find('>').unwrap_or(cap.len())];
        let attr = |name: &str| -> Option<f64> {
            let key = format!("{name}=\"");
            let s = head.find(&key)? + key.len();
            let e = s + head[s..].find('"')?;
            head[s..e].parse().ok()
        };
        if let (Some(x), Some(y), Some(w), Some(h)) =
            (attr("x"), attr("y"), attr("width"), attr("height"))
        {
            out.push((x, y, w, h));
        }
    }
    out
}

#[test]
fn issue_6045_split_cell_tb_picture_stays_on_canvas() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("read sample")).expect("open");
    let page_count = core.page_count();
    assert!(
        page_count >= 2,
        "표 8 축소본은 2쪽이어야 한다: {page_count}"
    );

    let mut tv = None;
    for page in 0..page_count {
        let svg = core
            .render_page_svg_native(page)
            .unwrap_or_else(|e| panic!("page {} svg: {e}", page + 1));
        for (x, y, _w, h) in image_rects(&svg) {
            if (h - TV_H).abs() < 0.6 {
                tv = Some((page, x, y, h));
                assert!(
                    y + h <= PAGE_H + 0.5,
                    "서울경제TV 캡처가 {}쪽 지면 밖으로 잘린다: y={y:.1} h={h:.1} (결함 시 1쪽 y=991 bottom=1280)",
                    page + 1
                );
            }
        }
    }

    let (page, x, y, h) = tv.expect("서울경제TV 캡처(h=289.2)가 어느 쪽에도 없다");
    assert!(
        page >= 1,
        "캡처는 다음 쪽에 있어야 한다 (결함 시 1쪽 y=991): page={} y={y:.1}",
        page + 1
    );
    assert!(
        x > 300.0,
        "캡처는 오른쪽 칸이어야 한다: x={x:.1} y={y:.1} h={h:.1}"
    );
}
