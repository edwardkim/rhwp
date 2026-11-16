//! [#9010] 고정형 계약 — 원본 파일명이 길어도 쪽별 산출 파일 이름이 파일 시스템 한도
//! (255바이트) 안에 들어 모든 쪽을 저장한다.
//!
//! 주석 갱신: 2026-11-12 20:40 KST(+09:00), 새 파일.
//!
//! - 입력: 공개 법령 별표 `samples/21868765_별표2_보건소_분장사무.hwp`(4쪽)를 실제 사례
//!   (rhwp-issue-docs#1092, 100k 스윕의 국방부·해양경찰청 정원표 별표)와 같은 길이의 한국어
//!   이름(UTF-8 250바이트 이상)으로 복사해 쓴다. 렌더링 결과가 아닌 산출 파일 이름 계약이므로
//!   시각 측정 대상이 아니다.
//! - 수정 전 rhwp(devel `80b1c354c2`)는 `{stem}_{NNN}.svg` 가 255바이트를 넘어 모든 쪽이
//!   `File name too long (os error 36)` 으로 실패하고 0개 파일을 남겼다.
//!
//! 검사: 종료 코드 0, 쪽마다 산출 1개(쪽수 = 4), 각 이름 ≤ 255바이트, 쪽 번호 접미사·확장자
//! 보존, 줄인 stem 이 원본 stem 의 앞부분(UTF-8 문자 경계)이다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

const SAMPLE: &str = "samples/21868765_별표2_보건소_분장사무.hwp";
const PAGES: usize = 4;
const NAME_MAX: usize = 255;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "rhwp-9010-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// 실제 사례와 같은 모양의 긴 이름(확장자 포함 255바이트 이하, stem 250바이트 이상).
fn long_stem() -> String {
    let mut stem = String::from(
        "18058585_[별표 6] 국방부 한시조직에 두는 공무원(군인제외) 정원표(제2조제1항·제5조제1항 관련)(국방부와 그 소속기관 직제 시행규칙)",
    );
    while stem.len() + "가".len() + ".hwp".len() <= NAME_MAX {
        stem.push('가');
    }
    assert!(stem.len() >= 250, "stem bytes {}", stem.len());
    stem
}

fn export(command: &str, input: &Path, out: &Path) -> std::process::Output {
    Command::new(rhwp_bin())
        .arg(command)
        .arg(input)
        .arg("-o")
        .arg(out)
        .env("RHWP_FONT_PATH", "")
        .output()
        .unwrap()
}

fn check_names(out: &Path, stem: &str, extension: &str) {
    let mut names: Vec<String> = std::fs::read_dir(out)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| name.ends_with(extension))
        .collect();
    names.sort();
    assert_eq!(names.len(), PAGES, "쪽마다 산출 1개: {names:?}");
    for (index, name) in names.iter().enumerate() {
        let suffix = format!("_{:03}{extension}", index + 1);
        assert!(name.len() <= NAME_MAX, "{} bytes: {name}", name.len());
        assert!(name.ends_with(&suffix), "쪽 번호 접미사 보존: {name}");
        let kept = name.strip_suffix(&suffix).unwrap();
        assert!(
            !kept.is_empty() && stem.starts_with(kept),
            "원본 stem 의 앞부분: {kept}"
        );
    }
}

#[test]
fn long_source_names_still_write_every_page_within_the_name_limit() {
    let sample = Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE);
    let dir = scratch("in");
    let stem = long_stem();
    let input = dir.join(format!("{stem}.hwp"));
    std::fs::copy(&sample, &input).unwrap();

    for (command, extension) in [("export-svg", ".svg"), ("export-text", ".txt")] {
        let out = scratch("out");
        let result = export(command, &input, &out);
        assert_eq!(
            result.status.code(),
            Some(0),
            "{command}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&result.stderr).contains("too long"),
            "{command}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
        check_names(&out, &stem, extension);
        std::fs::remove_dir_all(&out).ok();
    }
    std::fs::remove_dir_all(&dir).ok();
}
