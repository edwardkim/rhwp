//! [#5863] 잘린 셀 clip 아래에서 시작하는 다음 쪽 표 조각이 현재 쪽에 중복으로 실린다.
//!
//! `samples/hwpx_sample2.hwp` 8쪽 SVG 에는 9쪽 표(`구분/조회방법` 순위확인서 발급 안내)의
//! 글자가 한 벌 더 들어 있었다. 같은 표가 9쪽에 온전히 그려지고 한글 2024 정본
//! (`pdf/hwpx_sample2-2024.pdf`)도 9쪽에 두므로 순수한 중복이다.
//!
//! 조각은 본문 하한(1,084.7px)을 넘어 1,129.9px 까지 뻗어 `detect_table_clipping` 이
//! 이 문서를 클리핑으로 신고했다(`CLIP 1/29p max_overflow=45.1px`).
//!
//! 원인은 억제 창이 테두리 안티에일리어싱 폭(6px)뿐이었던 것이다 — 이 조각은 잘린 셀의
//! clip 바닥보다 34.4px 아래에서 시작한다. clip 바닥 아래에서 시작하는 표는 그 셀 안에서
//! 보일 수 있는 부분이 없으므로 거리와 무관하게 다음 쪽 조각이다.
#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

/// nextest archive 가 런타임에 주입하는 경로를 먼저 읽고, 없으면 컴파일타임 값을 쓴다
/// (local_validation.md 4.3 의 신규 CLI 통합 테스트 규칙 — #3289).
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const SAMPLE: &str = "samples/hwpx_sample2.hwp";

/// `-p` 는 0 기준이라 "7" 은 문서 8쪽이다.
fn render_page_svg(page_arg: &str) -> String {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nth = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out = std::env::temp_dir().join(format!("rhwp_issue_5863_{}_{}", std::process::id(), nth));
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("출력 디렉토리 생성");
    let done = Command::new(rhwp_bin())
        .current_dir(repo_root())
        .args([
            "export-svg",
            SAMPLE,
            "-p",
            page_arg,
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

/// `<text ...>내용</text>` 을 이어 붙인다.
fn svg_plain_text(svg: &str) -> String {
    let mut out = String::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(open_end) = chunk.find('>') else {
            continue;
        };
        let rest = &chunk[open_end + 1..];
        let Some(close) = rest.find("</text>") else {
            continue;
        };
        out.push_str(&rest[..close]);
    }
    out
}

/// 본문 clip 사각형의 아래 끝.
fn body_clip_bottom(svg: &str) -> f64 {
    let at = svg
        .find("<clipPath id=\"body-clip")
        .expect("body-clip 정의");
    let rect = &svg[at..];
    let pick = |key: &str| -> f64 {
        let i = rect.find(&format!("{key}=\"")).expect("속성") + key.len() + 2;
        let tail = &rect[i..];
        tail[..tail.find('"').expect("닫는 따옴표")]
            .parse()
            .expect("수치")
    };
    pick("y") + pick("height")
}

/// 8쪽에 9쪽 표의 글자가 한 벌 더 실리면 안 된다.
#[test]
fn next_page_table_fragment_is_not_duplicated_on_the_current_page() {
    let page8 = svg_plain_text(&render_page_svg("7"));
    for needle in ["본인확인", "내역확인", "한국부동산원"] {
        assert!(
            !page8.contains(needle),
            "8쪽에 9쪽 표의 글자 {needle:?} 가 중복으로 실렸다 (#5863)"
        );
    }
}

/// 9쪽은 그대로다 — 억제가 진짜 내용을 지우면 안 된다.
#[test]
fn the_owning_page_still_draws_the_whole_table() {
    let page9 = svg_plain_text(&render_page_svg("8"));
    for needle in ["본인확인", "내역확인", "한국부동산원", "구분"] {
        assert!(
            page9.contains(needle),
            "9쪽에서 표 내용 {needle:?} 가 사라졌다 — 억제가 과했다"
        );
    }
}

/// 8쪽 본문 하한 아래에 남는 글자는 꼬리말뿐이어야 한다.
#[test]
fn nothing_but_the_footer_is_drawn_below_the_body_on_page_eight() {
    let svg = render_page_svg("7");
    let bottom = body_clip_bottom(&svg);
    let mut below = Vec::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(open_end) = chunk.find('>') else {
            continue;
        };
        let (attrs, rest) = chunk.split_at(open_end);
        let Some(close) = rest[1..].find("</text>") else {
            continue;
        };
        let body = &rest[1..close + 1];
        let Some(i) = attrs.find("y=\"") else {
            continue;
        };
        let tail = &attrs[i + 3..];
        let Ok(y) = tail[..tail.find('"').unwrap_or(0)].parse::<f64>() else {
            continue;
        };
        if y > bottom + 0.5 && !body.trim().is_empty() {
            below.push((y, body.to_string()));
        }
    }
    // 꼬리말 쪽번호(`- 8 -`)는 본문 밖이 제자리다.
    let non_footer: Vec<_> = below
        .iter()
        .filter(|(_, t)| !matches!(t.trim(), "-" | "8" | ""))
        .collect();
    let shown = &non_footer[..non_footer.len().min(6)];
    assert!(
        non_footer.is_empty(),
        "본문 하한({:.1}) 아래에 꼬리말 아닌 글자가 남았다: {:?}",
        bottom,
        shown
    );
}
