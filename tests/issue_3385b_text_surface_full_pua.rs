//! Issue #3385 후속: 텍스트 추출이 **렌더의 글리프 치환 표를 쓰지 않아** PUA 가 그대로 나간다.
//!
//! #3499 는 사각 안 숫자(U+F02B1~F02C4) 한 대역만 막았다. 저장소 샘플 346건을 전수
//! 측정하니 추출 텍스트에 PUA 가 남는 문서가 **50건, 156,762자**였고, 그중 U+F080F
//! (한컴 굵은 가로선)만 **155,709자**였다. `hwp3-sample11.hwp` 는 한 쪽 1,398자 중
//! 181자가 이 문자이고 최장 96자 연속 — 머리말/꼬리말 가로선이 본문 텍스트로 나갔다.
//!
//! 정체는 저장소 안에 이미 있었다. `map_pua_bullet_char` 가 한컴 정답지 실측 근거와 함께
//! 표시 문자를 정의한다(`0xF080F => '━'`, `0xF0854 => '《'` 등). **렌더는 그 표를 쓰는데
//! 텍스트 추출은 쓰지 않았다.**
//!
//! 계약 두 가지를 고정한다.
//!   1) 추출 텍스트에는 표에 있는 PUA 가 남지 않는다
//!   2) 렌더는 건드리지 않는다 — `pua_to_text_surface` 는 추출 경로에서만 불린다
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

/// 머리말·꼬리말 가로선이 PUA 로 조판된 HWP 3.0 문서.
const LINE_SAMPLE: &str = "samples/hwp3-sample11.hwp";
/// 책괄호(《》)가 PUA 로 조판된 문서.
const BRACKET_SAMPLE: &str = "samples/exam_kor.hwp";
/// 중첩 표 안의 Wingdings 계열 글머리표가 반복되는 HWP 5.0 문서.
const TRIANGLE_BULLET_SAMPLE: &str = "samples/basic/issue2007_nested_cell_pagination_42065.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// nextest archive 는 런타임에 `CARGO_BIN_EXE_rhwp`를 주입한다(#3289).
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn out_dir(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-issue3385b-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn export(kind: &str, rel: &str, dir: &Path) -> String {
    let out = Command::new(rhwp_bin())
        .arg(kind)
        .arg(sample(rel))
        .arg("-o")
        .arg(dir)
        .output()
        .expect("rhwp 실행 실패");
    assert!(
        out.status.success(),
        "{kind} 실패: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let mut all = String::new();
    for entry in std::fs::read_dir(dir).expect("출력 디렉터리") {
        let path = entry.expect("항목").path();
        if path.is_file() {
            all.push_str(&std::fs::read_to_string(&path).unwrap_or_default());
        }
    }
    all
}

fn pua_chars(text: &str) -> Vec<char> {
    text.chars()
        .filter(|c| {
            let cp = *c as u32;
            (0xE000..=0xF8FF).contains(&cp) || (0xF0000..=0xFFFFD).contains(&cp)
        })
        .collect()
}

/// 가로선 PUA 가 추출 텍스트에서 사라지고 읽을 수 있는 괘선으로 바뀐다.
#[test]
fn heavy_horizontal_line_pua_is_readable_in_extracted_text() {
    let dir = out_dir("line");
    let text = export("export-text", LINE_SAMPLE, &dir);
    let leaked = pua_chars(&text);
    assert!(
        leaked.is_empty(),
        "추출 텍스트에 PUA 가 남았다 ({}건): {:?}",
        leaked.len(),
        &leaked[..leaked.len().min(3)]
    );
    assert!(
        text.contains('\u{2501}'),
        "굵은 가로선(━)으로 바뀐 흔적이 없다 — 매핑이 적용되지 않았을 수 있다"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// 책괄호 PUA(U+F0854·F0855)도 《 》 로 나온다.
#[test]
fn book_bracket_pua_becomes_angle_brackets() {
    let dir = out_dir("bracket");
    let text = export("export-text", BRACKET_SAMPLE, &dir);
    let leaked = pua_chars(&text);
    assert!(
        leaked.is_empty(),
        "추출 텍스트에 PUA 가 남았다 ({}건): {:?}",
        leaked.len(),
        &leaked[..leaked.len().min(3)]
    );
    assert!(
        text.contains('\u{300A}') && text.contains('\u{300B}'),
        "책괄호 《 》 로 바뀌지 않았다"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// 렌더는 그대로다 — 텍스트 표면 변환이 렌더까지 번지면 안 된다.
///
/// `map_pua_bullet_char` 는 원래 렌더 경로의 표이므로 SVG 에는 이미 치환된 글자가 나온다.
/// 여기서 고정하는 것은 **텍스트 전용 변환이 렌더 출력을 바꾸지 않는다**는 점이다 —
/// 사각 안 숫자는 렌더에서 원문 유지(캡스톤 F-1 결정)여야 한다.
#[test]
fn render_keeps_the_boxed_number_passthrough() {
    let dir = out_dir("svg");
    let svg = export("export-svg", "samples/2022년 국립국어원 업무계획.hwp", &dir);
    let boxed = svg
        .chars()
        .filter(|c| (0xF02B1..=0xF02C4).contains(&(*c as u32)))
        .count();
    assert!(
        boxed > 0,
        "렌더에서 사각 안 숫자 원문이 사라졌다 — 텍스트 표면 변환이 렌더까지 번졌다"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// `U+F02FB`는 전용 HNC 글꼴이 없으면 두부로 보이는 작은 오른쪽 삼각형이다.
///
/// 같은 composer 출력이 SVG와 Canvas에 공급되므로, SVG에 원문 PUA가 없고 `▸`가
/// 방출되는 계약을 고정한다. 텍스트 표면도 같은 검증된 대체값을 사용해야 한다.
#[test]
fn nested_table_triangle_bullet_is_font_independent_in_svg_and_text() {
    let svg_dir = out_dir("triangle-svg");
    let svg = export("export-svg", TRIANGLE_BULLET_SAMPLE, &svg_dir);
    assert!(
        !svg.contains('\u{F02FB}'),
        "SVG에 U+F02FB가 남으면 공개 글꼴 환경에서 두부가 된다"
    );
    assert!(
        svg.contains('▸'),
        "검증된 작은 오른쪽 삼각형 글머리표가 SVG에 방출되지 않았다"
    );

    let text_dir = out_dir("triangle-text");
    let text = export("export-text", TRIANGLE_BULLET_SAMPLE, &text_dir);
    assert!(
        !text.contains('\u{F02FB}'),
        "추출 텍스트에 U+F02FB가 남았다"
    );
    assert!(
        text.contains('▸'),
        "추출 텍스트에 읽을 수 있는 작은 오른쪽 삼각형이 없다"
    );

    let _ = std::fs::remove_dir_all(&svg_dir);
    let _ = std::fs::remove_dir_all(&text_dir);
}
