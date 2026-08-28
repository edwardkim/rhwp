//! [#6284] 그림 캡션(`hp:caption`)이 본문 그림 경로에서도 그려지고 띠를 예약한다.
//!
//! `samples/issue6284/child_policy_top_caption_charts.hwpx` 는 원본
//! (보건복지부 156562502, 7.0MB)의 **구조 보존 슬라이스**다 — BinData 이미지만
//! 8×8 PNG 로 바꿔 152KB 로 줄였고, 조판은 `<hp:sz>` 선언 크기를 쓰므로 좌표가
//! 원본과 같다(6쪽 캡션 192.06/192.64 · 그림 상단 208.17 로 실측 일치).
//!
//! **근인** — 본문 그림을 실제로 그리는 `layout_picture_full` 이 `picture.caption`
//! 을 한 번도 참조하지 않았다. 캡션 기계(띠 예약 + `layout_caption` 호출)는 형제
//! 함수 `layout_body_picture` 에만 있었고 이 문서는 그 경로를 타지 않는다.
//! 그래서 `side="TOP"` 캡션 17개가 통째로 사라지고, 캡션 띠가 없어진 만큼
//! 그림이 위로 올라왔다.
//!
//! **실측 (물리 6쪽, pt)**
//!
//! | | 캡션 | 그림 상단 |
//! |---|---|---|
//! | 종전 | 없음 | 191.51 |
//! | 수정 | 192.06 / 192.64 | **208.17** |
//! | 한글 2024 | 191.2 | **207.13** |
//!
//! 캡션↔그림 간격은 17.59pt 로 한글과 정확히 같다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6284/child_policy_top_caption_charts.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6284-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 물리 6쪽(0-base 5)의 render tree JSON.
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

/// 높이가 `min_h` 이상인 Image 노드의 y 목록.
fn image_tops(json: &str, min_h: f64) -> Vec<f64> {
    let mut tops = Vec::new();
    let mut rest = json;
    while let Some(at) = rest.find("\"Image\"") {
        let tail = &rest[at..];
        let Some(bbox_at) = tail.find("\"bbox\"") else {
            break;
        };
        let seg = &tail[bbox_at..(bbox_at + 200).min(tail.len())];
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
        if let (Some(y), Some(h)) = (num("y"), num("h")) {
            if h >= min_h {
                tops.push(y);
            }
        }
        rest = &tail[bbox_at + 6..];
    }
    tops
}

#[test]
fn top_caption_text_is_rendered() {
    let json = page6_render_tree();
    // 종전에는 이 두 캡션이 전혀 방출되지 않았다.
    assert!(
        json.contains("아동 삶의 만족도"),
        "TOP 캡션 '아동 삶의 만족도 (15세 아동)' 이 방출되지 않았다"
    );
    assert!(
        json.contains("자살률"),
        "TOP 캡션 '아동·청소년 자살률' 이 방출되지 않았다"
    );
}

#[test]
fn picture_sits_below_its_reserved_caption_band() {
    let json = page6_render_tree();
    let mut tops = image_tops(&json, 100.0);
    tops.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(tops.len() >= 2, "6쪽 차트 그림을 찾지 못했다: {tops:?}");

    // 한글 2024: 좌상 그림 상단 207.13pt = 276.2px. 캡션 띠(≈16px)를 삼키면
    // 191.51pt = 255.3px 로 올라간다.
    let top = tops[0];
    assert!(
        (top - 276.2).abs() <= 6.0,
        "그림이 캡션 띠를 삼키고 위로 올라왔다: y={top:.1}px (기대 276.2px = 한글 207.13pt)"
    );
}
