//! Issue #3385: 한컴 PUA 사각 안 숫자가 텍스트 추출에 원문 그대로 남는다.
//!
//! `export-text` 는 표시 문자열 변환을 하지 않아 U+F02B1~F02C4 를 그대로 내보냈다.
//! 추출 결과는 폰트가 없는 소비자(RAG·LLM·grep)에게 가므로 **읽을 수 없는 코드포인트**다.
//!
//! 중요한 경계: **렌더는 원문 유지가 맞다.** Task #509 → 캡스톤 F-1 에서 표준 ①~⑳ 매핑을
//! 일부러 되돌렸다 — 매핑하면 1순위 폰트의 *원 안* 글리프가 즉시 잡혀 한컴 정답지의
//! *사각 안* 글리프와 멀어지기 때문이다. 그래서 이 수정은 **텍스트 표면에만** 적용하고
//! 렌더 출력은 건드리지 않는다. 두 계약을 함께 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

/// 섹션 헤딩 번호가 사각 안 숫자 PUA 로 조판된 실물 문서.
const SAMPLE: &str = "samples/2022년 국립국어원 업무계획.hwp";

fn sample_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

fn out_dir(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-issue3385-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn run_export(kind: &str, dir: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .arg(kind)
        .arg(sample_path())
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

/// 사각 안 숫자 PUA 대역.
fn boxed_number_pua(ch: char) -> bool {
    (0xF02B1..=0xF02C4).contains(&(ch as u32))
}

/// 텍스트 추출은 읽을 수 있는 문자를 준다.
#[test]
fn extracted_text_has_no_boxed_number_pua() {
    let dir = out_dir("text");
    let text = run_export("export-text", &dir);
    let leaked: Vec<char> = text.chars().filter(|c| boxed_number_pua(*c)).collect();
    assert!(
        leaked.is_empty(),
        "추출 텍스트에 사각 안 숫자 PUA 가 남았다 ({}건): {:?}",
        leaked.len(),
        &leaked[..leaked.len().min(3)]
    );
    // 읽을 수 있는 둘러싸인 숫자로 바뀌어야 한다.
    assert!(
        text.contains('\u{2460}'),
        "① 로 바뀐 흔적이 없다 — 매핑이 적용되지 않았을 수 있다"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// 렌더는 원문 PUA 를 그대로 흘린다 — 폰트 정합 결정(Task #509)을 깨지 않는다.
#[test]
fn rendered_svg_keeps_the_raw_pua() {
    let dir = out_dir("svg");
    let svg = run_export("export-svg", &dir);
    let kept = svg.chars().filter(|c| boxed_number_pua(*c)).count();
    assert!(
        kept > 0,
        "렌더에서 원문 PUA 가 사라졌다 — 텍스트 표면 변환이 렌더까지 번졌다"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
