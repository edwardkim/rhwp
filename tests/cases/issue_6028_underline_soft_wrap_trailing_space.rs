//! [#6028] 셀 밑줄이 soft-wrap 줄-말미 공백(+배분 여분)까지 이어져 표 오른쪽
//! 괘선 밖으로 나가던 결함 가드.
//!
//! 2307287(건설기계 규격표시방법, 76KB HWP5) 4쪽 "저.트럭지게차" 칸 — 밑줄
//! 문단의 soft-wrap 줄에서 밑줄이 마지막 글리프를 6.5~9.8pt 지나 표 밖 여백
//! 까지 그려졌다(한글 2020 PDF: 글자 끝에서 끊음). 근인 = 밑줄 길이를
//! `char_positions.last()`(줄-말미 공백 + 배분 여분 포함)로 잡던 것. 한글은
//! wrap 이 소비한 구분 공백을 줄에 남기지 않으므로 soft-wrap 줄의 말미 공백은
//! 장식선에서 제외한다. 문단 마지막 줄·강제 줄바꿈 줄의 밑줄 친 말미 공백
//! (서명란, issue_157 직선 골든)은 저자 콘텐츠라 유지 — 그 구별은 줄 마지막
//! 텍스트 run 의 is_para_end/is_line_break_end 로 한다.

#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

const SAMPLE: &str = "samples/issue6028/2307287_construction_machinery_spec.hwp";

fn render_p4_svg() -> String {
    let out = std::env::temp_dir().join(format!("rhwp_issue_6028_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).expect("출력 디렉토리 생성");
    let done = Command::new(rhwp_bin())
        .current_dir(repo_root())
        .args([
            "export-svg",
            SAMPLE,
            "-p",
            "3",
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

fn attr(attrs: &str, key: &str) -> Option<f64> {
    let at = attrs.find(&format!("{key}=\""))? + key.len() + 2;
    let tail = &attrs[at..];
    let stop = tail.find('"')?;
    tail[..stop].parse::<f64>().ok()
}

#[test]
fn issue_6028_underline_stops_at_last_glyph_on_soft_wrap_lines() {
    let svg = render_p4_svg();

    // 마지막 글리프('을') — y≈690 줄에서 x 최대 <text>.
    let mut glyph_end: Option<f64> = None;
    for chunk in svg.split("<text ").skip(1) {
        let Some(end) = chunk.find('>') else { continue };
        let (attrs, rest) = chunk.split_at(end);
        if !rest[1..].starts_with("을</text>") {
            continue;
        }
        let (Some(x), Some(y), Some(fs)) =
            (attr(attrs, "x"), attr(attrs, "y"), attr(attrs, "font-size"))
        else {
            continue;
        };
        if (688.0..=694.0).contains(&y) {
            let e = x + fs; // 한글 음절 advance = 1em
            if glyph_end.is_none_or(|g| e > g) {
                glyph_end = Some(e);
            }
        }
    }
    let glyph_end = glyph_end.expect("p4 y≈690 줄의 '을' 글리프");

    // 그 줄의 밑줄(수평 단선) 끝.
    let mut underline_x2: Option<f64> = None;
    for chunk in svg.split("<line ").skip(1) {
        let Some(end) = chunk.find("/>") else {
            continue;
        };
        let attrs = &chunk[..end];
        let (Some(y1), Some(y2), Some(x1), Some(x2)) = (
            attr(attrs, "y1"),
            attr(attrs, "y2"),
            attr(attrs, "x1"),
            attr(attrs, "x2"),
        ) else {
            continue;
        };
        if (y1 - y2).abs() < 0.01 && (691.0..=695.0).contains(&y1) && x2 - x1 > 100.0 {
            underline_x2 = Some(x2);
        }
    }
    let underline_x2 = underline_x2.expect("p4 y≈693 밑줄");

    // 한글은 밑줄을 마지막 글리프에서 끊는다. soft-wrap 말미 공백(+여분)이
    // 밑줄에 얹히면 +8px 이상 길어져 표 오른쪽 괘선(677.3px) 밖으로 나간다.
    assert!(
        underline_x2 <= glyph_end + 1.0,
        "밑줄 끝({underline_x2:.1})이 마지막 글리프 끝({glyph_end:.1})을 지나침 — \
         soft-wrap 말미 공백이 장식선에 포함됨",
    );
    assert!(
        underline_x2 >= glyph_end - 1.5,
        "밑줄 끝({underline_x2:.1})이 마지막 글리프({glyph_end:.1})에 못 미침 — 과소 트림",
    );
}
