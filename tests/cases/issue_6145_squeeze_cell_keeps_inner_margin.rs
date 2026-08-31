//! [#6145] `lineWrap="SQUEEZE"`("한 줄로 입력") 칸은 넘쳐도 안 여백을 깎지 않는다.
//!
//! `samples/issue6145/worklife_balance_index_156607916.hwpx` 는 고용노동부
//! 「2022년 기준 지역별 일･생활 균형 지수 발표」 보도자료 원본(HWPX, 272KB)이다.
//!
//! **증상.** 6쪽 마지막 열의 글자가 오른쪽 괘선을 넘어간다. 사용자 신고는
//! "`담당조직 형태 만점(1점)+`, `기업지원 명시 만점(1점)` 이 가려진다"였다.
//!
//! **근인 — 넘침 방어가 SQUEEZE 와 정반대로 동작한다.**
//!
//! `composer::shrunk_cell_horizontal_padding` 은 줄이 안쪽 폭을 `1.15` 배 넘으면
//! 좌우 안 여백을 **1px 까지 깎아** 자리를 만든다. 그런데 한/글이 SQUEEZE 칸에서
//! 하는 일은 반대다 — 여백은 그대로 두고 **자간을 줄여** 글자를 밀어 넣는다.
//!
//! 이 칸이 그 차이를 그대로 드러낸다. 선언은 `cellSz width=9906`,
//! `hasMargin="0"` 이므로 표의 `inMargin left/right=283` 이 적용돼야 하고, 저장
//! lineseg 3개가 전부 `horzsize=9340`(= 9906 − 283 − 283)으로 **안쪽 폭을 못박아**
//! 두었다. 그런데 여백을 1px 로 깎으면 안쪽 폭이 `9906 − 150 = 9756`(97.56pt)이
//! 되어 자간이 덜 줄고, 줄이 괘선 밖으로 나간다.
//!
//! **오라클 — 한글 2022** (문서 저장 버전 `appVersion major=10` = 한글 2018/2020,
//! 미설치 → 최근접 설치본. `producer=Hancom PDF 1.3.0.550`).
//!
//! | 6쪽 마지막 열 | 종전 | **수정 후** | 한글 2022 |
//! |---|---:|---:|---:|
//! | 글자 시작 x | 438.98 | **441.06** | 441.00 |
//! | `일･생활 균형 조례 제정,` 우단 | 534.67 | 533.14 | 534.37 |
//! | `기업지원 명시 만점(1점)` 우단 | 537.50 | **536.28** | 534.44 |
//! | `담당조직 형태 만점(1점)+` 우단 | **538.48 (괘선 밖)** | **537.28** | 534.41 |
//!
//! 괘선은 `537.79`(rhwp) / `537.43`(한글)이다. 종전에는 마지막 줄이 괘선을
//! `+0.69pt` 넘었고, 수정 후에는 세 줄 모두 안쪽에서 끝난다.
//!
//! 남는 우단 차(한글 대비 최대 2.9pt)는 조판 폭과 페인트 폭이 갈리는 **별개 축**
//! 이다(`#6303` 에서 같은 발산을 확인했다). 이 시험은 이 이슈가 지목한 축 —
//! **안 여백 보존** — 만 잠근다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

/// 6쪽 마지막 열 칸의 좌·우 경계 (px, 96dpi).
const CELL_LEFT_PX: f64 = 584.30;
const CELL_RIGHT_PX: f64 = 716.40;
/// 표 `inMargin left/right = 283 HWPUNIT` = 3.773px. 여백이 깎이면 1px 로 떨어진다.
const DECLARED_INNER_MARGIN_PX: f64 = 283.0 / 7200.0 * 96.0;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6145/worklife_balance_index_156607916.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6145-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 6쪽 render tree JSON.
fn page6_render_tree() -> String {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            &sample(),
            "-p",
            "5",
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

/// 이 칸 안의 모든 `TextRun` 을 `(y, 좌단, 우단, 텍스트)` 로 훑는다 (px).
fn cell_runs(json: &str) -> Vec<(f64, f64, f64, String)> {
    let mut out = Vec::new();
    let mut rest = json;
    while let Some(at) = rest.find("\"TextRun\"") {
        let tail = &rest[at..];
        // 이 노드의 JSON 만 본다 — 다음 노드 앞에서 끊지 않으면 이웃 run 의 텍스트가
        // 걸린다 (#6443 과 같은 함정).
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
        if let (Some(x), Some(y), Some(w)) = (num("x"), num("y"), num("w")) {
            if x >= CELL_LEFT_PX - 1.0 && x <= CELL_RIGHT_PX {
                out.push((y, x, x + w, text));
            }
        }
        rest = &tail[9..];
    }
    out
}

/// `needle` 을 담은 줄(같은 `y` 의 run 들을 합친 것)의 `(좌단, 우단)`.
fn line_span_containing(json: &str, needle: &str) -> Option<(f64, f64)> {
    let runs = cell_runs(json);
    let y = runs.iter().find(|r| r.3.contains(needle))?.0;
    let same: Vec<_> = runs.iter().filter(|r| (r.0 - y).abs() < 1.0).collect();
    Some((
        same.iter().map(|r| r.1).fold(f64::INFINITY, f64::min),
        same.iter().map(|r| r.2).fold(f64::NEG_INFINITY, f64::max),
    ))
}

/// SQUEEZE 칸의 글자는 선언된 안 여백만큼 안쪽에서 시작한다.
///
/// 종전에는 넘침 방어가 여백을 `1px` 로 깎아 글자가 `585.30`(칸 좌단 + 1.0)에서
/// 시작했다. 한/글은 여백을 지키므로 `588.07`(칸 좌단 + 3.77) 이다.
#[test]
fn squeeze_cell_text_starts_inside_the_declared_inner_margin() {
    let json = page6_render_tree();
    let (left, _) = line_span_containing(&json, "담당조직 형태 만점")
        .expect("6쪽에서 `담당조직 형태 만점` 줄을 찾지 못했다");
    let inset = left - CELL_LEFT_PX;
    assert!(
        inset >= DECLARED_INNER_MARGIN_PX - 0.5,
        "SQUEEZE 칸은 선언된 안 여백({DECLARED_INNER_MARGIN_PX:.2}px)을 지켜야 한다: \
         좌단 {left:.2}px, 들여쓴 폭 {inset:.2}px"
    );
}

/// SQUEEZE 칸의 줄 폭이 저장 lineseg 가 못박은 안쪽 폭 가까이로 조여진다.
///
/// 저장 lineseg 는 `horzsize=9340`(= 124.53px)이다. 여백을 1px 로 깎으면 목표가
/// `130.10px` 로 헐거워져 자간이 덜 줄고, 그 결과 `담당조직 형태 만점(1점)+` 이
/// PDF 우단 `538.48pt` 로 괘선(`537.79pt`)을 `+0.69pt` 넘었다.
///
/// 상한을 선언 폭 그대로가 아니라 `+3px` 로 두는 이유: 압축이 수렴한 뒤에도 조판
/// 폭과 페인트 폭이 ~1.5% 갈리는 **별개 축**이 남아 있다(`#6303`). 이 시험이
/// 잠그는 것은 "목표 폭이 헐거워지지 않는다"이지 그 잔여 축이 아니다.
#[test]
fn squeeze_cell_line_width_tracks_the_stored_inner_width() {
    /// 저장 lineseg `horzsize = 9340 HWPUNIT`.
    const STORED_INNER_WIDTH_PX: f64 = 9340.0 / 7200.0 * 96.0;
    let json = page6_render_tree();
    for needle in ["담당조직 형태 만점", "기업지원 명시 만점"] {
        let (left, right) = line_span_containing(&json, needle)
            .unwrap_or_else(|| panic!("6쪽에서 `{needle}` 줄을 찾지 못했다"));
        let width = right - left;
        assert!(
            width <= STORED_INNER_WIDTH_PX + 3.0,
            "`{needle}` 줄이 저장 안쪽 폭({STORED_INNER_WIDTH_PX:.2}px) 가까이로 조여져야 한다: \
             폭 {width:.2}px (좌 {left:.2} 우 {right:.2})"
        );
    }
}
