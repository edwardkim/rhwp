//! Issue #3495: 저장하면 미주 **앞의 공백 한 칸이 사라진다**.
//!
//! `convert` 로 HWP5 를 쓰면 미주가 있는 문단에서 공백이 하나 줄고, 그 뒤 텍스트가 통째로
//! 한 칸 당겨진다. `--verify` 는 exit 0 이라 종료코드로는 드러나지 않는다.
//!
//! ## 근인
//!
//! PARA_TEXT 저장기의 **자동번호 placeholder 휴리스틱**이 각주·미주까지 받고 있었다.
//!
//! ```text
//! ch == ' ' && offset == prev_end && 다음 컨트롤 코드 ∈ {0x0011, 0x0012} && next_offset >= offset + 8
//! ```
//!
//! placeholder 공백을 만드는 것은 **0x0012(자동번호)뿐이다** — `parser/body_text.rs` 는
//! `ch == 0x0012` 일 때만, HWPX `section.rs` 는 `\u{0012}` 파트일 때만 text 에 공백을
//! push 한다. 각주·미주(0x0011)는 placeholder 를 만들지 않으므로, 그 앞에 놓인 공백은
//! **진짜 공백**인데 휴리스틱이 이를 컨트롤로 덮어썼다.
//!
//! 조건의 나머지(`offset == prev_end`, `next_offset >= offset + 8`)는 연속 텍스트 뒤에
//! 컨트롤이 오면 자연히 성립해서 판별력이 없다. 실제 판별자는 컨트롤 코드뿐이다.
#![cfg(not(target_arch = "wasm32"))]

use rhwp::model::control::Control;

/// 미주가 본문 중간에 앵커된 HWP 3.0 문서.
const SAMPLE: &str = "samples/SO-SUEOP.hwp";

fn load(path: &std::path::Path) -> rhwp::model::document::Document {
    let data = std::fs::read(path).expect("파일 읽기");
    rhwp::parser::parse_document(&data).expect("파싱")
}

fn sample_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// nextest archive 는 런타임에 `CARGO_BIN_EXE_rhwp`를 주입한다(#3289).
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

/// 미주를 가진 문단의 (텍스트, 미주 위치) 목록.
fn endnote_paragraphs(doc: &rhwp::model::document::Document) -> Vec<(String, Vec<usize>)> {
    doc.sections
        .iter()
        .flat_map(|s| s.paragraphs.iter())
        .filter(|p| {
            p.controls
                .iter()
                .any(|c| matches!(c, Control::Endnote(_) | Control::Footnote(_)))
        })
        .map(|p| (p.text.clone(), p.control_text_positions()))
        .collect()
}

fn convert_roundtrip() -> std::path::PathBuf {
    let out = std::env::temp_dir().join(format!(
        "rhwp-issue3495-{}-{}.hwp",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    let status = std::process::Command::new(rhwp_bin())
        .arg("convert")
        .arg(sample_path())
        .arg(&out)
        .output()
        .expect("rhwp 실행");
    assert!(
        status.status.success(),
        "convert 실패: {}",
        String::from_utf8_lossy(&status.stderr)
    );
    out
}

/// 미주가 있는 문단의 텍스트가 저장으로 바뀌지 않는다.
#[test]
fn saving_keeps_the_space_before_an_endnote() {
    let before = endnote_paragraphs(&load(&sample_path()));
    assert!(
        !before.is_empty(),
        "미주 문단을 찾지 못했다 — 표본이 바뀌었는지 확인하라"
    );

    let out = convert_roundtrip();
    let after = endnote_paragraphs(&load(&out));
    let _ = std::fs::remove_file(&out);

    assert_eq!(after.len(), before.len(), "저장 후 미주 문단 수가 달라졌다");
    for (i, ((text_a, _), (text_b, _))) in before.iter().zip(after.iter()).enumerate() {
        assert_eq!(
            text_a.chars().count(),
            text_b.chars().count(),
            "미주 문단 {i} 의 글자 수가 달라졌다 (공백이 먹혔는지 확인):\n  전: {text_a:?}\n  후: {text_b:?}"
        );
        assert_eq!(text_a, text_b, "미주 문단 {i} 의 텍스트가 달라졌다");
    }
}

/// 미주 표시 위치도 그대로다 — 앞 문자가 사라지면 위치가 당겨진다.
#[test]
fn endnote_marker_position_survives_saving() {
    let before = endnote_paragraphs(&load(&sample_path()));
    let out = convert_roundtrip();
    let after = endnote_paragraphs(&load(&out));
    let _ = std::fs::remove_file(&out);

    let mut checked = 0usize;
    for (i, ((_, pos_a), (_, pos_b))) in before.iter().zip(after.iter()).enumerate() {
        // 본문 중간 앵커(위치가 0 이 아닌 것)만 — 문단 첫머리는 밀릴 여지가 없다.
        if pos_a.iter().all(|p| *p == 0) {
            continue;
        }
        assert_eq!(
            pos_a, pos_b,
            "미주 표시 위치가 저장으로 이동했다 (문단 {i})"
        );
        checked += 1;
    }
    assert!(checked > 0, "본문 중간 앵커 미주를 찾지 못했다");
}
