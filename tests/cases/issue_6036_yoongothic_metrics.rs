//! [#6036] 한컴 윤고딕 720 메트릭 부재로 대체 글꼴(맑은 고딕 Bold) 전진폭이
//! 쓰여 로고 글상자 clip 이 "경찰청" 글자를 깎던 결함 가드.
//!
//! 156509073(경찰청 보도자료, HWPX 2018) 1쪽 머리 묶음 — "경<fwSpace>찰
//! <fwSpace>청" 이 한글 2020 은 50.2pt(윤고딕 720 한글 0.88em)인데 rhwp 는
//! 맑은 고딕 Bold 대체 폭으로 64.0pt 가 되어 글상자 clip(55.07pt)을 8.9pt
//! 넘고 글자별 오른쪽이 깎였다. 수정 = 한컴오피스 동봉
//! HANYoonGothic720/740/760.ttf 의 hmtx 를 메트릭 overlay 에 등재(한글 지배
//! 폭 880/920/960 per mille, ASCII 전량; 이름은 HWPX 원문 "한컴 윤고딕 NNN").
//! 잔여: fwSpace(U+2007) 0.5em vs 한글 ~0.25em 은 #3216 필드 마커와 코드포인트
//! 를 공유해 별도 오라클 축으로 이슈에 남김.

#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const SAMPLE: &str = "samples/issue6036/156509073_police_press_release.hwpx";

fn render_p1_svg() -> String {
    let out = std::env::temp_dir().join(format!("rhwp_issue_6036_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("출력 디렉토리 생성");
    let done = Command::new(rhwp_bin())
        .current_dir(repo_root())
        .args([
            "export-svg",
            SAMPLE,
            "-p",
            "0",
            "-o",
            out.to_str().expect("출력 경로"),
        ])
        .output()
        .expect("rhwp export-svg 실행");
    assert!(
        done.status.success(),
        "export-svg 실패: {}",
        String::from_utf8_lossy(&done.stderr)
    );
    let svg = std::fs::read_dir(&out)
        .expect("출력 디렉토리")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "svg"))
        .expect("SVG 산출물");
    let text = std::fs::read_to_string(svg).expect("SVG 읽기");
    let _ = std::fs::remove_dir_all(&out);
    text
}

fn attr(attrs: &str, key: &str) -> Option<f64> {
    let at = attrs.find(&format!("{key}=\""))? + key.len() + 2;
    let tail = &attrs[at..];
    let stop = tail.find('"')?;
    tail[..stop].parse::<f64>().ok()
}

#[test]
fn issue_6036_yoongothic_logo_advances_use_real_metrics() {
    let svg = render_p1_svg();

    // 로고 줄의 경/찰/청 (윤고딕 720 16pt = 21.33px 크기가 판별자).
    let mut glyphs: Vec<(f64, &str, f64)> = Vec::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(end) = chunk.find('>') else { continue };
        let (attrs, rest) = chunk.split_at(end);
        let body_end = rest[1..].find("</text>").map(|e| e + 1).unwrap_or(1);
        let body = &rest[1..body_end];
        if !matches!(body, "경" | "찰" | "청") {
            continue;
        }
        let (Some(x), Some(y), Some(fs)) =
            (attr(attrs, "x"), attr(attrs, "y"), attr(attrs, "font-size"))
        else {
            continue;
        };
        // 1쪽 머리 로고 한정: 16pt(21.33px) + 페이지 상단 y<200.
        if (21.0..=22.0).contains(&fs) && y < 200.0 {
            glyphs.push((x, if body == "경" { "경" } else { body }, fs));
        }
    }
    glyphs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    assert!(glyphs.len() >= 3, "로고 경/찰/청 글리프 3개: {glyphs:?}");
    let fs = glyphs[0].2;
    let gap1 = glyphs[1].0 - glyphs[0].0;
    let gap2 = glyphs[2].0 - glyphs[1].0;

    // 글리프 간 전진 = 한글 0.88em + fwSpace 0.5em ≈ 1.3~1.4em.
    // 메트릭 부재(맑은 고딕 대체)면 1.55em+ 로 벌어져 글상자 clip 을 깎는다.
    for gap in [gap1, gap2] {
        let em = gap / fs;
        assert!(
            (1.15..=1.48).contains(&em),
            "로고 글리프 전진 {em:.2}em — 윤고딕 720 실메트릭(0.88em)이 아니라 \
             대체 폭으로 측정됨 (gap {gap:.1}px @ fs {fs:.1})",
        );
    }
}
