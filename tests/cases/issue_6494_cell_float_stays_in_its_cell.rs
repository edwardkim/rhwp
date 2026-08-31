//! [#6494] 칸 앵커 그림은 자기 칸 밖으로 나가지 않는다.
//!
//! **근인은 앵커 계약이었다.** 한 문단이 float 그림을 둘 이상 달고 있으면 그것들은
//! **나란히 놓이는 한 무리**이고, 칸 valign 이나 저장 vpos 가 아니라 문단 앵커
//! (`para_y_before_compose`)에 함께 걸린다. 칸 봉쇄는 그 위에 남긴 안전망이다.
//!
//! **증상.** `156489219` 5쪽에서 한 문단에 든 float 그림 두 장이 서로 반대로 찢어진다.
//! 한 장은 칸 **위로 246px** 올라가 앞 그림 자리에 겹쳐 보이지 않게 되고, 다른 한 장은
//! 칸 **아래로** 빠져 용지 밖 `51.4pt` 까지 나가 캡션·주석·쪽번호를 덮는다.
//!
//! **선언.**
//!
//! ```text
//! p[1] (빈 문단, ctrls=2)  ls[0] vpos=27069
//!   ctrl[0] 지도 ①  wrap=Square        vert=Para(off=-18576)  horz=Column(off=1130)
//!   ctrl[1] 지도 ②  wrap=TopAndBottom  vert=Para(off=0)       horz=Column(off=25383)
//! ```
//!
//! **기계.** 두 그림 다 `displaced_empty_line_para` 로 앵커가 칸 콘텐츠 상단(590.5px)
//! 으로 리셋되는데, 그 뒤가 갈린다 — 지도 ①은 거기에 음수 오프셋 `-247.7px` 를 더해
//! 칸 위로 나가고, 지도 ②는 이후 단계가 저장 `vpos`(360.9px) 자리로 재배치해 칸 아래로
//! 나간다.
//!
//! **오라클 — 한글 2022** (저장 버전 `hancom-office-2018` 미설치 → 최근접 설치본,
//! `producer=Hancom PDF 1.3.0.547`). 한글은 두 장을 **소수점까지 같은 `y=516.04`** 에
//! 나란히 놓는다.
//!
//! **쪽 전체가 25~44pt 드리프트해 있어 절대 좌표로는 모델이 안 갈린다.** 표 상단 기준
//! 상대로 재면 갈린다 — 표 높이는 `383.59` vs `384.09` 로 맞고 상단만 `44.1pt` 어긋난다.
//!
//! | 5쪽 (표 상단 기준 상대) | 종전 | **수정 후** | 한글 2022 |
//! |---|---:|---:|---:|
//! | 지도 ① | −194.45 | **108.78** | 118.68 |
//! | 지도 ② | 272.10 | **118.88** | 118.68 |
//! | `layout-anomaly` off-canvas | 1 | **0** | — |
//!
//! 지도 ②는 `0.20pt` 로 맞는다. 지도 ①의 남은 `9.9pt` 는 **크롭 미적용**이라는 별개
//! 결함이다 — 노드는 `118.9`(0.22pt 일치)인데, 크롭 사각형으로 확대만 하고 자르지 않아
//! 그려진 이미지가 상자 위로 `10.1pt` 삐져나온다(폭도 `1.335배 = 1/크롭비율`).
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

/// 재현 문서. 저장소 밖(10k 코퍼스)이라 없으면 시험을 건너뛴다.
const DOC: &str = "C:/Users/planet/hwpdocs/korea_downloads/환경부/156489219_환경위성-인공지능으로 파악한 지상 미세먼지 영상 공개(12.30).hwp";

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6494-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 5쪽 render tree 에서 `(칸 상자, 그 칸 안 그림들)` 을 모은다 (px).
///
/// 칸 안 그림은 **같은 문단(`pi`)에 매달린 것들**이라 그 지문으로 고른다 — 좌표
/// 밴드로 고르면 칸 밖 그림(누리집 화면)까지 걸린다.
fn page5_cell_and_images() -> Option<(f64, f64, Vec<(f64, f64)>)> {
    if !Path::new(DOC).exists() {
        return None;
    }
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            DOC,
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
        .expect("5쪽 render tree JSON 이 없다");
    let json = std::fs::read_to_string(path).unwrap();

    // 지도를 담은 칸은 이 쪽에서 가장 큰 Cell 이다.
    let mut best: Option<(f64, f64)> = None;
    let mut images: Vec<(i64, f64, f64)> = Vec::new();
    for seg in json.split("{\"type\"").skip(1) {
        let num = |key: &str| -> Option<f64> {
            seg.split(&format!("\"{key}\":")).nth(1).and_then(|r| {
                r.trim_start()
                    .split(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
                    .find(|s| !s.is_empty())
                    .and_then(|s| s.parse::<f64>().ok())
            })
        };
        let head = &seg[..seg.len().min(24)];
        let (Some(y), Some(h)) = (num("y"), num("h")) else {
            continue;
        };
        if head.contains("\"Cell\"") && h > 400.0 && best.is_none_or(|(_, bh)| h > bh) {
            best = Some((y, h));
        }
        if head.contains("\"Image\"") {
            images.push((num("pi").map(|v| v as i64).unwrap_or(-1), y, h));
        }
    }
    // 칸 안 그림들은 같은 문단(`pi`)에 매달려 있다 — 그림이 가장 많은 `pi` 가 그 문단이다.
    // (이 쪽은 위성영상 전시 1장 + 지도 2장 = 3장이 같은 `pi`, 누리집 화면만 다른 `pi`.)
    let mut in_cell: Vec<(f64, f64)> = Vec::new();
    for (pi, _, _) in &images {
        let same: Vec<_> = images
            .iter()
            .filter(|(q, _, _)| q == pi && *q >= 0)
            .map(|(_, y, h)| (*y, *h))
            .collect();
        if same.len() > in_cell.len() {
            in_cell = same;
        }
    }
    best.map(|(y, h)| (y, h, in_cell))
}

/// 칸 안 그림이 칸 상자를 위·아래로 벗어나지 않는다.
///
/// 종전에는 한 장이 칸 위로 `246px`, 다른 한 장이 칸 아래로 나가 용지 밖까지 갔다.
#[test]
fn cell_anchored_pictures_stay_inside_their_cell() {
    let Some((cell_y, cell_h, images)) = page5_cell_and_images() else {
        eprintln!("[#6494] 재현 문서가 없어 건너뛴다: {DOC}");
        return;
    };
    let cell_bottom = cell_y + cell_h;
    assert!(
        images.len() >= 3,
        "칸 안 그림 3장(위성영상 + 지도 2장)을 찾아야 한다: {images:?}"
    );
    for (y, h) in &images {
        assert!(
            *y >= cell_y - 0.5,
            "칸 앵커 그림이 칸 위로 나갔다: y={y:.2} 칸 상단={cell_y:.2}"
        );
        assert!(
            y + h <= cell_bottom + 0.5,
            "칸 앵커 그림이 칸 아래로 나갔다: 아래끝={:.2} 칸 하단={cell_bottom:.2}",
            y + h
        );
    }
}

/// 나란히 무리의 두 그림이 **같은 세로 자리**에 놓인다.
///
/// 한글 2022 는 둘을 소수점까지 같은 `y=516.04` 에 둔다. 종전에는 `247.05` 와 `713.60`
/// 으로 `466pt` 벌어져 있었다.
///
/// 노드 기준으로 비교한다 — 지도 ①의 그려진 상자는 크롭 미적용 때문에 `10.1pt` 위로
/// 삐져나오므로(별개 결함) 허용치를 `12pt` 로 둔다.
#[test]
fn side_by_side_floats_share_one_vertical_band() {
    let Some((_, _, images)) = page5_cell_and_images() else {
        eprintln!("[#6494] 재현 문서가 없어 건너뛴다: {DOC}");
        return;
    };
    let tops: Vec<f64> = images.iter().map(|(y, _)| *y).collect();
    let lo = tops.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = tops.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    // 위성영상 전시 그림은 무리 위에 따로 있으므로, 무리 두 장만 본다 —
    // 가장 아래 두 장이 지도 쌍이다.
    let mut sorted = tops.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let pair = &sorted[sorted.len() - 2..];
    assert!(
        (pair[1] - pair[0]).abs() <= 16.0,
        "나란히 무리의 두 그림이 같은 밴드에 있어야 한다: {pair:?} (전체 {lo:.1}..{hi:.1})"
    );
}

/// `layout-anomaly` 의 용지 밖 신고가 0 이다.
///
/// 종전 `[OFF-CANVAS] page 4 45.51px Table8`.
#[test]
fn page5_has_no_off_canvas_report() {
    if !Path::new(DOC).exists() {
        eprintln!("[#6494] 재현 문서가 없어 건너뛴다: {DOC}");
        return;
    }
    let out = Command::new(rhwp_bin())
        .args(["layout-anomaly", DOC])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let off_canvas: Vec<_> = stdout
        .lines()
        .filter(|l| l.contains("[OFF-CANVAS]"))
        .collect();
    assert!(
        off_canvas.is_empty(),
        "용지 밖 신고가 없어야 한다: {off_canvas:?}"
    );
}
