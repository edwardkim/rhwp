//! Issue #1789: exclusion 겹침 프로브는 line_spacing 을 제외한 잉크/줄 높이로 판정해야 한다.
//!
//! HWPX 경로 전용 `overlaps_zone` 프로브(layout.rs, task 1510 도입)가
//! `line_height + line_spacing` 으로 겹침을 판정하면, 잉크는 문단 기준 자리차지 표 위
//! 공간에 들어가는 줄이 spacing 포함분 수 px 겹침만으로 표 아래로 밀린다.
//!
//! Regression shape (samples/task1789/exclusion_probe_line_spacing.hwpx, 36385142):
//! - 문단 0.8("다. 위원구성…") 저장 lineseg vpos=34925 → y≈529.9px (표 위 유지)
//! - zone top ≈ 552.7px, 잉크 하단 545.9px(겹침 없음), lh+ls 555.5px(2.8px 겹침)
//! - 수정 전: 문단이 zone.bottom(≈875px)으로 밀려 345px 변위. HWP5 재파스 렌더와
//!   저장 lineseg 는 529.9px 로 일치 (한컴 정답).

use std::fs;
use std::path::Path;

const SAMPLE: &str = "samples/task1789/exclusion_probe_line_spacing.hwpx";

#[test]
fn issue_1789_line_above_para_float_table_stays_at_saved_vpos() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = fs::read(&path).unwrap_or_else(|e| panic!("read {}: {}", SAMPLE, e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {}: {}", SAMPLE, e));
    let tree = doc
        .build_page_render_tree(0)
        .unwrap_or_else(|e| panic!("render tree: {:?}", e));
    let json: serde_json::Value =
        serde_json::from_str(&tree.root.to_json()).expect("parse tree json");

    // 본문(표/글상자 외부) TextLine y 수집
    fn collect(v: &serde_json::Value, in_container: bool, out: &mut Vec<f64>) {
        if let Some(o) = v.as_object() {
            let ty = o.get("type").and_then(|t| t.as_str()).unwrap_or("");
            let next_container =
                in_container || matches!(ty, "Table" | "Rect" | "TextBox" | "Header" | "Footer");
            if ty == "TextLine" && !next_container {
                if let Some(b) = o.get("bbox") {
                    if let Some(y) = b.get("y").and_then(|y| y.as_f64()) {
                        out.push(y);
                    }
                }
            }
            for c in o.values() {
                collect(c, next_container, out);
            }
        } else if let Some(a) = v.as_array() {
            for c in a {
                collect(c, in_container, out);
            }
        }
    }
    let mut ys = Vec::new();
    collect(&json, false, &mut ys);

    // 문단 0.8 첫 줄: 저장 lineseg 위치(≈529.9px)에 있어야 한다.
    assert!(
        ys.iter().any(|y| (y - 529.9).abs() < 2.0),
        "표 위 공간에 들어가는 줄(저장 vpos=34925 → ≈529.9px)이 없다 — exclusion 프로브가 \
         line_spacing 포함 높이로 과대 판정하여 표 아래로 밀었을 가능성. body TextLine y: {:?}",
        ys.iter()
            .map(|y| (y * 10.0).round() / 10.0)
            .collect::<Vec<_>>()
    );
}
