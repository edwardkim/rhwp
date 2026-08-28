//! [#6313] 표 칸 안 그림이 자기 높이만큼 한 번 더 전진하지 않는다.
//!
//! `samples/issue6313/microbe_bank_cell_picture.hwpx` 는 원본(농촌진흥청 156624779,
//! 2.46MB)의 **구조 보존 슬라이스**다 — BinData 이미지만 8×8 PNG 로 바꿔 81KB 로
//! 줄였고, 조판은 `<hp:sz>` 선언 크기를 쓰므로 좌표가 원본과 같다.
//!
//! **근인 — 자기 변위 지문에서 오프셋이 빠졌다.**
//!
//! `#6110` 은 "빈 문단이 host 인 셀 float 의 저장 `vertical_pos` 가 **그 그림의 자기
//! 변위**면 흐름 오프셋이 아니다"를 세웠다. 그 지문을 `|vpos − 그림 높이| ≤ 1px` 로
//! 적었는데, 한글이 밀어 둔 줄의 vpos 는 **그림 바닥**을 가리키므로 세로 오프셋이
//! 0 이 아니면 높이만으로는 맞지 않는다.
//!
//! | | 저장 vpos | 그림 높이 | 세로 오프셋 | 합 |
//! |---|---|---|---|---|
//! | 왼쪽 칸 | 15250 | 14530 | **720** | **15250** |
//! | 오른쪽 칸 | 17188 | 16899 | **289** | **17188** |
//!
//! 둘 다 **단위까지** 맞는다. 종전 지문은 각각 9.6px·3.9px 차로 빗나가 vpos 를 흐름
//! 오프셋으로 신뢰했고, 그림이 제 높이만큼 더 내려가 칸과 **용지 밖**으로 나갔다
//! (아래끝 1146.6px·1198.3px, 칸은 738.1..984.4).
//!
//! `#6175`·`#6280` 이 세운 "개체 흐름 높이 = 높이 + 오프셋" 과 같은 계약이다.
//!
//! 한글 2022 실측(문서 저장 버전 = 한글 2020 → 가장 가까운 설치본):
//! 5쪽 아래 두 그림 상단 560.03pt · 574.05pt.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6313/microbe_bank_cell_picture.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6313-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 5쪽 render tree JSON.
fn page5_render_tree() -> String {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            &sample(),
            "-p",
            "4",
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

/// `"Image"` 노드의 (y, y+h) 목록.
fn image_extents(json: &str) -> Vec<(f64, f64)> {
    let mut out = Vec::new();
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
            out.push((y, y + h));
        }
        rest = &tail[bbox_at + 6..];
    }
    out
}

#[test]
fn cell_pictures_stay_inside_the_page() {
    let json = page5_render_tree();
    let images = image_extents(&json);
    assert!(images.len() >= 4, "5쪽 그림 4개를 기대했다: {images:?}");

    // A4 세로 = 1122.5px. 종전에는 아래 두 그림이 1146.6·1198.3 으로 용지 밖이었다.
    for (top, bottom) in &images {
        assert!(
            *bottom <= 1122.5,
            "그림이 용지 아래로 나갔다: {top:.1}..{bottom:.1}"
        );
    }
}

#[test]
fn lower_cell_pictures_match_the_hangul_oracle() {
    let json = page5_render_tree();
    let mut tops: Vec<f64> = image_extents(&json).into_iter().map(|(t, _)| t).collect();
    tops.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(tops.len() >= 4, "그림 4개를 기대했다: {tops:?}");

    // 한글 2022: 560.03pt = 746.7px, 574.05pt = 765.4px.
    // 종전: 973.0px·952.9px (각각 +169.7pt·+140.6pt 이탈).
    let (third, fourth) = (tops[2], tops[3]);
    assert!(
        (third - 746.7).abs() <= 8.0,
        "아래 왼쪽 그림이 한글(746.7px)에서 벗어났다: {third:.1}"
    );
    assert!(
        (fourth - 765.4).abs() <= 8.0,
        "아래 오른쪽 그림이 한글(765.4px)에서 벗어났다: {fourth:.1}"
    );
}
