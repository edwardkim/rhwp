//! [#5542/#5532] HWP3 구역 첫 문단의 secd/cold 8유닛 슬롯 계상 계약.
//!
//! HWP5/HWPX 위치 좌표계는 구역 첫 문단 앞머리의 확장 컨트롤(secd·cold)을
//! 8유닛씩 계상한다. HWP3 파서는 두 컨트롤을 스트림 밖에서 합성하므로 종전에는
//! 계상이 빠졌고, h2x 저장-재파싱에서 첫 문단 char_shapes 경계가 컨트롤 몫만큼
//! 어긋났다(hwp3-curve·hwp3-sample5: +16, `--verify` exit 3). SectionDef 컨트롤
//! 합성 없이 좌표만 밀면 HWPX 직렬화기의 hidden-슬롯 정합이 깨져 저장 lineseg 가
//! 억제된다 — 계약은 CLI `--verify` 왕복 무차이로 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn repo_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn temp(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-5542-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn verify_roundtrip_clean(sample: &str) {
    let out_path = temp(sample.trim_end_matches(".hwp"));
    let output = Command::new(rhwp_bin())
        .arg("export-hwpx")
        .arg(repo_path(&format!("samples/{sample}")))
        .arg(&out_path)
        .arg("--verify")
        .output()
        .expect("export-hwpx 실행");
    let _ = std::fs::remove_file(&out_path);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.code() == Some(0) && stdout.contains("IR 차이 없음"),
        "{sample} h2x --verify 왕복이 깨졌다 (exit={:?}):\n{stdout}\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn hwp3_curve_roundtrip_char_shapes_clean() {
    // [#5542] 종전: paragraph[0] char_shapes (10,·)→(26,·) — secd+cold 16유닛 미계상.
    verify_roundtrip_clean("hwp3-curve.hwp");
}

#[test]
fn hwp3_sample5_roundtrip_char_shapes_clean() {
    // [#5532] 종전: paragraph[0] char_shapes +16 동형.
    verify_roundtrip_clean("hwp3-sample5.hwp");
}

#[test]
fn hwp3_first_paragraph_carries_section_def_control() {
    // IR 계약(HWP5 파서 동형): 구역 첫 문단 controls 는 SectionDef 로 시작하고,
    // char_shapes 의 후속 경계는 앞머리 슬롯(8×n)을 포함한 좌표다.
    let bytes = std::fs::read(repo_path("samples/hwp3-curve.hwp")).expect("샘플 읽기");
    let doc = HwpDocument::from_bytes(&bytes).expect("파싱");
    let first = &doc.document().sections[0].paragraphs[0];
    assert!(
        matches!(first.controls.first(), Some(Control::SectionDef(_))),
        "구역 첫 문단 앞머리에 SectionDef 컨트롤이 없다: {:?}",
        first
            .controls
            .iter()
            .map(std::mem::discriminant)
            .collect::<Vec<_>>()
    );
    let leading_slots = first
        .controls
        .iter()
        .take_while(|c| matches!(c, Control::SectionDef(_) | Control::ColumnDef(_)))
        .count() as u32;
    assert!(leading_slots >= 2, "secd+cold 연쇄가 없다: {leading_slots}");
    // 두 번째 char_shapes 경계(텍스트 10자 뒤)는 슬롯 좌표를 포함해야 한다.
    let second = first.char_shapes.get(1).expect("경계 2개");
    assert_eq!(
        second.start_pos,
        leading_slots * 8 + 10,
        "char_shapes 경계가 슬롯 좌표를 계상하지 않았다"
    );
}
