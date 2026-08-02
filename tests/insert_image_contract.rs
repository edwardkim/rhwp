//! [#3719 §6-5] `edit insert-image` 출력·안전 계약 회귀 테스트 (도장·서명 삽입).
//!
//! 실물 서식 제출의 마지막 조각이다. 검증 원칙은 형제 편집 명령(#3381 set-cell)과 같다:
//! ① `--dry-run` 은 파일을 만들지 않는다 ② 실패 경로의 stdout 은 0바이트 ③ 반영 여부는
//! **산출물 재파싱**으로 확인한다(선언을 믿지 않는다) ④ 쪽 밖 배치는 조용히 잘리지 않고
//! `overflow` 로 보고된다.
//!
//! 단위 함정이 이 명령의 핵심 위험이다 — 길이는 전부 HWPUNIT(1/7200 inch)이며 픽셀이
//! 아니다. px 로 오해하면 도장이 점만 하게 찍혀도 종료 코드는 0 이므로, 크기 규약
//! (생략=원본 픽셀 ×75 / 한쪽만=비율 유지)을 값으로 고정한다.
#![cfg(not(target_arch = "wasm32"))]

use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use rhwp::model::control::Control;
use rhwp::wasm_api::HwpDocument;

/// 3쪽짜리 실물 HWP5 — 형제 편집 계약 시험이 쓰는 것과 같은 문서.
const SAMPLE: &str = "samples/field-01.hwp";

/// 96dpi 픽셀 1개 = 75 HWPUNIT. CLI 의 기본 환산비와 같은 상수를 시험 쪽에서도 고정한다.
const HWPUNIT_PER_PX: u32 = 75;

fn sample() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(SAMPLE)
}

/// argv 에 그대로 실을 수 있는 소유 문자열 — `&[&str]` 배열이 임시값을 빌리지 않게 한다.
fn sample_arg() -> String {
    sample().to_string_lossy().to_string()
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-insimg-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], output: &Output) -> String {
    format!(
        "명령: rhwp {}\nstdout:\n{}\nstderr:\n{}",
        args.join(" "),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn parse_json(args: &[&str], output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(args, output)
        )
    })
}

/// 시험 시점에 도장 대용 그림을 합성한다 — 이미지 파일을 저장소에 커밋하지 않기 위해서다.
/// 가로세로를 다르게 잡아 "한쪽만 지정 시 비율 유지"를 값으로 확인할 수 있게 한다.
fn write_stamp_as(path: &Path, format: image::ImageFormat, width: u32, height: u32) {
    // JPEG 은 알파 채널을 받지 않는다 — 형식별 색공간 차이는 인코더에 맡기고
    // 시험은 "확장자별로 받아들이는가" 만 본다.
    let image = image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        width,
        height,
        image::Rgb([210, 30, 30]),
    ));
    let mut bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut bytes), format)
        .unwrap_or_else(|e| panic!("{format:?} 인코딩 실패: {e}"));
    std::fs::write(path, &bytes).expect("도장 그림 쓰기");
}

fn write_stamp_png(path: &Path, width: u32, height: u32) {
    write_stamp_as(path, image::ImageFormat::Png, width, height);
}

/// 문서의 쪽 수를 **문서에게 물어본다** — 샘플이 바뀌어도 시험이 거짓말하지 않게.
fn page_count_of(path: &str) -> u64 {
    let args = ["info", "--json", path];
    let output = run(&args);
    let v = parse_json(&args, &output);
    v["pageCount"].as_u64().expect("info.pageCount")
}

/// 용지 크기(HWPUNIT)를 **CLI 에게 물어본다**.
///
/// A4 를 상수로 박으면 다른 규격(A3·가로)의 샘플에서 시험이 조용히 무의미해진다.
/// 확실히 넘치는 좌표로 `--dry-run` 을 한 번 돌려 overflow 보고에서 용지 크기를 읽는다
/// (파일을 만들지 않으므로 부작용 0).
fn paper_size_hu(path: &str, image: &str) -> (i64, i64) {
    let probe = "1000000";
    let args = [
        "edit",
        "insert-image",
        path,
        "--image",
        image,
        "--x",
        probe,
        "--y",
        probe,
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    let v = parse_json(&args, &output);
    let report = &v["overflow"][0];
    (
        report["paperWidthHu"].as_i64().expect("paperWidthHu"),
        report["paperHeightHu"].as_i64().expect("paperHeightHu"),
    )
}

/// `capabilities --mcp` 가 선언한 `hwp_insert_image` 도구 정의.
fn insert_image_tool_definition() -> serde_json::Value {
    let args = ["capabilities", "--mcp"];
    let output = run(&args);
    let v = parse_json(&args, &output);
    v["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_insert_image")
        .expect("hwp_insert_image 도구")
        .clone()
}

/// 산출물을 다시 파싱해 **실제로 들어간** 그림들의 (binDataId, 폭, 높이, x, y) 를 모은다.
/// 선언(봉투)이 아니라 파일이 답하게 한다.
fn pictures_of(path: &Path) -> Vec<(u16, u32, u32, u32, u32)> {
    let bytes = std::fs::read(path).expect("산출물 읽기");
    let doc = HwpDocument::from_bytes(&bytes).expect("산출물 파싱");
    let mut found = Vec::new();
    for section in &doc.document().sections {
        for paragraph in &section.paragraphs {
            for control in &paragraph.controls {
                if let Control::Picture(picture) = control {
                    found.push((
                        picture.image_attr.bin_data_id,
                        picture.common.width,
                        picture.common.height,
                        picture.common.horizontal_offset,
                        picture.common.vertical_offset,
                    ));
                }
            }
        }
    }
    found
}

// ── ① 삽입 본류: 봉투 + 산출물 재파싱 ────────────────────────────────────────

#[test]
fn insert_image_writes_picture_and_reports_placement() {
    let src = sample_arg();
    let stamp = temp_path("stamp", "png");
    write_stamp_png(&stamp, 40, 20);
    let out = temp_path("out", "hwp");

    // 좌표는 박지 않고 **용지 크기에서 유도**한다 — 규격이 다른 샘플로 바뀌어도
    // "쪽 안쪽 배치" 라는 전제가 유지된다(도장은 보통 우하단에 찍힌다).
    let (paper_w, paper_h) = paper_size_hu(src.as_str(), stamp.to_str().unwrap());
    let width = (paper_w / 8).to_string();
    let height = (paper_h / 16).to_string();
    let x = (paper_w / 2).to_string();
    let y = (paper_h / 2).to_string();

    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "--page",
        "0",
        "--x",
        x.as_str(),
        "--y",
        y.as_str(),
        "--width",
        width.as_str(),
        "--height",
        height.as_str(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let v = parse_json(&args, &output);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert_eq!(v["page"], 0, "{v}");
    assert_eq!(v["x"].as_i64(), Some(paper_w / 2), "{v}");
    assert_eq!(v["y"].as_i64(), Some(paper_h / 2), "{v}");
    assert_eq!(v["width"].as_i64(), Some(paper_w / 8), "{v}");
    assert_eq!(v["height"].as_i64(), Some(paper_h / 16), "{v}");
    assert_eq!(v["dryRun"], false, "{v}");
    assert_eq!(v["outputFormat"], "hwp5", "{v}");
    assert!(
        v["overflow"].as_array().expect("overflow 배열").is_empty(),
        "쪽 안쪽 배치인데 넘침으로 보고됐습니다: {v}"
    );
    // 눈검증 대상 쪽은 비어 있지 않아야 한다 — 없으면 에이전트가 검증을 건너뛴다.
    let changed = v["changedPages"].as_array().expect("changedPages 배열");
    assert!(!changed.is_empty(), "{v}");

    let bin_data_id = v["binDataId"].as_u64().expect("binDataId") as u16;
    assert!(bin_data_id > 0, "BinData 등록 번호가 0 입니다: {v}");

    // 파일이 답한다 — 봉투가 말한 그대로의 그림이 실제로 들어갔는가.
    assert!(out.exists(), "{}", describe(&args, &output));
    let expected = (
        bin_data_id,
        (paper_w / 8) as u32,
        (paper_h / 16) as u32,
        (paper_w / 2) as u32,
        (paper_h / 2) as u32,
    );
    let pictures = pictures_of(&out);
    assert!(
        pictures.contains(&expected),
        "산출물에 봉투가 말한 그림이 없습니다: 기대 {expected:?} / 실제 {pictures:?} / {v}"
    );

    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&stamp);
}

/// 봉투가 **자기서술과 같은 말**을 하는가 — 매니페스트의 outputFields 전부가 실제
/// 봉투 키로 나와야 한다. 선언만 있고 값이 없으면 파서를 자동 생성한 에이전트가 깨진다.
#[test]
fn envelope_carries_every_declared_output_field() {
    let src = sample_arg();
    let stamp = temp_path("fields", "png");
    write_stamp_png(&stamp, 40, 20);
    let out = temp_path("fields-out", "hwp");

    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--verify",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    let envelope = v.as_object().expect("봉투는 JSON 객체");

    // 필드 이름을 시험에 박지 않는다 — 선언(매니페스트)을 읽어 그대로 대조한다.
    let declared = insert_image_tool_definition();
    let fields: Vec<&str> = declared["outputFields"]
        .as_array()
        .expect("outputFields")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    assert!(fields.len() >= 10, "선언이 너무 얕습니다: {fields:?}");
    let missing: Vec<&str> = fields
        .iter()
        .copied()
        .filter(|f| !envelope.contains_key(*f))
        .collect();
    assert!(
        missing.is_empty(),
        "선언했는데 봉투에 없는 키: {missing:?}\n봉투: {v}"
    );

    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&stamp);
}

// ── ② 크기 규약 (HWPUNIT 함정) ───────────────────────────────────────────────

#[test]
fn omitted_size_uses_natural_pixels_in_hwpunit() {
    let src = sample_arg();
    // 크기를 안 주면 원본 픽셀을 96dpi 로 환산한다(px × 75). px 를 그대로 쓰면
    // 도장이 1/75 크기로 찍히고도 종료 코드는 0 이므로 값으로 못 박는다.
    let stamp = temp_path("natural", "png");
    write_stamp_png(&stamp, 40, 20);
    let out = temp_path("natural-out", "hwp");

    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert_eq!(v["width"], 40 * HWPUNIT_PER_PX, "{v}");
    assert_eq!(v["height"], 20 * HWPUNIT_PER_PX, "{v}");

    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&stamp);
}

#[test]
fn single_axis_size_keeps_natural_aspect_ratio() {
    let src = sample_arg();
    // 도장은 보통 "폭만" 정해진다. 나머지 축을 0 이나 원본 픽셀로 두면 찌그러지므로
    // 원본 비율(40:20)을 지켜 채우고, 결과를 봉투에 실어 조용한 보정이 없게 한다.
    let stamp = temp_path("ratio", "png");
    write_stamp_png(&stamp, 40, 20);
    let out = temp_path("ratio-out", "hwp");

    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "--width",
        "8000",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert_eq!(v["width"], 8000, "{v}");
    assert_eq!(v["height"], 4000, "{v}");

    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&stamp);
}

// ── ③ dry-run 은 산출물을 만들지 않는다 ──────────────────────────────────────

#[test]
fn dry_run_creates_no_file_and_reports_plan_only() {
    let src = sample_arg();
    let stamp = temp_path("dry", "png");
    write_stamp_png(&stamp, 40, 20);
    let out = temp_path("dry-out", "hwp");

    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "--x",
        "1000",
        "--y",
        "2000",
        "-o",
        out.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert_eq!(v["dryRun"], true, "{v}");
    assert!(
        v["binDataId"].is_null(),
        "실행 0 인데 등록 번호가 있습니다: {v}"
    );
    assert!(v["output"].is_null(), "{v}");
    // 예측 목록을 실측 목록으로 오인하지 않도록 dry-run 은 null 이다(#3712 규약).
    assert!(v["changedPages"].is_null(), "{v}");
    assert!(
        !out.exists(),
        "--dry-run 이 파일을 만들었습니다: {}",
        out.display()
    );

    let _ = std::fs::remove_file(&stamp);
}

// ── ④ 쪽 밖 배치는 자르지 않고 보고한다 ─────────────────────────────────────

#[test]
fn out_of_page_placement_is_reported_not_silently_clipped() {
    let src = sample_arg();
    // 에이전트는 렌더 결과를 보지 않는다. 신호가 없으면 쪽 밖으로 나간 도장을
    // 완성본으로 판단하므로, 자르지 말고 넘친 양을 숫자로 알려야 한다.
    let stamp = temp_path("overflow", "png");
    write_stamp_png(&stamp, 40, 20);
    let out = temp_path("overflow-out", "hwp");

    // 용지 크기를 물어본 뒤 **딱 1 HWPUNIT 넘치게** 놓는다. 경계에서 반응하는지가
    // 핵심이다 — 아주 큰 값으로만 시험하면 경계 off-by-one 을 못 잡는다.
    let (paper_w, paper_h) = paper_size_hu(src.as_str(), stamp.to_str().unwrap());
    let width_hu = 6000i64;
    let height_hu = 3000i64;
    let x = (paper_w - width_hu + 1).to_string();
    let y = (paper_h - height_hu + 1).to_string();

    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "--x",
        x.as_str(),
        "--y",
        y.as_str(),
        "--width",
        "6000",
        "--height",
        "3000",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "쪽 밖 배치는 실패가 아니라 보고 대상이다: {}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    let overflow = v["overflow"].as_array().expect("overflow 배열");
    assert_eq!(overflow.len(), 1, "쪽 밖 배치인데 보고가 없습니다: {v}");
    let report = &overflow[0];
    assert_eq!(report["overflowXHu"].as_i64(), Some(1), "{v}");
    assert_eq!(report["overflowYHu"].as_i64(), Some(1), "{v}");
    assert_eq!(report["paperWidthHu"].as_i64(), Some(paper_w), "{v}");
    assert_eq!(report["paperHeightHu"].as_i64(), Some(paper_h), "{v}");

    // 자르지 않았다 — 요청한 좌표·크기가 그대로 들어가 있어야 한다.
    let pictures = pictures_of(&out);
    assert!(
        pictures.iter().any(|p| p.1 == width_hu as u32
            && p.2 == height_hu as u32
            && p.3 == (paper_w - width_hu + 1) as u32
            && p.4 == (paper_h - height_hu + 1) as u32),
        "쪽 밖 배치가 조용히 보정됐습니다: {pictures:?}"
    );

    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&stamp);
}

/// 경계 **안쪽** 1 HWPUNIT 은 넘침이 아니다 — 넘침 판정이 과민하면 정상 배치마다
/// 거짓 경보가 뜨고, 에이전트는 경보 자체를 무시하게 된다.
#[test]
fn exact_page_edge_is_not_reported_as_overflow() {
    let src = sample_arg();
    let stamp = temp_path("edge", "png");
    write_stamp_png(&stamp, 40, 20);

    let (paper_w, paper_h) = paper_size_hu(src.as_str(), stamp.to_str().unwrap());
    let width_hu = 6000i64;
    let height_hu = 3000i64;
    let x = (paper_w - width_hu).to_string();
    let y = (paper_h - height_hu).to_string();

    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "--x",
        x.as_str(),
        "--y",
        y.as_str(),
        "--width",
        "6000",
        "--height",
        "3000",
        "--dry-run",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert!(
        v["overflow"].as_array().expect("overflow 배열").is_empty(),
        "용지 경계에 정확히 닿는 배치를 넘침으로 봤습니다: {v}"
    );

    let _ = std::fs::remove_file(&stamp);
}

// ── ⑤ 인자 오류 (exit 2) + stdout 0바이트 ───────────────────────────────────

#[test]
fn unsupported_image_format_is_usage_error_with_silent_stdout() {
    let src = sample_arg();
    // 지원하지 않는 형식은 **인자 문제**다 — 런타임 실패(1)가 아니라 2 로 끊는다.
    let bogus = temp_path("bogus", "svg");
    std::fs::write(&bogus, b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>").expect("bogus 쓰기");
    let out = temp_path("bogus-out", "hwp");

    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        bogus.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 0바이트여야 합니다: {}",
        describe(&args, &output)
    );
    assert!(!out.exists(), "실패했는데 산출물이 생겼습니다");

    let _ = std::fs::remove_file(&bogus);
}

#[test]
fn extension_lie_is_caught_by_magic_bytes() {
    let src = sample_arg();
    // 확장자만 믿으면 BinData 에 그림 아닌 바이트가 들어가고, 크기를 못 재
    // 배치 좌표가 의미를 잃는다. 내용으로 다시 판정해 인자 오류로 끊는다.
    let liar = temp_path("liar", "png");
    std::fs::write(&liar, b"NOT-A-PNG-AT-ALL").expect("liar 쓰기");

    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        liar.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));

    let _ = std::fs::remove_file(&liar);
}

#[test]
fn page_out_of_range_is_usage_error() {
    let src = sample_arg();
    let stamp = temp_path("range", "png");
    write_stamp_png(&stamp, 40, 20);

    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "--page",
        "9999",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));

    let _ = std::fs::remove_file(&stamp);
}

#[test]
fn missing_image_argument_is_usage_error() {
    let src = sample_arg();
    let args = ["edit", "insert-image", src.as_str(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
}

#[test]
fn negative_or_non_numeric_length_is_usage_error() {
    let src = sample_arg();
    // 음수 오프셋은 코어가 0 으로 깎는다 — 조용한 보정 대신 인자 오류로 끊는다.
    let stamp = temp_path("neg", "png");
    write_stamp_png(&stamp, 40, 20);

    for bad in ["-100", "3.5", "3000px"] {
        let args = [
            "edit",
            "insert-image",
            src.as_str(),
            "--image",
            stamp.to_str().unwrap(),
            "--x",
            bad,
            "--json",
        ];
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{}",
            describe(&args, &output)
        );
        assert!(output.stdout.is_empty(), "{}", describe(&args, &output));
    }

    let _ = std::fs::remove_file(&stamp);
}

// ── ⑥ 저장 직후 자기검증 (--verify) ─────────────────────────────────────────

#[test]
fn verify_reports_identical_ir_after_save() {
    let src = sample_arg();
    let stamp = temp_path("verify", "png");
    write_stamp_png(&stamp, 40, 20);
    let out = temp_path("verify-out", "hwp");

    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "--x",
        "10000",
        "--y",
        "10000",
        "-o",
        out.to_str().unwrap(),
        "--verify",
        "--json",
    ];
    let output = run(&args);
    let v = parse_json(&args, &output);
    // 판정은 데이터다 — verify 봉투가 반드시 있어야 하고, 차이가 있으면 exit 3.
    assert!(v["verify"].is_object(), "--verify 봉투 누락: {v}");
    assert_eq!(
        output.status.code(),
        Some(0),
        "저장본 재파싱 IR 차이: {}",
        describe(&args, &output)
    );

    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&stamp);
}

// ── ⑦ 입력 형식 보존 (HWPX 입력 → HWPX 산출) ────────────────────────────────

#[test]
fn hwpx_input_keeps_hwpx_output_format() {
    const HWPX_SAMPLE: &str = "samples/2025년 기부·답례품 실적 지자체 보고서_양식.hwpx";
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join(HWPX_SAMPLE);
    if !src.exists() {
        eprintln!("HWPX 샘플 없음 — 건너뜀");
        return;
    }
    let stamp = temp_path("hwpx", "png");
    write_stamp_png(&stamp, 40, 20);
    let out = temp_path("hwpx-out", "hwpx");

    let args = [
        "edit",
        "insert-image",
        src.to_str().unwrap(),
        "--image",
        stamp.to_str().unwrap(),
        "--x",
        "20000",
        "--y",
        "20000",
        "--width",
        "5000",
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v = parse_json(&args, &output);
    assert_eq!(v["outputFormat"], "hwpx", "{v}");
    assert!(out.exists(), "{}", describe(&args, &output));

    let pictures = pictures_of(&out);
    assert!(
        pictures.iter().any(|p| p.1 == 5000 && p.3 == 20000),
        "HWPX 산출물에 그림이 없습니다: {pictures:?}"
    );

    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&stamp);
}

// ── ⑧ 형식별 수용 (png·jpg·jpeg·bmp·tif·tiff) ──────────────────────────────

#[test]
fn every_declared_format_is_actually_accepted() {
    // 형식 목록은 도움말·오류 문구가 광고하는 계약이다. 선언만 하고 실제로는 거부하면
    // 에이전트는 "지원한다더니 안 된다" 는 막다른 길에 선다 — 확장자마다 실제로 넣어 본다.
    let src = sample_arg();
    let cases: [(&str, image::ImageFormat); 6] = [
        ("png", image::ImageFormat::Png),
        ("jpg", image::ImageFormat::Jpeg),
        ("jpeg", image::ImageFormat::Jpeg),
        ("bmp", image::ImageFormat::Bmp),
        ("tif", image::ImageFormat::Tiff),
        ("tiff", image::ImageFormat::Tiff),
    ];

    for (ext, format) in cases {
        let stamp = temp_path(&format!("fmt-{ext}"), ext);
        write_stamp_as(&stamp, format, 40, 20);
        let out = temp_path(&format!("fmt-{ext}-out"), "hwp");

        let args = [
            "edit",
            "insert-image",
            src.as_str(),
            "--image",
            stamp.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--json",
        ];
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{ext} 를 받아들이지 못했습니다.\n{}",
            describe(&args, &output)
        );
        let v = parse_json(&args, &output);
        // 원본 픽셀을 형식과 무관하게 같은 규약으로 읽어야 한다(40×20 → ×75).
        assert_eq!(v["width"], 40 * HWPUNIT_PER_PX, "{ext}: {v}");
        assert_eq!(v["height"], 20 * HWPUNIT_PER_PX, "{ext}: {v}");
        assert!(v["binDataId"].as_u64().unwrap_or(0) > 0, "{ext}: {v}");
        assert!(out.exists(), "{ext}: 산출물 없음");

        let _ = std::fs::remove_file(&out);
        let _ = std::fs::remove_file(&stamp);
    }
}

// ── ⑨ 쪽 지정(앵커) ─────────────────────────────────────────────────────────

#[test]
fn page_argument_anchors_the_picture_on_that_page() {
    // `--page N` 은 "N쪽에 놓아라" 는 뜻이고, 그 판정은 조판이 한다. 앵커 문단을
    // 잘못 고르면 도장이 다른 쪽에 찍히는데 종료 코드는 0 이다 — changedPages 가
    // 요청한 쪽을 담는지로 확인한다(쪽 번호를 박지 않고 문서에게 물어 순회).
    let src = sample_arg();
    let stamp = temp_path("anchor", "png");
    write_stamp_png(&stamp, 40, 20);

    let pages = page_count_of(src.as_str());
    assert!(
        pages >= 2,
        "쪽 지정 시험에는 2쪽 이상이 필요합니다: {pages}"
    );

    for page in 0..pages {
        let page_arg = page.to_string();
        let out = temp_path(&format!("anchor-{page}"), "hwp");
        let args = [
            "edit",
            "insert-image",
            src.as_str(),
            "--image",
            stamp.to_str().unwrap(),
            "--page",
            page_arg.as_str(),
            "-o",
            out.to_str().unwrap(),
            "--json",
        ];
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            describe(&args, &output)
        );
        let v = parse_json(&args, &output);
        assert_eq!(v["page"].as_u64(), Some(page), "{v}");
        let changed: Vec<u64> = v["changedPages"]
            .as_array()
            .expect("changedPages")
            .iter()
            .filter_map(|p| p.as_u64())
            .collect();
        assert!(
            changed.contains(&page),
            "{page}쪽을 요청했는데 바뀐 쪽 목록에 없습니다 (앵커 문단 오선택): {changed:?} / {v}"
        );

        let _ = std::fs::remove_file(&out);
    }

    // 마지막 쪽 다음은 인자 오류다 — 쪽 수도 문서에게 물어 유도한다.
    let over = pages.to_string();
    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "--page",
        over.as_str(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));

    let _ = std::fs::remove_file(&stamp);
}

// ── ⑩ 없는 파일 — 런타임 실패(1)이지 인자 오류(2)가 아니다 ──────────────────

#[test]
fn missing_files_are_runtime_errors_with_silent_stdout() {
    // 종료 코드 사전(#2707)의 구분을 지킨다: 인자 형태는 맞는데 그 파일이 없는 것은
    // 런타임 실패(1)다. 형식 미지원(2)과 뒤섞이면 에이전트의 재시도 판단이 무너진다.
    let src = sample_arg();
    let stamp = temp_path("exists", "png");
    write_stamp_png(&stamp, 40, 20);
    let ghost_image = temp_path("ghost", "png");
    let ghost_doc = temp_path("ghost-doc", "hwp");

    // ① 그림이 없다
    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        ghost_image.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));

    // ② 문서가 없다
    let args = [
        "edit",
        "insert-image",
        ghost_doc.to_str().unwrap(),
        "--image",
        stamp.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));

    let _ = std::fs::remove_file(&stamp);
}

#[test]
fn unknown_option_and_duplicate_input_are_usage_errors() {
    let src = sample_arg();
    let stamp = temp_path("parse", "png");
    write_stamp_png(&stamp, 40, 20);

    // 알 수 없는 옵션을 조용히 무시하면 "요청한 대로 됐다" 는 거짓 성공이 된다.
    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "--rotate",
        "90",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));

    // 입력 파일은 하나뿐이다.
    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
    assert!(output.stdout.is_empty(), "{}", describe(&args, &output));

    // 값이 필요한 플래그에 값이 없다.
    for flag in [
        "--image", "--page", "--x", "--y", "--width", "--height", "-o",
    ] {
        let args = ["edit", "insert-image", src.as_str(), flag];
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{flag}: {}",
            describe(&args, &output)
        );
        assert!(
            output.stdout.is_empty(),
            "{flag}: {}",
            describe(&args, &output)
        );
    }

    // 0 크기는 그림이 아니다.
    for flag in ["--width", "--height"] {
        let args = [
            "edit",
            "insert-image",
            src.as_str(),
            "--image",
            stamp.to_str().unwrap(),
            flag,
            "0",
            "--json",
        ];
        let output = run(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{flag}: {}",
            describe(&args, &output)
        );
        assert!(
            output.stdout.is_empty(),
            "{flag}: {}",
            describe(&args, &output)
        );
    }

    let _ = std::fs::remove_file(&stamp);
}

// ── ⑪ 자기서술 계약 (capabilities / MCP) ────────────────────────────────────

#[test]
fn capabilities_and_mcp_declare_insert_image_axis() {
    // 매니페스트만 읽는 에이전트에게는 선언이 곧 기능이다 — 빠지면 영영 못 쓴다.
    let cap: serde_json::Value =
        serde_json::from_slice(&run(&["capabilities"]).stdout).expect("capabilities JSON");
    let edit = cap["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .find(|c| c["name"] == "edit")
        .expect("edit 항목");
    let flags: Vec<&str> = edit["flags"]
        .as_array()
        .expect("edit flags")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    for expected in ["--image", "--page", "--x", "--y", "--width", "--height"] {
        assert!(
            flags.contains(&expected),
            "edit flags 에 {expected} 누락: {flags:?}"
        );
    }

    let mcp: serde_json::Value = serde_json::from_slice(&run(&["capabilities", "--mcp"]).stdout)
        .expect("capabilities --mcp JSON");
    let tool = mcp["tools"]
        .as_array()
        .expect("tools")
        .iter()
        .find(|t| t["name"] == "hwp_insert_image")
        .expect("hwp_insert_image 도구");
    let required: Vec<&str> = tool["inputSchema"]["required"]
        .as_array()
        .expect("required 배열")
        .iter()
        .filter_map(|r| r.as_str())
        .collect();
    assert!(required.contains(&"path"), "{tool}");
    assert!(required.contains(&"image"), "{tool}");
    // 단위 함정은 설명에 박혀 있어야 한다 — 픽셀로 오해하면 도장이 사라진다.
    let description = tool["description"].as_str().unwrap_or_default();
    assert!(description.contains("HWPUNIT"), "{tool}");
    let output_fields: Vec<&str> = tool["outputFields"]
        .as_array()
        .expect("outputFields")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    for expected in ["binDataId", "overflow", "changedPages"] {
        assert!(
            output_fields.contains(&expected),
            "hwp_insert_image 출력 계약에 {expected} 누락: {tool}"
        );
    }

    // 선언한 입력 속성은 전부 자식 CLI 에 닿아야 한다. 닿지 않으면 서버가 그 인자를
    // 조용히 버리고 성공을 보고한다 — 도구 하나 범위에서 같은 규칙을 다시 못 박는다.
    let wired: Vec<String> = tool["cli"]["args"]
        .as_array()
        .expect("cli.args")
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|s| s.starts_with('{') && s.ends_with('}') && s.len() > 2)
        .map(|s| s[1..s.len() - 1].to_string())
        .chain(
            tool["cli"]["optionalArgs"]
                .as_array()
                .expect("cli.optionalArgs")
                .iter()
                .filter_map(|o| o["when"].as_str().map(String::from)),
        )
        .collect();
    let properties = tool["inputSchema"]["properties"]
        .as_object()
        .expect("properties");
    let orphans: Vec<&String> = properties.keys().filter(|k| !wired.contains(k)).collect();
    assert!(
        orphans.is_empty(),
        "선언만 되고 배선되지 않은 입력 인자: {orphans:?}\n{tool}"
    );

    // 선언한 CLI 플래그는 실제로 수용돼야 한다 — 매니페스트가 광고한 축을 그대로 호출해 본다.
    let src = sample_arg();
    let stamp = temp_path("declared", "png");
    write_stamp_png(&stamp, 40, 20);
    let out = temp_path("declared-out", "hwp");
    let args = [
        "edit",
        "insert-image",
        src.as_str(),
        "--image",
        stamp.to_str().unwrap(),
        "--page",
        "0",
        "--x",
        "1000",
        "--y",
        "1000",
        "--width",
        "3000",
        "--height",
        "1500",
        "-o",
        out.to_str().unwrap(),
        "--verify",
        "--json",
    ];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "선언한 플래그 조합이 거부됐습니다.\n{}",
        describe(&args, &output)
    );
    let _ = std::fs::remove_file(&out);
    let _ = std::fs::remove_file(&stamp);

    // `--help` 에도 있어야 한다(사람용/기계용 양방향 현행화).
    let help = String::from_utf8_lossy(&run(&["--help"]).stdout).to_string();
    assert!(help.contains("edit insert-image"), "--help 에 누락");
    assert!(help.contains("HWPUNIT"), "--help 에 단위 규약 누락");
    for flag in ["--image", "--page", "--x", "--y", "--width", "--height"] {
        assert!(help.contains(flag), "--help 에 {flag} 안내 누락");
    }
}
