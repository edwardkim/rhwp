//! [#5886] 미주 흐름의 용지-밖 관문은 **용지에 남은 여백**을 넘어 허용하지 않는다.
//!
//! `samples/3-09월_교육_통합_2022.hwpx` 는 저장소 안 수능 모의고사 문서다. 해설이
//! 미주(endnote)로 들어 있고 본문이 2단이다.
//!
//! **증상.** 12쪽 0단 마지막 세 문단이 단 하단(1092.3)을 지나 1095.3 / 1113.3 /
//! 1131.4 에 놓이고, 마지막 `[알짜 풀이]` 는 용지(1122.5) 밖이다. 12쪽은 pi 650~680,
//! 13쪽은 681~ 이라 **다시 그려지지 않는다** — 글자가 그대로 사라진다.
//!
//! **근인 — 관문의 허용치가 용지 여백보다 크다.**
//!
//! `page_offcanvas_with_para` 는 이름 그대로 "용지 밖"을 막는 장치다. 시뮬레이션
//! (`simulate_endnote_column_bottom_y`)은 정확하다 — 실제 하단 998.6 / 1016.6 /
//! 1034.6 / 1052.7 을 그대로 맞춘다. 그런데 발동 조건이 단 하단 기준 **고정 56px**
//! (`ENDNOTE_PAGE_OFFCANVAS_GUARD_PX`)이라, 쪽 아래 여백이 그보다 좁으면
//! **여백과 56px 사이가 통째로 사각지대**가 된다.
//!
//! ```text
//! 단 하단 1092.3 → 용지 1122.5 = 실제 여백 30.2px
//! 관문 허용치 56.0px            → 30.2 ~ 56.0 구간은 용지 밖인데도 침묵
//! ```
//!
//! 그래서 관문은 시뮬 하단이 1070.7 이 되는 ep=15 에서야 발동했고, 그 전에 세
//! 문단이 이미 놓였다.
//!
//! **수정.** 허용치를 `min(56px, 용지에 남은 여백)` 으로 묶는다. 여백이 56px 이상인
//! 문서는 종전과 같고, 좁은 쪽에서만 조인다.
//!
//! **결과.** `rhwp layout-anomaly` 기준 이 문서의 off-canvas `10 → 6`,
//! overflow `23 → 16`, 쪽수는 23 그대로다. 12쪽 0단의 용지 밖 줄은 0 이 된다.
//!
//! 남는 6건(8·9·12단1·18·21쪽)은 이 관문이 켜지지 않는 다른 갈래라 별건이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

/// 용지 높이 (px, 96dpi).
const PAGE_HEIGHT_PX: f64 = 1122.5;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/3-09월_교육_통합_2022.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-5886-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 0-기반 `page` 의 render tree JSON.
fn render_tree(page: usize) -> String {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            &sample(),
            "-p",
            &page.to_string(),
            "-o",
            &dir.to_string_lossy(),
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let path = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "json"))
        .expect("render tree JSON 이 없다");
    std::fs::read_to_string(path).unwrap()
}

/// 모든 `TextRun` 의 `(y, h, text)`.
fn text_runs(json: &str) -> Vec<(f64, f64, String)> {
    let mut out = Vec::new();
    let mut rest = json;
    while let Some(at) = rest.find("\"TextRun\"") {
        let tail = &rest[at..];
        // 이 노드의 JSON 만 본다 — 다음 노드 앞에서 끊지 않으면 이웃 run 이 걸린다.
        let seg_end = tail[1..]
            .find("{\"type\"")
            .map(|i| i + 1)
            .unwrap_or(tail.len());
        let seg = &tail[..seg_end];
        let num = |key: &str| -> Option<f64> {
            seg.split(&format!("\"{key}\""))
                .nth(1)
                .and_then(|r| r.trim_start().strip_prefix(':'))
                .and_then(|r| {
                    r.trim_start()
                        .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
                        .find(|s| !s.is_empty())
                        .and_then(|s| s.parse::<f64>().ok())
                })
        };
        let text = seg
            .split("\"text\"")
            .nth(1)
            .and_then(|r| r.trim_start().strip_prefix(':'))
            .and_then(|r| {
                let r = r.trim_start().strip_prefix('"')?;
                r.find('"').map(|e| r[..e].to_string())
            })
            .unwrap_or_default();
        if let (Some(y), Some(h)) = (num("y"), num("h")) {
            out.push((y, h, text));
        }
        rest = &tail[9..];
    }
    out
}

/// 12쪽에는 용지 밖으로 나가는 글자가 없다.
///
/// 종전에는 `[알짜 풀이]` 가 `y=1131.4`(용지 1122.5 밖)에 놓였고, 13쪽에도 없어
/// 그대로 소실됐다.
#[test]
fn page12_has_no_text_outside_the_paper() {
    let json = render_tree(11);
    let outside: Vec<_> = text_runs(&json)
        .into_iter()
        .filter(|(y, h, t)| y + h > PAGE_HEIGHT_PX + 0.5 && !t.trim().is_empty())
        .collect();
    assert!(
        outside.is_empty(),
        "12쪽에 용지({PAGE_HEIGHT_PX}px) 밖 글자가 남았다: {outside:?}"
    );
}

/// 이슈가 지목한 문장이 12쪽 안에 그려진다.
///
/// 종전 `y`: `이상에서 옳은 것은…` 1095.3 · `[알짜 풀이]` 1131.4 (둘 다 단 하단 밖,
/// 뒤엣것은 용지 밖). 관문이 제때 단을 넘기면 둘 다 1단 위쪽으로 옮겨진다.
#[test]
fn lost_solution_lines_are_drawn_inside_page12() {
    let json = render_tree(11);
    let runs = text_runs(&json);
    for needle in ["이상에서 옳은 것은", "[알짜 풀이]"] {
        let hit = runs
            .iter()
            .find(|(_, _, t)| t.contains(needle))
            .unwrap_or_else(|| panic!("12쪽에서 `{needle}` 을 찾지 못했다"));
        assert!(
            hit.0 + hit.1 <= PAGE_HEIGHT_PX,
            "`{needle}` 이 용지 안에서 끝나야 한다: y={:.1} h={:.1}",
            hit.0,
            hit.1
        );
    }
}
