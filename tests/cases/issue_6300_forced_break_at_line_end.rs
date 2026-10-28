//! [#6300] 줄 끝의 강제 줄나눔(`0x000A`)은 줄을 다시 쪼개지 않는다.
//!
//! `samples/issue6300/trade_report_forced_break_object.hwp` 17쪽 `농수산식품` 문단은
//! 저장 사다리가 줄 5개를 적어 두었는데, rhwp 는 그중 하나를 잃고 다음 줄에 두 줄을
//! 몰아 그렸다.
//!
//! ```text
//! 저장 경계: [0, 45, 83, 119, 152]
//! rhwp 경계: [0, 45, 83,  83, 152]   ← 119 소실 · 83 중복
//! 줄 길이:   [45, 38,  0,  69]        ← 한 줄이 비고 다음 줄이 두 줄 분량
//! ```
//!
//! 그 69자 줄은 본문 우단을 47.6pt 넘겨 용지 끝에서 잘렸다.
//!
//! **근인.** 문단 끝 제어문자 배치가 `0x000A`(151) → 인라인 개체(152) → 문단끝(160)
//! 이라, 저장 사다리는 `줄3 = 119..152`(끝 글자가 `\n`) · `줄4 = 개체` 로 적는다.
//! 그런데 `Task #19/20` 의 TAC `\n` 분할이 그 `\n` 을 **줄 안의 분할점**으로 보고
//! 앞부분을 이전 줄에 합쳐 버린다 — 저장 사다리가 이미 거기서 끊어 둔 자리다.
//!
//! `\n` 이 그 줄의 **마지막 글자**면 분할하지 않는다. Task #19/20 이 겨냥한 형상
//! (한 줄 안에서 `\n` 이 텍스트와 표를 가르는 것)은 `\n` 뒤에 같은 줄의 내용이 남아
//! 있으므로 그대로 걸린다.
//!
//! **전수 상관.** 문서 5,333 문단 중 강제 줄나눔 보유 25개, 그중 `\n` 뒤 인라인 개체
//! 문단 7개 — 본문 우단을 20pt 이상 넘긴 쪽도 **정확히 그 7쪽**이었다.
//!
//! 한글 2022 실측(편집 버전 한글 8.5 → 설치본 중 가장 가까운 판): 이 문단은 **4줄**
//! (`45 / 38 / 36 / 32+\n`)이고 총 쪽수는 39다(rhwp 43 → 40).
#![cfg(not(target_arch = "wasm32"))]

use std::path::Path;
use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn sample() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("samples/issue6300/trade_report_forced_break_object.hwp")
        .to_string_lossy()
        .into_owned()
}

/// `dump-extents` 에서 각 `TextLine` 의 (글자수, 오른쪽 잉크 끝) 을 모은다.
fn all_text_lines() -> Vec<(usize, f64, String)> {
    let out = Command::new(rhwp_bin())
        .args(["dump-extents", &sample()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();

    let num = |line: &str, key: &str| -> Option<f64> {
        line.split(key)
            .nth(1)?
            .split_whitespace()
            .next()?
            .parse::<f64>()
            .ok()
    };

    let mut rows: Vec<(usize, f64, String)> = Vec::new();
    let mut chars = 0usize;
    let mut right = f64::MIN;
    let mut buf = String::new();
    fn flush(
        chars: &mut usize,
        right: &mut f64,
        buf: &mut String,
        rows: &mut Vec<(usize, f64, String)>,
    ) {
        if *chars > 0 && *right > f64::MIN {
            rows.push((*chars, *right, std::mem::take(buf)));
        }
        *chars = 0;
        *right = f64::MIN;
        buf.clear();
    }
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with("TextLine") {
            flush(&mut chars, &mut right, &mut buf, &mut rows);
        } else if line.starts_with("TextRun") {
            if let (Some(x), Some(w)) = (num(line, " x="), num(line, " w=")) {
                right = right.max(x + w);
            }
            // 마지막 따옴표 쌍이 run 텍스트다.
            if let Some(start) = line.find('"') {
                let body = &line[start + 1..];
                if let Some(end) = body.rfind('"') {
                    chars += body[..end].chars().count();
                    buf.push_str(&body[..end]);
                }
            }
        }
    }
    flush(&mut chars, &mut right, &mut buf, &mut rows);
    rows
}

/// `needle` 을 담은 `TextLine` 의 (글자수, 오른쪽 잉크 끝).
fn line_containing(needle: &str) -> (usize, f64) {
    for (chars, right, text) in all_text_lines() {
        if text.contains(needle) {
            return (chars, right);
        }
    }
    panic!("{needle} 을 담은 줄을 찾지 못했다");
}

#[test]
fn forced_break_at_line_end_does_not_merge_two_rows() {
    // 대상은 17쪽 `농수산식품` 문단이다. 종전에는 `현지인의 입맛…` 줄이 다음 줄과
    // 합쳐져 69자·우단 790.7px 이 됐고, 그 앞자리에 빈 줄이 남았다.
    // 수정 후에는 36자로 끊기고 본문 우단(729.9px) 안쪽에 든다.
    let (chars, right) = line_containing("현지인의 입맛");
    assert!(
        chars < 50,
        "두 줄이 한 줄로 합쳐졌다: {chars}자 (기대 36자 안팎)"
    );
    assert!(
        right < 760.0,
        "합쳐진 줄이 본문 우단을 넘었다: right={right:.1}px (본문 우단 729.9)"
    );
}

#[test]
fn page_count_moves_toward_the_hangul_oracle() {
    let out = Command::new(rhwp_bin())
        .args(["info", &sample()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{out:?}");
    let text = String::from_utf8_lossy(&out.stdout).into_owned();
    let pages = text
        .lines()
        .find_map(|l| l.strip_prefix("페이지 수:"))
        .and_then(|v| v.trim().parse::<u32>().ok())
        .unwrap_or_else(|| {
            panic!(
                "페이지 수를 읽지 못했다:
{text}"
            )
        });

    // 한글 2022 = 39쪽. 종전 rhwp 43쪽 → 수정 40쪽.
    // 남은 1쪽은 이 축이 아니므로 상한만 잠근다.
    assert!(
        pages <= 40,
        "줄 경계 손실로 쪽이 늘었다: {pages}쪽 (한글 39, 수정 후 기대 40 이하)"
    );
}
