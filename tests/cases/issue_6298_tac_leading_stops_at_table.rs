//! [#6298] 자리차지 표의 leading 은 **표 앞** 텍스트만 센다.
//!
//! `samples/issue6298/copay_cap_tac_table_leading.hwpx` 는 원본(보건복지부 156586318)의
//! **구조 보존 슬라이스**다 — BinData 이미지만 8×8 PNG 로 바꿔 399KB → 110KB 로 줄였고,
//! 조판은 `<hp:sz>` 선언 크기를 쓰므로 좌표가 원본과 같다.
//!
//! **자기 자신이 통제군.** 12쪽의 두 【실제 사례】 표는 선언이 완전히 같다 —
//! `hp:sz width=47624`, `hp:pos treatAsChar=1 horzRelTo=PARA horzAlign=LEFT horzOffset=0`,
//! `outMargin=141`, 호스트 `paraPr` 도 동일, 저장 사다리도 둘 다
//! `lineseg textpos=0 horzpos=0 horzsize=48188` 한 줄뿐이다. **다른 것은 딱 하나** —
//! 표 **뒤**에 붙은 텍스트가 하나는 빈 `<hp:t/>`, 다른 하나는 공백 2칸이다.
//!
//! **종전 결함.** `compute_tac_leading_width` 의 블록 취급 갈래(`tac_controls` 에
//! 없는 표 — 폭 47624 가 줄폭 48188 의 98.8% 라 인라인으로 세지 않는다)가 **줄 0 의
//! 모든 run 을 합산**했다. 그래서 표 **뒤**의 공백까지 표의 x 로 실려 두 표가
//! 12.84pt 어긋나고(58.10 vs 70.94), 아래 표는 본문 우단을 7.2pt 넘겨 잘렸다.
//! 공백을 표 앞에 두나 뒤에 두나 결과가 같은 "순서 무관"이 그 정체였다.
//!
//! `#6167` 이 넣은 구제 조건(`stored_ladder_gives_tac_table_its_own_line`)은 이
//! 형상을 못 받는다 — `.skip(1)` 이라 표가 **문단 첫 글자**여서 자기 줄이 `ls[0]` 인
//! 경우를 건너뛴다.
//!
//! **수정.** leading 은 정의상 **앞**에 있는 것이므로, 블록 취급 갈래도 표의 문단 내
//! char 위치에서 멈춘다.
//!
//! 한글 2022 실측(문서 편집 버전 = 한글 10.x → 가장 가까운 설치본): 두 표 모두 좌단
//! 58.05pt — 12쪽 표 좌단 x 가 **한 값뿐**이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6298/copay_cap_tac_table_leading.hwpx")
        .to_string_lossy()
        .into_owned()
}

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-6298-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 12쪽(0-base 11) render tree 의 `Table` 노드 (x, x+w) 목록.
fn page12_table_extents() -> Vec<(f64, f64)> {
    let dir = temp_dir();
    let out = Command::new(rhwp_bin())
        .args([
            "export-render-tree",
            &sample(),
            "-p",
            "11",
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
    let json = std::fs::read_to_string(path).unwrap();

    let mut out = Vec::new();
    let mut rest = json.as_str();
    while let Some(at) = rest.find("\"Table\"") {
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
        if let (Some(x), Some(w)) = (num("x"), num("w")) {
            out.push((x, x + w));
        }
        rest = &tail[bbox_at + 6..];
    }
    out
}

#[test]
fn twin_tac_tables_share_one_left_edge() {
    let tables = page12_table_extents();
    // 폭 635.0px(= 47624HU) 인 【실제 사례】 표 두 개.
    let mut lefts: Vec<f64> = tables
        .iter()
        .filter(|(x, right)| (right - x - 635.0).abs() <= 2.0)
        .map(|(x, _)| *x)
        .collect();
    assert_eq!(lefts.len(), 2, "같은 선언의 표 2개를 기대했다: {tables:?}");
    lefts.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert!(
        (lefts[1] - lefts[0]).abs() <= 1.0,
        "같은 선언의 두 표가 어긋난다 — 표 뒤 공백이 leading 으로 실렸다: {lefts:?}"
    );
    // 한글 2022 실측 58.05pt = 77.4px.
    assert!(
        (lefts[0] - 77.4).abs() <= 2.0,
        "표 좌단이 한글(77.4px)에서 벗어났다: {:.1}",
        lefts[0]
    );
}

#[test]
fn tac_table_stays_inside_the_body_right_edge() {
    let tables = page12_table_extents();
    // 본문 우단 539.98pt = 719.97px. 종전에는 아래 표가 729.6px 로 7.2pt 넘겼다.
    for (x, right) in &tables {
        if (right - x - 635.0).abs() > 2.0 {
            continue;
        }
        assert!(
            *right <= 720.5,
            "표가 본문 우단을 넘는다: {x:.1}..{right:.1}"
        );
    }
}
