//! [#6656] 문단 안 줄 전진은 저장 줄의 글자 높이(th)를 따른다.
//!
//! 저장 줄 사다리(`ls[]`)는 문단 안에서 다음 줄을 `th + ls` 자리에 놓는다. 줄 상자 높이
//! `lh` 는 그 줄이 품은 개체까지 덮지만 전진에는 쓰이지 않는다 — 한/글은 개체가 아래 줄
//! 공간을 침범하게 둔다. 코퍼스 전수(samples↔pdf 215 문서, `lh != th` 인 문단 안 연속 줄):
//! `th + ls` 45건 / `lh + ls` 1건.
//!
//! `samples/hwpctl_ParameterSetID_Item_v1.2.hwp` 문단 0.7 은 3쪽에서 vpos 가 0 으로 리셋된다:
//! `ls[2] vpos=0 lh=1560 th=1000 ls=600` → `ls[3] vpos=1600`(= th+ls). 그 줄들에는 23.2px
//! 아이콘이 있어 `lh` 가 크다. 종전 rhwp 는 `lh + ls`(2160HU=28.8px)로 전진해 둘째 줄부터
//! 7.5px 씩 밀렸고, 3쪽 아이콘 넷이 한/글보다 15px 아래였다.
//!
//! 구현은 `th + ls` 를 다시 계산하지 않고 저장된 `next.vertical_pos - seg.vertical_pos` 를
//! 그대로 읽는다. 위 45:1 은 그 값이 어떤 규칙을 따르는지 보여 주는 근거일 뿐이다.
//!
//! 한/글 2022 PDF(`pdf/hwpctl_ParameterSetID_Item_v1.2-2022.pdf`, 같은 쪽 크기) 실측:
//! 위 아이콘 쌍 y=99.4, 아래 쌍 y=156.9.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;

use rhwp::renderer::composer::compose_paragraph;
use rhwp::renderer::height_measurer::HeightMeasurer;
use rhwp::renderer::style_resolver::resolve_styles;
use rhwp::DocumentCore;

fn page_svg(rel: &str, page: u32) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(rel);
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {rel}: {e}"));
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes)
        .unwrap_or_else(|e| panic!("parse {rel}: {e:?}"));
    doc.render_page_svg(page)
        .unwrap_or_else(|e| panic!("render {rel} p{}: {e:?}", page + 1))
}

/// 태그 조각에서 `name="…"` 숫자 속성. 첫 속성은 앞에 공백이 없고, 다른 속성 이름의 꼬리에
/// 걸리지 않게 앞 글자가 영숫자가 아닐 때만 받는다.
fn attr(tag: &str, name: &str) -> Option<f64> {
    let pat = format!("{name}=\"");
    let mut from = 0;
    while let Some(i) = tag[from..].find(&pat) {
        let at = from + i;
        if at == 0 || !tag.as_bytes()[at - 1].is_ascii_alphanumeric() {
            let s = at + pat.len();
            let e = s + tag[s..].find('"')?;
            return tag[s..e].parse().ok();
        }
        from = at + 1;
    }
    None
}

/// 폭이 `w`(±0.6) 인 그림 자리의 (x, y). 자른 그림은 `<svg>` 래퍼로 나오므로 둘 다 본다.
fn images_with_width(svg: &str, w: f64) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for open in ["<image ", "<svg "] {
        for t in svg.split(open).skip(1) {
            let t = &t[..t.find('>').expect("태그 닫힘")];
            if (attr(t, "width").unwrap_or(0.0) - w).abs() < 0.6 {
                if let (Some(x), Some(y)) = (attr(t, "x"), attr(t, "y")) {
                    out.push((x, y));
                }
            }
        }
    }
    out
}

#[test]
fn lines_advance_by_stored_text_height_not_line_box_height() {
    let svg = page_svg("samples/hwpctl_ParameterSetID_Item_v1.2.hwp", 2);
    let icons = images_with_width(&svg, 23.2);
    assert!(icons.len() >= 4, "3쪽 23.2px 아이콘 4개: {icons:?}");
    let mut ys: Vec<f64> = icons.iter().map(|(_, y)| *y).collect();
    ys.sort_by(f64::total_cmp);
    ys.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    assert!(
        ys.iter().any(|y| (y - 99.4).abs() < 0.7),
        "위 아이콘 줄 y 99.4 (한/글 99.4, 종전 114.3): {ys:?}"
    );
    assert!(
        ys.iter().any(|y| (y - 157.0).abs() < 0.7),
        "아래 아이콘 줄 y 157.0 (한/글 156.9, 종전 171.9): {ys:?}"
    );
}

#[test]
fn measurement_uses_the_same_saved_ladder_step_as_layout() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("samples/hwpctl_ParameterSetID_Item_v1.2.hwp");
    let core = DocumentCore::from_bytes(&std::fs::read(path).expect("#6656 기준 HWP 읽기"))
        .expect("#6656 기준 HWP 파싱");
    let document = core.document();
    let para = document.sections[0].paragraphs[7].clone();
    let composed = compose_paragraph(&para);
    let styles = resolve_styles(&document.doc_info, 96.0);
    let measured = HeightMeasurer::new(96.0)
        .with_native_hwp5(true)
        .measure_section(&[para.clone()], &[composed], &styles, None);
    let paragraph = measured.get_measured_paragraph(0).expect("문단 측정");

    let current = &para.line_segs[2];
    let next = &para.line_segs[3];
    let stored_advance = f64::from(next.vertical_pos - current.vertical_pos) / 75.0;
    let line_box_advance = f64::from(current.line_height + current.line_spacing) / 75.0;
    assert!(
        (stored_advance - line_box_advance).abs() > 1.0,
        "이 입력은 저장 사다리와 줄 상자 advance가 달라야 한다"
    );
    assert!(
        (paragraph.line_advance(2) - stored_advance).abs() < 0.01,
        "측정도 배치와 같은 저장 사다리 advance를 써야 한다: measured={}, stored={}, box={}",
        paragraph.line_advance(2),
        stored_advance,
        line_box_advance,
    );
}
