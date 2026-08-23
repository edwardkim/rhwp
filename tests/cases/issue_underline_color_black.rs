//! 밑줄 색으로 저장된 **검정(COLORREF 0)** 은 글자색으로 대체되지 않는다.
//!
//! 종전(devel `72674c5653`): 렌더러 세 백엔드가 모두 `underline_color != 0` 을
//! "밑줄색 지정됨" sentinel 로 썼다. 그러나 `underline_color` 는 HWP5 CHAR_SHAPE 의
//! **고정 위치 필수 필드**(`src/parser/doc_info.rs` 의 `text_color` 바로 뒤)이고,
//! COLORREF 0 은 "미지정"이 아니라 **검정**이다. 그래서 문서가 `글자색=유채색 /
//! 밑줄색=검정` 으로 지정한 밑줄이 글자색으로 그려졌다.
//!
//! 한글 정답지 대조:
//! - `pdf/hwpx_sample2-2024.pdf` 12쪽 — 가로 괘선 29개 중 25개가 검정.
//!   rhwp 는 그중 24개를 글자색 `#082108` 로 그렸다.
//! - `pdf/pr-1674-2024.pdf` 7쪽 — 가로선 27개가 검정 19 + 초록 8 이고
//!   **파랑·빨강은 0개**. rhwp 는 `#0000ff` 2개를 그렸다.
//!
//! 이 시험이 고정하는 것은 두 가지다:
//! 1. 저장된 검정 밑줄이 글자색으로 새지 않는다.
//! 2. 0 이 아닌 색(빨강)은 **그대로 유지**된다 — 무조건 검정으로 칠하는 게 아니다.
#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn render_page_svg(sample: &str, page: &str, tag: &str) -> String {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nth = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out = std::env::temp_dir().join(format!(
        "rhwp_underline_color_{}_{}_{}",
        tag,
        std::process::id(),
        nth
    ));
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("출력 디렉토리 생성");
    let done = Command::new(rhwp_bin())
        .current_dir(repo_root())
        .args([
            "export-svg",
            sample,
            "-p",
            page,
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

/// `<line …>` 요소 중 지정한 stroke 색을 쓰는 것의 개수.
fn line_stroke_count(svg: &str, color: &str) -> usize {
    let needle = format!("stroke=\"{color}\"");
    svg.match_indices("<line")
        .filter(|(at, _)| {
            let end = svg[*at..].find('>').map_or(svg.len(), |off| at + off);
            svg[*at..end].contains(&needle)
        })
        .count()
}

#[test]
fn stored_black_underline_does_not_take_the_text_color() {
    // charPr 260·265·364: textColor="#082108" + underline color="#000000"
    let svg = render_page_svg("samples/hwpx_sample2.hwpx", "11", "hwpx2");
    assert_eq!(
        line_stroke_count(&svg, "#082108"),
        0,
        "글자색 #082108 으로 그려진 밑줄이 남아 있다 — 정답지 12쪽 밑줄은 전부 검정이다"
    );
    assert!(
        line_stroke_count(&svg, "#000000") >= 24,
        "검정 밑줄이 24개 이상이어야 한다 (정답지 25개)"
    );
}

#[test]
fn stored_black_underline_survives_a_blue_text_run() {
    // HWP5 경로: underline_color=0x000000 인데 text_color=0x0000FF 인 run.
    let svg = render_page_svg("samples/pr-1674.hwp", "6", "pr1674");
    assert_eq!(
        line_stroke_count(&svg, "#0000ff"),
        0,
        "파란 밑줄이 남아 있다 — 정답지 7쪽에는 파랑 선이 0개다"
    );
}

#[test]
fn non_zero_underline_color_is_still_honored() {
    // 회귀 방지: 0 이 아닌 색은 그대로 살아야 한다. 무조건 검정이 아니다.
    let svg = render_page_svg("samples/hwpx_sample2.hwpx", "15", "hwpx2red");
    assert_eq!(
        line_stroke_count(&svg, "#ff0000"),
        4,
        "color=\"#FF0000\" 밑줄 4개가 그대로 빨강이어야 한다"
    );
}
