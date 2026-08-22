//! [#5862] 쪽 분할 조각 셀의 clip 이 자기 글줄보다 짧아 마지막 줄이 사라진다.
//!
//! `samples/hwpx_sample2.hwp` 8쪽에서 조각 셀의 clip 바닥은 985.7px 인데, 그 셀이 자기
//! 자식으로 배치한 직계 글줄은 1,018.2px 까지 간다. 그래서
//! `[청약신청주택] 주택명 : 부도임대(경주금장로얄) , 관리번호 : 2026 - 000262` 한 줄이
//! **한 픽셀도 그려지지 않았다** — 다음 쪽(9쪽)도 그 줄을 그리지 않으므로 순수한 소실이다.
//! 한글 2024 정본(`pdf/hwpx_sample2-2024.pdf`) 8쪽은 그 줄을 그린다.
//!
//! 조각 셀의 clip 높이는 괘선이 아니라 쪽 컷 부기가 정하므로, 부기와 조판이 어긋나면
//! 이렇게 어긋난다. 보정은 `page_fragment` 셀의 **직계 글줄**만 포섭한다.
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
const PAGE_ARG: &str = "7";

fn render_page_svg(page_arg: &str) -> String {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nth = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out = std::env::temp_dir().join(format!("rhwp_issue_5862_{}_{}", std::process::id(), nth));
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
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .find(|path| path.extension().is_some_and(|ext| ext == "svg"))
        .expect("SVG 산출물");
    let text = std::fs::read_to_string(svg).expect("SVG 읽기");
    let _ = std::fs::remove_dir_all(&out);
    text
}

/// `id` 로 지정한 clipPath 사각형의 (top, bottom).
fn clip_bounds(svg: &str, id_prefix: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
    for chunk in svg.split("<clipPath id=\"").skip(1) {
        if !chunk.starts_with(id_prefix) {
            continue;
        }
        let pick = |key: &str| -> Option<f64> {
            let at = chunk.find(&format!("{key}=\""))? + key.len() + 2;
            let tail = &chunk[at..];
            tail[..tail.find('"')?].parse().ok()
        };
        if let (Some(y), Some(h)) = (pick("y"), pick("height")) {
            out.push((y, y + h));
        }
    }
    out
}

/// (baseline y, 글자) — 글자마다 개별 `<text>` 다.
fn glyphs(svg: &str) -> Vec<(f64, String)> {
    let mut out = Vec::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(open_end) = chunk.find('>') else {
            continue;
        };
        let (attrs, rest) = chunk.split_at(open_end);
        let Some(close) = rest[1..].find("</text>") else {
            continue;
        };
        let body = rest[1..close + 1].to_string();
        let Some(at) = attrs.find("y=\"") else {
            continue;
        };
        let tail = &attrs[at + 3..];
        let Ok(y) = tail[..tail.find('"').unwrap_or(0)].parse::<f64>() else {
            continue;
        };
        out.push((y, body));
    }
    out
}

/// 8쪽 마지막 줄(`관리번호 : 2026 - 000262`)이 조각 셀 clip 안에 있어야 한다.
#[test]
fn split_fragment_keeps_its_last_line_inside_the_clip() {
    let svg = render_page_svg(PAGE_ARG);

    // 그 줄의 baseline 을 글자에서 찾는다 — 한 줄에 같은 y 로 흩어져 있다.
    let mut line_bottom = f64::MIN;
    let mut found = false;
    for (y, ch) in glyphs(&svg) {
        if ch == "관" || ch == "리" || ch == "번" || ch == "호" {
            line_bottom = line_bottom.max(y);
            found = true;
        }
    }
    assert!(
        found,
        "8쪽에 `관리번호` 줄이 아예 없다 — 표본이 바뀌었는지 확인 필요"
    );

    let covering = clip_bounds(&svg, "cell-clip")
        .into_iter()
        .any(|(top, bottom)| top <= line_bottom && line_bottom <= bottom);
    assert!(
        covering,
        "마지막 글줄(baseline {line_bottom:.1})을 담는 셀 clip 이 없다 — \
         조각 clip 이 자기 글줄보다 짧아 그 줄이 통째로 사라진다 (#5862)"
    );
}

/// 같은 baseline 의 글자를 이어 붙여 줄 단위 문자열로 만든다.
fn lines(svg: &str) -> Vec<String> {
    let mut by_y: std::collections::BTreeMap<i64, String> = std::collections::BTreeMap::new();
    for (y, ch) in glyphs(svg) {
        by_y.entry((y * 10.0).round() as i64)
            .or_default()
            .push_str(&ch);
    }
    by_y.into_values().collect()
}

/// 조각이 자기 것으로 그린 줄을 다음 쪽이 다시 그리면 안 된다 (중복 금지).
///
/// 낱글자로 세면 `관리`·`번호` 가 다른 문맥에도 나오므로 **줄 단위**로 본다.
#[test]
fn the_next_page_does_not_repeat_the_recovered_line() {
    let repeated = lines(&render_page_svg("8"))
        .into_iter()
        .filter(|line| line.contains("관리번호") && line.contains("청약신청주택"))
        .count();
    assert_eq!(
        repeated, 0,
        "9쪽이 8쪽의 마지막 줄을 다시 그렸다 — 확장이 중복을 만들었다"
    );
}

/// 되살린 줄은 8쪽에 정확히 한 번만 있어야 한다.
#[test]
fn the_recovered_line_appears_exactly_once_on_its_own_page() {
    let owned = lines(&render_page_svg(PAGE_ARG))
        .into_iter()
        .filter(|line| line.contains("관리번호") && line.contains("청약신청주택"))
        .count();
    assert_eq!(
        owned, 1,
        "8쪽의 `[청약신청주택] … 관리번호 …` 줄이 1회가 아니다 (#5862)"
    );
}
