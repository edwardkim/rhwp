//! [#3281] `fields` 출력 계약 회귀 테스트 (읽기 전용 누름틀 조사).
//!
//! 계약: `--json` 의 stdout 은 순수 JSON 한 덩어리이고 `schemaVersion` 을 포함한다.
//! 핵심 가치는 에이전트가 **서식이 무엇을 요구하는지** 읽는 것 — 이름·안내문·지시문·
//! 편집 가능 여부·위치. 종료 코드는 #2707 계약(0/1/2)을 따른다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 누름틀 11개(본문 + 글상자 + 표 셀 혼합).
const SAMPLE_FIELDS: &str = "samples/field-01.hwp";
/// 위와 같은 필드 구성에 HelpState 지시문(memo)이 붙어 있는 문서.
const SAMPLE_MEMO: &str = "samples/field-01-memo.hwp";
/// 누름틀이 없는 일반 문서.
const SAMPLE_NONE: &str = "samples/hwp3-sample.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn run(args: &[&str]) -> Output {
    Command::new(rhwp_bin())
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

fn run_json(rel: &str) -> serde_json::Value {
    let p = sample(rel);
    let args = ["fields", p.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 순수 JSON 이 아닙니다 ({e}).\n{}",
            describe(&args, &output)
        )
    })
}

#[test]
fn fields_json_envelope_contract() {
    let v = run_json(SAMPLE_FIELDS);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");
    assert!(v["source"].is_string(), "{v}");
    let count = v["fieldCount"].as_u64().expect("fieldCount");
    let fields = v["fields"].as_array().expect("fields 배열");
    assert_eq!(fields.len() as u64, count, "fieldCount 는 길이와 같다: {v}");
    assert!(count >= 1, "누름틀이 있는 문서인데 0건입니다: {v}");

    let f = &fields[0];
    assert!(f["fieldId"].as_u64().is_some(), "{f}");
    assert!(f["fieldType"].is_string(), "{f}");
    assert!(f["name"].is_string(), "{f}");
    assert!(f["value"].is_string(), "{f}");
    assert!(f["editableInForm"].is_boolean(), "{f}");
    // 위치는 인용·후속 편집의 좌표다.
    assert!(f["location"]["section"].as_u64().is_some(), "{f}");
    assert!(f["location"]["paragraph"].as_u64().is_some(), "{f}");
}

#[test]
fn fields_exposes_names_for_form_filling() {
    // 에이전트가 "무엇을 채워야 하는가"를 알려면 이름이 실제로 나와야 한다.
    let v = run_json(SAMPLE_FIELDS);
    let named: Vec<&str> = v["fields"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["name"].as_str())
        .filter(|s| !s.is_empty())
        .collect();
    assert!(
        named.len() >= 3,
        "이름 있는 필드가 여럿이어야 합니다: {named:?}"
    );
}

#[test]
fn fields_extracts_guide_or_memo_instructions() {
    // 지시문(HelpState memo)은 "이 칸에 무엇을 어떻게 쓰라"는 사람용 안내다.
    // 에이전트에게는 가장 값진 신호이므로 반드시 나와야 한다.
    let v = run_json(SAMPLE_MEMO);
    let has_instruction = v["fields"].as_array().unwrap().iter().any(|f| {
        let g = f["guide"].as_str().unwrap_or("");
        let m = f["memo"].as_str().unwrap_or("");
        !g.is_empty() || !m.is_empty()
    });
    assert!(
        has_instruction,
        "지시문(guide/memo)이 추출되어야 합니다: {v}"
    );
}

#[test]
fn fields_reports_nested_location_for_table_or_textbox() {
    // 표 셀·글상자 안의 필드는 중첩 경로가 있어야 후속 편집이 좌표를 찾는다.
    let v = run_json(SAMPLE_FIELDS);
    let all_have_nested_key = v["fields"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["location"]["nested"].is_array());
    assert!(
        all_have_nested_key,
        "location.nested 는 항상 배열입니다: {v}"
    );
}

#[test]
fn fields_document_without_fields_is_empty_not_error() {
    // 필드 없는 문서는 오류가 아니라 빈 목록이다 — 파이프라인이 멈추면 안 된다.
    let v = run_json(SAMPLE_NONE);
    assert_eq!(v["fieldCount"], 0, "{v}");
    assert_eq!(v["fields"].as_array().unwrap().len(), 0, "{v}");
}

#[test]
fn fields_default_output_is_human_summary() {
    let p = sample(SAMPLE_FIELDS);
    let args = ["fields", p.to_str().unwrap()];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        serde_json::from_str::<serde_json::Value>(&stdout).is_err(),
        "기본 출력은 JSON 이 아니어야 합니다(--json 전용).\n{}",
        describe(&args, &output)
    );
}

#[test]
fn fields_missing_file_exit_runtime_silent_stdout() {
    let args = ["fields", "없는파일-fields.hwp", "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        describe(&args, &output)
    );
    assert!(
        output.stdout.is_empty(),
        "실패 경로 stdout 은 비어야 합니다.\n{}",
        describe(&args, &output)
    );
}

#[test]
fn fields_usage_error_exit_two() {
    let args = ["fields"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(2),
        "{}",
        describe(&args, &output)
    );
}

/// [#3289] 아카이브 실행 시 컴파일타임 경로는 빌드 러너 전용이므로,
/// nextest가 런타임에 재매핑해 주입하는 CARGO_BIN_EXE_rhwp를 우선한다.
fn rhwp_bin() -> String {
    std::env::var("CARGO_BIN_EXE_rhwp").unwrap_or_else(|_| env!("CARGO_BIN_EXE_rhwp").to_string())
}

// ── [#3707] 유니코드 기만 판정 (textSecurity) ──────────────────────────────
//
// 봉투에 담기는 누름틀 이름은 공격자가 내용을 정할 수 있는 문서에서 온다. 에이전트는
// 그 이름으로 채울 칸을 지목하므로, 화면상 같지만 바이트가 다른 이름 쌍이 있으면
// 엉뚱한 칸을 채우고도 filledCount 는 성공을 보고한다. 아래는 그 판정의 계약이다.

/// 표본 문서의 누름틀 이름 두 개를 원하는 문자열로 바꾼 HWPX 를 임시로 만든다.
/// (공격 문서를 저장소에 두지 않고 시험 시점에 합성한다 — 실물 악성 파일을
///  리포지터리에 커밋하지 않기 위해서다.)
fn hwpx_with_field_names(tag: &str, first: &str, second: &str) -> PathBuf {
    let src = sample(SAMPLE_FIELDS);
    let hwpx = std::env::temp_dir().join(format!(
        "rhwp-textsec-src-{tag}-{}-{}.hwpx",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ));
    let out = run(&["export-hwpx", src.to_str().unwrap(), hwpx.to_str().unwrap()]);
    assert_eq!(out.status.code(), Some(0), "사전 HWPX 변환 실패");

    // section0.xml 의 name= 속성 두 개만 교체하고 다시 압축한다.
    let bytes = std::fs::read(&hwpx).expect("hwpx 읽기");
    let mut zin = zip::ZipArchive::new(std::io::Cursor::new(bytes)).expect("zip 열기");
    let patched = std::env::temp_dir().join(format!(
        "rhwp-textsec-{tag}-{}.hwpx",
        hwpx.file_stem().unwrap().to_string_lossy()
    ));
    let mut zout = zip::ZipWriter::new(std::fs::File::create(&patched).expect("출력 zip"));
    for i in 0..zin.len() {
        let mut e = zin.by_index(i).expect("zip 항목");
        let name = e.name().to_string();
        let mut buf = Vec::new();
        std::io::copy(&mut e, &mut buf).expect("항목 읽기");
        if name == "Contents/section0.xml" {
            let s = String::from_utf8_lossy(&buf)
                .replace("name=\"회사명\"", &format!("name=\"{first}\""))
                .replace("name=\"작성자\"", &format!("name=\"{second}\""));
            buf = s.into_bytes();
        }
        zout.start_file(
            name,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Deflated),
        )
        .expect("항목 쓰기 시작");
        std::io::Write::write_all(&mut zout, &buf).expect("항목 쓰기");
    }
    zout.finish().expect("zip 마감");
    let _ = std::fs::remove_file(&hwpx);
    patched
}

fn fields_json_of(path: &Path) -> serde_json::Value {
    let args = ["fields", path.to_str().unwrap(), "--json"];
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    serde_json::from_slice(&output.stdout).unwrap_or_else(|e| {
        panic!(
            "stdout 이 JSON 이 아닙니다 ({e}).\n{}",
            describe(&args, &output)
        )
    })
}

/// 평범한 한국어 서식은 조용해야 한다. 이 시험이 깨지면 오탐이 생긴 것이고,
/// 에이전트는 곧 textSecurity 를 무시하게 된다 — 없느니만 못한 상태다.
#[test]
fn text_security_is_clean_on_ordinary_korean_forms() {
    for rel in [SAMPLE_FIELDS, SAMPLE_MEMO, SAMPLE_NONE] {
        let v = run_json(rel);
        assert_eq!(
            v["textSecurity"]["status"], "clean",
            "평범한 문서에서 경고가 나면 안 됩니다 ({rel}): {}",
            v["textSecurity"]
        );
    }
}

/// 키릴 동형자로 만든 쌍둥이 이름 — 화면상 구별 불가.
#[test]
fn text_security_reports_cyrillic_twin_field_names() {
    let doc = hwpx_with_field_names("cyr", "Total", "\u{0422}otal");
    let v = fields_json_of(&doc);
    let ts = &v["textSecurity"];
    assert_eq!(ts["status"], "warning", "{ts}");
    let kinds: Vec<&str> = ts["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .filter_map(|f| f["kind"].as_str())
        .collect();
    assert!(
        kinds.contains(&"confusableFieldName"),
        "쌍둥이 이름 무리를 보고해야 합니다: {ts}"
    );
    assert!(
        kinds.contains(&"mixedScript"),
        "혼합 스크립트 이름도 짚어야 합니다: {ts}"
    );
    let _ = std::fs::remove_file(&doc);
}

/// 한글 조합형/완성형 쌍둥이 — 낯선 글자가 전혀 없어 '수상한 문서'로 보이지도 않는,
/// 한국어 서식에서 가장 현실적인 벡터다.
#[test]
fn text_security_reports_hangul_nfc_nfd_twin_field_names() {
    let nfd = "\u{110E}\u{1169}\u{11BC}\u{110B}\u{1162}\u{11A8}"; // 조합형 '총액'
    let doc = hwpx_with_field_names("nfd", "총액", nfd);
    let v = fields_json_of(&doc);
    let ts = &v["textSecurity"];
    assert_eq!(ts["status"], "warning", "{ts}");
    let group = ts["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .find(|f| f["kind"] == "confusableFieldName")
        .unwrap_or_else(|| panic!("한글 NFC/NFD 쌍둥이를 보고해야 합니다: {ts}"));
    assert_eq!(group["names"].as_array().map(|a| a.len()), Some(2), "{ts}");
    // 혼합 스크립트는 아니다 — 순수 한글이므로 그 축은 조용해야 한다.
    let kinds: Vec<&str> = ts["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["kind"].as_str())
        .collect();
    assert!(!kinds.contains(&"mixedScript"), "순수 한글 오탐: {ts}");
    let _ = std::fs::remove_file(&doc);
}

/// 채우기 판정에도 같은 어휘가 실린다 — 종전에는 ambiguous 가 비어 있는 채로
/// '완벽한 성공'을 보고했다.
#[test]
fn fill_fields_reports_confusable_twin() {
    let doc = hwpx_with_field_names("fill", "Total", "\u{0422}otal");
    let args = [
        "edit",
        "fill-fields",
        doc.to_str().unwrap(),
        "--data",
        "{\"Total\":\"999\"}",
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
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("fill-fields --json");
    assert_eq!(v["filledCount"].as_u64(), Some(1), "{v}");
    // 기존 판정은 여전히 조용하다 — 바이트가 다르므로 개수 판정에 걸리지 않는다.
    assert_eq!(v["ambiguous"].as_array().map(|a| a.len()), Some(0), "{v}");
    // 새 축이 그 공백을 메운다.
    let conf = v["confusable"].as_array().expect("confusable 배열");
    assert_eq!(conf.len(), 1, "쌍둥이를 보고해야 합니다: {v}");
    assert_eq!(conf[0]["name"], "Total", "{v}");
    assert_eq!(
        conf[0]["lookalikes"].as_array().map(|a| a.len()),
        Some(1),
        "{v}"
    );
    let _ = std::fs::remove_file(&doc);
}

/// 평범한 문서의 채우기는 confusable 이 빈 배열이어야 한다(키는 항상 있다 —
/// 소비자가 '검사함·깨끗함'과 '옛 바이너리'를 구별할 수 있어야 한다).
#[test]
fn fill_fields_confusable_is_empty_on_ordinary_form() {
    let p = sample(SAMPLE_FIELDS);
    let args = [
        "edit",
        "fill-fields",
        p.to_str().unwrap(),
        "--data",
        "{\"회사명\":\"주식회사 정상\"}",
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
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
    assert_eq!(v["confusable"].as_array().map(|a| a.len()), Some(0), "{v}");
}
