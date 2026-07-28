//! Issue #3476: `edit fill-fields` 가 같은 이름의 반복 필드 중 첫 번째만 채우고 침묵한다.
//!
//! 실제 제출 서식(규제영향분석서·사업계획서·평가표)은 같은 항목 묶음을 여러 번 요구한다.
//! `samples/80168_regulatory_analysis.hwp` 는 누름틀 1,070개 중 고유 이름이 151개뿐이고,
//! `피규제집단명` 은 규제 대상 집단 14개에 대응해 14번 반복된다.
//!
//! 종전에는 `{"피규제집단명": "..."}` 가 첫 번째만 채우면서 `filledCount:1`·`notFound:[]` 로
//! **요청대로 다 됐다** 처럼 보고했다. 에이전트는 13칸이 빈 문서를 완성본으로 제출한다.
//!
//! 계약 두 가지를 고정한다.
//!   1) 색인 없는 키가 반복 이름을 가리키면 `ambiguous` 로 몇 개 중 몇 개인지 보고
//!   2) `이름[N]`(0 기준)으로 N번째를 지목 — 범위 밖은 `notFound`
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::Command;

const REPEATED: &str = "samples/80168_regulatory_analysis.hwp";
/// 14회 반복되는 필드 이름.
const REPEATED_FIELD: &str = "피규제집단명";
const REPEAT_COUNT: u64 = 14;
/// 이름이 겹치지 않는 서식 — 무회귀 확인용.
const UNIQUE: &str = "samples/field-01.hwp";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn temp_path(tag: &str, ext: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-issue3476-{tag}-{}-{}.{ext}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}

fn run(args: &[&str]) -> (serde_json::Value, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_rhwp"))
        .args(args)
        .output()
        .expect("rhwp 실행 실패");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let last = stdout.lines().last().unwrap_or("{}").to_string();
    let json = serde_json::from_str(&last).unwrap_or_else(|e| {
        panic!(
            "JSON 파싱 실패({e}): {last}\nstderr={}",
            String::from_utf8_lossy(&out.stderr)
        )
    });
    (json, out.status.code().unwrap_or(-1))
}

fn fill(input: &str, data: &str, out: &Path, extra: &[&str]) -> serde_json::Value {
    let data_path = temp_path("data", "json");
    std::fs::write(&data_path, data).expect("data.json 쓰기");
    let data_arg = format!("@{}", data_path.to_str().unwrap());
    let input_path = sample(input);
    let input_arg = input_path.to_str().unwrap().to_string();
    let out_arg = out.to_str().unwrap().to_string();
    let mut args = vec![
        "edit",
        "fill-fields",
        input_arg.as_str(),
        "--data",
        data_arg.as_str(),
        "-o",
        out_arg.as_str(),
        "--json",
    ];
    args.extend_from_slice(extra);
    let (json, _) = run(&args);
    let _ = std::fs::remove_file(&data_path);
    json
}

fn field_values(path: &Path, name: &str) -> Vec<String> {
    let (json, _) = run(&["fields", path.to_str().unwrap(), "--json"]);
    json["fields"]
        .as_array()
        .expect("fields 배열")
        .iter()
        .filter(|f| f["name"] == name)
        .map(|f| f["value"].as_str().unwrap_or("").to_string())
        .collect()
}

/// 색인 없는 키가 반복 이름을 가리키면 몇 개 중 몇 개인지 보고해야 한다.
#[test]
fn plain_key_on_repeated_name_reports_ambiguous() {
    let out = temp_path("plain", "hwp");
    let json = fill(REPEATED, r#"{"피규제집단명":"가상협회 회원사"}"#, &out, &[]);
    assert_eq!(json["filledCount"].as_u64(), Some(1));
    let ambiguous = json["ambiguous"].as_array().expect("ambiguous 배열");
    let hit = ambiguous
        .iter()
        .find(|a| a["name"] == REPEATED_FIELD)
        .unwrap_or_else(|| panic!("반복 이름이 보고되지 않았다: {json}"));
    assert_eq!(hit["matched"].as_u64(), Some(1));
    assert_eq!(hit["total"].as_u64(), Some(REPEAT_COUNT));

    // 실제로도 첫 번째만 바뀐다(종전 동작 유지 — 무회귀).
    let values = field_values(&out, REPEATED_FIELD);
    assert_eq!(values.len() as u64, REPEAT_COUNT);
    assert_eq!(values[0], "가상협회 회원사");
    let _ = std::fs::remove_file(&out);
}

/// `이름[N]` 으로 N번째를 지목할 수 있어야 하고, 범위 밖은 `notFound` 로 드러나야 한다.
#[test]
fn indexed_keys_target_each_occurrence() {
    let out = temp_path("indexed", "hwp");
    let json = fill(
        REPEATED,
        r#"{"피규제집단명[0]":"첫번째집단","피규제집단명[1]":"두번째집단","피규제집단명[13]":"열네번째집단","피규제집단명[14]":"범위초과"}"#,
        &out,
        &[],
    );
    assert_eq!(
        json["filledCount"].as_u64(),
        Some(3),
        "유효 색인 3개, {json}"
    );
    let not_found: Vec<&str> = json["notFound"]
        .as_array()
        .expect("notFound 배열")
        .iter()
        .filter_map(|v| v.as_str())
        .collect();
    assert_eq!(
        not_found,
        vec!["피규제집단명[14]"],
        "범위 밖 색인은 notFound 로 보고돼야 한다"
    );
    assert!(
        json["ambiguous"].as_array().is_some_and(|a| a.is_empty()),
        "색인을 지정했으면 모호하지 않다: {json}"
    );

    let values = field_values(&out, REPEATED_FIELD);
    assert_eq!(values[0], "첫번째집단");
    assert_eq!(values[1], "두번째집단");
    assert_eq!(values[13], "열네번째집단");
    assert_ne!(values[12], "열네번째집단", "지목하지 않은 항목은 불변");
    let _ = std::fs::remove_file(&out);
}

/// 이름이 유일하면 `ambiguous` 는 비어 있어야 한다(무회귀). `--dry-run` 도 같은 보고를 한다.
#[test]
fn unique_names_are_not_ambiguous_and_dry_run_matches() {
    let out = temp_path("unique", "hwp");
    let json = fill(
        UNIQUE,
        r#"{"회사명":"주식회사 가나다"}"#,
        &out,
        &["--dry-run"],
    );
    assert_eq!(json["filledCount"].as_u64(), Some(1));
    assert!(
        json["ambiguous"].as_array().is_some_and(|a| a.is_empty()),
        "유일한 이름은 모호하지 않다: {json}"
    );
    assert!(!out.exists(), "--dry-run 은 출력 파일을 쓰지 않아야 한다");
}
