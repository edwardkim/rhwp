//! [#5830] dash leader 가 문단 마지막 줄에서 슬랙을 못 받아 폭이 절반이 된다.
//!
//! 종전(devel `0f9ceeb19`): 양쪽정렬 배분 대상이 아닌 줄(마지막 줄·강제 줄바꿈 줄)은
//! `compute_line_extra_spacing` 이 슬랙을 계산하지 않아 `extra_dash_advance` 가 0 —
//! dash 는 `char_width_decision` 의 leader 클램프 하한 `font_size * 0.3` 에 머물렀다.
//! 한글 2022 정본은 같은 런을 0.499~0.583em 으로 그린다(폭 −40~−47%).
//!
//! 정본 규칙(`pdf/issue1921/86712_regulatory_analysis-2024.pdf` p34·p35 글리프 실측):
//! - 여백이 충분하면 **자연 폭**(p35 10자·18자 = 8.00pt = 0.571em)으로 — 여백에 닿지
//!   않고 끝난다. **무한 신장이 아니다.**
//! - 여백이 좁으면 **여백까지만** 좁힌다(p34 10자 = 7.00pt = 0.499em, 끝점 = 여백).
//!
//! 그래서 계약은 양방향이다: 하한(0.45em — 클램프 반토막 검출)과
//! 상한(0.75em — 슬랙 전량을 소수 dash 에 쏟는 과신장 검출).
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;
use std::process::Command;

/// nextest archive 가 런타임에 주입하는 경로를 먼저 읽고, 없으면 컴파일타임 값을 쓴다
/// (local_validation.md 4.3 의 신규 CLI 통합 테스트 규칙 — #3289).
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const SAMPLE: &str = "samples/issue1891/86712_regulatory_analysis.hwpx";

fn render_page_svg(page_arg: &str) -> String {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nth = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out = std::env::temp_dir().join(format!("rhwp_issue_5830_{}_{}", std::process::id(), nth));
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
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "svg"))
        .expect("SVG 산출물");
    let text = std::fs::read_to_string(svg).expect("SVG 읽기");
    let _ = std::fs::remove_dir_all(&out);
    text
}

/// `<text ...>-</text>` 를 (y, x, font_size) 로 거둔다. 하이픈은 글자마다 개별 요소다.
fn hyphen_glyphs(svg: &str) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(end) = chunk.find('>') else { continue };
        let (attrs, rest) = chunk.split_at(end);
        if !rest.starts_with(">-</text>") {
            continue;
        }
        let pick = |key: &str| -> Option<f64> {
            let at = attrs.find(&format!("{key}=\""))? + key.len() + 2;
            let tail = &attrs[at..];
            let stop = tail.find('"')?;
            tail[..stop].parse::<f64>().ok()
        };
        if let (Some(x), Some(y), Some(fs)) = (pick("x"), pick("y"), pick("font-size")) {
            out.push((y, x, fs));
        }
    }
    out
}

struct Run {
    baseline: f64,
    pts: Vec<(f64, f64)>,
}

impl Run {
    /// 글자당 advance (px).
    fn advance(&self) -> f64 {
        (self.pts[self.pts.len() - 1].0 - self.pts[0].0) / (self.pts.len() - 1) as f64
    }
    fn font_size(&self) -> f64 {
        self.pts[0].1
    }
}

/// 같은 baseline 에 놓인 하이픈을 이어 런으로 묶는다 (issue_5804 와 같은 방식).
fn hyphen_runs(svg: &str, min_len: usize) -> Vec<Run> {
    let mut by_line: BTreeMap<i64, Vec<(f64, f64)>> = BTreeMap::new();
    for (y, x, fs) in hyphen_glyphs(svg) {
        by_line
            .entry((y * 10.0).round() as i64)
            .or_default()
            .push((x, fs));
    }
    let mut runs = Vec::new();
    for (key, mut pts) in by_line {
        let baseline = key as f64 / 10.0;
        pts.sort_by(|a, b| a.0.partial_cmp(&b.0).expect("좌표 비교"));
        let mut cur: Vec<(f64, f64)> = vec![pts[0]];
        for &(x, fs) in &pts[1..] {
            let last = cur[cur.len() - 1];
            // 과신장 회귀도 한 런으로 묶여야 검출된다 — 자연폭(≈0.6em)의 4배까지 허용.
            if x - last.0 < last.1 * 2.5 {
                cur.push((x, fs));
            } else {
                if cur.len() >= min_len {
                    runs.push(Run {
                        baseline,
                        pts: std::mem::take(&mut cur),
                    });
                }
                cur = vec![(x, fs)];
            }
        }
        if cur.len() >= min_len {
            runs.push(Run { baseline, pts: cur });
        }
    }
    runs
}

/// 마지막 줄 dash leader 도 정본의 advance 대역(0.45~0.75em)에 들어와야 한다.
///
/// 종전에는 문서 34쪽(0기준 33)의 10자·30자 런, 35쪽의 8자·18자 런이 정확히
/// 0.300em(클램프 하한)이었다. 상한은 반대 방향의 회귀 — 슬랙 전량을 소수 dash 에
/// 쏟으면 35쪽 8자 런이 1.9em 까지 벌어진다(구현 중 실제로 만든 오답).
#[test]
fn last_line_dash_leader_advance_stays_in_the_oracle_band() {
    for page_arg in ["32", "33"] {
        let svg = render_page_svg(page_arg);
        let runs = hyphen_runs(&svg, 8);
        assert!(
            runs.len() >= 3,
            "p{page_arg}: 8자 이상 dash 런이 3개는 있어야 한다 (실측 {})",
            runs.len()
        );
        for run in &runs {
            let em = run.advance() / run.font_size();
            assert!(
                (0.45..=0.75).contains(&em),
                "p{page_arg} y={:.1} {}자 런의 advance {:.3}em — 정본 대역(0.45~0.75em) 밖 \
                 (0.300em 은 마지막 줄 클램프 잔존, 0.75em 초과는 슬랙 과신장)",
                run.baseline,
                run.pts.len(),
                em
            );
        }
    }
}

/// dash leader 는 채움이지만 **여백을 넘지는** 않는다 — 확장 후에도 줄의 마지막
/// 글리프가 본문 오른쪽 경계 안에 있어야 한다.
#[test]
fn stretched_dash_leader_stays_inside_the_text_area() {
    // 이 문서(A4 세로, 96dpi SVG)의 정상(양쪽정렬) 런 마지막 글리프 **원점** 실측
    // 최대치는 707.1px 다. 확장된 마지막 줄 런의 원점이 그보다 밖이면 여백 침범이다.
    const RIGHT_EDGE_ORIGIN_PX: f64 = 708.0;
    for page_arg in ["32", "33"] {
        let svg = render_page_svg(page_arg);
        for run in hyphen_runs(&svg, 8) {
            let last_x = run.pts[run.pts.len() - 1].0;
            assert!(
                last_x <= RIGHT_EDGE_ORIGIN_PX,
                "p{page_arg} y={:.1} {}자 런이 본문 경계를 넘는다: 마지막 글리프 원점 {last_x:.1}px",
                run.baseline,
                run.pts.len(),
            );
        }
    }
}
