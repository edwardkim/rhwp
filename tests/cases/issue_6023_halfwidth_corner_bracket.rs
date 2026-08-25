//! [#6023] 반각 낫표 ｢｣(U+FF62/FF63) 전진폭이 전각(1em)으로 계산되던 폴백
//! 오분류 가드.
//!
//! 30269(제도개선권고안, 22쪽 HWP5) 1쪽 "｢스크린 골프연습장 …" — 메트릭
//! DB 에 U+FF62/FF63 항목이 없어 폴백으로 빠지는데, `is_cjk_char` 의
//! FF00–FFEF 블록 블랭킷이 반각 구간(FF61–FFDC·FFE8–FFEE, 유니코드 정의상
//! 반각)까지 전각으로 분류했다. 한글 2020 COM PDF 실측: ｢ 전진 7.9pt =
//! 0.5em @ 15.95pt (rhwp 는 16.0pt = 1em). 수정 = 폴백 폭 분류에서 반각
//! 구간을 전각 블랭킷보다 먼저 가른다(메트릭 DB 항목이 생기면 그쪽이 이김).

#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const SAMPLE: &str = "samples/issue6023/30269_reform_recommendation.hwp";

fn render_p1_svg() -> String {
    let out = std::env::temp_dir().join(format!("rhwp_issue_6023_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("출력 디렉토리 생성");
    let done = Command::new(rhwp_bin())
        .current_dir(repo_root())
        .args([
            "export-svg",
            SAMPLE,
            "-p",
            "0",
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

/// `<text x=".." y="..">c</text>` 를 (y, x, 본문) 으로 거둔다.
fn glyphs(svg: &str) -> Vec<(f64, f64, String)> {
    let mut out = Vec::new();
    for chunk in svg.split("<text ").skip(1) {
        let Some(end) = chunk.find('>') else { continue };
        let (attrs, rest) = chunk.split_at(end);
        let Some(close) = rest.find("</text>") else {
            continue;
        };
        let body = &rest[1..close];
        let pick = |key: &str| -> Option<f64> {
            let at = attrs.find(&format!("{key}=\""))? + key.len() + 2;
            let tail = &attrs[at..];
            let stop = tail.find('"')?;
            tail[..stop].parse::<f64>().ok()
        };
        if let (Some(x), Some(y)) = (pick("x"), pick("y")) {
            out.push((y, x, body.to_string()));
        }
    }
    out
}

#[test]
fn issue_6023_halfwidth_corner_bracket_advances_half_em() {
    let svg = render_p1_svg();
    let glyphs = glyphs(&svg);

    // "｢스크린" — ｢ 와 뒤따르는 음절들을 같은 baseline 에서 찾는다.
    let (open_y, open_x) = glyphs
        .iter()
        .find(|(_, _, t)| t == "｢")
        .map(|(y, x, _)| (*y, *x))
        .expect("p1 에 반각 낫표 ｢ 가 있어야 한다");
    let mut line: Vec<(f64, &str)> = glyphs
        .iter()
        .filter(|(y, _, _)| (y - open_y).abs() < 0.5)
        .map(|(_, x, t)| (*x, t.as_str()))
        .collect();
    line.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let open_idx = line
        .iter()
        .position(|(x, t)| *t == "｢" && (*x - open_x).abs() < 0.1)
        .expect("정렬된 줄에서 ｢ 위치");
    assert!(open_idx + 2 < line.len(), "｢ 뒤 음절 2개가 있어야 한다");
    let bracket_advance = line[open_idx + 1].0 - line[open_idx].0;
    let syllable_advance = line[open_idx + 2].0 - line[open_idx + 1].0;

    // 한글 실측: ｢ = 0.5em, 음절 = 1em. 전각 오분류면 둘이 같아진다.
    assert!(
        bracket_advance < syllable_advance * 0.62,
        "반각 낫표 ｢ 전진폭({bracket_advance:.1}px)이 음절({syllable_advance:.1}px)의 \
         반각(≤0.62배)이어야 한다 — 전각 오분류(1em)면 FAIL",
    );
    assert!(
        bracket_advance > syllable_advance * 0.38,
        "반각 낫표 전진폭({bracket_advance:.1}px)이 과소하다 (음절 {syllable_advance:.1}px)",
    );
}
