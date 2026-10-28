//! [#6465] 글상자 인라인 개체를 **개체가 놓인 줄** 기준으로 배치한다 — 첫 줄이 아니다.
//!
//! `samples/issue6465/press_release_footer_logos.hwpx` 는 과학기술정보통신부
//! 보도자료(156677575, 101KB)의 구조 보존 슬라이스다 — BinData 를 8×8 그림으로
//! 바꿔 0.06MB 로 줄였다(이 결함은 그림 내용과 무관하다).
//!
//! **증상.** 13쪽 하단 `정책브리핑`·`공공누리` 로고 두 개가 오른쪽으로 16.7pt,
//! 위로 25.1pt 어긋나 오른쪽 로고가 본문 우단(538.59)을 **11.6pt 넘는다.**
//! 같은 쪽 글자는 0.1pt 안에서 맞는다 — 어긋나는 건 이 두 그림뿐이다.
//!
//! **근인 — 배치가 문단의 첫 줄을 전제한다.**
//!
//! 저장 사다리는 이 문단을 두 줄로 적어 두었고 개체는 둘째 줄 몫이다.
//!
//! ```text
//! linesegarray: tp=0  vertpos=0     vertsize=2388
//!               tp=9  vertpos=2700  vertsize=3163
//! tac_controls: [(pos 9, ci 0), (pos 9, ci 1)]   ← 둘 다 둘째 줄 시작
//! composed:     line0 = 공백 9자 / line1 = 공백 3자
//! ```
//!
//! 방출 경로(`emit_line_runs`)는 **이미 둘째 줄에 옳게 싣고 있었다**(프로브로 확인:
//! `line=0 tacs=[]` / `line=1 tacs=[(0,0),(0,1)]`). 그런데
//! `shape_layout::layout_textbox_content` 의 배치 계산 **네 곳**이 첫 줄을 봤다.
//!
//! | | 첫 줄 기준(종전) | 소유 줄 기준(수정) |
//! |---|---|---|
//! | 정렬 기준 폭 | 공백 9자 = 90px | 공백 3자 = 30px |
//! | 말미 공백 | 첫 줄 | 소유 줄 |
//! | 개체 앞 전진 | `text_cursor = 0` (앞 줄 글자까지 가산) | 소유 줄 `char_start` |
//! | y | `para_start_y` / `inline_y` (첫 줄) | 소유 줄 `vertical_pos` |
//!
//! 정렬 기준 폭이 부풀어 여유가 **음수**(`inner 278.84` vs `total 300.80`)가 되면
//! 오른쪽 정렬이 무너지고, 개체 앞 전진이 앞 줄 공백 9자를 또 더한다.
//!
//! **오라클 — 한글 2022.**
//!
//! | 그림 | 종전 | **수정 후** | 한글 2022 |
//! |---|---|---|---|
//! | 정책브리핑 | `(392.09, 577.85)` | **`(375.62, 604.85)`** | `(375.40, 602.94)` |
//! | 공공누리 | `(460.69, 570.10)` | **`(444.22, 597.09)`** | `(443.88, 596.35)` |
//!
//! 오른쪽 로고 우단 `550.19` → **`533.72`**(한글 `533.35`), 본문 우단 `538.59` 안.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

/// 본문 우단 (pt) — 구역 좌우 여백 5669HU 기준.
const BODY_RIGHT_PT: f64 = 538.59;
/// 한글 2022 실측 — 두 로고의 좌상단 (pt).
const HANGUL_XY_PT: [(f64, f64); 2] = [(375.40, 602.94), (443.88, 596.35)];

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6465/press_release_footer_logos.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6465-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 13쪽 render tree JSON.
fn page13_render_tree() -> String {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            &sample(),
            "-p",
            "12",
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

/// `Image` 노드의 (x, y, x+w) — pt.
fn image_boxes(json: &str) -> Vec<(f64, f64, f64)> {
    const PX_TO_PT: f64 = 72.0 / 96.0;
    let mut out = Vec::new();
    let mut rest = json;
    while let Some(at) = rest.find("\"Image\"") {
        let tail = &rest[at..];
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
        if let (Some(x), Some(y), Some(w)) = (num("x"), num("y"), num("w")) {
            out.push((x * PX_TO_PT, y * PX_TO_PT, (x + w) * PX_TO_PT));
        }
        rest = &tail[7..];
    }
    out.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    out
}

/// 두 로고가 한글 실측 자리에 오고, 오른쪽 로고가 본문 우단 안에 든다.
#[test]
fn footer_logos_sit_on_their_own_line() {
    let boxes = image_boxes(&page13_render_tree());
    assert_eq!(boxes.len(), 2, "13쪽 로고 2개: {boxes:?}");

    for (got, want) in boxes.iter().zip(HANGUL_XY_PT.iter()) {
        assert!(
            (got.0 - want.0).abs() <= 1.5,
            "로고 x 가 한글 실측과 어긋난다: {boxes:?} vs {HANGUL_XY_PT:?}"
        );
        assert!(
            (got.1 - want.1).abs() <= 3.0,
            "로고 y 가 한글 실측과 어긋난다 (종전 −25.1pt): {boxes:?} vs {HANGUL_XY_PT:?}"
        );
    }

    let right = boxes.last().unwrap().2;
    assert!(
        right <= BODY_RIGHT_PT,
        "오른쪽 로고가 본문 우단({BODY_RIGHT_PT})을 넘었다: {right:.2}pt (종전 550.19)"
    );
}
