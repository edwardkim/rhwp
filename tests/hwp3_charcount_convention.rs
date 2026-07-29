//! [#3510] HWP3 파서의 `char_count` 규약을 HWP5·HWPX 와 맞춘다.
//!
//! HWPX(`parser/hwpx/section.rs`)와 HWP5(`parser/body_text.rs`)는 문단 끝 마커를
//! `char_count` 에 포함하는데 HWP3(`parser/hwp3/mod.rs`)만 포함하지 않았다. 그래서
//! HWP3 → HWPX 변환이 **내용은 완전히 같은데** 모든 문단·셀에서 1씩 차이가 나고,
//! `--verify` 게이트가 정상 변환을 exit 3 으로 거부했다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 표를 가진 HWP3 문서 — 차이가 셀 안에서도 나타난다.
const SAMPLE_HWP3: &str = "samples/hwp3-sample.hwp";
/// 무회귀 확인용 HWP5 문서.
const SAMPLE_HWP5: &str = "samples/field-01.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-cc-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// 원본을 HWPX 로 내보내고 `ir-diff --json` 봉투를 돌려준다.
fn roundtrip_diff(src_rel: &str, tag: &str) -> Option<serde_json::Value> {
    let src = sample(src_rel);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀: {src_rel}");
        return None;
    }
    let out = temp_path(tag, "hwpx");
    let conv = ["export-hwpx", src.to_str().unwrap(), out.to_str().unwrap()];
    let c = run(&conv);
    assert_eq!(c.status.code(), Some(0), "{}", describe(&conv, &c));
    assert!(out.exists(), "HWPX 산출물이 생성되어야 합니다");

    let args = [
        "ir-diff",
        src.to_str().unwrap(),
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    let v: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ir-diff --json 이 순수 JSON 이어야 합니다");
    let _ = std::fs::remove_file(&out);
    Some(v)
}

/// 카테고리 맵에서 `cc`(char_count) 차이 건수를 센다.
fn cc_diff_count(v: &serde_json::Value) -> u64 {
    v["categories"]
        .as_object()
        .map(|m| {
            m.iter()
                .filter(|(k, _)| k.as_str() == "cc")
                .filter_map(|(_, n)| n.as_u64())
                .sum()
        })
        .unwrap_or(0)
}

/// `cc`(char_count) 라인들을 뽑아 `(A, B)` 쌍으로 파싱한다. 원문 형식: `cc: A=<n> vs B=<n>`.
fn cc_pairs(diff_text: &str) -> Vec<(i64, i64)> {
    diff_text
        .lines()
        .filter_map(|l| {
            let l = l.trim();
            let rest = l.strip_prefix("[차이] cc: A=")?;
            let (a, rest) = rest.split_once(" vs B=")?;
            Some((a.trim().parse().ok()?, rest.trim().parse().ok()?))
        })
        .collect()
}

#[test]
fn hwp3_roundtrip_char_count_is_not_off_by_exactly_one() {
    // [#3510] 본론: 끝 마커 규약 불일치는 **모든** 문단·셀에서 예외 없이 정확히 1씩만
    // 어긋나는 것이 특징이었다(diffCount 298건, 전부 cc, 전부 |A-B|==1). 그 패턴이
    // 사라졌는지만 고정한다 — 이 표본에는 #3510 과 무관한 다른 결함(예: 구역 시작
    // 문단의 secd/cold 컨트롤 순서 뒤집힘, 필드 컨트롤 라운드트립 손실)이 남아 있어
    // diffCount 자체는 0 이 아니다.
    let src = sample(SAMPLE_HWP3);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out = temp_path("offbyone", "hwpx");
    let conv = ["export-hwpx", src.to_str().unwrap(), out.to_str().unwrap()];
    let c = run(&conv);
    assert_eq!(c.status.code(), Some(0), "{}", describe(&conv, &c));

    let args = ["ir-diff", src.to_str().unwrap(), out.to_str().unwrap()];
    let output = run(&args);
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let pairs = cc_pairs(&text);
    assert!(
        !pairs.is_empty(),
        "cc 라인 파싱 실패 — 형식이 바뀌었을 수 있습니다\n{text}"
    );
    for (a, b) in &pairs {
        assert_ne!(
            (a - b).abs(),
            1,
            "끝 마커 규약 불일치(off-by-one) 패턴이 재발했습니다: A={a} B={b}\n{text}"
        );
    }
    let _ = std::fs::remove_file(&out);
}

#[test]
fn hwp3_roundtrip_char_count_diff_count_dropped_sharply() {
    // 무회귀 가드: 수정 전 이 표본은 diffCount=298(전부 cc)였다. 수정 후에는
    // #3510 과 무관한 잔여 결함만 남아 diffCount 가 크게 줄어야 한다.
    let Some(v) = roundtrip_diff(SAMPLE_HWP3, "count") else {
        return;
    };
    let diff_count = v["diffCount"].as_u64().unwrap_or(u64::MAX);
    assert!(
        diff_count < 50,
        "끝 마커 규약 수정 후에도 차이가 298 근처로 많이 남아 있습니다: {v}"
    );
}

#[test]
fn hwp5_roundtrip_char_count_unaffected() {
    // 무회귀 가드: HWP5 는 원래 cc 차이가 0이었고 그대로여야 한다.
    let Some(v) = roundtrip_diff(SAMPLE_HWP5, "hwp5") else {
        return;
    };
    assert_eq!(
        cc_diff_count(&v),
        0,
        "HWP5 왕복에 char_count 차이가 새로 생겼습니다: {v}"
    );
}
