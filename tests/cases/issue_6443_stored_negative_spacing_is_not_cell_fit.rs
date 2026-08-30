//! [#6443] 칸 안 음수 자간을 **무조건** "칸 폭 맞춤"으로 보고 도로 늘리지 않는다.
//!
//! `samples/issue6443/research_project_design_form.hwpx` 는 해양수산부
//! 「국립수산과학원 연구사업 관리규정」[별지 7] 사업설계서 서식(법제처 3123751,
//! 39KB) 원본이다.
//!
//! **증상.** 8쪽 `산 출 내 역` 열 글자가 오른쪽 여백 없이 괘선에 붙는다.
//!
//! **⚠ 이슈가 말한 "괘선을 1.57pt 넘는다"는 측정 착시였다.** `537.33` 은 마지막
//! 글리프의 **전진폭 상자**(PDF text bbox)이지 잉크가 아니다. 래스터로 재면 괘선
//! 너머 어두운 픽셀이 rhwp·한글 **양쪽 다 0** 이다. `#6303` 도 같은 착시였다 —
//! 칸 꼬리 판정은 text bbox 가 아니라 **잉크**로 하라.
//!
//! **근인 — 규칙의 전제가 이 문서에서 거짓이다.**
//!
//! 문단 `charPr 74` 는 자간 **−16%** 를 선언하고 rhwp 도 그대로 적용해 한글과 같은
//! **8.40pt/글자**를 측정한다. 그런데 칸 안 underflow 규칙이
//!
//! > 칸 안 + 음수 자간 + 자연 폭(자간 0)이 칸을 넘음
//! > ⇒ 편집기가 칸 폭에 맞추려 압축한 것이니 칸 폭까지 되늘린다
//!
//! 로 보고 글자당 `+0.733px` 를 붙여 **8.95pt** 로 만든다. 줄이 22~36pt 길어져
//! 괘선에 딱 붙는다.
//!
//! 전제가 참이라면 **압축된 실제 폭이 칸 안쪽 폭에 근접**해야 한다 — 남는 잔여는
//! 우리 폰트 메트릭이 한컴보다 좁게 재는 몫뿐이라 작다. 이 열은 −16% 로 재도
//! 칸보다 **7.4~13.8%** 짧다(`avail 358.6` vs `total 309~332`). 칸 맞춤이 아니라
//! 조판 의도로 좁힌 글이다.
//!
//! **수정.** `STORED_CELL_FIT_MAX_SHORTFALL_RATIO`(5%) 를 전제 조건으로 세운다.
//!
//! **오라클 — 한글 2022** (문서 저장 버전 = 한글 2020, 미설치 → 최근접 설치본,
//! `producer=Hancom PDF 1.3.0.547`). 8쪽 산출내역 열 우단이 줄 단위로 맞는다.
//!
//! | 줄 | 종전 | **수정 후** | 한글 2022 |
//! |---|---:|---:|---:|
//! | ◦연구보조원 (월단가)원×… | 536.98 | **512.28** | 512.76 |
//! | ◦인턴연구원 … | 537.33 | **514.29** | 514.20 |
//!
//! 괘선은 `535.76`(rhwp) / `535.39`(한글)이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

/// 8쪽 산출내역 열의 오른쪽 괘선 (pt).
const COLUMN_RIGHT_RULE_PT: f64 = 535.76;
/// 한글 2022 실측 우단의 최댓값 (pt) — 이 열의 어느 줄도 이보다 오른쪽에서 끝나지 않는다.
const HANGUL_MAX_RIGHT_PT: f64 = 514.20;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6443/research_project_design_form.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6443-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 8쪽 render tree JSON.
fn page8_render_tree() -> String {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            &sample(),
            "-p",
            "7",
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

/// `"text"` 가 `needle` 을 담은 `TextRun` 들의 우단(pt) 목록.
fn run_rights_containing(json: &str, needle: &str) -> Vec<f64> {
    const PX_TO_PT: f64 = 72.0 / 96.0;
    let mut out = Vec::new();
    let mut rest = json;
    while let Some(at) = rest.find("\"TextRun\"") {
        let tail = &rest[at..];
        // 이 노드의 JSON 만 본다 — 다음 노드(`{"type"`) 앞에서 끊지 않으면 이웃
        // run 의 텍스트까지 걸려 개수가 6배로 부푼다.
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
        if seg.contains(needle) {
            if let (Some(x), Some(w)) = (num("x"), num("w")) {
                out.push((x + w) * PX_TO_PT);
            }
        }
        rest = &tail[9..];
    }
    out
}

/// 산출내역 열의 줄이 한글 실측 우단을 넘지 않는다.
///
/// 종전에는 칸 안 음수 자간을 "칸 폭 맞춤"으로 단정해 되늘려, 줄이 `536.98`~`537.33`
/// 까지 뻗어 괘선(`535.76`)에 딱 붙었다. 한글은 `512.76`~`514.20` 에서 멈춘다.
#[test]
fn cost_detail_column_keeps_its_stored_condensed_width() {
    let json = page8_render_tree();
    // 줄 **끝** run 을 겨냥한다 — 줄 앞부분(`◦연구보조원`) 은 되늘림이 있어도 짧아서
    // 판별력이 없다. `월=  원` 이 이 열 여러 줄의 마지막 run 이다.
    let rights = run_rights_containing(&json, "월=");
    assert_eq!(
        rights.len(),
        8,
        "8쪽 산출내역 열의 줄 끝 run(`월=`) 수가 달라졌다"
    );
    let max_right = rights.iter().cloned().fold(f64::MIN, f64::max);
    assert!(
        max_right <= HANGUL_MAX_RIGHT_PT + 1.5,
        "산출내역 열 우단이 한글 실측({HANGUL_MAX_RIGHT_PT}pt)을 넘었다: {max_right:.2}pt \
         (괘선 {COLUMN_RIGHT_RULE_PT}pt, 종전 rhwp 536.98~537.33pt)"
    );
}

/// 그 줄들이 괘선 안쪽에 **여백을 남긴다** — 되늘림이 없어야 성립한다.
///
/// 위 테스트만으로는 "줄이 짧아졌다"가 다른 이유(글자 누락 등)로도 통과할 수 있으므로,
/// 열 텍스트가 온전한지도 함께 잠근다.
#[test]
fn cost_detail_column_text_is_intact() {
    let json = page8_render_tree();
    assert_eq!(
        json.matches("연구보조원").count(),
        5,
        "산출내역 열 `연구보조원` 출현 수 (한글 2022 실측과 동일)"
    );
    assert_eq!(
        json.matches("인턴연구원").count(),
        3,
        "산출내역 열 `인턴연구원` 출현 수 (한글 2022 실측과 동일)"
    );
}
