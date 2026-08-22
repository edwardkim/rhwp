//! [#5854] 저장 LINE_SEG 사다리가 통짜 합성값인 문서의 줄 진행 계약.
//!
//! `samples/hwpx/hwpx-02.hwpx` 는 122 문단 전부가 같은 `<hp:lineseg>` 튜플
//! (`vertsize=1000 textheight=1000 baseline=850 spacing=600`)을 갖고 `vertpos` 가
//! 처음부터 끝까지 1600 씩 증가한다. 문서의 실제 글자 크기는 2pt~15pt 로 갈리므로
//! 그 사다리는 실측 조판이 아니다. 한글(정답지 `pdf/hwpx/hwpx-02-2022.pdf`)의 줄
//! 진행은 `advance = 최대글자크기 × 줄간격퍼센트 / 100` 하나로 설명된다.
//!
//! 이 테스트가 잠그는 것은 **두 가지를 함께** 지키는 것이다.
//!
//! 1. 빈 쪽이 끼어들지 않는다 (쪽수 6, 어느 쪽도 비어 있지 않다).
//! 2. 문단별 advance 가 한글 모델과 같다 — 쪽수만 맞추고 줄 간격을 틀리게 만드는
//!    보정(이슈 본문의 "시도했다가 접은 수정")을 되풀이하지 않는다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/hwpx/hwpx-02.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-5854-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn page_count() -> u32 {
    let out = Command::new(rhwp_bin())
        .args(["info", &sample()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.lines()
        .find_map(|line| line.strip_prefix("페이지 수:"))
        .and_then(|value| value.trim().parse::<u32>().ok())
        .unwrap_or_else(|| panic!("페이지 수를 읽지 못했다:\n{text}"))
}

/// `pi` → (쪽 번호, 문단 첫 줄의 y) — `export-render-tree` 산출에서 수집.
fn paragraph_tops() -> Vec<(u32, usize, f64)> {
    let dir = temp_dir("rt");
    let out = Command::new(rhwp_bin())
        .args(["export-render-tree", &sample(), "-o", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");

    let mut tops: Vec<(u32, usize, f64)> = Vec::new();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    files.sort();
    for (index, file) in files.iter().enumerate() {
        let page = index as u32 + 1;
        let text = std::fs::read_to_string(file).unwrap();
        let root: serde_json::Value = serde_json::from_str(&text).unwrap();
        collect_text_lines(&root, page, &mut tops);
    }
    let _ = std::fs::remove_dir_all(&dir);
    tops
}

fn collect_text_lines(node: &serde_json::Value, page: u32, out: &mut Vec<(u32, usize, f64)>) {
    match node {
        serde_json::Value::Object(map) => {
            if map.get("type").and_then(|t| t.as_str()) == Some("TextLine") {
                if let (Some(pi), Some(y)) = (
                    map.get("pi").and_then(|v| v.as_u64()),
                    map.get("bbox")
                        .and_then(|b| b.get("y"))
                        .and_then(|v| v.as_f64()),
                ) {
                    if !out.iter().any(|(_, seen, _)| *seen == pi as usize) {
                        out.push((page, pi as usize, y));
                    }
                }
            }
            for value in map.values() {
                collect_text_lines(value, page, out);
            }
        }
        serde_json::Value::Array(items) => {
            for value in items {
                collect_text_lines(value, page, out);
            }
        }
        _ => {}
    }
}

/// 문단 `pi` 의 줄 진행 = 다음 문단의 첫 줄 y − 자기 첫 줄 y (같은 쪽일 때만).
fn advance(tops: &[(u32, usize, f64)], pi: usize) -> Option<f64> {
    let this = tops.iter().find(|(_, p, _)| *p == pi)?;
    let next = tops.iter().find(|(_, p, _)| *p == pi + 1)?;
    (this.0 == next.0).then_some(next.2 - this.2)
}

#[test]
fn no_blank_page_is_inserted() {
    assert_eq!(
        page_count(),
        6,
        "합성 사다리를 조판 근거로 쓰면 3쪽 하단이 29px 모자라 빈 4쪽이 끼어든다"
    );

    let dir = temp_dir("txt");
    let out = Command::new(rhwp_bin())
        .args(["export-text", &sample(), "-o", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let mut pages: Vec<(String, usize)> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|ext| ext == "txt"))
        .map(|p| {
            let name = p.file_name().unwrap().to_string_lossy().into_owned();
            let body = std::fs::read_to_string(&p).unwrap();
            (
                name,
                body.split_whitespace().collect::<String>().chars().count(),
            )
        })
        .collect();
    pages.sort();
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(pages.len(), 6, "쪽 텍스트 파일 수: {pages:?}");
    for (name, chars) in &pages {
        assert!(*chars > 0, "{name} 이 비어 있다 — 빈 쪽이 끼어들었다");
    }
    // 정답지(한글 2022) 3~5쪽의 글자 수와 같아야 한다.
    assert_eq!(pages[2].1, 217, "3쪽 글자 수");
    assert_eq!(pages[3].1, 89, "4쪽 글자 수");
    assert_eq!(pages[4].1, 128, "5쪽 글자 수");
}

#[test]
fn paragraph_advances_match_hangul_line_spacing() {
    let tops = paragraph_tops();
    // (문단, 한글 모델 advance px = 최대글자크기 × 줄간격퍼센트 / 100)
    //
    // 앞의 둘은 이 결함의 원인(4pt·2pt 문단이 저장값 21.33px 을 그대로 썼다),
    // 뒤의 넷은 이슈가 기록한 실패한 수정이 틀리게 만들었던 줄이다. 두 묶음을
    // 한 테스트에서 함께 잠근다.
    let expected: [(usize, f64); 10] = [
        (35, 4.0 / 75.0 * 100.0 * 1.40),  // 4pt / 140%
        (36, 2.0 / 75.0 * 100.0 * 1.40),  // 2pt / 140%
        (37, 14.0 / 75.0 * 100.0 * 1.20), // 14pt / 120%
        (42, 15.0 / 75.0 * 100.0 * 1.30), // 15pt / 130%
        (43, 10.0 / 75.0 * 100.0 * 1.20), // 10pt / 120%
        (47, 10.0 / 75.0 * 100.0 * 1.20), // 10pt / 120% — 실패한 수정이 10.7px 로 만들었다
        (50, 15.0 / 75.0 * 100.0 * 1.40), // 15pt / 140% — 이미 맞던 줄
        (53, 15.0 / 75.0 * 100.0 * 1.30), // 15pt / 130% — 이미 맞던 줄
        (57, 15.0 / 75.0 * 100.0 * 1.40), // 15pt / 140% — 이미 맞던 줄
        (69, 10.0 / 75.0 * 100.0 * 1.10), // 10pt / 110%
    ];
    for (pi, model) in expected {
        let actual = advance(&tops, pi)
            .unwrap_or_else(|| panic!("pi{pi} 의 줄 진행을 잴 수 없다 (쪽 경계?)"));
        assert!(
            (actual - model).abs() <= 0.11,
            "pi{pi}: 한글 {model:.2}px 인데 rhwp {actual:.2}px — 쪽수만 맞추고 \
             줄 간격을 틀리게 만드는 보정이다"
        );
    }
}
