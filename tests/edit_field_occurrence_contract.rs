//! [#3476] 같은 이름이 반복되는 필드를 지목해 채우는 계약.
//!
//! 사람들이 실제로 제출하는 서식(규제영향분석서·사업계획서·평가표)은 **같은 항목 묶음을
//! 여러 번** 요구한다. `samples/80168_regulatory_analysis.hwp` 는 누름틀 1,070개에
//! 고유 이름이 151개뿐이고 `피규제집단명` 이 14번 나온다 — 규제 대상 집단이 14개이기 때문이다.
//!
//! 이름만으로 채우면 첫 매치만 바뀌는데, 그 사실을 알려주지 않으면 에이전트는
//! **14개 중 1개만 채워진 문서를 완성본으로 판단해 제출**한다. 제출 실패는 문서가
//! 깨져서가 아니라 빈칸이 남아서 일어난다.
#![cfg(not(target_arch = "wasm32"))]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// 규제영향분석서 — 157쪽, 누름틀 1,070개(고유 이름 151개), `피규제집단명` ×14.
const SAMPLE_REG: &str = "samples/80168_regulatory_analysis.hwp";
/// 반복 이름이 실제로 여러 번 나타나는 필드.
const REPEATED: &str = "피규제집단명";

fn sample(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn temp_path(tag: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "rhwp-occ-{tag}-{}-{}.hwp",
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

/// 문서의 `name` 필드 값들을 순서대로 돌려준다.
fn values_of(path: &Path, name: &str) -> Vec<String> {
    let out = run(&["fields", path.to_str().unwrap(), "--json"]);
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("fields --json");
    v["fields"]
        .as_array()
        .expect("fields 배열")
        .iter()
        .filter(|f| f["name"] == name)
        .map(|f| f["value"].as_str().unwrap_or("").to_string())
        .collect()
}

fn skip_if_missing() -> Option<PathBuf> {
    let p = sample(SAMPLE_REG);
    if !p.exists() {
        eprintln!("샘플 없음 — 건너뜀");
        return None;
    }
    Some(p)
}

#[test]
fn occurrence_index_targets_the_nth_field() {
    // 본론: `이름[N]` 으로 N 번째 반복 항목을 지목해 채울 수 있어야 한다.
    let Some(src) = skip_if_missing() else { return };
    let before = values_of(&src, REPEATED);
    assert!(
        before.len() >= 3,
        "이 테스트는 반복 필드가 3개 이상인 문서를 전제한다: {}개",
        before.len()
    );

    let out = temp_path("nth");
    let data =
        format!(r#"{{"{REPEATED}[0]":"가상협회 회원사","{REPEATED}[2]":"가상조합 조합원"}}"#);
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
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("fill-fields JSON");
    assert_eq!(v["filledCount"].as_u64().unwrap(), 2, "{v}");

    let after = values_of(&out, REPEATED);
    assert_eq!(after.len(), before.len(), "필드 개수가 변하면 안 됩니다");
    assert_eq!(after[0], "가상협회 회원사", "0번째가 채워져야 합니다");
    assert_eq!(after[2], "가상조합 조합원", "2번째가 채워져야 합니다");
    assert_eq!(
        after[1], before[1],
        "지목하지 않은 1번째는 그대로여야 합니다"
    );

    let _ = std::fs::remove_file(&out);
}

#[test]
fn ambiguous_name_without_index_is_reported() {
    // 침묵 제거: 이름이 여러 곳에 있으면 몇 개 중 몇 개를 채웠는지 알려야 한다.
    // 이것이 없으면 에이전트가 불완전한 산출물을 완성본으로 판단한다.
    let Some(src) = skip_if_missing() else { return };
    let total = values_of(&src, REPEATED).len();
    assert!(total >= 2, "반복 필드 전제");

    let out = temp_path("ambig");
    let data = format!(r#"{{"{REPEATED}":"가상협회 회원사"}}"#);
    let args = [
        "edit",
        "fill-fields",
        src.to_str().unwrap(),
        "--data",
        &data,
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
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("fill-fields JSON");

    let ambiguous = v["ambiguous"]
        .as_array()
        .unwrap_or_else(|| panic!("ambiguous 배열이 있어야 합니다: {v}"));
    let entry = ambiguous
        .iter()
        .find(|a| a["name"] == REPEATED)
        .unwrap_or_else(|| panic!("{REPEATED} 가 ambiguous 에 보고되어야 합니다: {v}"));
    assert_eq!(entry["matched"].as_u64().unwrap(), 1, "{entry}");
    assert_eq!(
        entry["total"].as_u64().unwrap(),
        total as u64,
        "문서의 실제 개수와 일치해야 합니다: {entry}"
    );
}

#[test]
fn plain_name_still_fills_first_match() {
    // 무회귀 가드: 색인 없는 키는 종전대로 첫 매치를 채운다.
    let Some(src) = skip_if_missing() else { return };
    let before = values_of(&src, REPEATED);
    let out = temp_path("plain");
    let data = format!(r#"{{"{REPEATED}":"가상협회 회원사"}}"#);
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
    let output = run(&args);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        describe(&args, &output)
    );

    let after = values_of(&out, REPEATED);
    assert_eq!(after[0], "가상협회 회원사", "첫 매치가 채워져야 합니다");
    for i in 1..after.len() {
        assert_eq!(after[i], before[i], "{i}번째는 그대로여야 합니다");
    }
    let _ = std::fs::remove_file(&out);
}

#[test]
fn out_of_range_index_is_reported_as_not_found() {
    let Some(src) = skip_if_missing() else { return };
    let total = values_of(&src, REPEATED).len();
    let out = temp_path("oor");
    let key = format!("{REPEATED}[{}]", total + 100);
    let data = format!(r#"{{"{key}":"값"}}"#);
    let args = [
        "edit",
        "fill-fields",
        src.to_str().unwrap(),
        "--data",
        &data,
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
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).expect("fill-fields JSON");
    let missing = v["notFound"].as_array().expect("notFound 배열");
    assert!(
        missing.iter().any(|m| m == key.as_str()),
        "범위를 벗어난 색인은 notFound 로 보고되어야 합니다: {v}"
    );
    assert_eq!(v["filledCount"].as_u64().unwrap(), 0, "{v}");
}
