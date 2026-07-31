//! Issue #3637: 자리차지 개체가 앵커된 문단의 줄이 쪽 밖에 배치돼 보이지 않는다.
//!
//! ## 근인
//!
//! `LINE_SEG.vertical_pos` 는 문단 기준이 아니라 **단(column) 기준**으로 누적된다
//! (재현 문서 1쪽: pi=5 → 13949, pi=17 → 58149). `paragraph_layout.rs` 의
//! `para_topbottom_line_vpos_base` 는 `start_line == 0` 일 때 기준을 0 으로 두어 그
//! 절대값을 흐름 y 에 **한 번 더** 더했다.
//!
//! 문단이 쪽 상단이면 vpos≈0 이라 무해했지만, 쪽 중간 문단이면 자기 vpos 만큼 아래로
//! 밀려 쪽 경계를 넘는다. 재현 문서 pi=17 실측:
//!
//! ```text
//! 흐름 커서 775.3px 가 정답인데 vpos 58149(=775.3px)를 더해 1660px 에 그렸다.
//! 쪽 하단 1028px 를 632px 넘겨 세 줄 93글자가 사라졌다.
//! ```
//!
//! 넘어간 줄은 `export-text` 에도 SVG `<text>` 요소에도 남아 있어 **텍스트 diff 나
//! IR diff 로는 잡히지 않는다**. 좌표가 쪽 밖이라 어느 렌더 경로에서도 안 보일 뿐이다.
//!
//! 발동 조건이 좁아(비-TAC · `TopAndBottom` · `VertRelTo::Para` · 단과 가로 교차)
//! 오래 남아 있었다. 10k 서베이에서 쪽수 불일치 코호트 390건 중 99건이 이 증상을
//! 보였고, 쪽수가 맞는 대조군 300건에서는 1건뿐이었다 (76배 농축).
//!
//! ## 계약
//!
//! 어떤 쪽에서도 쪽 높이를 넘는 자리에 글자를 그리지 않는다.
//!
//! 합성 문서로는 재현되지 않아 실제 문서를 표본으로 쓴다 — 결함이 저장 vpos 사다리와
//! 개체 기하의 특정 조합에서만 발동해서, 손으로 만든 `LINE_SEG` 로는 그 조합을
//! 맞추지 못했다 (음성 대조에서 수정 없이도 통과했다).
#![cfg(not(target_arch = "wasm32"))]

/// 해양수산부 보도자료. 1쪽 pi=17 이 자리차지 도형을 앵커한 문단이다.
///
/// 대한민국 정책브리핑 공개 자료.
const SAMPLE: &str = "samples/issue3637/press_release_topbottom_float.hwpx";

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// SVG 한 쪽에서 (뷰박스 높이, 쪽 높이를 넘는 글자와 그 y) 를 뽑는다.
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
        // 여유 2px — baseline 이 경계에 걸친 글리프는 결함으로 세지 않는다.
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

/// 모든 쪽에서 글자가 쪽 높이 안에 그려진다.
#[test]
fn no_glyph_is_drawn_outside_the_page() {
    let bytes = std::fs::read(sample_path()).expect("표본 읽기");
    let doc = rhwp::wasm_api::HwpDocument::from_bytes(&bytes).expect("파싱");
    let page_count = doc.page_count();
    assert!(
        page_count >= 2,
        "쪽이 너무 적다({page_count}) — 표본이 바뀌었는지 확인하라"
    );

    let mut offenders = Vec::new();
    for p in 0..page_count {
        let svg = doc.render_page_svg(p).expect("SVG 렌더");
        let (h, over) = out_of_page_glyphs(&svg);
        assert!(
            h > 0.0,
            "{}쪽 SVG 뷰박스 높이를 읽지 못했다 — 방출 형식 확인",
            p + 1
        );
        for (y, t) in over {
            offenders.push(format!(
                "{}쪽 y={y:.1} (쪽 높이 {h:.1}, 초과 {:.1}px) {t:?}",
                p + 1,
                y - h
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "쪽 높이를 넘는 자리에 그려진 글자가 {}개 있다.\n  {}\n\
         자리차지 개체가 앵커된 문단의 줄 기준을 단 기준 vpos 로 잡으면 흐름 y 에 \
         이중 계상되어 쪽 밖으로 나간다. 그렇게 나간 글자는 텍스트 추출에는 남아 있어 \
         텍스트/IR diff 로는 잡히지 않는다.",
        offenders.len(),
        offenders
            .iter()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
