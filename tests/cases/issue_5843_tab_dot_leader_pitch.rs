//! [#5843] 목차 탭 점선 리더는 글꼴 크기를 따라 점으로 그린다 — 실선처럼 뭉개지지 않는다.
//!
//! 종전(devel `0697bc5591`): 채움 종류 3(점선)을 `stroke-width="1.0"`,
//! `stroke-dasharray="0.1 3"` 로 그렸다 — **글꼴 크기와 무관한 고정값**이다.
//! 점 간격 3.1px 는 14pt 기준 4.64px 여야 할 것보다 33% 촘촘하고, 점 지름 1.0px 는
//! 이 저장소가 가운뎃점용으로 실측 보정해 둔 0.12 em(≈2.24px)의 절반이 안 된다.
//! 그래서 목차 점선이 점이 아니라 가는 실선으로 보였다.
//!
//! 오라클은 `pdf/KTX-2022.pdf` 2쪽이다 — 한글이 낱개 `·` 글리프로 그린 점 1,435개를
//! 좌표로 재면 글꼴 14.04pt 에서 3.48pt, 15.00pt 에서 3.72pt → **두 크기 모두 0.248 em**.
#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

/// nextest archive 가 런타임에 주입하는 경로를 먼저 읽고, 없으면 컴파일타임 값을 쓴다
/// (local_validation.md 4.3 의 신규 CLI 통합 시험 규칙 — #3289).
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// 목차가 있는 쪽. `-p` 는 0 기준이라 이 값이 문서 2쪽이다.
const PAGE_ARG: &str = "1";
const SAMPLE: &str = "samples/KTX.hwp";

fn render_toc_page() -> String {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nth = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out = std::env::temp_dir().join(format!("rhwp_issue_5843_{}_{}", std::process::id(), nth));
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("출력 디렉토리 생성");
    let done = Command::new(rhwp_bin())
        .current_dir(repo_root())
        .args([
            "export-svg",
            SAMPLE,
            "-p",
            PAGE_ARG,
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
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "svg"))
        .expect("SVG 산출물");
    let text = std::fs::read_to_string(svg).expect("SVG 읽기");
    let _ = std::fs::remove_dir_all(&out);
    text
}

/// 점선 리더 한 줄 — (선 두께, 점 간격) px.
struct Leader {
    width: f64,
    pitch: f64,
}

/// `stroke-linecap="round"` 가 붙은 가로선만 점선 리더다(표 괘선·밑줄과 구별된다).
fn dot_leaders(svg: &str) -> Vec<Leader> {
    let mut out = Vec::new();
    for chunk in svg.split("<line ").skip(1) {
        let Some(end) = chunk.find('>') else { continue };
        let attrs = &chunk[..end];
        if !attrs.contains("stroke-linecap=\"round\"") {
            continue;
        }
        let pick = |key: &str| -> Option<String> {
            let at = attrs.find(&format!("{key}=\""))? + key.len() + 2;
            let tail = &attrs[at..];
            let stop = tail.find('"')?;
            Some(tail[..stop].to_string())
        };
        let Some(width) = pick("stroke-width").and_then(|v| v.parse::<f64>().ok()) else {
            continue;
        };
        let Some(dash) = pick("stroke-dasharray") else {
            continue;
        };
        let parts: Vec<f64> = dash
            .split_whitespace()
            .filter_map(|v| v.parse::<f64>().ok())
            .collect();
        if parts.len() != 2 {
            continue;
        }
        out.push(Leader {
            width,
            pitch: parts[0] + parts[1],
        });
    }
    out
}

#[test]
fn toc_page_draws_dot_leaders() {
    let leaders = dot_leaders(&render_toc_page());
    assert!(
        leaders.len() >= 10,
        "목차 쪽에 점선 리더가 10줄 이상 있어야 한다 — 실측 {}줄",
        leaders.len()
    );
}

#[test]
fn dot_pitch_follows_the_hancom_oracle() {
    // 정본 실측: 0.248 em. 이 쪽 글꼴은 14.04pt(18.72px)·15.00pt(20px) 이므로
    // 간격은 4.6~5.0px 범위여야 한다. 종전 고정값 3.1px 는 이 범위 밖이다.
    let leaders = dot_leaders(&render_toc_page());
    for l in &leaders {
        assert!(
            (4.0..=5.6).contains(&l.pitch),
            "점 간격이 정본 범위(글꼴 크기 × 0.248 em)를 벗어났다: {:.3}px \
             — 고정값으로 되돌아갔는지 확인할 것(종전 3.1px)",
            l.pitch
        );
    }
}

#[test]
fn dot_size_matches_the_middle_dot_calibration() {
    // 리더의 점도 한글이 그리는 `·` 다 — 지름은 가운뎃점 실측(0.12 em)을 따라야 한다.
    // 이 쪽 글꼴이면 2.2~2.5px. 종전 고정값 1.0px 는 절반이 안 됐다.
    let leaders = dot_leaders(&render_toc_page());
    for l in &leaders {
        assert!(
            (1.8..=2.8).contains(&l.width),
            "점 지름이 가운뎃점 실측(0.12 em)에서 벗어났다: {:.3}px (종전 1.0px 고정)",
            l.width
        );
    }
}

#[test]
fn leader_geometry_tracks_font_size_rather_than_a_constant() {
    // 이 쪽에는 글꼴 크기가 다른 리더 줄이 섞여 있다(제목 15pt · 항목 14.04pt).
    // 고정값으로 되돌아가면 모든 줄이 같은 값이 되므로, **두 종류 이상**이어야 한다.
    let leaders = dot_leaders(&render_toc_page());
    let mut widths: Vec<i64> = leaders.iter().map(|l| (l.width * 100.0) as i64).collect();
    widths.sort_unstable();
    widths.dedup();
    assert!(
        widths.len() >= 2,
        "리더 두께가 한 가지뿐이다 — 글꼴 크기를 안 보고 고정값을 쓰는 상태다: {widths:?}"
    );
}
