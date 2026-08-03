//! [#3719 §6-11] `edit redact` · `edit sanitize` 계약 테스트.
//!
//! 이 명령들의 위험은 성격이 다르다:
//!
//! - `redact` 는 **되돌릴 수 없는 쓰기**다. 오탐 하나가 본문 숫자를 영구히 훼손하고,
//!   실수로 원본을 덮어쓰면 되돌릴 방법이 없다. 그래서 여기서 고정하는 것은 기능이
//!   아니라 **안 하는 것**이다 — 검증을 통과하지 못한 문자열은 마스킹하지 않고,
//!   산출 경로를 명시하지 않으면 실행하지 않으며, `--dry-run` 은 파일을 만들지 않는다.
//! - `sanitize` 는 **본문을 건드리면 안 된다**. `export-text` 전후 비교로 못박는다.
//!
//! 개인정보가 든 샘플은 저장소에 두지 않는다. 테스트가 `edit fill-fields` 로 형태만
//! 개인정보인 **가짜 값**을 심어 넣고 그 문서를 대상으로 돌린다.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 누름틀이 있는 서식 샘플 — 값을 심을 자리(회사명·작성자·전화번호·이메일)가 있다.
const SAMPLE: &str = "samples/field-01.hwp";

/// 검증 숫자(mod 11)를 통과하는 **가공** 주민등록번호. 실재 인물과 무관하다.
const VALID_SSN: &str = "900101-1234568";
/// 형태만 같고 검증 숫자가 틀린 문자열 — 마스킹되면 안 된다.
const INVALID_SSN: &str = "900101-1234567";
/// Luhn 을 통과하는 시험용 카드번호(공개 테스트 번호).
const VALID_CARD: &str = "4111-1111-1111-1111";
/// Luhn 실패 숫자열 — 마스킹되면 안 된다.
const INVALID_CARD: &str = "1234-5678-9012-3456";

fn repo(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rhwp-redact-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("작업 디렉터리 생성");
    dir.join(name)
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패")
}

fn describe(args: &[&str], out: &Output) -> String {
    format!(
        "args={args:?}\nexit={:?}\nstdout={}\nstderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn json_of(args: &[&str], out: &Output) -> serde_json::Value {
    serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|e| panic!("stdout 이 JSON 이 아닙니다({e}): {}", describe(args, out)))
}

/// 형태만 개인정보인 가짜 값을 누름틀에 심은 문서를 만든다.
///
/// 유효한 값과 **검증에 실패하는 미끼**를 같은 문서에 함께 넣는다 — 오탐 0을
/// 증명하려면 미끼가 같은 문서 안에 있어야 한다.
fn make_pii_document(name: &str) -> Option<PathBuf> {
    let src = repo(SAMPLE);
    if !src.exists() {
        eprintln!("샘플 없음({}) — 건너뜀", src.display());
        return None;
    }
    let out = scratch(name);
    let _ = std::fs::remove_file(&out);
    let data = format!(
        r#"{{"작성자":"홍길동 {VALID_SSN}","전화번호":"010-1234-5678","이메일":"hong@example.com","회사명":"카드 {VALID_CARD} / 미끼 {INVALID_SSN} / 미끼 {INVALID_CARD}"}}"#
    );
    let args = [
        "edit",
        "fill-fields",
        src.to_str().unwrap(),
        "--data",
        &data,
        "-o",
        out.to_str().unwrap(),
        "--json",
    ];
    let result = run(&args);
    assert_eq!(
        result.status.code(),
        Some(0),
        "{}",
        describe(&args, &result)
    );
    assert!(out.exists(), "가짜 개인정보 문서가 만들어지지 않았습니다");
    Some(out)
}

/// 문서 전체 텍스트 — `export-text --json` 의 페이지 배열을 이어 붙인다.
fn document_text(path: &Path) -> String {
    let args = ["export-text", path.to_str().unwrap(), "--json"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    v["pages"]
        .as_array()
        .expect("pages 배열")
        .iter()
        .map(|p| p["text"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>()
        .join("\n")
}

/// **오탐 0** — 형태가 같아도 검증(주민번호 mod 11 · 카드 Luhn)을 통과하지 못하면
/// 탐지 목록에 없어야 하고, 실제 마스킹 후에도 원문 그대로 남아 있어야 한다.
///
/// 이 계약이 깨지면 redact 는 문서를 훼손하는 도구가 된다 — 마스킹은 되돌릴 수 없다.
#[test]
fn checksum_failures_are_never_masked() {
    let Some(doc) = make_pii_document("fp.hwp") else {
        return;
    };
    let args = [
        "edit",
        "redact",
        doc.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);

    let raws: Vec<&str> = v["findings"]
        .as_array()
        .expect("findings 배열")
        .iter()
        .filter_map(|f| f["raw"].as_str())
        .collect();
    for decoy in [INVALID_SSN, INVALID_CARD] {
        assert!(
            !raws.contains(&decoy),
            "검증에 실패하는 미끼를 탐지했습니다({decoy}) — 오탐은 본문을 훼손합니다: {raws:?}"
        );
    }
    // 규칙이 통째로 죽어 있으면 위 단언이 공허하게 통과한다 — 진짜 값은 잡아야 한다.
    for real in [VALID_SSN, VALID_CARD, "010-1234-5678", "hong@example.com"] {
        assert!(
            raws.contains(&real),
            "탐지되어야 할 값을 놓쳤습니다({real}): {raws:?}"
        );
    }

    // 실제 마스킹 후에도 미끼는 그대로 있어야 한다.
    let masked = scratch("fp_masked.hwp");
    let _ = std::fs::remove_file(&masked);
    let args = [
        "edit",
        "redact",
        doc.to_str().unwrap(),
        "-o",
        masked.to_str().unwrap(),
        "--json",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let text = document_text(&masked);
    for decoy in [INVALID_SSN, INVALID_CARD] {
        assert!(
            text.contains(decoy),
            "미끼가 마스킹으로 훼손됐습니다({decoy})"
        );
    }
    for real in [VALID_SSN, VALID_CARD, "010-1234-5678", "hong@example.com"] {
        assert!(!text.contains(real), "마스킹되지 않고 남았습니다({real})");
    }
}

/// 마스킹 후에도 **문자 수가 유지**된다 — 길이가 바뀌면 조판이 흔들린다.
#[test]
fn masking_preserves_length() {
    let Some(doc) = make_pii_document("len.hwp") else {
        return;
    };
    let args = [
        "edit",
        "redact",
        doc.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let out = run(&args);
    let v = json_of(&args, &out);
    let findings = v["findings"].as_array().expect("findings");
    assert!(!findings.is_empty(), "탐지 0건이면 이 가드는 공허하다: {v}");
    for f in findings {
        let raw = f["raw"].as_str().expect("raw");
        let masked = f["masked"].as_str().expect("masked");
        assert_eq!(
            raw.chars().count(),
            masked.chars().count(),
            "마스킹으로 길이가 바뀌었습니다: {raw} → {masked}"
        );
        assert!(
            !masked.chars().any(char::is_alphanumeric),
            "마스킹 후에도 영숫자가 남았습니다: {masked}"
        );
    }

    // 산출 문서의 실제 본문에서도 자릿수가 유지되어야 한다.
    let masked_path = scratch("len_masked.hwp");
    let _ = std::fs::remove_file(&masked_path);
    let args = [
        "edit",
        "redact",
        doc.to_str().unwrap(),
        "-o",
        masked_path.to_str().unwrap(),
        "--json",
    ];
    assert_eq!(run(&args).status.code(), Some(0));
    let text = document_text(&masked_path);
    assert!(
        text.contains("******-*******"),
        "주민등록번호 자리가 원래 길이로 마스킹되지 않았습니다"
    );
    assert!(
        text.contains("****-****-****-****"),
        "카드번호 자리가 원래 길이로 마스킹되지 않았습니다"
    );
}

/// 원본 보호 — `-o` 도 `--in-place` 도 없으면 **실행하지 않는다**(exit 2).
///
/// 다른 edit 명령처럼 `_redacted.hwp` 같은 기본 이름을 만들지도 않는다: 되돌릴 수
/// 없는 작업에서 "어디에 무엇이 생겼는지 모르는 상태"를 만들지 않기 위해서다.
#[test]
fn refuses_to_run_without_an_explicit_destination() {
    let Some(doc) = make_pii_document("guard.hwp") else {
        return;
    };
    let before = std::fs::read(&doc).expect("원본 읽기");

    let args = ["edit", "redact", doc.to_str().unwrap(), "--json"];
    let out = run(&args);
    assert_eq!(
        out.status.code(),
        Some(2),
        "산출 경로 없는 실행은 exit 2 여야 합니다: {}",
        describe(&args, &out)
    );
    assert!(
        out.stdout.is_empty(),
        "실패 경로는 stdout 0바이트여야 합니다: {}",
        describe(&args, &out)
    );
    assert_eq!(
        std::fs::read(&doc).expect("원본 재읽기"),
        before,
        "거부됐는데 원본이 바뀌었습니다"
    );

    // `-o` 로 원본 자신을 지목하는 것도 같은 사고다.
    let args = [
        "edit",
        "redact",
        doc.to_str().unwrap(),
        "-o",
        doc.to_str().unwrap(),
        "--json",
    ];
    let out = run(&args);
    assert_eq!(
        out.status.code(),
        Some(2),
        "-o 가 원본을 가리키면 exit 2 여야 합니다: {}",
        describe(&args, &out)
    );
    assert_eq!(
        std::fs::read(&doc).expect("원본 재읽기"),
        before,
        "-o 원본 지목이 거부됐는데 원본이 바뀌었습니다"
    );

    // --in-place 를 명시하면 허용된다 — 원본이 실제로 바뀐다.
    let args = [
        "edit",
        "redact",
        doc.to_str().unwrap(),
        "--in-place",
        "--json",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    assert_ne!(
        std::fs::read(&doc).expect("원본 재읽기"),
        before,
        "--in-place 인데 원본이 그대로입니다"
    );
}

/// `--dry-run` 은 **산출 파일을 만들지 않는다**.
#[test]
fn dry_run_writes_nothing() {
    let Some(doc) = make_pii_document("dry.hwp") else {
        return;
    };
    let out_path = scratch("dry_out.hwp");
    let _ = std::fs::remove_file(&out_path);
    let before = std::fs::read(&doc).expect("원본 읽기");

    let args = [
        "edit",
        "redact",
        doc.to_str().unwrap(),
        "-o",
        out_path.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert_eq!(v["dryRun"], true, "{v}");
    assert_eq!(v["redactedCount"], 0, "dry-run 은 치환하지 않습니다: {v}");
    assert!(
        v["output"].is_null(),
        "dry-run 봉투에 output 이 있습니다: {v}"
    );
    assert!(
        !out_path.exists(),
        "--dry-run 인데 산출 파일이 생겼습니다: {}",
        out_path.display()
    );
    assert_eq!(
        std::fs::read(&doc).expect("원본 재읽기"),
        before,
        "--dry-run 인데 원본이 바뀌었습니다"
    );
}

/// `--mask` 는 자릿수를 유지하는 한 글자여야 한다 — 두 글자·영숫자는 거부한다.
#[test]
fn mask_must_be_a_single_non_alphanumeric_char() {
    let Some(doc) = make_pii_document("mask.hwp") else {
        return;
    };
    for bad in ["**", "x"] {
        let args = [
            "edit",
            "redact",
            doc.to_str().unwrap(),
            "--mask",
            bad,
            "--dry-run",
            "--json",
        ];
        let out = run(&args);
        assert_eq!(
            out.status.code(),
            Some(2),
            "--mask {bad} 는 거부되어야 합니다: {}",
            describe(&args, &out)
        );
        assert!(out.stdout.is_empty(), "{}", describe(&args, &out));
    }

    let args = [
        "edit",
        "redact",
        doc.to_str().unwrap(),
        "--mask",
        "#",
        "--dry-run",
        "--json",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert_eq!(v["mask"], "#", "{v}");
    assert!(
        v["findings"]
            .as_array()
            .expect("findings")
            .iter()
            .all(|f| f["masked"].as_str().is_some_and(|m| m.contains('#'))),
        "지정한 마스킹 문자가 쓰이지 않았습니다: {v}"
    );
}

/// `--kind` 로 종류를 좁히면 그 종류만 탐지된다.
#[test]
fn kind_filter_narrows_detection() {
    let Some(doc) = make_pii_document("kind.hwp") else {
        return;
    };
    let args = [
        "edit",
        "redact",
        doc.to_str().unwrap(),
        "--kind",
        "email",
        "--dry-run",
        "--json",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    let findings = v["findings"].as_array().expect("findings");
    assert!(!findings.is_empty(), "이메일을 놓쳤습니다: {v}");
    assert!(
        findings.iter().all(|f| f["kind"] == "email"),
        "--kind email 인데 다른 종류가 섞였습니다: {v}"
    );
}

/// **기본값 무회귀** — `--no-raw` 없이는 지금까지처럼 `findings[].raw` 가 그대로 나온다.
#[test]
fn default_still_includes_raw() {
    let Some(doc) = make_pii_document("default_raw.hwp") else {
        return;
    };
    let args = [
        "edit",
        "redact",
        doc.to_str().unwrap(),
        "--dry-run",
        "--json",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert_eq!(v["noRaw"], false, "{v}");
    let findings = v["findings"].as_array().expect("findings");
    assert!(!findings.is_empty(), "탐지 0건이면 이 가드는 공허하다: {v}");
    for f in findings {
        assert!(
            f.get("raw").and_then(|r| r.as_str()).is_some(),
            "기본값에서 raw 가 사라졌습니다(기존 계약 위반): {f}"
        );
    }
}

/// `--no-raw` — `findings[].raw` 필드 자체가 **생략**된다(`null` 이 아니다). 위치·종류
/// 정보(kind/masked/section/paragraph/page/charOffset)는 그대로 남아야 위치 검토가 된다.
#[test]
fn no_raw_omits_the_field_without_changing_other_fields() {
    let Some(doc) = make_pii_document("no_raw.hwp") else {
        return;
    };
    let args = [
        "edit",
        "redact",
        doc.to_str().unwrap(),
        "--dry-run",
        "--no-raw",
        "--json",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert_eq!(v["noRaw"], true, "{v}");
    let findings = v["findings"].as_array().expect("findings");
    assert!(!findings.is_empty(), "탐지 0건이면 이 가드는 공허하다: {v}");
    for f in findings {
        let obj = f.as_object().expect("finding 은 object");
        assert!(
            !obj.contains_key("raw"),
            "--no-raw 인데 raw 필드가 남아 있습니다(생략이 아니라 null 이거나 그대로임): {f}"
        );
        for field in ["kind", "masked", "section", "paragraph", "charOffset"] {
            assert!(
                obj.contains_key(field),
                "--no-raw 가 다른 필드까지 지웠습니다({field}): {f}"
            );
        }
    }

    // 사람용 출력(비-json)도 원문을 감춰야 한다 — 콘솔 로그도 유출 경로다.
    let args = [
        "edit",
        "redact",
        doc.to_str().unwrap(),
        "--dry-run",
        "--no-raw",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let stdout = String::from_utf8_lossy(&out.stdout);
    for real in [VALID_SSN, VALID_CARD, "010-1234-5678", "hong@example.com"] {
        assert!(
            !stdout.contains(real),
            "--no-raw 인데 사람용 출력에 원문이 남았습니다({real}): {stdout}"
        );
    }
}

/// `sanitize` 는 메타데이터만 지우고 **본문 텍스트를 바꾸지 않는다**.
#[test]
fn sanitize_removes_metadata_without_touching_the_body() {
    let src = repo(SAMPLE);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out_path = scratch("san.hwp");
    let _ = std::fs::remove_file(&out_path);

    let before_text = document_text(&src);
    let args = [
        "edit",
        "sanitize",
        src.to_str().unwrap(),
        "-o",
        out_path.to_str().unwrap(),
        "--json",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert_eq!(v["schemaVersion"], "1.0", "{v}");

    let removed = v["removed"].as_array().expect("removed 배열");
    assert!(
        !removed.is_empty(),
        "지운 것이 없으면 이 계약은 공허하다 — 샘플에 메타가 있어야 한다: {v}"
    );
    assert_eq!(v["removedCount"], removed.len(), "{v}");
    for entry in removed {
        assert!(entry["field"].is_string(), "field 누락: {entry}");
        assert!(entry["before"].is_string(), "before 누락: {entry}");
    }
    let fields: Vec<&str> = removed.iter().filter_map(|e| e["field"].as_str()).collect();
    for expected in ["author", "lastSavedBy"] {
        assert!(
            fields.contains(&expected),
            "작성자 계열 메타를 지우지 않았습니다({expected}): {fields:?}"
        );
    }

    assert_eq!(
        document_text(&out_path),
        before_text,
        "sanitize 가 본문 텍스트를 바꿨습니다"
    );

    // 두 번째 실행은 지울 것이 없어야 한다 — 첫 실행이 실제로 지웠다는 증거다.
    let again = scratch("san2.hwp");
    let _ = std::fs::remove_file(&again);
    let args = [
        "edit",
        "sanitize",
        out_path.to_str().unwrap(),
        "-o",
        again.to_str().unwrap(),
        "--json",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert_eq!(
        v["removedCount"], 0,
        "이미 정리된 문서에서 또 지울 것이 나왔습니다 — 첫 실행이 지우지 못한 것입니다: {v}"
    );
}

/// `--keep-preview` 는 미리보기 이미지를 남긴다.
#[test]
fn keep_preview_retains_the_thumbnail() {
    let src = repo(SAMPLE);
    if !src.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return;
    }
    let out_path = scratch("san_keep.hwp");
    let _ = std::fs::remove_file(&out_path);
    let args = [
        "edit",
        "sanitize",
        src.to_str().unwrap(),
        "--keep-preview",
        "-o",
        out_path.to_str().unwrap(),
        "--json",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert_eq!(v["keepPreview"], true, "{v}");
    assert!(
        !v["removed"]
            .as_array()
            .expect("removed")
            .iter()
            .any(|e| e["field"] == "preview.image"),
        "--keep-preview 인데 미리보기 이미지를 지웠습니다: {v}"
    );

    // 썸네일 추출이 여전히 성공해야 한다 — 봉투 선언만이 아니라 실물로 확인한다.
    let args = [
        "thumbnail",
        out_path.to_str().unwrap(),
        "--base64",
        "--json",
    ];
    let out = run(&args);
    assert_eq!(
        out.status.code(),
        Some(0),
        "--keep-preview 산출물에서 썸네일이 사라졌습니다: {}",
        describe(&args, &out)
    );
}

/// 산출물은 입력 형식을 보존한다 (HWPX 입력 → HWPX 산출).
#[test]
fn sanitize_preserves_the_input_format() {
    let src = repo("samples/3-09월_교육_통합_2022.hwpx");
    if !src.exists() {
        eprintln!("HWPX 샘플 없음 — 건너뜀");
        return;
    }
    let out_path = scratch("san_fmt.hwpx");
    let _ = std::fs::remove_file(&out_path);
    let args = [
        "edit",
        "sanitize",
        src.to_str().unwrap(),
        "-o",
        out_path.to_str().unwrap(),
        "--json",
    ];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert_eq!(v["outputFormat"], "hwpx", "{v}");
    assert_eq!(
        document_text(&out_path),
        document_text(&src),
        "HWPX sanitize 가 본문을 바꿨습니다"
    );
}

/// 없는 파일·알 수 없는 옵션은 stdout 을 오염시키지 않는다 (계약 봉투 규약).
///
/// 봉투 계약의 소비자는 `stdout` 을 통째로 `JSON.parse` 한다 — 실패 경로가 한 글자라도
/// 흘리면 파서가 죽거나(운이 좋으면) 잘못 읽는다(운이 나쁘면).
#[test]
fn failure_paths_keep_stdout_empty() {
    let sample_path = repo(SAMPLE);
    let sample = sample_path.to_str().expect("샘플 경로");
    let cases: [(Vec<&str>, i32); 6] = [
        (
            vec!["edit", "redact", "없는파일.hwp", "-o", "x.hwp", "--json"],
            1,
        ),
        (vec!["edit", "sanitize", "없는파일.hwp", "--json"], 1),
        (
            vec!["edit", "redact", sample, "--kind", "없는종류", "--json"],
            2,
        ),
        (vec!["edit", "sanitize", sample, "--없는옵션"], 2),
        // -o 와 --in-place 동시 지정은 산출 경로가 모호하다.
        (
            vec!["edit", "redact", sample, "-o", "x.hwp", "--in-place"],
            2,
        ),
        // 입력 파일 두 개는 어느 쪽을 지울지 알 수 없다.
        (vec!["edit", "redact", sample, sample, "--in-place"], 2),
    ];
    for (args, want) in cases {
        let out = run(&args);
        assert_eq!(out.status.code(), Some(want), "{}", describe(&args, &out));
        assert!(
            out.stdout.is_empty(),
            "실패 경로 stdout 오염: {}",
            describe(&args, &out)
        );
    }
}

/// 자기서술 — capabilities/MCP 에 두 명령이 배선되어 있어야 에이전트가 쓸 수 있다.
#[test]
fn capabilities_and_mcp_declare_both_commands() {
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
        .expect("flags")
        .iter()
        .filter_map(|f| f.as_str())
        .collect();
    for flag in ["--kind", "--mask", "--in-place", "--keep-preview"] {
        assert!(
            flags.contains(&flag),
            "commands[edit].flags 에 {flag} 누락: {flags:?}"
        );
    }

    let mcp: serde_json::Value =
        serde_json::from_slice(&run(&["capabilities", "--mcp"]).stdout).expect("mcp JSON");
    let tools = mcp["tools"].as_array().expect("tools");
    for name in ["hwp_redact", "hwp_sanitize"] {
        let tool = tools
            .iter()
            .find(|t| t["name"] == name)
            .unwrap_or_else(|| panic!("{name} 도구 누락"));
        assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
        assert!(tool["inputSchema"]["required"].is_array(), "{tool}");
        assert_eq!(tool["cli"]["command"], "edit", "{tool}");
    }

    // 사람이 보는 --help 에도 두 하위 명령이 있어야 한다(기계·사람 자기서술 동기).
    let help = String::from_utf8_lossy(&run(&["--help"]).stdout).to_string();
    for line in ["edit redact", "edit sanitize"] {
        assert!(help.contains(line), "--help 에 '{line}' 이 없습니다");
    }
}

/// [#3885] 봉투 출처 표지 — `findings[].raw` 는 문서에서 그대로 나온 개인정보라
/// untrusted 로 표지돼야 하고, `--no-raw` 면 그 경로가 봉투에 없으니 표지에서도
/// 빠져야 한다(실재 경로 필터). 표지가 없으면 가장 민감한 값을 실은 봉투가
/// "출처 판정 안 함"으로 나간다 — S1 계약이 정확히 그 지점에서 무너진다.
#[test]
fn redact_envelope_marks_document_values_as_untrusted() {
    let Some(doc) = make_pii_document("provmark.hwp") else {
        return;
    };
    let p = doc.to_str().expect("경로 UTF-8");

    let args = ["edit", "redact", p, "--dry-run", "--json"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert_eq!(
        v["untrustedContent"],
        serde_json::Value::Bool(true),
        "raw 원문을 싣는 봉투인데 untrustedContent 가 true 가 아닙니다: {v}"
    );
    let fields: Vec<&str> = v["untrustedFields"]
        .as_array()
        .expect("untrustedFields")
        .iter()
        .filter_map(|x| x.as_str())
        .collect();
    assert!(fields.contains(&"findings[].raw"), "{v}");
    assert!(fields.contains(&"findings[].masked"), "{v}");

    let args = ["edit", "redact", p, "--dry-run", "--json", "--no-raw"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert_eq!(v["untrustedContent"], serde_json::Value::Bool(true), "{v}");
    let fields: Vec<&str> = v["untrustedFields"]
        .as_array()
        .expect("untrustedFields")
        .iter()
        .filter_map(|x| x.as_str())
        .collect();
    assert!(
        !fields.contains(&"findings[].raw"),
        "--no-raw 로 raw 가 봉투에 없는데 표지가 그 경로를 주장합니다: {v}"
    );
    assert!(fields.contains(&"findings[].masked"), "{v}");
}

/// [#3885] sanitize 봉투 — `removed[].before` 는 지워진 문서 속성 원문(제목·작성자,
/// 그리고 본문 첫 화면 발췌인 preview.text)이라 untrusted 로 표지돼야 한다.
#[test]
fn sanitize_envelope_marks_removed_before_as_untrusted() {
    let Some(doc) = make_pii_document("provmark-san.hwp") else {
        return;
    };
    let p = doc.to_str().expect("경로 UTF-8");
    let outp = scratch("provmark-sanitized.hwp");
    let o = outp.to_str().expect("경로 UTF-8");

    let args = ["edit", "sanitize", p, "-o", o, "--json"];
    let out = run(&args);
    assert_eq!(out.status.code(), Some(0), "{}", describe(&args, &out));
    let v = json_of(&args, &out);
    assert!(
        v["removedCount"].as_u64().unwrap_or(0) > 0,
        "제거된 속성이 없으면 이 검사는 공허합니다: {v}"
    );
    assert_eq!(v["untrustedContent"], serde_json::Value::Bool(true), "{v}");
    let fields: Vec<&str> = v["untrustedFields"]
        .as_array()
        .expect("untrustedFields")
        .iter()
        .filter_map(|x| x.as_str())
        .collect();
    assert!(fields.contains(&"removed[].before"), "{v}");
}
