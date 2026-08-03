//! Issue #3738: 한 문단에 자리차지 개체가 둘 이상이면 뒤따르는 문단이 쪽 밖으로 밀린다.
//!
//! ## 근인
//!
//! 텍스트 없는 호스트 문단에서 자리차지 개체는 **하나가 한 줄**을 차지하고, 그 줄
//! 높이는 저장된 `LINE_SEG` 에 있다. 같은 오해가 조판·레이아웃 양쪽에 있었다.
//!
//! ```text
//! 표본 pi=0   ls[0] 표    vertsize 30600 + spacing 600 → 416.0px
//!             ls[1] 글상자 vertsize  6600 + spacing 600 →  96.0px
//! ```
//!
//! 1. `layout.rs` 의 자리차지 도형 전진이 언제나 `line_segs.first()` 를 봤다 —
//!    글상자가 앞선 **표**의 줄 높이(416.0px)로 커서를 밀어 320px 과전진한다.
//! 2. `typeset.rs` 의 "TAC 표 높이 보정" cap 은 `tac_seg_total` 을 **표만** 순회해
//!    계산한다. Task #402 경로가 방금 더한 글상자 줄(96.0px)을 cap 이 도로 되감아,
//!    조판은 같은 쪽에 문단을 더 얹는다.
//!
//! 방향이 반대라 서로를 가린다 — 쪽수는 그대로인데 글자만 쪽 밖으로 나간다.
//! `export-text` 바이트도 IR diff 도 이 글자를 잃지 않으므로 그 지표로는 안 잡힌다
//! ([[out-of-page-glyph-blind-spot]] 계열, `issue_3637_split_cell_nested_table_vpos`
//! 와 같은 증상의 다른 기전).
//!
//! ## 계약
//!
//! 자리차지 개체는 **자기 줄**만큼 전진한다. 표본은 그 차이가 쪽 경계를 넘도록
//! 맞춰 뒀다.
//!
//! ```text
//! 수정 전  도형 전진 416.0px   쪽 밖 글자 117   최대 초과 178.0px
//! 수정 후  도형 전진  96.0px   쪽 밖 글자   0   최대 초과   0.0px
//! ```
#![cfg(not(target_arch = "wasm32"))]

/// 손으로 지은 최소 표본. 호스트 문단 하나에 자리차지 표(30600 HU 줄) + 자리차지
/// 글상자(6600 HU 줄), 그 뒤에 본문 18줄. 수정 전에는 본문이 쪽 아래로 밀린다.
const SAMPLE: &str = "samples/issue3738/tac_sibling_shape_line_advance.hwpx";

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// SVG 한 쪽에서 (뷰박스 높이, 쪽 높이를 넘은 글자의 y 와 내용) 을 뽑는다.
fn out_of_page_glyphs(svg: &str) -> (f64, Vec<(f64, String)>) {
    let height = svg
        .split("height=\"")
        .nth(1)
        .and_then(|s| s.split('"').next())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let mut over = Vec::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(rest) = chunk.split(" y=\"").nth(1) else {
            continue;
        };
        let Some(y) = rest.split('"').next().and_then(|s| s.parse::<f64>().ok()) else {
            continue;
        };
        // 글리프 baseline 이 경계에 걸친 경우는 결함으로 세지 않는다.
        if y > height + 2.0 {
            let text = chunk
                .split('>')
                .nth(1)
                .and_then(|s| s.split('<').next())
                .unwrap_or("")
                .to_string();
            over.push((y, text));
        }
    }
    (height, over)
}

fn render_pages() -> Vec<String> {
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("파싱");
    (0..doc.page_count())
        .map(|p| doc.render_page_svg(p).expect("SVG 렌더"))
        .collect()
}

/// 자리차지 개체 뒤 본문이 쪽 안에 남는다.
#[test]
fn text_after_sibling_tac_objects_stays_inside_the_page() {
    let pages = render_pages();
    assert_eq!(
        pages.len(),
        1,
        "표본이 1쪽에 담기도록 지었다 — 쪽수가 달라졌으면 표본이나 본문 여백 계약을 확인하라"
    );

    let mut offenders = Vec::new();
    for (i, svg) in pages.iter().enumerate() {
        let (h, over) = out_of_page_glyphs(svg);
        assert!(
            h > 0.0,
            "{}쪽 SVG 뷰박스 높이를 읽지 못했다 — 방출 형식 확인",
            i + 1
        );
        for (y, t) in over {
            offenders.push(format!(
                "{}쪽 y={y:.1} (쪽 높이 {h:.1}, 초과 {:.1}px) {t:?}",
                i + 1,
                y - h
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "쪽 높이를 넘은 글자가 {}개 있다 (수정 전 실측 117개·최대 178.0px).\n  {}\n\
         한 문단의 두 번째 자리차지 개체가 자기 LINE_SEG 가 아니라 첫 줄(앞 개체)의 \
         높이로 커서를 전진시키면 뒤따르는 본문이 쪽 밖으로 나간다. 나간 글자는 텍스트 \
         추출에는 남아 있어 텍스트/IR diff 로는 잡히지 않는다.",
        offenders.len(),
        offenders
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

/// `<text>` 안의 글자를 방출 순서대로 이어 붙인다 (공백 제거).
///
/// SVG 는 글자를 낱개 `<text>` 로 쪼개 방출하므로 줄 단위 문자열이 그대로 남지 않는다.
fn flattened_text(svg: &str) -> String {
    let mut out = String::new();
    for chunk in svg.split("<text").skip(1) {
        let Some(body) = chunk.split_once('>').map(|(_, rest)| rest) else {
            continue;
        };
        let Some(inner) = body.split("</text>").next() else {
            continue;
        };
        for c in inner.chars() {
            if !c.is_whitespace() && c != '<' && c != '>' {
                out.push(c);
            }
        }
    }
    out
}

/// 본문 18줄이 모두 보인다 — "쪽 밖 글자 0" 을 빈 쪽으로 만족시키지 못하게 한다.
#[test]
fn all_body_lines_are_rendered_within_the_page() {
    let pages = render_pages();
    let svg = pages.first().expect("1쪽");
    let height = out_of_page_glyphs(svg).0;
    let flat = flattened_text(svg);
    let missing: Vec<String> = (1..=18)
        .map(|k| format!("본문{k:02}"))
        .filter(|marker| !flat.contains(marker))
        .collect();
    assert!(
        missing.is_empty(),
        "본문 {}줄이 1쪽(높이 {height:.1})에서 사라졌다: {missing:?} — \
         넘침이 0 이어도 내용이 빠지면 계약 위반이다",
        missing.len()
    );
}
