//! [#5804] 개정문 하이픈 표기는 글자로 그린다 — 실선으로 바꾸지 않는다.
//!
//! 종전(devel `fb434269e`): 3+ 연속 `-` 를 만나면 렌더러가 글리프 출력을 통째로
//! 건너뛰고 `<line>` 하나로 대체했다(Task #352). 법령 개정문·신구조문대비표에서
//! `- - - -` 는 "현행과 같음"을 뜻하는 **의미 있는 표기**라, 실선이 되면 읽는 사람이
//! 생략인지 밑줄 그은 빈칸인지 구분할 수 없다.
//!
//! 오라클은 한글 2022 정본 `pdf/issue1921/86712_regulatory_analysis-2024.pdf` 다.
//! 정본은 하이픈을 낱글자로 그리며, 글자당 advance 는 런마다 달라지고 끝점이
//! 오른쪽 여백에 수렴한다(탄력 leader). 그 분배는 이미 레이아웃이 만든다
//! (`compute_line_extra_spacing` 의 `extra_dash_sp` → `TextStyle::extra_dash_advance`).
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

/// 재현 문서: 규제영향분석서(입법예고 신구조문대비표). 이슈가 든 코퍼스와 같은 종류이며
/// 저장소에 한글 정본 PDF 가 짝으로 있다.
const SAMPLE: &str = "samples/issue1891/86712_regulatory_analysis.hwpx";

/// `-p` 는 0 기준이라 이 값은 문서 35 쪽(정본 p35)을 가리킨다.
// [#6389] KoPub돋움체 한글 폭을 임베드 실측 872/1000em 으로 되돌리면 앞쪽
// 본문이 조밀해져 신구조문대비표가 한 쪽 당겨진다(문서 65→64쪽, 표는 0기준 33).
const PAGE_ARG: &str = "33";

/// 새 의존성을 들이지 않으려고 표준 라이브러리만 쓴다 — 프로세스 id 와 호출 순번으로
/// 디렉토리를 갈라, 같은 프로세스 안에서 스레드로 병렬 실행되는 시험끼리도 부딪히지 않게 한다.
fn render_page_svg() -> String {
    static SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let nth = SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let out = std::env::temp_dir().join(format!("rhwp_issue_5804_{}_{}", std::process::id(), nth));
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

/// 하나의 하이픈 런 — baseline y 와 (x, font_size) 목록.
struct Run {
    baseline: f64,
    pts: Vec<(f64, f64)>,
}

impl Run {
    fn width(&self) -> f64 {
        self.pts[self.pts.len() - 1].0 - self.pts[0].0
    }
}

/// 같은 baseline 에 놓인 하이픈을 이어 런으로 묶는다.
fn hyphen_runs(svg: &str) -> Vec<Run> {
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
            if x - last.0 < last.1 * 1.2 {
                cur.push((x, fs));
            } else {
                if cur.len() >= 4 {
                    runs.push(Run {
                        baseline,
                        pts: std::mem::take(&mut cur),
                    });
                }
                cur = vec![(x, fs)];
            }
        }
        if cur.len() >= 4 {
            runs.push(Run { baseline, pts: cur });
        }
    }
    runs
}

fn widest_run(runs: &[Run]) -> &Run {
    runs.iter()
        .max_by(|a, b| a.width().partial_cmp(&b.width()).expect("폭 비교"))
        .expect("가장 넓은 런")
}

#[test]
fn dash_leader_is_drawn_as_glyphs_not_a_line() {
    let svg = render_page_svg();
    let glyphs = hyphen_glyphs(&svg);
    assert!(
        glyphs.len() >= 60,
        "개정문 하이픈은 글자로 그려야 한다 — 이 쪽의 하이픈 글자 수 실측 {} (수정 전 2)",
        glyphs.len()
    );
}

#[test]
fn dash_leader_run_keeps_the_elastic_advance() {
    // 정본(한글 2022) p35 의 26 자 런: x 324.00 → 528.72 pt, 글자당 advance 8.16~8.28 pt,
    // font-size 14.04 pt → 0.581~0.590 em. SVG 는 96dpi 라 pt 로 환산해 비교한다.
    let svg = render_page_svg();
    let runs = hyphen_runs(&svg);
    assert!(!runs.is_empty(), "하이픈 런을 하나도 못 찾았다");

    let widest = widest_run(&runs);
    let font_size = widest.pts[0].1;
    let advance = widest.pts[1].0 - widest.pts[0].0;
    let em = advance / font_size;
    assert!(
        (0.50..=0.65).contains(&em),
        "탄력 leader 의 글자당 advance 가 정본 범위(0.58 em 안팎)를 벗어났다: {em:.3} em \
         — 레이아웃의 슬랙 분배(extra_dash_advance)가 깨졌을 수 있다"
    );
}

#[test]
fn dash_leader_no_longer_emits_a_substitute_rule() {
    // 치환 라인은 글리프와 같은 줄, baseline 바로 위(`-font_size * 0.32`)에 그려졌었다.
    // 글자가 살아난 뒤에도 라인이 같이 남으면 이중선이 된다.
    //
    // 표 괘선도 같은 x 구간을 가로지르므로 x 만으로 판정하면 오탐이 난다 — 이 쪽의
    // 괘선은 y=75.9 · 108.0 · 466.4 이고 런 baseline 은 192.0 이다. 따라서 **그 줄 높이에
    // 놓인 가로선만** 본다.
    let svg = render_page_svg();
    let runs = hyphen_runs(&svg);
    let widest = widest_run(&runs);
    let (x1, x2) = (widest.pts[0].0, widest.pts[widest.pts.len() - 1].0);
    let font_size = widest.pts[0].1;
    let band = font_size * 0.6;

    for chunk in svg.split("<line ").skip(1) {
        let Some(end) = chunk.find('>') else { continue };
        let attrs = &chunk[..end];
        let pick = |key: &str| -> Option<f64> {
            let at = attrs.find(&format!("{key}=\""))? + key.len() + 2;
            let tail = &attrs[at..];
            let stop = tail.find('"')?;
            tail[..stop].parse::<f64>().ok()
        };
        let (Some(lx1), Some(ly1), Some(lx2), Some(ly2)) =
            (pick("x1"), pick("y1"), pick("x2"), pick("y2"))
        else {
            continue;
        };
        let horizontal = (ly1 - ly2).abs() < 0.5;
        let on_this_line = (ly1 - widest.baseline).abs() <= band;
        let covers = lx1 <= x1 + 1.0 && lx2 >= x2 - 1.0;
        assert!(
            !(horizontal && on_this_line && covers),
            "하이픈 런 자리에 치환 라인이 그대로 남았다: \
             line ({lx1:.1},{ly1:.1})→({lx2:.1},{ly2:.1}), 런 {x1:.1}→{x2:.1} baseline {:.1}",
            widest.baseline
        );
    }
}
