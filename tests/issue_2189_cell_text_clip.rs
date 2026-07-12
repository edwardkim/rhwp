//! Issue #2189: 고정폭 테두리 글상자(1×1 표 셀) 우측 텍스트 클리핑 회귀 테스트.
//!
//! `samples/basic/issue1994_behindtext_table_20200830.hwp` p2 좌측 성명서 박스는
//! 저장 줄바꿈(LINE_SEG) 문서 + 미보유 폰트(08서울한강체 M) 조합이다. 한컴이 실폰트
//! 폭으로 나눈 줄을 우리 폴백 메트릭(~5% 넓음)으로 그리면 줄이 셀 내부 폭을 넘어
//! 우측 테두리 클립에서 마지막 글자가 잘렸다 (수정 전 28줄 중 14줄, 최대 +15px).
//!
//! 정정: Justify(공백 있음) 셀 오버플로우에서 공백 -50% 클램프 도달 후 잔여 음수
//! 슬랙을 자간(extra_char_sp)으로 스필오버 + 실효 폭 재측정 수렴 반복
//! (compute_line_extra_spacing, #229 underflow 확장의 대칭).

use std::fs;
use std::path::Path;

/// `<text ...>` 조각에서 `key="숫자"` 값을 추출.
fn attr_f64(fragment: &str, key: &str) -> Option<f64> {
    let pat = format!("{key}=\"");
    let start = fragment.find(&pat)? + pat.len();
    let rest = &fragment[start..];
    let end = rest.find('"')?;
    rest[..end].parse().ok()
}

/// `transform="translate(x,y) scale(sx,1)"` 형태에서 (x, y, sx) 추출.
fn parse_translate(fragment: &str) -> Option<(f64, f64, f64)> {
    let start = fragment.find("translate(")? + "translate(".len();
    let rest = &fragment[start..];
    let end = rest.find(')')?;
    let mut it = rest[..end].split(',');
    let x: f64 = it.next()?.trim().parse().ok()?;
    let y: f64 = it.next()?.trim().parse().ok()?;
    let sx = fragment
        .find("scale(")
        .and_then(|i| {
            let r = &fragment[i + "scale(".len()..];
            let e = r.find([',', ')'])?;
            r[..e].parse::<f64>().ok()
        })
        .unwrap_or(1.0);
    Some((x, y, sx))
}

#[test]
fn issue_2189_statement_cell_text_stays_inside_clip() {
    let repo_root = env!("CARGO_MANIFEST_DIR");
    let hwp_path =
        Path::new(repo_root).join("samples/basic/issue1994_behindtext_table_20200830.hwp");
    let bytes =
        fs::read(&hwp_path).unwrap_or_else(|e| panic!("read {}: {}", hwp_path.display(), e));

    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .expect("parse issue1994_behindtext_table_20200830.hwp");
    let svg = doc
        .render_page_svg_native(1)
        .expect("render page 2 (index 1) SVG");

    // 성명서 셀 클립 rect 탐색: 페이지 좌측(x<100)의 대형 셀(w>400, h>600).
    let mut cell: Option<(f64, f64, f64, f64)> = None;
    for frag in svg.split("<clipPath id=\"cell-clip-").skip(1) {
        let rect = &frag[..frag.find("</clipPath>").unwrap_or(frag.len())];
        let (Some(x), Some(y), Some(w), Some(h)) = (
            attr_f64(rect, "x"),
            attr_f64(rect, "y"),
            attr_f64(rect, "width"),
            attr_f64(rect, "height"),
        ) else {
            continue;
        };
        if x < 100.0 && w > 400.0 && h > 600.0 {
            cell = Some((x, y, w, h));
            break;
        }
    }
    let (cx, cy, cw, ch) = cell.expect("성명서 셀 클립 rect (좌측 대형 셀)을 찾지 못함");
    let clip_right = cx + cw;

    // 셀 영역 안 모든 텍스트 글리프: 시작 x + 글리프 폭(fs×장평)이 클립 우측을
    // 넘으면 마지막 글자가 테두리에서 잘리는 #2189 회귀.
    let mut checked = 0usize;
    let mut worst: (f64, String) = (f64::MIN, String::new());
    for frag in svg.split("<text ").skip(1) {
        let elem = &frag[..frag.find("</text>").unwrap_or(frag.len())];
        let (x, y, sx) = if let Some(t) = parse_translate(elem) {
            t
        } else {
            match (attr_f64(elem, "x"), attr_f64(elem, "y")) {
                (Some(x), Some(y)) => (x, y, 1.0),
                _ => continue,
            }
        };
        if !(x >= cx && x <= cx + cw && y >= cy && y <= cy + ch) {
            continue;
        }
        let fs = attr_f64(elem, "font-size").unwrap_or(0.0);
        let right = x + fs * sx;
        checked += 1;
        if right > worst.0 {
            let glyph = elem[elem.rfind('>').map(|i| i + 1).unwrap_or(0)..].to_string();
            worst = (right, glyph);
        }
    }

    assert!(
        checked > 100,
        "성명서 셀 안에서 검사된 글리프가 {checked}개 — 레이아웃 붕괴 의심"
    );
    assert!(
        worst.0 <= clip_right + 1.0,
        "성명서 셀 텍스트가 우측 클립을 넘음: 최대 우측 {:.1} > 클립 {:.1} (+{:.1}px, 글리프 {:?}) — #2189 회귀 (수정 전 최대 +15px)",
        worst.0,
        clip_right,
        worst.0 - clip_right,
        worst.1,
    );
}
