//! [#5855] 쪽 하단에 앉은 부동 개체는 본문 클립에 잘리지 않는다.
//!
//! 종전(devel `0f9ceeb19a`): body clip 은 부동 개체 몫을 `본문하단 + 10px` 로 잘랐다.
//! `156618554_petfood_press.hwp` 의 발신기관 로고 띠는 1055.2px 까지 내려오는데
//! 클립이 1038.0px 에서 끊겨 **20.9px 가 사라졌다.**
//!
//! 한글 정답지 `pdf/task2137/156618554_petfood_press-2020.pdf` 는 같은 그림을
//! y=1013.3 h=42.7 (하단 1056.0px) 로 **끝까지 그린다** — 본문 하단 1028.1px 아래다.
//! 즉 한글은 쪽 기준으로 앉힌 개체를 본문 영역에 가두지 않는다.
//!
//! 이 시험이 고정하는 것은 두 가지다:
//! 1. 로고 띠가 body clip 안에 온전히 들어온다 (잘림 없음).
//! 2. 그렇다고 clip 이 용지 밖으로 나가지는 않는다 (상한이 사라진 게 아니다).
#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const SAMPLE: &str = "samples/task2137/156618554_petfood_press.hwp";

fn render_page_svg() -> String {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nth = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out = std::env::temp_dir().join(format!("rhwp_issue_5855_{}_{}", std::process::id(), nth));
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
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "svg"))
        .expect("SVG 산출물");
    std::fs::read_to_string(svg).expect("SVG 읽기")
}

/// `name="12.3"` 형태의 수치 속성을 읽는다. 앞에서부터 `from` 위치 이후만 본다.
fn attr_at(text: &str, from: usize, name: &str) -> Option<f64> {
    let needle = format!("{name}=\"");
    let start = text[from..].find(&needle)? + from + needle.len();
    let end = start + text[start..].find('"')?;
    text[start..end].parse().ok()
}

/// body clip 사각형의 (상단, 하단).
fn body_clip_span(svg: &str) -> (f64, f64) {
    let at = svg
        .find("<clipPath id=\"body-clip-")
        .expect("body clip 이 있어야 한다");
    let y = attr_at(svg, at, "y").expect("clip y");
    let h = attr_at(svg, at, "height").expect("clip height");
    (y, y + h)
}

/// `<image>` 들의 최하단 y.
fn lowest_image_bottom(svg: &str) -> f64 {
    let mut cursor = 0usize;
    let mut lowest = f64::MIN;
    while let Some(rel) = svg[cursor..].find("<image") {
        let at = cursor + rel;
        let end = at + svg[at..].find('>').expect("image 태그 끝");
        let tag = &svg[at..end];
        if let (Some(y), Some(h)) = (attr_at(tag, 0, "y"), attr_at(tag, 0, "height")) {
            lowest = lowest.max(y + h);
        }
        cursor = end;
    }
    assert!(lowest > f64::MIN, "그림이 하나도 없다");
    lowest
}

fn page_height(svg: &str) -> f64 {
    let at = svg.find("<svg").expect("svg 루트");
    attr_at(svg, at, "height").expect("svg height")
}

#[test]
fn page_bottom_logo_band_survives_the_body_clip() {
    let svg = render_page_svg();
    let (_, clip_bottom) = body_clip_span(&svg);
    let logo_bottom = lowest_image_bottom(&svg);

    // 로고 띠는 본문 하단(1028.1px)보다 아래에 있다 — 이 시험이 겨냥하는 형상이 맞는지 먼저 확인.
    assert!(
        logo_bottom > 1040.0,
        "로고 띠가 본문 하단 아래에 있어야 이 시험이 의미가 있다 — 실측 {logo_bottom:.1}px"
    );
    assert!(
        clip_bottom + 0.5 >= logo_bottom,
        "쪽 하단 로고 띠가 body clip 에 잘린다 — clip 하단 {clip_bottom:.1}px < 그림 하단 {logo_bottom:.1}px \
         (종전 상한 '본문하단+10px' = 1038.0px 로 20.9px 소실)"
    );
}

#[test]
fn body_clip_still_stops_at_the_paper() {
    let svg = render_page_svg();
    let (_, clip_bottom) = body_clip_span(&svg);
    let paper = page_height(&svg);
    assert!(
        clip_bottom <= paper + 0.5,
        "clip 이 용지({paper:.1}px)를 넘었다 — 상한이 사라지면 안 된다: {clip_bottom:.1}px"
    );
}
