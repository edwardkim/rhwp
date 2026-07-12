//! Issue #2220: BehindText+tac 표 호스트 줄의 outer_margin 이중 계상.
//!
//! `samples/basic/issue1994_behindtext_table_20200830.hwp` p1 우측 단의 문단 0.6
//! (빈 문단 + 1×1 표 wrap=글뒤로·treat_as_char=true·RowBreak)은 저장 host lh
//! 24700 = 표 22996 + outer_margin 상하 852×2 — 한컴은 호스트 줄이 곧 표 박스다.
//! 수정 전 흐름은 (para_y + om_top) 기준 저장 lh advance + om_bottom 후가산으로
//! om 상하합(1704HU=22.7px)을 이중 계상 → 우측 단 전체 +29px 하강, 마지막 줄
//! ("최한나…함솔이") 페이지 하단 절단.
//!
//! 정정: 저장 lh 가 표 높이+om 합을 포함하는 증거가 있으면 문단 줄 상단 기준
//! advance + om_bottom 후가산 생략 (layout.rs tac_seg_applied 음수 ls 분기).

use std::fs;
use std::path::Path;

/// 우측 단(x>560)에서 probe 두 글자가 같은 y 에 있는 줄의 baseline 최댓값.
fn find_text_y(svg: &str, probe: &str) -> Option<f64> {
    let chars: Vec<char> = probe.chars().collect();
    let mut firsts: Vec<f64> = Vec::new();
    let mut seconds: Vec<f64> = Vec::new();
    for frag in svg.split("<text ").skip(1) {
        let head = &frag[..frag.find('>').unwrap_or(frag.len())];
        let body_start = frag.find('>').map(|i| i + 1).unwrap_or(0);
        let body = &frag[body_start..frag.find("</text>").unwrap_or(frag.len())];
        let (x, y) = if let Some(i) = head.find("translate(") {
            let r = &head[i + "translate(".len()..];
            let Some(e) = r.find(')') else { continue };
            let mut it = r[..e].split(',');
            let (Some(xs), Some(ys)) = (it.next(), it.next()) else {
                continue;
            };
            match (xs.trim().parse::<f64>(), ys.trim().parse::<f64>()) {
                (Ok(x), Ok(y)) => (x, y),
                _ => continue,
            }
        } else {
            let attr = |k: &str| -> Option<f64> {
                let pat = format!("{k}=\"");
                let s = head.find(&pat)? + pat.len();
                head[s..].split('"').next()?.parse().ok()
            };
            match (attr("x"), attr("y")) {
                (Some(x), Some(y)) => (x, y),
                _ => continue,
            }
        };
        if x <= 560.0 {
            continue;
        }
        if body.contains(chars[0]) {
            firsts.push(y);
        }
        if body.contains(chars[1]) {
            seconds.push(y);
        }
    }
    firsts
        .iter()
        .filter(|y| seconds.iter().any(|y2| (*y - y2).abs() < 1.0))
        .fold(None, |acc: Option<f64>, y| {
            Some(acc.map_or(*y, |a: f64| a.max(*y)))
        })
}

#[test]
fn issue_2220_right_column_not_pushed_by_tac_host_outer_margin() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue1994_behindtext_table_20200830.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue1994_behindtext_table_20200830.hwp");
    let svg = doc.render_page_svg_native(0).expect("render page 1 SVG");

    // 문단 0.7 "▮▮ 기도 나눔…" — 저장 vpos 24440HU 기준 baseline ≈ 405.
    // 회귀(om 이중 계상) 시 ≈ 427.
    let y = find_text_y(&svg, "나눔").expect("우측 단 기도나눔 제목을 찾지 못함");
    assert!(
        y < 415.0,
        "문단 0.7 이 tac 호스트 om 이중 계상으로 밀림: baseline y={y:.1} (기대 ≈405, 회귀 시 ≈427) — #2220"
    );

    // 우측 단 마지막 줄("최한나, …")이 페이지 본문 하한(748.4) 안에 온전.
    let y2 = find_text_y(&svg, "최한나").expect("우측 단 마지막 줄을 찾지 못함");
    assert!(
        y2 < 748.0,
        "우측 단 마지막 줄이 페이지 하단을 넘음: baseline y={y2:.1} — #2220 회귀 (수정 전 ≈760 절단)"
    );
}
