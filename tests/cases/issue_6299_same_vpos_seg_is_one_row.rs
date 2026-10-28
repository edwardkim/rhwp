//! [#6299] 같은 `vertical_pos` 를 가진 저장 seg 는 **한 줄의 가로 조각**이다.
//!
//! `samples/issue6299/forest_press_wrap_seg_pairs.hwpx` 는 원본(산림청 156518878)의
//! **구조 보존 슬라이스**다 — BinData 이미지만 8×8 PNG 로 바꿔 285KB → 72KB 로 줄였고,
//! 조판은 `<hp:sz>` 선언 크기를 쓰므로 좌표가 원본과 같다.
//!
//! **형상.** 머리글 표 첫 칸(`기관명`)은 어울림(`textWrap="SQUARE"`) 그림을 끼고 있어
//! 각 글줄이 그림 **좌·우 조각**으로 쪼개진다. 한글은 그 짝을 **같은 `vertpos`** 로
//! 적어 둔다 — 물리 줄 3개인데 seg 는 6개다.
//!
//! ```text
//! 문단0 (그림 + 빈 텍스트)  (vp0    vs1200 hp0)  (vp0    vs1200 hp11210)
//! 문단1 '문화체육관광'        (vp1920 vs1000 hp0)  (vp1920 vs1000 hp11210)
//!                          (vp3520 vs1000 hp0)  (vp3520 vs1000 hp11210)
//! ```
//!
//! **종전 결함.** 측정과 페인트가 **둘 다** 조각을 별개 줄로 셌다. 칸 content 가
//! 68.3px 대신 136.5px 이 되고, 칸이 `vertAlign="CENTER"` 라 내용이 27.9pt 아래로
//! 내려와 다음 행과 겹쳤다. 페인트도 101.3 / 122.6 / 143.9 / 165.3 으로 4단 적층해
//! 마지막 줄이 칸 바닥(164.5)을 넘었다.
//!
//! **⚠ 판별은 `column_start` 가 함께 달라야 한다.** 같은 `vertical_pos` 쌍에는 세 가지
//! 뜻이 있고(좌우분할 · 쪽 리셋 · 중복), **이중 계상이 실제로 드러나는 것은 좌우분할
//! 하나뿐**이다 — 10k 모집단 실측에서 확인된 구분이다. cs·sw 가 같은 쌍은 렌더 결함이
//! 0 이라 건드리면 안 된다.
//!
//! 한글 2022 실측(문서 편집 버전 → 가장 가까운 설치본): 로고 상단 80.55pt,
//! 슬로건 상단 74.92pt, `보도자료` 85.74pt.
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6299/forest_press_wrap_seg_pairs.hwpx")
        .to_string_lossy()
        .into_owned()
}

/// `layout-anomaly` 요약 줄.
fn anomaly_summary() -> String {
    let out = Command::new(rhwp_bin())
        .args(["layout-anomaly", &sample()])
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    text.lines()
        .find(|l| l.contains("text-overlap"))
        .unwrap_or_default()
        .to_string()
}

#[test]
fn wrap_fragment_rows_do_not_double_count() {
    // 종전: overlap 2 · text-overlap 4. 조각을 한 줄로 세면 전부 사라진다.
    let summary = anomaly_summary();
    assert!(
        summary.contains("overlap: 0") && summary.contains("text-overlap: 0"),
        "어울림 조각 이중 계상이 남아 겹침이 생긴다: {summary}"
    );
}

#[test]
fn header_cell_content_matches_the_hangul_oracle() {
    let out = Command::new(rhwp_bin())
        .args(["dump-extents", &sample()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();

    // 각 줄의 (y, bottom) 을 뽑는다.
    let parse = |line: &str| -> Option<(f64, f64)> {
        let y = line
            .split("y=")
            .nth(1)?
            .split("..")
            .next()?
            .trim()
            .parse::<f64>()
            .ok()?;
        let h = line
            .split(" h=")
            .nth(1)?
            .split_whitespace()
            .next()?
            .parse::<f64>()
            .ok()?;
        Some((y, y + h))
    };

    // 머리글 표 첫 칸 = 첫 TableCell. 그 칸이 끝나면(다음 TableCell) 검사를 멈춘다.
    let mut cell: Option<(f64, f64)> = None;
    let mut checked = 0usize;
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with("TableCell") {
            if cell.is_some() {
                break; // 첫 칸만 본다
            }
            cell = parse(line);
        } else if line.starts_with("TextLine") {
            let (Some((_, bottom)), Some((y, line_bottom))) = (cell, parse(line)) else {
                continue;
            };
            checked += 1;
            assert!(
                line_bottom <= bottom + 0.5,
                "머리글 칸 줄이 칸 밖으로 흘러내렸다: {y:.1}..{line_bottom:.1} > 칸 바닥 {bottom:.1}"
            );
        }
    }
    assert!(checked >= 3, "머리글 칸 줄을 찾지 못했다(검사 {checked}건)");

    // 같은 vpos 조각은 같은 y 를 공유한다 — 4단 적층이면 서로 다른 y 가 4개 나온다.
    let ys: Vec<String> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| l.starts_with("TextLine") && l.contains("vpos="))
        .take(6)
        .filter_map(|l| {
            l.split("y=")
                .nth(1)
                .map(|r| r.split("..").next().unwrap_or("").trim().to_string())
        })
        .collect();
    let distinct: std::collections::BTreeSet<&String> = ys.iter().collect();
    assert!(
        distinct.len() <= 3,
        "어울림 조각이 세로로 쌓였다 — 서로 다른 y 가 {}개: {ys:?}",
        distinct.len()
    );
}
