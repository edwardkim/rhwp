//! 관계형·고정형 — [#8986] Print 에서 표 괘선과 그 괘선을 감싼 칸 clip 은 같은 물리 프레임을 쓰고, 장치 clip 은 세대와 무관하다.
//! 주석 갱신: 2026-11-12 19:00 KST (새 파일)
//!
//! 입력: 공개 결재문서 36325899(한컴오피스 2020 저장 HWPX) — `samples/issue8986/36325899-nested-table-top-border.hwpx`.
//! 기준: 한컴 engine 2020 Print PDF(PrintMethod 0) `pdf/issue8986/36325899-2020.pdf`, 2쪽.
//! - 고정형: 전체 2쪽.
//! - 관계형: 2쪽 「발신명의」 표(1×1 바깥 표 칸 맨 위의 중첩 표)의 괘선 중심은 자신을 감싼 칸 clip
//!   안에 있다. Print 는 표 괘선을 물리 종이 프레임으로 옮기므로(`publish_table_border_geometry`)
//!   옮긴 괘선을 담은 칸 clip 도 그 프레임을 포함해야 한다. 한컴 PDF 는 clip 위끝과 위 괘선이 같은
//!   135.212pt 다. 또 Print raster 는 세대와 무관하게 장치 clip 을 쓴다 — PDFium 이 한컴 PDF 의 축
//!   정렬 clip 을 바깥 화소로 넓혀 경계에 놓인 괘선이 남는다.
//!
//! 측정(2026-11-12, 코드 head e3427b7ba1, base = PR #8990 head 20e8ebacbe,
//! `scripts/visual_sweep.py --silhouette-only --dpi 96`, raw = `silhouette_raw_match_percent`,
//! `RHWP_FONT_PATH` 글꼴 공급, `--compat 2022`):
//! - 36325899 Native 2쪽 / 2쪽: p1 97.37 (base 97.37), p2 100.00 (base 83.95).
//! - 36325899 fresh WASM 2쪽 / 2쪽: p1 99.92, p2 100.00.
//! - 두 경로 전쪽 최저: Native 97.37, WASM 99.92.
//! - 증거: `mydocs/pr/assets/issue_8986/`
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::document_core::DocumentCore;
use rhwp::paint::RenderProfile;
use serde_json::Value;

const SAMPLE: &str = "samples/issue8986/36325899-nested-table-top-border.hwpx";

fn core() -> DocumentCore {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {SAMPLE}: {e}"));
    DocumentCore::from_bytes(&bytes).expect("open")
}

#[derive(Clone, Copy, Debug)]
struct Rect {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn rect(v: &Value) -> Rect {
    Rect {
        x: v["x"].as_f64().unwrap(),
        y: v["y"].as_f64().unwrap(),
        w: v["width"].as_f64().unwrap(),
        h: v["height"].as_f64().unwrap(),
    }
}

/// (괘선 중심 (x1,y1,x2,y2), 감싼 칸 clip 목록)
type Stroke = ((f64, f64, f64, f64), Vec<Rect>);

fn table_strokes(node: &Value, clips: &mut Vec<Rect>, out: &mut Vec<Stroke>) {
    match node {
        Value::Object(map) => {
            let cell_clip = map.get("kind").and_then(Value::as_str) == Some("clipRect")
                && map.get("clipKind").and_then(Value::as_str) == Some("tableCell");
            if cell_clip {
                clips.push(rect(&map["clip"]));
            }
            if map.get("type").and_then(Value::as_str) == Some("line") {
                let f = |k: &str| map[k].as_f64().unwrap();
                out.push(((f("x1"), f("y1"), f("x2"), f("y2")), clips.clone()));
            }
            for child in map.values() {
                table_strokes(child, clips, out);
            }
            if cell_clip {
                clips.pop();
            }
        }
        Value::Array(items) => items
            .iter()
            .for_each(|item| table_strokes(item, clips, out)),
        _ => {}
    }
}

#[test]
fn page_count_matches_hancom_print() {
    assert_eq!(core().page_count(), 2, "한컴 Print 2쪽");
}

/// 2쪽 「발신명의」 표는 1×1 바깥 표 칸 맨 위에 놓인 중첩 표다. 위 괘선은 바깥 칸 위끝과
/// 같은 좌표다(한컴 PDF 도 clip 위끝과 선이 135.212pt 로 같다). Print 는 괘선을 물리 종이
/// 프레임으로 옮기므로, 감싼 칸 clip 도 그 프레임을 담아야 괘선 중심이 clip 안에 남는다.
#[test]
fn nested_table_border_centers_stay_inside_enclosing_cell_clips() {
    let core = core();
    let json: Value = serde_json::from_str(
        &core
            .get_page_layer_tree_with_profile_native(1, RenderProfile::Print)
            .expect("layer tree"),
    )
    .expect("json");
    let mut strokes = Vec::new();
    table_strokes(&json["root"], &mut Vec::new(), &mut strokes);
    let nested: Vec<_> = strokes
        .iter()
        .filter(|(_, clips)| !clips.is_empty())
        .collect();
    assert_eq!(
        nested.len(),
        5,
        "「발신명의」 표 괘선 5개(가로 2·세로 3): {strokes:?}"
    );
    for ((x1, y1, x2, y2), clips) in nested {
        let (cx, cy) = ((x1 + x2) / 2.0, (y1 + y2) / 2.0);
        for clip in clips {
            let (top, bottom) = (clip.y, clip.y + clip.h);
            let ys = [*y1, *y2];
            assert!(
                ys.iter().all(|y| *y >= top && *y <= bottom),
                "괘선 y {y1:.3}~{y2:.3} 가 칸 clip {top:.3}~{bottom:.3} 밖"
            );
            assert!(
                cx >= clip.x && cx <= clip.x + clip.w && cy >= top && cy <= bottom,
                "괘선 중심 ({cx:.3},{cy:.3}) 이 칸 clip {clip:?} 밖"
            );
        }
    }
}

/// 칸 clip 경계에 놓인 괘선은 기준 raster(PDFium)에서 경계가 든 화소에 그려진다. 2020 기준
/// Print(2022 호환)도 장치 clip 을 써야 그 화소 행이 남는다 — 같은 프레임만 맞추면 1칸 미만 선이
/// 중심 화소로 옮겨진 뒤 소수 clip 에 다시 잘린다(36325899 2쪽, 프레임 단독 변형 67.50%).
#[test]
fn print_device_clip_is_generation_independent() {
    let mut core = core();
    for hangul2024 in [false, true] {
        core.set_hangul2024_compat(hangul2024);
        let json: Value = serde_json::from_str(
            &core
                .get_page_layer_tree_with_profile_native(1, RenderProfile::Print)
                .expect("layer tree"),
        )
        .expect("json");
        assert_eq!(
            json["rasterClipCoverage"], true,
            "Print 장치 clip (hangul2024={hangul2024})"
        );
        let screen: Value = serde_json::from_str(
            &core
                .get_page_layer_tree_with_profile_native(1, RenderProfile::Screen)
                .expect("layer tree"),
        )
        .expect("json");
        assert_eq!(screen["rasterClipCoverage"], false, "Screen 은 소수 clip");
    }
}
