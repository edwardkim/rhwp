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

/// 렌더는 사각형+숫자 **벡터 합성**으로 그린다 — 폰트 정합 결정의 현행 형태.
///
/// 캡스톤 F-1 은 표준 ①~⑳ **글리프 매핑**을 되돌렸다(1순위 폰트의 원-안 글리프가
/// 한컴 사각-안 정답지와 발산). 그 뒤 #4158 이 CharOverlap 경로에, web_canvas 가
/// 평문 경로에 사각형+숫자 벡터 합성을 도입했고, [#6127] 이 SVG·Skia 평문 경로를
/// 같은 합성으로 맞췄다 — 함초롬 확장 글꼴이 없는 소비자에서 raw PUA 는 빈칸이
/// 되기 때문이다. 이 테스트는 (a) 원-안 글리프 매핑이 되살아나지 않는 것과
/// (b) raw PUA 가 렌더로 새지 않는 것을 함께 고정한다.
#[test]
fn rendered_svg_synthesizes_boxed_numbers() {
    let dir = out_dir("svg");
    let svg = run_export("export-svg", &dir);
    let kept = svg.chars().filter(|c| boxed_number_pua(*c)).count();
    assert_eq!(
        kept, 0,
        "렌더에 raw 사각 안 숫자 PUA 가 남았다 — 글꼴 부재 환경에서 빈칸이 된다"
    );
    // 원-안 글리프 매핑(캡스톤 F-1 이 되돌린 발산)이 아니라 사각형 합성인지는
    // 합성 상자의 stroke rect 존재로 확인한다 — 문서 본문에 실제 ① 글자가 있을
    // 수 있어 ① 부재로는 판정하지 못한다.
    assert!(
        svg.contains("fill=\"none\" stroke=\"#"),
        "합성 사각형 rect 가 없다 — 벡터 합성이 적용되지 않았다"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
