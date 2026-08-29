//! `test-caption`은 고정 fixture 좌표 네 곳의 mutation과 verification이 모두
//! 성공한 경우에만 성공해야 한다. 임의 문서·부분 fixture에서도 panic하지 않고
//! false-pass 대신 exit 1을 반환하는지 실제 CLI 경계에서 고정한다.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::model::control::Control;
use rhwp::model::paragraph::Paragraph;
use rhwp::model::shape::ShapeObject;
use rhwp::wasm_api::HwpDocument;

fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

fn unique_temp_dir() -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos();
    let dir =
        std::env::temp_dir().join(format!("rhwp-test-caption-{}-{nonce}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("임시 폴더 생성 실패");
    dir
}

struct CaptionFixture {
    root: PathBuf,
    input: PathBuf,
    output: PathBuf,
}

impl CaptionFixture {
    fn with_second_paragraph_pictures(second_paragraph_pictures: usize) -> Self {
        assert!(second_paragraph_pictures <= 2);
        let root = unique_temp_dir();
        let input = root.join("caption-fixture.hwp");
        let output = root.join("svg");
        let mut doc = HwpDocument::create_empty();
        doc.document_mut().sections[0]
            .paragraphs
            .push(Paragraph::new_empty());
        let png = std::fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/logo/logo-16.png"),
        )
        .expect("tiny png");

        for (para, count) in [(0, 4), (1, second_paragraph_pictures)] {
            for _ in 0..count {
                doc.insert_picture_native(
                    0,
                    para,
                    0,
                    &[],
                    &png,
                    1200,
                    1200,
                    16,
                    16,
                    "png",
                    "caption fixture",
                    None,
                    None,
                )
                .expect("그림 삽입");
            }
        }

        let bytes = doc.export_hwp().expect("fixture HWP export");
        std::fs::write(&input, bytes).expect("fixture HWP 저장");
        Self {
            root,
            input,
            output,
        }
    }

    fn for_existing_input(input: PathBuf) -> Self {
        let root = unique_temp_dir();
        let output = root.join("svg");
        Self {
            root,
            input,
            output,
        }
    }
}

impl Drop for CaptionFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn run_test_caption(input: &Path, output: &Path) -> Output {
    Command::new(rhwp_bin())
        .args(["test-caption", input.to_str().expect("UTF-8 입력 경로")])
        .args(["--output", output.to_str().expect("UTF-8 출력 경로")])
        .output()
        .expect("test-caption 실행 실패")
}

fn assert_controlled_failure(out: &Output, output_dir: &Path) {
    let code = out.status.code();
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_ne!(
        code,
        Some(101),
        "Rust panic(exit 101) 발생 — 범위 밖 인덱싱 회귀. stderr: {stderr}"
    );
    assert_eq!(
        code,
        Some(1),
        "캡션 검증 실패는 exit 1이어야 합니다. stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        !stderr.trim().is_empty(),
        "실패 원인은 stderr에 기록해야 합니다. stdout: {stdout}"
    );
    assert!(
        !stdout.contains("완료"),
        "실패 실행이 성공 메시지를 출력하면 안 됩니다. stdout: {stdout}"
    );
    assert!(
        !output_dir.exists(),
        "검증 실패 전에 출력 폴더나 SVG를 만들면 안 됩니다: {}",
        output_dir.display()
    );
}

fn assert_success(out: &Output, output_dir: &Path) {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(0),
        "정상 fixture는 성공해야 합니다. stdout: {stdout}\nstderr: {stderr}"
    );
    assert_eq!(
        stdout.matches("caption=Some(").count(),
        4,
        "네 대상의 verification 증적이 모두 있어야 합니다: {stdout}"
    );
    assert!(stdout.contains("완료"), "성공 메시지가 없습니다: {stdout}");
    assert!(
        std::fs::read_dir(output_dir)
            .expect("출력 폴더 읽기 실패")
            .filter_map(Result::ok)
            .any(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("svg")),
        "정상 종료했다면 SVG가 하나 이상 생성되어야 합니다"
    );
}

#[test]
fn test_caption_rejects_all_fail_document_without_panic() {
    // fixture 전용 좌표가 없는 임의의 실문서: 네 mutation이 모두 실패해야 한다.
    let sample = std::fs::canonicalize("samples/2022년 국립국어원 업무계획.hwp")
        .expect("회귀 샘플이 저장소에 있어야 합니다");
    let fixture = CaptionFixture::for_existing_input(sample);
    let out = run_test_caption(&fixture.input, &fixture.output);
    assert_controlled_failure(&out, &fixture.output);
}

#[test]
fn test_caption_rejects_partial_failure_without_svg() {
    // para 0의 두 대상과 para 1의 첫 대상만 유효하고 마지막 (1,1)은 없다.
    let fixture = CaptionFixture::with_second_paragraph_pictures(1);
    let out = run_test_caption(&fixture.input, &fixture.output);
    assert_controlled_failure(&out, &fixture.output);
}

#[test]
fn test_caption_succeeds_only_when_all_targets_verify() {
    let fixture = CaptionFixture::with_second_paragraph_pictures(2);
    let out = run_test_caption(&fixture.input, &fixture.output);
    assert_success(&out, &fixture.output);
}

#[test]
fn test_shape_picture_caption_properties_round_trip() {
    let mut doc = HwpDocument::create_empty();
    let png =
        std::fs::read(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/logo/logo-16.png"))
            .expect("tiny png");
    doc.insert_picture_native(
        0,
        0,
        0,
        &[],
        &png,
        1200,
        1200,
        16,
        16,
        "png",
        "shape picture caption contract",
        None,
        None,
    )
    .expect("그림 삽입");

    let controls = &mut doc.document_mut().sections[0].paragraphs[0].controls;
    let picture = match controls.remove(0) {
        Control::Picture(picture) => picture,
        other => panic!("setter 계약 대상이 Picture가 아님: {other:?}"),
    };
    controls.insert(0, Control::Shape(Box::new(ShapeObject::Picture(picture))));

    doc.set_picture_properties_native(
        0,
        0,
        0,
        r#"{"hasCaption":true,"captionDirection":"Right","captionVertAlign":"Center","captionWidth":8504,"captionSpacing":850}"#,
    )
    .expect("Shape(Picture) 캡션 설정");
    let properties = doc
        .get_picture_properties_native(0, 0, 0)
        .expect("Shape(Picture) 캡션 재조회");
    let actual: serde_json::Value = serde_json::from_str(&properties).expect("그림 속성 JSON");
    assert_eq!(actual["hasCaption"], true);
    assert_eq!(actual["captionDirection"], "Right");
    assert_eq!(actual["captionVertAlign"], "Center");
    assert_eq!(actual["captionWidth"], 8504);
    assert_eq!(actual["captionSpacing"], 850);
}

#[test]
fn test_caption_self_description_declares_fixed_fixture_scope() {
    for args in [["--help"].as_slice(), ["capabilities"].as_slice()] {
        let out = Command::new(rhwp_bin())
            .args(args)
            .output()
            .expect("자기서술 명령 실행 실패");
        let stdout = String::from_utf8_lossy(&out.stdout);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert_eq!(
            out.status.code(),
            Some(0),
            "자기서술 명령은 성공해야 합니다. args={args:?} stderr={stderr}"
        );
        assert!(
            stdout.contains("test-caption") && stdout.contains("고정 fixture 캡션 라운드트립 검증"),
            "test-caption이 고정 fixture 전용임을 밝혀야 합니다. args={args:?} stdout={stdout}"
        );
    }
}
